use druid::{
    im::Vector,
    kurbo::{Affine, Line, Point, Rect},
    text::TextStorage,
    ArcStr, BoxConstraints, Color, Data, Env, Event, EventCtx, Insets, KeyOrValue, LayoutCtx, Lens,
    LifeCycle, LifeCycleCtx, PaintCtx, RenderContext, Size, TextLayout, UpdateCtx, Widget,
};
use druid_lens_compose::ComposeLens;
use itertools::{izip, Itertools};
use std::{iter, sync::Arc};

use crate::{
    axes::{calc_tick_spacing, Scale},
    theme, Range,
};

/// A histogram of equal width categories
#[derive(Debug, Clone, Data, ComposeLens)]
pub struct LineChartData<Title, XLabel> {
    pub title: Title,
    // x axis
    pub x_axis_label: XLabel,
    /// If `None`, then the scale `0..y_data.len()` will be used.
    pub x_range: Option<Range>,
    pub draw_x_tick_labels: bool,
    pub draw_x_axis: bool,
    pub x_data: Option<Vector<f64>>,
    // y axis
    pub y_range: Option<Range>,
    pub draw_y_tick_labels: bool,
    pub draw_y_axis: bool,
    pub y_data: Vector<f64>,
}

pub struct LineChart<Title, XLabel> {
    // retained state
    title_layout: TextLayout<Title>,
    x_label_layout: TextLayout<XLabel>,
    // we keep axes separate as we have to do less invalidation that way.
    // x axis
    /// We only need to calculate this if we aren't using a fixed range.
    data_range_x: Option<Range>,
    x_scale: Option<Scale>,
    // y axis
    data_range_y: Option<Range>,
    y_scale: Option<Scale>,
}

impl<Title, XLabel> LineChart<Title, XLabel>
where
    Title: TextStorage,
    XLabel: TextStorage,
{
    pub fn new() -> Self {
        let mut title_layout = TextLayout::new();
        title_layout.set_text_size(20.);
        LineChart {
            title_layout,
            x_label_layout: TextLayout::new(),
            data_range_x: None,
            data_range_y: None,
            x_scale: None,
            y_scale: None,
        }
    }

    fn calc_x_data_range(&mut self, data: &LineChartData<Title, XLabel>) {
        self.data_range_x = Some(Range::from_iter(resolve_x_data(
            data.x_data.as_ref(),
            data.y_data.len(),
        )));
        self.x_scale = None;
    }

    fn calc_y_data_range(&mut self, data: &LineChartData<Title, XLabel>) {
        self.data_range_y = Some(Range::from_iter(data.y_data.iter().copied()));
        self.y_scale = None;
    }

    fn x_range(&self, data: &LineChartData<Title, XLabel>) -> Option<Range> {
        data.x_range.or(self.data_range_x)
    }

    fn y_range(&self, data: &LineChartData<Title, XLabel>) -> Option<Range> {
        data.y_range.or(self.data_range_y)
    }

    fn rebuild_if_needed(
        &mut self,
        ctx: &mut PaintCtx,
        data: &LineChartData<Title, XLabel>,
        env: &Env,
    ) {
        let margin = env.get(theme::MARGIN);
        let scale_margin = env.get(theme::SCALE_MARGIN);

        self.title_layout.rebuild_if_needed(ctx.text(), env);
        self.x_label_layout.rebuild_if_needed(ctx.text(), env);
        if self.x_scale.is_none() {
            self.x_scale = Some(Scale::new_x(self.x_range(data).unwrap()));
        }
        if self.y_scale.is_none() {
            self.y_scale = Some(Scale::new_y(self.y_range(data).unwrap()));
        }

        // build twice because we want to check the size
        // Firstly try laying out with no size restriction
        //
        // There is a bit of a dance here because the borrow checker won't let us borrow both parts
        // of the struct at the same time.
        let draw_area = ctx.size().to_rect();
        let x_scale = self.x_scale.as_mut().unwrap();
        x_scale.set_graph_bounds(draw_area);
        x_scale.rebuild_if_needed(ctx, env);
        let y_scale = self.y_scale.as_mut().unwrap();
        y_scale.set_graph_bounds(draw_area);
        y_scale.rebuild_if_needed(ctx, env);

        // space for the y axis and tick labels
        let x0 = margin + self.y_scale.as_ref().unwrap().max_layout().width + scale_margin;
        // space for the chart title (if needed)
        let mut y0 = if data.title.as_str().is_empty() {
            margin
        } else {
            2. * margin + self.title_layout.size().height
        };
        // space for the x axis and tick labels
        let mut y1 = margin + self.x_scale.as_ref().unwrap().max_layout().height + scale_margin;
        // add space for the x axis label (if it's there)
        if !data.x_axis_label.as_str().is_empty() {
            y1 += margin + self.x_label_layout.size().height;
        }

        let graph_insets = Insets {
            x0: -x0,
            y0: -y0,
            x1: -margin,
            y1: -y1,
        };
        let graph_bounds = draw_area.inset(graph_insets);

        // now build again using the info we calculated.
        let x_scale = self.x_scale.as_mut().unwrap();
        x_scale.set_graph_bounds(graph_bounds);
        x_scale.rebuild_if_needed(ctx, env);
        let y_scale = self.y_scale.as_mut().unwrap();
        y_scale.set_graph_bounds(graph_bounds);
        y_scale.rebuild_if_needed(ctx, env);
    }
}

