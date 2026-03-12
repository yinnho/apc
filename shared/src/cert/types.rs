//! Certificate type definitions

use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

/// Agent certificate (simplified format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCertificate {
    /// Certificate version
    pub version: u32,

    /// Serial number
    pub serial_number: String,

    /// Subject (agent identity)
    pub subject: CertificateSubject,

    /// Public key (PEM format)
    #[serde(skip_serializing_if = "String::is_empty")]
    pub public_key: String,

    /// Validity period
    pub validity: Validity,

    /// Extensions
    pub extensions: CertificateExtensions,

    /// Signature
    pub signature: CertificateSignature,
}

/// Certificate subject
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateSubject {
    /// Agent ID
    pub agent_id: String,

    /// Agent name
    pub agent_name: String,

    /// Agent type
    pub agent_type: String,
}

/// Certificate validity period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validity {
    /// Not valid before
    pub not_before: DateTime<Utc>,

    /// Not valid after
    pub not_after: DateTime<Utc>,
}

impl Validity {
    /// Check if the certificate is currently valid
    pub fn is_valid(&self) -> bool {
        let now = Utc::now();
        now >= self.not_before && now <= self.not_after
    }

    /// Create a one year validity period
    pub fn one_year() -> Self {
        let now = Utc::now();
        Self {
            not_before: now,
            not_after: now + Duration::days(365),
        }
    }
}

/// Certificate extensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateExtensions {
    /// Allowed capabilities
    #[serde(default)]
    pub capabilities: Vec<String>,

    /// Registry URL
    pub registry: String,
}

/// Certificate signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateSignature {
    /// Signature algorithm
    pub algorithm: String,

    /// Signature value (base64)
    pub value: String,
}

/// Certificate request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateRequest {
    /// Agent ID
    pub agent_id: String,

    /// Agent name
    pub agent_name: String,

    /// Agent type
    pub agent_type: String,

    /// Public key (PEM format)
    pub public_key: String,

    /// Requested capabilities
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// Certificate store paths
#[derive(Debug, Clone)]
pub struct CertificatePaths {
    /// Agent certificate path
    pub cert: std::path::PathBuf,

    /// Agent private key path
    pub key: std::path::PathBuf,

    /// CA certificate path
    pub ca: std::path::PathBuf,

    /// CRL (Certificate Revocation List) path
    pub crl: std::path::PathBuf,
}

impl Default for CertificatePaths {
    fn default() -> Self {
        let base = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".agent")
            .join("certs");

        Self {
            cert: base.join("agent.crt"),
            key: base.join("agent.key"),
            ca: base.join("ca.crt"),
            crl: base.join("crl.pem"),
        }
    }
}

impl CertificatePaths {
    /// Create certificate paths from a base directory
    pub fn from_base(base: std::path::PathBuf) -> Self {
        Self {
            cert: base.join("agent.crt"),
            key: base.join("agent.key"),
            ca: base.join("ca.crt"),
            crl: base.join("crl.pem"),
        }
    }

    /// Ensure the certificate directory exists
    pub fn ensure_dir(&self) -> std::io::Result<()> {
        if let Some(parent) = self.cert.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(())
    }
}
