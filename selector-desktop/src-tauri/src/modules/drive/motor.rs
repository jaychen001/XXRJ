use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "general-motor-power";
const SOURCE: &str = "工程公式库 / 普通与调速电机";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "通用电机功率计算".to_string(),
        category: "驱动".to_string(),
        description: "用于输送线、滚筒、调速电机等连续驱动，按已知工况估算功率、扭矩和转速。"
            .to_string(),
        source_chapter: "普通电机 / 调速电机".to_string(),
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
            common::field_with_units(
                "accelerationTime",
                "启动加速时间",
                "s",
                &["s", "min"],
                0.001,
                1.0,
                "从静止到目标速度的时间",
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
                "externalForce",
                "外部阻力",
                "N",
                0.0,
                0.0,
                "张紧、刮料、拖链或工艺阻力",
                SOURCE,
            ),
            common::field(
                "verticalLoadFactor",
                "垂直负载系数",
                "ratio",
                0.0,
                0.0,
                "水平为 0，垂直提升为 1，斜面按 sinθ 输入",
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
    let accel_time = common::convert(
        common::positive(&fields, "accelerationTime")?,
        common::unit(&fields, "accelerationTime")?,
        "s",
        "accelerationTime",
    )?;
    let friction = common::positive_or_zero(&fields, "frictionCoefficient")?;
    let external_force = common::positive_or_zero(&fields, "externalForce")?;
    let vertical_factor = common::positive_or_zero(&fields, "verticalLoadFactor")?;
    let efficiency = common::efficiency(&fields, "efficiency")?;

    let gravity = 9.80665;
    let friction_force = mass * gravity * friction;
    let acceleration = speed_m_s / accel_time;
    let acceleration_force = mass * acceleration;
    let vertical_force = mass * gravity * vertical_factor;
    let working_force = friction_force + acceleration_force + vertical_force + external_force;
    let design_force = working_force * safety_factor / efficiency;
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
    if accel_time < 0.2 {
        risks.push(common::risk(
            "warning",
            "启动加速时间低于 0.2 s，需复核启动转矩、减速机冲击和电机过载能力。",
            Some("accelerationTime"),
            &source,
        ));
    }
    if vertical_factor > 0.0 {
        risks.push(common::risk(
            "warning",
            "存在提升或斜坡负载，普通电机需要复核制动器和断电保持能力。",
            Some("verticalLoadFactor"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "general-motor-power@0.2.0",
        format!(
            "功率 {} W，输出扭矩 {} Nm，需求转速 {} rpm",
            common::fmt(power_w),
            common::fmt(output_torque),
            common::fmt(rpm)
        ),
        format!(
            "按安全系数 {} 计算，电机侧至少需要 {} W、{} Nm、{} rpm；型号匹配时需再复核启动转矩和调速范围。",
            common::fmt(safety_factor),
            common::fmt(power_w),
            common::fmt(output_torque),
            common::fmt(rpm)
        ),
        vec![
            common::step("摩擦力", "Ff = m * g * μ", format!("{mass} * 9.80665 * {friction}"), friction_force, "N", &source),
            common::step("加速力", "Fa = m * v / t", format!("{mass} * {} / {}", common::fmt(speed_m_s), common::fmt(accel_time)), acceleration_force, "N", &source),
            common::step("垂直负载力", "Fg = m * g * Kv", format!("{mass} * 9.80665 * {}", common::fmt(vertical_factor)), vertical_force, "N", &source),
            common::step("等效推力", "F = (Ff + Fa + Fg + Fe) * K / η", format!(
                "({} + {} + {} + {}) * {} / {}",
                common::fmt(friction_force),
                common::fmt(acceleration_force),
                common::fmt(vertical_force),
                common::fmt(external_force),
                common::fmt(safety_factor),
                common::fmt(efficiency)
            ), design_force, "N", &source),
            common::step("输出扭矩", "T = F * D / 2", format!("{} * {} / 2", common::fmt(design_force), common::fmt(diameter_m)), output_torque, "Nm", &source),
            common::step("需求转速", "n = v / (πD) * 60", format!("{} / (π * {}) * 60", common::fmt(speed_m_s), common::fmt(diameter_m)), rpm, "rpm", &source),
            common::step("需求功率", "P = F * v", format!("{} * {}", common::fmt(design_force), common::fmt(speed_m_s)), power_w, "W", &source),
        ],
        vec![
            common::rule(
                "motor-power-margin",
                "功率余量",
                "按计算功率上取标准电机功率，并复核启动转矩。".to_string(),
                format!("计算功率 {} W，安全系数 {}", common::fmt(power_w), common::fmt(safety_factor)),
                "low",
                &source,
            ),
            common::rule(
                "motor-speed-range",
                "调速范围",
                if (50.0..=3000.0).contains(&rpm) { "需求转速在常规电机/减速电机初筛范围内。".to_string() } else { "需求转速偏离常规范围，建议先调整减速比或驱动轮直径。".to_string() },
                format!("需求转速 {} rpm", common::fmt(rpm)),
                if (50.0..=3000.0).contains(&rpm) { "low" } else { "warning" },
                &source,
            ),
        ],
        risks,
        vec![
            common::requirement("designForce", "等效推力", design_force, "N"),
            common::requirement("power", "需求功率", power_w, "W"),
            common::requirement("outputTorque", "输出扭矩", output_torque, "Nm"),
            common::requirement("requiredSpeed", "需求转速", rpm, "rpm"),
        ],
    ))
}
