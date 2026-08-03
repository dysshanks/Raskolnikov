#[derive(Debug, Clone)]
pub struct ImpacketCredential {
    pub service: String,
    pub user: String,
    pub secret: String,
}

/// Parses impacket script output into credentials:
/// - `GetNPUsers.py` / `GetUserSPNs.py`: Kerberos pre-auth / service tickets
///   in hashcat format, e.g. `$krb5asrep$23$user@REALM:hash` or
///   `$krb5tgs$23$*user$realm$spn$checksum`.
/// - `secretsdump.py`: `user:rid:lmhash:nthash:::` lines.
pub fn parse_impacket_output(output: &str) -> Vec<ImpacketCredential> {
    let mut creds = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("$krb5asrep$") || line.starts_with("$krb5tgs$") {
            if let Some((service, user, secret)) = parse_kerberos_line(line) {
                creds.push(ImpacketCredential {
                    service,
                    user,
                    secret,
                });
            }
        } else if let Some(cred) = parse_secretsdump_line(line) {
            creds.push(cred);
        }
    }

    creds
}

fn parse_kerberos_line(line: &str) -> Option<(String, String, String)> {
    let (prefix, service) = if line.starts_with("$krb5asrep$") {
        ("$krb5asrep$", "krb5asrep")
    } else {
        ("$krb5tgs$", "krb5tgs")
    };
    let rest = line.strip_prefix(prefix)?;
    // rest = "<etype>$<principal>..."
    let after_enc = rest.split('$').nth(1)?;
    let user = if service == "krb5asrep" {
        // "user@REALM:hash"
        after_enc.split('@').next()?.to_string()
    } else {
        // "*user$realm$spn$checksum"
        after_enc.strip_prefix('*')?.split('$').next()?.to_string()
    };
    if user.is_empty() {
        return None;
    }
    Some((service.to_string(), user, line.to_string()))
}

fn parse_secretsdump_line(line: &str) -> Option<ImpacketCredential> {
    // user:rid:lmhash:nthash:::  (rid is numeric — the format signature)
    let fields: Vec<&str> = line.split(':').collect();
    if fields.len() < 4 || fields[1].chars().all(|c| !c.is_ascii_digit()) {
        return None;
    }
    let user = fields[0].rsplit('\\').next().unwrap_or("").to_string();
    let nthash = fields[3].to_string();
    if user.is_empty() || nthash.len() < 32 {
        return None;
    }
    Some(ImpacketCredential {
        service: "ntlm".to_string(),
        user,
        secret: nthash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_asreproast() {
        let output =
            "$krb5asrep$23$svc_scan@CORP.LOCAL:f5d3f4b2a1c8e9d0deadbeef12345678abcde9876543210f";
        let creds = parse_impacket_output(output);
        assert_eq!(creds.len(), 1);
        assert_eq!(creds[0].service, "krb5asrep");
        assert_eq!(creds[0].user, "svc_scan");
        assert!(creds[0].secret.starts_with("$krb5asrep$"));
    }

    #[test]
    fn test_parse_kerberoast() {
        let output = "$krb5tgs$23$*svc_sql$CORP.LOCAL$MSSQLSvc/host.corp.local:1433$aabbccddeeff00112233445566778899";
        let creds = parse_impacket_output(output);
        assert_eq!(creds.len(), 1);
        assert_eq!(creds[0].service, "krb5tgs");
        assert_eq!(creds[0].user, "svc_sql");
    }

    #[test]
    fn test_parse_secretsdump() {
        let output = "Administrator:500:aad3b435b51404eeaad3b435b51404ee:31d6cfe0d16ae931b73c59d7e0c089c0:::\nCORP\\bob:1001:aad3b435b51404eeaad3b435b51404ee:e52cac67449a9a9a9a9a9a9a9a9a9a9a:::";
        let creds = parse_impacket_output(output);
        assert_eq!(creds.len(), 2);
        assert_eq!(creds[0].service, "ntlm");
        assert_eq!(creds[0].user, "Administrator");
        assert_eq!(creds[0].secret, "31d6cfe0d16ae931b73c59d7e0c089c0");
        assert_eq!(creds[1].user, "bob");
    }

    #[test]
    fn test_parse_empty_and_noise() {
        assert!(parse_impacket_output("").is_empty());
        let noise = "[-] Kerberos SessionError: KDC_ERR_C_PRINCIPAL_UNKNOWN\n[!] User: x does not have UF_DONT_REQUIRE_PREAUTH set\n[*] Dumping Domain Credentials";
        assert!(parse_impacket_output(noise).is_empty());
    }
}
