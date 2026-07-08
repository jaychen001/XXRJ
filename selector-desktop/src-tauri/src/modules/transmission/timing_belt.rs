use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "timing-belt-basic";
const SOURCE: &str = "PDF P34 / 文档页 31 / 同步带";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "同步带基础计算".to_string(),
        category: "传动".to_string(),
        description: "用于验证负载、速度、同步轮参数到扭矩和转速的过程输出。".to_string(),
        source_chapter: "同步带".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field(
                "loadMass",
                "负载质量",
                "kg",
                0.0,
                5.0,
                "移动负载质量",
                SOURCE,
            ),
            common::field(
                "frictionCoefficient",
                "摩擦系数",
                "ratio",
                0.0,
                0.1,
                "导向面或机构摩擦系数",
                SOURCE,
            ),
            common::field_with_units(
                "targetSpeed",
                "目标速度",
                "mm/s",
                &["mm/s", "m/s"],
                0.0,
                500.0,
                "机构目标线速度",
                SOURCE,
            ),
            common::field_with_units(
                "accelerationTime",
                "加速时间",
                "s",
                &["s", "min"],
                0.001,
                0.3,
                "从静止到目标速度的时间",
                SOURCE,
            ),
            common::field(
                "pulleyTeeth",
                "同步轮齿数",
                "teeth",
                1.0,
                20.0,
                "驱动轮齿数",
                SOURCE,
            ),
            common::field_with_units(
                "toothPitch",
                "齿距",
                "mm",
                &["mm", "m"],
                0.001,
                5.0,
                "同步带齿距",
                SOURCE,
            ),
            common::field(
                "efficiency",
                "传动效率",
                "ratio",
                0.01,
                0.9,
                "0-1 之间的小数",
                SOURCE,
            ),
        ],
    }
}

