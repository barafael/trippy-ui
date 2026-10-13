//! Keyboard commands and the key map.
//!
//! The key map follows trippy's terminal frontend, with the additions the
//! window needs (a filter prompt). Commands are read in the order the keys
//! were typed, so opening the prompt changes what the following keys mean.

use crate::config::AddressMode;
use egui::{Event, Key, Modifiers};

/// The kind of thing a `Next`/`Previous` command moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollType {
    /// The traces (or the settings tabs).
    Tab,
    /// The rows of the main list.
    List,
    /// The address of the selected hop.
    Address,
}

/// A command the application understands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Show or hide the help overlay.
    ShowHelp,
    /// Show or hide the settings dialog.
    ShowSettings,
    /// Show the settings dialog on a given tab.
    ShowSettingsTab(usize),
    /// Move forward.
    Next(ScrollType, usize),
    /// Move backward.
    Previous(ScrollType, usize),
    /// Go to the top.
    Top,
    /// Go to the bottom.
    Bottom,
    /// Pick how addresses are rendered.
    AddressMode(AddressMode),
    /// Freeze or resume the snapshots.
    ToggleFreeze,
    /// Show or hide the round-trip time chart.
    ToggleChart,
    /// Show or hide the world map.
    ToggleMap,
    /// Show or hide the flows view.
    ToggleFlows,
    /// Hide one more hop from the top.
    ExpandPrivacy,
    /// Hide one fewer hop from the top.
    ContractPrivacy,
    /// Show one more address per hop.
    ExpandHosts,
    /// Show one fewer address per hop.
    ContractHosts,
    /// Show every address per hop.
    ExpandHostsMax,
    /// Show a single address per hop.
    ContractHostsMin,
    /// Zoom the chart in.
    ZoomIn,
    /// Zoom the chart out.
    ZoomOut,
    /// Discard all tracing data.
    ClearTraceData,
    /// Flush the DNS cache.
    ClearDnsCache,
    /// Clear the selected hop.
    ClearSelection,
    /// Toggle autonomous system information.
    ToggleAsInfo,
    /// Toggle the hop details view.
    ToggleHopDetails,
    /// Copy the selected row to the clipboard.
    Copy,
    /// Activate the selected item (toggle a setting, show details).
    Activate,
    /// A prompt command.
    Input(InputCommand),
    /// Quit.
    Exit,
    /// Quit, keeping the last frame on screen where that is meaningful.
    ExitPreserveScreen,
}

/// A prompt command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputCommand {
    /// Open the filter prompt.
    Enter,
    /// Confirm the prompt.
    Confirm,
    /// Leave the prompt and clear it.
    Exit,
    /// Delete a character; outside the prompt, close or clear.
    Backspace,
    /// Text typed into the prompt.
    Handle(String),
}

impl Command {
    /// A short label, ignoring any step size, for the settings and the hints.
    pub fn label(&self) -> &'static str {
        match self {
            Self::ShowHelp => "toggle-help",
            Self::ShowSettings => "toggle-settings",
            Self::ShowSettingsTab(0) => "settings-tui",
            Self::ShowSettingsTab(1) => "settings-trace",
            Self::ShowSettingsTab(2) => "settings-dns",
            Self::ShowSettingsTab(3) => "settings-geoip",
            Self::ShowSettingsTab(4) => "settings-bindings",
            Self::ShowSettingsTab(5) => "settings-theme",
            Self::ShowSettingsTab(_) => "settings-columns",
            Self::Next(ScrollType::Tab, _) => "next-trace",
            Self::Previous(ScrollType::Tab, _) => "previous-trace",
            Self::Next(ScrollType::List, _) => "next-hop",
            Self::Previous(ScrollType::List, _) => "previous-hop",
            Self::Next(ScrollType::Address, _) => "next-hop-address",
            Self::Previous(ScrollType::Address, _) => "previous-hop-address",
            Self::Top => "top",
            Self::Bottom => "bottom",
            Self::AddressMode(AddressMode::Ip) => "address-mode-ip",
            Self::AddressMode(AddressMode::Host) => "address-mode-host",
            Self::AddressMode(AddressMode::Both) => "address-mode-both",
            Self::ToggleFreeze => "toggle-freeze",
            Self::ToggleChart => "toggle-chart",
            Self::ToggleMap => "toggle-map",
            Self::ToggleFlows => "toggle-flows",
            Self::ExpandPrivacy => "expand-privacy",
            Self::ContractPrivacy => "contract-privacy",
            Self::ExpandHosts => "expand-hosts",
            Self::ContractHosts => "contract-hosts",
            Self::ExpandHostsMax => "expand-hosts-max",
            Self::ContractHostsMin => "contract-hosts-min",
            Self::ZoomIn => "chart-zoom-in",
            Self::ZoomOut => "chart-zoom-out",
            Self::ClearTraceData => "clear-trace-data",
            Self::ClearDnsCache => "clear-dns-cache",
            Self::ClearSelection => "clear-selection",
            Self::ToggleAsInfo => "toggle-as-info",
            Self::ToggleHopDetails => "toggle-hop-details",
            Self::Copy => "copy",
            Self::Activate => "activate",
            Self::Input(InputCommand::Enter) => "search",
            Self::Input(_) => "prompt",
            Self::Exit => "quit",
            Self::ExitPreserveScreen => "quit-preserve-screen",
        }
    }
}

