use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
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

/// The guides shown on the About screen — one dedicated page per app screen
/// plus general and configuration guidance.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Guide {
    #[default]
    Main,
    Playground,
    Yaml,
    General,
}

impl Guide {
    pub const ALL: [Guide; 4] = [Guide::Main, Guide::Playground, Guide::Yaml, Guide::General];

    pub fn title(self) -> &'static str {
        match self {
            Guide::Main => "Main",
            Guide::Playground => "Playground",
            Guide::Yaml => "YAML",
            Guide::General => "General",
        }
    }

    fn shift(self, forward: bool) -> Self {
        let i = Self::ALL.iter().position(|&g| g == self).unwrap_or(0);
        let len = Self::ALL.len();
        Self::ALL[if forward {
            (i + 1) % len
        } else {
            (i + len - 1) % len
        }]
    }
}

/// A single key → action row in a guide.
struct KeyRow {
    key: &'static str,
    desc: &'static str,
}

/// A titled block of keybindings within a guide page, with an optional one-line
/// note.
struct GuideBlock {
    heading: &'static str,
    keys: &'static [KeyRow],
    note: Option<&'static str>,
}

/// The static content of a single guide (the General guide also appends
/// config-derived rows at render time).
struct GuideContent {
    /// A one-line summary shown under the heading.
    summary: &'static str,
    blocks: &'static [GuideBlock],
}

macro_rules! keys {
    ($( $k:literal => $d:literal ),* $(,)?) => {
        &[ $( KeyRow { key: $k, desc: $d } ),* ]
    };
}

fn guide_content(guide: Guide) -> GuideContent {
    match guide {
        Guide::Main => GuideContent {
            summary: "Encrypt/decrypt values against a saved environment.",
            blocks: &[
                GuideBlock {
                    heading: "Environments",
                    keys: keys![
                        "W / S"  => "Select environment",
                        "A"      => "Add environment",
                        "Enter"  => "Edit environment",
                        "X"      => "Delete environment",
                        "/"      => "Filter the list",
                        "R"      => "Reveal / hide key",
                    ],
                    note: None,
                },
                GuideBlock {
                    heading: "Encrypt / decrypt",
                    keys: keys![
                        "Tab"    => "Focus the value field",
                        "E"      => "Encrypt value",
                        "D"      => "Decrypt value",
                        "Ctrl+Y" => "Copy result",
                        "Esc"    => "Leave the field",
                    ],
                    note: None,
                },
            ],
        },
        Guide::Playground => GuideContent {
            summary: "One-off encrypt/decrypt, no saved environment.",
            blocks: &[GuideBlock {
                heading: "Fields & run",
                keys: keys![
                    "Tab / ↑ / ↓" => "Move between fields",
                    "← / →"       => "Change a choice",
                    "Enter"       => "Generate result",
                    "Ctrl+Y"      => "Copy result",
                    "Esc"         => "Back to Main",
                ],
                note: Some("Type into the Key and Value fields; choices are Operation, Algorithm, Mode and Random IV."),
            }],
        },
        Guide::Yaml => GuideContent {
            summary: "Encrypt/decrypt values inside a .yaml/.yml file, in place.",
            blocks: &[
                GuideBlock {
                    heading: "Open & navigate",
                    keys: keys![
                        "Ctrl+O" => "Open a file (browse / path)",
                        "W / S"  => "Move in the tree",
                        "← / →"  => "Collapse / expand",
                        "Tab"    => "Switch pane",
                        "/"      => "Search the tree",
                    ],
                    note: None,
                },
                GuideBlock {
                    heading: "Edit & crypt",
                    keys: keys![
                        "Enter"           => "Edit scalar",
                        "e / d"           => "Encrypt / decrypt value",
                        "E / D"           => "Bulk on the subtree",
                        "R"               => "Reveal value",
                        "A"               => "Add environment",
                        "Ctrl+Z / Ctrl+Y" => "Undo / redo",
                    ],
                    note: None,
                },
                GuideBlock {
                    heading: "Save & restore",
                    keys: keys![
                        "Ctrl+S" => "Save (atomic)",
                        "Ctrl+R" => "Restore to opened",
                    ],
                    note: Some("A ● marks each modified property until it is saved, restored, or edited back to its original value. Flow style, block scalars and anchors are not editable in place."),
                },
            ],
        },
        Guide::General => GuideContent {
            summary: "Application-wide navigation and configuration.",
            blocks: &[
                GuideBlock {
                    heading: "Navigation",
                    keys: keys![
                        "1 / 2 / 3 / 4" => "Jump to a screen",
                        "h / l"         => "Previous / next screen",
                        "?"             => "About / help",
                        "Esc"           => "Close modal / cancel",
                        "Q / Ctrl+C"    => "Quit",
                    ],
                    note: Some("Confirmations use Y / Esc; the unsaved-changes prompt uses S (save), D (discard) and Esc."),
                },
                GuideBlock {
                    heading: "Configuration",
                    keys: &[],
                    note: Some("Keybindings and styles load from a config file (else the bundled defaults). Environments: --envs / LAZYPROP_ENVS / ./envs.yaml / ~/.lazyprop/envs.yaml."),
                },
            ],
        },
    }
}

