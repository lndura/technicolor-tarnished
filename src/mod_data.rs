use std::{
    path::{Path, PathBuf},
    string::FromUtf16Error,
    sync::{OnceLock, RwLock},
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, unbounded};
use notify::{Config, Event, PollWatcher, Watcher};
use tracing::{error, info};
use windows::Win32::{
    Foundation::{GetLastError, HMODULE, WIN32_ERROR},
    System::LibraryLoader::GetModuleFileNameW,
};

use crate::{
    profile::{Profile, ScriptOverride},
    scripting::RuneInterface,
};

const DEFAULT_INTERVAL: u64 = 2500;
const PROFILE_PATH: &str = "phantom_color_profile.toml";
const N_SIZE: usize = 256;

#[derive(Debug)]
pub enum ModDataError {
    WriteLockError(String),
    AlreadyInitialized,
    NotInitialized,
    WinApiError(WIN32_ERROR),
    FromUtf16Error(FromUtf16Error),
    FileReadError(std::io::Error),
    TomlParseError(toml::de::Error),
    NotifyWatcherError(notify::Error),
    NoParentDirectory,
    RuneInterfaceError(crate::scripting::RuneInterfaceError),
}

impl core::fmt::Display for ModDataError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::WriteLockError(err) => write!(f, "RwLockWriteGuard error occurred: {:#?}", err),
            Self::AlreadyInitialized => write!(f, "MOD_DATA has already been initialized."),
            Self::NotInitialized => write!(f, "MOD_DATA has not been initialized."),
            Self::WinApiError(err) => write!(f, "WinAPI error occurred: {:#?}", err),
            Self::FromUtf16Error(err) => write!(f, "UTF-16 error occurred: {:#?}", err),
            Self::FileReadError(err) => write!(f, "File read error occurred: {:#?}", err),
            Self::TomlParseError(err) => write!(f, "Toml parse error occurred: {:#?}", err.message()),
            Self::NotifyWatcherError(err) => {
                write!(f, "Failed to watch profile path for changes: {:#?}", err)
            }
            Self::NoParentDirectory => write!(f, "No parent directory found for the module path."),
            Self::RuneInterfaceError(err) => write!(f, "{}", err),
        }
    }
}

pub static MOD_DATA: OnceLock<RwLock<ModData>> = OnceLock::new();

pub struct ModData {
    pub profile_data: Profile,
    pub directory_path: PathBuf,
    pub should_patch: bool,
    pub watcher: PollWatcher,
    pub receiver: Receiver<notify::Result<Event>>,
    pub rune_interface: Option<RuneInterface>,
    pub interval: Duration,
    pub duration: Instant,
}

pub fn init_mod_data(hmodule: HMODULE) -> Result<(), ModDataError> {
    if let Some(_) = MOD_DATA.get() {
        return Err(ModDataError::AlreadyInitialized);
    }

    let mut lpfilename = [0u16; N_SIZE];
    let str_len = unsafe { GetModuleFileNameW(Some(hmodule), &mut lpfilename) };
    if str_len == 0 {
        let err = unsafe { GetLastError() };
        return Err(ModDataError::WinApiError(err));
    }

    let file_bytes = &mut lpfilename[..str_len as usize];
    let file_str =
        String::from_utf16(file_bytes).map_err(|err| ModDataError::FromUtf16Error(err))?;

    let path_buffer = PathBuf::from(file_str);

    let directory = path_buffer
        .parent()
        .ok_or(ModDataError::NoParentDirectory)?
        .to_path_buf();

    let profile_path = directory.join(PROFILE_PATH);

    info!("Attempting to load config from {:#?}", profile_path);

    let profile_file = std::fs::read_to_string(profile_path.clone())
        .map_err(|err| ModDataError::FileReadError(err))?;

    let mut profile_data = toml::from_str::<Profile>(&profile_file)
        .map_err(|err| ModDataError::TomlParseError(err))?;

    let millis = profile_data.interval.unwrap_or(DEFAULT_INTERVAL);
    info!(
        "Adjusting Polling & ChrSet iteration interval to {} milliseconds",
        millis
    );
    let interval = Duration::from_millis(millis);
    let duration = Instant::now();

    let (sender, receiver) = unbounded();

    let mut watcher = PollWatcher::new(
        move |event| {
            let _ = sender.send(event);
        },
        notify::Config::default().with_poll_interval(interval),
    )
    .map_err(|err| ModDataError::NotifyWatcherError(err))?;

    let path = profile_path.as_path();
    watcher
        .watch(path, notify::RecursiveMode::NonRecursive)
        .map_err(|err| ModDataError::NotifyWatcherError(err))?;

    profile_data
        .script
        .iter_mut()
        .filter(|script_override| {
            script_override
                .script_path
                .extension()
                .is_some_and(|ext| ext == "rn")
        })
        .try_for_each(|script_override| {
            let script_path = directory.join(&script_override.script_path);
            script_override.script_path = script_path;
            watcher
                .watch(
                    script_override.script_path.as_path(),
                    notify::RecursiveMode::NonRecursive,
                )
                .map_err(|err| ModDataError::NotifyWatcherError(err))
        })?;

    let rune_interface = RuneInterface::compile_scripts(&profile_data.script)
        .map_err(|err| error!("Failed to initialize Rune Interface: {:#?}", err))
        .ok();

    MOD_DATA
        .set(RwLock::new(ModData {
            profile_data: profile_data,
            directory_path: directory,
            should_patch: true,
            watcher: watcher,
            receiver: receiver,
            rune_interface: rune_interface,
            interval: interval,
            duration: duration,
        }))
        .map_err(|_| ModDataError::AlreadyInitialized)?;

    Ok(())
}

