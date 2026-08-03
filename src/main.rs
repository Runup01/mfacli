#![allow(clippy::unnecessary_cast)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::empty_line_after_doc_comments)]
#![allow(clippy::needless_range_loop)]

mod almanac;
mod cli;
mod config;
mod crypto;
mod import;
mod keychain;
mod otp;
mod pet;
mod storage;
mod tui;
mod utils;
mod weather;

use clap::Parser;
use cli::{Cli, Commands};
use colored::Colorize;

fn main() {
    // Intercept clap's own errors so we can rephrase them kindly.
    // (--help / --version also arrive as Err; print them as-is and exit 0.)
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            use clap::error::ErrorKind as K;
            if matches!(e.kind(), K::DisplayHelp | K::DisplayVersion) {
                let _ = e.print();
                return;
            }
            print_clap_error(&e);
            std::process::exit(2);
        }
    };

    if let Err(e) = run(cli) {
        print_app_error(e);
        std::process::exit(1);
    }
}

/// Rephrase clap's parse errors with actionable, security-aware hints.
fn print_clap_error(e: &clap::Error) {
    eprintln!("{e}"); // keep clap's own usage line / "did you mean" suggestions
    let msg = e.to_string();
    eprintln!();
    if msg.contains("unexpected argument")
        || msg.contains("unexpected value")
        || msg.contains("found argument")
        || msg.contains("found positional")
    {
        eprintln!(
            "  {} 像是把值放错了位置，或漏了某个选项：",
            "提示".cyan().bold()
        );
        eprintln!(
            "    • 密码 / 密钥{}写在命令行 —— 会进 history 和进程列表，是泄露面。",
            "不要".red().bold()
        );
        eprintln!(
            "      密码用回车后的隐藏弹窗，或环境变量 {}；密钥用 {} 或省略后交互输入。",
            "MFA_PASSWORD".cyan(),
            "--secret".cyan()
        );
        eprintln!("    • 看这个命令的完整用法：{}", "mfa <命令> --help".cyan());
    } else if msg.contains("required argument")
        || msg.contains("required arguments were not provided")
        || msg.contains("requires a subcommand")
    {
        eprintln!(
            "  {} 少了必填参数，运行 {} 看需要填什么。",
            "提示".cyan().bold(),
            "mfa <命令> --help".cyan()
        );
    } else if msg.contains("invalid value") || msg.contains("invalid subcommand") {
        eprintln!(
            "  {} 值或子命令不对，运行 {} 看可选项。",
            "提示".cyan().bold(),
            "mfa --help".cyan()
        );
    } else {
        eprintln!(
            "  {} 不确定用法时，运行 {} 或 {}。",
            "提示".cyan().bold(),
            "mfa --help".cyan(),
            "mfa <命令> --help".cyan()
        );
    }
}

/// Rephrase runtime (business) errors with targeted next-steps.
fn print_app_error(e: Box<dyn std::error::Error>) {
    let msg = e.to_string();
    eprintln!("{} {}", "error:".red().bold(), msg);
    if (msg.contains("not found") || msg.contains("Not found"))
        && (msg.to_lowercase().contains("entry") || msg.to_lowercase().contains("name"))
    {
        eprintln!(
            "  {} 用 {} 查看现有条目（名称区分大小写）。",
            "提示:".cyan(),
            "mfa list".cyan()
        );
    } else if msg.contains("already exists") {
        eprintln!(
            "  {} 名字已存在：改名用 {}，或先 {} 再添加。",
            "提示:".cyan(),
            "mfa edit <name> --rename <new>".cyan(),
            arg_hint("remove")
        );
    } else if msg.to_lowercase().contains("base32") {
        eprintln!(
            "  {} 批量添加可直接导入文件：{}",
            "提示:".cyan(),
            "mfa import <文件>".cyan()
        );
    } else if msg.contains("cannot be empty") {
        eprintln!("  {} 内容不能为空，已取消，未做任何更改。", "提示:".cyan());
    }
    // 解密/密码类错误本身已带"备份可恢复"的逃生提示，不重复追加。
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    if cli.no_keychain {
        std::env::set_var("MFA_NO_KEYCHAIN", "1");
    }
    match cli.command {
        None => cmd_list(None, false),
        Some(Commands::Tui) => cmd_tui(),
        Some(Commands::Init { encrypt }) => cmd_init(encrypt),
        Some(Commands::Lock { backup }) => cmd_lock(backup.as_deref()),
        Some(Commands::Unlock) => cmd_unlock(),
        Some(Commands::Add {
            name,
            secret,
            issuer,
            algorithm,
            digits,
            period,
            conflict,
        }) => cmd_add(
            &name,
            secret.as_deref(),
            issuer.as_deref(),
            &algorithm,
            digits,
            period,
            &conflict,
        ),
        Some(Commands::Code { name, copy }) => cmd_code(&name, copy),
        Some(Commands::Copy { name }) => cmd_code(&name, true),
        Some(Commands::Show { name }) => cmd_show(&name),
        Some(Commands::Scan {
            paths,
            name,
            filter,
            conflict,
        }) => cmd_scan(&paths, name.as_deref(), filter.as_deref(), &conflict),
        Some(Commands::Backup { output, plain }) => cmd_backup(output.as_deref(), plain),
        Some(Commands::Clear) => cmd_clear(),
        Some(Commands::List { limit, all }) => cmd_list(limit, all),
        Some(Commands::Edit {
            name,
            rename,
            secret,
            issuer,
        }) => cmd_edit(
            &name,
            rename.as_deref(),
            secret.as_deref(),
            issuer.as_deref(),
        ),
        Some(Commands::Rename { old, new }) => cmd_rename(&old, &new),
        Some(Commands::Remove { names, filter }) => cmd_remove(&names, filter.as_deref()),
        Some(Commands::Export { output, format }) => cmd_export(output.as_deref(), &format),
        Some(Commands::Import {
            source,
            path,
            conflict,
        }) => cmd_import(source.as_deref(), &path, &conflict),
        Some(Commands::Config {
            pet,
            city,
            show_weather,
            show_bazi,
            show_pet,
            keychain,
            reset,
        }) => cmd_config(pet, city, show_weather, show_bazi, show_pet, keychain, reset),
    }
}

fn cmd_tui() -> Result<(), Box<dyn std::error::Error>> {
    let vault = storage::vault::Vault::load()?;
    let entries = vault.list_entries().to_vec();

    if entries.is_empty() {
        println!("No entries found. Add one first:");
        println!(
            "  {}",
            "mfa add <name> --secret <SECRET> [--issuer <ISSUER>]".cyan()
        );
        println!("  {}", "mfa scan <qr-image.png>".cyan());
        return Ok(());
    }

    let config = config::Config::load();
    let mut app = tui::TuiApp::new(entries, config);
    app.run()
}

