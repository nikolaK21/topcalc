use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

fn default_hotkey() -> String {
    "Alt+Space".into()
}

fn default_search_url() -> String {
    "https://duckduckgo.com/?q={query}".into()
}

fn default_theme() -> String {
    "dark".into()
}

fn default_anchor() -> String {
    "top".into()
}

fn default_x_percent() -> u8 {
    50
}

fn default_y_percent() -> u8 {
    12
}

fn default_colors() -> ThemeColors {
    preset_colors("dark")
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ThemeColors {
    pub background: String,
    pub text: String,
    pub muted: String,
    pub accent: String,
    pub surface: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[serde(default = "default_hotkey")]
    pub hotkey: String,
    #[serde(default = "default_search_url")]
    pub search_url: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_colors")]
    pub colors: ThemeColors,
    #[serde(default = "default_anchor")]
    pub anchor: String,
    #[serde(default = "default_x_percent")]
    pub x_percent: u8,
    #[serde(default = "default_y_percent")]
    pub y_percent: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey: default_hotkey(),
            search_url: default_search_url(),
            theme: default_theme(),
            colors: default_colors(),
            anchor: default_anchor(),
            x_percent: default_x_percent(),
            y_percent: default_y_percent(),
        }
    }
}

pub fn load_settings(path: &Path) -> Settings {
    let Ok(text) = fs::read_to_string(path) else {
        return Settings::default();
    };
    normalize(serde_json::from_str(&text).unwrap_or_default())
}

pub fn normalize(mut settings: Settings) -> Settings {
    settings.hotkey = settings.hotkey.trim().to_string();
    settings.search_url = settings.search_url.trim().to_string();
    settings.theme = canonical_theme(&settings.theme).to_string();
    settings.colors = if settings.theme == "custom" {
        sanitize_colors(settings.colors)
    } else {
        preset_colors(&settings.theme)
    };
    settings.anchor = canonical_anchor(&settings.anchor).to_string();
    settings.x_percent = settings.x_percent.min(100);
    settings.y_percent = settings.y_percent.min(100);
    settings
}

pub fn canonical_theme(value: &str) -> &'static str {
    match value.trim().to_ascii_lowercase().as_str() {
        "light" => "light",
        "slate" => "slate",
        "forest" => "forest",
        "custom" => "custom",
        _ => "dark",
    }
}

pub fn canonical_anchor(value: &str) -> &'static str {
    match value.trim().to_ascii_lowercase().as_str() {
        "center" => "center",
        "bottom" => "bottom",
        "top-left" | "topleft" => "top-left",
        "top-right" | "topright" => "top-right",
        "bottom-left" | "bottomleft" => "bottom-left",
        "bottom-right" | "bottomright" => "bottom-right",
        "custom" => "custom",
        _ => "top",
    }
}

pub fn preset_colors(theme: &str) -> ThemeColors {
    match canonical_theme(theme) {
        "light" => ThemeColors {
            background: "#f3f4f6".into(),
            text: "#1c1c20".into(),
            muted: "#5c6570".into(),
            accent: "#2563eb".into(),
            surface: "#ffffff".into(),
        },
        "slate" => ThemeColors {
            background: "#0f172a".into(),
            text: "#e2e8f0".into(),
            muted: "#94a3b8".into(),
            accent: "#38bdf8".into(),
            surface: "#1e293b".into(),
        },
        "forest" => ThemeColors {
            background: "#13241c".into(),
            text: "#e7f6ee".into(),
            muted: "#8eaea0".into(),
            accent: "#3dd68c".into(),
            surface: "#0c1b14".into(),
        },
        _ => ThemeColors {
            background: "#1c1c20".into(),
            text: "#f3f4f6".into(),
            muted: "#8b93a7".into(),
            accent: "#6ea8fe".into(),
            surface: "#12141a".into(),
        },
    }
}

fn sanitize_colors(colors: ThemeColors) -> ThemeColors {
    let fallback = preset_colors("dark");
    ThemeColors {
        background: hex_color(&colors.background, &fallback.background),
        text: hex_color(&colors.text, &fallback.text),
        muted: hex_color(&colors.muted, &fallback.muted),
        accent: hex_color(&colors.accent, &fallback.accent),
        surface: hex_color(&colors.surface, &fallback.surface),
    }
}

fn hex_color(value: &str, fallback: &str) -> String {
    let body = value.trim().trim_start_matches('#');
    if body.len() == 6 && body.chars().all(|c| c.is_ascii_hexdigit()) {
        format!("#{}", body.to_ascii_lowercase())
    } else {
        fallback.to_string()
    }
}

