use thiserror::Error;

#[derive(Debug, Error)]
pub enum UnitError {
    #[error("不支持单位换算：{from} -> {to}")]
    Unsupported { from: String, to: String },
}

pub fn convert(value: f64, from: &str, to: &str) -> Result<f64, UnitError> {
    if from == to {
        return Ok(value);
    }

    match (from, to) {
        ("mm", "m") => Ok(value / 1000.0),
        ("m", "mm") => Ok(value * 1000.0),
        ("mm/s", "m/s") => Ok(value / 1000.0),
        ("m/s", "mm/s") => Ok(value * 1000.0),
        ("min", "s") => Ok(value * 60.0),
        ("s", "min") => Ok(value / 60.0),
        ("rpm", "rps") => Ok(value / 60.0),
        ("rps", "rpm") => Ok(value * 60.0),
        ("kW", "W") => Ok(value * 1000.0),
        ("W", "kW") => Ok(value / 1000.0),
        ("kg", "N") => Ok(value * 9.80665),
        ("N", "kg") => Ok(value / 9.80665),
        _ => Err(UnitError::Unsupported {
            from: from.to_string(),
            to: to.to_string(),
        }),
    }
}
