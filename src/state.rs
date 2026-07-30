use std::path::PathBuf;

use crate::{
    action::Action,
    app::Mode,
    dencrypt::{decrypt, encrypt},
    environment::{Algorithm, Environment, Environments, State as CipherMode},
    text_field::TextField,
};
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

/// A cryptographic operation the user can run against the selected environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Operation {
    #[default]
    Encrypt,
    Decrypt,
}

/// Which screen a background crypto result belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CryptoTarget {
    Main,
    Playground,
}

/// Run an encrypt/decrypt off the UI thread (the JVM start-up is slow) and
/// deliver the outcome back as an [`Action::CryptoDone`].
pub fn spawn_crypto(
    tx: UnboundedSender<Action>,
    target: CryptoTarget,
    op: Operation,
    jar: PathBuf,
    env: Environment,
    value: String,
) {
    tokio::spawn(async move {
        let outcome = tokio::task::spawn_blocking(move || match op {
            Operation::Encrypt => encrypt(&value, &env, &jar),
            Operation::Decrypt => decrypt(&value, &env, &jar),
        })
        .await
        .unwrap_or_else(|e| Err(format!("background task failed: {e}")));
        let _ = tx.send(Action::CryptoDone(target, op, outcome));
    });
}

impl Operation {
    /// Past-tense label for a completed operation ("Encrypted" / "Decrypted").
    pub fn label(&self) -> &'static str {
        match self {
            Operation::Encrypt => "Encrypted",
            Operation::Decrypt => "Decrypted",
        }
    }

    /// Present-tense name for pickers ("Encrypt" / "Decrypt").
    pub fn name(&self) -> &'static str {
        match self {
            Operation::Encrypt => "Encrypt",
            Operation::Decrypt => "Decrypt",
        }
    }

    fn toggle(self) -> Self {
        match self {
            Operation::Encrypt => Operation::Decrypt,
            Operation::Decrypt => Operation::Encrypt,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormMode {
    #[default]
    Add,
    Edit(usize),
}

/// The field of the environment form the cursor is currently on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormField {
    #[default]
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
#[derive(Debug, Clone, Default)]
pub struct EnvForm {
    pub mode: FormMode,
    pub name: TextField,
    pub algorithm: Algorithm,
    pub cipher: CipherMode,
    pub use_random_ivs: bool,
    pub key: TextField,
    pub field: FormField,
    pub error: Option<String>,
}

impl EnvForm {
    fn add() -> Self {
        Self::default()
    }

    fn edit(index: usize, env: &Environment) -> Self {
        Self {
            mode: FormMode::Edit(index),
            name: TextField::from_text(&env.name),
            algorithm: env.algorithm,
            cipher: env.state,
            use_random_ivs: env.use_random_ivs,
            key: TextField::from_text(&env.key),
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

    fn active_text(&mut self) -> Option<&mut TextField> {
        match self.field {
            FormField::Name => Some(&mut self.name),
            FormField::Key => Some(&mut self.key),
            _ => None,
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
            FormField::Algorithm => {
                self.algorithm = self.algorithm.cycle(forward);
                // Keep the mode valid for the newly selected algorithm.
                self.cipher = self.algorithm.reconcile_mode(self.cipher);
            }
            FormField::Mode => self.cipher = self.algorithm.cycle_mode(self.cipher, forward),
            FormField::RandomIv => self.use_random_ivs = !self.use_random_ivs,
            FormField::Name | FormField::Key => {}
        }
    }

    pub fn type_char(&mut self, c: char) {
        if let Some(t) = self.active_text() {
            t.insert(c);
        }
    }

    pub fn backspace(&mut self) {
        if let Some(t) = self.active_text() {
            t.backspace();
        }
    }

    pub fn delete(&mut self) {
        if let Some(t) = self.active_text() {
            t.delete();
        }
    }

    pub fn cursor_left(&mut self) {
        if let Some(t) = self.active_text() {
            t.left();
        }
    }

    pub fn cursor_right(&mut self) {
        if let Some(t) = self.active_text() {
            t.right();
        }
    }

    pub fn cursor_home(&mut self) {
        if let Some(t) = self.active_text() {
            t.home();
        }
    }

    pub fn cursor_end(&mut self) {
        if let Some(t) = self.active_text() {
            t.end();
        }
    }

    fn to_environment(&self) -> Environment {
        Environment::new(
            self.name.value().trim(),
            self.algorithm,
            self.cipher,
            self.use_random_ivs,
            self.key.value().trim(),
        )
    }
}

/// A field of the playground editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaygroundField {
    #[default]
    Operation,
    Algorithm,
    Mode,
    RandomIv,
    Key,
    Value,
}

impl PlaygroundField {
    const ORDER: [PlaygroundField; 6] = [
        PlaygroundField::Operation,
        PlaygroundField::Algorithm,
        PlaygroundField::Mode,
        PlaygroundField::RandomIv,
        PlaygroundField::Key,
        PlaygroundField::Value,
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

/// Ad-hoc encrypt/decrypt with parameters entered directly, no environment.
/// Mirrors MuleSoft's Secure Properties Tool "string" form.
#[derive(Debug, Clone, Default)]
pub struct Playground {
    pub operation: Operation,
    pub algorithm: Algorithm,
    pub cipher: CipherMode,
    pub use_random_ivs: bool,
    pub key: TextField,
    pub value: TextField,
    pub field: PlaygroundField,
    pub result: Option<CryptoResult>,
}

impl Playground {
    fn active_text(&mut self) -> Option<&mut TextField> {
        match self.field {
            PlaygroundField::Key => Some(&mut self.key),
            PlaygroundField::Value => Some(&mut self.value),
            _ => None,
        }
    }

    pub fn next_field(&mut self) {
        self.field = self.field.shift(true);
    }

    pub fn prev_field(&mut self) {
        self.field = self.field.shift(false);
    }

    /// Change the value of the active choice field; text fields are unaffected.
    pub fn adjust(&mut self, forward: bool) {
        match self.field {
            PlaygroundField::Operation => self.operation = self.operation.toggle(),
            PlaygroundField::Algorithm => {
                self.algorithm = self.algorithm.cycle(forward);
                self.cipher = self.algorithm.reconcile_mode(self.cipher);
            }
            PlaygroundField::Mode => self.cipher = self.algorithm.cycle_mode(self.cipher, forward),
            PlaygroundField::RandomIv => self.use_random_ivs = !self.use_random_ivs,
            PlaygroundField::Key | PlaygroundField::Value => {}
        }
    }

    pub fn type_char(&mut self, c: char) {
        if let Some(t) = self.active_text() {
            t.insert(c);
        }
    }

    pub fn backspace(&mut self) {
        if let Some(t) = self.active_text() {
            t.backspace();
        }
    }

    pub fn delete(&mut self) {
        if let Some(t) = self.active_text() {
            t.delete();
        }
    }

    pub fn cursor_left(&mut self) {
        if let Some(t) = self.active_text() {
            t.left();
        }
    }

    pub fn cursor_right(&mut self) {
        if let Some(t) = self.active_text() {
            t.right();
        }
    }

    pub fn cursor_home(&mut self) {
        if let Some(t) = self.active_text() {
            t.home();
        }
    }

    pub fn cursor_end(&mut self) {
        if let Some(t) = self.active_text() {
            t.end();
        }
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
    pub input_value: TextField,
    /// Result of the most recent encrypt/decrypt run.
    pub result: Option<CryptoResult>,
    /// Whether an encrypt/decrypt is currently running in the background.
    pub busy: bool,
    /// Whether secret keys are shown in full rather than masked.
    pub reveal_key: bool,
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
    /// State of the playground (no-environment) screen.
    pub playground: Playground,
}

impl State {
    pub fn new(envs_path: Option<String>, jar_path: Option<String>) -> Result<State> {
        // Resolve the environments file and jar, honouring CLI flags, env vars,
        // a project-local file, then the ~/.lazyprop home (created on first run).
        let envs_path = crate::config::resolve_envs_path(envs_path)?;
        let jar_path = crate::config::resolve_jar_path(jar_path)?;
        Ok(Self {
            envs: Environments::new(envs_path.to_string_lossy())?,
            current_env_index: 0,
            input_mode: InputMode::default(),
            search_query: None,
            searching: false,
            input_value: TextField::default(),
            result: None,
            busy: false,
            reveal_key: false,
            jar_path,
            envs_path,
            mode: Mode::default(),
            form: None,
            pending_delete: None,
            playground: Playground::default(),
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
            let name = form.name.value();
            if name.trim().is_empty() {
                return Err("Name cannot be empty.".to_string());
            }
            if form.key.value().trim().is_empty() {
                return Err("Key cannot be empty.".to_string());
            }
            (form.mode, form.to_environment(), name.trim().to_string())
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

    /// Start an encrypt/decrypt for the main screen in the background, using the
    /// current input value and the selected environment. Validation errors are
    /// stored immediately; otherwise `busy` is set until the result arrives.
    pub fn begin_main_crypto(&mut self, tx: UnboundedSender<Action>, op: Operation) {
        if self.busy {
            return;
        }
        let value = self.input_value.value();
        if value.trim().is_empty() {
            self.set_result(
                CryptoTarget::Main,
                op,
                Err("The input value is empty.".to_string()),
            );
            return;
        }
        let Some(env) = self.selected_env().cloned() else {
            self.set_result(
                CryptoTarget::Main,
                op,
                Err("No environment selected.".to_string()),
            );
            return;
        };
        if !env.algorithm.supports_modes() {
            self.set_result(
                CryptoTarget::Main,
                op,
                Err(format!("{:?} is not supported by the tool.", env.algorithm)),
            );
            return;
        }
        self.busy = true;
        self.result = None;
        spawn_crypto(
            tx,
            CryptoTarget::Main,
            op,
            self.jar_path.clone(),
            env,
            value,
        );
    }

    /// Start the playground's encrypt/decrypt in the background.
    pub fn begin_playground(&mut self, tx: UnboundedSender<Action>) {
        if self.busy {
            return;
        }
        let op = self.playground.operation;
        let key = self.playground.key.value();
        let value = self.playground.value.value();
        if key.trim().is_empty() {
            self.set_result(
                CryptoTarget::Playground,
                op,
                Err("Key cannot be empty.".to_string()),
            );
            return;
        }
        if value.trim().is_empty() {
            self.set_result(
                CryptoTarget::Playground,
                op,
                Err("Value cannot be empty.".to_string()),
            );
            return;
        }
        if !self.playground.algorithm.supports_modes() {
            self.set_result(
                CryptoTarget::Playground,
                op,
                Err(format!(
                    "{:?} is not supported by the tool.",
                    self.playground.algorithm
                )),
            );
            return;
        }
        let env = Environment::new(
            "playground",
            self.playground.algorithm,
            self.playground.cipher,
            self.playground.use_random_ivs,
            key.trim(),
        );
        self.busy = true;
        self.playground.result = None;
        spawn_crypto(
            tx,
            CryptoTarget::Playground,
            op,
            self.jar_path.clone(),
            env,
            value,
        );
    }

    /// Record a completed (or immediately-failed) crypto outcome.
    pub fn set_result(
        &mut self,
        target: CryptoTarget,
        op: Operation,
        outcome: std::result::Result<String, String>,
    ) {
        self.busy = false;
        let result = Some(CryptoResult { op, outcome });
        match target {
            CryptoTarget::Main => self.result = result,
            CryptoTarget::Playground => self.playground.result = result,
        }
    }

    /// The successful output text of the last result for `target`, if any.
    pub fn result_output(&self, target: CryptoTarget) -> Option<String> {
        let result = match target {
            CryptoTarget::Main => self.result.as_ref(),
            CryptoTarget::Playground => self.playground.result.as_ref(),
        };
        result.and_then(|r| r.outcome.as_ref().ok()).cloned()
    }

    /// Copy the selected environment's parameters into the playground and
    /// switch to the playground screen.
    pub fn send_to_playground(&mut self) {
        let Some(env) = self.selected_env().cloned() else {
            return;
        };
        self.playground.operation = Operation::Encrypt;
        self.playground.algorithm = env.algorithm;
        self.playground.cipher = env.state;
        self.playground.use_random_ivs = env.use_random_ivs;
        self.playground.key = TextField::from_text(&env.key);
        self.playground.value = TextField::default();
        self.playground.field = PlaygroundField::Value;
        self.playground.result = None;
        self.mode = Mode::Playground;
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
        assert_eq!(form.name.value(), "ab");
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
        form.name = TextField::from_text("  Prod  ");
        form.key = TextField::from_text("  k123  ");
        let env = form.to_environment();
        assert_eq!(env.name, "Prod");
        assert_eq!(env.key, "k123");
    }

    #[test]
    fn playground_field_navigation_and_choices() {
        let mut p = Playground::default();
        assert_eq!(p.field, PlaygroundField::Operation);
        p.prev_field();
        assert_eq!(p.field, PlaygroundField::Value); // wraps to last

        // Operation toggles between encrypt and decrypt.
        p.field = PlaygroundField::Operation;
        assert_eq!(p.operation, Operation::Encrypt);
        p.adjust(true);
        assert_eq!(p.operation, Operation::Decrypt);

        // Typing only lands in the active text field.
        p.field = PlaygroundField::Key;
        p.type_char('k');
        p.field = PlaygroundField::Value;
        p.type_char('v');
        assert_eq!(p.key.value(), "k");
        assert_eq!(p.value.value(), "v");
    }

    fn test_state() -> State {
        State::new(Some("tests/fixtures/envs.yaml".to_string()), None).expect("load fixture")
    }

    #[test]
    fn begin_main_crypto_reports_empty_input_without_spawning() {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let mut state = test_state();
        // Empty input value: fails validation immediately, no background task.
        state.begin_main_crypto(tx, Operation::Encrypt);
        assert!(!state.busy, "must not be busy after a validation failure");
        let result = state.result.expect("a result should be stored");
        assert!(result.outcome.is_err());
    }

    #[test]
    fn set_result_and_result_output() {
        let mut state = test_state();
        state.busy = true;
        state.set_result(
            CryptoTarget::Main,
            Operation::Encrypt,
            Ok("cipher".to_string()),
        );
        assert!(!state.busy, "set_result clears busy");
        assert_eq!(
            state.result_output(CryptoTarget::Main),
            Some("cipher".to_string())
        );
        // An error result yields no copyable output.
        state.set_result(
            CryptoTarget::Playground,
            Operation::Decrypt,
            Err("boom".into()),
        );
        assert_eq!(state.result_output(CryptoTarget::Playground), None);
    }

    #[test]
    fn send_to_playground_copies_env_and_switches() {
        let mut state = test_state();
        let env = state.selected_env().cloned().expect("fixture has envs");
        state.send_to_playground();
        assert_eq!(state.mode, Mode::Playground);
        assert_eq!(state.playground.key.value(), env.key);
        assert_eq!(state.playground.algorithm, env.algorithm);
        assert_eq!(state.playground.field, PlaygroundField::Value);
    }
}
