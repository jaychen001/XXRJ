mod cases;
mod db;
mod engine;
mod knowledge;
mod modules;
mod pdf;
mod qa;
mod report;
mod vendor;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .setup(|app| {
            db::initialize_database(app.handle()).map_err(|error| error.to_string())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cases::delete_calculation_case,
            cases::duplicate_calculation_case,
            cases::get_calculation_case_detail,
            cases::list_calculation_cases,
            cases::rerun_calculation_case,
            cases::rerun_calculation_case_with_request,
            cases::save_calculation_case,
            cases::update_calculation_case,
            db::get_database_health,
            engine::list_calculation_modules,
            engine::run_calculation,
            pdf::root_note_ingest::get_pdf_coverage_items,
            pdf::root_note_ingest::ingest_root_pdf_note,
            pdf::root_note_ingest::list_parameter_candidates,
            pdf::root_note_ingest::list_recent_knowledge_entries,
            pdf::root_note_ingest::search_knowledge_entries,
            pdf::root_note_ingest::update_parameter_candidate_status,
            qa::get_qa_coverage_audit,
            report::export_calculation_report,
            report::export_case_report,
            report::list_report_exports,
            vendor::confirm_vendor_import,
            vendor::delete_vendor_library,
            vendor::list_vendor_libraries,
            vendor::list_vendor_models,
            vendor::preview_vendor_import,
            vendor::recommend_vendor_models,
            vendor::set_vendor_library_enabled
        ])
        .run(tauri::generate_context!())
        .expect("failed to run selector desktop app");
}
