//! Agent Card types for Agent Protocol
//!
//! Agent Card describes an agent's capabilities, security requirements, and metadata.
//! Based on Google A2A specification.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent Card - describes an agent's capabilities and configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentCard {
    /// Human-readable name of the agent
    pub name: String,

    /// Human-readable description of the agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Agent URL (e.g., "agent://hotel.example.com:86")
    pub url: String,

    /// Agent version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Agent type (e.g., "hotel", "assistant", "tool")
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub agent_type: Option<String>,

    /// Capabilities provided by this agent
    #[serde(default)]
    pub capabilities: Vec<CapabilityDefinition>,

    /// Security schemes supported by this agent
    #[serde(rename = "securitySchemes", default, skip_serializing_if = "HashMap::is_empty")]
    pub security_schemes: HashMap<String, SecurityScheme>,

    /// Security requirements (references to security schemes)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub security: Vec<SecurityRequirement>,

    /// Provider information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<Provider>,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl AgentCard {
    /// Create a new agent card with minimal required fields
    pub fn new(name: &str, url: &str, capabilities: Vec<CapabilityDefinition>) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            url: url.to_string(),
            version: None,
            agent_type: None,
            capabilities,
            security_schemes: HashMap::new(),
            security: Vec::new(),
            provider: None,
            metadata: None,
        }
    }

    /// Add a capability
    pub fn with_capability(mut self, capability: CapabilityDefinition) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Add a security scheme
    pub fn with_security_scheme(mut self, name: &str, scheme: SecurityScheme) -> Self {
        self.security_schemes.insert(name.to_string(), scheme);
        self
    }

    /// Add a security requirement
    pub fn with_security_requirement(mut self, requirement: SecurityRequirement) -> Self {
        self.security.push(requirement);
        self
    }

    /// Set provider
    pub fn with_provider(mut self, provider: Provider) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Parse from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Capability definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityDefinition {
    /// Unique capability identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Description of the capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Input schema (JSON Schema)
    #[serde(rename = "inputSchema", skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,

    /// Output schema (JSON Schema)
    #[serde(rename = "outputSchema", skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,
}

impl CapabilityDefinition {
    /// Create a new capability
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: None,
            input_schema: None,
            output_schema: None,
        }
    }

    /// Add description
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Add input schema
    pub fn with_input_schema(mut self, schema: serde_json::Value) -> Self {
        self.input_schema = Some(schema);
        self
    }

    /// Add output schema
    pub fn with_output_schema(mut self, schema: serde_json::Value) -> Self {
        self.output_schema = Some(schema);
        self
    }
}

/// Security scheme definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SecurityScheme {
    /// Mutual TLS authentication
    #[serde(rename = "mutualTLS")]
    MutualTLS {
        /// Description
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },

    /// API Key authentication
    #[serde(rename = "apiKey")]
    ApiKey {
        /// Where to put the key (header, query, cookie)
        #[serde(rename = "in")]
        location: String,

        /// Name of the header/query parameter/cookie
        name: String,

        /// Description
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },

    /// OAuth2 authentication
    #[serde(rename = "oauth2")]
    OAuth2 {
        /// OAuth2 flows
        flows: OAuth2Flows,

        /// Description
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },

    /// HTTP authentication (Basic, Bearer, etc.)
    #[serde(rename = "http")]
    Http {
        /// Scheme (basic, bearer, etc.)
        scheme: String,

        /// Bearer format (if scheme is bearer)
        #[serde(rename = "bearerFormat", skip_serializing_if = "Option::is_none")]
        bearer_format: Option<String>,

        /// Description
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
}

impl SecurityScheme {
    /// Create a mutual TLS scheme
    pub fn mutual_tls() -> Self {
        Self::MutualTLS { description: None }
    }

    /// Create an API key scheme
    pub fn api_key(location: &str, name: &str) -> Self {
        Self::ApiKey {
            location: location.to_string(),
            name: name.to_string(),
            description: None,
        }
    }

    /// Create a bearer token scheme
    pub fn bearer() -> Self {
        Self::Http {
            scheme: "bearer".to_string(),
            bearer_format: Some("JWT".to_string()),
            description: None,
        }
    }
}

/// OAuth2 flows configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OAuth2Flows {
    /// Authorization code flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_code: Option<OAuth2Flow>,

    /// Client credentials flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_credentials: Option<OAuth2Flow>,

    /// Implicit flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implicit: Option<ImplicitFlow>,

    /// Resource owner password flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<OAuth2Flow>,
}

