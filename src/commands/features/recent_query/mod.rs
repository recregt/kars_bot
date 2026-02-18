mod filter;
mod model;
mod parser;

pub(super) use filter::apply_recent_query;
pub(super) use model::RecentQuery;
pub(super) use parser::parse_recent_query;

#[cfg(test)]
mod tests {
    use super::{RecentQuery, parse_recent_query};

    #[test]
    fn parses_combined_metric_and_window_filters() {
        let query = parse_recent_query(Some("cpu>85 ram>80 6h")).expect("should parse");
        match query {
            RecentQuery::Filters(filters) => {
                assert!(filters.window.is_some());
                assert_eq!(filters.metrics.len(), 2);
            }
            _ => panic!("expected filter query"),
        }
    }

    #[test]
    fn rejects_multiple_windows() {
        let query = parse_recent_query(Some("6h 1d"));
        assert!(query.is_err());
    }

    #[test]
    fn supports_single_limit() {
        let query = parse_recent_query(Some("5")).expect("should parse");
        match query {
            RecentQuery::Limit(5) => {}
            _ => panic!("expected limit query"),
        }
    }
}
