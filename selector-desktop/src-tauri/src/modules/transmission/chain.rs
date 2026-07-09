use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "chain-selector";
const SOURCE: &str = "工程公式库 / 链条";

#[rustfmt::skip]
pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "链条选型计算".to_string(),
        category: "传动".to_string(),
        description: "按节距、链轮、中心距、功率和候选额定功率估算链速、链节数和承载余量。".to_string(),
        source_chapter: "链条".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field_with_units("pitch", "节距", "mm", &["mm", "m"], 0.001, 12.7, "链条节距", SOURCE),
            common::field("smallSprocketTeeth", "小链轮齿数", "teeth", 1.0, 18.0, "主动小链轮齿数", SOURCE),
            common::field("largeSprocketTeeth", "大链轮齿数", "teeth", 1.0, 36.0, "从动大链轮齿数", SOURCE),
            common::field_with_units("centerDistance", "中心距", "mm", &["mm", "m"], 0.001, 500.0, "两链轮中心距", SOURCE),
            common::field_with_units("sprocketSpeed", "小链轮转速", "rpm", &["rpm", "rps"], 0.0, 200.0, "小链轮工作转速", SOURCE),
            common::field_with_units("transmitPower", "传递功率", "kW", &["kW", "W"], 0.0, 0.5, "链传动输入功率", SOURCE),
            common::field("serviceFactor", "工况系数", "ratio", 0.1, 1.4, "冲击、启停和润滑修正", SOURCE),
            common::field("strandCount", "链排数", "pcs", 1.0, 1.0, "单排、双排或多排链", SOURCE),
            common::field("candidatePowerRating", "单排额定功率", "kW", 0.0, 1.0, "候选链条单排样册额定功率", SOURCE),
        ],
    }
}