/// The normal-mode key map, checked in order.
///
/// `Modifiers::NONE` ignores shift when matching, so shifted and Ctrl bindings
/// come before the bare letters.
#[allow(clippy::too_many_lines)]
pub const BINDINGS: &[(Modifiers, Key, Command)] = &[
    (
        Modifiers::SHIFT,
        Key::Tab,
        Command::Previous(ScrollType::Tab, 1),
    ),
    (Modifiers::NONE, Key::Tab, Command::Next(ScrollType::Tab, 1)),
    (
        Modifiers::COMMAND,
        Key::D,
        Command::Next(ScrollType::List, 5),
    ),
    (
        Modifiers::COMMAND,
        Key::U,
        Command::Previous(ScrollType::List, 5),
    ),
    (Modifiers::COMMAND, Key::F, Command::ToggleFreeze),
    (Modifiers::COMMAND, Key::R, Command::ClearTraceData),
    (Modifiers::COMMAND, Key::K, Command::ClearDnsCache),
    (
        Modifiers::NONE,
        Key::ArrowRight,
        Command::Next(ScrollType::Tab, 1),
    ),
    (Modifiers::NONE, Key::L, Command::Next(ScrollType::Tab, 1)),
    (
        Modifiers::NONE,
        Key::ArrowLeft,
        Command::Previous(ScrollType::Tab, 1),
    ),
    (
        Modifiers::NONE,
        Key::ArrowDown,
        Command::Next(ScrollType::List, 1),
    ),
    (Modifiers::NONE, Key::J, Command::Next(ScrollType::List, 1)),
    (
        Modifiers::NONE,
        Key::ArrowUp,
        Command::Previous(ScrollType::List, 1),
    ),
    (
        Modifiers::NONE,
        Key::K,
        Command::Previous(ScrollType::List, 1),
    ),
    (
        Modifiers::NONE,
        Key::PageDown,
        Command::Next(ScrollType::List, 5),
    ),
    (
        Modifiers::NONE,
        Key::PageUp,
        Command::Previous(ScrollType::List, 5),
    ),
    (Modifiers::NONE, Key::Home, Command::Top),
    (Modifiers::NONE, Key::End, Command::Bottom),
    (
        Modifiers::NONE,
        Key::Period,
        Command::Next(ScrollType::Address, 1),
    ),
    (
        Modifiers::NONE,
        Key::Comma,
        Command::Previous(ScrollType::Address, 1),
    ),
    (Modifiers::NONE, Key::Plus, Command::ZoomIn),
    (Modifiers::NONE, Key::Equals, Command::ZoomIn),
    (Modifiers::NONE, Key::Minus, Command::ZoomOut),
    (
        Modifiers::NONE,
        Key::I,
        Command::AddressMode(AddressMode::Ip),
    ),
    (
        Modifiers::NONE,
        Key::N,
        Command::AddressMode(AddressMode::Host),
    ),
    (
        Modifiers::NONE,
        Key::B,
        Command::AddressMode(AddressMode::Both),
    ),
    (Modifiers::NONE, Key::C, Command::ToggleChart),
    (Modifiers::NONE, Key::M, Command::ToggleMap),
    (Modifiers::NONE, Key::F, Command::ToggleFlows),
    (Modifiers::NONE, Key::Z, Command::ToggleAsInfo),
    (Modifiers::NONE, Key::D, Command::ToggleHopDetails),
    (Modifiers::NONE, Key::Y, Command::Copy),
    (Modifiers::NONE, Key::Enter, Command::Activate),
    (Modifiers::NONE, Key::Space, Command::Activate),
    (Modifiers::NONE, Key::P, Command::ExpandPrivacy),
    (Modifiers::NONE, Key::O, Command::ContractPrivacy),
    (Modifiers::SHIFT, Key::CloseBracket, Command::ExpandHostsMax),
    (
        Modifiers::SHIFT,
        Key::OpenBracket,
        Command::ContractHostsMin,
    ),
    (Modifiers::NONE, Key::CloseBracket, Command::ExpandHosts),
    (Modifiers::NONE, Key::OpenBracket, Command::ContractHosts),
    (Modifiers::SHIFT, Key::Slash, Command::ShowHelp),
    (
        Modifiers::NONE,
        Key::Slash,
        Command::Input(InputCommand::Enter),
    ),
    (
        Modifiers::NONE,
        Key::Backspace,
        Command::Input(InputCommand::Backspace),
    ),
    (Modifiers::NONE, Key::H, Command::ShowHelp),
    (Modifiers::NONE, Key::S, Command::ShowSettings),
    (Modifiers::NONE, Key::Num1, Command::ShowSettingsTab(0)),
    (Modifiers::NONE, Key::Num2, Command::ShowSettingsTab(1)),
    (Modifiers::NONE, Key::Num3, Command::ShowSettingsTab(2)),
    (Modifiers::NONE, Key::Num4, Command::ShowSettingsTab(3)),
    (Modifiers::NONE, Key::Num5, Command::ShowSettingsTab(4)),
    (Modifiers::NONE, Key::Num6, Command::ShowSettingsTab(5)),
    (Modifiers::NONE, Key::Num7, Command::ShowSettingsTab(6)),
    (Modifiers::NONE, Key::Escape, Command::ClearSelection),
    (Modifiers::SHIFT, Key::Q, Command::ExitPreserveScreen),
    (Modifiers::NONE, Key::Q, Command::Exit),
];

