use std::path::Path;

#[derive(Debug, Clone)]
pub struct WebPath {
    pub path: String,
    pub status_code: u16,
}

pub const BUILTIN_WORDLIST: &[&str] = &[
    "admin",
    "login",
    "wp-admin",
    "wp-content",
    "wp-includes",
    "uploads",
    "downloads",
    "images",
    "css",
    "js",
    "assets",
    "backup",
    "config",
    "db",
    "sql",
    "dump",
    "test",
    "tmp",
    "private",
    "secure",
    "api",
    "v1",
    "v2",
    "graphql",
    "rest",
    ".git",
    ".env",
    ".htaccess",
    "robots.txt",
    "sitemap.xml",
    "index.php",
    "index.html",
    "default.aspx",
    "server-status",
    "phpmyadmin",
    "pma",
    "adminer",
    "cgi-bin",
    "shell",
    "vendor",
    "node_modules",
    "src",
    "dist",
    "build",
];

pub fn find_wordlist(paths: &[String]) -> Option<String> {
    for path in paths {
        if Path::new(path).exists() {
            return Some(path.clone());
        }
    }
    None
}

pub fn parse_gobuster_output(output: &str) -> Vec<WebPath> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || !line.starts_with('/') {
                return None;
            }
            if let Some(end) = line.find("(Status:") {
                let path = line[..end].trim().to_string();
                let rest = &line[end..];
                let code_str = rest
                    .trim_start_matches("(Status: ")
                    .trim_end_matches(')')
                    .trim();
                if let Ok(code) = code_str.parse::<u16>() {
                    if matches!(code, 200 | 301 | 302 | 403) {
                        return Some(WebPath {
                            path,
                            status_code: code,
                        });
                    }
                }
            }
            None
        })
        .collect()
}

pub fn parse_ffuf_json(output: &str) -> Vec<WebPath> {
    // ffuf JSON output: [{"url": "...", "status": 200, ...}]
    if let Ok(results) = serde_json::from_str::<Vec<serde_json::Value>>(output) {
        return results
            .into_iter()
            .filter_map(|r| {
                let url = r.get("url")?.as_str()?;
                let status = r.get("status")?.as_u64()?;
                let path = url.split('/').skip(3).collect::<Vec<_>>().join("/");
                Some(WebPath {
                    path: format!("/{}", path),
                    status_code: status as u16,
                })
            })
            .collect();
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gobuster_output() {
        let output = "/admin (Status: 302)
/uploads (Status: 200)
/test (Status: 404)
";
        let paths = parse_gobuster_output(output);
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0].path, "/admin");
        assert_eq!(paths[0].status_code, 302);
        assert_eq!(paths[1].path, "/uploads");
        assert_eq!(paths[1].status_code, 200);
    }

    #[test]
    fn test_parse_ffuf_json() {
        let output = r#"[
            {"url": "http://test.local/admin", "status": 302},
            {"url": "http://test.local/uploads", "status": 200}
        ]"#;
        let paths = parse_ffuf_json(output);
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_find_wordlist_not_found() {
        let result = find_wordlist(&["/nonexistent/path.txt".to_string()]);
        assert!(result.is_none());
    }

    #[test]
    fn test_builtin_wordlist_not_empty() {
        assert!(!BUILTIN_WORDLIST.is_empty());
        assert!(BUILTIN_WORDLIST.contains(&"admin"));
    }
}
