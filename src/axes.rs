use crate::{theme, Range};
use druid::{
    kurbo::{Line, Point, Rect},
    text::TextStorage,
    ArcStr, Color, Env, KeyOrValue, PaintCtx, RenderContext, Size, TextLayout, UpdateCtx,
};

const SCALE_TICK_MARGIN: f64 = 5.;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Direction {
    X,
    Y,
}

impl Direction {
    /// How many labels can we fit. It's a guess
    fn max_labels(self, bounds: Rect) -> usize {
        match self {
            Direction::X => (bounds.width() / 100.).floor() as usize + 1,
            Direction::Y => (bounds.height() / 40.).floor() as usize + 1,
        }
    }

    fn label_position(self, bounds: Rect, t: f64, size: Size, margin: f64) -> Point {
        let p = self.position(bounds, t);
        match self {
            Direction::X => Point::new(p - 0.5 * size.width, bounds.y1 + SCALE_TICK_MARGIN),
            Direction::Y => Point::new(
                bounds.x0 - size.width - SCALE_TICK_MARGIN,
                p - 0.5 * size.height,
            ),
        }
    }

    fn position(self, bounds: Rect, t: f64) -> f64 {
        match self {
            Direction::X => bounds.x0 + t * bounds.width(),
            Direction::Y => bounds.y1 - t * bounds.height(),
        }
    }

    fn axis_line(self, Rect { x0, y0, x1, y1 }: Rect) -> Line {
        match self {
            Direction::X => Line::new((x0, y1), (x1, y1)),
            Direction::Y => Line::new((x0, y0), (x0, y1)),
        }
    }
}

/// A struct for retaining text layout information for a y axis scale.
///
/// [matplotlib ticker](https://github.com/matplotlib/matplotlib/blob/master/lib/matplotlib/ticker.py#L2057)
/// is a good resource.
#[derive(Clone)]
pub struct Scale {
    direction: Direction,
    /// (min, max) the range of the data we are graphing. Can overspill if you want gaps at the
    /// top/bottom, or include 0 if you want.
    data_range: Range,
    /// The graph area
    graph_bounds: Rect,
    /// Axis/mark color
    axis_color: KeyOrValue<Color>,
    // retained
    /// Our computed scale. The length is the computed number of scale ticks we should show. Format
    /// is `(data value, y-coordinate of the tick)`
    scale_ticker: Option<Ticker>,
    /// Our computed text layouts for the tick labels.
    layouts: Option<Vec<PositionedLayout<ArcStr>>>,
}

impl Scale {
    /// Create a new scale object.
    ///
    ///  - `data_range` is the range of the data, from lowest to highest.
    ///  - `graph_bounds` is the rectangle where the graph will be drawn. We will draw outside this
    ///    area a bit.
    pub fn new(data_range: impl Into<Range>, direction: Direction) -> Self {
        Scale {
            direction,
            data_range: data_range.into(),
            graph_bounds: Rect::ZERO,
            axis_color: theme::AXES_COLOR.into(),
            scale_ticker: None,
            layouts: None,
        }
    }

    pub fn new_y(data_range: impl Into<Range>) -> Self {
        Self::new(data_range, Direction::Y)
    }

    pub fn new_x(data_range: impl Into<Range>) -> Self {
        Self::new(data_range, Direction::X)
    }

    pub fn set_direction(&mut self, d: Direction) {
        if self.direction != d {
            self.direction = d;
            self.invalidate();
        }
    }

    /// Helper function to make sure the range includes 0.
    pub fn include_zero(&mut self) {
        if self.data_range.extend_to(0.) {
            self.invalidate();
        }
    }

    pub fn needs_rebuild_after_update(&mut self, ctx: &mut UpdateCtx) -> bool {
        match self.layouts.as_mut() {
            Some(layouts) => layouts
                .iter_mut()
                .any(|layout| layout.layout.needs_rebuild_after_update(ctx)),
            None => false,
        }
    }

    /// Rebuild the retained state, as needed.
    pub fn rebuild_if_needed(&mut self, ctx: &mut PaintCtx, env: &Env) {
        if self.scale_ticker.is_none() {
            self.layouts = None;
            self.scale_ticker = Some(Ticker::new(
                self.data_range,
                self.direction.max_labels(self.graph_bounds),
            ));
        }
        if self.layouts.is_none() {
            self.layouts = Some(
                self.scale_ticker
                    .unwrap()
                    .into_iter()
                    .map(|tick| {
                        let mut layout = TextLayout::from_text(for_label(tick.value));
                        layout.rebuild_if_needed(ctx.text(), env);
                        let size = layout.size();
                        let mut layout = PositionedLayout {
                            position: self.direction.label_position(
                                self.graph_bounds,
                                tick.t,
                                layout.size(),
                                SCALE_TICK_MARGIN,
                            ),
                            layout,
                        };
                        layout.rebuild_if_needed(ctx, env);
                        layout
                    })
                    .collect(),
            )
        }
    }

