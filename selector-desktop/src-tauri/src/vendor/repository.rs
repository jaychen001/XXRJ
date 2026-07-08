use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection, OptionalExtension};
use thiserror::Error;

use super::models::{
    NewVendorLibrary, NewVendorModel, NormalizedParameter, VendorImportPreview,
    VendorLibraryRecord, VendorModelRecord, VendorParameter,
};

#[derive(Debug, Error)]
pub enum VendorRepositoryError {
    #[error("SQLite 执行失败：{0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("JSON 处理失败：{0}")]
    Json(#[from] serde_json::Error),
}

pub struct VendorRepository<'a> {
    connection: &'a Connection,
}

impl<'a> VendorRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn insert_import_job(
        &self,
        id: &str,
        library_id: Option<&str>,
        preview: &VendorImportPreview,
        status: &str,
    ) -> Result<(), VendorRepositoryError> {
        self.connection.execute(
            "INSERT INTO vendor_import_jobs
                (id, library_id, source_file, source_format, status, extracted_preview_json,
                 confidence, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'))
             ON CONFLICT(id) DO UPDATE SET
                library_id = excluded.library_id,
                status = excluded.status,
                extracted_preview_json = excluded.extracted_preview_json,
                confidence = excluded.confidence,
                updated_at = datetime('now');",
            params![
                id,
                library_id,
                preview.source_file,
                preview.source_format,
                status,
                serde_json::to_string(preview)?,
                preview.confidence
            ],
        )?;
        Ok(())
    }

    pub fn insert_library(
        &self,
        library: &NewVendorLibrary,
    ) -> Result<VendorLibraryRecord, VendorRepositoryError> {
        self.connection.execute(
            "INSERT INTO vendor_libraries
                (id, name, component_type, source_file, source_format, version_name, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'));",
            params![
                library.id,
                library.name,
                library.component_type,
                library.source_file,
                library.source_format,
                library.version_name
            ],
        )?;
        self.find_library(&library.id)?
            .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows.into())
    }

    pub fn insert_models(&self, models: &[NewVendorModel]) -> Result<(), VendorRepositoryError> {
        for model in models {
            self.connection.execute(
                "INSERT INTO vendor_models
                    (id, library_id, model_name, brand, series, parameters_json,
                     normalized_parameters_json, source_page, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, datetime('now'));",
                params![
                    model.id,
                    model.library_id,
                    model.model_name,
                    model.brand,
                    model.series,
                    serde_json::to_string(&model.parameters)?,
                    serde_json::to_string(&model.normalized_parameters)?,
                    model.source_page
                ],
            )?;
        }
        Ok(())
    }

    pub fn list_libraries(&self) -> Result<Vec<VendorLibraryRecord>, VendorRepositoryError> {
        let mut statement = self.connection.prepare(
            "SELECT l.id, l.name, l.component_type, l.source_file, l.source_format,
                    l.version_name, l.imported_at, l.enabled,
                    COUNT(m.id), l.created_at, l.updated_at
             FROM vendor_libraries l
             LEFT JOIN vendor_models m ON m.library_id = l.id
             GROUP BY l.id
             ORDER BY l.updated_at DESC;",
        )?;
        let rows = statement.query_map([], map_library_row)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(VendorRepositoryError::from)
    }

    pub fn list_models(
        &self,
        library_id: Option<&str>,
        only_enabled: bool,
        component_type: Option<&str>,
    ) -> Result<Vec<VendorModelRecord>, VendorRepositoryError> {
        let mut statement = self.connection.prepare(
            "SELECT m.id, m.library_id, l.name, l.component_type, m.model_name, m.brand, m.series,
                    m.parameters_json, m.normalized_parameters_json, m.source_page, l.enabled
             FROM vendor_models m
             JOIN vendor_libraries l ON l.id = m.library_id
             WHERE (?1 IS NULL OR m.library_id = ?1)
               AND (?2 = 0 OR l.enabled = 1)
               AND (?3 IS NULL OR l.component_type = ?3)
             ORDER BY l.updated_at DESC, m.model_name;",
        )?;
        let rows = statement.query_map(
            params![library_id, if only_enabled { 1 } else { 0 }, component_type],
            map_model_row,
        )?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(VendorRepositoryError::from)
    }

    pub fn set_library_enabled(
        &self,
        id: &str,
        enabled: bool,
    ) -> Result<Option<VendorLibraryRecord>, VendorRepositoryError> {
        let changed = self.connection.execute(
            "UPDATE vendor_libraries
             SET enabled = ?1, updated_at = datetime('now')
             WHERE id = ?2;",
            params![if enabled { 1 } else { 0 }, id],
        )?;
        if changed == 0 {
            return Ok(None);
        }
        self.find_library(id)
    }

    pub fn delete_library(&self, id: &str) -> Result<bool, VendorRepositoryError> {
        let changed = self
            .connection
            .execute("DELETE FROM vendor_libraries WHERE id = ?1;", [id])?;
        Ok(changed > 0)
    }

    fn find_library(
        &self,
        id: &str,
    ) -> Result<Option<VendorLibraryRecord>, VendorRepositoryError> {
        self.connection
            .query_row(
                "SELECT l.id, l.name, l.component_type, l.source_file, l.source_format,
                        l.version_name, l.imported_at, l.enabled,
                        COUNT(m.id), l.created_at, l.updated_at
                 FROM vendor_libraries l
                 LEFT JOIN vendor_models m ON m.library_id = l.id
                 WHERE l.id = ?1
                 GROUP BY l.id;",
                [id],
                map_library_row,
            )
            .optional()
            .map_err(VendorRepositoryError::from)
    }
}

pub fn next_id(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("{prefix}-{nanos}")
}

fn map_library_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<VendorLibraryRecord> {
    Ok(VendorLibraryRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        component_type: row.get(2)?,
        source_file: row.get(3)?,
        source_format: row.get(4)?,
        version_name: row.get(5)?,
        imported_at: row.get(6)?,
        enabled: row.get::<_, i64>(7)? == 1,
        model_count: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

fn map_model_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<VendorModelRecord> {
    let parameters_json: String = row.get(7)?;
    let normalized_json: String = row.get(8)?;
    let parameters =
        serde_json::from_str::<Vec<VendorParameter>>(&parameters_json).unwrap_or_default();
    let normalized_parameters = serde_json::from_str::<HashMap<String, NormalizedParameter>>(
        &normalized_json,
    )
    .unwrap_or_default();
    Ok(VendorModelRecord {
        id: row.get(0)?,
        library_id: row.get(1)?,
        library_name: row.get(2)?,
        component_type: row.get(3)?,
        model_name: row.get(4)?,
        brand: row.get(5)?,
        series: row.get(6)?,
        parameters,
        normalized_parameters,
        source_page: row.get(9)?,
        enabled: row.get::<_, i64>(10)? == 1,
    })
}
