use crate::state::{InputMode, State};
use color_eyre::eyre::Result;
use ratatui::{
    prelude::*,
    widgets::{block::*, *},
};

use crate::{action::Action, panes::Pane};

/// A single-line text input holding the value to encrypt or decrypt.
#[derive(Debug)]
pub struct InputPane {
    focused: bool,
    focused_border_style: Style,
}

impl InputPane {
    pub fn new(focused: bool, focused_border_style: Style) -> Self {
        Self {
            focused,
            focused_border_style,
        }
    }

    fn border_style(&self) -> Style {
        match self.focused {
            true => self.focused_border_style,
            false => Style::default(),
        }
    }

    fn border_type(&self) -> BorderType {
        match self.focused {
            true => BorderType::Thick,
            false => BorderType::Plain,
        }
    }
}

impl Pane for InputPane {
    fn height_constraint(&self) -> Constraint {
        Constraint::Length(3)
    }

    fn update(&mut self, action: Action, state: &mut State) -> Result<Option<Action>> {
        match action {
            Action::Focus => {
                self.focused = true;
                state.input_mode = InputMode::Insert;
            }
            Action::UnFocus => {
                self.focused = false;
                state.input_mode = InputMode::Normal;
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect, state: &State) -> Result<()> {
        let block = Block::default()
            .title(" Value ")
            .borders(Borders::ALL)
            .border_style(self.border_style())
            .border_type(self.border_type());

        let inner_width = (area.width as usize).saturating_sub(2);
        let spans = state.input_value.spans(
            inner_width,
            self.focused,
            "Focus with Tab, then type a value…",
        );
        let paragraph = Paragraph::new(Line::from(spans)).block(block);
        frame.render_widget(paragraph, area);
        Ok(())
    }

    fn focusable(&self) -> bool {
        true
    }
}
