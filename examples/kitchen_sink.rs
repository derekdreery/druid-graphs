use anyhow::Error;
use druid::{
    im::{vector, Vector},
    lens,
    theme::{WIDGET_PADDING_HORIZONTAL, WIDGET_PADDING_VERTICAL},
    widget::{Align, Checkbox, CrossAxisAlignment, Flex, Label, Painter, TextBox, ViewSwitcher},
    AppLauncher, ArcStr, BoxConstraints, Color, Data, Env, Event, EventCtx, LayoutCtx, Lens,
    LensExt, LifeCycle, LifeCycleCtx, LocalizedString, PaintCtx, RenderContext, Size, UpdateCtx,
    Widget, WidgetExt, WindowDesc,
};
use druid_graphs::{
    BoxPlot, BoxPlotData, Histogram, HistogramData, LineChart, LineChartData, PieChart,
    PieChartData,
};
use std::sync::Arc;

const VERTICAL_WIDGET_SPACING: f64 = 20.0;
const TEXT_BOX_WIDTH: f64 = 200.0;
const WINDOW_TITLE: LocalizedString<HelloState> =
    LocalizedString::new("Graphs of the MONICA dataset");

#[derive(Debug, Clone, Data, Lens)]
struct HelloState {
    active_tab_idx: usize,
    monica: MonicaData,
    box_title: ArcStr,
    line_title: Arc<String>,
    line_x_label: Arc<String>,
    show_x_axis: bool,
    show_x_tick_labels: bool,
    show_y_axis: bool,
    show_y_tick_labels: bool,
}

fn main() {
    // describe the main window
    let main_window = WindowDesc::new(build_root_widget)
        .title(WINDOW_TITLE)
        .window_size((400.0, 400.0));

    // create the initial app state
    let initial_state = HelloState {
        active_tab_idx: 0,
        monica: MonicaData::load().unwrap(),
        box_title: "Systolic BP".into(),
        line_title: Arc::new(String::from("Blood pressure")),
        line_x_label: Arc::new(String::from("Person number (order meaningless)")),
        show_x_axis: true,
        show_x_tick_labels: true,
        show_y_axis: true,
        show_y_tick_labels: true,
    };

    // start the application
    AppLauncher::with_window(main_window)
        .configure_env(|env, _| druid_graphs::add_to_env(env))
        .launch(initial_state)
        .expect("Failed to launch application");
}

fn build_root_widget() -> impl Widget<HelloState> {
    let tab_labels = ["Histogram", "Box Plot", "Pie Chart", "Line Chart"];

    let mut tabs = Flex::row();
    for (idx, label) in tab_labels.iter().enumerate() {
        tabs = tabs.with_flex_child(
            Label::new(*label)
                .padding((24.0, 8.0))
                .background(make_background(idx))
                .on_click(move |ctx, data: &mut HelloState, env| {
                    data.active_tab_idx = idx;
                }),
            1.0,
        );
    }

    let main_content = ViewSwitcher::new(
        |state: &HelloState, _env| state.active_tab_idx,
        move |tab_idx, state, env| {
            let vspace = env.get(WIDGET_PADDING_VERTICAL);
            let hspace = env.get(WIDGET_PADDING_HORIZONTAL);
            match tab_idx {
                0 => Histogram::new()
                    .lens(HistogramData::compose_lens(
                        Constant("Distribution of BMI".into()),
                        Constant("BMI".into()),
                        Constant(vector![
                            "10-15".into(),
                            "15-20".into(),
                            "20-25".into(),
                            "25-30".into(),
                            "30-35".into(),
                            "35-40".into(),
                            "40-45".into(),
                            "45-50".into()
                        ]),
                        HelloState::monica.then(MonicaData::bucket_bmi),
                    ))
                    .boxed(),
                1 => BoxPlot::new()
                    .lens(BoxPlotData::compose_lens(
                        HelloState::box_title,
                        HelloState::monica.then(MonicaData::systm),
                    ))
                    .fix_width(300.)
                    .boxed(),
                2 => PieChart::new()
                    .lens(PieChartData::compose_lens(
                        Constant("Gender".into()),
                        Constant(vector!["female".into(), "male".into()]),
                        HelloState::monica.then(MonicaData::bucket_sex),
                    ))
                    .boxed(),
                3 => Flex::row()
                    .with_flex_child(
                        LineChart::new().lens(LineChartData::compose_lens(
                            HelloState::line_title,
                            // x axis
                            HelloState::line_x_label,
                            Constant(None),
                            HelloState::show_x_tick_labels,
                            HelloState::show_x_axis,
                            Constant(None),
                            // y axis
                            Constant(None),
                            HelloState::show_y_tick_labels,
                            HelloState::show_y_axis,
                            HelloState::monica.then(MonicaData::systm),
                        )),
                        2.,
                    )
                    .with_spacer(hspace)
                    .with_flex_child(
                        Flex::column()
                            .cross_axis_alignment(CrossAxisAlignment::Start)
                            .with_child(Label::new("Chart title"))
                            .with_child(TextBox::new().lens(HelloState::line_title).fix_width(300.))
                            .with_spacer(vspace)
                            .with_child(Label::new("X-axis label"))
                            .with_child(
                                TextBox::new()
                                    .lens(HelloState::line_x_label)
                                    .fix_width(300.),
                            )
                            .with_spacer(vspace)
                            .with_child(Checkbox::new("show x axis").lens(HelloState::show_x_axis))
                            .with_spacer(vspace)
                            .with_child(
                                Checkbox::new("show x value labels")
                                    .lens(HelloState::show_x_tick_labels),
                            )
                            .with_spacer(vspace)
                            .with_child(Checkbox::new("show y axis").lens(HelloState::show_y_axis))
                            .with_spacer(vspace)
                            .with_child(
                                Checkbox::new("show y value labels")
                                    .lens(HelloState::show_y_tick_labels),
                            ),
                        1.,
                    )
                    .boxed(),
                _ => unreachable!(),
            }
        },
    );

    Flex::column()
        .with_child(tabs)
        .with_flex_child(main_content, 1.0)
        .center()
}

