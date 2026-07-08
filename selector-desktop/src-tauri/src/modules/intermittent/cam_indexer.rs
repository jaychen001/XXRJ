use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "cam-indexer-sizing";
const SOURCE: &str = "PDF P59 / 文档页 56 / 凸轮分割器";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "凸轮分割器选型计算".to_string(),
        category: "间歇传动".to_string(),
        description: "按工位数、节拍、转角、惯量和负载扭矩估算分割器驱动需求。".to_string(),
        source_chapter: "凸轮分割器".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field(
                "stationCount",
                "工位数",
                "station",
                1.0,
                8.0,
                "转盘或机构工位数",
                SOURCE,
            ),
            common::field_with_units(
                "cycleTime",
                "循环时间",
                "s",
                &["s", "min"],
                0.001,
                2.0,
                "完成一工位分割的节拍",
                SOURCE,
            ),
            common::field(
                "indexAngle",
                "分割角度",
                "deg",
                0.1,
                45.0,
                "单次分割角度",
                SOURCE,
            ),
            common::field(
                "tableInertia",
                "负载惯量",
                "kg·m²",
                0.0,
                0.05,
                "转盘和工装折算惯量",
                SOURCE,
            ),
            common::field(
                "loadTorque",
                "外部负载扭矩",
                "Nm",
                0.0,
                5.0,
                "摩擦、偏载或工艺负载扭矩",
                SOURCE,
            ),
            common::field(
                "efficiency",
                "传动效率",
                "ratio",
                0.01,
                0.8,
                "0-1 之间的小数",
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
    let stations = common::positive(&fields, "stationCount")?;
    let cycle_time = common::convert(
        common::positive(&fields, "cycleTime")?,
        common::unit(&fields, "cycleTime")?,
        "s",
        "cycleTime",
    )?;
    let index_angle = common::positive(&fields, "indexAngle")?;
    let inertia = common::positive_or_zero(&fields, "tableInertia")?;
    let load_torque = common::positive_or_zero(&fields, "loadTorque")?;
    let efficiency = common::efficiency(&fields, "efficiency")?;
    let index_time = cycle_time * 0.5;
    let output_rpm = 60.0 / (cycle_time * stations);
    let angle_rad = index_angle.to_radians();
    let peak_speed = angle_rad / index_time;
    let angular_acceleration = peak_speed / (index_time / 2.0);
    let inertia_torque = inertia * angular_acceleration;
    let design_torque = (inertia_torque + load_torque) * safety_factor / efficiency;
    let mut risks = common::safety_risk(safety_factor, &source);
    if index_time < 0.25 {
        risks.push(common::risk(
            "warning",
            "分割时间低于 0.25 s，冲击和凸轮寿命风险较高。",
            Some("cycleTime"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "cam-indexer-sizing@0.1.0",
        format!(
            "输出转速 {} rpm，设计扭矩 {} Nm",
            common::fmt(output_rpm),
            common::fmt(design_torque)
        ),
        format!(
            "按 {} 工位、{} s 节拍估算，分割器输出侧需 {} Nm。",
            common::fmt(stations),
            common::fmt(cycle_time),
            common::fmt(design_torque)
        ),
        vec![
            common::step(
                "输出转速",
                "n = 60 / (tc * N)",
                format!(
                    "60 / ({} * {})",
                    common::fmt(cycle_time),
                    common::fmt(stations)
                ),
                output_rpm,
                "rpm",
                &source,
            ),
            common::step(
                "峰值角速度",
                "ω = θ / ti",
                format!("{} / {}", common::fmt(angle_rad), common::fmt(index_time)),
                peak_speed,
                "rad/s",
                &source,
            ),
            common::step(
                "角加速度",
                "α = ω / (ti/2)",
                format!(
                    "{} / ({} / 2)",
                    common::fmt(peak_speed),
                    common::fmt(index_time)
                ),
                angular_acceleration,
                "rad/s²",
                &source,
            ),
            common::step(
                "惯量扭矩",
                "Tj = J * α",
                format!(
                    "{} * {}",
                    common::fmt(inertia),
                    common::fmt(angular_acceleration)
                ),
                inertia_torque,
                "Nm",
                &source,
            ),
            common::step(
                "设计扭矩",
                "T = (Tj + TL) * K / η",
                format!(
                    "({} + {}) * {} / {}",
                    common::fmt(inertia_torque),
                    common::fmt(load_torque),
                    common::fmt(safety_factor),
                    common::fmt(efficiency)
                ),
                design_torque,
                "Nm",
                &source,
            ),
        ],
        vec![
            common::rule(
                "cam-indexer-station",
                "工位数",
                "按工位数和输出转速进入分割器型号表初筛。".to_string(),
                format!(
                    "{} 工位，输出转速 {} rpm",
                    common::fmt(stations),
                    common::fmt(output_rpm)
                ),
                "low",
                &source,
            ),
            common::rule(
                "cam-indexer-impact",
                "冲击风险",
                if index_time >= 0.25 {
                    "分割时间可进入常规风险复核。".to_string()
                } else {
                    "分割时间偏短，需复核凸轮曲线和驱动余量。".to_string()
                },
                format!("分割时间 {} s", common::fmt(index_time)),
                if index_time >= 0.25 { "low" } else { "warning" },
                &source,
            ),
        ],
        risks,
        vec![
            common::requirement("outputSpeed", "输出转速", output_rpm, "rpm"),
            common::requirement(
                "angularAcceleration",
                "角加速度",
                angular_acceleration,
                "rad/s²",
            ),
            common::requirement("designTorque", "设计扭矩", design_torque, "Nm"),
        ],
    ))
}
