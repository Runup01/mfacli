use crate::storage::models::OtpEntry;
use base64::Engine;

/// Import from Google Authenticator migration QR code data
///
/// Google Authenticator exports use a protobuf-encoded format inside an
/// otpauth-migration:// URI. This is a simplified parser for the common case.
pub fn import(path: &str) -> Result<Vec<OtpEntry>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let uri = content.trim();

    if !uri.starts_with("otpauth-migration://offline?data=") {
        return Err("Expected otpauth-migration:// URI".into());
    }

    let data_param = uri
        .strip_prefix("otpauth-migration://offline?data=")
        .ok_or("Invalid migration URI")?;

    let decoded = base64::engine::general_purpose::URL_SAFE.decode(data_param)?;

    // Parse protobuf wire format (simplified)
    parse_migration_protobuf(&decoded)
}

/// Simplified protobuf parser for Google Authenticator migration data
fn parse_migration_protobuf(data: &[u8]) -> Result<Vec<OtpEntry>, Box<dyn std::error::Error>> {
    let mut entries = Vec::new();
    let mut pos = 0;

    while pos < data.len() {
        // Field 1 (repeated OtpParameters) - wire type 2 (length-delimited)
        if data[pos] == 0x0a {
            pos += 1;
            let (len, bytes_read) = read_varint(&data[pos..])?;
            pos += bytes_read;

            if pos + len as usize > data.len() {
                break;
            }

            let entry_data = &data[pos..pos + len as usize];
            if let Ok(entry) = parse_otp_parameters(entry_data) {
                entries.push(entry);
            }
            pos += len as usize;
        } else {
            pos += 1;
        }
    }

    if entries.is_empty() {
        return Err("No entries found in migration data".into());
    }

    Ok(entries)
}

fn parse_otp_parameters(data: &[u8]) -> Result<OtpEntry, Box<dyn std::error::Error>> {
    let mut secret = Vec::new();
    let mut name = String::new();
    let mut issuer = String::new();
    let mut algorithm = "SHA1".to_string();
    let mut digits = 6u32;

    let mut pos = 0;
    while pos < data.len() {
        let field_byte = data[pos];
        pos += 1;

        match field_byte {
            0x0a => {
                // Field 1: secret (bytes)
                let (len, bytes_read) = read_varint(&data[pos..])?;
                pos += bytes_read;
                secret = data[pos..pos + len as usize].to_vec();
                pos += len as usize;
            }
            0x12 => {
                // Field 2: name (string)
                let (len, bytes_read) = read_varint(&data[pos..])?;
                pos += bytes_read;
                name = String::from_utf8_lossy(&data[pos..pos + len as usize]).to_string();
                pos += len as usize;
            }
            0x1a => {
                // Field 3: issuer (string)
                let (len, bytes_read) = read_varint(&data[pos..])?;
                pos += bytes_read;
                issuer = String::from_utf8_lossy(&data[pos..pos + len as usize]).to_string();
                pos += len as usize;
            }
            0x20 => {
                // Field 4: algorithm (enum)
                let (val, bytes_read) = read_varint(&data[pos..])?;
                pos += bytes_read;
                algorithm = match val {
                    1 => "SHA1".to_string(),
                    2 => "SHA256".to_string(),
                    3 => "SHA512".to_string(),
                    _ => "SHA1".to_string(),
                };
            }
            0x28 => {
                // Field 5: digits (enum)
                let (val, bytes_read) = read_varint(&data[pos..])?;
                pos += bytes_read;
                digits = if val == 2 { 8 } else { 6 };
            }
            _ => {
                // Skip unknown fields
                break;
            }
        }
    }

    let secret_b32 = base32::encode(base32::Alphabet::Rfc4648 { padding: false }, &secret);

    Ok(OtpEntry {
        name: if name.is_empty() {
            "unknown".to_string()
        } else {
            name
        },
        secret: secret_b32,
        issuer: if issuer.is_empty() {
            None
        } else {
            Some(issuer)
        },
        algorithm,
        digits,
        period: 30,
        otp_type: "totp".to_string(),
        counter: 0,
        created_at: Some(crate::storage::models::today_str()),
        group: None,
    })
}

fn read_varint(data: &[u8]) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    let mut result: usize = 0;
    let mut shift = 0;
    let mut bytes_read = 0;

    for &byte in data.iter() {
        bytes_read += 1;
        result |= ((byte & 0x7f) as usize) << shift;
        if byte & 0x80 == 0 {
            return Ok((result, bytes_read));
        }
        shift += 7;
        if shift >= 64 {
            return Err("Varint too long".into());
        }
    }

    Err("Unexpected end of varint".into())
}
