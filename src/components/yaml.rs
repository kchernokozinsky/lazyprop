use color_eyre::Result;
use ratatui::{
    prelude::*,
    widgets::{block::*, *},
};

use super::Component;
use crate::{
    state::State,
    theme,
    yaml_editor::{
        document::{self, NodeKind, ScalarStyle},
        state::{Confirm, OpenMode, YamlFocus},
    },
};

/// The YAML editor screen. Key handling lives in `yaml_editor::input`.
#[derive(Default)]
pub struct YamlScreen {}

impl YamlScreen {
    pub fn new() -> Self {
        Self {}
    }
}

fn focus_border(active: bool) -> (Style, BorderType) {
    if active {
        (Style::default().fg(theme::ACCENT), BorderType::Thick)
    } else {
        (Style::default(), BorderType::Plain)
    }
}

impl Component for YamlScreen {
    fn draw(&mut self, frame: &mut Frame, area: Rect, state: &State) -> Result<()> {
        let y = &state.yaml;

        let [info, body] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);
        draw_info(frame, info, state);

        if !y.is_open() {
            let hint = Paragraph::new(
                "No file open. Press Ctrl-o to open a .yaml/.yml file (browse or type a path).",
            )
            .style(theme::hint_italic())
            .block(Block::default().borders(Borders::ALL).title(" YAML "));
            frame.render_widget(hint, body);
            draw_overlays(frame, area, state);
            return Ok(());
        }

        let [left, right] =
            Layout::horizontal([Constraint::Length(26), Constraint::Fill(1)]).areas(body);
        draw_environments(frame, left, state);

        let [tree_area, prop_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(8)]).areas(right);
        draw_tree(frame, tree_area, state);
        draw_property(frame, prop_area, state);

        draw_overlays(frame, area, state);
        Ok(())
    }
}

fn draw_info(frame: &mut Frame, area: Rect, state: &State) {
    let y = &state.yaml;
    let env = state
        .selected_env()
        .map(|e| e.name.clone())
        .unwrap_or_else(|| "none".to_string());
    let path = y
        .file_path
        .as_ref()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "(no file)".to_string());
    let mut spans = vec![
        Span::styled(" YAML ", theme::label()),
        Span::styled("│ ", theme::hint()),
        Span::raw(path),
        Span::styled(" │ ", theme::hint()),
    ];
    if y.dirty() {
        spans.push(Span::styled("Modified", Style::default().fg(theme::ERROR)));
    } else {
        spans.push(Span::styled("Saved", theme::hint()));
    }
    spans.push(Span::styled(" │ Env: ", theme::hint()));
    spans.push(Span::styled(env, Style::default().fg(theme::ACCENT)));
    frame.render_widget(Line::from(spans), area);
}

fn draw_environments(frame: &mut Frame, area: Rect, state: &State) {
    let active = state.yaml.focus == YamlFocus::Environments;
    let (style, btype) = focus_border(active);
    let items: Vec<ListItem> = state
        .envs
        .environments
        .iter()
        .map(|e| ListItem::new(e.name.clone()))
        .collect();
    let block = Block::default()
        .title(" Environments ")
        .borders(Borders::ALL)
        .border_style(style)
        .border_type(btype);
    let list = List::new(items)
        .block(block)
        .highlight_symbol("> ")
        .highlight_style(
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD),
        );
    let mut ls = ListState::default().with_selected(Some(state.cur()));
    frame.render_stateful_widget(list, area, &mut ls);
}

