use regex::Regex;

#[derive(Debug, Clone)]
pub struct NiktoFinding {
    pub description: String,
    pub cve: Option<String>,
}

pub fn parse_nikto_output(output: &str) -> Vec<NiktoFinding> {
    let cve_pattern = Regex::new(r"CVE-\d{4}-\d{4,}").unwrap();
    let mut findings = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('+') {
            continue;
        }

        let description = trimmed.trim_start_matches('+').trim().to_string();
        let cve = cve_pattern
            .find(&description)
            .map(|m| m.as_str().to_string());

        findings.push(NiktoFinding { description, cve });
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nikto_output() {
        let output = "- This is not a finding
+ Apache/2.4.52 appears to be outdated
+ /admin/: Admin login page/section found.
+ OSVDB-1234: /uploads/: Directory indexing allows listing of files.
+ CVE-2023-1234: Mod security bypass via null byte injection
";
        let findings = parse_nikto_output(output);
        assert_eq!(findings.len(), 4);
        assert_eq!(
            findings[0].description,
            "Apache/2.4.52 appears to be outdated"
        );
        assert_eq!(findings[3].cve, Some("CVE-2023-1234".to_string()));
    }

    #[test]
    fn test_parse_nikto_output_no_findings() {
        let output = "No findings in this output\n- Just a line with dash\n";
        let findings = parse_nikto_output(output);
        assert!(findings.is_empty());
    }
}
