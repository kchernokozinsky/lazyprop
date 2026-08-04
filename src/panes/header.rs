use color_eyre::eyre::Result;
use ratatui::prelude::*;

use crate::{app::Mode, cli::VERSION_MESSAGE, panes::Pane, state::State, theme};

#[derive(Default)]
pub struct HeaderPane {}

impl HeaderPane {
    pub fn new() -> Self {
        Self {}
    }

    /// A tab as two spans: a dimmed shortcut number and the screen name.
    /// The active screen's name is accented, bold and underlined; the rest dim.
    fn tab(num: &str, label: &str, active: bool) -> [Span<'static>; 3] {
        let name_style = if active {
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            theme::hint()
        };
        [
            Span::styled(num.to_string(), theme::hint()),
            Span::raw(" "),
            Span::styled(label.to_string(), name_style),
        ]
    }
}

impl Pane for HeaderPane {
    fn height_constraint(&self) -> Constraint {
        Constraint::Max(1)
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect, state: &State) -> Result<()> {
        // Size the title column to the title itself so the leading "l" of
        // "lazyprop" is never clipped by a too-narrow fixed width.
        let title_width = ("lazyprop ".len() + VERSION_MESSAGE.len() + 1) as u16;
        let [left, right] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(title_width)]).areas(area);

        let gap = || Span::raw("   ");
        let mut spans = vec![Span::raw(" ")];
        spans.extend(Self::tab("1", "Main", state.mode == Mode::Main));
        spans.push(gap());
        spans.extend(Self::tab("2", "Playground", state.mode == Mode::Playground));
        spans.push(gap());
        spans.extend(Self::tab("3", "YAML", state.mode == Mode::Yaml));
        spans.push(gap());
        spans.extend(Self::tab("4", "About", state.mode == Mode::About));
        frame.render_widget(Line::from(spans), left);

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