    pub fn graph_bounds(&self) -> Rect {
        self.graph_bounds
    }

    pub fn set_graph_bounds(&mut self, graph_bounds: Rect) {
        let graph_bounds = graph_bounds.abs();
        if self.graph_bounds != graph_bounds {
            self.invalidate();
            self.graph_bounds = graph_bounds;
        }
    }

    pub fn set_axis_color(&mut self, color: impl Into<KeyOrValue<Color>>) {
        self.axis_color = color.into();
    }

    fn invalidate(&mut self) {
        self.scale_ticker = None;
        self.layouts = None;
    }

    pub fn draw(&mut self, ctx: &mut PaintCtx, env: &Env) {
        // draw axis
        let axis_brush = ctx.solid_brush(self.axis_color.resolve(env));
        ctx.stroke(self.direction.axis_line(self.graph_bounds), &axis_brush, 2.);
        // draw tick labels
        for layout in self.layouts.as_mut().unwrap().iter_mut() {
            layout.draw(ctx);
        }
    }

    /// Convert a data point to a pixel location on this axis
    pub fn pixel_location(&self, v: f64) -> f64 {
        let (min, max) = self.data_range.into();
        let t = (v - min) / (max - min);
        self.direction.position(self.graph_bounds(), t)
    }
}

#[derive(Clone)]
pub struct PositionedLayout<T> {
    /// The position that the layout should be displayed.
    pub position: Point,
    pub layout: TextLayout<T>,
}

impl<T: TextStorage> PositionedLayout<T> {
    pub fn rebuild_if_needed(&mut self, ctx: &mut PaintCtx, env: &Env) {
        self.layout.rebuild_if_needed(ctx.text(), env);
    }
    pub fn draw(&mut self, ctx: &mut PaintCtx) {
        self.layout.draw(ctx, self.position)
    }
}

/// Able to return a sequence of locations along an axis where ticks should be displayed, and the
/// values that should be displayed there.
#[derive(Debug, Copy, Clone)]
pub struct Ticker {
    data_range: Range,
    target_num_points: usize,
    // calculated
    spacing: f64,
}

impl Ticker {
    pub fn new(data_range: Range, target_num_points: usize) -> Self {
        let spacing = calc_tick_spacing(data_range, target_num_points as f64);
        Self {
            data_range,
            target_num_points,
            spacing,
        }
    }

    fn first_tick(&self) -> f64 {
        match self.target_num_points {
            0 | 1 | 2 => self.data_range.min(),
            n => calc_next_tick(self.data_range.min(), self.spacing),
        }
    }
}

impl IntoIterator for Ticker {
    type IntoIter = TickerIter;
    type Item = Tick;

