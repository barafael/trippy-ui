//! The configurable key map: trippy's commands, keys and syntax, on egui keys.
//!
//! Every command trippy can rebind (`--tui-key-bindings` and the `[bindings]`
//! section) is an [`Item`]; the window adds a few fixed keys of its own on top,
//! see [`Bindings::lookup`].

use crate::command::{Command, InputCommand, ScrollType};
use anyhow::anyhow;
use egui::{Key, Modifiers};
use std::fmt::{Display, Formatter};

/// A command trippy can bind a key to, in trippy's order and names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Item {
    ToggleHelp,
    ToggleHelpAlt,
    ToggleSettings,
    ToggleSettingsTui,
    ToggleSettingsTrace,
    ToggleSettingsDns,
    ToggleSettingsGeoip,
    ToggleSettingsBindings,
    ToggleSettingsTheme,
    ToggleSettingsColumns,
    NextHop,
    PreviousHop,
    NextTrace,
    PreviousTrace,
    NextHopAddress,
    PreviousHopAddress,
    AddressModeIp,
    AddressModeHost,
    AddressModeBoth,
    ToggleFreeze,
    ToggleChart,
    ToggleMap,
    ToggleFlows,
    ExpandPrivacy,
    ContractPrivacy,
    ExpandHosts,
    ContractHosts,
    ExpandHostsMax,
    ContractHostsMin,
    ChartZoomIn,
    ChartZoomOut,
    ClearTraceData,
    ClearDnsCache,
    ClearSelection,
    ToggleAsInfo,
    ToggleHopDetails,
    Quit,
    QuitPreserveScreen,
}

impl Item {
    /// Every item, in the order the settings list them.
    pub const ALL: [Item; 38] = [
        Item::ToggleHelp,
        Item::ToggleHelpAlt,
        Item::ToggleSettings,
        Item::ToggleSettingsTui,
        Item::ToggleSettingsTrace,
        Item::ToggleSettingsDns,
        Item::ToggleSettingsGeoip,
        Item::ToggleSettingsBindings,
        Item::ToggleSettingsTheme,
        Item::ToggleSettingsColumns,
        Item::NextHop,
        Item::PreviousHop,
        Item::NextTrace,
        Item::PreviousTrace,
        Item::NextHopAddress,
        Item::PreviousHopAddress,
        Item::AddressModeIp,
        Item::AddressModeHost,
        Item::AddressModeBoth,
        Item::ToggleFreeze,
        Item::ToggleChart,
        Item::ToggleMap,
        Item::ToggleFlows,
        Item::ExpandPrivacy,
        Item::ContractPrivacy,
        Item::ExpandHosts,
        Item::ContractHosts,
        Item::ExpandHostsMax,
        Item::ContractHostsMin,
        Item::ChartZoomIn,
        Item::ChartZoomOut,
        Item::ClearTraceData,
        Item::ClearDnsCache,
        Item::ClearSelection,
        Item::ToggleAsInfo,
        Item::ToggleHopDetails,
        Item::Quit,
        Item::QuitPreserveScreen,
    ];

