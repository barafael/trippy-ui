//! High-contrast palettes and the flat, dense style.
//!
//! The terminal frontend exposes a named colour for every element. We keep the
//! same idea but derive the element colours from a small base palette, so a
//! theme is a handful of colours rather than three dozen, while still allowing
//! every slot to be overridden from the settings dialog.

use egui::{Color32, CornerRadius, FontId, Shadow, Stroke, Style, TextStyle, ThemePreference, Ui};
use std::collections::BTreeMap;

/// A base palette from which the element colours are derived.
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    /// Window background.
    pub bg: Color32,
    /// Panels, popups and bars.
    pub surface: Color32,
    /// Odd rows and subtle fills.
    pub stripe: Color32,
    /// Separators and outlines.
    pub border: Color32,
    /// Body text.
    pub text: Color32,
    /// Emphasized text.
    pub strong: Color32,
    /// Secondary text.
    pub weak: Color32,
    /// Selection and focus.
    pub accent: Color32,
    /// Text on top of `accent`.
    pub on_accent: Color32,
    /// Field labels and table headings.
    pub label: Color32,
    /// Success.
    pub green: Color32,
    /// Errors.
    pub red: Color32,
    /// Warnings and highlights.
    pub yellow: Color32,
    /// Information.
    pub blue: Color32,
    /// Control and extension data.
    pub magenta: Color32,
    /// Muted data.
    pub dim: Color32,
}

const fn hex(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

const DARK: Palette = Palette {
    bg: hex(0x0e1117),
    surface: hex(0x1a1f29),
    stripe: hex(0x151a22),
    border: hex(0x3b4455),
    text: hex(0xe6edf3),
    strong: hex(0xffffff),
    weak: hex(0xa9b4c4),
    accent: hex(0xf8be75),
    on_accent: hex(0x15100a),
    label: hex(0x6fd3e8),
    green: hex(0x7ee787),
    red: hex(0xff7b72),
    yellow: hex(0xf5d76e),
    blue: hex(0x79b8ff),
    magenta: hex(0xe2a5ff),
    dim: hex(0x7d8798),
};

const LIGHT: Palette = Palette {
    bg: hex(0xffffff),
    surface: hex(0xeef1f5),
    stripe: hex(0xf5f7fa),
    border: hex(0xb9c2cf),
    text: hex(0x1a1e24),
    strong: hex(0x000000),
    weak: hex(0x4a5361),
    accent: hex(0x9c4a00),
    on_accent: hex(0xffffff),
    label: hex(0x075f74),
    green: hex(0x176b30),
    red: hex(0xb3261e),
    yellow: hex(0x7a5600),
    blue: hex(0x0b4fae),
    magenta: hex(0x7a35c9),
    dim: hex(0x6a7380),
};

const GRUVBOX_DARK: Palette = Palette {
    bg: hex(0x1d2021),
    surface: hex(0x32302f),
    stripe: hex(0x282828),
    border: hex(0x665c54),
    text: hex(0xebdbb2),
    strong: hex(0xfbf1c7),
    weak: hex(0xbdae93),
    accent: hex(0xfe8019),
    on_accent: hex(0x1d2021),
    label: hex(0x8ec07c),
    green: hex(0xb8bb26),
    red: hex(0xfb4934),
    yellow: hex(0xfabd2f),
    blue: hex(0x83a598),
    magenta: hex(0xd3869b),
    dim: hex(0x928374),
};

const GRUVBOX_LIGHT: Palette = Palette {
    bg: hex(0xfbf1c7),
    surface: hex(0xebdbb2),
    stripe: hex(0xf2e5bc),
    border: hex(0xbdae93),
    text: hex(0x3c3836),
    strong: hex(0x1d2021),
    weak: hex(0x665c54),
    accent: hex(0xaf3a03),
    on_accent: hex(0xfbf1c7),
    label: hex(0x427b58),
    green: hex(0x79740e),
    red: hex(0x9d0006),
    yellow: hex(0xb57614),
    blue: hex(0x076678),
    magenta: hex(0x8f3f71),
    dim: hex(0x928374),
};

const SOLARIZED_DARK: Palette = Palette {
    bg: hex(0x002b36),
    surface: hex(0x073642),
    stripe: hex(0x01303c),
    border: hex(0x586e75),
    text: hex(0x93a1a1),
    strong: hex(0xeee8d5),
    weak: hex(0x839496),
    accent: hex(0xb58900),
    on_accent: hex(0x002b36),
    label: hex(0x2aa198),
    green: hex(0x859900),
    red: hex(0xdc322f),
    yellow: hex(0xcb4b16),
    blue: hex(0x268bd2),
    magenta: hex(0xd33682),
    dim: hex(0x657b83),
};

const SOLARIZED_LIGHT: Palette = Palette {
    bg: hex(0xfdf6e3),
    surface: hex(0xeee8d5),
    stripe: hex(0xf6efdb),
    border: hex(0x93a1a1),
    text: hex(0x46585f),
    strong: hex(0x002b36),
    weak: hex(0x586e75),
    accent: hex(0xb3410f),
    on_accent: hex(0xfdf6e3),
    label: hex(0x1a7d76),
    green: hex(0x5f6e00),
    red: hex(0xc4231f),
    yellow: hex(0x8a6800),
    blue: hex(0x1f6fa8),
    magenta: hex(0xb52a6c),
    dim: hex(0x93a1a1),
};

const CATPPUCCIN_DARK: Palette = Palette {
    bg: hex(0x1e1e2e),
    surface: hex(0x313244),
    stripe: hex(0x24243a),
    border: hex(0x585b70),
    text: hex(0xcdd6f4),
    strong: hex(0xffffff),
    weak: hex(0xa6adc8),
    accent: hex(0xfab387),
    on_accent: hex(0x1e1e2e),
    label: hex(0x89dceb),
    green: hex(0xa6e3a1),
    red: hex(0xf38ba8),
    yellow: hex(0xf9e2af),
    blue: hex(0x89b4fa),
    magenta: hex(0xcba6f7),
    dim: hex(0x7f849c),
};

const CATPPUCCIN_LIGHT: Palette = Palette {
    bg: hex(0xeff1f5),
    surface: hex(0xdce0e8),
    stripe: hex(0xe6e9ef),
    border: hex(0xacb0be),
    text: hex(0x4c4f69),
    strong: hex(0x11111b),
    weak: hex(0x5c5f77),
    accent: hex(0xc94f0c),
    on_accent: hex(0xffffff),
    label: hex(0x1a7f8a),
    green: hex(0x2f7d1f),
    red: hex(0xd20f39),
    yellow: hex(0xa86a10),
    blue: hex(0x1e66f5),
    magenta: hex(0x8839ef),
    dim: hex(0x8c8fa1),
};

/// A palette family with a dark and a light variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Family {
    /// trippy-ui's own high-contrast palettes.
    #[default]
    Default,
    /// Gruvbox.
    Gruvbox,
    /// Solarized.
    Solarized,
    /// Catppuccin (Mocha and Latte).
    Catppuccin,
}

