use rusqlite::Connection;
use std::path::Path;

/// Read cookies from Firefox's cookies.sqlite database.
/// Firefox does not encrypt cookie values on Windows/Linux.
pub fn read_firefox_cookies(
    profile_path: &Path,
    host: &str,
    name: &str,
) -> anyhow::Result<Option<String>> {
    let cookies_db = profile_path.join("cookies.sqlite");
    if !cookies_db.exists() {
        return Ok(None);
    }

    let conn = Connection::open_with_flags(
        &cookies_db,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;

    let mut stmt = conn.prepare(
        "SELECT value FROM moz_cookies WHERE host LIKE ?1 AND name = ?2 ORDER BY lastAccessed DESC LIMIT 1",
    )?;

    let result = stmt
        .query_row(
            rusqlite::params![format!("%{}", host), name],
            |row| row.get::<_, String>(0),
        )
        .ok();

    Ok(result)
}
