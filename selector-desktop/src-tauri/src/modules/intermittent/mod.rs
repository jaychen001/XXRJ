use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

pub mod brake_clutch;
pub mod cam_indexer;

pub fn module_definitions() -> Vec<ModuleDefinition> {
    vec![cam_indexer::definition(), brake_clutch::definition()]
}

pub fn calculate(request: &CalculationRequest) -> Option<Result<CalculationResult, FieldError>> {
    match request.module_id.as_str() {
        cam_indexer::MODULE_ID => Some(cam_indexer::calculate(request)),
        brake_clutch::MODULE_ID => Some(brake_clutch::calculate(request)),
        _ => None,
    }
}
