use rusqlite::{params, OptionalExtension};

use super::models::ParameterCandidateRecord;
use super::repository::{KnowledgeRepository, KnowledgeRepositoryError};

impl<'a> KnowledgeRepository<'a> {
    pub fn list_candidates(
        &self,
    ) -> Result<Vec<ParameterCandidateRecord>, KnowledgeRepositoryError> {
        let mut statement = self.connection.prepare(
            "SELECT id, name, value, unit, scenario, source_page, status
             FROM internal_parameter_candidates
             ORDER BY
                CASE status WHEN 'pending' THEN 0 WHEN 'confirmed' THEN 1 ELSE 2 END,
                name;",
        )?;

        let rows = statement.query_map([], |row| {
            Ok(ParameterCandidateRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                value: row.get(2)?,
                unit: row.get(3)?,
                scenario: row.get(4)?,
                source_page: row.get(5)?,
                status: row.get(6)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(KnowledgeRepositoryError::from)
    }

    pub fn update_candidate_status(
        &self,
        id: &str,
        status: &str,
    ) -> Result<Option<ParameterCandidateRecord>, KnowledgeRepositoryError> {
        self.connection.execute(
            "UPDATE internal_parameter_candidates
             SET status = ?1, updated_at = datetime('now')
             WHERE id = ?2;",
            params![status, id],
        )?;

        self.find_candidate(id)
    }

    pub fn insert_confirmed_parameter(
        &self,
        candidate: &ParameterCandidateRecord,
    ) -> Result<(), KnowledgeRepositoryError> {
        self.connection.execute(
            "INSERT INTO internal_parameters
                (id, name, category, value, unit, scenario, source, updated_at)
             VALUES (?1, ?2, 'PDF 候选参数', ?3, ?4, ?5, ?6, datetime('now'))
             ON CONFLICT(id) DO UPDATE SET
                value = excluded.value,
                unit = excluded.unit,
                scenario = excluded.scenario,
                source = excluded.source,
                updated_at = datetime('now');",
            params![
                format!("parameter-{}", candidate.id),
                candidate.name,
                candidate.value,
                candidate.unit,
                candidate.scenario,
                candidate
                    .source_page
                    .clone()
                    .unwrap_or_else(|| "根目录 PDF".to_string())
            ],
        )?;
        Ok(())
    }

    fn find_candidate(
        &self,
        id: &str,
    ) -> Result<Option<ParameterCandidateRecord>, KnowledgeRepositoryError> {
        self.connection
            .query_row(
                "SELECT id, name, value, unit, scenario, source_page, status
                 FROM internal_parameter_candidates
                 WHERE id = ?1;",
                params![id],
                |row| {
                    Ok(ParameterCandidateRecord {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        value: row.get(2)?,
                        unit: row.get(3)?,
                        scenario: row.get(4)?,
                        source_page: row.get(5)?,
                        status: row.get(6)?,
                    })
                },
            )
            .optional()
            .map_err(KnowledgeRepositoryError::from)
    }
}
