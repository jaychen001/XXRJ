use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "sensor-rule-selector";
const SOURCE: &str = "PDF P123 / 文档页 120 / 传感器";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "传感器规则选型".to_string(),
        category: "规则选型".to_string(),
        description: "按检测对象、距离、响应时间、环境和安装空间推荐传感器类型。".to_string(),
        source_chapter: "传感器".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field(
                "objectCode",
                "检测对象",
                "code",
                1.0,
                1.0,
                "1金属 2非金属 3透明 4颜色/标记",
                SOURCE,
            ),
            common::field_with_units(
                "detectDistance",
                "检测距离",
                "mm",
                &["mm", "m"],
                0.0,
                20.0,
                "检测面到传感器距离",
                SOURCE,
            ),
            common::field(
                "responseTime",
                "响应时间",
                "ms",
                0.0,
                10.0,
                "允许响应时间",
                SOURCE,
            ),
            common::field(
                "environmentCode",
                "环境等级",
                "code",
                1.0,
                1.0,
                "1普通 2粉尘/油污 3水汽 4强光/反光",
                SOURCE,
            ),
            common::field(
                "spaceLimit",
                "安装空间",
                "mm",
                1.0,
                30.0,
                "可安装外形空间",
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
    let object_code = common::positive(&fields, "objectCode")?;
    let distance = common::convert(
        common::positive(&fields, "detectDistance")?,
        common::unit(&fields, "detectDistance")?,
        "mm",
        "detectDistance",
    )?;
    let response = common::positive(&fields, "responseTime")?;
    let environment = common::positive(&fields, "environmentCode")?;
    let space = common::positive(&fields, "spaceLimit")?;
    let recommended = recommend_sensor(object_code, distance, environment);
    let distance_margin = distance * safety_factor;
    let mut risks = common::safety_risk(safety_factor, &source);
    if environment >= 3.0 {
        risks.push(common::risk(
            "warning",
            "环境干扰较强，需复核防护等级、抗干扰和安装遮光。",
            Some("environmentCode"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "sensor-rule-selector@0.1.0",
        format!(
            "建议 {}，距离余量 {} mm",
            recommended,
            common::fmt(distance_margin)
        ),
        "传感器规则输出用于样本筛选，最终需现场复核材质、背景和安装角度。".to_string(),
        vec![
            common::step(
                "问题1 检测对象",
                "objectCode",
                format!("{}", common::fmt(object_code)),
                object_code,
                "code",
                &source,
            ),
            common::step(
                "问题2 检测距离",
                "distance * K",
                format!("{}*{}", common::fmt(distance), common::fmt(safety_factor)),
                distance_margin,
                "mm",
                &source,
            ),
            common::step(
                "问题3 响应时间",
                "responseTime",
                format!("{} ms", common::fmt(response)),
                response,
                "ms",
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
                "sensor-type",
                "推荐类型",
                recommended.to_string(),
                format!(
                    "对象代码 {}，距离 {} mm",
                    common::fmt(object_code),
                    common::fmt(distance)
                ),
                "low",
                &source,
            ),
            common::rule(
                "sensor-response",
                "响应风险",
                if response <= 5.0 {
                    "高速节拍需选择高速响应型号。".to_string()
                } else {
                    "响应时间可进入常规模块初筛。".to_string()
                },
                format!("响应 {} ms", common::fmt(response)),
                if response <= 5.0 { "warning" } else { "low" },
                &source,
            ),
            common::rule(
                "sensor-environment",
                "环境适配",
                if environment >= 3.0 {
                    "优先选防水/抗光/抗油污结构并做现场验证。".to_string()
                } else {
                    "普通环境可按距离和对象筛选。".to_string()
                },
                format!("环境代码 {}", common::fmt(environment)),
                if environment >= 3.0 { "warning" } else { "low" },
                &source,
            ),
        ],
        risks,
        vec![
            common::requirement("detectDistance", "检测距离余量", distance_margin, "mm"),
            common::requirement("responseTime", "响应时间", response, "ms"),
            common::requirement("spaceLimit", "安装空间", space, "mm"),
        ],
    ))
}

fn recommend_sensor(object_code: f64, distance: f64, environment: f64) -> &'static str {
    if object_code <= 1.0 {
        "接近开关"
    } else if object_code >= 3.0 {
        "光电传感器或颜色传感器"
    } else if distance > 80.0 {
        "漫反射/对射光电传感器"
    } else if environment >= 3.0 {
        "防护型光电或磁性传感器"
    } else {
        "光电或磁性传感器"
    }
}
