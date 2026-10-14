//! The rows of the settings dialog as data: one list per tab, each row
//! knowing its label, its value and what Enter or the mouse does to it.
//!
//! The dialog renders these, and the keyboard commands act on them, so a tab
//! is described in exactly one place.

use crate::bindings::Item;
use crate::config::{AddressMode, AsMode, GeoIpMode, IcmpExtensionMode, Named};
use crate::state::{SettingsTab, State};
use crate::theme;
use egui::Color32;
use trippy_dns::ResolveMethod;

/// An action on the state, run from Enter or the mouse.
pub type Action = Box<dyn Fn(&mut State)>;

/// An action with a value picked by the mouse.
pub type Setter<T> = Box<dyn Fn(&mut State, T)>;

/// How a row is changed.
pub enum Control {
    /// A read-only value; Enter may still run an action.
    Text { activate: Option<Action> },
    /// One of a fixed list; Enter picks the next one.
    Choice {
        options: Vec<&'static str>,
        selected: usize,
        set: Setter<usize>,
    },
    /// On or off; Enter flips it.
    Toggle { on: bool, toggle: Action },
    /// A colour; the picker sets it and Enter resets it.
    Color {
        color: Color32,
        set: Setter<Color32>,
        reset: Action,
    },
}

/// One row of a settings tab.
pub struct SettingsItem {
    pub label: String,
    pub value: String,
    pub control: Control,
}

impl SettingsItem {
    fn text(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            control: Control::Text { activate: None },
        }
    }

    fn action(
        label: &str,
        value: impl Into<String>,
        activate: impl Fn(&mut State) + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            control: Control::Text {
                activate: Some(Box::new(activate)),
            },
        }
    }

    fn choice<T: Named + PartialEq>(
        label: &str,
        current: T,
        set: impl Fn(&mut State, usize) + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            value: current.name().into(),
            control: Control::Choice {
                options: T::ALL.iter().map(|v| v.name()).collect(),
                selected: T::ALL.iter().position(|v| *v == current).unwrap_or(0),
                set: Box::new(set),
            },
        }
    }

    fn toggle(label: impl Into<String>, on: bool, toggle: impl Fn(&mut State) + 'static) -> Self {
        Self {
            label: label.into(),
            value: on.to_string(),
            control: Control::Toggle {
                on,
                toggle: Box::new(toggle),
            },
        }
    }

    /// Runs what Enter does on this row.
    pub fn activate(&self, state: &mut State) {
        match &self.control {
            Control::Text { activate } => {
                if let Some(activate) = activate {
                    activate(state);
                }
            }
            Control::Choice {
                options,
                selected,
                set,
            } => set(state, (selected + 1) % options.len().max(1)),
            Control::Toggle { toggle, .. } => toggle(state),
            Control::Color { reset, .. } => reset(state),
        }
    }
}

impl State {
    /// The rows of a settings tab.
    pub fn settings_items(&self, tab: SettingsTab) -> Vec<SettingsItem> {
        match tab {
            SettingsTab::Tui => self.tui_items(),
            SettingsTab::Trace => self.trace_items(),
            SettingsTab::Dns => self.dns_items(),
            SettingsTab::GeoIp => vec![SettingsItem::text(
                "geoip-mmdb-file",
                self.trace_config
                    .geoip_mmdb_file
                    .clone()
                    .unwrap_or_else(|| String::from("none")),
            )],
            SettingsTab::Bindings => Item::ALL
                .into_iter()
                .map(|item| SettingsItem::text(item.name(), self.bindings.display(item)))
                .collect(),
            SettingsTab::Theme => self.theme_items(),
            SettingsTab::Columns => self
                .settings
                .columns
                .all()
                .iter()
                .enumerate()
                .map(|(i, entry)| {
                    SettingsItem::toggle(entry.column.name(), entry.shown, move |state| {
                        state.settings.columns.toggle(i);
                    })
                })
                .collect(),
        }
    }

    /// How many rows a tab has.
    pub fn settings_count(&self, tab: SettingsTab) -> usize {
        self.settings_items(tab).len()
    }

    fn tui_items(&self) -> Vec<SettingsItem> {
        let s = &self.settings;
        vec![
            SettingsItem::choice("tui-address-mode", s.address_mode, |state, i| {
                state.settings.address_mode = AddressMode::ALL[i];
            }),
            SettingsItem::choice("tui-as-mode", s.as_mode, |state, i| {
                state.settings.as_mode = AsMode::ALL[i];
            }),
            SettingsItem::choice(
                "tui-icmp-extension-mode",
                s.icmp_extension_mode,
                |state, i| {
                    state.settings.icmp_extension_mode = IcmpExtensionMode::ALL[i];
                },
            ),
            SettingsItem::choice("tui-geoip-mode", s.geoip_mode, |state, i| {
                state.settings.geoip_mode = GeoIpMode::ALL[i];
            }),
            SettingsItem::action(
                "tui-max-addrs",
                s.max_addrs
                    .map_or_else(|| String::from("auto"), |n| n.to_string()),
                |state| {
                    let max = state.max_hosts();
                    state.settings.max_addrs = cycle_max_addrs(state.settings.max_addrs, max);
                },
            ),
            SettingsItem::action(
                "tui-privacy-max-ttl",
                s.privacy_max_ttl
                    .map_or_else(|| String::from("off"), |n| n.to_string()),
                |state| {
                    let hops = u8::try_from(state.hops().len()).unwrap_or(u8::MAX);
                    state.settings.privacy_max_ttl =
                        cycle_privacy(state.settings.privacy_max_ttl, hops);
                },
            ),
            SettingsItem::action("tui-custom-columns", s.columns.to_codes(), |state| {
                state.open_settings_tab(SettingsTab::Columns as usize);
            }),
            SettingsItem::text(
                "tui-refresh-rate",
                humantime::format_duration(self.refresh_rate).to_string(),
            ),
            SettingsItem::text(
                "tui-timezone",
                s.timezone
                    .map_or_else(|| String::from("auto"), |tz| tz.to_string()),
            ),
        ]
    }

