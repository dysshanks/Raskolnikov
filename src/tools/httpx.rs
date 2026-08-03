use crate::tools::gobuster::WebPath;
use regex::Regex;

/// Parses httpx default text output. Lines look like:
/// `http://10.0.0.1 [200] [nginx/1.18.0]`
/// `http://10.0.0.1/admin [302] [nginx/1.18.0] [redirect_to:http://10.0.0.1/login]`
pub fn parse_httpx_output(output: &str) -> Vec<WebPath> {
    let re = Regex::new(r"^(\S+)\s+\[(\d+)\](.*)$").unwrap();
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let caps = re.captures(line)?;
            let url = caps[1].to_string();
            let status: u16 = caps[2].parse().ok()?;
            let path = url
                .split("://")
                .nth(1)
                .map(|rest| {
                    rest.split_once('/')
                        .map(|(_, p)| format!("/{}", p.trim_end_matches('/')))
                        .unwrap_or_else(|| "/".to_string())
                })
                .unwrap_or_else(|| url.clone());
            Some(WebPath {
                path,
                status_code: status,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_httpx_output() {
        let output = "\
http://10.0.0.1 [200] [nginx/1.18.0]
http://10.0.0.1/admin [302] [nginx/1.18.0]
https://10.0.0.2 [200] [apache]
";
        let paths = parse_httpx_output(output);
        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0].path, "/");
        assert_eq!(paths[0].status_code, 200);
        assert_eq!(paths[1].path, "/admin");
        assert_eq!(paths[1].status_code, 302);
        assert_eq!(paths[2].path, "/");
    }

    #[test]
    fn test_parse_httpx_empty() {
        assert!(parse_httpx_output("").is_empty());
        assert!(parse_httpx_output("no matches\n").is_empty());
    }
}
