//! The eframe application wrapper.

use crate::command::{self, Command};
use crate::config::Options;
use crate::geoip;
use crate::geometry;
use crate::state::State;
use crate::theme;
use crate::ui;

/// Storage key of the theme setting.
const THEME_KEY: &str = "theme";

/// Storage key of the window scale.
const ZOOM_KEY: &str = "zoom";

/// The smallest and largest window scale.
const ZOOM_RANGE: std::ops::RangeInclusive<f32> = 0.5..=3.0;

/// One zoom step, as a factor.
const ZOOM_STEP: f32 = 1.1;

/// The trippy-ui window.
pub struct TrippyApp {
    state: State,
    /// For saving settings, which happens outside a frame.
    ctx: egui::Context,
    /// Saves the window's size and position.
    window: geometry::Tracker,
    /// The desktop's light/dark preference.
    #[cfg(target_os = "linux")]
    system_theme: crate::portal::SystemTheme,
}

impl TrippyApp {
    /// Builds the window from the parsed command line.
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        options: Options,
        geoip: geoip::Lookup,
        window: geometry::Tracker,
    ) -> Self {
        let settings = cc
            .storage
            .and_then(|storage| storage.get_string(THEME_KEY))
            .map(|stored| theme::Settings::from_stored(&stored))
            .unwrap_or_default();
        if let Some(zoom) = cc
            .storage
            .and_then(|storage| storage.get_string(ZOOM_KEY))
            .and_then(|stored| stored.parse::<f32>().ok())
            .filter(|zoom| ZOOM_RANGE.contains(zoom))
        {
            cc.egui_ctx.set_zoom_factor(zoom);
        }
        let mut options = options;
        options.settings.theme = settings;
        // Validated at startup, so this cannot fail.
        let _ = options.settings.theme.apply_items(&options.theme_colors);
        theme::install(&cc.egui_ctx, &options.settings.theme);
        Self {
            state: State::new(options, geoip),
            ctx: cc.egui_ctx.clone(),
            window,
            #[cfg(target_os = "linux")]
            system_theme: crate::portal::SystemTheme::watch(cc.egui_ctx.clone()),
        }
    }

    /// Keeps the installed theme in sync with the settings.
    fn sync_theme(&self) {
        if theme::current(&self.ctx) != self.state.settings.theme {
            theme::install(&self.ctx, &self.state.settings.theme);
        }
    }

    /// Runs a command from a key press or a clicked key hint.
    fn run(&mut self, command: Command) {
        match command {
            Command::UiZoom(step) => self.zoom(match step {
                0 => None,
                s if s > 0 => Some(ZOOM_STEP),
                _ => Some(1.0 / ZOOM_STEP),
            }),
            command => self.state.run_command(command, &self.ctx),
        }
    }

    /// Scales the window by `factor`, or back to normal.
    fn zoom(&self, factor: Option<f32>) {
        let zoom = match factor {
            Some(factor) => {
                (self.ctx.zoom_factor() * factor).clamp(*ZOOM_RANGE.start(), *ZOOM_RANGE.end())
            }
            None => 1.0,
        };
        self.ctx.set_zoom_factor(zoom);
    }
}

impl eframe::App for TrippyApp {
    /// Remembers the theme, the window scale and the window's geometry.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string(THEME_KEY, self.state.settings.theme.to_stored());
        storage.set_string(ZOOM_KEY, self.ctx.zoom_factor().to_string());
        self.window.save(&self.ctx);
    }

    /// Supplies the system theme where winit does not know it.
    #[cfg(target_os = "linux")]
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        if raw_input.system_theme.is_none() {
            raw_input.system_theme = self.system_theme.get();
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.window.observe(&ctx);
        self.state.dark_mode = ui.visuals().dark_mode;
        self.state.poll();
        self.sync_theme();
        // Ctrl+wheel (and pinching) scales the window.
        let wheel_zoom = ctx.input(|i| i.zoom_delta());
        if (wheel_zoom - 1.0).abs() > f32::EPSILON {
            self.zoom(Some(wheel_zoom));
        }
        // An open menu or combo box takes Escape to close itself; it must
        // not also clear the selection or close the dialog behind it.
        if egui::Popup::is_any_open(&ctx)
            && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            egui::Popup::close_all(&ctx);
        }
        let mut keys = command::Keys::take(&ctx);
        while let Some(command) = keys.next(self.state.prompt.is_some(), &self.state.bindings) {
            self.run(command);
        }
        if !self.state.running {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        if let Some(command) = ui::render(&mut self.state, ui) {
            self.run(command);
            ctx.request_repaint();
        }
        ctx.request_repaint_after(self.state.refresh_rate);
    }
}
