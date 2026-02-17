use std::io::Cursor;

use image::{DynamicImage, ImageFormat, RgbImage};
use plotters::prelude::*;

use super::{
    stats::GraphPoint,
    types::GraphMetric,
};

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
) -> Result<Vec<u8>, String> {
    if points.len() < 2 {
        return Err("not enough points to render".to_string());
    }

    let width = GRAPH_WIDTH_PX;
    let height = GRAPH_HEIGHT_PX;
    let mut rgb_buffer = vec![255u8; width as usize * height as usize * 3];

    {
        let drawing_area =
            BitMapBackend::with_buffer(&mut rgb_buffer, (width, height)).into_drawing_area();
        drawing_area
            .fill(&GraphStyle::BACKGROUND)
            .map_err(|error| format!("background fill error: {:?}", error))?;

        let mut x_start = points
            .first()
            .map(|point| point.timestamp)
            .ok_or_else(|| "missing start timestamp".to_string())?;
        let mut x_end = points
            .last()
            .map(|point| point.timestamp)
            .ok_or_else(|| "missing end timestamp".to_string())?;

        if x_start == x_end {
            x_start -= chrono::Duration::seconds(1);
            x_end += chrono::Duration::seconds(1);
        }

        let mut chart = ChartBuilder::on(&drawing_area)
            .margin(GraphStyle::MARGIN)
            .caption(
                format!("{} Usage", metric.title()),
                (GraphStyle::CAPTION_FONT_FAMILY, GraphStyle::CAPTION_FONT_SIZE),
            )
            .x_label_area_size(GraphStyle::X_LABEL_AREA_SIZE)
            .y_label_area_size(GraphStyle::Y_LABEL_AREA_SIZE)
            .build_cartesian_2d(x_start..x_end, GraphStyle::Y_MIN..GraphStyle::Y_MAX)
            .map_err(|error| format!("chart build error: {:?}", error))?;

        chart
            .configure_mesh()
            .x_labels(GraphStyle::X_LABEL_COUNT)
            .y_labels(GraphStyle::Y_LABEL_COUNT)
            .y_desc("Usage %")
            .x_desc("Time (UTC)")
            .draw()
            .map_err(|error| format!("mesh draw error: {:?}", error))?;

        chart
            .draw_series(std::iter::once(PathElement::new(
                points
                    .iter()
                    .map(|point| (point.timestamp, point.value))
                    .collect::<Vec<_>>(),
                GraphStyle::metric_line(metric),
            )))
            .map_err(|error| format!("series draw error: {:?}", error))?;

        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![(x_start, threshold), (x_end, threshold)],
                GraphStyle::THRESHOLD_LINE.mix(GraphStyle::THRESHOLD_ALPHA),
            )))
            .map_err(|error| format!("threshold draw error: {:?}", error))?;

        drawing_area
            .present()
            .map_err(|error| format!("present error: {:?}", error))?;
    }

    let rgb_image = RgbImage::from_raw(width, height, rgb_buffer)
        .ok_or_else(|| "image buffer conversion failed".to_string())?;
    let mut output = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(rgb_image)
        .write_to(&mut output, ImageFormat::Png)
        .map_err(|error| format!("png encoding error: {}", error))?;

    Ok(output.into_inner())
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::{render_graph_png, GraphPoint};
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
}