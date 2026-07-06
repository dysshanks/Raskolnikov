use crate::agent::context::EngagementContext;

pub struct PromptBuilder;

impl PromptBuilder {
    pub fn build(context: &EngagementContext, available_tools: &[&str]) -> String {
        let tools_list = available_tools.join(", ");
        let context_str = context.to_context_string();
        let template = include_str!("SYSTEM_PROMPT.md");

        template
            .replace("{tools_list}", &tools_list)
            .replace("{context_str}", &context_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_file_exists() {
        let ctx = EngagementContext::new();
        let prompt = PromptBuilder::build(&ctx, &["nmap"]);
        assert!(!prompt.is_empty());
        assert!(prompt.contains("Raskolnikov"));
    }
}
