use std::sync::Arc;

use eldenring::cs::SoloParamRepository;
use rune::{
    Context, Diagnostics, FromValue, Module, Source, Sources, Unit, Vm, diagnostics::Diagnostic,
    runtime::RuntimeContext,
};

use tracing::{error, info};

use crate::profile::{PhantomParam, ScriptOverride};

#[derive(Debug)]
pub enum RuneInterfaceError {
    ContextError(rune::ContextError),
    BuildError(rune::BuildError),
    RuntimeError(rune::alloc::Error),
    InvalidParamRowError(u32),
    ValueReturnError(rune::runtime::VmError),
    ValueConversionError(rune::runtime::RuntimeError),
}

impl core::fmt::Display for RuneInterfaceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ContextError(err) => write!(f, "Rune context error occurred: {:#?}", err),
            Self::BuildError(err) => write!(f, "Rune build error occurred: {:#?}", err),
            Self::RuntimeError(err) => write!(f, "Rune runtime error occurred: {:#?}", err),
            Self::InvalidParamRowError(err) => write!(
                f,
                "PHANTOM_PARAM_ST with id {:#?} not found in SoloParamRepository",
                err
            ),
            Self::ValueReturnError(err) => {
                write!(f, "Return value of script contained an error: {:#?}", err)
            }
            Self::ValueConversionError(err) => write!(f, "Value conversion error: {:#?}", err),
        }
    }
}

pub struct RuneInterface {
    unit: Arc<Unit>,
    runtime: Arc<RuntimeContext>,
}

impl RuneInterface {
    pub fn create_vm(&self) -> Vm {
        let runtime = self.runtime.clone();
        let unit = self.unit.clone();
        Vm::new(runtime, unit)
    }
    pub fn run_scripts(
        &self,
        solo_param: &mut SoloParamRepository,
        scripts: &[ScriptOverride],
    ) -> Result<(), RuneInterfaceError> {
        let mut vm = self.create_vm();
        for (index, script_override) in scripts.iter().enumerate() {
            let script = format!("script_{}", index);
            for param_id in script_override.param_id_list.iter() {
                let Some(row) = solo_param.get_mut::<eldenring::cs::PhantomParam>(*param_id) else {
                    return Err(RuneInterfaceError::InvalidParamRowError(*param_id));
                };

                let phantom_param = PhantomParam::from_row(row);
                let value = vm
                    .call([script.as_str(), "main"], (phantom_param,))
                    .map_err(|err| RuneInterfaceError::ValueReturnError(err))?;

                let param = PhantomParam::from_value(value)
                    .map_err(|err| RuneInterfaceError::ValueConversionError(err))?;

                param.patch(row);
            }
        }

        Ok(())
    }
    pub fn compile_scripts(scripts: &[ScriptOverride]) -> Result<Self, RuneInterfaceError> {
        let mut sources = Sources::new();

        for (index, script_override) in scripts.iter().enumerate() {
            info!("Attempting to compile: {:#?}", &script_override.script_path);
            let file = match std::fs::read_to_string(&script_override.script_path) {
                Ok(file) => file,
                Err(err) => {
                    error!("Failed to read file: {:#?}", err);
                    continue;
                }
            };

            let name = format!("script_{}", index);
            let source = format!("pub mod script_{} {{\n{}\n}}", index, file);
            let rune_source = match Source::new(&name, source) {
                Ok(src) => src,
                Err(err) => {
                    error!("Failed to create Source: {:#?}", err);
                    continue;
                }
            };

            match sources.insert(rune_source) {
                Ok(_) => info!("Successfully compiled script!"),
                Err(err) => error!("Failed to insert Source: {:#?}", err),
            }
        }

        let mut module = Module::new();
        module
            .ty::<PhantomParam>()
            .map_err(|err| RuneInterfaceError::ContextError(err))?;

        let mut context =
            Context::with_default_modules().map_err(|err| RuneInterfaceError::ContextError(err))?;

        context
            .install(module)
            .map_err(|err| RuneInterfaceError::ContextError(err))?;

        let mut diagnostics = Diagnostics::new();

        let unit = rune::prepare(&mut sources)
            .with_context(&context)
            .with_diagnostics(&mut diagnostics)
            .build()
            .map_err(|err| {
                for diagnostic in diagnostics.diagnostics() {
                    match diagnostic {
                        Diagnostic::Fatal(err) => {
                            error!("Fatal error occurred:\n{:#?}", err)
                        }
                        Diagnostic::Warning(err) => info!("Warning occurred:\n{:#?}", err),
                        Diagnostic::RuntimeWarning(err) => {
                            info!("Runtime warning occurred:\n{:#?}", err)
                        }
                        _ => {}
                    }
                }
                RuneInterfaceError::BuildError(err)
            })?;

        let runtime = context
            .runtime()
            .map_err(|err| RuneInterfaceError::RuntimeError(err))?;

        let rune_interface = Self {
            unit: Arc::new(unit),
            runtime: Arc::new(runtime),
        };

        info!("Rune Interface successfully set up");

        Ok(rune_interface)
    }
}
