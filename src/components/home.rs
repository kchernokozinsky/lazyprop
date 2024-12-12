use color_eyre::Result;
use ratatui::prelude::*;
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
    action::Action,
    config::Config,
    panes::{details::DetailsPane, envs::EnvsPane, status::StatusPane, Pane},
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
                Box::new(EnvsPane::new(false, focused_border_style, Config::new()?)),
                Box::new(StatusPane::new(false, focused_border_style, Config::new()?)),
                Box::new(DetailsPane::new(
                    false,
                    focused_border_style,
                    Config::new()?,
                )),
            ],

            focused_pane_index: 0,
            fullscreen_pane_index: None,
        })
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
            Action::Tick => {
                self.panes[self.focused_pane_index].update(Action::Focus, state)?;
            }
            Action::Render => {}
            Action::Down => {
                self.panes[self.focused_pane_index].update(Action::Down, state)?;
            }
            Action::Up => {
                self.panes[self.focused_pane_index].update(Action::Up, state)?;
            }
            Action::Tab => {
                self.panes[self.focused_pane_index].update(Action::UnFocus, state)?;
                self.focused_pane_index = (self.focused_pane_index + 1) % (self.panes.len());
                self.panes[self.focused_pane_index].update(Action::Focus, state)?;
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
                .constraints(vec![Constraint::Fill(1), Constraint::Fill(4)])
                .split(area);
            let left_panes = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    self.panes[0].height_constraint(),
                    self.panes[1].height_constraint(),
                ])
                .split(outer_layout[0]);

            let right_panes = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![self.panes[2].height_constraint()])
                .split(outer_layout[1]);

            self.panes[0].draw(frame, left_panes[0], state)?;
            self.panes[1].draw(frame, left_panes[1], state)?;
            self.panes[2].draw(frame, right_panes[0], state)?;
        }
        Ok(())
    }
}