/// The "About / Help" screen: a tabbed set of page-specific guides. Left/Right
/// switch guide; the shared footer shows the contextual hints.
#[derive(Default)]
pub struct AboutScreen {
    config: Config,
    guide: Guide,
    scroll: u16,
    content_len: u16,
}

impl AboutScreen {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            guide: Guide::Main,
            scroll: 0,
            content_len: 0,
        }
    }

    fn switch_guide(&mut self, forward: bool) {
        self.guide = self.guide.shift(forward);
        self.scroll = 0; // reset scroll when changing guide
    }

    /// Config-derived keybinding rows for the Main screen, shown in the General
    /// guide so displayed keys always match the active configuration.
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

    /// The tab bar of guide names, active one accented. Generously spaced with
    /// a dim dot separator so the guides read as distinct tabs.
    fn guide_tabs(&self) -> Line<'static> {
        let mut spans = vec![Span::raw("  ")];
        for (i, g) in Guide::ALL.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled("     ·     ", theme::hint()));
            }
            let style = if *g == self.guide {
                Style::default()
                    .fg(theme::accent())
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else {
                theme::hint()
            };
            spans.push(Span::styled(g.title().to_string(), style));
        }
        Line::from(spans)
    }

    fn content(&self, width: usize) -> Vec<Line<'static>> {
        let content = guide_content(self.guide);
        let mut lines: Vec<Line> = Vec::new();

        // Summary line, then a blank spacer.
        lines.extend(wrap_styled(content.summary, width, 0, theme::hint_italic()));
        lines.push(Line::raw(""));

        for block in content.blocks {
            lines.push(Line::from(Span::styled(
                block.heading.to_string(),
                theme::label(),
            )));
            lines.push(Line::raw(""));
            for row in block.keys {
                lines.push(key_row(row.key, row.desc));
            }
            if let Some(note) = block.note {
                if !block.keys.is_empty() {
                    lines.push(Line::raw(""));
                }
                lines.extend(wrap_styled(note, width, 2, theme::hint_italic()));
            }
            // Generous spacing between blocks.
            lines.push(Line::raw(""));
            lines.push(Line::raw(""));
        }

        // The General guide also lists the configured Main keybindings and app info.
        if self.guide == Guide::General {
            lines.push(Line::from(Span::styled(
                "Main keybindings".to_string(),
                theme::label(),
            )));
            lines.push(Line::raw(""));
            lines.extend(self.keybinding_lines());
            lines.push(Line::raw(""));
            lines.push(Line::raw(""));
            lines.push(Line::from(Span::styled(
                "About".to_string(),
                theme::label(),
            )));
            lines.push(Line::raw(""));
            for (label, value) in [
                ("Version", VERSION_MESSAGE.to_string()),
                (
                    "Author",
                    env!("CARGO_PKG_AUTHORS")
                        .split('<')
                        .next()
                        .unwrap_or_default()
                        .trim()
                        .to_string(),
                ),
                ("Repo", env!("CARGO_PKG_REPOSITORY").to_string()),
            ] {
                lines.push(Line::from(vec![
                    Span::styled(format!("  {label:<8}"), theme::key()),
                    Span::raw("  "),
                    Span::raw(value),
                ]));
            }
        }
        lines
    }
}

