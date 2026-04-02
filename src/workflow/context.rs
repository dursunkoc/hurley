use evalexpr::eval_boolean;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

use crate::error::{Result, RurlError};

pub struct WorkflowContext {
    pub responses: HashMap<String, JsonValue>,
}

impl WorkflowContext {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
        }
    }

    pub fn store_response(&mut self, step_id: &str, body: JsonValue) {
        self.responses.insert(step_id.to_string(), body);
    }

    pub fn evaluate_condition(&self, condition: &str) -> Result<bool> {
        let mut evaluated_expr = condition.to_string();

        let mut context_json = serde_json::Map::new();
        let mut responses_json = serde_json::Map::new();
        for (id, val) in &self.responses {
            responses_json.insert(id.clone(), val.clone());
        }
        context_json.insert("responses".to_string(), JsonValue::Object(responses_json));
        let root_val_str = JsonValue::Object(context_json).to_string();

        let parts: Vec<String> = evaluated_expr
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        for part in parts {
            if part.starts_with("responses.") {
                let value = gjson::get(&root_val_str, &part);

                let value_str = if value.exists() {
                    match value.kind() {
                        gjson::Kind::String => format!("\"{}\"", value.str()),
                        _ => value.str().to_string(),
                    }
                } else {
                    "null".to_string()
                };

                evaluated_expr = evaluated_expr.replace(&part, &value_str);
            }
        }

        match eval_boolean(&evaluated_expr) {
            Ok(res) => Ok(res),
            Err(e) => Err(RurlError::WorkflowError(format!(
                "Failed to evaluate condition '{}': {}",
                evaluated_expr, e
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_evaluate_condition() {
        let mut ctx = WorkflowContext::new();
        ctx.store_response(
            "get_user",
            json!({
                "error": false,
                "message": "OK",
                "user": {
                    "id": 42,
                    "name": "Alice"
                }
            }),
        );

        // Boolean comparison
        assert_eq!(
            ctx.evaluate_condition("responses.get_user.error == false").unwrap(),
            true
        );
        assert_eq!(
            ctx.evaluate_condition("responses.get_user.error == true").unwrap(),
            false
        );

        // String comparison
        assert_eq!(
            ctx.evaluate_condition("responses.get_user.message == \"OK\"").unwrap(),
            true
        );
        assert_eq!(
            ctx.evaluate_condition("responses.get_user.user.name == \"Alice\"").unwrap(),
            true
        );

        // Number comparison
        assert_eq!(
            ctx.evaluate_condition("responses.get_user.user.id == 42").unwrap(),
            true
        );
        assert_eq!(
            ctx.evaluate_condition("responses.get_user.user.id > 40").unwrap(),
            true
        );
    }
}
