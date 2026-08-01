use crate::storage::models::OtpEntry;
use hmac::{Hmac, Mac};
use sha1::Sha1;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha1 = Hmac<Sha1>;

const STEAM_CHARS: &[u8] = b"23456789BCDFGHJKMNPQRTVWXY";

/// Generate a Steam Guard code
pub fn generate(entry: &OtpEntry) -> Result<String, Box<dyn std::error::Error>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let counter = now / 30;

    let secret = base32::decode(base32::Alphabet::Rfc4648 { padding: false }, &entry.secret)
        .ok_or("Invalid base32 secret (allowed: A-Z and 2-7; common typos 0->O, 1->I, 8->B)")?;

    let counter_bytes = counter.to_be_bytes();

    let mut mac = HmacSha1::new_from_slice(&secret)?;
    mac.update(&counter_bytes);
    let hash = mac.finalize().into_bytes();

    let offset = (hash[hash.len() - 1] & 0x0f) as usize;
    let mut full_code = ((hash[offset] & 0x7f) as u32) << 24
        | (hash[offset + 1] as u32) << 16
        | (hash[offset + 2] as u32) << 8
        | hash[offset + 3] as u32;

    let mut code = String::with_capacity(5);
    for _ in 0..5 {
        code.push(STEAM_CHARS[(full_code % 26) as usize] as char);
        full_code /= 26;
    }

    Ok(code)
}
