//! URL parser implementation

use super::{AgentUrl, UrlError};

/// Parse an agent URL string
pub fn parse_agent_url(url: &str) -> Result<AgentUrl, UrlError> {
    let url = url.trim();

    // Check scheme
    if !url.starts_with("agent://") {
        return Err(UrlError::InvalidScheme(
            url.split(':').next().unwrap_or("").to_string(),
        ));
    }

    let rest = &url["agent://".len()..];

    // Handle query string
    let (path_part, query) = if let Some(idx) = rest.find('?') {
        (&rest[..idx], Some(rest[idx + 1..].to_string()))
    } else {
        (rest, None)
    };

    // Find the first '/' to separate host:port from agent
    let slash_idx = path_part
        .find('/')
        .ok_or_else(|| UrlError::MissingAgent)?;

    let host_port = &path_part[..slash_idx];
    let agent = &path_part[slash_idx + 1..];

    if agent.is_empty() {
        return Err(UrlError::MissingAgent);
    }

    // Parse host and port
    let (host, port) = if let Some(colon_idx) = host_port.rfind(':') {
        // Check if this is an IPv6 address
        if host_port.starts_with('[') {
            // IPv6 address: [::1]:port
            if let Some(bracket_idx) = host_port.find(']') {
                let host = &host_port[..=bracket_idx];
                if colon_idx > bracket_idx {
                    let port_str = &host_port[colon_idx + 1..];
                    let port: u16 = port_str
                        .parse()
                        .map_err(|_| UrlError::InvalidPort(port_str.to_string()))?;
                    (host.to_string(), Some(port))
                } else {
                    (host.to_string(), None)
                }
            } else {
                (host_port.to_string(), None)
            }
        } else {
            // IPv4 or hostname: port
            let host = &host_port[..colon_idx];
            let port_str = &host_port[colon_idx + 1..];
            let port: u16 = port_str
                .parse()
                .map_err(|_| UrlError::InvalidPort(port_str.to_string()))?;
            (host.to_string(), Some(port))
        }
    } else {
        (host_port.to_string(), None)
    };

    if host.is_empty() {
        return Err(UrlError::MissingHost);
    }

    // Decode URL-encoded agent name
    let decoded_agent = urlencoding::decode(agent)
        .map_err(|e| UrlError::ParseError(e.to_string()))?
        .into_owned();

    Ok(AgentUrl {
        host,
        port: port.unwrap_or(AgentUrl::DEFAULT_PORT),
        agent: decoded_agent,
        query,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_url() {
        let url = parse_agent_url("agent://example.com/test").unwrap();
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, 86);
        assert_eq!(url.agent, "test");
    }

    #[test]
    fn test_url_with_port() {
        let url = parse_agent_url("agent://example.com:8080/test").unwrap();
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, 8080);
    }

    #[test]
    fn test_url_with_query() {
        let url = parse_agent_url("agent://example.com/test?foo=bar").unwrap();
        assert_eq!(url.agent, "test");
        assert_eq!(url.query, Some("foo=bar".to_string()));
    }

    #[test]
    fn test_url_encoded_agent() {
        let url = parse_agent_url("agent://example.com/my%20agent").unwrap();
        assert_eq!(url.agent, "my agent");
    }

    #[test]
    fn test_invalid_scheme() {
        let result = parse_agent_url("http://example.com/test");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_agent() {
        let result = parse_agent_url("agent://example.com/");
        assert!(result.is_err());
    }
}
