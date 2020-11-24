use druid::{
    im::Vector,
    kurbo::{Line, Rect},
    theme::LABEL_COLOR,
    ArcStr, BoxConstraints, Color, Data, Env, Event, EventCtx, KeyOrValue, LayoutCtx, LifeCycle,
    LifeCycleCtx, PaintCtx, RenderContext, Size, TextLayout, UpdateCtx, Widget,
};
use druid_lens_compose::ComposeLens;

use crate::{
    axes::{data_as_range, Scale},
    GRAPH_INSETS,
};

/// A histogram of equal width categories
#[derive(Debug, Clone, Data, ComposeLens)]
pub struct BoxPlotData {
    pub title: ArcStr,
    pub data_points: Vector<f64>,
}

#[derive(Clone)]
pub struct BoxPlot {
    title_layout: TextLayout<ArcStr>,
    // retained sorted list of data points
    sorted_data_points: Option<Vec<f64>>,
    graph_color: KeyOrValue<Color>,
    // retained state for rendering the y axis.
    y_scale: Option<Scale>,
}

impl BoxPlot {
    pub fn new() -> Self {
        let mut title_layout = TextLayout::new();
        title_layout.set_text_size(20.);
        BoxPlot {
            title_layout,
            sorted_data_points: None,
            graph_color: LABEL_COLOR.into(),
            y_scale: None,
        }
    }

    /// Rebuild any parts of the retained state that need rebuilding.
    fn rebuild_if_needed(&mut self, ctx: &mut PaintCtx, data: &BoxPlotData, env: &Env) {
        self.title_layout.rebuild_if_needed(ctx.text(), env);
        if self.sorted_data_points.is_none() {
            let mut dp: Vec<f64> = data.data_points.iter().copied().collect();
            dp.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
            self.sorted_data_points = Some(dp);
        }
        if self.y_scale.is_none() {
            self.y_scale = Some(Scale::new_y(data_as_range(
                self.sorted_data_points.as_ref().unwrap().iter().copied(),
            )));
        }
        let graph_bounds = self.graph_bounds(ctx.size());
        let y_scale = self.y_scale.as_mut().unwrap();
        y_scale.set_graph_bounds(graph_bounds);
        y_scale.rebuild_if_needed(ctx, env);
    }

    pub fn graph_bounds(&self, size: Size) -> Rect {
        size.to_rect().inset(GRAPH_INSETS)
    }
}

