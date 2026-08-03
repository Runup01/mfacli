#![allow(clippy::unnecessary_cast)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::empty_line_after_doc_comments)]
#![allow(clippy::needless_range_loop)]

mod almanac;
mod cli;
mod config;
mod crypto;
mod import;
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
            "mfa remove <name|#>".cyan()
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
        }) => cmd_add(
            &name,
            secret.as_deref(),
            issuer.as_deref(),
            &algorithm,
            digits,
            period,
        ),
        Some(Commands::Code { name, copy }) => cmd_code(&name, copy),
        Some(Commands::Copy { name }) => cmd_code(&name, true),
        Some(Commands::Show { name }) => cmd_show(&name),
        Some(Commands::Scan { path, name }) => cmd_scan(&path, name.as_deref()),
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
        Some(Commands::Remove { name }) => cmd_remove(&name),
        Some(Commands::Export { output, format }) => cmd_export(output.as_deref(), &format),
        Some(Commands::Import { source, path }) => cmd_import(source.as_deref(), &path),
        Some(Commands::Config {
            pet,
            city,
            show_weather,
            show_bazi,
            show_pet,
        }) => cmd_config(pet, city, show_weather, show_bazi, show_pet),
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
    vault.add_entry(entry)?;
    vault.save()?;
    println!("{} Added '{}'", "✓".green(), name);
    Ok(())
}

fn cmd_code(name: &str, copy: bool) -> Result<(), Box<dyn std::error::Error>> {
    let vault = storage::vault::Vault::load()?;
    let resolved = resolve_name(&vault, name)?;
    let entry = vault.get_entry(&resolved)?;
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
    let resolved = resolve_name(&vault, name)?;
    let entry = vault.get_entry(&resolved)?;

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
    println!("  {} {}", "URI:".bold(), entry.to_otpauth_uri());
    println!();
    println!("  {}", "QR Code (scan with phone authenticator):".bold());
    println!();

    let qr = utils::qrcode_util::render_to_terminal(&entry.to_otpauth_uri())?;
    for line in qr.lines() {
        println!("  {}", line);
    }

    Ok(())
}

