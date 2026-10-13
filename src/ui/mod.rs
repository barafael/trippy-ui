//! Window rendering: header with trace tabs, info bar, status bar, the body
//! and the overlays.

pub mod charts;
pub mod settings;
pub mod table;

use crate::command::{self, Command};
use crate::config::Named;
use crate::state::{Prompt, State};
use crate::theme;
use egui::{Align, Align2, Color32, CursorIcon, Label, Layout, RichText, Sense, Ui};

/// Horizontal gap between unrelated items.
pub const GAP: f32 = 10.0;

/// How long a key hint lights up after its command runs, in seconds.
const PULSE: f64 = 0.4;

/// How long the "Copied" confirmation stays in the status bar, in seconds.
const TOAST: f64 = 1.8;

/// Panel frame without padding.
pub fn flat_panel(ui: &Ui) -> egui::Frame {
    egui::Frame::NONE.fill(ui.visuals().panel_fill)
}

fn toast_id() -> egui::Id {
    egui::Id::new("copied_toast")
}

/// Copies `text` to the clipboard and confirms it in the status bar.
pub fn copy(ctx: &egui::Context, text: String) {
    let first_line = text.lines().next().unwrap_or_default();
    let mut preview: String = first_line.chars().take(48).collect();
    if preview.len() < text.len() {
        preview.push('…');
    }
    let now = ctx.input(|i| i.time);
    ctx.data_mut(|d| d.insert_temp(toast_id(), (format!("Copied {preview}"), now)));
    ctx.copy_text(text);
}

/// A full-width bar titling a block; filled with the accent when focused.
pub fn section_bar(ui: &mut Ui, title: &str, right: &str, focused: bool) {
    let p = theme::of(ui);
    let height = ui.text_style_height(&egui::TextStyle::Heading) + 8.0;
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), height), Sense::hover());
    let (bg, fg, sub) = if focused {
        (p.accent, p.on_accent, p.on_accent)
    } else {
        (p.surface, p.strong, p.weak)
    };
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, bg);
    painter.text(
        rect.left_center() + egui::vec2(GAP, 0.0),
        Align2::LEFT_CENTER,
        title,
        egui::TextStyle::Heading.resolve(ui.style()),
        fg,
    );
    painter.text(
        rect.right_center() - egui::vec2(GAP, 0.0),
        Align2::RIGHT_CENTER,
        right,
        egui::TextStyle::Body.resolve(ui.style()),
        sub,
    );
}

/// Renders the whole window; returns the command of a clicked key hint.
pub fn render(state: &mut State, ui: &mut Ui) -> Option<Command> {
    let mut command = None;

    egui::Panel::top("header")
        .frame(flat_panel(ui))
        .show_separator_line(false)
        .show(ui, |ui| {
            render_header(state, ui);
        });

    egui::Panel::bottom("info")
        .frame(flat_panel(ui))
        .exact_size(24.0)
        .show(ui, |ui| {
            if let Some(c) = render_info_bar(state, ui) {
                command = Some(c);
            }
        });

    let status = egui::Panel::bottom("status")
        .frame(flat_panel(ui))
        .show(ui, |ui| render_status(state, ui));
    if command.is_none() {
        command = status.inner;
    }
    render_toast(ui, status.response.rect.height());

    let loaded = state.fatal.is_none() && !state.traces.is_empty();
    if loaded {
        egui::Panel::bottom("footer")
            .frame(flat_panel(ui))
            .exact_size(86.0)
            .show(ui, |ui| charts::footer(state, ui));
    }
    if loaded && state.show_flows {
        egui::Panel::bottom("flows")
            .frame(flat_panel(ui))
            .exact_size(104.0)
            .show(ui, |ui| charts::flows(state, ui));
    }

    egui::CentralPanel::default()
        .frame(flat_panel(ui))
        .show(ui, |ui| {
            if !loaded {
                let message = state.fatal.as_deref().unwrap_or("no trace");
                centered_message(
                    ui,
                    RichText::new(message).color(ui.visuals().error_fg_color),
                );
            } else if state.show_chart {
                charts::chart(state, ui);
            } else if state.show_map {
                charts::map(state, ui);
            } else {
                if let Some(c) = table::render(state, ui) {
                    command = Some(c);
                }
            }
        });

    if state.show_help
        && let Some(c) = render_help(state, ui)
    {
        command = Some(c);
    }
    if state.show_settings
        && let Some(c) = settings::render(state, ui)
    {
        command = Some(c);
    }
    command
}

