use crate::agent::context::EngagementContext;

pub struct PromptBuilder;

impl PromptBuilder {
    pub fn build(context: &EngagementContext, available_tools: &[&str]) -> String {
        let tools_list = available_tools.join(", ");
        let context_str = context.to_context_string();

        format!(
            r#"You are Raskolnikov, a security agent running in a terminal.
You are assisting a security operator with penetration testing.

=== AVAILABLE TOOLS ===
{tools_list}

=== RULES ===
1. Always explain your reasoning before proposing a tool.
2. Always state the exact command you want to run in a code block.
3. Never execute a tool without operator approval.
4. Wait for "yes" or "go ahead" before proceeding.
5. If the operator says "no" or changes direction, adapt.
6. If you want to run a tool, end your message with " — run this?"

=== CURRENT CONTEXT ===
{context_str}

Respond naturally. Be concise but informative.
"#,
            tools_list = tools_list,
            context_str = context_str,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_contains_context() {
        let ctx = EngagementContext::new();
        let prompt = PromptBuilder::build(&ctx, &["nmap", "gobuster"]);
        assert!(prompt.contains("Raskolnikov"));
        assert!(prompt.contains("nmap"));
        assert!(prompt.contains("gobuster"));
        assert!(prompt.contains("AVAILABLE TOOLS"));
    }
}
