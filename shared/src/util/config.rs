//! Configuration utilities

use serde::de::DeserializeOwned;
use std::path::Path;

/// Load a TOML configuration file
pub fn load_toml_config<P: AsRef<Path>, T: DeserializeOwned>(path: P) -> crate::Result<T> {
    let content = std::fs::read_to_string(path)?;
    let config: T = toml::from_str(&content)?;
    Ok(config)
}

/// Save a configuration to a TOML file
pub fn save_toml_config<P: AsRef<Path>, T: serde::Serialize>(path: P, config: &T) -> crate::Result<()> {
    let content = toml::to_string_pretty(config)?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Load a YAML configuration file
pub fn load_yaml_config<P: AsRef<Path>, T: DeserializeOwned>(path: P) -> crate::Result<T> {
    let content = std::fs::read_to_string(path)?;
    let config: T = serde_yaml::from_str(&content)?;
    Ok(config)
}

/// Save a configuration to a YAML file
pub fn save_yaml_config<P: AsRef<Path>, T: serde::Serialize>(path: P, config: &T) -> crate::Result<()> {
    let content = serde_yaml::to_string(config)?;
    std::fs::write(path, content)?;
    Ok(())
}

/// Get the default config directory
pub fn config_dir() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".agent")
}

/// Get the default data directory
pub fn data_dir() -> std::path::PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".agent")
}

/// Ensure a directory exists
pub fn ensure_dir<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    std::fs::create_dir_all(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestConfig {
        name: String,
        value: i32,
    }

    #[test]
    fn test_toml_config_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let original = TestConfig {
            name: "test".to_string(),
            value: 42,
        };

        save_toml_config(&path, &original).unwrap();
        let loaded: TestConfig = load_toml_config(&path).unwrap();

        assert_eq!(original, loaded);
    }

    #[test]
    fn test_yaml_config_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.yaml");

        let original = TestConfig {
            name: "test".to_string(),
            value: 42,
        };

        save_yaml_config(&path, &original).unwrap();
        let loaded: TestConfig = load_yaml_config(&path).unwrap();

        assert_eq!(original, loaded);
    }
}
