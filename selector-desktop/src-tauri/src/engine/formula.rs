use std::collections::HashMap;

use serde_json::json;

use super::models::{
    CalculationRequest, CalculationResult, FieldError, FormulaStep, RequirementParameter, RiskItem,
};
use super::modules::{get_module_definition, TIMING_BELT_MODULE_ID};
use super::rules;
use super::safety_factor;
use super::units;

pub fn run_calculation(request: &CalculationRequest) -> Result<CalculationResult, FieldError> {
    match request.module_id.as_str() {
        TIMING_BELT_MODULE_ID => run_timing_belt(request),
        _ => Err(FieldError {
            field_id: "moduleId".to_string(),
            message: "该模块尚未实现计算公式".to_string(),
        }),
    }
}

fn run_timing_belt(request: &CalculationRequest) -> Result<CalculationResult, FieldError> {
    let module = get_module_definition(TIMING_BELT_MODULE_ID).expect("module exists");
    let fields = request
        .fields
        .iter()
        .map(|field| (field.id.as_str(), (field.value, field.unit.as_str())))
        .collect::<HashMap<_, _>>();
    let safety_factor =
        safety_factor::validate(request.safety_factor, request.safety_factor_confirmed)?;

    let mass = positive(&fields, "loadMass")?;
    let friction = positive_or_zero(&fields, "frictionCoefficient")?;
    let target_speed = positive(&fields, "targetSpeed")?;
    let target_speed_unit = unit(&fields, "targetSpeed")?;
    let speed_mm_s = convert(target_speed, target_speed_unit, "mm/s", "targetSpeed")?;
    let speed_m_s = convert(target_speed, target_speed_unit, "m/s", "targetSpeed")?;
    let accel_time_raw = positive(&fields, "accelerationTime")?;
    let accel_time = convert(
        accel_time_raw,
        unit(&fields, "accelerationTime")?,
        "s",
        "accelerationTime",
    )?;
    let pulley_teeth = positive(&fields, "pulleyTeeth")?;
    let tooth_pitch_raw = positive(&fields, "toothPitch")?;
    let tooth_pitch_mm = convert(
        tooth_pitch_raw,
        unit(&fields, "toothPitch")?,
        "mm",
        "toothPitch",
    )?;
    let efficiency = positive(&fields, "efficiency")?;
    if efficiency > 1.0 {
        return field_error("efficiency", "传动效率必须使用 0-1 之间的小数");
    }

    let friction_force = mass * 9.80665 * friction;
    let acceleration = speed_m_s / accel_time;
    let acceleration_force = mass * acceleration;
    let total_force = (friction_force + acceleration_force) * safety_factor / efficiency;
    let pitch_diameter_m = pulley_teeth * tooth_pitch_mm / std::f64::consts::PI / 1000.0;
    let torque_nm = total_force * pitch_diameter_m / 2.0;
    let circumference_m = pulley_teeth * tooth_pitch_mm / 1000.0;
    let rpm = speed_m_s / circumference_m * 60.0;
    let power_w = total_force * speed_m_s;
    let source = module.source_page.clone();
    let rules = rules::timing_belt_rules(speed_mm_s, efficiency, safety_factor, &source);
    let mut risks = timing_belt_risks(safety_factor, speed_mm_s, efficiency, &source);

    if risks.is_empty() {
        risks.push(RiskItem {
            level: "success".to_string(),
            message: "未发现基础速度、效率或安全系数风险。".to_string(),
            field_id: None,
            source: source.clone(),
        });
    }
    Ok(CalculationResult {
        module_id: module.id,
        module_name: module.name,
        formula_version: "timing-belt-basic@0.1.0".to_string(),
        summary: format!("输出扭矩 {} Nm，需求转速 {} rpm", fmt(torque_nm), fmt(rpm)),
        conclusion: format!(
            "按安全系数 {} 计算，驱动端至少需要 {} Nm、{} rpm，功率约 {} W。",
            fmt(safety_factor),
            fmt(torque_nm),
            fmt(rpm),
            fmt(power_w)
        ),
        steps: vec![
            step(
                "摩擦力",
                "Ff = m * g * μ",
                format!("{mass} * 9.80665 * {friction}"),
                friction_force,
                "N",
                &source,
            ),
            step(
                "加速度",
                "a = v / t",
                format!("{} / {}", fmt(speed_m_s), accel_time),
                acceleration,
                "m/s²",
                &source,
            ),
            step(
                "加速力",
                "Fa = m * a",
                format!("{mass} * {}", fmt(acceleration)),
                acceleration_force,
                "N",
                &source,
            ),
            step(
                "等效推力",
                "F = (Ff + Fa) * K / η",
                format!(
                    "({} + {}) * {} / {}",
                    fmt(friction_force),
                    fmt(acceleration_force),
                    fmt(safety_factor),
                    fmt(efficiency)
                ),
                total_force,
                "N",
                &source,
            ),
            step(
                "输出扭矩",
                "T = F * Dp / 2",
                format!("{} * {} / 2", fmt(total_force), fmt(pitch_diameter_m)),
                torque_nm,
                "Nm",
                &source,
            ),
            step(
                "需求转速",
                "n = v / (z * p) * 60",
                format!("{} / {} * 60", fmt(speed_m_s), fmt(circumference_m)),
                rpm,
                "rpm",
                &source,
            ),
        ],
        rules,
        risks,
        requirements: vec![
            requirement("outputTorque", "输出扭矩", torque_nm, "Nm"),
            requirement("requiredSpeed", "需求转速", rpm, "rpm"),
            requirement("power", "估算功率", power_w, "W"),
        ],
        source_pages: vec![source],
        input_snapshot: serde_json::to_value(request).unwrap_or_else(|_| json!({})),
        defaults_snapshot: json!({"gravity": {"value": 9.80665, "unit": "m/s²", "source": "系统默认"}}),
    })
}

