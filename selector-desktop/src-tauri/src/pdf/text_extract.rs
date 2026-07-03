use std::path::Path;

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfPageText {
    pub page: u32,
    pub text: String,
}

#[derive(Debug, Error)]
pub enum PdfTextError {
    #[error("PDF 文本抽取失败：{0}")]
    Extract(String),
    #[error("PDF 可抽取文本为空，可能是扫描版 PDF")]
    EmptyText,
}

pub fn extract_pages(path: &Path) -> Result<Vec<PdfPageText>, PdfTextError> {
    let raw_pages = pdf_extract::extract_text_by_pages(path)
        .map_err(|error| PdfTextError::Extract(error.to_string()))?;
    let pages = raw_pages
        .into_iter()
        .enumerate()
        .map(|(index, text)| PdfPageText {
            page: index as u32 + 1,
            text: normalize_text(&text),
        })
        .collect::<Vec<_>>();

    let total_chars = pages
        .iter()
        .map(|page| page.text.chars().count())
        .sum::<usize>();
    if total_chars < 100 {
        return Err(PdfTextError::EmptyText);
    }

    Ok(pages)
}

pub fn normalize_text(value: &str) -> String {
    value
        .split_whitespace()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}
