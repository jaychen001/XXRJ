use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "chain-selector";
const SOURCE: &str = "PDF P49 / 文档页 46 / 链条";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "链条选型计算".to_string(),
        category: "传动".to_string(),
        description: "按节距、链轮齿数、中心距和转速估算链速、链节数和类型风险。".to_string(),
        source_chapter: "链条".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field_with_units(
                "pitch",
                "节距",
                "mm",
                &["mm", "m"],
                0.001,
                12.7,
                "链条节距",
                SOURCE,
            ),
            common::field(
                "smallSprocketTeeth",
                "小链轮齿数",
                "teeth",
                1.0,
                18.0,
                "主动小链轮齿数",
                SOURCE,
            ),
            common::field(
                "largeSprocketTeeth",
                "大链轮齿数",
                "teeth",
                1.0,
                36.0,
                "从动大链轮齿数",
                SOURCE,
            ),
            common::field_with_units(
                "centerDistance",
                "中心距",
                "mm",
                &["mm", "m"],
                0.001,
                500.0,
                "两链轮中心距",
                SOURCE,
            ),
            common::field_with_units(
                "sprocketSpeed",
                "小链轮转速",
                "rpm",
                &["rpm", "rps"],
                0.0,
                200.0,
                "小链轮工作转速",
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
    let pitch_m = common::convert(
        common::positive(&fields, "pitch")?,
        common::unit(&fields, "pitch")?,
        "m",
        "pitch",
    )?;
    let pitch_mm = common::convert(
        common::positive(&fields, "pitch")?,
        common::unit(&fields, "pitch")?,
        "mm",
        "pitch",
    )?;
    let z1 = common::positive(&fields, "smallSprocketTeeth")?;
    let z2 = common::positive(&fields, "largeSprocketTeeth")?;
    let center_m = common::convert(
        common::positive(&fields, "centerDistance")?,
        common::unit(&fields, "centerDistance")?,
        "m",
        "centerDistance",
    )?;
    let speed_rpm = common::convert(
        common::positive(&fields, "sprocketSpeed")?,
        common::unit(&fields, "sprocketSpeed")?,
        "rpm",
        "sprocketSpeed",
    )?;
    let ratio = z2 / z1;
    let chain_speed = pitch_m * z1 * speed_rpm / 60.0;
    let center_pitch = center_m / pitch_m;
    let chain_links = 2.0 * center_pitch
        + (z1 + z2) / 2.0
        + (z2 - z1).powi(2) / (4.0 * std::f64::consts::PI.powi(2) * center_pitch);
    let mut risks = common::safety_risk(safety_factor, &source);
    if z1 < 17.0 {
        risks.push(common::risk(
            "warning",
            "小链轮齿数偏少，链条多边形效应和振动风险上升。",
            Some("smallSprocketTeeth"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "chain-selector@0.1.0",
        format!(
            "链速 {} m/s，链节数约 {} 节",
            common::fmt(chain_speed),
            common::fmt(chain_links)
        ),
        format!(
            "按节距 {} mm 和齿数 {}/{} 计算，传动比为 {}。",
            common::fmt(pitch_mm),
            common::fmt(z1),
            common::fmt(z2),
            common::fmt(ratio)
        ),
        vec![
            common::step(
                "传动比",
                "i = z2 / z1",
                format!("{} / {}", common::fmt(z2), common::fmt(z1)),
                ratio,
                "ratio",
                &source,
            ),
            common::step(
                "链速",
                "v = p * z1 * n / 60",
                format!(
                    "{} * {} * {} / 60",
                    common::fmt(pitch_m),
                    common::fmt(z1),
                    common::fmt(speed_rpm)
                ),
                chain_speed,
                "m/s",
                &source,
            ),
            common::step(
                "链节数",
                "Lp = 2C/p + (z1+z2)/2 + (z2-z1)^2/(4π²C/p)",
                format!(
                    "2*{} + ({}+{})/2 + ...",
                    common::fmt(center_pitch),
                    common::fmt(z1),
                    common::fmt(z2)
                ),
                chain_links,
                "links",
                &source,
            ),
        ],
        vec![
            common::rule(
                "chain-teeth",
                "小链轮齿数",
                if z1 >= 17.0 {
                    "小链轮齿数可进入常规链传动初筛。".to_string()
                } else {
                    "小链轮齿数偏少，建议增大齿数或降低转速。".to_string()
                },
                format!("小链轮齿数 {}", common::fmt(z1)),
                if z1 >= 17.0 { "low" } else { "warning" },
                &source,
            ),
            common::rule(
                "chain-speed",
                "链速判断",
                if chain_speed <= 8.0 {
                    "链速适合进入常规滚子链样本匹配。".to_string()
                } else {
                    "链速偏高，需复核润滑、冲击和链型。".to_string()
                },
                format!("链速 {} m/s", common::fmt(chain_speed)),
                if chain_speed <= 8.0 { "low" } else { "warning" },
                &source,
            ),
        ],
        risks,
        vec![
            common::requirement("chainRatio", "传动比", ratio, "ratio"),
            common::requirement("chainSpeed", "链速", chain_speed, "m/s"),
            common::requirement("chainLinks", "链节数", chain_links, "links"),
        ],
    ))
}
