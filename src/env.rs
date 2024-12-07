use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Algorithm {
    AES,
    Blowfish,
    DES,
    DESede,
    RC2,
    RCA,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum State {
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
    /// Creates a new `Environment` with the given parameters.
    pub fn new(
        name: &str,
        algorithm: Algorithm,
        state: State,
        use_random_ivs: bool,
        key: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            algorithm,
            state,
            use_random_ivs,
            key: key.to_string(),
        }
    }
}
