//! Application state and command handling.

use crate::command::{self, Command, InputCommand, ScrollType};
use crate::config::{AddressMode, Named, Options, TraceConfig};
use crate::format;
use crate::geoip;
use crate::list::Selection;
use crate::settings::Settings;
use crate::theme;
use crate::trace::{self, Trace};
use std::time::{Duration, SystemTime};
use trippy_core::{FlowId, Hop};
use trippy_dns::{Config as DnsConfig, DnsResolver, ResolveMethod};

/// The maximum chart zoom.
pub const MAX_ZOOM: usize = 16;

/// Which prompt is open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prompt {
    /// The hop filter.
    Search,
}

/// The tabs of the settings dialog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsTab {
    /// View settings.
    Tui,
    /// Tracing parameters.
    Trace,
    /// DNS settings.
    Dns,
    /// GeoIP settings.
    GeoIp,
    /// The key map.
    Bindings,
    /// The theme.
    Theme,
    /// The table columns.
    Columns,
}

impl SettingsTab {
    /// All tabs, in order.
    pub const ALL: [SettingsTab; 7] = [
        SettingsTab::Tui,
        SettingsTab::Trace,
        SettingsTab::Dns,
        SettingsTab::GeoIp,
        SettingsTab::Bindings,
        SettingsTab::Theme,
        SettingsTab::Columns,
    ];

    /// The tab title.
    pub const fn title(self) -> &'static str {
        match self {
            SettingsTab::Tui => "Tui",
            SettingsTab::Trace => "Trace",
            SettingsTab::Dns => "DNS",
            SettingsTab::GeoIp => "GeoIp",
            SettingsTab::Bindings => "Bindings",
            SettingsTab::Theme => "Theme",
            SettingsTab::Columns => "Columns",
        }
    }

    /// How many items the tab lists.
    pub fn count(self) -> usize {
        match self {
            SettingsTab::Tui => 9,
            SettingsTab::Trace => 18,
            SettingsTab::Dns => 5,
            SettingsTab::GeoIp => 1,
            SettingsTab::Bindings => command::bindings().len(),
            SettingsTab::Theme => theme::Slot::ALL.len(),
            SettingsTab::Columns => crate::columns::ALL.len(),
        }
    }
}

/// The application state.
pub struct State {
    /// Is the window running?
    pub running: bool,
    /// The DNS resolver, shared by every trace.
    pub resolver: DnsResolver,
    /// Optional GeoIP lookups.
    pub geoip: geoip::Lookup,
    /// One trace per resolved target.
    pub traces: Vec<Trace>,
    /// The selected trace.
    pub selected_trace: usize,
    /// The view settings.
    pub settings: Settings,
    /// The tracing parameters, fixed at startup.
    pub trace_config: TraceConfig,
    /// The targets as given on the command line.
    pub targets: Vec<String>,
    /// Why no trace could be started.
    pub fatal: Option<String>,
    /// Transient message for the status bar.
    pub notice: Option<String>,
    /// The open prompt, if any.
    pub prompt: Option<Prompt>,
    /// The hop filter.
    pub search: String,
    /// The selected hop, as an index into [`State::rows`].
    pub selection: Selection,
    /// Indices into the current flow's hops that pass the filter.
    pub rows: Vec<usize>,
    /// The selected flow.
    pub selected_flow: FlowId,
    /// The selected address of the selected hop, in detail mode.
    pub selected_hop_address: usize,
    /// The first flow shown in the flows strip.
    pub flows_start_index: usize,
    /// The help overlay is open.
    pub show_help: bool,
    /// The settings dialog is open.
    pub show_settings: bool,
    /// The selected settings tab.
    pub settings_tab: SettingsTab,
    /// The selected item of the settings tab.
    pub settings_item: usize,
    /// Show the round-trip time chart instead of the table.
    pub show_chart: bool,
    /// Show the world map instead of the table.
    pub show_map: bool,
    /// Show the flows view.
    pub show_flows: bool,
    /// The chart zoom, 1 to [`MAX_ZOOM`].
    pub zoom: usize,
    /// When the snapshots were frozen.
    pub frozen_start: Option<SystemTime>,
    /// The last command and when it ran, for the status bar hints.
    pub pressed: Option<(Command, f64)>,
    /// How often the window repaints.
    pub refresh_rate: Duration,
}

