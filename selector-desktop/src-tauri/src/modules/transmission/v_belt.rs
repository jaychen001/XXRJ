use crate::engine::models::{CalculationRequest, CalculationResult, FieldError, ModuleDefinition};

use super::super::common;

pub const MODULE_ID: &str = "v-belt-selector";
const SOURCE: &str = "PDF P40 / 文档页 37 / V 带";

pub fn definition() -> ModuleDefinition {
    ModuleDefinition {
        id: MODULE_ID.to_string(),
        name: "V 带选型计算".to_string(),
        category: "传动".to_string(),
        description: "按功率、带轮直径、转速和工况系数判断 V 带速度与类型。".to_string(),
        source_chapter: "V 带".to_string(),
        source_page: SOURCE.to_string(),
        fields: vec![
            common::field_with_units(
                "transmitPower",
                "传递功率",
                "kW",
                &["kW", "W"],
                0.0,
                0.75,
                "电机或负载侧功率",
                SOURCE,
            ),
            common::field_with_units(
                "smallPulleyDiameter",
                "小带轮直径",
                "mm",
                &["mm", "m"],
                0.001,
                100.0,
                "主动小带轮节圆直径",
                SOURCE,
            ),
            common::field_with_units(
                "smallPulleySpeed",
                "小带轮转速",
                "rpm",
                &["rpm", "rps"],
                0.0,
                1450.0,
                "主动小带轮转速",
                SOURCE,
            ),
            common::field(
                "serviceFactor",
                "工况系数",
                "ratio",
                0.1,
                1.3,
                "冲击、启停和工作时长修正",
                SOURCE,
            ),
            common::field(
                "beltEfficiency",
                "传动效率",
                "ratio",
                0.01,
                0.95,
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
    let power_kw = common::convert(
        common::positive(&fields, "transmitPower")?,
        common::unit(&fields, "transmitPower")?,
        "kW",
        "transmitPower",
    )?;
    let diameter_m = common::convert(
        common::positive(&fields, "smallPulleyDiameter")?,
        common::unit(&fields, "smallPulleyDiameter")?,
        "m",
        "smallPulleyDiameter",
    )?;
    let speed_rpm = common::convert(
        common::positive(&fields, "smallPulleySpeed")?,
        common::unit(&fields, "smallPulleySpeed")?,
        "rpm",
        "smallPulleySpeed",
    )?;
    let service_factor = common::positive(&fields, "serviceFactor")?;
    let efficiency = common::efficiency(&fields, "beltEfficiency")?;
    let belt_speed = std::f64::consts::PI * diameter_m * speed_rpm / 60.0;
    let design_power = power_kw * service_factor * safety_factor / efficiency;
    let belt_type = recommend_type(design_power, belt_speed);
    let mut risks = common::safety_risk(safety_factor, &source);
    if !(5.0..=25.0).contains(&belt_speed) {
        risks.push(common::risk(
            "warning",
            "带速不在 5-25 m/s 常用初筛区间，需复核带型和带轮直径。",
            Some("smallPulleyDiameter"),
            &source,
        ));
    }

    Ok(common::result(
        module,
        request,
        "v-belt-selector@0.1.0",
        format!(
            "设计功率 {} kW，带速 {} m/s",
            common::fmt(design_power),
            common::fmt(belt_speed)
        ),
        format!(
            "按安全系数 {} 计算，建议从 {} V 带开始匹配样本。",
            common::fmt(safety_factor),
            belt_type
        ),
        vec![
            common::step(
                "带速",
                "v = π * D * n / 60",
                format!(
                    "π * {} * {} / 60",
                    common::fmt(diameter_m),
                    common::fmt(speed_rpm)
                ),
                belt_speed,
                "m/s",
                &source,
            ),
            common::step(
                "设计功率",
                "Pd = P * Ka * K / η",
                format!(
                    "{} * {} * {} / {}",
                    common::fmt(power_kw),
                    common::fmt(service_factor),
                    common::fmt(safety_factor),
                    common::fmt(efficiency)
                ),
                design_power,
                "kW",
                &source,
            ),
        ],
        vec![
            common::rule(
                "v-belt-type",
                "带型建议",
                format!("优先匹配 {} V 带", belt_type),
                format!(
                    "设计功率 {} kW，带速 {} m/s",
                    common::fmt(design_power),
                    common::fmt(belt_speed)
                ),
                "low",
                &source,
            ),
            common::rule(
                "v-belt-speed",
                "带速区间",
                if (5.0..=25.0).contains(&belt_speed) {
                    "带速处于常用初筛区间。".to_string()
                } else {
                    "带速偏离常用区间，先复核带轮直径和转速。".to_string()
                },
                format!("带速 {} m/s", common::fmt(belt_speed)),
                if (5.0..=25.0).contains(&belt_speed) {
                    "low"
                } else {
                    "warning"
                },
                &source,
            ),
        ],
        risks,
        vec![
            common::requirement("designPower", "设计功率", design_power, "kW"),
            common::requirement("beltSpeed", "带速", belt_speed, "m/s"),
        ],
    ))
}

fn recommend_type(power_kw: f64, belt_speed: f64) -> &'static str {
    if power_kw <= 1.5 && belt_speed <= 18.0 {
        "A 型"
    } else if power_kw <= 7.5 {
        "B 型"
    } else {
        "C 型或多根并联"
    }
}
