mod db;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .setup(|app| {
            db::initialize_database(app.handle()).map_err(|error| error.to_string())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![db::get_database_health])
        .run(tauri::generate_context!())
        .expect("failed to run selector desktop app");
}