fn make_background(idx: usize) -> Painter<HelloState> {
    Painter::new(move |ctx, data: &HelloState, env| {
        let bounds = ctx.size().to_rect();
        if data.active_tab_idx == idx {
            ctx.fill(bounds, &Color::hlc(0.0, 40.0, 0.0));
        } else {
            ctx.fill(bounds, &Color::hlc(0.0, 20.0, 0.0));
        }
    })
}

// load monica data

#[derive(Debug, Default, Clone, Data, Lens)]
struct MonicaData {
    sex: Vector<u8>,
    marit: Vector<u8>,
    edlevel: Vector<u8>,
    age: Vector<u8>,
    systm: Vector<f64>,
    diastm: Vector<f64>,
    bmi: Vector<f64>,
    bucket_bmi: Vector<usize>,
    bucket_sex: Vector<usize>,
}

impl MonicaData {
    fn load() -> Result<Self, Error> {
        let mut data = Self::default();

        let mut rdr = csv::Reader::from_path("monica.csv")?;
        for result in rdr.records() {
            let record = result?;
            data.sex.push_back(record.get(0).unwrap().parse()?);
            data.marit.push_back(record.get(1).unwrap().parse()?);
            data.edlevel.push_back(record.get(2).unwrap().parse()?);
            data.age.push_back(record.get(3).unwrap().parse()?);
            data.systm.push_back(record.get(4).unwrap().parse()?);
            data.diastm.push_back(record.get(5).unwrap().parse()?);
            data.bmi.push_back(record.get(6).unwrap().parse()?);
        }
        data.calc_bucket_bmi();
        data.calc_bucket_sex();
        Ok(data)
    }

    /// Collect BMI data into buckets.
    fn calc_bucket_bmi(&mut self) {
        let mut out = vector![0, 0, 0, 0, 0, 0, 0, 0];
        for datum in self.bmi.iter().copied() {
            if datum <= 10.0 {
                panic!("invalid bmi");
            } else if datum < 15.0 {
                out[0] += 1;
            } else if datum < 20.0 {
                out[1] += 1;
            } else if datum < 25.0 {
                out[2] += 1;
            } else if datum < 30.0 {
                out[3] += 1;
            } else if datum < 35.0 {
                out[4] += 1;
            } else if datum < 40.0 {
                out[5] += 1;
            } else if datum < 45.0 {
                out[6] += 1;
            } else if datum < 50.0 {
                out[7] += 1;
            } else {
                panic!("very large bmi");
            }
        }
        self.bucket_bmi = out;
    }

    fn calc_bucket_sex(&mut self) {
        let mut male = 0;
        let mut female = 0;
        for datum in self.sex.iter().copied() {
            match datum {
                0 => female += 1,
                1 => male += 1,
                _ => panic!("invalid sex"),
            }
        }
        self.bucket_sex = vector![female, male];
    }
}

/// A lens that always gives the same value and discards changes.
#[derive(Debug, Copy, Clone)]
pub struct Constant<T>(pub T);

impl<A, B: Clone> Lens<A, B> for Constant<B> {
    fn with<V, F: FnOnce(&B) -> V>(&self, _: &A, f: F) -> V {
        f(&self.0)
    }
    fn with_mut<V, F: FnOnce(&mut B) -> V>(&self, _: &mut A, f: F) -> V {
        let mut tmp = self.0.clone();
        f(&mut tmp)
    }
}
