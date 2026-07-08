pub mod models;
mod content;
mod excel_report;
mod pdf_report;
mod repository;

#[cfg(test)]
mod tests;

use std::path::Path;

use tauri::AppHandle;

use crate::db;

use self::models::{
    ExportCaseReportRequest, ExportReportRequest, ReportExportRecord, ReportPayload,
};
use self::repository::{next_id, ReportRepository};

#[tauri::command]
pub fn export_calculation_report(
    app_handle: AppHandle,
    request: ExportReportRequest,
) -> Result<ReportExportRecord, String> {
    db::initialize_database(&app_handle).map_err(|error| error.to_string())?;
    let connection = db::open_database(&app_handle).map_err(|error| error.to_string())?;
    let format = normalize_format(&request.format)?;
    let output_path = normalize_output_path(&request.output_path)?;
    let payload = ReportPayload::from_request(request);
    write_report(&format, Path::new(&output_path), &payload)?;
    let record = ReportRepository::new(&connection)
        .insert_export(&payload, &format, &output_path)
        .map_err(|error| error.to_string())?
        .unwrap_or_else(|| ad_hoc_record(&payload, &format, &output_path));
    Ok(record)
}

#[tauri::command]
pub fn export_case_report(
    app_handle: AppHandle,
    request: ExportCaseReportRequest,
) -> Result<ReportExportRecord, String> {
    db::initialize_database(&app_handle).map_err(|error| error.to_string())?;
    let connection = db::open_database(&app_handle).map_err(|error| error.to_string())?;
    let format = normalize_format(&request.format)?;
    let output_path = normalize_output_path(&request.output_path)?;
    let repository = ReportRepository::new(&connection);
    let payload = repository
        .load_case_payload(&request.case_id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "案例不存在或没有当前计算结果。".to_string())?;
    write_report(&format, Path::new(&output_path), &payload)?;
    repository
        .insert_export(&payload, &format, &output_path)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "导出记录写入失败。".to_string())
}

#[tauri::command]
pub fn list_report_exports(app_handle: AppHandle) -> Result<Vec<ReportExportRecord>, String> {
    db::initialize_database(&app_handle).map_err(|error| error.to_string())?;
    let connection = db::open_database(&app_handle).map_err(|error| error.to_string())?;
    ReportRepository::new(&connection)
        .list_exports()
        .map_err(|error| error.to_string())
}

fn write_report(format: &str, path: &Path, payload: &ReportPayload) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            return Err("导出路径所在文件夹不存在，请更换路径。".to_string());
        }
    }
    match format {
        "pdf" => pdf_report::write_pdf(path, payload),
        "xlsx" => excel_report::write_excel(path, payload),
        _ => Err("仅支持 PDF 和 Excel 导出。".to_string()),
    }
}

fn normalize_format(format: &str) -> Result<String, String> {
    match format.trim().to_lowercase().as_str() {
        "pdf" => Ok("pdf".to_string()),
        "excel" | "xlsx" => Ok("xlsx".to_string()),
        _ => Err("导出格式必须是 PDF 或 Excel。".to_string()),
    }
}

fn normalize_output_path(path: &str) -> Result<String, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("导出路径不能为空。".to_string());
    }
    Ok(trimmed.to_string())
}

fn ad_hoc_record(payload: &ReportPayload, format: &str, path: &str) -> ReportExportRecord {
    ReportExportRecord {
        id: next_id("report-export"),
        case_id: payload.case_id.clone(),
        run_id: payload.run_id.clone(),
        format: format.to_string(),
        path: path.to_string(),
        exported_at: "未落库".to_string(),
    }
}
