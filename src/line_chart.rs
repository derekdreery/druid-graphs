use druid::{
    im::Vector,
    kurbo::{Affine, Line, Point, Rect},
    ArcStr, BoxConstraints, Color, Data, Env, Event, EventCtx, KeyOrValue, LayoutCtx, Lens,
    LifeCycle, LifeCycleCtx, PaintCtx, RenderContext, Size, TextLayout, UpdateCtx, Widget,
};
use druid_lens_compose::ComposeLens;
use itertools::{izip, Itertools};
use std::{iter, sync::Arc};

use crate::{
    axes::{calc_tick_spacing, Scale},
    theme, Range, GRAPH_INSETS,
};

/// A histogram of equal width categories
#[derive(Debug, Clone, Data, ComposeLens)]
pub struct LineChartData {
    pub title: ArcStr,
    pub x_axis_label: Option<ArcStr>,
    pub y_data: Vector<f64>,
    /// If `None`, then the scale `0..y_data.len()` will be used.
    pub x_data: Option<Vector<f64>>,
}

pub struct LineChart {
    // retained state
    title_layout: TextLayout<ArcStr>,
    x_label_layout: TextLayout<ArcStr>,
    /// If this is set, it is used instead of the values range for setting the axes.
    fixed_x: Option<Range>,
    fixed_y: Option<Range>,
    label_margin: KeyOrValue<f64>,
    // retained
    // we keep axes separate as we have to do less invalidation that way.
    data_range_x: Option<Range>,
    data_range_y: Option<Range>,
    x_scale: Option<Scale>,
    y_scale: Option<Scale>,
}

impl LineChart {
    pub fn new() -> Self {
        let mut title_layout = TextLayout::new();
        title_layout.set_text_size(20.);
        LineChart {
            title_layout,
            x_label_layout: TextLayout::new(),
            fixed_x: None,
            fixed_y: None,
            label_margin: theme::MARGIN.into(),
            data_range_x: None,
            data_range_y: None,
            x_scale: None,
            y_scale: None,
        }
    }

    pub fn set_fixed_x(&mut self, fixed_x: Option<Range>) {
        if self.fixed_x != fixed_x {
            self.fixed_x = fixed_x;
            self.data_range_x = None;
            self.x_scale = None;
        }
    }

    pub fn set_fixed_y(&mut self, fixed_y: Option<Range>) {
        if self.fixed_y != fixed_y {
            self.fixed_y = fixed_y;
            self.data_range_y = None;
            self.y_scale = None;
        }
    }

    fn calc_x_data_range(&mut self, data: &LineChartData) {
        self.data_range_x = Some(Range::from_iter(resolve_x_data(
            data.x_data.as_ref(),
            data.y_data.len(),
        )));
        self.x_scale = None;
    }

    fn calc_y_data_range(&mut self, data: &LineChartData) {
        self.data_range_y = Some(Range::from_iter(data.y_data.iter().copied()));
        self.y_scale = None;
    }

    fn x_range(&self) -> Option<Range> {
        self.fixed_x.or(self.data_range_x)
    }

    fn y_range(&self) -> Option<Range> {
        self.fixed_y.or(self.data_range_y)
    }

    fn rebuild_if_needed(&mut self, ctx: &mut PaintCtx, env: &Env) {
        self.title_layout.rebuild_if_needed(ctx.text(), env);
        self.x_label_layout.rebuild_if_needed(ctx.text(), env);
        if self.x_scale.is_none() {
            self.x_scale = Some(Scale::new_x(self.x_range().unwrap()));
        }
        if self.y_scale.is_none() {
            self.y_scale = Some(Scale::new_y(self.y_range().unwrap()));
        }
        let graph_bounds = self.graph_bounds(ctx.size());
        let x_scale = self.x_scale.as_mut().unwrap();
        x_scale.set_graph_bounds(graph_bounds);
        x_scale.rebuild_if_needed(ctx, env);
        let y_scale = self.y_scale.as_mut().unwrap();
        y_scale.set_graph_bounds(graph_bounds);
        y_scale.rebuild_if_needed(ctx, env);
    }

