use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub cli: CliConfig,
    #[serde(default)]
    pub ai: AiConfig,
    #[serde(default)]
    pub ollama: OllamaConfig,
    #[serde(default)]
    pub anthropic: AnthropicConfig,
    #[serde(default)]
    pub openai: OpenAiConfig,
    #[serde(default)]
    pub nous: NousConfig,
    #[serde(default)]
    pub groq: GroqConfig,
    #[serde(default)]
    pub llama_api: LlamaApiConfig,
    #[serde(default)]
    pub together: TogetherConfig,
    #[serde(default)]
    pub tools: ToolsConfig,
    #[serde(default)]
    pub wordlists: WordlistsConfig,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub network: NetworkConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CliConfig {
    #[serde(default)]
    pub alias: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default = "default_model")]
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    #[serde(default = "default_ollama_host")]
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicConfig {
    #[serde(default = "default_anthropic_base_url")]
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    #[serde(default = "default_openai_base_url")]
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NousConfig {
    #[serde(default = "default_nous_base_url")]
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroqConfig {
    #[serde(default = "default_groq_base_url")]
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlamaApiConfig {
    #[serde(default = "default_llama_base_url")]
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TogetherConfig {
    #[serde(default = "default_together_base_url")]
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    #[serde(default)]
    pub prefer_ffuf: bool,
    #[serde(default = "default_nmap_timing")]
    pub nmap_timing: u8,
    #[serde(default = "default_sqlmap_level")]
    pub sqlmap_level: u8,
    #[serde(default = "default_sqlmap_risk")]
    pub sqlmap_risk: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordlistsConfig {
    #[serde(default = "default_wordlist_paths")]
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_true")]
    pub stream_output: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(default)]
    pub proxy: String,
    #[serde(default)]
    pub proxy_https: String,
    #[serde(default)]
    pub no_proxy: Vec<String>,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            model: default_model(),
        }
    }
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            host: default_ollama_host(),
        }
    }
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            base_url: default_anthropic_base_url(),
        }
    }
}

impl Default for OpenAiConfig {
    fn default() -> Self {
        Self {
            base_url: default_openai_base_url(),
        }
    }
}

impl Default for NousConfig {
    fn default() -> Self {
        Self {
            base_url: default_nous_base_url(),
        }
    }
}

impl Default for GroqConfig {
    fn default() -> Self {
        Self {
            base_url: default_groq_base_url(),
        }
    }
}

impl Default for LlamaApiConfig {
    fn default() -> Self {
        Self {
            base_url: default_llama_base_url(),
        }
    }
}

impl Default for TogetherConfig {
    fn default() -> Self {
        Self {
            base_url: default_together_base_url(),
        }
    }
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            prefer_ffuf: false,
            nmap_timing: default_nmap_timing(),
            sqlmap_level: default_sqlmap_level(),
            sqlmap_risk: default_sqlmap_risk(),
        }
    }
}

