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

/// A titled block of guidance within a guide page.
struct GuideBlock {
    heading: &'static str,
    body: &'static [&'static str],
}

/// The static content of a single guide (config-derived rows are appended at
/// render time, so raw key labels are not duplicated here).
struct GuideContent {
    blocks: &'static [GuideBlock],
}

fn guide_content(guide: Guide) -> GuideContent {
    match guide {
        Guide::Main => GuideContent {
            blocks: &[
                GuideBlock {
                    heading: "Main screen",
                    body: &["Encrypt or decrypt a value against a saved environment. \
                         The environment's algorithm, cipher mode and key come from \
                         your environments file."],
                },
                GuideBlock {
                    heading: "Environments",
                    body: &[
                        "The list on the left shows every saved environment. Select \
                         one with W/S (or the arrow keys).",
                        "A: add a new environment. Enter: edit the selected one. \
                         X: delete it (with confirmation). /: filter the list by name.",
                        "Keys are masked; press R to reveal or hide the selected key.",
                    ],
                },
                GuideBlock {
                    heading: "Encrypting",
                    body: &[
                        "Press Tab to focus the value field and type or paste a value, \
                         then Esc to return. Press E to encrypt or D to decrypt; the \
                         result appears in the Result pane and Ctrl+Y copies it.",
                    ],
                },
            ],
        },
        Guide::Playground => GuideContent {
            blocks: &[
                GuideBlock {
                    heading: "Playground screen",
                    body: &[
                        "A one-off encrypt/decrypt form with no saved environment — \
                         enter every parameter directly.",
                    ],
                },
                GuideBlock {
                    heading: "Fields",
                    body: &["Move between fields with Tab or the arrow keys. Change a \
                         choice (Operation, Algorithm, Mode, Random IV) with ←/→. \
                         Type into the Key and Value fields."],
                },
                GuideBlock {
                    heading: "Running",
                    body: &["Press Enter to generate the result. Ctrl+Y copies it. \
                         Errors and progress are shown in the Result pane. Esc \
                         returns to Main."],
                },
            ],
        },
        Guide::Yaml => GuideContent {
            blocks: &[
                GuideBlock {
                    heading: "YAML screen",
                    body: &["Encrypt or decrypt individual values inside a .yaml/.yml \
                         file in place — comments, ordering and formatting are \
                         preserved."],
                },
                GuideBlock {
                    heading: "Opening a file",
                    body: &[
                        "Press Ctrl+O to open the file dialog. Browse the filesystem \
                         with the arrow keys and Enter, or press Tab to type a path \
                         (~ is expanded). You can also start with --file <path>.",
                    ],
                },
                GuideBlock {
                    heading: "Navigating the tree",
                    body: &["Move with W/S (or arrows). Expand/collapse a mapping or \
                         sequence with →/← or Enter. Tab switches focus between the \
                         environments list and the tree."],
                },
                GuideBlock {
                    heading: "Editing values",
                    body: &[
                        "Select an environment, then on a scalar press E to encrypt \
                         or D to decrypt only that value. Enter edits it manually. \
                         Encrypted values are stored as \"![...]\" and masked; R \
                         reveals them. Ctrl+Z / Ctrl+Y undo and redo.",
                    ],
                },
                GuideBlock {
                    heading: "Modified highlighting",
                    body: &["A property whose value differs from the file as opened is \
                         marked with a subtle ● before its name (containers show it \
                         when a descendant changed). Editing a value back to its \
                         original clears the mark."],
                },
                GuideBlock {
                    heading: "Saving and restoring",
                    body: &["Ctrl+S saves atomically and becomes the new baseline. \
                         Ctrl+R restores the document to exactly how it was opened. \
                         Leaving or opening another file with unsaved changes prompts \
                         to Save, Discard or Cancel."],
                },
                GuideBlock {
                    heading: "Limitations",
                    body: &["Flow style ({}/[]), block/multiline scalars (|/>) and \
                         anchors/aliases/tags are shown but not editable in place — \
                         the editor refuses to reformat the file. Encrypting a \
                         non-string scalar makes it a quoted string."],
                },
            ],
        },
        Guide::General => GuideContent {
            blocks: &[
                GuideBlock {
                    heading: "Navigation",
                    body: &[
                        "Switch screens with h/l (previous/next). The footer always \
                         shows the actions valid right now for the current screen, \
                         focus and mode.",
                    ],
                },
                GuideBlock {
                    heading: "Modals and dialogs",
                    body: &[
                        "While a dialog is open only its actions apply. Confirmations \
                         use Y / Esc; the unsaved-changes prompt uses S (save), D \
                         (discard) and Esc (cancel). Esc closes a modal or leaves an \
                         input.",
                    ],
                },
                GuideBlock {
                    heading: "Quitting",
                    body: &["Press Q or Ctrl+C to quit."],
                },
                GuideBlock {
                    heading: "Configuration",
                    body: &[
                        "Keybindings and styles are read from a config file in your \
                         config directory, falling back to the bundled defaults. \
                         Environments live in your environments file (--envs / \
                         LAZYPROP_ENVS / ./envs.yaml / ~/.lazyprop/envs.yaml).",
                    ],
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

    /// The tab bar of guide names, active one accented.
    fn guide_tabs(&self) -> Line<'static> {
        let mut spans = Vec::new();
        for (i, g) in Guide::ALL.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw("   "));
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
        let mut lines: Vec<Line> = Vec::new();
        for block in guide_content(self.guide).blocks {
            lines.push(Line::from(Span::styled(
                block.heading.to_string(),
                theme::label(),
            )));
            for para in block.body {
                lines.extend(wrap_styled(para, width, 2, theme::hint()));
                lines.push(Line::raw(""));
            }
        }

        // The General guide lists the configured Main keybindings and app info.
        if self.guide == Guide::General {
            lines.push(Line::from(Span::styled(
                "Main keybindings".to_string(),
                theme::label(),
            )));
            lines.extend(self.keybinding_lines());
            lines.push(Line::raw(""));
            lines.push(Line::from(Span::styled(
                "About".to_string(),
                theme::label(),
            )));
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
            .padding(Padding::new(2, 2, 0, 0));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        if inner.width == 0 || inner.height == 0 {
            return Ok(());
        }

        // Prioritise (top to bottom): guide tabs, a separator, then scrollable
        // content. All are optional as height shrinks.
        let [tabs_area, body_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(inner);
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
        guide_content(guide)
            .blocks
            .iter()
            .flat_map(|b| std::iter::once(b.heading).chain(b.body.iter().copied()))
            .collect::<Vec<_>>()
            .join("\n")
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
