use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Console::GetConsoleWindow;
use windows::Win32::System::Power::{
    SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, IsWindowVisible, SetForegroundWindow, SetWindowLongPtrW, ShowWindow,
    GWL_EXSTYLE, SW_HIDE, SW_SHOW, WS_EX_TOOLWINDOW,
};

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
