use super::models::ReportPayload;

pub fn report_lines(payload: &ReportPayload) -> Vec<String> {
    let mut lines = vec![
        "非标选型计算报告".to_string(),
        format!("案例：{}", empty_dash(&payload.case_name)),
        format!("模块：{}", payload.result.module_name),
        format!("摘要：{}", payload.result.summary),
        format!("结论：{}", payload.result.conclusion),
        format!("备注：{}", empty_dash(&payload.notes)),
        String::new(),
        "输入参数".to_string(),
    ];

    lines.extend(
        payload
            .request
            .fields
            .iter()
            .map(|field| format!("{} = {:.6} {}", field.id, field.value, field.unit)),
    );
    lines.push(format!(
        "安全系数 = {}",
        payload
            .request
            .safety_factor
            .map(|value| format!("{value:.3}"))
            .unwrap_or_else(|| "-".to_string())
    ));
    lines.push(String::new());
    lines.push("计算过程".to_string());
    lines.extend(payload.result.steps.iter().map(|step| {
        format!(
            "{}：{}；{}；{} {}",
            step.label, step.formula, step.substitution, step.result, step.unit
        )
    }));
    lines.push(String::new());
    lines.push("风险与规则".to_string());
    lines.extend(payload.result.rules.iter().map(|rule| {
        format!("{}：{}；依据 {}", rule.label, rule.recommendation, rule.basis)
    }));
    lines.extend(
        payload
            .result
            .risks
            .iter()
            .map(|risk| format!("{}：{}", risk.level, risk.message)),
    );
    lines.push(String::new());
    lines.push("候选型号".to_string());
    if payload.candidates.is_empty() {
        lines.push("无候选型号记录".to_string());
    } else {
        lines.extend(payload.candidates.iter().map(|candidate| {
            format!(
                "{} / {} / 匹配 {:.0}%",
                candidate.model.model_name,
                candidate.model.library_name,
                candidate.score * 100.0
            )
        }));
    }
    lines.push(format!(
        "最终选择：{}",
        payload
            .final_model_name
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("未选择")
    ));
    lines
}

fn empty_dash(value: &str) -> &str {
    if value.trim().is_empty() {
        "-"
    } else {
        value
    }
}
