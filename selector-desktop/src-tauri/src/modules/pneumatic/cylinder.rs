use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "pneumatic-cylinder-sizing";
const SOURCE: &str = "PDF P69 / 文档页 66 / 气动执行元件";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "气缸".to_string(),
        category: "气动".to_string(),
        description: "按负载、摩擦、加速度、负载率和气压估算气缸输出力与缸径。".to_string(),
        source_chapter: "气动执行元件".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field(
                "loadMass",
                "负载质量",
                "kg",
                0.0,
                8.0,
                "气缸推动或提升的等效负载",
                SOURCE,
            ),
            common::field(
                "frictionCoefficient",
                "摩擦系数",
                "ratio",
                0.0,
                0.2,
                "导向或滑台摩擦系数",
                SOURCE,
            ),
            common::field(
                "acceleration",
                "加速度",
                "m/s²",
                0.0,
                1.5,
                "启动阶段等效加速度",
                SOURCE,
            ),
            common::field(
                "workingPressure",
                "工作压力",
                "MPa",
                0.01,
                0.5,
                "气源有效工作压力，1 MPa = 1 N/mm²",
                SOURCE,
            ),
            common::field(
                "loadRateLimit",
                "负载率上限",
                "ratio",
                0.1,
                0.5,
                "常用负载率修正值",
                SOURCE,
            ),
            common::field(
                "mechanicalEfficiency",
                "机械效率",
                "ratio",
                0.01,
                0.9,
                "密封和机构损失修正",
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
    let mass = common::positive(&fields, "loadMass")?;
    let friction = common::positive_or_zero(&fields, "frictionCoefficient")?;
    let acceleration = common::positive_or_zero(&fields, "acceleration")?;
    let pressure = common::positive(&fields, "workingPressure")?;
    let load_rate = common::efficiency(&fields, "loadRateLimit")?;
    let efficiency = common::efficiency(&fields, "mechanicalEfficiency")?;
    let friction_force = mass * 9.80665 * friction;
    let acceleration_force = mass * acceleration;
    let basic_force = friction_force + acceleration_force;
    let output_force = basic_force * safety_factor / (load_rate * efficiency);
    let bore_diameter = (4.0 * output_force / (std::f64::consts::PI * pressure)).sqrt();
    let mut risks = common::safety_risk(safety_factor, &source);
    if load_rate > 0.7 {
        risks.push(common::risk(
            "warning",
            "负载率高于 0.7，气缸速度和寿命余量偏紧。",
            Some("loadRateLimit"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "pneumatic-cylinder-sizing@0.1.0",
        format!(
            "选型输出力 {} N，缸径需求 {} mm",
            common::fmt(output_force),
            common::fmt(bore_diameter)
        ),
        format!(
            "按安全系数 {} 和负载率 {} 修正，气缸理论输出力至少 {} N。",
            common::fmt(safety_factor),
            common::fmt(load_rate),
            common::fmt(output_force)
        ),
        vec![
            common::step(
                "摩擦力",
                "Ff = m * g * μ",
                format!(
                    "{} * 9.80665 * {}",
                    common::fmt(mass),
                    common::fmt(friction)
                ),
                friction_force,
                "N",
                &source,
            ),
            common::step(
                "加速力",
                "Fa = m * a",
                format!("{} * {}", common::fmt(mass), common::fmt(acceleration)),
                acceleration_force,
                "N",
                &source,
            ),
            common::step(
                "负载率修正",
                "F = (Ff + Fa) * K / (η * λ)",
                format!(
                    "({} + {}) * {} / ({} * {})",
                    common::fmt(friction_force),
                    common::fmt(acceleration_force),
                    common::fmt(safety_factor),
                    common::fmt(efficiency),
                    common::fmt(load_rate)
                ),
                output_force,
                "N",
                &source,
            ),
            common::step(
                "缸径需求",
                "D = sqrt(4F / (πP))",
                format!(
                    "sqrt(4*{} / (π*{}))",
                    common::fmt(output_force),
                    common::fmt(pressure)
                ),
                bore_diameter,
                "mm",
                &source,
            ),
        ],
        vec![common::rule(
            "cylinder-bore",
            "缸径初筛",
            "按计算缸径上取厂家标准缸径，并复核安装形式和缓冲。".to_string(),
            format!("需求缸径 {} mm", common::fmt(bore_diameter)),
            "low",
            &source,
        )],
        risks,
        vec![
            common::requirement("frictionForce", "摩擦力", friction_force, "N"),
            common::requirement("accelerationForce", "加速力", acceleration_force, "N"),
            common::requirement("outputForce", "选型输出力", output_force, "N"),
            common::requirement("boreDiameter", "缸径需求", bore_diameter, "mm"),
        ],
    ))
}