    /// The name used on the command line and in the config file.
    pub const fn name(self) -> &'static str {
        match self {
            Item::ToggleHelp => "toggle-help",
            Item::ToggleHelpAlt => "toggle-help-alt",
            Item::ToggleSettings => "toggle-settings",
            Item::ToggleSettingsTui => "toggle-settings-tui",
            Item::ToggleSettingsTrace => "toggle-settings-trace",
            Item::ToggleSettingsDns => "toggle-settings-dns",
            Item::ToggleSettingsGeoip => "toggle-settings-geoip",
            Item::ToggleSettingsBindings => "toggle-settings-bindings",
            Item::ToggleSettingsTheme => "toggle-settings-theme",
            Item::ToggleSettingsColumns => "toggle-settings-columns",
            Item::NextHop => "next-hop",
            Item::PreviousHop => "previous-hop",
            Item::NextTrace => "next-trace",
            Item::PreviousTrace => "previous-trace",
            Item::NextHopAddress => "next-hop-address",
            Item::PreviousHopAddress => "previous-hop-address",
            Item::AddressModeIp => "address-mode-ip",
            Item::AddressModeHost => "address-mode-host",
            Item::AddressModeBoth => "address-mode-both",
            Item::ToggleFreeze => "toggle-freeze",
            Item::ToggleChart => "toggle-chart",
            Item::ToggleMap => "toggle-map",
            Item::ToggleFlows => "toggle-flows",
            Item::ExpandPrivacy => "expand-privacy",
            Item::ContractPrivacy => "contract-privacy",
            Item::ExpandHosts => "expand-hosts",
            Item::ContractHosts => "contract-hosts",
            Item::ExpandHostsMax => "expand-hosts-max",
            Item::ContractHostsMin => "contract-hosts-min",
            Item::ChartZoomIn => "chart-zoom-in",
            Item::ChartZoomOut => "chart-zoom-out",
            Item::ClearTraceData => "clear-trace-data",
            Item::ClearDnsCache => "clear-dns-cache",
            Item::ClearSelection => "clear-selection",
            Item::ToggleAsInfo => "toggle-as-info",
            Item::ToggleHopDetails => "toggle-hop-details",
            Item::Quit => "quit",
            Item::QuitPreserveScreen => "quit-preserve-screen",
        }
    }

    /// A short description for the help overlay.
    pub const fn description(self) -> &'static str {
        match self {
            Item::ToggleHelp | Item::ToggleHelpAlt => "Help",
            Item::ToggleSettings => "Settings",
            Item::ToggleSettingsTui => "Settings: Tui",
            Item::ToggleSettingsTrace => "Settings: Trace",
            Item::ToggleSettingsDns => "Settings: DNS",
            Item::ToggleSettingsGeoip => "Settings: GeoIp",
            Item::ToggleSettingsBindings => "Settings: Bindings",
            Item::ToggleSettingsTheme => "Settings: Theme",
            Item::ToggleSettingsColumns => "Settings: Columns",
            Item::NextHop => "Next hop",
            Item::PreviousHop => "Previous hop",
            Item::NextTrace => "Next trace or flow",
            Item::PreviousTrace => "Previous trace or flow",
            Item::NextHopAddress => "Next address of the hop",
            Item::PreviousHopAddress => "Previous address of the hop",
            Item::AddressModeIp => "Show IP addresses",
            Item::AddressModeHost => "Show hostnames",
            Item::AddressModeBoth => "Show hostnames and addresses",
            Item::ToggleFreeze => "Freeze the display",
            Item::ToggleChart => "Chart",
            Item::ToggleMap => "Map",
            Item::ToggleFlows => "Flows",
            Item::ExpandPrivacy => "Hide one more hop",
            Item::ContractPrivacy => "Hide one hop fewer",
            Item::ExpandHosts => "One more address per hop",
            Item::ContractHosts => "One address fewer per hop",
            Item::ExpandHostsMax => "All addresses per hop",
            Item::ContractHostsMin => "One address per hop",
            Item::ChartZoomIn => "Zoom the chart in",
            Item::ChartZoomOut => "Zoom the chart out",
            Item::ClearTraceData => "Clear the trace data",
            Item::ClearDnsCache => "Clear the DNS cache",
            Item::ClearSelection => "Clear the selection",
            Item::ToggleAsInfo => "AS information",
            Item::ToggleHopDetails => "Hop details",
            Item::Quit => "Quit",
            Item::QuitPreserveScreen => "Quit (preserve screen)",
        }
    }

    /// Looks an item up by name.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|item| item.name() == name)
    }

    /// The command the item runs.
    pub const fn command(self) -> Command {
        match self {
            Item::ToggleHelp | Item::ToggleHelpAlt => Command::ShowHelp,
            Item::ToggleSettings => Command::ShowSettings,
            Item::ToggleSettingsTui => Command::ShowSettingsTab(0),
            Item::ToggleSettingsTrace => Command::ShowSettingsTab(1),
            Item::ToggleSettingsDns => Command::ShowSettingsTab(2),
            Item::ToggleSettingsGeoip => Command::ShowSettingsTab(3),
            Item::ToggleSettingsBindings => Command::ShowSettingsTab(4),
            Item::ToggleSettingsTheme => Command::ShowSettingsTab(5),
            Item::ToggleSettingsColumns => Command::ShowSettingsTab(6),
            Item::NextHop => Command::Next(ScrollType::List, 1),
            Item::PreviousHop => Command::Previous(ScrollType::List, 1),
            Item::NextTrace => Command::Next(ScrollType::Tab, 1),
            Item::PreviousTrace => Command::Previous(ScrollType::Tab, 1),
            Item::NextHopAddress => Command::Next(ScrollType::Address, 1),
            Item::PreviousHopAddress => Command::Previous(ScrollType::Address, 1),
            Item::AddressModeIp => Command::AddressMode(crate::config::AddressMode::Ip),
            Item::AddressModeHost => Command::AddressMode(crate::config::AddressMode::Host),
            Item::AddressModeBoth => Command::AddressMode(crate::config::AddressMode::Both),
            Item::ToggleFreeze => Command::ToggleFreeze,
            Item::ToggleChart => Command::ToggleChart,
            Item::ToggleMap => Command::ToggleMap,
            Item::ToggleFlows => Command::ToggleFlows,
            Item::ExpandPrivacy => Command::ExpandPrivacy,
            Item::ContractPrivacy => Command::ContractPrivacy,
            Item::ExpandHosts => Command::ExpandHosts,
            Item::ContractHosts => Command::ContractHosts,
            Item::ExpandHostsMax => Command::ExpandHostsMax,
            Item::ContractHostsMin => Command::ContractHostsMin,
            Item::ChartZoomIn => Command::ZoomIn,
            Item::ChartZoomOut => Command::ZoomOut,
            Item::ClearTraceData => Command::ClearTraceData,
            Item::ClearDnsCache => Command::ClearDnsCache,
            Item::ClearSelection => Command::ClearSelection,
            Item::ToggleAsInfo => Command::ToggleAsInfo,
            Item::ToggleHopDetails => Command::ToggleHopDetails,
            Item::Quit | Item::QuitPreserveScreen => Command::Exit,
        }
    }

    const fn default_binding(self) -> KeyBinding {
        let plain = KeyBinding::plain;
        match self {
            Item::ToggleHelp => plain(Key::H),
            Item::ToggleHelpAlt => plain(Key::Questionmark),
            Item::ToggleSettings => plain(Key::S),
            Item::ToggleSettingsTui => plain(Key::Num1),
            Item::ToggleSettingsTrace => plain(Key::Num2),
            Item::ToggleSettingsDns => plain(Key::Num3),
            Item::ToggleSettingsGeoip => plain(Key::Num4),
            Item::ToggleSettingsBindings => plain(Key::Num5),
            Item::ToggleSettingsTheme => plain(Key::Num6),
            Item::ToggleSettingsColumns => plain(Key::Num7),
            Item::NextHop => plain(Key::ArrowDown),
            Item::PreviousHop => plain(Key::ArrowUp),
            Item::NextTrace => plain(Key::ArrowRight),
            Item::PreviousTrace => plain(Key::ArrowLeft),
            Item::NextHopAddress => plain(Key::Period),
            Item::PreviousHopAddress => plain(Key::Comma),
            Item::AddressModeIp => plain(Key::I),
            Item::AddressModeHost => plain(Key::N),
            Item::AddressModeBoth => plain(Key::B),
            Item::ToggleFreeze => KeyBinding::ctrl(Key::F),
            Item::ToggleChart => plain(Key::C),
            Item::ToggleMap => plain(Key::M),
            Item::ToggleFlows => plain(Key::F),
            Item::ExpandPrivacy => plain(Key::P),
            Item::ContractPrivacy => plain(Key::O),
            Item::ExpandHosts => plain(Key::CloseBracket),
            Item::ContractHosts => plain(Key::OpenBracket),
            Item::ExpandHostsMax => plain(Key::CloseCurlyBracket),
            Item::ContractHostsMin => plain(Key::OpenCurlyBracket),
            Item::ChartZoomIn => plain(Key::Equals),
            Item::ChartZoomOut => plain(Key::Minus),
            Item::ClearTraceData => KeyBinding::ctrl(Key::R),
            Item::ClearDnsCache => KeyBinding::ctrl(Key::K),
            Item::ClearSelection => plain(Key::Escape),
            Item::ToggleAsInfo => plain(Key::Z),
            Item::ToggleHopDetails => plain(Key::D),
            Item::Quit => plain(Key::Q),
            Item::QuitPreserveScreen => KeyBinding::shift(Key::Q),
        }
    }
}