    fn graph_bounds(&self, size: Size) -> Rect {
        Rect::from_origin_size(Point::ZERO, size).inset(GRAPH_INSETS)
    }
}

impl Widget<LineChartData> for LineChart {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut LineChartData, env: &Env) {}

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &LineChartData,
        env: &Env,
    ) {
        match event {
            LifeCycle::WidgetAdded => {
                self.title_layout.set_text(data.title.clone());
                if let Some(label) = data.x_axis_label.as_ref() {
                    self.x_label_layout.set_text(label.clone());
                }
                if self.fixed_x.is_none() {
                    self.calc_x_data_range(data);
                }
                if self.fixed_y.is_none() {
                    self.calc_y_data_range(data);
                }
            }
            _ => (),
        }
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &LineChartData,
        data: &LineChartData,
        env: &Env,
    ) {
        if !old_data.title.same(&data.title) {
            self.title_layout.set_text(data.title.clone());
        }
        self.title_layout.needs_rebuild_after_update(ctx);
        if !old_data.x_axis_label.same(&data.x_axis_label) {
            if let Some(label) = data.x_axis_label.as_ref() {
                self.x_label_layout.set_text(label.clone());
            }
        }
        self.x_label_layout.needs_rebuild_after_update(ctx);

        if (!Data::same(&old_data.y_data, &data.y_data) || self.data_range_y.is_none())
            && self.fixed_y.is_none()
        {
            self.calc_y_data_range(data);
        }
        if (!Data::same(&old_data.x_data, &data.x_data) || self.data_range_x.is_none())
            && self.fixed_x.is_none()
        {
            self.calc_x_data_range(data);
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &LineChartData,
        env: &Env,
    ) -> Size {
        bc.max() // or costrain to some size.
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &LineChartData, env: &Env) {
        self.rebuild_if_needed(ctx, env);
        let line_brush = ctx.solid_brush(Color::hlc(0.0, 50.0, 50.0));
        let size = ctx.size();
        let bounds = size.to_rect();
        let graph_bounds = bounds.inset(GRAPH_INSETS);

        // data
        for ((x0, x1), (y0, y1)) in izip!(
            resolve_x_data(data.x_data.as_ref(), data.y_data.len()).tuple_windows(),
            data.y_data.iter().tuple_windows()
        ) {
            let x_scale = self.x_scale.as_ref().unwrap();
            let y_scale = self.y_scale.as_ref().unwrap();
            let x0 = x_scale.pixel_location(x0);
            let x1 = x_scale.pixel_location(x1);
            let y0 = y_scale.pixel_location(*y0);
            let y1 = y_scale.pixel_location(*y1);
            ctx.stroke(Line::new((x0, y0), (x1, y1)), &line_brush, 1.);
        }

        // title
        let title_width = self.title_layout.size().width;
        self.title_layout
            .draw(ctx, ((size.width - title_width) * 0.5, 10.0));

        // x axis
        self.x_scale.as_mut().unwrap().draw(ctx, env);
        if data.x_axis_label.is_some() {
            let x_label_width = self.x_label_layout.size().width;
            self.x_label_layout.draw(
                ctx,
                ((size.width - x_label_width) * 0.5, size.height - 40.0),
            );
        }

        // y axis
        self.y_scale.as_mut().unwrap().draw(ctx, env);
    }
}

/// return either the data or a range
fn resolve_x_data<'a>(data: Option<&'a Vector<f64>>, len: usize) -> impl Iterator<Item = f64> + 'a {
    let len = len as f64;
    match data {
        Some(data) => Either::Left(data.iter().copied()),
        None => Either::Right(iter::successors(Some(0.0f64), move |n| {
            if *n <= len {
                Some(n + 1.)
            } else {
                None
            }
        })),
    }
}

enum Either<T, U> {
    Left(T),
    Right(U),
}

impl<T: Copy, I1: Iterator<Item = T>, I2: Iterator<Item = T>> Iterator for Either<I1, I2> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Either::Left(i1) => i1.next(),
            Either::Right(i2) => i2.next(),
        }
    }
}
