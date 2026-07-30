use color_eyre::Result;
use ratatui::{
    prelude::*,
    widgets::{block::*, *},
};

use super::Component;
use crate::{
    action::Action,
    app::Mode,
    cli::VERSION_MESSAGE,
    config::{key_event_to_string, Config},
    state::State,
    theme,
};

/// ASCII art logo shown at the top of the About screen.
const LOGO: &str = include_str!("../../assets/logo.txt");

/// The "About / Help" screen: the logo, keybindings and app info. Scrollable
/// with the same navigation keys as the main screen.
#[derive(Default)]
pub struct AboutScreen {
    config: Config,
    scroll: u16,
    content_len: u16,
}

impl AboutScreen {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            scroll: 0,
            content_len: 0,
        }
    }

    fn section(title: &str) -> Line<'static> {
        Line::from(Span::styled(title.to_string(), theme::label()))
    }

    /// Keybinding rows for the main screen, read from the active config and
    /// shown with friendly descriptions (never raw action names).
    fn keybinding_lines(&self) -> Vec<Line<'static>> {
        let mut bindings: Vec<(String, &'static str)> = self
            .config
            .keybindings
            .get(&Mode::Main)
            .map(|km| {
                km.iter()
                    .map(|(keys, action)| {
                        let key = keys
                            .iter()
                            .map(key_event_to_string)
                            .collect::<Vec<_>>()
                            .join(" ");
                        (key, action.description())
                    })
                    .filter(|(_, desc)| !desc.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        bindings.sort_by(|a, b| a.1.cmp(b.1));

        bindings
            .into_iter()
            .map(|(key, desc)| {
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(format!("{key:<12}"), theme::key()),
                    Span::raw("  "),
                    Span::raw(desc),
                ])
            })
            .collect()
    }

    fn content(&self, width: usize) -> Vec<Line<'static>> {
        let mut lines: Vec<Line> = LOGO
            .lines()
            .map(|l| {
                Line::from(Span::styled(
                    l.to_string(),
                    Style::default().fg(theme::ACCENT),
                ))
            })
            .collect();

        lines.push(Line::raw(""));
        lines.extend(wrap_styled(
            env!("CARGO_PKG_DESCRIPTION"),
            width,
            2,
            theme::hint(),
        ));

        lines.push(Line::raw(""));
        lines.push(Self::section("Keybindings"));
        lines.extend(self.keybinding_lines());

        lines.push(Line::raw(""));
        lines.push(Self::section("About"));
        for (label, value) in [
            ("Version", VERSION_MESSAGE.to_string()),
            ("Author", env!("CARGO_PKG_AUTHORS").to_string()),
            ("Repo", env!("CARGO_PKG_REPOSITORY").to_string()),
        ] {
            lines.push(Line::from(vec![
                Span::styled(format!("  {label:<8}"), theme::label()),
                Span::raw("  "),
                Span::raw(value),
            ]));
        }

        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled(
            "Switch screens: 1 Main · 2 Playground · 3 About · h/l · Esc",
            theme::hint(),
        )));
        lines
    }
}

/// Word-wrap `text` to `width` cells with a left `indent`, returning styled lines.
fn wrap_styled(text: &str, width: usize, indent: usize, style: Style) -> Vec<Line<'static>> {
    let pad = " ".repeat(indent);
    let avail = width.saturating_sub(indent).max(1);
    let mut out = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        if cur.is_empty() {
            cur = word.to_string();
        } else if cur.chars().count() + 1 + word.chars().count() <= avail {
            cur.push(' ');
            cur.push_str(word);
        } else {
            out.push(Line::from(Span::styled(format!("{pad}{cur}"), style)));
            cur = word.to_string();
        }
    }
    if !cur.is_empty() {
        out.push(Line::from(Span::styled(format!("{pad}{cur}"), style)));
    }
    out
}

impl Component for AboutScreen {
    fn register_config_handler(&mut self, config: Config) -> Result<()> {
        self.config = config;
        Ok(())
    }

    fn update(&mut self, action: Action, _state: &mut State) -> Result<Option<Action>> {
        match action {
            Action::Down => {
                self.scroll = (self.scroll + 1).min(self.content_len.saturating_sub(1));
            }
            Action::Up => self.scroll = self.scroll.saturating_sub(1),
            _ => {}
        }
        Ok(None)
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect, _state: &State) -> Result<()> {
        let block = Block::default()
            .title(" About lazyprop ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::ACCENT))
            // A bit of breathing room between the frame and the text.
            .padding(Padding::new(2, 2, 1, 0));

        // Width available for text (frame borders + horizontal padding).
        let width = (area.width as usize).saturating_sub(2 + 4);
        let lines = self.content(width);
        self.content_len = lines.len() as u16;

        let paragraph = Paragraph::new(lines).block(block).scroll((self.scroll, 0));
        frame.render_widget(paragraph, area);
        Ok(())
    }
}
