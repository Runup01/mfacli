/// Weather module: fetches from wttr.in with IP auto-detection
/// Runs in background thread, caches result, never blocks main logic.
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::time::Duration;

const CACHE_TTL_SECS: u64 = 1800; // 30 minutes

#[allow(dead_code)]
pub struct WeatherInfo {
    pub display: String,
}

/// Check network connectivity (tries domestic DNS first)
#[allow(dead_code)]
pub fn check_network() -> bool {
    let targets = [
        "223.5.5.5:53",
        "114.114.114.114:53",
        "8.8.8.8:53",
        "1.1.1.1:53",
    ];

    for addr in &targets {
        if let Ok(parsed) = addr.parse() {
            if TcpStream::connect_timeout(&parsed, Duration::from_secs(2)).is_ok() {
                return true;
            }
        }
    }
    false
}

/// Get weather string (from cache or network)
pub fn get_weather(city: Option<&str>) -> Option<String> {
    // Try cache first
    if let Some(cached) = read_cache() {
        return Some(cached);
    }

    // Fetch from network
    let result = fetch_from_network(city)?;
    write_cache(&result);
    Some(result)
}

/// Spawn a background thread to fetch weather, returns a receiver
pub fn spawn_weather_fetch(city: Option<String>) -> std::sync::mpsc::Receiver<Option<String>> {
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let result = get_weather(city.as_deref());
        let _ = tx.send(result);
    });

    rx
}

fn fetch_from_network(city: Option<&str>) -> Option<String> {
    let host = "wttr.in";
    let path = match city {
        Some(c) => format!("/{}?format=3&lang=zh", c),
        None => "/?format=3&lang=zh".to_string(),
    };

    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: mfa-cli/0.1\r\nConnection: close\r\n\r\n",
        path, host
    );

    let stream = TcpStream::connect_timeout(
        &(host, 80).to_socket_addrs().ok()?.next()?,
        Duration::from_secs(3),
    )
    .ok()?;

    stream.set_read_timeout(Some(Duration::from_secs(3))).ok()?;
    let mut stream = stream;
    stream.write_all(request.as_bytes()).ok()?;

    let mut response = String::new();
    stream.read_to_string(&mut response).ok()?;

    // Parse HTTP response body (after \r\n\r\n)
    let body = response.split("\r\n\r\n").nth(1)?;

    // Handle chunked encoding: extract actual content
    let content = if body.contains("\r\n") {
        // Chunked: skip chunk size line
        body.lines().nth(1).unwrap_or("").trim().to_string()
    } else {
        body.trim().to_string()
    };

    if content.is_empty() {
        return None;
    }

    Some(clean_weather(&content))
}

fn cache_path() -> Option<PathBuf> {
    let config_dir = dirs::config_dir()?;
    let dir = config_dir.join("mfa-cli");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir.join("weather_cache.txt"))
}

fn read_cache() -> Option<String> {
    let path = cache_path()?;
    let content = std::fs::read_to_string(&path).ok()?;
    let mut lines = content.lines();
    let timestamp: u64 = lines.next()?.parse().ok()?;
    let weather = lines.next()?.to_string();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();

    if now - timestamp < CACHE_TTL_SECS {
        Some(weather)
    } else {
        None
    }
}

fn write_cache(weather: &str) {
    if let Some(path) = cache_path() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let _ = std::fs::write(path, format!("{}\n{}", now, weather));
    }
}

use std::net::ToSocketAddrs;

fn clean_weather(raw: &str) -> String {
    if let Some((loc, rest)) = raw.split_once(':') {
        let city = loc.split(',').next().unwrap_or(loc).trim();
        let cond = rest.trim();
        if city.is_empty() {
            raw.to_string()
        } else if city.parse::<f64>().is_ok() {
            // wttr.in falls back to raw coordinates when the (datacenter)
            // IP has no city name — show a friendly label instead
            format!("IP定位 {}", cond)
        } else {
            format!("{} {}", city, cond)
        }
    } else {
        raw.to_string()
    }
}
