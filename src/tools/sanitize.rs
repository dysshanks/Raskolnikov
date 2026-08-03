use regex::Regex;

static ANSI_RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
static CONTROL_RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();

fn ansi_re() -> &'static Regex {
    ANSI_RE.get_or_init(|| {
        // CSI sequences like \x1b[2J, \x1b[38;5;196m, and OSC sequences like
        // \x1b]0;title\x07 or \x1b]... \x1b\\.
        Regex::new(r"\x1b\[[0-9;?]*[a-zA-Z]|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)|\x1b[@-_]").unwrap()
    })
}

fn control_re() -> &'static Regex {
    CONTROL_RE.get_or_init(|| {
        // Strip C0 control characters but keep newline and tab.
        Regex::new(r"[\x00-\x08\x0b\x0c\x0e-\x1f\x7f]").unwrap()
    })
}

/// Strips ANSI escape sequences and stray control characters from a line of
/// tool output before it is rendered in the TUI. Raw bytes on disk are
/// untouched — this only affects what reaches the terminal.
pub fn sanitize_output(line: &str) -> String {
    let stripped = ansi_re().replace_all(line, "").into_owned();
    control_re().replace_all(&stripped, "").into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strips_csi_sequences() {
        let input = "\x1b[2J\x1b[38;5;196mred\x1b[0m";
        assert_eq!(sanitize_output(input), "red");
    }

    #[test]
    fn test_strips_osc_sequences() {
        let input = "title\x1b]0;PWNED\x07done";
        assert_eq!(sanitize_output(input), "titledone");
    }

    #[test]
    fn test_strips_single_escape() {
        let input = "abc\x1bdef";
        assert_eq!(sanitize_output(input), "abcdef");
    }

    #[test]
    fn test_strips_control_chars_keeps_newline_tab() {
        let input = "a\x0fb\nc\td";
        assert_eq!(sanitize_output(input), "ab\nc\td");
    }

    #[test]
    fn test_plain_text_unchanged() {
        let input = "PORT     STATE SERVICE\n22/tcp   open  ssh\n";
        assert_eq!(sanitize_output(input), input);
    }
}
