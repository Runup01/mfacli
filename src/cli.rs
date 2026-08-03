use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "mfa",
    version,
    about = "A developer-friendly, local-first MFA/OTP manager"
)]
pub struct Cli {
    /// Bypass OS keychain for this run and enter the password manually
    #[arg(long, global = true)]
    pub no_keychain: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

fn parse_on_off(s: &str) -> Result<bool, String> {
    match s.to_lowercase().as_str() {
        "on" | "true" | "yes" => Ok(true),
        "off" | "false" | "no" => Ok(false),
        other => Err(format!("expected on/off (or true/false), got '{other}'")),
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Launch the interactive TUI (default when no command is given)
    Tui,

    /// Initialize the vault (optionally encrypted)
    Init {
        /// Enable AES-256-GCM encryption
        #[arg(long)]
        encrypt: bool,
    },

    /// Enable the app lock (every access will require the password)
    Lock {
        /// Where to write the mandatory plain-text escape-hatch backup
        #[arg(short, long)]
        backup: Option<String>,
    },

    /// Disable the app lock (vault is decrypted back to plain text, mode 600)
    Unlock,

    /// Add a new OTP entry
    Add {
        /// Unique name for this entry
        name: String,

        /// Base32-encoded secret key (omit to type it hidden)
        #[arg(short, long)]
        secret: Option<String>,

        /// Issuer name (e.g. GitHub)
        #[arg(short, long)]
        issuer: Option<String>,

        /// Hash algorithm: SHA1 / SHA256 / SHA512
        #[arg(short, long, default_value = "SHA1")]
        algorithm: String,

        /// Number of digits (6 or 8)
        #[arg(short, long, default_value_t = 6)]
        digits: u32,

        /// Time period in seconds
        #[arg(short, long, default_value_t = 30)]
        period: u64,

        /// Policy when the entry already exists: ask (default) / rename (auto _2) / skip / overwrite
        #[arg(short, long, default_value = "ask", value_parser = ["ask", "rename", "skip", "overwrite"])]
        conflict: String,
    },

    /// Generate the current OTP code (entry by name or index)
    Code {
        /// Entry name or index (INDEX column of `mfa list`)
        name: String,

        /// Copy the code to the clipboard
        #[arg(short, long)]
        copy: bool,
    },

    /// Copy the current OTP code to the clipboard (entry by name or index)
    Copy {
        /// Entry name or index (INDEX column of `mfa list`)
        name: String,
    },

    /// Show entry details: secret + QR code (entry by name or index)
    Show {
        /// Entry name or index (INDEX column of `mfa list`)
        name: String,
    },

    /// Scan QR image(s) to add entries (batch: multiple paths, dirs recursive)
    Scan {
        /// QR image or directory paths (PNG/JPG/WebP; directories scanned recursively)
        #[arg(required = true)]
        paths: Vec<String>,

        /// Custom name for the entry (single file only)
        #[arg(short, long)]
        name: Option<String>,

        /// Only import entries whose name/issuer match (supports | or, * wildcard, ^/$ anchors; case-insensitive)
        #[arg(short, long)]
        filter: Option<String>,

        /// Policy when the entry already exists: ask (default) / rename (auto _2) / skip / overwrite
        #[arg(short, long, default_value = "ask", value_parser = ["ask", "rename", "skip", "overwrite"])]
        conflict: String,
    },

    /// List all entries with their current codes
    List {
        /// Max entries to display (default: terminal height - 8)
        #[arg(short, long)]
        limit: Option<usize>,

        /// Show all entries (no limit)
        #[arg(long)]
        all: bool,
    },

    /// Rename an entry (shortcut for `edit --rename`)
    Rename {
        /// Current entry name or index (INDEX column of `mfa list`)
        old: String,

        /// New name
        new: String,
    },

    /// Edit an entry: name / secret / issuer (entry by name or index)
    Edit {
        /// Entry name or index (INDEX column of `mfa list`)
        name: String,

        /// New name
        #[arg(short, long)]
        rename: Option<String>,

        /// New secret key (omit the value to type it hidden)
        #[arg(short, long, num_args = 0..=1, default_missing_value = "")]
        secret: Option<String>,

        /// New issuer
        #[arg(short, long)]
        issuer: Option<String>,
    },

    /// Remove entries: by name/index, or bulk delete with --filter
    Remove {
        /// Entry names or indexes (INDEX column of `mfa list`), space-separated
        names: Vec<String>,

        /// Bulk delete all entries whose name/issuer match (supports | or, * wildcard, ^/$ anchors; auto-backup + yes confirm)
        #[arg(short, long)]
        filter: Option<String>,
    },

    /// Export entries
    Export {
        /// Output file path (prints to stdout if omitted)
        #[arg(short, long)]
        output: Option<String>,

        /// Export format: otpauth (default, universal) / json (full fidelity) / encrypted (password-protected)
        #[arg(short, long, default_value = "otpauth")]
        format: String,
    },

    /// Import entries from another tool
    Import {
        /// File to import (format auto-detected when --source is omitted)
        path: String,

        /// Force source format: google / json / csv / otpauth
        #[arg(short, long)]
        source: Option<String>,

        /// Policy when the entry already exists: ask (default) / rename (auto _2) / skip / overwrite
        #[arg(short, long, default_value = "ask", value_parser = ["ask", "rename", "skip", "overwrite"])]
        conflict: String,
    },

    /// One-click backup (timestamped; an encrypted vault stays encrypted)
    Backup {
        /// Output path (default: auto-timestamped file in the config dir)
        #[arg(short, long)]
        output: Option<String>,

        /// Force a plain-text escape-hatch backup (use with care)
        #[arg(long)]
        plain: bool,
    },

    /// Wipe ALL entries (auto-backup first, then type yes to confirm)
    Clear,

    /// Configure appearance and behavior (pet / weather / almanac / keychain)
    Config {
        /// Pet style: robot / dino / cat / ghost / dragon
        #[arg(long)]
        pet: Option<String>,

        /// Weather city (default: auto IP geolocation)
        #[arg(long)]
        city: Option<String>,

        /// Show weather (on/off)
        #[arg(long, value_parser = parse_on_off)]
        show_weather: Option<bool>,

        /// Show BaZi almanac (on/off)
        #[arg(long, value_parser = parse_on_off)]
        show_bazi: Option<bool>,

        /// Show pet (on/off)
        #[arg(long, value_parser = parse_on_off)]
        show_pet: Option<bool>,

        /// Store the vault password in the OS keychain (macOS Keychain / Windows DPAPI / Linux Secret Service)
        #[arg(long, value_parser = parse_on_off)]
        keychain: Option<bool>,

        /// QR render style: half = compact half-blocks (default) / block = full-blocks (for terminals that distort half-blocks)
        #[arg(long, value_parser = ["half", "block"])]
        qr_style: Option<String>,

        /// Restore all settings to defaults
        #[arg(long)]
        reset: bool,
    },
}
