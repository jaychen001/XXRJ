use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "robot-rule-selector";
const SOURCE: &str = "工程规则库 / 机器人";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "机器人规则选型".to_string(),
        category: "规则选型".to_string(),
        description: "按负载、臂展、节拍、精度和应用场景推荐机器人类型。".to_string(),
        source_chapter: "机器人".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field(
                "payload",
                "负载",
                "kg",
                0.0,
                5.0,
                "末端负载，含夹具和工件",
                SOURCE,
            ),
            common::field_with_units(
                "reach",
                "臂展",
                "mm",
                &["mm", "m"],
                1.0,
                600.0,
                "最大工作半径或行程",
                SOURCE,
            ),
            common::field_with_units(
                "cycleTime",
                "节拍",
                "s",
                &["s", "min"],
                0.001,
                2.0,
                "单循环动作时间",
                SOURCE,
            ),
            common::field(
                "precision",
                "重复精度",
                "mm",
                0.001,
                0.05,
                "目标重复定位精度",
                SOURCE,
            ),
            common::field(
                "applicationCode",
                "应用场景",
                "code",
                1.0,
                1.0,
                "1搬运 2装配 3高速取放 4焊接/喷涂",
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
    let payload = common::positive(&fields, "payload")?;
    let reach_mm = common::convert(
        common::positive(&fields, "reach")?,
        common::unit(&fields, "reach")?,
        "mm",
        "reach",
    )?;
    let cycle = common::convert(
        common::positive(&fields, "cycleTime")?,
        common::unit(&fields, "cycleTime")?,
        "s",
        "cycleTime",
    )?;
    let precision = common::positive(&fields, "precision")?;
    let application = common::positive(&fields, "applicationCode")?;
    let design_payload = payload * safety_factor;
    let recommendation = recommend_robot(design_payload, reach_mm, cycle, precision, application);
    let mut risks = common::safety_risk(safety_factor, &source);
    if design_payload > 20.0 || reach_mm > 1200.0 {
        risks.push(common::risk(
            "warning",
            "负载或臂展偏大，需复核机器人额定负载曲线和安装姿态。",
            Some("payload"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "robot-rule-selector@0.1.0",
        format!(
            "建议 {}，设计负载 {} kg",
            recommendation,
            common::fmt(design_payload)
        ),
        "规则选型结果可作为厂家样本库筛选条件，最终型号需复核负载曲线、惯量和节拍。".to_string(),
        vec![
            common::step(
                "问题1 负载余量",
                "payload * K",
                format!("{} * {}", common::fmt(payload), common::fmt(safety_factor)),
                design_payload,
                "kg",
                &source,
            ),
            common::step(
                "问题2 臂展",
                "reach",
                format!("{} mm", common::fmt(reach_mm)),
                reach_mm,
                "mm",
                &source,
            ),
            common::step(
                "问题3 节拍",
                "cycleTime",
                format!("{} s", common::fmt(cycle)),
                cycle,
                "s",
                &source,
            ),
            common::step(
                "问题4 精度",
                "precision",
                format!("{} mm", common::fmt(precision)),
                precision,
                "mm",
                &source,
            ),
        ],
        vec![
            common::rule(
                "robot-type",
                "推荐类型",
                recommendation.to_string(),
                format!(
                    "负载 {} kg，臂展 {} mm",
                    common::fmt(design_payload),
                    common::fmt(reach_mm)
                ),
                "low",
                &source,
            ),
            common::rule(
                "robot-speed",
                "节拍判断",
                if cycle <= 1.0 {
                    "优先高速取放结构，复核振动和夹具重量。".to_string()
                } else {
                    "常规搬运/装配机器人可进入样本筛选。".to_string()
                },
                format!("节拍 {} s", common::fmt(cycle)),
                if cycle <= 1.0 { "warning" } else { "low" },
                &source,
            ),
            common::rule(
                "robot-precision",
                "精度风险",
                if precision < 0.03 {
                    "高精度场景需复核重复精度、刚性和末端工具误差。".to_string()
                } else {
                    "精度需求处于常规初筛范围。".to_string()
                },
                format!("重复精度 {} mm", common::fmt(precision)),
                if precision < 0.03 { "warning" } else { "low" },
                &source,
            ),
        ],
        risks,
        vec![
            common::requirement("designPayload", "设计负载", design_payload, "kg"),
            common::requirement("reach", "臂展", reach_mm, "mm"),
            common::requirement("cycleTime", "节拍", cycle, "s"),
        ],
    ))
}

fn recommend_robot(
    payload: f64,
    reach: f64,
    cycle: f64,
    precision: f64,
    application: f64,
) -> &'static str {
    if application >= 3.0 && cycle <= 1.0 && payload <= 5.0 {
        "SCARA 或 Delta 高速取放机器人"
    } else if precision < 0.03 && reach <= 1000.0 {
        "直角坐标或高刚性装配机器人"
    } else if payload > 10.0 || reach > 900.0 {
        "六轴工业机器人"
    } else {
        "SCARA 或小型六轴机器人"
    }
}
