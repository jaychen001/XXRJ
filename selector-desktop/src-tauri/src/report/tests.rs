use std::fs;

use super::models::ReportPayload;
use super::pdf_report::build_pdf_bytes;

#[test]
fn pdf_writer_builds_valid_pdf_bytes() {
    let bytes = build_pdf_bytes(&[
        "非标选型计算报告".to_string(),
        "输出扭矩 1.280 Nm".to_string(),
    ]);
    assert!(bytes.starts_with(b"%PDF-1.4"));
    assert!(bytes.ends_with(b"%%EOF\n"));
}

#[test]
fn excel_writer_creates_xlsx_file() {
    let payload = ReportPayload {
        run_id: None,
        case_id: None,
        case_name: "测试报告".to_string(),
        notes: String::new(),
        request: crate::engine::models::CalculationRequest {
            module_id: "timing-belt-basic".to_string(),
            fields: vec![],
            safety_factor: Some(1.5),
            safety_factor_confirmed: true,
        },
        result: crate::engine::calculate_request(&test_request()).expect("calculation"),
        candidates: Vec::new(),
        final_model_name: None,
    };
    let path = std::env::temp_dir().join("selector-report-test.xlsx");
    super::excel_report::write_excel(&path, &payload).expect("xlsx export");
    let metadata = fs::metadata(&path).expect("xlsx metadata");
    assert!(metadata.len() > 1000);
    fs::remove_file(path).ok();
}

fn test_request() -> crate::engine::models::CalculationRequest {
    crate::engine::models::CalculationRequest {
        module_id: "timing-belt-basic".to_string(),
        safety_factor: Some(1.5),
        safety_factor_confirmed: true,
        fields: vec![
            input("loadMass", 5.0, "kg"),
            input("frictionCoefficient", 0.1, "ratio"),
            input("targetSpeed", 500.0, "mm/s"),
            input("accelerationTime", 0.3, "s"),
            input("pulleyTeeth", 20.0, "teeth"),
            input("toothPitch", 5.0, "mm"),
            input("efficiency", 0.9, "ratio"),
        ],
    }
}

fn input(id: &str, value: f64, unit: &str) -> crate::engine::models::FieldInput {
    crate::engine::models::FieldInput {
        id: id.to_string(),
        value,
        unit: unit.to_string(),
    }
}
