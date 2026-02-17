use super::model::{MetricField, Operator, RecentQuery};

pub(crate) fn apply_recent_query(
    events: Vec<crate::anomaly_db::AnomalyEvent>,
    query: RecentQuery,
) -> Vec<crate::anomaly_db::AnomalyEvent> {
    match query {
        RecentQuery::Default | RecentQuery::Limit(_) => events,
        RecentQuery::Filters(filters) => {
            let cutoff = filters.window.map(|window| chrono::Utc::now() - window);

            events
                .into_iter()
                .filter(|event| {
                    if let Some(cutoff) = cutoff {
                        let timestamp_ok = chrono::DateTime::parse_from_rfc3339(&event.timestamp)
                            .map(|ts| ts.with_timezone(&chrono::Utc) >= cutoff)
                            .unwrap_or(false);
                        if !timestamp_ok {
                            return false;
                        }
                    }

                    filters.metrics.iter().all(|condition| {
                        let value = match condition.field {
                            MetricField::Cpu => event.cpu,
                            MetricField::Ram => event.ram,
                            MetricField::Disk => event.disk,
                        };

                        match condition.op {
                            Operator::Gt => value > condition.threshold,
                            Operator::Gte => value >= condition.threshold,
                            Operator::Lt => value < condition.threshold,
                            Operator::Lte => value <= condition.threshold,
                        }
                    })
                })
                .collect()
        }
    }
}
