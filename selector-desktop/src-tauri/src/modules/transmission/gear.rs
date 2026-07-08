use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "gear-basic";
const SOURCE: &str = "PDF P44 / 文档页 41 / 齿轮";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "齿轮参数计算".to_string(),
        category: "传动".to_string(),
        description: "按模数、齿数和齿宽计算分度圆、中心距、减速比与结构风险。".to_string(),
        source_chapter: "齿轮".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field("module", "模数", "mm", 0.1, 2.0, "标准齿轮模数", SOURCE),
            common::field(
                "driveTeeth",
                "主动齿数",
                "teeth",
                1.0,
                20.0,
                "主动齿轮齿数",
                SOURCE,
            ),
            common::field(
                "drivenTeeth",
                "从动齿数",
                "teeth",
                1.0,
                60.0,
                "从动齿轮齿数",
                SOURCE,
            ),
            common::field("faceWidth", "齿宽", "mm", 0.1, 20.0, "有效啮合齿宽", SOURCE),
        ],
    }
}

pub fn calculate(request: &CalculationRequest) -> Result<CalculationResult, FieldError> {
    let module = definition();
    let source = module.source_page.clone();
    let fields = common::fields_map(request);
    let safety_factor = common::safety_factor(request)?;
    let gear_module = common::positive(&fields, "module")?;
    let drive_teeth = common::positive(&fields, "driveTeeth")?;
    let driven_teeth = common::positive(&fields, "drivenTeeth")?;
    let face_width = common::positive(&fields, "faceWidth")?;
    let drive_pitch_diameter = gear_module * drive_teeth;
    let driven_pitch_diameter = gear_module * driven_teeth;
    let center_distance = (drive_pitch_diameter + driven_pitch_diameter) / 2.0;
    let ratio = driven_teeth / drive_teeth;
    let width_ratio = face_width / gear_module;
    let mut risks = common::safety_risk(safety_factor, &source);
    if drive_teeth < 17.0 {
        risks.push(common::risk(
            "warning",
            "主动齿数低于 17，标准直齿轮可能存在根切风险。",
            Some("driveTeeth"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "gear-basic@0.1.0",
        format!(
            "中心距 {} mm，减速比 {}",
            common::fmt(center_distance),
            common::fmt(ratio)
        ),
        format!(
            "按模数 {}、齿数 {}/{} 计算，中心距为 {} mm。",
            common::fmt(gear_module),
            common::fmt(drive_teeth),
            common::fmt(driven_teeth),
            common::fmt(center_distance)
        ),
        vec![
            common::step(
                "主动分度圆",
                "d1 = m * z1",
                format!(
                    "{} * {}",
                    common::fmt(gear_module),
                    common::fmt(drive_teeth)
                ),
                drive_pitch_diameter,
                "mm",
                &source,
            ),
            common::step(
                "从动分度圆",
                "d2 = m * z2",
                format!(
                    "{} * {}",
                    common::fmt(gear_module),
                    common::fmt(driven_teeth)
                ),
                driven_pitch_diameter,
                "mm",
                &source,
            ),
            common::step(
                "中心距",
                "a = (d1 + d2) / 2",
                format!(
                    "({} + {}) / 2",
                    common::fmt(drive_pitch_diameter),
                    common::fmt(driven_pitch_diameter)
                ),
                center_distance,
                "mm",
                &source,
            ),
            common::step(
                "减速比",
                "i = z2 / z1",
                format!(
                    "{} / {}",
                    common::fmt(driven_teeth),
                    common::fmt(drive_teeth)
                ),
                ratio,
                "ratio",
                &source,
            ),
        ],
        vec![
            common::rule(
                "gear-undercut",
                "齿数风险",
                if drive_teeth >= 17.0 {
                    "齿数可进入标准直齿轮初筛。".to_string()
                } else {
                    "齿数偏少，需改齿数、变位或更换结构。".to_string()
                },
                format!("主动齿数 {}", common::fmt(drive_teeth)),
                if drive_teeth >= 17.0 {
                    "low"
                } else {
                    "warning"
                },
                &source,
            ),
            common::rule(
                "gear-face-width",
                "齿宽比例",
                "按齿宽/模数复核承载和加工空间。".to_string(),
                format!("b/m = {}", common::fmt(width_ratio)),
                "low",
                &source,
            ),
        ],
        risks,
        vec![
            common::requirement("module", "模数", gear_module, "mm"),
            common::requirement("centerDistance", "中心距", center_distance, "mm"),
            common::requirement("gearRatio", "减速比", ratio, "ratio"),
            common::requirement(
                "drivePitchDiameter",
                "主动分度圆",
                drive_pitch_diameter,
                "mm",
            ),
            common::requirement(
                "drivenPitchDiameter",
                "从动分度圆",
                driven_pitch_diameter,
                "mm",
            ),
        ],
    ))
}
