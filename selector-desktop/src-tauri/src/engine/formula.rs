use super::models::{CalculationRequest, CalculationResult, FieldError};

pub fn run_calculation(request: &CalculationRequest) -> Result<CalculationResult, FieldError> {
    crate::modules::calculate(request).unwrap_or_else(|| {
        Err(FieldError {
            field_id: "moduleId".to_string(),
            message: "该模块尚未实现计算公式".to_string(),
        })
    })
}