/// Fills the body with one centered message.
pub fn centered_message(ui: &mut Ui, text: RichText) {
    ui.centered_and_justified(|ui| ui.label(text));
}

/// A clickable key cap followed by its description.
fn key_hint(ui: &mut Ui, keys: &str, description: &str, pulse: f32, clickable: bool) -> bool {
    let p = theme::of(ui);
    let sense = if clickable {
        Sense::CLICK
    } else {
        Sense::hover()
    };
    let key = ui.add(
        Label::new(
            RichText::new(format!(" {keys} "))
                .monospace()
                .color(p.accent.lerp_to_gamma(p.on_accent, pulse))
                .background_color(p.surface.lerp_to_gamma(p.accent, pulse)),
        )
        .sense(sense),
    );
    let text = ui.add(Label::new(RichText::new(description).color(p.text)).sense(sense));
    if clickable {
        if key.hovered() || text.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            ui.painter().hline(
                text.rect.x_range(),
                text.rect.bottom() - 1.0,
                egui::Stroke::new(1.0, p.text),
            );
        }
        if key.clicked() || text.clicked() {
            return true;
        }
    }
    false
}

/// The status bar: notice, prompt and clickable key hints.
fn render_status(state: &State, ui: &mut Ui) -> Option<Command> {
    let p = theme::of(ui);
    let now = ui.input(|i| i.time);
    let mut clicked = None;
    egui::Frame::NONE
        .inner_margin(egui::Margin::symmetric(GAP as i8, 4))
        .show(ui, |ui| {
            if let Some(notice) = &state.notice {
                ui.label(RichText::new(notice).color(p.red).strong());
            }
            ui.horizontal_wrapped(|ui| {
                if state.prompt == Some(Prompt::Search) || !state.search.is_empty() {
                    let open = state.prompt == Some(Prompt::Search);
                    ui.label(RichText::new("filter:").color(p.label));
                    let text =
                        RichText::new(format!(" {}{} ", state.search, if open { "▏" } else { "" }))
                            .monospace();
                    ui.label(if open {
                        text.color(p.on_accent).background_color(p.accent)
                    } else {
                        text.color(p.strong)
                    });
                    ui.add_space(GAP * 2.0);
                }
                for (keys, description) in state.key_bindings() {
                    let commands = command::hinted(keys);
                    let pulse = state
                        .pressed
                        .as_ref()
                        .filter(|(pressed, _)| {
                            commands.iter().any(|c| command::same_action(c, pressed))
                        })
                        .map_or(0.0, |(_, at)| 1.0 - ((now - at) / PULSE).clamp(0.0, 1.0));
                    if pulse > 0.0 {
                        ui.ctx().request_repaint();
                    }
                    let clickable = commands.len() == 1 && state.prompt.is_none();
                    if key_hint(ui, keys, description, pulse as f32, clickable) {
                        clicked = commands.into_iter().next();
                    }
                    ui.add_space(GAP);
                }
            });
        });
    clicked
}

