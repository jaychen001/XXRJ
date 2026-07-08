use std::collections::{HashMap, HashSet};

use super::models::{
    ConfirmVendorImportRequest, NewVendorModel, NormalizedParameter, VendorParameter,
};
use super::repository::next_id;

const RESERVED_FIELDS: [&str; 3] = ["modelName", "brand", "series"];

pub fn build_models(request: &ConfirmVendorImportRequest, library_id: &str) -> Vec<NewVendorModel> {
    let accepted_sources = request
        .mappings
        .iter()
        .map(|mapping| mapping.source_field.as_str())
        .collect::<HashSet<_>>();

    request
        .preview
        .rows
        .iter()
        .filter(|row| !row.model_name.trim().is_empty())
        .filter_map(|row| {
            let parameters = row
                .parameters
                .iter()
                .filter(|parameter| accepted_sources.contains(parameter.source_field.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            if parameters.is_empty() {
                return None;
            }
            let normalized_parameters = normalize_parameters(&parameters);
            Some(NewVendorModel {
                id: next_id("vendor-model"),
                library_id: library_id.to_string(),
                model_name: row.model_name.trim().to_string(),
                brand: row.brand.trim().to_string(),
                series: row.series.trim().to_string(),
                parameters,
                normalized_parameters,
                source_page: row.source_page.clone(),
            })
        })
        .collect()
}

pub fn validate_confirm_request(request: &ConfirmVendorImportRequest) -> Result<(), String> {
    if !request.confirmed {
        return Err("字段映射未经人工确认，不能写入厂家型号库。".to_string());
    }
    if request.library_name.trim().is_empty() {
        return Err("样本库名称不能为空。".to_string());
    }
    if request.component_type.trim().is_empty() {
        return Err("部件类型不能为空。".to_string());
    }
    if request.preview.rows.is_empty() {
        return Err("抽取预览为空，不能导入。".to_string());
    }
    let has_model = request
        .mappings
        .iter()
        .any(|mapping| mapping.target_field == "modelName");
    if !has_model {
        return Err("必须确认型号字段映射。".to_string());
    }
    let has_parameter = request
        .mappings
        .iter()
        .any(|mapping| !RESERVED_FIELDS.contains(&mapping.target_field.as_str()));
    if !has_parameter {
        return Err("至少需要确认一个可用于筛选的参数字段。".to_string());
    }
    Ok(())
}

pub fn normalize_parameters(parameters: &[VendorParameter]) -> HashMap<String, NormalizedParameter> {
    let mut normalized = HashMap::new();
    for parameter in parameters {
        let (value, unit) = normalize_value(parameter.value, &parameter.unit);
        normalized.insert(
            parameter.field.clone(),
            NormalizedParameter {
                label: parameter.label.clone(),
                value,
                unit,
                source_field: parameter.source_field.clone(),
            },
        );
    }
    normalized
}

pub fn normalize_value(value: f64, unit: &str) -> (f64, String) {
    let compact = unit.trim().replace('μ', "u").to_lowercase();
    match compact.as_str() {
        "nmm" | "n.mm" | "n-mm" => (value / 1000.0, "Nm".to_string()),
        "kgf.cm" | "kgfcm" => (value * 0.098_066_5, "Nm".to_string()),
        "kw" => (value * 1000.0, "W".to_string()),
        "w" => (value, "W".to_string()),
        "rps" => (value * 60.0, "rpm".to_string()),
        "rpm" | "r/min" | "min-1" => (value, "rpm".to_string()),
        "kn" => (value * 1000.0, "N".to_string()),
        "n" => (value, "N".to_string()),
        "m" => (value * 1000.0, "mm".to_string()),
        "mm" => (value, "mm".to_string()),
        "mpa" => (value * 1000.0, "kPa".to_string()),
        "bar" => (value * 100.0, "kPa".to_string()),
        "kpa" => (value, "kPa".to_string()),
        "m3/min" | "m³/min" => (value * 1000.0, "L/min".to_string()),
        "l/min" | "lpm" => (value, "L/min".to_string()),
        "nm" | "n.m" | "n-m" => (value, "Nm".to_string()),
        _ => (value, unit.trim().to_string()),
    }
}

pub fn canonical_field_for_label(label: &str, unit: &str, fallback_index: usize) -> String {
    let text = label.to_lowercase();
    let unit_text = unit.to_lowercase();
    if contains_any(&text, &["型号", "model"]) {
        "modelName".to_string()
    } else if contains_any(&text, &["品牌", "brand"]) {
        "brand".to_string()
    } else if contains_any(&text, &["系列", "series"]) {
        "series".to_string()
    } else if contains_any(&text, &["扭矩", "力矩", "torque"]) || unit_text.contains("nm") {
        "outputTorque".to_string()
    } else if contains_any(&text, &["转速", "速度", "speed"]) || unit_text.contains("rpm") {
        "requiredSpeed".to_string()
    } else if contains_any(&text, &["功率", "power"]) || unit_text == "kw" || unit_text == "w" {
        "power".to_string()
    } else if contains_any(&text, &["缸径", "bore"]) {
        "bore".to_string()
    } else if contains_any(&text, &["行程", "stroke"]) {
        "stroke".to_string()
    } else if contains_any(&text, &["推力", "负载", "载荷", "force", "load"]) {
        "load".to_string()
    } else if contains_any(&text, &["真空", "vacuum"]) {
        "vacuumPressure".to_string()
    } else if contains_any(&text, &["流量", "flow"]) {
        "flowRate".to_string()
    } else if contains_any(&text, &["动额定", "dynamic"]) {
        "dynamicLoad".to_string()
    } else if contains_any(&text, &["静额定", "static"]) {
        "staticLoad".to_string()
    } else {
        format!("parameter{}", fallback_index + 1)
    }
}

pub fn label_for_field(field: &str) -> String {
    match field {
        "outputTorque" => "输出扭矩",
        "requiredSpeed" => "需求转速",
        "power" => "功率",
        "bore" => "缸径",
        "stroke" => "行程",
        "load" => "负载/推力",
        "vacuumPressure" => "真空压力",
        "flowRate" => "流量",
        "dynamicLoad" => "动额定载荷",
        "staticLoad" => "静额定载荷",
        _ => field,
    }
    .to_string()
}

pub fn parse_number(value: &str) -> Option<f64> {
    let cleaned = value
        .chars()
        .filter(|char| char.is_ascii_digit() || matches!(char, '.' | '-' | '+'))
        .collect::<String>();
    if cleaned.is_empty() || cleaned == "-" || cleaned == "+" {
        return None;
    }
    cleaned.parse::<f64>().ok()
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}