impl State {
    /// Builds the resolver, resolves the targets and starts a trace for each.
    pub fn new(options: Options) -> Self {
        let mut state = Self {
            running: true,
            resolver: DnsResolver::start(DnsConfig::new(
                options.trace.dns_resolve_method,
                options.trace.addr_family,
                options.trace.dns_timeout,
                options.trace.dns_ttl,
            ))
            .expect("DNS resolver"),
            geoip: match &options.trace.geoip_mmdb_file {
                Some(path) => geoip::Lookup::from_file(std::path::Path::new(path))
                    .unwrap_or_else(|_| geoip::Lookup::empty()),
                None => geoip::Lookup::empty(),
            },
            traces: Vec::new(),
            selected_trace: 0,
            settings: options.settings,
            trace_config: options.trace,
            targets: options.targets,
            fatal: None,
            notice: None,
            prompt: None,
            search: String::new(),
            selection: Selection::default(),
            rows: Vec::new(),
            selected_flow: FlowId(0),
            selected_hop_address: 0,
            flows_start_index: 0,
            show_help: false,
            show_settings: false,
            settings_tab: SettingsTab::Tui,
            settings_item: 0,
            show_chart: false,
            show_map: false,
            show_flows: false,
            zoom: 1,
            frozen_start: None,
            pressed: None,
            refresh_rate: options.refresh_rate,
        };
        match trace::resolve_targets(&state.targets, &state.trace_config, &state.resolver) {
            Ok(targets) if targets.is_empty() => {
                state.fatal = Some(String::from("no valid IP addresses for the given targets"));
            }
            Ok(targets) => {
                let pid = std::process::id() as u16;
                for (i, (hostname, addr)) in targets.into_iter().enumerate() {
                    match Trace::start(
                        &state.trace_config,
                        hostname,
                        addr,
                        pid.wrapping_add(i as u16),
                    ) {
                        Ok(trace) => state.traces.push(trace),
                        Err(e) => state.fatal = Some(e.to_string()),
                    }
                }
            }
            Err(e) => state.fatal = Some(e.to_string()),
        }
        state.refresh_rows();
        state
    }

    /// The selected trace, if any.
    pub fn trace(&self) -> Option<&Trace> {
        self.traces.get(self.selected_trace)
    }

    /// The hops of the selected flow.
    pub fn hops(&self) -> &[Hop] {
        self.trace()
            .map_or(&[], |t| t.snapshot.hops_for_flow(self.selected_flow))
    }

    /// The hop under the cursor.
    pub fn selected_hop(&self) -> Option<&Hop> {
        let index = *self.rows.get(self.selection.selected?)?;
        self.hops().get(index)
    }

    /// The selected hop, or the target when nothing is selected.
    pub fn selected_hop_or_target(&self) -> Option<&Hop> {
        self.selected_hop().or_else(|| {
            self.trace()
                .map(|t| t.snapshot.target_hop(self.selected_flow))
        })
    }

    /// The largest number of addresses seen at any hop.
    pub fn max_hosts(&self) -> Option<u8> {
        self.hops()
            .iter()
            .map(Hop::addr_count)
            .max()
            .and_then(|n| u8::try_from(n).ok())
    }

    /// Whether a GeoIP database is loaded.
    pub const fn geoip_enabled(&self) -> bool {
        self.geoip.enabled()
    }

    /// Whether the trace is frozen.
    pub const fn frozen(&self) -> bool {
        self.frozen_start.is_some()
    }

    /// Refreshes the snapshots and the filtered rows. Called once per frame.
    pub fn poll(&mut self) {
        if !self.frozen() {
            for trace in &mut self.traces {
                trace.poll();
            }
        }
        self.refresh_rows();
        let max = self
            .selected_hop()
            .map_or(0, |h| h.addr_count().saturating_sub(1));
        self.selected_hop_address = self.selected_hop_address.min(max);
    }

