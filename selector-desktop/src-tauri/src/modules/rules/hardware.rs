use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "hardware-rule-selector";
const SOURCE: &str = "PDF P146 / 文档页 143 / 常用五金件";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "常用五金件规则选型".to_string(),
        category: "规则选型".to_string(),
        description: "按载荷、振动、调节频率、安装空间和拆装需求推荐紧固/定位五金件。".to_string(),
        source_chapter: "常用五金件".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field(
                "load",
                "载荷",
                "N",
                0.0,
                500.0,
                "连接或支撑承受载荷",
                SOURCE,
            ),
            common::field(
                "vibrationLevel",
                "振动等级",
                "score",
                0.0,
                1.0,
                "0低 1中 2高",
                SOURCE,
            ),
            common::field(
                "adjustFrequency",
                "调节频率",
                "score",
                0.0,
                1.0,
                "0少 1偶尔 2频繁",
                SOURCE,
            ),
            common::field(
                "spaceLimit",
                "安装空间",
                "mm",
                1.0,
                20.0,
                "可安装工具和头部空间",
                SOURCE,
            ),
            common::field(
                "disassemblyNeed",
                "拆装需求",
                "score",
                0.0,
                1.0,
                "0少 1偶尔 2频繁",
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
    let load = common::positive(&fields, "load")?;
    let vibration = common::positive_or_zero(&fields, "vibrationLevel")?;
    let adjust = common::positive_or_zero(&fields, "adjustFrequency")?;
    let space = common::positive(&fields, "spaceLimit")?;
    let disassembly = common::positive_or_zero(&fields, "disassemblyNeed")?;
    let design_load = load * safety_factor;
    let recommendation = recommend_hardware(design_load, vibration, adjust, space, disassembly);
    let mut risks = common::safety_risk(safety_factor, &source);
    if vibration >= 2.0 {
        risks.push(common::risk(
            "warning",
            "高振动场景需加防松结构或锁固胶。",
            Some("vibrationLevel"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "hardware-rule-selector@0.1.0",
        format!(
            "建议 {}，设计载荷 {} N",
            recommendation,
            common::fmt(design_load)
        ),
        "五金件规则输出用于类型初筛，规格仍需按螺纹、材料和安装空间复核。".to_string(),
        vec![
            common::step(
                "问题1 设计载荷",
                "F = load * K",
                format!("{}*{}", common::fmt(load), common::fmt(safety_factor)),
                design_load,
                "N",
                &source,
            ),
            common::step(
                "问题2 振动等级",
                "vibrationLevel",
                format!("{}", common::fmt(vibration)),
                vibration,
                "score",
                &source,
            ),
            common::step(
                "问题3 调节频率",
                "adjustFrequency",
                format!("{}", common::fmt(adjust)),
                adjust,
                "score",
                &source,
            ),
            common::step(
                "问题4 安装空间",
                "spaceLimit",
                format!("{} mm", common::fmt(space)),
                space,
                "mm",
                &source,
            ),
        ],
        vec![
            common::rule(
                "hardware-type",
                "推荐五金件",
                recommendation.to_string(),
                format!(
                    "载荷 {} N，空间 {} mm",
                    common::fmt(design_load),
                    common::fmt(space)
                ),
                "low",
                &source,
            ),
            common::rule(
                "hardware-lock",
                "防松风险",
                if vibration >= 2.0 {
                    "建议防松螺母、弹垫、止动垫片或螺纹胶。".to_string()
                } else {
                    "低振动场景可按常规紧固件初筛。".to_string()
                },
                format!("振动 {}", common::fmt(vibration)),
                if vibration >= 2.0 { "warning" } else { "low" },
                &source,
            ),
            common::rule(
                "hardware-adjust",
                "调节维护",
                if adjust >= 2.0 {
                    "频繁调节优先旋钮、手拧件、锁紧手柄或快拆结构。".to_string()
                } else {
                    "低频调节可用常规螺钉和定位件。".to_string()
                },
                format!("调节频率 {}", common::fmt(adjust)),
                if adjust >= 2.0 { "warning" } else { "low" },
                &source,
            ),
        ],
        risks,
        vec![
            common::requirement("designLoad", "设计载荷", design_load, "N"),
            common::requirement("spaceLimit", "安装空间", space, "mm"),
            common::requirement("disassemblyNeed", "拆装需求", disassembly, "score"),
        ],
    ))
}

fn recommend_hardware(
    load: f64,
    vibration: f64,
    adjust: f64,
    space: f64,
    disassembly: f64,
) -> &'static str {
    if adjust >= 2.0 || disassembly >= 2.0 {
        "手拧螺钉、锁紧手柄或快拆件"
    } else if vibration >= 2.0 {
        "防松螺母、螺纹胶和定位销组合"
    } else if load > 1000.0 {
        "高强度螺栓和定位销"
    } else if space < 10.0 {
        "低头螺钉或内六角紧凑安装"
    } else {
        "常规内六角螺钉、销钉和垫圈"
    }
}
