use color_eyre::eyre::Result;
use ratatui::prelude::*;

use crate::{app::Mode, cli::VERSION_MESSAGE, panes::Pane, state::State, theme};

#[derive(Default)]
pub struct HeaderPane {
    /// Clickable tab regions from the last draw: `(mode, row, start_col, end_col)`.
    tabs: Vec<(Mode, u16, u16, u16)>,
}

impl HeaderPane {
    pub fn new() -> Self {
        Self { tabs: Vec::new() }
    }

    /// The screen whose tab covers `(col, row)`, if any (for mouse clicks).
    pub fn tab_at(&self, col: u16, row: u16) -> Option<Mode> {
        self.tabs
            .iter()
            .find(|(_, r, start, end)| *r == row && col >= *start && col < *end)
            .map(|(mode, _, _, _)| *mode)
    }

    /// A tab as two spans: a dimmed shortcut number and the screen name.
    /// The active screen's name is accented, bold and underlined; the rest dim.
    fn tab(num: &str, label: &str, active: bool) -> [Span<'static>; 3] {
        let name_style = if active {
            Style::default()
                .fg(theme::accent())
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
        let tabs = [
            (Mode::Main, "1", "Main"),
            (Mode::Playground, "2", "Playground"),
            (Mode::Yaml, "3", "YAML"),
            (Mode::About, "4", "About"),
        ];
        let mut spans = vec![Span::raw(" ")];
        // Track each tab's clickable column range for mouse hit-testing.
        self.tabs.clear();
        let mut col = left.x + 1; // leading space
        for (i, (mode, num, label)) in tabs.iter().enumerate() {
            if i > 0 {
                spans.push(gap());
                col += 3;
            }
            spans.extend(Self::tab(num, label, state.mode == *mode));
            let width = (num.len() + 1 + label.len()) as u16;
            self.tabs.push((*mode, left.y, col, col + width));
            col += width;
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
