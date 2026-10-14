//! The visualizations: chart, frequency histogram, history, flows and map.
//!
//! Built on `egui_plot`. The main chart is a line per hop, as in the terminal
//! frontend; the footer is a pair of compact, axes-less bar charts that fill
//! their space, like the terminal frontend's sparkline and bar chart.

use crate::state::State;
use crate::theme;
use crate::ui::GAP;
use egui::Ui;
use egui_plot::{Bar, BarChart, Line, Plot, Points, Text};
use std::time::Duration;
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

/// The newest `slots` samples, newest first, in microseconds; `None` is a
/// lost probe.
///
/// Samples are stored newest first, so the window is the head of the slice,
/// as in the terminal frontend's history.
pub fn history_window(samples: &[Duration], slots: usize) -> Vec<Option<u64>> {
    samples[..slots.min(samples.len())]
        .iter()
        .map(|s| (!s.is_zero()).then_some(s.as_micros() as u64))
        .collect()
}

/// The history of the selected hop as a sparkline: one bar per sample on a
/// fixed axis, so it fills from the left as samples arrive instead of
/// stretching to fill the width. Lost samples are full-height bars in the lost
/// colour, like the terminal frontend's absent value.
fn sparkline(ui: &mut Ui, hop: &Hop, slots: usize, height: f32) {
    let p = theme::of(ui);
    let window = history_window(hop.samples(), slots);
    let max = window.iter().flatten().copied().max().unwrap_or(1).max(1) as f64;
    let mut bars = Vec::with_capacity(slots);
    let mut lost = Vec::new();
    for (i, sample) in window.into_iter().enumerate() {
        match sample {
            // Bars are centred on their x, so slot `i` spans `i..i + 1`.
            None => lost.push(Bar::new(i as f64 + 0.5, max).fill(p.samples_lost)),
            Some(us) => bars.push(Bar::new(i as f64 + 0.5, us as f64).fill(p.samples)),
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
    let values = histogram_of(hop.samples());
    let Some(max_ms) = values.last().map(|(ms, _)| *ms) else {
        return;
    };
    let span = (max_ms + 1).max(FREQUENCY_SPAN) as f64;
    let bars: Vec<Bar> = values
        .iter()
        .map(|(ms, pct)| Bar::new(*ms as f64 + 0.5, *pct as f64).fill(p.freq_bar))
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
///
/// Lost probes count towards the total but have no bucket, as in the terminal
/// frontend's histogram.
pub fn histogram_of(samples: &[Duration]) -> Vec<(u64, u64)> {
    let total = samples.len();
    if total == 0 {
        return Vec::new();
    }
    let mut counts: std::collections::BTreeMap<u64, usize> = std::collections::BTreeMap::new();
    for sample in samples.iter().filter(|s| !s.is_zero()) {
        *counts.entry(sample.as_millis() as u64).or_default() += 1;
    }
    counts
        .into_iter()
        .map(|(ms, count)| (ms, (count * 100 / total) as u64))
        .collect()
}

/// The newest `window` samples of a hop as chart points, newest at x = 0.
pub fn chart_series(samples: &[Duration], window: usize) -> Vec<[f64; 2]> {
    samples[..window.min(samples.len())]
        .iter()
        .enumerate()
        .map(|(i, d)| [i as f64, d.as_secs_f64() * 1000.0])
        .collect()
}

/// The round-trip time of every hop over the newest `max_samples / zoom` samples.
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
        .map(|(i, hop)| (i, chart_series(hop.samples(), window)))
        .collect();
    let height = ui.available_height();
    Plot::new("rtt_chart")
        .height(height)
        .grid_color(p.border)
        .x_axis_label(format!("Samples ({window} at {}x)", state.zoom))
        .y_axis_label("RTT (ms)")
        .default_x_bounds(0.0, window as f64)
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

/// The width of one flow bar with its gap, in pixels.
const FLOW_BAR_PX: f32 = 44.0;

/// The first visible flow, given the window width, the previous start and the
/// selection: the selected flow is always kept in view, as in the terminal
/// frontend's flows chart.
pub fn flows_window(total: usize, visible: usize, start: usize, selected: usize) -> usize {
    let visible = visible.max(1);
    let mut start = start.min(total.saturating_sub(visible));
    if selected < start {
        start = selected;
    } else if selected >= start + visible {
        start = selected + 1 - visible;
    }
    start
}

/// The flows of the trace, as a bar per flow, scrolled to keep the selected
/// flow visible.
pub fn flows(state: &mut State, ui: &mut Ui) {
    let p = theme::of(ui);
    let flows = state.flows();
    let Some(trace) = state.trace() else {
        return;
    };
    let visible = ((ui.available_width() - 2.0 * GAP) / FLOW_BAR_PX) as usize;
    let selected_index = flows
        .iter()
        .position(|id| *id == state.selected_flow)
        .unwrap_or(0);
    let start = flows_window(
        flows.len(),
        visible,
        state.flows_start_index,
        selected_index,
    );
    let end = (start + visible.max(1)).min(flows.len());
    let round_flow = trace.snapshot.round_flow_id();
    let mut bars = Vec::new();
    let mut labels = Vec::new();
    for (x, id) in flows[start..end].iter().enumerate() {
        let count = trace.snapshot.round_count(*id);
        let color = if *id == state.selected_flow {
            p.flows_selected
        } else {
            p.flows_unselected
        };
        let label_color = if *id == round_flow {
            p.flows_current
        } else {
            p.flows_noncurrent
        };
        bars.push(Bar::new(x as f64, count as f64).fill(color));
        labels.push(
            Text::new(
                format!("flow-{id}"),
                egui_plot::PlotPoint::new(x as f64, count as f64 / 2.0),
                egui::RichText::new(format!("{id}\n{count}"))
                    .monospace()
                    .strong(),
            )
            .color(label_color),
        );
    }
    state.flows_start_index = start;
    let (left, right) = (start > 0, end < flows.len());
    let height = ui.available_height();
    let response = Plot::new("flows")
        .height(height)
        .show_axes([false, false])
        .show_grid([false, false])
        .allow_drag(false)
        .allow_zoom(false)
        .allow_scroll(false)
        .auto_bounds([false, true])
        .default_x_bounds(-0.5, visible.max(1) as f64 - 0.5)
        .set_margin_fraction(egui::vec2(0.0, 0.1))
        .show(ui, |plot_ui| {
            if !bars.is_empty() {
                plot_ui.add(BarChart::new("Flows", bars).width(0.8));
            }
            for label in labels {
                plot_ui.text(label);
            }
        })
        .response;
    let rect = response.rect;
    let font = egui::TextStyle::Body.resolve(ui.style());
    if left {
        ui.painter().text(
            rect.left_center() + egui::vec2(4.0, 0.0),
            egui::Align2::LEFT_CENTER,
            "◀",
            font.clone(),
            p.border,
        );
    }
    if right {
        ui.painter().text(
            rect.right_center() - egui::vec2(4.0, 0.0),
            egui::Align2::RIGHT_CENTER,
            "▶",
            font,
            p.border,
        );
    }
}

/// The world map with a pin per GeoIP location seen on the selected flow.
pub fn map(state: &State, ui: &mut Ui) {
    let p = theme::of(ui);
    let selected_hop = state.selected_hop_or_target();
    let selected = selected_hop.map(Hop::ttl);
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
                let is_selected = selected.is_some_and(|ttl| entry.hops.contains(&ttl));
                let color = if is_selected {
                    p.map_selected
                } else {
                    p.map_radius
                };
                let radius = if is_selected { 6.0 } else { 3.0 };
                plot_ui.add(
                    Points::new(entry.name.clone(), vec![[entry.longitude, entry.latitude]])
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
    let text = match (selected_hop, state.geoip_enabled()) {
        (None, _) => String::from("Select a hop"),
        (Some(hop), _) if state.settings.privacy_max_ttl >= Some(hop.ttl()) => {
            String::from("**Hidden**")
        }
        (Some(_), false) => String::from("GeoIP not enabled (pass --geoip-mmdb-file)"),
        (Some(hop), true) => {
            let ttl = hop.ttl();
            let hits: Vec<&MapEntry> = entries.iter().filter(|e| e.hops.contains(&ttl)).collect();
            match hits.as_slice() {
                [] if hop.addr_count() > 0 => {
                    let addrs: Vec<String> = hop.addrs().map(ToString::to_string).collect();
                    format!("No GeoIP data for hop {ttl} ({})", addrs.join(", "))
                }
                [] => format!("No GeoIP data for hop {ttl}"),
                [only] => format!(
                    "Hop {ttl}: {} [{}, {} ~{}km]",
                    only.name, only.latitude, only.longitude, only.radius_km
                ),
                _ => format!("Multiple GeoIP locations for hop {ttl}"),
            }
        }
    };
    ui.label(egui::RichText::new(text).color(p.text));
}

/// One GeoIP location on the map, shared by every hop seen there.
struct MapEntry {
    name: String,
    latitude: f64,
    longitude: f64,
    radius_km: f64,
    hops: Vec<u8>,
}

/// Every GeoIP location of the visible hops, hiding private ones, one entry
/// per distinct place.
fn map_entries(state: &State) -> Vec<MapEntry> {
    let mut entries: Vec<MapEntry> = Vec::new();
    for hop in state.hops() {
        if state.settings.privacy_max_ttl >= Some(hop.ttl()) {
            continue;
        }
        for addr in hop.addrs() {
            let Ok(Some(city)) = state.geoip.lookup(*addr) else {
                continue;
            };
            let Some((latitude, longitude, radius)) = city.coordinates() else {
                continue;
            };
            let name = city.long_name();
            match entries.iter_mut().find(|e| e.name == name) {
                Some(entry) => {
                    if !entry.hops.contains(&hop.ttl()) {
                        entry.hops.push(hop.ttl());
                    }
                }
                None => entries.push(MapEntry {
                    name,
                    latitude,
                    longitude,
                    radius_km: f64::from(radius),
                    hops: vec![hop.ttl()],
                }),
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(values: &[u64]) -> Vec<Duration> {
        values.iter().map(|v| Duration::from_millis(*v)).collect()
    }

    #[test]
    fn history_takes_the_newest_samples() {
        let samples = ms(&[5, 0, 3, 2, 1]);
        assert_eq!(history_window(&samples, 3), [Some(5000), None, Some(3000)]);
        assert_eq!(history_window(&samples, 10).len(), 5);
        assert!(history_window(&[], 4).is_empty());
    }

    #[test]
    fn chart_series_starts_at_the_newest_sample() {
        let samples = ms(&[5, 4, 3]);
        assert_eq!(chart_series(&samples, 2), [[0.0, 5.0], [1.0, 4.0]]);
        assert_eq!(chart_series(&samples, 8).len(), 3);
    }

    #[test]
    fn histogram_skips_lost_probes_but_counts_them() {
        let samples = ms(&[3, 0, 3, 7]);
        assert_eq!(histogram_of(&samples), [(3, 50), (7, 25)]);
        assert!(histogram_of(&[]).is_empty());
    }

    #[test]
    fn flows_window_keeps_the_selection_visible() {
        assert_eq!(flows_window(10, 4, 0, 0), 0);
        assert_eq!(flows_window(10, 4, 0, 5), 2);
        assert_eq!(flows_window(10, 4, 6, 1), 1);
        assert_eq!(flows_window(10, 4, 9, 9), 6);
        assert_eq!(flows_window(2, 4, 0, 1), 0);
        assert_eq!(flows_window(0, 0, 0, 0), 0);
    }
}
