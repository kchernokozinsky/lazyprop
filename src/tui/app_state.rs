use crate::{
    config::env::EnvironmentsConfig, env::Environment, error::env::EnvironmentError,
    tui::popup::PopupState,
};

#[derive(Debug)]
pub enum InputMode {
    Normal,
    Adding,
    Editing,
}

#[derive(Debug)]
pub struct AppState<'a> {
    pub envs: &'a mut EnvironmentsConfig,
    pub selected_index: usize,
    pub status_message: String,
    pub popup: PopupState,
}

impl<'a> AppState<'a> {
    pub fn new(envs: &'a mut EnvironmentsConfig, popup: PopupState) -> Self {
        Self {
            envs,
            selected_index: 0,
            status_message: "".into(),
            popup,
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

    pub fn len_env(&self) -> usize {
        self.envs.len()
    }

    pub fn is_empty_env(&self) -> bool {
        self.envs.is_empty()
    }
}
