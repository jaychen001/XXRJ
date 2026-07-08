use std::path::Path;

use super::models::{VendorImportPreview, VendorImportPreviewRequest};
use super::{pdf_import, spreadsheet_import};

pub fn build_preview(request: &VendorImportPreviewRequest) -> Result<VendorImportPreview, String> {
    let path = Path::new(&request.source_file);
    if !path.exists() {
        return Err("样本文件不存在，请检查路径。".to_string());
    }
    let source_format = normalize_source_format(&request.source_format, path);
    match source_format.as_str() {
        "pdf" => pdf_import::extract_pdf_preview(path, &request.source_file),
        "csv" | "tsv" | "excel" | "xlsx" | "xls" | "xlsm" => {
            spreadsheet_import::extract_spreadsheet_preview(
                path,
                &request.source_file,
                &source_format,
            )
        }
        _ => Err("暂不支持该样本格式，请使用 PDF、CSV、TSV 或 Excel。".to_string()),
    }
}

fn normalize_source_format(format: &str, path: &Path) -> String {
    let trimmed = format.trim().to_lowercase();
    if !trimmed.is_empty() {
        return trimmed;
    }
    path.extension()
        .and_then(|value| value.to_str())
        .unwrap_or("pdf")
        .to_lowercase()
}
