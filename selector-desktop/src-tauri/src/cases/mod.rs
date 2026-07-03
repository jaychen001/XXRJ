pub mod models;
mod repository;
#[cfg(test)]
mod tests;

use tauri::AppHandle;

use crate::db;
use crate::engine;

use self::models::{
    CaseDetailRecord, CaseFilter, CaseRecord, CaseRunRecord, SaveCaseRequest, UpdateCaseRequest,
};
use self::repository::CaseRepository;

#[tauri::command]
pub fn save_calculation_case(
    app_handle: AppHandle,
    payload: SaveCaseRequest,
) -> Result<CaseRunRecord, String> {
    validate_case_name(&payload.name)?;
    let module = engine::get_module_definition(&payload.request.module_id)
        .ok_or_else(|| "计算模块不存在".to_string())?;
    let result = engine::calculate_request(&payload.request)
        .map_err(|error| format!("{}：{}", error.field_id, error.message))?;
    db::initialize_database(&app_handle).map_err(|error| error.to_string())?;
    let mut connection = db::open_database(&app_handle).map_err(|error| error.to_string())?;
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    let repository = CaseRepository::new(&transaction);
    let case_record = repository
        .save_case(
            &payload.name,
            &payload.notes,
            &module,
            &payload.request,
            &result,
        )
        .map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())?;
    Ok(CaseRunRecord {
        case_record,
        result,
    })
}

#[tauri::command]
pub fn list_calculation_cases(
    app_handle: AppHandle,
    filter: Option<CaseFilter>,
) -> Result<Vec<CaseRecord>, String> {
    db::initialize_database(&app_handle).map_err(|error| error.to_string())?;
    let connection = db::open_database(&app_handle).map_err(|error| error.to_string())?;
    let filter = filter.unwrap_or_default();
    CaseRepository::new(&connection)
        .list_cases(&filter)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_calculation_case(
    app_handle: AppHandle,
    payload: UpdateCaseRequest,
) -> Result<CaseRecord, String> {
    validate_case_name(&payload.name)?;
    db::initialize_database(&app_handle).map_err(|error| error.to_string())?;
    let connection = db::open_database(&app_handle).map_err(|error| error.to_string())?;
    CaseRepository::new(&connection)
        .update_case(&payload.id, &payload.name, &payload.notes)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "案例不存在".to_string())
}

#[tauri::command]
pub fn duplicate_calculation_case(app_handle: AppHandle, id: String) -> Result<CaseRecord, String> {
    db::initialize_database(&app_handle).map_err(|error| error.to_string())?;
    let mut connection = db::open_database(&app_handle).map_err(|error| error.to_string())?;
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    let repository = CaseRepository::new(&transaction);
    let case_record = repository
        .duplicate_case(&id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "案例不存在或模块未实现".to_string())?;
    transaction.commit().map_err(|error| error.to_string())?;
    Ok(case_record)
}

#[tauri::command]
pub fn get_calculation_case_detail(
    app_handle: AppHandle,
    id: String,
) -> Result<CaseDetailRecord, String> {
    db::initialize_database(&app_handle).map_err(|error| error.to_string())?;
    let connection = db::open_database(&app_handle).map_err(|error| error.to_string())?;
    let repository = CaseRepository::new(&connection);
    let (case_record, request, result) = repository
        .load_case_payload(&id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "案例不存在".to_string())?;
    Ok(CaseDetailRecord {
        case_record,
        request,
        result,
    })
}

#[tauri::command]
pub fn rerun_calculation_case(app_handle: AppHandle, id: String) -> Result<CaseRunRecord, String> {
    db::initialize_database(&app_handle).map_err(|error| error.to_string())?;
    let mut connection = db::open_database(&app_handle).map_err(|error| error.to_string())?;
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    let repository = CaseRepository::new(&transaction);
    let request = repository
        .load_request(&id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "案例不存在".to_string())?;
    let module = engine::get_module_definition(&request.module_id)
        .ok_or_else(|| "计算模块不存在".to_string())?;
    let result = engine::calculate_request(&request)
        .map_err(|error| format!("{}：{}", error.field_id, error.message))?;
    let case_record = repository
        .append_run(&id, &module, &request, &result)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "案例不存在".to_string())?;
    transaction.commit().map_err(|error| error.to_string())?;
    Ok(CaseRunRecord {
        case_record,
        result,
    })
}

#[tauri::command]
pub fn rerun_calculation_case_with_request(
    app_handle: AppHandle,
    id: String,
    request: crate::engine::models::CalculationRequest,
) -> Result<CaseRunRecord, String> {
    db::initialize_database(&app_handle).map_err(|error| error.to_string())?;
    let mut connection = db::open_database(&app_handle).map_err(|error| error.to_string())?;
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    let repository = CaseRepository::new(&transaction);
    let existing = repository
        .find_case(&id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "案例不存在".to_string())?;
    if existing.module_id != request.module_id {
        return Err("案例模块与请求模块不一致".to_string());
    }
    let module = engine::get_module_definition(&request.module_id)
        .ok_or_else(|| "计算模块不存在".to_string())?;
    let result = engine::calculate_request(&request)
        .map_err(|error| format!("{}：{}", error.field_id, error.message))?;
    let case_record = repository
        .append_run(&id, &module, &request, &result)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "案例不存在".to_string())?;
    transaction.commit().map_err(|error| error.to_string())?;
    Ok(CaseRunRecord {
        case_record,
        result,
    })
}

#[tauri::command]
pub fn delete_calculation_case(app_handle: AppHandle, id: String) -> Result<bool, String> {
    db::initialize_database(&app_handle).map_err(|error| error.to_string())?;
    let connection = db::open_database(&app_handle).map_err(|error| error.to_string())?;
    CaseRepository::new(&connection)
        .delete_case(&id)
        .map_err(|error| error.to_string())
}

fn validate_case_name(name: &str) -> Result<(), String> {
    let length = name.trim().chars().count();
    if length == 0 || length > 100 {
        return Err("案例名称必须为 1-100 个字符".to_string());
    }
    Ok(())
}
