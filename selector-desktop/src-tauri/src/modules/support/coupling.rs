use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "coupling-selector";
const SOURCE: &str = "PDF P116 / 文档页 113 / 联轴器";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "联轴器".to_string(),
        category: "连接件".to_string(),
        description: "按扭矩、冲击、转速、惯量比和偏差补偿判断联轴器类型。".to_string(),
        source_chapter: "联轴器".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field(
                "motorTorque",
                "电机扭矩",
                "Nm",
                0.0,
                2.0,
                "电机额定或峰值扭矩",
                SOURCE,
            ),
            common::field(
                "shockFactor",
                "冲击系数",
                "ratio",
                0.1,
                1.5,
                "启停、冲击和反转修正",
                SOURCE,
            ),
            common::field_with_units(
                "shaftSpeed",
                "轴转速",
                "rpm",
                &["rpm", "rps"],
                0.0,
                1500.0,
                "联轴器工作转速",
                SOURCE,
            ),
            common::field(
                "inertiaRatio",
                "负载惯量比",
                "ratio",
                0.0,
                3.0,
                "负载惯量/电机惯量",
                SOURCE,
            ),
            common::field(
                "parallelOffset",
                "平行偏差",
                "mm",
                0.0,
                0.05,
                "两轴平行偏差需求",
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
    let motor_torque = common::positive(&fields, "motorTorque")?;
    let shock = common::positive(&fields, "shockFactor")?;
    let speed = common::convert(
        common::positive(&fields, "shaftSpeed")?,
        common::unit(&fields, "shaftSpeed")?,
        "rpm",
        "shaftSpeed",
    )?;
    let inertia_ratio = common::positive_or_zero(&fields, "inertiaRatio")?;
    let offset = common::positive_or_zero(&fields, "parallelOffset")?;
    let design_torque = motor_torque * shock * safety_factor;
    let torsional_index = design_torque * (1.0 + inertia_ratio);
    let recommendation = recommend_type(speed, inertia_ratio, offset, shock);
    let mut risks = common::safety_risk(safety_factor, &source);
    if inertia_ratio > 5.0 {
        risks.push(common::risk(
            "warning",
            "负载惯量比过高，联轴器刚性和伺服调试风险上升。",
            Some("inertiaRatio"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "coupling-selector@0.1.0",
        format!(
            "设计扭矩 {} Nm，建议 {}",
            common::fmt(design_torque),
            recommendation
        ),
        format!(
            "按冲击系数 {} 和安全系数 {} 修正，联轴器额定扭矩至少 {} Nm。",
            common::fmt(shock),
            common::fmt(safety_factor),
            common::fmt(design_torque)
        ),
        vec![
            common::step(
                "设计扭矩",
                "T = Tm * Ka * K",
                format!(
                    "{}*{}*{}",
                    common::fmt(motor_torque),
                    common::fmt(shock),
                    common::fmt(safety_factor)
                ),
                design_torque,
                "Nm",
                &source,
            ),
            common::step(
                "扭转需求指标",
                "It = T * (1 + Jratio)",
                format!(
                    "{}*(1+{})",
                    common::fmt(design_torque),
                    common::fmt(inertia_ratio)
                ),
                torsional_index,
                "index",
                &source,
            ),
        ],
        vec![
            common::rule(
                "coupling-type",
                "类型建议",
                recommendation.to_string(),
                format!(
                    "转速 {} rpm，偏差 {} mm",
                    common::fmt(speed),
                    common::fmt(offset)
                ),
                "low",
                &source,
            ),
            common::rule(
                "coupling-offset",
                "偏差补偿",
                "按样册复核平行、角向和轴向三类偏差，不能只看扭矩。".to_string(),
                format!("平行偏差 {} mm", common::fmt(offset)),
                if offset <= 0.2 { "low" } else { "warning" },
                &source,
            ),
        ],
        risks,
        vec![
            common::requirement("designTorque", "设计扭矩", design_torque, "Nm"),
            common::requirement("torsionalIndex", "扭转需求指标", torsional_index, "index"),
            common::requirement("shaftSpeed", "轴转速", speed, "rpm"),
        ],
    ))
}

fn recommend_type(speed: f64, inertia_ratio: f64, offset: f64, shock: f64) -> &'static str {
    if speed > 1500.0 || inertia_ratio <= 3.0 {
        "膜片/波纹管联轴器"
    } else if shock > 1.8 || offset > 0.2 {
        "梅花弹性联轴器"
    } else {
        "刚性或弹性联轴器"
    }
}