/// This frame's keyboard events, in the order they were typed.
///
/// Each command must run before the next event is read, because the prompt it
/// may open or close decides what the following keys mean.
pub struct Keys {
    events: std::iter::Peekable<std::vec::IntoIter<Event>>,
    /// The previous event was this key and it ran a command. A key's text
    /// follows it, and the `/` that opened a prompt must not be typed into it.
    command_key: Option<Key>,
}

impl Keys {
    /// Takes the key presses, text and copy events out of this frame's input.
    pub fn take(ctx: &egui::Context) -> Self {
        let events = ctx.input_mut(|input| {
            let (keys, rest) = std::mem::take(&mut input.events)
                .into_iter()
                .partition(|event| {
                    matches!(
                        event,
                        Event::Key { pressed: true, .. } | Event::Text(_) | Event::Copy
                    )
                });
            input.events = rest;
            keys
        });
        Self {
            events: Vec::into_iter(events).peekable(),
            command_key: None,
        }
    }

    /// The next command, given whether a prompt is open now.
    pub fn next(&mut self, prompt_open: bool) -> Option<Command> {
        while let Some(event) = self.events.next() {
            let command_key = self.command_key.take();
            let own_text = |text: &str| {
                command_key.is_some_and(|k| k.symbol_or_name().eq_ignore_ascii_case(text))
            };
            let command = match event {
                // egui-winit turns Ctrl+C into a `Copy` event and drops the key.
                Event::Copy => Some(Command::Exit),
                Event::Text(mut text) if prompt_open && !own_text(&text) => {
                    // Text typed within one frame is one command, so the filter
                    // runs once per frame instead of once per character.
                    let typing = |e: &Event| match e {
                        Event::Text(_) => true,
                        Event::Key { key, .. } => {
                            !matches!(key, Key::Escape | Key::Enter | Key::Backspace)
                        }
                        _ => false,
                    };
                    while let Some(next) = self.events.next_if(typing) {
                        if let Event::Text(more) = next {
                            text.push_str(&more);
                        }
                    }
                    Some(Command::Input(InputCommand::Handle(text)))
                }
                Event::Key { key, modifiers, .. } if prompt_open => match key {
                    Key::Escape => Some(Command::Input(InputCommand::Exit)),
                    Key::Enter => Some(Command::Input(InputCommand::Confirm)),
                    Key::Backspace => Some(Command::Input(InputCommand::Backspace)),
                    _ => None,
                }
                .filter(|_| modifiers.matches_logically(Modifiers::NONE)),
                // A held Escape must not run on from closing something into quitting.
                Event::Key {
                    key,
                    repeat: true,
                    modifiers,
                    ..
                } if binding(key, modifiers) == Some(Command::ClearSelection) => None,
                Event::Key { key, modifiers, .. } => {
                    let command = binding(key, modifiers);
                    self.command_key = command.is_some().then_some(key);
                    command
                }
                _ => None,
            };
            if command.is_some() {
                return command;
            }
        }
        None
    }
}

