use std::path::PathBuf;

use crate::{
    app::Mode,
    config::Config,
    dencrypt::{decrypt, encrypt},
    environment::{Algorithm, Environment, Environments, State as CipherMode},
};
use color_eyre::Result;

/// A cryptographic operation the user can run against the selected environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Encrypt,
    Decrypt,
}

impl Operation {
    pub fn label(&self) -> &'static str {
        match self {
            Operation::Encrypt => "Encrypted",
            Operation::Decrypt => "Decrypted",
        }
    }
}

/// The outcome of the last encrypt/decrypt run, kept so panes can render it.
#[derive(Debug, Clone)]
pub struct CryptoResult {
    pub op: Operation,
    pub outcome: std::result::Result<String, String>,
}

/// Whether the environment form is creating a new entry or editing an existing
/// one (by index into the full environments list).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormMode {
    Add,
    Edit(usize),
}

/// The field of the environment form the cursor is currently on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormField {
    Name,
    Algorithm,
    Mode,
    RandomIv,
    Key,
}

impl FormField {
    const ORDER: [FormField; 5] = [
        FormField::Name,
        FormField::Algorithm,
        FormField::Mode,
        FormField::RandomIv,
        FormField::Key,
    ];

    fn shift(self, forward: bool) -> Self {
        let len = Self::ORDER.len();
        let pos = Self::ORDER.iter().position(|&f| f == self).unwrap_or(0);
        let next = if forward {
            (pos + 1) % len
        } else {
            (pos + len - 1) % len
        };
        Self::ORDER[next]
    }
}

/// A draft environment being edited in the modal form.
#[derive(Debug, Clone)]
pub struct EnvForm {
    pub mode: FormMode,
    pub name: String,
    pub algorithm: Algorithm,
    pub cipher: CipherMode,
    pub use_random_ivs: bool,
    pub key: String,
    pub field: FormField,
    pub error: Option<String>,
}

impl EnvForm {
    fn add() -> Self {
        Self {
            mode: FormMode::Add,
            name: String::new(),
            algorithm: Algorithm::default(),
            cipher: CipherMode::default(),
            use_random_ivs: false,
            key: String::new(),
            field: FormField::Name,
            error: None,
        }
    }

    fn edit(index: usize, env: &Environment) -> Self {
        Self {
            mode: FormMode::Edit(index),
            name: env.name.clone(),
            algorithm: env.algorithm,
            cipher: env.state,
            use_random_ivs: env.use_random_ivs,
            key: env.key.clone(),
            field: FormField::Name,
            error: None,
        }
    }

    pub fn title(&self) -> &'static str {
        match self.mode {
            FormMode::Add => " Add environment ",
            FormMode::Edit(_) => " Edit environment ",
        }
    }

    pub fn next_field(&mut self) {
        self.field = self.field.shift(true);
    }

    pub fn prev_field(&mut self) {
        self.field = self.field.shift(false);
    }

    /// Change the value of the active field. For choice fields this cycles
    /// through the options; text fields are unaffected.
    pub fn adjust(&mut self, forward: bool) {
        match self.field {
            FormField::Algorithm => self.algorithm = self.algorithm.cycle(forward),
            FormField::Mode => self.cipher = self.cipher.cycle(forward),
            FormField::RandomIv => self.use_random_ivs = !self.use_random_ivs,
            FormField::Name | FormField::Key => {}
        }
    }

    /// Type a character into the active text field (name or key).
    pub fn type_char(&mut self, c: char) {
        match self.field {
            FormField::Name => self.name.push(c),
            FormField::Key => self.key.push(c),
            _ => {}
        }
    }

    /// Delete a character from the active text field (name or key).
    pub fn backspace(&mut self) {
        match self.field {
            FormField::Name => {
                self.name.pop();
            }
            FormField::Key => {
                self.key.pop();
            }
            _ => {}
        }
    }

    fn to_environment(&self) -> Environment {
        Environment::new(
            self.name.trim(),
            self.algorithm,
            self.cipher,
            self.use_random_ivs,
            self.key.trim(),
        )
    }
}

