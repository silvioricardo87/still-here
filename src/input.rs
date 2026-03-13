use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

use rand::Rng;
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_TYPE, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_MOVE, MOUSEINPUT, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN,
};

const INPUT_KEYBOARD: INPUT_TYPE = INPUT_TYPE(1);

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

/// Sends a Unicode character via SendInput (key-down + key-up in one call).
pub fn send_unicode_char(c: char) {
    let scan = c as u16;
    let inputs = [
        make_key_input(0, scan, KEYEVENTF_UNICODE),
        make_key_input(0, scan, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP),
    ];
    unsafe {
        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
}

/// Sends VK_BACK (backspace) key-down + key-up.
pub fn send_backspace() {
    const VK_BACK: u16 = 0x08;
    let inputs = [
        make_key_input(VK_BACK, 0, KEYBD_EVENT_FLAGS(0)),
        make_key_input(VK_BACK, 0, KEYEVENTF_KEYUP),
    ];
    unsafe {
        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
}

/// Sends VK_RETURN (Enter) key-down + key-up.
pub fn send_enter() {
    const VK_RETURN: u16 = 0x0D;
    let inputs = [
        make_key_input(VK_RETURN, 0, KEYBD_EVENT_FLAGS(0)),
        make_key_input(VK_RETURN, 0, KEYEVENTF_KEYUP),
    ];
    unsafe {
        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
}

/// Returns adjacent keys on the QWERTY layout for typo simulation.
pub fn nearby_keys(c: char) -> Vec<char> {
    match c {
        'a' => vec!['q', 'w', 's', 'z'],
        'b' => vec!['v', 'g', 'h', 'n'],
        'c' => vec!['x', 'd', 'f', 'v'],
        'd' => vec!['s', 'e', 'r', 'f', 'c', 'x'],
        'e' => vec!['w', 'r', 'd', 's'],
        'f' => vec!['d', 'r', 't', 'g', 'v', 'c'],
        'g' => vec!['f', 't', 'y', 'h', 'b', 'v'],
        'h' => vec!['g', 'y', 'u', 'j', 'n', 'b'],
        'i' => vec!['u', 'o', 'k', 'j'],
        'j' => vec!['h', 'u', 'i', 'k', 'm', 'n'],
        'k' => vec!['j', 'i', 'o', 'l', 'm'],
        'l' => vec!['k', 'o', 'p'],
        'm' => vec!['n', 'j', 'k'],
        'n' => vec!['b', 'h', 'j', 'm'],
        'o' => vec!['i', 'p', 'l', 'k'],
        'p' => vec!['o', 'l'],
        'q' => vec!['w', 'a'],
        'r' => vec!['e', 't', 'f', 'd'],
        's' => vec!['a', 'w', 'e', 'd', 'x', 'z'],
        't' => vec!['r', 'y', 'g', 'f'],
        'u' => vec!['y', 'i', 'j', 'h'],
        'v' => vec!['c', 'f', 'g', 'b'],
        'w' => vec!['q', 'e', 's', 'a'],
        'x' => vec!['z', 's', 'd', 'c'],
        'y' => vec!['t', 'u', 'h', 'g'],
        'z' => vec!['a', 's', 'x'],
        _ => vec![],
    }
}

/// Returns true ~8% of the time.
pub fn should_typo() -> bool {
    rand::thread_rng().gen_range(0u32..100) < 8
}

/// Returns a random inter-character delay in ms (40–250ms).
pub fn random_char_delay() -> u64 {
    rand::thread_rng().gen_range(40..=250)
}

/// Returns a random inter-word delay in ms (150–600ms).
pub fn random_word_delay() -> u64 {
    rand::thread_rng().gen_range(150..=600)
}

/// Returns a random inter-phrase delay in ms (5000–45000ms).
pub fn random_phrase_delay() -> u64 {
    rand::thread_rng().gen_range(5000..=45000)
}

/// Types a phrase with human-like delays, occasional typos, and shutdown awareness.
pub fn type_phrase(phrase: &str, shutdown: &AtomicBool) {
    let mut rng = rand::thread_rng();
    let mut at_word_boundary = false;

    for c in phrase.chars() {
        if shutdown.load(Ordering::Relaxed) {
            return;
        }

        if c == ' ' {
            send_unicode_char(c);
            at_word_boundary = true;
            sleep(Duration::from_millis(random_word_delay()));
            continue;
        }

        // Possibly insert a typo at word boundaries (start of a word)
        if at_word_boundary && should_typo() {
            let neighbors = nearby_keys(c.to_ascii_lowercase());
            if !neighbors.is_empty() {
                let typo = neighbors[rng.gen_range(0..neighbors.len())];
                // Send the typo character, brief pause, then backspace
                send_unicode_char(typo);
                sleep(Duration::from_millis(random_char_delay()));
                if shutdown.load(Ordering::Relaxed) {
                    return;
                }
                send_backspace();
                sleep(Duration::from_millis(random_char_delay()));
                if shutdown.load(Ordering::Relaxed) {
                    return;
                }
            }
        }

        at_word_boundary = false;
        send_unicode_char(c);
        sleep(Duration::from_millis(random_char_delay()));
    }
}

/// Quadratic bezier interpolation between start and end with control point.
pub fn bezier_point(start: (f64, f64), control: (f64, f64), end: (f64, f64), t: f64) -> (f64, f64) {
    let u = 1.0 - t;
    let x = u * u * start.0 + 2.0 * u * t * control.0 + t * t * end.0;
    let y = u * u * start.1 + 2.0 * u * t * control.1 + t * t * end.1;
    (x, y)
}

/// Returns (dx, dy) where each component is in [-15, -1] or [1, 15], never both zero.
pub fn random_subtle_offset() -> (i32, i32) {
    let mut rng = rand::thread_rng();
    let dx = {
        let v = rng.gen_range(1..=15i32);
        if rng.gen_bool(0.5) { -v } else { v }
    };
    let dy = {
        let v = rng.gen_range(1..=15i32);
        if rng.gen_bool(0.5) { -v } else { v }
    };
    (dx, dy)
}

/// Converts pixel coordinates to normalized 0-65535 range for absolute mouse input.
pub fn pixels_to_normalized(x: i32, y: i32, screen_w: i32, screen_h: i32) -> (i32, i32) {
    let nx = x * 65535 / (screen_w - 1);
    let ny = y * 65535 / (screen_h - 1);
    (nx, ny)
}

const INPUT_MOUSE: INPUT_TYPE = INPUT_TYPE(0);

fn send_mouse_move_absolute(nx: i32, ny: i32) {
    let input = INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: nx,
                dy: ny,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    unsafe {
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
}

/// Moves mouse by a small random offset (1–15px) from current position.
pub fn move_mouse_subtle() {
    let mut cursor = POINT { x: 0, y: 0 };
    unsafe {
        let _ = GetCursorPos(&mut cursor);
    }
    let screen_w = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let screen_h = unsafe { GetSystemMetrics(SM_CYSCREEN) };

    let (dx, dy) = random_subtle_offset();
    let new_x = (cursor.x + dx).clamp(0, screen_w - 1);
    let new_y = (cursor.y + dy).clamp(0, screen_h - 1);

    let (nx, ny) = pixels_to_normalized(new_x, new_y, screen_w, screen_h);
    send_mouse_move_absolute(nx, ny);
}

/// Moves mouse along a quadratic bezier curve to a random target position.
pub fn move_mouse_wide() {
    let mut cursor = POINT { x: 0, y: 0 };
    unsafe {
        let _ = GetCursorPos(&mut cursor);
    }
    let screen_w = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let screen_h = unsafe { GetSystemMetrics(SM_CYSCREEN) };

    let mut rng = rand::thread_rng();
    let target_x = rng.gen_range(0..screen_w);
    let target_y = rng.gen_range(0..screen_h);

    let ctrl_x = rng.gen_range(0..screen_w);
    let ctrl_y = rng.gen_range(0..screen_h);

    let start = (cursor.x as f64, cursor.y as f64);
    let control = (ctrl_x as f64, ctrl_y as f64);
    let end = (target_x as f64, target_y as f64);

    let steps = rng.gen_range(15..=25usize);
    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let (px, py) = bezier_point(start, control, end, t);
        let px = (px as i32).clamp(0, screen_w - 1);
        let py = (py as i32).clamp(0, screen_h - 1);
        let (nx, ny) = pixels_to_normalized(px, py, screen_w, screen_h);
        send_mouse_move_absolute(nx, ny);
        let delay = rng.gen_range(10..=30u64);
        sleep(Duration::from_millis(delay));
    }
}

/// Moves mouse according to the given mode.
pub fn move_mouse(mode: crate::config::MouseMode) {
    match mode {
        crate::config::MouseMode::Subtle => move_mouse_subtle(),
        crate::config::MouseMode::Wide => move_mouse_wide(),
        crate::config::MouseMode::Mixed => {
            let roll: u32 = rand::thread_rng().gen_range(0..100);
            if roll < 70 {
                move_mouse_subtle();
            } else {
                move_mouse_wide();
            }
        }
    }
}

/// Returns a random mouse movement delay in ms for active periods (10000–60000ms).
pub fn random_mouse_delay_active() -> u64 {
    rand::thread_rng().gen_range(10000..=60000)
}

/// Returns a random mouse movement delay in ms for inactive periods (30000–90000ms).
pub fn random_mouse_delay_inactive() -> u64 {
    rand::thread_rng().gen_range(30000..=90000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_typo_char_is_nearby() {
        let nearby = nearby_keys('a');
        assert!(!nearby.is_empty());
    }

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
    fn test_phrase_delay_within_range() {
        for _ in 0..100 {
            let delay = random_phrase_delay();
            assert!(delay >= 5000 && delay <= 45000);
        }
    }

    #[test]
    fn test_should_typo_probability() {
        let count = (0..10000).filter(|_| should_typo()).count();
        assert!(count > 400 && count < 1200, "Typo rate: {}/10000", count);
    }

    #[test]
    fn test_bezier_point_at_boundaries() {
        let start = (0.0, 0.0);
        let control = (50.0, 100.0);
        let end = (100.0, 0.0);
        let p0 = bezier_point(start, control, end, 0.0);
        let p1 = bezier_point(start, control, end, 1.0);
        assert_eq!(p0, (0.0, 0.0));
        assert_eq!(p1, (100.0, 0.0));
    }

    #[test]
    fn test_random_subtle_offset_within_range() {
        for _ in 0..100 {
            let (dx, dy) = random_subtle_offset();
            assert!(dx.abs() <= 15 && dy.abs() <= 15);
            assert!(dx.abs() >= 1 || dy.abs() >= 1);
        }
    }

    #[test]
    fn test_mouse_delay_active_within_range() {
        for _ in 0..100 {
            let delay = random_mouse_delay_active();
            assert!(delay >= 10000 && delay <= 60000);
        }
    }

    #[test]
    fn test_mouse_delay_inactive_within_range() {
        for _ in 0..100 {
            let delay = random_mouse_delay_inactive();
            assert!(delay >= 30000 && delay <= 90000);
        }
    }
}
