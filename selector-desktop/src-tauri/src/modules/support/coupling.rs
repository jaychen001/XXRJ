use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "coupling-selector";
const SOURCE: &str = "工程公式库 / 联轴器";

#[rustfmt::skip]
pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "联轴器".to_string(),
        category: "连接件".to_string(),
        description: "按额定/峰值扭矩、冲击、转速、惯量比和三向偏差判断联轴器余量与类型。".to_string(),
        source_chapter: "联轴器".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field("motorTorque", "电机额定扭矩", "Nm", 0.0, 2.0, "电机或减速机额定输出扭矩", SOURCE),
            common::field("peakTorque", "峰值扭矩", "Nm", 0.0, 4.0, "启动、急停或伺服峰值扭矩", SOURCE),
            common::field("ratedTorque", "联轴器额定扭矩", "Nm", 0.0, 10.0, "候选联轴器样册额定扭矩", SOURCE),
            common::field("shockFactor", "冲击系数", "ratio", 0.1, 1.5, "启停、冲击和反转修正", SOURCE),
            common::field("temperatureFactor", "温度修正系数", "ratio", 0.1, 1.0, "高温、环境和材料降额修正", SOURCE),
            common::field_with_units("shaftSpeed", "轴转速", "rpm", &["rpm", "rps"], 0.0, 1500.0, "联轴器工作转速", SOURCE),
            common::field("inertiaRatio", "负载惯量比", "ratio", 0.0, 3.0, "负载惯量/电机惯量", SOURCE),
            common::field("parallelOffset", "平行偏差", "mm", 0.0, 0.05, "两轴平行偏差需求", SOURCE),
            common::field("angularOffset", "角向偏差", "deg", 0.0, 0.5, "两轴角向偏差需求", SOURCE),
            common::field("axialOffset", "轴向位移", "mm", 0.0, 0.2, "轴向伸缩补偿需求", SOURCE),
        ],
    }
}

