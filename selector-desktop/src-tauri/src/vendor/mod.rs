pub mod models;
mod field_mapping;
mod import_job;
mod pdf_import;
mod recommendation;
mod repository;
mod spreadsheet_import;

#[cfg(test)]
mod tests;

use tauri::AppHandle;

use crate::db;

use self::field_mapping::{build_models, validate_confirm_request};
use self::models::{
    ConfirmVendorImportRequest, NewVendorLibrary, RecommendationCandidate, RecommendationRequest,
    VendorImportPreview, VendorImportPreviewRequest, VendorImportSummary, VendorLibraryRecord,
    VendorModelRecord,
};
use self::recommendation::{infer_component_type, recommend_models};
use self::repository::{next_id, VendorRepository};

#[tauri::command]
pub fn preview_vendor_import(
    app_handle: AppHandle,
    request: VendorImportPreviewRequest,
) -> Result<VendorImportPreview, String> {
    db::initialize_database(&app_handle).map_err(|error| error.to_string())?;
    let preview = import_job::build_preview(&request)?;
    let connection = db::open_database(&app_handle).map_err(|error| error.to_string())?;
    VendorRepository::new(&connection)
        .insert_import_job(&preview.job_id, None, &preview, "previewed")
        .map_err(|error| error.to_string())?;
    Ok(preview)
}

#[tauri::command]
pub fn confirm_vendor_import(
    app_handle: AppHandle,
    request: ConfirmVendorImportRequest,
) -> Result<VendorImportSummary, String> {
    validate_confirm_request(&request)?;
    db::initialize_database(&app_handle).map_err(|error| error.to_string())?;
    let mut connection = db::open_database(&app_handle).map_err(|error| error.to_string())?;
    let transaction = connection.transaction().map_err(|error| error.to_string())?;
    let repository = VendorRepository::new(&transaction);
    let library_id = next_id("vendor-library");
    let library = repository
        .insert_library(&NewVendorLibrary {
            id: library_id.clone(),
            name: request.library_name.trim().to_string(),
            component_type: request.component_type.trim().to_string(),
            source_file: request.preview.source_file.clone(),
            source_format: request.preview.source_format.clone(),
            version_name: request.version_name.trim().to_string(),
        })
        .map_err(|error| error.to_string())?;
    let models = build_models(&request, &library_id);
    repository
        .insert_models(&models)
        .map_err(|error| error.to_string())?;
    repository
        .insert_import_job(
            &request.preview.job_id,
            Some(&library_id),
            &request.preview,
            "confirmed",
        )
        .map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())?;
    Ok(VendorImportSummary {
        library,
        imported_models: models.len(),
        failed_rows: request.preview.failed_rows.len(),
        job_id: request.preview.job_id,
    })
}

#[tauri::command]
pub fn list_vendor_libraries(app_handle: AppHandle) -> Result<Vec<VendorLibraryRecord>, String> {
    db::initialize_database(&app_handle).map_err(|error| error.to_string())?;
    let connection = db::open_database(&app_handle).map_err(|error| error.to_string())?;
    VendorRepository::new(&connection)
        .list_libraries()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_vendor_models(
    app_handle: AppHandle,
    library_id: Option<String>,
) -> Result<Vec<VendorModelRecord>, String> {
    db::initialize_database(&app_handle).map_err(|error| error.to_string())?;
    let connection = db::open_database(&app_handle).map_err(|error| error.to_string())?;
    VendorRepository::new(&connection)
        .list_models(library_id.as_deref(), false, None)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_vendor_library_enabled(
    app_handle: AppHandle,
    id: String,
    enabled: bool,
) -> Result<VendorLibraryRecord, String> {
    db::initialize_database(&app_handle).map_err(|error| error.to_string())?;
    let connection = db::open_database(&app_handle).map_err(|error| error.to_string())?;
    VendorRepository::new(&connection)
        .set_library_enabled(&id, enabled)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "厂家样本库不存在。".to_string())
}

#[tauri::command]
pub fn delete_vendor_library(app_handle: AppHandle, id: String) -> Result<bool, String> {
    db::initialize_database(&app_handle).map_err(|error| error.to_string())?;
    let connection = db::open_database(&app_handle).map_err(|error| error.to_string())?;
    VendorRepository::new(&connection)
        .delete_library(&id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn recommend_vendor_models(
    app_handle: AppHandle,
    request: RecommendationRequest,
) -> Result<Vec<RecommendationCandidate>, String> {
    db::initialize_database(&app_handle).map_err(|error| error.to_string())?;
    let connection = db::open_database(&app_handle).map_err(|error| error.to_string())?;
    let component_type = request
        .component_type
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(ToString::to_string)
        .or_else(|| infer_component_type(&request.module_id));
    let repository = VendorRepository::new(&connection);
    let mut models = repository
        .list_models(None, true, component_type.as_deref())
        .map_err(|error| error.to_string())?;
    if models.is_empty() && component_type.is_some() {
        models = repository
            .list_models(None, true, None)
            .map_err(|error| error.to_string())?;
    }
    Ok(recommend_models(&request, models))
}
