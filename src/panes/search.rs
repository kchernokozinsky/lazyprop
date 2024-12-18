use crate::state::{InputMode, State};
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{block::*, *},
};

use crate::{action::Action, panes::Pane};

#[derive(Debug)]
pub struct SearchPane {
    focused: bool,
    focused_border_style: Style,
    input: Option<String>,
}

impl SearchPane {
    pub fn new(focused: bool, focused_border_style: Style) -> Self {
        Self {
            focused,
            focused_border_style,
            input: None,
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

impl Pane for SearchPane {
    fn height_constraint(&self) -> Constraint {
        Constraint::Max(3)
    }

    fn handle_key_events(&mut self, key: KeyEvent, state: &mut State) -> Result<Option<Action>> {
        match state.input_mode {
            InputMode::Insert => match key.code {
                KeyCode::Char(c) => Ok(Some(Action::Input(c))),
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }

    fn update(&mut self, action: Action, state: &mut State) -> Result<Option<Action>> {
        match action {
            Action::Tick => {}
            Action::Input(c) => { match self.input {
                Some(ref mut s) => s.push(c),
                None => self.input = Some(format!("{}", c)),
            }
            state.search_query = self.input.clone();
        },
            Action::Backspace => {
                if let Some(ref mut s) = self.input {
                    match s.pop() {
                        Some(_) => {}
                        None => self.input = None,
                    };
                };

                state.search_query = self.input.clone();
            }
            Action::Focus => {
                self.focused = true;
                state.input_mode = InputMode::Insert;
                return Ok(Some(Action::Message(format!("{:?}", state))));
            }
            Action::UnFocus => {
                state.input_mode = InputMode::Normal;
                self.focused = false;
            }
            Action::Submit => {}
            Action::Update => {}
            _ => {}
        }

        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect, _state: &State) -> Result<()> {
        let search_block = Block::default()
            .title(" Search ")
            .borders(Borders::ALL)
            .border_style(self.border_style())
            .border_type(self.border_type());

        frame.render_widget(search_block, area);

        let inner_area = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Max(1000)])
            .split(area)[0];

        let input_text = if let Some(ref input) = self.input {
            Span::raw(input)
        } else {
            Span::styled(
                "Type to search...",
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            )
        };

        let display_input = if self.focused && self.input.is_some() {
            Line::from(vec![input_text, Span::raw("▌")])
        } else {
            Line::from(vec![input_text])
        };

        let search_paragraph = Paragraph::new(display_input)
            .style(Style::default().fg(Color::Black))
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(search_paragraph, inner_area);

        Ok(())
    }

    fn focusable(&self) -> bool {
        true
    }
}
