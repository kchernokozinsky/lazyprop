use crate::state::{InputMode, State};
use color_eyre::eyre::Result;
use ratatui::{
    prelude::*,
    widgets::{block::*, *},
};

use crate::{action::Action, panes::Pane, theme};

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
        Constraint::Fill(1)
    }

    fn update(&mut self, action: Action, state: &mut State) -> Result<Option<Action>> {
        match action {
            Action::Down => state.next(),
            Action::Up => state.prev(),
            Action::Focus => {
                self.focused = true;
                state.input_mode = InputMode::Normal;
            }
            Action::UnFocus => {
                self.focused = false;
            }
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect, state: &State) -> Result<()> {
        let filtered = state.filtered();
        let items: Vec<ListItem> = filtered
            .iter()
            .map(|&i| ListItem::new(state.envs.environments[i].name.clone()))
            .collect();

        // Bottom title: either the live search box or the "x of y" counter.
        let bottom = if state.searching {
            Line::from(vec![
                Span::styled(" /", Style::default().fg(theme::ACCENT)),
                Span::raw(state.search_query.clone().unwrap_or_default()),
                Span::styled("▌ ", Style::default().fg(theme::ACCENT)),
            ])
        } else if state.search_query.is_some() {
            Line::from(Span::styled(
                format!(
                    " filter: {} ",
                    state.search_query.clone().unwrap_or_default()
                ),
                Style::default().fg(theme::ACCENT),
            ))
            .right_aligned()
        } else {
            Line::from(format!(
                " {} of {} ",
                state.selected_position().map(|p| p + 1).unwrap_or(0),
                filtered.len()
            ))
            .right_aligned()
        };

        let block = Block::default()
            .title(" Environments ")
            .borders(Borders::ALL)
            .border_style(self.border_style())
            .border_type(self.border_type())
            .title_bottom(bottom);

        if items.is_empty() {
            let msg = if state.envs.is_empty() {
                "No environments found.\nPress 'a' to add one."
            } else {
                "No environments match the filter."
            };
            let empty = Paragraph::new(msg).style(theme::hint_italic()).block(block);
            frame.render_widget(empty, area);
            return Ok(());
        }

        let list = List::new(items)
            .block(block)
            .highlight_symbol("▶ ")
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_style(
                Style::default()
                    .fg(theme::ACCENT)
                    .add_modifier(Modifier::BOLD),
            );
        let mut list_state = ListState::default().with_selected(state.selected_position());

        frame.render_stateful_widget(list, area, &mut list_state);
        Ok(())
    }

    fn focusable(&self) -> bool {
        true
    }
}
