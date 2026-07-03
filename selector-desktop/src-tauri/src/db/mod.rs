use std::fs;
use std::path::PathBuf;

use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;
use tauri::{AppHandle, Manager};
use thiserror::Error;

const DATABASE_FILE_NAME: &str = "selector.db";
const INITIAL_MIGRATION_ID: &str = "0001_init";
const INITIAL_MIGRATION_SQL: &str = include_str!("../../migrations/0001_init.sql");

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("无法定位应用数据目录：{0}")]
    AppDataDir(#[from] tauri::Error),
    #[error("无法创建应用数据目录：{0}")]
    CreateDir(#[from] std::io::Error),
    #[error("SQLite 执行失败：{0}")]
    Sqlite(#[from] rusqlite::Error),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseHealth {
    status: String,
    database_path: String,
    applied_migrations: i64,
    table_count: i64,
    message: String,
}

pub fn initialize_database(app_handle: &AppHandle) -> Result<(), DatabaseError> {
    let database_path = database_path(app_handle)?;
    let mut connection = Connection::open(database_path)?;
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;
    run_migrations(&mut connection)?;

    Ok(())
}

fn run_migrations(connection: &mut Connection) -> Result<(), DatabaseError> {
    let transaction = connection.transaction()?;
    transaction.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            id TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;

    let applied = transaction
        .query_row(
            "SELECT id FROM schema_migrations WHERE id = ?1;",
            [INITIAL_MIGRATION_ID],
            |row| row.get::<_, String>(0),
        )
        .optional()?;

    if applied.is_none() {
        transaction.execute_batch(INITIAL_MIGRATION_SQL)?;
        transaction.execute(
            "INSERT INTO schema_migrations (id) VALUES (?1);",
            [INITIAL_MIGRATION_ID],
        )?;
    }

    transaction.commit()?;
    Ok(())
}

#[tauri::command]
pub fn get_database_health(app_handle: AppHandle) -> Result<DatabaseHealth, String> {
    initialize_database(&app_handle).map_err(|error| error.to_string())?;

    let path = database_path(&app_handle).map_err(|error| error.to_string())?;
    let connection = Connection::open(&path).map_err(|error| error.to_string())?;

    let applied_migrations = connection
        .query_row("SELECT COUNT(*) FROM schema_migrations;", [], |row| {
            row.get::<_, i64>(0)
        })
        .map_err(|error| error.to_string())?;

    let table_count = connection
        .query_row(
            "SELECT COUNT(*) FROM sqlite_schema
             WHERE type = 'table' AND name NOT LIKE 'sqlite_%';",
            [],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| error.to_string())?;

    Ok(DatabaseHealth {
        status: "ok".to_string(),
        database_path: path.to_string_lossy().to_string(),
        applied_migrations,
        table_count,
        message: "SQLite 数据库已初始化".to_string(),
    })
}

fn database_path(app_handle: &AppHandle) -> Result<PathBuf, DatabaseError> {
    let app_data_dir = app_handle.path().app_data_dir()?;
    fs::create_dir_all(&app_data_dir)?;
    Ok(app_data_dir.join(DATABASE_FILE_NAME))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_create_expected_tables_and_stay_idempotent() -> Result<(), DatabaseError> {
        let mut connection = Connection::open_in_memory()?;
        run_migrations(&mut connection)?;
        run_migrations(&mut connection)?;

        let expected_tables = [
            "schema_migrations",
            "app_meta",
            "calculation_modules",
            "module_items",
            "knowledge_sources",
            "knowledge_entries",
            "pdf_coverage_items",
            "internal_parameter_candidates",
            "internal_parameters",
            "calculation_cases",
            "calculation_runs",
            "module_fixtures",
            "vendor_libraries",
            "vendor_import_jobs",
            "vendor_models",
            "recommendation_results",
            "report_exports",
        ];

        for table_name in expected_tables {
            let table_count = connection.query_row(
                "SELECT COUNT(*) FROM sqlite_schema WHERE type = 'table' AND name = ?1;",
                [table_name],
                |row| row.get::<_, i64>(0),
            )?;
            assert_eq!(table_count, 1, "missing table {table_name}");
        }

        let migration_count = connection.query_row(
            "SELECT COUNT(*) FROM schema_migrations WHERE id = ?1;",
            [INITIAL_MIGRATION_ID],
            |row| row.get::<_, i64>(0),
        )?;
        assert_eq!(migration_count, 1);

        Ok(())
    }
}
