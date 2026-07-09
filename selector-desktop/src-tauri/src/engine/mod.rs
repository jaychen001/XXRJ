mod formula;
pub mod models;
mod modules;
pub(crate) mod safety_factor;
pub(crate) mod units;

use tauri::AppHandle;

use self::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

pub fn calculate_request(request: &CalculationRequest) -> Result<CalculationResult, FieldError> {
    formula::run_calculation(request)
}

pub fn get_module_definition(module_id: &str) -> Option<ModuleDefinition> {
    modules::get_module_definition(module_id)
}

#[tauri::command]
pub fn list_calculation_modules(_app_handle: AppHandle) -> Result<Vec<ModuleDefinition>, String> {
    Ok(modules::module_definitions())
}

#[tauri::command]
pub fn run_calculation(
    _app_handle: AppHandle,
    request: CalculationRequest,
) -> Result<CalculationResult, FieldError> {
    calculate_request(&request)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timing_belt_requires_confirmed_safety_factor() {
        let mut request = timing_belt_request();
        request.safety_factor_confirmed = false;
        let error = formula::run_calculation(&request).expect_err("safety factor should block");
        assert_eq!(error.field_id, "safetyFactor");
    }

    #[test]
    fn timing_belt_returns_process_steps_and_requirements() {
        let result = formula::run_calculation(&timing_belt_request()).expect("calculation");
        assert!(result.steps.iter().any(|step| step.label == "摩擦力"));
        assert!(result.steps.iter().any(|step| step.label == "输出扭矩"));
        assert!(result
            .requirements
            .iter()
            .any(|parameter| parameter.id == "requiredSpeed"));
        assert!(result.rules.iter().any(|rule| rule.label == "速度区间"));
    }

    #[test]
    fn timing_belt_converts_selected_units_before_calculation() {
        let mut request = timing_belt_request();
        set_input(&mut request, "targetSpeed", 0.5, "m/s");
        set_input(&mut request, "accelerationTime", 0.005, "min");
        set_input(&mut request, "toothPitch", 0.005, "m");

        let result = formula::run_calculation(&request).expect("calculation");

        assert!(result.summary.contains("300.000 rpm"));
    }

    fn timing_belt_request() -> CalculationRequest {
        CalculationRequest {
            module_id: modules::TIMING_BELT_MODULE_ID.to_string(),
            safety_factor: Some(1.5),
            safety_factor_confirmed: true,
            fields: vec![
                input("loadMass", 5.0, "kg"),
                input("frictionCoefficient", 0.1, "ratio"),
                input("targetSpeed", 500.0, "mm/s"),
                input("accelerationTime", 0.3, "s"),
                input("externalForce", 0.0, "N"),
                input("verticalLoadFactor", 0.0, "ratio"),
                input("pulleyTeeth", 20.0, "teeth"),
                input("toothPitch", 5.0, "mm"),
                input("efficiency", 0.9, "ratio"),
            ],
        }
    }

    fn input(id: &str, value: f64, unit: &str) -> models::FieldInput {
        models::FieldInput {
            id: id.to_string(),
            value,
            unit: unit.to_string(),
        }
    }

    fn set_input(request: &mut CalculationRequest, id: &str, value: f64, unit: &str) {
        let input = request
            .fields
            .iter_mut()
            .find(|field| field.id == id)
            .expect("field exists");
        input.value = value;
        input.unit = unit.to_string();
    }
}