fn cmd_init(encrypt: bool) -> Result<(), Box<dyn std::error::Error>> {
    if encrypt {
        // Forward to the guarded lock flow (mandatory backup + double-confirm).
        return cmd_lock(None);
    } else {
        let vault = storage::vault::Vault::load()?;
        vault.save()?;
        println!(
            "{} Vault initialized (plain mode, file permissions 600)",
            "✓".green()
        );
        println!("  Use {} to enable encryption", "mfa init --encrypt".cyan());
    }
    Ok(())
}

fn cmd_add(
    name: &str,
    secret: Option<&str>,
    issuer: Option<&str>,
    algorithm: &str,
    digits: u32,
    period: u64,
    conflict: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Resolve secret: use the provided value, or prompt securely
    // (input is hidden and never written to shell history).
    let secret_val = match secret {
        Some(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ => {
            let s = rpassword::prompt_password("Secret (hidden, not saved to history): ")?;
            let s = s.trim().to_string();
            if s.is_empty() {
                return Err("Secret cannot be empty".into());
            }
            s
        }
    };
    let mut vault = storage::vault::Vault::load()?;
    let entry = storage::models::OtpEntry::new(
        name.to_string(),
        secret_val,
        issuer.map(|s| s.to_string()),
        algorithm.to_string(),
        digits,
        period,
    )?;
    if vault
        .list_entries()
        .iter()
        .any(|e| e.name == entry.name && e.issuer == entry.issuer)
    {
        match conflict_action(&entry.name, entry.issuer.as_deref(), conflict, "vault 中已存在")? {
            ConflictAction::Overwrite => {
                vault.replace_entry(entry);
                println!("{} Overwrote '{}'", "✓".green(), name);
            }
            ConflictAction::Skip => {
                println!("{} Skipped '{}' (already exists)", "→".yellow(), name);
                return Ok(());
            }
            ConflictAction::Rename => {
                let base = entry.name.clone();
                let iss = entry.issuer.clone();
                let mut e = entry;
                e.name = dedup_name(&vault, &base, &iss);
                vault.add_entry(e.clone())?;
                println!("{} Added as '{}' ('{}' exists)", "✓".green(), e.name, base);
            }
        }
    } else {
        vault.add_entry(entry)?;
        println!("{} Added '{}'", "✓".green(), name);
    }
    vault.save()?;
    Ok(())
}

fn cmd_code(name: &str, copy: bool) -> Result<(), Box<dyn std::error::Error>> {
    let vault = storage::vault::Vault::load()?;
    let (rn, ri) = resolve_target(&vault, name)?;
    let entry = vault.get_entry(&rn, ri.as_deref())?;
    let code = otp::generate_code(entry)?;

    if copy {
        utils::clipboard::copy_to_clipboard(&code)?;
        eprintln!("{} {} → clipboard", "✓".green(), code.green().bold());
    } else {
        println!("{}", code);
    }
    Ok(())
}

fn cmd_show(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let vault = storage::vault::Vault::load()?;
    let (rn, ri) = resolve_target(&vault, name)?;
    let entry = vault.get_entry(&rn, ri.as_deref())?;

    println!();
    println!("  {} {}", "Name:".bold(), entry.name);
    println!(
        "  {} {}",
        "Issuer:".bold(),
        entry.issuer.as_deref().unwrap_or("-")
    );
    println!("  {} {}", "Secret:".bold(), entry.secret.yellow());
    println!("  {} {}", "Algorithm:".bold(), entry.algorithm);
    println!("  {} {}", "Digits:".bold(), entry.digits);
    println!("  {} {}s", "Period:".bold(), entry.period);
    let uri = entry.to_otpauth_uri();
    println!("  {} {}", "URI:".bold(), uri);
    if uri.contains('%') {
        println!(
            "  {} {}",
            "Readable:".bold(),
            entry.to_otpauth_uri_readable()
        );
        println!(
            "  {}",
            "→ URI 中的 %XX 是中文/特殊字符的 percent-encoding（otpauth 规范要求），二维码与导入不受影响".dimmed()
        );
    }
    println!();
    println!("  {}", "QR Code (scan with phone authenticator):".bold());
    println!();

    let qr = utils::qrcode_util::render_to_terminal(&entry.to_otpauth_uri())?;
    for line in qr.lines() {
        println!("  {}", line);
    }

    Ok(())
}

enum ConflictAction {
    Overwrite,
    Rename,
    Skip,
}

/// 统一冲突策略：overwrite/skip/rename 直接生效；ask 交互询问（非 TTY 回退 rename）
fn conflict_action(
    name: &str,
    issuer: Option<&str>,
    mode: &str,
    reason: &str,
) -> Result<ConflictAction, Box<dyn std::error::Error>> {
    match mode {
        "overwrite" => return Ok(ConflictAction::Overwrite),
        "skip" => return Ok(ConflictAction::Skip),
        "rename" => return Ok(ConflictAction::Rename),
        _ => {}
    }
    use std::io::IsTerminal;
    if !std::io::stdin().is_terminal() {
        return Ok(ConflictAction::Rename);
    }
    print!(
        "  '{}' ({}) {} — [y] 覆盖 / [r] 重命名 _2 / [s] 跳过：",
        name,
        issuer.unwrap_or("-"),
        reason
    );
    std::io::Write::flush(&mut std::io::stdout())?;
    let mut ans = String::new();
    std::io::BufRead::read_line(&mut std::io::stdin().lock(), &mut ans)?;
    Ok(match ans.trim() {
        "y" | "Y" | "yes" => ConflictAction::Overwrite,
        "r" | "R" => ConflictAction::Rename,
        _ => ConflictAction::Skip,
    })
}

/// 重命名 _2 递增直到无冲突
fn dedup_name(vault: &storage::vault::Vault, base: &str, issuer: &Option<String>) -> String {
    let mut k = 2u32;
    loop {
        let cand = format!("{}_{}", base, k);
        if !vault
            .list_entries()
            .iter()
            .any(|e| e.name == cand && e.issuer == *issuer)
        {
            return cand;
        }
        k += 1;
    }
}

/// 简易过滤模式：忽略大小写；`|` 或、`*` 通配、`^`/`$` 锚点
fn filter_match(pattern: &str, text: &str) -> bool {
    let text = text.to_lowercase();
    pattern.split('|').any(|alt| {
        let alt = alt.trim();
        if alt.is_empty() {
            return false;
        }
        let anchor_start = alt.starts_with('^');
        let anchor_end = alt.ends_with('$');
        let core = alt.trim_start_matches('^').trim_end_matches('$');
        let segs: Vec<String> = core
            .split('*')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_lowercase())
            .collect();
        if segs.is_empty() {
            return false;
        }
        let mut pos = 0usize;
        for (i, seg) in segs.iter().enumerate() {
            if i == 0 && anchor_start {
                if !text[pos..].starts_with(seg) {
                    return false;
                }
                pos += seg.len();
                continue;
            }
            match text[pos..].find(seg.as_str()) {
                Some(idx) => pos = pos + idx + seg.len(),
                None => return false,
            }
        }
        if anchor_end && !text.ends_with(segs.last().unwrap().as_str()) {
            return false;
        }
        true
    })
}

