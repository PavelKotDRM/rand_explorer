use std::process::Command;
use std::fs;
use std::time::Instant;

use eframe::{egui, App as EguiApp};
use egui::{Align, Color32, ComboBox, Frame, Layout, Panel, RichText, Slider, Window};
use egui_extras::syntax_highlighting::{CodeTheme, code_view_ui};
use egui_plot::{Bar, BarChart, Line, Plot, PlotPoints};
use plotly::{Plot as PlotlyPlot, Scatter};
use rand::distr::{weighted::WeightedIndex, Bernoulli, Distribution, Uniform};
use rand::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;
use rand_distr::{Exp, Normal, Poisson};

use crate::build_info::BuildInfo;
use crate::state::{
    sample_uniform_range, AppState, DistributionKind, EngineKind, generate_seed, reseed_for_kind,
};

const RAND_COOKBOOK_EXAMPLE: &str = r#"// import commonly used items from the prelude:
use rand::prelude::*;

fn main() {
    // We can use random() immediately. It can produce values of many common types:
    let x: u8 = rand::random();
    println!("{}", x);

    if rand::random() { // generates a boolean
        println!("Heads!");
    }

    // If we want to be a bit more explicit (and a little more efficient) we can
    // make a handle to the thread-local generator:
    let mut rng = rand::rng();
    if rng.random() { // random bool
        let x: f64 = rng.random(); // random number in range [0, 1)
        let y = rng.random_range(-10.0..10.0);
        println!("x is: {}", x);
        println!("y is: {}", y);
    }

    println!("Dice roll: {}", rng.random_range(1..=6));
    println!("Number from 0 to 9: {}", rng.random_range(0..10));

    // Sometimes it's useful to use distributions directly:
    let distr = rand::distr::Uniform::new_inclusive(1, 100).unwrap();
    let mut nums = [0i32; 3];
    for x in &mut nums {
        *x = rng.sample(distr);
    }
    println!("Some numbers: {:?}", nums);

    // We can also interact with iterators and slices:
    let arrows_iter = "➡⬈⬆⬉⬅⬋⬇⬊".chars();
    println!("Lets go in this direction: {}", arrows_iter.choose(&mut rng).unwrap());
    let mut nums = [1, 2, 3, 4, 5];
    nums.shuffle(&mut rng);
    println!("I shuffled my {:?}", nums);
}"#;

const RANGE_EXAMPLE: &str = r#"let mut rng = rand::rng();
let value = rng.random_range(-10.0..10.0);
let inclusive = rng.random_range(1..=6);
println!("value: {value}, dice: {inclusive}");"#;

const DISTRIBUTION_EXAMPLE: &str = r#"use rand::distr::{Distribution, Uniform};
use rand_distr::Normal;

let mut rng = rand::rng();
let uniform = Uniform::new(0.0, 1.0).unwrap();
let normal = Normal::new(0.0, 1.0).unwrap();
let sample = uniform.sample(&mut rng);
let gaussian = normal.sample(&mut rng);
println!("{sample} {gaussian}");"#;

const SEQUENCE_EXAMPLE: &str = r#"use rand::seq::{IndexedRandom, SliceRandom};

let mut rng = rand::rng();
let mut values = ["A", "B", "C", "D"];
let chosen = values.choose(&mut rng);
let many = values.sample(&mut rng, 2);
values.shuffle(&mut rng);
println!("{chosen:?} {many:?} {values:?}");"#;

const ENGINE_EXAMPLE: &str = r#"use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

let seed = 42;
let mut first = ChaCha20Rng::seed_from_u64(seed);
let mut second = ChaCha20Rng::seed_from_u64(seed);
assert_eq!(first.random::<u64>(), second.random::<u64>());"#;

const SYSTEM_EXAMPLE: &str = r#"println!("package: {}", env!("CARGO_PKG_VERSION"));
println!("target: {}", std::env::consts::ARCH);
println!("renderer: Glow / OpenGL");"#;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Primitives,
    Ranges,
    Distributions,
    Sequences,
    Engines,
    System,
}

