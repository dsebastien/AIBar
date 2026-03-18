use crate::models::UsageSnapshot;
use crate::traits::{FetchContext, FetchKind, FetchResult, FetchStrategy};
use async_trait::async_trait;
use chrono::Utc;
use std::path::PathBuf;

/// Strategy that parses JetBrains AI configuration from local XML config files.
pub struct JetBrainsLocalConfigStrategy;

fn jetbrains_config_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    #[cfg(target_os = "linux")]
    {
        if let Ok(home) = std::env::var("HOME") {
            let home = PathBuf::from(home);
            let config_base = home.join(".config").join("JetBrains");
            if config_base.exists() {
                // Look for all IDE config directories
                if let Ok(entries) = std::fs::read_dir(&config_base) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            dirs.push(path);
                        }
                    }
                }
            }
            // Also check ~/.local/share/JetBrains
            let data_base = home.join(".local").join("share").join("JetBrains");
            if data_base.exists() {
                if let Ok(entries) = std::fs::read_dir(&data_base) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            dirs.push(path);
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            let jb_base = PathBuf::from(appdata).join("JetBrains");
            if jb_base.exists() {
                if let Ok(entries) = std::fs::read_dir(&jb_base) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            dirs.push(path);
                        }
                    }
                }
            }
        }
    }

    dirs
}

fn find_jetbrains_ai_config() -> Option<(String, Option<String>)> {
    for dir in jetbrains_config_dirs() {
        // Look for AI Assistant configuration XML
        let ai_config = dir.join("options").join("ai-assistant.xml");
        if ai_config.exists() {
            if let Ok(contents) = std::fs::read_to_string(&ai_config) {
                let plan = parse_ai_assistant_plan(&contents);
                let email = parse_ai_assistant_email(&contents);
                return Some((
                    plan.unwrap_or_else(|| "Unknown".to_string()),
                    email,
                ));
            }
        }

        // Also check the general options XML
        let general_config = dir.join("options").join("other.xml");
        if general_config.exists() {
            if let Ok(contents) = std::fs::read_to_string(&general_config) {
                if contents.contains("ai.assistant") || contents.contains("AIAssistant") {
                    let plan = parse_ai_assistant_plan(&contents);
                    let email = parse_ai_assistant_email(&contents);
                    return Some((
                        plan.unwrap_or_else(|| "Unknown".to_string()),
                        email,
                    ));
                }
            }
        }
    }
    None
}

fn parse_ai_assistant_plan(xml_content: &str) -> Option<String> {
    // Simple XML parsing - look for subscription/plan info
    // Format: <option name="subscriptionPlan" value="..." />
    for line in xml_content.lines() {
        let trimmed = line.trim();
        if trimmed.contains("subscriptionPlan") || trimmed.contains("subscription_plan") {
            if let Some(start) = trimmed.find("value=\"") {
                let rest = &trimmed[start + 7..];
                if let Some(end) = rest.find('"') {
                    return Some(rest[..end].to_string());
                }
            }
        }
    }
    None
}

fn parse_ai_assistant_email(xml_content: &str) -> Option<String> {
    for line in xml_content.lines() {
        let trimmed = line.trim();
        if trimmed.contains("accountEmail") || trimmed.contains("account_email") {
            if let Some(start) = trimmed.find("value=\"") {
                let rest = &trimmed[start + 7..];
                if let Some(end) = rest.find('"') {
                    return Some(rest[..end].to_string());
                }
            }
        }
    }
    None
}

fn find_jetbrains_usage_data() -> Option<(f64, f64)> {
    for dir in jetbrains_config_dirs() {
        // Look for usage tracking data
        let usage_path = dir.join("usage").join("ai-assistant-usage.json");
        if usage_path.exists() {
            if let Ok(contents) = std::fs::read_to_string(&usage_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&contents) {
                    let used = json["requests_used"].as_f64().unwrap_or(0.0);
                    let limit = json["requests_limit"].as_f64().unwrap_or(0.0);
                    if limit > 0.0 {
                        return Some((used, limit));
                    }
                }
            }
        }
    }
    None
}

#[async_trait]
impl FetchStrategy for JetBrainsLocalConfigStrategy {
    fn id(&self) -> &str {
        "jetbrains.local_config"
    }

    fn kind(&self) -> FetchKind {
        FetchKind::LocalProbe
    }

    async fn is_available(&self, _ctx: &FetchContext) -> bool {
        find_jetbrains_ai_config().is_some()
    }

    async fn fetch(&self, _ctx: &FetchContext) -> anyhow::Result<FetchResult> {
        let (plan, email) = find_jetbrains_ai_config()
            .ok_or_else(|| anyhow::anyhow!("JetBrains AI config not found"))?;

        let usage_data = find_jetbrains_usage_data();

        let primary = usage_data.map(|(used, limit)| {
            let used_percent = if limit > 0.0 {
                (used / limit) * 100.0
            } else {
                0.0
            };
            crate::models::RateWindow {
                used_percent,
                window_minutes: None,
                resets_at: None,
                reset_description: Some(format!("{:.0}/{:.0} requests", used, limit)),
            }
        });

        Ok(FetchResult {
            usage: UsageSnapshot {
                primary,
                secondary: None,
                tertiary: None,
                provider_cost: None,
                updated_at: Utc::now(),
                identity: Some(crate::models::ProviderIdentitySnapshot {
                    email,
                    team: None,
                    plan: Some(plan),
                }),
            },
            credits: None,
            source_label: "local".to_string(),
            strategy_id: self.id().to_string(),
            strategy_kind: self.kind(),
        })
    }

    fn should_fallback(&self, _error: &anyhow::Error, _ctx: &FetchContext) -> bool {
        false
    }
}
