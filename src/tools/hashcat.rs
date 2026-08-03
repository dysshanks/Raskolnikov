#[derive(Debug, Clone)]
pub struct HashcatCracked {
    pub hash: String,
    pub password: String,
}

/// Parses `hashcat --show` output. Entries look like:
/// `hash:password`              (no username)
/// `user:hash:password:...`     (with --username)
pub fn parse_hashcat_output(output: &str) -> Vec<HashcatCracked> {
    let mut cracked = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
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
        cracked.push(HashcatCracked { hash, password });
    }
    cracked
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hashcat_output() {
        let output = "5f4dcc3b5aa765d61d8327deb882cf99:password123\ne10adc3949ba59abbe56e057f20f883e:123456\n";
        let cracked = parse_hashcat_output(output);
        assert_eq!(cracked.len(), 2);
        assert_eq!(cracked[0].password, "password123");
        assert_eq!(cracked[1].hash, "e10adc3949ba59abbe56e057f20f883e");
    }

    #[test]
    fn test_parse_hashcat_output_with_username() {
        let output = "admin:5f4dcc3b5aa765d61d8327deb882cf99:password123:...\n";
        let cracked = parse_hashcat_output(output);
        assert_eq!(cracked.len(), 1);
        assert_eq!(cracked[0].password, "password123");
    }

    #[test]
    fn test_parse_hashcat_empty() {
        assert!(parse_hashcat_output("").is_empty());
        assert!(parse_hashcat_output("No hashes loaded\n").is_empty());
    }
}
