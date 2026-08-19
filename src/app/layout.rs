use eframe::egui;
use egui::{Align, Color32, ComboBox, Frame, Layout, Panel, RichText, Window};

use super::App;

impl App {
    pub(super) fn render_topbar(&mut self, ui: &mut egui::Ui) {
        Panel::top("topbar")
            .frame(Frame::side_top_panel(&ui.style()))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("rand_explorer");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .button(if self.state.dark_mode { "Light theme" } else { "Dark theme" })
                            .clicked()
                        {
                            self.state.dark_mode = !self.state.dark_mode;
                        }
                        ui.label("seed");
                        ui.add(egui::DragValue::new(&mut self.state.seed).speed(1.0));
                    });
                });
            });
    }

    pub(super) fn render_sidebar(&mut self, ui: &mut egui::Ui) {
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
                                for kind in crate::state::EngineKind::all() {
                                    ui.selectable_value(&mut self.state.engine, *kind, format!("{kind:?}"));
                                }
                            });
                        ui.label("Seed");
                        ui.add(egui::DragValue::new(&mut self.state.seed).speed(1.0));
                        if ui.button("Re-seed").clicked() {
                            self.state.seed = crate::state::generate_seed();
                        }
                        ui.separator();
                        ui.label("Status");
                        ui.colored_label(Color32::from_rgb(70, 200, 120), "Glow OpenGL ready");
                        ui.label(format!("Profile: {}", self.build_info.cargo_profile));
                        ui.label(format!("Target: {}", self.build_info.target_triple));
                    });
            });
    }

    pub(super) fn render_status_bar(&self, ui: &mut egui::Ui) {
        Panel::bottom("status")
            .frame(Frame::side_top_panel(&ui.style()))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Glow OpenGL").color(Color32::LIGHT_GREEN));
                    ui.label(format!("Seed: {}", self.state.seed));
                    ui.label(format!("Engine: {:?}", self.state.engine));
                });
            });
    }

    pub(super) fn render_sample_window(&self, ui: &mut egui::Ui) {
        if self.show_sample_window {
            Window::new("Sample inspection").show(ui.ctx(), |ui| {
                ui.label(format!("Current primitive sample: {:?}", self.state.primitive_samples.u8));
            });
        }
    }
}
