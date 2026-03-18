use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderStatus {
    Operational,
    DegradedPerformance,
    PartialOutage,
    MajorOutage,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStatusInfo {
    pub status: ProviderStatus,
    pub description: Option<String>,
    pub updated_at: Option<String>,
}

/// Poll a StatusPage.io status page for current status.
pub async fn poll_status_page(url: &str) -> anyhow::Result<ProviderStatusInfo> {
    let api_url = format!("{}/api/v2/status.json", url.trim_end_matches('/'));
    let response = reqwest::get(&api_url).await?;
    let body: serde_json::Value = response.json().await?;

    let indicator = body["status"]["indicator"]
        .as_str()
        .unwrap_or("unknown");

    let status = match indicator {
        "none" => ProviderStatus::Operational,
        "minor" => ProviderStatus::DegradedPerformance,
        "major" => ProviderStatus::PartialOutage,
        "critical" => ProviderStatus::MajorOutage,
        _ => ProviderStatus::Unknown,
    };

    let description = body["status"]["description"]
        .as_str()
        .map(|s| s.to_string());

    Ok(ProviderStatusInfo {
        status,
        description,
        updated_at: body["page"]["updated_at"].as_str().map(|s| s.to_string()),
    })
}
