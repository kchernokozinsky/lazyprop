//! A reusable, size-safe popup layout.
//!
//! Every modal dialog (confirmations, the unsaved-changes prompt, errors) is
//! drawn through [`render_popup`], which guarantees:
//!
//! * the popup never renders outside the terminal bounds;
//! * the action hints (Save / Discard / Cancel, …) are always on the last row,
//!   even in a tiny terminal — explanatory text is dropped or truncated first;
//! * no panic and no invalid `Rect` for any terminal size, down to 0×0.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::hints::{action_lines, min_action_width, KeyHint};

/// Draw a centered popup with a title, optional message lines, and a mandatory
/// row of action hints. `border` colours the frame. Message lines are truncated
/// to the popup width and dropped (top-most last) when height is scarce; the
/// action row is never dropped.
pub fn render_popup(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    message: &[String],
    actions: &[KeyHint],
    border: Color,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    // Desired inner width: enough for the actions, the title, and the widest
    // message line — clamped to what the terminal actually offers.
    let want_inner = [
        min_action_width(actions),
        title.chars().count() + 2,
        message.iter().map(|m| m.chars().count()).max().unwrap_or(0) + 2,
    ]
    .into_iter()
    .max()
    .unwrap_or(0);

    let max_inner_w = (area.width as usize).saturating_sub(2);
    let inner_w = want_inner.clamp(1, max_inner_w.max(1));
    let popup_w = (inner_w + 2).min(area.width as usize) as u16;

    // Actions may need more than one row on a narrow popup — they wrap rather
    // than being dropped or clipped.
    let act_rows = action_lines(actions, inner_w);
    let n_act = act_rows.len();

    // Height: the action rows, plus message rows (with a spacer) when there is
    // room.
    let want_inner_h = n_act
        + if message.is_empty() {
            0
        } else {
            message.len() + 1
        };
    let max_inner_h = (area.height as usize).saturating_sub(2).max(1);
    let inner_h = want_inner_h.min(max_inner_h).max(1);
    let popup_h = (inner_h + 2).min(area.height as usize) as u16;

    // Too short for a bordered box that fits the action rows: fall back to the
    // action rows alone (no border) at the bottom of the area, so they stay
    // visible on an extremely small terminal.
    if (popup_h as usize) < n_act + 2 {
        let rows = (area.height as usize).min(n_act);
        let top = area.y + area.height.saturating_sub(rows as u16);
        for (i, line) in act_rows.into_iter().take(rows).enumerate() {
            let row = Rect {
                x: area.x,
                y: top + i as u16,
                width: area.width,
                height: 1,
            };
            frame.render_widget(Clear, row);
            frame.render_widget(line, row);
        }
        return;
    }

    let popup = centered(popup_w, popup_h, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(format!(" {title} "))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    // Reserve the last `act_h` inner rows for the actions; message fills the
    // rest. Never let the message push the actions off the popup.
    let act_h = (n_act as u16).min(inner.height);
    let action_area = Rect {
        x: inner.x,
        y: inner.y + inner.height - act_h,
        width: inner.width,
        height: act_h,
    };
    let msg_h = inner.height - act_h;
    if msg_h > 0 && !message.is_empty() {
        let msg_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: msg_h,
        };
        let w = inner.width as usize;
        let lines: Vec<Line> = message
            .iter()
            .take(msg_h as usize)
            .map(|m| Line::from(truncate(m, w)))
            .collect();
        frame.render_widget(Paragraph::new(lines), msg_area);
    }

    frame.render_widget(Paragraph::new(act_rows), action_area);
}

/// Truncate `s` to `width` cells, adding a leading `…` when clipped so the end
/// (most informative part of a path) stays visible.
fn truncate(s: &str, width: usize) -> String {
    let len = s.chars().count();
    if len <= width {
        return s.to_string();
    }
    if width <= 1 {
        return "…".to_string();
    }
    let keep = width - 1;
    let tail: String = s.chars().skip(len - keep).collect();
    format!("…{tail}")
}

/// A centered rect of at most `w`×`h`, clamped inside `area` using saturating
/// arithmetic so it can never fall outside the terminal.
fn centered(w: u16, h: u16, area: Rect) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    Rect {
        x,
        y,
        width: w,
        height: h,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{hints::KeyHint, theme};
    use ratatui::{backend::TestBackend, Terminal};

    fn actions() -> Vec<KeyHint> {
        vec![
            KeyHint::critical("S", "Save"),
            KeyHint::critical("D", "Discard"),
            KeyHint::critical("Esc", "Cancel"),
        ]
    }

    fn render_at(w: u16, h: u16) -> String {
        let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
        term.draw(|f| {
            render_popup(
                f,
                f.area(),
                "Unsaved changes",
                &["A very long file path that should be truncated safely".to_string()],
                &actions(),
                theme::error(),
            );
        })
        .unwrap();
        let buf = term.backend().buffer().clone();
        buf.content().iter().map(|c| c.symbol()).collect::<String>()
    }

    #[test]
    fn actions_visible_at_many_sizes() {
        for (w, h) in [(120, 30), (80, 24), (60, 18), (40, 12), (30, 10), (20, 6)] {
            let text = render_at(w, h);
            assert!(text.contains("Save"), "Save missing at {w}x{h}");
            assert!(text.contains("Discard"), "Discard missing at {w}x{h}");
            assert!(text.contains("Cancel"), "Cancel missing at {w}x{h}");
        }
    }

    #[test]
    fn does_not_panic_on_tiny_terminals() {
        for (w, h) in [(1, 1), (2, 1), (1, 3), (4, 2), (6, 3), (10, 4)] {
            let _ = render_at(w, h); // must not panic
        }
    }

    #[test]
    fn truncate_keeps_tail() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello-world", 6), "…world");
        assert_eq!(truncate("x", 0), "…");
    }
}