pub struct App {
    state: AppState,
    active_tab: ActiveTab,
    show_sample_window: bool,
    build_info: BuildInfo,
    last_primitive_update: Instant,
    last_range_update: Instant,
    status_message: String,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::configure_fonts(&cc.egui_ctx);
        let mut app = Self {
            state: AppState::default(),
            active_tab: ActiveTab::Primitives,
            show_sample_window: false,
            build_info: BuildInfo::new(),
            last_primitive_update: Instant::now(),
            last_range_update: Instant::now(),
            status_message: String::new(),
        };
        app.rebuild_primitive_samples();
        app.rebuild_distribution();
        app.rebuild_range_samples();
        app
    }

    fn configure_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "noto_sans".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/NotoSans-Regular.ttf")).into(),
        );
        fonts.font_data.insert(
            "noto_symbols".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "../assets/fonts/NotoSansSymbols2-Regular.ttf"
            ))
            .into(),
        );
        fonts.font_data.insert(
            "unifont".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "../assets/fonts/Unifont-17.0.05.otf"
            ))
            .into(),
        );
        fonts.font_data.insert(
            "noto_hebrew".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "../assets/fonts/NotoSansHebrew-Regular.ttf"
            ))
            .into(),
        );
        fonts.font_data.insert(
            "noto_arabic".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "../assets/fonts/NotoSansArabic-Regular.ttf"
            ))
            .into(),
        );
        fonts.font_data.insert(
            "noto_cjk".to_owned(),
            egui::FontData::from_static(include_bytes!(
                "../assets/fonts/NotoSansCJKsc-Regular.otf"
            ))
            .into(),
        );

        let proportional = fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .expect("egui proportional font family");
        proportional.insert(0, "noto_sans".to_owned());
        proportional.push("noto_symbols".to_owned());
        proportional.push("noto_hebrew".to_owned());
        proportional.push("noto_arabic".to_owned());
        proportional.push("noto_cjk".to_owned());
        proportional.push("unifont".to_owned());

        let monospace = fonts
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .expect("egui monospace font family");
        monospace.push("noto_symbols".to_owned());
        monospace.push("noto_hebrew".to_owned());
        monospace.push("noto_arabic".to_owned());
        monospace.push("noto_cjk".to_owned());
        monospace.push("unifont".to_owned());
        ctx.set_fonts(fonts);
    }

    fn save_preset(&mut self) {
        match serde_json::to_string_pretty(&self.state)
            .map_err(|error| error.to_string())
            .and_then(|json| fs::write("rand_explorer_preset.json", json).map_err(|error| error.to_string()))
        {
            Ok(()) => self.status_message = "Preset saved to rand_explorer_preset.json".to_owned(),
            Err(error) => self.status_message = format!("Save failed: {error}"),
        }
    }

    fn load_preset(&mut self) {
        match fs::read_to_string("rand_explorer_preset.json")
            .map_err(|error| error.to_string())
            .and_then(|json| serde_json::from_str(&json).map_err(|error| error.to_string()))
        {
            Ok(state) => {
                self.state = state;
                self.rebuild_primitive_samples();
                self.rebuild_distribution();
                self.status_message = "Preset loaded".to_owned();
            }
            Err(error) => self.status_message = format!("Load failed: {error}"),
        }
    }

    fn export_distribution(&mut self) {
        let csv = self
            .state
            .distribution
            .values
            .iter()
            .enumerate()
            .map(|(index, value)| format!("{index},{value}"))
            .collect::<Vec<_>>()
            .join("\n");
        match fs::write("rand_explorer_samples.csv", format!("index,value\n{csv}\n")) {
            Ok(()) => self.status_message = "Samples exported to rand_explorer_samples.csv".to_owned(),
            Err(error) => self.status_message = format!("Export failed: {error}"),
        }
    }

    fn open_plotly_distribution(&self) {
        let values = self.state.distribution.values.clone();
        let x_values: Vec<f64> = (0..values.len()).map(|idx| idx as f64).collect();

        let mut plot = PlotlyPlot::new();
        plot.add_trace(Scatter::new(x_values, values).mode(plotly::common::Mode::Lines));
        plot.set_layout(
            plotly::Layout::new().title(plotly::common::Title::with_text("rand_explorer distribution")),
        );

        let path = std::env::temp_dir().join("rand_explorer_distribution_plot.html");
        plot.write_html(path.to_string_lossy().as_ref());

        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("cmd").args(["/C", "start", "", path.to_string_lossy().as_ref()]).status();
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = Command::new("xdg-open").arg(&path).status();
        }
    }

    fn rebuild_primitive_samples(&mut self) {
        let count = self.state.primitive_count;
        let mut rng = rand::rng();
        self.state.primitive_samples.u8 = (0..count).map(|_| rng.random::<u8>()).collect();
        self.state.primitive_samples.i8 = (0..count).map(|_| rng.random::<i8>()).collect();
        self.state.primitive_samples.u32 = (0..count).map(|_| rng.random::<u32>()).collect();
        self.state.primitive_samples.i32 = (0..count).map(|_| rng.random::<i32>()).collect();
        self.state.primitive_samples.u64 = (0..count).map(|_| rng.random::<u64>()).collect();
        self.state.primitive_samples.i64 = (0..count).map(|_| rng.random::<i64>()).collect();
        self.state.primitive_samples.u128 = (0..count).map(|_| rng.random::<u128>()).collect();
        self.state.primitive_samples.i128 = (0..count).map(|_| rng.random::<i128>()).collect();
        self.state.primitive_samples.f32 = (0..count).map(|_| rng.random::<f32>()).collect();
        self.state.primitive_samples.f64 = (0..count).map(|_| rng.random::<f64>()).collect();
        self.state.primitive_samples.bools = (0..count).map(|_| rng.random()).collect();
        self.state.primitive_samples.chars = (0..count)
            .map(|_| Self::random_visible_char(&mut rng))
            .collect();
        self.state.primitive_samples.arrows = Self::random_arrow_sequence(&mut rng);
    }

    fn random_visible_char(rng: &mut impl Rng) -> char {
        const RANGES: &[(u32, u32)] = &[
            (0x21, 0x7E),
            (0x391, 0x3FF),
            (0x410, 0x44F),
            (0x590, 0x5FF),
            (0x600, 0x6FF),
            (0x4E00, 0x9FFF),
        ];
        let (start, end) = RANGES[rng.random_range(0..RANGES.len())];
        loop {
            if let Some(character) = char::from_u32(rng.random_range(start..=end))
                && !character.is_control()
                && !character.is_whitespace()
                && (start == 0x21 || character.is_alphanumeric())
            {
                return character;
            }
        }
    }

    fn random_arrow_sequence(rng: &mut impl Rng) -> String {
        let mut arrows = "➡⬈⬆⬉⬅⬋⬇⬊".chars().collect::<Vec<_>>();
        arrows.shuffle(rng);
        arrows.into_iter().collect()
    }

    fn format_char_sample(values: &[char]) -> String {
        values.iter().collect()
    }

    fn code_example(active_tab: ActiveTab) -> (&'static str, &'static str) {
        match active_tab {
            ActiveTab::Primitives => ("Primitive generation", RAND_COOKBOOK_EXAMPLE),
            ActiveTab::Ranges => ("Range generation", RANGE_EXAMPLE),
            ActiveTab::Distributions => ("Distribution sampling", DISTRIBUTION_EXAMPLE),
            ActiveTab::Sequences => ("Sequence operations", SEQUENCE_EXAMPLE),
            ActiveTab::Engines => ("Deterministic RNG", ENGINE_EXAMPLE),
            ActiveTab::System => ("Build information", SYSTEM_EXAMPLE),
        }
    }

    fn render_code_example(&self, ui: &mut egui::Ui) {
        let (title, source) = Self::code_example(self.active_tab);
        let mut theme = CodeTheme::from_memory(ui.ctx(), ui.style());

        ui.collapsing(format!("Rust example: {title}"), |ui| {
            ui.horizontal(|ui| {
                if ui.button("Copy").clicked() {
                    ui.ctx().copy_text(source.to_owned());
                }
                ui.label("Source code");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    theme.ui(ui);
                });
            });

            let frame = Frame::group(ui.style()).inner_margin(egui::Margin::same(8));
            frame.show(ui, |ui| {
                egui::ScrollArea::both()
                    .max_height(260.0)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        code_view_ui(ui, &theme, source, "rs");
                    });
            });
            theme.store_in_memory(ui.ctx());
        });
    }

    fn rebuild_range_samples(&mut self) {
        let bounds = &self.state.range_bounds;
        if bounds.start >= bounds.end {
            self.state.range_bounds.values.clear();
            return;
        }
        self.state.range_bounds.values = sample_uniform_range(
            bounds.start,
            bounds.end,
            bounds.sample_count,
            bounds.inclusive,
        );
    }

    fn distribution_pdf(&self, x: f64) -> Option<f64> {
        let dist = &self.state.distribution;
        match dist.kind {
            DistributionKind::Standard => {
                Some((-0.5 * x * x).exp() / (2.0 * std::f64::consts::PI).sqrt())
            }
            DistributionKind::Uniform => Some(if (0.0..=1.0).contains(&x) { 1.0 } else { 0.0 }),
            DistributionKind::Bernoulli => None,
            DistributionKind::Normal => {
                let sigma = dist.sigma.abs().max(f64::EPSILON);
                let z = (x - dist.mu) / sigma;
                Some((-0.5 * z * z).exp() / (sigma * (2.0 * std::f64::consts::PI).sqrt()))
            }
            DistributionKind::Exponential => {
                let lambda = dist.lambda.max(f64::EPSILON);
                Some(if x >= 0.0 { lambda * (-lambda * x).exp() } else { 0.0 })
            }
            DistributionKind::Poisson | DistributionKind::WeightedIndex => None,
        }
    }

    fn rebuild_distribution(&mut self) {
        let count = self.state.distribution.sample_size;
        let mut values = Vec::with_capacity(count);
        match self.state.distribution.kind {
            DistributionKind::Standard => {
                let dist = Normal::new(0.0, 1.0).expect("standard Normal parameters");
                let mut rng = rand::rng();
                for _ in 0..count {
                    values.push(dist.sample(&mut rng));
                }
            }
            DistributionKind::Uniform => {
                let dist = Uniform::new(0.0_f64, 1.0_f64).unwrap();
                let mut rng = rand::rng();
                for _ in 0..count {
                    values.push(dist.sample(&mut rng));
                }
            }
            DistributionKind::Bernoulli => {
                let probability = self.state.distribution.probability.clamp(0.0, 1.0);
                let dist = Bernoulli::new(probability).expect("clamped Bernoulli probability");
                let mut rng = rand::rng();
                for _ in 0..count {
                    values.push(if dist.sample(&mut rng) { 1.0 } else { 0.0 });
                }
            }
            DistributionKind::Normal => {
                let sigma = self.state.distribution.sigma.abs().max(f64::EPSILON);
                let dist = Normal::new(self.state.distribution.mu, sigma)
                    .expect("positive Normal standard deviation");
                let mut rng = rand::rng();
                for _ in 0..count {
                    values.push(dist.sample(&mut rng));
                }
            }
            DistributionKind::Exponential => {
                let lambda = self.state.distribution.lambda.max(f64::EPSILON);
                let dist = Exp::new(lambda).expect("positive Exponential rate");
                let mut rng = rand::rng();
                for _ in 0..count {
                    values.push(dist.sample(&mut rng));
                }
            }
            DistributionKind::Poisson => {
                let lambda = self.state.distribution.lambda.max(0.0);
                let dist = Poisson::new(lambda).expect("non-negative Poisson rate");
                let mut rng = rand::rng();
                for _ in 0..count {
                    values.push(dist.sample(&mut rng));
                }
            }
            DistributionKind::WeightedIndex => {
                let weights: Vec<f64> = self
                    .state
                    .distribution
                    .weights
                    .split(',')
                    .filter_map(|s| s.trim().parse::<f64>().ok().filter(|weight| *weight >= 0.0))
                    .collect();
                if let Ok(dist) = WeightedIndex::new(&weights) {
                    let mut rng = rand::rng();
                    for _ in 0..count {
                        values.push(dist.sample(&mut rng) as f64);
                    }
                }
            }
        }
        if values.is_empty() {
            values = vec![0.0; count];
        }
        self.state.distribution.values = values;
        if let Some(first) = self.state.distribution.values.first() {
            self.state.distribution.min = self.state.distribution.values.iter().copied().fold(*first, f64::min);
            self.state.distribution.max = self.state.distribution.values.iter().copied().fold(*first, f64::max);
        }
        let n = self.state.distribution.values.len() as f64; 
        let mean = self.state.distribution.values.iter().sum::<f64>() / n.max(1.0);
        let variance = self.state.distribution.values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n.max(1.0);
        self.state.distribution.mean = mean;
        self.state.distribution.variance = variance;
        self.state.distribution.min = self.state.distribution.values.iter().copied().fold(f64::INFINITY, f64::min);
        self.state.distribution.max = self.state.distribution.values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    }

    fn render_tab_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            for tab in [
                ActiveTab::Primitives,
                ActiveTab::Ranges,
                ActiveTab::Distributions,
                ActiveTab::Sequences,
                ActiveTab::Engines,
                ActiveTab::System,
            ] {
                let label = match tab {
                    ActiveTab::Primitives => "Primitives",
                    ActiveTab::Ranges => "Ranges",
                    ActiveTab::Distributions => "Distributions",
                    ActiveTab::Sequences => "Sequences",
                    ActiveTab::Engines => "RNG Engines",
                    ActiveTab::System => "System",
                };
                let selected = self.active_tab == tab;
                let button = egui::Button::new(label).selected(selected).fill(if selected {
                    ui.visuals().selection.bg_fill
                } else {
                    ui.visuals().widgets.inactive.bg_fill
                });
                if ui.add(button).clicked() {
                    self.active_tab = tab;
                }
            }
        });
    }

    fn render_primitives(&mut self, ui: &mut egui::Ui) {
        ui.heading("Primitive generation");
        self.render_code_example(ui);

        ui.horizontal(|ui| {
            ui.label("Sample count");
            ui.add(Slider::new(&mut self.state.primitive_count, 1..=2000).logarithmic(true));
            ui.label("Update interval (ms)");
            ui.add(Slider::new(&mut self.state.primitive_stream_ms, 10..=2000));
            if ui.button("Regenerate").clicked() {
                self.rebuild_primitive_samples();
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Coin flip").clicked() {
                let result = rand::random();
                self.state.primitive_samples.bools.push(result);
                if result {
                    self.state.primitive_samples.coin_heads += 1;
                } else {
                    self.state.primitive_samples.coin_tails += 1;
                }
            }
            if ui.button("Dice roll").clicked() {
                let roll = (rand::random::<u8>() % 6) as usize;
                self.state.primitive_samples.u8.push((roll + 1) as u8);
                self.state.primitive_samples.dice_counts[roll] += 1;
            }
            if ui.button("Random arrows").clicked() {
                self.state.primitive_samples.arrows = Self::random_arrow_sequence(&mut rand::rng());
            }
            ui.checkbox(&mut self.state.primitive_stream_enabled, "Live stream")
                .on_hover_text("Regenerate the primitive sample at the selected interval.");
            if ui.button("Inspect sample").clicked() {
                self.show_sample_window = true;
            }
        });

        let coin_total = self.state.primitive_samples.coin_heads + self.state.primitive_samples.coin_tails;
        ui.horizontal(|ui| {
            ui.label(format!("Heads: {}", self.state.primitive_samples.coin_heads));
            ui.label(format!("Tails: {}", self.state.primitive_samples.coin_tails));
            ui.label(format!("Coin total: {coin_total}"));
            ui.label(format!("Dice: {:?}", self.state.primitive_samples.dice_counts));
        });
        ui.label(format!("Random direction sequence: {}", self.state.primitive_samples.arrows));

        ui.separator();
        ui.group(|ui| {
            egui::ScrollArea::both()
                .max_height(360.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Grid::new("primitive-grid").show(ui, |ui| {
                        ui.label("u8"); ui.monospace(format!("{:?}", &self.state.primitive_samples.u8[..self.state.primitive_count.min(self.state.primitive_samples.u8.len())])); ui.end_row();
                        ui.label("i32"); ui.monospace(format!("{:?}", &self.state.primitive_samples.i32[..self.state.primitive_count.min(self.state.primitive_samples.i32.len())])); ui.end_row();
                        ui.label("u64"); ui.monospace(format!("{:?}", &self.state.primitive_samples.u64[..self.state.primitive_count.min(self.state.primitive_samples.u64.len())])); ui.end_row();
                        ui.label("f64"); ui.monospace(format!("{:?}", &self.state.primitive_samples.f64[..self.state.primitive_count.min(self.state.primitive_samples.f64.len())])); ui.end_row();
                        ui.label("bool"); ui.monospace(format!("{:?}", &self.state.primitive_samples.bools[..self.state.primitive_count.min(self.state.primitive_samples.bools.len())])); ui.end_row();
                        ui.label("char"); ui.monospace(Self::format_char_sample(&self.state.primitive_samples.chars[..self.state.primitive_count.min(self.state.primitive_samples.chars.len())])); ui.end_row();
                    });
                });
        });
    }

    fn render_ranges(&mut self, ui: &mut egui::Ui) {
        ui.heading("Range exploration");
        self.render_code_example(ui);
        ui.horizontal(|ui| {
            ui.label("Start");
            ui.add(egui::DragValue::new(&mut self.state.range_bounds.start).speed(0.1));
            ui.label("End");
            ui.add(egui::DragValue::new(&mut self.state.range_bounds.end).speed(0.1));
            ui.checkbox(&mut self.state.range_bounds.inclusive, "Inclusive ..=");
        });

        ui.horizontal(|ui| {
            ui.label("Sample count");
            ui.add(Slider::new(&mut self.state.range_bounds.sample_count, 1..=5000).logarithmic(true));
            ui.label("Update interval (ms)");
            ui.add(Slider::new(&mut self.state.range_bounds.stream_ms, 10..=2000));
            if ui.button("Generate").clicked() {
                self.rebuild_range_samples();
                self.last_range_update = Instant::now();
            }
            ui.checkbox(&mut self.state.range_bounds.live_enabled, "Live generation")
                .on_hover_text("Regenerate range values at the selected interval.");
        });

        let valid = self.state.range_bounds.start != self.state.range_bounds.end && self.state.range_bounds.start < self.state.range_bounds.end;
        if !valid {
            ui.colored_label(Color32::from_rgb(255, 120, 120), "Range is invalid: zero-width or inverted intervals are blocked.");
        }

        if valid {
            ui.label(format!("Sample: {:?}", self.state.range_bounds.values));
        } else {
            ui.label("Sample unavailable until the range is valid.");
        }
    }

    fn render_distributions(&mut self, ui: &mut egui::Ui) {
        ui.heading("Distributions");
        self.render_code_example(ui);
        ui.horizontal(|ui| {
            ComboBox::from_id_salt("dist-kind")
                .selected_text(match self.state.distribution.kind {
                    DistributionKind::Uniform => "Uniform",
                    DistributionKind::Standard => "Standard",
                    DistributionKind::Bernoulli => "Bernoulli",
                    DistributionKind::Normal => "Normal",
                    DistributionKind::Exponential => "Exponential",
                    DistributionKind::Poisson => "Poisson",
                    DistributionKind::WeightedIndex => "WeightedIndex",
                })
                .show_ui(ui, |ui| {
                    for kind in DistributionKind::all() {
                        ui.selectable_value(&mut self.state.distribution.kind, *kind, format!("{kind:?}"));
                    }
                });
            ui.label("N");
            ui.add(Slider::new(&mut self.state.distribution.sample_size, 10..=5000).logarithmic(true));
        });

        ui.horizontal(|ui| {
            ui.label("mu");
            ui.add(egui::DragValue::new(&mut self.state.distribution.mu).speed(0.1));
            ui.label("sigma");
            ui.add(egui::DragValue::new(&mut self.state.distribution.sigma).speed(0.1));
        });
        ui.horizontal(|ui| {
            ui.label("lambda");
            ui.add(egui::DragValue::new(&mut self.state.distribution.lambda).speed(0.1));
            ui.label("p");
            ui.add(egui::DragValue::new(&mut self.state.distribution.probability).speed(0.01));
        });
        ui.horizontal(|ui| {
            ui.label("weights");
            ui.text_edit_singleline(&mut self.state.distribution.weights);
        });
        if ui.button("Refresh stats").clicked() {
            self.rebuild_distribution();
        }

        ui.separator();
        let dist = &self.state.distribution;
        ui.horizontal(|ui| {
            ui.label(format!("Mean: {:.4}", dist.mean));
            ui.label(format!("Variance: {:.4}", dist.variance));
            ui.label(format!("StdDev: {:.4}", dist.variance.sqrt()));
            ui.label(format!("Min: {:.4}", dist.min));
            ui.label(format!("Max: {:.4}", dist.max));
        });

        ui.group(|ui| {
            let values = self.state.distribution.values.clone();
            let bin_count = ((values.len() as f64).sqrt() as usize).clamp(8, 64);
            let min = values.iter().copied().fold(f64::INFINITY, f64::min);
            let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let width = ((max - min) / bin_count as f64).max(f64::EPSILON);
            let mut bins = vec![0_usize; bin_count];
            for value in &values {
                let index = (((*value - min) / width) as usize).min(bin_count - 1);
                bins[index] += 1;
            }
            let bars = bins
                .iter()
                .enumerate()
                .map(|(index, count)| Bar::new(min + (index as f64 + 0.5) * width, *count as f64 / values.len().max(1) as f64 / width).width(width * 0.9))
                .collect();
            let histogram = BarChart::new("histogram", bars);
            let pdf_points: PlotPoints = (0..=200)
                .map(|index| {
                    let x = min + (max - min) * index as f64 / 200.0;
                    [x, self.distribution_pdf(x).unwrap_or(0.0)]
                })
                .collect();
            let pdf = Line::new("theoretical PDF", pdf_points).color(Color32::LIGHT_RED);
            let plot = Plot::new("distribution_plot").height(240.0).show_axes([true, true]).show_grid(true);
            plot.show(ui, |plot_ui| {
                plot_ui.bar_chart(histogram);
                plot_ui.line(pdf);
            });

            ui.horizontal(|ui| {
                if ui.button("Open Plotly graph").clicked() {
                    self.open_plotly_distribution();
                }
            });
        });
    }

    fn render_sequences(&mut self, ui: &mut egui::Ui) {
        ui.heading("Sequences and iterators");
        self.render_code_example(ui);
        ui.horizontal(|ui| {
            ui.label("Choose count");
            ui.add(Slider::new(&mut self.state.sequence.choose_count, 1..=8));
            ui.label("Password length");
            ui.add(Slider::new(&mut self.state.sequence.password_len, 4..=256));
        });
        ui.horizontal(|ui| {
            ui.label("Alphabet");
            ui.text_edit_singleline(&mut self.state.sequence.alphabet);
        });
        ui.horizontal(|ui| {
            if ui.button("Choose one").clicked() {
                let mut rng = rand::rng();
                self.state.sequence.chosen_item = self.state.sequence.items.choose(&mut rng).cloned();
            }
            if ui.button("Choose multiple").clicked() {
                let mut rng = rand::rng();
                self.state.sequence.chosen_items = self
                    .state
                    .sequence
                    .items
                    .sample(&mut rng, self.state.sequence.choose_count)
                    .cloned()
                    .collect();
            }
            if ui.button("Choose weighted").clicked() {
                let mut rng = rand::rng();
                let weights: Vec<f64> = (0..self.state.sequence.items.len())
                    .map(|index| (index + 1) as f64)
                    .collect();
                if let Ok(dist) = WeightedIndex::new(weights) {
                    self.state.sequence.chosen_item = self
                        .state
                        .sequence
                        .items
                        .get(dist.sample(&mut rng))
                        .cloned();
                }
            }
            if ui.button("Shuffle").clicked() {
                let mut items = self.state.sequence.items.clone();
                let mut rng = rand::rng();
                items.shuffle(&mut rng);
                self.state.sequence.items = items;
                self.state.sequence.shuffle_step += 1;
            }
        });

        ui.horizontal(|ui| {
            if ui.button("Generate password").clicked() {
                let mut rng = rand::rng();
                let alphabet: Vec<char> = self.state.sequence.alphabet.chars().collect();
                self.state.sequence.generated_password = if alphabet.is_empty() {
                    String::new()
                } else {
                    (0..self.state.sequence.password_len)
                        .map(|_| alphabet[rng.random_range(0..alphabet.len())])
                        .collect()
                };
            }
            if let Some(item) = &self.state.sequence.chosen_item {
                ui.label(format!("Chosen: {item}"));
            }
            if !self.state.sequence.chosen_items.is_empty() {
                ui.label(format!("Multiple: {:?}", self.state.sequence.chosen_items));
            }
        });

        ui.label(format!("Current shuffle step: {}", self.state.sequence.shuffle_step));
        ui.horizontal_wrapped(|ui| {
            for item in &self.state.sequence.items {
                ui.colored_label(Color32::LIGHT_BLUE, item);
            }
        });

        ui.label(format!("Generated password: {}", self.state.sequence.generated_password));
    }

    fn render_engines(&mut self, ui: &mut egui::Ui) {
        ui.heading("RNG engines and determinism");
        self.render_code_example(ui);
        ui.horizontal(|ui| {
            ComboBox::from_id_salt("rng-kind")
                .selected_text(format!("{:?}", self.state.engine))
                .show_ui(ui, |ui| {
                    for kind in EngineKind::all() {
                        ui.selectable_value(&mut self.state.engine, *kind, format!("{kind:?}"));
                    }
                });
            if ui.button("Random seed").clicked() {
                self.state.seed = generate_seed();
            }
            if ui.button("Apply seed").clicked() {
                self.state.benchmark.seed = self.state.seed;
            }
        });

        ui.horizontal(|ui| {
            ui.label("Seed");
            ui.add(egui::DragValue::new(&mut self.state.seed).speed(1.0));
        });

        if ui.button("Benchmark").clicked() {
            let start = Instant::now();
            let mut rng = reseed_for_kind(self.state.engine, self.state.seed);
            let count = self.state.benchmark.total_values;
            let mut total = 0_u64;
            for _ in 0..count {
                total = total.wrapping_add(rng.next_u64());
            }
            let elapsed = start.elapsed();
            let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
            let bytes = (count as f64) * std::mem::size_of::<u64>() as f64;
            self.state.benchmark.elapsed_ms = elapsed_ms;
            self.state.benchmark.throughput_mbps = (bytes / 1_048_576.0) / (elapsed_ms / 1000.0).max(0.0001);
            self.state.benchmark.sample = (0..32).map(|_| rng.next_u64()).collect();
            self.state.benchmark.selected = self.state.engine;
            let mut stream_a_rng = reseed_for_kind(self.state.engine, self.state.seed);
            let mut stream_b_rng = reseed_for_kind(self.state.engine, self.state.seed);
            self.state.benchmark.stream_a = (0..32).map(|_| stream_a_rng.next_u64()).collect();
            self.state.benchmark.stream_b = (0..32).map(|_| stream_b_rng.next_u64()).collect();
            self.state.benchmark.streams_match =
                self.state.benchmark.stream_a == self.state.benchmark.stream_b;
            let _ = total;
        }

        ui.horizontal(|ui| {
            ui.label("Total values");
            ui.add(Slider::new(&mut self.state.benchmark.total_values, 100000..=10_000_000).logarithmic(true));
        });
        ui.horizontal(|ui| {
            ui.label(format!("Throughput: {:.2} MB/s", self.state.benchmark.throughput_mbps));
            ui.label(format!("Elapsed: {:.2} ms", self.state.benchmark.elapsed_ms));
        });
        ui.label(format!("Sample: {:?}", self.state.benchmark.sample));
        ui.label(format!(
            "Determinism: {}",
            if self.state.benchmark.streams_match { "streams match" } else { "not checked" }
        ));
        if !self.state.benchmark.stream_a.is_empty() {
            ui.collapsing("Compare streams", |ui| {
                egui::Grid::new("determinism-grid").show(ui, |ui| {
                    ui.label("A");
                    ui.label("B");
                    ui.end_row();
                    for (left, right) in self.state.benchmark.stream_a.iter().zip(&self.state.benchmark.stream_b) {
                        ui.monospace(left.to_string());
                        ui.monospace(right.to_string());
                        ui.end_row();
                    }
                });
            });
        }
    }

    fn render_system(&mut self, ui: &mut egui::Ui) {
        ui.heading("System and build info");
        self.render_code_example(ui);
        ui.horizontal(|ui| {
            if ui.button("Save preset").clicked() {
                self.save_preset();
            }
            if ui.button("Load preset").clicked() {
                self.load_preset();
            }
            if ui.button("Export distribution CSV").clicked() {
                self.export_distribution();
            }
        });
        if !self.status_message.is_empty() {
            ui.label(&self.status_message);
        }
        ui.group(|ui| {
            ui.label(format!("Git SHA: {}", self.build_info.git_sha));
            ui.label(format!("Branch: {}", self.build_info.git_branch));
            ui.label(format!("Timestamp: {}", self.build_info.git_timestamp));
            ui.label(format!("Rust: {}", self.build_info.rustc_version));
            ui.label(format!("Target: {}", self.build_info.target_triple));
            ui.label(format!("Profile: {}", self.build_info.cargo_profile));
            ui.label(format!("Package: {}", self.build_info.package_version));
        });

        ui.separator();
        ui.label("Active renderer: Glow (OpenGL) on Windows, when running with eframe::Renderer::Glow.");
        ui.label("Dependency bundle: eframe, egui, egui_plot, plotly, rand, rand_distr, rand_chacha, rand_pcg, serde.");

        ui.separator();
        ui.heading("Build dependency versions");
        const MAIN_DEPENDENCIES: &[&str] = &[
            "eframe",
            "egui",
            "egui_plot",
            "plotly",
            "rand",
            "rand_core",
            "rand_distr",
            "rand_chacha",
            "rand_pcg",
            "serde",
            "serde_json",
        ];
        for dependency in MAIN_DEPENDENCIES {
            if let Some(version) = self
                .build_info
                .dependency_versions
                .iter()
                .find_map(|entry| {
                    let (name, version) = entry.split_once('=')?;
                    (name == *dependency).then_some(version)
                })
            {
                ui.monospace(format!("{dependency}={version}"));
            }
        }
    }
}

