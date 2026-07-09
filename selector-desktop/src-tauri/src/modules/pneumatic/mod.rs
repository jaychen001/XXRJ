use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

pub mod cylinder;
pub mod flow_control;
pub mod gripper;
pub mod rotary;
pub mod slide_table;
pub mod vacuum;

pub fn module_definitions() -> Vec<ModuleDefinition> {
    vec![
        cylinder::definition(),
        gripper::definition(),
        slide_table::definition(),
        rotary::definition(),
        vacuum::definition(),
        flow_control::definition(),
    ]
}

pub fn calculate(request: &CalculationRequest) -> Option<Result<CalculationResult, FieldError>> {
    match request.module_id.as_str() {
        cylinder::MODULE_ID => Some(cylinder::calculate(request)),
        gripper::MODULE_ID => Some(gripper::calculate(request)),
        slide_table::MODULE_ID => Some(slide_table::calculate(request)),
        rotary::MODULE_ID => Some(rotary::calculate(request)),
        vacuum::MODULE_ID => Some(vacuum::calculate(request)),
        flow_control::MODULE_ID => Some(flow_control::calculate(request)),
        _ => None,
    }
}
