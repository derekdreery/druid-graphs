use druid::{
    im::Vector,
    kurbo::{Affine, CircleSegment, Line, Rect},
    piet::{PietTextLayout, Text, TextLayoutBuilder},
    theme::LABEL_COLOR,
    ArcStr, BoxConstraints, Color, Data, Env, Event, EventCtx, Insets, KeyOrValue, LayoutCtx,
    LifeCycle, LifeCycleCtx, PaintCtx, RenderContext, Size, TextLayout, UpdateCtx, Widget,
};
use druid_lens_compose::ComposeLens;
use itertools::izip;
use std::{cmp::Ordering, f64::consts::PI, iter};

use crate::{new_color, square, theme};

#[derive(Debug, Clone, Data, ComposeLens)]
pub struct PieChartData {
    pub title: ArcStr,
    pub category_labels: Vector<ArcStr>,
    pub counts: Vector<usize>,
}

#[derive(Clone)]
pub struct PieChart {
    title_layout: TextLayout<ArcStr>,
    key_title_layout: TextLayout<ArcStr>,
    category_layouts: Vec<TextLayout<ArcStr>>,
    // theme stuff
    key_stroke_color: KeyOrValue<Color>,
    key_margin: KeyOrValue<f64>,
}

impl PieChart {
    pub fn new() -> Self {
        let mut key_title_layout = TextLayout::from_text("Key");
        key_title_layout.set_text_size(20.);
        let mut title_layout = TextLayout::new();
        title_layout.set_text_size(20.);
        PieChart {
            title_layout,
            key_title_layout,
            category_layouts: vec![],
            key_stroke_color: LABEL_COLOR.into(),
            key_margin: theme::MARGIN.into(),
        }
    }

    pub fn rebuild_if_needed(&mut self, ctx: &mut PaintCtx, env: &Env) {
        self.title_layout.rebuild_if_needed(ctx.text(), env);
        self.key_title_layout.rebuild_if_needed(ctx.text(), env);
        for layout in self.category_layouts.iter_mut() {
            layout.rebuild_if_needed(ctx.text(), env);
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
        match event {
            LifeCycle::WidgetAdded => {
                self.title_layout.set_text(data.title.clone());
                self.category_layouts = data
                    .category_labels
                    .iter()
                    .cloned()
                    .map(|text| TextLayout::from_text(text))
                    .collect()
            }
            _ => (),
        }
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &PieChartData,
        data: &PieChartData,
        env: &Env,
    ) {
        if !Data::same(&old_data.title, &data.title) {
            self.title_layout.set_text(data.title.clone());
        }
        self.title_layout.needs_rebuild_after_update(ctx);
        self.key_title_layout.needs_rebuild_after_update(ctx);
        if !Data::same(&old_data.category_labels, &data.category_labels) {
            // If we don't have enough labels add some on the end.
            //
            // Note that we might have too many. That is why we only take the first
            // `category_labels.len()` items during paint.
            if self.category_layouts.len() < data.category_labels.len() {
                self.category_layouts.extend(
                    iter::repeat(TextLayout::new())
                        .take(data.category_labels.len() - self.category_layouts.len()),
                );
            }
            for (label_text, layout) in izip!(&data.category_labels, &mut self.category_layouts) {
                layout.set_text(label_text.clone());
            }
        }
        for layout in self.category_layouts.iter_mut() {
            layout.needs_rebuild_after_update(ctx);
        }
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
        self.rebuild_if_needed(ctx, env);
        let bg_brush = ctx.solid_brush(Color::hlc(0.0, 90.0, 0.0));
        let axes_brush = ctx.solid_brush(Color::hlc(0.0, 60.0, 0.0));
        let text_brush = ctx.solid_brush(self.key_stroke_color.resolve(env));
        let bar_brush = ctx.solid_brush(Color::hlc(0.0, 50.0, 50.0));
        let size = ctx.size();
        let bounds = size.to_rect();
        let total: usize = data.counts.iter().copied().sum();
        let categories_count = data.category_labels.len();

        // background & title
        let title_width = self.title_layout.size().width;
        self.title_layout
            .draw(ctx, ((size.width - title_width) * 0.5, 40.0));

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
        const COLOR_SIZE: f64 = 12.0;
        // last 40% of the width
        let key_bounds = bounds.inset((-bounds.width() * 0.6, 0.0, 0.0, 0.0));
        // Calculate some stuff about label text layout:
        let key_margin = self.key_margin.resolve(env);
        let mut max_color_label_width = 0.;
        let mut total_label_height = 0.;
        for layout in self.category_layouts.iter().take(categories_count) {
            let size = layout.size();
            let new_width = size.width + size.height + 3. * key_margin; // m color m label m
            if new_width > max_color_label_width {
                max_color_label_width = new_width;
            }
            total_label_height += size.height;
        }
        let height = total_label_height
            + self.key_title_layout.size().height
            + (categories_count as f64 + 2.0) * key_margin;

        let key_width =
            (self.key_title_layout.size().width + 2.0 * key_margin).max(max_color_label_width);

        let key_bounds = Rect::from_center_size(key_bounds.center(), (key_width, height));
        ctx.stroke(key_bounds, &text_brush, 2.0);
        self.key_title_layout.draw(
            ctx,
            (
                key_bounds.x0 + (key_bounds.width() - self.key_title_layout.size().width) * 0.5,
                key_bounds.y0 + key_margin,
            ),
        );
        let mut next_loc = key_bounds.y0 + key_margin * 2. + self.key_title_layout.size().height;
        // important: only take the right amount of layouts here.
        for (idx, layout) in self
            .category_layouts
            .iter()
            .take(categories_count)
            .enumerate()
        {
            let height = layout.size().height;
            let color_rect = Rect::new(
                key_bounds.x0 + key_margin,
                next_loc,
                // use the text's height to make the color square match the text.
                key_bounds.x0 + key_margin + height,
                next_loc + height,
            );
            ctx.fill(color_rect, &new_color(idx));
            ctx.stroke(color_rect, &text_brush, 1.0);
            layout.draw(
                ctx,
                (
                    key_bounds.x0 + key_margin + height + key_margin, // m color m label
                    next_loc,
                ),
            );
            next_loc += key_margin + height;
        }
    }
}
