pub mod app;
pub mod input;
pub mod layout;

pub async fn run(config: crate::config::Config) {
    let data_dir = crate::config::init_data_dirs().unwrap_or_else(|e| {
        eprintln!("Failed to init data dirs: {}", e);
        std::process::exit(1);
    });

    let tools = crate::tools::check_all_tools();
    let available = crate::tools::available_tool_names();
    let agent_shell = crate::agent::shell::AgentShell::new(available);
    let provider = crate::ai::resolve_provider(&config);

    let session_id = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%S").to_string();
    let session_dir = data_dir.join("sessions").join(&session_id);
    let logger = crate::session::logger::SessionLogger::new(&session_dir).ok();

    let mut app = app::App::new_with(
        provider,
        agent_shell,
        logger,
        config,
        session_id,
        session_dir,
    );

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

    let provider_name = app.provider_name.clone();
    let model_name = app.model_name.clone();
    app.conversation
        .push(format!("[system] AI {} — {}", provider_name, model_name));

    app.conversation.push(String::new());
    app.conversation.push(" Ready. Type anything.".to_string());
    app.run().await;
}
