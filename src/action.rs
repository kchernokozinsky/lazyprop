use serde::{Deserialize, Serialize};
use strum::Display;

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
    /// Switch to the about / help screen.
    GoAbout,
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
    /// Encrypt the current input value with the selected environment.
    Encrypt,
    /// Decrypt the current input value with the selected environment.
    Decrypt,
    /// Start filtering the environments list by name.
    Search,
    /// Open the form to add a new environment.
    AddEnv,
    /// Open the form to edit the selected environment.
    EditEnv,
    /// Ask to delete the selected environment.
    DeleteEnv,
}
