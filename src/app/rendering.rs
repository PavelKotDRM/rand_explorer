use std::time::Instant;

use eframe::egui;
use egui::{Color32, ComboBox, Frame, Slider};
use egui_plot::{Bar, BarChart, Line, Plot, PlotPoints};
use rand::distr::Distribution;
use rand::prelude::{IndexedRandom, SliceRandom};
use rand::RngExt;

use super::{actions::random_arrow_sequence, examples::code_example, ActiveTab, App};

impl App {
    fn adaptive_height(ui: &egui::Ui, fraction: f32, min: f32, max: f32) -> f32 {
        (ui.available_height() * fraction).clamp(min, max)
    }

    pub(super) fn render_tab_buttons(&mut self, ui: &mut egui::Ui) {
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

    pub(super) fn render_code_example(&self, ui: &mut egui::Ui) {
        let (title, source) = code_example(self.active_tab);

        ui.collapsing(format!("Rust example: {title}"), |ui| {
            ui.horizontal(|ui| {
                if ui.button("Copy").clicked() {
                    ui.ctx().copy_text(source.to_owned());
                }
                ui.label("Source code");
            });

            let frame = Frame::group(ui.style()).inner_margin(egui::Margin::same(8));
            frame.show(ui, |ui| {
                egui::ScrollArea::both()
                    .max_height(Self::adaptive_height(ui, 0.28, 180.0, 360.0))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut source.to_owned())
                                .font(egui::TextStyle::Monospace)
                                .code_editor()
                                .desired_width(f32::INFINITY),
                        );
                    });
            });
        });
    }

    pub(super) fn render_primitives(&mut self, ui: &mut egui::Ui) {
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
                self.state.primitive_samples.arrows = random_arrow_sequence(&mut rand::rng());
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
                .max_height(Self::adaptive_height(ui, 0.48, 220.0, 560.0))
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

    fn format_char_sample(values: &[char]) -> String {
        values.iter().collect()
    }

    pub(super) fn render_ranges(&mut self, ui: &mut egui::Ui) {
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

    pub(super) fn render_distributions(&mut self, ui: &mut egui::Ui) {
        ui.heading("Distributions");
        self.render_code_example(ui);
        ui.horizontal(|ui| {
            ComboBox::from_id_salt("dist-kind")
                .selected_text(match self.state.distribution.kind {
                    crate::state::DistributionKind::Uniform => "Uniform",
                    crate::state::DistributionKind::Standard => "Standard",
                    crate::state::DistributionKind::Bernoulli => "Bernoulli",
                    crate::state::DistributionKind::Normal => "Normal",
                    crate::state::DistributionKind::Exponential => "Exponential",
                    crate::state::DistributionKind::Poisson => "Poisson",
                    crate::state::DistributionKind::WeightedIndex => "WeightedIndex",
                })
                .show_ui(ui, |ui| {
                    for kind in crate::state::DistributionKind::all() {
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
            let plot_height = Self::adaptive_height(ui, 0.58, 220.0, 520.0);
            let plot = Plot::new("distribution_plot")
                .height(plot_height)
                .show_axes([true, true])
                .show_grid(true)
                .legend(egui_plot::Legend::default());
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

    pub(super) fn render_sequences(&mut self, ui: &mut egui::Ui) {
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
                if let Ok(dist) = rand::distr::weighted::WeightedIndex::new(weights) {
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

    pub(super) fn render_engines(&mut self, ui: &mut egui::Ui) {
        ui.heading("RNG engines and determinism");
        self.render_code_example(ui);
        ui.horizontal(|ui| {
            ComboBox::from_id_salt("rng-kind")
                .selected_text(format!("{:?}", self.state.engine))
                .show_ui(ui, |ui| {
                    for kind in crate::state::EngineKind::all() {
                        ui.selectable_value(&mut self.state.engine, *kind, format!("{kind:?}"));
                    }
                });
            if ui.button("Random seed").clicked() {
                self.state.seed = crate::state::generate_seed();
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
            let mut rng = crate::state::reseed_for_kind(self.state.engine, self.state.seed);
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
            let mut stream_a_rng = crate::state::reseed_for_kind(self.state.engine, self.state.seed);
            let mut stream_b_rng = crate::state::reseed_for_kind(self.state.engine, self.state.seed);
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
                egui::ScrollArea::vertical()
                    .max_height(Self::adaptive_height(ui, 0.42, 180.0, 480.0))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
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
            });
        }
    }

    pub(super) fn render_system(&mut self, ui: &mut egui::Ui) {
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
