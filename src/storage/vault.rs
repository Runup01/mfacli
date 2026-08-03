use super::models::OtpEntry;
use crate::crypto::encryption;
use std::path::PathBuf;

pub struct Vault {
    entries: Vec<OtpEntry>,
    path: PathBuf,
    encrypted: bool,
}

impl Vault {
    fn vault_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir().ok_or("Cannot determine config directory")?;
        let vault_dir = config_dir.join("mfa-cli");
        std::fs::create_dir_all(&vault_dir)?;
        Ok(vault_dir)
    }

    /// Load vault from disk. Auto-detects encrypted vs plain mode.
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let dir = Self::vault_dir()?;
        let enc_path = dir.join("vault.enc");
        let plain_path = dir.join("vault.json");

        if enc_path.exists() {
            let password = Self::get_password()?;
            let encrypted = std::fs::read_to_string(&enc_path)?;
            let json = encryption::decrypt(&encrypted, &password).map_err(|_| "密码错误，或 vault.enc 已损坏。若忘了密码但还有备份 json，可用 `mfa import <备份.json>` 恢复数据。")?;
            let mut entries: Vec<OtpEntry> = serde_json::from_str(&json)?;
            for e in entries.iter_mut() {
                e.sanitize();
            }
            Ok(Self {
                entries,
                path: enc_path,
                encrypted: true,
            })
        } else if plain_path.exists() {
            let json = std::fs::read_to_string(&plain_path)?;
            let mut entries: Vec<OtpEntry> = serde_json::from_str(&json)?;
            for e in entries.iter_mut() {
                e.sanitize();
            }
            Ok(Self {
                entries,
                path: plain_path,
                encrypted: false,
            })
        } else {
            Ok(Self {
                entries: Vec::new(),
                path: plain_path,
                encrypted: false,
            })
        }
    }

    /// Save vault to disk
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self.entries)?;

        if self.encrypted {
            let password = Self::get_password()?;
            let encrypted = encryption::encrypt(&json, &password)?;
            std::fs::write(&self.path, encrypted)?;
        } else {
            std::fs::write(&self.path, &json)?;
            Self::set_file_permissions(&self.path)?;
        }
        Ok(())
    }

    /// Initialize vault with encryption (legacy;  now uses the guarded flow)
    #[allow(dead_code)]
    pub fn init_encrypted() -> Result<Self, Box<dyn std::error::Error>> {
        let dir = Self::vault_dir()?;
        let enc_path = dir.join("vault.enc");
        let plain_path = dir.join("vault.json");

        // Migrate existing plain vault if present
        let entries = if plain_path.exists() {
            let json = std::fs::read_to_string(&plain_path)?;
            serde_json::from_str(&json)?
        } else {
            Vec::new()
        };

        let vault = Self {
            entries,
            path: enc_path,
            encrypted: true,
        };
        vault.save()?;

        // Remove plain file after successful migration
        if plain_path.exists() {
            std::fs::remove_file(&plain_path)?;
        }

        Ok(vault)
    }

    /// Get password from env var or interactive prompt
    fn get_password() -> Result<String, Box<dyn std::error::Error>> {
        if let Ok(pw) = std::env::var("MFA_PASSWORD") {
            return Ok(pw);
        }
        let password = rpassword::prompt_password("Vault password: ")?;
        if password.is_empty() {
            return Err("Password cannot be empty".into());
        }
        Ok(password)
    }

    /// Set file permissions to 600 (owner read/write only)
    #[cfg(unix)]
    pub fn set_file_permissions(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(path, perms)?;
        Ok(())
    }

    #[cfg(not(unix))]
    pub fn set_file_permissions(_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    pub fn add_entry(&mut self, entry: OtpEntry) -> Result<(), Box<dyn std::error::Error>> {
        if self.entries.iter().any(|e| e.name == entry.name) {
            return Err(format!("Entry '{}' already exists", entry.name).into());
        }
        self.entries.push(entry);
        Ok(())
    }

    pub fn get_entry(&self, name: &str) -> Result<&OtpEntry, Box<dyn std::error::Error>> {
        self.entries
            .iter()
            .find(|e| e.name == name)
            .ok_or_else(|| format!("Entry '{}' not found", name).into())
    }

    pub fn get_entry_mut(
        &mut self,
        name: &str,
    ) -> Result<&mut OtpEntry, Box<dyn std::error::Error>> {
        self.entries
            .iter_mut()
            .find(|e| e.name == name)
            .ok_or_else(|| format!("Entry '{}' not found", name).into())
    }

    pub fn remove_entry(&mut self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let idx = self
            .entries
            .iter()
            .position(|e| e.name == name)
            .ok_or_else(|| format!("Entry '{}' not found", name))?;
        self.entries.remove(idx);
        Ok(())
    }

    pub fn list_entries(&self) -> &[OtpEntry] {
        &self.entries
    }

    pub fn export_encrypted(&self) -> Result<String, Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self.entries)?;
        let password = Self::get_password()?;
        encryption::encrypt(&json, &password)
    }
}

