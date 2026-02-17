#[derive(Clone, Copy)]
pub(crate) enum MetricField {
    Cpu,
    Ram,
    Disk,
}

#[derive(Clone, Copy)]
pub(crate) enum Operator {
    Gt,
    Gte,
    Lt,
    Lte,
}

#[derive(Clone, Copy)]
pub(crate) struct MetricCondition {
    pub(crate) field: MetricField,
    pub(crate) op: Operator,
    pub(crate) threshold: f32,
}

#[derive(Clone)]
pub(crate) struct RecentFilters {
    pub(crate) window: Option<chrono::Duration>,
    pub(crate) metrics: Vec<MetricCondition>,
}

#[derive(Clone)]
pub(crate) enum RecentQuery {
    Default,
    Limit(usize),
    Filters(RecentFilters),
}
