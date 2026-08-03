/// Parses `subfinder -silent -d <domain>` output into a list of subdomains,
/// one per line. Log lines starting with `[` (`[INF]`, `[ERR]`, ...) are
/// skipped. Best-effort — every remaining non-empty line is a subdomain.
pub fn parse_subfinder_output(output: &str) -> Vec<String> {
    output
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .filter(|l| !l.starts_with('['))
        .map(|l| l.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_subfinder_output() {
        let output = "sub.example.com\nadmin.example.com\n";
        let subs = parse_subfinder_output(output);
        assert_eq!(subs, vec!["sub.example.com", "admin.example.com"]);
    }

    #[test]
    fn test_parse_subfinder_skips_logs() {
        let output =
            "[INF] Enumerating subdomains for example.com\n[ERR] api failed\nfoo.example.com\n";
        let subs = parse_subfinder_output(output);
        assert_eq!(subs, vec!["foo.example.com"]);
    }

    #[test]
    fn test_parse_subfinder_empty() {
        assert!(parse_subfinder_output("").is_empty());
    }
}
