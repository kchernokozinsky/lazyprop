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
    components::{about::AboutScreen, home::Home, Component},
    config::Config,
    panes::{footer::FooterPane, header::HeaderPane, Pane},
    state::{InputMode, State},
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
    About,
}

impl Mode {
    /// Index of this screen's component in `App::components`.
    fn component_index(self) -> usize {
        match self {
            Mode::Main => 0,
            Mode::About => 1,
        }
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
        Ok(Self {
            tick_rate,
            frame_rate,
            components: vec![Box::new(Home::new()?), Box::new(AboutScreen::new())],
            should_quit: false,
            should_suspend: false,
            header: HeaderPane::new(),
            footer: FooterPane::new(),
            config: Config::new()?,
            last_tick_key_events: Vec::new(),
            action_tx,
            action_rx,
            state: State::new(envs_path, jar_path)?,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?
            // .mouse(true) // uncomment this line to enable mouse support
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
            KeyCode::Char(c) => Action::Input(c),
            _ => return Ok(()),
        };

        action_tx.send(action)?;

        Ok(())
    }

    /// Handle keys while the add/edit environment form is open.
    fn handle_form_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let Some(form) = self.state.form.as_mut() else {
            return Ok(());
        };
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
            KeyCode::Left => form.adjust(false),
            KeyCode::Right => form.adjust(true),
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
                Action::GoAbout => self.state.mode = Mode::About,
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
            let vertical_layout = Layout::vertical(vec![
                Constraint::Max(1),
                Constraint::Fill(1),
                Constraint::Max(1),
            ])
            .split(frame.area());

            if let Err(err) = self.header.draw(frame, vertical_layout[0], &self.state) {
                let _ = self
                    .action_tx
                    .send(Action::Error(format!("Failed to draw: {:?}", err)));
            }

            let active = self.state.mode.component_index();
            if let Err(err) = self.components[active].draw(frame, vertical_layout[1], &self.state) {
                let _ = self
                    .action_tx
                    .send(Action::Error(format!("Failed to draw: {:?}", err)));
            }

            if let Err(err) = self.footer.draw(frame, vertical_layout[2], &self.state) {
                let _ = self
                    .action_tx
                    .send(Action::Error(format!("Failed to draw: {:?}", err)));
            }
        })?;
        Ok(())
    }
}
