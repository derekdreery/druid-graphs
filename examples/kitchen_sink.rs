use anyhow::Error;
use druid::widget::{Align, Flex, Label, Painter, TextBox, ViewSwitcher};
use druid::{
    AppLauncher, BoxConstraints, Color, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle,
    LifeCycleCtx, LocalizedString, PaintCtx, RenderContext, Size, UpdateCtx, Widget, WidgetExt,
    WindowDesc,
};
use druid_graphs::{BoxPlot, BoxPlotData, Histogram, HistogramData, PieChart, PieChartData};
use im::{vector, Vector};

const VERTICAL_WIDGET_SPACING: f64 = 20.0;
const TEXT_BOX_WIDTH: f64 = 200.0;
const WINDOW_TITLE: LocalizedString<HelloState> = LocalizedString::new("Hello World!");

#[derive(Debug, Clone, Data, Lens)]
struct HelloState {
    active_tab_idx: usize,
    monica: MonicaData,
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
    };

    // start the application
    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .expect("Failed to launch application");
}

fn build_root_widget() -> impl Widget<HelloState> {
    let tab_labels = ["Histogram", "Box Plot", "Pie Chart"];

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
        move |tab_idx, state, env| match tab_idx {
            0 => Histogram::new().lens(HistogramLens).boxed(),
            1 => BoxPlot::new().lens(BoxPlotLens).boxed(),
            2 => PieChart::new().lens(PieChartLens).boxed(),
            _ => unreachable!(),
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
            ctx.fill(bounds, &Color::hlc(0.0, 20.0, 0.0));
        } else {
            ctx.fill(bounds, &Color::hlc(0.0, 40.0, 0.0));
        }
    })
}

struct HistogramLens;

impl Lens<HelloState, HistogramData> for HistogramLens {
    fn with<V, F: FnOnce(&HistogramData) -> V>(&self, data: &HelloState, f: F) -> V {
        f(&HistogramData {
            title: "Distribution of BMI".to_string(),
            x_axis_label: "BMI".to_string(),
            x_axis: vector![
                "10-15".into(),
                "15-20".into(),
                "20-25".into(),
                "25-30".into(),
                "30-35".into(),
                "35-40".into(),
                "40-45".into(),
                "45-50".into()
            ],
            counts: data.monica.bucket_bmi(),
        })
    }
    fn with_mut<V, F: FnOnce(&mut HistogramData) -> V>(&self, data: &mut HelloState, f: F) -> V {
        f(&mut HistogramData {
            title: "Distribution of BMI".to_string(),
            x_axis_label: "BMI".to_string(),
            x_axis: vector![
                "10-15".into(),
                "15-20".into(),
                "20-25".into(),
                "25-30".into(),
                "30-35".into(),
                "35-40".into(),
                "40-45".into(),
                "45-50".into()
            ],
            counts: data.monica.bucket_bmi(),
        })
    }
}

struct PieChartLens;

impl Lens<HelloState, PieChartData> for PieChartLens {
    fn with<V, F: FnOnce(&PieChartData) -> V>(&self, data: &HelloState, f: F) -> V {
        f(&PieChartData {
            title: "Gender".to_string(),
            category_labels: vector!["Female".to_string(), "Male".to_string()],
            counts: data.monica.bucket_sex(),
        })
    }
    fn with_mut<V, F: FnOnce(&mut PieChartData) -> V>(&self, data: &mut HelloState, f: F) -> V {
        f(&mut PieChartData {
            title: "Gender".to_string(),
            category_labels: vector!["Female".to_string(), "Male".to_string()],
            counts: data.monica.bucket_sex(),
        })
    }
}

struct BoxPlotLens;

impl Lens<HelloState, BoxPlotData> for BoxPlotLens {
    fn with<V, F: FnOnce(&BoxPlotData) -> V>(&self, data: &HelloState, f: F) -> V {
        f(&BoxPlotData {
            title: "Systolic BP".to_string(),
            y_axis_label: "Systolic BP".to_string(),
            data_points: data.monica.systm.clone(),
        })
    }
    fn with_mut<V, F: FnOnce(&mut BoxPlotData) -> V>(&self, data: &mut HelloState, f: F) -> V {
        f(&mut BoxPlotData {
            title: "Systolic BP".to_string(),
            y_axis_label: "Systolic BP".to_string(),
            data_points: data.monica.systm.clone(),
        })
    }
}

// load monica data

#[derive(Debug, Default, Clone, Data)]
struct MonicaData {
    #[data(same_fn = "Vector::ptr_eq")]
    sex: Vector<u8>,
    #[data(same_fn = "Vector::ptr_eq")]
    marit: Vector<u8>,
    #[data(same_fn = "Vector::ptr_eq")]
    edlevel: Vector<u8>,
    #[data(same_fn = "Vector::ptr_eq")]
    age: Vector<u8>,
    #[data(same_fn = "Vector::ptr_eq")]
    systm: Vector<f64>,
    #[data(same_fn = "Vector::ptr_eq")]
    diastm: Vector<f64>,
    #[data(same_fn = "Vector::ptr_eq")]
    bmi: Vector<f64>,
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
        Ok(data)
    }

    /// Collect BMI data into buckets.
    fn bucket_bmi(&self) -> Vector<usize> {
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
        out
    }

    fn bucket_sex(&self) -> Vector<usize> {
        let mut male = 0;
        let mut female = 0;
        for datum in self.sex.iter().copied() {
            match datum {
                0 => female += 1,
                1 => male += 1,
                _ => panic!("invalid sex"),
            }
        }
        vector![female, male]
    }
}
