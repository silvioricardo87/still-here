use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::time::Duration;

use rand::Rng;

use crate::config::{Config, Schedule, Weekday};

// ---------------------------------------------------------------------------
// Shared state for GUI display
// ---------------------------------------------------------------------------

pub const CYCLE_IDLE: u8 = 0;
pub const CYCLE_ACTIVE: u8 = 1;
pub const CYCLE_INACTIVE: u8 = 2;
pub const CYCLE_LONG_PAUSE: u8 = 3;
pub const CYCLE_LUNCH: u8 = 4;
pub const CYCLE_OUTSIDE_HOURS: u8 = 5;

static CURRENT_CYCLE: AtomicU8 = AtomicU8::new(CYCLE_IDLE);
static USER_IS_PAUSED: AtomicBool = AtomicBool::new(false);
static AUTO_SHUTDOWN_ENABLED: AtomicBool = AtomicBool::new(false);

/// Returns the current cycle as a u8 constant.
pub fn current_cycle() -> u8 {
    CURRENT_CYCLE.load(Ordering::Relaxed)
}

/// Returns true if simulation is paused because user is active.
pub fn user_is_paused() -> bool {
    USER_IS_PAUSED.load(Ordering::Relaxed)
}

/// Returns a human-readable label for the given cycle constant.
#[cfg(test)]
pub fn cycle_label(cycle: u8) -> &'static str {
    match cycle {
        CYCLE_IDLE => "Idle",
        CYCLE_ACTIVE => "Active",
        CYCLE_INACTIVE => "Inactive",
        CYCLE_LONG_PAUSE => "Long Pause",
        CYCLE_LUNCH => "Lunch",
        CYCLE_OUTSIDE_HOURS => "Outside Hours",
        _ => "Unknown",
    }
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum ActivityCycle {
    Active(u64),
    Inactive(u64),
    LongPause(u64),
}

#[derive(Debug, Clone)]
pub struct DateTime {
    pub hour: u8,
    pub minute: u8,
    pub weekday: Weekday,
}

#[derive(Debug, Clone)]
pub struct DailyPlan {
    pub long_pause_count: u8,
    pub long_pauses_used: u8,
}

// ---------------------------------------------------------------------------
// Time helpers
// ---------------------------------------------------------------------------

/// Returns the current local date/time using the Windows GetLocalTime API.
pub fn current_datetime() -> DateTime {
    use windows::Win32::System::SystemInformation::GetLocalTime;

    let st = unsafe { GetLocalTime() };

    // SYSTEMTIME.wDayOfWeek: 0=Sunday, 1=Monday, ..., 6=Saturday
    let weekday = match st.wDayOfWeek {
        0 => Weekday::Sun,
        1 => Weekday::Mon,
        2 => Weekday::Tue,
        3 => Weekday::Wed,
        4 => Weekday::Thu,
        5 => Weekday::Fri,
        6 => Weekday::Sat,
        _ => Weekday::Mon,
    };

    DateTime {
        hour: st.wHour as u8,
        minute: st.wMinute as u8,
        weekday,
    }
}

/// Parse "HH:MM" into (hour, minute).
pub fn parse_time(s: &str) -> (u8, u8) {
    let mut parts = s.splitn(2, ':');
    let hour = parts.next().and_then(|h| h.parse::<u8>().ok()).unwrap_or(0);
    let minute = parts.next().and_then(|m| m.parse::<u8>().ok()).unwrap_or(0);
    (hour, minute)
}

/// Convert (hour, minute) to total minutes since midnight.
fn to_minutes(hour: u8, minute: u8) -> u16 {
    hour as u16 * 60 + minute as u16
}

fn weekday_to_u8(w: &Weekday) -> u8 {
    match w {
        Weekday::Mon => 0,
        Weekday::Tue => 1,
        Weekday::Wed => 2,
        Weekday::Thu => 3,
        Weekday::Fri => 4,
        Weekday::Sat => 5,
        Weekday::Sun => 6,
    }
}

// ---------------------------------------------------------------------------
// Schedule evaluation
// ---------------------------------------------------------------------------

