use std::f64::consts::PI;

use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "pneumatic-rotary-actuator-sizing";
const SOURCE: &str = "工程公式库 / 旋转气缸";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "旋转气缸".to_string(),
        category: "气动".to_string(),
        description: "按负载惯量、旋转角度、动作时间和候选规格校核扭矩与允许动能。".to_string(),
        source_chapter: "气动执行元件".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field(
                "loadInertia",
                "负载惯量",
                "kg·m²",
                0.0,
                0.01,
                "折算到旋转轴的总惯量",
                SOURCE,
            ),
            common::field(
                "rotationAngle",
                "旋转角度",
                "deg",
                0.0,
                90.0,
                "单次旋转角度",
                SOURCE,
            ),
            common::field_with_units(
                "rotationTime",
                "旋转时间",
                "s",
                &["s", "min"],
                0.001,
                0.5,
                "完成单次旋转的时间",
                SOURCE,
            ),
            common::field(
                "externalTorque",
                "外部阻力矩",
                "Nm",
                0.0,
                0.5,
                "旋转方向上的外部阻力矩",
                SOURCE,
            ),
            common::field(
                "torqueLoadRate",
                "扭矩负载率上限",
                "ratio",
                0.1,
                0.5,
                "候选旋转气缸推荐负载率上限",
                SOURCE,
            ),
            common::field(
                "candidateTorque",
                "候选额定扭矩",
                "Nm",
                0.0,
                5.0,
                "样册中候选旋转气缸额定扭矩",
                SOURCE,
            ),
            common::field(
                "allowableKineticEnergy",
                "候选允许动能",
                "J",
                0.0,
                0.2,
                "样册中候选旋转气缸允许动能",
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
    let inertia = common::positive(&fields, "loadInertia")?;
    let angle_deg = common::positive(&fields, "rotationAngle")?;
    let rotation_time = common::convert(
        common::positive(&fields, "rotationTime")?,
        common::unit(&fields, "rotationTime")?,
        "s",
        "rotationTime",
    )?;
    let external_torque = common::positive_or_zero(&fields, "externalTorque")?;
    let load_rate = common::efficiency(&fields, "torqueLoadRate")?;
    let candidate_torque = common::positive(&fields, "candidateTorque")?;
    let allowable_energy = common::positive(&fields, "allowableKineticEnergy")?;

    let angle_rad = angle_deg * PI / 180.0;
    let peak_angular_speed = 2.0 * angle_rad / rotation_time;
    let angular_acceleration = 4.0 * angle_rad / rotation_time.powi(2);
    let inertia_torque = inertia * angular_acceleration;
    let required_torque = (inertia_torque + external_torque) * safety_factor / load_rate;
    let kinetic_energy = 0.5 * inertia * peak_angular_speed.powi(2);
    let torque_margin = candidate_torque / required_torque;
    let energy_margin = allowable_energy / kinetic_energy;
    let minimum_rotation_time = (2.0 * inertia * angle_rad.powi(2) / allowable_energy).sqrt();

    let mut risks = common::safety_risk(safety_factor, &source);
    if torque_margin < 1.2 {
        risks.push(common::risk(
            "warning",
            "候选旋转气缸扭矩余量低于 1.2。",
            Some("candidateTorque"),
            &source,
        ));
    }
    if energy_margin < 1.2 {
        risks.push(common::risk(
            "warning",
            "候选旋转气缸允许动能余量低于 1.2，需降低速度或加外部缓冲。",
            Some("allowableKineticEnergy"),
            &source,
        ));
    }
    if rotation_time < minimum_rotation_time {
        risks.push(common::risk(
            "warning",
            "当前旋转时间低于按允许动能反推的最小时间。",
            Some("rotationTime"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "pneumatic-rotary-actuator-sizing@0.1.0",
        format!(
            "需求扭矩 {} Nm，动能余量 {}",
            common::fmt(required_torque),
            common::fmt(energy_margin)
        ),
        format!(
            "候选旋转气缸需提供不小于 {} Nm 扭矩，并满足 {} J 旋转动能。",
            common::fmt(required_torque),
            common::fmt(kinetic_energy)
        ),
        vec![
            common::step(
                "峰值角速度",
                "ωp = 2θ / t",
                format!(
                    "2*{} / {}",
                    common::fmt(angle_rad),
                    common::fmt(rotation_time)
                ),
                peak_angular_speed,
                "rad/s",
                &source,
            ),
            common::step(
                "角加速度",
                "α = 4θ / t²",
                format!(
                    "4*{} / {}²",
                    common::fmt(angle_rad),
                    common::fmt(rotation_time)
                ),
                angular_acceleration,
                "rad/s²",
                &source,
            ),
            common::step(
                "惯量扭矩",
                "Tj = J * α",
                format!(
                    "{}*{}",
                    common::fmt(inertia),
                    common::fmt(angular_acceleration)
                ),
                inertia_torque,
                "Nm",
                &source,
            ),
            common::step(
                "需求扭矩",
                "T = (Tj + Text) * K / λ",
                format!(
                    "({}+{})*{} / {}",
                    common::fmt(inertia_torque),
                    common::fmt(external_torque),
                    common::fmt(safety_factor),
                    common::fmt(load_rate)
                ),
                required_torque,
                "Nm",
                &source,
            ),
            common::step(
                "负载动能",
                "E = 0.5 * J * ωp²",
                format!(
                    "0.5*{}*{}²",
                    common::fmt(inertia),
                    common::fmt(peak_angular_speed)
                ),
                kinetic_energy,
                "J",
                &source,
            ),
            common::step(
                "扭矩余量",
                "Storque = Trated / T",
                format!(
                    "{} / {}",
                    common::fmt(candidate_torque),
                    common::fmt(required_torque)
                ),
                torque_margin,
                "ratio",
                &source,
            ),
            common::step(
                "动能余量",
                "Senergy = Erated / E",
                format!(
                    "{} / {}",
                    common::fmt(allowable_energy),
                    common::fmt(kinetic_energy)
                ),
                energy_margin,
                "ratio",
                &source,
            ),
            common::step(
                "最小旋转时间",
                "tmin = sqrt(2 * J * θ² / Erated)",
                format!(
                    "sqrt(2*{}*{}² / {})",
                    common::fmt(inertia),
                    common::fmt(angle_rad),
                    common::fmt(allowable_energy)
                ),
                minimum_rotation_time,
                "s",
                &source,
            ),
        ],
        vec![common::rule(
            "rotary-actuator-candidate",
            "候选规格判断",
            "候选旋转气缸需同时满足额定扭矩、负载率和允许动能。".to_string(),
            format!(
                "扭矩余量 {}，动能余量 {}",
                common::fmt(torque_margin),
                common::fmt(energy_margin)
            ),
            if torque_margin >= 1.2 && energy_margin >= 1.2 {
                "low"
            } else {
                "warning"
            },
            &source,
        )],
        risks,
        vec![
            common::requirement("requiredTorque", "需求扭矩", required_torque, "Nm"),
            common::requirement("kineticEnergy", "负载动能", kinetic_energy, "J"),
            common::requirement("torqueMargin", "扭矩余量", torque_margin, "ratio"),
            common::requirement("energyMargin", "动能余量", energy_margin, "ratio"),
            common::requirement(
                "minimumRotationTime",
                "最小旋转时间",
                minimum_rotation_time,
                "s",
            ),
        ],
    ))
}
