use crate::agent::context::EngagementContext;
use crate::agent::prompt::PromptBuilder;

pub struct AgentShell {
    pub context: EngagementContext,
    pub history: Vec<String>,
    pub available_tools: Vec<String>,
}

impl AgentShell {
    pub fn new(available_tools: Vec<String>) -> Self {
        Self {
            context: EngagementContext::new(),
            history: Vec::new(),
            available_tools,
        }
    }

    pub fn build_prompt(&self) -> String {
        let tool_names: Vec<&str> = self.available_tools.iter().map(|s| s.as_str()).collect();
        PromptBuilder::build(&self.context, &tool_names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_new() {
        let tools = vec!["nmap".to_string(), "gobuster".to_string()];
        let shell = AgentShell::new(tools.clone());
        assert_eq!(shell.available_tools.len(), 2);
        assert!(shell.history.is_empty());
    }

    #[test]
    fn test_build_prompt() {
        let shell = AgentShell::new(vec!["nmap".to_string()]);
        let prompt = shell.build_prompt();
        assert!(prompt.contains("nmap"));
    }

    #[test]
    fn test_build_prompt_includes_context() {
        let mut shell = AgentShell::new(vec!["nmap".to_string()]);
        shell.context.ports.push(crate::agent::context::Port {
            port: 80,
            protocol: "tcp".to_string(),
            state: "open".to_string(),
            service: "http".to_string(),
            version: "Apache 2.4.52".to_string(),
        });
        shell
            .context
            .add_credential(crate::agent::context::Credential {
                service: "ssh".to_string(),
                host: "10.0.0.1".to_string(),
                user: "root".to_string(),
                password: "toor".to_string(),
            });
        let prompt = shell.build_prompt();
        assert!(prompt.contains("80/tcp"));
        assert!(prompt.contains("Apache 2.4.52"));
        assert!(prompt.contains("CREDENTIALS"));
        assert!(prompt.contains("toor"));
    }
}
