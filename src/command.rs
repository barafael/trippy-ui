//! Keyboard commands and the per-frame key reader.
//!
//! Which key runs which command is decided by [`crate::bindings::Bindings`].
//! Commands are read in the order the keys were typed, so opening the filter
//! prompt changes what the following keys mean.

use crate::bindings::Bindings;
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
    ///
    /// While a text field has keyboard focus the events are left to it.
    pub fn take(ctx: &egui::Context) -> Self {
        let text_field_focused = ctx
            .memory(|m| m.focused())
            .is_some_and(|id| egui::text_edit::TextEditState::load(ctx, id).is_some());
        if text_field_focused {
            return Self::from_events(Vec::new());
        }
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
        Self::from_events(events)
    }

    fn from_events(events: Vec<Event>) -> Self {
        Self {
            events: events.into_iter().peekable(),
            command_key: None,
        }
    }

    /// The next command, given whether a prompt is open now.
    pub fn next(&mut self, prompt_open: bool, bindings: &Bindings) -> Option<Command> {
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
                } if bindings.lookup(key, modifiers) == Some(Command::ClearSelection) => None,
                Event::Key { key, modifiers, .. } => {
                    let command = bindings.lookup(key, modifiers);
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

/// Whether two commands are the same action, whatever the step size.
pub fn same_action(a: &Command, b: &Command) -> bool {
    match (a, b) {
        (Command::Next(x, _), Command::Next(y, _))
        | (Command::Previous(x, _), Command::Previous(y, _)) => x == y,
        _ => a == b,
    }
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
        let bindings = Bindings::default();
        let mut keys = Keys::from_events(events);
        let (mut prompt_open, mut commands) = (false, Vec::new());
        while let Some(command) = keys.next(prompt_open, &bindings) {
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

    #[test]
    fn same_action_ignores_the_step() {
        assert!(same_action(
            &Command::Next(ScrollType::List, 5),
            &Command::Next(ScrollType::List, 1)
        ));
        assert!(!same_action(&Command::ShowHelp, &Command::ShowSettings));
    }
}