/// Place the window on a monitor. Percentages put the window's center at that
/// point of the screen. The result stays inside the monitor.
pub fn window_origin(
    origin_x: i32,
    origin_y: i32,
    screen_w: i32,
    screen_h: i32,
    win_w: i32,
    win_h: i32,
    anchor: &str,
    x_percent: u8,
    y_percent: u8,
) -> (i32, i32) {
    if screen_w <= 0 || screen_h <= 0 || win_w <= 0 || win_h <= 0 {
        return (origin_x, origin_y);
    }
    let margin = 24;
    let (x, y) = match canonical_anchor(anchor) {
        "center" => ((screen_w - win_w) / 2, (screen_h - win_h) / 2),
        "bottom" => ((screen_w - win_w) / 2, screen_h - win_h - 48),
        "top-left" => (margin, margin),
        "top-right" => (screen_w - win_w - margin, margin),
        "bottom-left" => (margin, screen_h - win_h - margin),
        "bottom-right" => (screen_w - win_w - margin, screen_h - win_h - margin),
        "custom" => {
            let x_percent = x_percent.min(100) as i32;
            let y_percent = y_percent.min(100) as i32;
            let cx = screen_w * x_percent / 100;
            let cy = screen_h * y_percent / 100;
            (cx - win_w / 2, cy - win_h / 2)
        }
        _ => ((screen_w - win_w) / 2, (screen_h as f64 * 0.12) as i32),
    };
    let max_x = (screen_w - win_w).max(0);
    let max_y = (screen_h - win_h).max(0);
    (origin_x + x.clamp(0, max_x), origin_y + y.clamp(0, max_y))
}

pub fn save_settings_file(path: &Path, settings: &Settings) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let text = serde_json::to_string_pretty(settings).map_err(|err| err.to_string())?;
    fs::write(path, text).map_err(|err| err.to_string())
}

pub fn build_search_url(template: &str, query: &str) -> Result<String, String> {
    let template = template.trim();
    if template.is_empty() {
        return Err("Search URL is empty".into());
    }
    let encoded = percent_encode(query);
    let url = if template.contains("{query}") {
        template.replace("{query}", &encoded)
    } else if template.contains("%s") {
        template.replace("%s", &encoded)
    } else {
        let joiner = if template.contains('?') { "&q=" } else { "?q=" };
        format!("{template}{joiner}{encoded}")
    };
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("Search URL must start with http:// or https://".into());
    }
    Ok(url)
}

pub fn percent_encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_and_builds_duckduckgo_url() {
        let url = build_search_url("https://duckduckgo.com/?q={query}", "a b+c").unwrap();
        assert_eq!(url, "https://duckduckgo.com/?q=a+b%2Bc");
        let fallback = build_search_url("https://example.com/search", "hi").unwrap();
        assert_eq!(fallback, "https://example.com/search?q=hi");
        assert!(build_search_url("file:///tmp/{query}", "x").is_err());
    }

    #[test]
    fn settings_round_trip_and_defaults() {
        let dir = std::env::temp_dir().join(format!(
            "topcalc-settings-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        assert_eq!(load_settings(&path), Settings::default());
        let custom = Settings {
            hotkey: "Ctrl+Alt+Space".into(),
            search_url: "https://example.com/?q={query}".into(),
            theme: "custom".into(),
            colors: ThemeColors {
                background: "#ABCDEF".into(),
                text: "not-a-color".into(),
                muted: "#112233".into(),
                accent: "#445566".into(),
                surface: "#778899".into(),
            },
            anchor: "bottom-right".into(),
            x_percent: 150,
            y_percent: 20,
        };
        save_settings_file(&path, &normalize(custom)).unwrap();
        let loaded = load_settings(&path);
        assert_eq!(loaded.hotkey, "Ctrl+Alt+Space");
        assert_eq!(loaded.theme, "custom");
        assert_eq!(loaded.colors.background, "#abcdef");
        assert_eq!(loaded.colors.text, preset_colors("dark").text);
        assert_eq!(loaded.anchor, "bottom-right");
        assert_eq!(loaded.x_percent, 100);
        fs::write(&path, "{\"hotkey\":\"Alt+K\"}").unwrap();
        let partial = load_settings(&path);
        assert_eq!(partial.hotkey, "Alt+K");
        assert_eq!(partial.search_url, Settings::default().search_url);
        assert_eq!(partial.theme, "dark");
        assert_eq!(partial.anchor, "top");
        fs::write(&path, "{\"theme\":\"light\",\"anchor\":\"nope\"}").unwrap();
        let repaired = load_settings(&path);
        assert_eq!(repaired.theme, "light");
        assert_eq!(repaired.colors, preset_colors("light"));
        assert_eq!(repaired.anchor, "top");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn window_stays_on_the_chosen_spot() {
        let screen = (1000, 800);
        let window = (200, 100);
        let at = |anchor: &str, x: u8, y: u8| {
            window_origin(100, 50, screen.0, screen.1, window.0, window.1, anchor, x, y)
        };
        assert_eq!(at("top", 0, 0), (500, 146));
        assert_eq!(at("center", 0, 0), (500, 400));
        assert_eq!(at("bottom", 0, 0), (500, 702));
        assert_eq!(at("top-left", 0, 0), (124, 74));
        assert_eq!(at("bottom-right", 0, 0), (876, 726));
        assert_eq!(at("custom", 0, 0), (100, 50));
        assert_eq!(at("custom", 50, 50), (500, 400));
        assert_eq!(at("custom", 100, 100), (900, 750));
        assert_eq!(at("somewhere", 0, 0), (500, 146));
    }
}
