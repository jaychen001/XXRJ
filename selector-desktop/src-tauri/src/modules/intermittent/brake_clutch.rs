use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "brake-clutch-selector";
const SOURCE: &str = "PDF P65 / 文档页 62 / 制动器/离合器";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "制动器/离合器选型".to_string(),
        category: "间歇传动".to_string(),
        description: "按惯量、转速、停止时间和动作频率估算制动/离合扭矩与类型。".to_string(),
        source_chapter: "制动器/离合器".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field(
                "loadTorque",
                "负载扭矩",
                "Nm",
                0.0,
                5.0,
                "保持或传递的外部负载扭矩",
                SOURCE,
            ),
            common::field(
                "rotatingInertia",
                "旋转惯量",
                "kg·m²",
                0.0,
                0.02,
                "折算到制动轴的转动惯量",
                SOURCE,
            ),
            common::field_with_units(
                "shaftSpeed",
                "轴转速",
                "rpm",
                &["rpm", "rps"],
                0.0,
                600.0,
                "制动或离合前轴转速",
                SOURCE,
            ),
            common::field_with_units(
                "stopTime",
                "停止时间",
                "s",
                &["s", "min"],
                0.001,
                0.3,
                "要求停止或接合时间",
                SOURCE,
            ),
            common::field(
                "cyclesPerMinute",
                "动作频率",
                "cycle/min",
                0.0,
                20.0,
                "每分钟制动或离合次数",
                SOURCE,
            ),
            common::field(
                "responseTime",
                "响应时间",
                "ms",
                0.0,
                50.0,
                "控制响应要求",
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
    let load_torque = common::positive_or_zero(&fields, "loadTorque")?;
    let inertia = common::positive_or_zero(&fields, "rotatingInertia")?;
    let speed_rpm = common::convert(
        common::positive(&fields, "shaftSpeed")?,
        common::unit(&fields, "shaftSpeed")?,
        "rpm",
        "shaftSpeed",
    )?;
    let stop_time = common::convert(
        common::positive(&fields, "stopTime")?,
        common::unit(&fields, "stopTime")?,
        "s",
        "stopTime",
    )?;
    let cycles = common::positive_or_zero(&fields, "cyclesPerMinute")?;
    let response_ms = common::positive_or_zero(&fields, "responseTime")?;
    let omega = speed_rpm * std::f64::consts::TAU / 60.0;
    let deceleration = omega / stop_time;
    let inertia_torque = inertia * deceleration;
    let design_torque = (inertia_torque + load_torque) * safety_factor;
    let heat_index = design_torque * cycles;
    let recommendation = recommend_type(cycles, response_ms);
    let mut risks = common::safety_risk(safety_factor, &source);
    if cycles > 60.0 {
        risks.push(common::risk(
            "warning",
            "动作频率超过 60 次/min，需重点复核发热和寿命。",
            Some("cyclesPerMinute"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "brake-clutch-selector@0.1.0",
        format!(
            "设计扭矩 {} Nm，建议 {}",
            common::fmt(design_torque),
            recommendation
        ),
        format!(
            "按安全系数 {} 计算，制动/离合器至少需 {} Nm，并复核热负荷指标 {}。",
            common::fmt(safety_factor),
            common::fmt(design_torque),
            common::fmt(heat_index)
        ),
        vec![
            common::step(
                "角速度",
                "ω = n * 2π / 60",
                format!("{} * 2π / 60", common::fmt(speed_rpm)),
                omega,
                "rad/s",
                &source,
            ),
            common::step(
                "角减速度",
                "α = ω / t",
                format!("{} / {}", common::fmt(omega), common::fmt(stop_time)),
                deceleration,
                "rad/s²",
                &source,
            ),
            common::step(
                "惯量扭矩",
                "Tj = J * α",
                format!("{} * {}", common::fmt(inertia), common::fmt(deceleration)),
                inertia_torque,
                "Nm",
                &source,
            ),
            common::step(
                "设计扭矩",
                "T = (Tj + TL) * K",
                format!(
                    "({} + {}) * {}",
                    common::fmt(inertia_torque),
                    common::fmt(load_torque),
                    common::fmt(safety_factor)
                ),
                design_torque,
                "Nm",
                &source,
            ),
            common::step(
                "热负荷指标",
                "H = T * cycles",
                format!("{} * {}", common::fmt(design_torque), common::fmt(cycles)),
                heat_index,
                "Nm·cycle/min",
                &source,
            ),
        ],
        vec![
            common::rule(
                "brake-clutch-type",
                "类型建议",
                recommendation.to_string(),
                format!(
                    "动作频率 {} 次/min，响应 {} ms",
                    common::fmt(cycles),
                    common::fmt(response_ms)
                ),
                "low",
                &source,
            ),
            common::rule(
                "brake-clutch-heat",
                "热负荷",
                if cycles <= 60.0 {
                    "动作频率可进入常规样本热容量复核。".to_string()
                } else {
                    "动作频率偏高，优先复核热容量和冷却方式。".to_string()
                },
                format!("热负荷指标 {}", common::fmt(heat_index)),
                if cycles <= 60.0 { "low" } else { "warning" },
                &source,
            ),
        ],
        risks,
        vec![
            common::requirement("designTorque", "设计扭矩", design_torque, "Nm"),
            common::requirement("heatIndex", "热负荷指标", heat_index, "Nm·cycle/min"),
            common::requirement("responseTime", "响应时间", response_ms, "ms"),
        ],
    ))
}

fn recommend_type(cycles: f64, response_ms: f64) -> &'static str {
    if response_ms <= 30.0 || cycles > 40.0 {
        "电磁制动器/离合器"
    } else {
        "弹簧加压制动器或常规离合器"
    }
}
