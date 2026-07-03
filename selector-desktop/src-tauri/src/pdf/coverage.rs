use std::collections::BTreeSet;

use serde::Deserialize;

use crate::knowledge::models::NewCoverageRecord;

use super::text_extract::PdfPageText;

const COVERAGE_MATRIX_JSON: &str = include_str!("../../resources/pdf_coverage_matrix.json");

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CoverageSeed {
    pub id: String,
    pub chapter: String,
    pub implementation_shape: String,
    pub requirement: String,
    pub source: String,
}

pub(crate) fn coverage_seeds() -> Result<Vec<CoverageSeed>, serde_json::Error> {
    serde_json::from_str(COVERAGE_MATRIX_JSON)
}

pub(crate) fn build_coverage_records(
    seeds: &[CoverageSeed],
    pages: &[PdfPageText],
) -> Vec<NewCoverageRecord> {
    seeds
        .iter()
        .map(|seed| {
            let page_numbers = matching_pages(seed, pages);
            NewCoverageRecord {
                id: seed.id.clone(),
                chapter: seed.chapter.clone(),
                implementation_shape: seed.implementation_shape.clone(),
                status: if page_numbers.is_empty() {
                    "planned"
                } else {
                    "partial"
                }
                .to_string(),
                source_page_range: compact_pages(&page_numbers),
                notes: format!("要求：{}；种子来源：{}", seed.requirement, seed.source),
            }
        })
        .collect()
}

pub(super) fn matching_pages(seed: &CoverageSeed, pages: &[PdfPageText]) -> Vec<u32> {
    let terms = chapter_terms(seed);
    pages
        .iter()
        .filter(|page| terms.iter().any(|term| page.text.contains(term)))
        .map(|page| page.page)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn chapter_terms(seed: &CoverageSeed) -> Vec<String> {
    let mut terms = vec![seed.chapter.clone()];
    terms.extend(
        seed.requirement
            .split(['、', '，', ',', '/', ' '])
            .filter(|term| term.chars().count() >= 2)
            .take(6)
            .map(ToString::to_string),
    );
    terms
}

pub(super) fn compact_pages(pages: &[u32]) -> Option<String> {
    match pages {
        [] => None,
        [single] => Some(format!("P{}", single)),
        [first, second] if *second == *first + 1 => Some(format!("P{}-P{}", first, second)),
        [first, .., last] => Some(format!("P{}-P{}", first, last)),
    }
}
