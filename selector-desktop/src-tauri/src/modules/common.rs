use std::collections::HashMap;

use serde_json::{json, Value};

use crate::engine::models::{
    CalculationRequest, CalculationResult, FieldError, FormulaStep, ModuleDefinition, ModuleField,
    RequirementParameter, RiskItem, RuleDecision,
};
use crate::engine::{safety_factor, units};

pub type FieldMap<'a> = HashMap<&'a str, (f64, &'a str)>;

pub fn fields_map(request: &CalculationRequest) -> FieldMap<'_> {
    request
        .fields
        .iter()
        .map(|field| (field.id.as_str(), (field.value, field.unit.as_str())))
        .collect()
}

pub fn positive(fields: &FieldMap<'_>, field_id: &str) -> Result<f64, FieldError> {
    let value = raw_value(fields, field_id)?;
    if value <= 0.0 {
        return field_error(field_id, "字段值必须大于 0");
    }
    Ok(value)
}

pub fn positive_or_zero(fields: &FieldMap<'_>, field_id: &str) -> Result<f64, FieldError> {
    let value = raw_value(fields, field_id)?;
    if value < 0.0 {
        return field_error(field_id, "字段值不能小于 0");
    }
    Ok(value)
}

pub fn efficiency(fields: &FieldMap<'_>, field_id: &str) -> Result<f64, FieldError> {
    let value = positive(fields, field_id)?;
    if value > 1.0 {
        return field_error(field_id, "效率必须使用 0-1 之间的小数");
    }
    Ok(value)
}

pub fn unit<'a>(fields: &'a FieldMap<'a>, field_id: &str) -> Result<&'a str, FieldError> {
    fields
        .get(field_id)
        .map(|(_, unit)| *unit)
        .ok_or_else(|| FieldError {
            field_id: field_id.to_string(),
            message: "字段单位不能为空".to_string(),
        })
}

pub fn convert(value: f64, from: &str, to: &str, field_id: &str) -> Result<f64, FieldError> {
    units::convert(value, from, to).map_err(|error| FieldError {
        field_id: field_id.to_string(),
        message: error.to_string(),
    })
}

pub fn safety_factor(request: &CalculationRequest) -> Result<f64, FieldError> {
    safety_factor::validate(request.safety_factor, request.safety_factor_confirmed)
}

pub fn safety_risk(value: f64, source: &str) -> Vec<RiskItem> {
    safety_factor::risk(value, source).into_iter().collect()
}

pub fn result(
    module: ModuleDefinition,
    request: &CalculationRequest,
    formula_version: &str,
    summary: String,
    conclusion: String,
    steps: Vec<FormulaStep>,
    rules: Vec<RuleDecision>,
    mut risks: Vec<RiskItem>,
    requirements: Vec<RequirementParameter>,
) -> CalculationResult {
    if risks.is_empty() {
        risks.push(risk(
            "success",
            "未发现基础计算风险。",
            None,
            &module.source_page,
        ));
    }
    CalculationResult {
        module_id: module.id,
        module_name: module.name,
        formula_version: formula_version.to_string(),
        summary,
        conclusion,
        steps,
        rules,
        risks,
        requirements,
        source_pages: vec![module.source_page],
        input_snapshot: serde_json::to_value(request).unwrap_or_else(|_| json!({})),
        defaults_snapshot: defaults_snapshot(),
    }
}

pub fn step(
    label: &str,
    formula: &str,
    substitution: String,
    value: f64,
    unit: &str,
    source: &str,
) -> FormulaStep {
    FormulaStep {
        label: label.to_string(),
        formula: formula.to_string(),
        substitution,
        result: fmt(value),
        unit: unit.to_string(),
        source: source.to_string(),
    }
}

pub fn requirement(id: &str, label: &str, value: f64, unit: &str) -> RequirementParameter {
    RequirementParameter {
        id: id.to_string(),
        label: label.to_string(),
        value,
        unit: unit.to_string(),
    }
}

pub fn rule(
    id: &str,
    label: &str,
    recommendation: String,
    basis: String,
    risk: &str,
    source: &str,
) -> RuleDecision {
    RuleDecision {
        id: id.to_string(),
        label: label.to_string(),
        recommendation,
        basis,
        risk: risk.to_string(),
        source: source.to_string(),
    }
}

pub fn risk(level: &str, message: &str, field_id: Option<&str>, source: &str) -> RiskItem {
    RiskItem {
        level: level.to_string(),
        message: message.to_string(),
        field_id: field_id.map(ToString::to_string),
        source: source.to_string(),
    }
}

pub fn field(
    id: &str,
    label: &str,
    unit: &str,
    min: f64,
    default_value: f64,
    helper: &str,
    source: &str,
) -> ModuleField {
    field_with_units(id, label, unit, &[unit], min, default_value, helper, source)
}

pub fn field_with_units(
    id: &str,
    label: &str,
    unit: &str,
    unit_options: &[&str],
    min: f64,
    default_value: f64,
    helper: &str,
    source: &str,
) -> ModuleField {
    ModuleField {
        id: id.to_string(),
        label: label.to_string(),
        unit: unit.to_string(),
        unit_options: unit_options
            .iter()
            .map(|item| (*item).to_string())
            .collect(),
        required: true,
        min: Some(min),
        default_value: Some(default_value),
        helper: helper.to_string(),
        source: source.to_string(),
    }
}

pub fn fmt(value: f64) -> String {
    format!("{value:.3}")
}

fn raw_value(fields: &FieldMap<'_>, field_id: &str) -> Result<f64, FieldError> {
    fields
        .get(field_id)
        .map(|(value, _)| *value)
        .ok_or_else(|| FieldError {
            field_id: field_id.to_string(),
            message: "必填字段不能为空".to_string(),
        })
}

fn field_error<T>(field_id: &str, message: &str) -> Result<T, FieldError> {
    Err(FieldError {
        field_id: field_id.to_string(),
        message: message.to_string(),
    })
}

fn defaults_snapshot() -> Value {
    json!({"gravity": {"value": 9.80665, "unit": "m/s²", "source": "系统默认"}})
}
