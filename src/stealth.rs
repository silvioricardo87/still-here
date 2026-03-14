use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::Console::{
    GetConsoleWindow, GetStdHandle, ReadConsoleInputW, SetConsoleCursorPosition, COORD,
    INPUT_RECORD, KEY_EVENT, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
};
use windows::Win32::System::Power::{
    SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    HOT_KEY_MODIFIERS, RegisterHotKey, UnregisterHotKey,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, IsWindowVisible, PostThreadMessageW, SetForegroundWindow,
    SetWindowLongPtrW, ShowWindow, GWL_EXSTYLE, SW_HIDE, SW_SHOW, WM_QUIT, WS_EX_TOOLWINDOW,
};

use crate::config::{Config, Schedule};

/// Hides the console window and removes it from Alt+Tab and the taskbar.
pub fn hide_console() {
    unsafe {
        let hwnd: HWND = GetConsoleWindow();
        if hwnd.0 == std::ptr::null_mut() {
            return;
        }
        // Remove from Alt+Tab and taskbar by setting WS_EX_TOOLWINDOW
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(
            hwnd,
            GWL_EXSTYLE,
            ex_style | WS_EX_TOOLWINDOW.0 as isize,
        );
        let _ = ShowWindow(hwnd, SW_HIDE);
    }
}

/// Shows and focuses the console window.
pub fn show_console() {
    unsafe {
        let hwnd: HWND = GetConsoleWindow();
        if hwnd.0 == std::ptr::null_mut() {
            return;
        }
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
    }
}

/// Returns true if the console window is currently visible.
pub fn is_console_visible() -> bool {
    unsafe {
        let hwnd: HWND = GetConsoleWindow();
        if hwnd.0 == std::ptr::null_mut() {
            return false;
        }
        IsWindowVisible(hwnd).as_bool()
    }
}

/// Toggles the console window visibility.
pub fn toggle_console() {
    if is_console_visible() {
        hide_console();
    } else {
        show_console();
    }
}

/// Prevents the display and system from sleeping.
pub fn prevent_sleep() {
    unsafe {
        SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED);
    }
}

/// Restores default sleep behavior by clearing the execution state flags.
pub fn restore_sleep() {
    unsafe {
        SetThreadExecutionState(ES_CONTINUOUS);
    }
}

/// Spawns a background thread that calls `prevent_sleep()` every 30 seconds.
/// Checks the shutdown flag in 1-second increments to stay responsive.
pub fn start_keep_alive_thread(shutdown: &'static AtomicBool) {
    thread::spawn(move || {
        while !shutdown.load(Ordering::Relaxed) {
            prevent_sleep();
            // Sleep in 1s increments so shutdown is detected promptly
            for _ in 0..30 {
                if shutdown.load(Ordering::Relaxed) {
                    break;
                }
                thread::sleep(Duration::from_secs(1));
            }
        }
        restore_sleep();
    });
}

/// Tries to register the given hotkey string (e.g. "Ctrl+Shift+F9") as hotkey id=1.
/// Falls back to Ctrl+Shift+F10, then Ctrl+Shift+F11 on failure.
/// Returns the successfully registered hotkey string, or an error if all attempts fail.
pub fn register_hotkey(hotkey_str: &str) -> Result<String, String> {
    // Try the requested hotkey first
    if let Ok((mods, vk)) = crate::config::parse_hotkey(hotkey_str) {
        unsafe {
            if RegisterHotKey(HWND::default(), 1, HOT_KEY_MODIFIERS(mods.0), vk as u32).is_ok() {
                return Ok(hotkey_str.to_string());
            }
        }
    }

    // Fallback 1: Ctrl+Shift+F10
    let fallback1 = "Ctrl+Shift+F10";
    if let Ok((mods, vk)) = crate::config::parse_hotkey(fallback1) {
        unsafe {
            if RegisterHotKey(HWND::default(), 1, HOT_KEY_MODIFIERS(mods.0), vk as u32).is_ok() {
                return Ok(fallback1.to_string());
            }
        }
    }

    // Fallback 2: Ctrl+Shift+F11
    let fallback2 = "Ctrl+Shift+F11";
    if let Ok((mods, vk)) = crate::config::parse_hotkey(fallback2) {
        unsafe {
            if RegisterHotKey(HWND::default(), 1, HOT_KEY_MODIFIERS(mods.0), vk as u32).is_ok() {
                return Ok(fallback2.to_string());
            }
        }
    }

    Err(format!(
        "Failed to register hotkey '{}' or any fallback (Ctrl+Shift+F10, Ctrl+Shift+F11)",
        hotkey_str
    ))
}

