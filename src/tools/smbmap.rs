use crate::tools::smbclient::SmbShare;

/// Parses `smbmap -H host` output into shares with access permissions:
///
/// ```text
/// [+] IP: 10.0.0.1:445    Name: 10.0.0.1    Status: Guest
///    Disk                Permissions    Comment
///    ----                -----------    -------
///    ADMIN$              NO ACCESS
///    Public              READ, WRITE
/// ```
pub fn parse_smbmap_output(output: &str) -> Vec<SmbShare> {
    let mut shares = Vec::new();
    let mut in_table = false;

    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line
            .split('\t')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if cols.len() >= 3 && cols[0] == "Disk" && cols[1] == "Permissions" && cols[2] == "Comment"
        {
            in_table = true;
            continue;
        }
        if !in_table || cols.len() < 2 {
            continue;
        }

        let name = cols[0];
        if name.starts_with('#') || name.starts_with('-') {
            continue;
        }
        shares.push(SmbShare {
            name: name.to_string(),
            share_type: cols[1].to_string(),
        });
    }

    shares
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_smbmap_output() {
        let output = "[+] IP: 10.0.0.1:445\tName: 10.0.0.1\tStatus: Guest\n\tDisk\t\t\tPermissions\tComment\n\t----\t\t\t-----------\t-------\n\tADMIN$\t\t\tNO ACCESS\t\n\tC$\t\t\tNO ACCESS\t\n\tPublic\t\t\tREAD, WRITE\t\n[+] Host 10.0.0.1 Local admin: False\n";
        let shares = parse_smbmap_output(output);
        assert_eq!(shares.len(), 3);
        assert_eq!(shares[0].name, "ADMIN$");
        assert_eq!(shares[0].share_type, "NO ACCESS");
        assert_eq!(shares[2].name, "Public");
        assert_eq!(shares[2].share_type, "READ, WRITE");
    }

    #[test]
    fn test_parse_smbmap_empty() {
        assert!(parse_smbmap_output("").is_empty());
    }

    #[test]
    fn test_parse_smbmap_no_access() {
        let output = "[+] IP: 10.0.0.1:445\tName: 10.0.0.1\tStatus: User\n[-] Error connecting to 10.0.0.1\n";
        assert!(parse_smbmap_output(output).is_empty());
    }
}