fn timing_belt_risks(
    safety_factor: f64,
    speed_mm_s: f64,
    efficiency: f64,
    source: &str,
) -> Vec<RiskItem> {
    let mut risks = Vec::new();
    if let Some(risk) = safety_factor::risk(safety_factor, source) {
        risks.push(risk);
    }
    if speed_mm_s > 2000.0 {
        risks.push(RiskItem {
            level: "warning".to_string(),
            message: "目标速度超过 2000 mm/s，建议复核同步带齿形、导轨阻力和张紧方式。".to_string(),
            field_id: Some("targetSpeed".to_string()),
            source: source.to_string(),
        });
    }
    if efficiency < 0.7 {
        risks.push(RiskItem {
            level: "warning".to_string(),
            message: "传动效率低于 0.7，扭矩需求会明显放大。".to_string(),
            field_id: Some("efficiency".to_string()),
            source: source.to_string(),
        });
    }
    risks
}

fn positive(fields: &HashMap<&str, (f64, &str)>, field_id: &str) -> Result<f64, FieldError> {
    let value = fields
        .get(field_id)
        .map(|(value, _)| *value)
        .ok_or_else(|| FieldError {
            field_id: field_id.to_string(),
            message: "必填字段不能为空".to_string(),
        })?;
    if value <= 0.0 {
        return field_error(field_id, "字段值必须大于 0");
    }
    Ok(value)
}

fn positive_or_zero(
    fields: &HashMap<&str, (f64, &str)>,
    field_id: &str,
) -> Result<f64, FieldError> {
    let value = fields
        .get(field_id)
        .map(|(value, _)| *value)
        .ok_or_else(|| FieldError {
            field_id: field_id.to_string(),
            message: "必填字段不能为空".to_string(),
        })?;
    if value < 0.0 {
        return field_error(field_id, "字段值不能小于 0");
    }
    Ok(value)
}

fn unit<'a>(
    fields: &'a HashMap<&str, (f64, &'a str)>,
    field_id: &str,
) -> Result<&'a str, FieldError> {
    fields
        .get(field_id)
        .map(|(_, unit)| *unit)
        .ok_or_else(|| FieldError {
            field_id: field_id.to_string(),
            message: "字段单位不能为空".to_string(),
        })
}

fn convert(value: f64, from: &str, to: &str, field_id: &str) -> Result<f64, FieldError> {
    units::convert(value, from, to).map_err(|error| FieldError {
        field_id: field_id.to_string(),
        message: error.to_string(),
    })
}

fn field_error<T>(field_id: &str, message: &str) -> Result<T, FieldError> {
    Err(FieldError {
        field_id: field_id.to_string(),
        message: message.to_string(),
    })
}

fn step(
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

fn requirement(id: &str, label: &str, value: f64, unit: &str) -> RequirementParameter {
    RequirementParameter {
        id: id.to_string(),
        label: label.to_string(),
        value,
        unit: unit.to_string(),
    }
}

fn fmt(value: f64) -> String {
    format!("{value:.3}")
}
