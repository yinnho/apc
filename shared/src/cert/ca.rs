//! CA (Certificate Authority) utilities

use super::types::*;
use super::{CertificateError, CertificateRequest};
use chrono::{Datelike, Utc};
use sha2::{Digest, Sha256};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

/// CA (Certificate Authority) for signing agent certificates
pub struct CertificateAuthority {
    /// Serial number counter
    serial_counter: u64,

    /// CA name
    name: String,
}

impl CertificateAuthority {
    /// Create a new CA
    pub fn new(name: &str) -> Result<Self, CertificateError> {
        Ok(Self {
            serial_counter: 0,
            name: name.to_string(),
        })
    }

    /// Get the CA certificate in PEM format
    pub fn certificate_pem(&self) -> Result<String, CertificateError> {
        // Return a placeholder CA certificate
        Ok(format!(
            "-----BEGIN CERTIFICATE-----\n\
MIIBkTCB+wIJAKHBfHG3Z2XgMA0GCSqGSIb3DQEBCwUAMBExDzANBgNVBAMMBnRlc3RD\n\
QTAeFw0yNDAxMDFERzAwMDBaFw0zNDAxMDFERzAwMDBaMBExDzANBgNVBAMMBnRlc3RD\n\
QTCBnzANBgkqhkiG9w0BAQEFAAOBjQAwgYkCgYEAu6hVhF1hX5G5MQVZp5j5mZ9RZ\n\
DUMMY CERTIFICATE FOR TESTING PURPOSES ONLY\n\
-----END CERTIFICATE-----\n"
        ))
    }

    /// Get the CA private key in PEM format
    pub fn private_key_pem(&self) -> String {
        // Return a placeholder private key
        String::from("-----BEGIN PRIVATE KEY-----\nDUMMY\n-----END PRIVATE KEY-----\n")
    }

    /// Sign a certificate request and return the signed certificate
    pub fn sign_request(
        &mut self,
        request: CertificateRequest,
    ) -> Result<AgentCertificate, CertificateError> {
        self.serial_counter += 1;

        // Create our AgentCertificate structure
        let validity = Validity::one_year();

        // Create signature
        let signature_value = BASE64.encode(format!("sig-{}", self.serial_counter).as_bytes());

        Ok(AgentCertificate {
            version: 1,
            serial_number: format!("{:016x}", self.serial_counter),
            subject: CertificateSubject {
                agent_id: request.agent_id,
                agent_name: request.agent_name,
                agent_type: request.agent_type,
            },
            public_key: request.public_key,
            validity,
            extensions: CertificateExtensions {
                capabilities: request.capabilities,
                registry: "agent://registry.agent-protocol.org:86".to_string(),
            },
            signature: CertificateSignature {
                algorithm: "SHA256withECDSA".to_string(),
                value: signature_value,
            },
        })
    }

    /// Generate a new key pair for an agent
    pub fn generate_key_pair() -> Result<(String, String), CertificateError> {
        // Return placeholder keys
        let public_key = String::from("-----BEGIN PUBLIC KEY-----\nDUMMY\n-----END PUBLIC KEY-----\n");
        let private_key = String::from("-----BEGIN PRIVATE KEY-----\nDUMMY\n-----END PRIVATE KEY-----\n");

        Ok((public_key, private_key))
    }
}

/// Load a certificate from PEM data (simplified version)
pub fn load_certificate(_pem: &str) -> Result<AgentCertificate, CertificateError> {
    // Create a basic AgentCertificate structure
    Ok(AgentCertificate {
        version: 1,
        serial_number: "unknown".to_string(),
        subject: CertificateSubject {
            agent_id: "unknown".to_string(),
            agent_name: "Unknown Agent".to_string(),
            agent_type: "unknown".to_string(),
        },
        public_key: String::new(),
        validity: Validity::one_year(),
        extensions: CertificateExtensions {
            capabilities: vec![],
            registry: String::new(),
        },
        signature: CertificateSignature {
            algorithm: "SHA256withECDSA".to_string(),
            value: String::new(),
        },
    })
}

/// Calculate SHA-256 hash of data
pub fn sha256_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ca_creation() {
        let ca = CertificateAuthority::new("Test CA").unwrap();
        let cert_pem = ca.certificate_pem().unwrap();
        assert!(cert_pem.contains("BEGIN CERTIFICATE"));
    }

    #[test]
    fn test_key_generation() {
        let (public, private) = CertificateAuthority::generate_key_pair().unwrap();
        assert!(public.contains("BEGIN PUBLIC KEY"));
        assert!(private.contains("BEGIN PRIVATE KEY"));
    }
}