pub fn calculate(request: &CalculationRequest) -> Result<CalculationResult, FieldError> {
    let module = definition();
    let source = module.source_page.clone();
    let fields = common::fields_map(request);
    let safety_factor = common::safety_factor(request)?;
    let mass = common::positive(&fields, "loadMass")?;
    let friction = common::positive_or_zero(&fields, "frictionCoefficient")?;
    let speed_raw = common::positive(&fields, "targetSpeed")?;
    let speed_mm_s = common::convert(
        speed_raw,
        common::unit(&fields, "targetSpeed")?,
        "mm/s",
        "targetSpeed",
    )?;
    let speed_m_s = common::convert(
        speed_raw,
        common::unit(&fields, "targetSpeed")?,
        "m/s",
        "targetSpeed",
    )?;
    let accel_time = common::convert(
        common::positive(&fields, "accelerationTime")?,
        common::unit(&fields, "accelerationTime")?,
        "s",
        "accelerationTime",
    )?;
    let pulley_teeth = common::positive(&fields, "pulleyTeeth")?;
    let tooth_pitch_mm = common::convert(
        common::positive(&fields, "toothPitch")?,
        common::unit(&fields, "toothPitch")?,
        "mm",
        "toothPitch",
    )?;
    let efficiency = common::efficiency(&fields, "efficiency")?;

    let friction_force = mass * 9.80665 * friction;
    let acceleration = speed_m_s / accel_time;
    let acceleration_force = mass * acceleration;
    let total_force = (friction_force + acceleration_force) * safety_factor / efficiency;
    let pitch_diameter_m = pulley_teeth * tooth_pitch_mm / std::f64::consts::PI / 1000.0;
    let torque_nm = total_force * pitch_diameter_m / 2.0;
    let circumference_m = pulley_teeth * tooth_pitch_mm / 1000.0;
    let rpm = speed_m_s / circumference_m * 60.0;
    let power_w = total_force * speed_m_s;
    let mut risks = common::safety_risk(safety_factor, &source);
    if speed_mm_s > 2000.0 {
        risks.push(common::risk(
            "warning",
            "目标速度超过 2000 mm/s，建议复核同步带齿形、导轨阻力和张紧方式。",
            Some("targetSpeed"),
            &source,
        ));
    }
    if efficiency < 0.7 {
        risks.push(common::risk(
            "warning",
            "传动效率低于 0.7，扭矩需求会明显放大。",
            Some("efficiency"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "timing-belt-basic@0.1.0",
        format!(
            "输出扭矩 {} Nm，需求转速 {} rpm",
            common::fmt(torque_nm),
            common::fmt(rpm)
        ),
        format!(
            "按安全系数 {} 计算，驱动端至少需要 {} Nm、{} rpm，功率约 {} W。",
            common::fmt(safety_factor),
            common::fmt(torque_nm),
            common::fmt(rpm),
            common::fmt(power_w)
        ),
        vec![
            common::step(
                "摩擦力",
                "Ff = m * g * μ",
                format!("{mass} * 9.80665 * {friction}"),
                friction_force,
                "N",
                &source,
            ),
            common::step(
                "加速度",
                "a = v / t",
                format!("{} / {}", common::fmt(speed_m_s), common::fmt(accel_time)),
                acceleration,
                "m/s²",
                &source,
            ),
            common::step(
                "加速力",
                "Fa = m * a",
                format!("{mass} * {}", common::fmt(acceleration)),
                acceleration_force,
                "N",
                &source,
            ),
            common::step(
                "等效推力",
                "F = (Ff + Fa) * K / η",
                format!(
                    "({} + {}) * {} / {}",
                    common::fmt(friction_force),
                    common::fmt(acceleration_force),
                    common::fmt(safety_factor),
                    common::fmt(efficiency)
                ),
                total_force,
                "N",
                &source,
            ),
            common::step(
                "输出扭矩",
                "T = F * Dp / 2",
                format!(
                    "{} * {} / 2",
                    common::fmt(total_force),
                    common::fmt(pitch_diameter_m)
                ),
                torque_nm,
                "Nm",
                &source,
            ),
            common::step(
                "需求转速",
                "n = v / (z * p) * 60",
                format!(
                    "{} / {} * 60",
                    common::fmt(speed_m_s),
                    common::fmt(circumference_m)
                ),
                rpm,
                "rpm",
                &source,
            ),
        ],
        vec![
            common::rule(
                "timing-belt-speed",
                "速度区间",
                if speed_mm_s <= 2000.0 {
                    "同步带传动可进入型号匹配".to_string()
                } else {
                    "速度偏高，优先复核齿形、张紧和导轨阻力".to_string()
                },
                format!(
                    "目标速度 {} mm/s，基础阈值 2000 mm/s",
                    common::fmt(speed_mm_s)
                ),
                if speed_mm_s <= 2000.0 {
                    "low"
                } else {
                    "warning"
                },
                &source,
            ),
            common::rule(
                "timing-belt-efficiency",
                "效率输入",
                if efficiency >= 0.7 {
                    "效率输入可用于基础扭矩计算".to_string()
                } else {
                    "效率过低，建议复核机构阻力或改用人工确认值".to_string()
                },
                format!("传动效率 {}", common::fmt(efficiency)),
                if efficiency >= 0.7 { "low" } else { "warning" },
                &source,
            ),
            common::rule(
                "timing-belt-safety-factor",
                "安全系数",
                if safety_factor >= 1.2 {
                    "安全系数已确认，可记录到结果快照".to_string()
                } else {
                    "安全系数偏低，需要复核冲击、偏载和启停工况".to_string()
                },
                format!("用户确认安全系数 {}", common::fmt(safety_factor)),
                if safety_factor >= 1.2 {
                    "low"
                } else {
                    "warning"
                },
                &source,
            ),
        ],
        risks,
        vec![
            common::requirement("outputTorque", "输出扭矩", torque_nm, "Nm"),
            common::requirement("requiredSpeed", "需求转速", rpm, "rpm"),
            common::requirement("power", "估算功率", power_w, "W"),
        ],
    ))
}
