use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreateRoundRectRgn, CreateSolidBrush, DeleteObject, EndPaint,
    FillRect, InvalidateRect, SelectObject, SetBkMode, SetTextColor, SetWindowRgn,
    TextOutW, HFONT, HGDIOBJ, PAINTSTRUCT, TRANSPARENT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect, IsWindowVisible, KillTimer,
    LoadCursorW, RegisterClassExW, SetTimer, SetWindowPos, ShowWindow, SystemParametersInfoW,
    CS_DROPSHADOW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, HWND_TOPMOST, IDC_ARROW,
    SPI_GETWORKAREA, SW_HIDE, SW_SHOWNOACTIVATE, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SWP_SHOWWINDOW, WM_CREATE, WM_DESTROY, WM_PAINT, WM_TIMER,
    WNDCLASSEXW, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

use crate::config::{Config, MouseMode, Schedule};
use crate::input;
use crate::scheduler;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const TIMER_ID: usize = 1;
const TIMER_INTERVAL_MS: u32 = 2000;

// Colors (0x00BBGGRR format for Win32 COLORREF)
const COLOR_BG: u32 = 0x00FFFFFF; // white
const COLOR_TEXT: u32 = 0x00333333; // dark gray
const COLOR_LABEL: u32 = 0x00888888; // medium gray
const COLOR_GREEN: u32 = 0x005EC522; // #22c55e
const COLOR_RED: u32 = 0x004444EF; // #ef4444
const COLOR_YELLOW: u32 = 0x0000BFEA; // #eabf00
const COLOR_SEPARATOR: u32 = 0x00EBE7E5; // #e5e7eb
const COLOR_FOOTER: u32 = 0x00AAAAAA; // #aaaaaa
// const COLOR_TAG_BG: u32 = 0x00F6F4F3; // #f3f4f6 — reserved for future tag styling

// Window dimensions (base, before DPI scaling)
const BASE_WIDTH: i32 = 280;
const BASE_HEIGHT: i32 = 370;
const MARGIN: i32 = 16;

// ---------------------------------------------------------------------------
// Global state
// ---------------------------------------------------------------------------

static mut GUI_HWND: HWND = HWND(std::ptr::null_mut());
static mut GUI_START_TIME: Option<Instant> = None;
static mut GUI_CONFIG: Option<Config> = None;
static GUI_VISIBLE: AtomicBool = AtomicBool::new(false);

// ---------------------------------------------------------------------------
// DPI helpers
// ---------------------------------------------------------------------------

fn get_dpi_scale() -> f32 {
    // Try to get per-monitor DPI, fall back to 1.0
    unsafe {
        use windows::Win32::UI::HiDpi::GetDpiForWindow;
        let hwnd = GUI_HWND;
        if hwnd.0.is_null() {
            return 1.0;
        }
        let dpi = GetDpiForWindow(hwnd);
        if dpi == 0 {
            1.0
        } else {
            dpi as f32 / 96.0
        }
    }
}

fn scaled(value: i32, scale: f32) -> i32 {
    (value as f32 * scale) as i32
}

// ---------------------------------------------------------------------------
// Window class registration
// ---------------------------------------------------------------------------

fn class_name() -> Vec<u16> {
    "StillHereOverlay\0".encode_utf16().collect()
}

pub fn register_window_class() {
    let class = class_name();
    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW | CS_DROPSHADOW,
        lpfnWndProc: Some(gui_wndproc),
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW).unwrap_or_default() },
        hbrBackground: unsafe { CreateSolidBrush(windows::Win32::Foundation::COLORREF(COLOR_BG)) },
        lpszClassName: PCWSTR(class.as_ptr()),
        ..Default::default()
    };
    unsafe {
        RegisterClassExW(&wc);
    }
}

// ---------------------------------------------------------------------------
// Window creation and positioning
// ---------------------------------------------------------------------------

