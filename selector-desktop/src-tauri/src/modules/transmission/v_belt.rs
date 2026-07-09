use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "v-belt-selector";
const SOURCE: &str = "工程公式库 / V 带";

#[rustfmt::skip]
pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "V 带选型计算".to_string(),
        category: "传动".to_string(),
        description: "按功率、带轮直径、转速、修正系数和候选带功率判断 V 带规格余量。".to_string(),
        source_chapter: "V 带".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field_with_units("transmitPower", "传递功率", "kW", &["kW", "W"], 0.0, 0.75, "电机或负载侧功率", SOURCE),
            common::field_with_units("smallPulleyDiameter", "小带轮直径", "mm", &["mm", "m"], 0.001, 100.0, "主动小带轮节圆直径", SOURCE),
            common::field_with_units("smallPulleySpeed", "小带轮转速", "rpm", &["rpm", "rps"], 0.0, 1450.0, "主动小带轮转速", SOURCE),
            common::field("serviceFactor", "工况系数", "ratio", 0.1, 1.3, "冲击、启停和工作时长修正", SOURCE),
            common::field("arcCorrectionFactor", "包角修正系数", "ratio", 0.1, 0.95, "小带轮包角修正，未知时先取 0.95", SOURCE),
            common::field("lengthCorrectionFactor", "带长修正系数", "ratio", 0.1, 1.0, "带长修正，未知时先取 1", SOURCE),
            common::field("beltCount", "皮带根数", "pcs", 1.0, 1.0, "并联 V 带根数", SOURCE),
            common::field("candidateBeltPower", "单根额定功率", "kW", 0.0, 1.5, "候选单根 V 带样册额定功率", SOURCE),
            common::field("beltEfficiency", "传动效率", "ratio", 0.01, 0.95, "0-1 之间的小数", SOURCE),
        ],
    }
}

