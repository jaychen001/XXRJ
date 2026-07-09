use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "timing-belt-basic";
const SOURCE: &str = "工程公式库 / 同步带";

#[rustfmt::skip]
pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "同步带基础计算".to_string(),
        category: "传动".to_string(),
        description: "按负载、速度、同步轮和附加载荷估算同步带驱动扭矩、转速和功率。".to_string(),
        source_chapter: "同步带".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field("loadMass", "负载质量", "kg", 0.0, 5.0, "移动负载质量", SOURCE),
            common::field("frictionCoefficient", "摩擦系数", "ratio", 0.0, 0.1, "导向面或机构摩擦系数", SOURCE),
            common::field_with_units("targetSpeed", "目标速度", "mm/s", &["mm/s", "m/s"], 0.0, 500.0, "机构目标线速度", SOURCE),
            common::field_with_units("accelerationTime", "加速时间", "s", &["s", "min"], 0.001, 0.3, "从静止到目标速度的时间", SOURCE),
            common::field("externalForce", "外部阻力", "N", 0.0, 0.0, "张紧、拖链、线缆或工艺阻力", SOURCE),
            common::field("verticalLoadFactor", "垂直负载系数", "ratio", 0.0, 0.0, "水平为 0，垂直提升为 1，斜面按 sinθ 输入", SOURCE),
            common::field("pulleyTeeth", "同步轮齿数", "teeth", 1.0, 20.0, "驱动轮齿数", SOURCE),
            common::field_with_units("toothPitch", "齿距", "mm", &["mm", "m"], 0.001, 5.0, "同步带齿距", SOURCE),
            common::field("efficiency", "传动效率", "ratio", 0.01, 0.9, "0-1 之间的小数", SOURCE),
        ],
    }
}

