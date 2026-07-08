use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "reducer-basic";
const SOURCE: &str = "PDF P54 / 文档页 51 / 减速机";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "减速机基础计算".to_string(),
        category: "传动".to_string(),
        description: "根据输入输出转速、负载扭矩和效率计算减速比与电机侧需求。".to_string(),
        source_chapter: "减速机".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field_with_units(
                "motorSpeed",
                "电机转速",
                "rpm",
                &["rpm", "rps"],
                0.0,
                1500.0,
                "电机额定或工作转速",
                SOURCE,
            ),
            common::field_with_units(
                "outputSpeed",
                "输出转速",
                "rpm",
                &["rpm", "rps"],
                0.0,
                60.0,
                "机构输出轴需求转速",
                SOURCE,
            ),
            common::field(
                "loadTorque",
                "负载扭矩",
                "Nm",
                0.0,
                20.0,
                "输出侧负载扭矩",
                SOURCE,
            ),
            common::field(
                "efficiency",
                "减速机效率",
                "ratio",
                0.01,
                0.85,
                "0-1 之间的小数",
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
    let motor_speed = common::convert(
        common::positive(&fields, "motorSpeed")?,
        common::unit(&fields, "motorSpeed")?,
        "rpm",
        "motorSpeed",
    )?;
    let output_speed = common::convert(
        common::positive(&fields, "outputSpeed")?,
        common::unit(&fields, "outputSpeed")?,
        "rpm",
        "outputSpeed",
    )?;
    let load_torque = common::positive(&fields, "loadTorque")?;
    let efficiency = common::efficiency(&fields, "efficiency")?;

    let ratio = motor_speed / output_speed;
    let output_torque = load_torque * safety_factor;
    let input_torque = output_torque / ratio / efficiency;
    let output_power = output_torque * output_speed * std::f64::consts::TAU / 60.0;
    let mut risks = common::safety_risk(safety_factor, &source);
    if ratio > 100.0 {
        risks.push(common::risk(
            "warning",
            "单级减速比偏大，建议拆成多级或改用组合减速方案。",
            Some("motorSpeed"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "reducer-basic@0.1.0",
        format!(
            "减速比 {}，输入扭矩 {} Nm",
            common::fmt(ratio),
            common::fmt(input_torque)
        ),
        format!(
            "按安全系数 {} 计算，输出侧需 {} Nm，推荐减速比约 {}。",
            common::fmt(safety_factor),
            common::fmt(output_torque),
            common::fmt(ratio)
        ),
        vec![
            common::step(
                "减速比",
                "i = n1 / n2",
                format!(
                    "{} / {}",
                    common::fmt(motor_speed),
                    common::fmt(output_speed)
                ),
                ratio,
                "ratio",
                &source,
            ),
            common::step(
                "输出扭矩",
                "T2 = TL * K",
                format!(
                    "{} * {}",
                    common::fmt(load_torque),
                    common::fmt(safety_factor)
                ),
                output_torque,
                "Nm",
                &source,
            ),
            common::step(
                "输入扭矩",
                "T1 = T2 / i / η",
                format!(
                    "{} / {} / {}",
                    common::fmt(output_torque),
                    common::fmt(ratio),
                    common::fmt(efficiency)
                ),
                input_torque,
                "Nm",
                &source,
            ),
            common::step(
                "输出功率",
                "P = T2 * ω2",
                format!(
                    "{} * {} * 2π / 60",
                    common::fmt(output_torque),
                    common::fmt(output_speed)
                ),
                output_power,
                "W",
                &source,
            ),
        ],
        vec![common::rule(
            "reducer-ratio",
            "减速比区间",
            if ratio <= 100.0 {
                "减速比可进入标准减速机样本匹配。".to_string()
            } else {
                "减速比偏大，建议拆级复核效率和回程间隙。".to_string()
            },
            format!("计算减速比 {}", common::fmt(ratio)),
            if ratio <= 100.0 { "low" } else { "warning" },
            &source,
        )],
        risks,
        vec![
            common::requirement("reducerRatio", "减速比", ratio, "ratio"),
            common::requirement("outputTorque", "输出扭矩", output_torque, "Nm"),
            common::requirement("inputTorque", "输入扭矩", input_torque, "Nm"),
            common::requirement("outputPower", "输出功率", output_power, "W"),
        ],
    ))
}