pub fn get_mod_data() -> Result<std::sync::RwLockWriteGuard<'static, ModData>, ModDataError> {
    MOD_DATA
        .get()
        .ok_or(ModDataError::NotInitialized)
        .and_then(|data_lock| {
            data_lock
                .write()
                .map_err(|err| ModDataError::WriteLockError(err.to_string()))
        })
}

impl ModData {
    fn update_scripts_watcher<F>(
        &mut self,
        slice: &mut [ScriptOverride],
        mut callback: F,
    ) -> Result<(), ModDataError>
    where
        F: FnMut(&mut PollWatcher, &Path) -> Result<(), notify::Error>,
    {
        slice
            .iter_mut()
            .filter(|script_override| {
                script_override
                    .script_path
                    .extension()
                    .is_some_and(|ext| ext == "rn")
            })
            .try_for_each(|script_override| {
                let script_path = self.directory_path.join(&script_override.script_path);
                script_override.script_path = script_path;
                callback(&mut self.watcher, script_override.script_path.as_path())
                    .map_err(|err| ModDataError::NotifyWatcherError(err))
            })
    }
    fn watch_scripts(&mut self, slice: &mut [ScriptOverride]) -> Result<(), ModDataError> {
        self.update_scripts_watcher(slice, |watcher, path| {
            watcher.watch(path, notify::RecursiveMode::NonRecursive)
        })
    }
    fn unwatch_scripts(&mut self, slice: &mut [ScriptOverride]) -> Result<(), ModDataError> {
        self.update_scripts_watcher(slice, |watcher: &mut PollWatcher, path: &Path| {
            watcher.unwatch(path)
        })
    }
    pub fn update_profile(&mut self) -> Result<(), ModDataError> {
        let profile_path = self.directory_path.join(PROFILE_PATH);
        let profile_file = std::fs::read_to_string(profile_path)
            .map_err(|err| ModDataError::FileReadError(err))?;

        let mut profile_data = toml::from_str::<Profile>(&profile_file)
            .map_err(|err| ModDataError::TomlParseError(err))?;

        let mut scripts_slice = std::mem::take(&mut self.profile_data.script);
        self.unwatch_scripts(&mut scripts_slice)?;

        self.watch_scripts(&mut profile_data.script)?;

        let rune_interface = RuneInterface::compile_scripts(&profile_data.script)
            .map_err(|err| ModDataError::RuneInterfaceError(err))?;

        let millis = profile_data.interval.unwrap_or(DEFAULT_INTERVAL);
        info!(
            "Adjusting Polling & ChrSet iteration interval to {} milliseconds",
            millis
        );
        let interval = Duration::from_millis(millis);
        let config = Config::default().with_poll_interval(interval);
        if let Err(err) = self.watcher.configure(config) {
            error!("Watcher config error occurred: {:#?}", err);
        };
        self.interval = interval;
        self.rune_interface = Some(rune_interface);
        self.profile_data = profile_data;
        self.should_patch = true;

        info!("Profile data updated from file changes");

        Ok(())
    }
}
