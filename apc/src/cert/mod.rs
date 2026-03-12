//! Certificate management for apc

use std::path::PathBuf;

use crate::cli::CertCommand;
use crate::config::Config;
use crate::error::Result;

/// Handle certificate commands
pub fn handle_cert_command(command: &CertCommand, config: &Config) -> Result<()> {
    match command {
        CertCommand::Generate { id, name, agent_type, output } => {
            generate_key_pair(id, name, agent_type, output)
        }
        CertCommand::Request { id, name, agent_type, registry } => {
            request_certificate(id, name, agent_type, registry)
        }
        CertCommand::Verify { cert_path } => {
            verify_certificate(cert_path)
        }
        CertCommand::Info { cert_path } => {
            show_certificate_info(cert_path)
        }
    }
}

/// Generate a new key pair
fn generate_key_pair(id: &str, name: &str, _agent_type: &str, output: &str) -> Result<()> {
    let output_dir = shellexpand::tilde(output).to_string();
    let output_path = PathBuf::from(output_dir);

    // Ensure output directory exists
    std::fs::create_dir_all(&output_path)?;

    // Generate key pair
    let (public_key, private_key) = shared::cert::CertificateAuthority::generate_key_pair()?;

    // Save keys
    let key_path = output_path.join(format!("{}.key", id));
    let pub_path = output_path.join(format!("{}.pub", id));

    std::fs::write(&key_path, &private_key)?;
    std::fs::write(&pub_path, &public_key)?;

    println!("Generated key pair for agent '{}':", name);
    println!("  Private key: {}", key_path.display());
    println!("  Public key:  {}", pub_path.display());
    println!();
    println!("WARNING: Keep your private key secure!");

    Ok(())
}

/// Request a certificate from the CA
fn request_certificate(id: &str, name: &str, agent_type: &str, _registry: &str) -> Result<()> {
    println!("Certificate request for agent '{}':", name);
    println!("  ID: {}", id);
    println!("  Type: {}", agent_type);
    println!();
    println!("To request a certificate:");
    println!("  1. Generate a key pair: apc cert generate --id {} --name '{}'", id, name);
    println!("  2. Submit the public key to the CA");
    println!("  3. Save the signed certificate to ~/.agent/certs/agent.crt");

    Ok(())
}

/// Verify a certificate
fn verify_certificate(cert_path: &str) -> Result<()> {
    let pem = std::fs::read_to_string(cert_path)?;
    let cert = shared::cert::load_certificate(&pem)?;

    let verifier = shared::cert::CertificateVerifier::new();
    match verifier.verify(&cert) {
        Ok(()) => {
            println!("Certificate is valid");
            println!("  Agent ID: {}", cert.subject.agent_id);
            println!("  Valid from: {}", cert.validity.not_before);
            println!("  Valid until: {}", cert.validity.not_after);
        }
        Err(e) => {
            println!("Certificate verification failed: {}", e);
        }
    }

    Ok(())
}

/// Show certificate information
fn show_certificate_info(cert_path: &str) -> Result<()> {
    let pem = std::fs::read_to_string(cert_path)?;
    let cert = shared::cert::load_certificate(&pem)?;

    println!("Certificate Information:");
    println!("  Version: {}", cert.version);
    println!("  Serial Number: {}", cert.serial_number);
    println!();
    println!("Subject:");
    println!("  Agent ID: {}", cert.subject.agent_id);
    println!("  Agent Name: {}", cert.subject.agent_name);
    println!("  Agent Type: {}", cert.subject.agent_type);
    println!();
    println!("Validity:");
    println!("  Not Before: {}", cert.validity.not_before);
    println!("  Not After: {}", cert.validity.not_after);

    if !cert.extensions.capabilities.is_empty() {
        println!();
        println!("Capabilities:");
        for cap in &cert.extensions.capabilities {
            println!("  - {}", cap);
        }
    }

    println!();
    println!("Signature:");
    println!("  Algorithm: {}", cert.signature.algorithm);

    Ok(())
}
