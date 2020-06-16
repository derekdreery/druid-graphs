//! Some graph widgets for use with druid
use druid::{
    kurbo::{Affine, CircleSegment, Line, Rect},
    piet::{FontBuilder, PietTextLayout, Text, TextLayout, TextLayoutBuilder},
    BoxConstraints, Color, Data, Env, Event, EventCtx, Insets, LayoutCtx, LifeCycle, LifeCycleCtx,
    PaintCtx, RenderContext, Size, UpdateCtx, Widget,
};
use im::Vector;
use std::{convert::TryInto, f64::consts::PI};

mod box_plot;
mod histogram;
mod parts;
mod pie_chart;

pub use box_plot::*;
pub use histogram::*;
pub use pie_chart::*;

const GRAPH_INSETS: Insets = Insets::new(-40.0, -100.0, -40.0, -60.0);

/// For now assume start at 0. Returns gap between each mark.
///
/// `max_value` is the maximum value that will be graphed, and `target_count` is the maximum number
/// of increments of the y axis scale we want.
fn axis_heuristic(max_value: f64, target_count: usize) -> f64 {
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
