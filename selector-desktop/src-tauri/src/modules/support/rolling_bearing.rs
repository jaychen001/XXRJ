use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "rolling-bearing-life";
const SOURCE: &str = "工程公式库 / 滚动轴承";

#[rustfmt::skip]
pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "滚动轴承".to_string(),
        category: "支撑导向".to_string(),
        description: "按径向/轴向载荷、寿命目标和样册 C/C0 值估算滚动轴承动静载余量。".to_string(),
        source_chapter: "滚动轴承".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field("radialLoad", "径向载荷", "N", 0.0, 500.0, "轴承径向载荷 Fr", SOURCE),
            common::field("axialLoad", "轴向载荷", "N", 0.0, 100.0, "轴承轴向载荷 Fa", SOURCE),
            common::field("xFactor", "径向系数 X", "ratio", 0.0, 1.0, "等效动载荷径向系数，未知时先取 1", SOURCE),
            common::field("yFactor", "轴向系数 Y", "ratio", 0.0, 0.6, "等效动载荷轴向系数，未知时先取 0.6", SOURCE),
            common::field_with_units("shaftSpeed", "转速", "rpm", &["rpm", "rps"], 0.0, 600.0, "轴承工作转速", SOURCE),
            common::field("requiredLifeHours", "目标寿命", "h", 1.0, 10000.0, "期望 L10 寿命小时", SOURCE),
            common::field("dynamicLoadRating", "动额定载荷 C", "N", 1.0, 3000.0, "候选轴承样册 C 值", SOURCE),
            common::field("staticLoadRating", "静额定载荷 C0", "N", 1.0, 2000.0, "候选轴承样册 C0 值", SOURCE),
            common::field("lifeExponent", "寿命指数", "ratio", 1.0, 3.0, "球轴承取 3，滚子轴承取 3.333", SOURCE),
            common::field("applicationFactor", "工况系数", "ratio", 0.1, 1.2, "冲击、温升、润滑修正", SOURCE),
        ],
    }
}

#[rustfmt::skip]
pub fn calculate(request: &CalculationRequest) -> Result<CalculationResult, FieldError> {
    let module = definition();
    let source = module.source_page.clone();
    let fields = common::fields_map(request);
    let safety_factor = common::safety_factor(request)?;
    let radial = common::positive(&fields, "radialLoad")?;
    let axial = common::positive_or_zero(&fields, "axialLoad")?;
    let x_factor = common::positive_or_zero(&fields, "xFactor")?;
    let y_factor = common::positive_or_zero(&fields, "yFactor")?;
    let speed = common::convert(common::positive(&fields, "shaftSpeed")?, common::unit(&fields, "shaftSpeed")?, "rpm", "shaftSpeed")?;
    let required_life_hours = common::positive(&fields, "requiredLifeHours")?;
    let dynamic_rating = common::positive(&fields, "dynamicLoadRating")?;
    let static_rating = common::positive(&fields, "staticLoadRating")?;
    let life_exponent = common::positive(&fields, "lifeExponent")?;
    let application = common::positive(&fields, "applicationFactor")?;

    let equivalent_load = (x_factor * radial + y_factor * axial) * application * safety_factor;
    let static_equivalent_load = (radial + 0.5 * axial) * safety_factor;
    let load_ratio = dynamic_rating / equivalent_load;
    let life_million_rev = load_ratio.powf(life_exponent);
    let life_hours = life_million_rev * 1_000_000.0 / (60.0 * speed);
    let required_million_rev = required_life_hours * 60.0 * speed / 1_000_000.0;
    let required_dynamic_rating = equivalent_load * required_million_rev.powf(1.0 / life_exponent);
    let static_margin = static_rating / static_equivalent_load;
    let dynamic_margin = dynamic_rating / required_dynamic_rating;
    let mut risks = common::safety_risk(safety_factor, &source);

    if life_hours < required_life_hours {
        risks.push(common::risk("warning", "估算 L10 寿命低于目标寿命，需提高轴承规格或降低载荷。", Some("dynamicLoadRating"), &source));
    }
    if static_margin < 1.5 {
        risks.push(common::risk("warning", "静载余量低于 1.5，冲击或停机承载风险偏高。", Some("staticLoadRating"), &source));
    }

    Ok(common::result(
        module,
        request,
        "rolling-bearing-life@0.2.0",
        format!("等效动载荷 {} N，估算寿命 {} h", common::fmt(equivalent_load), common::fmt(life_hours)),
        format!("候选轴承 C 值需不低于 {} N；当前动载余量 {}，静载余量 {}。", common::fmt(required_dynamic_rating), common::fmt(dynamic_margin), common::fmt(static_margin)),
        vec![
            common::step("等效动载荷", "P = (XFr + YFa) * Ka * K", format!("({}*{} + {}*{})*{}*{}", common::fmt(x_factor), common::fmt(radial), common::fmt(y_factor), common::fmt(axial), common::fmt(application), common::fmt(safety_factor)), equivalent_load, "N", &source),
            common::step("额定寿命", "L10 = (C/P)^p", format!("({}/{})^{}", common::fmt(dynamic_rating), common::fmt(equivalent_load), common::fmt(life_exponent)), life_million_rev, "10⁶ rev", &source),
            common::step("寿命小时", "Lh = L10*10⁶/(60n)", format!("{}*10⁶/(60*{})", common::fmt(life_million_rev), common::fmt(speed)), life_hours, "h", &source),
            common::step("所需动额定载荷", "Creq = P * (Lreq*60n/10⁶)^(1/p)", format!("{} * ({}*60*{}/10⁶)^(1/{})", common::fmt(equivalent_load), common::fmt(required_life_hours), common::fmt(speed), common::fmt(life_exponent)), required_dynamic_rating, "N", &source),
            common::step("静载余量", "S0 = C0 / P0", format!("{} / {}", common::fmt(static_rating), common::fmt(static_equivalent_load)), static_margin, "ratio", &source),
        ],
        vec![
            common::rule("bearing-life-match", "寿命匹配", if life_hours >= required_life_hours { "候选轴承寿命满足目标，可继续复核极限转速和安装空间。".to_string() } else { "候选轴承寿命不足，需提高 C 值或减小载荷。".to_string() }, format!("估算寿命 {} h，目标 {} h", common::fmt(life_hours), common::fmt(required_life_hours)), if life_hours >= required_life_hours { "low" } else { "warning" }, &source),
            common::rule("bearing-static-match", "静载匹配", if static_margin >= 1.5 { "静载余量满足基础冲击复核。".to_string() } else { "静载余量不足，需复核 C0 或冲击载荷。".to_string() }, format!("静载余量 {}", common::fmt(static_margin)), if static_margin >= 1.5 { "low" } else { "warning" }, &source),
        ],
        risks,
        vec![
            common::requirement("equivalentLoad", "等效动载荷", equivalent_load, "N"),
            common::requirement("requiredDynamicLoadRating", "所需动额定载荷", required_dynamic_rating, "N"),
            common::requirement("lifeHours", "寿命小时", life_hours, "h"),
            common::requirement("staticMargin", "静载余量", static_margin, "ratio"),
        ],
    ))
}
