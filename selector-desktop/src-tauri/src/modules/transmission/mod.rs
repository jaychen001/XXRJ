use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

pub mod ball_screw;
pub mod linear_module;
pub mod reducer;
pub mod timing_belt;

pub fn module_definitions() -> Vec<ModuleDefinition> {
    vec![
        timing_belt::definition(),
        ball_screw::definition(),
        reducer::definition(),
        linear_module::definition(),
    ]
}

pub fn calculate(request: &CalculationRequest) -> Option<Result<CalculationResult, FieldError>> {
    match request.module_id.as_str() {
        timing_belt::MODULE_ID => Some(timing_belt::calculate(request)),
        ball_screw::MODULE_ID => Some(ball_screw::calculate(request)),
        reducer::MODULE_ID => Some(reducer::calculate(request)),
        linear_module::MODULE_ID => Some(linear_module::calculate(request)),
        _ => None,
    }
}
