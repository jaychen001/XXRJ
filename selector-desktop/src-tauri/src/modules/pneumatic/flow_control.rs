use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "pneumatic-flow-control";
const SOURCE: &str = "工程公式库 / 气动控制";

#[rustfmt::skip]
pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "电磁阀".to_string(),
        category: "气动".to_string(),
        description: "按气缸、管路、节拍和压力估算耗气量，输出电磁阀额定流量需求和余量。".to_string(),
        source_chapter: "气动控制 / 电磁阀".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field("cylinderCount", "气缸数量", "pcs", 1.0, 1.0, "同一阀组同时动作的气缸数量", SOURCE),
            common::field_with_units("cylinderBore", "气缸缸径", "mm", &["mm", "m"], 0.001, 32.0, "气缸内径", SOURCE),
            common::field_with_units("rodDiameter", "活塞杆直径", "mm", &["mm", "m"], 0.0, 12.0, "无杆腔或忽略回程差异时可填 0", SOURCE),
            common::field_with_units("stroke", "行程", "mm", &["mm", "m"], 0.001, 100.0, "单次动作行程", SOURCE),
            common::field_with_units("tubeInnerDiameter", "管路内径", "mm", &["mm", "m"], 0.0, 6.0, "阀到气缸的单侧管路内径", SOURCE),
            common::field_with_units("tubeLength", "管路长度", "mm", &["mm", "m"], 0.0, 1000.0, "阀到气缸的单侧管路长度", SOURCE),
            common::field_with_units("cycleTime", "循环时间", "s", &["s", "min"], 0.001, 2.0, "完成一次往返动作的时间", SOURCE),
            common::field("workingPressure", "工作压力", "MPa", 0.01, 0.5, "气源表压", SOURCE),
            common::field("actuationFrequency", "动作频率", "cycle/min", 0.0, 20.0, "每分钟往返循环次数", SOURCE),
            common::field("valveRatedFlow", "阀额定流量", "L/min", 0.0, 200.0, "候选电磁阀样本标称自由空气流量", SOURCE),
        ],
    }
}

