use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleDefinition {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub source_chapter: String,
    pub source_page: String,
    pub fields: Vec<ModuleField>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleField {
    pub id: String,
    pub label: String,
    pub unit: String,
    pub unit_options: Vec<String>,
    pub required: bool,
    pub min: Option<f64>,
    pub default_value: Option<f64>,
    pub helper: String,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculationRequest {
    pub module_id: String,
    pub fields: Vec<FieldInput>,
    pub safety_factor: Option<f64>,
    pub safety_factor_confirmed: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldInput {
    pub id: String,
    pub value: f64,
    pub unit: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculationResult {
    pub module_id: String,
    pub module_name: String,
    pub formula_version: String,
    pub summary: String,
    pub conclusion: String,
    pub steps: Vec<FormulaStep>,
    #[serde(default)]
    pub rules: Vec<RuleDecision>,
    pub risks: Vec<RiskItem>,
    pub requirements: Vec<RequirementParameter>,
    pub source_pages: Vec<String>,
    pub input_snapshot: Value,
    pub defaults_snapshot: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FormulaStep {
    pub label: String,
    pub formula: String,
    pub substitution: String,
    pub result: String,
    pub unit: String,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequirementParameter {
    pub id: String,
    pub label: String,
    pub value: f64,
    pub unit: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleDecision {
    pub id: String,
    pub label: String,
    pub recommendation: String,
    pub basis: String,
    pub risk: String,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskItem {
    pub level: String,
    pub message: String,
    pub field_id: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldError {
    pub field_id: String,
    pub message: String,
}
