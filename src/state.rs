use crate::env::Environment;

#[derive(Debug)]
pub struct AppState {
    pub envs: Vec<Environment>,
    pub selected_index: usize,
    pub input_text: String,
    pub output_text: String,
    pub temp_env: Option<Environment>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            envs: vec![],
            selected_index: 0,
            input_text: String::new(),
            output_text: String::new(),
            temp_env: None,
        }
    }

    pub fn add_env(&mut self, env: Environment) {
        self.envs.push(env);
        self.selected_index = self.envs.len().saturating_sub(1);
    }

    pub fn remove_env(&mut self) {
        if self.envs.is_empty() {
            return;
        }

        self.envs.remove(self.selected_index);

        if self.selected_index >= self.envs.len() {
            if !self.envs.is_empty() {
                self.selected_index = self.envs.len().saturating_sub(1);
            } else {
                self.selected_index = 0;
            }
        }
    }

    pub fn set_temp_env(&mut self, index: usize) {
        self.temp_env = self.envs.get(index).cloned();
    }
}
