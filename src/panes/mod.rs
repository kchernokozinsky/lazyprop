use color_eyre::eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{
    layout::{Constraint, Rect},
    Frame,
};

use crate::{action::Action, state::State, tui::Event};

pub mod details;
pub mod envs;
pub mod header;
pub mod status;

pub trait Pane {
    fn init(&mut self, _state: &State) -> Result<()> {
        Ok(())
    }

    fn height_constraint(&self) -> Constraint;

    fn handle_events(&mut self, event: Event, state: &mut State) -> Result<Option<Action>> {
        let r = match event {
            Event::Key(key_event) => self.handle_key_events(key_event, state)?,
            Event::Mouse(mouse_event) => self.handle_mouse_events(mouse_event, state)?,
            _ => None,
        };
        Ok(r)
    }

    fn handle_key_events(&mut self, _key: KeyEvent, _state: &mut State) -> Result<Option<Action>> {
        Ok(None)
    }

    fn handle_mouse_events(
        &mut self,
        _mouse: MouseEvent,
        _state: &mut State,
    ) -> Result<Option<Action>> {
        Ok(None)
    }

    fn update(&mut self, _action: Action, _state: &mut State) -> Result<Option<Action>> {
        Ok(None)
    }

    fn draw(&mut self, f: &mut Frame<'_>, area: Rect, state: &State) -> Result<()>;
}