/// A key with its modifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyBinding {
    pub key: Key,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

impl KeyBinding {
    /// A key with no modifiers.
    pub const fn plain(key: Key) -> Self {
        Self {
            key,
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        }
    }

    /// A key with Ctrl (Command on macOS).
    pub const fn ctrl(key: Key) -> Self {
        Self {
            ctrl: true,
            ..Self::plain(key)
        }
    }

    /// A key with Shift.
    pub const fn shift(key: Key) -> Self {
        Self {
            shift: true,
            ..Self::plain(key)
        }
    }

    /// Whether a key press is this binding.
    ///
    /// Ctrl, Alt and the platform command key must match. Shift must match
    /// for letters and named keys (so `q` and `Q` differ), and is ignored for
    /// digits and symbols because keyboard layouts produce those with or
    /// without it.
    pub fn matches(&self, key: Key, modifiers: Modifiers) -> bool {
        key == self.key
            && (modifiers.ctrl || modifiers.command) == self.ctrl
            && modifiers.alt == self.alt
            && modifiers.mac_cmd == self.meta
            && (!shift_matters(key) || modifiers.shift == self.shift)
    }

    /// Parses trippy's syntax: `[modifier+]*key`, with `shift`, `ctrl`, `alt`
    /// and `super`, a single character, or a named key such as `esc`.
    pub fn parse(text: &str) -> anyhow::Result<Self> {
        let mut binding = Self::plain(Key::Escape);
        let mut parts = text.split('+').collect::<Vec<_>>();
        // A trailing `+` is the plus key itself.
        if text.ends_with('+') {
            parts.pop();
            parts.pop();
            parts.push("+");
        }
        let Some((key, modifiers)) = parts.split_last() else {
            return Err(anyhow!("empty key binding"));
        };
        for modifier in modifiers {
            match modifier.to_ascii_lowercase().as_str() {
                "shift" => binding.shift = true,
                "ctrl" => binding.ctrl = true,
                "alt" => binding.alt = true,
                "super" | "meta" | "hyper" => binding.meta = true,
                other => return Err(anyhow!("unknown modifier '{other}'")),
            }
        }
        let (key, shifted) = parse_key(key)?;
        binding.key = key;
        binding.shift |= shifted;
        Ok(binding)
    }
}

