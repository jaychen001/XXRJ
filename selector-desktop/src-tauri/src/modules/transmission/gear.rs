use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "gear-basic";
const SOURCE: &str = "工程公式库 / 齿轮";

#[rustfmt::skip]
pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "齿轮参数计算".to_string(),
        category: "传动".to_string(),
        description: "按模数、齿数、齿宽、扭矩和许用应力估算中心距、减速比和承载余量。".to_string(),
        source_chapter: "齿轮".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field("module", "模数", "mm", 0.1, 2.0, "标准齿轮模数", SOURCE),
            common::field("driveTeeth", "主动齿数", "teeth", 1.0, 20.0, "主动齿轮齿数", SOURCE),
            common::field("drivenTeeth", "从动齿数", "teeth", 1.0, 60.0, "从动齿轮齿数", SOURCE),
            common::field("faceWidth", "齿宽", "mm", 0.1, 20.0, "有效啮合齿宽", SOURCE),
            common::field("transmitTorque", "传递扭矩", "Nm", 0.0, 5.0, "主动齿轮传递扭矩", SOURCE),
            common::field_with_units("gearSpeed", "主动齿轮转速", "rpm", &["rpm", "rps"], 0.0, 300.0, "主动齿轮工作转速", SOURCE),
            common::field("serviceFactor", "工况系数", "ratio", 0.1, 1.25, "冲击、启停和载荷波动修正", SOURCE),
            common::field("allowableToothStress", "许用齿根应力", "N/mm²", 0.1, 120.0, "材料和热处理后的许用齿根应力", SOURCE),
        ],
    }
}

#[rustfmt::skip]
pub fn calculate(request: &CalculationRequest) -> Result<CalculationResult, FieldError> {
    let module = definition();
    let source = module.source_page.clone();
    let fields = common::fields_map(request);
    let safety_factor = common::safety_factor(request)?;
    let gear_module = common::positive(&fields, "module")?;
    let drive_teeth = common::positive(&fields, "driveTeeth")?;
    let driven_teeth = common::positive(&fields, "drivenTeeth")?;
    let face_width = common::positive(&fields, "faceWidth")?;
    let transmit_torque = common::positive(&fields, "transmitTorque")?;
    let speed_rpm = common::convert(common::positive(&fields, "gearSpeed")?, common::unit(&fields, "gearSpeed")?, "rpm", "gearSpeed")?;
    let service_factor = common::positive(&fields, "serviceFactor")?;
    let allowable_stress = common::positive(&fields, "allowableToothStress")?;

    let drive_pitch_diameter = gear_module * drive_teeth;
    let driven_pitch_diameter = gear_module * driven_teeth;
    let center_distance = (drive_pitch_diameter + driven_pitch_diameter) / 2.0;
    let ratio = driven_teeth / drive_teeth;
    let width_ratio = face_width / gear_module;
    let design_torque = transmit_torque * service_factor * safety_factor;
    let tangential_force = 2000.0 * design_torque / drive_pitch_diameter;
    let bending_stress_index = tangential_force / (face_width * gear_module);
    let stress_margin = allowable_stress / bending_stress_index;
    let pitch_line_speed = std::f64::consts::PI * drive_pitch_diameter / 1000.0 * speed_rpm / 60.0;
    let mut risks = common::safety_risk(safety_factor, &source);

    if drive_teeth < 17.0 {
        risks.push(common::risk("warning", "主动齿数低于 17，标准直齿轮可能存在根切风险。", Some("driveTeeth"), &source));
    }
    if stress_margin < 1.2 {
        risks.push(common::risk("warning", "简化齿根应力余量低于 1.2，需提高模数、齿宽或材料强度。", Some("allowableToothStress"), &source));
    }

    Ok(common::result(
        module,
        request,
        "gear-basic@0.2.0",
        format!("中心距 {} mm，减速比 {}", common::fmt(center_distance), common::fmt(ratio)),
        format!("按模数 {}、齿数 {}/{} 计算，切向力 {} N，齿根应力余量 {}。", common::fmt(gear_module), common::fmt(drive_teeth), common::fmt(driven_teeth), common::fmt(tangential_force), common::fmt(stress_margin)),
        vec![
            common::step("主动分度圆", "d1 = m * z1", format!("{} * {}", common::fmt(gear_module), common::fmt(drive_teeth)), drive_pitch_diameter, "mm", &source),
            common::step("从动分度圆", "d2 = m * z2", format!("{} * {}", common::fmt(gear_module), common::fmt(driven_teeth)), driven_pitch_diameter, "mm", &source),
            common::step("中心距", "a = (d1 + d2) / 2", format!("({} + {}) / 2", common::fmt(drive_pitch_diameter), common::fmt(driven_pitch_diameter)), center_distance, "mm", &source),
            common::step("减速比", "i = z2 / z1", format!("{} / {}", common::fmt(driven_teeth), common::fmt(drive_teeth)), ratio, "ratio", &source),
            common::step("设计扭矩", "Td = T * Ka * K", format!("{} * {} * {}", common::fmt(transmit_torque), common::fmt(service_factor), common::fmt(safety_factor)), design_torque, "Nm", &source),
            common::step("齿面切向力", "Ft = 2000 * Td / d1", format!("2000 * {} / {}", common::fmt(design_torque), common::fmt(drive_pitch_diameter)), tangential_force, "N", &source),
            common::step("弯曲应力指标", "σ = Ft / (b * m)", format!("{} / ({} * {})", common::fmt(tangential_force), common::fmt(face_width), common::fmt(gear_module)), bending_stress_index, "N/mm²", &source),
            common::step("齿根应力余量", "M = σallow / σ", format!("{} / {}", common::fmt(allowable_stress), common::fmt(bending_stress_index)), stress_margin, "ratio", &source),
            common::step("节线速度", "v = π * d1 * n / 60", format!("π * {} / 1000 * {} / 60", common::fmt(drive_pitch_diameter), common::fmt(speed_rpm)), pitch_line_speed, "m/s", &source),
        ],
        vec![
            common::rule("gear-undercut", "齿数风险", if drive_teeth >= 17.0 { "齿数可进入标准直齿轮初筛。".to_string() } else { "齿数偏少，需改齿数、变位或更换结构。".to_string() }, format!("主动齿数 {}", common::fmt(drive_teeth)), if drive_teeth >= 17.0 { "low" } else { "warning" }, &source),
            common::rule("gear-face-width", "齿宽比例", "按齿宽/模数复核承载和加工空间。".to_string(), format!("b/m = {}", common::fmt(width_ratio)), if (6.0..=16.0).contains(&width_ratio) { "low" } else { "warning" }, &source),
            common::rule("gear-stress-margin", "承载余量", if stress_margin >= 1.2 { "简化齿根应力余量可进入样本匹配。".to_string() } else { "承载余量不足，需提高模数、齿宽或材料强度。".to_string() }, format!("齿根应力余量 {}", common::fmt(stress_margin)), if stress_margin >= 1.2 { "low" } else { "warning" }, &source),
        ],
        risks,
        vec![
            common::requirement("module", "模数", gear_module, "mm"),
            common::requirement("centerDistance", "中心距", center_distance, "mm"),
            common::requirement("gearRatio", "减速比", ratio, "ratio"),
            common::requirement("tangentialForce", "齿面切向力", tangential_force, "N"),
            common::requirement("stressMargin", "齿根应力余量", stress_margin, "ratio"),
        ],
    ))
}
