use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "machining-rule-selector";
const SOURCE: &str = "工程规则库 / 机加工";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "机加工规则选型".to_string(),
        category: "规则选型".to_string(),
        description: "按精度、批量、材料硬度和结构复杂度推荐加工方式与注意事项。".to_string(),
        source_chapter: "机加工".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field(
                "tolerance",
                "公差需求",
                "mm",
                0.001,
                0.05,
                "关键尺寸公差",
                SOURCE,
            ),
            common::field("batchQty", "批量", "pcs", 1.0, 5.0, "单次加工数量", SOURCE),
            common::field(
                "hardness",
                "材料硬度",
                "HB",
                0.0,
                180.0,
                "材料硬度或等效加工难度",
                SOURCE,
            ),
            common::field(
                "complexity",
                "结构复杂度",
                "score",
                0.0,
                1.0,
                "0简单 1中等 2复杂",
                SOURCE,
            ),
            common::field(
                "surfaceDemand",
                "表面要求",
                "score",
                0.0,
                1.0,
                "0普通 1较高 2外观/密封",
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
    let tolerance = common::positive(&fields, "tolerance")?;
    let batch = common::positive(&fields, "batchQty")?;
    let hardness = common::positive_or_zero(&fields, "hardness")?;
    let complexity = common::positive_or_zero(&fields, "complexity")?;
    let surface = common::positive_or_zero(&fields, "surfaceDemand")?;
    let precision_index = safety_factor / tolerance;
    let recommendation = recommend_process(tolerance, batch, hardness, complexity, surface);
    let mut risks = common::safety_risk(safety_factor, &source);
    if tolerance < 0.02 || hardness > 300.0 {
        risks.push(common::risk(
            "warning",
            "精度或硬度要求偏高，需增加磨削/慢走丝/热处理后加工评审。",
            Some("tolerance"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "machining-rule-selector@0.1.0",
        format!(
            "建议 {}，精度指标 {}",
            recommendation,
            common::fmt(precision_index)
        ),
        "加工建议需同步考虑装夹基准、热处理变形、表面处理余量和成本。".to_string(),
        vec![
            common::step(
                "问题1 公差",
                "1/tolerance * K",
                format!(
                    "1/{}*{}",
                    common::fmt(tolerance),
                    common::fmt(safety_factor)
                ),
                precision_index,
                "index",
                &source,
            ),
            common::step(
                "问题2 批量",
                "batchQty",
                format!("{}", common::fmt(batch)),
                batch,
                "pcs",
                &source,
            ),
            common::step(
                "问题3 硬度",
                "hardness",
                format!("{} HB", common::fmt(hardness)),
                hardness,
                "HB",
                &source,
            ),
            common::step(
                "问题4 复杂度",
                "complexity",
                format!("{}", common::fmt(complexity)),
                complexity,
                "score",
                &source,
            ),
        ],
        vec![
            common::rule(
                "machining-process",
                "加工方式",
                recommendation.to_string(),
                format!(
                    "公差 {} mm，复杂度 {}",
                    common::fmt(tolerance),
                    common::fmt(complexity)
                ),
                "low",
                &source,
            ),
            common::rule(
                "machining-cost",
                "成本风险",
                if batch <= 3.0 {
                    "小批量优先通用 CNC/车铣，避免专用工装。".to_string()
                } else {
                    "批量加工可评估工装和工序合并。".to_string()
                },
                format!("批量 {} pcs", common::fmt(batch)),
                "low",
                &source,
            ),
            common::rule(
                "machining-quality",
                "质量注意",
                if surface >= 2.0 {
                    "表面或密封要求高，需定义粗糙度和后处理余量。".to_string()
                } else {
                    "普通表面按常规粗糙度和去毛刺控制。".to_string()
                },
                format!("表面要求 {}", common::fmt(surface)),
                if surface >= 2.0 { "warning" } else { "low" },
                &source,
            ),
        ],
        risks,
        vec![
            common::requirement("precisionIndex", "精度指标", precision_index, "index"),
            common::requirement("batchQty", "批量", batch, "pcs"),
            common::requirement("hardness", "材料硬度", hardness, "HB"),
        ],
    ))
}

fn recommend_process(
    tolerance: f64,
    batch: f64,
    hardness: f64,
    complexity: f64,
    surface: f64,
) -> &'static str {
    if tolerance < 0.01 || surface >= 2.0 {
        "磨削/精加工"
    } else if hardness > 300.0 {
        "热处理后磨削或慢走丝"
    } else if complexity >= 2.0 {
        "CNC 加工中心"
    } else if batch > 50.0 {
        "批量 CNC 或专用工装加工"
    } else {
        "车铣常规加工"
    }
}