/// OAuth2 flow configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OAuth2Flow {
    /// Authorization URL (for authorization code and implicit)
    #[serde(rename = "authorizationUrl", skip_serializing_if = "Option::is_none")]
    pub authorization_url: Option<String>,

    /// Token URL
    #[serde(rename = "tokenUrl", skip_serializing_if = "Option::is_none")]
    pub token_url: Option<String>,

    /// Refresh URL
    #[serde(rename = "refreshUrl", skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<String>,

    /// Required scopes
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<String>,
}

/// Implicit flow configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImplicitFlow {
    /// Authorization URL
    #[serde(rename = "authorizationUrl")]
    pub authorization_url: String,

    /// Refresh URL
    #[serde(rename = "refreshUrl", skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<String>,

    /// Required scopes
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<String>,
}

/// Security requirement - references a security scheme
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityRequirement {
    /// Map of scheme name to required scopes
    #[serde(flatten)]
    pub requirements: HashMap<String, Vec<String>>,
}

impl SecurityRequirement {
    /// Create a new security requirement
    pub fn new(scheme_name: &str, scopes: Vec<String>) -> Self {
        let mut requirements = HashMap::new();
        requirements.insert(scheme_name.to_string(), scopes);
        Self { requirements }
    }

    /// Create with no scopes
    pub fn no_scopes(scheme_name: &str) -> Self {
        Self::new(scheme_name, Vec::new())
    }
}

/// Provider information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provider {
    /// Organization name
    pub organization: String,

    /// URL to organization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Contact email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

impl Provider {
    /// Create a new provider
    pub fn new(organization: &str) -> Self {
        Self {
            organization: organization.to_string(),
            url: None,
            email: None,
        }
    }

    /// Add URL
    pub fn with_url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }

    /// Add email
    pub fn with_email(mut self, email: &str) -> Self {
        self.email = Some(email.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_agent_card_serialization() {
        let card = AgentCard::new(
            "Hotel Agent",
            "agent://hotel.example.com:86",
            vec![CapabilityDefinition::new("booking", "Room Booking")],
        );

        let json = card.to_json();
        assert!(json.contains(r#""name":"Hotel Agent""#));
        assert!(json.contains(r#""url":"agent://hotel.example.com:86""#));
    }

    #[test]
    fn test_agent_card_with_security() {
        let card = AgentCard::new(
            "Secure Agent",
            "agent://secure.example.com:86",
            vec![],
        )
        .with_security_scheme("mtls", SecurityScheme::mutual_tls())
        .with_security_requirement(SecurityRequirement::no_scopes("mtls"));

        let json = card.to_json();
        assert!(json.contains(r#""mutualTLS""#));
    }

    #[test]
    fn test_capability_definition() {
        let cap = CapabilityDefinition::new("query", "Query Rooms")
            .with_description("Query available rooms")
            .with_input_schema(json!({
                "type": "object",
                "properties": {
                    "check_in": {"type": "string"},
                    "check_out": {"type": "string"}
                }
            }));

        assert_eq!(cap.id, "query");
        assert_eq!(cap.name, "Query Rooms");
        assert!(cap.description.is_some());
        assert!(cap.input_schema.is_some());
    }

    #[test]
    fn test_security_schemes() {
        let mtls = SecurityScheme::mutual_tls();
        let api_key = SecurityScheme::api_key("header", "X-API-Key");
        let bearer = SecurityScheme::bearer();

        // Verify serialization
        let mtls_json = serde_json::to_string(&mtls).unwrap();
        assert!(mtls_json.contains(r#""type":"mutualTLS""#));

        let api_key_json = serde_json::to_string(&api_key).unwrap();
        assert!(api_key_json.contains(r#""type":"apiKey""#));

        let bearer_json = serde_json::to_string(&bearer).unwrap();
        assert!(bearer_json.contains(r#""type":"http""#));
        assert!(bearer_json.contains(r#""scheme":"bearer""#));
    }

    #[test]
    fn test_deserialization() {
        let json = r#"{
            "name": "Test Agent",
            "url": "agent://test.example.com:86",
            "capabilities": [
                {"id": "echo", "name": "Echo"}
            ]
        }"#;

        let card: AgentCard = serde_json::from_str(json).unwrap();
        assert_eq!(card.name, "Test Agent");
        assert_eq!(card.capabilities.len(), 1);
    }
}
