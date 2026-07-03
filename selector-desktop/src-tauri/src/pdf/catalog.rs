use crate::knowledge::models::NewCatalogItem;

use super::coverage::{compact_pages, matching_pages, CoverageSeed};
use super::snippets::excerpt_for_terms;
use super::text_extract::PdfPageText;

pub(crate) fn build_catalog_items(
    seeds: &[CoverageSeed],
    pages: &[PdfPageText],
) -> Vec<NewCatalogItem> {
    seeds
        .iter()
        .map(|seed| {
            let page_numbers = matching_pages(seed, pages);
            let catalog_page_number =
                find_catalog_page(seed, seeds, pages).or_else(|| page_numbers.first().copied());
            let catalog_page = catalog_page_number.map(|page| format!("P{}", page));
            let excerpt = page_numbers
                .iter()
                .find(|page_number| Some(**page_number) == catalog_page_number)
                .or_else(|| page_numbers.first())
                .and_then(|page_number| pages.iter().find(|page| page.page == *page_number))
                .map(|page| excerpt_for_terms(&page.text, &[seed.chapter.as_str()], 420))
                .unwrap_or_else(|| format!("PDF 正文未匹配章节关键词：{}", seed.chapter));

            NewCatalogItem {
                id: seed.id.clone(),
                chapter: seed.chapter.clone(),
                catalog_page,
                matched_pages: compact_pages(&page_numbers),
                excerpt,
            }
        })
        .collect()
}

fn find_catalog_page(
    seed: &CoverageSeed,
    all_seeds: &[CoverageSeed],
    pages: &[PdfPageText],
) -> Option<u32> {
    pages
        .iter()
        .filter(|page| page.text.contains(&seed.chapter))
        .max_by_key(|page| catalog_score(page, all_seeds))
        .filter(|page| catalog_score(page, all_seeds) >= 3)
        .map(|page| page.page)
}

fn catalog_score(page: &PdfPageText, all_seeds: &[CoverageSeed]) -> usize {
    let chapter_hits = all_seeds
        .iter()
        .filter(|seed| page.text.contains(&seed.chapter))
        .count();
    chapter_hits + usize::from(page.text.contains("目录")) * 3
}
