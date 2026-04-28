use std::time::{Duration, Instant};
use tracing::{error, info};
use windows::Win32::Foundation::HMODULE;

use eldenring::{
    cs::{CSTaskGroupIndex, CSTaskImp, SoloParamRepository, WorldChrMan},
    fd4::FD4TaskData,
    util::system::wait_for_system_init,
};
use fromsoftware_shared::{FromStatic, Program, SharedTaskImpExt};

mod console;
mod mod_data;
mod profile;
mod scripting;

use console::tracing_init;
use mod_data::{get_mod_data, init_mod_data};

#[unsafe(no_mangle)]
unsafe extern "C" fn DllMain(hmodule: HMODULE, reason: u32) -> bool {
    if reason == 1 {
        if let Err(err) = tracing_init() {
            println!("Technicolor Tarnished Error: {}", err);
            return true;
        }

        if hmodule.is_invalid() {
            error!("Invalid HMODULE provided to DllMain: {:#?}", hmodule);
            return true;
        }

        if let Err(err) = init_mod_data(hmodule) {
            error!("Failed to initialize ModData: {:#?}", err);
            return true;
        }

        info!("Successfully initialized ModData");

        std::thread::spawn(|| {
            wait_for_system_init(&Program::current(), Duration::MAX)
                .expect("Could not wait for system init");

            let Ok(cs_task) = unsafe { CSTaskImp::instance() }
                .map_err(|err| error!("Failed to obtain CSTaskImp: {:#?}", err))
            else {
                return;
            };

            cs_task.run_recurring(
                |_task: &FD4TaskData| {
                    let Ok(mut mod_data) = get_mod_data()
                        .map_err(|err| error!("Failed to acquire ModData write guard: {:#?}", err))
                    else {
                        return;
                    };

                    if mod_data.receiver.try_iter().any(|r_event| match r_event {
                        Ok(event) => {
                            info!(
                                "File change event recieved for the following file(s):\n{:#?}",
                                event.paths
                            );
                            true
                        }
                        Err(err) => {
                            error!("Error receiving file change event: {:#?}", err);
                            false
                        }
                    }) {
                        if let Err(err) = mod_data.update_profile() {
                            error!("Failed to update ModData from file changes: {:#?}", err);
                        }
                    }

                    if unsafe { WorldChrMan::instance() }
                        .ok()
                        .and_then(|w| w.main_player.as_mut())
                        .is_none()
                    {
                        return;
                    };

                    if mod_data.should_patch {
                        match mod_data.profile_data.patch() {
                            Ok(_) => mod_data.should_patch = false,
                            Err(msg) => error!("Failed to apply profile patch: {:#?}", msg),
                        }
                    }

                    if mod_data.duration.elapsed() >= mod_data.interval {
                        let profile_data = &mod_data.profile_data;
                        if let Ok(w_char_man) = unsafe { WorldChrMan::instance() } {
                            if let Some(override_data) = &profile_data.summon {
                                for chr_ins in w_char_man.summon_buddy_chr_set.characters() {
                                    let param_id = override_data.param_id as i32;
                                    match param_id {
                                        -1 | 0 => chr_ins.phantom_param_override = -1,
                                        _ => chr_ins.phantom_param_override = param_id,
                                    }
                                }
                            }

                            if !profile_data.chr_id.is_empty() {
                                for index in 0..w_char_man.chr_set_holder_count as usize {
                                    let chr_set_holder = &mut w_char_man.chr_set_holders[index];
                                    let chr_set = unsafe { chr_set_holder.chr_set.as_mut() };
                                    for chr_ins in chr_set.characters() {
                                        if let Some(override_data) =
                                            profile_data.chr_id.iter().find(|override_data| {
                                                override_data
                                                    .chr_id_list
                                                    .iter()
                                                    .any(|id| *id == chr_ins.character_id)
                                            })
                                        {
                                            let param_id = override_data.param_id as i32;
                                            match param_id {
                                                -1 | 0 => chr_ins.phantom_param_override = -1,
                                                _ => chr_ins.phantom_param_override = param_id,
                                            }
                                        }
                                    }
                                }
                            }

                            if let Some(override_data) = &profile_data.player
                                && let Some(main_player) = w_char_man.main_player.as_mut()
                            {
                                let param_id = override_data.param_id as i32;
                                match param_id {
                                    -1 | 0 => main_player.phantom_param_override = -1,
                                    _ => main_player.phantom_param_override = param_id,
                                }
                                let ride_module = main_player.chr_ins.modules.ride.as_mut();

                                if override_data.override_ridden
                                    && let Some(mut last_mounted_ptr) = ride_module.last_mounted
                                {
                                    let last_mounted = unsafe { last_mounted_ptr.as_mut() };
                                    match param_id {
                                        -1 | 0 => last_mounted.phantom_param_override = -1,
                                        _ => last_mounted.phantom_param_override = param_id,
                                    }
                                }
                            }
                        }

                        mod_data.duration = Instant::now();
                    }

                    if let Some(interface) = &mod_data.rune_interface
                        && let Ok(solo_param) = unsafe { SoloParamRepository::instance() }
                        && let Err(err) =
                            interface.run_scripts(solo_param, &mod_data.profile_data.script)
                    {
                        error!("Rune interface error occurred: {:#?}", err);
                        mod_data.rune_interface = None;
                    }
                },
                CSTaskGroupIndex::FrameBegin,
            );
        });
    }

    true
}