fn draw_tree(frame: &mut Frame, area: Rect, state: &State) {
    let y = &state.yaml;
    let (style, btype) = focus_border(y.focus == YamlFocus::Tree);
    let mut title = " YAML tree ".to_string();
    if let Some(q) = y.search_query() {
        let editing = if y.search_editing { "_" } else { "" };
        title = format!(" YAML tree — /{q}{editing} ");
    } else if y.search_editing {
        title = " YAML tree — /_ ".to_string();
    }
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(style)
        .border_type(btype);
    let inner_h = area.height.saturating_sub(2) as usize;
    let visible = y.visible();
    let sel = y.selected_index().unwrap_or(0);
    // Simple auto-scroll to keep the selection in view.
    let offset = if sel >= inner_h { sel + 1 - inner_h } else { 0 };

    let mut lines: Vec<Line> = Vec::new();
    for (i, &id) in visible.iter().enumerate().skip(offset).take(inner_h) {
        let node = &y.doc().nodes()[id];
        let indent = "  ".repeat(node.depth);
        let marker = match node.kind {
            NodeKind::Scalar => "  ".to_string(),
            _ => {
                if y.is_expanded(&node.path) {
                    "v ".to_string()
                } else {
                    "> ".to_string()
                }
            }
        };
        let mut spans = vec![Span::raw(format!("{indent}{marker}"))];
        let label_style = if i == sel {
            Style::default()
                .fg(theme::ACCENT)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        spans.push(Span::styled(node.label.clone(), label_style));
        if node.kind == NodeKind::Scalar {
            let val = display_value(y, id);
            spans.push(Span::styled(format!(": {val}"), theme::hint()));
        }
        lines.push(Line::from(spans));
    }
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_property(frame: &mut Frame, area: Rect, state: &State) {
    let y = &state.yaml;
    let block = Block::default().title(" Selected ").borders(Borders::ALL);
    let mut lines: Vec<Line> = Vec::new();

    if let Some(field) = &y.editing {
        let width = area.width.saturating_sub(2) as usize;
        let mut spans = vec![Span::styled("edit: ", theme::label())];
        spans.extend(field.spans(width.saturating_sub(6), true, ""));
        lines.push(Line::from(spans));
        lines.push(Line::from(Span::styled(
            "Enter apply · Esc cancel",
            theme::hint(),
        )));
    } else if let Some(path) = y.selected_path_string() {
        lines.push(Line::from(vec![
            Span::styled("path: ", theme::label()),
            Span::raw(path),
        ]));
        if let Some((_src, style, kind)) = y.selected_info() {
            let sel_id = y.selected_id().unwrap();
            let (type_str, encrypted) = match kind {
                NodeKind::Scalar => {
                    let logical = y.doc().logical_value(sel_id).unwrap_or_default();
                    (scalar_type(style, &logical), document::is_wrapped(&logical))
                }
                NodeKind::Mapping => ("mapping".to_string(), false),
                NodeKind::Sequence => ("sequence".to_string(), false),
            };
            lines.push(Line::from(vec![
                Span::styled("value: ", theme::label()),
                Span::raw(display_value(y, sel_id)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("type: ", theme::label()),
                Span::raw(type_str),
                Span::styled("   encrypted: ", theme::label()),
                Span::raw(if encrypted { "yes" } else { "no" }),
            ]));
        }
    }

    if let Some((msg, is_err)) = y.message() {
        let st = if is_err {
            Style::default().fg(theme::ERROR)
        } else {
            Style::default().fg(theme::SUCCESS)
        };
        lines.push(Line::from(Span::styled(msg.to_string(), st)));
    }

    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: true }),
        area,
    );
}

/// The displayed value of a scalar, masking encrypted values unless revealed.
fn display_value(y: &crate::yaml_editor::state::YamlEditorState, id: usize) -> String {
    let Some(logical) = y.doc().logical_value(id) else {
        return String::new();
    };
    if document::is_wrapped(&logical) && !y.reveal {
        "![••••••]".to_string()
    } else {
        y.doc().value_source(id).unwrap_or("").to_string()
    }
}

fn scalar_type(style: ScalarStyle, logical: &str) -> String {
    match style {
        ScalarStyle::SingleQuoted | ScalarStyle::DoubleQuoted => "string (quoted)".to_string(),
        ScalarStyle::Unsupported => "unsupported".to_string(),
        ScalarStyle::Plain => {
            let low = logical.to_ascii_lowercase();
            if logical.is_empty() {
                "null".to_string()
            } else if matches!(low.as_str(), "true" | "false") {
                "boolean".to_string()
            } else if matches!(low.as_str(), "null" | "~") {
                "null".to_string()
            } else if logical.parse::<f64>().is_ok() {
                "number".to_string()
            } else {
                "string".to_string()
            }
        }
    }
}

