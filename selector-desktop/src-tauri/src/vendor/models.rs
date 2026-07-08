use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VendorImportPreviewRequest {
    pub source_file: String,
    pub source_format: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VendorImportPreview {
    pub job_id: String,
    pub source_file: String,
    pub source_format: String,
    pub confidence: f64,
    pub rows: Vec<VendorPreviewRow>,
    pub failed_rows: Vec<ImportFailureRow>,
    pub suggested_mappings: Vec<FieldMapping>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportFailureRow {
    pub row_index: usize,
    pub reason: String,
    pub raw_text: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VendorPreviewRow {
    pub row_index: usize,
    pub model_name: String,
    pub brand: String,
    pub series: String,
    pub source_page: Option<String>,
    pub confidence: f64,
    pub raw_text: String,
    pub parameters: Vec<VendorParameter>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VendorParameter {
    pub field: String,
    pub label: String,
    pub value: f64,
    pub unit: String,
    pub source_field: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldMapping {
    pub source_field: String,
    pub target_field: String,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmVendorImportRequest {
    pub library_name: String,
    pub component_type: String,
    pub version_name: String,
    pub confirmed: bool,
    pub preview: VendorImportPreview,
    pub mappings: Vec<FieldMapping>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VendorImportSummary {
    pub library: VendorLibraryRecord,
    pub imported_models: usize,
    pub failed_rows: usize,
    pub job_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VendorLibraryRecord {
    pub id: String,
    pub name: String,
    pub component_type: String,
    pub source_file: String,
    pub source_format: String,
    pub version_name: String,
    pub imported_at: String,
    pub enabled: bool,
    pub model_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VendorModelRecord {
    pub id: String,
    pub library_id: String,
    pub library_name: String,
    pub component_type: String,
    pub model_name: String,
    pub brand: String,
    pub series: String,
    pub parameters: Vec<VendorParameter>,
    pub normalized_parameters: HashMap<String, NormalizedParameter>,
    pub source_page: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedParameter {
    pub label: String,
    pub value: f64,
    pub unit: String,
    pub source_field: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewVendorLibrary {
    pub id: String,
    pub name: String,
    pub component_type: String,
    pub source_file: String,
    pub source_format: String,
    pub version_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewVendorModel {
    pub id: String,
    pub library_id: String,
    pub model_name: String,
    pub brand: String,
    pub series: String,
    pub parameters: Vec<VendorParameter>,
    pub normalized_parameters: HashMap<String, NormalizedParameter>,
    pub source_page: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationRequest {
    pub module_id: String,
    pub component_type: Option<String>,
    pub requirements: Vec<RecommendationRequirement>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationRequirement {
    pub id: String,
    pub label: String,
    pub value: f64,
    pub unit: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationCandidate {
    pub model: VendorModelRecord,
    pub score: f64,
    pub matched_rules: Vec<MatchRuleResult>,
    pub failed_rules: Vec<MatchRuleResult>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchRuleResult {
    pub requirement_id: String,
    pub label: String,
    pub message: String,
    pub required_value: f64,
    pub candidate_value: Option<f64>,
    pub unit: String,
}
