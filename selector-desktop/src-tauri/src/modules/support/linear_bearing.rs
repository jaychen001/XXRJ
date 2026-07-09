use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "linear-bearing-selector";
const SOURCE: &str = "工程公式库 / 直线轴承";

#[rustfmt::skip]
pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "直线轴承".to_string(),
        category: "支撑导向".to_string(),
        description: "按载荷、轴承数量、冲击系数、速度和目标行走寿命估算直线轴承 C 值需求。".to_string(),
        source_chapter: "直线轴承".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field("radialLoad", "总径向载荷", "N", 0.0, 200.0, "机构作用在直线轴承组上的径向载荷", SOURCE),
            common::field("bearingCount", "轴承数量", "pcs", 1.0, 2.0, "共同承载的直线轴承数量", SOURCE),
            common::field("loadDirectionFactor", "方向系数", "ratio", 0.1, 1.0, "安装方向和受力不均修正", SOURCE),
            common::field("impactFactor", "冲击系数", "ratio", 0.1, 1.5, "启停、偏载和冲击修正", SOURCE),
            common::field("shaftDiameter", "轴径", "mm", 1.0, 20.0, "导向轴直径", SOURCE),
            common::field_with_units("travelSpeed", "运行速度", "mm/s", &["mm/s", "m/s"], 0.0, 300.0, "直线运动速度", SOURCE),
            common::field("requiredTravelLife", "目标行走寿命", "km", 1.0, 5000.0, "期望行走寿命", SOURCE),
            common::field("loadRating", "额定动载荷 C", "N", 1.0, 1000.0, "候选直线轴承样册 C 值", SOURCE),
        ],
    }
}

#[rustfmt::skip]
pub fn calculate(request: &CalculationRequest) -> Result<CalculationResult, FieldError> {
    let module = definition();
    let source = module.source_page.clone();
    let fields = common::fields_map(request);
    let safety_factor = common::safety_factor(request)?;
    let load = common::positive(&fields, "radialLoad")?;
    let bearing_count = common::positive(&fields, "bearingCount")?;
    let direction_factor = common::positive(&fields, "loadDirectionFactor")?;
    let impact_factor = common::positive(&fields, "impactFactor")?;
    let shaft = common::positive(&fields, "shaftDiameter")?;
    let speed_m_s = common::convert(common::positive(&fields, "travelSpeed")?, common::unit(&fields, "travelSpeed")?, "m/s", "travelSpeed")?;
    let required_life = common::positive(&fields, "requiredTravelLife")?;
    let rating = common::positive(&fields, "loadRating")?;

    let design_load = load * direction_factor * impact_factor * safety_factor / bearing_count;
    let load_margin = rating / design_load;
    let rated_life_km = 50.0 * load_margin.powi(3);
    let required_rating = design_load * (required_life / 50.0).powf(1.0 / 3.0);
    let speed_index = speed_m_s * 60.0 / shaft;
    let recommendation = if speed_m_s <= 0.5 && rated_life_km >= required_life {
        "普通直线轴承或法兰型可初筛"
    } else if rated_life_km < required_life {
        "寿命不足，先提高 C 值或增加轴承数量"
    } else {
        "优先复核高速低摩擦型或直线导轨替代"
    };
    let mut risks = common::safety_risk(safety_factor, &source);

    if rated_life_km < required_life {
        risks.push(common::risk("warning", "估算行走寿命低于目标寿命，需提高规格或增加支撑点。", Some("loadRating"), &source));
    }
    if speed_m_s > 0.5 {
        risks.push(common::risk("warning", "运行速度超过 0.5 m/s，需复核润滑、保持架和发热。", Some("travelSpeed"), &source));
    }

    Ok(common::result(
        module,
        request,
        "linear-bearing-selector@0.2.0",
        format!("单轴承设计载荷 {} N，估算寿命 {} km", common::fmt(design_load), common::fmt(rated_life_km)),
        format!("候选直线轴承 C 值需不低于 {} N；当前载荷余量 {}。", common::fmt(required_rating), common::fmt(load_margin)),
        vec![
            common::step("设计载荷", "P = F * Kd * Ki * K / N", format!("{}*{}*{}*{}/{}", common::fmt(load), common::fmt(direction_factor), common::fmt(impact_factor), common::fmt(safety_factor), common::fmt(bearing_count)), design_load, "N", &source),
            common::step("载荷余量", "S = C / P", format!("{} / {}", common::fmt(rating), common::fmt(design_load)), load_margin, "ratio", &source),
            common::step("额定寿命", "L = 50 * (C/P)^3", format!("50 * {}^3", common::fmt(load_margin)), rated_life_km, "km", &source),
            common::step("所需动额定载荷", "Creq = P * (Lreq/50)^(1/3)", format!("{} * ({}/50)^(1/3)", common::fmt(design_load), common::fmt(required_life)), required_rating, "N", &source),
            common::step("速度指标", "Iv = v * 60 / d", format!("{}*60/{}", common::fmt(speed_m_s), common::fmt(shaft)), speed_index, "index", &source),
        ],
        vec![
            common::rule("linear-bearing-type", "类型判断", recommendation.to_string(), format!("速度 {} m/s，轴径 {} mm", common::fmt(speed_m_s), common::fmt(shaft)), if rated_life_km >= required_life { "low" } else { "warning" }, &source),
            common::rule("linear-bearing-life", "寿命匹配", if rated_life_km >= required_life { "候选直线轴承寿命满足目标。".to_string() } else { "候选直线轴承寿命不足，需提高 C 值。".to_string() }, format!("估算寿命 {} km，目标 {} km", common::fmt(rated_life_km), common::fmt(required_life)), if rated_life_km >= required_life { "low" } else { "warning" }, &source),
        ],
        risks,
        vec![
            common::requirement("designLoad", "设计载荷", design_load, "N"),
            common::requirement("loadMargin", "载荷余量", load_margin, "ratio"),
            common::requirement("ratedLife", "额定寿命", rated_life_km, "km"),
            common::requirement("requiredLoadRating", "所需动额定载荷", required_rating, "N"),
            common::requirement("speedIndex", "速度指标", speed_index, "index"),
        ],
    ))
}
