use druid::{
    kurbo::{Affine, Circle, CircleSegment, Line, Rect},
    piet::{FontBuilder, PietTextLayout, Text, TextLayout, TextLayoutBuilder},
    BoxConstraints, Color, Data, Env, Event, EventCtx, Insets, LayoutCtx, LifeCycle, LifeCycleCtx,
    PaintCtx, RenderContext, Size, UpdateCtx, Widget,
};
use im::Vector;
use std::f64::consts::PI;

use crate::{axis_heuristic, GRAPH_INSETS};

/// A histogram of equal width categories
#[derive(Debug, Clone, Data)]
pub struct BoxPlotData {
    pub title: String,
    pub y_axis_label: String,
    #[data(same_fn = "Vector::ptr_eq")]
    pub data_points: Vector<f64>,
}

pub struct BoxPlot;

impl BoxPlot {
    pub fn new() -> Self {
        BoxPlot
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
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &BoxPlotData,
        data: &BoxPlotData,
        env: &Env,
    ) {
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
        let bg_brush = ctx.solid_brush(Color::hlc(0.0, 90.0, 0.0));
        let axes_brush = ctx.solid_brush(Color::hlc(0.0, 60.0, 0.0));
        let text_brush = ctx.solid_brush(Color::BLACK);
        let bar_brush = ctx.solid_brush(Color::hlc(0.0, 50.0, 50.0));
        let size = ctx.size();
        let bounds = size.to_rect();
        let graph_bounds = bounds.inset(GRAPH_INSETS);

        // data stats
        let mut data_points = data.data_points.clone();
        data_points.sort_by(|left, right| left.partial_cmp(right).expect("cannot sort nans"));
        assert!(data.data_points.len() > 0);
        let data_min = *data_points.front().unwrap();
        let data_qn10 = quantile(&data_points, 0.1);
        let data_qn25 = quantile(&data_points, 0.25);
        let data_qn50 = quantile(&data_points, 0.5);
        let data_qn75 = quantile(&data_points, 0.75);
        let data_qn90 = quantile(&data_points, 0.9);
        let data_max = *data_points.back().unwrap();

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

        let datum_to_height =
            |datum: f64| -> f64 { graph_bounds.y1 - (datum / data_max * graph_bounds.height()) };

        // y axis
        {
            let y_axis = Line::new(
                (graph_bounds.x0, graph_bounds.y0),
                (graph_bounds.x0, graph_bounds.y1),
            );
            ctx.stroke(y_axis, &axes_brush, 2.0);
            let label_gap =
                axis_heuristic(data_max, (graph_bounds.height() / 40.0).floor() as usize);
            let mut label_pos = 0.0;
            while label_pos < data_max {
                let y_pos = datum_to_height(label_pos);
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
        const PLOT_WIDTH: f64 = 32.0;
        let x_center = (graph_bounds.x1 + graph_bounds.x0) * 0.5;
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
