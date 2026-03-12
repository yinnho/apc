//! Configuration management for apc

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// apc configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Registry configuration
    pub registry: RegistryConfig,

    /// Certificate paths
    pub certificates: Option<CertificateConfig>,

    /// Connection settings
    pub connection: Option<ConnectionConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            registry: RegistryConfig::default(),
            certificates: None,
            connection: None,
        }
    }
}

/// Registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Primary registry URL
    pub primary: String,

    /// Mirror URLs (fallback order)
    #[serde(default)]
    pub mirrors: Vec<String>,

    /// Fallback to primary if mirrors fail
    #[serde(default = "default_true")]
    pub fallback_to_primary: bool,

    /// Verify registry signature
    #[serde(default = "default_true")]
    pub verify_signature: bool,

    /// Local cache TTL
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: String,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            primary: "agent://registry.agent-protocol.org:86".to_string(),
            mirrors: vec![],
            fallback_to_primary: true,
            verify_signature: true,
            cache_ttl: "1h".to_string(),
        }
    }
}

/// Certificate configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CertificateConfig {
    /// Agent certificate path
    #[serde(default)]
    pub cert: PathBuf,

    /// Agent private key path
    #[serde(default)]
    pub key: PathBuf,

    /// CA certificate path
    #[serde(default)]
    pub ca: PathBuf,

    /// CRL path
    pub crl: Option<PathBuf>,
}

/// Connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Connection timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Read timeout in seconds
    #[serde(default = "default_timeout")]
    pub read_timeout: u64,

    /// Write timeout in seconds
    #[serde(default = "default_timeout")]
    pub write_timeout: u64,

    /// Maximum retries
    #[serde(default)]
    pub max_retries: u32,

    /// Retry delay (exponential backoff base)
    #[serde(default = "default_retry_delay")]
    pub retry_delay_ms: u64,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            timeout: 30,
            read_timeout: 30,
            write_timeout: 30,
            max_retries: 3,
            retry_delay_ms: 100,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_cache_ttl() -> String {
    "1h".to_string()
}

fn default_timeout() -> u64 {
    30
}

fn default_retry_delay() -> u64 {
    100
}

/// Load configuration from file
pub fn load_config() -> crate::error::Result<Config> {
    // Try multiple config locations
    let config_paths = [
        // Current directory
        PathBuf::from(".apc.toml"),
        PathBuf::from("apc.toml"),
        // Config directory
        dirs::config_dir()
            .map(|p| p.join("apc").join("config.toml"))
            .unwrap_or_default(),
        // Home directory (legacy)
        dirs::home_dir()
            .map(|p| p.join(".apc.toml"))
            .unwrap_or_default(),
    ];

    for path in &config_paths {
        if path.exists() {
            match shared::util::config::load_toml_config(path) {
                Ok(config) => {
                    tracing::debug!("Loaded config from {:?}", path);
                    return Ok(config);
                }
                Err(e) => {
                    tracing::warn!("Failed to load config from {:?}: {}", path, e);
                }
            }
        }
    }

    // Return default config if no file found
    tracing::debug!("Using default configuration");
    Ok(Config::default())
}

/// Load configuration and apply CLI overrides
pub fn load_config_with_args(args: &crate::cli::Args) -> crate::error::Result<Config> {
    let mut config = load_config()?;

    // Override with CLI args
    if let Some(ref cert) = args.cert {
        let cert_config = config.certificates.get_or_insert_with(Default::default);
        cert_config.cert = PathBuf::from(cert);
    }

    if let Some(ref key) = args.key {
        let cert_config = config.certificates.get_or_insert_with(Default::default);
        cert_config.key = PathBuf::from(key);
    }

    if let Some(ref ca) = args.ca {
        let cert_config = config.certificates.get_or_insert_with(Default::default);
        cert_config.ca = PathBuf::from(ca);
    }

    // Set connection timeout from args
    let conn_config = config.connection.get_or_insert_with(Default::default);
    conn_config.timeout = args.timeout;

    Ok(config)
}

/// Save configuration to file
pub fn save_config(config: &Config, path: &PathBuf) -> crate::error::Result<()> {
    shared::util::config::save_toml_config(path, config)
        .map_err(|e| crate::error::Error::Config(e.to_string()))
}
