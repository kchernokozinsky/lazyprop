use serde::{Deserialize, Serialize};
use strum::Display;

use crate::state::{CryptoTarget, Operation};

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Message(String),
    /// Switch to the main screen.
    GoMain,
    /// Switch to the playground screen.
    GoPlayground,
    /// Switch to the about / help screen.
    GoAbout,
    /// Switch to the YAML editor screen.
    GoYaml,
    /// Switch to the previous screen (wrapping).
    PrevScreen,
    /// Switch to the next screen (wrapping).
    NextScreen,
    /// Move the selection down / to the next item.
    Down,
    /// Move the selection up / to the previous item.
    Up,
    /// Cycle focus to the next focusable pane.
    Tab,
    /// Give focus to a pane.
    Focus,
    /// Remove focus from a pane.
    UnFocus,
    /// Leave insert mode / dismiss the help overlay.
    Escape,
    /// A character was typed while a text input is focused.
    Input(char),
    /// Backspace was pressed while a text input is focused.
    Backspace,
    /// Delete the character under the cursor.
    DeleteChar,
    /// Move the text cursor left.
    CursorLeft,
    /// Move the text cursor right.
    CursorRight,
    /// Move the text cursor to the start.
    CursorHome,
    /// Move the text cursor to the end.
    CursorEnd,
    /// Encrypt the current input value with the selected environment.
    Encrypt,
    /// Decrypt the current input value with the selected environment.
    Decrypt,
    /// Copy the current result to the system clipboard.
    CopyResult,
    /// Toggle whether secret keys are shown in full.
    ToggleReveal,
    /// Send the selected environment's parameters to the playground.
    SendToPlayground,
    /// A background encrypt/decrypt finished.
    CryptoDone(CryptoTarget, Operation, Result<String, String>),
    /// Start filtering the environments list by name.
    Search,
    /// Open the form to add a new environment.
    AddEnv,
    /// Open the form to edit the selected environment.
    EditEnv,
    /// Ask to delete the selected environment.
    DeleteEnv,
}

impl Action {
    /// A short, human-readable description for the keybindings help. Returns an
    /// empty string for internal actions that are never bound to a key.
    pub fn description(&self) -> &'static str {
        match self {
            Action::Quit => "Quit",
            Action::Suspend => "Suspend to shell",
            Action::Up => "Move up / previous",
            Action::Down => "Move down / next",
            Action::Tab => "Switch focus",
            Action::Escape => "Cancel / clear focus",
            Action::Encrypt => "Encrypt the value",
            Action::Decrypt => "Decrypt the value",
            Action::CopyResult => "Copy result to clipboard",
            Action::ToggleReveal => "Reveal / hide the key",
            Action::SendToPlayground => "Send environment to playground",
            Action::Search => "Search environments",
            Action::AddEnv => "Add environment",
            Action::EditEnv => "Edit environment",
            Action::DeleteEnv => "Delete environment",
            Action::GoMain => "Go to Main screen",
            Action::GoPlayground => "Go to Playground screen",
            Action::GoAbout => "Go to About screen",
            Action::GoYaml => "Go to YAML editor screen",
            Action::PrevScreen => "Previous screen",
            Action::NextScreen => "Next screen",
            _ => "",
        }
    }
}
