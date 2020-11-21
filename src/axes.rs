use druid::{
    im::{vector, Vector},
    kurbo::Point,
    text::TextStorage,
    Env, PaintCtx, TextLayout,
};
use std::{convert::TryInto, sync::Arc};

/// For now assume start at 0. Returns gap between each mark, in terms of the y variable.
///
/// `max_value` is the maximum value that will be graphed, and `target_count` is the maximum number
/// of increments of the y axis scale we want.
pub fn axis_heuristic(max_value: f64, target_count: usize) -> f64 {
    if target_count <= 1 {
        // nothing will be drawn apart from 0.
        return f64::NAN;
    }
    assert!(max_value > 0., "{} > 0", max_value);
    if target_count == 2 {
        // if there are only 2 scale labels, use the min and max value
        return max_value;
    }
    let target_count = target_count as f64;
    let ideal_scale = max_value / target_count;
    // Find the smalles power of 10 that, if used, would give > the required count of scale
    // labels.
    let mut log_too_many_tens_scale = ideal_scale.log10().floor() as i32;
    let too_many_tens_scale = 10.0f64.powi(log_too_many_tens_scale);
    // check that the next power of 10 would be ok
    debug_assert!(
        max_value / too_many_tens_scale > target_count,
        "{} > {}",
        max_value / too_many_tens_scale,
        target_count
    );
    debug_assert!(
        max_value / (too_many_tens_scale * 10.) <= target_count,
        "{} <= {}",
        max_value / (too_many_tens_scale * 10.),
        target_count
    );
    // try 2 * our power of 10 that gives too many
    if max_value / (2. * too_many_tens_scale) <= target_count {
        return 2. * too_many_tens_scale;
    }
    // next try 5 * our power of 10 that gives too many
    if max_value / (5. * too_many_tens_scale) <= target_count {
        return 5. * too_many_tens_scale;
    }
    // then it must be the next power of 10
    too_many_tens_scale * 10.
}

/// A struct for retaining text layout information for a y axis scale.
pub struct YAxisScale {
    /// (min, max)
    range: (f64, f64),
    scale: Vector<f64>,
    layouts: Option<Vec<TextLayout<Arc<str>>>>,
}

impl YAxisScale {
    pub fn new(max_value: f64) -> Self {
        YAxisScale {
            range: (0., max_value),
            scale: vector![],
            layouts: None,
        }
    }
}

#[derive(Clone)]
pub struct PositionedLayout<T> {
    /// The centre of the label, as a percentage from bottom to top.
    pub position: Point,
    pub layout: TextLayout<T>,
}

impl<T: TextStorage> PositionedLayout<T> {
    pub fn draw(&mut self, ctx: &mut PaintCtx, env: &Env) {
        self.layout.rebuild_if_needed(ctx.text(), env);
        self.layout.draw(ctx, self.position)
    }
}
