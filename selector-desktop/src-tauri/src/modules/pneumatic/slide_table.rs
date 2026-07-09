use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "pneumatic-slide-table-sizing";
const SOURCE: &str = "工程公式库 / 滑台气缸";

#[rustfmt::skip]
pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "滑台气缸".to_string(),
        category: "气动".to_string(),
        description: "按负载、行程时间、偏载距离和候选规格校核推力、导向力矩与动能。".to_string(),
        source_chapter: "气动执行元件".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field("loadMass", "负载质量", "kg", 0.0, 5.0, "滑台承载总质量", SOURCE),
            common::field("guideFriction", "导向摩擦系数", "ratio", 0.0, 0.15, "滑台导向摩擦系数", SOURCE),
            common::field("acceleration", "加速度", "m/s²", 0.0, 2.0, "动作加速阶段等效加速度", SOURCE),
            common::field_with_units("stroke", "行程", "mm", &["mm", "m"], 0.001, 100.0, "滑台单程行程", SOURCE),
            common::field_with_units("moveTime", "移动时间", "s", &["s", "min"], 0.001, 0.5, "完成单程移动的时间", SOURCE),
            common::field("loadRateLimit", "负载率上限", "ratio", 0.1, 0.5, "推力负载率上限", SOURCE),
            common::field("externalForce", "外部阻力", "N", 0.0, 0.0, "运动方向上的外部阻力", SOURCE),
            common::field("verticalLoadFactor", "垂直负载系数", "ratio", 0.0, 0.0, "水平运动取 0，垂直提升取 1", SOURCE),
            common::field("candidateRatedThrust", "候选额定推力", "N", 0.0, 300.0, "候选滑台气缸额定推力", SOURCE),
            common::field_with_units("loadCenterOffset", "负载偏心距", "mm", &["mm", "m"], 0.0, 40.0, "负载重心到导向中心的距离", SOURCE),
            common::field("allowableMoment", "候选允许力矩", "Nm", 0.0, 8.0, "样册中候选滑台允许力矩", SOURCE),
            common::field("allowableKineticEnergy", "候选允许动能", "J", 0.0, 0.5, "样册中候选滑台缓冲允许动能", SOURCE),
        ],
    }
}

