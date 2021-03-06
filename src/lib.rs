//! Some graph widgets for use with druid
use druid::{kurbo::Rect, Color, Insets};

mod axes;
mod box_plot;
mod histogram;
mod line_chart;
mod pie_chart;
mod range;
pub mod theme;

pub use crate::{
    box_plot::{BoxPlot, BoxPlotData},
    histogram::{Histogram, HistogramData},
    line_chart::{LineChart, LineChartData},
    pie_chart::{PieChart, PieChartData},
    range::Range,
    theme::add_to_env,
};

const GRAPH_INSETS: Insets = Insets::new(-200.0, -100.0, -40.0, -60.0);

fn new_color(idx: usize) -> Color {
    let idx = idx as f64;
    // use a number that is fairly coprime with 360.
    Color::hlc(idx * 140.0, 50.0, 50.0)
}

/// Take a rect and shrink it to a square centered within the original rectangle.
fn square(input: Rect) -> Rect {
    let (width, height) = (input.width(), input.height());
    assert!(width >= 0.0 && height >= 0.0);
    if width == height {
        input
    } else if width < height {
        let half_overlap = 0.5 * (height - width);
        let y0 = input.y0 + half_overlap;
        let y1 = input.y1 - half_overlap;
        Rect::new(input.x0, y0, input.x1, y1)
    } else {
        let half_overlap = 0.5 * (width - height);
        let x0 = input.x0 + half_overlap;
        let x1 = input.x1 - half_overlap;
        Rect::new(x0, input.y0, x1, input.y1)
    }
}