impl Family {
    /// All families, in menu order.
    pub const ALL: [Family; 4] = [
        Family::Default,
        Family::Gruvbox,
        Family::Solarized,
        Family::Catppuccin,
    ];

    /// Name in the settings.
    pub const fn name(self) -> &'static str {
        match self {
            Family::Default => "Default",
            Family::Gruvbox => "Gruvbox",
            Family::Solarized => "Solarized",
            Family::Catppuccin => "Catppuccin",
        }
    }

    /// The dark or light variant.
    pub const fn palette(self, dark: bool) -> &'static Palette {
        match (self, dark) {
            (Family::Default, true) => &DARK,
            (Family::Default, false) => &LIGHT,
            (Family::Gruvbox, true) => &GRUVBOX_DARK,
            (Family::Gruvbox, false) => &GRUVBOX_LIGHT,
            (Family::Solarized, true) => &SOLARIZED_DARK,
            (Family::Solarized, false) => &SOLARIZED_LIGHT,
            (Family::Catppuccin, true) => &CATPPUCCIN_DARK,
            (Family::Catppuccin, false) => &CATPPUCCIN_LIGHT,
        }
    }
}

macro_rules! define_slots {
    ($($field:ident => $name:literal),* $(,)?) => {
        /// Every colour slot, in the order the settings dialog lists them.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[allow(non_camel_case_types)]
        pub enum Slot { $($field),* }

        impl Slot {
            /// All slots, in settings order.
            pub const ALL: &'static [Slot] = &[$(Slot::$field),*];

            /// The slot's name.
            pub const fn name(self) -> &'static str {
                match self { $(Slot::$field => $name),* }
            }

            /// Looks the slot up by name.
            pub fn from_name(name: &str) -> Option<Self> {
                Self::ALL.iter().copied().find(|s| s.name() == name)
            }

            /// Reads the slot from a theme.
            pub const fn get(self, theme: &Theme) -> Color32 {
                match self { $(Slot::$field => theme.$field),* }
            }

            /// Writes the slot into a theme.
            pub fn set(self, theme: &mut Theme, color: Color32) {
                match self { $(Slot::$field => theme.$field = color),* }
            }
        }

        /// The colours the window is drawn with.
        #[derive(Clone, Copy, Debug)]
        pub struct Theme { $(pub $field: Color32),* }

        impl Default for Theme {
            fn default() -> Self { Self::from_palette(&DARK) }
        }
    };
}