impl Widget<BoxPlotData> for BoxPlot {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut BoxPlotData, env: &Env) {}

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &BoxPlotData,
        env: &Env,
    ) {
        match event {
            LifeCycle::WidgetAdded => {
                self.title_layout.set_text(data.title.clone());
            }
            _ => (),
        }
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &BoxPlotData,
        data: &BoxPlotData,
        env: &Env,
    ) {
        if !Data::same(&old_data.title, &data.title) {
            // relaying out the text is potentially expensive, so worth an equality check.
            if old_data.title != data.title {
                self.title_layout.set_text(data.title.clone());
            }
        }
        self.title_layout.needs_rebuild_after_update(ctx);
        if !Data::same(&old_data.data_points, &data.data_points) {
            if old_data.data_points != data.data_points {
                self.sorted_data_points = None;
                self.y_scale = None;
            }
        } else {
            if let Some(y_scale) = self.y_scale.as_mut() {
                y_scale.needs_rebuild_after_update(ctx);
            }
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &BoxPlotData,
        env: &Env,
    ) -> Size {
        bc.constrain((f64::INFINITY, f64::INFINITY))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &BoxPlotData, env: &Env) {
        self.rebuild_if_needed(ctx, data, env);
        let size = ctx.size();
        let bounds = size.to_rect();
        let graph_bounds = self.graph_bounds(size);
        let axes_brush = ctx.solid_brush(Color::hlc(0.0, 60.0, 0.0));
        let text_brush = ctx.solid_brush(Color::WHITE);
        let bar_brush = ctx.solid_brush(Color::hlc(0.0, 50.0, 50.0));

        // data stats
        let mut data_points = data.data_points.clone();
        data_points.sort_by(|left, right| left.partial_cmp(right).expect("cannot sort NaNs"));
        assert!(data.data_points.len() > 0);
        let data_min = *data_points.front().unwrap();
        let data_qn10 = quantile(&data_points, 0.1);
        let data_qn25 = quantile(&data_points, 0.25);
        let data_qn50 = quantile(&data_points, 0.5);
        let data_qn75 = quantile(&data_points, 0.75);
        let data_qn90 = quantile(&data_points, 0.9);
        let data_max = *data_points.back().unwrap();

        // title
        let title_size = self.title_layout.size();
        self.title_layout
            .draw(ctx, ((size.width - title_size.width) * 0.5, 40.0));

        let datum_to_height = |datum: f64| -> f64 {
            let t = (datum - data_min) / (data_max - data_min);
            graph_bounds.y1 - t * graph_bounds.height()
        };

        // y axis
        self.y_scale.as_mut().unwrap().draw(ctx, env);

        // data
        const PLOT_WIDTH: f64 = 32.0;
        let x_center =
            ((graph_bounds.x1 + graph_bounds.x0) * 0.5).max(graph_bounds.x0 + PLOT_WIDTH * 0.5);
        let horiz_line = |datum| {
            let y = datum_to_height(datum);
            Line::new(
                (x_center - PLOT_WIDTH * 0.5, y),
                (x_center + PLOT_WIDTH * 0.5, y),
            )
        };
        ctx.stroke(horiz_line(data_qn90), &text_brush, 1.0);
        ctx.stroke(
            Line::new(
                (x_center, datum_to_height(data_qn90)),
                (x_center, datum_to_height(data_qn75)),
            ),
            &text_brush,
            1.0,
        );
        ctx.stroke(
            Rect::new(
                x_center - PLOT_WIDTH * 0.5,
                datum_to_height(data_qn75),
                x_center + PLOT_WIDTH * 0.5,
                datum_to_height(data_qn25),
            ),
            &text_brush,
            1.0,
        );
        ctx.stroke(horiz_line(data_qn50), &text_brush, 1.0);
        ctx.stroke(
            Line::new(
                (x_center, datum_to_height(data_qn25)),
                (x_center, datum_to_height(data_qn10)),
            ),
            &text_brush,
            1.0,
        );
        ctx.stroke(horiz_line(data_qn10), &text_brush, 1.0);

        let mut draw_cross = |(x, y)| {
            let cross = Rect::from_center_size((x, y), (PLOT_WIDTH * 0.25, PLOT_WIDTH * 0.25));
            ctx.stroke(
                Line::new((cross.x0, cross.y0), (cross.x1, cross.y1)),
                &text_brush,
                1.0,
            );
            ctx.stroke(
                Line::new((cross.x0, cross.y1), (cross.x1, cross.y0)),
                &text_brush,
                1.0,
            );
        };
        for datum in data_points.iter().copied() {
            let mut prev_datum = None;
            if datum < data_qn10 || datum > data_qn90 {
                if let Some(d) = prev_datum {
                    if d == datum {
                        continue;
                    }
                }
                /*
                ctx.stroke(
                    Circle::new((x_center, datum_to_height(datum)), 4.0),
                    &text_brush,
                    1.0,
                );
                */
                draw_cross((x_center, datum_to_height(datum)));
                prev_datum = Some(datum);
            }
        }
    }
}

/// Get the pth quantile from data.
fn quantile(data: &Vector<f64>, p: f64) -> f64 {
    let np1 = (data.len() + 1) as f64;
    let k = (p * np1).floor() as usize;
    let x_k = data.iter().copied().nth(k).unwrap();
    let x_kp1 = data.iter().copied().nth(k + 1).unwrap();
    let alpha = p * np1 - k as f64;
    x_k + alpha * (x_kp1 - x_k)
}
