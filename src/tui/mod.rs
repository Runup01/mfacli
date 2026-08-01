use crate::almanac;
use crate::config::Config;
use crate::otp;
use crate::pet::{self, PetMood};
use crate::storage::models::OtpEntry;
use crate::storage::vault::Vault;
use crate::utils::clipboard;
use crate::weather;
use crossterm::{
    event::{MouseButton, MouseEventKind},
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{DisableMouseCapture, EnableMouseCapture},
    ExecutableCommand,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io::stdout;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Terminal display width: CJK = 2, most others = 1.
/// Handles "…" (U+2026) as 1-width (matches most terminal fonts).
fn tw(s: &str) -> usize {
    s.chars().map(|c| {
        let cp = c as u32;
        if cp == 0x2026 || cp == 0x2022 || cp == 0x00B7 { 1 } // … • ·
        else if cp >= 0x1100 && (
            cp <= 0x115f || cp == 0x2329 || cp == 0x232a ||
            (cp >= 0x2e80 && cp <= 0x303e) ||
            (cp >= 0x3040 && cp <= 0x33bf) ||
            (cp >= 0x3400 && cp <= 0x4dbf) ||
            (cp >= 0x4e00 && cp <= 0xa4cf) ||
            (cp >= 0xac00 && cp <= 0xd7af) ||
            (cp >= 0xf900 && cp <= 0xfaff) ||
            (cp >= 0xfe30 && cp <= 0xfe6f) ||
            (cp >= 0xff01 && cp <= 0xff60) ||
            (cp >= 0xffe0 && cp <= 0xffe6) ||
            (cp >= 0x20000 && cp <= 0x2fffd) ||
            (cp >= 0x30000 && cp <= 0x3fffd)
        ) { 2 } else { 1 }
    }).sum()
}

fn trunc(s: &str, max: usize) -> String {
    if tw(s) <= max { return s.to_string(); }
    let t: String = s.chars().scan(0usize, |w, c| {
        let cp = c as u32;
        let cw = if cp == 0x2026 || cp == 0x2022 || cp == 0x00B7 { 1 }
            else if cp >= 0x1100 && (cp <= 0x115f || cp == 0x2329 || cp == 0x232a ||
                (cp >= 0x2e80 && cp <= 0x303e) || (cp >= 0x3040 && cp <= 0x33bf) ||
                (cp >= 0x3400 && cp <= 0x4dbf) || (cp >= 0x4e00 && cp <= 0xa4cf) ||
                (cp >= 0xac00 && cp <= 0xd7af) || (cp >= 0xf900 && cp <= 0xfaff) ||
                (cp >= 0xfe30 && cp <= 0xfe6f) || (cp >= 0xff01 && cp <= 0xff60) ||
                (cp >= 0xffe0 && cp <= 0xffe6) || (cp >= 0x20000 && cp <= 0x2fffd) ||
                (cp >= 0x30000 && cp <= 0x3fffd)) { 2 } else { 1 };
        if *w + cw > max - 1 { None } else { *w += cw; Some(c) }
    }).collect();
    format!("{}…", t)
}

fn pad(s: &str, w: usize) -> String {
    let dw = tw(s);
    if dw >= w { s.to_string() } else { format!("{}{}", s, " ".repeat(w - dw)) }
}


#[derive(PartialEq)]
enum Mode {
    Normal,
    AddName,
    AddSecret,
    AddIssuer,
    Rename,
    ConfirmDelete,
    Settings,
    ViewQR,
    EditMenu,
    EditSecret,
    EditName,
    EditIssuer,
    ImportPath,
    ExportPath,
}

pub struct TuiApp {
    entries: Vec<OtpEntry>,
    list_state: ListState,
    status_message: Option<(String, StatusKind)>,
    should_quit: bool,
    config: Config,
    mode: Mode,
    input_buffer: String,
    // Add flow temp data
    add_name: String,
    add_secret: String,
    add_issuer: String,
    // Decoration
    weather_text: Option<String>,
    weather_rx: Option<Receiver<Option<String>>>,
    almanac_info: almanac::AlmanacInfo,
    pet_frame: usize,
    pet_mood: PetMood,
    last_tick: Instant,
    mood_timer: Option<Instant>,
    settings_cursor: usize,
    qr_lines: Vec<String>,
    // Mouse double-click tracking
    last_click_time: Option<Instant>,
    list_area: Rect,}

enum StatusKind {
    Success,
    Error,
    Info,
}

const SETTINGS_ITEMS: [&str; 9] = [
    "Pet Style",
    "Toggle Weather",
    "Toggle BaZi",
    "Toggle Pet Display",
    "Set City",
    "Import (otpauth:// file)",
    "Export (encrypted backup)",
    "Toggle Encryption",
    "Close Settings",
];

impl TuiApp {
    pub fn new(entries: Vec<OtpEntry>, config: Config) -> Self {
        let mut list_state = ListState::default();
        if !entries.is_empty() {
            list_state.select(Some(0));
        }

        let almanac_info = almanac::get_almanac();

        let weather_rx = if config.show_weather {
            Some(weather::spawn_weather_fetch(config.city.clone()))
        } else {
            None
        };

        Self {
            entries,
            list_state,
            status_message: None,
            should_quit: false,
            config,
            mode: Mode::Normal,
            input_buffer: String::new(),
            add_name: String::new(),
            add_secret: String::new(),
            add_issuer: String::new(),
            weather_text: None,
            weather_rx,
            almanac_info,
            pet_frame: 0,
            pet_mood: PetMood::Idle(0),
            last_tick: Instant::now(),
            mood_timer: None,
            settings_cursor: 0,
            qr_lines: Vec::new(),
            last_click_time: None,
            list_area: Rect::default(),        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        stdout().execute(EnableMouseCapture)?;
        let backend = ratatui::backend::CrosstermBackend::new(stdout());
        let mut terminal = Terminal::new(backend)?;

        let result = self.event_loop(&mut terminal);

        stdout().execute(DisableMouseCapture)?;        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        // Save config on exit
        let _ = self.config.save();

        result
    }

    fn event_loop(
        &mut self,
        terminal: &mut Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            self.tick();
            terminal.draw(|f| self.render(f))?;

            if self.should_quit {
                break;
            }

            if event::poll(Duration::from_millis(200))? {
                match event::read()? {
                    Event::Key(key) => {
                        if key.kind == KeyEventKind::Press {
                            self.handle_key(key.code);
                        }
                    }
                    Event::Mouse(mouse)
                        if mouse.kind == MouseEventKind::Down(MouseButton::Left) =>
                    {
                            let now = Instant::now();
                            // Double-click detection: two clicks within 400ms
                            if let Some(last) = self.last_click_time {
                                if now.duration_since(last) < Duration::from_millis(400) {
                                    // Map mouse Y to list item index
                                    let area = self.list_area;
                                    if mouse.row > area.y && mouse.row < area.y + area.height - 1
                                        && mouse.column >= area.x && mouse.column < area.x + area.width
                                    {
                                        let idx = (mouse.row - area.y - 1) as usize;
                                        if idx < self.entries.len() {
                                            self.list_state.select(Some(idx));
                                            self.copy_selected();
                                        }
                                    }
                                    self.last_click_time = None;
                                } else {
                                    self.last_click_time = Some(now);
                                }
                            } else {
                                self.last_click_time = Some(now);
                            }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn tick(&mut self) {
        let now = Instant::now();

        if now.duration_since(self.last_tick) >= Duration::from_millis(500) {
            self.last_tick = now;
            self.pet_frame = (self.pet_frame + 1) % 2;

            if let Some(timer) = self.mood_timer {
                if now.duration_since(timer) >= Duration::from_secs(2) {
                    self.pet_mood = PetMood::Idle(self.pet_frame);
                    self.mood_timer = None;
                }
            } else if self.mode == Mode::Normal {
                self.pet_mood = PetMood::Idle(self.pet_frame);
            }
        }

        if let Some(rx) = &self.weather_rx {
            if let Ok(result) = rx.try_recv() {
                self.weather_text = result;
                self.weather_rx = None;
            }
        }

        self.almanac_info = almanac::get_almanac();
    }

    fn handle_key(&mut self, key: KeyCode) {
        match self.mode {
            Mode::Normal => self.handle_normal(key),
            Mode::AddName | Mode::AddSecret | Mode::AddIssuer | Mode::Rename => {
                self.handle_input(key)
            }
            Mode::ConfirmDelete => self.handle_confirm_delete(key),
            Mode::Settings => self.handle_settings(key),
            Mode::ViewQR => {
                if matches!(key, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('v') | KeyCode::Enter) {
                    self.mode = Mode::Normal;
                }
            }
            Mode::EditMenu => self.handle_edit_menu(key),
            Mode::EditSecret | Mode::EditName | Mode::EditIssuer | Mode::ImportPath | Mode::ExportPath => self.handle_input(key),
        }
    }

    fn handle_normal(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('c') | KeyCode::Enter => self.copy_selected(),
            KeyCode::Up | KeyCode::Char('k') => self.move_up(),
            KeyCode::Down | KeyCode::Char('j') => self.move_down(),
            KeyCode::Char('a') => {
                self.mode = Mode::AddName;
                self.input_buffer.clear();
                self.status_message = Some(("Enter name for new entry:".to_string(), StatusKind::Info));
            }
            KeyCode::Char('d') => {
                if self.list_state.selected().is_some() {
                    self.mode = Mode::ConfirmDelete;
                    let name = self.entries[self.list_state.selected().unwrap()].name.clone();
                    self.status_message = Some((format!("Delete '{}'? [y/N]", name), StatusKind::Error));
                }
            }
            KeyCode::Char('r') => {
                if let Some(idx) = self.list_state.selected() {
                    self.input_buffer = self.entries[idx].name.clone();
                    self.mode = Mode::Rename;
                    self.status_message = Some(("New name:".to_string(), StatusKind::Info));
                }
            }
            KeyCode::Char('v') => {
                self.show_qr_overlay();
            }
            KeyCode::Char('e') => {
                if self.list_state.selected().is_some() {
                    self.mode = Mode::EditMenu;
                }
            }
            KeyCode::Char('s') | KeyCode::Tab => {
                self.mode = Mode::Settings;
                self.settings_cursor = 0;
            }
            KeyCode::Home | KeyCode::Char('g') => {
                if !self.entries.is_empty() {
                    self.list_state.select(Some(0));
                }
            }
            KeyCode::End | KeyCode::Char('G') if !self.entries.is_empty() => {
                self.list_state.select(Some(self.entries.len() - 1));
            }
            _ => {}
        }
    }

    fn handle_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.status_message = None;
            }
            KeyCode::Enter => {
                let value = self.input_buffer.clone();
                match self.mode {
                    Mode::AddName => {
                        self.add_name = value;
                        self.input_buffer.clear();
                        self.mode = Mode::AddSecret;
                        self.status_message = Some(("Enter secret (base32):".to_string(), StatusKind::Info));
                    }
                    Mode::AddSecret => {
                        self.add_secret = value;
                        self.input_buffer.clear();
                        self.mode = Mode::AddIssuer;
                        self.status_message = Some(("Enter issuer (optional, Enter to skip):".to_string(), StatusKind::Info));
                    }
                    Mode::AddIssuer => {
                        self.add_issuer = value;
                        self.finish_add();
                    }
                    Mode::Rename => {
                        self.finish_rename(&value);
                    }
                    Mode::EditSecret => {
                        self.finish_edit_secret(&value);
                    }
                    Mode::EditName => {
                        self.finish_edit_name(&value);
                    }
                    Mode::EditIssuer => {
                        self.finish_edit_issuer(&value);
                    }
                    Mode::ImportPath => {
                        self.finish_import(&value);
                    }
                    Mode::ExportPath => {
                        self.finish_export(&value);
                    }
                    _ => {}
                }
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            _ => {}
        }
    }

    fn handle_confirm_delete(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(idx) = self.list_state.selected() {
                    let name = self.entries[idx].name.clone();
                    self.entries.remove(idx);
                    self.save_vault();
                    if self.entries.is_empty() {
                        self.list_state.select(None);
                    } else if idx >= self.entries.len() {
                        self.list_state.select(Some(self.entries.len() - 1));
                    }
                    self.status_message = Some((format!("Deleted '{}'", name), StatusKind::Success));
                }
                self.mode = Mode::Normal;
            }
            _ => {
                self.mode = Mode::Normal;
                self.status_message = None;
            }
        }
    }

    fn handle_settings(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Tab => {
                self.mode = Mode::Normal;
                let _ = self.config.save();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.settings_cursor > 0 {
                    self.settings_cursor -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.settings_cursor < SETTINGS_ITEMS.len() - 1 {
                    self.settings_cursor += 1;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.apply_setting();
            }
            _ => {}
        }
    }

    fn apply_setting(&mut self) {
        match self.settings_cursor {
            0 => {
                // Cycle pet style
                let pets = ["robot", "dino", "cat", "ghost", "dragon"];
                let current = pets.iter().position(|p| *p == self.config.pet).unwrap_or(0);
                let next = (current + 1) % pets.len();
                self.config.pet = pets[next].to_string();
                self.status_message = Some((format!("Pet: {}", pets[next]), StatusKind::Success));
            }
            1 => {
                self.config.show_weather = !self.config.show_weather;
                if self.config.show_weather && self.weather_rx.is_none() {
                    self.weather_rx = Some(weather::spawn_weather_fetch(self.config.city.clone()));
                }
            }
            2 => {
                self.config.show_bazi = !self.config.show_bazi;
            }
            3 => {
                self.config.show_pet = !self.config.show_pet;
            }
            4 => {
                // City: cycle between auto and common cities
                let cities: Vec<Option<String>> = vec![None, Some("Beijing".into()), Some("Shanghai".into()), Some("Shenzhen".into()), Some("Guangzhou".into())];
                let current_idx = cities.iter().position(|c| *c == self.config.city).unwrap_or(0);
                let next_idx = (current_idx + 1) % cities.len();
                self.config.city = cities[next_idx].clone();
                let display = self.config.city.as_deref().unwrap_or("Auto (IP)");
                // Clear old weather data and cache
                self.weather_text = None;
                if let Some(config_dir) = dirs::config_dir() {
                    let cache_path = config_dir.join("mfa-cli").join("weather_cache.txt");
                    let _ = std::fs::remove_file(cache_path);
                }
                // Re-fetch with new city
                self.weather_rx = Some(weather::spawn_weather_fetch(self.config.city.clone()));
                self.status_message = Some((format!("City → {} (fetching weather...)", display), StatusKind::Success));
            }
            5 => {
                // Import
                self.input_buffer.clear();
                self.mode = Mode::ImportPath;
                self.status_message = Some(("Import file path (otpauth/json/csv):".to_string(), StatusKind::Info));
            }
            6 => {
                // Export
                self.input_buffer.clear();
                self.mode = Mode::ExportPath;
                self.status_message = Some(("Export file path:".to_string(), StatusKind::Info));
            }
            7 => {
                // Toggle encryption info
                self.status_message = Some(("Use CLI: mfa lock / mfa unlock (requires password setup)".to_string(), StatusKind::Info));
            }
            8 => {
                self.mode = Mode::Normal;
                let _ = self.config.save();
            }
            _ => {}
        }
    }

    fn finish_add(&mut self) {
        let issuer = if self.add_issuer.is_empty() { None } else { Some(self.add_issuer.clone()) };
        match OtpEntry::new(
            self.add_name.clone(),
            self.add_secret.clone(),
            issuer,
            "SHA1".to_string(),
            6,
            30,
        ) {
            Ok(entry) => {
                let name = entry.name.clone();
                self.entries.push(entry);
                self.save_vault();
                self.list_state.select(Some(self.entries.len() - 1));
                self.status_message = Some((format!("Added '{}'", name), StatusKind::Success));
                self.pet_mood = PetMood::Happy;
                self.mood_timer = Some(Instant::now());
            }
            Err(e) => {
                self.status_message = Some((format!("Error: {}", e), StatusKind::Error));
            }
        }
        self.mode = Mode::Normal;
        self.add_name.clear();
        self.add_secret.clear();
        self.add_issuer.clear();
    }

    fn finish_rename(&mut self, new_name: &str) {
        if let Some(idx) = self.list_state.selected() {
            let old = self.entries[idx].name.clone();
            if self.entries.iter().any(|e| e.name == new_name) {
                self.status_message = Some((format!("'{}' already exists", new_name), StatusKind::Error));
            } else {
                self.entries[idx].name = new_name.to_string();
                self.save_vault();
                self.status_message = Some((format!("Renamed '{}' → '{}'", old, new_name), StatusKind::Success));
            }
        }
        self.mode = Mode::Normal;
    }

    fn show_qr_overlay(&mut self) {
        if let Some(idx) = self.list_state.selected() {
            if let Some(entry) = self.entries.get(idx) {
                let uri = entry.to_otpauth_uri();
                match crate::utils::qrcode_util::render_to_terminal(&uri) {
                    Ok(qr) => {
                        self.qr_lines = qr.lines().map(|l| l.to_string()).collect();
                        self.mode = Mode::ViewQR;
                    }
                    Err(_) => {
                        self.status_message = Some(("QR generation failed".to_string(), StatusKind::Error));
                    }
                }
            }
        }
    }

    fn handle_edit_menu(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.mode = Mode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Char('1') => {
                if let Some(idx) = self.list_state.selected() {
                    self.input_buffer = self.entries[idx].name.clone();
                    self.mode = Mode::EditName;
                    self.status_message = Some(("New name:".to_string(), StatusKind::Info));
                }
            }
            KeyCode::Char('i') | KeyCode::Char('2') => {
                if let Some(idx) = self.list_state.selected() {
                    self.input_buffer = self.entries[idx].issuer.clone().unwrap_or_default();
                    self.mode = Mode::EditIssuer;
                    self.status_message = Some(("New issuer:".to_string(), StatusKind::Info));
                }
            }
            KeyCode::Char('s') | KeyCode::Char('3') => {
                if let Some(idx) = self.list_state.selected() {
                    self.input_buffer = self.entries[idx].secret.clone();
                    self.mode = Mode::EditSecret;
                    self.status_message = Some(("New secret (base32):".to_string(), StatusKind::Info));
                }
            }
            _ => {}
        }
    }

    fn finish_edit_name(&mut self, new_name: &str) {
        if new_name.is_empty() {
            self.mode = Mode::Normal;
            return;
        }
        if let Some(idx) = self.list_state.selected() {
            if self.entries.iter().any(|e| e.name == new_name) {
                self.status_message = Some((format!("'{}' already exists", new_name), StatusKind::Error));
            } else {
                let old = self.entries[idx].name.clone();
                self.entries[idx].name = new_name.to_string();
                self.save_vault();
                self.status_message = Some((format!("Renamed '{}' → '{}'", old, new_name), StatusKind::Success));
            }
        }
        self.mode = Mode::Normal;
    }

    fn finish_edit_issuer(&mut self, new_issuer: &str) {
        if let Some(idx) = self.list_state.selected() {
            self.entries[idx].issuer = if new_issuer.is_empty() { None } else { Some(new_issuer.to_string()) };
            self.save_vault();
            self.status_message = Some((format!("Updated issuer for '{}'", self.entries[idx].name), StatusKind::Success));
        }
        self.mode = Mode::Normal;
    }

    fn finish_import(&mut self, path: &str) {
        if path.is_empty() {
            self.mode = Mode::Normal;
            return;
        }
        match crate::import::import_from(None, path) {
            Ok(entries) => {
                let count = entries.len();
                for entry in entries {
                    if !self.entries.iter().any(|e| e.name == entry.name) {
                        self.entries.push(entry);
                    }
                }
                self.save_vault();
                self.status_message = Some((format!("Imported {} entries from {}", count, path), StatusKind::Success));
            }
            Err(e) => {
                self.status_message = Some((format!("Import failed: {}", e), StatusKind::Error));
            }
        }
        self.mode = Mode::Normal;
    }

    fn finish_export(&mut self, path: &str) {
        if path.is_empty() { self.mode = Mode::Normal; return; }
        let lower = path.to_lowercase();
        // Pick format by extension so TUI export stays symmetric with import.
        if lower.ends_with(".enc") || lower.ends_with(".encrypted") {
            self.status_message = Some(("Encrypted export needs a password — use CLI: mfa export --format encrypted".into(), StatusKind::Info));
            self.mode = Mode::Normal;
            return;
        }
        let (data, fmt) = if lower.ends_with(".json") {
            let file = crate::storage::models::ExportFile {
                version: crate::storage::models::ExportFile::VERSION,
                entries: self.entries.clone(),
            };
            (serde_json::to_string_pretty(&file).unwrap_or_default(), "json")
        } else {
            let lines: Vec<String> = self.entries.iter()
                .filter(|e| e.otp_type != "steam")
                .map(|e| e.to_otpauth_uri())
                .collect();
            (if lines.is_empty() { String::new() } else { lines.join("\n") + "\n" }, "otpauth")
        };
        match std::fs::write(path, &data) {
            Ok(()) => {
                let n = if fmt == "json" { self.entries.len() } else { self.entries.iter().filter(|e| e.otp_type != "steam").count() };
                self.status_message = Some((format!("Exported {} entries as {} → {}", n, fmt, path), StatusKind::Success));
            }
            Err(e) => { self.status_message = Some((format!("Export failed: {}", e), StatusKind::Error)); }
        }
        self.mode = Mode::Normal;
    }

    fn finish_edit_secret(&mut self, new_secret: &str) {
        if let Some(idx) = self.list_state.selected() {
            let normalized = new_secret.replace([' ', '-'], "").to_uppercase();
            if base32::decode(base32::Alphabet::Rfc4648 { padding: false }, &normalized).is_none() {
                self.status_message = Some(("Invalid base32 secret (allowed: A-Z and 2-7; common typos 0->O, 1->I, 8->B)".to_string(), StatusKind::Error));
            } else {
                self.entries[idx].secret = normalized;
                self.save_vault();
                self.status_message = Some((format!("Updated secret for '{}'", self.entries[idx].name), StatusKind::Success));
            }
        }
        self.mode = Mode::Normal;
    }

    fn save_vault(&self) {
        // Save entries directly (bypass password prompt for plain mode)
        if Vault::load().is_ok() {
            // Rebuild vault with current entries
            let json = serde_json::to_string_pretty(&self.entries).unwrap_or_default();
            if let Some(config_dir) = dirs::config_dir() {
                let path = config_dir.join("mfa-cli").join("vault.json");
                let _ = std::fs::write(path, json);
            }
        }
    }

    fn copy_selected(&mut self) {
        if let Some(idx) = self.list_state.selected() {
            if let Some(entry) = self.entries.get(idx) {
                match otp::generate_code(entry) {
                    Ok(code) => match clipboard::copy_to_clipboard(&code) {
                        Ok(()) => {
                            self.status_message = Some((
                                format!("✓ 已复制 {} → {}  可粘贴", entry.name, code),
                                StatusKind::Success,
                            ));
                            self.pet_mood = PetMood::Happy;
                            self.mood_timer = Some(Instant::now());
                        }
                        Err(e) => {
                            self.status_message = Some((format!("✗ 剪贴板失败: {}", e), StatusKind::Error));
                        }
                    },
                    Err(e) => {
                        self.status_message = Some((format!("✗ 验证码生成失败: {}", e), StatusKind::Error));
                    }
                }
            }
        }
    }

    fn move_up(&mut self) {
        if self.entries.is_empty() { return; }
        let current = self.list_state.selected().unwrap_or(0);
        let prev = if current == 0 { self.entries.len() - 1 } else { current - 1 };
        self.list_state.select(Some(prev));
    }

    fn move_down(&mut self) {
        if self.entries.is_empty() { return; }
        let current = self.list_state.selected().unwrap_or(0);
        let next = if current >= self.entries.len() - 1 { 0 } else { current + 1 };
        self.list_state.select(Some(next));
    }

    // ─── Rendering ───────────────────────────────────────────────

    fn render(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // header
                Constraint::Length(10), // pet
                Constraint::Min(1),    // list
                Constraint::Length(3), // status / input
            ])
            .split(f.area());

        self.render_header(f, chunks[0]);
        self.render_pet(f, chunks[1]);
        self.list_area = chunks[2];        self.render_list(f, chunks[2]);
        self.render_footer(f, chunks[3]);

        // QR overlay
        if self.mode == Mode::ViewQR {
            self.render_qr_overlay(f);
        }

        // Settings popup
        if self.mode == Mode::Settings {
            self.render_settings_popup(f);
        }
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let mut spans = vec![
            Span::styled(" ", Style::default()),
            Span::styled(
                format!("{} {}", self.almanac_info.date_str, self.almanac_info.weekday),
                Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  {}", self.almanac_info.time_str),
                Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD),
            ),
        ];

        if self.config.show_bazi {
            spans.push(Span::styled("  ┃", Style::default().fg(Color::DarkGray)));
            spans.push(Span::styled(
                format!("  {}年{}月{}日", self.almanac_info.year_ganzhi, self.almanac_info.month_ganzhi, self.almanac_info.day_ganzhi),
                Style::default().fg(Color::Yellow),
            ));
            spans.push(Span::styled(
                format!("  {}日", self.almanac_info.officer_name),
                Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD),
            ));
        }

        if let Some(weather) = &self.weather_text {
            spans.push(Span::styled("  ┃", Style::default().fg(Color::DarkGray)));
            spans.push(Span::styled(
                format!("  {}", weather),
                Style::default().fg(Color::LightGreen),
            ));
        }

        let header = Paragraph::new(Line::from(spans))
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
        f.render_widget(header, area);
    }

    fn render_pet(&self, f: &mut Frame, area: Rect) {
        if !self.config.show_pet {
            return;
        }

        let pet_data = pet::get_pet(&self.config.pet);
        let lines = pet_data.render(&self.pet_mood);

        let mut text_lines: Vec<Line> = lines
            .iter()
            .map(|l| Line::from(Span::styled(format!(" {}", l), Style::default().fg(Color::LightGreen))))
            .collect();

        if self.config.show_bazi {
            text_lines.push(Line::from(vec![
                Span::styled(format!(" [{}] ", self.config.pet), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("宜:{} ", self.almanac_info.yi), Style::default().fg(Color::LightGreen)),
                Span::styled(format!("忌:{}", self.almanac_info.ji), Style::default().fg(Color::LightRed)),
            ]));
        }

        let widget = Paragraph::new(text_lines)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
        f.render_widget(widget, area);
    }

    fn render_list(&mut self, f: &mut Frame, area: Rect) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

        // Adaptive column widths
        let name_col = self.entries.iter().map(|e| tw(&trunc(&e.name, 28))).max().unwrap_or(4).max(4) + 1;
        let issuer_col = self.entries.iter().map(|e| tw(&trunc(e.issuer.as_deref().unwrap_or(""), 28))).max().unwrap_or(4).max(4) + 1;

        let items: Vec<ListItem> = self.entries.iter().map(|entry| {
            let code = otp::generate_code(entry).unwrap_or_else(|_| "------".to_string());
            let remaining = entry.period - (now % entry.period);
            let progress = remaining as f64 / entry.period as f64;
            let bar_width = 10;
            let filled = (progress * bar_width as f64) as usize;
            let bar: String = "●".repeat(filled) + &"○".repeat(bar_width - filled);

            let code_style = if remaining <= 5 {
                Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD)
            };

            let name_d = pad(&trunc(&entry.name, 28), name_col);
            let issuer_d = pad(&trunc(entry.issuer.as_deref().unwrap_or(""), 28), issuer_col);

            let line = Line::from(vec![
                Span::styled(name_d, Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD)),
                Span::styled(issuer_d, Style::default().fg(Color::LightMagenta)),
                Span::styled(pad(&code, 8), code_style),
                Span::styled(format!("{} ", bar), if remaining <= 5 { Style::default().fg(Color::LightRed) } else { Style::default().fg(Color::LightCyan) }),
                Span::styled(format!("{:>2}s", remaining), if remaining <= 5 { Style::default().fg(Color::LightRed) } else { Style::default().fg(Color::DarkGray) }),
            ]);

            ListItem::new(line)
        }).collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)).title(" Tokens "))
            .highlight_style(Style::default())
            .highlight_symbol("▸ ");

        f.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let content = match &self.mode {
            Mode::AddName | Mode::AddSecret | Mode::AddIssuer | Mode::Rename => {
                let label = match self.mode {
                    Mode::AddName => "Name",
                    Mode::AddSecret => "Secret",
                    Mode::AddIssuer => "Issuer",
                    Mode::Rename => "Rename",
                    _ => "",
                };
                Line::from(vec![
                    Span::styled(format!(" {}: ", label), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(&self.input_buffer, Style::default().fg(Color::LightCyan)),
                    Span::styled("█", Style::default().fg(Color::LightCyan)),
                    Span::styled("  [Enter] confirm  [Esc] cancel", Style::default().fg(Color::DarkGray)),
                ])
            }
            Mode::ConfirmDelete => {
                let msg = self.status_message.as_ref().map(|(m, _)| m.clone()).unwrap_or_default();
                Line::from(vec![
                    Span::styled(format!(" {}", msg), Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)),
                    Span::styled("  [y] yes  [any] no", Style::default().fg(Color::DarkGray)),
                ])
            }
            Mode::Settings => {
                let item = SETTINGS_ITEMS[self.settings_cursor];
                let value = match self.settings_cursor {
                    0 => self.config.pet.as_str(),
                    1 => if self.config.show_weather { "ON" } else { "OFF" },
                    2 => if self.config.show_bazi { "ON" } else { "OFF" },
                    3 => if self.config.show_pet { "ON" } else { "OFF" },
                    4 => self.config.city.as_deref().unwrap_or("Auto (IP)"),
                    5 => "enter file path",
                    6 => "enter file path",
                    7 => "CLI: mfa lock / mfa unlock",
                    _ => "",
                };
                Line::from(vec![
                    Span::styled(" ⚙ ", Style::default().fg(Color::Yellow)),
                    Span::styled(format!("{}: ", item), Style::default().fg(Color::LightCyan)),
                    Span::styled(value, Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD)),
                    Span::styled("  [↑↓] select  [Enter] toggle  [Esc] close", Style::default().fg(Color::DarkGray)),
                ])
            }
            Mode::ViewQR => {
                Line::from(Span::styled(" [Esc/v] close QR view", Style::default().fg(Color::DarkGray)))
            }
            Mode::EditMenu => {
                Line::from(vec![
                    Span::styled(" Edit: ", Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD)),
                    Span::styled("n", Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD)),
                    Span::styled(" 名称  ", Style::default().fg(Color::DarkGray)),
                    Span::styled("i", Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD)),
                    Span::styled(" 发行方  ", Style::default().fg(Color::DarkGray)),
                    Span::styled("s", Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD)),
                    Span::styled(" 密钥  ", Style::default().fg(Color::DarkGray)),
                    Span::styled(" [Esc] cancel", Style::default().fg(Color::DarkGray)),
                ])
            }
            Mode::EditSecret | Mode::EditName | Mode::EditIssuer => {
                let label = match &self.mode {
                    Mode::EditName => "Name",
                    Mode::EditIssuer => "Issuer",
                    _ => "Secret",
                };
                Line::from(vec![
                    Span::styled(format!(" {}: ", label), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(&self.input_buffer, Style::default().fg(Color::LightCyan)),
                    Span::styled("█", Style::default().fg(Color::LightCyan)),
                    Span::styled("  [Enter] save  [Esc] cancel", Style::default().fg(Color::DarkGray)),
                ])
            }
            Mode::ImportPath => {
                Line::from(vec![
                    Span::styled(" Import: ", Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD)),
                    Span::styled(&self.input_buffer, Style::default().fg(Color::LightCyan)),
                    Span::styled("█", Style::default().fg(Color::LightCyan)),
                    Span::styled("  [Enter] import  [Esc] cancel", Style::default().fg(Color::DarkGray)),
                ])
            }
            Mode::ExportPath => {
                Line::from(vec![
                    Span::styled(" Export: ", Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD)),
                    Span::styled(&self.input_buffer, Style::default().fg(Color::LightCyan)),
                    Span::styled("█", Style::default().fg(Color::LightCyan)),
                    Span::styled("  [Enter] export  [Esc] cancel", Style::default().fg(Color::DarkGray)),
                ])
            }
            Mode::Normal => {
                let (msg, style) = self.status_message.as_ref().map(|(m, k)| {
                    let style = match k {
                        StatusKind::Success => Style::default().fg(Color::LightGreen),
                        StatusKind::Error => Style::default().fg(Color::LightRed),
                        StatusKind::Info => Style::default().fg(Color::DarkGray),
                    };
                    (m.clone(), style)
                }).unwrap_or_default();

                {
                    let mut spans = Vec::new();
                    // Status message or entry info
                    if !msg.is_empty() {
                        spans.push(Span::styled(" ● ", style));
                        spans.push(Span::styled(format!("{}  ", msg), style));
                    } else if let Some(idx) = self.list_state.selected() {
                        if let Some(entry) = self.entries.get(idx) {
                            spans.push(Span::styled(" ", Style::default()));
                            spans.push(Span::styled(&entry.name, Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD)));
                            spans.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
                            spans.push(Span::styled(entry.issuer.as_deref().unwrap_or("-"), Style::default().fg(Color::LightMagenta)));
                            spans.push(Span::styled("  ", Style::default()));
                        }
                    }
                    // Shortcuts always visible
                    spans.push(Span::styled(" c", Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD)));
                    spans.push(Span::styled(" 复制", Style::default().fg(Color::DarkGray)));
                    spans.push(Span::styled("  a", Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD)));
                    spans.push(Span::styled(" 添加", Style::default().fg(Color::DarkGray)));
                    spans.push(Span::styled("  e", Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD)));
                    spans.push(Span::styled(" 编辑", Style::default().fg(Color::DarkGray)));
                    spans.push(Span::styled("  r", Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD)));
                    spans.push(Span::styled(" 重命名", Style::default().fg(Color::DarkGray)));
                    spans.push(Span::styled("  v", Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD)));
                    spans.push(Span::styled(" 二维码", Style::default().fg(Color::DarkGray)));
                    spans.push(Span::styled("  d", Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)));
                    spans.push(Span::styled(" 删除", Style::default().fg(Color::DarkGray)));
                    spans.push(Span::styled("  s", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
                    spans.push(Span::styled(" 设置", Style::default().fg(Color::DarkGray)));
                    spans.push(Span::styled("  q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));
                    spans.push(Span::styled(" 退出", Style::default().fg(Color::DarkGray)));
                    Line::from(spans)
                }
            }
        };

        let footer = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
        f.render_widget(footer, area);
    }

    fn render_settings_popup(&self, f: &mut Frame) {
        let area = f.area();
        let popup_width = 52u16.min(area.width.saturating_sub(4));
        let popup_height = 16u16.min(area.height.saturating_sub(4));
        let x = (area.width.saturating_sub(popup_width)) / 2;
        let y = (area.height.saturating_sub(popup_height)) / 2;
        let popup_area = Rect::new(x, y, popup_width, popup_height);

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" ⚙ Settings ");

        let items: Vec<(&str, String)> = vec![
            ("Pet", self.config.pet.clone()),
            ("Weather", if self.config.show_weather { "ON".into() } else { "OFF".into() }),
            ("BaZi", if self.config.show_bazi { "ON".into() } else { "OFF".into() }),
            ("Pet Display", if self.config.show_pet { "ON".into() } else { "OFF".into() }),
            ("City", self.config.city.clone().unwrap_or_else(|| "Auto (IP)".into())),
            ("Import", "enter path →".into()),
            ("Export", "enter path →".into()),
            ("Encryption", "mfa lock / mfa unlock".into()),
            ("Close", "Esc".into()),
        ];

        let lines: Vec<Line> = items.iter().enumerate().map(|(i, (label, value))| {
            let selected = i == self.settings_cursor;
            let marker = if selected { "  ▸ " } else { "    " };
            let label_style = if selected {
                Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let value_style = if selected {
                Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::LightGreen)
            };

            Line::from(vec![
                Span::styled(marker, Style::default().fg(Color::LightCyan)),
                Span::styled(format!("{:<14}", label), label_style),
                Span::styled(value, value_style),
            ])
        }).collect();

        let mut all_lines = lines;
        all_lines.push(Line::from(""));
        all_lines.push(Line::from(Span::styled(
            "  [↑↓] select  [Enter] toggle  [Esc] close",
            Style::default().fg(Color::DarkGray),
        )));

        let paragraph = Paragraph::new(all_lines).block(block);
        f.render_widget(paragraph, popup_area);
    }

    fn render_qr_overlay(&self, f: &mut Frame) {
        let area = f.area();
        let popup_width = 46u16.min(area.width.saturating_sub(4));
        let popup_height = (self.qr_lines.len() as u16 + 6).min(area.height.saturating_sub(2));
        let x = (area.width.saturating_sub(popup_width)) / 2;
        let y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = Rect::new(x, y, popup_width, popup_height);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Scan with phone ");

        let mut lines: Vec<Line> = Vec::new();

        // QR code only (no extra text that could interfere with scanning)
        for qr_line in &self.qr_lines {
            lines.push(Line::from(Span::raw(qr_line.clone())));
        }

        lines.push(Line::from(""));
        if let Some(idx) = self.list_state.selected() {
            if let Some(entry) = self.entries.get(idx) {
                lines.push(Line::from(vec![
                    Span::styled(format!(" {}", entry.name), Style::default().fg(Color::LightCyan).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("  {}", entry.secret), Style::default().fg(Color::Yellow)),
                ]));
            }
        }
        lines.push(Line::from(Span::styled(" [Esc] close", Style::default().fg(Color::DarkGray))));

        // Clear background to avoid selection bar bleeding through
        f.render_widget(Clear, popup_area);
        let paragraph = Paragraph::new(lines).block(block);
        f.render_widget(paragraph, popup_area);
    }
}
