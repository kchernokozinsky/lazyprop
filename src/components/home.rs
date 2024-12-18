use color_eyre::Result;
use ratatui::prelude::*;
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
    action::Action,
    config::Config,
    panes::{details::DetailsPane, envs::EnvsPane, search::SearchPane, status::StatusPane, Pane},
    state::State,
};

pub struct Home {
    command_tx: Option<UnboundedSender<Action>>,
    config: Config,
    panes: Vec<Box<dyn Pane>>,
    focused_pane_index: usize,
    fullscreen_pane_index: Option<usize>,
}

impl Home {
    pub fn new() -> Result<Self> {
        let focused_border_style = Style::default().fg(Color::Green);
        Ok(Self {
            command_tx: None,
            config: Config::default(),
            panes: vec![
                Box::new(EnvsPane::new(true, focused_border_style)),
                Box::new(StatusPane::new()),
                Box::new(DetailsPane::new()),
                Box::new(SearchPane::new(false, focused_border_style)),
            ],

            focused_pane_index: 0,
            fullscreen_pane_index: None,
        })
    }

    pub fn next_focused_pane(&mut self) {
        self.focused_pane_index = (self.focused_pane_index + 1) % (self.panes.len());

        while !self.panes[self.focused_pane_index].focusable() {
            self.focused_pane_index = (self.focused_pane_index + 1) % (self.panes.len());
        }
    }
}

impl Component for Home {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action, state: &mut State) -> Result<Option<Action>> {
        match action {
            Action::Tick => {}
            Action::Render => {}
            Action::Down => return self.panes[self.focused_pane_index].update(Action::Down, state),
            Action::Up => return self.panes[self.focused_pane_index].update(Action::Up, state),
            Action::Tab => {
                self.panes[self.focused_pane_index].update(Action::UnFocus, state)?;
                self.next_focused_pane();
                return self.panes[self.focused_pane_index].update(Action::Focus, state);
            }
            Action::Error(message) => return self.panes[1].update(Action::Error(message), state),
            Action::Message(message) => {
                return self.panes[1].update(Action::Message(message), state)
            }
            Action::Input(c) => {
                return self.panes[self.focused_pane_index].update(Action::Input(c), state)
            }
            Action::Backspace => {
                return self.panes[self.focused_pane_index].update(Action::Backspace, state)
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, state: &State) -> Result<()> {
        if let Some(fullscreen_pane_index) = self.fullscreen_pane_index {
            self.panes[fullscreen_pane_index].draw(frame, area, state)?;
        } else {
            let outer_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Fill(1), Constraint::Fill(1)])
                .split(area);

            let bottom_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    self.panes[3].height_constraint(),
                    self.panes[0].height_constraint(),
                    self.panes[1].height_constraint(),
                ])
                .split(area);

            let left_panes = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    self.panes[3].height_constraint(),
                    self.panes[0].height_constraint(),
                    self.panes[1].height_constraint(),
                ])
                .split(outer_layout[0]);

            let right_panes = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    self.panes[3].height_constraint(),
                    self.panes[0].height_constraint(),
                    self.panes[1].height_constraint(),
                ])
                .split(outer_layout[1]);

            self.panes[0].draw(frame, left_panes[1], state)?;
            self.panes[2].draw(frame, right_panes[1], state)?;
            self.panes[1].draw(frame, bottom_layout[2], state)?;
            self.panes[3].draw(frame, left_panes[0], state)?;
        }
        Ok(())
    }
}
