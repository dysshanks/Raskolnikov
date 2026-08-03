use regex::Regex;

#[derive(Debug, Clone)]
pub struct NetexecCredential {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub pwned: bool,
}

/// Parses netexec (nxc) output. Successful authentication lines look like:
/// `[+] 10.0.0.1:445 - admin:Password123 (Pwn3d!)`
/// or, without admin rights:
/// `[+] 10.0.0.1:445 - admin:Password123`
/// Failed attempts start with `[-]` and are ignored.
pub fn parse_netexec_output(output: &str) -> Vec<NetexecCredential> {
    let re = Regex::new(r"^\[\+\]\s+(\S+):(\d+)\s+-\s+(\S+):(\S+)(?:\s+\((Pwn3d!)\))?").unwrap();
    output
        .lines()
        .filter_map(|line| {
            let caps = re.captures(line)?;
            Some(NetexecCredential {
                host: caps[1].to_string(),
                port: caps[2].parse().ok()?,
                user: caps[3].to_string(),
                password: caps[4].to_string(),
                pwned: caps.get(5).is_some(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_netexec_output() {
        let output = "\
[+] 10.0.0.1:445 - admin:Password123 (Pwn3d!)
[+] 10.0.0.1:445 - user:userpass
[-] 10.0.0.1:445 - guest:wrong
";
        let creds = parse_netexec_output(output);
        assert_eq!(creds.len(), 2);
        assert_eq!(creds[0].host, "10.0.0.1");
        assert_eq!(creds[0].port, 445);
        assert_eq!(creds[0].user, "admin");
        assert_eq!(creds[0].password, "Password123");
        assert!(creds[0].pwned);
        assert!(!creds[1].pwned);
    }

    #[test]
    fn test_parse_netexec_no_creds() {
        assert!(parse_netexec_output("").is_empty());
        assert!(parse_netexec_output("[-] 10.0.0.1:445 - guest:no").is_empty());
    }
}
