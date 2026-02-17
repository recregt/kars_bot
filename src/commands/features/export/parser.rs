#[derive(Clone, Copy)]
pub(super) enum ExportMetric {
    Cpu,
    Ram,
    Disk,
}

impl ExportMetric {
    fn parse(input: &str) -> Option<Self> {
        match input.trim().to_lowercase().as_str() {
            "cpu" => Some(Self::Cpu),
            "ram" => Some(Self::Ram),
            "disk" => Some(Self::Disk),
            _ => None,
        }
    }

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            Self::Ram => "ram",
            Self::Disk => "disk",
        }
    }

    pub(super) fn value(self, sample: &crate::monitor::MetricSample) -> f32 {
        match self {
            Self::Cpu => sample.cpu,
            Self::Ram => sample.ram,
            Self::Disk => sample.disk,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum ExportFormat {
    Csv,
    Json,
}

impl ExportFormat {
    fn parse(input: &str) -> Option<Self> {
        match input.trim().to_lowercase().as_str() {
            "csv" => Some(Self::Csv),
            "json" => Some(Self::Json),
            _ => None,
        }
    }

    pub(super) fn extension(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Json => "json",
        }
    }
}

pub(super) struct ExportRequest {
    pub(super) metric: ExportMetric,
    pub(super) window_minutes: i64,
    pub(super) format: ExportFormat,
}

pub(super) fn parse_export_request(
    query: &str,
    default_window_minutes: i64,
    max_window_hours: i64,
) -> Option<ExportRequest> {
    let mut args = query.split_whitespace();
    let metric_arg = args.next()?;
    let second = args.next();
    let third = args.next();
    if args.next().is_some() {
        return None;
    }

    let metric = ExportMetric::parse(metric_arg)?;
    let max_window_minutes = max_window_hours.checked_mul(60)?;

    let mut window_minutes = default_window_minutes;
    let mut format = ExportFormat::Csv;

    match (second, third) {
        (None, None) => {}
        (Some(token), None) => {
            if let Some(parsed_format) = ExportFormat::parse(token) {
                format = parsed_format;
            } else {
                window_minutes = parse_window_minutes(token)?;
            }
        }
        (Some(window_token), Some(format_token)) => {
            window_minutes = parse_window_minutes(window_token)?;
            format = ExportFormat::parse(format_token)?;
        }
        _ => return None,
    }

    if window_minutes <= 0 || window_minutes > max_window_minutes {
        return None;
    }

    Some(ExportRequest {
        metric,
        window_minutes,
        format,
    })
}

pub(super) fn format_window_suffix(window_minutes: i64) -> String {
    if window_minutes % 60 == 0 {
        format!("{}h", window_minutes / 60)
    } else {
        format!("{}m", window_minutes)
    }
}

fn parse_window_minutes(input: &str) -> Option<i64> {
    let value = input.trim().to_lowercase();
    if value.len() < 2 {
        return None;
    }

    let (number_part, unit_part) = value.split_at(value.len() - 1);
    let number = number_part.parse::<i64>().ok()?;
    if number <= 0 {
        return None;
    }

    match unit_part {
        "m" => Some(number),
        "h" => number.checked_mul(60),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::parse_export_request;

    #[test]
    fn parses_default_window_and_format() {
        let request = parse_export_request("cpu", 60, 24).expect("should parse");
        assert_eq!(request.window_minutes, 60);
    }

    #[test]
    fn parses_window_and_format() {
        let request = parse_export_request("ram 6h json", 60, 24).expect("should parse");
        assert_eq!(request.window_minutes, 360);
    }

    #[test]
    fn rejects_invalid_queries() {
        assert!(parse_export_request("", 60, 24).is_none());
        assert!(parse_export_request("cpu 25h", 60, 24).is_none());
        assert!(parse_export_request("cpu foo bar", 60, 24).is_none());
    }
}