#[rustfmt::skip]
pub fn calculate(request: &CalculationRequest) -> Result<CalculationResult, FieldError> {
    let module = definition();
    let source = module.source_page.clone();
    let fields = common::fields_map(request);
    let safety_factor = common::safety_factor(request)?;
    let pitch_m = common::convert(common::positive(&fields, "pitch")?, common::unit(&fields, "pitch")?, "m", "pitch")?;
    let pitch_mm = common::convert(common::positive(&fields, "pitch")?, common::unit(&fields, "pitch")?, "mm", "pitch")?;
    let z1 = common::positive(&fields, "smallSprocketTeeth")?;
    let z2 = common::positive(&fields, "largeSprocketTeeth")?;
    let center_m = common::convert(common::positive(&fields, "centerDistance")?, common::unit(&fields, "centerDistance")?, "m", "centerDistance")?;
    let speed_rpm = common::convert(common::positive(&fields, "sprocketSpeed")?, common::unit(&fields, "sprocketSpeed")?, "rpm", "sprocketSpeed")?;
    let power_kw = common::convert(common::positive(&fields, "transmitPower")?, common::unit(&fields, "transmitPower")?, "kW", "transmitPower")?;
    let service_factor = common::positive(&fields, "serviceFactor")?;
    let strand_count = common::positive(&fields, "strandCount")?;
    let candidate_power = common::positive_or_zero(&fields, "candidatePowerRating")?;

    let ratio = z2 / z1;
    let chain_speed = pitch_m * z1 * speed_rpm / 60.0;
    let center_pitch = center_m / pitch_m;
    let chain_links = 2.0 * center_pitch + (z1 + z2) / 2.0 + (z2 - z1).powi(2) / (4.0 * std::f64::consts::PI.powi(2) * center_pitch);
    let design_power = power_kw * service_factor * safety_factor;
    let required_power_per_strand = design_power / strand_count;
    let power_margin = if required_power_per_strand > 0.0 { candidate_power / required_power_per_strand } else { 0.0 };
    let chain_pull = if chain_speed > 0.0 { design_power * 1000.0 / chain_speed } else { 0.0 };
    let mut risks = common::safety_risk(safety_factor, &source);

    if z1 < 17.0 {
        risks.push(common::risk("warning", "小链轮齿数偏少，链条多边形效应和振动风险上升。", Some("smallSprocketTeeth"), &source));
    }
    if candidate_power > 0.0 && power_margin < 1.2 {
        risks.push(common::risk("warning", "候选链条功率余量低于 1.2，建议提高链号或增加排数。", Some("candidatePowerRating"), &source));
    }

    Ok(common::result(
        module,
        request,
        "chain-selector@0.2.0",
        format!("链速 {} m/s，链节数约 {} 节", common::fmt(chain_speed), common::fmt(chain_links)),
        format!("按节距 {} mm 和齿数 {}/{} 计算，设计功率 {} kW，功率余量 {}。", common::fmt(pitch_mm), common::fmt(z1), common::fmt(z2), common::fmt(design_power), common::fmt(power_margin)),
        vec![
            common::step("传动比", "i = z2 / z1", format!("{} / {}", common::fmt(z2), common::fmt(z1)), ratio, "ratio", &source),
            common::step("链速", "v = p * z1 * n / 60", format!("{} * {} * {} / 60", common::fmt(pitch_m), common::fmt(z1), common::fmt(speed_rpm)), chain_speed, "m/s", &source),
            common::step("链节数", "Lp = 2C/p + (z1+z2)/2 + (z2-z1)^2/(4π²C/p)", format!("2*{} + ({}+{})/2 + ...", common::fmt(center_pitch), common::fmt(z1), common::fmt(z2)), chain_links, "links", &source),
            common::step("设计功率", "Pd = P * Ka * K", format!("{} * {} * {}", common::fmt(power_kw), common::fmt(service_factor), common::fmt(safety_factor)), design_power, "kW", &source),
            common::step("单排需求功率", "P1 = Pd / N", format!("{} / {}", common::fmt(design_power), common::fmt(strand_count)), required_power_per_strand, "kW", &source),
            common::step("功率余量", "M = Pr / P1", format!("{} / {}", common::fmt(candidate_power), common::fmt(required_power_per_strand)), power_margin, "ratio", &source),
            common::step("链条有效拉力", "F = 1000 * Pd / v", format!("1000 * {} / {}", common::fmt(design_power), common::fmt(chain_speed)), chain_pull, "N", &source),
        ],
        vec![
            common::rule("chain-teeth", "小链轮齿数", if z1 >= 17.0 { "小链轮齿数可进入常规链传动初筛。".to_string() } else { "小链轮齿数偏少，建议增大齿数或降低转速。".to_string() }, format!("小链轮齿数 {}", common::fmt(z1)), if z1 >= 17.0 { "low" } else { "warning" }, &source),
            common::rule("chain-speed", "链速判断", if chain_speed <= 8.0 { "链速适合进入常规滚子链样本匹配。".to_string() } else { "链速偏高，需复核润滑、冲击和链型。".to_string() }, format!("链速 {} m/s", common::fmt(chain_speed)), if chain_speed <= 8.0 { "low" } else { "warning" }, &source),
            common::rule("chain-power-margin", "功率余量", if candidate_power <= 0.0 { "未输入候选链条额定功率，先用单排需求功率作为样本匹配条件。".to_string() } else if power_margin >= 1.2 { "候选链条功率余量可进入润滑和中心距复核。".to_string() } else { "候选链条功率余量不足，需要提高链号或增加排数。".to_string() }, format!("功率余量 {}", common::fmt(power_margin)), if candidate_power <= 0.0 || power_margin >= 1.2 { "low" } else { "warning" }, &source),
        ],
        risks,
        vec![
            common::requirement("chainRatio", "传动比", ratio, "ratio"),
            common::requirement("chainSpeed", "链速", chain_speed, "m/s"),
            common::requirement("chainLinks", "链节数", chain_links, "links"),
            common::requirement("designPower", "设计功率", design_power, "kW"),
            common::requirement("chainPull", "链条有效拉力", chain_pull, "N"),
            common::requirement("powerMargin", "功率余量", power_margin, "ratio"),
        ],
    ))
}
