use super::models::RuleDecision;

pub fn timing_belt_rules(
    speed_mm_s: f64,
    efficiency: f64,
    safety_factor: f64,
    source: &str,
) -> Vec<RuleDecision> {
    vec![
        RuleDecision {
            id: "timing-belt-speed".to_string(),
            label: "速度区间".to_string(),
            recommendation: if speed_mm_s <= 2000.0 {
                "同步带传动可进入型号匹配".to_string()
            } else {
                "速度偏高，优先复核齿形、张紧和导轨阻力".to_string()
            },
            basis: format!("目标速度 {speed_mm_s:.3} mm/s，基础阈值 2000 mm/s"),
            risk: if speed_mm_s <= 2000.0 {
                "low"
            } else {
                "warning"
            }
            .to_string(),
            source: source.to_string(),
        },
        RuleDecision {
            id: "timing-belt-efficiency".to_string(),
            label: "效率输入".to_string(),
            recommendation: if efficiency >= 0.7 {
                "效率输入可用于基础扭矩计算".to_string()
            } else {
                "效率过低，建议复核机构阻力或改用人工确认值".to_string()
            },
            basis: format!("传动效率 {efficiency:.3}"),
            risk: if efficiency >= 0.7 { "low" } else { "warning" }.to_string(),
            source: source.to_string(),
        },
        RuleDecision {
            id: "timing-belt-safety-factor".to_string(),
            label: "安全系数".to_string(),
            recommendation: if safety_factor >= 1.2 {
                "安全系数已确认，可记录到结果快照".to_string()
            } else {
                "安全系数偏低，需要复核冲击、偏载和启停工况".to_string()
            },
            basis: format!("用户确认安全系数 {safety_factor:.3}"),
            risk: if safety_factor >= 1.2 {
                "low"
            } else {
                "warning"
            }
            .to_string(),
            source: source.to_string(),
        },
    ]
}