    /// Recomputes the filtered rows and keeps the selection in range.
    fn refresh_rows(&mut self) {
        let query = self.search.to_lowercase();
        let hops = self.hops();
        let rows: Vec<usize> = if query.is_empty() {
            (0..hops.len()).collect()
        } else {
            hops.iter()
                .enumerate()
                .filter(|(_, hop)| self.hop_matches(hop, &query))
                .map(|(i, _)| i)
                .collect()
        };
        self.rows = rows;
        self.selection.len = self.rows.len();
        match self.selection.selected {
            Some(_) if self.rows.is_empty() => self.selection.selected = None,
            Some(selected) if selected >= self.rows.len() => {
                self.selection.selected = self.rows.len().checked_sub(1);
            }
            None if !self.rows.is_empty() => self.selection.selected = Some(0),
            _ => {}
        }
    }

    /// Whether a hop matches the filter.
    fn hop_matches(&self, hop: &Hop, query: &str) -> bool {
        format::host(hop, &self.resolver, &self.geoip, &self.settings)
            .to_lowercase()
            .contains(query)
    }

    /// Runs a command and updates the state.
    pub fn run_command(&mut self, command: Command, ctx: &egui::Context) {
        self.notice = None;
        self.pressed = Some((command.clone(), ctx.input(|i| i.time)));
        if self.show_help {
            self.help_command(command);
        } else if self.show_settings {
            self.settings_command(command);
        } else {
            self.normal_command(command, ctx);
        }
    }

    fn help_command(&mut self, command: Command) {
        match command {
            Command::ShowHelp | Command::ClearSelection | Command::Exit => self.show_help = false,
            Command::ShowSettings => {
                self.show_help = false;
                self.show_settings = true;
            }
            Command::ShowSettingsTab(tab) => {
                self.show_help = false;
                self.open_settings_tab(tab);
            }
            _ => {}
        }
    }

    fn settings_command(&mut self, command: Command) {
        match command {
            Command::ShowSettings | Command::ClearSelection | Command::Exit => {
                self.show_settings = false;
            }
            Command::ShowHelp => {
                self.show_settings = false;
                self.show_help = true;
            }
            Command::ShowSettingsTab(tab) => self.open_settings_tab(tab),
            Command::Next(ScrollType::Tab, n) => {
                let i = (self.settings_tab as usize + n) % SettingsTab::ALL.len();
                self.open_settings_tab(i);
            }
            Command::Previous(ScrollType::Tab, n) => {
                let len = SettingsTab::ALL.len();
                let i = (self.settings_tab as usize + len - n % len) % len;
                self.open_settings_tab(i);
            }
            Command::Next(ScrollType::List, n) => self.move_settings_item(1, n),
            Command::Previous(ScrollType::List, n) => self.move_settings_item(-1, n),
            Command::Top => {
                self.settings_item = 0;
            }
            Command::Bottom => {
                self.settings_item = self.settings_tab.count().saturating_sub(1);
            }
            Command::Activate => self.activate_setting(),
            Command::ToggleChart if self.settings_tab == SettingsTab::Columns => {
                self.settings.columns.toggle(self.settings_item);
            }
            Command::Next(ScrollType::Address, _) if self.settings_tab == SettingsTab::Columns => {
                self.settings.columns.move_down(self.settings_item);
                self.move_settings_item(1, 1);
            }
            Command::Previous(ScrollType::Address, _)
                if self.settings_tab == SettingsTab::Columns =>
            {
                self.settings.columns.move_up(self.settings_item);
                self.move_settings_item(-1, 1);
            }
            _ => {}
        }
    }

