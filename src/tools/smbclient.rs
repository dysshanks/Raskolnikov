#[derive(Debug, Clone)]
pub struct SmbShare {
    pub name: String,
    pub share_type: String,
}

/// Parses `smbclient -L //host -N` output into a share list:
///
/// ```text
///    Sharename       Type      Comment
///    ---------       ----      -------
///    ADMIN$          Disk      Remote Admin
///    IPC$            IPC       Remote IPC
///    Public          Disk
/// ```
pub fn parse_smbclient_shares(output: &str) -> Vec<SmbShare> {
    let mut shares = Vec::new();
    let mut in_table = false;

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.contains("Sharename") && line.contains("Type") && line.contains("Comment") {
            in_table = true;
            continue;
        }
        if line.starts_with("----") || line.starts_with("SMB1 disabled") {
            continue;
        }
        if line.starts_with("Domain=")
            || line.starts_with("OS=")
            || line.starts_with("Password for")
        {
            continue;
        }
        if !in_table {
            continue;
        }

        let mut parts = line.split_whitespace();
        if let (Some(name), Some(share_type)) = (parts.next(), parts.next()) {
            if matches!(share_type, "Disk" | "IPC") {
                shares.push(SmbShare {
                    name: name.to_string(),
                    share_type: share_type.to_string(),
                });
            }
        }
    }

    shares
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_smbclient_shares() {
        let output = "\tSharename       Type      Comment\n\t---------       ----      -------\n\tADMIN$          Disk      Remote Admin\n\tC$              Disk      Default share\n\tIPC$            IPC       Remote IPC\n\tPublic          Disk\nSMB1 disabled -- no workgroup available\n";
        let shares = parse_smbclient_shares(output);
        assert_eq!(shares.len(), 4);
        assert_eq!(shares[0].name, "ADMIN$");
        assert_eq!(shares[0].share_type, "Disk");
        assert_eq!(shares[3].name, "Public");
    }

    #[test]
    fn test_parse_smbclient_empty() {
        assert!(parse_smbclient_shares("").is_empty());
    }

    #[test]
    fn test_parse_smbclient_noise() {
        let noise = "session setup failed: NT_STATUS_LOGON_FAILURE\n";
        assert!(parse_smbclient_shares(noise).is_empty());
    }
}
