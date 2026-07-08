use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension};
use thiserror::Error;

use super::models::{ReportExportRecord, ReportPayload};

#[derive(Debug, Error)]
pub enum ReportRepositoryError {
    #[error("SQLite 执行失败：{0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("JSON 处理失败：{0}")]
    Json(#[from] serde_json::Error),
}

pub struct ReportRepository<'a> {
    connection: &'a Connection,
}

impl<'a> ReportRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn load_case_payload(
        &self,
        case_id: &str,
    ) -> Result<Option<ReportPayload>, ReportRepositoryError> {
        let row = self
            .connection
            .query_row(
                "SELECT c.id, c.name, c.notes, c.current_run_id,
                        r.input_snapshot_json, r.result_snapshot_json
                 FROM calculation_cases c
                 JOIN calculation_runs r ON r.id = c.current_run_id
                 WHERE c.id = ?1;",
                [case_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, String>(5)?,
                    ))
                },
            )
            .optional()?;

        row.map(
            |(case_id, case_name, notes, run_id, input_json, result_json)| {
                Ok(ReportPayload {
                    run_id: Some(run_id),
                    case_id: Some(case_id),
                    case_name,
                    notes,
                    request: serde_json::from_str(&input_json)?,
                    result: serde_json::from_str(&result_json)?,
                    candidates: Vec::new(),
                    final_model_name: None,
                })
            },
        )
        .transpose()
    }

    pub fn insert_export(
        &self,
        payload: &ReportPayload,
        format: &str,
        path: &str,
    ) -> Result<Option<ReportExportRecord>, ReportRepositoryError> {
        let Some(run_id) = payload.run_id.as_deref() else {
            return Ok(None);
        };
        let id = next_id("report-export");
        self.connection.execute(
            "INSERT INTO report_exports
                (id, case_id, run_id, format, path, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'));",
            params![id, payload.case_id, run_id, format, path],
        )?;
        self.find_export(&id)
    }

    pub fn list_exports(&self) -> Result<Vec<ReportExportRecord>, ReportRepositoryError> {
        let mut statement = self.connection.prepare(
            "SELECT id, case_id, run_id, format, path, exported_at
             FROM report_exports
             ORDER BY exported_at DESC
             LIMIT 50;",
        )?;
        let rows = statement.query_map([], map_export_row)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(ReportRepositoryError::from)
    }

    fn find_export(&self, id: &str) -> Result<Option<ReportExportRecord>, ReportRepositoryError> {
        self.connection
            .query_row(
                "SELECT id, case_id, run_id, format, path, exported_at
                 FROM report_exports
                 WHERE id = ?1;",
                [id],
                map_export_row,
            )
            .optional()
            .map_err(ReportRepositoryError::from)
    }
}

pub fn next_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("{prefix}-{nanos}")
}

fn map_export_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ReportExportRecord> {
    Ok(ReportExportRecord {
        id: row.get(0)?,
        case_id: row.get(1)?,
        run_id: row.get(2)?,
        format: row.get(3)?,
        path: row.get(4)?,
        exported_at: row.get(5)?,
    })
}
