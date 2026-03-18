use std::path::PathBuf;

/// Find a CLI binary by checking `which` for each candidate name.
pub fn which_cli(candidates: &[&str]) -> Option<String> {
    for name in candidates {
        if let Ok(output) = std::process::Command::new("which").arg(name).output() {
            if output.status.success() {
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Get a platform-specific app config directory.
/// On Linux: $HOME/.config/{linux_name}
/// On Windows: ${win_env_var}/{windows_name}
pub fn app_config_dir(linux_name: &str, windows_name: &str) -> Option<PathBuf> {
    app_config_dir_with_env(linux_name, windows_name, "APPDATA")
}

pub fn app_config_dir_with_env(linux_name: &str, windows_name: &str, win_env: &str) -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        let _ = (windows_name, win_env);
        std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".config").join(linux_name))
    }
    #[cfg(target_os = "windows")]
    {
        let _ = linux_name;
        std::env::var(win_env).ok().map(|a| PathBuf::from(a).join(windows_name))
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        let _ = (linux_name, windows_name, win_env);
        None
    }
}

/// Parse a percentage value from CLI text output.
/// Looks for tokens ending in '%' and returns the first found value.
pub fn parse_percent_from_text(output: &str) -> f64 {
    for line in output.lines() {
        for word in line.split_whitespace() {
            if word.ends_with('%') {
                if let Ok(val) = word.trim_end_matches('%').parse::<f64>() {
                    return val;
                }
            }
        }
    }
    0.0
}