fn render_header(state: &mut State, ui: &mut Ui) {
    let p = theme::of(ui);
    egui::Frame::NONE
        .inner_margin(egui::Margin::symmetric(GAP as i8, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("trippy-ui")
                        .heading()
                        .strong()
                        .color(p.accent),
                );
                ui.weak(env!("CARGO_PKG_VERSION"));
                ui.add_space(GAP);
                ui.label(
                    RichText::new("Trace network paths like a normal person.")
                        .italics()
                        .color(p.weak),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new(clock(state)).monospace().color(p.weak));
                });
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("Target:").color(p.label));
                ui.monospace(RichText::new(endpoint_source(state)).color(p.strong));
                ui.label(RichText::new("->").color(p.weak));
                ui.monospace(RichText::new(endpoint_target(state)).color(p.strong));
                ui.add_space(GAP * 2.0);
                ui.label(RichText::new("Status:").color(p.label));
                let status = state.trace().map_or_else(
                    || String::from("no trace"),
                    |trace| status_line(state, trace),
                );
                ui.label(RichText::new(status).color(status_color(state, ui)));
            });
        });
    if state.traces.len() > 1 {
        let titles: Vec<String> = state.traces.iter().map(|t| t.hostname.clone()).collect();
        let refs: Vec<&str> = titles.iter().map(String::as_str).collect();
        if let Some(i) = tab_strip(ui, &refs, state.selected_trace, false, "") {
            state.select_trace(i);
        }
    }
}

/// Cycles the address mode, as clicking the info bar does.
fn next_address_mode(mode: crate::config::AddressMode) -> crate::config::AddressMode {
    use crate::config::AddressMode;
    match mode {
        AddressMode::Ip => AddressMode::Host,
        AddressMode::Host => AddressMode::Both,
        AddressMode::Both => AddressMode::Ip,
    }
}

fn status_line(state: &State, trace: &crate::trace::Trace) -> String {
    if let Some(error) = &trace.error {
        return format!("Failed: {error}");
    }
    if let Some(start) = state.frozen_start {
        let elapsed = start.elapsed().unwrap_or_default();
        return format!("Frozen ({})", humantime::format_duration(elapsed));
    }
    let hops = state.hops().len();
    let flows = trace.snapshot.flows().len();
    if trace.max_flows() > 1 {
        format!("Running, discovered {hops} hops and {flows} unique flows")
    } else {
        format!("Running, discovered {hops} hops")
    }
}

fn status_color(state: &State, ui: &Ui) -> Color32 {
    let p = theme::of(ui);
    if state.trace().is_some_and(|t| t.error.is_some()) {
        p.red
    } else if state.frozen() {
        p.yellow
    } else {
        p.green
    }
}

