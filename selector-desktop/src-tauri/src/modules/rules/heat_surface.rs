use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "heat-surface-rule-selector";
const SOURCE: &str = "工程规则库 / 热处理&表面处理";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "热处理&表面处理规则选型".to_string(),
        category: "规则选型".to_string(),
        description: "按硬度、耐磨、防腐、外观和尺寸稳定性推荐热处理/表面处理。".to_string(),
        source_chapter: "热处理&表面处理".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field(
                "hardnessTarget",
                "目标硬度",
                "HRC",
                0.0,
                35.0,
                "目标表面或整体硬度",
                SOURCE,
            ),
            common::field(
                "wearDemand",
                "耐磨需求",
                "score",
                0.0,
                1.0,
                "0低 1中 2高",
                SOURCE,
            ),
            common::field(
                "corrosionDemand",
                "防腐需求",
                "score",
                0.0,
                1.0,
                "0低 1中 2高",
                SOURCE,
            ),
            common::field(
                "appearanceDemand",
                "外观需求",
                "score",
                0.0,
                1.0,
                "0低 1中 2高",
                SOURCE,
            ),
            common::field(
                "deformationRisk",
                "变形敏感",
                "score",
                0.0,
                1.0,
                "0低 1中 2高",
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
    let hardness = common::positive_or_zero(&fields, "hardnessTarget")?;
    let wear = common::positive_or_zero(&fields, "wearDemand")?;
    let corrosion = common::positive_or_zero(&fields, "corrosionDemand")?;
    let appearance = common::positive_or_zero(&fields, "appearanceDemand")?;
    let deformation = common::positive_or_zero(&fields, "deformationRisk")?;
    let process_score = hardness / 10.0 + wear + corrosion + appearance;
    let recommendation = recommend_treatment(hardness, wear, corrosion, appearance, deformation);
    let mut risks = common::safety_risk(safety_factor, &source);
    if deformation >= 2.0 && hardness > 45.0 {
        risks.push(common::risk(
            "warning",
            "高硬度且变形敏感，需预留加工余量并评审热处理变形。",
            Some("deformationRisk"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "heat-surface-rule-selector@0.1.0",
        format!(
            "建议 {}，处理需求指标 {}",
            recommendation,
            common::fmt(process_score)
        ),
        "处理建议需与材料、机加工余量和最终尺寸检验一起确认。".to_string(),
        vec![
            common::step(
                "问题1 硬度",
                "hardnessTarget",
                format!("{} HRC", common::fmt(hardness)),
                hardness,
                "HRC",
                &source,
            ),
            common::step(
                "问题2 耐磨",
                "wearDemand",
                format!("{}", common::fmt(wear)),
                wear,
                "score",
                &source,
            ),
            common::step(
                "问题3 防腐",
                "corrosionDemand",
                format!("{}", common::fmt(corrosion)),
                corrosion,
                "score",
                &source,
            ),
            common::step(
                "问题4 外观",
                "appearanceDemand",
                format!("{}", common::fmt(appearance)),
                appearance,
                "score",
                &source,
            ),
        ],
        vec![
            common::rule(
                "treatment-type",
                "推荐处理",
                recommendation.to_string(),
                format!(
                    "硬度 {} HRC，防腐 {}",
                    common::fmt(hardness),
                    common::fmt(corrosion)
                ),
                "low",
                &source,
            ),
            common::rule(
                "treatment-deformation",
                "变形风险",
                if deformation >= 2.0 {
                    "变形敏感件优先表面处理或低变形热处理，并预留精加工。".to_string()
                } else {
                    "变形风险可按常规工艺控制。".to_string()
                },
                format!("变形敏感 {}", common::fmt(deformation)),
                if deformation >= 2.0 { "warning" } else { "low" },
                &source,
            ),
            common::rule(
                "treatment-appearance",
                "外观注意",
                if appearance >= 2.0 {
                    "外观件需明确颜色、膜厚、遮蔽和检验标准。".to_string()
                } else {
                    "普通外观按功能防护优先。".to_string()
                },
                format!("外观需求 {}", common::fmt(appearance)),
                if appearance >= 2.0 { "warning" } else { "low" },
                &source,
            ),
        ],
        risks,
        vec![
            common::requirement("processScore", "处理需求指标", process_score, "score"),
            common::requirement("hardnessTarget", "目标硬度", hardness, "HRC"),
            common::requirement("corrosionDemand", "防腐需求", corrosion, "score"),
        ],
    ))
}

fn recommend_treatment(
    hardness: f64,
    wear: f64,
    corrosion: f64,
    appearance: f64,
    deformation: f64,
) -> &'static str {
    if corrosion >= 2.0 && appearance >= 1.0 {
        "阳极氧化、镀镍或发黑防锈"
    } else if hardness >= 50.0 || wear >= 2.0 {
        "淬火、渗碳或氮化"
    } else if deformation >= 2.0 {
        "表面处理或低温氮化"
    } else if appearance >= 2.0 {
        "阳极氧化或喷涂"
    } else {
        "调质、发黑或常规防锈"
    }
}
