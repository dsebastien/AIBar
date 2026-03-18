use crate::models::{CreditsSnapshot, UsageSnapshot};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FetchKind {
    Cli,
    Web,
    OAuth,
    ApiToken,
    LocalProbe,
    WebDashboard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Runtime {
    App,
    Cli,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceMode {
    Auto,
    Web,
    Cli,
    OAuth,
    Api,
}

pub struct FetchContext {
    pub runtime: Runtime,
    pub source_mode: SourceMode,
    pub include_credits: bool,
    pub web_timeout: Duration,
    pub verbose: bool,
    pub env: HashMap<String, String>,
    pub http_client: reqwest::Client,
}

impl Default for FetchContext {
    fn default() -> Self {
        Self {
            runtime: Runtime::App,
            source_mode: SourceMode::Auto,
            include_credits: true,
            web_timeout: Duration::from_secs(30),
            verbose: false,
            env: HashMap::new(),
            http_client: reqwest::Client::new(),
        }
    }
}

impl FetchContext {
    pub fn has_env(&self, key: &str) -> bool {
        self.env.contains_key(key) || std::env::var(key).is_ok()
    }

    pub fn get_env(&self, key: &str) -> Option<String> {
        self.env.get(key).cloned().or_else(|| std::env::var(key).ok())
    }

    pub fn require_env(&self, key: &str) -> anyhow::Result<String> {
        self.get_env(key)
            .ok_or_else(|| anyhow::anyhow!("{} not set", key))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchResult {
    pub usage: UsageSnapshot,
    pub credits: Option<CreditsSnapshot>,
    pub source_label: String,
    pub strategy_id: String,
    pub strategy_kind: FetchKind,
}

#[async_trait]
pub trait FetchStrategy: Send + Sync {
    fn id(&self) -> &str;
    fn kind(&self) -> FetchKind;
    async fn is_available(&self, ctx: &FetchContext) -> bool;
    async fn fetch(&self, ctx: &FetchContext) -> anyhow::Result<FetchResult>;
    fn should_fallback(&self, error: &anyhow::Error, ctx: &FetchContext) -> bool;
}
