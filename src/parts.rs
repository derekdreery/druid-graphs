//! This module provides canvases on which to draw graphs like box plots and histograms

use druid::{
    kurbo::{Line, Size},
    piet::{self, Brush, Color, RenderContext},
    Data,
};
use std::{marker::PhantomData, ops::Range};

const DEFAULT_AXIS_COLOR: Color = Color::grey8(50);

/// An object that knows how to draw 2 axes for a graph.
///
/// The axes need knowledge of each other to be able to position themselves correctly.
#[derive(Debug, Clone, Data)]
pub struct Axes<Text> {
    size: Size,
    axis_color: Color,
    axis_width: f64,
    text_color: Color,
    x_axis: Option<XAxisData>,
    y_axis: Range<f64>,
    dirty: bool,
    text_marker: PhantomData<Text>,
}

#[derive(Debug, Data, PartialEq)]
pub enum XAxisData {
    /// The data are in categories. This can also be used when the data have been put into buckets,
    /// where each bucket is considered a category.
    Categorical(Arc<Vec<String>>),
    /// The data take a fixed set of points, but the distances between them are meaningful.
    Discrete(Range<f64>),
    /// The data can take any numeric value.
    Continuous(Range<f64>),
}

impl<Text> Axes<Text> {
    pub fn new() -> Self {
        Axes {
            size: Size::new(0., 0.),
            interval_font: None,
            label_font: None,
            axis_color: DEFAULT_AXIS_COLOR,
            text_color: Color::BLACK,
            x_axis: None,
            dirty: true,
            text_marker: PhantomData,
        }
    }
}

impl<Text> Axes<Text> {
    pub fn with_interval_font(self, interval_font: &'a Font) -> Self {
        self.interval_font = Some(interval_font);
        self
    }

    pub fn with_label_font(self, label_font: &'a Font) -> Self {
        self.label_font = Some(label_font);
        self
    }

    pub fn with_axis_color(self, axis_color: Color) -> Self {
        self.axis_color = axis_color;
        self
    }

    pub fn with_axis_width(self, axis_width: f64) -> Self {
        self.axis_width = axis_width;
        self
    }

    pub fn with_text_color(self, text_color: Color) -> Self {
        self.text_color = text_color;
        self
    }

    pub fn with_size(self, size: Size) -> Self {
        self.size = size;
        self
    }

    fn build(&mut self) {
        // TODO
        self.dirty = false;
    }
}

impl<Text> Axes<Text>
where
    Text: piet::Text,
{
    /// We assume here that we are drawing in an area from (0, 0) to (size.width, size.height). Use
    /// an affine transformation if you want to draw somewhere else.
    pub fn paint(&self, ctx: impl RenderContext) {
        let axes_brush = Brush::from(self.axis_color);
        let text_brush = Brush::from(self.text_color);
        // x axis
        if let Some(axis) = self.x_axis {
            let x_axis = Line::new((0.0, size.height), (size.width, size.height));
            ctx.stroke(x_axis, &axes_brush, self.axis_width);
            let x_label_layout = ctx
                .text()
                .new_text_layout(&self.label_font, &self.x_axis_label)
                .build()
                .unwrap();
            let x_label_width = x_label_layout.width();
            ctx.draw_text(
                &x_label_layout,
                ((size.width - x_label_width) * 0.5, size.height - 20.0),
                &text_brush,
            );
        }

        // y axis
        {
            let y_axis = Line::new((0., 0.), (0., size.height + 1.0));
            ctx.stroke(y_axis, &axes_brush, 2.0);
            let label_gap =
                linear_axis_heuristic(max_data, (graph_bounds.height() / 40.0).floor() as usize);
            let mut label_pos = 0.0;
            while label_pos < max_data {
                let y_pos = graph_bounds.y1 - (label_pos / max_data) * graph_bounds.height();
                let label_layout = ctx
                    .text()
                    .new_text_layout(&font_sm, &label_pos.to_string())
                    .build()
                    .unwrap();
                let label_width = label_layout.width();
                ctx.draw_text(
                    &label_layout,
                    (graph_bounds.x0 - label_width - 5.0, y_pos + 6.0),
                    &text_brush,
                );
                label_pos += label_gap;
            }
        }
    }

    /// Get the amount that we will draw outside our draw area
    pub fn insets(&self) -> Insets {
        // TODO
        Insets::ZERO
    }
}

/// For now assume start at 0. Returns gap between each mark.
///
/// `max_value` is the maximum value that will be graphed, and `target_count` is the maximum number
/// of increments of the y axis scale we want.
fn linear_axis_heuristic(max_value: f64, target_count: usize) -> f64 {
    // get the biggest power of 10 smaller than max_value
    let mut ten_step = (max_value / target_count as f64).log(10.0).ceil() as usize + 1;
    let mut target_count_mod = target_count;
    while target_count_mod >= 10 {
        ten_step -= 1;
        target_count_mod /= 10;
    }
    let ten_step = 10.0f64.powi(ten_step.try_into().unwrap_or(i32::MAX));
    let count = (ten_step / max_value).floor() as usize;
    if count == target_count {
        return ten_step;
    }
    // try fives now
    let five_step = ten_step * 0.5;
    let count = (max_value / five_step).floor() as usize;
    if count == target_count {
        // we are optimal
        return five_step;
    } else if count > target_count {
        // there are already too many steps
        return ten_step;
    }

    // try twos now
    let two_step = ten_step * 0.2;
    let count = (max_value / two_step).floor() as usize;
    if count > target_count {
        five_step
    } else {
        two_step
    }
}
