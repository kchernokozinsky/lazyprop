use crate::{config::Config, state::State};
use color_eyre::eyre::Result;
use ratatui::{
    prelude::*,
    widgets::{block::*, *},
};

use crate::{action::Action, panes::Pane};

pub struct EnvsPane {
    config: Config,
    focused: bool,
    focused_border_style: Style,
}

impl EnvsPane {
    pub fn new(focused: bool, focused_border_style: Style, config: Config) -> Self {
        Self {
            config,
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
            true => Constraint::Fill(3),
            false => Constraint::Fill(2),
        }
    }

    fn update(&mut self, action: Action, state: &mut State) -> Result<Option<Action>> {
        match action {
            Action::Down => {
                state.next();
                return Ok(Some(Action::Update));
            }
            Action::Up => {
                state.prev();
            }
            Action::Focus => {
                self.focused = true;
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
                        "{} of {}",
                        state.cur().saturating_add(1).min(state.envs.len()),
                        state.envs.len()
                    ))
                    .right_aligned(),
                ),
            area,
        );
        Ok(())
    }
}
