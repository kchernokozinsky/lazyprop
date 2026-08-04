use crate::state::State;
use color_eyre::eyre::Result;
use ratatui::{
    prelude::*,
    widgets::{block::*, *},
};

use crate::{panes::Pane, theme};

/// Displays the outcome of the most recent encrypt/decrypt run.
#[derive(Default)]
pub struct ResultPane {}

impl ResultPane {
    pub fn new() -> Self {
        Self {}
    }
}

impl Pane for ResultPane {
    fn height_constraint(&self) -> Constraint {
        Constraint::Fill(1)
    }

    fn draw(&mut self, frame: &mut Frame<'_>, area: Rect, state: &State) -> Result<()> {
        let (title, body, style) = if state.busy {
            (
                " Result ".to_string(),
                Text::from(Span::styled("Working…", theme::hint_italic())),
                Style::default().fg(theme::accent()),
            )
        } else {
            match &state.result {
                None => (
                    " Result ".to_string(),
                    Text::from(vec![
                        Line::from(Span::styled(
                            "No result yet.",
                            Style::default().add_modifier(Modifier::ITALIC),
                        )),
                        Line::from(Span::styled(
                            "e encrypt · d decrypt · Ctrl-y copy",
                            theme::hint_italic(),
                        )),
                    ]),
                    Style::default(),
                ),
                Some(res) => match &res.outcome {
                    Ok(output) => (
                        format!(" {} ", res.op.label()),
                        Text::from(output.clone()),
                        Style::default().fg(theme::success()),
                    ),
                    Err(err) => (
                        " Error ".to_string(),
                        Text::from(err.clone()),
                        Style::default().fg(theme::error()),
                    ),
                },
            }
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(style);

        let paragraph = Paragraph::new(body).block(block).wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
        Ok(())
    }

    fn focusable(&self) -> bool {
        false
    }
}