fn collect_images(path: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    if path.is_dir() {
        if let Ok(rd) = std::fs::read_dir(path) {
            let mut entries: Vec<_> = rd.filter_map(|e| e.ok()).collect();
            entries.sort_by_key(|e| e.file_name());
            for e in entries {
                collect_images(&e.path(), out);
            }
        }
    } else if path.is_file() {
        let ext = path
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or("")
            .to_lowercase();
        if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "bmp") {
            out.push(path.to_path_buf());
        }
    }
}

fn cmd_scan(
    paths: &[String],
    name: Option<&str>,
    filter: Option<&str>,
    conflict: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let filter = filter.map(|f| f.to_string());

    let mut images: Vec<std::path::PathBuf> = Vec::new();
    for p in paths {
        collect_images(std::path::Path::new(p), &mut images);
    }
    if images.is_empty() {
        return Err(
            "No QR image files found (png/jpg/jpeg/webp/bmp; directories are scanned recursively)"
                .into(),
        );
    }

    let mut vault = storage::vault::Vault::load()?;
    // Snapshot of what was on disk before this run, to tell
    // "already in vault" apart from "duplicate inside this batch".
    let initial: std::collections::HashSet<(String, Option<String>)> = vault
        .list_entries()
        .iter()
        .map(|e| (e.name.clone(), e.issuer.clone()))
        .collect();
    let (mut added, mut overwritten, mut skipped, mut failed) = (0usize, 0usize, 0usize, 0usize);
    let mut report: Vec<String> = Vec::new();

    for img in &images {
        let label = img.display().to_string();
        match utils::qrcode_util::decode_from_image(&label) {
            Ok(content) if content.starts_with("otpauth://") => {
                match storage::models::OtpEntry::from_otpauth_uri(&content) {
                    Ok(mut entry) => {
                        if images.len() == 1 {
                            if let Some(n) = name {
                                entry.name = n.to_string();
                            }
                        }
                        if let Some(f) = &filter {
                            let hit = filter_match(f, &entry.name)
                                || entry.issuer.as_deref().map(|i| filter_match(f, i)).unwrap_or(false);
                            if !hit {
                                skipped += 1;
                                report.push(format!("  = {} skipped (filter miss)", entry.name));
                                continue;
                            }
                        }
                        let base = entry.name.clone();
                        let iss = entry.issuer.clone();
                        if vault
                            .list_entries()
                            .iter()
                            .any(|e| e.name == base && e.issuer == iss)
                        {
                            let reason = if initial.contains(&(base.clone(), iss.clone())) {
                                "vault 中已存在"
                            } else {
                                "本批次重复（前面已扫入同名条目）"
                            };
                            match conflict_action(&base, iss.as_deref(), conflict, reason)? {
                                ConflictAction::Skip => {
                                    skipped += 1;
                                    report.push(format!("  = {} skipped (exists)", base));
                                    continue;
                                }
                                ConflictAction::Overwrite => {
                                    vault.replace_entry(entry);
                                    overwritten += 1;
                                    report.push(format!("  ↑ {} overwritten", base));
                                    continue;
                                }
                                ConflictAction::Rename => {
                                    entry.name = dedup_name(&vault, &base, &iss);
                                }
                            }
                        }
                        let renamed = entry.name != base;
                        vault.add_entry(entry.clone())?;
                        added += 1;
                        report.push(format!(
                            "  + {}{}",
                            entry.name,
                            if renamed {
                                format!(" (dup renamed, was '{}')", base)
                            } else {
                                String::new()
                            }
                        ));
                    }
                    Err(e) => {
                        failed += 1;
                        report.push(format!("  ✗ {}: otpauth parse failed ({})", label, e));
                    }
                }
            }
            Ok(content) => {
                failed += 1;
                let head: String = content.chars().take(30).collect();
                report.push(format!("  ✗ {}: not an otpauth:// QR ({})", label, head));
            }
            Err(e) => {
                failed += 1;
                report.push(format!("  ✗ {}: no QR detected ({})", label, e));
            }
        }
    }

    if added > 0 {
        vault.save()?;
    }
    for line in &report {
        println!("{}", line);
    }
    println!(
        "{} Scan done: {} image(s) | {} added | {} overwritten | {} skipped | {} failed",
        "✓".green(),
        images.len(),
        added,
        overwritten,
        skipped,
        failed
    );
    Ok(())
}

/// 读取一行确认输入；清洗退格符 (^? / ^H)，避免终端未行缓冲时误判
fn read_yes() -> Result<bool, Box<dyn std::error::Error>> {
    let mut ans = String::new();
    std::io::BufRead::read_line(&mut std::io::stdin().lock(), &mut ans)?;
    let cleaned: String = ans
        .chars()
        .filter(|c| *c != '\u{7f}' && *c != '\u{8}')
        .collect();
    Ok(cleaned.trim() == "yes")
}

fn backups_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let dir = dirs::config_dir()
        .ok_or("Cannot determine config directory")?
        .join("mfa-cli")
        .join("backups");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn cmd_backup(output: Option<&str>, plain: bool) -> Result<(), Box<dyn std::error::Error>> {
    let vault = storage::vault::Vault::load()?;
    let n = vault.list_entries().len();
    let encrypted = dirs::config_dir()
        .map(|d| d.join("mfa-cli").join("vault.enc").exists())
        .unwrap_or(false);
    let use_plain = plain || !encrypted;

    let path = match output {
        Some(p) => std::path::PathBuf::from(p),
        None => {
            let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
            backups_dir()?.join(format!(
                "vault-{}.{}",
                ts,
                if use_plain { "json" } else { "enc" }
            ))
        }
    };

    if use_plain {
        let json = serde_json::to_string_pretty(vault.list_entries())?;
        std::fs::write(&path, json)?;
        storage::vault::Vault::set_file_permissions(&path)?;
        println!("{} Backed up {} entries (plain) → {}", "✓".green(), n, path.display().to_string().cyan());
        println!(
            "  {} Plain backup: keep off repos/cloud; restore with {}",
            "→".yellow(),
            "mfa import <path>".cyan()
        );
    } else {
        let data = vault.export_encrypted()?;
        std::fs::write(&path, data)?;
        storage::vault::Vault::set_file_permissions(&path)?;
        println!("{} Backed up {} entries (encrypted) → {}", "✓".green(), n, path.display().to_string().cyan());
        println!(
            "  {} Restore with {}",
            "→".yellow(),
            format!("mfa import -s encrypted {}", path.display()).cyan()
        );
    }
    Ok(())
}

