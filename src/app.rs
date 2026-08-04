use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Layout},
    prelude::Rect,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, info};

use crate::{
    action::Action,
    components::{
        about::AboutScreen, home::Home, playground::PlaygroundScreen, yaml::YamlScreen, Component,
    },
    config::Config,
    panes::{footer::FooterPane, header::HeaderPane, Pane},
    state::{CryptoTarget, InputMode, Operation, State},
    tui::{Event, Tui},
};

pub struct App {
    config: Config,
    tick_rate: f64,
    frame_rate: f64,
    components: Vec<Box<dyn Component>>,
    should_quit: bool,
    should_suspend: bool,
    header: HeaderPane,
    footer: FooterPane,
    last_tick_key_events: Vec<KeyEvent>,
    action_tx: mpsc::UnboundedSender<Action>,
    action_rx: mpsc::UnboundedReceiver<Action>,
    state: State,
}

/// Top-level screens the user can switch between.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    #[default]
    Main,
    Playground,
    About,
    Yaml,
}

impl Mode {
    const ORDER: [Mode; 4] = [Mode::Main, Mode::Playground, Mode::Yaml, Mode::About];

    /// Index of this screen's component in `App::components`.
    fn component_index(self) -> usize {
        match self {
            Mode::Main => 0,
            Mode::Playground => 1,
            Mode::About => 2,
            Mode::Yaml => 3,
        }
    }

    fn shift(self, forward: bool) -> Self {
        let len = Self::ORDER.len();
        let i = Self::ORDER.iter().position(|&m| m == self).unwrap_or(0);
        Self::ORDER[if forward {
            (i + 1) % len
        } else {
            (i + len - 1) % len
        }]
    }
}

