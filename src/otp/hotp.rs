use crate::storage::models::OtpEntry;
use hmac::{Hmac, Mac};
use sha1::Sha1;

type HmacSha1 = Hmac<Sha1>;

/// Generate an HOTP code for the entry's current counter
pub fn generate(entry: &OtpEntry) -> Result<String, Box<dyn std::error::Error>> {
    generate_at(entry, entry.counter)
}

/// Generate an HOTP code at a specific counter value
pub fn generate_at(entry: &OtpEntry, counter: u64) -> Result<String, Box<dyn std::error::Error>> {
    let secret = base32::decode(base32::Alphabet::Rfc4648 { padding: false }, &entry.normalized_secret())
        .ok_or("Invalid base32 secret (allowed: A-Z and 2-7; common typos 0->O, 1->I, 8->B)")?;

    let counter_bytes = counter.to_be_bytes();

    let mut mac = HmacSha1::new_from_slice(&secret)?;
    mac.update(&counter_bytes);
    let hash = mac.finalize().into_bytes();

    let offset = (hash[hash.len() - 1] & 0x0f) as usize;
    let binary = ((hash[offset] & 0x7f) as u32) << 24
        | (hash[offset + 1] as u32) << 16
        | (hash[offset + 2] as u32) << 8
        | hash[offset + 3] as u32;

    let code = binary % 10u32.pow(entry.digits);
    Ok(format!("{:0>width$}", code, width = entry.digits as usize))
}
