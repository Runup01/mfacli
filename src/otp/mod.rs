pub mod hotp;
pub mod steam;
pub mod totp;

use crate::storage::models::OtpEntry;

/// Generate the current OTP code for an entry
pub fn generate_code(entry: &OtpEntry) -> Result<String, Box<dyn std::error::Error>> {
    match entry.otp_type.as_str() {
        "totp" => totp::generate(entry),
        "hotp" => hotp::generate(entry),
        "steam" => steam::generate(entry),
        other => Err(format!("Unsupported OTP type: {}", other).into()),
    }
}
