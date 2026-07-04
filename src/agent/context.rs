#[derive(Debug, Clone, Default)]
pub struct EngagementContext {
    pub ports: Vec<Port>,
    pub web_paths: Vec<WebPath>,
    pub findings: Vec<Finding>,
    pub targets: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Port {
    pub port: u16,
    pub protocol: String,
    pub state: String,
    pub service: String,
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct WebPath {
    pub path: String,
    pub status_code: u16,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub description: String,
    pub source: String,
}

impl EngagementContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_finding(&mut self, description: String, source: String) {
        let finding = Finding {
            description,
            source,
        };
        if !self
            .findings
            .iter()
            .any(|f| f.description == finding.description)
        {
            self.findings.push(finding);
        }
    }

    pub fn to_context_string(&self) -> String {
        let mut s = String::new();

        if !self.ports.is_empty() {
            s.push_str("=== DISCOVERED PORTS ===\n");
            for p in &self.ports {
                s.push_str(&format!(
                    "  {}/{} {} {} {}\n",
                    p.port, p.protocol, p.state, p.service, p.version
                ));
            }
        }

        if !self.web_paths.is_empty() {
            s.push_str("=== WEB PATHS ===\n");
            for p in &self.web_paths {
                s.push_str(&format!("  {} {} {}\n", p.path, p.status_code, p.notes));
            }
        }

        if !self.findings.is_empty() {
            s.push_str("=== FINDINGS ===\n");
            for f in &self.findings {
                s.push_str(&format!("  [{}] {}\n", f.source, f.description));
            }
        }

        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finding_dedup() {
        let mut ctx = EngagementContext::new();
        ctx.add_finding("Port 80 open".to_string(), "nmap".to_string());
        ctx.add_finding("Port 80 open".to_string(), "nikto".to_string());
        assert_eq!(ctx.findings.len(), 1);
    }

    #[test]
    fn test_context_string() {
        let mut ctx = EngagementContext::new();
        ctx.ports.push(Port {
            port: 80,
            protocol: "tcp".to_string(),
            state: "open".to_string(),
            service: "http".to_string(),
            version: "Apache 2.4.52".to_string(),
        });
        let s = ctx.to_context_string();
        assert!(s.contains("80/tcp"));
        assert!(s.contains("Apache"));
    }
}
