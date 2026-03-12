//! Output formatting for apc

mod text;
mod json;

pub use text::CapabilitiesInfo;
pub use text::print_find_results_text;
pub use text::print_capabilities_text;
pub use json::print_find_results_json;
pub use json::print_capabilities_json;
pub use json::print_response_json;

use crate::cli::OutputFormat;
use shared::protocol::AgentInfo;

/// Output formatter trait
pub trait Output {
    /// Print find results
    fn print_find_results(&self, results: &[AgentInfo]);

    /// Print capabilities
    fn print_capabilities(&self, capa: &CapabilitiesInfo);

    /// Print response
    fn print_response(&self, response: &str);
}

/// Create an output formatter based on format
pub struct OutputFormatter {
    format: OutputFormat,
}

impl OutputFormatter {
    /// Create a new output formatter
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// Print find results
    pub fn print_find_results(&self, results: &[AgentInfo]) {
        match self.format {
            OutputFormat::Text => print_find_results_text(results),
            OutputFormat::Json => print_find_results_json(results),
        }
    }

    /// Print capabilities
    pub fn print_capabilities(&self, capa: &CapabilitiesInfo) {
        match self.format {
            OutputFormat::Text => print_capabilities_text(capa),
            OutputFormat::Json => print_capabilities_json(capa),
        }
    }

    /// Print response
    pub fn print_response(&self, response: &str) {
        match self.format {
            OutputFormat::Text => println!("{}", response),
            OutputFormat::Json => print_response_json(response),
        }
    }
}
