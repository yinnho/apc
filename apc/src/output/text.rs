//! Text output formatter

use shared::protocol::AgentInfo;

/// Capabilities info for output
pub struct CapabilitiesInfo {
    pub name: String,
    pub capabilities: Vec<String>,
}

/// Print find results in text format
pub fn print_find_results_text(results: &[AgentInfo]) {
    if results.is_empty() {
        println!("No agents found");
        return;
    }

    println!("Found {} agent(s):\n", results.len());
    for agent in results {
        println!("  {} ({})", agent.name, agent.id);
        println!("    Type: {}", agent.agent_type);
        println!("    Address: {}", agent.address);
        if let Some(rating) = agent.rating {
            println!("    Rating: {:.1}", rating);
        }
        if !agent.capabilities.is_empty() {
            println!("    Capabilities: {}", agent.capabilities.join(", "));
        }
        println!();
    }
}

/// Print capabilities in text format
pub fn print_capabilities_text(capa: &CapabilitiesInfo) {
    println!("Agent: {}", capa.name);
    println!("Capabilities:");
    for cap in &capa.capabilities {
        println!("  - {}", cap);
    }
}

/// Print response in text format (just pass through)
pub fn print_response_text(response: &str) {
    println!("{}", response);
}
