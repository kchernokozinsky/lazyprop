use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default, Copy, PartialEq, Eq)]
pub enum Algorithm {
    #[default]
    AES,
    Blowfish,
    DES,
    DESede,
    RC2,
    RCA,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, Copy, PartialEq, Eq)]
pub enum State {
    #[default]
    CBC,
    CFB,
    ECB,
    OFB,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Environment {
    pub name: String,
    pub algorithm: Algorithm,
    pub state: State,
    pub use_random_ivs: bool,
    pub key: String,
}

impl Environment {
    pub fn new<A>(name: A, algorithm: Algorithm, state: State, use_random_ivs: bool, key: A) -> Self
    where
        A: Into<String>,
    {
        Self {
            name: name.into(),
            algorithm,
            state,
            use_random_ivs,
            key: key.into(),
        }
    }
}
