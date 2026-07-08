use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "ball-screw-servo";
const SOURCE: &str = "PDF P25 / 文档页 22 / 丝杆篇";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "滚珠丝杠伺服计算".to_string(),
        category: "传动".to_string(),
        description: "滚珠丝杠直线负载折算惯量、力矩和伺服转速。".to_string(),
        source_chapter: "丝杆篇".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field(
                "loadMass",
                "负载质量",
                "kg",
                0.0,
                10.0,
                "移动负载质量",
                SOURCE,
            ),
            common::field_with_units(
                "lead",
                "丝杠导程",
                "mm",
                &["mm", "m"],
                0.001,
                10.0,
                "丝杠每转直线位移",
                SOURCE,
            ),
            common::field_with_units(
                "targetSpeed",
                "目标速度",
                "mm/s",
                &["mm/s", "m/s"],
                0.0,
                300.0,
                "机构目标线速度",
                SOURCE,
            ),
            common::field_with_units(
                "accelerationTime",
                "加速时间",
                "s",
                &["s", "min"],
                0.001,
                0.2,
                "从静止到目标速度的时间",
                SOURCE,
            ),
            common::field(
                "frictionCoefficient",
                "摩擦系数",
                "ratio",
                0.0,
                0.05,
                "导轨或丝杠等效摩擦系数",
                SOURCE,
            ),
            common::field(
                "efficiency",
                "丝杠效率",
                "ratio",
                0.01,
                0.9,
                "滚珠丝杠常用效率人工确认",
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
    let lead_raw = common::positive(&fields, "lead")?;
    let lead_m = common::convert(lead_raw, common::unit(&fields, "lead")?, "m", "lead")?;
    let speed_raw = common::positive(&fields, "targetSpeed")?;
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
    let friction = common::positive_or_zero(&fields, "frictionCoefficient")?;
    let efficiency = common::efficiency(&fields, "efficiency")?;

    let rpm = speed_m_s / lead_m * 60.0;
    let omega = rpm * std::f64::consts::TAU / 60.0;
    let angular_acceleration = omega / accel_time;
    let reflected_inertia = mass * (lead_m / std::f64::consts::TAU).powi(2);
    let friction_force = mass * 9.80665 * friction;
    let acceleration = speed_m_s / accel_time;
    let acceleration_force = mass * acceleration;
    let acceleration_torque = reflected_inertia * angular_acceleration * safety_factor / efficiency;
    let uniform_torque =
        friction_force * lead_m / std::f64::consts::TAU * safety_factor / efficiency;
    let total_torque = acceleration_torque + uniform_torque;
    let mut risks = common::safety_risk(safety_factor, &source);
    if rpm > 3000.0 {
        risks.push(common::risk(
            "warning",
            "丝杠需求转速超过 3000 rpm，需要复核临界转速、支撑方式和导程。",
            Some("targetSpeed"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "ball-screw-servo@0.1.0",
        format!(
            "总力矩 {} Nm，需求转速 {} rpm",
            common::fmt(total_torque),
            common::fmt(rpm)
        ),
        format!(
            "按安全系数 {} 计算，丝杠伺服至少需要 {} Nm、{} rpm。",
            common::fmt(safety_factor),
            common::fmt(total_torque),
            common::fmt(rpm)
        ),
        vec![
            common::step(
                "直动惯量",
                "J = m * (L / 2π)^2",
                format!("{mass} * ({} / 2π)^2", common::fmt(lead_m)),
                reflected_inertia,
                "kg·m²",
                &source,
            ),
            common::step(
                "角加速度",
                "α = ω / t",
                format!("{} / {}", common::fmt(omega), common::fmt(accel_time)),
                angular_acceleration,
                "rad/s²",
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
                "摩擦力",
                "Ff = m * g * μ",
                format!("{mass} * 9.80665 * {friction}"),
                friction_force,
                "N",
                &source,
            ),
            common::step(
                "加速力矩",
                "Ta = J * α * K / η",
                format!(
                    "{} * {} * {} / {}",
                    common::fmt(reflected_inertia),
                    common::fmt(angular_acceleration),
                    common::fmt(safety_factor),
                    common::fmt(efficiency)
                ),
                acceleration_torque,
                "Nm",
                &source,
            ),
            common::step(
                "匀速力矩",
                "Tc = Ff * L / 2π * K / η",
                format!(
                    "{} * {} / 2π * {} / {}",
                    common::fmt(friction_force),
                    common::fmt(lead_m),
                    common::fmt(safety_factor),
                    common::fmt(efficiency)
                ),
                uniform_torque,
                "Nm",
                &source,
            ),
            common::step(
                "总力矩",
                "T = Ta + Tc",
                format!(
                    "{} + {}",
                    common::fmt(acceleration_torque),
                    common::fmt(uniform_torque)
                ),
                total_torque,
                "Nm",
                &source,
            ),
            common::step(
                "需求转速",
                "n = v / L * 60",
                format!("{} / {} * 60", common::fmt(speed_m_s), common::fmt(lead_m)),
                rpm,
                "rpm",
                &source,
            ),
        ],
        vec![
            common::rule(
                "ball-screw-speed",
                "临界转速",
                if rpm <= 3000.0 {
                    "可进入样本临界转速和支撑方式复核。".to_string()
                } else {
                    "转速偏高，优先调整导程或改同步带模组。".to_string()
                },
                format!("需求转速 {} rpm", common::fmt(rpm)),
                if rpm <= 3000.0 { "low" } else { "warning" },
                &source,
            ),
            common::rule(
                "ball-screw-efficiency",
                "效率输入",
                "效率来自人工输入，报告记录到输入快照。".to_string(),
                format!("效率 {}", common::fmt(efficiency)),
                "low",
                &source,
            ),
        ],
        risks,
        vec![
            common::requirement("reflectedInertia", "直动惯量", reflected_inertia, "kg·m²"),
            common::requirement(
                "angularAcceleration",
                "角加速度",
                angular_acceleration,
                "rad/s²",
            ),
            common::requirement("accelerationTorque", "加速力矩", acceleration_torque, "Nm"),
            common::requirement("uniformTorque", "匀速力矩", uniform_torque, "Nm"),
            common::requirement("totalTorque", "总力矩", total_torque, "Nm"),
            common::requirement("requiredSpeed", "需求转速", rpm, "rpm"),
        ],
    ))
}
