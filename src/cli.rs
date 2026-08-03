use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "mfa",
    version,
    about = "A developer-friendly CLI MFA/OTP manager"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Launch interactive TUI (default when no command given)
    Tui,

    /// Initialize vault (optionally with encryption)
    Init {
        /// Enable AES-256-GCM encryption for the vault
        #[arg(long)]
        encrypt: bool,
    },

    /// Enable the app lock (every access requires the password)
    Lock {
        /// Where to write the mandatory plain-text escape-hatch backup
        #[arg(short, long)]
        backup: Option<String>,
    },

    /// Disable the app lock (decrypts vault back to plain, mode 600)
    Unlock,

    /// Add a new OTP entry
    Add {
        /// Unique name for this entry
        name: String,

        /// Base32-encoded secret key (omit to type it securely, hidden)
        #[arg(short, long)]
        secret: Option<String>,

        /// Issuer name
        #[arg(short, long)]
        issuer: Option<String>,

        /// Hash algorithm: SHA1, SHA256, SHA512
        #[arg(short, long, default_value = "SHA1")]
        algorithm: String,

        /// Number of digits (6 or 8)
        #[arg(short, long, default_value_t = 6)]
        digits: u32,

        /// Time period in seconds
        #[arg(short, long, default_value_t = 30)]
        period: u64,
    },

    /// Generate the current OTP code
    Code {
        /// Entry name or index (# column of `mfa list`)
        name: String,

        /// Copy code to clipboard
        #[arg(short, long)]
        copy: bool,
    },

    /// Copy OTP code to clipboard
    Copy {
        /// Entry name or index (# column of `mfa list`)
        name: String,
    },

    /// Show entry details: secret + QR code
    Show {
        /// Entry name or index (# column of `mfa list`)
        name: String,
    },

    /// Scan a QR code image to add an entry
    Scan {
        /// Path to QR code image (PNG/JPG)
        path: String,

        /// Custom name for the entry
        #[arg(short, long)]
        name: Option<String>,
    },

    /// List all entries with current codes
    List {
        /// Max entries to display (default: terminal height - 8)
        #[arg(short, long)]
        limit: Option<usize>,

        /// Show all entries (no limit)
        #[arg(long)]
        all: bool,
    },

    /// Rename an entry (shortcut for `edit <name> --rename <new>`)
    Rename {
        /// Current entry name or index (# column of `mfa list`)
        old: String,

        /// New name
        new: String,
    },

    /// Edit an entry (name, secret, issuer)
    Edit {
        /// Entry name or index (# column of `mfa list`)
        name: String,

        /// New name
        #[arg(short, long)]
        rename: Option<String>,

        /// New secret key (omit value to type it securely, hidden)
        #[arg(short, long, num_args = 0..=1, default_missing_value = "")]
        secret: Option<String>,

        /// New issuer
        #[arg(short, long)]
        issuer: Option<String>,
    },

    /// Remove an entry
    Remove {
        /// Entry name or index (# column of `mfa list`)
        name: String,
    },

    /// Export entries (encrypted)
    Export {
        /// Output file path (prints to stdout if omitted)
        #[arg(short, long)]
        output: Option<String>,

        /// Format: otpauth (default, universal), json (full-fidelity), encrypted (password-protected)
        #[arg(short, long, default_value = "otpauth")]
        format: String,
    },

    /// Import entries from another tool
    Import {
        /// File to import (format auto-detected when --source is omitted)
        path: String,

        /// Force source format: google, json, csv, otpauth
        #[arg(short, long)]
        source: Option<String>,
    },

    /// Configure TUI appearance (pet, weather, bazi)
    Config {
        /// Set pet style: robot, dino, cat, ghost, dragon
        #[arg(long)]
        pet: Option<String>,

        /// Set weather city (default: auto IP detection)
        #[arg(long)]
        city: Option<String>,

        /// Toggle weather display
        #[arg(long)]
        show_weather: Option<bool>,

        /// Toggle BaZi/almanac display
        #[arg(long)]
        show_bazi: Option<bool>,

        /// Toggle pet display
        #[arg(long)]
        show_pet: Option<bool>,
    },
}
