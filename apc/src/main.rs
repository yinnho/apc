//! apc - Agent Protocol client CLI
//!
//! `apc` is to agents what `curl` is to HTTP.
//!
//! Usage:
//!   apc agent://hotel.example.com "查询房间"
//!   apc --find "type=hotel AND location=Beijing"
//!   apc agent://local/claude "hello"

mod cli;
mod url;
mod protocol;
mod client;
mod cert;
mod registry;
mod config;
mod output;
mod error;

use std::process::ExitCode;

use crate::cli::Args;
use crate::client::Client;
use crate::output::OutputFormatter;

#[tokio::main]
async fn main() -> ExitCode {
    let args = cli::parse_args();

    // Initialize logging
    shared::util::logging::init_logging(args.verbose, args.debug);

    // Run the main logic
    match run(args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::from(1)
        }
    }
}

async fn run(args: Args) -> crate::error::Result<()> {
    // Load configuration
    let config = config::load_config_with_args(&args)?;

    // Handle different modes
    match &args.command {
        Some(cli::Command::Find { query }) => {
            // Query registry
            let registry_client = registry::RegistryClient::new(&config)?;
            let results = registry_client.find(query).await?;

            let formatter = OutputFormatter::new(args.output_format);
            formatter.print_find_results(&results);
        }

        Some(cli::Command::Capa { url }) => {
            // Query capabilities
            let url = shared::url::AgentUrl::parse(url)?;
            let mut client = Client::new(&config)?;
            let capa = client.query_capabilities(&url).await?;

            let formatter = OutputFormatter::new(args.output_format);
            formatter.print_capabilities(&capa);
        }

        Some(cli::Command::Register { url, cert_path, key_path }) => {
            // Register with registry
            let registry_client = registry::RegistryClient::new(&config)?;
            registry_client.register(url, cert_path, key_path).await?;
            println!("Registration successful");
        }

        Some(cli::Command::Cert { command }) => {
            // Certificate management
            cert::handle_cert_command(command, &config)?;
        }

        None => {
            // Send message to agent
            let url_str = args.url.as_ref().ok_or_else(|| {
                crate::error::Error::Url("URL is required when not using subcommands".to_string())
            })?;
            let url = shared::url::AgentUrl::parse(url_str)?;
            let message = args.message.clone().unwrap_or_default();

            let mut client = Client::new(&config)?;
            let response = client.send(&url, &message).await?;

            let formatter = OutputFormatter::new(args.output_format);
            formatter.print_response(&response);
        }
    }

    Ok(())
}
