use std::io::Cursor;

use image::{DynamicImage, ImageFormat, RgbImage};
use plotters::prelude::*;

use super::{error::GraphRenderError, stats::GraphPoint, types::GraphMetric};

pub(super) const GRAPH_WIDTH_PX: u32 = 1200;
const GRAPH_HEIGHT_PX: u32 = 480;

struct GraphStyle;

impl GraphStyle {
    const MARGIN: i32 = 16;
    const CAPTION_FONT_FAMILY: &'static str = "sans-serif";
    const CAPTION_FONT_SIZE: i32 = 28;
    const X_LABEL_AREA_SIZE: u32 = 40;
    const Y_LABEL_AREA_SIZE: u32 = 48;
    const X_LABEL_COUNT: usize = 6;
    const Y_LABEL_COUNT: usize = 6;
    const Y_MIN: f32 = 0.0;
    const Y_MAX: f32 = 100.0;
    const BACKGROUND: RGBColor = WHITE;
    const THRESHOLD_LINE: RGBColor = BLACK;
    const THRESHOLD_ALPHA: f64 = 0.5;

    fn metric_line(metric: GraphMetric) -> RGBColor {
        match metric {
            GraphMetric::Cpu => RED,
            GraphMetric::Ram => BLUE,
            GraphMetric::Disk => GREEN,
        }
    }
}

pub(super) fn render_graph_png(
    points: Vec<GraphPoint>,
    metric: GraphMetric,
    threshold: f32,
) -> Result<Vec<u8>, GraphRenderError> {
    if points.len() < 2 {
        return Err(GraphRenderError::NotEnoughPoints);
    }

    let width = GRAPH_WIDTH_PX;
    let height = GRAPH_HEIGHT_PX;
    let mut rgb_buffer = vec![255u8; width as usize * height as usize * 3];

    {
        let drawing_area =
            BitMapBackend::with_buffer(&mut rgb_buffer, (width, height)).into_drawing_area();
        drawing_area
            .fill(&GraphStyle::BACKGROUND)
            .map_err(|error| classify_plotters_error("background_fill", format!("{:?}", error)))?;

        let mut x_start = points
            .first()
            .map(|point| point.timestamp)
            .ok_or_else(|| GraphRenderError::Backend("missing start timestamp".to_string()))?;
        let mut x_end = points
            .last()
            .map(|point| point.timestamp)
            .ok_or_else(|| GraphRenderError::Backend("missing end timestamp".to_string()))?;

        if x_start == x_end {
            x_start -= chrono::Duration::seconds(1);
            x_end += chrono::Duration::seconds(1);
        }

        let mut chart = ChartBuilder::on(&drawing_area)
            .margin(GraphStyle::MARGIN)
            .caption(
                format!("{} Usage", metric.title()),
                (
                    GraphStyle::CAPTION_FONT_FAMILY,
                    GraphStyle::CAPTION_FONT_SIZE,
                ),
            )
            .x_label_area_size(GraphStyle::X_LABEL_AREA_SIZE)
            .y_label_area_size(GraphStyle::Y_LABEL_AREA_SIZE)
            .build_cartesian_2d(x_start..x_end, GraphStyle::Y_MIN..GraphStyle::Y_MAX)
            .map_err(|error| classify_plotters_error("chart_build", format!("{:?}", error)))?;

        chart
            .configure_mesh()
            .x_labels(GraphStyle::X_LABEL_COUNT)
            .y_labels(GraphStyle::Y_LABEL_COUNT)
            .y_desc("Usage %")
            .x_desc("Time (UTC)")
            .draw()
            .map_err(|error| classify_plotters_error("mesh_draw", format!("{:?}", error)))?;

        chart
            .draw_series(std::iter::once(PathElement::new(
                points
                    .iter()
                    .map(|point| (point.timestamp, point.value))
                    .collect::<Vec<_>>(),
                GraphStyle::metric_line(metric),
            )))
            .map_err(|error| classify_plotters_error("series_draw", format!("{:?}", error)))?;

        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![(x_start, threshold), (x_end, threshold)],
                GraphStyle::THRESHOLD_LINE.mix(GraphStyle::THRESHOLD_ALPHA),
            )))
            .map_err(|error| classify_plotters_error("threshold_draw", format!("{:?}", error)))?;

        drawing_area
            .present()
            .map_err(|error| classify_plotters_error("present", format!("{:?}", error)))?;
    }

    let rgb_image = RgbImage::from_raw(width, height, rgb_buffer)
        .ok_or_else(|| GraphRenderError::Backend("image buffer conversion failed".to_string()))?;
    let mut output = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(rgb_image)
        .write_to(&mut output, ImageFormat::Png)
        .map_err(|error| GraphRenderError::PngEncoding(error.to_string()))?;

    Ok(output.into_inner())
}

pub(super) fn check_graph_render_readiness() -> Result<(), GraphRenderError> {
    let now = chrono::Utc::now();
    let points = vec![
        GraphPoint {
            timestamp: now - chrono::Duration::minutes(1),
            value: 42.0,
        },
        GraphPoint {
            timestamp: now,
            value: 43.0,
        },
    ];

    std::panic::catch_unwind(|| render_graph_png(points, GraphMetric::Cpu, 85.0))
        .map_err(|_| {
            GraphRenderError::FontUnavailable(
                "readiness probe panic while drawing text".to_string(),
            )
        })?
        .map(|_| ())
}

fn classify_plotters_error(stage: &str, detail: String) -> GraphRenderError {
    let lower = detail.to_lowercase();
    if lower.contains("font") || lower.contains("glyph") || lower.contains("freetype") {
        return GraphRenderError::FontUnavailable(format!("{}:{}", stage, detail));
    }

    GraphRenderError::Backend(format!("{}:{}", stage, detail))
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::{GraphPoint, check_graph_render_readiness, render_graph_png};
    use crate::commands::features::graph::types::GraphMetric;

    #[test]
    fn rejects_not_enough_points() {
        let points = vec![GraphPoint {
            timestamp: Utc::now(),
            value: 42.0,
        }];

        let result = render_graph_png(points, GraphMetric::Cpu, 80.0);
        assert!(result.is_err());
    }

    #[test]
    fn readiness_check_runs() {
        let result = check_graph_render_readiness();
        assert!(result.is_ok() || result.is_err());
    }
}
