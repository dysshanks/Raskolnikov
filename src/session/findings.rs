use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct PortFinding {
    pub port: u16,
    pub protocol: String,
    pub service: String,
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct WebPathFinding {
    pub path: String,
    pub status_code: u16,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct FlagFinding {
    pub description: String,
}

pub struct FindingsExport;

impl FindingsExport {
    pub fn write(
        path: &Path,
        date: &str,
        model: &str,
        provider: &str,
        ports: &[PortFinding],
        web_paths: &[WebPathFinding],
        flags: &[FlagFinding],
    ) -> Result<(), std::io::Error> {
        let mut md = String::new();
        md.push_str("# Findings\n");
        md.push_str(&format!(
            "**Date:** {}  **Model:** {}  **Provider:** {}\n\n",
            date, model, provider
        ));

        if !ports.is_empty() {
            md.push_str("## Open Ports\n");
            md.push_str("| Port | Protocol | Service | Version |\n");
            md.push_str("|------|----------|---------|--------|\n");
            for p in ports {
                md.push_str(&format!(
                    "| {}/{} | {} | {} | {} |\n",
                    p.port, p.protocol, p.protocol, p.service, p.version
                ));
            }
            md.push('\n');
        }

        if !web_paths.is_empty() {
            md.push_str("## Web Paths\n");
            md.push_str("| Path | Status | Notes |\n");
            md.push_str("|------|--------|-------|\n");
            for p in web_paths {
                md.push_str(&format!(
                    "| {} | {} | {} |\n",
                    p.path, p.status_code, p.notes
                ));
            }
            md.push('\n');
        }

        if !flags.is_empty() {
            md.push_str("## Flags\n");
            for f in flags {
                md.push_str(&format!("- {}\n", f.description));
            }
            md.push('\n');
        }

        fs::write(path, md)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_findings_write() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("findings.md");

        let ports = vec![
            PortFinding {
                port: 22,
                protocol: "tcp".to_string(),
                service: "SSH".to_string(),
                version: "OpenSSH 8.9".to_string(),
            },
            PortFinding {
                port: 80,
                protocol: "tcp".to_string(),
                service: "HTTP".to_string(),
                version: "Apache 2.4.52".to_string(),
            },
        ];
        let web_paths = vec![WebPathFinding {
            path: "/admin".to_string(),
            status_code: 302,
            notes: "Redirects to /admin/login".to_string(),
        }];
        let flags = vec![FlagFinding {
            description: "MySQL exposed directly to network".to_string(),
        }];

        FindingsExport::write(
            &path,
            "2025-06-16",
            "qwen3",
            "ollama",
            &ports,
            &web_paths,
            &flags,
        )
        .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("22/tcp"));
        assert!(content.contains("Apache 2.4.52"));
        assert!(content.contains("/admin"));
        assert!(content.contains("MySQL exposed"));
    }
}