/// The commands a status bar hint such as `j/k`, `/` or `Shift+Tab` stands for.
pub fn hinted(keys: &str) -> Vec<Command> {
    let tokens: Vec<&str> = if keys == "/" {
        vec![keys]
    } else {
        keys.split('/').collect()
    };
    tokens.into_iter().filter_map(token_command).collect()
}

/// The command bound to a hint token.
fn token_command(token: &str) -> Option<Command> {
    let token = token.trim();
    let (modifiers, key) = match token {
        "Shift+Tab" => (Modifiers::SHIFT, Key::Tab),
        "Tab" => (Modifiers::NONE, Key::Tab),
        "Ctrl+D" => (Modifiers::COMMAND, Key::D),
        "Ctrl+U" => (Modifiers::COMMAND, Key::U),
        "Ctrl+F" => (Modifiers::COMMAND, Key::F),
        "Ctrl+R" => (Modifiers::COMMAND, Key::R),
        "Ctrl+K" => (Modifiers::COMMAND, Key::K),
        "←" | "Left" => (Modifiers::NONE, Key::ArrowLeft),
        "→" | "Right" => (Modifiers::NONE, Key::ArrowRight),
        "↑" | "Up" => (Modifiers::NONE, Key::ArrowUp),
        "↓" | "Down" => (Modifiers::NONE, Key::ArrowDown),
        "PgUp" => (Modifiers::NONE, Key::PageUp),
        "PgDn" => (Modifiers::NONE, Key::PageDown),
        "Home" => (Modifiers::NONE, Key::Home),
        "End" => (Modifiers::NONE, Key::End),
        "Esc" => (Modifiers::NONE, Key::Escape),
        "Enter" => (Modifiers::NONE, Key::Enter),
        "Bksp" => (Modifiers::NONE, Key::Backspace),
        "?" => (Modifiers::SHIFT, Key::Slash),
        "/" => (Modifiers::NONE, Key::Slash),
        "=" => (Modifiers::NONE, Key::Equals),
        "+" => (Modifiers::NONE, Key::Plus),
        "-" => (Modifiers::NONE, Key::Minus),
        "," => (Modifiers::NONE, Key::Comma),
        "." => (Modifiers::NONE, Key::Period),
        key => (Modifiers::NONE, Key::from_name(key)?),
    };
    binding(key, modifiers)
}

/// The key map as `(keys, label)` pairs, one entry per distinct command.
pub fn bindings() -> Vec<(String, &'static str)> {
    let mut seen = Vec::new();
    let mut out = Vec::new();
    for (modifiers, key, command) in BINDINGS {
        let label = command.label();
        if seen.contains(&label) {
            continue;
        }
        seen.push(label);
        out.push((key_display(*modifiers, *key), label));
    }
    out
}

/// How a key and its modifiers are written in the hints and the settings.
pub fn key_display(modifiers: Modifiers, key: Key) -> String {
    let mut out = String::new();
    if modifiers.ctrl || modifiers.command {
        out.push_str("Ctrl+");
    }
    if modifiers.alt {
        out.push_str("Alt+");
    }
    if modifiers.shift {
        out.push_str("Shift+");
    }
    let name = key_name(key);
    if modifiers.shift && name.len() == 1 && name.chars().all(|c| c.is_ascii_lowercase()) {
        out.push_str(&name.to_uppercase());
    } else {
        out.push_str(name);
    }
    out
}

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
        Key::Comma => ",",
        Key::Period => ".",
        Key::Slash => "/",
        Key::Equals => "=",
        Key::Minus => "-",
        Key::Plus => "+",
        Key::OpenBracket => "[",
        Key::CloseBracket => "]",
        other => other.symbol_or_name(),
    }
}

