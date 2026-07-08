use serde::{Deserialize, Serialize};

use crate::engine::models::{CalculationRequest, CalculationResult};
use crate::vendor::models::RecommendationCandidate;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportReportRequest {
    pub format: String,
    pub output_path: String,
    pub case_id: Option<String>,
    pub case_name: String,
    pub notes: String,
    pub request: CalculationRequest,
    pub result: CalculationResult,
    #[serde(default)]
    pub candidates: Vec<RecommendationCandidate>,
    pub final_model_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportCaseReportRequest {
    pub case_id: String,
    pub format: String,
    pub output_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportExportRecord {
    pub id: String,
    pub case_id: Option<String>,
    pub run_id: Option<String>,
    pub format: String,
    pub path: String,
    pub exported_at: String,
}

#[derive(Debug, Clone)]
pub struct ReportPayload {
    pub run_id: Option<String>,
    pub case_id: Option<String>,
    pub case_name: String,
    pub notes: String,
    pub request: CalculationRequest,
    pub result: CalculationResult,
    pub candidates: Vec<RecommendationCandidate>,
    pub final_model_name: Option<String>,
}

impl ReportPayload {
    pub fn from_request(request: ExportReportRequest) -> Self {
        Self {
            run_id: None,
            case_id: request.case_id,
            case_name: request.case_name,
            notes: request.notes,
            request: request.request,
            result: request.result,
            candidates: request.candidates,
            final_model_name: request.final_model_name,
        }
    }
}
