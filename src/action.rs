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
    Help,
    Down,
    Up,
    Update,
    TimedStatusLine(String, u64),
    Tab,
    Focus,
    UnFocus,
    Submit,
    Input(char),
}
