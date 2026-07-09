use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "cam-indexer-sizing";
const SOURCE: &str = "工程公式库 / 凸轮分割器";

#[rustfmt::skip]
pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "凸轮分割器选型计算".to_string(),
        category: "间歇传动".to_string(),
        description: "按工位数、节拍、分割时间、惯量和负载扭矩估算分割器驱动需求。".to_string(),
        source_chapter: "凸轮分割器".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field("stationCount", "工位数", "station", 1.0, 8.0, "转盘或机构工位数", SOURCE),
            common::field_with_units("cycleTime", "循环时间", "s", &["s", "min"], 0.001, 2.0, "完成一工位循环的总节拍", SOURCE),
            common::field_with_units("indexTime", "分割时间", "s", &["s", "min"], 0.001, 0.8, "实际转位运动时间，不含停歇时间", SOURCE),
            common::field("indexAngle", "分割角度", "deg", 0.1, 45.0, "单次分割角度", SOURCE),
            common::field("motionCoefficient", "运动曲线系数", "ratio", 0.1, 4.0, "用于峰值角加速度估算，未知时可先取 4", SOURCE),
            common::field("tableInertia", "负载惯量", "kg·m²", 0.0, 0.05, "转盘和工装折算到输出轴的惯量", SOURCE),
            common::field("loadTorque", "外部负载扭矩", "Nm", 0.0, 5.0, "摩擦、偏载或工艺负载扭矩", SOURCE),
            common::field("efficiency", "传动效率", "ratio", 0.01, 0.8, "0-1 之间的小数", SOURCE),
        ],
    }
}

#[rustfmt::skip]
pub fn calculate(request: &CalculationRequest) -> Result<CalculationResult, FieldError> {
    let module = definition();
    let source = module.source_page.clone();
    let fields = common::fields_map(request);
    let safety_factor = common::safety_factor(request)?;
    let stations = common::positive(&fields, "stationCount")?;
    let cycle_time = common::convert(common::positive(&fields, "cycleTime")?, common::unit(&fields, "cycleTime")?, "s", "cycleTime")?;
    let index_time = common::convert(common::positive(&fields, "indexTime")?, common::unit(&fields, "indexTime")?, "s", "indexTime")?;
    let index_angle = common::positive(&fields, "indexAngle")?;
    let motion_coefficient = common::positive(&fields, "motionCoefficient")?;
    let inertia = common::positive_or_zero(&fields, "tableInertia")?;
    let load_torque = common::positive_or_zero(&fields, "loadTorque")?;
    let efficiency = common::efficiency(&fields, "efficiency")?;

    if index_time >= cycle_time {
        return Err(FieldError { field_id: "indexTime".to_string(), message: "分割时间必须小于循环时间".to_string() });
    }

    let dwell_time = cycle_time - index_time;
    let output_rpm = 60.0 / (cycle_time * stations);
    let angle_rad = index_angle.to_radians();
    let peak_speed = 2.0 * angle_rad / index_time;
    let angular_acceleration = motion_coefficient * angle_rad / index_time.powi(2);
    let inertia_torque = inertia * angular_acceleration;
    let working_torque = inertia_torque + load_torque;
    let design_torque = working_torque * safety_factor / efficiency;
    let peak_power_w = design_torque * peak_speed;
    let mut risks = common::safety_risk(safety_factor, &source);

    if index_time < 0.25 {
        risks.push(common::risk("warning", "分割时间低于 0.25 s，冲击和凸轮寿命风险较高。", Some("indexTime"), &source));
    }
    if dwell_time < cycle_time * 0.2 {
        risks.push(common::risk("warning", "停歇时间低于循环时间 20%，定位、夹治具动作和传感器确认余量偏小。", Some("cycleTime"), &source));
    }

    Ok(common::result(
        module,
        request,
        "cam-indexer-sizing@0.2.0",
        format!("输出转速 {} rpm，设计扭矩 {} Nm", common::fmt(output_rpm), common::fmt(design_torque)),
        format!("按 {} 工位、{} s 循环和 {} s 分割估算，分割器输出侧需 {} Nm，峰值功率约 {} W。", common::fmt(stations), common::fmt(cycle_time), common::fmt(index_time), common::fmt(design_torque), common::fmt(peak_power_w)),
        vec![
            common::step("输出转速", "n = 60 / (tc * N)", format!("60 / ({} * {})", common::fmt(cycle_time), common::fmt(stations)), output_rpm, "rpm", &source),
            common::step("停歇时间", "td = tc - ti", format!("{} - {}", common::fmt(cycle_time), common::fmt(index_time)), dwell_time, "s", &source),
            common::step("峰值角速度", "ωmax = 2θ / ti", format!("2 * {} / {}", common::fmt(angle_rad), common::fmt(index_time)), peak_speed, "rad/s", &source),
            common::step("角加速度", "α = Cm * θ / ti²", format!("{} * {} / {}²", common::fmt(motion_coefficient), common::fmt(angle_rad), common::fmt(index_time)), angular_acceleration, "rad/s²", &source),
            common::step("惯量扭矩", "Tj = J * α", format!("{} * {}", common::fmt(inertia), common::fmt(angular_acceleration)), inertia_torque, "Nm", &source),
            common::step("设计扭矩", "T = (Tj + TL) * K / η", format!("({} + {}) * {} / {}", common::fmt(inertia_torque), common::fmt(load_torque), common::fmt(safety_factor), common::fmt(efficiency)), design_torque, "Nm", &source),
            common::step("峰值功率", "P = T * ωmax", format!("{} * {}", common::fmt(design_torque), common::fmt(peak_speed)), peak_power_w, "W", &source),
        ],
        vec![
            common::rule("cam-indexer-station", "工位数", "按工位数、输出转速和设计扭矩进入分割器型号表初筛。".to_string(), format!("{} 工位，输出转速 {} rpm", common::fmt(stations), common::fmt(output_rpm)), "low", &source),
            common::rule("cam-indexer-impact", "冲击风险", if index_time >= 0.25 { "分割时间可进入常规风险复核。".to_string() } else { "分割时间偏短，需复核凸轮曲线和驱动余量。".to_string() }, format!("分割时间 {} s，运动系数 {}", common::fmt(index_time), common::fmt(motion_coefficient)), if index_time >= 0.25 { "low" } else { "warning" }, &source),
            common::rule("cam-indexer-dwell", "停歇余量", if dwell_time >= cycle_time * 0.2 { "停歇时间具备基础动作余量。".to_string() } else { "停歇时间偏短，需复核定位销、夹具和传感器确认时间。".to_string() }, format!("停歇时间 {} s", common::fmt(dwell_time)), if dwell_time >= cycle_time * 0.2 { "low" } else { "warning" }, &source),
        ],
        risks,
        vec![
            common::requirement("outputSpeed", "输出转速", output_rpm, "rpm"),
            common::requirement("angularAcceleration", "角加速度", angular_acceleration, "rad/s²"),
            common::requirement("designTorque", "设计扭矩", design_torque, "Nm"),
            common::requirement("peakPower", "峰值功率", peak_power_w, "W"),
        ],
    ))
}
