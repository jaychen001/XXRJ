use std::collections::BTreeSet;

use serde::Deserialize;

use crate::knowledge::models::NewCoverageRecord;

use super::text_extract::PdfPageText;

const COVERAGE_MATRIX_JSON: &str = include_str!("../../resources/pdf_coverage_matrix.json");
const IMPLEMENTED_CHAPTER_IDS: &[&str] = &[
    "motor",
    "ball-screw",
    "timing-belt",
    "v-belt",
    "gear",
    "chain",
    "reducer",
    "linear-module",
    "cam-indexer",
    "brake-clutch",
    "pneumatic-actuator",
    "pneumatic-control",
    "linear-guide",
    "linear-bearing",
    "rolling-bearing",
    "coupling",
    "robot",
    "cable-chain",
    "sensor",
    "material",
    "machining",
    "heat-surface",
    "hardware",
];

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
                status: coverage_status(&seed.id, !page_numbers.is_empty()).to_string(),
                source_page_range: implemented_source_page(&seed.id)
                    .map(ToString::to_string)
                    .or_else(|| compact_pages(&page_numbers)),
                notes: format!("要求：{}；种子来源：{}", seed.requirement, seed.source),
            }
        })
        .collect()
}

pub(crate) fn coverage_status(id: &str, has_pages: bool) -> &'static str {
    if IMPLEMENTED_CHAPTER_IDS.contains(&id) {
        "done"
    } else if has_pages {
        "partial"
    } else {
        "planned"
    }
}

pub(crate) fn implemented_source_page(id: &str) -> Option<&'static str> {
    match id {
        "motor" => Some("PDF P4 / 文档页 1"),
        "ball-screw" => Some("PDF P25 / 文档页 22"),
        "timing-belt" => Some("PDF P34 / 文档页 31"),
        "v-belt" => Some("PDF P40 / 文档页 37"),
        "gear" => Some("PDF P44 / 文档页 41"),
        "chain" => Some("PDF P49 / 文档页 46"),
        "reducer" => Some("PDF P54 / 文档页 51"),
        "linear-module" => Some("PDF P57 / 文档页 54"),
        "cam-indexer" => Some("PDF P59 / 文档页 56"),
        "brake-clutch" => Some("PDF P65 / 文档页 62"),
        "pneumatic-actuator" => Some("PDF P69 / 文档页 66"),
        "pneumatic-control" => Some("PDF P88 / 文档页 85"),
        "linear-guide" => Some("PDF P103 / 文档页 100"),
        "linear-bearing" => Some("PDF P104 / 文档页 101"),
        "rolling-bearing" => Some("PDF P109 / 文档页 106"),
        "coupling" => Some("PDF P116 / 文档页 113"),
        "robot" => Some("PDF P67 / 文档页 64"),
        "cable-chain" => Some("PDF P121 / 文档页 118"),
        "sensor" => Some("PDF P123 / 文档页 120"),
        "material" => Some("PDF P135 / 文档页 132"),
        "machining" => Some("PDF P139 / 文档页 136"),
        "heat-surface" => Some("PDF P141 / 文档页 138"),
        "hardware" => Some("PDF P146 / 文档页 143"),
        _ => None,
    }
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