    fn normal_command(&mut self, command: Command, ctx: &egui::Context) {
        match command {
            Command::ShowHelp => self.show_help = true,
            Command::ShowSettings => self.show_settings = true,
            Command::ShowSettingsTab(tab) => self.open_settings_tab(tab),
            Command::Exit => self.running = false,
            Command::ClearSelection => {
                self.selection.clear();
                self.selected_hop_address = 0;
            }
            Command::Copy => match self.selection_text() {
                Some(text) => crate::ui::copy(ctx, text),
                None => self.notice = Some(String::from("Nothing to copy")),
            },
            Command::Activate | Command::ToggleHopDetails => self.toggle_hop_details(),
            Command::Next(ScrollType::Tab, n) => {
                if self.show_flows {
                    self.move_flow(n, true);
                } else {
                    self.select_trace(self.selected_trace + n);
                }
            }
            Command::Previous(ScrollType::Tab, n) => {
                if self.show_flows {
                    self.move_flow(n, false);
                } else {
                    self.select_trace(
                        self.selected_trace
                            .checked_sub(n)
                            .unwrap_or(self.traces.len().saturating_sub(1)),
                    );
                }
            }
            Command::Next(ScrollType::List, n) => self.selection.next(n),
            Command::Previous(ScrollType::List, n) => self.selection.previous(n),
            Command::Next(ScrollType::Address, n) => self.move_hop_address(n, true),
            Command::Previous(ScrollType::Address, n) => self.move_hop_address(n, false),
            Command::Top => self.selection.first(),
            Command::Bottom => self.selection.last(),
            Command::AddressMode(mode) => self.settings.address_mode = mode,
            Command::ToggleFreeze => {
                self.frozen_start = match self.frozen_start {
                    None => Some(SystemTime::now()),
                    Some(_) => None,
                };
            }
            Command::ToggleChart => {
                self.show_chart = !self.show_chart;
                self.show_map = false;
            }
            Command::ToggleMap => {
                self.show_map = !self.show_map;
                self.show_chart = false;
            }
            Command::ToggleFlows => self.toggle_flows(),
            Command::ExpandPrivacy => {
                self.settings.privacy_max_ttl = match self.settings.privacy_max_ttl {
                    None => Some(0),
                    Some(ttl) if usize::from(ttl) < self.hops().len() => Some(ttl + 1),
                    Some(ttl) => Some(ttl),
                };
            }
            Command::ContractPrivacy => {
                self.settings.privacy_max_ttl = match self.settings.privacy_max_ttl {
                    Some(0) | None => None,
                    Some(ttl) => Some(ttl - 1),
                };
            }
            Command::ExpandHosts => {
                self.settings.max_addrs = match self.settings.max_addrs {
                    None => Some(1),
                    Some(n) if Some(n) < self.max_hosts() => Some(n + 1),
                    Some(n) => Some(n),
                };
            }
            Command::ContractHosts => {
                self.settings.max_addrs = match self.settings.max_addrs {
                    Some(n) if n > 1 => Some(n - 1),
                    _ => None,
                };
            }
            Command::ExpandHostsMax => self.settings.max_addrs = self.max_hosts(),
            Command::ContractHostsMin => self.settings.max_addrs = Some(1),
            Command::ZoomIn => self.zoom = (self.zoom + 1).min(MAX_ZOOM),
            Command::ZoomOut => self.zoom = self.zoom.saturating_sub(1).max(1),
            Command::ClearTraceData => {
                if let Some(trace) = self.traces.get_mut(self.selected_trace) {
                    trace.clear();
                }
                self.selection.clear();
            }
            Command::ClearDnsCache => self.resolver.flush(),
            Command::ToggleAsInfo => {
                if self.trace_config.dns_resolve_method != ResolveMethod::System {
                    self.settings.lookup_as_info = !self.settings.lookup_as_info;
                    self.resolver.flush();
                } else {
                    self.notice = Some(String::from("AS lookup needs a non-system resolver"));
                }
            }
            Command::Input(input) => self.input_command(input),
        }
    }

    fn input_command(&mut self, input: InputCommand) {
        match (self.prompt, input) {
            (None, InputCommand::Enter) => self.prompt = Some(Prompt::Search),
            (None, InputCommand::Backspace) => {
                if !self.search.is_empty() {
                    self.prompt = Some(Prompt::Search);
                    self.search.pop();
                }
            }
            (None, _) => {}
            (Some(Prompt::Search), InputCommand::Handle(text)) => self.search.push_str(&text),
            (Some(Prompt::Search), InputCommand::Backspace) => {
                if self.search.pop().is_none() {
                    self.prompt = None;
                }
            }
            (Some(Prompt::Search), InputCommand::Confirm) => self.prompt = None,
            (Some(Prompt::Search), InputCommand::Exit) => {
                self.prompt = None;
                self.search.clear();
            }
            (Some(Prompt::Search), InputCommand::Enter) => {}
        }
    }

    /// Opens the settings dialog on a tab.
    pub fn open_settings_tab(&mut self, tab: usize) {
        let tab = SettingsTab::ALL[tab.min(SettingsTab::ALL.len() - 1)];
        if tab != self.settings_tab {
            self.settings_item = 0;
        }
        self.settings_tab = tab;
        self.show_settings = true;
    }