pub fn create_gui_window(config: &Config, start_time: Instant) {
    unsafe {
        GUI_CONFIG = Some(config.clone());
        GUI_START_TIME = Some(start_time);
    }

    let class = class_name();
    let title: Vec<u16> = "Still Here\0".encode_utf16().collect();

    unsafe {
        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
            PCWSTR(class.as_ptr()),
            PCWSTR(title.as_ptr()),
            WS_POPUP,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            BASE_WIDTH,
            BASE_HEIGHT,
            None,
            None,
            None,
            None,
        )
        .unwrap_or(HWND(std::ptr::null_mut()));

        GUI_HWND = hwnd;

        if !hwnd.0.is_null() {
            position_window(hwnd);
            apply_rounded_corners(hwnd);
        }
    }
}

fn position_window(hwnd: HWND) {
    let scale = get_dpi_scale();
    let w = scaled(BASE_WIDTH, scale);
    let h = scaled(BASE_HEIGHT, scale);
    let margin = scaled(MARGIN, scale);

    let mut work_area = RECT::default();
    unsafe {
        let _ = SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut work_area as *mut RECT as *mut _),
            Default::default(),
        );
    }

    let x = work_area.right - w - margin;
    let y = work_area.bottom - h - margin;

    unsafe {
        let _ = SetWindowPos(hwnd, HWND_TOPMOST, x, y, w, h, SWP_NOACTIVATE);
    }
}

fn apply_rounded_corners(hwnd: HWND) {
    let scale = get_dpi_scale();
    let w = scaled(BASE_WIDTH, scale);
    let h = scaled(BASE_HEIGHT, scale);
    let radius = scaled(10, scale);

    unsafe {
        let rgn = CreateRoundRectRgn(0, 0, w + 1, h + 1, radius, radius);
        SetWindowRgn(hwnd, rgn, true);
        // Do not delete rgn — Windows owns it after SetWindowRgn
    }
}

// ---------------------------------------------------------------------------
// Show / Hide / Toggle
// ---------------------------------------------------------------------------

