use crate::tools::gobuster::WebPath;

/// Parses feroxbuster output. feroxbuster emits JSON Lines when run with
/// `--json` (e.g. `feroxbuster -u <url> -o - -j`). Each line looks like:
/// `{"url":"http://host/admin","status":302,"content_length":123,"ext":[]}`
pub fn parse_feroxbuster_output(output: &str) -> Vec<WebPath> {
    let mut paths = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(url) = value.get("url").and_then(|u| u.as_str()) {
                let status = value.get("status").and_then(|s| s.as_u64()).unwrap_or(0) as u16;
                let path = url.split('/').skip(3).collect::<Vec<_>>().join("/");
                paths.push(WebPath {
                    path: format!("/{}", path),
                    status_code: status,
                });
            }
        }
    }
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_feroxbuster_output() {
        let output = "\
{\"url\":\"http://host/admin\",\"status\":302,\"content_length\":145}
{\"url\":\"http://host/uploads\",\"status\":200,\"content_length\":12}
{\"url\":\"http://host/nope\",\"status\":404,\"content_length\":0}
";
        let paths = parse_feroxbuster_output(output);
        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0].path, "/admin");
        assert_eq!(paths[0].status_code, 302);
        assert_eq!(paths[1].path, "/uploads");
        assert_eq!(paths[1].status_code, 200);
    }

    #[test]
    fn test_parse_feroxbuster_empty() {
        assert!(parse_feroxbuster_output("").is_empty());
    }

    #[test]
    fn test_parse_feroxbuster_invalid_lines_ignored() {
        let output = "not json\n{\"url\":\"http://host/x\"}\n";
        let paths = parse_feroxbuster_output(output);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].path, "/x");
    }
}
