use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "pneumatic-slide-table-sizing";
const SOURCE: &str = "PDF P80 / 文档页 77 / 滑台气缸";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "滑台气缸".to_string(),
        category: "气动".to_string(),
        description: "按滑台负载、导向摩擦、加速度和行程速度估算滑台气缸推力需求。".to_string(),
        source_chapter: "气动执行元件".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field(
                "loadMass",
                "负载质量",
                "kg",
                0.0,
                5.0,
                "滑台承载总质量",
                SOURCE,
            ),
            common::field(
                "guideFriction",
                "导向摩擦系数",
                "ratio",
                0.0,
                0.15,
                "滑台导向摩擦系数",
                SOURCE,
            ),
            common::field(
                "acceleration",
                "加速度",
                "m/s²",
                0.0,
                2.0,
                "动作加速阶段等效加速度",
                SOURCE,
            ),
            common::field_with_units(
                "stroke",
                "行程",
                "mm",
                &["mm", "m"],
                0.001,
                100.0,
                "滑台单程行程",
                SOURCE,
            ),
            common::field_with_units(
                "moveTime",
                "移动时间",
                "s",
                &["s", "min"],
                0.001,
                0.5,
                "完成单程移动的时间",
                SOURCE,
            ),
            common::field(
                "loadRateLimit",
                "负载率上限",
                "ratio",
                0.1,
                0.5,
                "滑台气缸负载率修正",
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
    let friction = common::positive_or_zero(&fields, "guideFriction")?;
    let acceleration = common::positive_or_zero(&fields, "acceleration")?;
    let stroke_m = common::convert(
        common::positive(&fields, "stroke")?,
        common::unit(&fields, "stroke")?,
        "m",
        "stroke",
    )?;
    let move_time = common::convert(
        common::positive(&fields, "moveTime")?,
        common::unit(&fields, "moveTime")?,
        "s",
        "moveTime",
    )?;
    let load_rate = common::efficiency(&fields, "loadRateLimit")?;
    let friction_force = mass * 9.80665 * friction;
    let acceleration_force = mass * acceleration;
    let required_thrust = (friction_force + acceleration_force) * safety_factor / load_rate;
    let average_speed = stroke_m / move_time;
    let mut risks = common::safety_risk(safety_factor, &source);
    if average_speed > 0.8 {
        risks.push(common::risk(
            "warning",
            "滑台平均速度高于 0.8 m/s，需复核缓冲和导向刚性。",
            Some("moveTime"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "pneumatic-slide-table-sizing@0.1.0",
        format!(
            "推力需求 {} N，平均速度 {} m/s",
            common::fmt(required_thrust),
            common::fmt(average_speed)
        ),
        format!(
            "滑台气缸至少需要 {} N 推力，并复核行程、缓冲和导向负载。",
            common::fmt(required_thrust)
        ),
        vec![
            common::step(
                "导向摩擦力",
                "Ff = m * g * μ",
                format!("{}*9.80665*{}", common::fmt(mass), common::fmt(friction)),
                friction_force,
                "N",
                &source,
            ),
            common::step(
                "加速力",
                "Fa = m * a",
                format!("{}*{}", common::fmt(mass), common::fmt(acceleration)),
                acceleration_force,
                "N",
                &source,
            ),
            common::step(
                "推力需求",
                "F = (Ff + Fa) * K / λ",
                format!(
                    "({}+{})*{} / {}",
                    common::fmt(friction_force),
                    common::fmt(acceleration_force),
                    common::fmt(safety_factor),
                    common::fmt(load_rate)
                ),
                required_thrust,
                "N",
                &source,
            ),
            common::step(
                "平均速度",
                "v = S / t",
                format!("{} / {}", common::fmt(stroke_m), common::fmt(move_time)),
                average_speed,
                "m/s",
                &source,
            ),
        ],
        vec![common::rule(
            "slide-table-buffer",
            "缓冲复核",
            "按推力需求上取滑台气缸规格，并复核缓冲、导向力矩和行程调节。".to_string(),
            format!("平均速度 {} m/s", common::fmt(average_speed)),
            if average_speed <= 0.8 {
                "low"
            } else {
                "warning"
            },
            &source,
        )],
        risks,
        vec![
            common::requirement("requiredThrust", "推力需求", required_thrust, "N"),
            common::requirement("averageSpeed", "平均速度", average_speed, "m/s"),
        ],
    ))
}
