use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "pneumatic-gripper-sizing";
const SOURCE: &str = "工程公式库 / 手指气缸";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "手指气缸".to_string(),
        category: "气动".to_string(),
        description: "按工件质量、夹持摩擦、搬运加速度和候选规格校核夹持力与手指力矩。".to_string(),
        source_chapter: "气动执行元件".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field(
                "partMass",
                "工件质量",
                "kg",
                0.0,
                1.0,
                "被夹持工件质量",
                SOURCE,
            ),
            common::field(
                "frictionCoefficient",
                "夹持摩擦系数",
                "ratio",
                0.0,
                0.3,
                "夹爪与工件之间的有效摩擦系数",
                SOURCE,
            ),
            common::field(
                "jawCount",
                "夹爪数量",
                "pcs",
                1.0,
                2.0,
                "参与夹持的夹爪数量",
                SOURCE,
            ),
            common::field_with_units(
                "jawArm",
                "夹持力臂",
                "mm",
                &["mm", "m"],
                0.001,
                30.0,
                "夹持点到手指根部的距离",
                SOURCE,
            ),
            common::field(
                "acceleration",
                "搬运加速度",
                "m/s²",
                0.0,
                2.0,
                "搬运或翻转时的等效加速度",
                SOURCE,
            ),
            common::field(
                "orientationFactor",
                "姿态重力系数",
                "ratio",
                0.0,
                1.0,
                "水平支撑可取 0，垂直或侧向吊取通常取 1",
                SOURCE,
            ),
            common::field(
                "externalForce",
                "外部扰动力",
                "N",
                0.0,
                0.0,
                "夹持方向需抵抗的额外外力",
                SOURCE,
            ),
            common::field(
                "candidateGripForce",
                "候选单爪有效夹持力",
                "N",
                0.0,
                80.0,
                "样册中候选手指的单爪有效夹持力",
                SOURCE,
            ),
            common::field(
                "allowableFingerMoment",
                "候选允许手指力矩",
                "Nm",
                0.0,
                3.0,
                "样册中候选手指允许的力矩",
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
    let friction = common::efficiency(&fields, "frictionCoefficient")?;
    let jaw_count = common::positive(&fields, "jawCount")?;
    let jaw_arm_m = common::convert(
        common::positive(&fields, "jawArm")?,
        common::unit(&fields, "jawArm")?,
        "m",
        "jawArm",
    )?;
    let acceleration = common::positive_or_zero(&fields, "acceleration")?;
    let orientation_factor = common::positive_or_zero(&fields, "orientationFactor")?;
    let external_force = common::positive_or_zero(&fields, "externalForce")?;
    let candidate_force = common::positive(&fields, "candidateGripForce")?;
    let allowable_moment = common::positive(&fields, "allowableFingerMoment")?;

    let load_force = mass * (9.80665 * orientation_factor + acceleration) + external_force;
    let force_per_jaw = load_force * safety_factor / (friction * jaw_count);
    let holding_force = force_per_jaw * jaw_count;
    let grip_torque = force_per_jaw * jaw_arm_m;
    let force_margin = candidate_force / force_per_jaw;
    let moment_margin = allowable_moment / grip_torque;

    let mut risks = common::safety_risk(safety_factor, &source);
    if friction < 0.2 {
        risks.push(common::risk(
            "warning",
            "夹持摩擦系数偏低，建议增加包胶、纹路或夹持面定位。",
            Some("frictionCoefficient"),
            &source,
        ));
    }
    if force_margin < 1.2 {
        risks.push(common::risk(
            "warning",
            "候选手指夹持力余量低于 1.2。",
            Some("candidateGripForce"),
            &source,
        ));
    }
    if moment_margin < 1.2 {
        risks.push(common::risk(
            "warning",
            "候选手指允许力矩余量低于 1.2。",
            Some("allowableFingerMoment"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "pneumatic-gripper-sizing@0.2.0",
        format!(
            "单爪夹持力 {} N，夹持力余量 {}",
            common::fmt(force_per_jaw),
            common::fmt(force_margin)
        ),
        format!(
            "候选手指需提供不小于 {} N 的单爪有效夹持力，并满足 {} Nm 手指力矩。",
            common::fmt(force_per_jaw),
            common::fmt(grip_torque)
        ),
        vec![
            common::step(
                "夹持负载",
                "Fload = m * (g * kg + a) + Fext",
                format!(
                    "{}*(9.80665*{}+{})+{}",
                    common::fmt(mass),
                    common::fmt(orientation_factor),
                    common::fmt(acceleration),
                    common::fmt(external_force)
                ),
                load_force,
                "N",
                &source,
            ),
            common::step(
                "单爪夹持力",
                "Fjaw = Fload * K / (μ * n)",
                format!(
                    "{}*{} / ({}*{})",
                    common::fmt(load_force),
                    common::fmt(safety_factor),
                    common::fmt(friction),
                    common::fmt(jaw_count)
                ),
                force_per_jaw,
                "N",
                &source,
            ),
            common::step(
                "总夹持力",
                "Ftotal = Fjaw * n",
                format!("{}*{}", common::fmt(force_per_jaw), common::fmt(jaw_count)),
                holding_force,
                "N",
                &source,
            ),
            common::step(
                "夹持力矩",
                "M = Fjaw * L",
                format!("{}*{}", common::fmt(force_per_jaw), common::fmt(jaw_arm_m)),
                grip_torque,
                "Nm",
                &source,
            ),
            common::step(
                "夹持力余量",
                "Sforce = Frated / Fjaw",
                format!(
                    "{} / {}",
                    common::fmt(candidate_force),
                    common::fmt(force_per_jaw)
                ),
                force_margin,
                "ratio",
                &source,
            ),
            common::step(
                "手指力矩余量",
                "Smoment = Mrated / M",
                format!(
                    "{} / {}",
                    common::fmt(allowable_moment),
                    common::fmt(grip_torque)
                ),
                moment_margin,
                "ratio",
                &source,
            ),
        ],
        vec![common::rule(
            "gripper-candidate",
            "候选规格判断",
            "候选手指气缸需同时满足单爪有效夹持力和允许手指力矩。".to_string(),
            format!(
                "夹持力余量 {}，力矩余量 {}",
                common::fmt(force_margin),
                common::fmt(moment_margin)
            ),
            if force_margin >= 1.2 && moment_margin >= 1.2 {
                "low"
            } else {
                "warning"
            },
            &source,
        )],
        risks,
        vec![
            common::requirement("holdingForce", "总夹持力", holding_force, "N"),
            common::requirement("forcePerJaw", "单爪夹持力", force_per_jaw, "N"),
            common::requirement("gripTorque", "夹持力矩", grip_torque, "Nm"),
            common::requirement("forceMargin", "夹持力余量", force_margin, "ratio"),
            common::requirement("momentMargin", "手指力矩余量", moment_margin, "ratio"),
        ],
    ))
}
