use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderId {
    Codex,
    Claude,
    Cursor,
    Gemini,
    Copilot,
    Augment,
    Amp,
    Kimi,
    KimiK2,
    Zai,
    MiniMax,
    Factory,
    JetBrains,
    Kilo,
    Kiro,
    VertexAi,
    Ollama,
    Synthetic,
    Warp,
    OpenRouter,
    Antigravity,
    OpenCode,
}

impl ProviderId {
    pub fn all() -> &'static [ProviderId] {
        &[
            ProviderId::Codex,
            ProviderId::Claude,
            ProviderId::Cursor,
            ProviderId::Gemini,
            ProviderId::Copilot,
            ProviderId::Augment,
            ProviderId::Amp,
            ProviderId::Kimi,
            ProviderId::KimiK2,
            ProviderId::Zai,
            ProviderId::MiniMax,
            ProviderId::Factory,
            ProviderId::JetBrains,
            ProviderId::Kilo,
            ProviderId::Kiro,
            ProviderId::VertexAi,
            ProviderId::Ollama,
            ProviderId::Synthetic,
            ProviderId::Warp,
            ProviderId::OpenRouter,
            ProviderId::Antigravity,
            ProviderId::OpenCode,
        ]
    }
}

impl std::str::FromStr for ProviderId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::Value::String(s.to_string()))
            .map_err(|_| format!("Unknown provider: {}", s))
    }
}

impl fmt::Display for ProviderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ProviderId::Codex => "Codex",
            ProviderId::Claude => "Claude",
            ProviderId::Cursor => "Cursor",
            ProviderId::Gemini => "Gemini",
            ProviderId::Copilot => "Copilot",
            ProviderId::Augment => "Augment",
            ProviderId::Amp => "Amp",
            ProviderId::Kimi => "Kimi",
            ProviderId::KimiK2 => "Kimi K2",
            ProviderId::Zai => "z.ai",
            ProviderId::MiniMax => "MiniMax",
            ProviderId::Factory => "Factory",
            ProviderId::JetBrains => "JetBrains",
            ProviderId::Kilo => "Kilo",
            ProviderId::Kiro => "Kiro",
            ProviderId::VertexAi => "Vertex AI",
            ProviderId::Ollama => "Ollama",
            ProviderId::Synthetic => "Synthetic",
            ProviderId::Warp => "Warp",
            ProviderId::OpenRouter => "OpenRouter",
            ProviderId::Antigravity => "Antigravity",
            ProviderId::OpenCode => "OpenCode",
        };
        write!(f, "{}", name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateWindow {
    pub used_percent: f64,
    pub window_minutes: Option<i32>,
    pub resets_at: Option<DateTime<Utc>>,
    pub reset_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCostSnapshot {
    pub used: f64,
    pub limit: f64,
    pub currency_code: String,
    pub period: Option<String>,
    pub resets_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderIdentitySnapshot {
    pub email: Option<String>,
    pub team: Option<String>,
    pub plan: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageSnapshot {
    pub primary: Option<RateWindow>,
    pub secondary: Option<RateWindow>,
    pub tertiary: Option<RateWindow>,
    pub provider_cost: Option<ProviderCostSnapshot>,
    pub updated_at: DateTime<Utc>,
    pub identity: Option<ProviderIdentitySnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditsSnapshot {
    pub remaining: f64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RefreshCadence {
    Manual,
    OneMinute,
    TwoMinutes,
    FiveMinutes,
    FifteenMinutes,
}

impl RefreshCadence {
    pub fn to_seconds(self) -> Option<u64> {
        match self {
            RefreshCadence::Manual => None,
            RefreshCadence::OneMinute => Some(60),
            RefreshCadence::TwoMinutes => Some(120),
            RefreshCadence::FiveMinutes => Some(300),
            RefreshCadence::FifteenMinutes => Some(900),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplayMode {
    Individual,
    Merged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub enabled_providers: Vec<ProviderId>,
    pub provider_order: Vec<ProviderId>,
    pub refresh_cadence: RefreshCadence,
    pub display_mode: DisplayMode,
    pub launch_at_login: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            enabled_providers: vec![
                ProviderId::Claude,
                ProviderId::Codex,
                ProviderId::Cursor,
                ProviderId::Gemini,
            ],
            provider_order: ProviderId::all().to_vec(),
            refresh_cadence: RefreshCadence::FiveMinutes,
            display_mode: DisplayMode::Individual,
            launch_at_login: false,
        }
    }
}
