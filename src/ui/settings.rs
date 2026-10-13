//! The settings dialog: the same tabs as the terminal frontend, made editable.

use crate::command::Command;
use crate::config::{AddressMode, AsMode, GeoIpMode, IcmpExtensionMode, Named};
use crate::state::{SettingsTab, State};
use crate::theme;
use crate::ui::{GAP, section_bar, tab_strip};
use egui::{Color32, RichText, Sense, Ui};

/// Renders the settings dialog; returns the command of a clicked control.
pub fn render(state: &mut State, ui: &mut Ui) -> Option<Command> {
    let p = theme::of(ui);
    let frame = egui::Frame::popup(ui.style())
        .corner_radius(0.0)
        .inner_margin(0.0)
        .stroke(egui::Stroke::new(1.0, p.accent));
    let mut command = None;
    egui::Modal::new(egui::Id::new("settings"))
        .frame(frame)
        .backdrop_color(Color32::from_black_alpha(160))
        .show(ui.ctx(), |ui| {
            let screen = ui.ctx().content_rect();
            ui.set_max_width(screen.width() * 0.72);
            ui.set_min_width(screen.width() * 0.42);
            section_bar(ui, "Settings", "s, Esc or q to close", true);
            let titles: Vec<&str> = SettingsTab::ALL.iter().map(|tab| tab.title()).collect();
            if let Some(i) = tab_strip(ui, &titles, state.settings_tab as usize, true, "") {
                command = Some(Command::ShowSettingsTab(i));
            }
            egui::Frame::NONE.inner_margin(GAP).show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(screen.height() * 0.45)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        render_items(state, ui);
                    });
            });
            section_bar(ui, "Info", "", false);
            egui::Frame::NONE.inner_margin(GAP).show(ui, |ui| {
                ui.label(RichText::new(info(state)).color(p.weak));
            });
        });
    command
}

fn info(state: &State) -> String {
    match state.settings_tab {
        SettingsTab::Tui => {
            "View settings. Use the mouse or Enter to change the selected item.".into()
        }
        SettingsTab::Trace => "Tracing parameters, fixed when the tracers start.".into(),
        SettingsTab::Dns => "DNS resolution settings.".into(),
        SettingsTab::GeoIp => "The MaxMind database file, given with --geoip-mmdb-file.".into(),
        SettingsTab::Bindings => "The key map, as configured by the terminal frontend.".into(),
        SettingsTab::Theme => "Enter or click a colour to reset it to the palette default.".into(),
        SettingsTab::Columns => {
            "Enter toggles a column; , and . move the selected column up and down.".into()
        }
    }
}

fn render_items(state: &mut State, ui: &mut Ui) {
    match state.settings_tab {
        SettingsTab::Tui => render_tui(state, ui),
        SettingsTab::Trace => render_trace(state, ui),
        SettingsTab::Dns => render_dns(state, ui),
        SettingsTab::GeoIp => render_geoip(state, ui),
        SettingsTab::Bindings => render_bindings(state, ui),
        SettingsTab::Theme => render_theme(state, ui),
        SettingsTab::Columns => render_columns(state, ui),
    }
}

/// One settings row: a label and a value widget, highlighted when selected.
///
/// Returns whether the row was clicked, so the caller can select it.
fn row(ui: &mut Ui, selected: bool, label: &str, value: impl FnOnce(&mut Ui)) -> bool {
    let p = theme::of(ui);
    let bg = if selected {
        p.accent
    } else {
        Color32::TRANSPARENT
    };
    let response = egui::Frame::NONE
        .fill(bg)
        .inner_margin(egui::Margin::symmetric(GAP as i8, 3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.visuals_mut().override_text_color =
                    Some(if selected { p.on_accent } else { p.label });
                ui.set_min_width(240.0);
                ui.label(RichText::new(label).monospace());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.visuals_mut().override_text_color =
                        Some(if selected { p.on_accent } else { p.text });
                    value(ui);
                });
            });
        })
        .response
        .interact(Sense::click());
    response.clicked()
}

fn value_text(ui: &mut Ui, text: String) {
    ui.label(RichText::new(text).monospace());
}

