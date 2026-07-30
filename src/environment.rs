use std::fs;

use config::{ConfigError, File};
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::errors::env_error::EnvironmentError;

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

impl Algorithm {
    pub const ALL: [Algorithm; 6] = [
        Algorithm::AES,
        Algorithm::Blowfish,
        Algorithm::DES,
        Algorithm::DESede,
        Algorithm::RC2,
        Algorithm::RCA,
    ];

    /// The next/previous variant, wrapping around.
    pub fn cycle(self, forward: bool) -> Self {
        cycle_variant(&Self::ALL, self, forward)
    }
}

impl State {
    pub const ALL: [State; 4] = [State::CBC, State::CFB, State::ECB, State::OFB];

    /// The next/previous variant, wrapping around.
    pub fn cycle(self, forward: bool) -> Self {
        cycle_variant(&Self::ALL, self, forward)
    }
}

fn cycle_variant<T: Copy + PartialEq>(all: &[T], current: T, forward: bool) -> T {
    let len = all.len();
    let pos = all.iter().position(|&v| v == current).unwrap_or(0);
    let next = if forward {
        (pos + 1) % len
    } else {
        (pos + len - 1) % len
    };
    all[next]
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

#[derive(Serialize, Deserialize, Debug, SmartDefault, Clone)]
pub struct Environments {
    pub environments: Vec<Environment>,
}

impl Environments {
    pub fn new(conf_file: impl AsRef<str>) -> Result<Self, ConfigError> {
        let s = config::Config::builder().add_source(File::with_name(conf_file.as_ref()));

        s.build()?.try_deserialize()
    }

    /// Add an environment, checking for duplicate names.
    pub fn add(&mut self, env: Environment) -> Result<(), EnvironmentError> {
        if self.environments.iter().any(|e| e.name == env.name) {
            return Err(EnvironmentError::DuplicateName(env.name));
        }
        self.environments.push(env);
        Ok(())
    }

    /// Remove an environment by index.
    pub fn remove(&mut self, index: usize) -> Result<(), EnvironmentError> {
        if index < self.environments.len() {
            self.environments.remove(index);
            Ok(())
        } else {
            Err(EnvironmentError::InvalidIndex(index))
        }
    }

    /// Edit an environment by index, checking for duplicate names.
    pub fn edit(&mut self, index: usize, new_env: Environment) -> Result<(), EnvironmentError> {
        if index >= self.environments.len() {
            return Err(EnvironmentError::InvalidIndex(index));
        }

        let old_name = &self.environments[index].name;
        if new_env.name == *old_name {
            self.environments[index] = new_env;
            return Ok(());
        }

        if self.environments.iter().any(|e| e.name == new_env.name) {
            return Err(EnvironmentError::DuplicateName(new_env.name));
        }

        self.environments[index] = new_env;
        Ok(())
    }

    /// Get a reference to an environment by index.
    pub fn get(&self, index: usize) -> Result<&Environment, EnvironmentError> {
        self.environments
            .get(index)
            .ok_or(EnvironmentError::InvalidIndex(index))
    }

    pub fn len(&self) -> usize {
        self.environments.len()
    }

    pub fn is_empty(&self) -> bool {
        self.environments.is_empty()
    }

    /// Save the current configuration to a YAML file.
    pub fn save(&self, file_path: &str) -> anyhow::Result<()> {
        let yaml_str = serde_yaml::to_string(self)?;
        fs::write(file_path, yaml_str)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(name: &str) -> Environment {
        Environment::new(name, Algorithm::AES, State::CBC, false, "secret1234567890")
    }

    fn envs(names: &[&str]) -> Environments {
        Environments {
            environments: names.iter().map(|n| env(n)).collect(),
        }
    }

    #[test]
    fn algorithm_cycles_and_wraps() {
        assert_eq!(Algorithm::AES.cycle(true), Algorithm::Blowfish);
        assert_eq!(Algorithm::AES.cycle(false), Algorithm::RCA); // wraps backwards
        assert_eq!(Algorithm::RCA.cycle(true), Algorithm::AES); // wraps forwards
    }

    #[test]
    fn mode_cycles_and_wraps() {
        assert_eq!(State::CBC.cycle(true), State::CFB);
        assert_eq!(State::CBC.cycle(false), State::OFB);
        assert_eq!(State::OFB.cycle(true), State::CBC);
    }

    #[test]
    fn save_then_reload_roundtrips() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("lazyprop_test_{}.yaml", std::process::id()));
        let path_str = path.to_string_lossy().to_string();

        let mut original = envs(&["Alpha", "Beta"]);
        original.environments[1].use_random_ivs = true;
        original.save(&path_str).expect("save should succeed");

        let reloaded = Environments::new(&path_str).expect("reload should succeed");
        assert_eq!(reloaded.len(), 2);
        assert_eq!(reloaded.get(0).unwrap().name, "Alpha");
        assert!(reloaded.get(1).unwrap().use_random_ivs);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn add_rejects_duplicate_names() {
        let mut e = envs(&["A"]);
        assert!(e.add(env("B")).is_ok());
        assert_eq!(e.len(), 2);
        assert!(matches!(
            e.add(env("A")),
            Err(EnvironmentError::DuplicateName(_))
        ));
    }

    #[test]
    fn remove_by_index_and_bounds() {
        let mut e = envs(&["A", "B"]);
        assert!(e.remove(0).is_ok());
        assert_eq!(e.get(0).unwrap().name, "B");
        assert!(matches!(
            e.remove(5),
            Err(EnvironmentError::InvalidIndex(5))
        ));
    }

    #[test]
    fn edit_allows_same_name_but_rejects_clash() {
        let mut e = envs(&["A", "B"]);
        // Renaming to its own name is fine.
        assert!(e.edit(0, env("A")).is_ok());
        // Renaming to an existing sibling's name is rejected.
        assert!(matches!(
            e.edit(0, env("B")),
            Err(EnvironmentError::DuplicateName(_))
        ));
    }

    #[test]
    fn empty_and_len() {
        assert!(envs(&[]).is_empty());
        assert_eq!(envs(&["A", "B", "C"]).len(), 3);
    }
}
