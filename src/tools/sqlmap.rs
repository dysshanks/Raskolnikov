#[derive(Debug, Clone)]
pub struct SqlmapFinding {
    pub parameter: String,
    pub injection_type: String,
    pub technique: String,
    pub payload: String,
}

pub fn parse_sqlmap_output(output: &str) -> Vec<SqlmapFinding> {
    let mut findings = Vec::new();
    let mut current: Option<SqlmapFinding> = None;

    for line in output.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("Parameter:") {
            if let Some(f) = current.take() {
                findings.push(f);
            }
            // Extract parameter name
            let param = trimmed
                .trim_start_matches("Parameter:")
                .trim()
                .split(' ')
                .next()
                .unwrap_or("")
                .to_string();
            current = Some(SqlmapFinding {
                parameter: param,
                injection_type: String::new(),
                technique: String::new(),
                payload: String::new(),
            });
        } else if trimmed.starts_with("Type:") {
            if let Some(ref mut f) = current {
                f.injection_type = trimmed.trim_start_matches("Type:").trim().to_string();
            }
        } else if trimmed.starts_with("Technique:") {
            if let Some(ref mut f) = current {
                f.technique = trimmed.trim_start_matches("Technique:").trim().to_string();
            }
        } else if trimmed.starts_with("Payload:") {
            if let Some(ref mut f) = current {
                f.payload = trimmed.trim_start_matches("Payload:").trim().to_string();
            }
        }
    }

    if let Some(f) = current.take() {
        findings.push(f);
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sqlmap_injection_found() {
        let output = "---
Parameter: id (GET)
Type: boolean-based blind
Technique: OR boolean-based blind
Payload: id=1 OR 1=1--
---
Parameter: username (POST)
Type: time-based blind
Technique: SQLite time-based
Payload: username=' AND 1234=LIKE('ABCDEF','%')
---
";
        let findings = parse_sqlmap_output(output);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].parameter, "id");
        assert_eq!(findings[0].injection_type, "boolean-based blind");
        assert_eq!(findings[1].parameter, "username");
    }

    #[test]
    fn test_parse_sqlmap_no_findings() {
        let output = "[INFO] testing connection to the target URL
[INFO] testing all parameters
[INFO] target URL appears to be not injectable
";
        let findings = parse_sqlmap_output(output);
        assert!(findings.is_empty());
    }
}
