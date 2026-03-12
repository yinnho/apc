//! CLI argument parsing

mod args;

pub use args::*;

use clap::{Parser, Subcommand};

/// apc - Agent Protocol client CLI
///
/// `apc` is to agents what `curl` is to HTTP.
///
/// Examples:
///   apc agent://hotel.example.com "查询房间"
///   apc find "type=hotel AND location=Beijing"
///   apc agent://local/claude "hello"
///   apc capa agent://hotel.example.com
#[derive(Parser, Debug)]
#[command(name = "apc")]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Agent URL (e.g., agent://hotel.example.com/booking)
    #[arg(value_name = "URL")]
    pub url: Option<String>,

    /// Message to send to the agent
    #[arg(value_name = "MESSAGE")]
    pub message: Option<String>,

    /// Output format: text, json
    #[arg(short = 'o', long, default_value = "text")]
    pub output_format: OutputFormat,

    /// Certificate file path
    #[arg(short = 'c', long, value_name = "FILE")]
    pub cert: Option<String>,

    /// Private key file path
    #[arg(short = 'k', long, value_name = "FILE")]
    pub key: Option<String>,

    /// CA certificate file path
    #[arg(long, value_name = "FILE")]
    pub ca: Option<String>,

    /// Connection timeout in seconds
    #[arg(short = 't', long, default_value = "30")]
    pub timeout: u64,

    /// Verbose output
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Debug output
    #[arg(short = 'd', long)]
    pub debug: bool,

    /// Subcommand
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Output format
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

/// Subcommands
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Find agents in registry
    #[command(name = "find")]
    Find {
        /// Query string (e.g., "type=hotel AND location=Beijing")
        #[arg(value_name = "QUERY")]
        query: String,
    },

    /// Query agent capabilities
    #[command(name = "capa")]
    Capa {
        /// Agent URL
        #[arg(value_name = "URL")]
        url: String,
    },

    /// Register with registry
    #[command(name = "register")]
    Register {
        /// Agent URL
        #[arg(value_name = "URL")]
        url: String,

        /// Certificate file path
        #[arg(short = 'c', long, value_name = "FILE")]
        cert_path: String,

        /// Private key file path
        #[arg(short = 'k', long, value_name = "FILE")]
        key_path: String,
    },

    /// Certificate management
    #[command(name = "cert")]
    Cert {
        #[command(subcommand)]
        command: CertCommand,
    },
}

/// Certificate management commands
#[derive(Debug, Subcommand)]
pub enum CertCommand {
    /// Generate a new key pair
    #[command(name = "generate")]
    Generate {
        /// Agent ID
        #[arg(long)]
        id: String,

        /// Agent name
        #[arg(long)]
        name: String,

        /// Agent type
        #[arg(long)]
        agent_type: String,

        /// Output directory
        #[arg(short = 'o', long, default_value = "~/.agent/certs")]
        output: String,
    },

    /// Request a certificate from CA
    #[command(name = "request")]
    Request {
        /// Agent ID
        #[arg(long)]
        id: String,

        /// Agent name
        #[arg(long)]
        name: String,

        /// Agent type
        #[arg(long)]
        agent_type: String,

        /// Registry URL
        #[arg(long, default_value = "agent://registry.agent-protocol.org:86")]
        registry: String,
    },

    /// Verify a certificate
    #[command(name = "verify")]
    Verify {
        /// Certificate file path
        #[arg(value_name = "FILE")]
        cert_path: String,
    },

    /// Show certificate info
    #[command(name = "info")]
    Info {
        /// Certificate file path
        #[arg(value_name = "FILE")]
        cert_path: String,
    },
}

/// Parse command line arguments
pub fn parse_args() -> Args {
    Args::parse()
}