#[rustfmt::skip]
pub fn calculate(request: &CalculationRequest) -> Result<CalculationResult, FieldError> {
    let module = definition();
    let source = module.source_page.clone();
    let fields = common::fields_map(request);
    let safety_factor = common::safety_factor(request)?;
    let power_kw = common::convert(common::positive(&fields, "transmitPower")?, common::unit(&fields, "transmitPower")?, "kW", "transmitPower")?;
    let diameter_m = common::convert(common::positive(&fields, "smallPulleyDiameter")?, common::unit(&fields, "smallPulleyDiameter")?, "m", "smallPulleyDiameter")?;
    let speed_rpm = common::convert(common::positive(&fields, "smallPulleySpeed")?, common::unit(&fields, "smallPulleySpeed")?, "rpm", "smallPulleySpeed")?;
    let service_factor = common::positive(&fields, "serviceFactor")?;
    let arc_factor = common::positive(&fields, "arcCorrectionFactor")?;
    let length_factor = common::positive(&fields, "lengthCorrectionFactor")?;
    let belt_count = common::positive(&fields, "beltCount")?;
    let candidate_power = common::positive_or_zero(&fields, "candidateBeltPower")?;
    let efficiency = common::efficiency(&fields, "beltEfficiency")?;

    let belt_speed = std::f64::consts::PI * diameter_m * speed_rpm / 60.0;
    let design_power = power_kw * service_factor * safety_factor / efficiency;
    let corrected_capacity_per_belt = candidate_power * arc_factor * length_factor;
    let required_power_per_belt = design_power / belt_count;
    let power_margin = if required_power_per_belt > 0.0 { corrected_capacity_per_belt / required_power_per_belt } else { 0.0 };
    let effective_pull = if belt_speed > 0.0 { design_power * 1000.0 / belt_speed } else { 0.0 };
    let belt_type = recommend_type(design_power, belt_speed);
    let mut risks = common::safety_risk(safety_factor, &source);

    if !(5.0..=25.0).contains(&belt_speed) {
        risks.push(common::risk("warning", "带速不在 5-25 m/s 常用初筛区间，需复核带型和带轮直径。", Some("smallPulleyDiameter"), &source));
    }
    if candidate_power > 0.0 && power_margin < 1.1 {
        risks.push(common::risk("warning", "候选 V 带功率余量低于 1.1，建议增加根数或提高带型。", Some("candidateBeltPower"), &source));
    }

    Ok(common::result(
        module,
        request,
        "v-belt-selector@0.2.0",
        format!("设计功率 {} kW，带速 {} m/s", common::fmt(design_power), common::fmt(belt_speed)),
        format!("按安全系数 {} 计算，建议从 {} V 带开始匹配；当前单根功率余量 {}。", common::fmt(safety_factor), belt_type, common::fmt(power_margin)),
        vec![
            common::step("带速", "v = π * D * n / 60", format!("π * {} * {} / 60", common::fmt(diameter_m), common::fmt(speed_rpm)), belt_speed, "m/s", &source),
            common::step("设计功率", "Pd = P * Ka * K / η", format!("{} * {} * {} / {}", common::fmt(power_kw), common::fmt(service_factor), common::fmt(safety_factor), common::fmt(efficiency)), design_power, "kW", &source),
            common::step("单根需求功率", "P1 = Pd / N", format!("{} / {}", common::fmt(design_power), common::fmt(belt_count)), required_power_per_belt, "kW", &source),
            common::step("修正额定功率", "Pr = Pc * Cθ * CL", format!("{} * {} * {}", common::fmt(candidate_power), common::fmt(arc_factor), common::fmt(length_factor)), corrected_capacity_per_belt, "kW", &source),
            common::step("功率余量", "M = Pr / P1", format!("{} / {}", common::fmt(corrected_capacity_per_belt), common::fmt(required_power_per_belt)), power_margin, "ratio", &source),
            common::step("有效拉力", "Fe = 1000 * Pd / v", format!("1000 * {} / {}", common::fmt(design_power), common::fmt(belt_speed)), effective_pull, "N", &source),
        ],
        vec![
            common::rule("v-belt-type", "带型建议", format!("优先匹配 {} V 带", belt_type), format!("设计功率 {} kW，带速 {} m/s", common::fmt(design_power), common::fmt(belt_speed)), "low", &source),
            common::rule("v-belt-speed", "带速区间", if (5.0..=25.0).contains(&belt_speed) { "带速处于常用初筛区间。".to_string() } else { "带速偏离常用区间，先复核带轮直径和转速。".to_string() }, format!("带速 {} m/s", common::fmt(belt_speed)), if (5.0..=25.0).contains(&belt_speed) { "low" } else { "warning" }, &source),
            common::rule("v-belt-power-margin", "功率余量", if candidate_power <= 0.0 { "未输入候选单根额定功率，先用单根需求功率作为样本匹配条件。".to_string() } else if power_margin >= 1.1 { "候选 V 带功率余量可进入包角和张紧复核。".to_string() } else { "候选 V 带功率余量不足，需要增加根数或换带型。".to_string() }, format!("功率余量 {}", common::fmt(power_margin)), if candidate_power <= 0.0 || power_margin >= 1.1 { "low" } else { "warning" }, &source),
        ],
        risks,
        vec![
            common::requirement("designPower", "设计功率", design_power, "kW"),
            common::requirement("beltSpeed", "带速", belt_speed, "m/s"),
            common::requirement("requiredPowerPerBelt", "单根需求功率", required_power_per_belt, "kW"),
            common::requirement("powerMargin", "功率余量", power_margin, "ratio"),
            common::requirement("effectivePull", "有效拉力", effective_pull, "N"),
        ],
    ))
}

fn recommend_type(power_kw: f64, belt_speed: f64) -> &'static str {
    if power_kw <= 1.5 && belt_speed <= 18.0 {
        "A 型"
    } else if power_kw <= 7.5 {
        "B 型"
    } else {
        "C 型或多根并联"
    }
}
