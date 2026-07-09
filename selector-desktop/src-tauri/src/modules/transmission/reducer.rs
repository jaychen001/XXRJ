use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "reducer-basic";
const SOURCE: &str = "工程公式库 / 减速机";

#[rustfmt::skip]
pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "减速机基础计算".to_string(),
        category: "传动".to_string(),
        description: "按输出扭矩、工况系数、效率和候选样本参数复核减速比、输入扭矩和承载余量。".to_string(),
        source_chapter: "减速机".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field_with_units("motorSpeed", "电机转速", "rpm", &["rpm", "rps"], 0.0, 1500.0, "电机额定或工作转速", SOURCE),
            common::field_with_units("outputSpeed", "输出转速", "rpm", &["rpm", "rps"], 0.0, 60.0, "机构输出轴需求转速", SOURCE),
            common::field("loadTorque", "输出负载扭矩", "Nm", 0.0, 20.0, "工作机构折算到减速机输出轴的负载扭矩", SOURCE),
            common::field("serviceFactor", "工况系数", "ratio", 0.0, 1.5, "按冲击、启停频率和工作时长人工确认", SOURCE),
            common::field("efficiency", "减速机效率", "ratio", 0.01, 0.85, "0-1 之间的小数", SOURCE),
            common::field("candidateRatedTorque", "候选额定输出扭矩", "Nm", 0.0, 60.0, "样本中的额定输出扭矩", SOURCE),
            common::field("actualRadialLoad", "输出轴径向载荷", "N", 0.0, 300.0, "链轮、带轮、齿轮或联轴器对输出轴形成的径向载荷", SOURCE),
            common::field("allowableRadialLoad", "允许径向载荷", "N", 0.0, 900.0, "样本允许输出轴径向载荷", SOURCE),
            common::field("actualAxialLoad", "输出轴轴向载荷", "N", 0.0, 80.0, "斜齿轮、丝杆或安装预紧形成的轴向载荷", SOURCE),
            common::field("allowableAxialLoad", "允许轴向载荷", "N", 0.0, 300.0, "样本允许输出轴轴向载荷", SOURCE),
            common::field_with_units("maxInputSpeed", "允许输入转速", "rpm", &["rpm", "rps"], 0.0, 3000.0, "候选减速机允许最高输入转速", SOURCE),
        ],
    }
}

