use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtpEntry {
    pub name: String,
    pub secret: String,
    pub issuer: Option<String>,
    pub algorithm: String,
    pub digits: u32,
    pub period: u64,
    #[serde(default = "default_otp_type")]
    pub otp_type: String,
    #[serde(default)]
    pub counter: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// Custom group (overrides the issuer-derived group); None = auto
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

pub fn today_str() -> String {
    chrono::Local::now().format("%Y-%m-%d").to_string()
}

fn default_otp_type() -> String {
    "totp".to_string()
}

impl OtpEntry {
    pub fn new(
        name: String,
        secret: String,
        issuer: Option<String>,
        algorithm: String,
        digits: u32,
        period: u64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Validate secret is valid base32
        let normalized = secret.replace([' ', '-'], "").to_uppercase();
        base32::decode(base32::Alphabet::Rfc4648 { padding: false }, &normalized)
            .ok_or("Invalid base32 secret (allowed: A-Z and 2-7; common typos 0->O, 1->I, 8->B)")?;

        if digits != 6 && digits != 8 {
            return Err("Digits must be 6 or 8".into());
        }

        if !["SHA1", "SHA256", "SHA512"].contains(&algorithm.as_str()) {
            return Err(format!("Unsupported algorithm: {}", algorithm).into());
        }

        Ok(Self {
            name,
            secret: normalized,
            issuer,
            algorithm,
            digits,
            period,
            otp_type: "totp".to_string(),
            counter: 0,
            created_at: Some(today_str()),
            group: None,
        })
    }

    /// 7 天内的记录视为"新"，UI 挂 ✦ 标签
    pub fn is_new(&self) -> bool {
        match &self.created_at {
            Some(d) => match chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d") {
                Ok(c) => {
                    let age = (chrono::Local::now().date_naive() - c).num_days();
                    (0..=7).contains(&age)
                }
                Err(_) => false,
            },
            None => false,
        }
    }
    pub fn from_otpauth_uri(uri: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let parsed = url::Url::parse(uri)?;
        if parsed.scheme() != "otpauth" {
            return Err("Not an otpauth:// URI".into());
        }

        let otp_type = parsed.host_str().unwrap_or("totp").to_string();
        let label = parsed.path().trim_start_matches('/');
        // Label parts are percent-encoded in otpauth URIs (e.g. %40 = '@');
        // decode them so names show as the user expects.
        let (issuer, name) = if let Some((iss, n)) = label.split_once(':') {
            (Some(percent_decode(iss.trim())), percent_decode(n.trim()))
        } else {
            (None, percent_decode(label.trim()))
        };

        let params: std::collections::HashMap<String, String> =
            parsed.query_pairs().into_owned().collect();

        // Providers sometimes emit lowercase/dashed base32; normalize like add/edit does
        let secret = params
            .get("secret")
            .ok_or("Missing secret in URI")?
            .replace([' ', '-'], "")
            .to_uppercase();
        let algorithm = params
            .get("algorithm")
            .cloned()
            .unwrap_or_else(|| "SHA1".to_string());
        let digits = params
            .get("digits")
            .and_then(|d| d.parse().ok())
            .unwrap_or(6);
        let period = params
            .get("period")
            .and_then(|p| p.parse().ok())
            .unwrap_or(30);
        let issuer = params.get("issuer").cloned().or(issuer);
        // mfacli extension: custom group rides as a non-standard query param;
        // other authenticator apps simply ignore unknown params.
        let group = params
            .get("group")
            .map(|g| g.trim().to_string())
            .filter(|g| !g.is_empty());
        // Some URIs (e.g. "otpauth://totp/issuer:?secret=...") carry no name
        // after the colon; fall back to the issuer so the entry is usable.
        let name = if name.is_empty() {
            issuer.clone().unwrap_or_else(|| "imported".to_string())
        } else {
            name
        };

        Ok(Self {
            name,
            secret,
            issuer,
            algorithm,
            digits,
            period,
            otp_type,
            counter: 0,
            created_at: Some(today_str()),
            group,
        })
    }

    /// Convert to otpauth:// URI
    /// Decode any percent-encoded chars left in name/issuer from old imports.
    /// Idempotent: strings without '%' pass through unchanged, so it is safe
    /// to run on every load.
    pub fn sanitize(&mut self) {
        self.name = percent_decode(&self.name);
        if let Some(i) = self.issuer.as_mut() {
            *i = percent_decode(i);
        }
    }

    #[allow(dead_code)]
    /// Base32 is case-insensitive in the wild; canonical form is uppercased, separators stripped.
    pub fn normalized_secret(&self) -> String {
        self.secret.replace([' ', '-'], "").to_uppercase()
    }

    pub fn to_otpauth_uri(&self) -> String {
        // Percent-encode label parts so round-trip (export -> import) stays
        // symmetric for names/issuers containing '@', ':', CJK, spaces, etc.
        let enc_name = pct_encode(&self.name);
        let label = match &self.issuer {
            Some(iss) => format!("{}:{}", pct_encode(iss), enc_name),
            None => enc_name,
        };
        let mut uri = format!("otpauth://{}/{}", self.otp_type, label);
        uri.push_str(&format!("?secret={}", self.secret));
        if let Some(iss) = &self.issuer {
            uri.push_str(&format!("&issuer={}", pct_encode(iss)));
        }
        // Omit spec-default params to keep the QR payload (and thus the
        // QR itself) as small as possible
        if self.algorithm != "SHA1" {
            uri.push_str(&format!("&algorithm={}", self.algorithm));
        }
        if self.digits != 6 {
            uri.push_str(&format!("&digits={}", self.digits));
        }
        if self.period != 30 {
            uri.push_str(&format!("&period={}", self.period));
        }
        if self.otp_type == "hotp" {
            uri.push_str(&format!("&counter={}", self.counter));
        }
        if let Some(g) = &self.group {
            uri.push_str(&format!("&group={}", pct_encode(g)));
        }
        uri
    }

    /// 人类可读版 URI（标签不编码）——仅用于展示；QR/导出请用 to_otpauth_uri
    pub fn to_otpauth_uri_readable(&self) -> String {
        let label = match &self.issuer {
            Some(iss) => format!("{}:{}", iss, self.name),
            None => self.name.clone(),
        };
        let mut uri = format!("otpauth://{}/{}", self.otp_type, label);
        uri.push_str(&format!("?secret={}", self.secret));
        if let Some(iss) = &self.issuer {
            uri.push_str(&format!("&issuer={}", iss));
        }
        uri.push_str(&format!("&algorithm={}", self.algorithm));
        uri.push_str(&format!("&digits={}", self.digits));
        uri.push_str(&format!("&period={}", self.period));
        if self.otp_type == "hotp" {
            uri.push_str(&format!("&counter={}", self.counter));
        }
        if let Some(g) = &self.group {
            uri.push_str(&format!("&group={}", g));
        }
        uri
    }
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Decode percent-encoded bytes (e.g. "%40" -> "@", "%E4%B8%AD" -> "中").
/// Collects raw bytes first, then UTF-8 decodes, so multi-byte chars survive.
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                out.push((h << 4) | l);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_at_sign_in_name() {
        assert_eq!(
            percent_decode("dongshu.bu%401721358628378104"),
            "dongshu.bu@1721358628378104"
        );
    }

    #[test]
    fn decodes_multibyte_utf8() {
        assert_eq!(percent_decode("%E4%B8%AD%E6%96%87"), "中文");
    }

    #[test]
    fn passthrough_when_no_percent() {
        assert_eq!(percent_decode("github"), "github");
    }

    #[test]
    fn decodes_literal_percent() {
        assert_eq!(percent_decode("a%25b"), "a%b");
    }

    #[test]
    fn sanitize_cleans_entry_name_and_issuer() {
        let mut e = OtpEntry {
            name: "dongshu.bu%401721358628378104".into(),
            secret: "JBSWY3DPEHPK3PXP".into(),
            issuer: Some("Ali%20Cloud".into()),
            algorithm: "SHA1".into(),
            digits: 6,
            period: 30,
            otp_type: "totp".into(),
            counter: 0,
            created_at: None,
            group: None,
        };
        e.sanitize();
        assert_eq!(e.name, "dongshu.bu@1721358628378104");
        assert_eq!(e.issuer.as_deref(), Some("Ali Cloud"));
    }
}

/// RFC3986 percent-encode (unreserved chars pass through).
fn pct_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        let ok = b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~';
        if ok {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{:02X}", b));
        }
    }
    out
}

/// Versioned native export container. Wrapping entries in a versioned object
/// (instead of a bare array) lets the schema evolve without breaking old files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportFile {
    pub version: u32,
    pub entries: Vec<OtpEntry>,
}

impl ExportFile {
    pub const VERSION: u32 = 1;
}
