use std::time::Duration;

/// Format a duration as a human-readable string with appropriate units.
/// Uses microseconds for < 1ms, milliseconds otherwise.
pub fn format_duration(d: Duration) -> String {
    let micros = d.as_micros();
    if micros < 1000 {
        format!("{}us", micros)
    } else {
        format!("{:.2}ms", d.as_secs_f64() * 1000.0)
    }
}

/// Format a percentage with one decimal place.
pub fn format_percent(value: f64) -> String {
    format!("{:.1}%", value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_duration_microseconds() {
        assert_eq!(format_duration(Duration::from_micros(500)), "500us");
    }

    #[test]
    fn format_duration_milliseconds() {
        let result = format_duration(Duration::from_micros(1500));
        assert_eq!(result, "1.50ms");
    }

    #[test]
    fn format_duration_zero() {
        assert_eq!(format_duration(Duration::ZERO), "0us");
    }

    #[test]
    fn format_percent_normal() {
        assert_eq!(format_percent(12.345), "12.3%");
    }

    #[test]
    fn format_percent_zero() {
        assert_eq!(format_percent(0.0), "0.0%");
    }
}
