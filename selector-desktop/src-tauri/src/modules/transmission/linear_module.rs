use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "linear-module-selector";
const SOURCE: &str = "工程公式库 / 直线模组";

#[rustfmt::skip]
pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "直线模组选型判断".to_string(),
        category: "传动".to_string(),
        description: "按负载、行程、速度、定位精度和候选样本能力初筛丝杆、同步带或气动直线模组。".to_string(),
        source_chapter: "直线模组".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field("loadMass", "负载质量", "kg", 0.0, 8.0, "移动负载质量", SOURCE),
            common::field_with_units("stroke", "行程", "mm", &["mm", "m"], 0.0, 600.0, "有效运动行程", SOURCE),
            common::field_with_units("targetSpeed", "目标速度", "mm/s", &["mm/s", "m/s"], 0.0, 500.0, "机构目标线速度", SOURCE),
            common::field_with_units("accelerationTime", "加速时间", "s", &["s", "min"], 0.001, 0.3, "从静止到目标速度的时间", SOURCE),
            common::field("positioningAccuracy", "目标定位精度", "mm", 0.0, 0.05, "允许重复定位误差或定位公差", SOURCE),
            common::field("candidateRepeatability", "候选重复定位精度", "mm", 0.0, 0.02, "样本标称重复定位精度", SOURCE),
            common::field("frictionCoefficient", "摩擦系数", "ratio", 0.0, 0.1, "导轨或模组摩擦系数", SOURCE),
            common::field("externalForce", "外部阻力", "N", 0.0, 0.0, "拖链、线缆、压装或工艺阻力", SOURCE),
            common::field("verticalLoadFactor", "垂直负载系数", "ratio", 0.0, 0.0, "水平为 0，垂直提升为 1，斜面按 sinθ 输入", SOURCE),
            common::field("driveEfficiency", "驱动效率", "ratio", 0.01, 0.85, "丝杆、同步带或齿轮齿条驱动效率", SOURCE),
            common::field("candidateRatedThrust", "候选额定推力", "N", 0.0, 600.0, "样本允许连续推力或额定推力", SOURCE),
            common::field("guideLoadFactor", "导向载荷系数", "ratio", 0.0, 1.2, "偏载、安装方向和冲击的等效修正", SOURCE),
            common::field("dynamicLoadRating", "额定动载荷", "N", 1.0, 4000.0, "候选模组滑块或导轨动额定载荷 C", SOURCE),
            common::field("staticLoadRating", "额定静载荷", "N", 1.0, 8000.0, "候选模组滑块或导轨静额定载荷 C0", SOURCE),
            common::field("targetTravelLife", "目标行走寿命", "km", 0.0, 10000.0, "按设备寿命折算的累计行走距离", SOURCE),
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
    let stroke_mm = common::convert(common::positive(&fields, "stroke")?, common::unit(&fields, "stroke")?, "mm", "stroke")?;
    let speed_raw = common::positive(&fields, "targetSpeed")?;
    let speed_mm_s = common::convert(speed_raw, common::unit(&fields, "targetSpeed")?, "mm/s", "targetSpeed")?;
    let speed_m_s = common::convert(speed_raw, common::unit(&fields, "targetSpeed")?, "m/s", "targetSpeed")?;
    let accel_time = common::convert(common::positive(&fields, "accelerationTime")?, common::unit(&fields, "accelerationTime")?, "s", "accelerationTime")?;
    let accuracy_mm = common::positive(&fields, "positioningAccuracy")?;
    let repeatability = common::positive(&fields, "candidateRepeatability")?;
    let friction = common::positive_or_zero(&fields, "frictionCoefficient")?;
    let external_force = common::positive_or_zero(&fields, "externalForce")?;
    let vertical_factor = common::positive_or_zero(&fields, "verticalLoadFactor")?;
    let efficiency = common::efficiency(&fields, "driveEfficiency")?;
    let candidate_thrust = common::positive(&fields, "candidateRatedThrust")?;
    let guide_load_factor = common::positive(&fields, "guideLoadFactor")?;
    let dynamic_load_rating = common::positive(&fields, "dynamicLoadRating")?;
    let static_load_rating = common::positive(&fields, "staticLoadRating")?;
    let target_life = common::positive_or_zero(&fields, "targetTravelLife")?;

    let gravity = 9.80665;
    let acceleration = speed_m_s / accel_time;
    let acceleration_force = mass * acceleration;
    let friction_force = mass * gravity * friction;
    let vertical_force = mass * gravity * vertical_factor;
    let working_force = acceleration_force + friction_force + vertical_force + external_force;
    let required_thrust = working_force * safety_factor / efficiency;
    let thrust_margin = candidate_thrust / required_thrust;
    let guide_design_load = mass * gravity * guide_load_factor * safety_factor;
    let static_margin = static_load_rating / guide_design_load;
    let rated_life = 50.0 * (dynamic_load_rating / guide_design_load).powi(3);
    let required_dynamic_rating = guide_design_load * (target_life / 50.0).powf(1.0 / 3.0);
    let accuracy_margin = accuracy_mm / repeatability;
    let cycle_time = stroke_mm / speed_mm_s + accel_time * 2.0;
    let recommendation = recommend_module(stroke_mm, speed_mm_s, accuracy_mm, required_thrust);
    let mut risks = common::safety_risk(safety_factor, &source);

    if thrust_margin < 1.2 { risks.push(common::risk("warning", "候选模组推力余量低于 1.2，建议提高规格或重新确认外部阻力。", Some("candidateRatedThrust"), &source)); }
    if static_margin < 1.5 { risks.push(common::risk("warning", "导向静载余量低于 1.5，偏载或冲击工况风险较高。", Some("staticLoadRating"), &source)); }
    if target_life > 0.0 && rated_life < target_life { risks.push(common::risk("warning", "估算行走寿命低于目标寿命，需提高导轨/滑块动载荷等级或降低载荷。", Some("dynamicLoadRating"), &source)); }
    if accuracy_margin < 1.0 { risks.push(common::risk("warning", "候选重复定位精度不能满足目标定位精度。", Some("candidateRepeatability"), &source)); }
    if vertical_factor > 0.0 { risks.push(common::risk("warning", "存在垂直或斜向负载，需复核断电保持、制动和防坠。", Some("verticalLoadFactor"), &source)); }

    Ok(common::result(
        module,
        request,
        "linear-module-selector@0.2.0",
        format!("推荐 {}，推力 {} N，寿命 {} km", recommendation, common::fmt(required_thrust), common::fmt(rated_life)),
        format!("按安全系数 {} 和效率 {} 计算，直线模组至少需要 {} N 推力；当前工况优先考虑 {}。",
            common::fmt(safety_factor), common::fmt(efficiency), common::fmt(required_thrust), recommendation),
        vec![
            common::step("加速度", "a = v / t", format!("{} / {}", common::fmt(speed_m_s), common::fmt(accel_time)), acceleration, "m/s²", &source),
            common::step("加速力", "Fa = m * a", format!("{mass} * {}", common::fmt(acceleration)), acceleration_force, "N", &source),
            common::step("摩擦力", "Ff = m * g * μ", format!("{mass} * 9.80665 * {friction}"), friction_force, "N", &source),
            common::step("垂直负载力", "Fg = m * g * Kv", format!("{mass} * 9.80665 * {}", common::fmt(vertical_factor)), vertical_force, "N", &source),
            common::step("推力需求", "F = (Fa + Ff + Fg + Fe) * K / η", format!("({} + {} + {} + {}) * {} / {}", common::fmt(acceleration_force), common::fmt(friction_force), common::fmt(vertical_force), common::fmt(external_force), common::fmt(safety_factor), common::fmt(efficiency)), required_thrust, "N", &source),
            common::step("推力余量", "M = Fc / F", format!("{} / {}", common::fmt(candidate_thrust), common::fmt(required_thrust)), thrust_margin, "ratio", &source),
            common::step("导向设计载荷", "P = m * g * Kp * K", format!("{mass} * 9.80665 * {} * {}", common::fmt(guide_load_factor), common::fmt(safety_factor)), guide_design_load, "N", &source),
            common::step("额定寿命", "L = 50 * (C / P)^3", format!("50 * ({} / {})^3", common::fmt(dynamic_load_rating), common::fmt(guide_design_load)), rated_life, "km", &source),
            common::step("所需动额定载荷", "Creq = P * (Lreq / 50)^(1/3)", format!("{} * ({} / 50)^(1/3)", common::fmt(guide_design_load), common::fmt(target_life)), required_dynamic_rating, "N", &source),
            common::step("静载余量", "S0 = C0 / P", format!("{} / {}", common::fmt(static_load_rating), common::fmt(guide_design_load)), static_margin, "ratio", &source),
            common::step("精度余量", "Ma = Areq / Arep", format!("{} / {}", common::fmt(accuracy_mm), common::fmt(repeatability)), accuracy_margin, "ratio", &source),
            common::step("单程时间", "t = S / v + 2ta", format!("{} / {} + 2 * {}", common::fmt(stroke_mm), common::fmt(speed_mm_s), common::fmt(accel_time)), cycle_time, "s", &source),
        ],
        vec![
            common::rule("linear-module-type", "模组类型", recommendation.to_string(), format!("行程 {} mm，速度 {} mm/s，定位精度 {} mm，推力 {} N", common::fmt(stroke_mm), common::fmt(speed_mm_s), common::fmt(accuracy_mm), common::fmt(required_thrust)), "low", &source),
            common::rule("linear-module-thrust", "推力余量", if thrust_margin >= 1.2 { "候选模组推力满足基础余量。".to_string() } else { "候选模组推力不足，需提高规格或降低阻力。".to_string() }, format!("推力余量 {}", common::fmt(thrust_margin)), if thrust_margin >= 1.2 { "low" } else { "warning" }, &source),
            common::rule("linear-module-life", "寿命匹配", if target_life <= 0.0 || rated_life >= target_life { "估算寿命满足目标或未设置目标寿命。".to_string() } else { "估算寿命不足，需提高动载荷等级。".to_string() }, format!("估算寿命 {} km，目标 {} km", common::fmt(rated_life), common::fmt(target_life)), if target_life <= 0.0 || rated_life >= target_life { "low" } else { "warning" }, &source),
        ],
        risks,
        vec![
            common::requirement("requiredThrust", "推力需求", required_thrust, "N"),
            common::requirement("thrustMargin", "推力余量", thrust_margin, "ratio"),
            common::requirement("guideDesignLoad", "导向设计载荷", guide_design_load, "N"),
            common::requirement("ratedLife", "额定寿命", rated_life, "km"),
            common::requirement("requiredDynamicLoadRating", "所需动额定载荷", required_dynamic_rating, "N"),
            common::requirement("staticMargin", "静载余量", static_margin, "ratio"),
            common::requirement("accuracyMargin", "精度余量", accuracy_margin, "ratio"),
            common::requirement("cycleTime", "单程时间", cycle_time, "s"),
        ],
    ))
}

fn recommend_module(
    stroke_mm: f64,
    speed_mm_s: f64,
    accuracy_mm: f64,
    required_thrust: f64,
) -> &'static str {
    if accuracy_mm <= 0.05 || required_thrust > 800.0 {
        "滚珠丝杠模组"
    } else if stroke_mm >= 1000.0 || speed_mm_s >= 800.0 {
        "同步带直线模组"
    } else {
        "常规直线模组"
    }
}
