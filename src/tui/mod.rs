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
    eprintln!("  │  First launch — choose your AI provider                 │");
    eprintln!("  ├──────────────────────────────────────────────────────────┤");
    eprintln!("  │    1. Auto (detect from environment)  [recommended]     │");
    eprintln!("  │    2. Ollama (local, no API key)                       │");
    eprintln!("  │                                                        │");
    eprintln!("  │  Cloud providers:                                      │");
    eprintln!(
        "  │    3. Anthropic        {}",
        key_status("ANTHROPIC_API_KEY")
    );
    eprintln!(
        "  │    4. OpenAI           {}",
        key_status("OPENAI_API_KEY")
    );
    eprintln!("  │    5. Groq             {}", key_status("GROQ_API_KEY"));
    eprintln!(
        "  │    6. OpenRouter       {}",
        key_status("OPENROUTER_API_KEY")
    );
    eprintln!("  │    7. Nous Research    {}", key_status("NOUS_API_KEY"));
    eprintln!("  │    8. Llama API        {}", key_status("LLAMA_API_KEY"));
    eprintln!(
        "  │    9. Together AI      {}",
        key_status("TOGETHER_API_KEY")
    );
    eprintln!("  └──────────────────────────────────────────────────────────┘");

    let provider = loop {
        eprint!("  Select provider [1]: ");
        std::io::stderr().flush().ok();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        let choice = input.trim();

        let p = match choice {
            "1" | "" => "auto",
            "2" => "ollama",
            "3" => "anthropic",
            "4" => "openai",
            "5" => "groq",
            "6" => "openrouter",
            "7" => "nous",
            "8" => "llama-api",
            "9" => "together",
            other => {
                if other.is_empty() {
                    "auto"
                } else {
                    eprintln!("  Invalid choice. Enter a number 1-9.");
                    continue;
                }
            }
        };
        break p;
    };

    config.ai.provider = provider.to_string();

    if provider != "ollama" && provider != "auto" {
        let env_key = match provider {
            "anthropic" => "ANTHROPIC_API_KEY",
            "openai" => "OPENAI_API_KEY",
            "groq" => "GROQ_API_KEY",
            "openrouter" => "OPENROUTER_API_KEY",
            "nous" => "NOUS_API_KEY",
            "llama-api" => "LLAMA_API_KEY",
            "together" => "TOGETHER_API_KEY",
            _ => "",
        };
        if !env_key.is_empty() && std::env::var(env_key).is_err() {
            eprintln!();
            eprintln!("  Warning: {} is not set.", env_key);
            eprintln!("  Set it before starting: export {}=your-key-here", env_key);
            eprintln!();
        }
    }

    let models = match provider {
        "anthropic" => vec![
            ("1", "claude-sonnet-4-6", "recommended"),
            ("2", "claude-opus-4-6", ""),
            ("3", "claude-haiku-3-5", "fast"),
        ],
        "openai" => vec![
            ("1", "gpt-4.1", "recommended"),
            ("2", "o3", ""),
            ("3", "gpt-4.1-mini", "fast"),
        ],
        "groq" => vec![
            ("1", "llama-4-maverick-17b", "recommended"),
            ("2", "llama-3.3-70b", ""),
            ("3", "mixtral-8x7b", "fast"),
        ],
        "openrouter" => vec![
            ("1", "anthropic/claude-sonnet-4", "recommended"),
            ("2", "openai/gpt-4.1", ""),
            ("3", "meta-llama/llama-4-maverick", ""),
        ],
        "nous" => vec![
            ("1", "hermes-3-llama-3.1-405b", "recommended"),
            ("2", "hermes-3-llama-3.1-70b", "fast"),
        ],
        "llama-api" => vec![
            ("1", "llama-4-maverick", "recommended"),
            ("2", "llama-4-scout", ""),
        ],
        "together" => vec![
            (
                "1",
                "meta-llama/Llama-4-Maverick-17B-128E-Instruct",
                "recommended",
            ),
            ("2", "meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo", ""),
        ],
        _ => vec![
            ("1", "qwen3", "recommended"),
            ("2", "nous-hermes3", ""),
            ("3", "deepseek-r1", ""),
            ("4", "mistral", ""),
        ],
    };

    eprintln!();
    eprintln!("  Available models:");
    for (num, name, tag) in &models {
        let tag_str = if tag.is_empty() {
            String::new()
        } else {
            format!("    [{}]", tag)
        };
        eprintln!("    {}. {}{}", num, name, tag_str);
    }

    loop {
        eprint!("  Select model [1]: ");
        std::io::stderr().flush().ok();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        let choice = input.trim();

        let model = match choice {
            "1" | "" => models[0].1,
            "2" => models.get(1).map(|m| m.1).unwrap_or(""),
            "3" => models.get(2).map(|m| m.1).unwrap_or(""),
            "4" => models.get(3).map(|m| m.1).unwrap_or(""),
            other => {
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

fn key_status(env_var: &str) -> String {
    match std::env::var(env_var) {
        Ok(_) => "\u{2713} key set".to_string(),
        Err(_) => "\u{2717} no key".to_string(),
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
    crate::ai::set_http_timeout(config.network.timeout_secs);

    let data_dir = crate::config::init_data_dirs_for(&config).unwrap_or_else(|e| {
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
