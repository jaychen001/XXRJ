use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "cable-chain-rule-selector";
const SOURCE: &str = "PDF P121 / 文档页 118 / 拖链";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "拖链规则选型".to_string(),
        category: "规则选型".to_string(),
        description: "按行程、弯曲半径、线缆数量、填充率和速度判断拖链规格风险。".to_string(),
        source_chapter: "拖链".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field_with_units(
                "travel",
                "行程",
                "mm",
                &["mm", "m"],
                1.0,
                800.0,
                "拖链运行行程",
                SOURCE,
            ),
            common::field_with_units(
                "bendRadius",
                "弯曲半径",
                "mm",
                &["mm", "m"],
                1.0,
                75.0,
                "拖链弯曲半径",
                SOURCE,
            ),
            common::field(
                "cableCount",
                "线缆数量",
                "pcs",
                1.0,
                8.0,
                "拖链内电缆/气管数量",
                SOURCE,
            ),
            common::field(
                "fillRate",
                "填充率",
                "ratio",
                0.01,
                0.5,
                "线缆占拖链内腔比例",
                SOURCE,
            ),
            common::field_with_units(
                "speed",
                "运行速度",
                "mm/s",
                &["mm/s", "m/s"],
                0.0,
                500.0,
                "拖链移动速度",
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
    let travel = common::convert(
        common::positive(&fields, "travel")?,
        common::unit(&fields, "travel")?,
        "mm",
        "travel",
    )?;
    let radius = common::convert(
        common::positive(&fields, "bendRadius")?,
        common::unit(&fields, "bendRadius")?,
        "mm",
        "bendRadius",
    )?;
    let cable_count = common::positive(&fields, "cableCount")?;
    let fill_rate = common::efficiency(&fields, "fillRate")?;
    let speed = common::convert(
        common::positive(&fields, "speed")?,
        common::unit(&fields, "speed")?,
        "mm/s",
        "speed",
    )?;
    let install_length = travel / 2.0 + radius * std::f64::consts::PI;
    let fill_margin = 0.6 / fill_rate;
    let recommendation = if speed > 1000.0 || travel > 2000.0 {
        "高速长行程拖链"
    } else {
        "常规封闭或桥式拖链"
    };
    let mut risks = common::safety_risk(safety_factor, &source);
    if fill_rate > 0.6 {
        risks.push(common::risk(
            "warning",
            "填充率超过 0.6，拖链内腔和线缆弯曲余量不足。",
            Some("fillRate"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "cable-chain-rule-selector@0.1.0",
        format!(
            "建议 {}，安装长度估算 {} mm",
            recommendation,
            common::fmt(install_length)
        ),
        "拖链选型需复核最大线缆外径、最小弯曲半径、隔片和安装空间。".to_string(),
        vec![
            common::step(
                "问题1 行程",
                "travel",
                format!("{} mm", common::fmt(travel)),
                travel,
                "mm",
                &source,
            ),
            common::step(
                "问题2 弯曲半径",
                "R",
                format!("{} mm", common::fmt(radius)),
                radius,
                "mm",
                &source,
            ),
            common::step(
                "问题3 填充余量",
                "0.6 / fillRate",
                format!("0.6 / {}", common::fmt(fill_rate)),
                fill_margin,
                "ratio",
                &source,
            ),
            common::step(
                "安装长度估算",
                "L = S/2 + πR",
                format!("{}/2 + π*{}", common::fmt(travel), common::fmt(radius)),
                install_length,
                "mm",
                &source,
            ),
        ],
        vec![
            common::rule(
                "chain-type",
                "推荐类型",
                recommendation.to_string(),
                format!(
                    "行程 {} mm，速度 {} mm/s",
                    common::fmt(travel),
                    common::fmt(speed)
                ),
                "low",
                &source,
            ),
            common::rule(
                "chain-fill",
                "填充率",
                if fill_rate <= 0.6 {
                    "填充率可进入常规初筛。".to_string()
                } else {
                    "需增大拖链内腔或减少线缆堆叠。".to_string()
                },
                format!("填充率 {}", common::fmt(fill_rate)),
                if fill_rate <= 0.6 { "low" } else { "warning" },
                &source,
            ),
            common::rule(
                "chain-cables",
                "线缆管理",
                "多线缆建议加隔片并按粗线缆弯曲半径选 R。".to_string(),
                format!("线缆数量 {}", common::fmt(cable_count)),
                if cable_count > 12.0 { "warning" } else { "low" },
                &source,
            ),
        ],
        risks,
        vec![
            common::requirement("installLength", "安装长度估算", install_length, "mm"),
            common::requirement("bendRadius", "弯曲半径", radius, "mm"),
            common::requirement("fillMargin", "填充余量", fill_margin, "ratio"),
        ],
    ))
}
