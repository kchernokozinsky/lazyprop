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
    config::{get_config_dir, get_data_dir, key_event_to_string, Config},
    state::State,
    theme,
};

/// The "About / Help" screen: what lazyprop is, the keybindings, and where its
/// files live. Scrollable with the same navigation keys as the main screen.
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

    /// Keybinding lines for the main screen, read from the active config.
    fn keybinding_lines(&self) -> Vec<Line<'static>> {
        let mut bindings: Vec<(String, String)> = self
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
                        (key, action.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default();
        bindings.sort_by(|a, b| a.1.cmp(&b.1));

        bindings
            .into_iter()
            .map(|(key, action)| {
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(format!("{key:<10}"), theme::key()),
                    Span::raw(" "),
                    Span::raw(action),
                ])
            })
            .collect()
    }

    fn content(&self) -> Vec<Line<'static>> {
        let mut lines = vec![
            Line::from(Span::styled(
                "lazyprop",
                Style::default()
                    .fg(theme::ACCENT)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(env!("CARGO_PKG_DESCRIPTION"), theme::hint())),
            Line::raw(""),
            Self::section("What it does"),
            Line::raw("  Pick an environment, type a value, and encrypt or decrypt it"),
            Line::raw("  with MuleSoft's Secure Properties Tool — no Java command line"),
            Line::raw("  to remember. Environments (algorithm, mode and key) are read"),
            Line::raw("  from a YAML file you can edit from inside the app."),
            Line::raw(""),
            Self::section("Keybindings"),
        ];
        lines.extend(self.keybinding_lines());
        lines.push(Line::raw(""));
        lines.push(Self::section("About"));
        for (k, v) in [
            ("Version", VERSION_MESSAGE.to_string()),
            ("Author", env!("CARGO_PKG_AUTHORS").to_string()),
            ("Repo", env!("CARGO_PKG_REPOSITORY").to_string()),
            ("Config", get_config_dir().display().to_string()),
            ("Data", get_data_dir().display().to_string()),
        ] {
            lines.push(Line::from(vec![
                Span::styled(format!("  {k:>8}: "), theme::label()),
                Span::raw(v),
            ]));
        }
        lines.push(Line::raw(""));
        lines.push(Line::from(Span::styled(
            "  Press 1 or Esc to return to the main screen.",
            theme::hint(),
        )));
        lines
    }
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
        let lines = self.content();
        self.content_len = lines.len() as u16;

        let block = Block::default()
            .title(" About lazyprop ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::ACCENT));

        let paragraph = Paragraph::new(lines)
            .block(block)
            .scroll((self.scroll, 0))
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
        Ok(())
    }
}