#[rustfmt::skip]
pub fn calculate(request: &CalculationRequest) -> Result<CalculationResult, FieldError> {
    let module = definition();
    let source = module.source_page.clone();
    let fields = common::fields_map(request);
    let safety_factor = common::safety_factor(request)?;
    let mass = common::positive(&fields, "loadMass")?;
    let friction = common::positive_or_zero(&fields, "frictionCoefficient")?;
    let speed_raw = common::positive(&fields, "targetSpeed")?;
    let speed_mm_s = common::convert(speed_raw, common::unit(&fields, "targetSpeed")?, "mm/s", "targetSpeed")?;
    let speed_m_s = common::convert(speed_raw, common::unit(&fields, "targetSpeed")?, "m/s", "targetSpeed")?;
    let accel_time = common::convert(common::positive(&fields, "accelerationTime")?, common::unit(&fields, "accelerationTime")?, "s", "accelerationTime")?;
    let external_force = common::positive_or_zero(&fields, "externalForce")?;
    let vertical_factor = common::positive_or_zero(&fields, "verticalLoadFactor")?;
    let pulley_teeth = common::positive(&fields, "pulleyTeeth")?;
    let tooth_pitch_mm = common::convert(common::positive(&fields, "toothPitch")?, common::unit(&fields, "toothPitch")?, "mm", "toothPitch")?;
    let efficiency = common::efficiency(&fields, "efficiency")?;

    let gravity = 9.80665;
    let friction_force = mass * gravity * friction;
    let acceleration = speed_m_s / accel_time;
    let acceleration_force = mass * acceleration;
    let vertical_force = mass * gravity * vertical_factor;
    let working_force = friction_force + acceleration_force + vertical_force + external_force;
    let design_force = working_force * safety_factor / efficiency;
    let pitch_diameter_m = pulley_teeth * tooth_pitch_mm / std::f64::consts::PI / 1000.0;
    let travel_per_rev_m = pulley_teeth * tooth_pitch_mm / 1000.0;
    let torque_nm = design_force * pitch_diameter_m / 2.0;
    let rpm = speed_m_s / travel_per_rev_m * 60.0;
    let power_w = design_force * speed_m_s;
    let mut risks = common::safety_risk(safety_factor, &source);

    if speed_mm_s > 2000.0 {
        risks.push(common::risk("warning", "目标速度超过 2000 mm/s，需复核齿形、带宽、张紧和导轨阻力。", Some("targetSpeed"), &source));
    }
    if vertical_factor > 0.0 {
        risks.push(common::risk("warning", "存在垂直或斜向负载，同步带需要复核断电防坠和保持制动。", Some("verticalLoadFactor"), &source));
    }
    if efficiency < 0.7 {
        risks.push(common::risk("warning", "传动效率低于 0.7，扭矩需求会明显放大。", Some("efficiency"), &source));
    }

    Ok(common::result(
        module,
        request,
        "timing-belt-basic@0.2.0",
        format!("输出扭矩 {} Nm，需求转速 {} rpm", common::fmt(torque_nm), common::fmt(rpm)),
        format!("按安全系数 {} 和效率 {} 计算，同步带驱动端至少需要 {} Nm、{} rpm，功率约 {} W。", common::fmt(safety_factor), common::fmt(efficiency), common::fmt(torque_nm), common::fmt(rpm), common::fmt(power_w)),
        vec![
            common::step("摩擦力", "Ff = m * g * μ", format!("{mass} * 9.80665 * {friction}"), friction_force, "N", &source),
            common::step("加速度", "a = v / t", format!("{} / {}", common::fmt(speed_m_s), common::fmt(accel_time)), acceleration, "m/s²", &source),
            common::step("加速力", "Fa = m * a", format!("{mass} * {}", common::fmt(acceleration)), acceleration_force, "N", &source),
            common::step("垂直负载力", "Fg = m * g * Kv", format!("{mass} * 9.80665 * {}", common::fmt(vertical_factor)), vertical_force, "N", &source),
            common::step("外部阻力", "Fe = 用户输入", common::fmt(external_force), external_force, "N", &source),
            common::step("等效推力", "F = (Ff + Fa + Fg + Fe) * K / η", format!("({} + {} + {} + {}) * {} / {}", common::fmt(friction_force), common::fmt(acceleration_force), common::fmt(vertical_force), common::fmt(external_force), common::fmt(safety_factor), common::fmt(efficiency)), design_force, "N", &source),
            common::step("输出扭矩", "T = F * Dp / 2", format!("{} * {} / 2", common::fmt(design_force), common::fmt(pitch_diameter_m)), torque_nm, "Nm", &source),
            common::step("需求转速", "n = v / (z * p) * 60", format!("{} / {} * 60", common::fmt(speed_m_s), common::fmt(travel_per_rev_m)), rpm, "rpm", &source),
            common::step("估算功率", "P = F * v", format!("{} * {}", common::fmt(design_force), common::fmt(speed_m_s)), power_w, "W", &source),
        ],
        vec![
            common::rule("timing-belt-speed", "速度区间", if speed_mm_s <= 2000.0 { "同步带传动可进入型号匹配".to_string() } else { "速度偏高，优先复核齿形、张紧和导轨阻力".to_string() }, format!("目标速度 {} mm/s，基础阈值 2000 mm/s", common::fmt(speed_mm_s)), if speed_mm_s <= 2000.0 { "low" } else { "warning" }, &source),
            common::rule("timing-belt-force", "带拉力初筛", "按等效推力上取同步带宽度和许用拉力，型号库匹配时必须满足额定拉力。".to_string(), format!("等效推力 {} N", common::fmt(design_force)), "low", &source),
            common::rule("timing-belt-efficiency", "效率输入", if efficiency >= 0.7 { "效率输入可用于基础扭矩计算".to_string() } else { "效率过低，建议复核机构阻力或改用人工确认值".to_string() }, format!("传动效率 {}", common::fmt(efficiency)), if efficiency >= 0.7 { "low" } else { "warning" }, &source),
            common::rule("timing-belt-safety-factor", "安全系数", if safety_factor >= 1.2 { "安全系数已确认，可记录到结果快照".to_string() } else { "安全系数偏低，需要复核冲击、偏载和启停工况".to_string() }, format!("用户确认安全系数 {}", common::fmt(safety_factor)), if safety_factor >= 1.2 { "low" } else { "warning" }, &source),
        ],
        risks,
        vec![
            common::requirement("beltForce", "等效推力", design_force, "N"),
            common::requirement("outputTorque", "输出扭矩", torque_nm, "Nm"),
            common::requirement("requiredSpeed", "需求转速", rpm, "rpm"),
            common::requirement("power", "估算功率", power_w, "W"),
        ],
    ))
}
