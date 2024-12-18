use crate::{config::Config, environment::Environments};
use color_eyre::Result;

#[derive(Debug)]
pub struct State {
    pub envs: Environments,
    current_env_index: usize,
    pub input_mode: InputMode,
    pub search_query: Option<String>,
}

impl State {
    pub fn new() -> Result<State> {
        let config = Config::new()?;
        Ok(Self {
            envs: Environments::new(config.envs_path)?,
            current_env_index: 0,
            input_mode: InputMode::default(),
            search_query: None,
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

#[derive(Default, Debug, PartialEq)]
pub enum InputMode {
    #[default]
    Normal,
    Insert,
}
