use druid::{
    kurbo::{Affine, CircleSegment, Line, Rect},
    piet::{FontBuilder, PietTextLayout, Text, TextLayout, TextLayoutBuilder},
    BoxConstraints, Color, Data, Env, Event, EventCtx, Insets, LayoutCtx, LifeCycle, LifeCycleCtx,
    PaintCtx, RenderContext, Size, UpdateCtx, Widget,
};
use im::Vector;
use std::f64::consts::PI;

use crate::{axis_heuristic, GRAPH_INSETS};

/// A histogram of equal width categories
#[derive(Debug, Clone, Data)]
pub struct HistogramData {
    pub title: String,
    pub x_axis_label: String,
    #[data(same_fn = "Vector::ptr_eq")]
    pub x_axis: Vector<String>,
    #[data(same_fn = "Vector::ptr_eq")]
    pub counts: Vector<usize>,
}

pub struct Histogram {
    bar_spacing: f64,
}

impl Histogram {
    pub fn new() -> Self {
        Histogram { bar_spacing: 10.0 }
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
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &HistogramData,
        data: &HistogramData,
        env: &Env,
    ) {
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &HistogramData,
        env: &Env,
    ) -> Size {
        bc.constrain((f64::INFINITY, f64::INFINITY))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &HistogramData, env: &Env) {
        let bg_brush = ctx.solid_brush(Color::hlc(0.0, 90.0, 0.0));
        let axes_brush = ctx.solid_brush(Color::hlc(0.0, 60.0, 0.0));
        let text_brush = ctx.solid_brush(Color::BLACK);
        let bar_brush = ctx.solid_brush(Color::hlc(0.0, 50.0, 50.0));
        let size = ctx.size();
        let bounds = size.to_rect();
        let graph_bounds = bounds.inset(GRAPH_INSETS);
        let max_data = *data.counts.iter().max().unwrap() as f64;

        // TODO: caching of both the format and the layout
        let font = ctx
            .text()
            .new_font_by_name("DejaVuSans", 20.0)
            .build()
            .unwrap();
        let font_sm = ctx
            .text()
            .new_font_by_name("DejaVuSans", 12.0)
            .build()
            .unwrap();

        // background & title
        ctx.fill(bounds, &bg_brush);
        let title_layout = ctx
            .text()
            .new_text_layout(&font, &data.title)
            .build()
            .unwrap();
        let title_width = title_layout.width();
        ctx.draw_text(
            &title_layout,
            ((size.width - title_width) * 0.5, 40.0),
            &text_brush,
        );

        // x axis
        let x_axis = Line::new(
            (graph_bounds.x0 - 1.0, graph_bounds.y1),
            (graph_bounds.x1, graph_bounds.y1),
        );
        ctx.stroke(x_axis, &axes_brush, 2.0);
        let x_label_layout = ctx
            .text()
            .new_text_layout(&font, &data.x_axis_label)
            .build()
            .unwrap();
        let x_label_width = x_label_layout.width();
        ctx.draw_text(
            &x_label_layout,
            ((size.width - x_label_width) * 0.5, size.height - 20.0),
            &text_brush,
        );

        // y axis
        {
            let y_axis = Line::new(
                (graph_bounds.x0, graph_bounds.y0),
                (graph_bounds.x0, graph_bounds.y1 + 1.0),
            );
            ctx.stroke(y_axis, &axes_brush, 2.0);
            let label_gap =
                axis_heuristic(max_data, (graph_bounds.height() / 40.0).floor() as usize);
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

        // data
        let data_len = data.counts.len() as f64;
        let (width, height) = (graph_bounds.width(), graph_bounds.height());
        let total_space = (data_len + 1.0) * self.bar_spacing;
        // give up if the area is too small.
        if total_space >= width {
            return;
        }
        let total_bar_width = width - total_space;
        let bar_width = total_bar_width / data_len;
        assert_eq!(
            bar_width * data_len + self.bar_spacing * (data_len + 1.0),
            width
        );
        ctx.with_save(|ctx| {
            ctx.transform(Affine::translate((
                graph_bounds.x0 + self.bar_spacing,
                graph_bounds.y0,
            )));
            for (idx, (count, label)) in data
                .counts
                .iter()
                .copied()
                .zip(data.x_axis.iter())
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
                let label_layout = ctx.text().new_text_layout(&font_sm, label).build().unwrap();
                let label_width = label_layout.width();
                ctx.draw_text(
                    &label_layout,
                    (mid_x - label_width * 0.5, height + 20.0),
                    &text_brush,
                );
            }
        });
    }
}
