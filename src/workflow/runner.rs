use super::{Workflow, WorkflowContext};
use crate::error::Result;
use crate::http::{HttpClient, HttpRequest};
use colored::Colorize;
use serde_json::Value as JsonValue;

pub struct WorkflowRunner {
    base_url: String,
    base_request: HttpRequest,
    verbose: bool,
}

impl WorkflowRunner {
    pub fn new(base_url: String, base_request: HttpRequest, verbose: bool) -> Self {
        Self {
            base_url,
            base_request,
            verbose,
        }
    }

    pub async fn run(&self, workflow: &Workflow) -> Result<()> {
        println!("{}", "🚀 Starting Workflow Execution".cyan().bold());
        let mut context = WorkflowContext::new();
        let client = HttpClient::new(self.verbose);

        for step in &workflow.steps {
            println!("\n{} Step: {}", "➜".yellow(), step.id.bold());

            // 1. Evaluate condition
            if let Some(condition) = &step.condition {
                print!("  Evaluating condition: {} ... ", condition.black());
                match context.evaluate_condition(condition) {
                    Ok(true) => println!("{}", "Matched".green()),
                    Ok(false) => {
                        println!("{}", "Skipped (false)".yellow());
                        continue;
                    }
                    Err(e) => {
                        println!("{}", "Error".red());
                        return Err(e);
                    }
                }
            }

            // 2. Build Request
            let url = if step.request.path.starts_with("http") {
                step.request.path.clone()
            } else {
                format!(
                    "{}{}",
                    self.base_url.trim_end_matches('/'),
                    step.request.path
                )
            };

            let mut req = HttpRequest::new(url)
                .method(&step.request.method)?
                .timeout(self.base_request.timeout)
                .follow_redirects(self.base_request.follow_redirects);

            if let Some(ref body) = step.request.body {
                req = req.body(body.to_string());
            }

            // 3. Execute Request
            let response = client.execute(&req).await?;
            response.print(false, self.verbose);

            // 4. Store Response in Context
            if let Ok(json_body) = serde_json::from_str::<JsonValue>(&response.body) {
                context.store_response(&step.id, json_body);
            }
        }

        println!("\n{}", "✅ Workflow Completed".green().bold());
        Ok(())
    }
}