/// Whether shift distinguishes two bindings on this key.
const fn shift_matters(key: Key) -> bool {
    !matches!(
        key,
        Key::Colon
            | Key::Comma
            | Key::Backslash
            | Key::Slash
            | Key::Pipe
            | Key::Questionmark
            | Key::Exclamationmark
            | Key::OpenBracket
            | Key::CloseBracket
            | Key::OpenCurlyBracket
            | Key::CloseCurlyBracket
            | Key::Backtick
            | Key::Minus
            | Key::Period
            | Key::Plus
            | Key::Equals
            | Key::Semicolon
            | Key::Quote
            | Key::Num0
            | Key::Num1
            | Key::Num2
            | Key::Num3
            | Key::Num4
            | Key::Num5
            | Key::Num6
            | Key::Num7
            | Key::Num8
            | Key::Num9
    )
}

/// The key a name stands for, and whether the name implies shift (`backtab`,
/// an upper-case letter).
fn parse_key(name: &str) -> anyhow::Result<(Key, bool)> {
    let named = match name.to_ascii_lowercase().as_str() {
        "backspace" => Some(Key::Backspace),
        "enter" => Some(Key::Enter),
        "left" => Some(Key::ArrowLeft),
        "right" => Some(Key::ArrowRight),
        "up" => Some(Key::ArrowUp),
        "down" => Some(Key::ArrowDown),
        "home" => Some(Key::Home),
        "end" => Some(Key::End),
        "pageup" => Some(Key::PageUp),
        "pagedown" => Some(Key::PageDown),
        "tab" => Some(Key::Tab),
        "backtab" => return Ok((Key::Tab, true)),
        "delete" => Some(Key::Delete),
        "insert" => Some(Key::Insert),
        "esc" => Some(Key::Escape),
        "space" => Some(Key::Space),
        _ => None,
    };
    if let Some(key) = named {
        return Ok((key, false));
    }
    let mut chars = name.chars();
    let (Some(c), None) = (chars.next(), chars.next()) else {
        return Err(anyhow!("unknown key binding '{name}'"));
    };
    let key = match c {
        '?' => Key::Questionmark,
        '!' => Key::Exclamationmark,
        '/' => Key::Slash,
        '\\' => Key::Backslash,
        '|' => Key::Pipe,
        '[' => Key::OpenBracket,
        ']' => Key::CloseBracket,
        '{' => Key::OpenCurlyBracket,
        '}' => Key::CloseCurlyBracket,
        '=' => Key::Equals,
        '-' => Key::Minus,
        '+' => Key::Plus,
        ',' => Key::Comma,
        '.' => Key::Period,
        ';' => Key::Semicolon,
        ':' => Key::Colon,
        '\'' => Key::Quote,
        '`' => Key::Backtick,
        ' ' => Key::Space,
        c if c.is_ascii_alphanumeric() => Key::from_name(&c.to_ascii_uppercase().to_string())
            .ok_or_else(|| anyhow!("unknown key binding '{name}'"))?,
        _ => return Err(anyhow!("unknown key binding '{name}'")),
    };
    Ok((key, c.is_ascii_uppercase()))
}

