mod config;
mod dictionary;
mod input;
mod stealth;

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use clap::Parser;
use config::{CliArgs, Config};
use windows::Win32::Foundation::{BOOL, LPARAM, TRUE, WPARAM};
use windows::Win32::System::Console::SetConsoleCtrlHandler;
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

    println!("Specter starting with config: {:?}", config);
}