fn cmd_clear() -> Result<(), Box<dyn std::error::Error>> {
    let mut vault = storage::vault::Vault::load()?;
    let n = vault.list_entries().len();
    if n == 0 {
        println!("{} Vault already empty.", "✓".green());
        return Ok(());
    }

    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let backup = backups_dir()?.join(format!("vault-{}-before-clear.json", ts));
    let json = serde_json::to_string_pretty(vault.list_entries())?;
    std::fs::write(&backup, json)?;
    storage::vault::Vault::set_file_permissions(&backup)?;

    println!();
    println!("  {} 即将清空全部 {} 条记录", "⚠".yellow().bold(), n);
    println!("  {} 自动备份已写入：{}", "✓".green(), backup.display().to_string().cyan());
    print!("  输入 yes 确认清空（其他取消）：");
    std::io::Write::flush(&mut std::io::stdout())?;
    if !read_yes()? {
        println!("  已取消，未做任何更改。");
        return Ok(());
    }
    vault.clear_entries();
    vault.save()?;
    println!("  {} 已清空 {} 条（备份在 {}）", "✓".green(), n, backup.display().to_string().cyan());
    Ok(())
}

/// Calculate terminal display width (CJK chars = 2, others = 1)
fn display_width(s: &str) -> usize {
    s.chars()
        .map(|c| {
            if c as u32 == 0xFE0F {
                return 0; // VS16: width carried by the base emoji
            }
            if c as u32 >= 0x1100
                && (
                    c as u32 <= 0x115f ||  // Hangul Jamo
            c as u32 == 0x2329 || c as u32 == 0x232a ||
            (c as u32 >= 0x2e80 && c as u32 <= 0x303e) ||  // CJK
            (c as u32 >= 0x3040 && c as u32 <= 0x33bf) ||  // Japanese
            (c as u32 >= 0x3400 && c as u32 <= 0x4dbf) ||  // CJK Ext A
            (c as u32 >= 0x4e00 && c as u32 <= 0xa4cf) ||  // CJK + Yi
            (c as u32 >= 0xac00 && c as u32 <= 0xd7af) ||  // Hangul
            (c as u32 >= 0xf900 && c as u32 <= 0xfaff) ||  // CJK Compat
            (c as u32 >= 0xfe30 && c as u32 <= 0xfe6f) ||  // CJK Compat Forms
            (c as u32 >= 0xff01 && c as u32 <= 0xff60) ||  // Fullwidth
            (c as u32 >= 0xffe0 && c as u32 <= 0xffe6) ||  // Fullwidth Signs
            (c as u32 >= 0x20000 && c as u32 <= 0x2fffd) || // CJK Ext B+
            (c as u32 >= 0x30000 && c as u32 <= 0x3fffd) ||
                    // CJK Ext G+
            (c as u32 >= 0x2300 && c as u32 <= 0x23ff) ||  // Misc Technical (⏱ )
            (c as u32 >= 0x2600 && c as u32 <= 0x27bf) ||  // Misc Symbols (☀️ 🌧️)
            (c as u32 >= 0x1F300 && c as u32 <= 0x1FAFF)   // Emoji (🐾 etc.)
                )
            {
                2
            } else {
                1
            }
        })
        .sum()
}

fn pad_to_width(s: &str, target: usize) -> String {
    let w = display_width(s);
    if w >= target {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(target - w))
    }
}

