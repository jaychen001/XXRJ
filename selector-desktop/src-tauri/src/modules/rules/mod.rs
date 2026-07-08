use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

pub mod cable_chain;
pub mod hardware;
pub mod heat_surface;
pub mod machining;
pub mod material;
pub mod robot;
pub mod sensor;

pub fn module_definitions() -> Vec<ModuleDefinition> {
    vec![
        robot::definition(),
        cable_chain::definition(),
        sensor::definition(),
        material::definition(),
        machining::definition(),
        heat_surface::definition(),
        hardware::definition(),
    ]
}

pub fn calculate(request: &CalculationRequest) -> Option<Result<CalculationResult, FieldError>> {
    match request.module_id.as_str() {
        robot::MODULE_ID => Some(robot::calculate(request)),
        cable_chain::MODULE_ID => Some(cable_chain::calculate(request)),
        sensor::MODULE_ID => Some(sensor::calculate(request)),
        material::MODULE_ID => Some(material::calculate(request)),
        machining::MODULE_ID => Some(machining::calculate(request)),
        heat_surface::MODULE_ID => Some(heat_surface::calculate(request)),
        hardware::MODULE_ID => Some(hardware::calculate(request)),
        _ => None,
    }
}