pub fn show_gui() {
    unsafe {
        if !GUI_HWND.0.is_null() {
            let _ = ShowWindow(GUI_HWND, SW_SHOWNOACTIVATE);
            let _ = SetWindowPos(
                GUI_HWND,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
            GUI_VISIBLE.store(true, Ordering::Relaxed);
        }
    }
}

pub fn hide_gui() {
    unsafe {
        if !GUI_HWND.0.is_null() {
            let _ = ShowWindow(GUI_HWND, SW_HIDE);
            GUI_VISIBLE.store(false, Ordering::Relaxed);
        }
    }
}

pub fn toggle_gui() {
    if is_gui_visible() {
        hide_gui();
    } else {
        show_gui();
    }
}

pub fn is_gui_visible() -> bool {
    unsafe {
        if GUI_HWND.0.is_null() {
            return false;
        }
        IsWindowVisible(GUI_HWND).as_bool()
    }
}

pub fn destroy_gui() {
    unsafe {
        if !GUI_HWND.0.is_null() {
            let _ = DestroyWindow(GUI_HWND);
            GUI_HWND = HWND(std::ptr::null_mut());
        }
    }
}

// ---------------------------------------------------------------------------
// Font creation
// ---------------------------------------------------------------------------

fn create_font(size: i32, bold: bool) -> HFONT {
    let weight = if bold { 700 } else { 400 };
    let face: Vec<u16> = "Segoe UI\0".encode_utf16().collect();
    unsafe {
        CreateFontW(
            size,            // height
            0,               // width (auto)
            0,               // escapement
            0,               // orientation
            weight,          // weight
            0,               // italic
            0,               // underline
            0,               // strikeout
            0,               // charset (DEFAULT_CHARSET)
            0,               // out precision
            0,               // clip precision
            5,               // quality (CLEARTYPE_QUALITY)
            0,               // pitch and family
            PCWSTR(face.as_ptr()),
        )
    }
}

// ---------------------------------------------------------------------------
// GDI text helpers
// ---------------------------------------------------------------------------

fn draw_text(hdc: windows::Win32::Graphics::Gdi::HDC, x: i32, y: i32, text: &str, color: u32) {
    let wide: Vec<u16> = text.encode_utf16().collect();
    unsafe {
        SetTextColor(hdc, windows::Win32::Foundation::COLORREF(color));
        let _ = TextOutW(hdc, x, y, &wide);
    }
}

fn fill_rect_color(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    color: u32,
) {
    let rect = RECT {
        left,
        top,
        right,
        bottom,
    };
    unsafe {
        let brush = CreateSolidBrush(windows::Win32::Foundation::COLORREF(color));
        FillRect(hdc, &rect, brush);
        let _ = DeleteObject(brush);
    }
}

fn draw_status_dot(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    x: i32,
    y: i32,
    radius: i32,
    color: u32,
) {
    let rect = RECT {
        left: x,
        top: y,
        right: x + radius * 2,
        bottom: y + radius * 2,
    };
    unsafe {
        let brush = CreateSolidBrush(windows::Win32::Foundation::COLORREF(color));
        let rgn = CreateRoundRectRgn(
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
            radius * 2,
            radius * 2,
        );
        let _ = windows::Win32::Graphics::Gdi::FillRgn(hdc, rgn, brush);
        let _ = DeleteObject(rgn);
        let _ = DeleteObject(brush);
    }
}

// ---------------------------------------------------------------------------
// Status helpers
// ---------------------------------------------------------------------------

/// Returns the dot color for the current state.
pub fn status_dot_color(cycle: u8, user_paused: bool) -> u32 {
    if user_paused {
        return COLOR_YELLOW;
    }
    match cycle {
        scheduler::CYCLE_ACTIVE => COLOR_GREEN,
        scheduler::CYCLE_INACTIVE | scheduler::CYCLE_LONG_PAUSE | scheduler::CYCLE_LUNCH => {
            COLOR_YELLOW
        }
        scheduler::CYCLE_OUTSIDE_HOURS => COLOR_RED,
        _ => COLOR_LABEL, // idle/unknown
    }
}

/// Returns the status text for the current state.
pub fn status_text(cycle: u8, user_paused: bool) -> &'static str {
    if user_paused {
        return "User Active — Paused";
    }
    scheduler::cycle_label(cycle)
}

