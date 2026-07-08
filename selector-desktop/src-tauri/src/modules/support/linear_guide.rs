use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "linear-guide-sizing";
const SOURCE: &str = "PDF P103 / 文档页 100 / 直线导轨";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "直线导轨".to_string(),
        category: "支撑导向".to_string(),
        description: "按负载、滑块数量、动载额定值和安装姿态估算导轨载荷余量。".to_string(),
        source_chapter: "直线导轨".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field(
                "loadMass",
                "负载质量",
                "kg",
                0.0,
                20.0,
                "导轨承载总质量",
                SOURCE,
            ),
            common::field(
                "sliderCount",
                "滑块数量",
                "pcs",
                1.0,
                4.0,
                "共同承载的滑块数量",
                SOURCE,
            ),
            common::field(
                "dynamicLoadRating",
                "单滑块动额定载荷",
                "N",
                1.0,
                5000.0,
                "样册单滑块 C 值",
                SOURCE,
            ),
            common::field(
                "impactFactor",
                "冲击系数",
                "ratio",
                0.1,
                1.5,
                "速度、冲击和安装姿态修正",
                SOURCE,
            ),
            common::field_with_units(
                "offsetDistance",
                "偏载距离",
                "mm",
                &["mm", "m"],
                0.0,
                50.0,
                "负载重心到滑块组中心偏距",
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
    let sliders = common::positive(&fields, "sliderCount")?;
    let rating = common::positive(&fields, "dynamicLoadRating")?;
    let impact = common::positive(&fields, "impactFactor")?;
    let offset_m = common::convert(
        common::positive_or_zero(&fields, "offsetDistance")?,
        common::unit(&fields, "offsetDistance")?,
        "m",
        "offsetDistance",
    )?;
    let total_load = mass * 9.80665 * impact * safety_factor;
    let load_per_slider = total_load / sliders;
    let static_margin = rating / load_per_slider;
    let moment_load = total_load * offset_m;
    let mut risks = common::safety_risk(safety_factor, &source);
    if static_margin < 2.0 {
        risks.push(common::risk(
            "warning",
            "导轨载荷余量低于 2，需上调规格或增加滑块。",
            Some("dynamicLoadRating"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "linear-guide-sizing@0.1.0",
        format!(
            "单滑块载荷 {} N，载荷余量 {}",
            common::fmt(load_per_slider),
            common::fmt(static_margin)
        ),
        format!(
            "按 {} 个滑块承载，建议样册动额定载荷至少覆盖 {} N/滑块。",
            common::fmt(sliders),
            common::fmt(load_per_slider)
        ),
        vec![
            common::step(
                "修正总载荷",
                "F = m * g * K * fs",
                format!(
                    "{}*9.80665*{}*{}",
                    common::fmt(mass),
                    common::fmt(impact),
                    common::fmt(safety_factor)
                ),
                total_load,
                "N",
                &source,
            ),
            common::step(
                "单滑块载荷",
                "Fb = F / n",
                format!("{} / {}", common::fmt(total_load), common::fmt(sliders)),
                load_per_slider,
                "N",
                &source,
            ),
            common::step(
                "载荷余量",
                "S = C / Fb",
                format!("{} / {}", common::fmt(rating), common::fmt(load_per_slider)),
                static_margin,
                "ratio",
                &source,
            ),
            common::step(
                "偏载力矩",
                "M = F * L",
                format!("{} * {}", common::fmt(total_load), common::fmt(offset_m)),
                moment_load,
                "Nm",
                &source,
            ),
        ],
        vec![common::rule(
            "linear-guide-blocks",
            "滑块数量",
            "余量不足时优先增加滑块或改用更高规格导轨，并复核力矩方向。".to_string(),
            format!("载荷余量 {}", common::fmt(static_margin)),
            if static_margin >= 2.0 {
                "low"
            } else {
                "warning"
            },
            &source,
        )],
        risks,
        vec![
            common::requirement("loadPerSlider", "单滑块载荷", load_per_slider, "N"),
            common::requirement("staticMargin", "载荷余量", static_margin, "ratio"),
            common::requirement("momentLoad", "偏载力矩", moment_load, "Nm"),
        ],
    ))
}
