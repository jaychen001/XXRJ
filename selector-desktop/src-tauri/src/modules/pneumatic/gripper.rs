use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "pneumatic-gripper-sizing";
const SOURCE: &str = "PDF P76 / 文档页 73 / 手指气缸";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "手指气缸".to_string(),
        category: "气动".to_string(),
        description: "按工件质量、摩擦系数、夹爪数量和力臂估算夹持力与夹持力矩。".to_string(),
        source_chapter: "气动执行元件".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field(
                "partMass",
                "工件质量",
                "kg",
                0.0,
                1.0,
                "夹持工件质量",
                SOURCE,
            ),
            common::field(
                "frictionCoefficient",
                "夹持摩擦系数",
                "ratio",
                0.01,
                0.3,
                "夹爪与工件之间的摩擦系数",
                SOURCE,
            ),
            common::field(
                "jawCount",
                "夹爪数量",
                "pcs",
                1.0,
                2.0,
                "参与夹持的手指数量",
                SOURCE,
            ),
            common::field_with_units(
                "jawArm",
                "夹持力臂",
                "mm",
                &["mm", "m"],
                0.001,
                30.0,
                "夹持点到转轴或导向中心距离",
                SOURCE,
            ),
            common::field(
                "acceleration",
                "搬运加速度",
                "m/s²",
                0.0,
                2.0,
                "搬运时的等效加速度",
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
    let mass = common::positive(&fields, "partMass")?;
    let friction = common::positive(&fields, "frictionCoefficient")?;
    let jaw_count = common::positive(&fields, "jawCount")?;
    let arm_m = common::convert(
        common::positive(&fields, "jawArm")?,
        common::unit(&fields, "jawArm")?,
        "m",
        "jawArm",
    )?;
    let acceleration = common::positive_or_zero(&fields, "acceleration")?;
    let holding_force = mass * (9.80665 + acceleration) * safety_factor / friction;
    let force_per_jaw = holding_force / jaw_count;
    let grip_torque = force_per_jaw * arm_m;
    let mut risks = common::safety_risk(safety_factor, &source);
    if friction < 0.2 {
        risks.push(common::risk(
            "warning",
            "夹持摩擦系数偏低，应加防滑面或提高夹紧余量。",
            Some("frictionCoefficient"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "pneumatic-gripper-sizing@0.1.0",
        format!(
            "单爪夹持力 {} N，夹持力矩 {} Nm",
            common::fmt(force_per_jaw),
            common::fmt(grip_torque)
        ),
        format!(
            "按 {} 个夹爪分担，单爪至少需要 {} N。",
            common::fmt(jaw_count),
            common::fmt(force_per_jaw)
        ),
        vec![
            common::step(
                "总夹持力",
                "F = m * (g+a) * K / μ",
                format!(
                    "{}*(9.80665+{})*{} / {}",
                    common::fmt(mass),
                    common::fmt(acceleration),
                    common::fmt(safety_factor),
                    common::fmt(friction)
                ),
                holding_force,
                "N",
                &source,
            ),
            common::step(
                "单爪夹持力",
                "Fj = F / n",
                format!(
                    "{} / {}",
                    common::fmt(holding_force),
                    common::fmt(jaw_count)
                ),
                force_per_jaw,
                "N",
                &source,
            ),
            common::step(
                "夹持力矩",
                "T = Fj * L",
                format!("{} * {}", common::fmt(force_per_jaw), common::fmt(arm_m)),
                grip_torque,
                "Nm",
                &source,
            ),
        ],
        vec![common::rule(
            "gripper-type",
            "夹爪类型",
            "按单爪夹持力匹配手指气缸，并复核开闭行程和夹持姿态。".to_string(),
            format!("单爪夹持力 {} N", common::fmt(force_per_jaw)),
            "low",
            &source,
        )],
        risks,
        vec![
            common::requirement("holdingForce", "总夹持力", holding_force, "N"),
            common::requirement("forcePerJaw", "单爪夹持力", force_per_jaw, "N"),
            common::requirement("gripTorque", "夹持力矩", grip_torque, "Nm"),
        ],
    ))
}
