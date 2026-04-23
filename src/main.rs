// Hide the console window entirely — no taskbar icon, no flash on startup.
// In release builds we use the Windows subsystem (no console allocated).
// In debug builds we keep the console for development convenience.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod dictionary;
mod gui;
mod input;
mod scheduler;
mod stealth;

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Instant;

use clap::Parser;
use config::{CliArgs, Config};
use windows::Win32::Foundation::{BOOL, HANDLE, LPARAM, TRUE, WPARAM};
use windows::Win32::System::Console::SetConsoleCtrlHandler;
use windows::Win32::System::Threading::{CreateMutexW, GetCurrentThreadId};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, PostThreadMessageW, TranslateMessage, MSG, WM_HOTKEY, WM_QUIT,
};
use windows::core::w;

static SHUTDOWN: AtomicBool = AtomicBool::new(false);
static MAIN_THREAD_ID: AtomicU32 = AtomicU32::new(0);

unsafe extern "system" fn ctrl_handler(_: u32) -> BOOL {
    SHUTDOWN.store(true, Ordering::SeqCst);
    let tid = MAIN_THREAD_ID.load(Ordering::SeqCst);
    let _ = PostThreadMessageW(tid, WM_QUIT, WPARAM(0), LPARAM(0));
    TRUE
}

fn setup_ctrl_handler() {
    unsafe {
        let _ = SetConsoleCtrlHandler(Some(ctrl_handler), true);
    }
}

fn run_message_pump(shutdown: &'static AtomicBool) {
    let mut msg = MSG::default();
    unsafe {
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            if msg.message == WM_HOTKEY {
                match msg.wParam.0 {
                    1 => gui::toggle_gui(),
                    2 => {
                        // Ctrl+Shift+Q — quit
                        shutdown.store(true, Ordering::SeqCst);
                        break;
                    }
                    3 => {
                        // Ctrl+Shift+F12 — toggle auto-shutdown
                        let current = scheduler::auto_shutdown_enabled();
                        scheduler::set_auto_shutdown_enabled(!current);
                    }
                    _ => {}
                }
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        shutdown.store(true, Ordering::SeqCst);
    }
}

/// Attempts to acquire a system-wide named mutex. Returns the handle if this
/// is the first instance, or `None` if another instance already holds it.
fn acquire_single_instance_lock() -> Option<HANDLE> {
    unsafe {
        let handle = CreateMutexW(None, true, w!("Global\\StillHere_SingleInstance")).ok()?;
        // ERROR_ALREADY_EXISTS (183) means another instance owns the mutex
        if windows::Win32::Foundation::GetLastError().0 == 183 {
            return None;
        }
        Some(handle)
    }
}

fn main() {
    // 0. Single-instance guard
    let _mutex = match acquire_single_instance_lock() {
        Some(handle) => handle,
        None => return, // Another instance is already running
    };

    // 1. Enable DPI awareness
    unsafe {
        use windows::Win32::UI::HiDpi::{
            SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        };
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }

    // 2. Parse config
    let args = CliArgs::parse();
    let mut config = Config::load();
    config.merge_cli(&args);

    // Initialize auto-shutdown state from config
    scheduler::set_auto_shutdown_enabled(config.auto_shutdown);

    if args.save_config {
        match config.save() {
            Ok(_) => println!("Config saved to %TEMP%\\sth.dat"),
            Err(e) => eprintln!("Failed to save config: {}", e),
        }
        return;
    }

    // 3. Store main thread ID (needed for PostThreadMessage from ctrl handler)
    let main_tid = unsafe { GetCurrentThreadId() };
    MAIN_THREAD_ID.store(main_tid, Ordering::SeqCst);

    // 4. Register hotkey (MUST succeed before hiding console)
    match stealth::register_hotkey(&config.hotkey) {
        Ok(key) => {
            if key != config.hotkey {
                eprintln!("Note: Using fallback hotkey {}", key);
            }
        }
        Err(e) => {
            eprintln!("FATAL: {}", e);
            return;
        }
    }

    // 4b. Register quit hotkey (Ctrl+Shift+Q)
    if let Err(e) = stealth::register_quit_hotkey() {
        eprintln!("Warning: {}", e);
    }

    // 4c. Register auto-shutdown toggle hotkey (Ctrl+Shift+F12)
    if let Err(e) = stealth::register_auto_shutdown_hotkey() {
        eprintln!("Warning: {}", e);
    }

    // 5. Always hide the console (GUI replaces it)
    stealth::hide_console();

    // 6. Register Ctrl+C handler
    setup_ctrl_handler();

    // 7. Start keep-alive thread
    stealth::start_keep_alive_thread(&SHUTDOWN);

    // 8. Create GUI window on main thread
    let start_time = Instant::now();
    gui::register_window_class();
    gui::create_gui_window(&config, start_time);

    // 9. Show GUI if --visible
    if config.visible {
        gui::show_gui();
    }

    // 10. Start activity thread
    let sched_config = config.clone();
    std::thread::spawn(move || {
        scheduler::run_scheduler(sched_config, &SHUTDOWN);
    });

    // 11. Start console input thread (still needed for Q to quit)
    stealth::start_console_input_thread(main_tid, &SHUTDOWN);

    // 12. Run message pump (blocks until WM_QUIT)
    // WM_TIMER for GUI repaint is handled automatically via DispatchMessageW
    run_message_pump(&SHUTDOWN);

    // 13. Cleanup
    gui::destroy_gui();
    stealth::unregister_hotkey();
    stealth::unregister_quit_hotkey();
    stealth::unregister_auto_shutdown_hotkey();
    stealth::restore_sleep();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_instance_lock_succeeds_first_time() {
        // First acquisition should succeed
        let handle = acquire_single_instance_lock();
        assert!(handle.is_some(), "First lock acquisition should succeed");

        // Second acquisition with the same mutex name should fail
        let handle2 = acquire_single_instance_lock();
        assert!(handle2.is_none(), "Second lock acquisition should fail");

        // Keep handle alive until assertions complete
        let _ = handle;
    }
}
