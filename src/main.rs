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
use windows::Win32::Foundation::{BOOL, LPARAM, TRUE, WPARAM};
use windows::Win32::System::Console::SetConsoleCtrlHandler;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, PostThreadMessageW, TranslateMessage, MSG, WM_HOTKEY, WM_QUIT,
};

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
                    _ => {}
                }
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        shutdown.store(true, Ordering::SeqCst);
    }
}

fn main() {
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

    if args.save_config {
        match config.save() {
            Ok(_) => println!("Config saved to %TEMP%\\wsh.dat"),
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
    stealth::restore_sleep();
}