#[derive(Debug)]
pub struct State {
    pub envs: Environments,
    current_env_index: usize,
    pub input_mode: InputMode,
    /// Active name filter for the environments list.
    pub search_query: Option<String>,
    /// Whether the search input is currently capturing keystrokes.
    pub searching: bool,
    /// Plain/cipher text the user is about to encrypt or decrypt.
    pub input_value: String,
    /// Result of the most recent encrypt/decrypt run.
    pub result: Option<CryptoResult>,
    /// Path to the MuleSoft Secure Properties Tool jar.
    pub jar_path: PathBuf,
    /// Path of the environments file, written back on add/edit/delete.
    pub envs_path: PathBuf,
    /// The active top-level screen.
    pub mode: Mode,
    /// The environment form, when open.
    pub form: Option<EnvForm>,
    /// Index of an environment pending delete confirmation.
    pub pending_delete: Option<usize>,
}

impl State {
    pub fn new(envs_path: Option<String>, jar_path: Option<String>) -> Result<State> {
        let config = Config::new()?;
        let envs_path = envs_path.unwrap_or(config.envs_path);
        let jar_path = jar_path.unwrap_or(config.jar_path);
        Ok(Self {
            envs: Environments::new(&envs_path)?,
            current_env_index: 0,
            input_mode: InputMode::default(),
            search_query: None,
            searching: false,
            input_value: String::new(),
            result: None,
            jar_path: PathBuf::from(jar_path),
            envs_path: PathBuf::from(envs_path),
            mode: Mode::default(),
            form: None,
            pending_delete: None,
        })
    }

    pub fn cur(&self) -> usize {
        self.current_env_index
    }

    /// The currently selected environment, if any exist.
    pub fn selected_env(&self) -> Option<&Environment> {
        self.envs.environments.get(self.current_env_index)
    }

    /// Indices (into the full list) of environments matching the search query.
    pub fn filtered(&self) -> Vec<usize> {
        let query = self.search_query.as_deref().unwrap_or("").to_lowercase();
        self.envs
            .environments
            .iter()
            .enumerate()
            .filter(|(_, e)| query.is_empty() || e.name.to_lowercase().contains(&query))
            .map(|(i, _)| i)
            .collect()
    }

    /// Position of the selection within the filtered list, for the list widget.
    pub fn selected_position(&self) -> Option<usize> {
        self.filtered()
            .iter()
            .position(|&i| i == self.current_env_index)
    }

    pub fn next(&mut self) {
        let filtered = self.filtered();
        if filtered.is_empty() {
            return;
        }
        let pos = filtered
            .iter()
            .position(|&i| i == self.current_env_index)
            .unwrap_or(0);
        self.current_env_index = filtered[(pos + 1) % filtered.len()];
    }

    pub fn prev(&mut self) {
        let filtered = self.filtered();
        if filtered.is_empty() {
            return;
        }
        let pos = filtered
            .iter()
            .position(|&i| i == self.current_env_index)
            .unwrap_or(0);
        self.current_env_index = filtered[(pos + filtered.len() - 1) % filtered.len()];
    }

    fn clamp_selection(&mut self) {
        let filtered = self.filtered();
        if !filtered.is_empty() && !filtered.contains(&self.current_env_index) {
            self.current_env_index = filtered[0];
        }
    }

    pub fn push_search(&mut self, c: char) {
        let mut query = self.search_query.take().unwrap_or_default();
        query.push(c);
        self.search_query = Some(query);
        self.clamp_selection();
    }

    pub fn pop_search(&mut self) {
        if let Some(mut query) = self.search_query.take() {
            query.pop();
            self.search_query = if query.is_empty() { None } else { Some(query) };
        }
        self.clamp_selection();
    }

    // --- Environment form ---------------------------------------------------

    pub fn open_add_form(&mut self) {
        self.form = Some(EnvForm::add());
    }

    pub fn open_edit_form(&mut self) {
        if let Some(env) = self.selected_env() {
            self.form = Some(EnvForm::edit(self.current_env_index, env));
        }
    }