/// Unregisters the global hotkey with id=1.
pub fn unregister_hotkey() {
    unsafe {
        let _ = UnregisterHotKey(HWND::default(), 1);
    }
}

/// Renders a status panel to the console by repositioning the cursor to (0,0)
/// and overwriting all lines. This avoids flicker from a full clear.
pub fn render_status(config: &Config, status: &str, uptime: Duration) {
    let total_secs = uptime.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    let typing_str = if config.typing { "ON" } else { "OFF" };
    let mouse_str = if config.mouse { "ON (silent)" } else { "OFF" };
    let schedule_str = match config.schedule {
        Schedule::Always => "always".to_string(),
        Schedule::Business => format!(
            "business ({}-{})",
            config.schedule_start, config.schedule_end
        ),
    };

    // Move cursor to top-left (0, 0) before drawing
    unsafe {
        if let Ok(handle) = GetStdHandle(STD_OUTPUT_HANDLE) {
            let _ = SetConsoleCursorPosition(handle, COORD { X: 0, Y: 0 });
        }
    }

    // Width of the inner area (between the box borders) is 38 characters
    print!("╔══════════════════════════════════════╗\r\n");
    print!("║  Still Here - Activity Simulator     ║\r\n");
    print!("╠══════════════════════════════════════╣\r\n");
    print!(
        "║  Status:   {:<26}║\r\n",
        status
    );
    print!(
        "║  Uptime:   {:02}h {:02}m {:02}s{:<15}║\r\n",
        hours, minutes, seconds, ""
    );
    print!(
        "║  Typing:   {:<26}║\r\n",
        typing_str
    );
    print!(
        "║  Mouse:    {:<26}║\r\n",
        mouse_str
    );
    print!(
        "║  Schedule: {:<26}║\r\n",
        schedule_str
    );
    print!(
        "║  Hotkey:   {:<26}║\r\n",
        config.hotkey
    );
    print!("╠══════════════════════════════════════╣\r\n");
    print!("║  Press Q to quit | Hotkey to toggle  ║\r\n");
    print!("╚══════════════════════════════════════╝\r\n");
}

/// Spawns a thread that reads console keyboard input and posts WM_QUIT when
/// the user presses Q or q, also setting the shutdown flag.
pub fn start_console_input_thread(main_thread_id: u32, shutdown: &'static AtomicBool) {
    thread::spawn(move || {
        unsafe {
            let handle = match GetStdHandle(STD_INPUT_HANDLE) {
                Ok(h) => h,
                Err(_) => return,
            };

            let mut record = INPUT_RECORD::default();
            let mut num_read: u32 = 0;

            loop {
                if shutdown.load(Ordering::Relaxed) {
                    break;
                }

                // ReadConsoleInputW blocks until at least one input event is available
                let result = ReadConsoleInputW(handle, std::slice::from_mut(&mut record), &mut num_read);
                if result.is_err() || num_read == 0 {
                    break;
                }

                // Only handle key-down events
                if record.EventType == KEY_EVENT as u16 {
                    let key_event = record.Event.KeyEvent;
                    if key_event.bKeyDown.as_bool() {
                        let ch = key_event.uChar.UnicodeChar;
                        // Q (0x51) or q (0x71)
                        if ch == b'Q' as u16 || ch == b'q' as u16 {
                            shutdown.store(true, Ordering::SeqCst);
                            let _ = PostThreadMessageW(main_thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
                            break;
                        }
                    }
                }
            }
        }
    });
}