impl<Title, XLabel> Widget<LineChartData<Title, XLabel>> for LineChart<Title, XLabel>
where
    Title: TextStorage,
    XLabel: TextStorage,
{
    fn event(
        &mut self,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut LineChartData<Title, XLabel>,
        env: &Env,
    ) {
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &LineChartData<Title, XLabel>,
        env: &Env,
    ) {
        match event {
            LifeCycle::WidgetAdded => {
                self.title_layout.set_text(data.title.clone());
                self.x_label_layout.set_text(data.x_axis_label.clone());
                if data.x_range.is_none() {
                    self.calc_x_data_range(data);
                }
                if data.y_range.is_none() {
                    self.calc_y_data_range(data);
                }
            }
            _ => (),
        }
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &LineChartData<Title, XLabel>,
        data: &LineChartData<Title, XLabel>,
        env: &Env,
    ) {
        // the job of this method is to invalidate parts of the retained state that are no longer
        // valid, and to request a repaint/relayout if necessary.
        if !old_data.title.same(&data.title) {
            self.title_layout.set_text(data.title.clone());
        }
        self.title_layout.needs_rebuild_after_update(ctx);

        // x axis
        if !old_data.x_axis_label.same(&data.x_axis_label) {
            self.x_label_layout.set_text(data.x_axis_label.clone());
        }
        self.x_label_layout.needs_rebuild_after_update(ctx);
        if data.draw_x_tick_labels != old_data.draw_x_tick_labels {
            ctx.request_layout();
        }
        if data.draw_x_axis != old_data.draw_x_axis {
            // don't need to re-layout in this case.
            ctx.request_paint();
        }
        if (!Data::same(&old_data.x_data, &data.x_data) || self.data_range_x.is_none())
            && data.x_range.is_none()
        {
            self.calc_x_data_range(data);
            ctx.request_layout();
        }
        if !Data::same(&old_data.x_data, &data.x_data) {
            ctx.request_layout();
        }

        // y axis
        if (!Data::same(&old_data.y_data, &data.y_data) || self.data_range_y.is_none())
            && data.y_range.is_none()
        {
            self.calc_y_data_range(data);
            ctx.request_layout();
        }
        if data.draw_y_tick_labels != old_data.draw_y_tick_labels {
            ctx.request_layout();
        }
        if data.draw_y_axis != old_data.draw_y_axis {
            // don't need to re-layout in this case.
            ctx.request_paint();
        }
        if !Data::same(&old_data.y_data, &data.y_data) {
            ctx.request_layout();
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &LineChartData<Title, XLabel>,
        env: &Env,
    ) -> Size {
        bc.max() // or costrain to some size.
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &LineChartData<Title, XLabel>, env: &Env) {
        self.rebuild_if_needed(ctx, data, env);
        let line_brush = ctx.solid_brush(Color::hlc(0.0, 50.0, 50.0));
        let size = ctx.size();
        let bounds = size.to_rect();
        let margin = env.get(theme::MARGIN);

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
        self.x_scale
            .as_mut()
            .unwrap()
            .draw(ctx, env, data.draw_x_axis, data.draw_x_tick_labels);
        if !data.x_axis_label.as_str().is_empty() {
            let label_size = self.x_label_layout.size();
            self.x_label_layout.draw(
                ctx,
                (
                    (size.width - label_size.width) * 0.5,
                    size.height - label_size.height - margin,
                ),
            );
        }

        // y axis
        self.y_scale
            .as_mut()
            .unwrap()
            .draw(ctx, env, data.draw_y_axis, data.draw_y_tick_labels);
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
