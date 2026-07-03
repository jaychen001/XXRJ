use crate::knowledge::models::NewKnowledgeEntry;

use super::coverage::{matching_pages, CoverageSeed};
use super::snippets::excerpt_for_terms;
use super::text_extract::PdfPageText;

const KNOWLEDGE_KEYWORDS: [(&str, [&str; 2]); 3] = [
    ("惯量比", ["惯量比", "惯量"]),
    ("摩擦系数", ["摩擦系数", "摩擦"]),
    ("负载率", ["负载率", "负载"]),
];

pub(crate) fn build_knowledge_entries(
    seeds: &[CoverageSeed],
    pages: &[PdfPageText],
) -> Vec<NewKnowledgeEntry> {
    let mut entries = Vec::new();

    for seed in seeds {
        for page_number in matching_pages(seed, pages).into_iter().take(3) {
            if let Some(page) = pages.iter().find(|page| page.page == page_number) {
                entries.push(NewKnowledgeEntry {
                    id: format!("chapter-{}-p{}", seed.id, page.page),
                    page: Some(format!("P{}", page.page)),
                    title: format!("{} / 来源 P{}", seed.chapter, page.page),
                    content: excerpt_for_terms(&page.text, &[seed.chapter.as_str()], 700),
                    tags: vec!["PDF章节".to_string(), seed.chapter.clone()],
                });
            }
        }
    }

    for (label, terms) in KNOWLEDGE_KEYWORDS {
        for page in pages
            .iter()
            .filter(|page| terms.iter().any(|term| page.text.contains(term)))
            .take(6)
        {
            entries.push(NewKnowledgeEntry {
                id: format!("keyword-{}-p{}", label, page.page),
                page: Some(format!("P{}", page.page)),
                title: format!("{} / 来源 P{}", label, page.page),
                content: excerpt_for_terms(&page.text, &terms, 900),
                tags: vec!["PDF知识检索".to_string(), label.to_string()],
            });
        }
    }

    entries
}