#[rustfmt::skip]
pub fn calculate(request: &CalculationRequest) -> Result<CalculationResult, FieldError> {
    let module = definition();
    let source = module.source_page.clone();
    let fields = common::fields_map(request);
    let safety_factor = common::safety_factor(request)?;
    let motor_torque = common::positive(&fields, "motorTorque")?;
    let peak_torque = common::positive_or_zero(&fields, "peakTorque")?;
    let rated_torque = common::positive_or_zero(&fields, "ratedTorque")?;
    let shock = common::positive(&fields, "shockFactor")?;
    let temperature = common::positive(&fields, "temperatureFactor")?;
    let speed = common::convert(common::positive(&fields, "shaftSpeed")?, common::unit(&fields, "shaftSpeed")?, "rpm", "shaftSpeed")?;
    let inertia_ratio = common::positive_or_zero(&fields, "inertiaRatio")?;
    let parallel_offset = common::positive_or_zero(&fields, "parallelOffset")?;
    let angular_offset = common::positive_or_zero(&fields, "angularOffset")?;
    let axial_offset = common::positive_or_zero(&fields, "axialOffset")?;

    let rated_design_torque = motor_torque * shock * temperature * safety_factor;
    let peak_design_torque = peak_torque * shock * temperature;
    let design_torque = rated_design_torque.max(peak_design_torque);
    let torque_margin = if rated_torque > 0.0 { rated_torque / design_torque } else { 0.0 };
    let torsional_index = design_torque * (1.0 + inertia_ratio);
    let misalignment_index = parallel_offset / 0.2 + angular_offset / 1.0 + axial_offset / 0.5;
    let recommendation = recommend_type(speed, inertia_ratio, parallel_offset, angular_offset, shock);
    let mut risks = common::safety_risk(safety_factor, &source);

    if inertia_ratio > 5.0 {
        risks.push(common::risk("warning", "负载惯量比过高，联轴器刚性和伺服调试风险上升。", Some("inertiaRatio"), &source));
    }
    if rated_torque > 0.0 && torque_margin < 1.2 {
        risks.push(common::risk("warning", "候选联轴器扭矩余量低于 1.2，建议提高规格或重新评估峰值扭矩。", Some("ratedTorque"), &source));
    }

    Ok(common::result(
        module,
        request,
        "coupling-selector@0.2.0",
        format!("设计扭矩 {} Nm，建议 {}", common::fmt(design_torque), recommendation),
        format!("联轴器额定扭矩至少 {} Nm；当前扭矩余量 {}，偏差指标 {}。", common::fmt(design_torque), common::fmt(torque_margin), common::fmt(misalignment_index)),
        vec![
            common::step("额定修正扭矩", "Tr = Tm * Ka * Kt * K", format!("{}*{}*{}*{}", common::fmt(motor_torque), common::fmt(shock), common::fmt(temperature), common::fmt(safety_factor)), rated_design_torque, "Nm", &source),
            common::step("峰值修正扭矩", "Tp = Tpeak * Ka * Kt", format!("{}*{}*{}", common::fmt(peak_torque), common::fmt(shock), common::fmt(temperature)), peak_design_torque, "Nm", &source),
            common::step("设计扭矩", "T = max(Tr, Tp)", format!("max({}, {})", common::fmt(rated_design_torque), common::fmt(peak_design_torque)), design_torque, "Nm", &source),
            common::step("扭矩余量", "M = Tcoupling / T", format!("{} / {}", common::fmt(rated_torque), common::fmt(design_torque)), torque_margin, "ratio", &source),
            common::step("扭转需求指标", "It = T * (1 + Jratio)", format!("{}*(1+{})", common::fmt(design_torque), common::fmt(inertia_ratio)), torsional_index, "index", &source),
            common::step("偏差指标", "Im = Δp/0.2 + Δa/1 + Δx/0.5", format!("{}/0.2 + {}/1 + {}/0.5", common::fmt(parallel_offset), common::fmt(angular_offset), common::fmt(axial_offset)), misalignment_index, "index", &source),
        ],
        vec![
            common::rule("coupling-type", "类型建议", recommendation.to_string(), format!("转速 {} rpm，惯量比 {}", common::fmt(speed), common::fmt(inertia_ratio)), "low", &source),
            common::rule("coupling-torque-margin", "扭矩余量", if rated_torque <= 0.0 { "未输入候选联轴器额定扭矩，先用设计扭矩作为样本库匹配条件。".to_string() } else if torque_margin >= 1.2 { "候选联轴器扭矩余量可进入偏差复核。".to_string() } else { "扭矩余量不足，需要上调联轴器规格。".to_string() }, format!("扭矩余量 {}", common::fmt(torque_margin)), if rated_torque <= 0.0 || torque_margin >= 1.2 { "low" } else { "warning" }, &source),
            common::rule("coupling-offset", "偏差补偿", "按样册复核平行、角向和轴向三类偏差，不能只看扭矩。".to_string(), format!("偏差指标 {}", common::fmt(misalignment_index)), if misalignment_index <= 1.0 { "low" } else { "warning" }, &source),
        ],
        risks,
        vec![
            common::requirement("designTorque", "设计扭矩", design_torque, "Nm"),
            common::requirement("torqueMargin", "扭矩余量", torque_margin, "ratio"),
            common::requirement("torsionalIndex", "扭转需求指标", torsional_index, "index"),
            common::requirement("shaftSpeed", "轴转速", speed, "rpm"),
        ],
    ))
}

fn recommend_type(
    speed: f64,
    inertia_ratio: f64,
    parallel_offset: f64,
    angular_offset: f64,
    shock: f64,
) -> &'static str {
    if speed > 1500.0 || inertia_ratio <= 3.0 {
        "膜片/波纹管联轴器"
    } else if shock > 1.8 || parallel_offset > 0.2 || angular_offset > 1.0 {
        "梅花弹性联轴器"
    } else {
        "刚性或弹性联轴器"
    }
}
