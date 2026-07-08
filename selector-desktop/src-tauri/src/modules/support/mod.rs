use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

pub mod coupling;
pub mod linear_bearing;
pub mod linear_guide;
pub mod rolling_bearing;

pub fn module_definitions() -> Vec<ModuleDefinition> {
    vec![
        linear_guide::definition(),
        linear_bearing::definition(),
        rolling_bearing::definition(),
        coupling::definition(),
    ]
}

pub fn calculate(request: &CalculationRequest) -> Option<Result<CalculationResult, FieldError>> {
    match request.module_id.as_str() {
        linear_guide::MODULE_ID => Some(linear_guide::calculate(request)),
        linear_bearing::MODULE_ID => Some(linear_bearing::calculate(request)),
        rolling_bearing::MODULE_ID => Some(rolling_bearing::calculate(request)),
        coupling::MODULE_ID => Some(coupling::calculate(request)),
        _ => None,
    }
}