define_slots!(
    bg => "background",
    surface => "surface",
    stripe => "stripe",
    border => "border",
    text => "text",
    strong => "strong text",
    weak => "weak text",
    accent => "accent",
    on_accent => "text on accent",
    label => "label",
    green => "green",
    red => "red",
    yellow => "yellow",
    blue => "blue",
    magenta => "magenta",
    dim => "dim",
    header_bg => "header background",
    header_text => "header text",
    chart_selected => "chart: selected hop",
    chart_unselected => "chart: other hops",
    chart_axis => "chart: axis",
    freq_bar => "histogram: bars",
    freq_text => "histogram: text",
    flows_selected => "flows: selected bar",
    flows_unselected => "flows: other bars",
    flows_current => "flows: current round",
    flows_noncurrent => "flows: other rounds",
    samples => "history: samples",
    samples_lost => "history: lost samples",
    map_world => "map: world",
    map_radius => "map: accuracy radius",
    map_selected => "map: selected location",
    info_bar_bg => "info bar background",
    info_bar_text => "info bar text",
    help_bg => "help background",
    help_text => "help text",
    settings_bg => "settings background",
);

impl Theme {
    /// Derives every element colour from a base palette.
    pub const fn from_palette(p: &Palette) -> Self {
        Self {
            bg: p.bg,
            surface: p.surface,
            stripe: p.stripe,
            border: p.border,
            text: p.text,
            strong: p.strong,
            weak: p.weak,
            accent: p.accent,
            on_accent: p.on_accent,
            label: p.label,
            green: p.green,
            red: p.red,
            yellow: p.yellow,
            blue: p.blue,
            magenta: p.magenta,
            dim: p.dim,
            header_bg: p.surface,
            header_text: p.strong,
            chart_selected: p.green,
            chart_unselected: p.dim,
            chart_axis: p.weak,
            freq_bar: p.green,
            freq_text: p.text,
            flows_selected: p.green,
            flows_unselected: p.dim,
            flows_current: p.yellow,
            flows_noncurrent: p.text,
            samples: p.yellow,
            samples_lost: p.red,
            map_world: p.dim,
            map_radius: p.yellow,
            map_selected: p.green,
            info_bar_bg: p.surface,
            info_bar_text: p.text,
            help_bg: p.surface,
            help_text: p.text,
            settings_bg: p.surface,
        }
    }
}

/// The theme setting: a palette family, a light/dark mode and per-slot overrides.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Settings {
    /// Palette family.
    pub family: Family,
    /// System, light or dark.
    pub mode: ThemePreference,
    /// Per-slot overrides applied on top of the family's palette.
    pub overrides: BTreeMap<Slot, [u8; 3]>,
}

impl Settings {
    /// The theme the family and overrides produce for a mode.
    pub fn theme(&self, dark: bool) -> Theme {
        let mut theme = Theme::from_palette(self.family.palette(dark));
        for (slot, [r, g, b]) in &self.overrides {
            slot.set(&mut theme, Color32::from_rgb(*r, *g, *b));
        }
        theme
    }

    /// The value of a slot as currently shown.
    pub fn slot(&self, slot: Slot, dark: bool) -> Color32 {
        self.overrides.get(&slot).map_or_else(
            || slot.get(&self.theme(dark)),
            |[r, g, b]| Color32::from_rgb(*r, *g, *b),
        )
    }

    /// Overrides a slot.
    pub fn set_slot(&mut self, slot: Slot, color: Color32) {
        self.overrides
            .insert(slot, [color.r(), color.g(), color.b()]);
    }

    /// Stored form, e.g. `gruvbox/dark;accent=f8be75`.
    pub fn to_stored(&self) -> String {
        let mode = match self.mode {
            ThemePreference::System => "system",
            ThemePreference::Light => "light",
            ThemePreference::Dark => "dark",
        };
        let mut out = format!("{}/{mode}", self.family.name().to_lowercase());
        for (slot, [r, g, b]) in &self.overrides {
            out.push(';');
            out.push_str(slot.name());
            out.push('=');
            out.push_str(&format!("{r:02x}{g:02x}{b:02x}"));
        }
        out
    }

    /// Parses the stored form; anything unknown falls back to the default.
    pub fn from_stored(stored: &str) -> Self {
        let mut settings = Self::default();
        let mut parts = stored.split(';');
        if let Some(head) = parts.next() {
            let (family, mode) = head.split_once('/').unwrap_or((head, ""));
            settings.family = Family::ALL
                .into_iter()
                .find(|f| f.name().eq_ignore_ascii_case(family))
                .unwrap_or_default();
            settings.mode = match mode {
                "light" => ThemePreference::Light,
                "dark" => ThemePreference::Dark,
                _ => ThemePreference::System,
            };
        }
        for part in parts {
            let Some((name, hex)) = part.split_once('=') else {
                continue;
            };
            let (Some(slot), Ok(value)) = (Slot::from_name(name), u32::from_str_radix(hex, 16))
            else {
                continue;
            };
            if hex.len() == 6 {
                settings
                    .overrides
                    .insert(slot, [(value >> 16) as u8, (value >> 8) as u8, value as u8]);
            }
        }
        settings
    }
}

