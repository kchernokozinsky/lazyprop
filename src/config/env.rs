use std::{default, fs};

use config::{ConfigError, File};
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::{env::Environment, error::env::EnvironmentError};

#[derive(Serialize, Deserialize, Debug, SmartDefault)]
pub struct EnvironmentsConfig {
    #[default(vec![])]
    pub environments: Vec<Environment>,
}

impl EnvironmentsConfig {
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
