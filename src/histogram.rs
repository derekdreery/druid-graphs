use druid::{
    im::Vector,
    kurbo::{Affine, Line, Point, Rect},
    ArcStr, BoxConstraints, Color, Data, Env, Event, EventCtx, KeyOrValue, LayoutCtx, LifeCycle,
    LifeCycleCtx, PaintCtx, RenderContext, Size, TextLayout, UpdateCtx, Widget,
};
use druid_lens_compose::ComposeLens;
use itertools::izip;
use std::sync::Arc;

use crate::{
    axes::{calc_tick_spacing, Scale},
    theme, GRAPH_INSETS,
};

/// A histogram of equal width categories
#[derive(Debug, Clone, Data, ComposeLens)]
pub struct HistogramData {
    pub title: ArcStr,
    pub x_axis_label: ArcStr,
    pub x_axis: Vector<ArcStr>,
    pub counts: Vector<usize>,
}

pub struct Histogram {
    bar_spacing: KeyOrValue<f64>,
    axis_color: KeyOrValue<Color>,
    // retained state
    title_layout: TextLayout<ArcStr>,
    x_label_layout: TextLayout<ArcStr>,
    x_axis_layouts: Option<Vec<TextLayout<ArcStr>>>,
    y_scale: Option<Scale>,
}

impl Histogram {
    pub fn new() -> Self {
        let mut title_layout = TextLayout::new();
        title_layout.set_text_size(20.);
        Histogram {
            bar_spacing: theme::BAR_SPACING.into(),
            axis_color: theme::AXES_COLOR.into(),
            title_layout,
            x_label_layout: TextLayout::new(),
            x_axis_layouts: None,
            y_scale: None,
        }
    }

    fn rebuild_if_needed(&mut self, ctx: &mut PaintCtx, data: &HistogramData, env: &Env) {
        self.title_layout.rebuild_if_needed(ctx.text(), env);
        self.x_label_layout.rebuild_if_needed(ctx.text(), env);
        if self.x_axis_layouts.is_none() {
            self.x_axis_layouts = Some(
                data.x_axis
                    .iter()
                    .cloned()
                    .map(|label| {
                        let mut layout = TextLayout::from_text(label);
                        layout.rebuild_if_needed(ctx.text(), env);
                        layout
                    })
                    .collect(),
            );
        }
        if self.y_scale.is_none() {
            self.y_scale = Some(Scale::new_y((
                0.,
                data.counts.iter().copied().max().unwrap_or(0) as f64,
            )))
        }
        let graph_bounds = self.graph_bounds(ctx.size());
        let y_scale = self.y_scale.as_mut().unwrap();
        y_scale.set_graph_bounds(graph_bounds);
        y_scale.rebuild_if_needed(ctx, env);
    }

    fn graph_bounds(&self, size: Size) -> Rect {
        Rect::from_origin_size(Point::ZERO, size).inset(GRAPH_INSETS)
    }
}

impl Widget<HistogramData> for Histogram {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut HistogramData, env: &Env) {}

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &HistogramData,
        env: &Env,
    ) {
        match event {
            LifeCycle::WidgetAdded => {
                self.title_layout.set_text(data.title.clone());
                self.x_label_layout.set_text(data.x_axis_label.clone());
                // TODO reuse x axis tick label layouts
            }
            _ => (),
        }
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &HistogramData,
        data: &HistogramData,
        env: &Env,
    ) {
        if !old_data.title.same(&data.title) {
            self.title_layout.set_text(data.title.clone());
        }
        if !old_data.x_axis_label.same(&data.x_axis_label) {
            self.x_label_layout.set_text(data.x_axis_label.clone());
        }
        if !old_data.x_axis.same(&data.x_axis) {
            self.x_axis_layouts = None;
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &HistogramData,
        env: &Env,
    ) -> Size {
        bc.max() // or costrain to some size.
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &HistogramData, env: &Env) {
        self.rebuild_if_needed(ctx, data, env);
        let bg_brush = ctx.solid_brush(Color::hlc(0.0, 90.0, 0.0));
        let axes_brush = ctx.solid_brush(self.axis_color.resolve(env));
        let bar_brush = ctx.solid_brush(Color::hlc(0.0, 50.0, 50.0));
        let size = ctx.size();
        let bounds = size.to_rect();
        let graph_bounds = bounds.inset(GRAPH_INSETS);
        let max_data = *data.counts.iter().max().unwrap() as f64;
        let bar_spacing = self.bar_spacing.resolve(env);

        // data
        let data_len = data.counts.len() as f64;
        let (width, height) = (graph_bounds.width(), graph_bounds.height());
        let total_space = (data_len + 1.0) * bar_spacing;
        // give up if the area is too small.
        if total_space >= width {
            return;
        }
        let total_bar_width = width - total_space;
        let bar_width = total_bar_width / data_len;
        assert_eq!(bar_width * data_len + bar_spacing * (data_len + 1.0), width);
        ctx.with_save(|ctx| {
            ctx.transform(Affine::translate((
                graph_bounds.x0 + bar_spacing,
                graph_bounds.y0,
            )));
            for (idx, (count, label, label_layout)) in izip!(
                data.counts.iter().copied(),
                data.x_axis.iter().cloned(),
                self.x_axis_layouts.as_ref().unwrap()
            )
            .enumerate()
            {
                let idx = idx as f64;
                let start_x = width * idx / data_len;
                let end_x = start_x + bar_width;
                let mid_x = start_x + (end_x - start_x) * 0.5;

                // bar
                let end_y = (count as f64) * height / max_data;
                ctx.fill(
                    Rect::new(start_x, height - end_y, end_x, height),
                    &bar_brush,
                );

                // data label
                let label_width = label_layout.size().width;
                label_layout.draw(ctx, (mid_x - label_width * 0.5, height + 2.));
            }
        });

        // title
        let title_width = self.title_layout.size().width;
        self.title_layout
            .draw(ctx, ((size.width - title_width) * 0.5, 10.0));

        // x axis
        let x_axis = Line::new(
            (graph_bounds.x0 - 1.0, graph_bounds.y1),
            (graph_bounds.x1, graph_bounds.y1),
        );
        ctx.stroke(x_axis, &axes_brush, 2.0);
        let x_label_width = self.x_label_layout.size().width;
        self.x_label_layout.draw(
            ctx,
            ((size.width - x_label_width) * 0.5, size.height - 40.0),
        );

        // y axis
        self.y_scale.as_mut().unwrap().draw(ctx, env, true, true);
    }
}
