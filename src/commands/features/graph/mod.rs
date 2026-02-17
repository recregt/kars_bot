mod cooldown;
mod handler;
mod parser;
mod render;
mod stats;
mod types;
mod weekly;

pub(crate) use handler::handle_graph;
pub(crate) use weekly::build_weekly_cpu_report;

pub(crate) struct GeneratedGraphReport {
    pub png_bytes: Vec<u8>,
    pub file_name: String,
    pub caption: String,
}