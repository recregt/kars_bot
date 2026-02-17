use super::types::{GraphMetric, GraphRequest, GraphWindow};

pub(super) fn parse_graph_request(
    query: &str,
    default_window_minutes: i64,
    max_window_hours: i64,
) -> Option<GraphRequest> {
    let mut args = query.split_whitespace();
    let metric_arg = args.next();
    let window_arg = args.next();
    let has_extra = args.next().is_some();

    let (Some(metric_text), false) = (metric_arg, has_extra) else {
        return None;
    };

    let metric = GraphMetric::parse(metric_text)?;
    let max_window_minutes = max_window_hours.checked_mul(60)?;

    let requested_minutes = match window_arg {
        Some(window_text) => parse_window_minutes(window_text)?,
        None => default_window_minutes,
    };

    if requested_minutes <= 0 || requested_minutes > max_window_minutes {
        return None;
    }

    let window = GraphWindow::from_minutes(requested_minutes)?;

    Some(GraphRequest { metric, window })
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
    use super::parse_graph_request;

    #[test]
    fn parses_metric_with_default_window() {
        let request = parse_graph_request("cpu", 60, 24).expect("request should parse");
        assert_eq!(request.window.minutes(), 60);
    }

    #[test]
    fn parses_metric_with_explicit_window() {
        let request = parse_graph_request("ram 6h", 60, 24).expect("request should parse");
        assert_eq!(request.window.minutes(), 360);
    }

    #[test]
    fn rejects_parse_errors_and_out_of_bounds_windows() {
        assert!(parse_graph_request("", 60, 24).is_none());
        assert!(parse_graph_request("cpu 99x", 60, 24).is_none());
        assert!(parse_graph_request("cpu 25h", 60, 24).is_none());
        assert!(parse_graph_request("cpu 1h extra", 60, 24).is_none());
    }
}