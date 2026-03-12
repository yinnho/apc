//! Certificate verification utilities

use super::{AgentCertificate, CertificateError, CertificatePaths};
use std::path::Path;
use base64::Engine;

/// Certificate verifier
pub struct CertificateVerifier {
    /// CA certificate (PEM)
    ca_cert_pem: Option<String>,

    /// Revoked certificate serials
    revoked_serials: Vec<String>,
}

impl CertificateVerifier {
    /// Create a new verifier
    pub fn new() -> Self {
        Self {
            ca_cert_pem: None,
            revoked_serials: Vec::new(),
        }
    }

    /// Load CA certificate from file
    pub fn load_ca<P: AsRef<Path>>(&mut self, path: P) -> Result<(), CertificateError> {
        let pem = std::fs::read_to_string(path)?;
        self.ca_cert_pem = Some(pem);
        Ok(())
    }

    /// Load CRL (Certificate Revocation List)
    pub fn load_crl<P: AsRef<Path>>(&mut self, path: P) -> Result<(), CertificateError> {
        let content = std::fs::read_to_string(path)?;
        // Parse revoked serials (simple format, one per line)
        self.revoked_serials = content
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.trim().to_string())
            .collect();
        Ok(())
    }

    /// Verify a certificate
    pub fn verify(&self, cert: &AgentCertificate) -> Result<(), CertificateError> {
        // Check validity period
        if !cert.validity.is_valid() {
            if chrono::Utc::now() < cert.validity.not_before {
                return Err(CertificateError::NotYetValid);
            } else {
                return Err(CertificateError::Expired);
            }
        }

        // Check revocation
        if self.revoked_serials.contains(&cert.serial_number) {
            return Err(CertificateError::Revoked);
        }

        // In a real implementation, we would also verify:
        // 1. CA signature
        // 2. Chain of trust
        // 3. Agent registration status

        Ok(())
    }

    /// Check if a certificate is expired
    pub fn is_expired(cert: &AgentCertificate) -> bool {
        chrono::Utc::now() > cert.validity.not_after
    }

    /// Check if a certificate is not yet valid
    pub fn is_not_yet_valid(cert: &AgentCertificate) -> bool {
        chrono::Utc::now() < cert.validity.not_before
    }
}

impl Default for CertificateVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Load certificate from default paths
pub fn load_certificate_from_paths(paths: &CertificatePaths) -> Result<AgentCertificate, CertificateError> {
    let pem = std::fs::read_to_string(&paths.cert)?;
    super::load_certificate(&pem)
}

/// Load private key from file
pub fn load_private_key<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, CertificateError> {
    let pem = std::fs::read_to_string(path)?;

    // Extract the base64 content from PEM
    let lines: Vec<&str> = pem
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect();

    let base64_content = lines.join("");
    let der = base64::engine::general_purpose::STANDARD
        .decode(&base64_content)
        .map_err(|e| CertificateError::ParsingFailed(e.to_string()))?;

    Ok(der)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    fn create_test_cert() -> AgentCertificate {
        AgentCertificate {
            version: 1,
            serial_number: "0000000000000001".to_string(),
            subject: super::super::CertificateSubject {
                agent_id: "test".to_string(),
                agent_name: "Test Agent".to_string(),
                agent_type: "test".to_string(),
            },
            public_key: String::new(),
            validity: super::super::Validity {
                not_before: Utc::now() - Duration::days(1),
                not_after: Utc::now() + Duration::days(365),
            },
            extensions: super::super::CertificateExtensions {
                capabilities: vec![],
                registry: String::new(),
            },
            signature: super::super::CertificateSignature {
                algorithm: "SHA256withECDSA".to_string(),
                value: String::new(),
            },
        }
    }

    #[test]
    fn test_verify_valid_cert() {
        let verifier = CertificateVerifier::new();
        let cert = create_test_cert();
        assert!(verifier.verify(&cert).is_ok());
    }

    #[test]
    fn test_verify_revoked_cert() {
        let mut verifier = CertificateVerifier::new();
        let cert = create_test_cert();
        verifier.revoked_serials.push(cert.serial_number.clone());
        assert!(matches!(
            verifier.verify(&cert),
            Err(CertificateError::Revoked)
        ));
    }
}