#[rustfmt::skip]
pub fn calculate(request: &CalculationRequest) -> Result<CalculationResult, FieldError> {
    let module = definition();
    let source = module.source_page.clone();
    let fields = common::fields_map(request);
    let safety_factor = common::safety_factor(request)?;
    let motor_speed = common::convert(common::positive(&fields, "motorSpeed")?, common::unit(&fields, "motorSpeed")?, "rpm", "motorSpeed")?;
    let output_speed = common::convert(common::positive(&fields, "outputSpeed")?, common::unit(&fields, "outputSpeed")?, "rpm", "outputSpeed")?;
    let load_torque = common::positive(&fields, "loadTorque")?;
    let service_factor = common::positive(&fields, "serviceFactor")?;
    let efficiency = common::efficiency(&fields, "efficiency")?;
    let rated_torque = common::positive(&fields, "candidateRatedTorque")?;
    let radial_load = common::positive(&fields, "actualRadialLoad")?;
    let allowable_radial = common::positive(&fields, "allowableRadialLoad")?;
    let axial_load = common::positive(&fields, "actualAxialLoad")?;
    let allowable_axial = common::positive(&fields, "allowableAxialLoad")?;
    let max_input_speed = common::convert(common::positive(&fields, "maxInputSpeed")?, common::unit(&fields, "maxInputSpeed")?, "rpm", "maxInputSpeed")?;

    let ratio = motor_speed / output_speed;
    let output_torque = load_torque * service_factor * safety_factor;
    let input_torque = output_torque / ratio / efficiency;
    let output_power = output_torque * output_speed * std::f64::consts::TAU / 60.0;
    let torque_margin = rated_torque / output_torque;
    let radial_margin = allowable_radial / radial_load;
    let axial_margin = allowable_axial / axial_load;
    let input_speed_margin = max_input_speed / motor_speed;
    let mut risks = common::safety_risk(safety_factor, &source);

    if ratio > 100.0 { risks.push(common::risk("warning", "减速比超过 100，单台减速机初筛风险高，建议拆成多级或组合传动。", Some("outputSpeed"), &source)); }
    if efficiency < 0.65 { risks.push(common::risk("warning", "减速机效率低于 0.65，电机侧扭矩和发热风险需要复核。", Some("efficiency"), &source)); }
    if torque_margin < 1.2 { risks.push(common::risk("warning", "候选减速机额定输出扭矩余量低于 1.2，建议提高规格或重新确认工况系数。", Some("candidateRatedTorque"), &source)); }
    if radial_margin < 1.2 { risks.push(common::risk("warning", "输出轴径向载荷余量低于 1.2，需复核带轮/链轮悬臂距离或提高轴承规格。", Some("actualRadialLoad"), &source)); }
    if axial_margin < 1.2 { risks.push(common::risk("warning", "输出轴轴向载荷余量低于 1.2，需复核安装方式和允许轴向力。", Some("actualAxialLoad"), &source)); }
    if input_speed_margin < 1.0 { risks.push(common::risk("warning", "电机转速超过候选减速机允许输入转速。", Some("maxInputSpeed"), &source)); }

    Ok(common::result(
        module,
        request,
        "reducer-basic@0.2.0",
        format!("减速比 {}，输出扭矩 {} Nm，扭矩余量 {}", common::fmt(ratio), common::fmt(output_torque), common::fmt(torque_margin)),
        format!("按安全系数 {}、工况系数 {} 和效率 {} 计算，减速机输出侧至少需要 {} Nm，电机侧扭矩约 {} Nm。",
            common::fmt(safety_factor), common::fmt(service_factor), common::fmt(efficiency), common::fmt(output_torque), common::fmt(input_torque)),
        vec![
            common::step("减速比", "i = n1 / n2", format!("{} / {}", common::fmt(motor_speed), common::fmt(output_speed)), ratio, "ratio", &source),
            common::step("设计输出扭矩", "T2 = TL * Ka * K", format!("{} * {} * {}", common::fmt(load_torque), common::fmt(service_factor), common::fmt(safety_factor)), output_torque, "Nm", &source),
            common::step("输入扭矩", "T1 = T2 / i / η", format!("{} / {} / {}", common::fmt(output_torque), common::fmt(ratio), common::fmt(efficiency)), input_torque, "Nm", &source),
            common::step("输出功率", "P = T2 * 2πn2 / 60", format!("{} * 2π * {} / 60", common::fmt(output_torque), common::fmt(output_speed)), output_power, "W", &source),
            common::step("扭矩余量", "M = Tr / T2", format!("{} / {}", common::fmt(rated_torque), common::fmt(output_torque)), torque_margin, "ratio", &source),
            common::step("径向载荷余量", "Mr = Fr_allow / Fr", format!("{} / {}", common::fmt(allowable_radial), common::fmt(radial_load)), radial_margin, "ratio", &source),
            common::step("轴向载荷余量", "Ma = Fa_allow / Fa", format!("{} / {}", common::fmt(allowable_axial), common::fmt(axial_load)), axial_margin, "ratio", &source),
            common::step("输入转速余量", "Mn = nmax / n1", format!("{} / {}", common::fmt(max_input_speed), common::fmt(motor_speed)), input_speed_margin, "ratio", &source),
        ],
        vec![
            common::rule("reducer-ratio", "减速比区间", if ratio <= 100.0 { "减速比可进入标准减速机样本匹配。".to_string() } else { "减速比偏大，优先拆级或改组合减速方案。".to_string() }, format!("计算减速比 {}", common::fmt(ratio)), if ratio <= 100.0 { "low" } else { "warning" }, &source),
            common::rule("reducer-torque-margin", "扭矩余量", if torque_margin >= 1.2 { "候选额定输出扭矩满足基础余量。".to_string() } else { "候选额定输出扭矩不足，需提高规格。".to_string() }, format!("扭矩余量 {}", common::fmt(torque_margin)), if torque_margin >= 1.2 { "low" } else { "warning" }, &source),
            common::rule("reducer-overhung-load", "输出轴载荷", if radial_margin >= 1.2 && axial_margin >= 1.2 { "径向和轴向载荷余量满足基础初筛。".to_string() } else { "输出轴载荷余量不足，需复核悬臂距离和安装方式。".to_string() }, format!("径向余量 {}，轴向余量 {}", common::fmt(radial_margin), common::fmt(axial_margin)), if radial_margin >= 1.2 && axial_margin >= 1.2 { "low" } else { "warning" }, &source),
        ],
        risks,
        vec![
            common::requirement("reducerRatio", "减速比", ratio, "ratio"),
            common::requirement("outputTorque", "设计输出扭矩", output_torque, "Nm"),
            common::requirement("inputTorque", "输入扭矩", input_torque, "Nm"),
            common::requirement("outputPower", "输出功率", output_power, "W"),
            common::requirement("torqueMargin", "扭矩余量", torque_margin, "ratio"),
            common::requirement("radialLoadMargin", "径向载荷余量", radial_margin, "ratio"),
            common::requirement("axialLoadMargin", "轴向载荷余量", axial_margin, "ratio"),
        ],
    ))
}
