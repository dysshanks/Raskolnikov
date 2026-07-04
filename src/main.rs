use clap::{Parser, Subcommand};
use raskolnikov::ai::Provider;

#[derive(Parser, Debug)]
#[command(
    name = "raskolnikov",
    about = "Terminal-native AI security operating environment"
)]
struct Args {
    #[arg(long = "version", help = "Print version and exit")]
    version: bool,

    #[arg(long = "model", help = "Override default model for this session")]
    model: Option<String>,

    #[arg(long = "provider", help = "Override default provider for this session")]
    provider: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List and manage past sessions
    Sessions {
        #[command(subcommand)]
        action: SessionsAction,
    },
    /// View or modify configuration
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },
    /// Check tool availability and versions
    Tools,
}

#[derive(Subcommand, Debug)]
enum SessionsAction {
    /// List all sessions
    List,
    /// Show conversation transcript
    Show { id: String },
    /// Show findings summary
    Findings { id: String },
    /// Dump raw JSON session log
    Log { id: String },
    /// Remove old sessions
    Prune {
        #[arg(
            long = "keep",
            help = "Keep sessions from the last N days (--keep 30) or N most recent (--keep 10)"
        )]
        keep: Option<u32>,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigAction {
    /// Show current configuration
    Show,
    /// Set AI provider
    Provider { provider: String },
    /// Set default model
    Model { model: String },
    /// Set an arbitrary config key-value pair
    Set { key: String, value: String },
}

#[tokio::main]
async fn main() -> raskolnikov::config::Result<()> {
    let args = Args::parse();

    if args.version {
        println!("raskolnikov {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    match args.command {
        Some(Commands::Sessions { action }) => handle_sessions(action),
        Some(Commands::Config { action }) => handle_config(action),
        Some(Commands::Tools) => handle_tools(),
        None => {
            let config = raskolnikov::config::load()?;
            let _data_dir = raskolnikov::config::init_data_dirs()?;

            let tools = raskolnikov::tools::check_all_tools();
            for tool in &tools {
                if tool.available {
                    let ver = tool.version.as_deref().unwrap_or("?");
                    eprintln!("  \u{2713} {} {}", tool.name, ver);
                } else {
                    eprintln!("  \u{2717} {}  (not found)", tool.name);
                }
            }

            if let Some(provider) = raskolnikov::ai::resolve_provider(&config) {
                eprintln!("  \u{2713} {} — {}", provider.name(), config.ai.model);
            } else {
                eprintln!("  \u{2717} No AI provider available");
            }

            raskolnikov::tui::run().await;
            Ok(())
        }
    }
}

fn handle_sessions(action: SessionsAction) -> raskolnikov::config::Result<()> {
    match action {
        SessionsAction::List => {
            let dir = raskolnikov::config::data_dir().join("sessions");
            if !dir.exists() {
                println!("No sessions found.");
                return Ok(());
            }
            let mut entries: Vec<_> = std::fs::read_dir(&dir)
                .map_err(|e| format!("Failed to read sessions: {}", e))?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .collect();
            entries.sort_by_key(|e| e.path());

            if entries.is_empty() {
                println!("No sessions found.");
            } else {
                println!("Sessions:");
                for entry in &entries {
                    let name = entry.file_name();
                    let conv_path = entry.path().join("conversation.md");
                    let has_conv = conv_path.exists();
                    println!(
                        "  {} {}",
                        name.to_string_lossy(),
                        if has_conv { "" } else { "(incomplete)" }
                    );
                }
            }
            Ok(())
        }
        SessionsAction::Show { id } => {
            let path = raskolnikov::config::data_dir()
                .join("sessions")
                .join(&id)
                .join("conversation.md");
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("Session '{}' not found: {}", id, e))?;
            print!("{}", content);
            Ok(())
        }
        SessionsAction::Findings { id } => {
            let path = raskolnikov::config::data_dir()
                .join("sessions")
                .join(&id)
                .join("findings.md");
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("Findings for session '{}' not found: {}", id, e))?;
            print!("{}", content);
            Ok(())
        }
        SessionsAction::Log { id } => {
            let path = raskolnikov::config::data_dir()
                .join("sessions")
                .join(&id)
                .join("session.log");
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("Session log '{}' not found: {}", id, e))?;
            print!("{}", content);
            Ok(())
        }
        SessionsAction::Prune { keep } => {
            let days = keep.unwrap_or(30);
            println!(
                "Pruning sessions older than {} days (not yet implemented).",
                days
            );
            Ok(())
        }
    }
}

fn handle_config(action: Option<ConfigAction>) -> raskolnikov::config::Result<()> {
    let mut config = raskolnikov::config::load()?;

    match action {
        None | Some(ConfigAction::Show) => {
            let content = toml::to_string_pretty(&config)
                .map_err(|e| format!("Failed to serialize config: {}", e))?;
            print!("{}", content);
            Ok(())
        }
        Some(ConfigAction::Provider { provider }) => {
            config.ai.provider = provider;
            raskolnikov::config::save(&config)?;
            println!("Provider set to {}", config.ai.provider);
            Ok(())
        }
        Some(ConfigAction::Model { model }) => {
            config.ai.model = model;
            raskolnikov::config::save(&config)?;
            println!("Model set to {}", config.ai.model);
            Ok(())
        }
        Some(ConfigAction::Set { key, value }) => {
            match key.as_str() {
                "provider" => config.ai.provider = value.clone(),
                "model" => config.ai.model = value.clone(),
                "nmap_timing" => {
                    config.tools.nmap_timing = value
                        .parse()
                        .map_err(|_| format!("Invalid nmap_timing: {}", value))?;
                }
                "prefer_ffuf" => {
                    config.tools.prefer_ffuf = value
                        .parse()
                        .map_err(|_| format!("Invalid prefer_ffuf: {}", value))?;
                }
                _ => return Err(format!("Unknown config key: {}", key).into()),
            }
            raskolnikov::config::save(&config)?;
            println!("Set {} = {}", key, value);
            Ok(())
        }
    }
}

fn handle_tools() -> raskolnikov::config::Result<()> {
    let tools = raskolnikov::tools::check_all_tools();
    for tool in &tools {
        if tool.available {
            let ver = tool.version.as_deref().unwrap_or("?");
            println!("  \u{2713} {}  {}", tool.name, ver);
        } else {
            println!("  \u{2717} {}  (not found)", tool.name);
        }
    }
    Ok(())
}
