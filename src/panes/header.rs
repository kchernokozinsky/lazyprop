use color_eyre::eyre::Result;
use ratatui::prelude::*;

use crate::{app::Mode, cli::VERSION_MESSAGE, panes::Pane, state::State, theme};

#[derive(Default)]
pub struct HeaderPane {}

impl HeaderPane {
    pub fn new() -> Self {
        Self {}
    }

    /// A tab as a single span: the screen name. The active screen is accented,
    /// bold and underlined; the rest are dim. Numeric shortcuts are intentionally
    /// not shown here (they still work) — the contextual footer and the About
    /// guides are the source of truth for keybindings.
    fn tab(label: &str, active: bool) -> Span<'static> {
        let name_style = if active {
            Style::default()
                .fg(theme::accent())
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            theme::hint()
        };
        Span::styled(label.to_string(), name_style)
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

        let tabs = [
            (Mode::Main, "Main"),
            (Mode::Playground, "Playground"),
            (Mode::Yaml, "YAML"),
            (Mode::About, "About"),
        ];
        let mut spans = vec![Span::raw(" ")];
        for (i, (mode, label)) in tabs.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw("   "));
            }
            spans.push(Self::tab(label, state.mode == *mode));
        }
        frame.render_widget(Line::from(spans), left);

        let title = Line::from(vec![
            Span::styled("lazyprop ", Style::default().fg(theme::accent())),
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