impl EguiApp for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.active_tab == ActiveTab::Primitives
            && self.state.primitive_stream_enabled
            && self.last_primitive_update.elapsed().as_millis()
                >= self.state.primitive_stream_ms as u128
        {
            self.rebuild_primitive_samples();
            self.last_primitive_update = Instant::now();
            ui.ctx().request_repaint();
        }
        if self.active_tab == ActiveTab::Ranges
            && self.state.range_bounds.live_enabled
            && self.last_range_update.elapsed().as_millis() >= self.state.range_bounds.stream_ms as u128
        {
            self.rebuild_range_samples();
            self.last_range_update = Instant::now();
            ui.ctx().request_repaint();
        }
        ui.ctx().set_visuals(if self.state.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });

        Panel::top("topbar")
            .frame(Frame::side_top_panel(&ui.style()))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("rand_explorer");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button(if self.state.dark_mode { "Light theme" } else { "Dark theme" }).clicked() {
                            self.state.dark_mode = !self.state.dark_mode;
                        }
                        ui.label("seed");
                        ui.add(egui::DragValue::new(&mut self.state.seed).speed(1.0));
                    });
                });
            });

        Panel::left("sidebar")
            .frame(Frame::side_top_panel(&ui.style()))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.heading("Global RNG controls");
                        ui.label("Engine");
                        ComboBox::from_id_salt("engine_selector")
                            .selected_text(format!("{:?}", self.state.engine))
                            .show_ui(ui, |ui| {
                                for kind in EngineKind::all() {
                                    ui.selectable_value(&mut self.state.engine, *kind, format!("{kind:?}"));
                                }
                            });
                        ui.label("Seed");
                        ui.add(egui::DragValue::new(&mut self.state.seed).speed(1.0));
                        if ui.button("Re-seed").clicked() {
                            self.state.seed = generate_seed();
                        }
                        ui.separator();
                        ui.label("Status");
                        ui.colored_label(Color32::from_rgb(70, 200, 120), "Glow OpenGL ready");
                        ui.label(format!("Profile: {}", self.build_info.cargo_profile));
                        ui.label(format!("Target: {}", self.build_info.target_triple));
                    });
            });

        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    self.render_tab_buttons(ui);
                    ui.separator();
                    match self.active_tab {
                        ActiveTab::Primitives => self.render_primitives(ui),
                        ActiveTab::Ranges => self.render_ranges(ui),
                        ActiveTab::Distributions => self.render_distributions(ui),
                        ActiveTab::Sequences => self.render_sequences(ui),
                        ActiveTab::Engines => self.render_engines(ui),
                        ActiveTab::System => self.render_system(ui),
                    }
                });
        });

        Panel::bottom("status")
            .frame(Frame::side_top_panel(&ui.style()))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Glow OpenGL").color(Color32::LIGHT_GREEN));
                    ui.label(format!("Seed: {}", self.state.seed));
                    ui.label(format!("Engine: {:?}", self.state.engine));
                });
            });

        if self.show_sample_window {
            Window::new("Sample inspection").show(ui.ctx(), |ui| {
                ui.label(format!("Current primitive sample: {:?}", self.state.primitive_samples.u8));
            });
        }
    }
}
