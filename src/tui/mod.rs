pub mod app;
pub mod input_handler;
pub mod layout;
pub mod tool_handler;

use crate::ai::Provider;

fn print_ok(msg: impl std::fmt::Display) {
    eprintln!("  [\x1b[32m\u{2713}\x1b[0m] {} ", msg);
}

fn print_fail(msg: impl std::fmt::Display) {
    eprintln!("  [\x1b[31m\u{2717}\x1b[0m] {} ", msg);
}

fn print_info(msg: impl std::fmt::Display) {
    eprintln!("  [\x1b[34m*\x1b[0m] {} ", msg);
}

fn prompt_first_launch(config: &mut crate::config::Config) {
    use std::io::Write;

    eprintln!();
    eprintln!("  ┌──────────────────────────────────────────────────────────┐");
    eprintln!("  │  First launch — choose your default model               │");
    eprintln!("  ├──────────────────────────────────────────────────────────┤");
    eprintln!("  │  Available models:                                      │");
    eprintln!("  │    1. qwen3          (local)    [recommended]           │");
    eprintln!("  │    2. nous-hermes3   (local)                            │");
    eprintln!("  │    3. deepseek-r1    (local)                            │");
    eprintln!("  │    4. mistral        (local)                            │");
    eprintln!("  │                                                        │");
    eprintln!("  │  Cloud models (set API key in environment):            │");
    eprintln!("  │    anthropic, openai, openrouter, groq, nous,          │");
    eprintln!("  │    llama-api, together                                 │");
    eprintln!("  └──────────────────────────────────────────────────────────┘");

    loop {
        eprint!("  Select default model [1]: ");
        std::io::stderr().flush().ok();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        let choice = input.trim();

        let model = match choice {
            "1" | "" => "qwen3",
            "2" => "nous-hermes3",
            "3" => "deepseek-r1",
            "4" => "mistral",
            other => {
                // Allow typing model name directly
                if other.contains(' ') || other.is_empty() {
                    eprintln!("  Invalid choice. Enter a number or model name.");
                    continue;
                }
                other
            }
        };

        config.ai.model = model.to_string();
        if crate::config::save(config).is_ok() {
            print_info("Config written");
        }
        eprintln!();
        break;
    }
}

async fn prompt_ollama_unreachable(
    config: &mut crate::config::Config,
) -> Option<crate::ai::ProviderKind> {
    use std::io::Write;

    loop {
        eprintln!();
        eprintln!("╔══════════════════════════════════════════════════════════╗");
        eprintln!("║  Ollama is configured but unreachable.                 ║");
        eprintln!("║                                                       ║");
        eprintln!("║  Host: {}", config.ollama.host);
        eprintln!("║                                                       ║");
        eprintln!("║  Ensure Ollama is installed and running.              ║");
        eprintln!("║  https://ollama.com/download                          ║");
        eprintln!("╚══════════════════════════════════════════════════════════╝");
        eprint!("(R)etry or (c)ontinue without AI? [R/c] ");
        std::io::stderr().flush().ok();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        match input.trim().to_lowercase().as_str() {
            "r" | "retry" | "" => {
                eprint!("Checking Ollama... ");
                std::io::stderr().flush().ok();
                let provider = crate::ai::resolve_provider(config);
                if let Some(crate::ai::ProviderKind::Ollama(ref p)) = provider {
                    if p.check_connection().await {
                        eprintln!("connected.\n");
                        return provider;
                    }
                }
                eprintln!("still unreachable.\n");
            }
            "c" | "continue" | "n" | "no" => {
                eprintln!("Continuing without an AI provider.\n");
                return None;
            }
            _ => {
                eprintln!("Please answer r or c.");
            }
        }
    }
}

fn prompt_fallback_provider(config: &mut crate::config::Config) -> Option<crate::ai::ProviderKind> {
    use std::io::Write;

    loop {
        eprintln!();
        eprintln!("╔══════════════════════════════════════════════════════════╗");
        eprintln!("║  No AI provider is configured and usable.              ║");
        eprintln!("║                                                       ║");
        eprintln!("║  Configured: {}", config.ai.provider);
        eprintln!("║                                                       ║");
        eprintln!("║  Fallback: Ollama (localhost:11434)                   ║");
        eprintln!("║  ─ Free, no API key, runs locally on your machine.    ║");
        eprintln!("║  ─ Requires Ollama to be installed and running.       ║");
        eprintln!("║  ─ Model quality depends on the model you pull.       ║");
        eprintln!("║  ─ Only pull models from trusted sources.             ║");
        eprintln!("╚══════════════════════════════════════════════════════════╝");
        eprint!("Use Ollama as your AI provider? [y/N] ");
        std::io::stderr().flush().ok();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => {
                config.ai.provider = "ollama".to_string();
                let provider = crate::ai::resolve_provider(config);
                if provider.is_some() {
                    eprintln!("  \u{2713} Using Ollama (model: {}).\n", config.ai.model);
                    return provider;
                }
                eprintln!("  \u{2717} Could not configure Ollama. Continuing without AI.\n");
                return None;
            }
            "n" | "no" | "" => {
                eprintln!("Continuing without an AI provider.\n");
                return None;
            }
            _ => {
                eprintln!("Please answer y or n.");
            }
        }
    }
}

pub async fn run(config: crate::config::Config) {
    let data_dir = crate::config::init_data_dirs().unwrap_or_else(|e| {
        eprintln!("Failed to init data dirs: {}", e);
        std::process::exit(1);
    });

    let mut config = config;
    let is_first_launch = crate::config::is_first_launch();

    // Boot-style tool check
    eprintln!();
    print_info("Checking tools...");
    let tools = crate::tools::check_all_tools();
    for tool in &tools {
        if tool.available {
            let ver = tool.version.as_deref().unwrap_or("?");
            print_ok(format!("{} {}", tool.name, ver));
        } else {
            print_fail(format!("{} (not found)", tool.name));
        }
    }

    // Provider check
    eprintln!();
    print_info("Checking AI providers...");
    let mut provider = crate::ai::resolve_provider(&config);
    let provider_name = provider
        .as_ref()
        .map(|p| p.name().to_string())
        .unwrap_or_else(|| "none".to_string());

    if provider.is_some() {
        print_ok(format!("{} — {}", provider_name, config.ai.model));
    } else {
        print_fail("No AI provider detected");
    }

    // First launch: model selection
    if is_first_launch {
        prompt_first_launch(&mut config);
        // Re-resolve provider after model selection
        provider = crate::ai::resolve_provider(&config);
    }

    // Provider availability fallbacks
    if provider.is_none() {
        provider = prompt_fallback_provider(&mut config);
    } else if let Some(crate::ai::ProviderKind::Ollama(ref p)) = provider {
        if !p.check_connection().await {
            provider = prompt_ollama_unreachable(&mut config).await;
        }
    }

    let available = crate::tools::available_tool_names();
    let agent_shell = crate::agent::shell::AgentShell::new(available);

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
