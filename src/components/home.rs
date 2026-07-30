use color_eyre::Result;
use ratatui::{
    prelude::*,
    widgets::{block::*, *},
};
use tokio::sync::mpsc::UnboundedSender;

use super::Component;
use crate::{
    action::Action,
    panes::{
        details::DetailsPane, envs::EnvsPane, input::InputPane, result::ResultPane,
        status::StatusPane, Pane,
    },
    state::{FormField, InputMode, State},
    theme,
};

/// Which focusable pane currently receives navigation / typing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Envs,
    Input,
}

impl Focus {
    fn next(self) -> Self {
        match self {
            Focus::Envs => Focus::Input,
            Focus::Input => Focus::Envs,
        }
    }
}

pub struct Home {
    command_tx: Option<UnboundedSender<Action>>,
    focus: Focus,
    envs: EnvsPane,
    input: InputPane,
    details: DetailsPane,
    result: ResultPane,
    status: StatusPane,
}

impl Home {
    pub fn new() -> Result<Self> {
        let focused_border_style = Style::default().fg(theme::ACCENT);
        Ok(Self {
            command_tx: None,
            focus: Focus::Envs,
            envs: EnvsPane::new(true, focused_border_style),
            input: InputPane::new(false, focused_border_style),
            details: DetailsPane::new(),
            result: ResultPane::new(),
            status: StatusPane::new(),
        })
    }

    /// Move focus to `target`, updating pane state and the global input mode.
    fn set_focus(&mut self, target: Focus, state: &mut State) -> Result<()> {
        self.envs.update(Action::UnFocus, state)?;
        self.input.update(Action::UnFocus, state)?;
        match target {
            Focus::Envs => self.envs.update(Action::Focus, state)?,
            Focus::Input => self.input.update(Action::Focus, state)?,
        };
        self.focus = target;
        Ok(())
    }

    fn draw_env_form(&self, frame: &mut Frame, area: Rect, state: &State) {
        let Some(form) = &state.form else { return };

        let popup = centered_rect(60, 60, area);
        // Width available for a text field's value (borders + marker + label).
        let field_width = (popup.width as usize).saturating_sub(2 + 2 + 12 + 1);

        let mut lines: Vec<Line> = vec![
            Line::raw(""),
            form_text_line(
                "Name",
                &form.name,
                form.field == FormField::Name,
                field_width,
            ),
            form_choice_line(
                "Algorithm",
                &format!("{:?}", form.algorithm),
                form.field == FormField::Algorithm,
            ),
            form_choice_line(
                "Mode",
                &if form.algorithm.supports_modes() {
                    format!("{:?}", form.cipher)
                } else {
                    "n/a".to_string()
                },
                form.field == FormField::Mode,
            ),
            form_choice_line(
                "Random IV",
                if form.use_random_ivs { "yes" } else { "no" },
                form.field == FormField::RandomIv,
            ),
            form_text_line("Key", &form.key, form.field == FormField::Key, field_width),
            Line::raw(""),
        ];
        if let Some(err) = &form.error {
            lines.push(Line::from(Span::styled(
                format!("  {err}"),
                Style::default().fg(theme::ERROR),
            )));
        }
        lines.push(Line::from(Span::styled(
            "  Tab/↑↓ move · ←/→ change · Enter save · Esc cancel",
            theme::hint(),
        )));

        frame.render_widget(Clear, popup);
        let block = Block::default()
            .title(form.title())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::ACCENT));
        frame.render_widget(Paragraph::new(lines).block(block), popup);
    }

    fn draw_delete_confirm(&self, frame: &mut Frame, area: Rect, state: &State) {
        let Some(name) = state.pending_delete_name() else {
            return;
        };
        let popup = centered_rect(50, 20, area);
        frame.render_widget(Clear, popup);
        let lines = vec![
            Line::raw(""),
            Line::from(vec![
                Span::raw("  Delete environment "),
                Span::styled(
                    format!("'{name}'"),
                    Style::default()
                        .fg(theme::ERROR)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("?"),
            ]),
            Line::raw(""),
            Line::from(Span::styled("  y confirm · n / Esc cancel", theme::hint())),
        ];
        let block = Block::default()
            .title(" Confirm delete ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::ERROR));
        frame.render_widget(Paragraph::new(lines).block(block), popup);
    }
}

fn form_text_line(
    label: &str,
    value: &crate::text_field::TextField,
    active: bool,
    width: usize,
) -> Line<'static> {
    let mut spans = vec![
        Span::styled(
            if active { "▶ " } else { "  " },
            Style::default().fg(theme::ACCENT),
        ),
        Span::styled(format!("{label:>10}: "), theme::label()),
    ];
    spans.extend(value.spans(width, active, ""));
    Line::from(spans)
}

