use crate::{config::Config, environment::Environments};
use color_eyre::Result;
#[derive(Debug)]
pub struct State {
    pub envs: Environments,
    current_env_index: usize,
}

impl State {
    pub fn new() -> Result<State> {
        let config = Config::new()?;
        Ok(Self {
            envs: Environments::new(config.envs_path)?,
            current_env_index: 0,
        })
    }

    pub fn cur(&self) -> usize {
        self.current_env_index
    }

    pub fn next(&mut self) {
        self.current_env_index = (self.current_env_index + 1) % self.envs.len();
    }

    pub fn prev(&mut self) {
        self.current_env_index =
            self.current_env_index.saturating_add(self.envs.len() - 1) % self.envs.len();
    }
}