#[rustfmt::skip]
pub fn calculate(request: &CalculationRequest) -> Result<CalculationResult, FieldError> {
    let module = definition();
    let source = module.source_page.clone();
    let fields = common::fields_map(request);
    let safety_factor = common::safety_factor(request)?;
    let cylinder_count = common::positive(&fields, "cylinderCount")?;
    let bore_m = common::convert(common::positive(&fields, "cylinderBore")?, common::unit(&fields, "cylinderBore")?, "m", "cylinderBore")?;
    let rod_m = common::convert(common::positive_or_zero(&fields, "rodDiameter")?, common::unit(&fields, "rodDiameter")?, "m", "rodDiameter")?;
    let stroke_m = common::convert(common::positive(&fields, "stroke")?, common::unit(&fields, "stroke")?, "m", "stroke")?;
    let tube_id_m = common::convert(common::positive_or_zero(&fields, "tubeInnerDiameter")?, common::unit(&fields, "tubeInnerDiameter")?, "m", "tubeInnerDiameter")?;
    let tube_length_m = common::convert(common::positive_or_zero(&fields, "tubeLength")?, common::unit(&fields, "tubeLength")?, "m", "tubeLength")?;
    let cycle_time = common::convert(common::positive(&fields, "cycleTime")?, common::unit(&fields, "cycleTime")?, "s", "cycleTime")?;
    let pressure = common::positive(&fields, "workingPressure")?;
    let frequency = common::positive_or_zero(&fields, "actuationFrequency")?;
    let valve_rated_flow = common::positive_or_zero(&fields, "valveRatedFlow")?;

    if rod_m >= bore_m {
        return Err(FieldError { field_id: "rodDiameter".to_string(), message: "活塞杆直径必须小于气缸缸径".to_string() });
    }

    let bore_area = std::f64::consts::PI * bore_m.powi(2) / 4.0;
    let rod_area = std::f64::consts::PI * rod_m.powi(2) / 4.0;
    let extend_volume_l = bore_area * stroke_m * 1000.0;
    let retract_volume_l = (bore_area - rod_area) * stroke_m * 1000.0;
    let tube_volume_l = std::f64::consts::PI * tube_id_m.powi(2) / 4.0 * tube_length_m * 1000.0;
    let free_air_factor = (pressure + 0.101325) / 0.101325;
    let air_per_cycle_l = (extend_volume_l + retract_volume_l + tube_volume_l * 2.0) * free_air_factor * cylinder_count * safety_factor;
    let frequency_by_time = 60.0 / cycle_time;
    let peak_flow_l_min = air_per_cycle_l * frequency_by_time;
    let continuous_flow_l_min = air_per_cycle_l * frequency;
    let valve_margin = if peak_flow_l_min > 0.0 { valve_rated_flow / peak_flow_l_min } else { 0.0 };
    let mut risks = common::safety_risk(safety_factor, &source);

    if peak_flow_l_min > 200.0 || frequency > 60.0 {
        risks.push(common::risk("warning", "耗气量或动作频率偏高，需复核阀口径、管径和排气能力。", Some("actuationFrequency"), &source));
    }
    if valve_rated_flow > 0.0 && valve_margin < 1.2 {
        risks.push(common::risk("warning", "候选电磁阀额定流量余量低于 1.2，建议上调阀规格或缩短管路。", Some("valveRatedFlow"), &source));
    }
    if frequency > 0.0 && (frequency - frequency_by_time).abs() / frequency_by_time > 0.2 {
        risks.push(common::risk("warning", "动作频率与循环时间不一致，报告同时给出峰值流量和持续耗气量。", Some("actuationFrequency"), &source));
    }

    Ok(common::result(
        module,
        request,
        "pneumatic-flow-control@0.2.0",
        format!("阀流量需求 {} L/min，单循环耗气 {} L", common::fmt(peak_flow_l_min), common::fmt(air_per_cycle_l)),
        "电磁阀按峰值自由空气流量上取规格；调速阀优先排气节流，垂直负载需增加保持回路。".to_string(),
        vec![
            common::step("单腔容积", "V1 = πD²/4 * S", format!("π*{}²/4*{}", common::fmt(bore_m), common::fmt(stroke_m)), extend_volume_l, "L", &source),
            common::step("回程腔容积", "V2 = (πD²/4 - πd²/4) * S", format!("({} - {})*{}", common::fmt(bore_area), common::fmt(rod_area), common::fmt(stroke_m)), retract_volume_l, "L", &source),
            common::step("管路容积", "Vp = πdi²/4 * L", format!("π*{}²/4*{}", common::fmt(tube_id_m), common::fmt(tube_length_m)), tube_volume_l, "L", &source),
            common::step("单循环耗气", "Qa = (V1 + V2 + 2Vp) * (Pabs/Patm) * N * K", format!("({} + {} + 2*{}) * {} * {} * {}", common::fmt(extend_volume_l), common::fmt(retract_volume_l), common::fmt(tube_volume_l), common::fmt(free_air_factor), common::fmt(cylinder_count), common::fmt(safety_factor)), air_per_cycle_l, "L", &source),
            common::step("流量需求", "Qpeak = Qa * 60 / tc", format!("{} * 60 / {}", common::fmt(air_per_cycle_l), common::fmt(cycle_time)), peak_flow_l_min, "L/min", &source),
            common::step("持续耗气量", "Qavg = Qa * f", format!("{} * {}", common::fmt(air_per_cycle_l), common::fmt(frequency)), continuous_flow_l_min, "L/min", &source),
            common::step("阀流量余量", "M = Qrated / Qpeak", format!("{} / {}", common::fmt(valve_rated_flow), common::fmt(peak_flow_l_min)), valve_margin, "ratio", &source),
        ],
        vec![
            common::rule("flow-control-mode", "调速方式", "气缸速度控制优先排气节流，垂直负载需加止回或中封回路。".to_string(), format!("峰值流量 {} L/min", common::fmt(peak_flow_l_min)), "low", &source),
            common::rule("valve-size", "阀口径初筛", if valve_rated_flow <= 0.0 { "未输入候选阀额定流量，先用流量需求作为样本库匹配条件。".to_string() } else if valve_margin >= 1.2 { "候选阀额定流量有基础余量，可继续复核响应时间和管径。".to_string() } else { "候选阀额定流量不足，建议提高阀规格或拆分阀组。".to_string() }, format!("额定流量 {} L/min，余量 {}", common::fmt(valve_rated_flow), common::fmt(valve_margin)), if valve_rated_flow <= 0.0 || valve_margin >= 1.2 { "low" } else { "warning" }, &source),
        ],
        risks,
        vec![
            common::requirement("airPerCycle", "单循环耗气", air_per_cycle_l, "L"),
            common::requirement("flowRate", "阀流量需求", peak_flow_l_min, "L/min"),
            common::requirement("continuousFlow", "持续耗气量", continuous_flow_l_min, "L/min"),
            common::requirement("valveMargin", "阀流量余量", valve_margin, "ratio"),
        ],
    ))
}
