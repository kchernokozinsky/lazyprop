//! A single-line editable text field with a movable cursor and horizontal
//! scrolling, so long values stay usable in the narrow input boxes.

use ratatui::prelude::*;

use crate::theme;

#[derive(Debug, Clone, Default)]
pub struct TextField {
    chars: Vec<char>,
    cursor: usize,
}

impl TextField {
    pub fn from_text(s: &str) -> Self {
        let chars: Vec<char> = s.chars().collect();
        let cursor = chars.len();
        Self { chars, cursor }
    }

    pub fn value(&self) -> String {
        self.chars.iter().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    pub fn clear(&mut self) {
        self.chars.clear();
        self.cursor = 0;
    }

    pub fn insert(&mut self, c: char) {
        self.chars.insert(self.cursor, c);
        self.cursor += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.chars.remove(self.cursor);
        }
    }

    pub fn delete(&mut self) {
        if self.cursor < self.chars.len() {
            self.chars.remove(self.cursor);
        }
    }

    pub fn left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn right(&mut self) {
        if self.cursor < self.chars.len() {
            self.cursor += 1;
        }
    }

    pub fn home(&mut self) {
        self.cursor = 0;
    }

    pub fn end(&mut self) {
        self.cursor = self.chars.len();
    }

    /// The window of characters visible in `width` cells, following the cursor,
    /// plus the cursor's column within that window.
    fn windowed(&self, width: usize) -> (Vec<char>, usize) {
        let text_w = width.max(1).saturating_sub(1).max(1); // reserve a cell for the caret
        let start = if self.cursor < text_w {
            0
        } else {
            self.cursor - text_w + 1
        };
        let end = (start + text_w).min(self.chars.len());
        (self.chars[start..end].to_vec(), self.cursor - start)
    }

    /// Render the field's content as styled spans, windowed to `width` cells.
    /// When `active`, a block caret is drawn at the cursor; when empty and not
    /// active, `placeholder` is shown.
    pub fn spans(&self, width: usize, active: bool, placeholder: &str) -> Vec<Span<'static>> {
        if self.chars.is_empty() && !active {
            return vec![Span::styled(placeholder.to_string(), theme::hint_italic())];
        }

        let (vis, col) = self.windowed(width);
        if !active {
            return vec![Span::raw(vis.iter().collect::<String>())];
        }

        let cursor_style = Style::default()
            .fg(theme::accent())
            .add_modifier(Modifier::REVERSED);
        let before: String = vis[..col.min(vis.len())].iter().collect();
        if col < vis.len() {
            let at = vis[col].to_string();
            let after: String = vis[col + 1..].iter().collect();
            vec![
                Span::raw(before),
                Span::styled(at, cursor_style),
                Span::raw(after),
            ]
        } else {
            vec![Span::raw(before), Span::styled(" ", cursor_style)]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_cursor_movement() {
        let mut f = TextField::from_text("abc");
        assert_eq!(f.value(), "abc");
        f.home();
        f.right();
        f.insert('X'); // aXbc
        assert_eq!(f.value(), "aXbc");
        f.backspace(); // abc
        assert_eq!(f.value(), "abc");
    }

    #[test]
    fn delete_at_cursor() {
        let mut f = TextField::from_text("abc");
        f.home();
        f.delete(); // remove 'a'
        assert_eq!(f.value(), "bc");
        f.end();
        f.delete(); // nothing to delete at end
        assert_eq!(f.value(), "bc");
    }

    #[test]
    fn windowing_follows_cursor() {
        let mut f = TextField::from_text("abcdefghij");
        f.end();
        // With width 5 the tail should be visible around the cursor.
        let (vis, col) = f.windowed(5);
        assert!(vis.iter().collect::<String>().ends_with("ij"));
        assert_eq!(col, vis.len());
    }
}
