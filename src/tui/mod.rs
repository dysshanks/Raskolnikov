pub mod app;
pub mod input;
pub mod layout;

use crate::ai::Provider;

pub async fn run() {
    let tools = crate::tools::check_all_tools();
    let mut app = app::App::new();
    for tool in &tools {
        if tool.available {
            let ver = tool.version.as_deref().unwrap_or("?");
            app.conversation
                .push(format!("[system] Tool \u{2713} {} {}", tool.name, ver));
        } else {
            app.conversation
                .push(format!("[system] Tool \u{2717} {} (not found)", tool.name));
        }
    }

    let config = crate::config::load().unwrap_or_default();
    if let Some(provider) = crate::ai::resolve_provider(&config) {
        app.conversation.push(format!(
            "[system] AI {} — {}",
            provider.name(),
            config.ai.model
        ));
    } else {
        app.conversation
            .push("[system] AI no provider available".to_string());
    }

    app.conversation.push(String::new());
    app.conversation.push(" Ready. Type anything.".to_string());
    app.run().await;
}
