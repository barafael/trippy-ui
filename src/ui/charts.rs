//! The visualizations: chart, frequency histogram, history, flows and map.
//!
//! Built on `egui_plot`. The main chart is a line per hop, as in the terminal
//! frontend; the footer is a pair of compact, axes-less bar charts that fill
//! their space, like the terminal frontend's sparkline and bar chart.

use crate::state::State;
use crate::theme;
use egui::Ui;
use egui_plot::{Bar, BarChart, Line, Plot, Points};
use trippy_core::Hop;

/// The width of one bar, in pixels, so the charts fill gradually instead of
/// stretching a few bars over the whole width. Roughly a terminal cell.
const BAR_PX: f32 = 9.0;

/// The footer: the history of the selected hop and its frequency histogram.
pub fn footer(state: &State, ui: &mut Ui) {
    let p = theme::of(ui);
    let Some(hop) = state.selected_hop_or_target() else {
        return;
    };
    let max_samples = state.trace().map_or(256, crate::trace::Trace::max_samples);
    let width = ui.available_width();
    let height = ui.available_height();
    let gap = 8.0;
    let usable = (width - gap - ui.spacing().item_spacing.x).max(64.0);
    let chart_height = (height - 18.0).max(24.0);
    let samples_slots = slots(usable * 0.74).min(max_samples);
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(usable * 0.74, height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.label(
                    egui::RichText::new(format!("Samples #{}", hop.ttl()))
                        .small()
                        .color(p.weak),
                );
                sparkline(ui, hop, samples_slots, chart_height);
            },
        );
        ui.add_space(gap);
        ui.allocate_ui_with_layout(
            egui::vec2(usable * 0.26, height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.label(
                    egui::RichText::new(format!("Frequency #{}", hop.ttl()))
                        .small()
                        .color(p.weak),
                );
                frequency(ui, hop, chart_height);
            },
        );
    });
}

/// How many bars of [`BAR_PX`] fit in `width`.
fn slots(width: f32) -> usize {
    ((width / BAR_PX) as usize).max(8)
}

/// The history of the selected hop as a sparkline: one bar per sample on a
/// fixed axis, so it fills from the left as samples arrive instead of
/// stretching to fill the width. Lost samples are full-height bars in the lost
/// colour, like the terminal frontend's absent value.
fn sparkline(ui: &mut Ui, hop: &Hop, slots: usize, height: f32) {
    let p = theme::of(ui);
    let samples = hop.samples();
    let max = samples
        .iter()
        .map(|d| d.as_micros() as f64)
        .fold(0.0_f64, f64::max);
    let start = samples.len().saturating_sub(slots);
    let mut bars = Vec::with_capacity(slots);
    let mut lost = Vec::new();
    for (i, sample) in samples[start..].iter().enumerate() {
        if sample.is_zero() {
            lost.push(Bar::new(i as f64, max.max(1.0)).fill(p.samples_lost));
        } else {
            bars.push(Bar::new(i as f64, sample.as_micros() as f64).fill(p.samples));
        }
    }
    Plot::new("history")
        .height(height)
        .allow_drag(false)
        .allow_zoom(false)
        .allow_scroll(false)
        .show_axes([false, false])
        .show_grid([false, false])
        .auto_bounds([false, true])
        .default_x_bounds(0.0, slots.max(1) as f64)
        .set_margin_fraction(egui::vec2(0.0, 0.06))
        .show(ui, |plot_ui| {
            plot_ui.add(BarChart::new("samples", bars).width(1.0));
            if !lost.is_empty() {
                plot_ui.add(BarChart::new("lost", lost).width(1.0));
            }
        });
}

