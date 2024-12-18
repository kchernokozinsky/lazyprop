use crate::state::{InputMode, State};
use color_eyre::eyre::Result;
use ratatui::{
    prelude::*,
    widgets::{block::*, *},
};

use crate::{action::Action, panes::Pane};

#[derive(Debug)]
pub struct EnvsPane {
    focused: bool,
    focused_border_style: Style,
}

impl EnvsPane {
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

impl Pane for EnvsPane {
    fn height_constraint(&self) -> Constraint {
        match self.focused {
            true => Constraint::Fill(2),
            false => Constraint::Fill(2),
        }
    }

    fn update(&mut self, action: Action, state: &mut State) -> Result<Option<Action>> {
        match action {
            Action::Tick => {}
            Action::Down => {
                state.next();
                return Ok(Some(Action::Message(format!("{:?}", state))));
            }
            Action::Up => {
                state.prev();
                return Ok(Some(Action::Message(format!("{:?}", state))));
            }
            Action::Focus => {
                self.focused = true;
                state.input_mode = InputMode::Normal;
                return Ok(Some(Action::Message(format!("{:?}", state))));
            }
            Action::UnFocus => {
                self.focused = false;
            }
            Action::Submit => {}
            Action::Update => {}
            _ => {}
        }

        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect, state: &State) -> Result<()> {
        let items: Vec<String> = state
            .envs
            .environments
            .clone()
            .into_iter()
            .map(|env| env.name)
            .filter(|e| e.starts_with(state.search_query.clone().unwrap_or_default().as_str()))
            .collect();


        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_symbol(symbols::scrollbar::HORIZONTAL.end)
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));
        let mut list_state = ListState::default().with_selected(Some(state.cur()));

        frame.render_stateful_widget(list, area, &mut list_state);
        frame.render_widget(
            Block::default()
                .title(" Environments ")
                .borders(Borders::ALL)
                .border_style(self.border_style())
                .border_type(self.border_type())
                .title_bottom(
                    Line::from(format!(
                        " {} of {} ",
                        state.cur().saturating_add(1).min(state.envs.len()),
                        state.envs.len()
                    ))
                    .right_aligned(),
                ),
            area,
        );
        Ok(())
    }

    fn focusable(&self) -> bool {
        true
    }
}
