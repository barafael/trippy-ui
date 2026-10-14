//! The settings dialog: the same tabs as the terminal frontend, made editable.

use crate::bindings::Item;
use crate::command::Command;
use crate::settings_items::Control;
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
            let close = format!(
                "{}, {} or {} to close",
                state.bindings.display(Item::ToggleSettings),
                state.bindings.display(Item::ClearSelection),
                state.bindings.display(Item::Quit),
            );
            section_bar(ui, "Settings", &close, true);
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
        SettingsTab::Bindings => {
            "The key map; change it with --tui-key-bindings or the [bindings] section.".into()
        }
        SettingsTab::Theme => {
            "Enter cycles the palette and mode; Enter on a colour resets it to the palette default."
                .into()
        }
        SettingsTab::Columns => format!(
            "Enter or {} toggles a column; {} and {} move the selected column up and down.",
            state.bindings.display(Item::ToggleChart),
            state.bindings.display(Item::PreviousHopAddress),
            state.bindings.display(Item::NextHopAddress),
        ),
    }
}

/// A change a control asked for, applied once the state is free again.
type Change = Box<dyn FnOnce(&mut State)>;

/// The rows of the selected tab, each with its control.
fn render_items(state: &mut State, ui: &mut Ui) {
    let items = state.settings_items(state.settings_tab);
    // A control's change is applied after the loop, once `state` is free.
    let mut change: Option<Change> = None;
    let mut select = None;
    for (i, item) in items.into_iter().enumerate() {
        let selected = state.settings_item == i;
        let label = item.label.clone();
        let clicked = row(ui, selected, &label, |ui| match item.control {
            Control::Text { .. } => value_text(ui, &item.value),
            Control::Choice {
                options,
                selected,
                set,
            } => {
                let mut picked = None;
                egui::ComboBox::from_id_salt(&label)
                    .selected_text(options[selected])
                    .show_ui(ui, |ui| {
                        for (j, name) in options.iter().enumerate() {
                            if ui.selectable_label(j == selected, *name).clicked() {
                                picked = Some(j);
                            }
                        }
                    });
                if let Some(j) = picked {
                    change = Some(Box::new(move |state| set(state, j)));
                }
            }
            Control::Toggle { on, toggle } => {
                let mut value = on;
                if ui.checkbox(&mut value, "").changed() {
                    change = Some(Box::new(move |state| toggle(state)));
                }
            }
            Control::Color { color, set, .. } => {
                let mut value = color;
                if ui.color_edit_button_srgba(&mut value).changed() {
                    change = Some(Box::new(move |state| set(state, value)));
                }
            }
        });
        if clicked {
            select = Some(i);
        }
    }
    if let Some(i) = select {
        state.settings_item = i;
    }
    if let Some(change) = change {
        change(state);
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

fn value_text(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).monospace());
}
