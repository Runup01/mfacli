use crate::storage::models::OtpEntry;
use hmac::{Hmac, Mac};
use sha1::Sha1;
use sha2::{Sha256, Sha512};
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha1 = Hmac<Sha1>;
type HmacSha256 = Hmac<Sha256>;
type HmacSha512 = Hmac<Sha512>;

/// Generate a TOTP code for the current time
pub fn generate(entry: &OtpEntry) -> Result<String, Box<dyn std::error::Error>> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let counter = now / entry.period;
    generate_at(entry, counter)
}

/// Generate a TOTP code for a specific counter value (useful for testing)
pub fn generate_at(entry: &OtpEntry, counter: u64) -> Result<String, Box<dyn std::error::Error>> {
    let secret = base32::decode(base32::Alphabet::Rfc4648 { padding: false }, &entry.secret)
        .ok_or("Invalid base32 secret (allowed: A-Z and 2-7; common typos 0->O, 1->I, 8->B)")?;

    let counter_bytes = counter.to_be_bytes();

    let hash = match entry.algorithm.as_str() {
        "SHA1" => {
            let mut mac = HmacSha1::new_from_slice(&secret)?;
            mac.update(&counter_bytes);
            mac.finalize().into_bytes().to_vec()
        }
        "SHA256" => {
            let mut mac = HmacSha256::new_from_slice(&secret)?;
            mac.update(&counter_bytes);
            mac.finalize().into_bytes().to_vec()
        }
        "SHA512" => {
            let mut mac = HmacSha512::new_from_slice(&secret)?;
            mac.update(&counter_bytes);
            mac.finalize().into_bytes().to_vec()
        }
        other => return Err(format!("Unsupported algorithm: {}", other).into()),
    };

    let code = dynamic_truncation(&hash, entry.digits);
    Ok(format!("{:0>width$}", code, width = entry.digits as usize))
}

/// RFC 4226 dynamic truncation
fn dynamic_truncation(hash: &[u8], digits: u32) -> u32 {
    let offset = (hash[hash.len() - 1] & 0x0f) as usize;
    let binary = ((hash[offset] & 0x7f) as u32) << 24
        | (hash[offset + 1] as u32) << 16
        | (hash[offset + 2] as u32) << 8
        | hash[offset + 3] as u32;

    binary % 10u32.pow(digits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::models::OtpEntry;

    #[test]
    fn test_totp_sha1() {
        // RFC 6238 test vector: secret = "12345678901234567890" (ASCII)
        // Base32 of that = GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ
        let entry = OtpEntry::new(
            "test".to_string(),
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ".to_string(),
            None,
            "SHA1".to_string(),
            8,
            30,
        )
        .unwrap();

        // Counter = 59 / 30 = 1
        let code = generate_at(&entry, 1).unwrap();
        assert_eq!(code, "94287082");
    }
}