fn render_tui(state: &mut State, ui: &mut Ui) {
    let selected = state.settings.address_mode;
    if row(ui, state.settings_item == 0, "tui-address-mode", |ui| {
        combo(
            ui,
            "address_mode",
            AddressMode::ALL,
            selected,
            |v| v.name(),
            |v| {
                state.settings.address_mode = v;
            },
        );
    }) {
        state.settings_item = 0;
    }
    let selected = state.settings.as_mode;
    if row(ui, state.settings_item == 1, "tui-as-mode", |ui| {
        combo(
            ui,
            "as_mode",
            AsMode::ALL,
            selected,
            |v| v.name(),
            |v| {
                state.settings.as_mode = v;
            },
        );
    }) {
        state.settings_item = 1;
    }
    let selected = state.settings.icmp_extension_mode;
    if row(
        ui,
        state.settings_item == 2,
        "tui-icmp-extension-mode",
        |ui| {
            combo(
                ui,
                "icmp_mode",
                IcmpExtensionMode::ALL,
                selected,
                |v| v.name(),
                |v| {
                    state.settings.icmp_extension_mode = v;
                },
            );
        },
    ) {
        state.settings_item = 2;
    }
    let selected = state.settings.geoip_mode;
    if row(ui, state.settings_item == 3, "tui-geoip-mode", |ui| {
        combo(
            ui,
            "geoip_mode",
            GeoIpMode::ALL,
            selected,
            |v| v.name(),
            |v| {
                state.settings.geoip_mode = v;
            },
        );
    }) {
        state.settings_item = 3;
    }
    let value = state
        .settings
        .max_addrs
        .map_or_else(|| String::from("auto"), |n| n.to_string());
    if row(ui, state.settings_item == 4, "tui-max-addrs", |ui| {
        value_text(ui, value)
    }) {
        state.settings_item = 4;
    }
    let value = state
        .settings
        .privacy_max_ttl
        .map_or_else(|| String::from("off"), |n| n.to_string());
    if row(ui, state.settings_item == 5, "tui-privacy-max-ttl", |ui| {
        value_text(ui, value)
    }) {
        state.settings_item = 5;
    }
    let value = state.settings.columns.to_codes();
    if row(ui, state.settings_item == 6, "tui-custom-columns", |ui| {
        value_text(ui, value)
    }) {
        state.settings_item = 6;
    }
    let value = humantime::format_duration(state.refresh_rate).to_string();
    if row(ui, state.settings_item == 7, "tui-refresh-rate", |ui| {
        value_text(ui, value)
    }) {
        state.settings_item = 7;
    }
    let value = state
        .settings
        .timezone
        .clone()
        .unwrap_or_else(|| String::from("local"));
    if row(ui, state.settings_item == 8, "tui-timezone", |ui| {
        value_text(ui, value)
    }) {
        state.settings_item = 8;
    }
}

fn render_trace(state: &mut State, ui: &mut Ui) {
    let c = &state.trace_config;
    let items: Vec<(&str, String)> = vec![
        ("first-ttl", c.first_ttl.to_string()),
        ("max-ttl", c.max_ttl.to_string()),
        (
            "min-round-duration",
            humantime::format_duration(c.min_round_duration).to_string(),
        ),
        (
            "max-round-duration",
            humantime::format_duration(c.max_round_duration).to_string(),
        ),
        (
            "grace-duration",
            humantime::format_duration(c.grace_duration).to_string(),
        ),
        ("max-inflight", c.max_inflight.to_string()),
        ("initial-sequence", c.initial_sequence.to_string()),
        (
            "read-timeout",
            humantime::format_duration(c.read_timeout).to_string(),
        ),
        ("packet-size", c.packet_size.to_string()),
        ("payload-pattern", c.payload_pattern.to_string()),
        ("tos", c.tos.to_string()),
        ("icmp-extensions", c.icmp_extensions.to_string()),
        (
            "interface",
            c.interface.clone().unwrap_or_else(|| String::from("auto")),
        ),
        ("multipath-strategy", c.multipath_strategy.to_string()),
        (
            "target-port",
            c.port_direction
                .dest()
                .map_or_else(|| String::from("-"), |p| p.0.to_string()),
        ),
        (
            "source-port",
            c.port_direction
                .src()
                .map_or_else(|| String::from("-"), |p| p.0.to_string()),
        ),
        ("max-samples", c.max_samples.to_string()),
        ("max-flows", c.max_flows().to_string()),
    ];
    for (i, (label, value)) in items.into_iter().enumerate() {
        if row(ui, state.settings_item == i, label, |ui| {
            value_text(ui, value)
        }) {
            state.settings_item = i;
        }
    }
}

