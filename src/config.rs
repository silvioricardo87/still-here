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

    #[cfg(test)]
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
    pub auto_shutdown: bool,
    pub auto_shutdown_delay: u32,
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
            auto_shutdown: false,
            auto_shutdown_delay: 0,
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
        if args.auto_shutdown {
            self.auto_shutdown = true;
        }
        if let Some(delay) = args.auto_shutdown_delay {
            self.auto_shutdown_delay = delay;
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

    /// Enable auto-shutdown after business hours when idle
    #[arg(long = "auto-shutdown")]
    pub auto_shutdown: bool,

    /// Idle minutes before auto-shutdown (0 = random 5-15 min)
    #[arg(long = "auto-shutdown-delay", value_name = "MINUTES")]
    pub auto_shutdown_delay: Option<u32>,
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

    #[test]
    fn test_parse_schedule_days_full_names() {
        let days = parse_schedule_days("monday,wednesday,friday").unwrap();
        assert_eq!(days, vec![Weekday::Mon, Weekday::Wed, Weekday::Fri]);
    }

    #[test]
    fn test_parse_schedule_days_weekend() {
        let days = parse_schedule_days("sat,sun").unwrap();
        assert_eq!(days, vec![Weekday::Sat, Weekday::Sun]);
    }

    #[test]
    fn test_parse_schedule_days_all_days() {
        let days = parse_schedule_days("mon,tue,wed,thu,fri,sat,sun").unwrap();
        assert_eq!(days.len(), 7);
    }

    #[test]
    fn test_parse_schedule_days_with_spaces() {
        let days = parse_schedule_days("mon , wed , fri").unwrap();
        assert_eq!(days, vec![Weekday::Mon, Weekday::Wed, Weekday::Fri]);
    }

    #[test]
    fn test_parse_hotkey_with_alt() {
        let (mods, vk) = parse_hotkey("Alt+Shift+F1").unwrap();
        assert!(mods.contains(HotkeyModifiers::ALT));
        assert!(mods.contains(HotkeyModifiers::SHIFT));
        assert_eq!(vk, 0x70); // VK_F1
    }

    #[test]
    fn test_parse_hotkey_with_win() {
        let (mods, _vk) = parse_hotkey("Win+A").unwrap();
        assert!(mods.contains(HotkeyModifiers::WIN));
    }

    #[test]
    fn test_parse_hotkey_letter_key() {
        let (mods, vk) = parse_hotkey("Ctrl+A").unwrap();
        assert!(mods.contains(HotkeyModifiers::CONTROL));
        assert_eq!(vk, b'A' as u16);
    }

    #[test]
    fn test_parse_hotkey_digit_key() {
        let (mods, vk) = parse_hotkey("Ctrl+5").unwrap();
        assert!(mods.contains(HotkeyModifiers::CONTROL));
        assert_eq!(vk, b'5' as u16);
    }

    #[test]
    fn test_parse_hotkey_all_function_keys() {
        for n in 1..=12u16 {
            let s = format!("Ctrl+F{}", n);
            let (_, vk) = parse_hotkey(&s).unwrap();
            assert_eq!(vk, 0x70 + n - 1, "F{} should map to VK 0x{:02X}", n, 0x70 + n - 1);
        }
    }

    #[test]
    fn test_parse_hotkey_no_key() {
        assert!(parse_hotkey("Ctrl+Shift").is_err());
    }

    #[test]
    fn test_parse_hotkey_control_alias() {
        let (mods, _) = parse_hotkey("Control+F1").unwrap();
        assert!(mods.contains(HotkeyModifiers::CONTROL));
    }

    #[test]
    fn test_parse_hotkey_windows_alias() {
        let (mods, _) = parse_hotkey("Windows+F1").unwrap();
        assert!(mods.contains(HotkeyModifiers::WIN));
    }

    #[test]
    fn test_hotkey_modifiers_bitor() {
        let combined = HotkeyModifiers::CONTROL | HotkeyModifiers::SHIFT;
        assert!(combined.contains(HotkeyModifiers::CONTROL));
        assert!(combined.contains(HotkeyModifiers::SHIFT));
        assert!(!combined.contains(HotkeyModifiers::ALT));
    }

    #[test]
    fn test_hotkey_modifiers_bitor_assign() {
        let mut mods = HotkeyModifiers::NONE;
        mods |= HotkeyModifiers::ALT;
        mods |= HotkeyModifiers::SHIFT;
        assert!(mods.contains(HotkeyModifiers::ALT));
        assert!(mods.contains(HotkeyModifiers::SHIFT));
        assert!(!mods.contains(HotkeyModifiers::CONTROL));
    }

    #[test]
    fn test_merge_cli_visible() {
        let mut config = Config::default();
        let args = CliArgs::parse_from(["test", "--visible"]);
        config.merge_cli(&args);
        assert!(config.visible);
    }

    #[test]
    fn test_merge_cli_no_typing_no_mouse() {
        let mut config = Config::default();
        let args = CliArgs::parse_from(["test", "--no-typing", "--no-mouse"]);
        config.merge_cli(&args);
        assert!(!config.typing);
        assert!(!config.mouse);
    }

    #[test]
    fn test_merge_cli_mouse_mode() {
        let mut config = Config::default();
        let args = CliArgs::parse_from(["test", "--mouse-mode", "subtle"]);
        config.merge_cli(&args);
        assert_eq!(config.mouse_mode, MouseMode::Subtle);

        let args = CliArgs::parse_from(["test", "--mouse-mode", "wide"]);
        config.merge_cli(&args);
        assert_eq!(config.mouse_mode, MouseMode::Wide);
    }

    #[test]
    fn test_merge_cli_schedule_always() {
        let mut config = Config::default();
        let args = CliArgs::parse_from(["test", "--schedule", "always"]);
        config.merge_cli(&args);
        assert_eq!(config.schedule, Schedule::Always);
    }

    #[test]
    fn test_merge_cli_language_en() {
        let mut config = Config::default();
        let args = CliArgs::parse_from(["test", "--language", "en"]);
        config.merge_cli(&args);
        assert_eq!(config.language, Language::En);
    }

    #[test]
    fn test_merge_cli_language_ptbr_alias() {
        let mut config = Config::default();
        let args = CliArgs::parse_from(["test", "--language", "ptbr"]);
        config.merge_cli(&args);
        assert_eq!(config.language, Language::PtBr);
    }

    #[test]
    fn test_merge_cli_custom_schedule_times() {
        let mut config = Config::default();
        let args = CliArgs::parse_from([
            "test",
            "--schedule-start", "08:00",
            "--schedule-end", "17:00",
            "--lunch-start", "12:00",
            "--lunch-duration", "45",
        ]);
        config.merge_cli(&args);
        assert_eq!(config.schedule_start, "08:00");
        assert_eq!(config.schedule_end, "17:00");
        assert_eq!(config.lunch_start, "12:00");
        assert_eq!(config.lunch_duration, 45);
    }

    #[test]
    fn test_merge_cli_schedule_days() {
        let mut config = Config::default();
        let args = CliArgs::parse_from(["test", "--schedule-days", "mon,wed,fri"]);
        config.merge_cli(&args);
        assert_eq!(config.schedule_days, vec![Weekday::Mon, Weekday::Wed, Weekday::Fri]);
    }

    #[test]
    fn test_merge_cli_hotkey() {
        let mut config = Config::default();
        let args = CliArgs::parse_from(["test", "--hotkey", "Ctrl+Alt+F12"]);
        config.merge_cli(&args);
        assert_eq!(config.hotkey, "Ctrl+Alt+F12");
    }

    #[test]
    fn test_merge_cli_no_args_preserves_defaults() {
        let mut config = Config::default();
        let original = Config::default();
        let args = CliArgs::parse_from(["test"]);
        config.merge_cli(&args);
        assert_eq!(config.visible, original.visible);
        assert_eq!(config.typing, original.typing);
        assert_eq!(config.mouse, original.mouse);
        assert_eq!(config.mouse_mode, original.mouse_mode);
        assert_eq!(config.schedule, original.schedule);
        assert_eq!(config.language, original.language);
    }

    #[test]
    fn test_default_auto_shutdown_disabled() {
        let config = Config::default();
        assert!(!config.auto_shutdown);
        assert_eq!(config.auto_shutdown_delay, 0);
    }

    #[test]
    fn test_merge_cli_auto_shutdown() {
        let mut config = Config::default();
        let args = CliArgs::parse_from(["test", "--auto-shutdown"]);
        config.merge_cli(&args);
        assert!(config.auto_shutdown);
    }

    #[test]
    fn test_merge_cli_auto_shutdown_delay() {
        let mut config = Config::default();
        let args = CliArgs::parse_from(["test", "--auto-shutdown", "--auto-shutdown-delay", "10"]);
        config.merge_cli(&args);
        assert!(config.auto_shutdown);
        assert_eq!(config.auto_shutdown_delay, 10);
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = Config::default();
        let bytes = bincode::serialize(&config).unwrap();
        let deserialized: Config = bincode::deserialize(&bytes).unwrap();
        assert_eq!(deserialized.visible, config.visible);
        assert_eq!(deserialized.typing, config.typing);
        assert_eq!(deserialized.mouse, config.mouse);
        assert_eq!(deserialized.mouse_mode, config.mouse_mode);
        assert_eq!(deserialized.schedule, config.schedule);
        assert_eq!(deserialized.language, config.language);
        assert_eq!(deserialized.hotkey, config.hotkey);
        assert_eq!(deserialized.schedule_days, config.schedule_days);
    }
}