impl Vault {
    pub fn rename_entry(&mut self, old: &str, new: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.entries.iter().any(|e| e.name == new) {
            return Err(format!("Entry '{}' already exists", new).into());
        }
        let entry = self
            .entries
            .iter_mut()
            .find(|e| e.name == old)
            .ok_or_else(|| format!("Entry '{}' not found", old))?;
        entry.name = new.to_string();
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum LockStatus {
    Locked,
    Unlocked,
    Empty,
}

impl Vault {
    fn enc_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        Ok(Self::vault_dir()?.join("vault.enc"))
    }
    fn plain_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        Ok(Self::vault_dir()?.join("vault.json"))
    }

    /// Lock state from file presence only — never reads content or prompts.
    pub fn lock_status() -> LockStatus {
        let enc = Self::enc_path().map(|p| p.exists()).unwrap_or(false);
        let plain = Self::plain_path().map(|p| p.exists()).unwrap_or(false);
        if enc {
            LockStatus::Locked
        } else if plain {
            LockStatus::Unlocked
        } else {
            LockStatus::Empty
        }
    }

    /// Read the plain vault's entries (no password). Used by the lock flow,
    /// which only runs when the vault is NOT yet encrypted.
    pub fn read_plain_entries() -> Result<Vec<OtpEntry>, Box<dyn std::error::Error>> {
        let p = Self::plain_path()?;
        if !p.exists() {
            return Ok(Vec::new());
        }
        let json = std::fs::read_to_string(&p)?;
        let mut entries: Vec<OtpEntry> = serde_json::from_str(&json)?;
        for e in entries.iter_mut() {
            e.sanitize();
        }
        Ok(entries)
    }

    /// Encrypt `entries` with `password` and write vault.enc (no prompt).
    pub fn write_encrypted(
        entries: &[OtpEntry],
        password: &str,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(entries)?;
        let enc = encryption::encrypt(&json, password)?;
        let path = Self::enc_path()?;
        std::fs::write(&path, enc)?;
        Self::set_file_permissions(&path)?;
        Ok(path)
    }

    /// Write entries as plain vault.json with mode 600 (no prompt).
    pub fn write_plain(entries: &[OtpEntry]) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(entries)?;
        let path = Self::plain_path()?;
        std::fs::write(&path, &json)?;
        Self::set_file_permissions(&path)?;
        Ok(path)
    }

    pub fn delete_plain() -> Result<(), Box<dyn std::error::Error>> {
        let p = Self::plain_path()?;
        if p.exists() {
            std::fs::remove_file(p)?;
        }
        Ok(())
    }
    pub fn delete_enc() -> Result<(), Box<dyn std::error::Error>> {
        let p = Self::enc_path()?;
        if p.exists() {
            std::fs::remove_file(p)?;
        }
        Ok(())
    }

    /// Write a PLAIN json backup (the escape hatch). This is the file that
    /// saves you if the lock password is ever forgotten.
    pub fn backup_plain_json(
        entries: &[OtpEntry],
        custom: Option<&str>,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(entries)?;
        let path = match custom {
            Some(p) => PathBuf::from(p),
            None => {
                let ts = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
                Self::vault_dir()?.join(format!("vault.backup-{}.json", ts))
            }
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, &json)?;
        Self::set_file_permissions(&path)?;
        Ok(path)
    }
}
