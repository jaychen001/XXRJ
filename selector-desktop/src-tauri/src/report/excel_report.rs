use std::path::Path;

use rust_xlsxwriter::Workbook;

use super::content::report_lines;
use super::models::ReportPayload;

pub fn write_excel(path: &Path, payload: &ReportPayload) -> Result<(), String> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet
        .set_name("选型报告")
        .map_err(|error| error.to_string())?;

    for (row, line) in report_lines(payload).iter().enumerate() {
        worksheet
            .write_string(row as u32, 0, line)
            .map_err(|error| error.to_string())?;
    }
    worksheet.autofit();
    workbook.save(path).map_err(|error| error.to_string())
}