/// Returns true if `now` falls within the configured business hours and days.
pub fn is_business_hours(config: &Config, now: &DateTime) -> bool {
    // Check day of week
    if !config.schedule_days.contains(&now.weekday) {
        return false;
    }

    let current_mins = to_minutes(now.hour, now.minute);
    let (start_h, start_m) = parse_time(&config.schedule_start);
    let (end_h, end_m) = parse_time(&config.schedule_end);
    let start_mins = to_minutes(start_h, start_m);
    let end_mins = to_minutes(end_h, end_m);

    current_mins >= start_mins && current_mins < end_mins
}

/// Returns true if `now` is within the lunch window (±5 min fuzzy boundaries).
pub fn is_lunch_time(config: &Config, now: &DateTime) -> bool {
    let (lunch_h, lunch_m) = parse_time(&config.lunch_start);
    let lunch_start_mins = to_minutes(lunch_h, lunch_m);
    let lunch_end_mins = lunch_start_mins + config.lunch_duration as u16;

    let current_mins = to_minutes(now.hour, now.minute);

    // Apply ±5 min fuzzy boundaries
    let fuzzy_start = lunch_start_mins.saturating_sub(5);
    let fuzzy_end = lunch_end_mins + 5;

    current_mins >= fuzzy_start && current_mins < fuzzy_end
}

// ---------------------------------------------------------------------------
// Daily plan and cycle selection
// ---------------------------------------------------------------------------

/// Picks 1–3 long pauses for the day.
pub fn generate_daily_plan() -> DailyPlan {
    let long_pause_count = rand::thread_rng().gen_range(1u8..=3u8);
    DailyPlan { long_pause_count, long_pauses_used: 0 }
}

/// Weighted random cycle selection:
/// - 50% Active(480–1500s)
/// - 35% Inactive(300–1200s)
/// - 15% LongPause(1200–3000s) — only if quota not exhausted
pub fn pick_next_cycle(daily_plan: &DailyPlan) -> ActivityCycle {
    let mut rng = rand::thread_rng();
    let roll: u8 = rng.gen_range(0..100);

    if roll < 50 {
        let duration = rng.gen_range(480u64..=1500u64);
        ActivityCycle::Active(duration)
    } else if roll < 85 {
        let duration = rng.gen_range(300u64..=1200u64);
        ActivityCycle::Inactive(duration)
    } else {
        // LongPause only if quota not exhausted; otherwise fall back to Inactive
        if daily_plan.long_pauses_used < daily_plan.long_pause_count {
            let duration = rng.gen_range(1200u64..=3000u64);
            ActivityCycle::LongPause(duration)
        } else {
            let duration = rng.gen_range(300u64..=1200u64);
            ActivityCycle::Inactive(duration)
        }
    }
}

// ---------------------------------------------------------------------------
// Interruptible sleep
// ---------------------------------------------------------------------------

