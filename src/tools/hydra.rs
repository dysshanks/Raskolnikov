use regex::Regex;

#[derive(Debug, Clone)]
pub struct HydraCredential {
    pub port: u16,
    pub service: String,
    pub host: String,
    pub user: String,
    pub password: String,
}

/// Parses hydra successful-login lines of the form:
/// `[22][ssh] host  login: admin   password: admin`
/// or
/// `[80][http-post-form] host/login.php  login: admin password: admin`
pub fn parse_hydra_output(output: &str) -> Vec<HydraCredential> {
    let re =
        Regex::new(r"\[(\d+)\]\[([^\]]+)\]\s+(\S+)\s+login:\s*(\S+)\s+password:\s*(\S+)").unwrap();
    output
        .lines()
        .filter_map(|line| {
            let caps = re.captures(line)?;
            Some(HydraCredential {
                port: caps[1].parse().ok()?,
                service: caps[2].to_string(),
                host: caps[3].to_string(),
                user: caps[4].to_string(),
                password: caps[5].to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hydra_output() {
        let output = "\
[22][ssh] 10.0.0.1 login: root   password: toor
[80][http-post-form] 10.0.0.1/login.php login: admin password: admin123
[3306][mysql] 10.0.0.1 login: dbuser password: letmein
";
        let creds = parse_hydra_output(output);
        assert_eq!(creds.len(), 3);
        assert_eq!(creds[0].port, 22);
        assert_eq!(creds[0].service, "ssh");
        assert_eq!(creds[0].host, "10.0.0.1");
        assert_eq!(creds[0].user, "root");
        assert_eq!(creds[0].password, "toor");
        assert_eq!(creds[2].service, "mysql");
        assert_eq!(creds[2].user, "dbuser");
    }

    #[test]
    fn test_parse_hydra_no_creds() {
        let output = "Hydra v9.5 (c) 2023 by van Hauser/THC\n[INFO] No password found yet\n";
        assert!(parse_hydra_output(output).is_empty());
    }

    #[test]
    fn test_parse_hydra_empty() {
        assert!(parse_hydra_output("").is_empty());
    }
}
