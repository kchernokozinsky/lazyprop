use color_eyre::Result;
use ratatui::{
    prelude::*,
    widgets::{block::*, *},
};

use super::Component;
use crate::{
    state::{Playground, PlaygroundField, State},
    text_field::TextField,
    theme,
};

/// The "playground" screen: encrypt/decrypt with parameters typed directly,
/// without a saved environment. Mirrors MuleSoft's Secure Properties Tool
/// "string" form (Operation, Algorithm, State, Key, Value → Result).
#[derive(Default)]
pub struct PlaygroundScreen {}

impl PlaygroundScreen {
    pub fn new() -> Self {
        Self {}
    }
}

fn active_border(active: bool) -> (Style, BorderType) {
    if active {
        (Style::default().fg(theme::ACCENT), BorderType::Thick)
    } else {
        (Style::default(), BorderType::Plain)
    }
}

fn choice_box(frame: &mut Frame, area: Rect, title: &str, value: &str, active: bool) {
    let (style, btype) = active_border(active);
    let block = Block::default()
        .title(format!(" {title} "))
        .borders(Borders::ALL)
        .border_style(style)
        .border_type(btype);
    let content = if active {
        format!("‹ {value} ›")
    } else {
        value.to_string()
    };
    let para = Paragraph::new(Line::from(Span::styled(
        content,
        Style::default().add_modifier(Modifier::BOLD),
    )))
    .block(block)
    .alignment(Alignment::Center);
    frame.render_widget(para, area);
}

fn text_box(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    value: &TextField,
    active: bool,
    placeholder: &str,
) {
    let (style, btype) = active_border(active);
    let block = Block::default()
        .title(format!(" {title} "))
        .borders(Borders::ALL)
        .border_style(style)
        .border_type(btype);
    let inner_width = (area.width as usize).saturating_sub(2);
    let spans = value.spans(inner_width, active, placeholder);
    let para = Paragraph::new(Line::from(spans)).block(block);
    frame.render_widget(para, area);
}

fn draw_result(frame: &mut Frame, area: Rect, p: &Playground, busy: bool) {
    let (title, body, style) = if busy {
        (
            " Result ".to_string(),
            Text::from(Span::styled("Working…", theme::hint_italic())),
            Style::default().fg(theme::ACCENT),
        )
    } else {
        match &p.result {
            None => (
                " Result ".to_string(),
                Text::from(Span::styled(
                    "Fill in the fields and press Enter to generate.",
                    theme::hint_italic(),
                )),
                Style::default(),
            ),
            Some(res) => match &res.outcome {
                Ok(output) => (
                    format!(" {} ", res.op.label()),
                    Text::from(output.clone()),
                    Style::default().fg(theme::SUCCESS),
                ),
                Err(err) => (
                    " Error ".to_string(),
                    Text::from(err.clone()),
                    Style::default().fg(theme::ERROR),
                ),
            },
        }
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(style);
    frame.render_widget(
        Paragraph::new(body).block(block).wrap(Wrap { trim: false }),
        area,
    );
}

impl Component for PlaygroundScreen {
    fn draw(&mut self, frame: &mut Frame, area: Rect, state: &State) -> Result<()> {
        let p = &state.playground;

        let [selectors, key_area, value_area, result_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .areas(area);

        let [op, alg, mode, iv] = Layout::horizontal([Constraint::Ratio(1, 4); 4]).areas(selectors);
        choice_box(
            frame,
            op,
            "Operation",
            p.operation.name(),
            p.field == PlaygroundField::Operation,
        );
        choice_box(
            frame,
            alg,
            "Algorithm",
            &format!("{:?}", p.algorithm),
            p.field == PlaygroundField::Algorithm,
        );
        let mode_text = if p.algorithm.supports_modes() {
            format!("{:?}", p.cipher)
        } else {
            "n/a".to_string()
        };
        choice_box(
            frame,
            mode,
            "State",
            &mode_text,
            p.field == PlaygroundField::Mode,
        );
        choice_box(
            frame,
            iv,
            "Random IV",
            if p.use_random_ivs { "yes" } else { "no" },
            p.field == PlaygroundField::RandomIv,
        );

        text_box(
            frame,
            key_area,
            "Key",
            &p.key,
            p.field == PlaygroundField::Key,
            "encryption key",
        );
        text_box(
            frame,
            value_area,
            "Value",
            &p.value,
            p.field == PlaygroundField::Value,
            "text to process",
        );

        draw_result(frame, result_area, p, state.busy);
        Ok(())
    }
}
