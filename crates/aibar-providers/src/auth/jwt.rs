use base64::Engine;
use serde_json::Value;

/// Decode a JWT token's payload (without verification).
pub fn decode_jwt_payload(token: &str) -> anyhow::Result<Value> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid JWT: expected 3 parts, got {}", parts.len());
    }

    let payload_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[1])?;
    let payload: Value = serde_json::from_slice(&payload_bytes)?;
    Ok(payload)
}

/// Extract expiration timestamp from JWT payload.
pub fn jwt_expiration(token: &str) -> anyhow::Result<Option<i64>> {
    let payload = decode_jwt_payload(token)?;
    Ok(payload["exp"].as_i64())
}
