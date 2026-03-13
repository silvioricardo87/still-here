use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rand::Rng;

use crate::config::{Config, Schedule, Weekday};

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

/// Returns the current local hour offset in seconds by checking the local time
/// vs UTC. We approximate by using the system timezone offset.
/// Since Rust std has no timezone support, we use UTC for scheduling logic
/// (acceptable for this use case where users configure explicit times).
pub fn current_datetime() -> DateTime {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Day of week: epoch (Jan 1 1970) was a Thursday = 4
    // 0=Sun, 1=Mon, 2=Tue, 3=Wed, 4=Thu, 5=Fri, 6=Sat
    let day_of_week = ((secs / 86400 + 4) % 7) as u8;
    let weekday = match day_of_week {
        0 => Weekday::Sun,
        1 => Weekday::Mon,
        2 => Weekday::Tue,
        3 => Weekday::Wed,
        4 => Weekday::Thu,
        5 => Weekday::Fri,
        6 => Weekday::Sat,
        _ => Weekday::Mon, // unreachable
    };

    let seconds_today = secs % 86400;
    let hour = (seconds_today / 3600) as u8;
    let minute = ((seconds_today % 3600) / 60) as u8;

    DateTime { hour, minute, weekday }
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

        // --- Outside business hours (Business schedule only) ---
        if config.schedule == Schedule::Business && !is_business_hours(&config, &now) {
            // Mouse subtle every 3–5 minutes, no typing
            if config.mouse {
                crate::input::move_mouse_subtle();
            }
            let delay = rng.gen_range(180u64..=300u64);
            if sleep_interruptible(delay, shutdown) {
                break;
            }
            continue;
        }

        // --- Lunch time ---
        if is_lunch_time(&config, &now) {
            // Mouse subtle every 2–4 minutes, no typing
            if config.mouse {
                crate::input::move_mouse_subtle();
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
                run_active_cycle(duration, &config, shutdown, &mut daily_plan);
            }
            ActivityCycle::Inactive(duration) => {
                run_inactive_cycle(duration, &config, shutdown);
            }
            ActivityCycle::LongPause(duration) => {
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

        let time_left = duration.saturating_sub(elapsed());
        let winding_down = elapsed() >= wind_down_start;

        // --- Typing ---
        if config.typing {
            let phrase = crate::dictionary::random_phrase(config.language.clone());
            crate::input::type_phrase(phrase, shutdown);
            if shutdown.load(Ordering::Relaxed) {
                return;
            }
            phrases_in_block += 1;
        }

        // --- Mouse movement during active cycle ---
        if config.mouse {
            crate::input::move_mouse(config.mouse_mode.clone());
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

        if config.mouse {
            crate::input::move_mouse_subtle();
        }

        // Wait 30–90s before next subtle move
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

        if config.mouse {
            crate::input::move_mouse_subtle();
        }

        // Wait 2–4 minutes before next subtle move
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
}
