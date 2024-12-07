use crate::{config::env::EnvironmentsConfig, env::Environment, error::env::EnvironmentError};

#[derive(Debug)]
pub struct AppState<'a> {
    pub envs: &'a mut EnvironmentsConfig,
    pub selected_index: usize,
    pub input_text: String,
    pub output_text: String,
}

impl<'a> AppState<'a> {
    pub fn new(envs: &'a mut EnvironmentsConfig) -> Self {
        Self {
            envs,
            selected_index: 0,
            input_text: String::new(),
            output_text: String::new(),
        }
    }

    pub fn add_env(&mut self, env: Environment) -> Result<(), EnvironmentError> {
        self.envs.add(env)
    }

    pub fn remove_env(&mut self) -> Result<(), EnvironmentError> {
        self.envs.remove(self.selected_index)
    }

    pub fn edit_env(&mut self, env: Environment) -> Result<(), EnvironmentError> {
        self.envs.edit(self.selected_index, env)
    }

    pub fn curr_env(&self) -> Result<&Environment, EnvironmentError> {
        self.envs.get(self.selected_index)
    }
}
