use super::models::{FieldError, RiskItem};

pub fn validate(safety_factor: Option<f64>, confirmed: bool) -> Result<f64, FieldError> {
    if !confirmed {
        return Err(FieldError {
            field_id: "safetyFactor".to_string(),
            message: "安全系数必须由用户手动输入或确认".to_string(),
        });
    }

    let value = safety_factor.ok_or_else(|| FieldError {
        field_id: "safetyFactor".to_string(),
        message: "安全系数不能为空".to_string(),
    })?;

    if value <= 0.0 {
        return Err(FieldError {
            field_id: "safetyFactor".to_string(),
            message: "安全系数必须大于 0".to_string(),
        });
    }

    Ok(value)
}

pub fn risk(value: f64, source: &str) -> Option<RiskItem> {
    if value < 1.2 {
        return Some(RiskItem {
            level: "warning".to_string(),
            message: "安全系数低于 1.2，建议复核冲击、偏载和启停工况。".to_string(),
            field_id: Some("safetyFactor".to_string()),
            source: source.to_string(),
        });
    }

    None
}
