use std::sync::{Mutex, OnceLock};

/// The clipboard handle is kept alive for the whole process (statics are
/// never dropped): dropping an `arboard::Clipboard` right after `set_text`
/// makes arboard print a warning to stderr — the terminal in a TUI — in
/// debug builds, and releases the X11 selection so the copied text can be
/// lost when no clipboard manager is running.
fn clipboard() -> Result<&'static Mutex<arboard::Clipboard>, String> {
    static CLIPBOARD: OnceLock<Result<Mutex<arboard::Clipboard>, String>> = OnceLock::new();
    CLIPBOARD
        .get_or_init(|| {
            arboard::Clipboard::new()
                .map(Mutex::new)
                .map_err(|e| e.to_string())
        })
        .as_ref()
        .map_err(|e| e.clone())
}

pub fn copy_text(text: &str) -> Result<(), String> {
    clipboard()?
        .lock()
        .map_err(|e| e.to_string())?
        .set_text(text.to_string())
        .map_err(|e| e.to_string())
}

pub fn read_text() -> Result<String, String> {
    clipboard()?
        .lock()
        .map_err(|e| e.to_string())?
        .get_text()
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_handle_is_reused_across_calls() {
        let first = clipboard();
        let second = clipboard();
        assert_eq!(
            first.is_ok(),
            second.is_ok(),
            "clipboard availability must be stable across calls"
        );
        if let (Ok(a), Ok(b)) = (first, second) {
            assert!(std::ptr::eq(a, b), "the same static handle must be reused");
        }
    }

    #[test]
    fn test_copy_and_read_text_fail_gracefully_without_display() {
        // Headless CI has no clipboard backend: calls must return Err, not panic.
        let _ = copy_text("hello");
        let _ = read_text();
    }
}