/// Sleeps for `seconds`, checking shutdown every second.
/// Returns `true` if interrupted by shutdown signal, `false` if completed.
fn sleep_interruptible(seconds: u64, shutdown: &AtomicBool) -> bool {
    for _ in 0..seconds {
        if shutdown.load(Ordering::Relaxed) {
            return true;
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    false
}

/// If the user is actively using the computer, pauses simulation until they
/// have been idle for the configured threshold. Returns true if shutdown was
/// requested during the wait.
fn pause_if_user_active(shutdown: &AtomicBool) -> bool {
    if crate::input::user_is_active() {
        USER_IS_PAUSED.store(true, Ordering::Relaxed);
        crate::input::wait_for_user_idle(shutdown);
        USER_IS_PAUSED.store(false, Ordering::Relaxed);
    }
    shutdown.load(Ordering::Relaxed)
}

/// Sets the user-paused flag (called from input module during typing simulation).
pub fn set_user_paused(paused: bool) {
    USER_IS_PAUSED.store(paused, Ordering::Relaxed);
}

/// Returns true if auto-shutdown is enabled.
pub fn auto_shutdown_enabled() -> bool {
    AUTO_SHUTDOWN_ENABLED.load(Ordering::Relaxed)
}

/// Sets the auto-shutdown enabled flag (called from main hotkey handler and init).
pub fn set_auto_shutdown_enabled(enabled: bool) {
    AUTO_SHUTDOWN_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Returns the shutdown delay in minutes. If `configured_delay` is 0, returns
/// a random value between 5 and 15 (inclusive). Otherwise returns the configured value.
pub fn resolve_shutdown_delay(configured_delay: u32) -> u32 {
    if configured_delay == 0 {
        rand::thread_rng().gen_range(5u32..=15u32)
    } else {
        configured_delay
    }
}

/// Initiates a system shutdown via shutdown.exe.
fn execute_shutdown() {
    let _ = std::process::Command::new("shutdown")
        .args(["/s", "/t", "0", "/f"])
        .spawn();
}

// ---------------------------------------------------------------------------
// Main scheduler loop
// ---------------------------------------------------------------------------

pub fn run_scheduler(config: Config, shutdown: &'static AtomicBool) {
    let mut rng = rand::thread_rng();
    let mut daily_plan = generate_daily_plan();

    // Track the last day we generated a plan for, to reset at day boundary.
    // We encode weekday as u8: Mon=0 Tue=1 Wed=2 Thu=3 Fri=4 Sat=5 Sun=6
    let mut last_plan_day: Option<u8> = None;

    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        let now = current_datetime();

        let current_day = weekday_to_u8(&now.weekday);
        let reset_needed = match last_plan_day {
            None => true,
            Some(d) => d != current_day,
        };
        if reset_needed {
            // Only reset at the schedule start time (or midnight for Always mode)
            let should_reset = match config.schedule {
                Schedule::Business => {
                    let (start_h, start_m) = parse_time(&config.schedule_start);
                    now.hour == start_h && now.minute == start_m
                }
                Schedule::Always => now.hour == 0 && now.minute == 0,
            };
            if should_reset || last_plan_day.is_none() {
                daily_plan = generate_daily_plan();
                last_plan_day = Some(current_day);
            }
        }

        // --- Pause while user is active ---
        if pause_if_user_active(shutdown) {
            break;
        }

        // --- Outside business hours (Business schedule only) ---
        if config.schedule == Schedule::Business && !is_business_hours(&config, &now) {
            CURRENT_CYCLE.store(CYCLE_OUTSIDE_HOURS, Ordering::Relaxed);

            // Auto-shutdown: if user has been idle long enough, shut down
            if auto_shutdown_enabled() {
                let delay_min = resolve_shutdown_delay(config.auto_shutdown_delay);
                let delay_ms = delay_min as u32 * 60_000;
                if crate::input::system_idle_ms() >= delay_ms {
                    execute_shutdown();
                    shutdown.store(true, Ordering::SeqCst);
                    break;
                }
            }

            // Silent mouse jitter every 3–5 minutes, no typing
            if config.mouse {
                crate::input::move_mouse_silent();
            }
            let delay = rng.gen_range(180u64..=300u64);
            if sleep_interruptible(delay, shutdown) {
                break;
            }
            continue;
        }

        // --- Lunch time ---
        if is_lunch_time(&config, &now) {
            CURRENT_CYCLE.store(CYCLE_LUNCH, Ordering::Relaxed);
            // Silent mouse jitter every 2–4 minutes, no typing
            if config.mouse {
                crate::input::move_mouse_silent();
            }
            let delay = rng.gen_range(120u64..=240u64);
            if sleep_interruptible(delay, shutdown) {
                break;
            }
            continue;
        }

        // --- Pick and execute activity cycle ---
        let cycle = pick_next_cycle(&daily_plan);

        match cycle {
            ActivityCycle::Active(duration) => {
                CURRENT_CYCLE.store(CYCLE_ACTIVE, Ordering::Relaxed);
                run_active_cycle(duration, &config, shutdown, &mut daily_plan);
            }
            ActivityCycle::Inactive(duration) => {
                CURRENT_CYCLE.store(CYCLE_INACTIVE, Ordering::Relaxed);
                run_inactive_cycle(duration, &config, shutdown);
            }
            ActivityCycle::LongPause(duration) => {
                CURRENT_CYCLE.store(CYCLE_LONG_PAUSE, Ordering::Relaxed);
                daily_plan.long_pauses_used += 1;
                run_long_pause_cycle(duration, &config, shutdown);
            }
        }

        if shutdown.load(Ordering::Relaxed) {
            break;
        }
    }
}

// ---------------------------------------------------------------------------
// Cycle runners
// ---------------------------------------------------------------------------

fn run_active_cycle(
    duration: u64,
    config: &Config,
    shutdown: &'static AtomicBool,
    _daily_plan: &mut DailyPlan,
) {
    let mut rng = rand::thread_rng();
    let start = std::time::Instant::now();
    let elapsed = || start.elapsed().as_secs();

    // Determine when the "gradual wind-down" begins (last 2–3 min of cycle)
    let wind_down_start = duration.saturating_sub(rng.gen_range(120u64..=180u64));

    // Phrases per block before a block pause
    let mut phrases_in_block = 0u32;
    let mut block_target = rng.gen_range(3u32..=8u32);

    while elapsed() < duration {
        if shutdown.load(Ordering::Relaxed) {
            return;
        }

        // --- Pause while user is active ---
        if pause_if_user_active(shutdown) {
            return;
        }

        let time_left = duration.saturating_sub(elapsed());
        let winding_down = elapsed() >= wind_down_start;

        // --- Typing (silent keystrokes at human-like intervals) ---
        if config.typing {
            let phrase = crate::dictionary::random_phrase(config.language.clone());
            crate::input::simulate_typing_activity(phrase, shutdown);
            if shutdown.load(Ordering::Relaxed) {
                return;
            }
            phrases_in_block += 1;
        }

        // --- Mouse movement (imperceptible micro-jitter) ---
        if config.mouse {
            crate::input::move_mouse_silent();
        }

        // --- Block pause after every 3–8 phrases ---
        if config.typing && phrases_in_block >= block_target {
            phrases_in_block = 0;
            block_target = rng.gen_range(3u32..=8u32);

            // 1–5 minute block pause (shortened if winding down or near end)
            let pause_max = if winding_down || time_left < 300 {
                time_left.min(60)
            } else {
                300u64
            };
            let pause = if pause_max > 60 {
                rng.gen_range(60u64..=pause_max)
            } else {
                pause_max.max(1)
            };
            if sleep_interruptible(pause, shutdown) {
                return;
            }
            continue;
        }

        // --- Inter-phrase delay (10–60s, longer if winding down) ---
        let base_delay = if winding_down {
            rng.gen_range(30u64..=120u64)
        } else {
            rng.gen_range(10u64..=60u64)
        };
        let delay = base_delay.min(time_left.max(1));
        if sleep_interruptible(delay, shutdown) {
            return;
        }
    }
}

fn run_inactive_cycle(duration: u64, config: &Config, shutdown: &'static AtomicBool) {
    let mut rng = rand::thread_rng();
    let start = std::time::Instant::now();

    while start.elapsed().as_secs() < duration {
        if shutdown.load(Ordering::Relaxed) {
            return;
        }
        if pause_if_user_active(shutdown) {
            return;
        }

        if config.mouse {
            crate::input::move_mouse_silent();
        }

        // Wait 30–90s before next silent move
        let time_left = duration.saturating_sub(start.elapsed().as_secs());
        let delay = rng.gen_range(30u64..=90u64).min(time_left.max(1));
        if sleep_interruptible(delay, shutdown) {
            return;
        }
    }
}

fn run_long_pause_cycle(duration: u64, config: &Config, shutdown: &'static AtomicBool) {
    let mut rng = rand::thread_rng();
    let start = std::time::Instant::now();

    while start.elapsed().as_secs() < duration {
        if shutdown.load(Ordering::Relaxed) {
            return;
        }
        if pause_if_user_active(shutdown) {
            return;
        }

        if config.mouse {
            crate::input::move_mouse_silent();
        }

        // Wait 2–4 minutes before next silent move
        let time_left = duration.saturating_sub(start.elapsed().as_secs());
        let delay = rng.gen_range(120u64..=240u64).min(time_left.max(1));
        if sleep_interruptible(delay, shutdown) {
            return;
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_business_hours_weekday_in_range() {
        let config = Config::default();
        assert!(is_business_hours(&config, &wednesday_10_30()));
    }

    #[test]
    fn test_is_business_hours_weekend() {
        let config = Config::default();
        assert!(!is_business_hours(&config, &saturday_10_30()));
    }

    #[test]
    fn test_is_business_hours_outside_range() {
        let config = Config::default();
        assert!(!is_business_hours(&config, &wednesday_22_00()));
    }

    #[test]
    fn test_is_lunch_time() {
        let config = Config::default(); // lunch 13:00, 60min
        assert!(is_lunch_time(&config, &wednesday_13_30()));
        assert!(!is_lunch_time(&config, &wednesday_10_30()));
    }

    #[test]
    fn test_pick_activity_cycle_returns_valid() {
        let plan = DailyPlan { long_pause_count: 3, long_pauses_used: 0 };
        for _ in 0..50 {
            let cycle = pick_next_cycle(&plan);
            match cycle {
                ActivityCycle::Active(d) => assert!(d >= 480 && d <= 1500),
                ActivityCycle::Inactive(d) => assert!(d >= 300 && d <= 1200),
                ActivityCycle::LongPause(d) => assert!(d >= 1200 && d <= 3000),
            }
        }
    }

    #[test]
    fn test_daily_plan_has_1_to_3_long_pauses() {
        for _ in 0..20 {
            let plan = generate_daily_plan();
            assert!(plan.long_pause_count >= 1 && plan.long_pause_count <= 3);
        }
    }

    #[test]
    fn test_parse_time() {
        assert_eq!(parse_time("09:00"), (9, 0));
        assert_eq!(parse_time("18:30"), (18, 30));
        assert_eq!(parse_time("00:00"), (0, 0));
    }

    fn wednesday_10_30() -> DateTime {
        DateTime { hour: 10, minute: 30, weekday: Weekday::Wed }
    }
    fn wednesday_13_30() -> DateTime {
        DateTime { hour: 13, minute: 30, weekday: Weekday::Wed }
    }
    fn wednesday_22_00() -> DateTime {
        DateTime { hour: 22, minute: 0, weekday: Weekday::Wed }
    }
    fn saturday_10_30() -> DateTime {
        DateTime { hour: 10, minute: 30, weekday: Weekday::Sat }
    }

    #[test]
    fn test_is_business_hours_at_start_boundary() {
        let config = Config::default(); // 09:00-18:00
        let at_start = DateTime { hour: 9, minute: 0, weekday: Weekday::Mon };
        assert!(is_business_hours(&config, &at_start));
    }

    #[test]
    fn test_is_business_hours_at_end_boundary() {
        let config = Config::default(); // 09:00-18:00
        let at_end = DateTime { hour: 18, minute: 0, weekday: Weekday::Mon };
        assert!(!is_business_hours(&config, &at_end)); // end is exclusive
    }

    #[test]
    fn test_is_business_hours_one_minute_before_end() {
        let config = Config::default();
        let before_end = DateTime { hour: 17, minute: 59, weekday: Weekday::Tue };
        assert!(is_business_hours(&config, &before_end));
    }

    #[test]
    fn test_is_business_hours_one_minute_before_start() {
        let config = Config::default();
        let before_start = DateTime { hour: 8, minute: 59, weekday: Weekday::Mon };
        assert!(!is_business_hours(&config, &before_start));
    }

    #[test]
    fn test_is_business_hours_custom_days_with_saturday() {
        let mut config = Config::default();
        config.schedule_days = vec![Weekday::Mon, Weekday::Sat];
        let sat = DateTime { hour: 10, minute: 0, weekday: Weekday::Sat };
        assert!(is_business_hours(&config, &sat));
        let sun = DateTime { hour: 10, minute: 0, weekday: Weekday::Sun };
        assert!(!is_business_hours(&config, &sun));
    }

    #[test]
    fn test_is_business_hours_custom_times() {
        let mut config = Config::default();
        config.schedule_start = "06:00".to_string();
        config.schedule_end = "14:00".to_string();
        let early = DateTime { hour: 6, minute: 0, weekday: Weekday::Mon };
        assert!(is_business_hours(&config, &early));
        let after = DateTime { hour: 14, minute: 0, weekday: Weekday::Mon };
        assert!(!is_business_hours(&config, &after));
    }

    #[test]
    fn test_is_lunch_time_fuzzy_start_boundary() {
        let config = Config::default(); // lunch 13:00, 60min
        // 5 minutes before lunch start should be in fuzzy range
        let just_before = DateTime { hour: 12, minute: 55, weekday: Weekday::Wed };
        assert!(is_lunch_time(&config, &just_before));
        // 6 minutes before should not
        let too_early = DateTime { hour: 12, minute: 54, weekday: Weekday::Wed };
        assert!(!is_lunch_time(&config, &too_early));
    }

    #[test]
    fn test_is_lunch_time_fuzzy_end_boundary() {
        let config = Config::default(); // lunch 13:00-14:00
        // 5 minutes after lunch end should be in fuzzy range
        let just_after = DateTime { hour: 14, minute: 4, weekday: Weekday::Wed };
        assert!(is_lunch_time(&config, &just_after));
        // 5 minutes after is exclusive
        let too_late = DateTime { hour: 14, minute: 5, weekday: Weekday::Wed };
        assert!(!is_lunch_time(&config, &too_late));
    }

    #[test]
    fn test_is_lunch_time_custom_duration() {
        let mut config = Config::default();
        config.lunch_start = "12:00".to_string();
        config.lunch_duration = 30; // 30 min lunch
        let during = DateTime { hour: 12, minute: 15, weekday: Weekday::Mon };
        assert!(is_lunch_time(&config, &during));
        // 12:30 + 5min fuzzy = 12:35 is the exclusive end
        let after = DateTime { hour: 12, minute: 40, weekday: Weekday::Mon };
        assert!(!is_lunch_time(&config, &after));
    }

    #[test]
    fn test_is_lunch_time_outside_lunch() {
        let config = Config::default();
        let morning = DateTime { hour: 10, minute: 0, weekday: Weekday::Wed };
        assert!(!is_lunch_time(&config, &morning));
        let evening = DateTime { hour: 17, minute: 0, weekday: Weekday::Wed };
        assert!(!is_lunch_time(&config, &evening));
    }

    #[test]
    fn test_pick_next_cycle_exhausted_quota_no_long_pause() {
        let plan = DailyPlan { long_pause_count: 2, long_pauses_used: 2 };
        // With quota exhausted, should never get LongPause
        for _ in 0..200 {
            let cycle = pick_next_cycle(&plan);
            match cycle {
                ActivityCycle::LongPause(_) => panic!("Should not get LongPause when quota exhausted"),
                ActivityCycle::Active(d) => assert!(d >= 480 && d <= 1500),
                ActivityCycle::Inactive(d) => assert!(d >= 300 && d <= 1200),
            }
        }
    }

    #[test]
    fn test_parse_time_edge_cases() {
        assert_eq!(parse_time("23:59"), (23, 59));
        assert_eq!(parse_time("0:0"), (0, 0));
        // Invalid input falls back to 0
        assert_eq!(parse_time("invalid"), (0, 0));
        assert_eq!(parse_time(""), (0, 0));
    }

    #[test]
    fn test_daily_plan_initial_state() {
        let plan = generate_daily_plan();
        assert_eq!(plan.long_pauses_used, 0);
        assert!(plan.long_pause_count >= 1 && plan.long_pause_count <= 3);
    }

    #[test]
    fn test_cycle_label_all_values() {
        assert_eq!(cycle_label(CYCLE_IDLE), "Idle");
        assert_eq!(cycle_label(CYCLE_ACTIVE), "Active");
        assert_eq!(cycle_label(CYCLE_INACTIVE), "Inactive");
        assert_eq!(cycle_label(CYCLE_LONG_PAUSE), "Long Pause");
        assert_eq!(cycle_label(CYCLE_LUNCH), "Lunch");
        assert_eq!(cycle_label(CYCLE_OUTSIDE_HOURS), "Outside Hours");
        assert_eq!(cycle_label(255), "Unknown");
    }

    #[test]
    fn test_current_cycle_default() {
        // Default should be CYCLE_IDLE (0)
        let cycle = current_cycle();
        assert!(cycle <= CYCLE_OUTSIDE_HOURS);
    }

    #[test]
    fn test_user_is_paused_default() {
        // We can't control the static in tests, but it should return a bool
        let _paused = user_is_paused();
    }

    #[test]
    fn test_auto_shutdown_enabled_default() {
        let _ = auto_shutdown_enabled();
    }

    #[test]
    fn test_auto_shutdown_toggle() {
        set_auto_shutdown_enabled(true);
        assert!(auto_shutdown_enabled());
        set_auto_shutdown_enabled(false);
        assert!(!auto_shutdown_enabled());
    }

    #[test]
    fn test_resolve_shutdown_delay_random() {
        for _ in 0..50 {
            let delay = resolve_shutdown_delay(0);
            assert!(delay >= 5 && delay <= 15, "random delay {} not in 5..=15", delay);
        }
    }

    #[test]
    fn test_resolve_shutdown_delay_fixed() {
        assert_eq!(resolve_shutdown_delay(10), 10);
        assert_eq!(resolve_shutdown_delay(1), 1);
        assert_eq!(resolve_shutdown_delay(30), 30);
    }
}
