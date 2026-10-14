//! The eframe application wrapper.

use crate::command::{self, Command};
use crate::config::Options;
use crate::geoip;
use crate::state::State;
use crate::theme;
use crate::ui;

/// Storage key of the theme setting.
const THEME_KEY: &str = "theme";

/// The trippy-ui window.
pub struct TrippyApp {
    state: State,
    /// For saving settings, which happens outside a frame.
    ctx: egui::Context,
}

impl TrippyApp {
    /// Builds the window from the parsed command line.
    pub fn new(cc: &eframe::CreationContext<'_>, options: Options, geoip: geoip::Lookup) -> Self {
        let settings = cc
            .storage
            .and_then(|storage| storage.get_string(THEME_KEY))
            .map(|stored| theme::Settings::from_stored(&stored))
            .unwrap_or_default();
        let mut options = options;
        options.settings.theme = settings;
        // Validated at startup, so this cannot fail.
        let _ = options.settings.theme.apply_items(&options.theme_colors);
        theme::install(&cc.egui_ctx, &options.settings.theme);
        Self {
            state: State::new(options, geoip),
            ctx: cc.egui_ctx.clone(),
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
        self.state.run_command(command, &self.ctx);
    }
}

impl eframe::App for TrippyApp {
    /// Remembers the theme between runs.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string(THEME_KEY, self.state.settings.theme.to_stored());
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.state.poll();
        self.sync_theme();
        // An open modal takes Escape to close itself; it must not also clear the
        // selection. egui routes Escape to the modal first, so nothing to do here.
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
