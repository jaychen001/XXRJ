use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "linear-bearing-selector";
const SOURCE: &str = "PDF P104 / 文档页 101 / 直线轴承";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "直线轴承".to_string(),
        category: "支撑导向".to_string(),
        description: "按径向载荷、轴径、速度和额定载荷判断直线轴承类型与余量。".to_string(),
        source_chapter: "直线轴承".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field(
                "radialLoad",
                "径向载荷",
                "N",
                0.0,
                200.0,
                "单轴承承载径向力",
                SOURCE,
            ),
            common::field(
                "shaftDiameter",
                "轴径",
                "mm",
                1.0,
                20.0,
                "导向轴直径",
                SOURCE,
            ),
            common::field_with_units(
                "travelSpeed",
                "运行速度",
                "mm/s",
                &["mm/s", "m/s"],
                0.0,
                300.0,
                "直线运动速度",
                SOURCE,
            ),
            common::field(
                "loadRating",
                "额定载荷",
                "N",
                1.0,
                1000.0,
                "样册 C 值或基本额定载荷",
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
    let load = common::positive(&fields, "radialLoad")?;
    let shaft = common::positive(&fields, "shaftDiameter")?;
    let speed_m_s = common::convert(
        common::positive(&fields, "travelSpeed")?,
        common::unit(&fields, "travelSpeed")?,
        "m/s",
        "travelSpeed",
    )?;
    let rating = common::positive(&fields, "loadRating")?;
    let design_load = load * safety_factor;
    let load_margin = rating / design_load;
    let speed_index = speed_m_s * 60.0 / shaft;
    let recommendation = if speed_m_s <= 0.5 {
        "普通直线轴承或法兰型可初筛"
    } else {
        "优先复核高速低摩擦型或直线导轨替代"
    };
    let mut risks = common::safety_risk(safety_factor, &source);
    if load_margin < 1.5 {
        risks.push(common::risk(
            "warning",
            "直线轴承载荷余量低于 1.5，需提高规格或增加支撑点。",
            Some("loadRating"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "linear-bearing-selector@0.1.0",
        format!(
            "设计载荷 {} N，载荷余量 {}",
            common::fmt(design_load),
            common::fmt(load_margin)
        ),
        recommendation.to_string(),
        vec![
            common::step(
                "设计载荷",
                "P = F * K",
                format!("{} * {}", common::fmt(load), common::fmt(safety_factor)),
                design_load,
                "N",
                &source,
            ),
            common::step(
                "载荷余量",
                "S = C / P",
                format!("{} / {}", common::fmt(rating), common::fmt(design_load)),
                load_margin,
                "ratio",
                &source,
            ),
            common::step(
                "速度指标",
                "Iv = v * 60 / d",
                format!("{}*60/{}", common::fmt(speed_m_s), common::fmt(shaft)),
                speed_index,
                "index",
                &source,
            ),
        ],
        vec![common::rule(
            "linear-bearing-type",
            "类型判断",
            recommendation.to_string(),
            format!(
                "速度 {} m/s，轴径 {} mm",
                common::fmt(speed_m_s),
                common::fmt(shaft)
            ),
            if load_margin >= 1.5 { "low" } else { "warning" },
            &source,
        )],
        risks,
        vec![
            common::requirement("designLoad", "设计载荷", design_load, "N"),
            common::requirement("loadMargin", "载荷余量", load_margin, "ratio"),
            common::requirement("speedIndex", "速度指标", speed_index, "index"),
        ],
    ))
}