    fn move_settings_item(&mut self, direction: i64, amount: usize) {
        let count = self.settings_tab.count();
        if count == 0 {
            return;
        }
        let amount = amount as i64;
        let index = self.settings_item as i64 + direction * amount;
        self.settings_item = index.clamp(0, count as i64 - 1) as usize;
    }

    /// Applies the selected settings item (Enter, or a click).
    pub fn activate_setting(&mut self) {
        match self.settings_tab {
            SettingsTab::Tui => match self.settings_item {
                0 => {
                    self.settings.address_mode = next(AddressMode::ALL, self.settings.address_mode)
                }
                1 => {
                    self.settings.as_mode = next(crate::config::AsMode::ALL, self.settings.as_mode)
                }
                2 => {
                    self.settings.icmp_extension_mode = next(
                        crate::config::IcmpExtensionMode::ALL,
                        self.settings.icmp_extension_mode,
                    );
                }
                3 => {
                    self.settings.geoip_mode =
                        next(crate::config::GeoIpMode::ALL, self.settings.geoip_mode);
                }
                4 => {
                    let max = self.max_hosts();
                    self.settings.max_addrs = cycle_max_addrs(self.settings.max_addrs, max);
                }
                5 => {
                    let hops = u8::try_from(self.hops().len()).unwrap_or(u8::MAX);
                    self.settings.privacy_max_ttl =
                        cycle_privacy(self.settings.privacy_max_ttl, hops);
                }
                6 => self.open_settings_tab(SettingsTab::Columns as usize),
                _ => {}
            },
            SettingsTab::Dns if self.settings_item == 4 => {
                if self.trace_config.dns_resolve_method != ResolveMethod::System {
                    self.settings.lookup_as_info = !self.settings.lookup_as_info;
                    self.resolver.flush();
                }
            }
            SettingsTab::Theme => {
                if let Some(slot) = theme::Slot::ALL.get(self.settings_item) {
                    self.settings.theme.overrides.remove(slot);
                }
            }
            SettingsTab::Columns => self.settings.columns.toggle(self.settings_item),
            _ => {}
        }
    }

    /// Selects a trace by index (mouse click on a tab).
    pub fn select_trace(&mut self, index: usize) {
        if self.traces.is_empty() {
            return;
        }
        let index = index.min(self.traces.len() - 1);
        if index != self.selected_trace {
            self.selected_trace = index;
            self.selected_flow = FlowId(0);
            self.selection.clear();
            self.selected_hop_address = 0;
            self.show_flows = false;
            self.refresh_rows();
        }
    }

    fn move_hop_address(&mut self, amount: usize, forward: bool) {
        let count = self.selected_hop().map_or(0, Hop::addr_count);
        if count == 0 {
            return;
        }
        self.selected_hop_address = if forward {
            (self.selected_hop_address + amount).min(count - 1)
        } else {
            self.selected_hop_address.saturating_sub(amount)
        };
    }

    /// Shows or hides the hop details; detail mode shows one address per hop,
    /// as in the terminal frontend.
    fn toggle_hop_details(&mut self) {
        self.settings.max_addrs = if self.settings.show_hop_details {
            None
        } else {
            Some(1)
        };
        self.settings.show_hop_details = !self.settings.show_hop_details;
    }

    fn toggle_flows(&mut self) {
        let max_flows = self.trace().map_or(1, Trace::max_flows);
        if self.traces.len() != 1 || max_flows <= 1 {
            self.notice = Some(String::from(
                "Flows need a single target and the paris or dublin strategy",
            ));
            return;
        }
        if self.show_flows {
            self.selected_flow = FlowId(0);
            self.show_flows = false;
        } else if !self.flows().is_empty() {
            self.selected_flow = FlowId(1);
            self.show_flows = true;
        }
        self.selected_hop_address = 0;
        self.flows_start_index = 0;
    }

