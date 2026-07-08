use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::engine::models::{CalculationRequest, FieldInput};

const DRIVE_CASES_JSON: &str = include_str!("../modules/fixtures/drive_cases.json");
const MECHANICAL_TRANSMISSION_CASES_JSON: &str =
    include_str!("../modules/fixtures/mechanical_transmission_cases.json");
const PNEUMATIC_SUPPORT_CASES_JSON: &str =
    include_str!("../modules/fixtures/pneumatic_support_cases.json");
const RULE_MODULE_CASES_JSON: &str = include_str!("../modules/fixtures/rule_modules_cases.json");

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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QaRegressionReport {
    pub status: String,
    pub total_cases: usize,
    pub passed_cases: usize,
    pub failed_cases: usize,
    pub groups: Vec<QaRegressionGroup>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QaRegressionGroup {
    pub label: String,
    pub total_cases: usize,
    pub passed_cases: usize,
    pub failed_cases: usize,
    pub cases: Vec<QaRegressionCaseResult>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QaRegressionCaseResult {
    pub name: String,
    pub module_id: String,
    pub status: String,
    pub detail: String,
}

#[tauri::command]
pub fn run_qa_regression(_app_handle: AppHandle) -> Result<QaRegressionReport, String> {
    build_regression_report()
}

fn build_regression_report() -> Result<QaRegressionReport, String> {
    let groups = vec![
        run_group("Phase 4 驱动与线性传动", DRIVE_CASES_JSON, 1)?,
        run_group(
            "Phase 5 机械传动与间歇机构",
            MECHANICAL_TRANSMISSION_CASES_JSON,
            1,
        )?,
        run_group("Phase 6 气动与支撑件", PNEUMATIC_SUPPORT_CASES_JSON, 1)?,
        run_group("Phase 7 规则选型模块", RULE_MODULE_CASES_JSON, 3)?,
    ];
    let total_cases = groups.iter().map(|group| group.total_cases).sum();
    let passed_cases = groups.iter().map(|group| group.passed_cases).sum();
    let failed_cases = total_cases - passed_cases;
    Ok(QaRegressionReport {
        status: if failed_cases == 0 { "pass" } else { "fail" }.to_string(),
        total_cases,
        passed_cases,
        failed_cases,
        groups,
    })
}

fn run_group(
    label: &str,
    fixture_json: &str,
    min_rule_count: usize,
) -> Result<QaRegressionGroup, String> {
    let fixtures = serde_json::from_str::<Vec<FixtureCase>>(fixture_json)
        .map_err(|error| format!("{label} fixture 解析失败: {error}"))?;
    let cases = fixtures
        .into_iter()
        .map(|fixture| run_case(fixture, min_rule_count))
        .collect::<Vec<_>>();
    let passed_cases = cases.iter().filter(|case| case.status == "pass").count();
    let total_cases = cases.len();
    Ok(QaRegressionGroup {
        label: label.to_string(),
        total_cases,
        passed_cases,
        failed_cases: total_cases - passed_cases,
        cases,
    })
}

fn run_case(fixture: FixtureCase, min_rule_count: usize) -> QaRegressionCaseResult {
    let FixtureCase {
        name,
        module_id,
        safety_factor,
        safety_factor_confirmed,
        fields,
        expected_step_labels,
        expected_requirement_ids,
    } = fixture;
    let request = CalculationRequest {
        module_id: module_id.clone(),
        fields,
        safety_factor: Some(safety_factor),
        safety_factor_confirmed,
    };
    let Some(result) = crate::modules::calculate(&request) else {
        return failed_case(name, module_id, "模块未注册");
    };
    let result = match result {
        Ok(result) => result,
        Err(error) => return failed_case(name, module_id, error.message),
    };

    let mut failures = Vec::new();
    for expected in expected_step_labels {
        if !result.steps.iter().any(|step| step.label == expected) {
            failures.push(format!("缺少过程步骤: {expected}"));
        }
    }
    for expected in expected_requirement_ids {
        if !result
            .requirements
            .iter()
            .any(|parameter| parameter.id == expected)
        {
            failures.push(format!("缺少需求参数: {expected}"));
        }
    }
    if result.source_pages.is_empty() {
        failures.push("缺少来源页码".to_string());
    }
    if result.rules.len() < min_rule_count {
        failures.push(format!("规则判断少于 {min_rule_count} 条"));
    }
    let recorded_safety_factor = result
        .input_snapshot
        .get("safetyFactor")
        .and_then(|value| value.as_f64());
    if !matches!(recorded_safety_factor, Some(value) if (value - safety_factor).abs() < f64::EPSILON)
    {
        failures.push("安全系数未写入输入快照".to_string());
    }

    if failures.is_empty() {
        QaRegressionCaseResult {
            name,
            module_id,
            status: "pass".to_string(),
            detail: format!(
                "步骤 {} 项，规则 {} 项，来源 {} 项",
                result.steps.len(),
                result.rules.len(),
                result.source_pages.len()
            ),
        }
    } else {
        failed_case(name, module_id, failures.join("；"))
    }
}

fn failed_case(
    name: String,
    module_id: String,
    detail: impl Into<String>,
) -> QaRegressionCaseResult {
    QaRegressionCaseResult {
        name,
        module_id,
        status: "fail".to_string(),
        detail: detail.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regression_runner_passes_all_fixture_groups() {
        let report = build_regression_report().expect("regression report");

        assert_eq!(report.status, "pass");
        assert_eq!(report.failed_cases, 0);
        assert!(report.total_cases >= 4);
        assert_eq!(report.groups.len(), 4);
    }
}
