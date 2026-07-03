use super::catalog::build_catalog_items;
use super::coverage::{build_coverage_records, coverage_seeds};
use super::knowledge_entries::build_knowledge_entries;
use super::parameters::build_parameter_candidates;
use super::root_note_ingest::find_root_pdf;
use super::text_extract::extract_pages;

#[test]
fn root_pdf_extracts_phase2_seed_data() {
    let pdf_path = find_root_pdf().expect("root PDF should exist in repository");
    let pages = extract_pages(&pdf_path).expect("root PDF should be text extractable");
    let seeds = coverage_seeds().expect("coverage seed should be valid JSON");
    let coverage = build_coverage_records(&seeds, &pages);
    let catalog = build_catalog_items(&seeds, &pages);
    let entries = build_knowledge_entries(&seeds, &pages);
    let candidates = build_parameter_candidates(&pages);
    let combined_text = pages
        .iter()
        .map(|page| page.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    assert_eq!(seeds.len(), 23);
    assert_eq!(catalog.len(), 23);
    assert_eq!(coverage.len(), 23);
    assert!(pages.len() > 10);
    assert!(entries.iter().any(|entry| entry.title.contains("惯量比")));
    assert!(entries.iter().any(|entry| entry.title.contains("摩擦系数")));
    assert!(entries.iter().any(|entry| entry.title.contains("负载率")));
    assert!(catalog.iter().any(|entry| entry.catalog_page.is_some()));
    assert!(combined_text.contains("摩擦系数"));
    assert!(combined_text.contains("负载率"));
    assert!(candidates
        .iter()
        .any(|candidate| candidate.name == "摩擦系数"));
    assert!(candidates
        .iter()
        .any(|candidate| candidate.name == "负载率"));
}
