use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

use rand::Rng;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_TYPE, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, VIRTUAL_KEY,
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
}
