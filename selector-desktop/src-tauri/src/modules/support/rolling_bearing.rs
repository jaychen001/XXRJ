use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "rolling-bearing-life";
const SOURCE: &str = "PDF P109 / 文档页 106 / 滚动轴承";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "滚动轴承".to_string(),
        category: "支撑导向".to_string(),
        description: "按径向/轴向载荷、转速、系数和动额定载荷估算轴承寿命需求。".to_string(),
        source_chapter: "滚动轴承".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field(
                "radialLoad",
                "径向载荷",
                "N",
                0.0,
                500.0,
                "轴承径向载荷 Fr",
                SOURCE,
            ),
            common::field(
                "axialLoad",
                "轴向载荷",
                "N",
                0.0,
                100.0,
                "轴承轴向载荷 Fa",
                SOURCE,
            ),
            common::field_with_units(
                "shaftSpeed",
                "转速",
                "rpm",
                &["rpm", "rps"],
                0.0,
                600.0,
                "轴承工作转速",
                SOURCE,
            ),
            common::field(
                "dynamicLoadRating",
                "动额定载荷",
                "N",
                1.0,
                3000.0,
                "样册 C 值",
                SOURCE,
            ),
            common::field(
                "applicationFactor",
                "工况系数",
                "ratio",
                0.1,
                1.2,
                "冲击、温升、润滑修正",
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
    let radial = common::positive(&fields, "radialLoad")?;
    let axial = common::positive_or_zero(&fields, "axialLoad")?;
    let speed = common::convert(
        common::positive(&fields, "shaftSpeed")?,
        common::unit(&fields, "shaftSpeed")?,
        "rpm",
        "shaftSpeed",
    )?;
    let rating = common::positive(&fields, "dynamicLoadRating")?;
    let application = common::positive(&fields, "applicationFactor")?;
    let equivalent_load = (radial + 0.6 * axial) * application * safety_factor;
    let load_ratio = rating / equivalent_load;
    let life_million_rev = load_ratio.powi(3);
    let life_hours = life_million_rev * 1_000_000.0 / (60.0 * speed);
    let mut risks = common::safety_risk(safety_factor, &source);
    if life_hours < 5000.0 {
        risks.push(common::risk(
            "warning",
            "估算寿命低于 5000 h，需提高轴承规格或降低载荷。",
            Some("dynamicLoadRating"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "rolling-bearing-life@0.1.0",
        format!(
            "等效载荷 {} N，估算寿命 {} h",
            common::fmt(equivalent_load),
            common::fmt(life_hours)
        ),
        format!(
            "样册 C 值需至少覆盖等效载荷 {} N，并按寿命 {} h 复核。",
            common::fmt(equivalent_load),
            common::fmt(life_hours)
        ),
        vec![
            common::step(
                "等效动载荷",
                "P = (Fr + 0.6Fa) * Ka * K",
                format!(
                    "({}+0.6*{})*{}*{}",
                    common::fmt(radial),
                    common::fmt(axial),
                    common::fmt(application),
                    common::fmt(safety_factor)
                ),
                equivalent_load,
                "N",
                &source,
            ),
            common::step(
                "载荷比",
                "C/P",
                format!("{} / {}", common::fmt(rating), common::fmt(equivalent_load)),
                load_ratio,
                "ratio",
                &source,
            ),
            common::step(
                "额定寿命",
                "L10 = (C/P)^3",
                format!("{}^3", common::fmt(load_ratio)),
                life_million_rev,
                "10⁶ rev",
                &source,
            ),
            common::step(
                "寿命小时",
                "Lh = L10*10⁶/(60n)",
                format!(
                    "{}*10⁶/(60*{})",
                    common::fmt(life_million_rev),
                    common::fmt(speed)
                ),
                life_hours,
                "h",
                &source,
            ),
        ],
        vec![common::rule(
            "bearing-sample-match",
            "样册匹配需求",
            "候选轴承必须满足 C 值、极限转速、内径和寿命小时。".to_string(),
            format!(
                "C/P = {}，n = {} rpm",
                common::fmt(load_ratio),
                common::fmt(speed)
            ),
            if life_hours >= 5000.0 {
                "low"
            } else {
                "warning"
            },
            &source,
        )],
        risks,
        vec![
            common::requirement("equivalentLoad", "等效动载荷", equivalent_load, "N"),
            common::requirement("dynamicLoadRating", "动额定载荷需求", equivalent_load, "N"),
            common::requirement("lifeHours", "寿命小时", life_hours, "h"),
        ],
    ))
}
