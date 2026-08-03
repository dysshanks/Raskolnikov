use regex::Regex;

#[derive(Debug, Clone)]
pub struct TechFinding {
    pub url: String,
    pub tech: String,
    pub version: Option<String>,
}

/// Parses whatweb output. Lines look like:
/// `http://10.0.0.1 [200 OK] Apache[2.4.52], PHP[8.1.2], Cookies[PHPSESSID]`
fn split_tech(token: &str) -> (String, Option<String>) {
    if let Some(open) = token.find('[') {
        let name = token[..open].trim().to_string();
        let version = token[open..]
            .trim_start_matches('[')
            .trim_end_matches(']')
            .trim()
            .to_string();
        (
            name,
            if version.is_empty() {
                None
            } else {
                Some(version)
            },
        )
    } else {
        (token.trim().to_string(), None)
    }
}

pub fn parse_whatweb_output(output: &str) -> Vec<TechFinding> {
    let re = Regex::new(r"^(\S+)\s+\[[^\]]+\](.*)$").unwrap();
    let mut findings = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some(caps) = re.captures(line) else {
            continue;
        };
        let url = caps[1].trim_end_matches('/').to_string();
        let rest = caps[2].trim();
        if rest.is_empty() {
            continue;
        }
        for token in rest.split(", ") {
            let token = token.trim();
            if token.is_empty() {
                continue;
            }
            let (tech, version) = split_tech(token);
            findings.push(TechFinding {
                url: url.clone(),
                tech,
                version,
            });
        }
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_whatweb_output() {
        let output = "\
http://10.0.0.1 [200 OK] Apache[2.4.52], PHP[8.1.2], Cookies[PHPSESSID]
http://10.0.0.1 [200 OK] HTTPServer[nginx/1.18.0]
";
        let findings = parse_whatweb_output(output);
        assert_eq!(findings.len(), 4);
        assert_eq!(findings[0].tech, "Apache");
        assert_eq!(findings[0].version.as_deref(), Some("2.4.52"));
        assert_eq!(findings[1].tech, "PHP");
        assert_eq!(findings[3].tech, "HTTPServer");
    }

    #[test]
    fn test_parse_whatweb_no_tech() {
        let output = "http://10.0.0.1 [200 OK]\n";
        assert!(parse_whatweb_output(output).is_empty());
    }

    #[test]
    fn test_parse_whatweb_empty() {
        assert!(parse_whatweb_output("").is_empty());
    }
}
