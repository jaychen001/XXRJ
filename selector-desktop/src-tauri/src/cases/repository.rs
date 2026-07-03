use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::Value;
use thiserror::Error;

use crate::engine::models::{CalculationRequest, CalculationResult, ModuleDefinition};

use super::models::{CaseFilter, CaseRecord};

#[derive(Debug, Error)]
pub enum CaseRepositoryError {
    #[error("SQLite 执行失败：{0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("JSON 处理失败：{0}")]
    Json(#[from] serde_json::Error),
}

pub struct CaseRepository<'a> {
    connection: &'a Connection,
}

impl<'a> CaseRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn save_case(
        &self,
        name: &str,
        notes: &str,
        module: &ModuleDefinition,
        request: &CalculationRequest,
        result: &CalculationResult,
    ) -> Result<CaseRecord, CaseRepositoryError> {
        self.upsert_module(module)?;
        let case_id = next_id("case");
        let run_id = next_id("run");
        self.insert_case(&case_id, name, notes, module)?;
        self.insert_run(&run_id, Some(&case_id), module, request, result)?;
        self.connection.execute(
            "UPDATE calculation_cases SET current_run_id = ?1, updated_at = datetime('now')
             WHERE id = ?2;",
            params![run_id, case_id],
        )?;
        self.find_case(&case_id)?
            .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows.into())
    }

    pub fn list_cases(&self, filter: &CaseFilter) -> Result<Vec<CaseRecord>, CaseRepositoryError> {
        let like_query =
            normalize_text(filter.query.as_deref()).map(|value| format!("%{}%", value));
        let module_id = normalize_text(filter.module_id.as_deref());
        let created_from = normalize_text(filter.created_from.as_deref());
        let created_to = normalize_date_to(filter.created_to.as_deref());
        let mut statement = self.connection.prepare(
            "SELECT c.id, c.name, c.module_id, m.name, c.notes,
                    COALESCE(json_extract(r.result_snapshot_json, '$.summary'), ''),
                    COALESCE(json_array_length(json_extract(r.result_snapshot_json, '$.risks')), 0),
                    c.created_at, c.updated_at
             FROM calculation_cases c
             JOIN calculation_modules m ON m.id = c.module_id
             LEFT JOIN calculation_runs r ON r.id = c.current_run_id
             WHERE (?1 IS NULL OR c.name LIKE ?1 OR c.notes LIKE ?1 OR m.name LIKE ?1)
                AND (?2 IS NULL OR c.module_id = ?2)
                AND (?3 IS NULL OR c.created_at >= ?3)
                AND (?4 IS NULL OR c.created_at <= ?4)
             ORDER BY c.updated_at DESC;",
        )?;
        let rows = statement.query_map(
            params![like_query, module_id, created_from, created_to],
            map_case_row,
        )?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(CaseRepositoryError::from)
    }

    pub fn update_case(
        &self,
        id: &str,
        name: &str,
        notes: &str,
    ) -> Result<Option<CaseRecord>, CaseRepositoryError> {
        let changed = self.connection.execute(
            "UPDATE calculation_cases
             SET name = ?1, notes = ?2, updated_at = datetime('now')
             WHERE id = ?3;",
            params![name, notes, id],
        )?;
        if changed == 0 {
            return Ok(None);
        }
        self.find_case(id)
    }

    pub fn duplicate_case(&self, id: &str) -> Result<Option<CaseRecord>, CaseRepositoryError> {
        let Some((case, input, result)) = self.load_case_payload(id)? else {
            return Ok(None);
        };
        let Some(module) = crate::engine::get_module_definition(&case.module_id) else {
            return Ok(None);
        };
        self.save_case(
            &format!("{} - 副本", case.name),
            &case.notes,
            &module,
            &input,
            &result,
        )
        .map(Some)
    }

    pub fn load_request(
        &self,
        id: &str,
    ) -> Result<Option<CalculationRequest>, CaseRepositoryError> {
        self.connection
            .query_row(
                "SELECT r.input_snapshot_json
                 FROM calculation_cases c
                 JOIN calculation_runs r ON r.id = c.current_run_id
                 WHERE c.id = ?1;",
                [id],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .map(|json| serde_json::from_str(&json).map_err(CaseRepositoryError::from))
            .transpose()
    }

    pub fn append_run(
        &self,
        case_id: &str,
        module: &ModuleDefinition,
        request: &CalculationRequest,
        result: &CalculationResult,
    ) -> Result<Option<CaseRecord>, CaseRepositoryError> {
        let existing = self.find_case(case_id)?;
        if existing.is_none() {
            return Ok(None);
        }
        let run_id = next_id("run");
        self.insert_run(&run_id, Some(case_id), module, request, result)?;
        self.connection.execute(
            "UPDATE calculation_cases SET current_run_id = ?1, updated_at = datetime('now')
             WHERE id = ?2;",
            params![run_id, case_id],
        )?;
        self.find_case(case_id)
    }

    pub fn delete_case(&self, id: &str) -> Result<bool, CaseRepositoryError> {
        let changed = self
            .connection
            .execute("DELETE FROM calculation_cases WHERE id = ?1;", [id])?;
        Ok(changed > 0)
    }

    fn upsert_module(&self, module: &ModuleDefinition) -> Result<(), CaseRepositoryError> {
        self.connection.execute(
            "INSERT INTO calculation_modules (id, name, category, description, fields_json, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                category = excluded.category,
                description = excluded.description,
                fields_json = excluded.fields_json,
                updated_at = datetime('now');",
            params![
                module.id,
                module.name,
                module.category,
                module.description,
                serde_json::to_string(&module.fields)?
            ],
        )?;
        Ok(())
    }

    fn insert_case(
        &self,
        case_id: &str,
        name: &str,
        notes: &str,
        module: &ModuleDefinition,
    ) -> Result<(), CaseRepositoryError> {
        self.connection.execute(
            "INSERT INTO calculation_cases (id, name, module_id, notes, updated_at)
             VALUES (?1, ?2, ?3, ?4, datetime('now'));",
            params![case_id, name, module.id, notes],
        )?;
        Ok(())
    }

    fn insert_run(
        &self,
        run_id: &str,
        case_id: Option<&str>,
        module: &ModuleDefinition,
        request: &CalculationRequest,
        result: &CalculationResult,
    ) -> Result<(), CaseRepositoryError> {
        self.connection.execute(
            "INSERT INTO calculation_runs
                (id, case_id, module_id, input_snapshot_json, defaults_snapshot_json,
                 formula_version, result_snapshot_json, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'));",
            params![
                run_id,
                case_id,
                module.id,
                serde_json::to_string(request)?,
                result.defaults_snapshot.to_string(),
                result.formula_version,
                serde_json::to_string(result)?
            ],
        )?;
        Ok(())
    }

    pub fn find_case(&self, id: &str) -> Result<Option<CaseRecord>, CaseRepositoryError> {
        self.connection
            .query_row(
                "SELECT c.id, c.name, c.module_id, m.name, c.notes,
                        COALESCE(json_extract(r.result_snapshot_json, '$.summary'), ''),
                        COALESCE(json_array_length(json_extract(r.result_snapshot_json, '$.risks')), 0),
                        c.created_at, c.updated_at
                 FROM calculation_cases c
                 JOIN calculation_modules m ON m.id = c.module_id
                 LEFT JOIN calculation_runs r ON r.id = c.current_run_id
                 WHERE c.id = ?1;",
                [id],
                map_case_row,
            )
            .optional()
            .map_err(CaseRepositoryError::from)
    }

    pub fn load_case_payload(
        &self,
        id: &str,
    ) -> Result<Option<(CaseRecord, CalculationRequest, CalculationResult)>, CaseRepositoryError>
    {
        let Some(case) = self.find_case(id)? else {
            return Ok(None);
        };
        let (input_json, result_json): (String, String) = self.connection.query_row(
            "SELECT r.input_snapshot_json, r.result_snapshot_json
             FROM calculation_cases c
             JOIN calculation_runs r ON r.id = c.current_run_id
             WHERE c.id = ?1;",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        Ok(Some((
            case,
            serde_json::from_str(&input_json)?,
            serde_json::from_str::<Value>(&result_json).and_then(serde_json::from_value)?,
        )))
    }
}

fn map_case_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<CaseRecord> {
    Ok(CaseRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        module_id: row.get(2)?,
        module_name: row.get(3)?,
        notes: row.get(4)?,
        result_summary: row.get(5)?,
        risk_count: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn next_id(prefix: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("{prefix}-{millis}")
}

fn normalize_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn normalize_date_to(value: Option<&str>) -> Option<String> {
    normalize_text(value).map(|date| match date.len() {
        10 => format!("{date} 23:59:59"),
        _ => date,
    })
}
