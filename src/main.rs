use clap::{Parser, Subcommand};

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
            help = "Keep sessions from the last N days (--keep 30) or N most recent (--keep 10r)"
        )]
        keep: Option<String>,
    },
    /// Recover conversation.md and findings.md from session.log
    Recover { id: String },
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
    init_tracing();
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
            let mut config = raskolnikov::config::load()?;

            if let Some(model) = &args.model {
                config.ai.model = model.clone();
            }
            if let Some(provider) = &args.provider {
                config.ai.provider = provider.clone();
            }

            let _data_dir = raskolnikov::config::init_data_dirs_for(&config)?;

            if !config.cli.alias.is_empty() {
                create_alias_symlink(&config.cli.alias);
            }

            raskolnikov::tui::run(config).await;
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
            let dir = raskolnikov::config::data_dir().join("sessions");
            if !dir.exists() {
                println!("No sessions directory found.");
                return Ok(());
            }

            let entries: Vec<_> = std::fs::read_dir(&dir)
                .map_err(|e| format!("Failed to read sessions: {}", e))?
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .collect();

            let keep_str = keep.as_deref().unwrap_or("30d");

            if keep_str.ends_with('r') || keep_str.ends_with("recent") {
                let count: u32 = keep_str
                    .trim_end_matches("recent")
                    .trim_end_matches('r')
                    .trim()
                    .parse()
                    .map_err(|_| format!("Invalid keep count: {}", keep_str))?;

                let mut parsed: Vec<_> = entries
                    .iter()
                    .filter_map(|e| {
                        let name = e.file_name();
                        let name_str = name.to_string_lossy().to_string();
                        chrono::NaiveDateTime::parse_from_str(&name_str, "%Y-%m-%dT%H-%M-%S")
                            .ok()
                            .map(|ts| (e.path(), ts.and_utc()))
                    })
                    .collect();

                parsed.sort_by_key(|b| std::cmp::Reverse(b.1));

                let mut pruned = 0u32;
                for (path, _) in parsed.into_iter().skip(count as usize) {
                    if std::fs::remove_dir_all(path).is_ok() {
                        pruned += 1;
                    }
                }

                if pruned == 0 {
                    println!("No sessions to prune (keeping {} most recent).", count);
                } else {
                    println!(
                        "Pruned {} session{} (keeping {} most recent).",
                        pruned,
                        if pruned == 1 { "" } else { "s" },
                        count
                    );
                }
            } else {
                let days: u32 = keep_str
                    .parse()
                    .map_err(|_| format!("Invalid keep value: {}", keep_str))?;

                let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);
                let mut pruned = 0u32;

                for entry in &entries {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if let Ok(ts) =
                        chrono::NaiveDateTime::parse_from_str(&name_str, "%Y-%m-%dT%H-%M-%S")
                    {
                        let ts_utc = ts.and_utc();
                        if ts_utc < cutoff && std::fs::remove_dir_all(entry.path()).is_ok() {
                            pruned += 1;
                        }
                    }
                }

                if pruned == 0 {
                    println!("No sessions older than {} days to prune.", days);
                } else {
                    println!(
                        "Pruned {} session{} older than {} days.",
                        pruned,
                        if pruned == 1 { "" } else { "s" },
                        days
                    );
                }
            }
            Ok(())
        }
        SessionsAction::Recover { id } => {
            let session_dir = raskolnikov::config::data_dir().join("sessions").join(&id);
            if !session_dir.exists() {
                return Err(format!("Session '{}' not found", id).into());
            }
            raskolnikov::session::recover::recover_session(&session_dir)
                .map_err(|e| format!("Recovery failed: {}", e))?;
            println!("Recovered session '{}'", id);
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
                "ollama_host" => config.ollama.host = value.clone(),
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
                "sqlmap_level" => {
                    config.tools.sqlmap_level = value
                        .parse()
                        .map_err(|_| format!("Invalid sqlmap_level: {}", value))?;
                }
                "sqlmap_risk" => {
                    config.tools.sqlmap_risk = value
                        .parse()
                        .map_err(|_| format!("Invalid sqlmap_risk: {}", value))?;
                }
                "stream_output" => {
                    config.ui.stream_output = value
                        .parse()
                        .map_err(|_| format!("Invalid stream_output: {}", value))?;
                }
                "alias" => config.cli.alias = value.clone(),
                "context_window" => {
                    config.ai.context_window = value
                        .parse()
                        .map_err(|_| format!("Invalid context_window: {}", value))?;
                }
                "proxy" => config.network.proxy = value.clone(),
                "proxy_https" => config.network.proxy_https = value.clone(),
                "timeout_secs" => {
                    config.network.timeout_secs = value
                        .parse()
                        .map_err(|_| format!("Invalid timeout_secs: {}", value))?;
                }
                "no_proxy" => {
                    config.network.no_proxy = value
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                "use_tmp_data_dir" => {
                    config.use_tmp_data_dir = value
                        .parse()
                        .map_err(|_| format!("Invalid use_tmp_data_dir: {}", value))?;
                }
                _ => return Err(format!("Unknown config key: {}", key).into()),
            }
            raskolnikov::config::save(&config)?;
            println!("Set {} = {}", key, value);
            Ok(())
        }
    }
}

fn create_alias_symlink(alias: &str) {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return,
    };
    let dir = exe.parent().unwrap_or(&exe);
    let link = dir.join(alias);
    if link.exists() {
        return;
    }
    let _ = std::os::unix::fs::symlink(&exe, &link);
    eprintln!("Created alias: {} -> {}", link.display(), exe.display());
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

/// Initialises tracing. Writes to stderr when `RASKOLNIKOV_LOG` is set to a
/// RUST_LOG-style filter, otherwise stays quiet (no file, no noise).
fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    if std::env::var_os("RASKOLNIKOV_LOG").is_some() {
        let filter =
            EnvFilter::try_from_env("RASKOLNIKOV_LOG").unwrap_or_else(|_| EnvFilter::new("warn"));
        let _ = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(std::io::stderr)
            .try_init();
    }
}
