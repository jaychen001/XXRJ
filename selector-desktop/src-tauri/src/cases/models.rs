use serde::{Deserialize, Serialize};

use crate::engine::models::{CalculationRequest, CalculationResult};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveCaseRequest {
    pub name: String,
    pub notes: String,
    pub request: CalculationRequest,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseFilter {
    pub query: Option<String>,
    pub module_id: Option<String>,
    pub created_from: Option<String>,
    pub created_to: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCaseRequest {
    pub id: String,
    pub name: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseRecord {
    pub id: String,
    pub name: String,
    pub module_id: String,
    pub module_name: String,
    pub notes: String,
    pub result_summary: String,
    pub risk_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseRunRecord {
    pub case_record: CaseRecord,
    pub result: CalculationResult,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseDetailRecord {
    pub case_record: CaseRecord,
    pub request: CalculationRequest,
    pub result: CalculationResult,
}
