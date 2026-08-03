//! 本机免密：把 vault 主密码托管到操作系统级机密存储（opt-in）。
//!
//! - macOS：登录 Keychain（/usr/bin/security）
//! - Windows：DPAPI（CurrentUser 作用域，经 Windows PowerShell 5.1）
//! - Linux：Secret Service（secret-tool；无头环境自动降级回密码输入）
//!
//! 所有失败均非致命：不可用时调用方回退到交互式密码输入。

const SERVICE: &str = "mfacli";
const ACCOUNT: &str = "vault";

pub fn backend_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "macOS Keychain"
    } else if cfg!(target_os = "windows") {
        "Windows DPAPI"
    } else {
        "Linux Secret Service"
    }
}

/// 免密是否生效：配置开关 on 且本次未被 --no-keychain 绕过
pub fn enabled() -> bool {
    if std::env::var("MFA_NO_KEYCHAIN").is_ok() {
        return false;
    }
    crate::config::Config::load().keychain
}

pub fn store(password: &str) -> Result<(), String> {
    imp_store(password)
}

pub fn fetch() -> Option<String> {
    imp_fetch()
}

/// 删除托管密码（幂等：不存在也返回 Ok）
pub fn delete() -> Result<(), String> {
    imp_delete()
}

// ── macOS：Keychain ───────────────────────────────────────
#[cfg(target_os = "macos")]
fn imp_store(password: &str) -> Result<(), String> {
    run(std::process::Command::new("security").args([
        "add-generic-password", "-s", SERVICE, "-a", ACCOUNT, "-w", password, "-U",
    ]))
}

#[cfg(target_os = "macos")]
fn imp_fetch() -> Option<String> {
    let out = std::process::Command::new("security")
        .args(["find-generic-password", "-s", SERVICE, "-a", ACCOUNT, "-w"])
        .output()
        .ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim_end().to_string())
    } else {
        None
    }
}

#[cfg(target_os = "macos")]
fn imp_delete() -> Result<(), String> {
    match run(std::process::Command::new("security").args([
        "delete-generic-password", "-s", SERVICE, "-a", ACCOUNT,
    ])) {
        Ok(()) => Ok(()),
        Err(e) if e.contains("could not be found") => Ok(()),
        Err(e) => Err(e),
    }
}

#[cfg(target_os = "macos")]
fn run(cmd: &mut std::process::Command) -> Result<(), String> {
    let out = cmd
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

// ── Linux：Secret Service (secret-tool) ───────────────────
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn imp_store(password: &str) -> Result<(), String> {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut child = Command::new("secret-tool")
        .args(["store", "--label", "mfacli vault", "service", SERVICE, "account", ACCOUNT])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| "secret-tool 未安装 (apt: libsecret-tools / yum: libsecret)".to_string())?;
    child
        .stdin
        .as_mut()
        .ok_or("stdin unavailable")?
        .write_all(password.as_bytes())
        .map_err(|e| e.to_string())?;
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn imp_fetch() -> Option<String> {
    let out = std::process::Command::new("secret-tool")
        .args(["lookup", "service", SERVICE, "account", ACCOUNT])
        .output()
        .ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim_end().to_string())
    } else {
        None
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn imp_delete() -> Result<(), String> {
    let out = std::process::Command::new("secret-tool")
        .args(["clear", "service", SERVICE, "account", ACCOUNT])
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

// ── Windows：DPAPI (CurrentUser) ──────────────────────────
#[cfg(target_os = "windows")]
fn blob_path() -> Result<std::path::PathBuf, String> {
    let dir = dirs::data_local_dir()
        .ok_or("cannot determine LOCALAPPDATA")?
        .join("mfacli");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("key.dpapi"))
}

#[cfg(target_os = "windows")]
fn ps_quote(p: &std::path::Path) -> String {
    p.to_string_lossy().replace('\'', "''")
}

#[cfg(target_os = "windows")]
fn imp_store(password: &str) -> Result<(), String> {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let path = ps_quote(&blob_path()?);
    let script = format!(
        "Add-Type -AssemblyName System.Security; $p=[Console]::In.ReadToEnd(); \
         $b=[System.Security.Cryptography.ProtectedData]::Protect([System.Text.Encoding]::UTF8.GetBytes($p),$null,[System.Security.Cryptography.DataProtectionScope]::CurrentUser); \
         [System.IO.File]::WriteAllBytes('{}',$b)",
        path
    );
    let mut child = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    child
        .stdin
        .as_mut()
        .ok_or("stdin unavailable")?
        .write_all(password.as_bytes())
        .map_err(|e| e.to_string())?;
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

#[cfg(target_os = "windows")]
fn imp_fetch() -> Option<String> {
    let path = ps_quote(&blob_path().ok()?);
    if !std::path::Path::new(&path.replace("''", "'")).exists() {
        return None;
    }
    let script = format!(
        "Add-Type -AssemblyName System.Security; \
         $b=[System.IO.File]::ReadAllBytes('{}'); \
         [Console]::Out.Write([System.Text.Encoding]::UTF8.GetString([System.Security.Cryptography.ProtectedData]::Unprotect($b,$null,[System.Security.Cryptography.DataProtectionScope]::CurrentUser)))",
        path
    );
    let out = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim_end().to_string())
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
fn imp_delete() -> Result<(), String> {
    match blob_path() {
        Ok(p) => {
            if p.exists() {
                std::fs::remove_file(p).map_err(|e| e.to_string())?;
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}
