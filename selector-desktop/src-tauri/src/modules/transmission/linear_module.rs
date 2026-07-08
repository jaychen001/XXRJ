use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "linear-module-selector";
const SOURCE: &str = "PDF P57 / 文档页 54 / 直线模组";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "直线模组选型判断".to_string(),
        category: "传动".to_string(),
        description: "按负载、行程、速度和定位精度初筛丝杆、同步带或气动模组。".to_string(),
        source_chapter: "直线模组".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field(
                "loadMass",
                "负载质量",
                "kg",
                0.0,
                8.0,
                "移动负载质量",
                SOURCE,
            ),
            common::field_with_units(
                "stroke",
                "行程",
                "mm",
                &["mm", "m"],
                0.0,
                600.0,
                "有效运动行程",
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
                "positioningAccuracy",
                "定位精度",
                "mm",
                0.0,
                0.05,
                "允许重复定位误差",
                SOURCE,
            ),
            common::field(
                "frictionCoefficient",
                "摩擦系数",
                "ratio",
                0.0,
                0.1,
                "导轨或模组摩擦系数",
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
    let stroke_mm = common::convert(
        common::positive(&fields, "stroke")?,
        common::unit(&fields, "stroke")?,
        "mm",
        "stroke",
    )?;
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
    let accuracy_mm = common::positive(&fields, "positioningAccuracy")?;
    let friction = common::positive_or_zero(&fields, "frictionCoefficient")?;

    let acceleration = speed_m_s / accel_time;
    let acceleration_force = mass * acceleration;
    let friction_force = mass * 9.80665 * friction;
    let required_thrust = (acceleration_force + friction_force) * safety_factor;
    let cycle_time = stroke_mm / speed_mm_s + accel_time * 2.0;
    let recommendation = recommend_module(stroke_mm, speed_mm_s, accuracy_mm);
    let risks = common::safety_risk(safety_factor, &source);

    Ok(common::result(
        module,
        request,
        "linear-module-selector@0.1.0",
        format!(
            "推荐 {}，推力 {} N，估算单程 {} s",
            recommendation,
            common::fmt(required_thrust),
            common::fmt(cycle_time)
        ),
        format!(
            "按安全系数 {} 计算，直线模组至少需要 {} N 推力；当前工况优先考虑 {}。",
            common::fmt(safety_factor),
            common::fmt(required_thrust),
            recommendation
        ),
        vec![
            common::step(
                "加速力",
                "Fa = m * a",
                format!("{mass} * {}", common::fmt(acceleration)),
                acceleration_force,
                "N",
                &source,
            ),
            common::step(
                "摩擦力",
                "Ff = m * g * μ",
                format!("{mass} * 9.80665 * {friction}"),
                friction_force,
                "N",
                &source,
            ),
            common::step(
                "需求推力",
                "F = (Fa + Ff) * K",
                format!(
                    "({} + {}) * {}",
                    common::fmt(acceleration_force),
                    common::fmt(friction_force),
                    common::fmt(safety_factor)
                ),
                required_thrust,
                "N",
                &source,
            ),
            common::step(
                "单程时间",
                "t = S / v + 2ta",
                format!(
                    "{} / {} + 2 * {}",
                    common::fmt(stroke_mm),
                    common::fmt(speed_mm_s),
                    common::fmt(accel_time)
                ),
                cycle_time,
                "s",
                &source,
            ),
        ],
        vec![
            common::rule(
                "linear-module-type",
                "模组类型",
                recommendation.to_string(),
                format!(
                    "行程 {} mm，速度 {} mm/s，定位精度 {} mm",
                    common::fmt(stroke_mm),
                    common::fmt(speed_mm_s),
                    common::fmt(accuracy_mm)
                ),
                "low",
                &source,
            ),
            common::rule(
                "linear-module-accuracy",
                "精度复核",
                if accuracy_mm <= 0.05 {
                    "精度要求较高，优先丝杆或直线电机并复核导轨。".to_string()
                } else {
                    "精度要求可进入常规模组样本匹配。".to_string()
                },
                format!("定位精度 {} mm", common::fmt(accuracy_mm)),
                "low",
                &source,
            ),
        ],
        risks,
        vec![
            common::requirement("requiredThrust", "需求推力", required_thrust, "N"),
            common::requirement("cycleTime", "单程时间", cycle_time, "s"),
            common::requirement("targetAccuracy", "定位精度", accuracy_mm, "mm"),
        ],
    ))
}

fn recommend_module(stroke_mm: f64, speed_mm_s: f64, accuracy_mm: f64) -> &'static str {
    if accuracy_mm <= 0.05 {
        "滚珠丝杠模组"
    } else if stroke_mm >= 1000.0 || speed_mm_s >= 800.0 {
        "同步带直线模组"
    } else {
        "常规直线模组"
    }
}