impl Display for KeyBinding {
    /// The form shown in the window: `Ctrl+F`, `Shift+Tab`, `↑`, `Esc`, `?`.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.ctrl {
            f.write_str("Ctrl+")?;
        }
        if self.alt {
            f.write_str("Alt+")?;
        }
        if self.meta {
            f.write_str("Super+")?;
        }
        let name = key_name(self.key);
        if self.shift && shift_matters(self.key) {
            if name.len() == 1 {
                return f.write_str(&name.to_ascii_uppercase());
            }
            f.write_str("Shift+")?;
        }
        f.write_str(name)
    }
}

/// The display name of a key, lower-case for letters.
fn key_name(key: Key) -> &'static str {
    match key {
        Key::ArrowUp => "↑",
        Key::ArrowDown => "↓",
        Key::ArrowLeft => "←",
        Key::ArrowRight => "→",
        Key::PageUp => "PgUp",
        Key::PageDown => "PgDn",
        Key::Escape => "Esc",
        Key::Enter => "Enter",
        Key::Tab => "Tab",
        Key::Backspace => "Bksp",
        Key::Space => "Space",
        Key::Delete => "Del",
        Key::Insert => "Ins",
        Key::Home => "Home",
        Key::End => "End",
        Key::A => "a",
        Key::B => "b",
        Key::C => "c",
        Key::D => "d",
        Key::E => "e",
        Key::F => "f",
        Key::G => "g",
        Key::H => "h",
        Key::I => "i",
        Key::J => "j",
        Key::K => "k",
        Key::L => "l",
        Key::M => "m",
        Key::N => "n",
        Key::O => "o",
        Key::P => "p",
        Key::Q => "q",
        Key::R => "r",
        Key::S => "s",
        Key::T => "t",
        Key::U => "u",
        Key::V => "v",
        Key::W => "w",
        Key::X => "x",
        Key::Y => "y",
        Key::Z => "z",
        other => other.symbol_or_name(),
    }
}

