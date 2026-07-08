use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "material-rule-selector";
const SOURCE: &str = "PDF P135 / 文档页 132 / 材料";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "材料规则选型".to_string(),
        category: "规则选型".to_string(),
        description: "按强度、耐磨、耐腐蚀、重量和食品医药需求推荐常用材料。".to_string(),
        source_chapter: "材料".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field(
                "strengthLevel",
                "强度需求",
                "MPa",
                1.0,
                300.0,
                "目标屈服或抗拉强度等级",
                SOURCE,
            ),
            common::field(
                "wearDemand",
                "耐磨需求",
                "score",
                0.0,
                2.0,
                "0低 1中 2高",
                SOURCE,
            ),
            common::field(
                "corrosionDemand",
                "耐腐蚀需求",
                "score",
                0.0,
                1.0,
                "0低 1中 2高",
                SOURCE,
            ),
            common::field(
                "weightSensitive",
                "轻量化需求",
                "score",
                0.0,
                1.0,
                "0不敏感 1敏感 2强敏感",
                SOURCE,
            ),
            common::field(
                "foodGradeDemand",
                "食品医药需求",
                "score",
                0.0,
                0.0,
                "0无 1有",
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
    let strength = common::positive(&fields, "strengthLevel")?;
    let wear = common::positive_or_zero(&fields, "wearDemand")?;
    let corrosion = common::positive_or_zero(&fields, "corrosionDemand")?;
    let weight = common::positive_or_zero(&fields, "weightSensitive")?;
    let food = common::positive_or_zero(&fields, "foodGradeDemand")?;
    let design_strength = strength * safety_factor;
    let recommendation = recommend_material(design_strength, wear, corrosion, weight, food);
    let mut risks = common::safety_risk(safety_factor, &source);
    if corrosion >= 2.0 || food >= 1.0 {
        risks.push(common::risk(
            "warning",
            "腐蚀或食品医药场景需复核材质牌号、证明文件和表面处理。",
            Some("corrosionDemand"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "material-rule-selector@0.1.0",
        format!(
            "建议 {}，设计强度 {} MPa",
            recommendation,
            common::fmt(design_strength)
        ),
        "材料建议只做设计初筛，最终需结合加工、成本、热处理和采购可得性。".to_string(),
        vec![
            common::step(
                "问题1 强度",
                "σd = σ * K",
                format!("{}*{}", common::fmt(strength), common::fmt(safety_factor)),
                design_strength,
                "MPa",
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
                "问题3 耐腐蚀",
                "corrosionDemand",
                format!("{}", common::fmt(corrosion)),
                corrosion,
                "score",
                &source,
            ),
            common::step(
                "问题4 轻量化",
                "weightSensitive",
                format!("{}", common::fmt(weight)),
                weight,
                "score",
                &source,
            ),
        ],
        vec![
            common::rule(
                "material-type",
                "推荐材料",
                recommendation.to_string(),
                format!(
                    "强度 {} MPa，耐磨 {}",
                    common::fmt(design_strength),
                    common::fmt(wear)
                ),
                "low",
                &source,
            ),
            common::rule(
                "material-corrosion",
                "防腐风险",
                if corrosion >= 2.0 {
                    "优先不锈钢或表面处理，并复核环境介质。".to_string()
                } else {
                    "普通防腐需求可按成本和加工性筛选。".to_string()
                },
                format!("耐腐蚀 {}", common::fmt(corrosion)),
                if corrosion >= 2.0 { "warning" } else { "low" },
                &source,
            ),
            common::rule(
                "material-process",
                "加工适配",
                "材料选择需同步复核机加工、热处理和表面处理可行性。".to_string(),
                format!("食品医药 {}", common::fmt(food)),
                if food >= 1.0 { "warning" } else { "low" },
                &source,
            ),
        ],
        risks,
        vec![
            common::requirement("designStrength", "设计强度", design_strength, "MPa"),
            common::requirement("wearDemand", "耐磨需求", wear, "score"),
            common::requirement("corrosionDemand", "耐腐蚀需求", corrosion, "score"),
        ],
    ))
}

fn recommend_material(
    strength: f64,
    wear: f64,
    corrosion: f64,
    weight: f64,
    food: f64,
) -> &'static str {
    if food >= 1.0 || corrosion >= 2.0 {
        "304/316 不锈钢"
    } else if weight >= 2.0 {
        "铝合金"
    } else if wear >= 2.0 {
        "45 钢调质或耐磨工程塑料"
    } else if strength > 500.0 {
        "45 钢或合金钢"
    } else {
        "铝合金、Q235 或 POM"
    }
}
