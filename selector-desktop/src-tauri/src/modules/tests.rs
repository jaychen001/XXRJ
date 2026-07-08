use serde::Deserialize;

use crate::engine::models::{CalculationRequest, FieldInput};

const DRIVE_CASES_JSON: &str = include_str!("fixtures/drive_cases.json");

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FixtureCase {
    name: String,
    module_id: String,
    safety_factor: f64,
    safety_factor_confirmed: bool,
    fields: Vec<FieldInput>,
    expected_step_labels: Vec<String>,
    expected_requirement_ids: Vec<String>,
}

#[test]
fn phase4_drive_fixtures_return_required_process_and_sources() {
    let fixtures: Vec<FixtureCase> = serde_json::from_str(DRIVE_CASES_JSON).expect("fixture json");

    for fixture in fixtures {
        let request = CalculationRequest {
            module_id: fixture.module_id,
            fields: fixture.fields,
            safety_factor: Some(fixture.safety_factor),
            safety_factor_confirmed: fixture.safety_factor_confirmed,
        };
        let result = super::calculate(&request)
            .expect("phase4 module implemented")
            .unwrap_or_else(|error| panic!("{} failed: {}", fixture.name, error.message));

        for expected in fixture.expected_step_labels {
            assert!(
                result.steps.iter().any(|step| step.label == expected),
                "{} missing step {}",
                fixture.name,
                expected
            );
        }
        for expected in fixture.expected_requirement_ids {
            assert!(
                result
                    .requirements
                    .iter()
                    .any(|parameter| parameter.id == expected),
                "{} missing requirement {}",
                fixture.name,
                expected
            );
        }
        assert!(
            !result.source_pages.is_empty(),
            "{} missing source",
            fixture.name
        );
        assert_eq!(
            result.input_snapshot["safetyFactor"], fixture.safety_factor,
            "{} safety factor not recorded",
            fixture.name
        );
    }
}
