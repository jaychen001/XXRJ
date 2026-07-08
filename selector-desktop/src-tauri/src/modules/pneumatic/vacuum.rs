use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "vacuum-suction-sizing";
const SOURCE: &str = "PDF P98 / 文档页 95 / 真空吸附";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "真空吸附".to_string(),
        category: "气动".to_string(),
        description: "按工件质量、加速度、真空压力和吸盘数量估算吸附力与吸盘直径。".to_string(),
        source_chapter: "气动执行元件".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field(
                "workpieceMass",
                "工件质量",
                "kg",
                0.0,
                2.0,
                "被吸附工件质量",
                SOURCE,
            ),
            common::field(
                "acceleration",
                "搬运加速度",
                "m/s²",
                0.0,
                2.0,
                "搬运动作等效加速度",
                SOURCE,
            ),
            common::field(
                "vacuumPressure",
                "有效真空压力",
                "kPa",
                1.0,
                60.0,
                "吸盘处有效负压",
                SOURCE,
            ),
            common::field(
                "cupCount",
                "吸盘数量",
                "pcs",
                1.0,
                4.0,
                "参与承载的吸盘数量",
                SOURCE,
            ),
            common::field(
                "leakageFactor",
                "泄漏修正",
                "ratio",
                0.01,
                0.8,
                "表面粗糙、漏气和姿态修正",
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
    let mass = common::positive(&fields, "workpieceMass")?;
    let acceleration = common::positive_or_zero(&fields, "acceleration")?;
    let vacuum_kpa = common::positive(&fields, "vacuumPressure")?;
    let cup_count = common::positive(&fields, "cupCount")?;
    let leakage = common::efficiency(&fields, "leakageFactor")?;
    let suction_force = mass * (9.80665 + acceleration) * safety_factor / leakage;
    let total_area_m2 = suction_force / (vacuum_kpa * 1000.0);
    let cup_area_m2 = total_area_m2 / cup_count;
    let cup_diameter_mm = (4.0 * cup_area_m2 / std::f64::consts::PI).sqrt() * 1000.0;
    let mut risks = common::safety_risk(safety_factor, &source);
    if leakage < 0.6 {
        risks.push(common::risk(
            "warning",
            "泄漏修正低于 0.6，建议增加吸盘数量或改用海绵吸盘。",
            Some("leakageFactor"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "vacuum-suction-sizing@0.1.0",
        format!(
            "吸附力 {} N，单吸盘直径需求 {} mm",
            common::fmt(suction_force),
            common::fmt(cup_diameter_mm)
        ),
        format!(
            "安全系数 {} 已记录，{} 个吸盘时单个有效直径至少 {} mm。",
            common::fmt(safety_factor),
            common::fmt(cup_count),
            common::fmt(cup_diameter_mm)
        ),
        vec![
            common::step(
                "吸附力",
                "F = m * (g+a) * K / η",
                format!(
                    "{} * (9.80665+{}) * {} / {}",
                    common::fmt(mass),
                    common::fmt(acceleration),
                    common::fmt(safety_factor),
                    common::fmt(leakage)
                ),
                suction_force,
                "N",
                &source,
            ),
            common::step(
                "吸盘总面积",
                "A = F / ΔP",
                format!(
                    "{} / ({}*1000)",
                    common::fmt(suction_force),
                    common::fmt(vacuum_kpa)
                ),
                total_area_m2,
                "m²",
                &source,
            ),
            common::step(
                "吸盘直径",
                "D = sqrt(4A/(πn))",
                format!(
                    "sqrt(4*{} / (π*{}))",
                    common::fmt(total_area_m2),
                    common::fmt(cup_count)
                ),
                cup_diameter_mm,
                "mm",
                &source,
            ),
        ],
        vec![common::rule(
            "vacuum-cup-count",
            "吸盘数量",
            "按直径需求上取标准吸盘，并复核工件表面与失压保护。".to_string(),
            format!("单吸盘直径 {} mm", common::fmt(cup_diameter_mm)),
            "low",
            &source,
        )],
        risks,
        vec![
            common::requirement("suctionForce", "吸附力", suction_force, "N"),
            common::requirement("cupArea", "单吸盘面积", cup_area_m2, "m²"),
            common::requirement("cupDiameter", "单吸盘直径", cup_diameter_mm, "mm"),
        ],
    ))
}