/// The percentage of samples per exact millisecond, as fixed-width bars laid
/// out by their millisecond value, filling from the left.
///
/// The axis is at least [`FREQUENCY_SPAN`] milliseconds wide, so a single
/// bucket is a bar of the terminal frontend's width rather than a stretched
/// block.
fn frequency(ui: &mut Ui, hop: &Hop, height: f32) {
    let p = theme::of(ui);
    let values = histogram(hop);
    let Some(max_ms) = values.last().map(|(ms, _)| *ms) else {
        return;
    };
    let span = (max_ms + 1).max(FREQUENCY_SPAN) as f64;
    let bars: Vec<Bar> = values
        .iter()
        .map(|(ms, pct)| Bar::new(*ms as f64, *pct as f64).fill(p.freq_bar))
        .collect();
    Plot::new("frequency")
        .height(height)
        .allow_drag(false)
        .allow_zoom(false)
        .allow_scroll(false)
        .show_axes([false, false])
        .show_grid([false, false])
        .auto_bounds([false, true])
        .default_x_bounds(0.0, span)
        .set_margin_fraction(egui::vec2(0.03, 0.06))
        .show(ui, |plot_ui| {
            plot_ui.add(
                BarChart::new(format!("Frequency #{}", hop.ttl()), bars)
                    .color(p.freq_bar)
                    .width(1.0),
            );
        });
}

/// The smallest width of the frequency axis, in milliseconds.
const FREQUENCY_SPAN: u64 = 6;

/// The percentage of samples per exact millisecond, as `(milliseconds, percent)`.
fn histogram(hop: &Hop) -> Vec<(u64, u64)> {
    let samples = hop.samples();
    let total = samples.len();
    if total == 0 {
        return Vec::new();
    }
    let mut counts: std::collections::BTreeMap<u64, usize> = std::collections::BTreeMap::new();
    for sample in samples {
        *counts.entry(sample.as_millis() as u64).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(ms, count)| (ms, (count * 100 / total) as u64))
        .collect()
}

/// The round-trip time of every hop over the last `max_samples / zoom` samples.
pub fn chart(state: &State, ui: &mut Ui) {
    let p = theme::of(ui);
    let window = (state.trace().map_or(256, crate::trace::Trace::max_samples) / state.zoom).max(1);
    let selected = state
        .selection
        .selected
        .and_then(|i| state.rows.get(i).copied());
    let lines: Vec<(usize, Vec<[f64; 2]>)> = state
        .hops()
        .iter()
        .enumerate()
        .map(|(i, hop)| {
            let samples = hop.samples();
            let start = samples.len().saturating_sub(window);
            let points = samples[start..]
                .iter()
                .enumerate()
                .map(|(j, d)| [(start + j) as f64, d.as_secs_f64() * 1000.0])
                .collect();
            (i, points)
        })
        .collect();
    let height = ui.available_height();
    Plot::new("rtt_chart")
        .height(height)
        .grid_color(p.border)
        .x_axis_label("Samples")
        .y_axis_label("RTT (ms)")
        .show(ui, |plot_ui| {
            for (i, points) in lines {
                if points.is_empty() {
                    continue;
                }
                let selected = selected == Some(i);
                let color = if selected {
                    p.chart_selected
                } else {
                    p.chart_unselected
                };
                let width = if selected { 2.0 } else { 1.0 };
                let ttl = state.hops()[i].ttl();
                plot_ui.line(
                    Line::new(format!("#{ttl}"), points)
                        .color(color)
                        .width(width),
                );
            }
        });
}

/// The flows of the trace, as a bar per flow.
pub fn flows(state: &State, ui: &mut Ui) {
    let p = theme::of(ui);
    let Some(trace) = state.trace() else {
        return;
    };
    let bars: Vec<Bar> = state
        .flows()
        .iter()
        .enumerate()
        .map(|(i, id)| {
            let count = trace.snapshot.round_count(*id) as f64;
            let color = if *id == state.selected_flow {
                p.flows_selected
            } else {
                p.flows_unselected
            };
            Bar::new(i as f64, count).fill(color)
        })
        .collect();
    let labels: Vec<(f64, String)> = state
        .flows()
        .iter()
        .enumerate()
        .map(|(i, id)| (i as f64, format!("{id}")))
        .collect();
    let height = ui.available_height();
    Plot::new("flows")
        .height(height)
        .grid_color(p.border)
        .show_axes([false, true])
        .x_axis_formatter(move |mark, _| {
            labels
                .iter()
                .find(|(x, _)| (x - mark.value).abs() < 0.5)
                .map(|(_, name)| name.clone())
                .unwrap_or_default()
        })
        .show(ui, |plot_ui| {
            if !bars.is_empty() {
                plot_ui.add(BarChart::new("Flows", bars).width(0.7));
            }
        });
}

