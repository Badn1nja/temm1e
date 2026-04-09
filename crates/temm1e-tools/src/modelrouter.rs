//! Model router control tool — toggle tiered model routing on/off.

use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use temm1e_core::types::session::SessionContext;
use temm1e_core::{Tool, ToolOutput};
use tokio::sync::RwLock;

/// Shared model router configuration that can be updated at runtime.
pub type SharedModelRouterConfig = Arc<RwLock<temm1e_agent::ModelRouterConfig>>;

/// Tool to control tiered model routing behavior.
pub struct ModelRouterTool {
    config: SharedModelRouterConfig,
}

impl ModelRouterTool {
    /// Create a new model router control tool.
    pub fn new(config: SharedModelRouterConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl Tool for ModelRouterTool {
    fn name(&self) -> &str {
        "modelrouter"
    }

    fn description(&self) -> &str {
        "Toggle tiered model routing on/off. When enabled, simple tasks use fast models, complex tasks use premium models. Usage: modelrouter on|off|status"
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["on", "off", "status"],
                    "description": "Action to perform: 'on' to enable tiered routing, 'off' to disable (use primary model for all tasks), 'status' to check current state"
                }
            },
            "required": ["action"]
        })
    }

    async fn execute(&self, args: Value, _session: &mut SessionContext) -> Result<ToolOutput, Box<dyn std::error::Error + Send + Sync>> {
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or("Missing or invalid 'action' parameter")?;

        match action {
            "on" => {
                let mut config = self.config.write().await;
                config.enabled = true;
                Ok(ToolOutput {
                    content: "✅ Tiered model routing enabled.\n\n\
                             • Simple tasks (file reads, status checks) → Fast model\n\
                             • Standard tasks → Primary model\n\
                             • Complex tasks (architecture, debugging, refactoring) → Premium model\n\n\
                             Use `!fast` or `!best` prefixes to override routing for specific requests.".to_string(),
                    is_error: false,
                })
            }
            "off" => {
                let mut config = self.config.write().await;
                config.enabled = false;
                Ok(ToolOutput {
                    content: "⚪ Tiered model routing disabled. All tasks will use the primary model.".to_string(),
                    is_error: false,
                })
            }
            "status" => {
                let config = self.config.read().await;
                let status = if config.enabled { "enabled" } else { "disabled" };
                let fast_model = config.fast_model.as_deref().unwrap_or("(fallback to primary)");
                let premium_model = config.premium_model.as_deref().unwrap_or("(fallback to primary)");
                
                Ok(ToolOutput {
                    content: format!(
                        "📊 Model Router Status: **{}**\n\n\
                         **Configuration:**\n\
                         • Fast tier: {}\n\
                         • Primary tier: {}\n\
                         • Premium tier: {}\n\n\
                         **Routing Rules:**\n\
                         • Trivial/Simple → Fast tier\n\
                         • Standard → Primary tier\n\
                         • Complex → Premium tier\n\n\
                         Use `modelrouter {{\"action\": \"on\"}}` to enable or `modelrouter {{\"action\": \"off\"}}` to disable.",
                        status, fast_model, config.primary_model, premium_model
                    ),
                    is_error: false,
                })
            }
            _ => Ok(ToolOutput {
                content: "❌ Invalid action. Use 'on', 'off', or 'status'.".to_string(),
                is_error: true,
            }),
        }
    }
}