/// The info bar: protocol, privilege and the mode toggles.
fn render_info_bar(state: &mut State, ui: &mut Ui) -> Option<Command> {
    let p = theme::of(ui);
    let mut clicked = None;
    egui::Frame::NONE
        .fill(p.surface)
        .inner_margin(egui::Margin::symmetric(GAP as i8, 3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let protocol = format!(
                    "{}/{}",
                    match state.trace_config.addr_family {
                        trippy_dns::IpAddrFamily::Ipv4Only => "IPv4",
                        trippy_dns::IpAddrFamily::Ipv6Only => "IPv6",
                        _ => "IPv4/6",
                    },
                    state.trace_config.protocol
                );
                ui.label(RichText::new(protocol).monospace().color(p.info_bar_text));
                ui.label(
                    RichText::new(match state.trace_config.privilege_mode {
                        trippy_core::PrivilegeMode::Privileged => "privileged",
                        trippy_core::PrivilegeMode::Unprivileged => "unprivileged",
                    })
                    .monospace()
                    .color(p.info_bar_text),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let toggle = |ui: &mut Ui, text: String, on: bool| -> bool {
                        let color = if on { p.accent } else { p.info_bar_text };
                        let response = ui.add(
                            Label::new(RichText::new(text).monospace().color(color))
                                .sense(Sense::CLICK)
                                .selectable(false),
                        );
                        if response.hovered() {
                            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                        }
                        ui.add_space(GAP);
                        response.clicked()
                    };
                    if toggle(
                        ui,
                        format!(
                            "»:{}",
                            state
                                .settings
                                .max_addrs
                                .map_or_else(|| "-".into(), |n| n.to_string())
                        ),
                        state.settings.max_addrs.is_some(),
                    ) {
                        clicked = Some(Command::ExpandHosts);
                    }
                    if toggle(
                        ui,
                        format!(
                            "privacy:{}",
                            state
                                .settings
                                .privacy_max_ttl
                                .map_or_else(|| " -".into(), |n| format!("{n:2}"))
                        ),
                        state.settings.privacy_max_ttl.is_some(),
                    ) {
                        clicked = Some(Command::ExpandPrivacy);
                    }
                    if toggle(ui, state.settings.address_mode.name().to_string(), true) {
                        state.settings.address_mode =
                            next_address_mode(state.settings.address_mode);
                    }
                    if toggle(
                        ui,
                        format!(
                            "[{}] detail",
                            if state.settings.show_hop_details {
                                "x"
                            } else {
                                " "
                            }
                        ),
                        state.settings.show_hop_details,
                    ) {
                        clicked = Some(Command::ToggleHopDetails);
                    }
                    let as_on = state.settings.lookup_as_info;
                    if toggle(
                        ui,
                        format!("[{}] ASN", if as_on { "x" } else { " " }),
                        as_on,
                    ) {
                        clicked = Some(Command::ToggleAsInfo);
                    }
                });
            });
        });
    clicked
}

/// A row of tabs on a bar; returns the clicked one.
fn tab_strip(
    ui: &mut Ui,
    titles: &[&str],
    selected: usize,
    focused: bool,
    right: &str,
) -> Option<usize> {
    let p = theme::of(ui);
    let bar = if focused { p.accent } else { p.surface };
    let mut clicked = None;
    egui::Frame::NONE.fill(bar).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            for (i, title) in titles.iter().enumerate() {
                let (bg, fg) = match (i == selected, focused) {
                    (true, false) => (p.accent, p.on_accent),
                    (true, true) => (p.bg, p.accent),
                    (false, true) => (Color32::TRANSPARENT, p.on_accent),
                    (false, false) => (Color32::TRANSPARENT, p.text),
                };
                let text = RichText::new(*title).color(fg);
                let text = if i == selected { text.strong() } else { text };
                let response = egui::Frame::NONE
                    .fill(bg)
                    .inner_margin(egui::Margin::symmetric(GAP as i8, 4))
                    .show(ui, |ui| {
                        ui.add(Label::new(text).sense(Sense::CLICK).selectable(false))
                    })
                    .inner;
                if response.clicked() {
                    clicked = Some(i);
                }
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(GAP);
                ui.label(RichText::new(right).color(if focused { p.on_accent } else { p.weak }));
            });
        });
    });
    clicked
}

/// The source endpoint, or `unknown` until the tracer has discovered it.
fn endpoint_source(state: &State) -> String {
    if state.settings.privacy_max_ttl.is_some() {
        return String::from("**Hidden**");
    }
    let Some(trace) = state.trace() else {
        return String::from("unknown");
    };
    let Some(addr) = trace.source_addr() else {
        return String::from("unknown");
    };
    match state.trace_config.port_direction.src() {
        Some(port) => format!("{addr}:{}", port.0),
        None => addr.to_string(),
    }
}

/// The target endpoint with its hostname.
pub fn endpoint_target(state: &State) -> String {
    let Some(trace) = state.trace() else {
        return String::from("unknown");
    };
    let addr = trace.target_addr;
    let suffix = match state.trace_config.port_direction.dest() {
        Some(port) => format!(":{}", port.0),
        None => String::new(),
    };
    if trace.hostname == addr.to_string() {
        format!("{addr}{suffix}")
    } else {
        format!("{addr}{suffix} ({})", trace.hostname)
    }
}

