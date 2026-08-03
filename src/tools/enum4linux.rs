#[derive(Debug, Clone)]
pub struct Enum4linuxFinding {
    pub kind: String,
    pub value: String,
}

/// Parses enum4linux-ng text output. Relevant lines look like:
/// `[+] User: Administrator`
/// `[+] Share: C$`
/// `[+] Account: bob`
pub fn parse_enum4linux_output(output: &str) -> Vec<Enum4linuxFinding> {
    let mut findings = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        for prefix in ["User:", "Share:", "Account:", "Group:"] {
            if let Some(value) = line.strip_prefix("[+]").map(|s| s.trim()) {
                if let Some(v) = value.strip_prefix(prefix).map(|s| s.trim()) {
                    if !v.is_empty() {
                        findings.push(Enum4linuxFinding {
                            kind: prefix.trim_end_matches(':').to_lowercase(),
                            value: v.to_string(),
                        });
                    }
                }
            }
        }
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_enum4linux_output() {
        let output = "\
[+] User: Administrator
[+] User: bob
[+] Share: C$
[+] Share: IPC$
[+] Account: guest
";
        let findings = parse_enum4linux_output(output);
        assert_eq!(findings.len(), 5);
        assert_eq!(findings[0].kind, "user");
        assert_eq!(findings[0].value, "Administrator");
        assert_eq!(findings[2].kind, "share");
        assert_eq!(findings[2].value, "C$");
    }

    #[test]
    fn test_parse_enum4linux_empty() {
        assert!(parse_enum4linux_output("").is_empty());
        assert!(parse_enum4linux_output("[+] Got connection\n").is_empty());
    }
}
