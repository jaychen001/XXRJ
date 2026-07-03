use rusqlite::{params, Connection};
use thiserror::Error;

use super::models::{
    CoverageRecord, KnowledgeSearchRecord, NewCatalogItem, NewCoverageRecord, NewKnowledgeEntry,
    NewParameterCandidate,
};

#[derive(Debug, Error)]
pub enum KnowledgeRepositoryError {
    #[error("SQLite 执行失败：{0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("JSON 序列化失败：{0}")]
    Json(#[from] serde_json::Error),
}

pub struct KnowledgeRepository<'a> {
    pub(super) connection: &'a Connection,
}

impl<'a> KnowledgeRepository<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }

    pub fn upsert_source(
        &self,
        id: &str,
        file_path: &str,
        title: &str,
    ) -> Result<(), KnowledgeRepositoryError> {
        self.connection.execute(
            "INSERT INTO knowledge_sources (id, source_type, file_path, title, updated_at)
             VALUES (?1, 'pdf', ?2, ?3, datetime('now'))
             ON CONFLICT(id) DO UPDATE SET
                file_path = excluded.file_path,
                title = excluded.title,
                imported_at = datetime('now'),
                updated_at = datetime('now');",
            params![id, file_path, title],
        )?;
        Ok(())
    }

    pub fn replace_entries(
        &self,
        source_id: &str,
        entries: &[NewKnowledgeEntry],
    ) -> Result<(), KnowledgeRepositoryError> {
        self.connection.execute(
            "DELETE FROM knowledge_entries WHERE source_id = ?1;",
            params![source_id],
        )?;

        for entry in entries {
            let tags_json = serde_json::to_string(&entry.tags)?;
            self.connection.execute(
                "INSERT INTO knowledge_entries
                    (id, source_id, page, title, content, tags_json, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'));",
                params![
                    entry.id,
                    source_id,
                    entry.page,
                    entry.title,
                    entry.content,
                    tags_json
                ],
            )?;
        }

        Ok(())
    }

    pub fn replace_catalog(
        &self,
        source_id: &str,
        records: &[NewCatalogItem],
    ) -> Result<(), KnowledgeRepositoryError> {
        self.connection.execute(
            "DELETE FROM pdf_catalog_items WHERE source_id = ?1;",
            params![source_id],
        )?;

        for record in records {
            self.connection.execute(
                "INSERT INTO pdf_catalog_items
                    (id, source_id, chapter, catalog_page, matched_pages, excerpt, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'));",
                params![
                    record.id,
                    source_id,
                    record.chapter,
                    record.catalog_page,
                    record.matched_pages,
                    record.excerpt
                ],
            )?;
        }

        Ok(())
    }

    pub fn upsert_coverage(
        &self,
        records: &[NewCoverageRecord],
    ) -> Result<(), KnowledgeRepositoryError> {
        for record in records {
            self.connection.execute(
                "INSERT INTO pdf_coverage_items
                    (id, chapter, implementation_shape, status, source_page_range, notes, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'))
                 ON CONFLICT(id) DO UPDATE SET
                    chapter = excluded.chapter,
                    implementation_shape = excluded.implementation_shape,
                    status = excluded.status,
                    source_page_range = excluded.source_page_range,
                    notes = excluded.notes,
                    updated_at = datetime('now');",
                params![
                    record.id,
                    record.chapter,
                    record.implementation_shape,
                    record.status,
                    record.source_page_range,
                    record.notes
                ],
            )?;
        }
        Ok(())
    }

    pub fn replace_candidates(
        &self,
        records: &[NewParameterCandidate],
    ) -> Result<(), KnowledgeRepositoryError> {
        self.connection.execute(
            "DELETE FROM internal_parameter_candidates WHERE status = 'pending';",
            [],
        )?;

        for record in records {
            self.connection.execute(
                "INSERT OR IGNORE INTO internal_parameter_candidates
                    (id, name, value, unit, scenario, source_page, status, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending', datetime('now'));",
                params![
                    record.id,
                    record.name,
                    record.value,
                    record.unit,
                    record.scenario,
                    record.source_page
                ],
            )?;
        }

        Ok(())
    }

    pub fn list_coverage(&self) -> Result<Vec<CoverageRecord>, KnowledgeRepositoryError> {
        let mut statement = self.connection.prepare(
            "SELECT
                c.id,
                c.chapter,
                c.implementation_shape,
                c.status,
                c.source_page_range,
                catalog.catalog_page,
                COALESCE(catalog.excerpt, ''),
                COALESCE((
                    SELECT COUNT(*)
                    FROM knowledge_entries entry
                    WHERE entry.tags_json LIKE '%' || c.chapter || '%'
                ), 0),
                c.notes
             FROM pdf_coverage_items c
             LEFT JOIN pdf_catalog_items catalog ON catalog.id = c.id
             ORDER BY c.rowid;",
        )?;

        let rows = statement.query_map([], |row| {
            Ok(CoverageRecord {
                id: row.get(0)?,
                chapter: row.get(1)?,
                implementation_shape: row.get(2)?,
                status: row.get(3)?,
                source_page_range: row.get(4)?,
                catalog_page: row.get(5)?,
                catalog_excerpt: row.get(6)?,
                knowledge_entry_count: row.get(7)?,
                notes: row.get(8)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(KnowledgeRepositoryError::from)
    }

    pub fn list_recent_entries(
        &self,
        limit: i64,
    ) -> Result<Vec<KnowledgeSearchRecord>, KnowledgeRepositoryError> {
        let mut statement = self.connection.prepare(
            "SELECT e.id, e.title, e.content, e.page, e.tags_json, s.title
             FROM knowledge_entries e
             JOIN knowledge_sources s ON s.id = e.source_id
             ORDER BY e.updated_at DESC, e.page
             LIMIT ?1;",
        )?;

        let rows = statement.query_map([limit], map_knowledge_row)?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(KnowledgeRepositoryError::from)
    }

    pub fn search_entries(
        &self,
        query: &str,
        limit: i64,
    ) -> Result<Vec<KnowledgeSearchRecord>, KnowledgeRepositoryError> {
        let pattern = format!("%{}%", query);
        let mut statement = self.connection.prepare(
            "SELECT e.id, e.title, e.content, e.page, e.tags_json, s.title
             FROM knowledge_entries e
             JOIN knowledge_sources s ON s.id = e.source_id
             WHERE e.title LIKE ?1 OR e.content LIKE ?1 OR e.tags_json LIKE ?1
             ORDER BY
                CASE WHEN e.title LIKE ?1 THEN 0 ELSE 1 END,
                e.page IS NULL,
                e.page
             LIMIT ?2;",
        )?;

        let rows = statement.query_map(params![pattern, limit], map_knowledge_row)?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(KnowledgeRepositoryError::from)
    }
}

fn map_knowledge_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<KnowledgeSearchRecord> {
    let tags_json: String = row.get(4)?;
    let tags = serde_json::from_str::<Vec<String>>(&tags_json).unwrap_or_default();
    Ok(KnowledgeSearchRecord {
        id: row.get(0)?,
        title: row.get(1)?,
        content: row.get(2)?,
        page: row.get(3)?,
        tags,
        source_title: row.get(5)?,
    })
}
