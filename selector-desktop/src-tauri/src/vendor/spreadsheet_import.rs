use std::fs;
use std::path::Path;

use calamine::{open_workbook_auto, Reader};

use super::field_mapping::{canonical_field_for_label, label_for_field, parse_number};
use super::models::{
    FieldMapping, ImportFailureRow, VendorImportPreview, VendorParameter, VendorPreviewRow,
};
use super::repository::next_id;

pub fn extract_spreadsheet_preview(
    path: &Path,
    source_file: &str,
    source_format: &str,
) -> Result<VendorImportPreview, String> {
    let rows = if is_excel_format(path, source_format) {
        read_excel_rows(path)?
    } else {
        read_text_rows(path)?
    };
    let (preview_rows, failed_rows) = rows_to_preview(rows)?;
    let confidence = average_confidence(&preview_rows);
    let suggested_mappings = suggested_mappings(&preview_rows);
    let warnings = if preview_rows.is_empty() {
        vec!["表格中没有可导入的型号行。".to_string()]
    } else {
        Vec::new()
    };

    Ok(VendorImportPreview {
        job_id: next_id("vendor-import"),
        source_file: source_file.to_string(),
        source_format: source_format.to_string(),
        confidence,
        rows: preview_rows,
        failed_rows,
        suggested_mappings,
        warnings,
    })
}

fn read_excel_rows(path: &Path) -> Result<Vec<Vec<String>>, String> {
    let mut workbook = open_workbook_auto(path).map_err(|error| error.to_string())?;
    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| "Excel 文件没有工作表。".to_string())?;
    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|error| error.to_string())?;
    Ok(range
        .rows()
        .map(|row| row.iter().map(ToString::to_string).collect::<Vec<_>>())
        .collect())
}

fn read_text_rows(path: &Path) -> Result<Vec<Vec<String>>, String> {
    let content = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let delimiter = detect_delimiter(&content);
    Ok(content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| split_record(line, delimiter))
        .collect())
}

fn rows_to_preview(
    rows: Vec<Vec<String>>,
) -> Result<(Vec<VendorPreviewRow>, Vec<ImportFailureRow>), String> {
    let mut iter = rows
        .into_iter()
        .filter(|row| row.iter().any(|cell| !cell.trim().is_empty()));
    let headers = iter
        .next()
        .ok_or_else(|| "表格为空，无法读取表头。".to_string())?;
    let model_index = find_header_index(&headers, &["型号", "model"])
        .ok_or_else(|| "表头中没有型号/model 字段。".to_string())?;
    let brand_index = find_header_index(&headers, &["品牌", "brand"]);
    let series_index = find_header_index(&headers, &["系列", "series"]);

    let mut preview_rows = Vec::new();
    let mut failed_rows = Vec::new();
    for (row_index, row) in iter.enumerate() {
        let model_name = cell(&row, model_index);
        if model_name.trim().is_empty() {
            failed_rows.push(ImportFailureRow {
                row_index: row_index + 2,
                reason: "型号为空".to_string(),
                raw_text: row.join(" | "),
            });
            continue;
        }
        let parameters = headers
            .iter()
            .enumerate()
            .filter(|(index, _)| {
                *index != model_index && Some(*index) != brand_index && Some(*index) != series_index
            })
            .filter_map(|(index, header)| {
                let value = cell(&row, index);
                let number = parse_number(&value)?;
                let unit = unit_from_header(header);
                let field = canonical_field_for_label(header, &unit, index);
                Some(VendorParameter {
                    label: label_for_field(&field),
                    field,
                    value: number,
                    unit,
                    source_field: header.trim().to_string(),
                })
            })
            .collect::<Vec<_>>();
        if parameters.is_empty() {
            failed_rows.push(ImportFailureRow {
                row_index: row_index + 2,
                reason: "未识别可筛选参数".to_string(),
                raw_text: row.join(" | "),
            });
            continue;
        }
        let confidence = if parameters.is_empty() { 0.55 } else { 0.85 };
        preview_rows.push(VendorPreviewRow {
            row_index: row_index + 2,
            model_name,
            brand: brand_index
                .map(|index| cell(&row, index))
                .unwrap_or_default(),
            series: series_index
                .map(|index| cell(&row, index))
                .unwrap_or_default(),
            source_page: None,
            confidence,
            raw_text: row.join(" | "),
            parameters,
        });
    }
    Ok((preview_rows, failed_rows))
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

fn is_excel_format(path: &Path, source_format: &str) -> bool {
    let format = source_format.to_lowercase();
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_lowercase();
    matches!(format.as_str(), "excel" | "xlsx" | "xls" | "xlsm")
        || matches!(extension.as_str(), "xlsx" | "xls" | "xlsm")
}

fn detect_delimiter(content: &str) -> char {
    let header = content.lines().next().unwrap_or_default();
    let tab_count = header.matches('\t').count();
    let comma_count = header.matches(',').count();
    let semicolon_count = header.matches(';').count();
    if tab_count >= comma_count && tab_count >= semicolon_count {
        '\t'
    } else if semicolon_count > comma_count {
        ';'
    } else {
        ','
    }
}

fn split_record(line: &str, delimiter: char) -> Vec<String> {
    let mut cells = Vec::new();
    let mut cell = String::new();
    let mut in_quotes = false;
    for char in line.chars() {
        if char == '"' {
            in_quotes = !in_quotes;
        } else if char == delimiter && !in_quotes {
            cells.push(cell.trim().trim_matches('"').to_string());
            cell.clear();
        } else {
            cell.push(char);
        }
    }
    cells.push(cell.trim().trim_matches('"').to_string());
    cells
}

fn find_header_index(headers: &[String], needles: &[&str]) -> Option<usize> {
    headers.iter().position(|header| {
        let text = header.to_lowercase();
        needles.iter().any(|needle| text.contains(needle))
    })
}

fn cell(row: &[String], index: usize) -> String {
    row.get(index).cloned().unwrap_or_default()
}

fn unit_from_header(header: &str) -> String {
    let start = header.find('(').or_else(|| header.find('（'));
    let end = header.find(')').or_else(|| header.find('）'));
    match (start, end) {
        (Some(start), Some(end)) if end > start => header[start + 1..end].trim().to_string(),
        _ => String::new(),
    }
}

fn average_confidence(rows: &[VendorPreviewRow]) -> f64 {
    if rows.is_empty() {
        return 0.0;
    }
    rows.iter().map(|row| row.confidence).sum::<f64>() / rows.len() as f64
}
