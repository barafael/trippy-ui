//! The visualizations: chart, frequency histogram, history, flows and map.
//!
//! Built on `egui_plot`, which gives the same information as the terminal
//! frontend's Braille charts with real axes and zooming.

use crate::state::State;
use crate::theme;
use egui::Ui;
use egui_plot::{Bar, BarChart, Line, Plot, Points};
use trippy_core::Hop;

/// The number of samples the history sparkline shows.
const HISTORY_SAMPLES: usize = 160;

/// The footer: the history of the selected hop and its frequency histogram.
pub fn footer(state: &State, ui: &mut Ui) {
    let p = theme::of(ui);
    let Some(hop) = state.selected_hop_or_target() else {
        return;
    };
    let width = ui.available_width();
    let height = ui.available_height();
    ui.horizontal(|ui| {
        ui.allocate_ui(egui::vec2(width * 0.74 - 6.0, height), |ui| {
            let samples = hop.samples();
            let start = samples.len().saturating_sub(HISTORY_SAMPLES);
            let points: Vec<[f64; 2]> = samples[start..]
                .iter()
                .enumerate()
                .map(|(i, d)| [i as f64, d.as_secs_f64() * 1000.0])
                .collect();
            Plot::new("history")
                .height(height)
                .allow_drag(false)
                .allow_zoom(false)
                .allow_scroll(false)
                .show_axes([true, true])
                .show_grid([false, true])
                .grid_color(p.border)
                .x_axis_label("Samples")
                .y_axis_label("ms")
                .show(ui, |plot_ui| {
                    if !points.is_empty() {
                        plot_ui.line(
                            Line::new(format!("Samples #{}", hop.ttl()), points)
                                .color(p.samples)
                                .width(1.5),
                        );
                    }
                });
        });
        ui.allocate_ui(egui::vec2(width * 0.26 - 6.0, height), |ui| {
            let bars = histogram(hop);
            let single = bars.len() <= 1;
            Plot::new("frequency")
                .height(height)
                .allow_drag(false)
                .allow_zoom(false)
                .allow_scroll(false)
                .show_axes([true, true])
                .show_grid([false, true])
                .grid_color(p.border)
                .x_axis_label("ms")
                .y_axis_label("%")
                .show(ui, |plot_ui| {
                    if bars.is_empty() {
                        return;
                    }
                    if single {
                        plot_ui.set_plot_bounds_x(-1.0..=1.0);
                    }
                    plot_ui.add(
                        BarChart::new(format!("Frequency #{}", hop.ttl()), bars)
                            .color(p.freq_bar)
                            .width(0.8),
                    );
                });
        });
    });
}

/// The percentage of samples per exact millisecond, as bars.
fn histogram(hop: &Hop) -> Vec<Bar> {
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
        .map(|(ms, count)| {
            let pct = (count * 100 / total) as f64;
            Bar::new(ms as f64, pct)
        })
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