/// The key map: a binding per [`Item`], plus the window's fixed keys.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bindings {
    items: [KeyBinding; Item::ALL.len()],
}

impl Default for Bindings {
    fn default() -> Self {
        let mut items = [KeyBinding::plain(Key::Escape); Item::ALL.len()];
        for (slot, item) in items.iter_mut().zip(Item::ALL) {
            *slot = item.default_binding();
        }
        Self { items }
    }
}

/// The window's own keys, consulted after the configurable ones.
const EXTRA: &[(KeyBinding, Command)] = &[
    (
        KeyBinding::shift(Key::Tab),
        Command::Previous(ScrollType::Tab, 1),
    ),
    (
        KeyBinding::plain(Key::Tab),
        Command::Next(ScrollType::Tab, 1),
    ),
    (KeyBinding::ctrl(Key::D), Command::Next(ScrollType::List, 5)),
    (
        KeyBinding::ctrl(Key::U),
        Command::Previous(ScrollType::List, 5),
    ),
    (
        KeyBinding::plain(Key::J),
        Command::Next(ScrollType::List, 1),
    ),
    (
        KeyBinding::plain(Key::K),
        Command::Previous(ScrollType::List, 1),
    ),
    (KeyBinding::plain(Key::L), Command::Next(ScrollType::Tab, 1)),
    (
        KeyBinding::plain(Key::PageDown),
        Command::Next(ScrollType::List, 5),
    ),
    (
        KeyBinding::plain(Key::PageUp),
        Command::Previous(ScrollType::List, 5),
    ),
    (KeyBinding::plain(Key::Home), Command::Top),
    (KeyBinding::plain(Key::End), Command::Bottom),
    (KeyBinding::plain(Key::Plus), Command::ZoomIn),
    (KeyBinding::plain(Key::Enter), Command::Activate),
    (KeyBinding::plain(Key::Space), Command::Activate),
    (KeyBinding::plain(Key::Y), Command::Copy),
    (
        KeyBinding::plain(Key::Slash),
        Command::Input(InputCommand::Enter),
    ),
    (
        KeyBinding::plain(Key::Backspace),
        Command::Input(InputCommand::Backspace),
    ),
];

/// The window's own keys as `(keys, description)` for the help overlay.
pub const EXTRA_HELP: &[(&str, &str)] = &[
    ("Tab / Shift+Tab", "Next / previous trace or flow"),
    ("j / k", "Next / previous hop"),
    ("PgDn / PgUp, Ctrl+D / Ctrl+U", "Move five hops"),
    ("Home / End", "First / last hop"),
    (
        "Enter / Space",
        "Hop details, or change the selected setting",
    ),
    ("y", "Copy the selected row"),
    (
        "/",
        "Filter the hops; Enter keeps the filter, Esc clears it",
    ),
    ("+", "Zoom the chart in"),
    ("Ctrl+C", "Quit"),
];

