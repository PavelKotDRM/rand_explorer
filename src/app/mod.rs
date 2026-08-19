mod actions;
mod examples;
mod layout;
mod rendering;

use std::time::Instant;

use eframe::{egui, App as EguiApp};

use crate::build_info::BuildInfo;
use crate::state::AppState;

pub use self::examples::ActiveTab;

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
            egui::FontData::from_static(include_bytes!("../../assets/fonts/NotoSans-Regular.ttf")).into(),
        );
        fonts.font_data.insert(
            "noto_symbols".to_owned(),
            egui::FontData::from_static(include_bytes!("../../assets/fonts/NotoSansSymbols2-Regular.ttf")).into(),
        );
        fonts.font_data.insert(
            "unifont".to_owned(),
            egui::FontData::from_static(include_bytes!("../../assets/fonts/Unifont-17.0.05.otf")).into(),
        );
        fonts.font_data.insert(
            "noto_hebrew".to_owned(),
            egui::FontData::from_static(include_bytes!("../../assets/fonts/NotoSansHebrew-Regular.ttf")).into(),
        );
        fonts.font_data.insert(
            "noto_arabic".to_owned(),
            egui::FontData::from_static(include_bytes!("../../assets/fonts/NotoSansArabic-Regular.ttf")).into(),
        );
        fonts.font_data.insert(
            "noto_cjk".to_owned(),
            egui::FontData::from_static(include_bytes!("../../assets/fonts/NotoSansCJKsc-Regular.otf")).into(),
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

    fn update_live_state(&mut self, ui: &mut egui::Ui) {
        let primitive_due = self.active_tab == ActiveTab::Primitives
            && self.state.primitive_stream_enabled
            && self.last_primitive_update.elapsed().as_millis() >= self.state.primitive_stream_ms as u128;
        if primitive_due {
            self.rebuild_primitive_samples();
            self.last_primitive_update = Instant::now();
            ui.ctx().request_repaint();
        }

        let range_due = self.active_tab == ActiveTab::Ranges
            && self.state.range_bounds.live_enabled
            && self.last_range_update.elapsed().as_millis() >= self.state.range_bounds.stream_ms as u128;
        if range_due {
            self.rebuild_range_samples();
            self.last_range_update = Instant::now();
            ui.ctx().request_repaint();
        }
    }
}

impl EguiApp for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.update_live_state(ui);
        ui.ctx().set_visuals(if self.state.dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });

        self.render_topbar(ui);
        self.render_sidebar(ui);

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

        self.render_status_bar(ui);
        self.render_sample_window(ui);
    }
}
