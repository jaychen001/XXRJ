use std::env;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::AppHandle;

use crate::db;
use crate::knowledge::models::{CoverageRecord, KnowledgeSearchRecord, ParameterCandidateRecord};
use crate::knowledge::repository::KnowledgeRepository;

use super::catalog::build_catalog_items;
use super::coverage::{
    build_coverage_records, coverage_seeds, coverage_status, phase4_source_page,
};
use super::knowledge_entries::build_knowledge_entries;
use super::parameters::build_parameter_candidates;
use super::text_extract::extract_pages;

const ROOT_PDF_NAME: &str = "非标笔记 2025-6-20 18396 1(1).pdf";
const ROOT_SOURCE_ID: &str = "root-nonstandard-note";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RootPdfIngestSummary {
    pdf_path: String,
    page_count: usize,
    catalog_count: usize,
    coverage_count: usize,
    knowledge_entry_count: usize,
    parameter_candidate_count: usize,
}

#[tauri::command]
pub fn ingest_root_pdf_note(app_handle: AppHandle) -> Result<RootPdfIngestSummary, String> {
    db::initialize_database(&app_handle).map_err(|error| error.to_string())?;
    let pdf_path = find_root_pdf().map_err(|error| error.to_string())?;
    let pages = extract_pages(&pdf_path).map_err(|error| error.to_string())?;
    let seeds = coverage_seeds().map_err(|error| error.to_string())?;
    let catalog = build_catalog_items(&seeds, &pages);
    let coverage = build_coverage_records(&seeds, &pages);
    let entries = build_knowledge_entries(&seeds, &pages);
    let candidates = build_parameter_candidates(&pages);
    let mut connection = db::open_database(&app_handle).map_err(|error| error.to_string())?;
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    let repository = KnowledgeRepository::new(&transaction);

    repository
        .upsert_source(
            ROOT_SOURCE_ID,
            &pdf_path.to_string_lossy(),
            "根目录非标笔记 PDF",
        )
        .map_err(|error| error.to_string())?;
    repository
        .replace_catalog(ROOT_SOURCE_ID, &catalog)
        .map_err(|error| error.to_string())?;
    repository
        .upsert_coverage(&coverage)
        .map_err(|error| error.to_string())?;
    repository
        .replace_entries(ROOT_SOURCE_ID, &entries)
        .map_err(|error| error.to_string())?;
    repository
        .replace_candidates(&candidates)
        .map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())?;

    Ok(RootPdfIngestSummary {
        pdf_path: pdf_path.to_string_lossy().to_string(),
        page_count: pages.len(),
        catalog_count: catalog.len(),
        coverage_count: coverage.len(),
        knowledge_entry_count: entries.len(),
        parameter_candidate_count: candidates.len(),
    })
}

#[tauri::command]
pub fn get_pdf_coverage_items(app_handle: AppHandle) -> Result<Vec<CoverageRecord>, String> {
    db::initialize_database(&app_handle).map_err(|error| error.to_string())?;
    let connection = db::open_database(&app_handle).map_err(|error| error.to_string())?;
    let repository = KnowledgeRepository::new(&connection);
    let records = repository
        .list_coverage()
        .map_err(|error| error.to_string())?;
    if records.is_empty() {
        return planned_coverage_records();
    }
    Ok(records)
}

#[tauri::command]
pub fn list_recent_knowledge_entries(
    app_handle: AppHandle,
) -> Result<Vec<KnowledgeSearchRecord>, String> {
    db::initialize_database(&app_handle).map_err(|error| error.to_string())?;
    let connection = db::open_database(&app_handle).map_err(|error| error.to_string())?;
    KnowledgeRepository::new(&connection)
        .list_recent_entries(6)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn search_knowledge_entries(
    app_handle: AppHandle,
    query: String,
) -> Result<Vec<KnowledgeSearchRecord>, String> {
    let trimmed = query.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 100 {
        return Err("搜索词必须为 1-100 个字符".to_string());
    }

    db::initialize_database(&app_handle).map_err(|error| error.to_string())?;
    let connection = db::open_database(&app_handle).map_err(|error| error.to_string())?;
    KnowledgeRepository::new(&connection)
        .search_entries(trimmed, 50)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_parameter_candidates(
    app_handle: AppHandle,
) -> Result<Vec<ParameterCandidateRecord>, String> {
    db::initialize_database(&app_handle).map_err(|error| error.to_string())?;
    let connection = db::open_database(&app_handle).map_err(|error| error.to_string())?;
    KnowledgeRepository::new(&connection)
        .list_candidates()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_parameter_candidate_status(
    app_handle: AppHandle,
    id: String,
    status: String,
) -> Result<ParameterCandidateRecord, String> {
    if status != "confirmed" && status != "ignored" {
        return Err("候选参数状态只能是 confirmed 或 ignored".to_string());
    }

    db::initialize_database(&app_handle).map_err(|error| error.to_string())?;
    let mut connection = db::open_database(&app_handle).map_err(|error| error.to_string())?;
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    let repository = KnowledgeRepository::new(&transaction);
    let candidate = repository
        .update_candidate_status(&id, &status)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "候选参数不存在".to_string())?;

    if status == "confirmed" {
        repository
            .insert_confirmed_parameter(&candidate)
            .map_err(|error| error.to_string())?;
    }
    transaction.commit().map_err(|error| error.to_string())?;

    Ok(candidate)
}

pub(crate) fn find_root_pdf() -> Result<PathBuf, String> {
    let mut start_points = Vec::new();
    start_points.push(env::current_dir().map_err(|error| error.to_string())?);
    if let Ok(exe_path) = env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            start_points.push(parent.to_path_buf());
        }
    }

    for start in start_points {
        if let Some(path) = find_in_ancestors(&start, ROOT_PDF_NAME) {
            return Ok(path);
        }
    }

    Err(format!("未找到根目录 PDF：{}", ROOT_PDF_NAME))
}

fn planned_coverage_records() -> Result<Vec<CoverageRecord>, String> {
    coverage_seeds()
        .map(|seeds| {
            seeds
                .into_iter()
                .map(|seed| CoverageRecord {
                    id: seed.id.clone(),
                    chapter: seed.chapter,
                    implementation_shape: seed.implementation_shape,
                    status: coverage_status(&seed.id, false).to_string(),
                    source_page_range: phase4_source_page(&seed.id).map(ToString::to_string),
                    catalog_page: None,
                    catalog_excerpt: "尚未读取根目录 PDF".to_string(),
                    knowledge_entry_count: 0,
                    notes: seed.requirement,
                })
                .collect()
        })
        .map_err(|error| error.to_string())
}

fn find_in_ancestors(start: &Path, file_name: &str) -> Option<PathBuf> {
    for ancestor in start.ancestors() {
        let candidate = ancestor.join(file_name);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}
