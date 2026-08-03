use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::Engine;
use rand::RngCore;
use zeroize::Zeroize;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const MAGIC: &[u8; 4] = b"MFA1";
const HEADER_LEN: usize = 4 + 12; // magic + 3 x u32

/// Argon2id parameters aligned with OWASP / RFC 9106: 64 MiB, 3 passes, parallelism 4.
const DEF_M: u32 = 65536; // KiB
const DEF_T: u32 = 3;
const DEF_P: u32 = 4;

// Sanity caps for KDF params parsed from a file header (tamper/DoS guard).
const MAX_M: u32 = 1_048_576; // 1 GiB
const MAX_T: u32 = 100;
const MAX_P: u32 = 64;

fn argon2_with(m: u32, t: u32, p: u32) -> Result<Argon2<'static>, Box<dyn std::error::Error>> {
    let params = Params::new(m, t, p, None).map_err(|e| format!("Invalid KDF parameters: {}", e))?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

/// Derive a 256-bit key from password using Argon2id
fn derive_key(
    password: &str,
    salt: &[u8],
    m: u32,
    t: u32,
    p: u32,
) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let mut key = [0u8; 32];
    argon2_with(m, t, p)?
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    Ok(key)
}

/// Encrypt plaintext with password.
/// Output format: base64(MAGIC || m || t || p || salt || nonce || ciphertext)
pub fn encrypt(plaintext: &str, password: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut salt = [0u8; SALT_LEN];
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut nonce_bytes);

    let mut key = derive_key(password, &salt, DEF_M, DEF_T, DEF_P)?;
    let cipher = Aes256Gcm::new_from_slice(&key)?;
    key.zeroize();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut output = Vec::with_capacity(HEADER_LEN + SALT_LEN + NONCE_LEN + ciphertext.len());
    output.extend_from_slice(MAGIC);
    output.extend_from_slice(&DEF_M.to_be_bytes());
    output.extend_from_slice(&DEF_T.to_be_bytes());
    output.extend_from_slice(&DEF_P.to_be_bytes());
    output.extend_from_slice(&salt);
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    Ok(base64::engine::general_purpose::STANDARD.encode(&output))
}

/// Decrypt both the current headered format and the legacy (pre-0.1.5)
/// `salt || nonce || ciphertext` format, which used the argon2 crate defaults.
pub fn decrypt(encoded: &str, password: &str) -> Result<String, Box<dyn std::error::Error>> {
    let data = base64::engine::general_purpose::STANDARD.decode(encoded)?;

    let (m, t, p, body) = if data.len() >= 4 && &data[..4] == MAGIC {
        if data.len() < HEADER_LEN + SALT_LEN + NONCE_LEN + 1 {
            return Err("Invalid encrypted data".into());
        }
        let m = u32::from_be_bytes(data[4..8].try_into()?);
        let t = u32::from_be_bytes(data[8..12].try_into()?);
        let p = u32::from_be_bytes(data[12..16].try_into()?);
        if m > MAX_M || t == 0 || t > MAX_T || p == 0 || p > MAX_P {
            return Err("Unreasonable KDF parameters in encrypted data".into());
        }
        (m, t, p, &data[HEADER_LEN..])
    } else {
        let d = Params::default();
        (d.m_cost(), d.t_cost(), d.p_cost(), &data[..])
    };

    if body.len() < SALT_LEN + NONCE_LEN + 1 {
        return Err("Invalid encrypted data".into());
    }
    let salt = &body[..SALT_LEN];
    let nonce_bytes = &body[SALT_LEN..SALT_LEN + NONCE_LEN];
    let ciphertext = &body[SALT_LEN + NONCE_LEN..];

    let mut key = derive_key(password, salt, m, t, p)?;
    let cipher = Aes256Gcm::new_from_slice(&key)?;
    key.zeroize();
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed: wrong password or corrupted data")?;

    Ok(String::from_utf8(plaintext)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let plaintext = r#"{"entries": []}"#;
        let password = "test-password-123";

        let encrypted = encrypt(plaintext, password).unwrap();
        assert_ne!(encrypted, plaintext);

        let decrypted = decrypt(&encrypted, password).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_password_fails() {
        let encrypted = encrypt("secret data", "correct-password").unwrap();
        let result = decrypt(&encrypted, "wrong-password");
        assert!(result.is_err());
    }

    #[test]
    fn new_format_has_magic_header() {
        let encoded = encrypt("x", "pw").unwrap();
        let raw = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .unwrap();
        assert_eq!(&raw[..4], MAGIC);
    }

    #[test]
    fn legacy_format_still_decrypts() {
        // Simulate a pre-0.1.5 blob: salt || nonce || ct with Argon2::default()
        let password = "legacy-pw";
        let salt = [7u8; SALT_LEN];
        let nonce_b = [9u8; NONCE_LEN];
        let mut key = [0u8; 32];
        Argon2::default()
            .hash_password_into(password.as_bytes(), &salt, &mut key)
            .unwrap();
        let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
        let ct = cipher
            .encrypt(Nonce::from_slice(&nonce_b), &b"hello legacy"[..])
            .unwrap();
        let mut blob = Vec::new();
        blob.extend_from_slice(&salt);
        blob.extend_from_slice(&nonce_b);
        blob.extend_from_slice(&ct);
        let encoded = base64::engine::general_purpose::STANDARD.encode(&blob);
        assert_eq!(decrypt(&encoded, password).unwrap(), "hello legacy");
    }
}