fn render_dns(state: &mut State, ui: &mut Ui) {
    let c = &state.trace_config;
    let items: Vec<(&str, String)> = vec![
        (
            "dns-timeout",
            humantime::format_duration(c.dns_timeout).to_string(),
        ),
        ("dns-ttl", humantime::format_duration(c.dns_ttl).to_string()),
        (
            "dns-resolve-method",
            format!("{:?}", c.dns_resolve_method).to_lowercase(),
        ),
        ("dns-resolve-all", c.dns_resolve_all.to_string()),
    ];
    for (i, (label, value)) in items.into_iter().enumerate() {
        if row(ui, state.settings_item == i, label, |ui| {
            value_text(ui, value)
        }) {
            state.settings_item = i;
        }
    }
    let mut enabled = state.settings.lookup_as_info;
    if row(ui, state.settings_item == 4, "dns-lookup-as-info", |ui| {
        ui.checkbox(&mut enabled, "");
    }) {
        state.settings_item = 4;
    }
    if enabled != state.settings.lookup_as_info {
        state.settings.lookup_as_info = enabled;
    }
}

fn render_geoip(state: &mut State, ui: &mut Ui) {
    let value = state
        .trace_config
        .geoip_mmdb_file
        .clone()
        .unwrap_or_else(|| String::from("none"));
    if row(ui, state.settings_item == 0, "geoip-mmdb-file", |ui| {
        value_text(ui, value)
    }) {
        state.settings_item = 0;
    }
}

fn render_bindings(state: &mut State, ui: &mut Ui) {
    for (i, (keys, label)) in crate::command::bindings().into_iter().enumerate() {
        if row(ui, state.settings_item == i, label, |ui| {
            value_text(ui, keys)
        }) {
            state.settings_item = i;
        }
    }
}

fn render_theme(state: &mut State, ui: &mut Ui) {
    let dark = ui.visuals().dark_mode;
    for (i, slot) in theme::Slot::ALL.iter().enumerate() {
        let mut color = state.settings.theme.slot(*slot, dark);
        if row(ui, state.settings_item == i, slot.name(), |ui| {
            if ui.color_edit_button_srgba(&mut color).changed() {
                state.settings.theme.set_slot(*slot, color);
            }
        }) {
            state.settings_item = i;
        }
    }
}

fn render_columns(state: &mut State, ui: &mut Ui) {
    let entries: Vec<(crate::columns::Column, bool)> = state
        .settings
        .columns
        .all()
        .iter()
        .map(|e| (e.column, e.shown))
        .collect();
    for (i, (column, shown)) in entries.into_iter().enumerate() {
        let mut value = shown;
        if row(ui, state.settings_item == i, column.name(), |ui| {
            ui.checkbox(&mut value, "");
        }) {
            state.settings_item = i;
        }
        if value != shown {
            state.settings.columns.toggle(i);
        }
    }
}

/// A combo box over a [`Named`] enum.
fn combo<T: Named + PartialEq>(
    ui: &mut Ui,
    id: &str,
    all: &[T],
    current: T,
    name: impl Fn(T) -> &'static str,
    mut set: impl FnMut(T),
) {
    egui::ComboBox::from_id_salt(id)
        .selected_text(name(current))
        .show_ui(ui, |ui| {
            for value in all {
                if ui
                    .selectable_label(*value == current, name(*value))
                    .clicked()
                {
                    set(*value);
                }
            }
        });
}
