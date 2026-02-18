mod cooldown;
mod error;
mod executor;
mod handler;
mod parser;
mod render;
mod stats;
mod types;
mod weekly;

pub(crate) use handler::handle_graph;
pub(crate) use weekly::build_weekly_cpu_report;

pub(crate) fn check_graph_render_readiness() -> Result<(), String> {
    render::check_graph_render_readiness().map_err(|error| {
        format!(
            "startup graph readiness failed code={} error={}",
            error.code(),
            error
        )
    })
}

pub(crate) struct GeneratedGraphReport {
    pub png_bytes: Vec<u8>,
    pub file_name: String,
    pub caption: String,
}
