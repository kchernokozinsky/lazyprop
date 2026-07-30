//! Thin wrapper over the system clipboard.

/// Copy `text` to the system clipboard, returning a human-readable error on
/// failure (e.g. no clipboard available in the current environment).
pub fn copy(text: &str) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| format!("clipboard: {e}"))?;
    clipboard
        .set_text(text.to_string())
        .map_err(|e| format!("clipboard: {e}"))
}
