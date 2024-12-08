use smart_default::SmartDefault;

use crate::env::{Algorithm, State};

pub const ALGORITHMS: &[Algorithm] = &[
    Algorithm::AES,
    Algorithm::Blowfish,
    Algorithm::DES,
    Algorithm::DESede,
    Algorithm::RC2,
    Algorithm::RCA,
];

pub const STATES: &[State] = &[State::CBC, State::CFB, State::ECB, State::OFB];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupMode {
    None,
    Add,
    Edit,
    EncryptDecrypt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupField {
    Name,
    Algorithm,
    State,
    IV,
    Key,
    TextInput,
    EncryptButton,
    DecryptButton,
}

#[derive(Debug, SmartDefault)]
pub struct PopupState {
    #[default(PopupMode::None)]
    pub mode: PopupMode,
    #[default(PopupField::Name)]
    pub focus: PopupField,
    #[default(String::new())]
    pub name: String,
    #[default = 0]
    pub selected_algorithm: usize,
    #[default = 0]
    pub selected_state: usize,
    #[default = false]
    pub use_random_ivs: bool,
    #[default(String::new())]
    pub key: String,
    #[default(String::new())]
    pub text_input: String,
}

impl PopupState {
    pub fn new() -> Self {
        Self::default()
    }
}