fn cmd_list(limit: Option<usize>, show_all: bool) -> Result<(), Box<dyn std::error::Error>> {
    let vault = storage::vault::Vault::load()?;
    let entries = vault.list_entries();

    if entries.is_empty() {
        println!();
        println!("  No entries yet. Get started:");
        println!(
            "    {} mfa add github --secret JBSWY3DPEHPK3PXP --issuer GitHub",
            "$".green()
        );
        println!("    {} mfa scan qr-code.png", "$".green());
        println!();
        return Ok(());
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    let info = almanac::get_almanac();
    let config = config::Config::load();

    // ── Header: boxed info block, inner bars aligned by display width ──
    println!();
    let left1_raw = format!("{} {} {}", info.date_str, info.weekday, info.time_str);
    let left1 = format!(
        "{} {} {}",
        info.date_str.bold(),
        info.weekday.bold(),
        info.time_str.bold()
    );
    let mut right1_raw = String::new();
    let mut right1 = String::new();
    if config.show_weather {
        if let Some(w) = weather::get_weather(config.city.as_deref()) {
            right1_raw = w.clone();
            right1 = w.green().to_string();
        }
    }
    let left2_raw = format!(
        "{}年{}月{}日 {}日",
        info.year_ganzhi, info.month_ganzhi, info.day_ganzhi, info.officer_name
    );
    let left2 = format!(
        "{}年{}月{}日 {}",
        info.year_ganzhi.yellow(),
        info.month_ganzhi.yellow(),
        info.day_ganzhi.yellow(),
        format!("{}日", info.officer_name).magenta().bold()
    );
    let right2_raw = format!("宜: {}  忌:{}", info.yi, info.ji);
    let right2 = format!(
        "{} {}  {}",
        "宜:".green(),
        info.yi.green(),
        format!("忌:{}", info.ji).red()
    );

    let mut lines: Vec<(String, String, Option<String>, String)> = vec![
        (left1, left1_raw, Some(right1), right1_raw),
    ];
    if config.show_bazi {
        lines.push((left2, left2_raw, Some(right2), right2_raw));
    }
    // Empty right (e.g. offline weather) → no inner bar on that line
    for line in lines.iter_mut() {
        if line.2.as_deref().is_none_or(str::is_empty) {
            line.2 = None;
            line.3 = String::new();
        }
    }

    let left_w = lines.iter().map(|l| display_width(&l.1)).max().unwrap_or(0);
    let right_w = lines.iter().map(|l| display_width(&l.3)).max().unwrap_or(0);
    let has_right = lines.iter().any(|l| l.2.is_some());
    let sep_w = 3; // " │ "
    let inner_w = left_w + if has_right { sep_w + right_w } else { 0 };

    let border = format!("┌{}┐", "─".repeat(inner_w + 2));
    println!("  {}", border.dimmed());
    for (left, left_raw, right, right_raw) in lines {
        let mut out = format!("  {} ", "│".dimmed());
        out.push_str(&left);
        out.push_str(&" ".repeat(left_w - display_width(&left_raw)));
        let has_r = right.is_some();
        if let Some(r) = right {
            out.push_str(&format!(" {} ", "│".dimmed()));
            out.push_str(&r);
            out.push_str(&" ".repeat(right_w - display_width(&right_raw)));
        }
        let used = left_w + if has_r { sep_w + right_w } else { 0 };
        out.push_str(&" ".repeat(inner_w - used));
        out.push_str(&format!(" {}", "│".dimmed()));
        println!("{}", out);
    }
    println!("  {}", format!("└{}┘", "─".repeat(inner_w + 2)).dimmed());
    println!();

    // ── Sort entries by issuer then name (same order as `resolve_name`) ──
    let sorted = sorted_entries(entries);

    // ── Determine display limit ──
    let max_display = if show_all {
        sorted.len()
    } else {
        limit.unwrap_or_else(|| {
            crossterm::terminal::size()
                .map(|(_, h)| (h as usize).saturating_sub(10))
                .unwrap_or(20)
        })
    };
    let display_entries = &sorted[..max_display.min(sorted.len())];
    let hidden_count = sorted.len().saturating_sub(max_display);

    // ── Calculate adaptive column widths ──
    let max_name_w = display_entries
        .iter()
        .map(|e| display_width(&truncate_str(&e.name, 36)))
        .max()
        .unwrap_or(4)
        .max(4);
    let max_issuer_w = display_entries
        .iter()
        .map(|e| {
            let iss = e.issuer.as_deref().unwrap_or("");
            // If name == issuer, show "·" (1 char)
            display_width(&truncate_str(iss, 36))
        })
        .max()
        .unwrap_or(6)
        .max(6);

    let name_col = max_name_w.clamp(16, 36);
    let issuer_col = max_issuer_w.clamp(12, 36);
    let idx_w = 5.max(sorted.len().to_string().len()); // "INDEX" header width
    let idx_digits = 2.max(sorted.len().to_string().len()); // zero-padded: 01, 02, …
    let inner_w = idx_w + 2 + name_col + 2 + issuer_col + 2 + 10 + 2 + 12 + 2 + 4;

    // ── Table header (open rules) ──
    println!("  {}", "─".repeat(inner_w).dimmed());
    println!(
        "  {}  {}  {}  {}  {}  {}",
        pad_to_width("INDEX", idx_w).dimmed(),
        pad_to_width("NAME", name_col).blue().bold(),
        pad_to_width("ISSUER", issuer_col).magenta(),
        pad_to_width("CODE", 12).green().bold(),
        pad_to_width("⏱", 4).yellow(),
        pad_to_width("ADDED", 10).bold(),
    );
    println!("  {}", "─".repeat(inner_w).dimmed());

    // ── Data rows ──
    for (idx, entry) in display_entries.iter().enumerate() {
        let (code, code_ok) = match otp::generate_code(entry) {
            Ok(c) => (c, true),
            Err(_) => ("------".to_string(), false),
        };
        let remaining = entry.period - (now % entry.period);

        let is_new = entry.is_new();
        let name_w = name_col.saturating_sub(if is_new { 2 } else { 0 });
        let name_cell = if is_new {
            format!(
                "{}{}",
                pad_to_width(&truncate_str(&entry.name, name_w), name_w).cyan(),
                " ✦".bright_green()
            )
        } else {
            pad_to_width(&truncate_str(&entry.name, name_col), name_col)
                .cyan()
                .to_string()
        };
        let issuer_raw = entry.issuer.as_deref().unwrap_or("");
        let issuer_display = truncate_str(issuer_raw, 36);
        let added = pad_to_width(entry.created_at.as_deref().unwrap_or("-"), 10);

        let code_padded = pad_to_width(&code, 12);
        let code_style = if !code_ok {
            code_padded.red().to_string()
        } else if remaining <= 5 {
            code_padded.red().bold().to_string()
        } else {
            code_padded.green().bold().to_string()
        };

        let timer = if remaining <= 5 {
            pad_to_width(&format!("{}s", remaining), 4).red().bold().to_string()
        } else {
            pad_to_width(&format!("{}s", remaining), 4).yellow().to_string()
        };

        let num = format!("{:>w$}", format!("{:0d$}", idx + 1, d = idx_digits), w = idx_w);
        println!(
            "  {}  {}  {}  {}  {}  {}",
            num.dimmed(),
            name_cell,
            pad_to_width(&issuer_display, issuer_col).magenta(),
            code_style,
            timer,
            added,
        );
    }

    println!("  {}", "─".repeat(inner_w).dimmed());
    if hidden_count > 0 {
        println!(
            "  {} ({} more, use {} to see all)",
            "…".yellow(),
            hidden_count,
            "mfa list --all".cyan()
        );
    }
    println!(
        "  {}  {}  {}  {}",
        "tip".dimmed(),
        arg_hint("copy"),
        arg_hint("show"),
        "mfa tui".cyan(),
    );
    println!();
    Ok(())
}

/// Truncate a string to max display width (CJK-aware, char-safe)
fn truncate_str(s: &str, max_w: usize) -> String {
    let w = display_width(s);
    if w <= max_w {
        return s.to_string();
    }
    let truncated: String = s
        .chars()
        .scan(0usize, |acc, c| {
            let cw = if (c as u32) > 0x7f { 2 } else { 1 };
            if *acc + cw > max_w - 1 {
                None
            } else {
                *acc += cw;
                Some(c)
            }
        })
        .collect();
    format!("{}…", truncated)
}

/// Sort entries in the order `mfa list` displays them (issuer → name).
fn sorted_entries(entries: &[storage::models::OtpEntry]) -> Vec<&storage::models::OtpEntry> {
    let mut sorted: Vec<&storage::models::OtpEntry> = entries.iter().collect();
    sorted.sort_by(|a, b| {
        let ia = a.issuer.as_deref().unwrap_or("");
        let ib = b.issuer.as_deref().unwrap_or("");
        ia.to_lowercase()
            .cmp(&ib.to_lowercase())
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    sorted
}

/// Resolve `<name|index>` to a (name, issuer) identity.
/// Exact unique name wins; ambiguous names must use the index; otherwise 1-based index.
fn resolve_target(
    vault: &storage::vault::Vault,
    arg: &str,
) -> Result<(String, Option<String>), Box<dyn std::error::Error>> {
    let entries = vault.list_entries();
    let d = 2.max(entries.len().to_string().len()); // 与 list 显示一致的补零宽度
    let matches: Vec<&storage::models::OtpEntry> =
        entries.iter().filter(|e| e.name == arg).collect();
    if matches.len() == 1 {
        return Ok((matches[0].name.clone(), matches[0].issuer.clone()));
    }
    if matches.len() > 1 {
        let sorted = sorted_entries(entries);
        let idxs: Vec<String> = sorted
            .iter()
            .enumerate()
            .filter(|(_, e)| e.name == arg)
            .map(|(i, _)| format!("{:0d$}", i + 1, d = d))
            .collect();
        return Err(format!(
            "Name '{}' matches {} entries (INDEX {}); use the index instead",
            arg,
            matches.len(),
            idxs.join(", ")
        )
        .into());
    }
    if let Ok(idx) = arg.parse::<usize>() {
        let sorted = sorted_entries(entries);
        if (1..=sorted.len()).contains(&idx) {
            return Ok((sorted[idx - 1].name.clone(), sorted[idx - 1].issuer.clone()));
        }
        return Err(
            format!(
                "Invalid index {} (valid: {:0d$}-{:0d$}, see `mfa list`)",
                idx, 1, sorted.len(), d = d
            )
            .into(),
        );
    }
    Err(format!("Entry '{}' not found (see `mfa list`)", arg).into())
}

/// Styled `<name|index>` hint with the word highlighted for visibility.
fn arg_hint(cmd: &str) -> String {
    format!(
        "{}{}{}",
        format!("mfa {} <name|", cmd).cyan(),
        "index".yellow().bold(),
        ">".cyan()
    )
}

fn cmd_rename(old: &str, new: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut vault = storage::vault::Vault::load()?;
    let (rn, ri) = resolve_target(&vault, old)?;
    vault.rename_entry(&rn, ri.as_deref(), new)?;
    vault.save()?;
    println!("{} Renamed '{}' → '{}'", "✓".green(), rn, new);
    Ok(())
}

fn cmd_edit(
    name: &str,
    rename: Option<&str>,
    secret: Option<&str>,
    issuer: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut vault = storage::vault::Vault::load()?;
    let (rn, ri) = resolve_target(&vault, name)?;
    let name = rn.as_str();
    let issuer_q = ri.clone();

    if rename.is_none() && secret.is_none() && issuer.is_none() {
        // Note: secret = Some("") means user wants interactive prompt
    }
    let want_secret_change = secret.is_some();
    if rename.is_none() && !want_secret_change && issuer.is_none() {
        // Show current entry info if no changes specified
        let entry = vault.get_entry(name, issuer_q.as_deref())?;
        println!();
        println!("  {} {}", "Name:".bold(), entry.name);
        println!(
            "  {} {}",
            "Issuer:".bold(),
            entry.issuer.as_deref().unwrap_or("-")
        );
        println!("  {} {}", "Secret:".bold(), entry.secret.yellow());
        println!("  {} {}", "Algorithm:".bold(), entry.algorithm);
        println!();
        println!(
            "  Usage: mfa edit {} --rename <new> --secret --issuer <new>",
            name
        );
        println!("  (use --secret without a value to type it securely, hidden)");
        println!();
        return Ok(());
    }

    // Check rename validity before mutable borrow (same-issuer identity)
    if let Some(new_name) = rename {
        let eff_issuer: Option<String> = match issuer {
            Some(i) => {
                if i.is_empty() {
                    None
                } else {
                    Some(i.to_string())
                }
            }
            None => ri.clone(),
        };
        if vault
            .list_entries()
            .iter()
            .any(|e| e.name == new_name && e.issuer == eff_issuer)
        {
            return Err(format!("Name '{}' already exists", new_name).into());
        }
    }

    let entry = vault.get_entry_mut(name, issuer_q.as_deref())?;

    if let Some(new_name) = rename {
        entry.name = new_name.to_string();
    }
    if let Some(s) = secret {
        let raw = if s.is_empty() {
            // Interactive hidden input (not saved to shell history)
            let pw = rpassword::prompt_password("New secret (hidden): ")?;
            let pw = pw.trim().to_string();
            if pw.is_empty() {
                return Err("Secret cannot be empty".into());
            }
            pw
        } else {
            s.to_string()
        };
        let normalized = raw.replace([' ', '-'], "").to_uppercase();
        base32::decode(base32::Alphabet::Rfc4648 { padding: false }, &normalized)
            .ok_or("Invalid base32 secret (allowed: A-Z and 2-7; common typos 0->O, 1->I, 8->B)")?;
        entry.secret = normalized;
    }
    if let Some(i) = issuer {
        entry.issuer = if i.is_empty() {
            None
        } else {
            Some(i.to_string())
        };
    }

    vault.save()?;
    println!("{} Updated '{}'", "✓".green(), name);
    Ok(())
}

fn cmd_remove(names: &[String], filter: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    if names.is_empty() && filter.is_none() {
        return Err("提供条目名称/序号，或用 --filter 模式批量删除（mfa remove --help 看用法）".into());
    }
    let mut vault = storage::vault::Vault::load()?;

    // Resolve all targets first; abort entirely if any is invalid (no partial deletes)
    let mut resolved: Vec<(String, Option<String>)> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    for arg in names {
        match resolve_target(&vault, arg) {
            Ok(pair) => {
                if !resolved.contains(&pair) {
                    resolved.push(pair);
                }
            }
            Err(e) => errors.push(e.to_string()),
        }
    }
    if !errors.is_empty() {
        return Err(errors.join("; ").into());
    }

    // --filter 批量删：列命中 → 自动备份 → yes 确认
    if let Some(pat) = filter {
        let hits: Vec<(String, Option<String>)> = vault
            .list_entries()
            .iter()
            .filter(|e| {
                filter_match(pat, &e.name)
                    || e.issuer.as_deref().map(|i| filter_match(pat, i)).unwrap_or(false)
            })
            .map(|e| (e.name.clone(), e.issuer.clone()))
            .collect();
        if hits.is_empty() {
            println!("{} 模式 '{}' 未命中任何条目，无改动", "✓".green(), pat);
        } else {
            println!();
            println!(
                "  {} 模式 '{}' 命中 {} 条：",
                "⚠".yellow().bold(),
                pat,
                hits.len()
            );
            for (n, i) in &hits {
                println!("    - {} ({})", n, i.as_deref().unwrap_or("-"));
            }
            let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
            let backup = backups_dir()?.join(format!("vault-{}-before-remove.json", ts));
            let json = serde_json::to_string_pretty(vault.list_entries())?;
            std::fs::write(&backup, json)?;
            storage::vault::Vault::set_file_permissions(&backup)?;
            println!("  {} 自动备份已写入：{}", "✓".green(), backup.display().to_string().cyan());
            print!("  输入 yes 确认删除（其他取消）：");
            std::io::Write::flush(&mut std::io::stdout())?;
            if !read_yes()? {
                println!("  已取消，未做任何更改。");
                return Ok(());
            }
            for pair in hits {
                if !resolved.contains(&pair) {
                    resolved.push(pair);
                }
            }
        }
    }

    if resolved.is_empty() {
        return Ok(());
    }

    for (n, i) in &resolved {
        vault.remove_entry(n, i.as_deref())?;
    }
    vault.save()?;
    let names_str: Vec<&str> = resolved.iter().map(|(n, _)| n.as_str()).collect();
    if resolved.len() == 1 {
        println!("{} Removed '{}'", "✓".green(), names_str[0]);
    } else {
        println!(
            "{} Removed {} entries: {}",
            "✓".green(),
            resolved.len(),
            names_str.join(", ")
        );
    }
    Ok(())
}

fn cmd_export(output: Option<&str>, format: &str) -> Result<(), Box<dyn std::error::Error>> {
    let vault = storage::vault::Vault::load()?;
    let entries = vault.list_entries();

    let write_out = |data: &str| -> Result<(), Box<dyn std::error::Error>> {
        match output {
            Some(path) => {
                std::fs::write(path, data)?;
                storage::vault::Vault::set_file_permissions(&std::path::PathBuf::from(path))?;
            }
            None => print!("{}", data),
        }
        Ok(())
    };

    match format {
        "json" => {
            let file = storage::models::ExportFile {
                version: storage::models::ExportFile::VERSION,
                entries: entries.to_vec(),
            };
            let data = serde_json::to_string_pretty(&file)?;
            write_out(&data)?;
            if let Some(p) = output {
                println!(
                    "{} Exported {} entries as {} → {}",
                    "✓".green(),
                    entries.len(),
                    "json".cyan(),
                    p
                );
            } else {
                eprintln!(
                    "{} {} entries as {}",
                    "✓".green(),
                    entries.len(),
                    "json".cyan()
                );
            }
        }
        "encrypted" => {
            let data = vault.export_encrypted()?;
            write_out(&data)?;
            if let Some(p) = output {
                println!(
                    "{} Exported {} entries as {} → {}",
                    "✓".green(),
                    entries.len(),
                    "encrypted".cyan(),
                    p
                );
            } else {
                eprintln!(
                    "{} {} entries as {}",
                    "✓".green(),
                    entries.len(),
                    "encrypted".cyan()
                );
            }
        }
        _ => {
            // otpauth (default) — the universal, re-importable template
            let mut lines: Vec<String> = Vec::new();
            let mut skipped_steam = 0usize;
            for e in entries {
                if e.otp_type == "steam" {
                    skipped_steam += 1;
                    continue;
                }
                lines.push(e.to_otpauth_uri());
            }
            let data = if lines.is_empty() {
                String::new()
            } else {
                lines.join("\n") + "\n"
            };
            write_out(&data)?;
            let n = lines.len();
            if let Some(p) = output {
                println!(
                    "{} Exported {} entries as {} → {}",
                    "✓".green(),
                    n,
                    "otpauth".cyan(),
                    p
                );
            } else {
                eprintln!("{} {} entries as {}", "✓".green(), n, "otpauth".cyan());
            }
            if skipped_steam > 0 {
                eprintln!(
                    "  {} {} Steam entries skipped (not in otpauth standard; use {} to keep them)",
                    "!".yellow(),
                    skipped_steam,
                    "--format json".cyan()
                );
            }
        }
    }
    Ok(())
}

fn cmd_import(source: Option<&str>, path: &str, conflict: &str) -> Result<(), Box<dyn std::error::Error>> {
    let entries = import::import_from(source, path)?;
    let mut vault = storage::vault::Vault::load()?;
    let mut added = 0usize;
    let mut renamed = 0usize;
    let mut overwritten = 0usize;
    let mut skipped = 0usize;
    // Snapshot identities that existed before this import, to explain rename reasons
    let pre_existing: Vec<(String, Option<String>)> = vault
        .list_entries()
        .iter()
        .map(|e| (e.name.clone(), e.issuer.clone()))
        .collect();

    println!();
    println!("  {} Importing from {} ...", "→".cyan(), path);
    println!("  {}", "─".repeat(50));

    for mut entry in entries {
        let original_name = entry.name.clone();
        // Identity = (name, issuer): same name with a different issuer is a distinct entry
        if vault
            .list_entries()
            .iter()
            .any(|e| e.name == entry.name && e.issuer == entry.issuer)
        {
            let base = entry.name.clone();
            let iss = entry.issuer.clone();
            match conflict_action(&base, iss.as_deref(), conflict, "vault 中已存在")? {
                ConflictAction::Skip => {
                    skipped += 1;
                    println!("  = {} skipped (exists)", original_name);
                    continue;
                }
                ConflictAction::Overwrite => {
                    vault.replace_entry(entry.clone());
                    overwritten += 1;
                    println!("  ↑ {} overwritten", original_name);
                    continue;
                }
                ConflictAction::Rename => {
                    entry.name = dedup_name(&vault, &base, &iss);
                    renamed += 1;
                    let reason =
                        if pre_existing.contains(&(original_name.clone(), entry.issuer.clone())) {
                            "renamed, already exists in vault"
                        } else {
                            "renamed, duplicate in import file"
                        };
                    println!(
                        "  {} {} → {} ({})",
                        "⚠".yellow(),
                        original_name,
                        entry.name,
                        reason
                    );
                }
            }
        }
        match vault.add_entry(entry.clone()) {
            Ok(()) => {
                added += 1;
                let issuer = entry.issuer.as_deref().unwrap_or("-");
                println!("  {} {:<16} {}", "✓".green(), entry.name, issuer.dimmed());
            }
            Err(e) => {
                skipped += 1;
                println!(
                    "  {} {:<16} {}",
                    "✗".red(),
                    entry.name,
                    e.to_string().dimmed()
                );
            }
        }
    }

    vault.save()?;
    println!("  {}", "─".repeat(50));
    let src = source.unwrap_or("auto");
    println!(
        "  {} {} added, {} overwritten, {} renamed, {} skipped (source: {})",
        "✓".green(),
        added,
        overwritten,
        renamed,
        skipped,
        src
    );
    println!();
    Ok(())
}

fn cmd_config(
    pet: Option<String>,
    city: Option<String>,
    show_weather: Option<bool>,
    show_bazi: Option<bool>,
    show_pet: Option<bool>,
    keychain: Option<bool>,
    reset: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = config::Config::load();
    let mut changed = false;

    if reset {
        config = config::Config::default();
        let _ = keychain::delete();
        println!(
            "{} 设置已恢复默认 (pet=robot, weather/bazi/pet=ON, keychain=off；钥匙串托管已清除)",
            "✓".green()
        );
        changed = true;
    }

    if let Some(p) = &pet {
        config.set_pet(p)?;
        println!("{} Pet set to '{}'", "✓".green(), p);
        changed = true;
    }

    if let Some(c) = &city {
        config.city = Some(c.clone());
        println!("{} City set to '{}'", "✓".green(), c);
        changed = true;
    }

    if let Some(v) = show_weather {
        config.show_weather = v;
        println!("{} Weather: {}", "✓".green(), if v { "on" } else { "off" });
        changed = true;
    }

    if let Some(v) = show_bazi {
        config.show_bazi = v;
        println!("{} BaZi: {}", "✓".green(), if v { "on" } else { "off" });
        changed = true;
    }

    if let Some(v) = show_pet {
        config.show_pet = v;
        println!(
            "{} Pet display: {}",
            "✓".green(),
            if v { "on" } else { "off" }
        );
        changed = true;
    }

    if let Some(v) = keychain {
        config.keychain = v;
        if !v {
            let _ = keychain::delete();
        }
        println!(
            "{} 本机免密 ({}): {}",
            "✓".green(),
            keychain::backend_name(),
            if v { "on" } else { "off" }
        );
        if v {
            println!(
                "  {} 下次输入密码后自动托管到系统钥匙串；单次绕过: {}；off 即清除托管密码",
                "→".yellow(),
                "mfa --no-keychain list".cyan()
            );
        }
        changed = true;
    }

    if !changed {
        // Detect encryption status
        let encrypted = dirs::config_dir()
            .map(|d| d.join("mfa-cli").join("vault.enc").exists())
            .unwrap_or(false);

        println!();
        println!("  {} {}", "Pet:".bold(), config.pet.cyan());
        println!(
            "  {} {}",
            "City:".bold(),
            config.city.as_deref().unwrap_or("Auto (IP)").cyan()
        );
        println!(
            "  {} {}",
            "Weather:".bold(),
            if config.show_weather {
                "ON".green().to_string()
            } else {
                "OFF".red().to_string()
            }
        );
        println!(
            "  {} {}",
            "BaZi:".bold(),
            if config.show_bazi {
                "ON".green().to_string()
            } else {
                "OFF".red().to_string()
            }
        );
        println!(
            "  {} {}",
            "Pet Display:".bold(),
            if config.show_pet {
                "ON".green().to_string()
            } else {
                "OFF".red().to_string()
            }
        );
        println!(
            "  {} {}",
            "Keychain:".bold(),
            if config.keychain {
                format!("ON ({})", keychain::backend_name()).green().to_string()
            } else {
                "OFF".red().to_string()
            }
        );
        println!(
            "  {} {}",
            "应用锁:".bold(),
            if encrypted {
                "已启用 (每次访问需密码)".green().to_string()
            } else {
                "未启用".yellow().to_string()
            }
        );
        println!(
            "  {} {}",
            "  提示:".dimmed(),
            if encrypted {
                "忘记密码无法找回；务必保留备份 json (mfa import 可恢复)"
                    .dimmed()
                    .to_string()
            } else {
                "启用: mfa lock    关闭: mfa unlock".dimmed().to_string()
            }
        );
        println!();
        println!("  {} robot, dino, cat, ghost, dragon", "Pets:".bold());
        println!("  {} mfa init --encrypt", "Enable encryption:".bold());
        println!();
    } else {
        config.save()?;
    }

    Ok(())
}

fn cmd_lock(backup: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    if storage::vault::Vault::lock_status() == storage::vault::LockStatus::Locked {
        println!(
            "{} 应用锁已启用，无需重复加锁。关闭请用 {}",
            "✓".green(),
            "mfa unlock".cyan()
        );
        return Ok(());
    }

    println!();
    println!("  {} {}", "⚠".yellow().bold(), "即将启用应用锁".bold());
    println!("  启用后，每次访问 (list / tui / code / copy …) 都需要密码。");
    println!(
        "  {} 密码一旦忘记，加密的 vault.enc {}——没有后门、无法找回。",
        "!".red().bold(),
        "将永久无法打开".red().bold()
    );
    println!(
        "  所以下一步会先写一份{}，作为你的逃生绳。",
        "明文备份".cyan()
    );
    println!();

    let entries = storage::vault::Vault::read_plain_entries()?;

    // 1) 备份先行（硬步骤，不备份不加锁）
    let backup_path = storage::vault::Vault::backup_plain_json(&entries, backup)?;
    println!(
        "  {} 明文备份已写入：{}",
        "✓".green(),
        backup_path.display().to_string().cyan()
    );
    println!(
        "  {} 这是{}，请转存到密码管理器 / 离线介质，{}放仓库或云盘同步目录。",
        "→".yellow(),
        "明文".red().bold(),
        "勿".red().bold()
    );
    println!(
        "  {} 只要它在，忘密码也能用 {} 恢复。",
        "→".yellow(),
        format!("mfa import {}", backup_path.display()).cyan()
    );
    println!();

    // 2) 两次确认密码，必须完全一致
    let pw1 = rpassword::prompt_password("  设置锁密码: ")?;
    if pw1.is_empty() {
        return Err("密码不能为空".into());
    }
    if pw1.len() < 8 {
        println!("  {} 密码少于 8 位，记错 / 被撞风险高。", "!".yellow());
        print!("  仍要继续请输入 yes：");
        std::io::Write::flush(&mut std::io::stdout())?;
        if !read_yes()? {
            return Err("已取消加锁（备份已保留，可手动删除）".into());
        }
    }
    let pw2 = rpassword::prompt_password("  再次输入确认: ")?;
    if pw1 != pw2 {
        return Err("两次密码不一致，未做任何更改（备份已保留）".into());
    }

    // 3) 加密迁移
    storage::vault::Vault::write_encrypted(&entries, &pw1)?;
    storage::vault::Vault::delete_plain()?;

    // 4) 免密已开启则顺带托管新密码
    if config::Config::load().keychain {
        match keychain::store(&pw1) {
            Ok(()) => println!(
                "  {} 密码已同步存入 {}，后续免输入",
                "✓".green(),
                keychain::backend_name()
            ),
            Err(e) => println!("  {} 钥匙串写入失败: {}", "⚠".yellow(), e),
        }
    } else {
        println!(
            "  {} 想本机免输密码: {} (密码存入 {}，vault 文件被偷也打不开)",
            "→".yellow(),
            "mfa config --keychain on".cyan(),
            keychain::backend_name()
        );
    }

    println!();
    println!("  {} {}", "✓".green(), "应用锁已启用".bold());
    println!("  从现在起，每次访问都需要密码。");
    println!(
        "  {} 脚本 / 自动化可设环境变量 {} 免交互（该 shell 内等同常驻解锁，自行权衡）。",
        "→".yellow(),
        "MFA_PASSWORD".cyan()
    );
    println!("  {} 想取消：{}", "→".yellow(), "mfa unlock".cyan());
    println!();
    Ok(())
}

fn cmd_unlock() -> Result<(), Box<dyn std::error::Error>> {
    if storage::vault::Vault::lock_status() != storage::vault::LockStatus::Locked {
        println!("{} 当前未启用应用锁。", "✓".green());
        return Ok(());
    }
    println!();
    println!(
        "  {} 关闭应用锁后，密钥将以{}存于 vault.json (仅权限 600 保护)。",
        "⚠".yellow().bold(),
        "明文".red().bold()
    );
    println!("  共享 / 公共电脑请勿关闭。");
    println!();
    println!("  第 1/2 步：输入当前锁密码，证明你是所有者");
    // First prove ownership by decrypting (friendly error + escape hint on failure).
    let vault = storage::vault::Vault::load()?;
    println!("  {} 密码验证通过", "✓".green());
    print!("  第 2/2 步：确认关闭应用锁 [y/N]：");
    std::io::Write::flush(&mut std::io::stdout())?;
    let mut ans = String::new();
    std::io::BufRead::read_line(&mut std::io::stdin().lock(), &mut ans)?;
    if ans.trim().to_lowercase() != "y" {
        println!("  已取消，应用锁保持启用。");
        return Ok(());
    }
    let entries = vault.list_entries().to_vec();
    storage::vault::Vault::write_plain(&entries)?;
    storage::vault::Vault::delete_enc()?;
    println!("  {} {}", "✓".green(), "应用锁已关闭".bold());
    println!(
        "  数据已明文落盘 (权限 600)。如需重新保护：{}",
        "mfa lock".cyan()
    );
    println!();
    Ok(())
}