impl Default for WordlistsConfig {
    fn default() -> Self {
        Self {
            paths: default_wordlist_paths(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            stream_output: default_true(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            proxy: String::new(),
            proxy_https: String::new(),
            no_proxy: vec!["localhost".to_string(), "127.0.0.1".to_string()],
        }
    }
}

fn default_provider() -> String {
    "ollama".to_string()
}

fn default_model() -> String {
    "qwen3".to_string()
}

fn default_ollama_host() -> String {
    "http://localhost:11434".to_string()
}

fn default_anthropic_base_url() -> String {
    "https://api.anthropic.com".to_string()
}

fn default_openai_base_url() -> String {
    "https://api.openai.com/v1".to_string()
}

fn default_nous_base_url() -> String {
    "https://inference-api.nousresearch.com/v1".to_string()
}

fn default_groq_base_url() -> String {
    "https://api.groq.com/openai/v1".to_string()
}

fn default_llama_base_url() -> String {
    "https://api.llama.com/v1".to_string()
}

fn default_together_base_url() -> String {
    "https://api.together.xyz/v1".to_string()
}

fn default_nmap_timing() -> u8 {
    4
}

fn default_sqlmap_level() -> u8 {
    2
}

fn default_sqlmap_risk() -> u8 {
    1
}

fn default_wordlist_paths() -> Vec<String> {
    vec![
        "/usr/share/wordlists/dirbuster/directory-list-2.3-medium.txt".to_string(),
        "/usr/share/seclists/Discovery/Web-Content/common.txt".to_string(),
        "/usr/share/seclists/Discovery/Web-Content/raft-medium-words.txt".to_string(),
        "/usr/share/wordlists/dirb/common.txt".to_string(),
        "/usr/share/wordlists/dirb/big.txt".to_string(),
    ]
}

fn default_true() -> bool {
    true
}

pub struct ApiKeys {
    pub anthropic: Option<String>,
    pub openai: Option<String>,
    pub openrouter: Option<String>,
    pub groq: Option<String>,
    pub nous: Option<String>,
    pub llama: Option<String>,
    pub together: Option<String>,
}

impl ApiKeys {
    pub fn from_env() -> Self {
        Self {
            anthropic: std::env::var("ANTHROPIC_API_KEY").ok(),
            openai: std::env::var("OPENAI_API_KEY").ok(),
            openrouter: std::env::var("OPENROUTER_API_KEY").ok(),
            groq: std::env::var("GROQ_API_KEY").ok(),
            nous: std::env::var("NOUS_API_KEY").ok(),
            llama: std::env::var("LLAMA_API_KEY").ok(),
            together: std::env::var("TOGETHER_API_KEY").ok(),
        }
    }
}

fn config_path() -> PathBuf {
    if let Ok(path) = std::env::var("RASKOLNIKOV_CONFIG") {
        return PathBuf::from(path);
    }
    if let Some(home) = home_dir() {
        return home.join(".config/raskolnikov/config.toml");
    }
    PathBuf::from("/etc/raskolnikov/config.toml")
}

pub fn data_dir() -> PathBuf {
    if let Ok(path) = std::env::var("RASKOLNIKOV_DATA") {
        return PathBuf::from(path);
    }
    if let Some(home) = home_dir() {
        return home.join(".local/share/raskolnikov");
    }
    PathBuf::from("/var/lib/raskolnikov")
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

pub fn load() -> Result<Config> {
    let path = config_path();

    if !path.exists() {
        eprintln!("No config found at {}. Using defaults.", path.display());
        return Ok(Config::default());
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read config at {}: {}", path.display(), e))?;

    let config: Config = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse config at {}: {}", path.display(), e))?;

    Ok(config)
}

pub fn save(config: &Config) -> Result<()> {
    let path = config_path();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let content =
        toml::to_string_pretty(config).map_err(|e| format!("Failed to serialize config: {}", e))?;

    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write config to {}: {}", path.display(), e))?;

    Ok(())
}

pub fn init_data_dirs() -> Result<PathBuf> {
    let dir = data_dir();

    let sessions_dir = dir.join("sessions");
    std::fs::create_dir_all(&sessions_dir)
        .map_err(|e| format!("Failed to create sessions directory: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(&dir) {
            let perm = metadata.permissions();
            if perm.mode() & 0o077 != 0 {
                std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).ok();
            }
        }
    }

    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.ai.provider, "ollama");
        assert_eq!(config.ai.model, "qwen3");
        assert_eq!(config.ollama.host, "http://localhost:11434");
        assert!(!config.tools.prefer_ffuf);
        assert_eq!(config.tools.nmap_timing, 4);
    }

    #[test]
    fn test_api_keys_from_env() {
        temp_env::with_vars(
            vec![
                ("ANTHROPIC_API_KEY", Some("sk-ant-test")),
                ("OPENAI_API_KEY", Some("sk-test")),
                ("GROQ_API_KEY", Some("gsk_test")),
            ],
            || {
                let keys = ApiKeys::from_env();
                assert_eq!(keys.anthropic, Some("sk-ant-test".to_string()));
                assert_eq!(keys.openai, Some("sk-test".to_string()));
                assert_eq!(keys.groq, Some("gsk_test".to_string()));
                assert!(keys.openrouter.is_none());
                assert!(keys.nous.is_none());
            },
        );
    }

    #[test]
    fn test_config_parse() {
        let toml_str = r#"
[ai]
provider = "anthropic"
model = "claude-sonnet-4-6"

[tools]
prefer_ffuf = true
nmap_timing = 3
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.ai.provider, "anthropic");
        assert_eq!(config.ai.model, "claude-sonnet-4-6");
        assert!(config.tools.prefer_ffuf);
        assert_eq!(config.tools.nmap_timing, 3);
    }
}
