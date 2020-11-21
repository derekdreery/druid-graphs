use druid::{
    im::Vector,
    kurbo::{Affine, CircleSegment, Line, Rect},
    piet::{PietTextLayout, Text, TextLayout, TextLayoutBuilder},
    BoxConstraints, Color, Data, Env, Event, EventCtx, Insets, LayoutCtx, LifeCycle, LifeCycleCtx,
    PaintCtx, RenderContext, Size, UpdateCtx, Widget,
};
use std::{f64::consts::PI, sync::Arc};

use crate::{new_color, square};

const TITLE_HEIGHT: f64 = 20.0;
const TEXT_HEIGHT: f64 = 12.0;

#[derive(Debug, Clone, Data)]
pub struct PieChartData {
    pub title: Arc<str>,
    pub category_labels: Vector<Arc<str>>,
    pub counts: Vector<usize>,
}

pub struct PieChart {
    key_layouts: Vec<PietTextLayout>,
}

impl PieChart {
    pub fn new() -> Self {
        PieChart {
            key_layouts: vec![],
        }
    }
}

impl Widget<PieChartData> for PieChart {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut PieChartData, env: &Env) {}

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &PieChartData,
        env: &Env,
    ) {
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &PieChartData,
        data: &PieChartData,
        env: &Env,
    ) {
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &PieChartData,
        env: &Env,
    ) -> Size {
        bc.constrain((f64::INFINITY, f64::INFINITY))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &PieChartData, env: &Env) {
        let bg_brush = ctx.solid_brush(Color::hlc(0.0, 90.0, 0.0));
        let axes_brush = ctx.solid_brush(Color::hlc(0.0, 60.0, 0.0));
        let text_brush = ctx.solid_brush(Color::BLACK);
        let bar_brush = ctx.solid_brush(Color::hlc(0.0, 50.0, 50.0));
        let size = ctx.size();
        let bounds = size.to_rect();
        let total: usize = data.counts.iter().copied().sum();

        // TODO: caching of both the format and the layout
        // background & title
        ctx.fill(bounds, &bg_brush);
        let title_layout = ctx
            .text()
            .new_text_layout(data.title.clone())
            .build()
            .unwrap();
        let title_width = title_layout.width();
        ctx.draw_text(&title_layout, ((size.width - title_width) * 0.5, 40.0));

        // Pie
        let pie_area = square(
            bounds
                // the left 60% of the available area
                .inset((0.0, -40.0, -bounds.width() * 0.4, 0.0))
                // with a 10 px margin
                .inset(-10.0),
        );
        let mut start_angle = 0.0;
        for (idx, count) in data.counts.iter().copied().enumerate() {
            let sweep_angle = count as f64 / total as f64 * 2.0 * PI;
            ctx.fill(
                CircleSegment {
                    center: pie_area.center(),
                    outer_radius: pie_area.width() * 0.5,
                    inner_radius: 0.0,
                    start_angle,
                    sweep_angle,
                },
                &new_color(idx),
            );
            start_angle += sweep_angle;
        }

        // Key
        const KEY_MARGIN: f64 = 6.0;
        const COLOR_SIZE: f64 = 12.0;
        // last 40% of the width
        let key_bounds = bounds.inset((-bounds.width() * 0.6, 0.0, 0.0, 0.0));
        let len = data.category_labels.len() as f64;
        let height = len * TEXT_HEIGHT + TITLE_HEIGHT + (len + 3.0) * KEY_MARGIN;
        let title_layout = ctx.text().new_text_layout("Key").build().unwrap();
        self.key_layouts.clear();
        let mut max_label_len: f64 = 0.0;
        for label in data.category_labels.iter().cloned() {
            let layout = ctx.text().new_text_layout(label).build().unwrap();
            max_label_len = max_label_len.max(layout.width());
            self.key_layouts.push(layout);
        }

        let key_width = (title_layout.width() + 2.0 * KEY_MARGIN)
            .max(max_label_len + 3.0 * KEY_MARGIN + COLOR_SIZE);

        let key_bounds = Rect::from_center_size(key_bounds.center(), (key_width, height));
        ctx.stroke(key_bounds, &text_brush, 2.0);
        ctx.draw_text(
            &title_layout,
            (
                key_bounds.x0 + (key_bounds.width() - title_layout.width()) * 0.5,
                key_bounds.y0 + KEY_MARGIN + TITLE_HEIGHT,
            ),
        );
        for (idx, label) in self.key_layouts.iter().enumerate() {
            let top_of_text = key_bounds.y0
                + KEY_MARGIN
                + TITLE_HEIGHT
                + (2.0 * KEY_MARGIN)
                + (TEXT_HEIGHT + KEY_MARGIN) * idx as f64;
            let color_rect = Rect::new(
                key_bounds.x0 + KEY_MARGIN,
                top_of_text,
                // use TEXT_HEIGHT to make the color square match the text.
                key_bounds.x0 + KEY_MARGIN + TEXT_HEIGHT,
                top_of_text + TEXT_HEIGHT,
            );
            ctx.fill(color_rect, &new_color(idx));
            ctx.stroke(color_rect, &text_brush, 1.0);
            ctx.draw_text(
                &label,
                (
                    key_bounds.x0 + KEY_MARGIN + TEXT_HEIGHT + KEY_MARGIN,
                    top_of_text + TEXT_HEIGHT,
                ),
            );
        }
    }
}
