use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "general-motor-power";
const SOURCE: &str = "PDF P4 / 文档页 1 / 电机篇";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "通用电机功率计算".to_string(),
        category: "驱动".to_string(),
        description: "输送线、滚筒或普通旋转驱动的功率、扭矩和需求转速估算。".to_string(),
        source_chapter: "电机篇".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field(
                "loadMass",
                "负载质量",
                "kg",
                0.0,
                20.0,
                "输送或旋转等效移动负载",
                SOURCE,
            ),
            common::field_with_units(
                "driveDiameter",
                "驱动直径",
                "mm",
                &["mm", "m"],
                0.001,
                80.0,
                "滚筒、同步轮或等效驱动轮直径",
                SOURCE,
            ),
            common::field_with_units(
                "lineSpeed",
                "线速度",
                "mm/s",
                &["mm/s", "m/s"],
                0.0,
                300.0,
                "机构目标线速度",
                SOURCE,
            ),
            common::field(
                "frictionCoefficient",
                "摩擦系数",
                "ratio",
                0.0,
                0.15,
                "负载与导向/输送面的摩擦系数",
                SOURCE,
            ),
            common::field(
                "efficiency",
                "传动效率",
                "ratio",
                0.01,
                0.85,
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
    let diameter_raw = common::positive(&fields, "driveDiameter")?;
    let diameter_m = common::convert(
        diameter_raw,
        common::unit(&fields, "driveDiameter")?,
        "m",
        "driveDiameter",
    )?;
    let speed_raw = common::positive(&fields, "lineSpeed")?;
    let speed_m_s = common::convert(
        speed_raw,
        common::unit(&fields, "lineSpeed")?,
        "m/s",
        "lineSpeed",
    )?;
    let friction = common::positive_or_zero(&fields, "frictionCoefficient")?;
    let efficiency = common::efficiency(&fields, "efficiency")?;

    let friction_force = mass * 9.80665 * friction;
    let design_force = friction_force * safety_factor / efficiency;
    let output_torque = design_force * diameter_m / 2.0;
    let rpm = speed_m_s / (std::f64::consts::PI * diameter_m) * 60.0;
    let power_w = design_force * speed_m_s;
    let mut risks = common::safety_risk(safety_factor, &source);
    if efficiency < 0.7 {
        risks.push(common::risk(
            "warning",
            "传动效率低于 0.7，建议复核减速机构、链带传动和滚筒阻力。",
            Some("efficiency"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "general-motor-power@0.1.0",
        format!(
            "功率 {} W，输出扭矩 {} Nm，需求转速 {} rpm",
            common::fmt(power_w),
            common::fmt(output_torque),
            common::fmt(rpm)
        ),
        format!(
            "按安全系数 {} 计算，电机侧至少需要 {} W、{} Nm、{} rpm。",
            common::fmt(safety_factor),
            common::fmt(power_w),
            common::fmt(output_torque),
            common::fmt(rpm)
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
                "设计推力",
                "F = Ff * K / η",
                format!(
                    "{} * {} / {}",
                    common::fmt(friction_force),
                    common::fmt(safety_factor),
                    common::fmt(efficiency)
                ),
                design_force,
                "N",
                &source,
            ),
            common::step(
                "输出扭矩",
                "T = F * D / 2",
                format!(
                    "{} * {} / 2",
                    common::fmt(design_force),
                    common::fmt(diameter_m)
                ),
                output_torque,
                "Nm",
                &source,
            ),
            common::step(
                "需求转速",
                "n = v / (πD) * 60",
                format!(
                    "{} / (π * {}) * 60",
                    common::fmt(speed_m_s),
                    common::fmt(diameter_m)
                ),
                rpm,
                "rpm",
                &source,
            ),
            common::step(
                "需求功率",
                "P = F * v",
                format!("{} * {}", common::fmt(design_force), common::fmt(speed_m_s)),
                power_w,
                "W",
                &source,
            ),
        ],
        vec![common::rule(
            "motor-power-margin",
            "功率余量",
            "按计算功率上取标准电机功率，并复核启动转矩。".to_string(),
            format!(
                "计算功率 {} W，安全系数 {}",
                common::fmt(power_w),
                common::fmt(safety_factor)
            ),
            "low",
            &source,
        )],
        risks,
        vec![
            common::requirement("power", "需求功率", power_w, "W"),
            common::requirement("outputTorque", "输出扭矩", output_torque, "Nm"),
            common::requirement("requiredSpeed", "需求转速", rpm, "rpm"),
        ],
    ))
}