fn draw_overlays(frame: &mut Frame, area: Rect, state: &State) {
    let y = &state.yaml;
    if let Some(modal) = &y.open_modal {
        let popup = centered(70, 70, area);
        frame.render_widget(Clear, popup);
        let title = match modal.mode {
            OpenMode::Browse => " Open YAML — Browse (Tab: type a path) ",
            OpenMode::Path => " Open YAML — Path (Tab: browse) ",
        };
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::ACCENT));
        let inner = block.inner(popup);
        frame.render_widget(block, popup);

        match modal.mode {
            OpenMode::Browse => {
                let items: Vec<ListItem> = modal
                    .browser
                    .entries
                    .iter()
                    .map(|e| {
                        let style = if e.is_dir {
                            Style::default().fg(theme::ACCENT)
                        } else {
                            Style::default()
                        };
                        ListItem::new(Span::styled(e.label.clone(), style))
                    })
                    .collect();
                let list = List::new(items).highlight_symbol("> ").highlight_style(
                    Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED),
                );
                let mut ls = ListState::default().with_selected(Some(modal.browser.selected));
                let [dir, body, hint] = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Fill(1),
                    Constraint::Length(1),
                ])
                .areas(inner);
                frame.render_widget(
                    Line::from(Span::styled(
                        modal.browser.cwd.display().to_string(),
                        theme::hint(),
                    )),
                    dir,
                );
                frame.render_stateful_widget(list, body, &mut ls);
                let msg = modal
                    .browser
                    .error
                    .clone()
                    .or_else(|| modal.error.clone())
                    .unwrap_or_else(|| "↑↓ move · Enter open · Esc cancel".to_string());
                frame.render_widget(Line::from(Span::styled(msg, theme::hint())), hint);
            }
            OpenMode::Path => {
                let [label, input, hint] = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Fill(1),
                ])
                .areas(inner);
                frame.render_widget(Line::from(Span::styled("Path:", theme::label())), label);
                let spans = modal.path_input.spans(
                    input.width as usize,
                    true,
                    "e.g. ./config.yaml or ~/config.yml",
                );
                frame.render_widget(Line::from(spans), input);
                let msg = modal
                    .error
                    .clone()
                    .unwrap_or_else(|| "Enter to open · Esc cancel".to_string());
                frame.render_widget(
                    Line::from(Span::styled(msg, Style::default().fg(theme::ERROR))),
                    hint,
                );
            }
        }
    }

    if let Some(confirm) = &y.confirm {
        let popup = centered(50, 22, area);
        frame.render_widget(Clear, popup);
        let text = match confirm {
            Confirm::Restore => "Discard unsaved changes and restore the opened content?",
            Confirm::OverwriteExternal => "File changed on disk. Overwrite it?",
        };
        let lines = vec![
            Line::raw(""),
            Line::from(Span::raw(format!("  {text}"))),
            Line::raw(""),
            Line::from(Span::styled("  y confirm · n / Esc cancel", theme::hint())),
        ];
        let block = Block::default()
            .title(" Confirm ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::ERROR));
        frame.render_widget(Paragraph::new(lines).block(block), popup);
    }
}

fn centered(px: u16, py: u16, area: Rect) -> Rect {
    let v = Layout::vertical([
        Constraint::Percentage((100 - py) / 2),
        Constraint::Percentage(py),
        Constraint::Percentage((100 - py) / 2),
    ])
    .split(area);
    Layout::horizontal([
        Constraint::Percentage((100 - px) / 2),
        Constraint::Percentage(px),
        Constraint::Percentage((100 - px) / 2),
    ])
    .split(v[1])[1]
}