impl Bindings {
    /// The default map with trippy's `command=key` pairs applied on top.
    ///
    /// Fails on an unknown command or key, or when two commands share a key.
    pub fn from_items(items: &[(String, String)]) -> anyhow::Result<Self> {
        let mut bindings = Self::default();
        for (name, key) in items {
            let item = Item::from_name(name)
                .ok_or_else(|| anyhow!("unknown key binding command '{name}'"))?;
            bindings.items[item as usize] = KeyBinding::parse(key)?;
        }
        let duplicates = bindings.duplicates();
        if duplicates.is_empty() {
            Ok(bindings)
        } else {
            Err(anyhow!("Duplicate key bindings: {}", duplicates.join(", ")))
        }
    }

    /// The binding of an item.
    pub const fn get(&self, item: Item) -> KeyBinding {
        self.items[item as usize]
    }

    /// Items that share a key with an earlier item, as `key: a, b`.
    fn duplicates(&self) -> Vec<String> {
        let mut out = Vec::new();
        for (i, item) in Item::ALL.iter().enumerate() {
            let binding = self.items[i];
            let same: Vec<&str> = Item::ALL[i + 1..]
                .iter()
                .filter(|other| self.get(**other) == binding)
                .map(|other| other.name())
                .collect();
            if !same.is_empty()
                && !out
                    .iter()
                    .any(|s: &String| s.contains(&binding.to_string()))
            {
                out.push(format!("{binding}: {}, {}", item.name(), same.join(", ")));
            }
        }
        out
    }

    /// The command of a key press, if any: the configurable items first, then
    /// the window's fixed keys.
    pub fn lookup(&self, key: Key, modifiers: Modifiers) -> Option<Command> {
        Item::ALL
            .iter()
            .find(|item| self.get(**item).matches(key, modifiers))
            .map(|item| item.command())
            .or_else(|| {
                EXTRA
                    .iter()
                    .find(|(binding, _)| binding.matches(key, modifiers))
                    .map(|(_, command)| command.clone())
            })
    }

