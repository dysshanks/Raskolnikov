#[derive(Debug, Clone)]
pub struct CrackedHash {
    pub hash: String,
    pub password: String,
}

/// Parses `john --show` output. Cracked entries look like:
/// `hash:password`                        (no login field)
/// `login:hash:password:uid:gid:...`      (with login field)
/// Lines are skipped until the first colon-separated entry appears.
pub fn parse_john_output(output: &str) -> Vec<CrackedHash> {
    let mut cracked = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(':') {
            continue;
        }
        let fields: Vec<&str> = line.split(':').collect();
        let (hash, password) = match fields.len() {
            0 | 1 => continue,
            2 => (fields[0].to_string(), fields[1].to_string()),
            _ => (fields[1].to_string(), fields[2].to_string()),
        };
        if hash.is_empty() || password.is_empty() {
            continue;
        }
        cracked.push(CrackedHash { hash, password });
    }
    cracked
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_john_output_with_login() {
        let output = "admin:5f4dcc3b5aa765d61d8327deb882cf99:password123:1000:1000:Admin:/home/admin:/bin/bash\n";
        let cracked = parse_john_output(output);
        assert_eq!(cracked.len(), 1);
        assert_eq!(cracked[0].hash, "5f4dcc3b5aa765d61d8327deb882cf99");
        assert_eq!(cracked[0].password, "password123");
    }

    #[test]
    fn test_parse_john_output_plain_hash_password() {
        let output = "e10adc3949ba59abbe56e057f20f883e:123456\n";
        let cracked = parse_john_output(output);
        assert_eq!(cracked.len(), 1);
        assert_eq!(cracked[0].hash, "e10adc3949ba59abbe56e057f20f883e");
        assert_eq!(cracked[0].password, "123456");
    }

    #[test]
    fn test_parse_john_empty() {
        assert!(parse_john_output("").is_empty());
        assert!(parse_john_output("No password hashes loaded\n").is_empty());
    }
}
