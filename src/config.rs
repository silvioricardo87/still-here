use clap::Parser;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum MouseMode {
    Subtle,
    Wide,
    Mixed,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Schedule {
    Always,
    Business,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Language {
    PtBr,
    En,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Weekday {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

// ---------------------------------------------------------------------------
// HotkeyModifiers — manual bitflags (no external crate)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HotkeyModifiers(pub u32);

impl HotkeyModifiers {
    pub const ALT: HotkeyModifiers = HotkeyModifiers(0x0001);
    pub const CONTROL: HotkeyModifiers = HotkeyModifiers(0x0002);
    pub const SHIFT: HotkeyModifiers = HotkeyModifiers(0x0004);
    pub const WIN: HotkeyModifiers = HotkeyModifiers(0x0008);
    pub const NONE: HotkeyModifiers = HotkeyModifiers(0x0000);

    pub fn contains(self, other: HotkeyModifiers) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl std::ops::BitOr for HotkeyModifiers {
    type Output = HotkeyModifiers;
    fn bitor(self, rhs: HotkeyModifiers) -> HotkeyModifiers {
        HotkeyModifiers(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for HotkeyModifiers {
    fn bitor_assign(&mut self, rhs: HotkeyModifiers) {
        self.0 |= rhs.0;
    }
}

// ---------------------------------------------------------------------------
// Config struct
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub visible: bool,
    pub typing: bool,
    pub mouse: bool,
    pub mouse_mode: MouseMode,
    pub schedule: Schedule,
    pub schedule_start: String,
    pub schedule_end: String,
    pub lunch_start: String,
    pub lunch_duration: u32,
    pub language: Language,
    pub hotkey: String,
    pub schedule_days: Vec<Weekday>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            visible: false,
            typing: true,
            mouse: true,
            mouse_mode: MouseMode::Mixed,
            schedule: Schedule::Business,
            schedule_start: "09:00".to_string(),
            schedule_end: "18:00".to_string(),
            lunch_start: "13:00".to_string(),
            lunch_duration: 60,
            language: Language::PtBr,
            hotkey: "Ctrl+Shift+F9".to_string(),
            schedule_days: vec![
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri,
            ],
        }
    }
}

fn config_path() -> PathBuf {
    let temp = env::var("TEMP").unwrap_or_else(|_| env::temp_dir().to_string_lossy().into_owned());
    PathBuf::from(temp).join("wsh.dat")
}

impl Config {
    pub fn load() -> Config {
        let path = config_path();
        if let Ok(bytes) = fs::read(&path) {
            if let Ok(cfg) = bincode::deserialize::<Config>(&bytes) {
                return cfg;
            }
        }
        Config::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = config_path();
        let bytes = bincode::serialize(self).map_err(|e| format!("Serialization error: {}", e))?;
        fs::write(&path, &bytes).map_err(|e| format!("Write error: {}", e))?;
        Ok(())
    }

    pub fn merge_cli(&mut self, args: &CliArgs) {
        if args.visible {
            self.visible = true;
        }
        if args.no_typing {
            self.typing = false;
        }
        if args.no_mouse {
            self.mouse = false;
        }
        if let Some(ref mode) = args.mouse_mode {
            match mode.to_lowercase().as_str() {
                "subtle" => self.mouse_mode = MouseMode::Subtle,
                "wide" => self.mouse_mode = MouseMode::Wide,
                "mixed" => self.mouse_mode = MouseMode::Mixed,
                _ => eprintln!("Unknown mouse-mode '{}', ignoring", mode),
            }
        }
        if let Some(ref sched) = args.schedule {
            match sched.to_lowercase().as_str() {
                "always" => self.schedule = Schedule::Always,
                "business" => self.schedule = Schedule::Business,
                _ => eprintln!("Unknown schedule '{}', ignoring", sched),
            }
        }
        if let Some(ref s) = args.schedule_start {
            self.schedule_start = s.clone();
        }
        if let Some(ref s) = args.schedule_end {
            self.schedule_end = s.clone();
        }
        if let Some(ref s) = args.schedule_days {
            match parse_schedule_days(s) {
                Ok(days) => self.schedule_days = days,
                Err(e) => eprintln!("Invalid --schedule-days: {}", e),
            }
        }
        if let Some(ref s) = args.lunch_start {
            self.lunch_start = s.clone();
        }
        if let Some(mins) = args.lunch_duration {
            self.lunch_duration = mins;
        }
        if let Some(ref lang) = args.language {
            match lang.to_lowercase().as_str() {
                "pt-br" | "ptbr" => self.language = Language::PtBr,
                "en" => self.language = Language::En,
                _ => eprintln!("Unknown language '{}', ignoring", lang),
            }
        }
        if let Some(ref hk) = args.hotkey {
            self.hotkey = hk.clone();
        }
    }
}

// ---------------------------------------------------------------------------
// CLI argument parsing
// ---------------------------------------------------------------------------

#[derive(Parser, Debug)]
#[command(name = "WinServiceHost", about = "Still Here — stealth activity simulator")]
pub struct CliArgs {
    /// Show the console window (default: hidden)
    #[arg(long)]
    pub visible: bool,

    /// Disable keyboard typing simulation
    #[arg(long = "no-typing")]
    pub no_typing: bool,

    /// Disable mouse movement simulation
    #[arg(long = "no-mouse")]
    pub no_mouse: bool,

    /// Mouse movement mode: subtle, wide, mixed
    #[arg(long = "mouse-mode", value_name = "MODE")]
    pub mouse_mode: Option<String>,

    /// Schedule mode: always, business
    #[arg(long = "schedule", value_name = "SCHEDULE")]
    pub schedule: Option<String>,

    /// Schedule start time (HH:MM)
    #[arg(long = "schedule-start", value_name = "TIME")]
    pub schedule_start: Option<String>,

    /// Schedule end time (HH:MM)
    #[arg(long = "schedule-end", value_name = "TIME")]
    pub schedule_end: Option<String>,

    /// Active weekdays, comma-separated (e.g. mon,tue,wed,thu,fri)
    #[arg(long = "schedule-days", value_name = "DAYS")]
    pub schedule_days: Option<String>,

    /// Lunch start time (HH:MM)
    #[arg(long = "lunch-start", value_name = "TIME")]
    pub lunch_start: Option<String>,

    /// Lunch duration in minutes
    #[arg(long = "lunch-duration", value_name = "MINUTES")]
    pub lunch_duration: Option<u32>,

    /// UI language: pt-br, en
    #[arg(long = "language", value_name = "LANG")]
    pub language: Option<String>,

    /// Global hotkey string (e.g. Ctrl+Shift+F9)
    #[arg(long = "hotkey", value_name = "HOTKEY")]
    pub hotkey: Option<String>,

    /// Save the resulting config to %TEMP%\wsh.dat and exit
    #[arg(long = "save-config")]
    pub save_config: bool,
}

// ---------------------------------------------------------------------------
// Hotkey parsing
// ---------------------------------------------------------------------------

/// Parse a hotkey string like "Ctrl+Shift+F9" into Win32 modifier flags and VK code.
pub fn parse_hotkey(s: &str) -> Result<(HotkeyModifiers, u16), String> {
    if s.is_empty() {
        return Err("Hotkey string is empty".to_string());
    }

    let parts: Vec<&str> = s.split('+').collect();
    let mut modifiers = HotkeyModifiers::NONE;
    let mut vk: Option<u16> = None;

    for part in &parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= HotkeyModifiers::CONTROL,
            "alt" => modifiers |= HotkeyModifiers::ALT,
            "shift" => modifiers |= HotkeyModifiers::SHIFT,
            "win" | "windows" => modifiers |= HotkeyModifiers::WIN,
            other => {
                // Function keys F1-F12
                if let Some(stripped) = other.strip_prefix('f') {
                    if let Ok(n) = stripped.parse::<u16>() {
                        if n >= 1 && n <= 12 {
                            vk = Some(0x70 + n - 1);
                            continue;
                        }
                    }
                }
                // Single letter A-Z
                if other.len() == 1 {
                    let c = other.chars().next().unwrap();
                    if c.is_ascii_alphabetic() {
                        vk = Some(c.to_ascii_uppercase() as u16);
                        continue;
                    }
                    if c.is_ascii_digit() {
                        vk = Some(c as u16); // 0x30-0x39
                        continue;
                    }
                }
                return Err(format!("Unknown key token: '{}'", part));
            }
        }
    }

    if modifiers == HotkeyModifiers::NONE {
        return Err("Hotkey must have at least one modifier (Ctrl, Alt, Shift, Win)".to_string());
    }

    match vk {
        Some(k) => Ok((modifiers, k)),
        None => Err("Hotkey must include a non-modifier key (e.g. F9, A-Z, 0-9)".to_string()),
    }
}

// ---------------------------------------------------------------------------
// Schedule days parsing
// ---------------------------------------------------------------------------

/// Parse a comma-separated list of day names into Weekday values.
pub fn parse_schedule_days(s: &str) -> Result<Vec<Weekday>, String> {
    let mut days = Vec::new();
    for token in s.split(',') {
        let trimmed = token.trim();
        let day = match trimmed.to_lowercase().as_str() {
            "mon" | "monday" => Weekday::Mon,
            "tue" | "tuesday" => Weekday::Tue,
            "wed" | "wednesday" => Weekday::Wed,
            "thu" | "thursday" => Weekday::Thu,
            "fri" | "friday" => Weekday::Fri,
            "sat" | "saturday" => Weekday::Sat,
            "sun" | "sunday" => Weekday::Sun,
            other => return Err(format!("Unknown day: '{}'", other)),
        };
        days.push(day);
    }
    if days.is_empty() {
        return Err("No days provided".to_string());
    }
    Ok(days)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(!config.visible);
        assert!(config.typing);
        assert!(config.mouse);
        assert_eq!(config.mouse_mode, MouseMode::Mixed);
        assert_eq!(config.schedule, Schedule::Business);
        assert_eq!(config.schedule_start, "09:00");
        assert_eq!(config.schedule_end, "18:00");
        assert_eq!(config.lunch_start, "13:00");
        assert_eq!(config.lunch_duration, 60);
        assert_eq!(config.language, Language::PtBr);
        assert_eq!(config.hotkey, "Ctrl+Shift+F9");
        assert_eq!(
            config.schedule_days,
            vec![
                Weekday::Mon,
                Weekday::Tue,
                Weekday::Wed,
                Weekday::Thu,
                Weekday::Fri
            ]
        );
    }

    #[test]
    fn test_parse_hotkey_valid() {
        let (mods, vk) = parse_hotkey("Ctrl+Shift+F9").unwrap();
        assert!(mods.contains(HotkeyModifiers::CONTROL));
        assert!(mods.contains(HotkeyModifiers::SHIFT));
        assert_eq!(vk, 0x78); // VK_F9
    }

    #[test]
    fn test_parse_hotkey_case_insensitive() {
        let (mods, vk) = parse_hotkey("ctrl+shift+f9").unwrap();
        assert!(mods.contains(HotkeyModifiers::CONTROL));
        assert!(mods.contains(HotkeyModifiers::SHIFT));
        assert_eq!(vk, 0x78);
    }

    #[test]
    fn test_parse_hotkey_invalid() {
        assert!(parse_hotkey("Ctrl+Shift+INVALID").is_err());
        assert!(parse_hotkey("").is_err());
        assert!(parse_hotkey("F9").is_err()); // no modifier
    }

    #[test]
    fn test_parse_schedule_days() {
        let days = parse_schedule_days("mon,wed,fri").unwrap();
        assert_eq!(days, vec![Weekday::Mon, Weekday::Wed, Weekday::Fri]);
    }

    #[test]
    fn test_parse_schedule_days_invalid() {
        assert!(parse_schedule_days("mon,invalid").is_err());
    }
}
