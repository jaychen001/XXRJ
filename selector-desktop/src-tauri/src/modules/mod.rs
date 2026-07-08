use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

mod common;
pub mod drive;
pub mod intermittent;
#[cfg(test)]
mod tests;
pub mod transmission;

pub fn module_definitions() -> Vec<ModuleDefinition> {
    let mut definitions = Vec::new();
    definitions.extend(transmission::module_definitions());
    definitions.extend(intermittent::module_definitions());
    definitions.extend(drive::module_definitions());
    definitions
}

pub fn calculate(request: &CalculationRequest) -> Option<Result<CalculationResult, FieldError>> {
    transmission::calculate(request)
        .or_else(|| intermittent::calculate(request))
        .or_else(|| drive::calculate(request))
}
