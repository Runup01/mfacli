pub mod google;

use crate::storage::models::OtpEntry;
use colored::Colorize;

/// Import entries from a file. When `source` is None the format is
/// auto-detected from the file contents, so `mfa import auth.txt` just works.
pub fn import_from(source: Option<&str>, path: &str) -> Result<Vec<OtpEntry>, Box<dyn std::error::Error>> {
    let src = source.unwrap_or_else(|| detect_source(path));
    match src {
        "google" => google::import(path),
        "json" => import_json(path),
        "csv" => import_csv(path),
        "otpauth" => import_otpauth(path),
        "encrypted" => import_encrypted(path),
        other => Err(format!("Unsupported import source: {}", other).into()),
    }
}

/// Guess the format by peeking at the file contents.
fn detect_source(path: &str) -> &'static str {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let t = content.trim_start();
    if t.contains("otpauth-migration://") {
        "google"
    } else if t.starts_with('{') || t.starts_with('[') {
        "json"
    } else if t.contains("otpauth://") {
        "otpauth"
    } else {
        "csv"
    }
}

fn import_json(path: &str) -> Result<Vec<OtpEntry>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    parse_entries_json(&content)
}

/// Accept both the versioned native wrapper `{ "version", "entries" }` and a
/// legacy bare array, so old exports and hand-written templates both load.
fn parse_entries_json(json: &str) -> Result<Vec<OtpEntry>, Box<dyn std::error::Error>> {
    let v: serde_json::Value = serde_json::from_str(json)?;
    match v {
        serde_json::Value::Object(mut map) => match map.remove("entries") {
            Some(arr) => Ok(serde_json::from_value(arr)?),
            None => Err("JSON object missing 'entries' array".into()),
        },
        serde_json::Value::Array(_) => Ok(serde_json::from_value(v)?),
        _ => Err("Expected a JSON array or { \"version\", \"entries\" } object".into()),
    }
}

/// Import a password-encrypted export (the `encrypted` format). Reads the
/// password from $MFA_PASSWORD or prompts (hidden). Not auto-detected because
/// decryption needs a secret and must stay an explicit, opt-in path.
fn import_encrypted(path: &str) -> Result<Vec<OtpEntry>, Box<dyn std::error::Error>> {
    let blob = std::fs::read_to_string(path)?;
    let password = if let Ok(pw) = std::env::var("MFA_PASSWORD") {
        pw
    } else {
        let pw = rpassword::prompt_password("Export password: ")?;
        if pw.is_empty() { return Err("Password cannot be empty".into()); }
        pw
    };
    let json = crate::crypto::encryption::decrypt(&blob, &password)?;
    parse_entries_json(&json)
}

fn import_csv(path: &str) -> Result<Vec<OtpEntry>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let mut entries = Vec::new();
    for (i, line) in content.lines().enumerate() {
        if i == 0 && line.starts_with("name,") {
            continue;
        }
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 2 {
            continue;
        }
        let entry = OtpEntry::new(
            parts[0].trim().to_string(),
            parts[1].trim().to_string(),
            parts.get(2).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            parts.get(3).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).unwrap_or_else(|| "SHA1".to_string()),
            parts.get(4).and_then(|s| s.trim().parse().ok()).unwrap_or(6),
            parts.get(5).and_then(|s| s.trim().parse().ok()).unwrap_or(30),
        )?;
        entries.push(entry);
    }
    Ok(entries)
}

/// One otpauth:// URI per line. A bad line is skipped (with a warning)
/// instead of aborting the whole batch.
fn import_otpauth(path: &str) -> Result<Vec<OtpEntry>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let mut entries = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match OtpEntry::from_otpauth_uri(line) {
            Ok(e) => entries.push(e),
            Err(e) => eprintln!("  {} skipped line {}: {}", "!".yellow(), i + 1, e),
        }
    }
    Ok(entries)
}
