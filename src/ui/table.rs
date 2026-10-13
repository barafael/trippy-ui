//! The hops table: every column, formatted and navigable like the terminal.

use crate::columns::Column;
use crate::format;
use crate::state::State;
use crate::theme;
use crate::ui::section_bar;
use egui::{Align, Color32, Layout, RichText, Sense, Ui};
use egui_extras::{Column as TableColumn, TableBuilder};
use trippy_core::Hop;

/// The width of a fixed column.
fn width(column: Column) -> f32 {
    match column {
        Column::Ttl => 34.0,
        Column::Status => 40.0,
        Column::Asn => 84.0,
        Column::LossPct | Column::FlossPct | Column::StdDev => 66.0,
        _ => 62.0,
    }
}

/// Renders the hops table; returns the command of a double click.
pub fn render(state: &mut State, ui: &mut Ui) -> Option<crate::command::Command> {
    let p = theme::of(ui);
    let selected = state.selection.selected;
    let scroll_to = state.selection.take_moved().then_some(selected).flatten();
    let columns: Vec<Column> = state.settings.columns.shown().collect();
    let rows = state.rows.clone();
    let detail = state.settings.show_hop_details;

    let counter = match selected {
        Some(s) => format!("{}/{}", s + 1, rows.len()),
        None => format!("{} hops", rows.len()),
    };
    section_bar(ui, "Hops", &counter, true);
    if rows.is_empty() {
        let message = if state.search.is_empty() {
            "Awaiting data..."
        } else {
            "No hop matches the filter."
        };
        ui.add_space(8.0);
        ui.label(RichText::new(message).italics().color(p.weak));
        return None;
    }

    let row_height = ui.spacing().interact_size.y;
    let mut builder = TableBuilder::new(ui)
        .id_salt("hops")
        .striped(true)
        .sense(Sense::CLICK)
        .cell_layout(Layout::left_to_right(Align::Center))
        .max_scroll_height(f32::INFINITY);
    for column in &columns {
        builder = if column.is_variable() {
            builder.column(TableColumn::remainder().clip(true))
        } else {
            builder.column(TableColumn::exact(width(*column)))
        };
    }
    if let Some(row) = scroll_to {
        builder = builder.scroll_to_row(row, None);
    }

    let mut clicked = None;
    let mut double = false;
    builder
        .header(row_height, |mut header| {
            for column in &columns {
                header.col(|ui| {
                    ui.label(RichText::new(column.name()).strong().color(p.label));
                });
            }
        })
        .body(|mut body| {
            for (i, hop_index) in rows.iter().enumerate() {
                let hop = &state.hops()[*hop_index];
                let is_selected = selected == Some(i);
                let height = if detail && is_selected {
                    row_height * 6.5
                } else {
                    row_height
                };
                body.row(height, |mut row| {
                    row.set_selected(is_selected);
                    let is_target = state
                        .trace()
                        .is_some_and(|t| t.snapshot.is_target(hop, state.selected_flow));
                    for column in &columns {
                        row.col(|ui| {
                            if is_selected {
                                ui.visuals_mut().override_text_color = Some(p.on_accent);
                                ui.visuals_mut().weak_text_color = Some(p.on_accent);
                            }
                            if *column == Column::Status {
                                let color = status_color(ui, format::status_of(hop, is_target));
                                let (rect, _) =
                                    ui.allocate_exact_size(egui::vec2(12.0, 12.0), Sense::hover());
                                ui.painter().circle_filled(rect.center(), 5.0, color);
                                return;
                            }
                            let color = row_color(state, hop, is_selected, ui);
                            if *column == Column::Host && detail && is_selected {
                                let text = format::details(
                                    hop,
                                    state.selected_hop_address,
                                    &state.resolver,
                                    &state.geoip,
                                    &state.settings,
                                );
                                ui.vertical(|ui| {
                                    for line in text.lines() {
                                        ui.label(RichText::new(line).monospace().color(color));
                                    }
                                });
                            } else {
                                let text = format::cell(
                                    *column,
                                    hop,
                                    &state.resolver,
                                    &state.geoip,
                                    &state.settings,
                                );
                                ui.label(RichText::new(text).monospace().color(color));
                            }
                        });
                    }
                    let response = row.response();
                    if response.clicked() || response.double_clicked() {
                        clicked = Some(i);
                    }
                    if response.double_clicked() {
                        double = true;
                    }
                });
            }
        });

    if let Some(i) = clicked {
        state.selection.select(i);
        state.selected_hop_address = 0;
    }
    double.then_some(crate::command::Command::ToggleHopDetails)
}

/// The colour of a row, active or inactive in the current round.
fn row_color(state: &State, hop: &Hop, is_selected: bool, ui: &Ui) -> Color32 {
    let p = theme::of(ui);
    if is_selected {
        return p.on_accent;
    }
    let in_round = state
        .trace()
        .is_some_and(|t| t.snapshot.is_in_round(hop, state.selected_flow));
    if in_round { p.text } else { p.weak }
}

/// The colour of a status dot.
fn status_color(ui: &Ui, status: format::Status) -> Color32 {
    let p = theme::of(ui);
    match status {
        format::Status::Ok => p.green,
        format::Status::Partial => p.blue,
        format::Status::AllLost => p.dim,
        format::Status::TargetPartial => p.yellow,
        format::Status::TargetLost => p.red,
    }
}
