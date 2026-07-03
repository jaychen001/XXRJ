use rusqlite::Connection;

use super::models::{NewCatalogItem, NewCoverageRecord, NewKnowledgeEntry, NewParameterCandidate};
use super::repository::KnowledgeRepository;

#[test]
fn repository_searches_entries_and_confirms_parameter_candidate() {
    let connection = Connection::open_in_memory().expect("in-memory database");
    connection
        .execute_batch(include_str!("../../migrations/0001_init.sql"))
        .expect("schema should initialize");
    connection
        .execute_batch(include_str!("../../migrations/0002_pdf_catalog.sql"))
        .expect("catalog schema should initialize");
    let repository = KnowledgeRepository::new(&connection);

    repository
        .upsert_source("root-note", "root.pdf", "根目录非标笔记 PDF")
        .expect("source upsert");
    repository
        .replace_entries(
            "root-note",
            &[
                NewKnowledgeEntry {
                    id: "keyword-inertia-ratio-p8".to_string(),
                    page: Some("P8".to_string()),
                    title: "惯量比 / 来源 P8".to_string(),
                    content: "伺服电机惯量比用于判断负载惯量和电机惯量是否匹配。".to_string(),
                    tags: vec!["PDF知识检索".to_string(), "惯量比".to_string()],
                },
                NewKnowledgeEntry {
                    id: "chapter-motor-p3".to_string(),
                    page: Some("P3".to_string()),
                    title: "电机篇 / 来源 P3".to_string(),
                    content: "电机篇目录项和计算依据。".to_string(),
                    tags: vec!["PDF章节".to_string(), "电机篇".to_string()],
                },
            ],
        )
        .expect("knowledge entries");
    repository
        .replace_catalog(
            "root-note",
            &[NewCatalogItem {
                id: "motor".to_string(),
                chapter: "电机篇".to_string(),
                catalog_page: Some("P3".to_string()),
                matched_pages: Some("P3-P8".to_string()),
                excerpt: "电机篇目录摘录".to_string(),
            }],
        )
        .expect("catalog insert");
    repository
        .upsert_coverage(&[NewCoverageRecord {
            id: "motor".to_string(),
            chapter: "电机篇".to_string(),
            implementation_shape: "计算向导 + 规则选型".to_string(),
            status: "partial".to_string(),
            source_page_range: Some("P3-P8".to_string()),
            notes: "要求：电机计算".to_string(),
        }])
        .expect("coverage insert");

    let entries = repository
        .search_entries("惯量比", 10)
        .expect("search should work");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].page.as_deref(), Some("P8"));
    assert_eq!(repository.list_recent_entries(5).expect("recent").len(), 2);

    let coverage = repository.list_coverage().expect("coverage should list");
    assert_eq!(coverage[0].catalog_page.as_deref(), Some("P3"));
    assert_eq!(coverage[0].knowledge_entry_count, 1);

    repository
        .replace_candidates(&[
            NewParameterCandidate {
                id: "root-pdf-摩擦系数".to_string(),
                name: "摩擦系数".to_string(),
                value: "0.1".to_string(),
                unit: None,
                scenario: "同步带摩擦系数候选，需要人工确认。".to_string(),
                source_page: Some("P24".to_string()),
            },
            NewParameterCandidate {
                id: "root-pdf-负载率".to_string(),
                name: "负载率".to_string(),
                value: "需人工确认".to_string(),
                unit: None,
                scenario: "气缸负载率候选，需要人工确认。".to_string(),
                source_page: Some("P64".to_string()),
            },
        ])
        .expect("candidate insert");

    let candidate = repository
        .update_candidate_status("root-pdf-摩擦系数", "confirmed")
        .expect("candidate update")
        .expect("candidate exists");
    repository
        .insert_confirmed_parameter(&candidate)
        .expect("confirmed parameter insert");

    let saved_count = connection
        .query_row(
            "SELECT COUNT(*) FROM internal_parameters WHERE id = 'parameter-root-pdf-摩擦系数';",
            [],
            |row| row.get::<_, i64>(0),
        )
        .expect("parameter count");
    assert_eq!(saved_count, 1);

    repository
        .update_candidate_status("root-pdf-负载率", "ignored")
        .expect("ignore candidate")
        .expect("candidate exists");
    let ignored_count = connection
        .query_row(
            "SELECT COUNT(*) FROM internal_parameters WHERE id = 'parameter-root-pdf-负载率';",
            [],
            |row| row.get::<_, i64>(0),
        )
        .expect("ignored parameter count");
    assert_eq!(ignored_count, 0);
}