pub fn calculate(request: &CalculationRequest) -> Result<CalculationResult, FieldError> {
    let module = definition();
    let source = module.source_page.clone();
    let fields = common::fields_map(request);
    let safety_factor = common::safety_factor(request)?;
    let mass = common::positive(&fields, "loadMass")?;
    let friction = common::positive_or_zero(&fields, "guideFriction")?;
    let acceleration = common::positive_or_zero(&fields, "acceleration")?;
    let stroke_m = common::convert(
        common::positive(&fields, "stroke")?,
        common::unit(&fields, "stroke")?,
        "m",
        "stroke",
    )?;
    let move_time = common::convert(
        common::positive(&fields, "moveTime")?,
        common::unit(&fields, "moveTime")?,
        "s",
        "moveTime",
    )?;
    let load_rate = common::efficiency(&fields, "loadRateLimit")?;
    let external_force = common::positive_or_zero(&fields, "externalForce")?;
    let vertical_factor = common::positive_or_zero(&fields, "verticalLoadFactor")?;
    let candidate_thrust = common::positive(&fields, "candidateRatedThrust")?;
    let offset_m = common::convert(
        common::positive_or_zero(&fields, "loadCenterOffset")?,
        common::unit(&fields, "loadCenterOffset")?,
        "m",
        "loadCenterOffset",
    )?;
    let allowable_moment = common::positive(&fields, "allowableMoment")?;
    let allowable_energy = common::positive(&fields, "allowableKineticEnergy")?;

    let friction_force = mass * 9.80665 * friction;
    let acceleration_force = mass * acceleration;
    let vertical_force = mass * 9.80665 * vertical_factor;
    let working_force = friction_force + acceleration_force + vertical_force + external_force;
    let required_thrust = working_force * safety_factor / load_rate;
    let average_speed = stroke_m / move_time;
    let kinetic_energy = 0.5 * mass * average_speed.powi(2);
    let load_moment = mass * 9.80665 * offset_m * safety_factor;
    let thrust_margin = candidate_thrust / required_thrust;
    let moment_margin = if load_moment > 0.0 {
        allowable_moment / load_moment
    } else {
        999.0
    };
    let energy_margin = allowable_energy / kinetic_energy;

    let mut risks = common::safety_risk(safety_factor, &source);
    if average_speed > 0.8 {
        risks.push(common::risk(
            "warning",
            "滑台平均速度高于 0.8 m/s，需复核缓冲和导向刚性。",
            Some("moveTime"),
            &source,
        ));
    }
    if thrust_margin < 1.2 {
        risks.push(common::risk(
            "warning",
            "候选滑台推力余量低于 1.2。",
            Some("candidateRatedThrust"),
            &source,
        ));
    }
    if moment_margin < 1.2 {
        risks.push(common::risk(
            "warning",
            "候选滑台允许力矩余量低于 1.2。",
            Some("allowableMoment"),
            &source,
        ));
    }
    if energy_margin < 1.2 {
        risks.push(common::risk(
            "warning",
            "候选滑台缓冲允许动能余量低于 1.2。",
            Some("allowableKineticEnergy"),
            &source,
        ));
    }
    if vertical_factor > 0.0 {
        risks.push(common::risk(
            "info",
            "存在垂直负载，需确认断气保持、止落和缓冲方案。",
            Some("verticalLoadFactor"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "pneumatic-slide-table-sizing@0.2.0",
        format!(
            "推力需求 {} N，动能余量 {}",
            common::fmt(required_thrust),
            common::fmt(energy_margin)
        ),
        format!(
            "候选滑台气缸需提供不小于 {} N 推力，并满足 {} Nm 偏载力矩和 {} J 动能。",
            common::fmt(required_thrust),
            common::fmt(load_moment),
            common::fmt(kinetic_energy)
        ),
        vec![
            common::step(
                "导向摩擦力",
                "Ff = m * g * μ",
                format!("{}*9.80665*{}", common::fmt(mass), common::fmt(friction)),
                friction_force,
                "N",
                &source,
            ),
            common::step(
                "加速力",
                "Fa = m * a",
                format!("{}*{}", common::fmt(mass), common::fmt(acceleration)),
                acceleration_force,
                "N",
                &source,
            ),
            common::step(
                "垂直负载力",
                "Fg = m * g * kv",
                format!(
                    "{}*9.80665*{}",
                    common::fmt(mass),
                    common::fmt(vertical_factor)
                ),
                vertical_force,
                "N",
                &source,
            ),
            common::step(
                "推力需求",
                "F = (Ff + Fa + Fg + Fext) * K / λ",
                format!(
                    "({}+{}+{}+{})*{} / {}",
                    common::fmt(friction_force),
                    common::fmt(acceleration_force),
                    common::fmt(vertical_force),
                    common::fmt(external_force),
                    common::fmt(safety_factor),
                    common::fmt(load_rate)
                ),
                required_thrust,
                "N",
                &source,
            ),
            common::step(
                "平均速度",
                "v = S / t",
                format!("{} / {}", common::fmt(stroke_m), common::fmt(move_time)),
                average_speed,
                "m/s",
                &source,
            ),
            common::step(
                "负载动能",
                "E = 0.5 * m * v²",
                format!("0.5*{}*{}²", common::fmt(mass), common::fmt(average_speed)),
                kinetic_energy,
                "J",
                &source,
            ),
            common::step(
                "偏载力矩",
                "M = m * g * L * K",
                format!(
                    "{}*9.80665*{}*{}",
                    common::fmt(mass),
                    common::fmt(offset_m),
                    common::fmt(safety_factor)
                ),
                load_moment,
                "Nm",
                &source,
            ),
            common::step(
                "推力余量",
                "Sforce = Frated / F",
                format!(
                    "{} / {}",
                    common::fmt(candidate_thrust),
                    common::fmt(required_thrust)
                ),
                thrust_margin,
                "ratio",
                &source,
            ),
            common::step(
                "力矩余量",
                "Smoment = Mrated / M",
                format!(
                    "{} / {}",
                    common::fmt(allowable_moment),
                    common::fmt(load_moment)
                ),
                moment_margin,
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
        ],
        vec![common::rule(
            "slide-table-candidate",
            "候选规格判断",
            "候选滑台气缸需同时满足推力、导向允许力矩和缓冲动能。".to_string(),
            format!(
                "推力余量 {}，力矩余量 {}，动能余量 {}",
                common::fmt(thrust_margin),
                common::fmt(moment_margin),
                common::fmt(energy_margin)
            ),
            if thrust_margin >= 1.2 && moment_margin >= 1.2 && energy_margin >= 1.2 {
                "low"
            } else {
                "warning"
            },
            &source,
        )],
        risks,
        vec![
            common::requirement("requiredThrust", "推力需求", required_thrust, "N"),
            common::requirement("averageSpeed", "平均速度", average_speed, "m/s"),
            common::requirement("kineticEnergy", "负载动能", kinetic_energy, "J"),
            common::requirement("loadMoment", "偏载力矩", load_moment, "Nm"),
            common::requirement("thrustMargin", "推力余量", thrust_margin, "ratio"),
            common::requirement("momentMargin", "力矩余量", moment_margin, "ratio"),
            common::requirement("energyMargin", "动能余量", energy_margin, "ratio"),
        ],
    ))
}