impl App {
    pub fn new(
        tick_rate: f64,
        frame_rate: f64,
        envs_path: Option<String>,
        jar_path: Option<String>,
    ) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();
        let config = Config::new()?;
        crate::theme::init(&config.theme);
        Ok(Self {
            tick_rate,
            frame_rate,
            components: vec![
                Box::new(Home::new()?),
                Box::new(PlaygroundScreen::new()),
                Box::new(AboutScreen::new()),
                Box::new(YamlScreen::new()),
            ],
            should_quit: false,
            should_suspend: false,
            header: HeaderPane::new(),
            footer: FooterPane::new(),
            config,
            last_tick_key_events: Vec::new(),
            action_tx,
            action_rx,
            state: State::new(envs_path, jar_path)?,
        })
    }

    /// Open a YAML file on startup (from `--file`) and switch to the YAML
    /// screen. Errors are shown on that screen rather than aborting.
    pub fn open_yaml_file(&mut self, path: &str) {
        if let Err(e) = self.state.yaml.open_path(path) {
            self.state.yaml.report(e, true);
        }
        self.state.mode = Mode::Yaml;
    }

    pub async fn run(&mut self) -> Result<()> {
        // Mouse capture stays off so the terminal's own text selection keeps
        // working. All navigation has keyboard equivalents.
        let mut tui = Tui::new()?
            .tick_rate(self.tick_rate)
            .frame_rate(self.frame_rate);
        tui.enter()?;

        for component in self.components.iter_mut() {
            component.register_action_handler(self.action_tx.clone())?;
        }
        for component in self.components.iter_mut() {
            component.register_config_handler(self.config.clone())?;
        }
        for component in self.components.iter_mut() {
            component.init(tui.size()?, &self.state)?;
        }

        self.header.init(&self.state)?;

        if let Some(msg) = self.state.startup_message.take() {
            self.action_tx.send(Action::Error(msg))?;
        }

        let action_tx = self.action_tx.clone();
        loop {
            self.handle_events(&mut tui).await?;
            self.handle_actions(&mut tui)?;
            if self.should_suspend {
                tui.suspend()?;
                action_tx.send(Action::Resume)?;
                action_tx.send(Action::ClearScreen)?;
                tui.enter()?;
            } else if self.should_quit {
                tui.stop()?;
                break;
            }
        }
        tui.exit()?;
        Ok(())
    }

    async fn handle_events(&mut self, tui: &mut Tui) -> Result<()> {
        let Some(event) = tui.next_event().await else {
            return Ok(());
        };
        let action_tx = self.action_tx.clone();
        match event {
            Event::Quit if self.state.input_mode == InputMode::Normal => {
                action_tx.send(Action::Quit)?
            }
            Event::Tick => action_tx.send(Action::Tick)?,
            Event::Render => action_tx.send(Action::Render)?,
            Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
            Event::Key(key) => self.handle_key_event(key)?,
            _ => {}
        }
        let active = self.state.mode.component_index();
        if let Some(action) =
            self.components[active].handle_events(Some(event.clone()), &self.state)?
        {
            action_tx.send(action)?;
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        // Modal dialogs capture all input while open.
        if self.state.form.is_some() {
            return self.handle_form_key_event(key);
        }
        if self.state.pending_delete.is_some() {
            return self.handle_delete_key_event(key);
        }
        // The playground is a self-contained form screen with its own keys.
        if self.state.mode == Mode::Playground {
            return self.handle_playground_key_event(key);
        }
        // The YAML editor handles its own keys (tree, modals, edit mode).
        if self.state.mode == Mode::Yaml {
            return crate::yaml_editor::input::handle_key(key, &mut self.state, &self.action_tx);
        }

        let action_tx = self.action_tx.clone();
        let Some(keymap) = self.config.keybindings.get(&self.state.mode) else {
            return Ok(());
        };

        if self.state.input_mode == InputMode::Insert {
            return self.handle_input_key_event(key);
        }

        match keymap.get(&vec![key]) {
            Some(action) => {
                info!("Got action: {action:?}");
                action_tx.send(action.clone())?;
            }
            _ => {
                self.last_tick_key_events.push(key);

                if let Some(action) = keymap.get(&self.last_tick_key_events) {
                    info!("Got action: {action:?}");
                    action_tx.send(action.clone())?;
                }
            }
        }
        Ok(())
    }

    fn handle_input_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let action_tx = self.action_tx.clone();
        let action = match key.code {
            KeyCode::Tab => Action::Tab,
            KeyCode::Esc | KeyCode::Enter => Action::Escape,
            KeyCode::Backspace => Action::Backspace,
            KeyCode::Delete => Action::DeleteChar,
            KeyCode::Left => Action::CursorLeft,
            KeyCode::Right => Action::CursorRight,
            KeyCode::Home => Action::CursorHome,
            KeyCode::End => Action::CursorEnd,
            KeyCode::Char(c) => Action::Input(c),
            _ => return Ok(()),
        };

        action_tx.send(action)?;

        Ok(())
    }

    /// Handle keys while the add/edit environment form is open.
    fn handle_form_key_event(&mut self, key: KeyEvent) -> Result<()> {
        use crate::state::FormField;
        let Some(form) = self.state.form.as_mut() else {
            return Ok(());
        };
        let on_text = matches!(form.field, FormField::Name | FormField::Key);
        match key.code {
            KeyCode::Esc => self.state.cancel_form(),
            KeyCode::Enter => match self.state.submit_form() {
                Ok(()) => self.action_tx.send(Action::Message(
                    "Environment saved to envs.yaml.".to_string(),
                ))?,
                Err(e) => {
                    if let Some(form) = self.state.form.as_mut() {
                        form.error = Some(e);
                    }
                }
            },
            KeyCode::Tab | KeyCode::Down => form.next_field(),
            KeyCode::BackTab | KeyCode::Up => form.prev_field(),
            // On text fields the arrows move the cursor; on choice fields they
            // change the value.
            KeyCode::Left if on_text => form.cursor_left(),
            KeyCode::Right if on_text => form.cursor_right(),
            KeyCode::Left => form.adjust(false),
            KeyCode::Right => form.adjust(true),
            KeyCode::Home => form.cursor_home(),
            KeyCode::End => form.cursor_end(),
            KeyCode::Delete => form.delete(),
            // Space types into text fields and toggles/cycles choice fields;
            // each method no-ops on the field kind it does not apply to.
            KeyCode::Char(' ') => {
                form.type_char(' ');
                form.adjust(true);
            }
            KeyCode::Backspace => form.backspace(),
            KeyCode::Char(c) => form.type_char(c),
            _ => {}
        }
        Ok(())
    }

    /// Handle keys on the playground screen (its own form-like editor).
    fn handle_playground_key_event(&mut self, key: KeyEvent) -> Result<()> {
        use crossterm::event::KeyModifiers;

        // Ctrl-y copies the result regardless of the active field.
        if key.code == KeyCode::Char('y') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.copy_result(crate::state::CryptoTarget::Playground)?;
            return Ok(());
        }

        let on_text_field = matches!(
            self.state.playground.field,
            crate::state::PlaygroundField::Key | crate::state::PlaygroundField::Value
        );

        // On choice fields (Operation/Algorithm/State/Random-IV) letters and
        // digits aren't typed, so reuse them for screen switching — the same
        // keys that work on the other screens. On text fields they type.
        if !on_text_field {
            let nav = match key.code {
                KeyCode::Char('1') => Some(Action::GoMain),
                KeyCode::Char('2') => Some(Action::GoPlayground),
                KeyCode::Char('3') => Some(Action::GoYaml),
                KeyCode::Char('4') => Some(Action::GoAbout),
                KeyCode::Char('h') => Some(Action::PrevScreen),
                KeyCode::Char('l') => Some(Action::NextScreen),
                KeyCode::Char('q') => Some(Action::Quit),
                _ => None,
            };
            if let Some(action) = nav {
                self.action_tx.send(action)?;
                return Ok(());
            }
        }

        let p = &mut self.state.playground;
        match key.code {
            KeyCode::Esc => self.state.mode = Mode::Main,
            KeyCode::Enter => self.state.begin_playground(self.action_tx.clone()),
            KeyCode::Tab | KeyCode::Down => p.next_field(),
            KeyCode::BackTab | KeyCode::Up => p.prev_field(),
            // On text fields the arrows move the cursor; on choice fields they
            // change the value.
            KeyCode::Left if on_text_field => p.cursor_left(),
            KeyCode::Right if on_text_field => p.cursor_right(),
            KeyCode::Left => p.adjust(false),
            KeyCode::Right => p.adjust(true),
            KeyCode::Home => p.cursor_home(),
            KeyCode::End => p.cursor_end(),
            KeyCode::Delete => p.delete(),
            // Space types into text fields and toggles/cycles choice fields;
            // each method no-ops on the field kind it does not apply to.
            KeyCode::Char(' ') => {
                p.type_char(' ');
                p.adjust(true);
            }
            KeyCode::Backspace => p.backspace(),
            KeyCode::Char(c) => p.type_char(c),
            _ => {}
        }
        Ok(())
    }

    /// Copy the successful result for `target` to the clipboard and report it.
    fn copy_result(&mut self, target: crate::state::CryptoTarget) -> Result<()> {
        match self.state.result_output(target) {
            Some(text) => match crate::clipboard::copy(&text) {
                Ok(()) => self
                    .action_tx
                    .send(Action::Message("Result copied to clipboard.".to_string()))?,
                Err(e) => self.action_tx.send(Action::Error(e))?,
            },
            None => self
                .action_tx
                .send(Action::Message("No result to copy.".to_string()))?,
        }
        Ok(())
    }

    /// Handle keys while the delete confirmation dialog is open.
    fn handle_delete_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => match self.state.confirm_delete() {
                Ok(()) => self
                    .action_tx
                    .send(Action::Message("Environment deleted.".to_string()))?,
                Err(e) => self.action_tx.send(Action::Error(e))?,
            },
            KeyCode::Char('n') | KeyCode::Esc => self.state.cancel_delete(),
            _ => {}
        }
        Ok(())
    }

    fn handle_actions(&mut self, tui: &mut Tui) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            if action != Action::Tick && action != Action::Render {
                debug!("{action:?}");
            }
            match action {
                Action::Tick => {
                    self.last_tick_key_events.drain(..);
                }
                Action::Quit => self.should_quit = true,
                Action::Suspend => self.should_suspend = true,
                Action::Resume => self.should_suspend = false,
                Action::ClearScreen => tui.terminal.clear()?,
                Action::Resize(w, h) => self.handle_resize(tui, w, h)?,
                Action::Render => self.render(tui)?,
                Action::GoMain => self.state.mode = Mode::Main,
                Action::GoPlayground => self.state.mode = Mode::Playground,
                Action::GoAbout => self.state.mode = Mode::About,
                Action::GoYaml => self.state.mode = Mode::Yaml,
                Action::PrevScreen => self.state.mode = self.state.mode.shift(false),
                Action::NextScreen => self.state.mode = self.state.mode.shift(true),
                Action::Encrypt => self
                    .state
                    .begin_main_crypto(self.action_tx.clone(), Operation::Encrypt),
                Action::Decrypt => self
                    .state
                    .begin_main_crypto(self.action_tx.clone(), Operation::Decrypt),
                Action::CopyResult => self.copy_result(CryptoTarget::Main)?,
                Action::ToggleReveal => self.state.reveal_key = !self.state.reveal_key,
                Action::SendToPlayground => self.state.send_to_playground(),
                Action::CryptoDone(target, op, ref outcome) => {
                    self.state.set_result(target, op, outcome.clone());
                    if target == CryptoTarget::Yaml {
                        self.state.yaml_pump_bulk(self.action_tx.clone());
                    }
                }
                _ => {}
            }
            let active = self.state.mode.component_index();
            if let Some(action) = self.components[active].update(action.clone(), &mut self.state)? {
                self.action_tx.send(action)?
            };
        }
        Ok(())
    }

    fn handle_resize(&mut self, tui: &mut Tui, w: u16, h: u16) -> Result<()> {
        tui.resize(Rect::new(0, 0, w, h))?;
        self.render(tui)?;
        Ok(())
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        tui.draw(|frame| {
            // Tab bar, a blank spacer row, the active screen, then the footer.
            let [header_area, _spacer, content_area, footer_area] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(1),
            ])
            .areas(frame.area());

            if let Err(err) = self.header.draw(frame, header_area, &self.state) {
                let _ = self
                    .action_tx
                    .send(Action::Error(format!("Failed to draw: {:?}", err)));
            }

            let active = self.state.mode.component_index();
            if let Err(err) = self.components[active].draw(frame, content_area, &self.state) {
                let _ = self
                    .action_tx
                    .send(Action::Error(format!("Failed to draw: {:?}", err)));
            }

            if let Err(err) = self.footer.draw(frame, footer_area, &self.state) {
                let _ = self
                    .action_tx
                    .send(Action::Error(format!("Failed to draw: {:?}", err)));
            }
        })?;
        Ok(())
    }
}
