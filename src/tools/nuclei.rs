use regex::Regex;

#[derive(Debug, Clone)]
pub struct NucleiFinding {
    pub protocol: String,
    pub severity: String,
    pub url: String,
    pub matcher: Option<String>,
    pub template: Option<String>,
    pub cve: Option<String>,
}

/// Parses nuclei default text output. Lines look like:
/// `[http] [medium] https://host/wp-login.php [cve] [cve-2023-1234] [http-wordpress-login]`
pub fn parse_nuclei_output(output: &str) -> Vec<NucleiFinding> {
    let line_re = Regex::new(r"^\[(\w+)\]\s+\[(\w+)\]\s+(\S+)(.*)$").unwrap();
    let cve_re = Regex::new(r"(?i)CVE-\d{4}-\d{4,}").unwrap();
    let mut findings = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        let Some(caps) = line_re.captures(line) else {
            continue;
        };
        let protocol = caps[1].to_string();
        let severity = caps[2].to_string();
        let url = caps[3].to_string();
        let rest = caps[4].to_string();

        let cve = cve_re.find(line).map(|m| m.as_str().to_uppercase());
        let template = rest
            .split('[')
            .nth(1)
            .map(|t| t.trim_end_matches(']').trim().to_string())
            .filter(|t| !t.is_empty());

        findings.push(NucleiFinding {
            protocol,
            severity,
            url,
            matcher: None,
            template,
            cve,
        });
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nuclei_output() {
        let output = "\
[http] [info] http://host/robots.txt [robots] [93-0:robots]
[http] [medium] http://host/wp-login.php [cve] [cve-2023-1234] [http-wordpress-login]
[ssl] [low] https://host:8443 [ssl] [ssl-detect]
";
        let findings = parse_nuclei_output(output);
        assert_eq!(findings.len(), 3);
        assert_eq!(findings[0].severity, "info");
        assert_eq!(findings[0].url, "http://host/robots.txt");
        assert_eq!(findings[1].severity, "medium");
        assert_eq!(findings[1].cve.as_deref(), Some("CVE-2023-1234"));
        assert_eq!(findings[2].protocol, "ssl");
    }

    #[test]
    fn test_parse_nuclei_empty() {
        assert!(parse_nuclei_output("").is_empty());
        assert!(parse_nuclei_output("Nothing found\n").is_empty());
    }
}
