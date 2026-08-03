pub mod enum4linux;
pub mod executor;
pub mod feroxbuster;
pub mod gobuster;
pub mod hashcat;
pub mod httpx;
pub mod hydra;
pub mod john;
pub mod netexec;
pub mod nikto;
pub mod nmap;
pub mod nuclei;
pub mod parse;
pub mod sanitize;
pub mod sqlmap;
pub mod whatweb;

use std::process::Command;

pub struct ToolInfo {
    pub name: &'static str,
    pub available: bool,
    pub version: Option<String>,
}

pub fn check_tool(name: &'static str, version_flag: &str) -> ToolInfo {
    let mut cmd = Command::new(name);
    if !version_flag.is_empty() {
        cmd.arg(version_flag);
    }
    check_tool_cmd(name, cmd)
}

/// Checks each (binary, version-flag) candidate in order and returns the first
/// one that is installed. The advertised name is the binary that was actually
/// found (e.g. `nxc` rather than `netexec`, or `httpx-toolkit` on Arch).
/// If none are found, reports the given fallback name as unavailable so the
/// boot screen shows a single clear entry.
pub fn check_tool_aliases(
    fallback_name: &'static str,
    candidates: &[(&'static str, &'static str)],
) -> ToolInfo {
    for &(bin, flag) in candidates {
        let info = check_tool(bin, flag);
        if info.available {
            return info;
        }
    }
    ToolInfo {
        name: fallback_name,
        available: false,
        version: None,
    }
}

fn check_tool_cmd(name: &'static str, mut cmd: Command) -> ToolInfo {
    match cmd.output() {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let banner = banner_line(&stdout).or_else(|| banner_line(&stderr));
            let available = out.status.success() || banner.is_some();
            ToolInfo {
                name,
                available,
                version: banner,
            }
        }
        _ => ToolInfo {
            name,
            available: false,
            version: None,
        },
    }
}

/// Prefers the first non-empty line that contains a digit (version-like), so
/// ASCII-art banners (e.g. httpx-toolkit) are skipped. Falls back to the first
/// non-empty line.
fn banner_line(s: &str) -> Option<String> {
    let lines: Vec<String> = s
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    if lines.is_empty() {
        return None;
    }
    Some(
        lines
            .iter()
            .find(|l| l.chars().any(|c| c.is_ascii_digit()))
            .unwrap_or(&lines[0])
            .clone(),
    )
}

pub fn check_all_tools() -> Vec<ToolInfo> {
    vec![
        check_tool("nmap", "--version"),
        check_tool("gobuster", "--version"),
        check_tool("nikto", "-Version"),
        check_tool("sqlmap", "--version"),
        check_tool("hydra", "-h"),
        check_tool("whatweb", "--version"),
        check_tool("feroxbuster", "--version"),
        check_tool_aliases("netexec", &[("nxc", "--version"), ("netexec", "--version")]),
        check_tool("john", ""),
        check_tool("nuclei", "-version"),
        check_tool("enum4linux-ng", "-h"),
        check_tool_aliases(
            "httpx",
            &[("httpx", "-version"), ("httpx-toolkit", "-version")],
        ),
        check_tool("hashcat", "--version"),
    ]
}

pub fn available_tool_names() -> Vec<String> {
    check_all_tools()
        .into_iter()
        .filter(|t| t.available)
        .map(|t| t.name.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_tool_not_found() {
        let info = check_tool("nonexistent-tool-12345", "--version");
        assert!(!info.available);
        assert!(info.version.is_none());
    }

    #[test]
    fn test_check_tool_nonzero_exit_with_output() {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", "echo hello; exit 1"]);
        let info = check_tool_cmd("sh", cmd);
        assert!(info.available);
        assert_eq!(info.version.as_deref(), Some("hello"));
    }

    #[test]
    fn test_check_tool_nonzero_exit_empty_output() {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", "exit 1"]);
        let info = check_tool_cmd("sh", cmd);
        assert!(!info.available);
        assert!(info.version.is_none());
    }

    #[test]
    fn test_check_tool_empty_flag_skips_arg() {
        let info = check_tool("sh", "");
        assert!(info.available);
    }

    #[test]
    fn test_check_tool_aliases_returns_first_available() {
        let info = check_tool_aliases("netexec", &[("sh", ""), ("sh", "")]);
        assert!(info.available);
        assert_eq!(info.name, "sh");
    }

    #[test]
    fn test_check_tool_aliases_falls_back_to_name() {
        let info = check_tool_aliases(
            "netexec",
            &[
                ("nonexistent-tool-12345", "--version"),
                ("nonexistent-23456", "--version"),
            ],
        );
        assert!(!info.available);
        assert_eq!(info.name, "netexec");
        assert!(info.version.is_none());
    }

    #[test]
    fn test_banner_line_prefers_version_line() {
        let s = "\n  __    __  __\n  / /_\n[INF] Current Version: v1.10.0\n";
        assert_eq!(
            banner_line(s).as_deref(),
            Some("[INF] Current Version: v1.10.0")
        );
    }

    #[test]
    fn test_banner_line_no_digits_uses_first() {
        assert_eq!(
            banner_line("hello world\nfoo").as_deref(),
            Some("hello world")
        );
    }

    #[test]
    fn test_banner_line_empty() {
        assert_eq!(banner_line("\n  \n"), None);
    }
}
