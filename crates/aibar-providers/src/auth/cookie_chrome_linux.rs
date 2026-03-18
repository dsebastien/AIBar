use aes::Aes128;
use cbc::cipher::{BlockDecryptMut, KeyIvInit};
use pbkdf2::pbkdf2_hmac;
use rusqlite::Connection;
use sha1::Sha1;
use std::path::Path;

type Aes128CbcDec = cbc::Decryptor<Aes128>;

const CHROME_SALT: &[u8] = b"saltysalt";
const CHROME_ITERATIONS: u32 = 1;
const CHROME_KEY_LEN: usize = 16;
const CHROME_IV: &[u8; 16] = b"\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20";

/// Retrieve Chrome's safe storage password from the Linux keyring.
/// Falls back to the default "peanuts" password if unavailable.
async fn get_chrome_password() -> String {
    match try_secret_service().await {
        Ok(password) => password,
        Err(_) => "peanuts".to_string(),
    }
}

async fn try_secret_service() -> anyhow::Result<String> {
    let ss = secret_service::SecretService::connect(secret_service::EncryptionType::Dh).await?;
    let collection = ss.get_default_collection().await?;

    if collection.is_locked().await? {
        collection.unlock().await?;
    }

    let items = collection
        .search_items(
            [("application", "chrome")].into_iter().collect(),
        )
        .await?;

    if let Some(item) = items.first() {
        let secret = item.get_secret().await?;
        Ok(String::from_utf8(secret)?)
    } else {
        anyhow::bail!("Chrome password not found in keyring")
    }
}

fn derive_key(password: &str) -> [u8; CHROME_KEY_LEN] {
    let mut key = [0u8; CHROME_KEY_LEN];
    pbkdf2_hmac::<Sha1>(
        password.as_bytes(),
        CHROME_SALT,
        CHROME_ITERATIONS,
        &mut key,
    );
    key
}

fn decrypt_value(encrypted: &[u8], key: &[u8; CHROME_KEY_LEN]) -> anyhow::Result<String> {
    if encrypted.len() < 3 || &encrypted[..3] != b"v10" && &encrypted[..3] != b"v11" {
        anyhow::bail!("Unknown encryption version");
    }

    let ciphertext = &encrypted[3..];
    let mut buf = ciphertext.to_vec();

    let decryptor = Aes128CbcDec::new(key.into(), CHROME_IV.into());
    let decrypted = decryptor
        .decrypt_padded_mut::<cbc::cipher::block_padding::Pkcs7>(&mut buf)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    Ok(String::from_utf8_lossy(decrypted).to_string())
}

/// Read and decrypt a Chrome cookie on Linux.
pub async fn read_chrome_cookie(
    profile_path: &Path,
    host: &str,
    name: &str,
) -> anyhow::Result<Option<String>> {
    let cookies_db = profile_path.join("Cookies");
    if !cookies_db.exists() {
        return Ok(None);
    }

    let password = get_chrome_password().await;
    let key = derive_key(&password);

    let conn = Connection::open_with_flags(
        &cookies_db,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;

    let mut stmt = conn.prepare(
        "SELECT encrypted_value FROM cookies WHERE host_key LIKE ?1 AND name = ?2 ORDER BY last_access_utc DESC LIMIT 1",
    )?;

    let encrypted: Option<Vec<u8>> = stmt
        .query_row(
            rusqlite::params![format!("%{}", host), name],
            |row| row.get(0),
        )
        .ok();

    match encrypted {
        Some(data) if !data.is_empty() => {
            let value = decrypt_value(&data, &key)?;
            Ok(Some(value))
        }
        _ => Ok(None),
    }
}
