use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "ball-screw-servo";
const SOURCE: &str = "工程公式库 / 滚珠丝杠";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "滚珠丝杠伺服计算".to_string(),
        category: "传动".to_string(),
        description: "滚珠丝杠直线负载折算惯量、力矩和伺服转速。".to_string(),
        source_chapter: "滚珠丝杠".to_string(),
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
                "externalForce",
                "外部轴向力",
                "N",
                0.0,
                0.0,
                "压装、弹簧、工艺阻力等沿运动方向的反向力；没有填 0",
                SOURCE,
            ),
            common::field(
                "verticalLoadFactor",
                "垂直负载系数",
                "ratio",
                0.0,
                0.0,
                "水平运动填 0，垂直上升填 1，斜面按 sinθ 填",
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
            common::field(
                "supportSpan",
                "支撑跨距",
                "mm",
                1.0,
                600.0,
                "两端支撑间的有效丝杠长度",
                SOURCE,
            ),
            common::field(
                "screwRootDiameter",
                "丝杠底径",
                "mm",
                1.0,
                12.0,
                "样本中的螺纹底径或小径",
                SOURCE,
            ),
            common::field(
                "supportCoefficient",
                "支撑方式系数",
                "ratio",
                0.1,
                15.1,
                "固定-支撑可先填 15.1，固定-固定可填 21.9",
                SOURCE,
            ),
            common::field(
                "dynamicLoadRating",
                "额定动载荷",
                "N",
                1.0,
                5000.0,
                "候选丝杠样本中的 Ca/C 动载荷",
                SOURCE,
            ),
            common::field(
                "requiredTravelLife",
                "目标行走寿命",
                "km",
                0.0,
                10000.0,
                "按设备预期寿命折算的累计行走距离",
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
    let external_force = common::positive_or_zero(&fields, "externalForce")?;
    let vertical_load_factor = common::positive_or_zero(&fields, "verticalLoadFactor")?;
    let efficiency = common::efficiency(&fields, "efficiency")?;
    let support_span = common::positive(&fields, "supportSpan")?;
    let screw_root_diameter = common::positive(&fields, "screwRootDiameter")?;
    let support_coefficient = common::positive(&fields, "supportCoefficient")?;
    let dynamic_load_rating = common::positive(&fields, "dynamicLoadRating")?;
    let required_travel_life = common::positive_or_zero(&fields, "requiredTravelLife")?;

    let rpm = speed_m_s / lead_m * 60.0;
    let omega = rpm * std::f64::consts::TAU / 60.0;
    let angular_acceleration = omega / accel_time;
    let reflected_inertia = mass * (lead_m / std::f64::consts::TAU).powi(2);
    let friction_force = mass * 9.80665 * friction;
    let gravity_force = mass * 9.80665 * vertical_load_factor;
    let acceleration = speed_m_s / accel_time;
    let acceleration_force = mass * acceleration;
    let static_axial_force = friction_force + gravity_force + external_force;
    let design_axial_force = (static_axial_force + acceleration_force) * safety_factor;
    let acceleration_torque = reflected_inertia * angular_acceleration * safety_factor / efficiency;
    let uniform_torque =
        static_axial_force * lead_m / std::f64::consts::TAU * safety_factor / efficiency;
    let total_torque = acceleration_torque + uniform_torque;
    let critical_speed =
        support_coefficient * 10_000_000.0 * screw_root_diameter / support_span.powi(2);
    let travel_life_km = (dynamic_load_rating / design_axial_force).powi(3) * lead_m * 1000.0;
    let mut risks = common::safety_risk(safety_factor, &source);
    if rpm > critical_speed * 0.8 {
        risks.push(common::risk(
            "warning",
            "丝杠需求转速接近或超过临界转速 80%，需要调整导程、支撑方式或丝杠直径。",
            Some("targetSpeed"),
            &source,
        ));
    }
    if required_travel_life > 0.0 && travel_life_km < required_travel_life {
        risks.push(common::risk(
            "warning",
            "估算行走寿命低于目标寿命，需提高丝杠动载荷等级或降低轴向负载。",
            Some("dynamicLoadRating"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "ball-screw-servo@0.1.0",
        format!(
            "总力矩 {} Nm，需求转速 {} rpm，临界转速约 {} rpm",
            common::fmt(total_torque),
            common::fmt(rpm),
            common::fmt(critical_speed)
        ),
        format!(
            "按安全系数 {} 计算，丝杠伺服至少需要 {} Nm、{} rpm；估算行走寿命 {} km。",
            common::fmt(safety_factor),
            common::fmt(total_torque),
            common::fmt(rpm),
            common::fmt(travel_life_km)
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
                "垂直负载力",
                "Fg = m * g * Kv",
                format!("{mass} * 9.80665 * {}", common::fmt(vertical_load_factor)),
                gravity_force,
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
                "Tc = (Ff + Fg + Fe) * L / 2π * K / η",
                format!(
                    "({} + {} + {}) * {} / 2π * {} / {}",
                    common::fmt(friction_force),
                    common::fmt(gravity_force),
                    common::fmt(external_force),
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
            common::step(
                "临界转速",
                "ncr = Cf * 10^7 * dr / Ls²",
                format!(
                    "{} * 10^7 * {} / {}^2",
                    common::fmt(support_coefficient),
                    common::fmt(screw_root_diameter),
                    common::fmt(support_span)
                ),
                critical_speed,
                "rpm",
                &source,
            ),
            common::step(
                "行走寿命",
                "Lkm = (C / P)^3 * lead * 1000",
                format!(
                    "({} / {})^3 * {} * 1000",
                    common::fmt(dynamic_load_rating),
                    common::fmt(design_axial_force),
                    common::fmt(lead_m)
                ),
                travel_life_km,
                "km",
                &source,
            ),
        ],
        vec![
            common::rule(
                "ball-screw-speed",
                "临界转速",
                if rpm <= critical_speed * 0.8 {
                    "需求转速低于临界转速预警线，可继续复核样本。".to_string()
                } else {
                    "转速偏高，优先调整导程、支撑方式或改同步带模组。".to_string()
                },
                format!(
                    "需求转速 {} rpm，临界转速 80% 为 {} rpm",
                    common::fmt(rpm),
                    common::fmt(critical_speed * 0.8)
                ),
                if rpm <= critical_speed * 0.8 {
                    "low"
                } else {
                    "warning"
                },
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
            common::requirement("criticalSpeed", "临界转速", critical_speed, "rpm"),
            common::requirement("travelLife", "估算行走寿命", travel_life_km, "km"),
        ],
    ))
}
