use crate::models::ProviderId;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ProviderColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderBranding {
    pub icon_resource: &'static str,
    pub color: ProviderColor,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderMetadata {
    pub display_name: &'static str,
    pub session_label: &'static str,
    pub weekly_label: &'static str,
    pub supports_credits: bool,
    pub default_enabled: bool,
    pub dashboard_url: Option<&'static str>,
    pub status_page_url: Option<&'static str>,
}

#[derive(Debug, Clone)]
pub struct CliConfig {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
}

pub struct ProviderDescriptor {
    pub id: ProviderId,
    pub metadata: ProviderMetadata,
    pub branding: ProviderBranding,
    pub cli_config: CliConfig,
}

pub fn get_descriptor(id: ProviderId) -> ProviderDescriptor {
    match id {
        ProviderId::Claude => ProviderDescriptor {
            id,
            metadata: ProviderMetadata {
                display_name: "Claude",
                session_label: "Session",
                weekly_label: "Weekly",
                supports_credits: false,
                default_enabled: true,
                dashboard_url: Some("https://claude.ai"),
                status_page_url: Some("https://status.anthropic.com"),
            },
            branding: ProviderBranding {
                icon_resource: "claude",
                color: ProviderColor { r: 212, g: 165, b: 116 },
            },
            cli_config: CliConfig { name: "claude", aliases: &[] },
        },
        ProviderId::Codex => ProviderDescriptor {
            id,
            metadata: ProviderMetadata {
                display_name: "Codex",
                session_label: "Session",
                weekly_label: "Weekly",
                supports_credits: true,
                default_enabled: true,
                dashboard_url: Some("https://chatgpt.com"),
                status_page_url: None,
            },
            branding: ProviderBranding {
                icon_resource: "codex",
                color: ProviderColor { r: 16, g: 163, b: 127 },
            },
            cli_config: CliConfig { name: "codex", aliases: &["openai"] },
        },
        ProviderId::Cursor => ProviderDescriptor {
            id,
            metadata: ProviderMetadata {
                display_name: "Cursor",
                session_label: "Premium",
                weekly_label: "Usage",
                supports_credits: false,
                default_enabled: true,
                dashboard_url: Some("https://cursor.com/settings"),
                status_page_url: None,
            },
            branding: ProviderBranding {
                icon_resource: "cursor",
                color: ProviderColor { r: 0, g: 212, b: 170 },
            },
            cli_config: CliConfig { name: "cursor", aliases: &[] },
        },
        ProviderId::Gemini => ProviderDescriptor {
            id,
            metadata: ProviderMetadata {
                display_name: "Gemini",
                session_label: "Session",
                weekly_label: "Daily",
                supports_credits: false,
                default_enabled: true,
                dashboard_url: Some("https://gemini.google.com"),
                status_page_url: None,
            },
            branding: ProviderBranding {
                icon_resource: "gemini",
                color: ProviderColor { r: 66, g: 133, b: 244 },
            },
            cli_config: CliConfig { name: "gemini", aliases: &[] },
        },
        ProviderId::Copilot => ProviderDescriptor {
            id,
            metadata: ProviderMetadata {
                display_name: "Copilot",
                session_label: "Premium",
                weekly_label: "Chat",
                supports_credits: false,
                default_enabled: false,
                dashboard_url: Some("https://github.com/settings/copilot"),
                status_page_url: Some("https://www.githubstatus.com"),
            },
            branding: ProviderBranding {
                icon_resource: "copilot",
                color: ProviderColor { r: 168, g: 85, b: 247 },
            },
            cli_config: CliConfig { name: "copilot", aliases: &[] },
        },
        _ => {
            // For remaining providers, provide basic descriptors
            let (name, label, color, url) = match id {
                ProviderId::Augment => ("Augment", "augment", ProviderColor { r: 99, g: 102, b: 241 }, Some("https://augmentcode.com")),
                ProviderId::Amp => ("Amp", "amp", ProviderColor { r: 245, g: 158, b: 11 }, Some("https://ampcode.com")),
                ProviderId::Kimi => ("Kimi", "kimi", ProviderColor { r: 139, g: 92, b: 246 }, None),
                ProviderId::KimiK2 => ("Kimi K2", "kimi-k2", ProviderColor { r: 124, g: 58, b: 237 }, None),
                ProviderId::Zai => ("z.ai", "zai", ProviderColor { r: 236, g: 72, b: 153 }, Some("https://z.ai")),
                ProviderId::MiniMax => ("MiniMax", "minimax", ProviderColor { r: 20, g: 184, b: 166 }, None),
                ProviderId::Factory => ("Factory", "factory", ProviderColor { r: 249, g: 115, b: 22 }, None),
                ProviderId::JetBrains => ("JetBrains", "jetbrains", ProviderColor { r: 255, g: 49, b: 140 }, None),
                ProviderId::Kilo => ("Kilo", "kilo", ProviderColor { r: 6, g: 182, b: 212 }, None),
                ProviderId::Kiro => ("Kiro", "kiro", ProviderColor { r: 244, g: 63, b: 94 }, None),
                ProviderId::VertexAi => ("Vertex AI", "vertex-ai", ProviderColor { r: 66, g: 133, b: 244 }, None),
                ProviderId::Ollama => ("Ollama", "ollama", ProviderColor { r: 255, g: 255, b: 255 }, Some("http://localhost:11434")),
                ProviderId::Synthetic => ("Synthetic", "synthetic", ProviderColor { r: 163, g: 230, b: 53 }, None),
                ProviderId::Warp => ("Warp", "warp", ProviderColor { r: 0, g: 212, b: 255 }, None),
                ProviderId::OpenRouter => ("OpenRouter", "openrouter", ProviderColor { r: 109, g: 40, b: 217 }, Some("https://openrouter.ai")),
                ProviderId::Antigravity => ("Antigravity", "antigravity", ProviderColor { r: 132, g: 204, b: 22 }, None),
                ProviderId::OpenCode => ("OpenCode", "opencode", ProviderColor { r: 14, g: 165, b: 233 }, None),
                // Already handled above
                _ => unreachable!(),
            };
            ProviderDescriptor {
                id,
                metadata: ProviderMetadata {
                    display_name: name,
                    session_label: "Session",
                    weekly_label: "Weekly",
                    supports_credits: false,
                    default_enabled: false,
                    dashboard_url: url,
                    status_page_url: None,
                },
                branding: ProviderBranding {
                    icon_resource: label,
                    color,
                },
                cli_config: CliConfig {
                    name: label,
                    aliases: &[],
                },
            }
        }
    }
}