fn form_choice_line(label: &str, value: &str, active: bool) -> Line<'static> {
    let value = if active {
        format!("‹ {value} ›")
    } else {
        format!("  {value}")
    };
    Line::from(vec![
        Span::styled(
            if active { "▶ " } else { "  " },
            Style::default().fg(theme::ACCENT),
        ),
        Span::styled(format!("{label:>10}: "), theme::label()),
        Span::raw(value),
    ])
}

impl Component for Home {
    fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
        self.command_tx = Some(tx);
        Ok(())
    }

    fn update(&mut self, action: Action, state: &mut State) -> Result<Option<Action>> {
        match action {
            Action::Down | Action::Up => return self.envs.update(action, state),
            Action::Tab => {
                if state.searching {
                    state.searching = false;
                    state.input_mode = InputMode::Normal;
                }
                let next = self.focus.next();
                self.set_focus(next, state)?;
            }
            Action::Search => {
                self.set_focus(Focus::Envs, state)?;
                state.searching = true;
                state.input_mode = InputMode::Insert;
            }
            Action::Escape => {
                if state.searching {
                    state.searching = false;
                    state.input_mode = InputMode::Normal;
                } else {
                    self.set_focus(Focus::Envs, state)?;
                }
            }
            Action::AddEnv => state.open_add_form(),
            Action::EditEnv => state.open_edit_form(),
            Action::DeleteEnv => state.request_delete(),
            Action::Input(c) => {
                if state.searching {
                    state.push_search(c);
                } else if self.focus == Focus::Input {
                    state.input_value.insert(c);
                }
            }
            Action::Backspace => {
                if state.searching {
                    state.pop_search();
                } else if self.focus == Focus::Input {
                    state.input_value.backspace();
                }
            }
            Action::DeleteChar if self.focus == Focus::Input => state.input_value.delete(),
            Action::CursorLeft if self.focus == Focus::Input => state.input_value.left(),
            Action::CursorRight if self.focus == Focus::Input => state.input_value.right(),
            Action::CursorHome if self.focus == Focus::Input => state.input_value.home(),
            Action::CursorEnd if self.focus == Focus::Input => state.input_value.end(),
            Action::Error(_) | Action::Message(_) => return self.status.update(action, state),
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, state: &State) -> Result<()> {
        let [main_area, status_area] =
            Layout::vertical([Constraint::Fill(1), self.status.height_constraint()]).areas(area);

        let [left, right] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(main_area);

        let [details_area, input_area, result_area] = Layout::vertical([
            self.details.height_constraint(),
            self.input.height_constraint(),
            Constraint::Fill(1),
        ])
        .areas(right);

        self.envs.draw(frame, left, state)?;
        self.details.draw(frame, details_area, state)?;
        self.input.draw(frame, input_area, state)?;
        self.result.draw(frame, result_area, state)?;
        self.status.draw(frame, status_area, state)?;

        if state.form.is_some() {
            self.draw_env_form(frame, area, state);
        }
        if state.pending_delete.is_some() {
            self.draw_delete_confirm(frame, area, state);
        }
        Ok(())
    }
}

/// Compute a centered rectangle taking `percent_x`/`percent_y` of `area`.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);
    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1])[1]
}