/// Formats a number with thousand separators.
fn format_count(n: u32) -> String {
    if n < 1_000 {
        return n.to_string();
    }
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

// ---------------------------------------------------------------------------
// WM_PAINT handler
// ---------------------------------------------------------------------------

fn handle_paint(hwnd: HWND) {
    let mut ps = PAINTSTRUCT::default();
    let hdc = unsafe { BeginPaint(hwnd, &mut ps) };

    let scale = get_dpi_scale();
    let pad = scaled(16, scale);
    let row_height = scaled(22, scale);

    // Fonts
    let font_title = create_font(scaled(16, scale), true);
    let font_normal = create_font(scaled(13, scale), false);
    let font_small = create_font(scaled(11, scale), false);

    unsafe {
        SetBkMode(hdc, TRANSPARENT);
    }

    // Get client rect and fill background
    let mut client = RECT::default();
    unsafe {
        let _ = GetClientRect(hwnd, &mut client);
    }
    fill_rect_color(hdc, client.left, client.top, client.right, client.bottom, COLOR_BG);

    #[allow(static_mut_refs)]
    let config = unsafe { GUI_CONFIG.as_ref() };
    #[allow(static_mut_refs)]
    let start_time = unsafe { GUI_START_TIME };

    let (config, start_time) = match (config, start_time) {
        (Some(c), Some(t)) => (c, t),
        _ => {
            unsafe {
                let _ = EndPaint(hwnd, &ps);
            }
            return;
        }
    };

    let cycle = scheduler::current_cycle();
    let user_paused = scheduler::user_is_paused();
    let uptime = start_time.elapsed();
    let hours = uptime.as_secs() / 3600;
    let minutes = (uptime.as_secs() % 3600) / 60;
    let seconds = uptime.as_secs() % 60;

    let mut y = pad;

    // --- Header: Title ---
    unsafe {
        SelectObject(hdc, HGDIOBJ(font_title.0));
    }
    draw_text(hdc, pad, y, "Still Here", COLOR_TEXT);
    y += scaled(24, scale);

    // --- Status line with dot ---
    let dot_radius = scaled(5, scale);
    let dot_color = status_dot_color(cycle, user_paused);
    let status = status_text(cycle, user_paused);
    draw_status_dot(hdc, pad, y + scaled(3, scale), dot_radius, dot_color);

    unsafe {
        SelectObject(hdc, HGDIOBJ(font_normal.0));
    }
    draw_text(
        hdc,
        pad + dot_radius * 2 + scaled(8, scale),
        y,
        status,
        COLOR_TEXT,
    );
    y += row_height + scaled(4, scale);

    // --- Separator ---
    fill_rect_color(hdc, pad, y, client.right - pad, y + 1, COLOR_SEPARATOR);
    y += scaled(12, scale);

    // --- Info rows ---
    let label_x = pad;
    let value_x = pad + scaled(100, scale);

    // Uptime
    draw_text(hdc, label_x, y, "Uptime", COLOR_LABEL);
    draw_text(
        hdc,
        value_x,
        y,
        &format!("{:02}h {:02}m {:02}s", hours, minutes, seconds),
        COLOR_TEXT,
    );
    y += row_height;

    // Typing
    draw_text(hdc, label_x, y, "Typing", COLOR_LABEL);
    let (typing_str, typing_color) = if config.typing {
        ("ON", COLOR_GREEN)
    } else {
        ("OFF", COLOR_RED)
    };
    draw_text(hdc, value_x, y, typing_str, typing_color);
    y += row_height;

    // Mouse
    draw_text(hdc, label_x, y, "Mouse", COLOR_LABEL);
    let (mouse_str, mouse_color) = if config.mouse {
        ("ON (silent)", COLOR_GREEN)
    } else {
        ("OFF", COLOR_RED)
    };
    draw_text(hdc, value_x, y, mouse_str, mouse_color);
    y += row_height;

    // Mouse mode
    draw_text(hdc, label_x, y, "Mouse Mode", COLOR_LABEL);
    let mode_str = match config.mouse_mode {
        MouseMode::Subtle => "Subtle",
        MouseMode::Wide => "Wide",
        MouseMode::Mixed => "Mixed",
    };
    draw_text(hdc, value_x, y, mode_str, COLOR_TEXT);
    y += row_height;

    // Schedule
    draw_text(hdc, label_x, y, "Schedule", COLOR_LABEL);
    let sched_str = match config.schedule {
        Schedule::Always => "Always".to_string(),
        Schedule::Business => format!("{}-{}", config.schedule_start, config.schedule_end),
    };
    draw_text(hdc, value_x, y, &sched_str, COLOR_TEXT);
    y += row_height;

    // Language
    draw_text(hdc, label_x, y, "Language", COLOR_LABEL);
    let lang_str = match config.language {
        crate::config::Language::PtBr => "pt-br",
        crate::config::Language::En => "en",
    };
    draw_text(hdc, value_x, y, lang_str, COLOR_TEXT);
    y += row_height;

    // Hotkey
    draw_text(hdc, label_x, y, "Hotkey", COLOR_LABEL);
    draw_text(hdc, value_x, y, &config.hotkey, COLOR_TEXT);
    y += row_height;

    // --- Separator ---
    y += scaled(4, scale);
    fill_rect_color(hdc, pad, y, client.right - pad, y + 1, COLOR_SEPARATOR);
    y += scaled(12, scale);

    // Cycle
    draw_text(hdc, label_x, y, "Cycle", COLOR_LABEL);
    draw_text(
        hdc,
        value_x,
        y,
        scheduler::cycle_label(cycle),
        COLOR_TEXT,
    );
    y += row_height;

    // Keystrokes
    draw_text(hdc, label_x, y, "Keystrokes", COLOR_LABEL);
    draw_text(
        hdc,
        value_x,
        y,
        &format_count(input::keystroke_count()),
        COLOR_TEXT,
    );
    y += row_height;

    // Mouse moves
    draw_text(hdc, label_x, y, "Mouse Moves", COLOR_LABEL);
    draw_text(
        hdc,
        value_x,
        y,
        &format_count(input::mouse_move_count()),
        COLOR_TEXT,
    );
    y += row_height;

    // User status
    draw_text(hdc, label_x, y, "User", COLOR_LABEL);
    let (user_str, user_color) = if user_paused {
        ("Active (paused)", COLOR_YELLOW)
    } else {
        ("Idle", COLOR_GREEN)
    };
    draw_text(hdc, value_x, y, user_str, user_color);
    y += row_height + scaled(8, scale);

    // --- Footer ---
    fill_rect_color(hdc, pad, y, client.right - pad, y + 1, COLOR_SEPARATOR);
    y += scaled(8, scale);

    unsafe {
        SelectObject(hdc, HGDIOBJ(font_small.0));
    }
    draw_text(
        hdc,
        pad,
        y,
        &format!("{} to hide  |  Q to quit", config.hotkey),
        COLOR_FOOTER,
    );

    // Cleanup fonts
    unsafe {
        let _ = DeleteObject(HGDIOBJ(font_title.0));
        let _ = DeleteObject(HGDIOBJ(font_normal.0));
        let _ = DeleteObject(HGDIOBJ(font_small.0));
        let _ = EndPaint(hwnd, &ps);
    }
}

// ---------------------------------------------------------------------------
// Window procedure
// ---------------------------------------------------------------------------

unsafe extern "system" fn gui_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            SetTimer(hwnd, TIMER_ID, TIMER_INTERVAL_MS, None);
            LRESULT(0)
        }
        WM_TIMER => {
            if wparam.0 == TIMER_ID {
                let _ = InvalidateRect(hwnd, None, true);
            }
            LRESULT(0)
        }
        WM_PAINT => {
            handle_paint(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            KillTimer(hwnd, TIMER_ID).ok();
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_dot_color_active() {
        assert_eq!(
            status_dot_color(scheduler::CYCLE_ACTIVE, false),
            COLOR_GREEN
        );
    }

    #[test]
    fn test_status_dot_color_inactive() {
        assert_eq!(
            status_dot_color(scheduler::CYCLE_INACTIVE, false),
            COLOR_YELLOW
        );
    }

    #[test]
    fn test_status_dot_color_outside_hours() {
        assert_eq!(
            status_dot_color(scheduler::CYCLE_OUTSIDE_HOURS, false),
            COLOR_RED
        );
    }

    #[test]
    fn test_status_dot_color_user_paused_overrides() {
        // User paused should always be yellow regardless of cycle
        assert_eq!(
            status_dot_color(scheduler::CYCLE_ACTIVE, true),
            COLOR_YELLOW
        );
    }

    #[test]
    fn test_status_text_active() {
        assert_eq!(status_text(scheduler::CYCLE_ACTIVE, false), "Active");
    }

    #[test]
    fn test_status_text_user_paused() {
        assert_eq!(
            status_text(scheduler::CYCLE_ACTIVE, true),
            "User Active — Paused"
        );
    }

    #[test]
    fn test_format_count_small() {
        assert_eq!(format_count(0), "0");
        assert_eq!(format_count(42), "42");
        assert_eq!(format_count(999), "999");
    }

    #[test]
    fn test_format_count_thousands() {
        assert_eq!(format_count(1_000), "1,000");
        assert_eq!(format_count(1_247), "1,247");
        assert_eq!(format_count(12_345), "12,345");
        assert_eq!(format_count(1_234_567), "1,234,567");
    }
}