    fn trace_items(&self) -> Vec<SettingsItem> {
        let c = &self.trace_config;
        let duration = |d| humantime::format_duration(d).to_string();
        let port = |p: Option<trippy_core::Port>| {
            p.map_or_else(|| String::from("auto"), |p| p.0.to_string())
        };
        let (src, dest) = match c.port_direction {
            trippy_core::PortDirection::None => (String::from("n/a"), String::from("n/a")),
            direction => (port(direction.src()), port(direction.dest())),
        };
        vec![
            SettingsItem::text("first-ttl", c.first_ttl.to_string()),
            SettingsItem::text("max-ttl", c.max_ttl.to_string()),
            SettingsItem::text("min-round-duration", duration(c.min_round_duration)),
            SettingsItem::text("max-round-duration", duration(c.max_round_duration)),
            SettingsItem::text("grace-duration", duration(c.grace_duration)),
            SettingsItem::text("max-inflight", c.max_inflight.to_string()),
            SettingsItem::text("initial-sequence", c.initial_sequence.to_string()),
            SettingsItem::text("read-timeout", duration(c.read_timeout)),
            SettingsItem::text("packet-size", c.packet_size.to_string()),
            SettingsItem::text("payload-pattern", c.payload_pattern.to_string()),
            SettingsItem::text("tos", c.tos.to_string()),
            SettingsItem::text("icmp-extensions", c.icmp_extensions.to_string()),
            SettingsItem::text(
                "interface",
                c.interface.clone().unwrap_or_else(|| String::from("auto")),
            ),
            SettingsItem::text("multipath-strategy", c.multipath_strategy.to_string()),
            SettingsItem::text("target-port", dest),
            SettingsItem::text("source-port", src),
            SettingsItem::text("max-samples", c.max_samples.to_string()),
            SettingsItem::text("max-flows", c.max_flows().to_string()),
        ]
    }

    fn dns_items(&self) -> Vec<SettingsItem> {
        let c = &self.trace_config;
        let duration = |d| humantime::format_duration(d).to_string();
        vec![
            SettingsItem::text("dns-timeout", duration(c.dns_timeout)),
            SettingsItem::text("dns-ttl", duration(c.dns_ttl)),
            SettingsItem::text(
                "dns-resolve-method",
                format!("{:?}", c.dns_resolve_method).to_lowercase(),
            ),
            SettingsItem::text("dns-resolve-all", c.dns_resolve_all.to_string()),
            SettingsItem::toggle(
                "dns-lookup-as-info",
                self.settings.lookup_as_info,
                |state| {
                    if state.trace_config.dns_resolve_method == ResolveMethod::System {
                        state.notice = Some(String::from("AS lookup needs a non-system resolver"));
                    } else {
                        state.settings.lookup_as_info = !state.settings.lookup_as_info;
                        state.resolver.flush();
                    }
                },
            ),
        ]
    }

    fn theme_items(&self) -> Vec<SettingsItem> {
        let t = &self.settings.theme;
        let mut items = vec![
            SettingsItem {
                label: String::from("palette"),
                value: t.family.name().into(),
                control: Control::Choice {
                    options: theme::Family::ALL.iter().map(|f| f.name()).collect(),
                    selected: theme::Family::ALL
                        .iter()
                        .position(|f| *f == t.family)
                        .unwrap_or(0),
                    set: Box::new(|state, i| state.settings.theme.family = theme::Family::ALL[i]),
                },
            },
            SettingsItem {
                label: String::from("mode"),
                value: t.mode_name().into(),
                control: Control::Choice {
                    options: theme::MODES.iter().map(|(_, name)| *name).collect(),
                    selected: theme::MODES
                        .iter()
                        .position(|(mode, _)| *mode == t.mode)
                        .unwrap_or(0),
                    set: Box::new(|state, i| state.settings.theme.mode = theme::MODES[i].0),
                },
            },
        ];
        let dark = self.dark_mode;
        items.extend(theme::Slot::ALL.iter().map(|slot| {
            let slot = *slot;
            SettingsItem {
                label: slot.name().into(),
                value: String::new(),
                control: Control::Color {
                    color: t.slot(slot, dark),
                    set: Box::new(move |state, color| state.settings.theme.set_slot(slot, color)),
                    reset: Box::new(move |state| {
                        state.settings.theme.overrides.remove(&slot);
                    }),
                },
            }
        }));
        items
    }
}

/// The next `max-addrs` value: auto, 1, .., the most addresses at any hop.
fn cycle_max_addrs(current: Option<u8>, max: Option<u8>) -> Option<u8> {
    match (current, max) {
        (None, Some(_)) => Some(1),
        (Some(n), Some(max)) if n < max => Some(n + 1),
        _ => None,
    }
}

/// The next `privacy-max-ttl` value: off, 0, .., the number of hops.
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
    fn cycling_helpers_wrap() {
        assert_eq!(cycle_max_addrs(None, Some(3)), Some(1));
        assert_eq!(cycle_max_addrs(Some(3), Some(3)), None);
        assert_eq!(cycle_max_addrs(None, None), None);
        assert_eq!(cycle_privacy(None, 5), Some(0));
        assert_eq!(cycle_privacy(Some(5), 5), None);
    }
}