/// The world map with a pin per GeoIP location seen on the selected flow.
pub fn map(state: &State, ui: &mut Ui) {
    let p = theme::of(ui);
    let selected = state.selected_hop_or_target().map(Hop::ttl);
    let entries = map_entries(state);
    let height = (ui.available_height() - 60.0).max(80.0);
    Plot::new("world_map")
        .height(height)
        .allow_drag(false)
        .allow_zoom(false)
        .allow_scroll(false)
        .grid_color(p.border)
        .show(ui, |plot_ui| {
            for polyline in crate::world::WORLD {
                plot_ui.line(
                    Line::new("world", polyline.to_vec())
                        .color(p.map_world)
                        .width(0.5),
                );
            }
            for entry in &entries {
                let is_selected = Some(entry.ttl) == selected;
                let color = if is_selected {
                    p.map_selected
                } else {
                    p.map_radius
                };
                let radius = if is_selected { 6.0 } else { 3.0 };
                plot_ui.add(
                    Points::new(
                        format!("#{}", entry.ttl),
                        vec![[entry.longitude, entry.latitude]],
                    )
                    .color(color)
                    .radius(radius),
                );
                if entry.radius_km > 220.0 {
                    plot_ui.line(
                        Line::new(
                            "",
                            circle(entry.longitude, entry.latitude, entry.radius_km / 110.0),
                        )
                        .color(p.map_radius)
                        .width(0.5),
                    );
                }
            }
        });
    let text = match state.geoip_enabled() {
        false => "GeoIP not enabled (pass --geoip-mmdb-file)".to_string(),
        true => match selected {
            Some(ttl) => {
                let hits: Vec<&MapEntry> = entries.iter().filter(|e| e.ttl == ttl).collect();
                match hits.as_slice() {
                    [] => format!("No GeoIP data for hop {ttl}"),
                    [only] => format!(
                        "Hop {ttl}: {} [{}, {} ~{}km]",
                        only.name, only.latitude, only.longitude, only.radius_km
                    ),
                    _ => format!("Multiple GeoIP locations for hop {ttl}"),
                }
            }
            None => String::from("Select a hop"),
        },
    };
    ui.label(egui::RichText::new(text).color(p.text));
}

/// One GeoIP location on the map.
struct MapEntry {
    ttl: u8,
    name: String,
    latitude: f64,
    longitude: f64,
    radius_km: f64,
}

/// Every GeoIP location of the visible hops, hiding private ones.
fn map_entries(state: &State) -> Vec<MapEntry> {
    let mut entries = Vec::new();
    for hop in state.hops() {
        if state.settings.privacy_max_ttl >= Some(hop.ttl()) {
            continue;
        }
        for addr in hop.addrs() {
            let Ok(Some(city)) = state.geoip.lookup(*addr) else {
                continue;
            };
            let Some((latitude, longitude, radius)) = city.position() else {
                continue;
            };
            entries.push(MapEntry {
                ttl: hop.ttl(),
                name: city.long_name(),
                latitude,
                longitude,
                radius_km: f64::from(radius),
            });
        }
    }
    entries
}

/// A circle of latitude/longitude points around a location.
fn circle(longitude: f64, latitude: f64, radius_degrees: f64) -> Vec<[f64; 2]> {
    (0..=48)
        .map(|i| {
            let angle = f64::from(i) * std::f64::consts::TAU / 48.0;
            [
                longitude + radius_degrees * angle.cos(),
                latitude + radius_degrees * angle.sin(),
            ]
        })
        .collect()
}
