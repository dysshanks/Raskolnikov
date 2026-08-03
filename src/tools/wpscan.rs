#[derive(Debug, Clone)]
pub struct WpFinding {
    pub kind: String,
    pub detail: String,
}

/// Parses `wpscan --url <target>` output into findings. Extracts the
/// WordPress version, identified usernames, active themes, and `[!]`-flagged
/// vulnerabilities. Best-effort — unrecognised lines are ignored.
pub fn parse_wpscan_output(output: &str) -> Vec<WpFinding> {
    let mut findings = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix("[+] WordPress version ") {
            let ver = rest.split_whitespace().next().unwrap_or("");
            if !ver.is_empty() {
                findings.push(WpFinding {
                    kind: "version".to_string(),
                    detail: format!("WordPress version {}", ver),
                });
            }
        } else if let Some(rest) = line.strip_prefix("[i] User(s) Identified: ") {
            for user in rest.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                findings.push(WpFinding {
                    kind: "user".to_string(),
                    detail: format!("WordPress user: {}", user),
                });
            }
        } else if let Some(rest) = line.strip_prefix("[+] WordPress theme in use: ") {
            let theme = rest.split_whitespace().next().unwrap_or("");
            if !theme.is_empty() {
                findings.push(WpFinding {
                    kind: "theme".to_string(),
                    detail: format!("WordPress theme: {}", theme),
                });
            }
        } else if let Some(rest) = line.strip_prefix("[!] Title: ") {
            let title = rest.trim();
            if !title.is_empty() {
                findings.push(WpFinding {
                    kind: "vuln".to_string(),
                    detail: title.to_string(),
                });
            }
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_wpscan_version_and_users() {
        let output = "[+] WordPress version 6.0.2 identified (Insecure, released on 2022-07-12).\n[i] User(s) Identified: admin, editor\n";
        let findings = parse_wpscan_output(output);
        assert_eq!(findings.len(), 3);
        assert_eq!(findings[0].kind, "version");
        assert_eq!(findings[0].detail, "WordPress version 6.0.2");
        assert_eq!(findings[1].detail, "WordPress user: admin");
        assert_eq!(findings[2].detail, "WordPress user: editor");
    }

    #[test]
    fn test_parse_wpscan_vuln_and_theme() {
        let output = "[+] WordPress theme in use: twentytwentytwo\n[!] Title: Plugin X <= 2.3 - SQL Injection\n";
        let findings = parse_wpscan_output(output);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].kind, "theme");
        assert_eq!(findings[1].kind, "vuln");
        assert!(findings[1].detail.contains("SQL Injection"));
    }

    #[test]
    fn test_parse_wpscan_empty() {
        assert!(parse_wpscan_output("").is_empty());
    }
}
