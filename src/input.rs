use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::thread::sleep;
use std::time::Duration;

use rand::Rng;
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetLastInputInfo, LASTINPUTINFO, SendInput, INPUT, INPUT_0, INPUT_TYPE, KEYBDINPUT,
    KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, MOUSEEVENTF_MOVE, MOUSEINPUT, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

const INPUT_KEYBOARD: INPUT_TYPE = INPUT_TYPE(1);

/// How long (ms) the user must be idle before we resume simulation.
const IDLE_THRESHOLD_MS: u32 = 60_000;

/// Tick count of our last simulated input (updated after every SendInput we issue).
static LAST_SIMULATED_TICK: AtomicU32 = AtomicU32::new(0);

/// Session counters for GUI display.
static KEYSTROKE_COUNT: AtomicU32 = AtomicU32::new(0);
static MOUSE_MOVE_COUNT: AtomicU32 = AtomicU32::new(0);

/// Returns the total number of simulated keystrokes this session.
pub fn keystroke_count() -> u32 {
    KEYSTROKE_COUNT.load(Ordering::Relaxed)
}

/// Returns the total number of simulated mouse moves this session.
pub fn mouse_move_count() -> u32 {
    MOUSE_MOVE_COUNT.load(Ordering::Relaxed)
}

/// Records the current tick count as the time of our last simulated input.
fn mark_simulated() {
    let tick = unsafe { windows::Win32::System::SystemInformation::GetTickCount() };
    LAST_SIMULATED_TICK.store(tick, Ordering::Relaxed);
}

/// Returns the system idle time in milliseconds (time since last input event).
pub fn system_idle_ms() -> u32 {
    let mut lii = LASTINPUTINFO {
        cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    unsafe {
        let _ = GetLastInputInfo(&mut lii);
        let now = windows::Win32::System::SystemInformation::GetTickCount();
        now.wrapping_sub(lii.dwTime)
    }
}

/// Returns true if the user is actively using the computer.
/// Compares GetLastInputInfo against our last simulated input to distinguish
/// real user input from our own silent keystrokes/mouse jitter.
pub fn user_is_active() -> bool {
    let mut lii = LASTINPUTINFO {
        cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    unsafe {
        let _ = GetLastInputInfo(&mut lii);
    }

    let our_last = LAST_SIMULATED_TICK.load(Ordering::Relaxed);

    // If we haven't sent any simulated input yet, can't distinguish — assume not active
    if our_last == 0 {
        return false;
    }

    // Time elapsed between our last simulation and the system's last input event.
    // If the system saw input significantly after our last send, it's the user.
    let diff = lii.dwTime.wrapping_sub(our_last);

    // diff > 500ms means input arrived well after ours (user did something).
    // diff < 0x80000000 guards against wrapping (would mean our_last is actually newer).
    diff > 500 && diff < 0x80000000
}

/// Blocks until the user has been idle for at least `IDLE_THRESHOLD_MS`.
/// During this wait we send NO simulated input, so GetLastInputInfo only
/// reflects real user activity. Checks every second, respects shutdown.
pub fn wait_for_user_idle(shutdown: &AtomicBool) {
    loop {
        if shutdown.load(Ordering::Relaxed) {
            return;
        }
        if system_idle_ms() >= IDLE_THRESHOLD_MS {
            // Reset our marker so the first simulation after resume
            // doesn't immediately look like user activity.
            mark_simulated();
            return;
        }
        sleep(Duration::from_secs(1));
    }
}

fn make_key_input(vk: u16, scan: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// Sends a silent keystroke (VK_F13) that Windows registers as keyboard activity
/// but no application responds to. This keeps presence indicators active without
/// interfering with the user's work.
pub fn send_silent_keystroke() {
    const VK_F13: u16 = 0x7C;
    let inputs = [
        make_key_input(VK_F13, 0, KEYBD_EVENT_FLAGS(0)),
        make_key_input(VK_F13, 0, KEYEVENTF_KEYUP),
    ];
    unsafe {
        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
    mark_simulated();
    KEYSTROKE_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// Returns a random inter-character delay in ms (40–250ms).
pub fn random_char_delay() -> u64 {
    rand::thread_rng().gen_range(40..=250)
}

/// Returns a random inter-word delay in ms (150–600ms).
pub fn random_word_delay() -> u64 {
    rand::thread_rng().gen_range(150..=600)
}

/// Simulates typing activity with silent keystrokes at human-like intervals.
/// Sends invisible VK_F13 presses that register as keyboard activity without
/// producing any visible output in any application.
pub fn simulate_typing_activity(phrase: &str, shutdown: &AtomicBool) {
    for c in phrase.chars() {
        if shutdown.load(Ordering::Relaxed) {
            return;
        }

        // If the user starts typing/moving, pause until they're idle again
        if user_is_active() {
            crate::scheduler::set_user_paused(true);
            wait_for_user_idle(shutdown);
            crate::scheduler::set_user_paused(false);
            if shutdown.load(Ordering::Relaxed) {
                return;
            }
        }

        if c == ' ' {
            send_silent_keystroke();
            sleep(Duration::from_millis(random_word_delay()));
            continue;
        }

        send_silent_keystroke();
        sleep(Duration::from_millis(random_char_delay()));
    }
}

const INPUT_MOUSE: INPUT_TYPE = INPUT_TYPE(0);

/// Sends a relative mouse move via SendInput.
fn send_mouse_move_relative(dx: i32, dy: i32) {
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx,
                dy,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_MOVE,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    unsafe {
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
}

/// Moves mouse by 1 pixel and back, registering as mouse activity without
/// any visible cursor displacement. The OS detects the movement events,
/// keeping presence indicators active.
pub fn move_mouse_silent() {
    let mut rng = rand::thread_rng();
    // Pick a random direction for the micro-jitter
    let dx: i32 = if rng.gen_bool(0.5) { 1 } else { -1 };
    let dy: i32 = if rng.gen_bool(0.5) { 1 } else { -1 };

    // Check we're not at screen edge (would make the cursor stick)
    let mut cursor = POINT { x: 0, y: 0 };
    unsafe {
        let _ = GetCursorPos(&mut cursor);
    }
    // Adjust direction if at edge to avoid cursor getting stuck
    let dx = if cursor.x <= 1 { 1 } else if cursor.x >= 3000 { -1 } else { dx };
    let dy = if cursor.y <= 1 { 1 } else if cursor.y >= 2000 { -1 } else { dy };

    send_mouse_move_relative(dx, dy);
    sleep(Duration::from_millis(50));
    send_mouse_move_relative(-dx, -dy);
    mark_simulated();
    MOUSE_MOVE_COUNT.fetch_add(1, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_delay_within_range() {
        for _ in 0..100 {
            let delay = random_char_delay();
            assert!(delay >= 40 && delay <= 250);
        }
    }

    #[test]
    fn test_word_delay_within_range() {
        for _ in 0..100 {
            let delay = random_word_delay();
            assert!(delay >= 150 && delay <= 600);
        }
    }

    #[test]
    fn test_keystroke_count_returns_value() {
        // Counter starts at 0 or whatever accumulated in this test run
        let count = keystroke_count();
        assert!(count < u32::MAX);
    }

    #[test]
    fn test_mouse_move_count_returns_value() {
        let count = mouse_move_count();
        assert!(count < u32::MAX);
    }
}
