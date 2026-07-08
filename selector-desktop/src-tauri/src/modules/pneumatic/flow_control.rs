use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "pneumatic-flow-control";
const SOURCE: &str = "PDF P88 / 文档页 85 / 气动控制";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "电磁阀".to_string(),
        category: "气动".to_string(),
        description: "按缸径、行程、节拍和压力估算耗气量，并给出调速阀/电磁阀规则。".to_string(),
        source_chapter: "气动控制（调速阀）".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field_with_units(
                "cylinderBore",
                "气缸缸径",
                "mm",
                &["mm", "m"],
                0.001,
                32.0,
                "气缸内径",
                SOURCE,
            ),
            common::field_with_units(
                "stroke",
                "行程",
                "mm",
                &["mm", "m"],
                0.001,
                100.0,
                "单次动作行程",
                SOURCE,
            ),
            common::field_with_units(
                "cycleTime",
                "循环时间",
                "s",
                &["s", "min"],
                0.001,
                2.0,
                "完成一次往返动作的时间",
                SOURCE,
            ),
            common::field(
                "workingPressure",
                "工作压力",
                "MPa",
                0.01,
                0.5,
                "气源压力",
                SOURCE,
            ),
            common::field(
                "actuationFrequency",
                "动作频率",
                "cycle/min",
                0.0,
                20.0,
                "每分钟动作次数",
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
    let bore_m = common::convert(
        common::positive(&fields, "cylinderBore")?,
        common::unit(&fields, "cylinderBore")?,
        "m",
        "cylinderBore",
    )?;
    let stroke_m = common::convert(
        common::positive(&fields, "stroke")?,
        common::unit(&fields, "stroke")?,
        "m",
        "stroke",
    )?;
    let cycle_time = common::convert(
        common::positive(&fields, "cycleTime")?,
        common::unit(&fields, "cycleTime")?,
        "s",
        "cycleTime",
    )?;
    let pressure = common::positive(&fields, "workingPressure")?;
    let frequency = common::positive_or_zero(&fields, "actuationFrequency")?;
    let chamber_volume_l = std::f64::consts::PI * bore_m.powi(2) / 4.0 * stroke_m * 1000.0;
    let free_air_factor = pressure / 0.101 + 1.0;
    let air_per_cycle_l = chamber_volume_l * 2.0 * free_air_factor * safety_factor;
    let flow_l_min = air_per_cycle_l * 60.0 / cycle_time;
    let mut risks = common::safety_risk(safety_factor, &source);
    if flow_l_min > 200.0 || frequency > 60.0 {
        risks.push(common::risk(
            "warning",
            "耗气量或动作频率偏高，需复核阀口径、管径和排气能力。",
            Some("actuationFrequency"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "pneumatic-flow-control@0.1.0",
        format!(
            "自由空气耗量 {} L/min，单循环 {} L",
            common::fmt(flow_l_min),
            common::fmt(air_per_cycle_l)
        ),
        "优先按排气节流调速阀控制速度，电磁阀按流量和响应时间匹配样册。".to_string(),
        vec![
            common::step(
                "单腔容积",
                "V = πD²/4 * S",
                format!("π*{}²/4*{}", common::fmt(bore_m), common::fmt(stroke_m)),
                chamber_volume_l,
                "L",
                &source,
            ),
            common::step(
                "单循环耗气",
                "Qa = 2V * (P/0.101+1) * K",
                format!(
                    "2*{}*({}/0.101+1)*{}",
                    common::fmt(chamber_volume_l),
                    common::fmt(pressure),
                    common::fmt(safety_factor)
                ),
                air_per_cycle_l,
                "L",
                &source,
            ),
            common::step(
                "流量需求",
                "Q = Qa * 60 / t",
                format!(
                    "{}*60/{}",
                    common::fmt(air_per_cycle_l),
                    common::fmt(cycle_time)
                ),
                flow_l_min,
                "L/min",
                &source,
            ),
        ],
        vec![
            common::rule(
                "flow-control-mode",
                "调速方式",
                "气缸速度控制优先排气节流，垂直负载需加止回或中封回路。".to_string(),
                format!("流量 {} L/min", common::fmt(flow_l_min)),
                "low",
                &source,
            ),
            common::rule(
                "valve-size",
                "阀口径初筛",
                "按计算流量上取电磁阀 Cv/有效截面积，并校验响应时间。".to_string(),
                format!("动作频率 {} cycle/min", common::fmt(frequency)),
                "low",
                &source,
            ),
        ],
        risks,
        vec![
            common::requirement("airPerCycle", "单循环耗气", air_per_cycle_l, "L"),
            common::requirement("flowRate", "流量需求", flow_l_min, "L/min"),
        ],
    ))
}