    /// Validate and apply the open form, persisting the result to disk.
    /// On success the form is closed; on failure it stays open with an error.
    pub fn submit_form(&mut self) -> std::result::Result<(), String> {
        let (mode, env, name) = {
            let form = self.form.as_ref().ok_or("No form is open.")?;
            if form.name.trim().is_empty() {
                return Err("Name cannot be empty.".to_string());
            }
            if form.key.trim().is_empty() {
                return Err("Key cannot be empty.".to_string());
            }
            (
                form.mode,
                form.to_environment(),
                form.name.trim().to_string(),
            )
        };

        match mode {
            FormMode::Add => self.envs.add(env).map_err(|e| e.to_string())?,
            FormMode::Edit(index) => self.envs.edit(index, env).map_err(|e| e.to_string())?,
        }
        self.persist()?;

        if let Some(index) = self.envs.environments.iter().position(|e| e.name == name) {
            self.current_env_index = index;
        }
        self.form = None;
        Ok(())
    }

    pub fn cancel_form(&mut self) {
        self.form = None;
    }

    // --- Environment deletion ----------------------------------------------

    pub fn request_delete(&mut self) {
        if self.selected_env().is_some() {
            self.pending_delete = Some(self.current_env_index);
        }
    }

    pub fn cancel_delete(&mut self) {
        self.pending_delete = None;
    }

    /// The name of the environment pending deletion, if any.
    pub fn pending_delete_name(&self) -> Option<&str> {
        self.pending_delete
            .and_then(|i| self.envs.environments.get(i))
            .map(|e| e.name.as_str())
    }

    pub fn confirm_delete(&mut self) -> std::result::Result<(), String> {
        let Some(index) = self.pending_delete.take() else {
            return Ok(());
        };
        self.envs.remove(index).map_err(|e| e.to_string())?;
        self.persist()?;
        if self.current_env_index >= self.envs.len() {
            self.current_env_index = self.envs.len().saturating_sub(1);
        }
        self.clamp_selection();
        Ok(())
    }

    fn persist(&self) -> std::result::Result<(), String> {
        self.envs
            .save(&self.envs_path.to_string_lossy())
            .map_err(|e| e.to_string())
    }

    // --- Crypto -------------------------------------------------------------

    /// Run `op` on the current input value using the selected environment and
    /// store the outcome in `self.result`.
    pub fn run_crypto(&mut self, op: Operation) {
        let outcome = self.compute(op);
        self.result = Some(CryptoResult { op, outcome });
    }

    fn compute(&self, op: Operation) -> std::result::Result<String, String> {
        if self.input_value.trim().is_empty() {
            return Err("Nothing to process: the input value is empty.".to_string());
        }
        let env = self
            .selected_env()
            .ok_or_else(|| "No environment selected.".to_string())?;
        match op {
            Operation::Encrypt => encrypt(&self.input_value, env, &self.jar_path),
            Operation::Decrypt => decrypt(&self.input_value, env, &self.jar_path),
        }
    }
}

#[derive(Default, Debug, PartialEq)]
pub enum InputMode {
    #[default]
    Normal,
    Insert,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_field_navigation_wraps() {
        let mut form = EnvForm::add();
        assert_eq!(form.field, FormField::Name);
        form.prev_field();
        assert_eq!(form.field, FormField::Key); // wraps to the last field
        form.next_field();
        assert_eq!(form.field, FormField::Name); // wraps back to the first
    }

    #[test]
    fn form_typing_only_affects_text_fields() {
        let mut form = EnvForm::add();
        // On the Name field, characters are captured.
        form.type_char('a');
        form.type_char('b');
        assert_eq!(form.name, "ab");
        // Move to a choice field; typing is ignored, adjust cycles instead.
        form.field = FormField::Algorithm;
        form.type_char('z');
        assert_eq!(form.algorithm, Algorithm::AES); // typing did nothing
        form.adjust(true);
        assert_eq!(form.algorithm, Algorithm::Blowfish);
        // Random IV toggles.
        form.field = FormField::RandomIv;
        assert!(!form.use_random_ivs);
        form.adjust(true);
        assert!(form.use_random_ivs);
    }

    #[test]
    fn form_to_environment_trims() {
        let mut form = EnvForm::add();
        form.name = "  Prod  ".to_string();
        form.key = "  k123  ".to_string();
        let env = form.to_environment();
        assert_eq!(env.name, "Prod");
        assert_eq!(env.key, "k123");
    }
}