    /// The flows to show in the flows view, most rounds first.
    pub fn flows(&self) -> Vec<FlowId> {
        let Some(trace) = self.trace() else {
            return Vec::new();
        };
        let mut flows: Vec<(FlowId, usize)> = trace
            .snapshot
            .flows()
            .iter()
            .map(|&(_, id)| (id, trace.snapshot.round_count(id)))
            .collect();
        flows.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.cmp(&a.0)));
        flows
            .into_iter()
            .map(|(id, _)| id)
            .take(trace.max_flows())
            .collect()
    }

    fn move_flow(&mut self, amount: usize, forward: bool) {
        let flows = self.flows();
        let Some(index) = flows.iter().position(|id| *id == self.selected_flow) else {
            return;
        };
        self.selected_flow = if forward {
            flows[(index + amount).min(flows.len() - 1)]
        } else {
            flows[index.saturating_sub(amount)]
        };
        self.selection.clear();
        self.selected_hop_address = 0;
    }

    /// What `y` copies: the selected hop as a table row, or its details.
    pub fn selection_text(&self) -> Option<String> {
        let hop = self.selected_hop()?;
        if self.settings.show_hop_details {
            return Some(format::details(
                hop,
                self.selected_hop_address,
                &self.resolver,
                &self.geoip,
                &self.settings,
            ));
        }
        let cells: Vec<String> = self
            .settings
            .columns
            .shown()
            .map(|column| format::cell(column, hop, &self.resolver, &self.geoip, &self.settings))
            .collect();
        (!cells.is_empty()).then(|| cells.join("\t"))
    }

    /// Probes that failed to send on the selected flow, with the total sent.
    pub fn failures(&self) -> (usize, usize) {
        let hops = self.hops();
        (
            hops.iter().map(Hop::total_failed).sum(),
            hops.iter().map(Hop::total_sent).sum(),
        )
    }

    /// Key hints for the status bar.
    pub fn key_bindings(&self) -> Vec<(&'static str, &'static str)> {
        if self.fatal.is_some() || self.traces.is_empty() {
            return vec![("q", "Quit")];
        }
        if self.show_help {
            return vec![("h/?/Esc", "Close")];
        }
        if self.show_settings {
            return vec![
                ("↑/↓", "Item"),
                ("←/→", "Tab"),
                ("Enter", "Change"),
                ("s/Esc", "Close"),
            ];
        }
        self.view_key_bindings()
    }

    /// The key hints of the current view, as listed by the help overlay.
    pub fn view_key_bindings(&self) -> Vec<(&'static str, &'static str)> {
        let mut bindings = vec![
            ("↑/↓", "Hop"),
            ("←/→", "Trace"),
            ("c", "Chart"),
            ("m", "Map"),
            ("d", "Details"),
            ("f", "Flows"),
            ("z", "AS info"),
            ("/", "Filter"),
            ("y", "Copy"),
            ("s", "Settings"),
            ("h", "Help"),
            ("q", "Quit"),
        ];
        if self.show_chart {
            bindings.insert(2, ("=/-", "Zoom"));
        }
        if self.traces.len() <= 1 {
            bindings.retain(|(keys, _)| *keys != "←/→");
        }
        bindings
    }
}

/// The next value in a cyclic list.
fn next<T: Copy + PartialEq>(all: &[T], current: T) -> T {
    let index = all.iter().position(|v| *v == current).unwrap_or(0);
    all[(index + 1) % all.len()]
}

fn cycle_max_addrs(current: Option<u8>, max: Option<u8>) -> Option<u8> {
    match (current, max) {
        (None, Some(_)) => Some(1),
        (Some(n), Some(max)) if n < max => Some(n + 1),
        _ => None,
    }
}

fn cycle_privacy(current: Option<u8>, hops: u8) -> Option<u8> {
    match current {
        None => Some(0),
        Some(ttl) if ttl < hops => Some(ttl + 1),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_tabs_cycle() {
        let count = SettingsTab::ALL.len();
        assert_eq!(SettingsTab::ALL[0], SettingsTab::Tui);
        assert_eq!(SettingsTab::ALL[count - 1], SettingsTab::Columns);
        for tab in SettingsTab::ALL {
            assert!(!tab.title().is_empty());
        }
    }

    #[test]
    fn cycling_helpers_wrap() {
        assert_eq!(next(AddressMode::ALL, AddressMode::Ip), AddressMode::Host);
        assert_eq!(next(AddressMode::ALL, AddressMode::Both), AddressMode::Ip);
        assert_eq!(cycle_max_addrs(None, Some(3)), Some(1));
        assert_eq!(cycle_max_addrs(Some(3), Some(3)), None);
        assert_eq!(cycle_privacy(None, 5), Some(0));
        assert_eq!(cycle_privacy(Some(5), 5), None);
    }
}
