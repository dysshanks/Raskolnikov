/// Parses `dnsx -silent` output into a list of hosts:
///
/// ```text
/// example.com
/// sub.example.com [93.184.216.34]
/// ```
/// Log lines (`[INF]`, `[FTL]`, `[ERR]`, `[WRN]`) and A-record suffixes are
/// stripped. Best-effort — every non-empty host line is returned.
pub fn parse_dnsx_output(output: &str) -> Vec<String> {
    output
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .filter(|l| {
            !l.starts_with("[INF]")
                && !l.starts_with("[FTL]")
                && !l.starts_with("[ERR]")
                && !l.starts_with("[WRN]")
        })
        .filter_map(|l| {
            let host = l.split(' ').next().unwrap_or("").to_string();
            if host.is_empty() {
                None
            } else {
                Some(host)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dnsx_output() {
        let output =
            "example.com\nsub.example.com [93.184.216.34]\napi.example.com [93.184.216.35] [A]\n";
        let hosts = parse_dnsx_output(output);
        assert_eq!(
            hosts,
            vec!["example.com", "sub.example.com", "api.example.com"]
        );
    }

    #[test]
    fn test_parse_dnsx_skips_logs() {
        let output = "[INF] Running dnsx v1.2.0\n[FTL] no input provided\n";
        assert!(parse_dnsx_output(output).is_empty());
    }

    #[test]
    fn test_parse_dnsx_empty() {
        assert!(parse_dnsx_output("").is_empty());
    }
}
