//! JSON output formatter

use serde::Serialize;
use shared::protocol::AgentInfo;
use super::text::CapabilitiesInfo;

/// Print find results in JSON format
pub fn print_find_results_json(results: &[AgentInfo]) {
    let output = serde_json::to_string_pretty(results).unwrap_or_default();
    println!("{}", output);
}

/// Print capabilities in JSON format
pub fn print_capabilities_json(capa: &CapabilitiesInfo) {
    #[derive(Serialize)]
    struct CapaOutput<'a> {
        name: &'a str,
        capabilities: &'a [String],
    }

    let output = CapaOutput {
        name: &capa.name,
        capabilities: &capa.capabilities,
    };

    let json = serde_json::to_string_pretty(&output).unwrap_or_default();
    println!("{}", json);
}

/// Print response in JSON format
pub fn print_response_json(response: &str) {
    // Try to parse as JSON first
    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(response) {
        println!("{}", serde_json::to_string_pretty(&json_value).unwrap_or_default());
    } else {
        // Not valid JSON, wrap in an object
        let output = serde_json::json!({ "response": response });
        println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
    }
}
