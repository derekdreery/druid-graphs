use druid::{
    im::Vector,
    kurbo::{Affine, Circle, CircleSegment, Line, Point, Rect},
    piet::{self, Text, TextLayout as _, TextLayoutBuilder as _, TextStorage},
    BoxConstraints, Color, Data, Env, Event, EventCtx, Insets, LayoutCtx, LifeCycle, LifeCycleCtx,
    PaintCtx, RenderContext, Size, TextLayout, UpdateCtx, Widget,
};
use std::{f64::consts::PI, sync::Arc};

use crate::{
    axes::{axis_heuristic, PositionedLayout},
    GRAPH_INSETS,
};

/// A histogram of equal width categories
#[derive(Debug, Clone, Data)]
pub struct BoxPlotData {
    pub title: Arc<str>,
    pub data_points: Vector<f64>,
}

// When you continue: retain more state, especially but not solely text layout.

#[derive(Clone)]
pub struct BoxPlot {
    title_layout: TextLayout<Arc<str>>,
    // retained state for rendering the y axis.
    y_axis: Option<YAxis>,
}

#[derive(Clone)]
struct YAxis {
    /// height of the axis in druid pixel units that we built for.
    height: f64,
    /// (min, max) in variable units
    range: (f64, f64),
    /// y axis value label layouts.
    value_labels: Vec<PositionedLayout<Arc<str>>>,
}

impl BoxPlot {
    pub fn new() -> Self {
        let mut title_layout = TextLayout::new();
        title_layout.set_text_color(Color::BLACK);
        title_layout.set_text_size(20.);
        BoxPlot {
            title_layout,
            y_axis: None,
        }
    }

    /// Rebuild any parts of the retained state that need rebuilding.
    fn rebuild_as_needed(&mut self, ctx: &mut PaintCtx, data: &BoxPlotData, env: &Env) {
        if self.y_axis.is_none()
            || matches!(self.y_axis, Some(YAxis { height, ..}) if height != ctx.size().height)
        {
            self.y_axis = Some(self.build_y_axis(ctx, data, env));
        }
    }

    fn build_y_axis(&mut self, ctx: &mut PaintCtx, data: &BoxPlotData, env: &Env) -> YAxis {
        fn label_position(layout: &Layout, graph_bounds: Rect, label_ratio: f64) -> Point {
            Point::new(
                graph_bounds.x0 - layout.size().width - 5.,
                graph_bounds.y1 - label_ratio * graph_bounds.height() - layout.size().height * 0.5,
            )
        }
        let graph_bounds = self.graph_bounds(ctx.size());
        let height = graph_bounds.height();
        let range = y_axis_range(&data.data_points);
        if range.0 < 0. || !range.0.is_finite() || !range.1.is_finite() {
            todo!("negative data not yet supported");
        }
        // for now space labels a minimum of 40 pixels apart.
        let max_num_points = (graph_bounds.height() / 40.0).floor() as usize + 1;
        let mut value_labels = vec![];
        match max_num_points {
            0 => unreachable!(),
            1 => {
                let layout = self.new_y_label(0., ctx);
                value_labels.push(PositionedLayout {
                    position: Point::new(
                        graph_bounds.x0 - layout.size().width - 5.,
                        graph_bounds.y1 - layout.size().height * 0.5,
                    ),
                    layout,
                });
            }
            max_num_points => {
                let label_gap = axis_heuristic(range.1, max_num_points);
                debug_assert!(label_gap > 0.);
                let mut label_val = 0.;
                while label_val <= range.1 {
                    let mut layout = self.new_y_label(label_val, ctx);
                    layout.rebuild_if_needed(ctx.text(), env);
                    value_labels.push(PositionedLayout {
                        position: Point::new(
                            graph_bounds.x0 - layout.size().width - 5.,
                            graph_bounds.y1
                                - label_val / range.1 * graph_bounds.height()
                                - layout.size().height * 0.5,
                        ),
                        layout,
                    });
                    label_val += label_gap;
                }
            }
        }
        YAxis {
            height,
            range,
            value_labels,
        }
    }

    pub fn graph_bounds(&self, size: Size) -> Rect {
        size.to_rect().inset(GRAPH_INSETS)
    }

    fn new_y_label(&self, scale_val: f64, ctx: &mut PaintCtx) -> TextLayout<Arc<str>> {
        let scale_val = if scale_val < 1_000. && scale_val > 0.0001 || scale_val == 0. {
            scale_val.to_string()
        } else {
            format!("{:e}", scale_val)
        };
        let text_storage: Arc<str> = scale_val.into();
        let mut layout = TextLayout::from_text(text_storage);
        layout.set_text_color(Color::BLACK);
        layout
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
            if old_data.title != data.title {
                self.title_layout.set_text(data.title.clone());
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
        let graph_bounds = self.graph_bounds(ctx.size());
        self.rebuild_as_needed(ctx, data, env);
        let bg_brush = ctx.solid_brush(Color::hlc(0.0, 90.0, 0.0));
        let axes_brush = ctx.solid_brush(Color::hlc(0.0, 60.0, 0.0));
        let text_brush = ctx.solid_brush(Color::BLACK);
        let bar_brush = ctx.solid_brush(Color::hlc(0.0, 50.0, 50.0));
        let size = ctx.size();
        let bounds = size.to_rect();

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

        // background & title
        ctx.fill(bounds, &bg_brush);
        self.title_layout.rebuild_if_needed(ctx.text(), env);
        let title_size = self.title_layout.size();
        self.title_layout
            .draw(ctx, ((size.width - title_size.width) * 0.5, 40.0));

        let datum_to_height =
            |datum: f64| -> f64 { graph_bounds.y1 - (datum / data_max * graph_bounds.height()) };

        // y axis
        {
            if graph_bounds.height() <= 0. {
                // don't render the axis if there isn't any space
                return;
            }
            let y_axis = Line::new(
                (graph_bounds.x0, graph_bounds.y0),
                (graph_bounds.x0, graph_bounds.y1),
            );
            ctx.stroke(y_axis, &axes_brush, 2.0);
            for value_label in self.y_axis.as_mut().unwrap().value_labels.iter_mut() {
                value_label.draw(ctx, env);
            }
        }

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

/// Returns (min, max) of the vector.
///
/// NaNs are propogated.
fn y_axis_range(data: &Vector<f64>) -> (f64, f64) {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for v in data.iter().copied() {
        if v.is_nan() {
            return (f64::NAN, f64::NAN);
        }
        if v < min {
            min = v;
        }
        if v > max {
            max = v;
        }
    }
    (min, max)
}