/// The clock in the configured timezone.
fn clock(state: &State) -> String {
    match state.settings.timezone() {
        Some(tz) => chrono::Utc::now()
            .with_timezone(&tz)
            .format("%Y-%m-%d %H:%M:%S %Z")
            .to_string(),
        None => chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    }
}

/// The help overlay.
fn render_help(state: &mut State, ui: &mut Ui) -> Option<Command> {
    let p = theme::of(ui);
    let frame = egui::Frame::popup(ui.style())
        .corner_radius(0.0)
        .inner_margin(0.0)
        .stroke(egui::Stroke::new(1.0, p.accent));
    let mut command = None;
    egui::Modal::new(egui::Id::new("help"))
        .frame(frame)
        .backdrop_color(Color32::from_black_alpha(160))
        .show(ui.ctx(), |ui| {
            let screen = ui.ctx().content_rect();
            ui.set_max_width(screen.width() * 0.6);
            section_bar(ui, "Help", "h, ?, Esc or q to close", true);
            egui::Frame::NONE.inner_margin(GAP).show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.monospace(LOGO);
                    ui.add_space(GAP);
                    ui.label(
                        RichText::new("A network diagnostic tool")
                            .italics()
                            .color(p.weak),
                    );
                    ui.add_space(GAP);
                });
                ui.separator();
                egui::Grid::new("help_keys")
                    .num_columns(2)
                    .spacing([GAP * 2.0, 4.0])
                    .show(ui, |ui| {
                        for (keys, description) in state.view_key_bindings() {
                            ui.monospace(RichText::new(keys).color(p.accent));
                            ui.label(RichText::new(description).color(p.text));
                            ui.end_row();
                        }
                    });
                ui.add_space(GAP);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Press").color(p.weak));
                    if ui.link(RichText::new("s").color(p.accent)).clicked() {
                        command = Some(Command::ShowSettings);
                    }
                    ui.label(RichText::new("for all settings, or").color(p.weak));
                    if ui.link(RichText::new("5").color(p.accent)).clicked() {
                        command = Some(Command::ShowSettingsTab(4));
                    }
                    ui.label(RichText::new("for all key bindings.").color(p.weak));
                });
                ui.add_space(GAP);
                ui.label(RichText::new("https://github.com/fujiapple852/trippy").color(p.weak));
                ui.label(RichText::new("Distributed under the Apache License 2.0").color(p.weak));
            });
        });
    command
}

const LOGO: &str = "\
 _____    _                
|_   _| _(_)_ __ _ __ _  _ 
  | || '_| | '_ \\ '_ \\ || |
  |_||_| |_| .__/ .__/\\_, |
           |_|  |_|   |__/ ";

/// The "Copied" confirmation, floating above the status bar.
fn render_toast(ui: &Ui, status_height: f32) {
    let now = ui.input(|i| i.time);
    let Some((message, at)) = ui.ctx().data(|d| d.get_temp::<(String, f64)>(toast_id())) else {
        return;
    };
    let age = now - at;
    if age >= TOAST {
        return;
    }
    ui.ctx().request_repaint();
    let p = theme::of(ui);
    let opacity = ((TOAST - age) / (TOAST / 3.0)).clamp(0.0, 1.0) as f32;
    egui::Area::new(toast_id())
        .anchor(Align2::RIGHT_BOTTOM, [-GAP, -(status_height + GAP + 24.0)])
        .order(egui::Order::Tooltip)
        .interactable(false)
        .show(ui.ctx(), |ui| {
            ui.set_opacity(opacity);
            egui::Frame::NONE
                .fill(p.surface)
                .stroke(egui::Stroke::new(1.0, p.green))
                .inner_margin(egui::Margin::symmetric(GAP as i8, 4))
                .show(ui, |ui| {
                    ui.label(RichText::new(format!("✔ {message}")).color(p.green));
                });
        });
}
