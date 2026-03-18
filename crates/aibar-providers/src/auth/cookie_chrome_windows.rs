use std::path::Path;

/// Read and decrypt a Chrome cookie on Windows using DPAPI.
/// This is a placeholder - the actual Windows DPAPI integration
/// requires the `windows` crate with Win32_Security_Cryptography feature.
pub async fn read_chrome_cookie(
    _profile_path: &Path,
    _host: &str,
    _name: &str,
) -> anyhow::Result<Option<String>> {
    // TODO: Implement Windows DPAPI cookie decryption
    // 1. Read Local State file for os_crypt.encrypted_key
    // 2. Base64 decode and strip DPAPI prefix
    // 3. CryptUnprotectData to get the AES key
    // 4. Open Cookies SQLite DB
    // 5. Decrypt encrypted_value with AES-256-GCM
    anyhow::bail!("Windows cookie decryption not yet implemented")
}
