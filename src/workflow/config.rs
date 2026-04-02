use crate::error::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// A workflow of sequential dependent HTTP requests with conditionals.
#[derive(Debug, Clone, Deserialize)]
pub struct Workflow {
    /// List of workflow steps to execute
    pub steps: Vec<WorkflowStep>,
}

impl Workflow {
    /// Load a workflow from a JSON file
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }
}

/// A single step in a workflow representing one HTTP request and its execution condition.
#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowStep {
    /// Unique identifier for this step. Used to reference its response in later steps.
    pub id: String,

    /// Optional condition expression. If evaluated to false, this step is skipped.
    /// Example: `responses.get_user.body.error == true`
    pub condition: Option<String>,

    /// The HTTP request configuration for this step
    pub request: StepRequest,
}

/// Request configuration for a workflow step
#[derive(Debug, Clone, Deserialize)]
pub struct StepRequest {
    /// HTTP method (e.g., GET, POST)
    #[serde(default = "default_method")]
    pub method: String,

    /// Request path or absolute URL
    pub path: String,

    /// Request body as a JSON value
    #[serde(default)]
    pub body: Option<serde_json::Value>,

    /// Additional headers
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
}

fn default_method() -> String {
    "GET".to_string()
}