/// A single aligned key → description row: `  Ctrl+O        Open a file`.
fn key_row(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{key:<16}"), theme::key()),
        Span::raw("  "),
        Span::styled(desc.to_string(), theme::hint()),
    ])
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

    fn handle_key_event(&mut self, key: KeyEvent, _state: &State) -> Result<Option<Action>> {
        match key.code {
            KeyCode::Left => self.switch_guide(false),
            KeyCode::Right => self.switch_guide(true),
            _ => {}
        }
        Ok(None)
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
        let title = format!(" Guides — {} ", self.guide.title());
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::accent()))
            .padding(Padding::new(3, 3, 1, 1));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        if inner.width == 0 || inner.height == 0 {
            return Ok(());
        }

        // Prioritise (top to bottom): guide tabs, a blank separator, then the
        // scrollable content. All are optional as height shrinks.
        let [tabs_area, _gap, body_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .areas(inner);
        frame.render_widget(self.guide_tabs(), tabs_area);

        let width = inner.width as usize;
        let lines = self.content(width);
        self.content_len = lines.len() as u16;
        let paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));
        frame.render_widget(paragraph, body_area);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body_text(guide: Guide) -> String {
        let content = guide_content(guide);
        let mut parts: Vec<String> = vec![guide.title().to_string(), content.summary.to_string()];
        for b in content.blocks {
            parts.push(b.heading.to_string());
            for row in b.keys {
                parts.push(row.key.to_string());
                parts.push(row.desc.to_string());
            }
            if let Some(note) = b.note {
                parts.push(note.to_string());
            }
        }
        parts.join("\n")
    }

    #[test]
    fn every_screen_has_a_guide() {
        assert_eq!(Guide::ALL.len(), 4);
        for g in Guide::ALL {
            assert!(!body_text(g).is_empty(), "{} guide is empty", g.title());
        }
    }

    #[test]
    fn switching_guides_changes_content() {
        let mut about = AboutScreen::new();
        assert_eq!(about.guide, Guide::Main);
        about.switch_guide(true);
        assert_eq!(about.guide, Guide::Playground);
        about.switch_guide(false);
        assert_eq!(about.guide, Guide::Main);
        // Wrap around backwards.
        about.switch_guide(false);
        assert_eq!(about.guide, Guide::General);
    }

    #[test]
    fn switching_guide_resets_scroll() {
        let mut about = AboutScreen::new();
        about.scroll = 5;
        about.switch_guide(true);
        assert_eq!(about.scroll, 0);
    }

    #[test]
    fn guides_are_page_specific() {
        let main = body_text(Guide::Main);
        let play = body_text(Guide::Playground);
        let yaml = body_text(Guide::Yaml);
        // Main talks about environments, not YAML files.
        assert!(main.contains("environment"));
        assert!(!main.to_lowercase().contains(".yaml"));
        // Playground does not mention YAML files.
        assert!(!play.to_lowercase().contains(".yaml"));
        assert!(play.contains("Playground"));
        // YAML guide documents modified highlighting.
        assert!(yaml.contains('●'));
        assert!(yaml.to_lowercase().contains("modified"));
    }

    #[test]
    fn yaml_guide_documents_modified_highlighting() {
        assert!(body_text(Guide::Yaml).to_lowercase().contains("modified"));
    }

    #[test]
    fn renders_without_panic_on_tiny_areas() {
        use ratatui::{backend::TestBackend, Terminal};
        let state = State::for_test();
        for (w, h) in [(120, 30), (80, 24), (40, 12), (30, 10), (4, 3), (1, 1)] {
            let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
            let mut about = AboutScreen::new();
            term.draw(|f| {
                about.draw(f, f.area(), &state).unwrap();
            })
            .unwrap();
        }
    }
}