/// Whether two commands are the same action, whatever the step size.
pub fn same_action(a: &Command, b: &Command) -> bool {
    match (a, b) {
        (Command::Next(x, _), Command::Next(y, _))
        | (Command::Previous(x, _), Command::Previous(y, _)) => x == y,
        _ => a == b,
    }
}

/// The normal-mode command bound to a key press. Earlier bindings win.
fn binding(key: Key, modifiers: Modifiers) -> Option<Command> {
    BINDINGS
        .iter()
        .find(|(m, k, _)| *k == key && modifiers.matches_logically(*m))
        .map(|(_, _, command)| command.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(key: Key, modifiers: Modifiers) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }
    }

    fn text(text: &str) -> Event {
        Event::Text(text.into())
    }

    fn run(events: Vec<Event>) -> Vec<Command> {
        let mut keys = Keys {
            events: events.into_iter().peekable(),
            command_key: None,
        };
        let (mut prompt_open, mut commands) = (false, Vec::new());
        while let Some(command) = keys.next(prompt_open) {
            match command {
                Command::Input(InputCommand::Enter) => prompt_open = true,
                Command::Input(InputCommand::Confirm | InputCommand::Exit) => prompt_open = false,
                _ => {}
            }
            commands.push(command);
        }
        commands
    }

    #[test]
    fn keys_run_in_typing_order() {
        let commands = run(vec![
            key(Key::Tab, Modifiers::NONE),
            key(Key::J, Modifiers::NONE),
            key(Key::Tab, Modifiers::SHIFT),
            key(Key::C, Modifiers::NONE),
        ]);
        assert_eq!(
            commands,
            [
                Command::Next(ScrollType::Tab, 1),
                Command::Next(ScrollType::List, 1),
                Command::Previous(ScrollType::Tab, 1),
                Command::ToggleChart,
            ]
        );
    }

    #[test]
    fn prompt_takes_typed_text() {
        let commands = run(vec![
            key(Key::Slash, Modifiers::NONE),
            text("/"),
            key(Key::A, Modifiers::NONE),
            text("a"),
            key(Key::Backspace, Modifiers::NONE),
            key(Key::Enter, Modifiers::NONE),
        ]);
        assert_eq!(
            commands,
            [
                Command::Input(InputCommand::Enter),
                Command::Input(InputCommand::Handle("a".into())),
                Command::Input(InputCommand::Backspace),
                Command::Input(InputCommand::Confirm),
            ]
        );
    }

    #[test]
    fn shifted_bindings_win() {
        assert_eq!(
            binding(Key::Q, Modifiers::SHIFT),
            Some(Command::ExitPreserveScreen)
        );
        assert_eq!(binding(Key::Q, Modifiers::NONE), Some(Command::Exit));
        assert_eq!(
            binding(Key::Slash, Modifiers::SHIFT),
            Some(Command::ShowHelp)
        );
        assert_eq!(
            binding(Key::Slash, Modifiers::NONE),
            Some(Command::Input(InputCommand::Enter))
        );
    }

    #[test]
    fn shifted_brackets_win_and_hints_resolve() {
        assert_eq!(
            binding(Key::CloseBracket, Modifiers::SHIFT),
            Some(Command::ExpandHostsMax)
        );
        assert_eq!(
            binding(Key::CloseBracket, Modifiers::NONE),
            Some(Command::ExpandHosts)
        );
        assert_eq!(hinted("/"), [Command::Input(InputCommand::Enter)]);
        assert_eq!(hinted("=/-"), [Command::ZoomIn, Command::ZoomOut]);
    }

    #[test]
    fn hints_name_their_commands() {
        assert_eq!(
            hinted("j/k"),
            [
                Command::Next(ScrollType::List, 1),
                Command::Previous(ScrollType::List, 1)
            ]
        );
        assert_eq!(hinted("?"), [Command::ShowHelp]);
        assert_eq!(hinted("s"), [Command::ShowSettings]);
        assert_eq!(hinted("Shift+Tab"), [Command::Previous(ScrollType::Tab, 1)]);
        assert!(same_action(
            &Command::Next(ScrollType::List, 5),
            &Command::Next(ScrollType::List, 1)
        ));
    }

    #[test]
    fn held_escape_does_not_repeat() {
        let repeat = Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: true,
            modifiers: Modifiers::NONE,
        };
        let commands = run(vec![key(Key::Escape, Modifiers::NONE), repeat]);
        assert_eq!(commands, [Command::ClearSelection]);
    }
}
