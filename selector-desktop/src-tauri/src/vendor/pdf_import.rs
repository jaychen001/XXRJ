use std::path::Path;

use super::field_mapping::{
    canonical_field_for_label, label_for_field, parse_number, normalize_parameters,
};
use super::models::{FieldMapping, VendorImportPreview, VendorParameter, VendorPreviewRow};
use super::repository::next_id;

pub fn extract_pdf_preview(path: &Path, source_file: &str) -> Result<VendorImportPreview, String> {
    let pages = pdf_extract::extract_text_by_pages(path).map_err(|error| error.to_string())?;
    let mut rows = Vec::new();

    for (page_index, page_text) in pages.iter().enumerate() {
        for line in page_text.lines() {
            let normalized = normalize_line(line);
            if normalized.len() < 6 {
                continue;
            }
            let Some(model_name) = find_model_token(&normalized) else {
                continue;
            };
            let parameters = extract_line_parameters(&normalized);
            let confidence = confidence_for_row(!parameters.is_empty(), parameters.len());
            rows.push(VendorPreviewRow {
                row_index: rows.len() + 1,
                model_name,
                brand: String::new(),
                series: String::new(),
                source_page: Some(format!("P{}", page_index + 1)),
                confidence,
                raw_text: normalized,
                parameters,
            });
            if rows.len() >= 80 {
                break;
            }
        }
        if rows.len() >= 80 {
            break;
        }
    }

    let warnings = if rows.is_empty() {
        vec!["未从 PDF 中识别到型号行；如果是扫描版 PDF，需要先 OCR 或转成 CSV。".to_string()]
    } else {
        Vec::new()
    };
    let confidence = average_confidence(&rows);
    let suggested_mappings = suggested_mappings(&rows);

    Ok(VendorImportPreview {
        job_id: next_id("vendor-import"),
        source_file: source_file.to_string(),
        source_format: "pdf".to_string(),
        confidence,
        rows,
        failed_rows: Vec::new(),
        suggested_mappings,
        warnings,
    })
}

fn extract_line_parameters(line: &str) -> Vec<VendorParameter> {
    let parts = line.split_whitespace().collect::<Vec<_>>();
    let mut parameters = Vec::new();
    for (index, part) in parts.iter().enumerate() {
        let Some(value) = parse_number(part) else {
            continue;
        };
        if is_model_token(part) {
            continue;
        }
        let unit = parts
            .get(index + 1)
            .map(|value| sanitize_unit(value))
            .filter(|value| !value.is_empty())
            .unwrap_or_default();
        let field = canonical_field_for_label(line, &unit, parameters.len());
        let source_field = format!("pdf-{}-{}", field, parameters.len() + 1);
        parameters.push(VendorParameter {
            label: label_for_field(&field),
            field,
            value,
            unit,
            source_field,
        });
        if parameters.len() >= 8 {
            break;
        }
    }
    normalize_duplicate_fields(parameters)
}

fn normalize_duplicate_fields(parameters: Vec<VendorParameter>) -> Vec<VendorParameter> {
    let mut counts = std::collections::HashMap::<String, usize>::new();
    parameters
        .into_iter()
        .map(|mut parameter| {
            let count = counts.entry(parameter.field.clone()).or_insert(0);
            if *count > 0 {
                parameter.field = format!("{}{}", parameter.field, *count + 1);
            }
            *count += 1;
            parameter
        })
        .collect()
}

fn suggested_mappings(rows: &[VendorPreviewRow]) -> Vec<FieldMapping> {
    let mut mappings = vec![FieldMapping {
        source_field: "modelName".to_string(),
        target_field: "modelName".to_string(),
        unit: None,
    }];
    let mut seen = std::collections::HashSet::new();
    for row in rows {
        for parameter in &row.parameters {
            if seen.insert(parameter.source_field.clone()) {
                mappings.push(FieldMapping {
                    source_field: parameter.source_field.clone(),
                    target_field: parameter.field.clone(),
                    unit: Some(parameter.unit.clone()),
                });
            }
        }
    }
    mappings
}

fn find_model_token(line: &str) -> Option<String> {
    line.split_whitespace()
        .map(clean_token)
        .find(|token| is_model_token(token))
}

fn is_model_token(token: &str) -> bool {
    let has_letter = token.chars().any(|char| char.is_ascii_alphabetic());
    let has_digit = token.chars().any(|char| char.is_ascii_digit());
    let long_enough = token.chars().count() >= 4;
    has_letter && has_digit && long_enough && !token.starts_with('P')
}

fn clean_token(value: &str) -> String {
    value
        .trim_matches(|char: char| {
            matches!(char, ',' | ';' | ':' | '，' | '；' | '：' | '(' | ')' | '[' | ']')
        })
        .to_string()
}

fn sanitize_unit(value: &str) -> String {
    value
        .trim_matches(|char: char| {
            char.is_ascii_punctuation() || matches!(char, '，' | '；' | '：' | '。')
        })
        .chars()
        .filter(|char| char.is_ascii_alphabetic() || matches!(char, '/' | '-' | '.' | '³'))
        .collect()
}

fn normalize_line(value: &str) -> String {
    value
        .split_whitespace()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn confidence_for_row(has_parameters: bool, parameter_count: usize) -> f64 {
    let mut confidence = 0.55;
    if has_parameters {
        confidence += 0.15;
    }
    confidence += (parameter_count.min(4) as f64) * 0.05;
    confidence.min(0.9)
}

fn average_confidence(rows: &[VendorPreviewRow]) -> f64 {
    if rows.is_empty() {
        return 0.0;
    }
    rows.iter().map(|row| row.confidence).sum::<f64>() / rows.len() as f64
}

#[allow(dead_code)]
fn _normalized_parameter_count(row: &VendorPreviewRow) -> usize {
    normalize_parameters(&row.parameters).len()
}
