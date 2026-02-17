use super::model::{MetricCondition, MetricField, Operator, RecentFilters, RecentQuery};

pub(crate) fn parse_recent_query(query: Option<&str>) -> Result<RecentQuery, String> {
    let Some(raw) = query.map(str::trim) else {
        return Ok(RecentQuery::Default);
    };

    if raw.is_empty() {
        return Ok(RecentQuery::Default);
    }

    if let Ok(limit) = raw.parse::<usize>() {
        if (1..=100).contains(&limit) {
            return Ok(RecentQuery::Limit(limit));
        }
        return Err("limit must be between 1 and 100".to_string());
    }

    let mut window = None;
    let mut metrics = Vec::new();

    for token in raw.split_whitespace() {
        if let Some(parsed_window) = parse_window(token) {
            if window.is_some() {
                return Err(
                    "multiple time windows are not allowed; use a single token like 6h".to_string(),
                );
            }
            window = Some(parsed_window);
            continue;
        }

        if let Some(metric_filter) = parse_metric_filter(token) {
            metrics.push(metric_filter);
            continue;
        }

        return Err(format!(
            "unsupported token '{}' (expected one of: 6h, cpu>85, ram<=70, disk>90)",
            token
        ));
    }

    if window.is_none() && metrics.is_empty() {
        return Err("unsupported query format".to_string());
    }

    Ok(RecentQuery::Filters(RecentFilters { window, metrics }))
}

fn parse_window(raw: &str) -> Option<chrono::Duration> {
    if raw.len() < 2 {
        return None;
    }

    let (number_part, unit_part) = raw.split_at(raw.len() - 1);
    let value = number_part.parse::<i64>().ok()?;
    if value <= 0 {
        return None;
    }

    match unit_part {
        "m" => Some(chrono::Duration::minutes(value)),
        "h" => Some(chrono::Duration::hours(value)),
        "d" => Some(chrono::Duration::days(value)),
        _ => None,
    }
}

fn parse_metric_filter(raw: &str) -> Option<MetricCondition> {
    let normalized = raw.replace(' ', "").to_lowercase();
    let (left, op, right) = if let Some((left, right)) = normalized.split_once(">=") {
        (left, Operator::Gte, right)
    } else if let Some((left, right)) = normalized.split_once("<=") {
        (left, Operator::Lte, right)
    } else if let Some((left, right)) = normalized.split_once('>') {
        (left, Operator::Gt, right)
    } else if let Some((left, right)) = normalized.split_once('<') {
        (left, Operator::Lt, right)
    } else {
        return None;
    };

    let field = match left {
        "cpu" => MetricField::Cpu,
        "ram" => MetricField::Ram,
        "disk" => MetricField::Disk,
        _ => return None,
    };

    let threshold = right.parse::<f32>().ok()?;

    Some(MetricCondition {
        field,
        op,
        threshold,
    })
}