    /// The display form of an item's key.
    pub fn display(&self, item: Item) -> String {
        self.get(item).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pairs(items: &[(&str, &str)]) -> Vec<(String, String)> {
        items
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect()
    }

    #[test]
    fn parses_trippy_syntax() {
        assert_eq!(
            KeyBinding::parse("ctrl+f").unwrap(),
            KeyBinding::ctrl(Key::F)
        );
        assert_eq!(KeyBinding::parse("Q").unwrap(), KeyBinding::shift(Key::Q));
        assert_eq!(
            KeyBinding::parse("shift+q").unwrap(),
            KeyBinding::shift(Key::Q)
        );
        assert_eq!(
            KeyBinding::parse("esc").unwrap(),
            KeyBinding::plain(Key::Escape)
        );
        assert_eq!(
            KeyBinding::parse("?").unwrap(),
            KeyBinding::plain(Key::Questionmark)
        );
        assert_eq!(
            KeyBinding::parse("ctrl++").unwrap(),
            KeyBinding::ctrl(Key::Plus)
        );
        assert_eq!(
            KeyBinding::parse("backtab").unwrap(),
            KeyBinding::shift(Key::Tab)
        );
        assert_eq!(
            KeyBinding::parse("alt+pagedown").unwrap(),
            KeyBinding {
                alt: true,
                ..KeyBinding::plain(Key::PageDown)
            }
        );
        assert!(KeyBinding::parse("hyper+x").is_ok());
        assert!(KeyBinding::parse("ctl+f").is_err());
        assert!(KeyBinding::parse("f13").is_err());
        assert!(KeyBinding::parse("").is_err());
    }

    #[test]
    fn displays_like_the_hints() {
        assert_eq!(KeyBinding::ctrl(Key::F).to_string(), "Ctrl+f");
        assert_eq!(KeyBinding::shift(Key::Q).to_string(), "Q");
        assert_eq!(KeyBinding::shift(Key::Tab).to_string(), "Shift+Tab");
        assert_eq!(KeyBinding::plain(Key::ArrowUp).to_string(), "↑");
        assert_eq!(KeyBinding::plain(Key::Questionmark).to_string(), "?");
        assert_eq!(KeyBinding::plain(Key::CloseCurlyBracket).to_string(), "}");
    }

    #[test]
    fn matching_ignores_shift_on_symbols_only() {
        let q = KeyBinding::plain(Key::Q);
        assert!(q.matches(Key::Q, Modifiers::NONE));
        assert!(!q.matches(Key::Q, Modifiers::SHIFT));
        assert!(!q.matches(Key::Q, Modifiers::CTRL));
        assert!(KeyBinding::shift(Key::Q).matches(Key::Q, Modifiers::SHIFT));
        let slash = KeyBinding::plain(Key::Slash);
        assert!(slash.matches(Key::Slash, Modifiers::SHIFT));
        assert!(KeyBinding::plain(Key::Num7).matches(Key::Num7, Modifiers::SHIFT));
        assert!(KeyBinding::ctrl(Key::F).matches(Key::F, Modifiers::COMMAND));
        assert!(KeyBinding::ctrl(Key::F).matches(Key::F, Modifiers::CTRL));
    }

    #[test]
    fn defaults_follow_trippy_and_the_window_adds_its_own() {
        let bindings = Bindings::default();
        assert_eq!(
            bindings.lookup(Key::H, Modifiers::NONE),
            Some(Command::ShowHelp)
        );
        assert_eq!(
            bindings.lookup(Key::Questionmark, Modifiers::SHIFT),
            Some(Command::ShowHelp)
        );
        assert_eq!(
            bindings.lookup(Key::F, Modifiers::CTRL),
            Some(Command::ToggleFreeze)
        );
        assert_eq!(
            bindings.lookup(Key::F, Modifiers::NONE),
            Some(Command::ToggleFlows)
        );
        assert_eq!(
            bindings.lookup(Key::Q, Modifiers::SHIFT),
            Some(Command::Exit)
        );
        assert_eq!(
            bindings.lookup(Key::CloseCurlyBracket, Modifiers::SHIFT),
            Some(Command::ExpandHostsMax)
        );
        assert_eq!(
            bindings.lookup(Key::Tab, Modifiers::SHIFT),
            Some(Command::Previous(ScrollType::Tab, 1))
        );
        assert_eq!(
            bindings.lookup(Key::J, Modifiers::NONE),
            Some(Command::Next(ScrollType::List, 1))
        );
        assert_eq!(bindings.lookup(Key::J, Modifiers::SHIFT), None);
        assert_eq!(
            bindings.lookup(Key::Slash, Modifiers::NONE),
            Some(Command::Input(InputCommand::Enter))
        );
        assert_eq!(bindings.lookup(Key::X, Modifiers::NONE), None);
    }

    #[test]
    fn rebinding_replaces_and_rejects_duplicates() {
        let bindings = Bindings::from_items(&pairs(&[
            ("toggle-chart", "x"),
            ("toggle-freeze", "ctrl+g"),
        ]))
        .unwrap();
        assert_eq!(
            bindings.lookup(Key::X, Modifiers::NONE),
            Some(Command::ToggleChart)
        );
        assert_eq!(bindings.lookup(Key::C, Modifiers::NONE), None);
        assert_eq!(
            bindings.lookup(Key::G, Modifiers::CTRL),
            Some(Command::ToggleFreeze)
        );
        assert_eq!(bindings.display(Item::ToggleChart), "x");
        let error = Bindings::from_items(&pairs(&[("toggle-chart", "m")])).unwrap_err();
        assert!(
            error.to_string().contains("Duplicate key bindings"),
            "{error}"
        );
        assert!(
            error.to_string().contains("toggle-chart, toggle-map"),
            "{error}"
        );
        assert!(Bindings::from_items(&pairs(&[("toggle-chart", "ctl+x")])).is_err());
        assert!(Bindings::from_items(&pairs(&[("no-such-command", "x")])).is_err());
    }
}
