use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

pub mod motor;
pub mod servo_stepper;

pub fn module_definitions() -> Vec<ModuleDefinition> {
    vec![motor::definition(), servo_stepper::definition()]
}

pub fn calculate(request: &CalculationRequest) -> Option<Result<CalculationResult, FieldError>> {
    match request.module_id.as_str() {
        motor::MODULE_ID => Some(motor::calculate(request)),
        servo_stepper::MODULE_ID => Some(servo_stepper::calculate(request)),
        _ => None,
    }
}