fn settings_id() -> egui::Id {
    egui::Id::new("theme_settings")
}

/// The palette of the active theme.
pub fn of(ui: &Ui) -> Theme {
    current(ui.ctx()).theme(ui.visuals().dark_mode)
}

/// The active theme setting.
pub fn current(ctx: &egui::Context) -> Settings {
    ctx.data(|d| d.get_temp(settings_id())).unwrap_or_default()
}

/// Installs a theme setting: both variants of its family, and the mode.
pub fn install(ctx: &egui::Context, settings: &Settings) {
    ctx.style_mut_of(egui::Theme::Dark, |style| {
        apply(style, &settings.theme(true));
    });
    ctx.style_mut_of(egui::Theme::Light, |style| {
        apply(style, &settings.theme(false));
    });
    ctx.set_theme(settings.mode);
    ctx.data_mut(|d| d.insert_temp(settings_id(), settings.clone()));
}

fn apply(style: &mut Style, p: &Theme) {
    use egui::FontFamily::{Monospace, Proportional};
    style.text_styles = [
        (TextStyle::Heading, FontId::new(17.0, Proportional)),
        (TextStyle::Body, FontId::new(15.0, Proportional)),
        (TextStyle::Button, FontId::new(15.0, Proportional)),
        (TextStyle::Monospace, FontId::new(14.0, Monospace)),
        (TextStyle::Small, FontId::new(12.5, Proportional)),
    ]
    .into();

    let spacing = &mut style.spacing;
    spacing.item_spacing = egui::vec2(6.0, 2.0);
    spacing.button_padding = egui::vec2(10.0, 3.0);
    spacing.window_margin = egui::Margin::ZERO;
    spacing.menu_margin = egui::Margin::ZERO;
    spacing.interact_size.y = 22.0;

    let v = &mut style.visuals;
    v.override_text_color = None;
    v.weak_text_color = Some(p.weak);
    v.panel_fill = p.bg;
    v.window_fill = p.surface;
    v.window_stroke = Stroke::new(1.0, p.accent);
    v.extreme_bg_color = p.bg;
    v.faint_bg_color = p.stripe;
    v.code_bg_color = p.surface;
    v.hyperlink_color = p.accent;
    v.warn_fg_color = p.yellow;
    v.error_fg_color = p.red;
    v.selection.bg_fill = p.accent;
    v.selection.stroke = Stroke::new(1.0, p.on_accent);
    v.text_cursor.stroke = Stroke::new(2.0, p.accent);
    v.window_corner_radius = CornerRadius::ZERO;
    v.menu_corner_radius = CornerRadius::ZERO;
    v.window_shadow = Shadow::NONE;
    v.popup_shadow = Shadow::NONE;

    let w = &mut v.widgets;
    for (widget, bg, fg, stroke) in [
        (&mut w.noninteractive, p.bg, p.text, p.border),
        (
            &mut w.inactive,
            Color32::TRANSPARENT,
            p.text,
            Color32::TRANSPARENT,
        ),
        (&mut w.hovered, p.surface, p.strong, p.accent),
        (&mut w.active, p.accent, p.on_accent, p.accent),
        (&mut w.open, p.surface, p.strong, p.accent),
    ] {
        widget.bg_fill = bg;
        widget.weak_bg_fill = bg;
        widget.fg_stroke = Stroke::new(1.0, fg);
        widget.bg_stroke = Stroke::new(1.0, stroke);
        widget.corner_radius = CornerRadius::ZERO;
        widget.expansion = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip() {
        let mut settings = Settings {
            family: Family::Gruvbox,
            mode: ThemePreference::Dark,
            overrides: BTreeMap::new(),
        };
        settings.set_slot(Slot::accent, Color32::from_rgb(0x12, 0x34, 0x56));
        let stored = settings.to_stored();
        let parsed = Settings::from_stored(&stored);
        assert_eq!(parsed.family, Family::Gruvbox);
        assert_eq!(parsed.mode, ThemePreference::Dark);
        assert_eq!(
            parsed.slot(Slot::accent, true),
            Color32::from_rgb(0x12, 0x34, 0x56)
        );
        assert_eq!(Settings::from_stored("nonsense"), Settings::default());
    }

    #[test]
    fn every_slot_round_trips_through_its_name() {
        for slot in Slot::ALL {
            assert_eq!(Slot::from_name(slot.name()), Some(*slot));
        }
    }
}
