use color_eyre::eyre::Result;
use ratatui::prelude::*;

use crate::{app::Mode, cli::VERSION_MESSAGE, panes::Pane, state::State, theme};

#[derive(Default)]
pub struct HeaderPane {}

impl HeaderPane {
    pub fn new() -> Self {
        Self {}
    }

    fn tab(label: &str, active: bool) -> Span<'static> {
        if active {
            Span::styled(
                format!(" {label} "),
                Style::default()
                    .fg(theme::ACCENT)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED),
            )
        } else {
            Span::styled(format!(" {label} "), theme::hint())
        }
    }
}

impl Pane for HeaderPane {
    fn height_constraint(&self) -> Constraint {
        Constraint::Max(1)
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect, state: &State) -> Result<()> {
        let [left, right] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(28)]).areas(area);

        let tabs = Line::from(vec![
            Self::tab("1 Main", state.mode == Mode::Main),
            Span::styled("│", theme::hint()),
            Self::tab("2 About", state.mode == Mode::About),
        ]);
        frame.render_widget(tabs, left);

        let title = Line::from(vec![
            Span::styled("lazyprop ", Style::default().fg(theme::ACCENT)),
            Span::styled(VERSION_MESSAGE, theme::hint()),
            Span::raw(" "),
        ])
        .right_aligned();
        frame.render_widget(title, right);

        Ok(())
    }

    fn focusable(&self) -> bool {
        false
    }
}