fn cmd_scan(path: &str, name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let content = utils::qrcode_util::decode_from_image(path)?;

    if !content.starts_with("otpauth://") {
        return Err(format!("QR code does not contain an otpauth:// URI: {}", content).into());
    }

    let mut entry = storage::models::OtpEntry::from_otpauth_uri(&content)?;

    if let Some(n) = name {
        entry.name = n.to_string();
    }

    let mut vault = storage::vault::Vault::load()?;
    vault.add_entry(entry.clone())?;
    vault.save()?;

    println!("{} Scanned and added '{}'", "✓".green(), entry.name);
    if let Some(issuer) = &entry.issuer {
        println!("  Issuer: {}", issuer);
    }
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
    let total_w = 4 + name_col + 2 + issuer_col + 2 + 12 + 2 + 6;

    // ── Table header ──
    println!("  {}", "─".repeat(total_w).dimmed());
    println!(
        "  {} {} {} {} {}",
        pad_to_width("#", 2).dimmed(),
        pad_to_width("NAME", name_col).blue().bold(),
        pad_to_width("ISSUER", issuer_col).magenta(),
        pad_to_width("CODE", 12).green().bold(),
        "⏱".yellow(),
    );
    println!("  {}", "─".repeat(total_w).dimmed());

    // ── Data rows ──
    for (idx, entry) in display_entries.iter().enumerate() {
        let code = otp::generate_code(entry).unwrap_or_else(|_| "------".to_string());
        let remaining = entry.period - (now % entry.period);

        let name_display = truncate_str(&entry.name, 36);
        let issuer_raw = entry.issuer.as_deref().unwrap_or("");
        let issuer_display = truncate_str(issuer_raw, 36);

        let code_padded = pad_to_width(&code, 12);
        let code_style = if remaining <= 5 {
            code_padded.red().bold().to_string()
        } else {
            code_padded.green().bold().to_string()
        };

        let timer = if remaining <= 5 {
            format!("{}s", remaining).red().bold().to_string()
        } else {
            format!("{}s", remaining).yellow().to_string()
        };

        let num = format!("{:>2}", idx + 1);
        println!(
            "  {} {} {} {} {}",
            num.dimmed(),
            pad_to_width(&name_display, name_col).cyan(),
            pad_to_width(&issuer_display, issuer_col).magenta(),
            code_style,
            timer,
        );
    }

    println!("  {}", "─".repeat(total_w).dimmed());
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
        "mfa copy <name|#>".cyan(),
        "mfa show <name|#>".cyan(),
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

/// Resolve `<name|#>`: exact name wins; otherwise the 1-based index shown by `mfa list`.
fn resolve_name(
    vault: &storage::vault::Vault,
    arg: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let entries = vault.list_entries();
    if let Some(e) = entries.iter().find(|e| e.name == arg) {
        return Ok(e.name.clone());
    }
    if let Ok(idx) = arg.parse::<usize>() {
        let sorted = sorted_entries(entries);
        if (1..=sorted.len()).contains(&idx) {
            return Ok(sorted[idx - 1].name.clone());
        }
        return Err(
            format!("No entry #{} (valid: 1-{}, see `mfa list`)", idx, sorted.len()).into(),
        );
    }
    Err(format!("Entry '{}' not found (see `mfa list`)", arg).into())
}

fn cmd_rename(old: &str, new: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut vault = storage::vault::Vault::load()?;
    let resolved = resolve_name(&vault, old)?;
    vault.rename_entry(&resolved, new)?;
    vault.save()?;
    println!("{} Renamed '{}' → '{}'", "✓".green(), resolved, new);
    Ok(())
}

fn cmd_edit(
    name: &str,
    rename: Option<&str>,
    secret: Option<&str>,
    issuer: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut vault = storage::vault::Vault::load()?;
    let resolved = resolve_name(&vault, name)?;
    let name = resolved.as_str();

    if rename.is_none() && secret.is_none() && issuer.is_none() {
        // Note: secret = Some("") means user wants interactive prompt
    }
    let want_secret_change = secret.is_some();
    if rename.is_none() && !want_secret_change && issuer.is_none() {
        // Show current entry info if no changes specified
        let entry = vault.get_entry(name)?;
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

    // Check rename validity before mutable borrow
    if let Some(new_name) = rename {
        if vault.list_entries().iter().any(|e| e.name == new_name) {
            return Err(format!("Name '{}' already exists", new_name).into());
        }
    }

    let entry = vault.get_entry_mut(name)?;

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

fn cmd_remove(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut vault = storage::vault::Vault::load()?;
    let resolved = resolve_name(&vault, name)?;
    vault.remove_entry(&resolved)?;
    vault.save()?;
    println!("{} Removed '{}'", "✓".green(), resolved);
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

fn cmd_import(source: Option<&str>, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let entries = import::import_from(source, path)?;
    let mut vault = storage::vault::Vault::load()?;
    let mut added = 0usize;
    let mut renamed = 0usize;
    let mut skipped = 0usize;

    println!();
    println!("  {} Importing from {} ...", "→".cyan(), path);
    println!("  {}", "─".repeat(50));

    for mut entry in entries {
        let original_name = entry.name.clone();
        // Avoid name collisions: if the name already exists, append _2, _3, ...
        if vault.list_entries().iter().any(|e| e.name == entry.name) {
            let base = entry.name.clone();
            let mut n = 2u32;
            while vault
                .list_entries()
                .iter()
                .any(|e| e.name == format!("{}_{}", base, n))
            {
                n += 1;
            }
            entry.name = format!("{}_{}", base, n);
            renamed += 1;
            println!(
                "  {} {} → {} (renamed, already exists)",
                "⚠".yellow(),
                original_name,
                entry.name
            );
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
        "  {} {} added, {} renamed, {} skipped (source: {})",
        "✓".green(),
        added,
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
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = config::Config::load();
    let mut changed = false;

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
        let mut ans = String::new();
        std::io::BufRead::read_line(&mut std::io::stdin().lock(), &mut ans)?;
        if ans.trim() != "yes" {
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
    print!("  输入密码以证明你是所有者，并确认关闭 [y/N]：");
    // First prove ownership by decrypting (friendly error + escape hint on failure).
    let vault = storage::vault::Vault::load()?;
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
