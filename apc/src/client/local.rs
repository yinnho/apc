//! Local client for executing CLI commands

use crate::error::{Error, Result};

/// Local client for executing local CLI agents
pub struct LocalClient;

impl LocalClient {
    /// Create a new local client
    pub fn new() -> Self {
        Self
    }

    /// Execute a local agent command
    pub async fn send(&self, agent: &str, message: &str) -> Result<String> {
        // Map agent name to command
        let command = self.map_agent_to_command(agent)?;

        // Execute the command
        let mut cmd = tokio::process::Command::new(&command[0]);
        cmd.args(&command[1..]);

        // Only add message as argument if it's not empty
        if !message.is_empty() {
            cmd.arg(message);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| Error::Internal(format!("Failed to execute command: {}", e)))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(Error::Internal(format!("Command failed: {}", stderr)))
        }
    }

    /// Map agent name to a CLI command
    fn map_agent_to_command(&self, agent: &str) -> Result<Vec<String>> {
        // Check if agent is a direct command path
        if agent.starts_with('/') || agent.starts_with("./") || agent.contains('/') {
            return Ok(vec![agent.to_string()]);
        }

        // Map known agents to commands
        match agent {
            "claude" | "claude-code" => Ok(vec!["claude".to_string()]),
            "echo" => Ok(vec!["echo".to_string()]),
            "date" => Ok(vec!["date".to_string()]),
            "whoami" => Ok(vec!["whoami".to_string()]),
            _ => {
                // Try to find the command in PATH
                Ok(vec![agent.to_string()])
            }
        }
    }
}

impl Default for LocalClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_echo() {
        let client = LocalClient::new();
        let result = client.send("echo", "hello world").await.unwrap();
        assert_eq!(result, "hello world");
    }

    #[tokio::test]
    async fn test_local_date() {
        let client = LocalClient::new();
        let result = client.send("date", "").await.unwrap();
        assert!(!result.is_empty());
    }
}
