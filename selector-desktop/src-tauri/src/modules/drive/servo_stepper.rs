use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "servo-stepper-sizing";
const SOURCE: &str = "PDF P4 / 文档页 1 / 电机篇 / 伺服步进";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "伺服/步进选型计算".to_string(),
        category: "驱动".to_string(),
        description: "面向直线机构的转速、力矩、惯量比和分辨率复核。".to_string(),
        source_chapter: "电机篇".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field(
                "loadMass",
                "负载质量",
                "kg",
                0.0,
                8.0,
                "直线运动等效负载",
                SOURCE,
            ),
            common::field_with_units(
                "travelPerRev",
                "每转位移",
                "mm",
                &["mm", "m"],
                0.001,
                20.0,
                "丝杆导程或同步带单圈位移",
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
                0.2,
                "从静止到目标速度的时间",
                SOURCE,
            ),
            common::field(
                "frictionCoefficient",
                "摩擦系数",
                "ratio",
                0.0,
                0.1,
                "机构摩擦系数",
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
            common::field(
                "motorInertia",
                "电机转子惯量",
                "kg·m²",
                0.0000001,
                0.00002,
                "候选电机样本中的转子惯量",
                SOURCE,
            ),
            common::field(
                "encoderResolution",
                "编码器分辨率",
                "pulse/rev",
                1.0,
                10000.0,
                "每转脉冲数或等效细分数",
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
    let travel_raw = common::positive(&fields, "travelPerRev")?;
    let travel_m = common::convert(
        travel_raw,
        common::unit(&fields, "travelPerRev")?,
        "m",
        "travelPerRev",
    )?;
    let travel_mm = common::convert(
        travel_raw,
        common::unit(&fields, "travelPerRev")?,
        "mm",
        "travelPerRev",
    )?;
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
    let motor_inertia = common::positive(&fields, "motorInertia")?;
    let encoder_resolution = common::positive(&fields, "encoderResolution")?;

    let rpm = speed_m_s / travel_m * 60.0;
    let omega = rpm * std::f64::consts::TAU / 60.0;
    let angular_acceleration = omega / accel_time;
    let reflected_inertia = mass * (travel_m / std::f64::consts::TAU).powi(2);
    let acceleration = speed_m_s / accel_time;
    let acceleration_force = mass * acceleration;
    let friction_force = mass * 9.80665 * friction;
    let total_force = friction_force + acceleration_force;
    let total_torque = total_force * travel_m / std::f64::consts::TAU * safety_factor / efficiency;
    let inertia_ratio = reflected_inertia / motor_inertia;
    let resolution_mm = travel_mm / encoder_resolution;
    let mut risks = common::safety_risk(safety_factor, &source);
    if inertia_ratio > 10.0 {
        risks.push(common::risk(
            "warning",
            "惯量比超过 10，伺服响应和整定风险较高，建议增大电机或调整传动比。",
            Some("motorInertia"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "servo-stepper-sizing@0.1.0",
        format!(
            "总力矩 {} Nm，需求转速 {} rpm，惯量比 {}",
            common::fmt(total_torque),
            common::fmt(rpm),
            common::fmt(inertia_ratio)
        ),
        format!(
            "按安全系数 {} 计算，候选电机需满足 {} Nm、{} rpm，单脉冲分辨率约 {} mm。",
            common::fmt(safety_factor),
            common::fmt(total_torque),
            common::fmt(rpm),
            common::fmt(resolution_mm)
        ),
        vec![
            common::step(
                "直动惯量",
                "J = m * (L / 2π)^2",
                format!("{mass} * ({} / 2π)^2", common::fmt(travel_m)),
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
                "总力矩",
                "T = (Ff + Fa) * L / 2π * K / η",
                format!(
                    "({} + {}) * {} / 2π * {} / {}",
                    common::fmt(friction_force),
                    common::fmt(acceleration_force),
                    common::fmt(travel_m),
                    common::fmt(safety_factor),
                    common::fmt(efficiency)
                ),
                total_torque,
                "Nm",
                &source,
            ),
            common::step(
                "需求转速",
                "n = v / L * 60",
                format!(
                    "{} / {} * 60",
                    common::fmt(speed_m_s),
                    common::fmt(travel_m)
                ),
                rpm,
                "rpm",
                &source,
            ),
            common::step(
                "分辨率",
                "R = L / pulse",
                format!(
                    "{} / {}",
                    common::fmt(travel_mm),
                    common::fmt(encoder_resolution)
                ),
                resolution_mm,
                "mm/pulse",
                &source,
            ),
        ],
        vec![
            common::rule(
                "servo-inertia-ratio",
                "惯量比",
                if inertia_ratio <= 10.0 {
                    "惯量比可进入候选型号复核。".to_string()
                } else {
                    "惯量比偏高，需要调整电机惯量或传动比。".to_string()
                },
                format!("负载惯量 / 电机惯量 = {}", common::fmt(inertia_ratio)),
                if inertia_ratio <= 10.0 {
                    "low"
                } else {
                    "warning"
                },
                &source,
            ),
            common::rule(
                "servo-resolution",
                "分辨率",
                "按每转位移和编码器脉冲数复核最小位移。".to_string(),
                format!("单脉冲位移 {} mm", common::fmt(resolution_mm)),
                "low",
                &source,
            ),
        ],
        risks,
        vec![
            common::requirement("totalTorque", "总力矩", total_torque, "Nm"),
            common::requirement("requiredSpeed", "需求转速", rpm, "rpm"),
            common::requirement("inertiaRatio", "惯量比", inertia_ratio, "ratio"),
            common::requirement("resolution", "分辨率", resolution_mm, "mm/pulse"),
        ],
    ))
}