    fn into_iter(self) -> Self::IntoIter {
        TickerIter {
            inner: self,
            next_tick: 0,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
/// The position at which a tick should be drawn.
pub struct Tick {
    /// The distance along the axis that the value should be displayed at.
    pub t: f64,
    /// the value that should be displayed.
    pub value: f64,
}

impl Tick {
    pub fn new(t: f64, value: f64) -> Self {
        Self { t, value }
    }
}

pub struct TickerIter {
    inner: Ticker,
    next_tick: usize,
}

impl Iterator for TickerIter {
    type Item = Tick;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.target_num_points {
            0 => None,
            1 => match self.next_tick {
                0 => {
                    self.next_tick += 1;
                    Some(Tick::new(0., self.inner.data_range.min()))
                }
                _ => None,
            },
            2 => match self.next_tick {
                0 => {
                    self.next_tick += 1;
                    Some(Tick::new(0., self.inner.data_range.min()))
                }
                1 => {
                    self.next_tick += 1;
                    Some(Tick::new(1., self.inner.data_range.max()))
                }
                _ => None,
            },
            n => {
                let value = self.inner.first_tick() + (self.next_tick as f64) * self.inner.spacing;
                let (min, max) = self.inner.data_range.into();
                let t = (value - min) / (max - min);
                if t <= 1. {
                    self.next_tick += 1;
                    Some(Tick::new(t, value))
                } else {
                    None
                }
            }
        }
    }
}

/// Returns gap between each scale tick, in terms of the y variable, that gives closest to the
/// requested `target_count` and is either 1, 2 or 5 ×10<sup>n</sup> for some n (hardcoded for now).
///
/// `max_value` is the maximum value that will be graphed, and `target_count` is the maximum number
/// of increments of the y axis scale we want.
pub fn calc_tick_spacing(range: Range, target_count: f64) -> f64 {
    if target_count < 1. {
        // We don't support a number of ticks less than 1.
        return f64::NAN;
    }
    let too_many_10s = pow_10_just_too_many(range, target_count);
    // try 2 * our power of 10 that gives too many
    if count_ticks(range, 2. * too_many_10s) <= target_count {
        return 2. * too_many_10s;
    }
    // next try 5 * our power of 10 that gives too many
    if count_ticks(range, 5. * too_many_10s) <= target_count {
        return 5. * too_many_10s;
    }
    // then it must be the next power of 10
    too_many_10s * 10.
}

/// Find a value of type 10<sup>x</sup> where x is an integer, such that ticks at that distance
/// would result in too many ticks, but ticks at 10<sup>x+1</sup> would give too few (or just
/// right). Returns spacing of ticks
fn pow_10_just_too_many(range: Range, num_ticks: f64) -> f64 {
    // -1 for fence/fence post
    let num_ticks = num_ticks - 1.;
    let ideal_spacing = range.size() / num_ticks;
    let spacing = (10.0f64).powf(ideal_spacing.log10().floor());
    // The actual value where the first tick will go (we need to work out if we lose too much space
    // at the ends and we end up being too few instead of too many)
    let first_tick = calc_next_tick(range.min(), spacing);
    // If when taking account of the above we still have too many ticks
    if first_tick + num_ticks * (spacing + 1.) < calc_prev_tick(range.max(), spacing) {
        // then just return
        spacing
    } else {
        // else go to the next smaller power of 10
        spacing * 0.1
    }
}

/// Get the location of the first tick of the given spacing after the value.
#[inline]
pub fn calc_next_tick(v: f64, spacing: f64) -> f64 {
    // `prev tick <-> v`
    let v_tick_diff = v.rem_euclid(spacing);
    if v_tick_diff == 0. {
        v
    } else {
        v - v_tick_diff + spacing
    }
}

/// Get the location of the first tick of the given spacing before the value.
#[inline]
pub fn calc_prev_tick(v: f64, spacing: f64) -> f64 {
    // `prev tick <-> v`
    let v_tick_diff = v.rem_euclid(spacing);
    v - v_tick_diff
}

/// Count the number of ticks between min and max using the given step
#[inline]
fn count_ticks(range: Range, tick_step: f64) -> f64 {
    let start = calc_next_tick(range.min(), tick_step);
    let end = calc_prev_tick(range.max(), tick_step);
    ((end - start) / tick_step + 1.).floor() // fence/fencepost
}

#[test]
fn text_pow_10_just_too_many() {
    for (min, max, num_ticks) in vec![
        (0., 100., 10.),
        (-9., 109., 10.),
        (-9., 99., 10.),
        (1., 10., 1.),
    ] {
        let range = Range::new(min, max);
        let step = pow_10_just_too_many(range, num_ticks);
        debug_assert!(
            count_ticks(range, step) > num_ticks,
            "count_ticks({:?}, {}) > {}",
            range,
            step,
            num_ticks
        );
        debug_assert!(
            count_ticks(range, step * 10.) <= num_ticks,
            "count_ticks({??}, {}) <= {}",
            range,
            step * 10.,
            num_ticks
        );
    }
}

/// Returns (min, max) of the vector.
///
/// NaNs are propogated.
pub fn data_as_range(mut data: impl Iterator<Item = f64>) -> Range {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for v in data {
        if v.is_nan() {
            return (f64::NAN, f64::NAN).into();
        }
        if v < min {
            min = v;
        }
        if v > max {
            max = v;
        }
    }
    (min, max).into()
}

/// Formats a tick value for a label
fn for_label(v: f64) -> ArcStr {
    let vabs = v.abs();
    if vabs < 1000. && vabs > 0.0001 || vabs == 0. {
        format!("{:.2}", v)
    } else {
        format!("{:.2e}", v)
    }
    .into()
}
