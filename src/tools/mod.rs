pub mod executor;
pub mod gobuster;
pub mod nikto;
pub mod nmap;
pub mod sqlmap;

use std::process::Command;

pub struct ToolInfo {
    pub name: &'static str,
    pub available: bool,
    pub version: Option<String>,
}

pub fn check_tool(name: &'static str, version_flag: &str) -> ToolInfo {
    let output = Command::new(name).arg(version_flag).output();

    match output {
        Ok(out) if out.status.success() => {
            let version = String::from_utf8_lossy(&out.stdout)
                .lines()
                .next()
                .map(|l| l.to_string());
            ToolInfo {
                name,
                available: true,
                version,
            }
        }
        _ => ToolInfo {
            name,
            available: false,
            version: None,
        },
    }
}

pub fn check_all_tools() -> Vec<ToolInfo> {
    vec![
        check_tool("nmap", "--version"),
        check_tool("gobuster", "--version"),
        check_tool("nikto", "-Version"),
        check_tool("sqlmap", "--version"),
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
}
