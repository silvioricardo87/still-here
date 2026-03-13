mod config;
mod dictionary;
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
    GetMessageW, PostThreadMessageW, MSG, WM_HOTKEY, WM_QUIT,
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
                stealth::toggle_console();
            }
        }
        shutdown.store(true, Ordering::SeqCst);
    }
}

fn main() {
    // 1. Parse config
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

    // 2. Store main thread ID (needed for PostThreadMessage from ctrl handler)
    let main_tid = unsafe { GetCurrentThreadId() };
    MAIN_THREAD_ID.store(main_tid, Ordering::SeqCst);

    // 3. Register hotkey (MUST succeed before hiding console)
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

    // 4. Hide console (after hotkey is confirmed registered)
    if !config.visible {
        stealth::hide_console();
    }

    // 5. Register Ctrl+C handler
    setup_ctrl_handler();

    // 6. Start keep-alive thread
    stealth::start_keep_alive_thread(&SHUTDOWN);

    // 7. Start activity thread
    let sched_config = config.clone();
    std::thread::spawn(move || {
        scheduler::run_scheduler(sched_config, &SHUTDOWN);
    });

    // 8. Start console input thread
    stealth::start_console_input_thread(main_tid, &SHUTDOWN);

    // 9. Start display refresh thread
    let display_config = config.clone();
    let start_time = Instant::now();
    std::thread::spawn(move || {
        while !SHUTDOWN.load(Ordering::SeqCst) {
            if stealth::is_console_visible() {
                stealth::render_status(&display_config, "Active", start_time.elapsed());
            }
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    });

    // 10. Run message pump (blocks until WM_QUIT)
    run_message_pump(&SHUTDOWN);

    // 11. Cleanup
    stealth::unregister_hotkey();
    stealth::restore_sleep();
}
