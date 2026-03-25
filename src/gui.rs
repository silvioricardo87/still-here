use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreateRoundRectRgn, CreateSolidBrush, DeleteObject, EndPaint,
    FillRect, FrameRect, InvalidateRect, SelectObject, SetBkMode, SetTextColor, SetWindowRgn,
    TextOutW, HFONT, HGDIOBJ, PAINTSTRUCT, TRANSPARENT,
};
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ,
    REG_VALUE_TYPE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect, IsWindowVisible, KillTimer,
    LoadCursorW, RegisterClassExW, SetTimer, SetWindowPos, ShowWindow, SystemParametersInfoW,
    CS_DROPSHADOW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, HWND_TOPMOST, IDC_ARROW,
    SPI_GETWORKAREA, SW_HIDE, SW_SHOWNOACTIVATE, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
    SWP_SHOWWINDOW, WM_CREATE, WM_DESTROY, WM_PAINT, WM_TIMER,
    WNDCLASSEXW, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
};

use crate::config::{Config, Language, MouseMode, Schedule};
use crate::input;
use crate::scheduler;

// ---------------------------------------------------------------------------
// Localization
// ---------------------------------------------------------------------------

struct GuiStrings {
    title: &'static str,
    uptime: &'static str,
    typing: &'static str,
    mouse: &'static str,
    mouse_mode: &'static str,
    schedule: &'static str,
    language_label: &'static str,
    hotkey: &'static str,
    cycle: &'static str,
    keystrokes: &'static str,
    mouse_moves: &'static str,
    user: &'static str,
    on: &'static str,
    off: &'static str,
    on_silent: &'static str,
    subtle: &'static str,
    wide: &'static str,
    mixed: &'static str,
    always: &'static str,
    active_paused: &'static str,
    idle: &'static str,
    user_active_paused: &'static str,
    footer_hide: &'static str,
    footer_quit: &'static str,
    cycle_idle: &'static str,
    cycle_active: &'static str,
    cycle_inactive: &'static str,
    cycle_long_pause: &'static str,
    cycle_lunch: &'static str,
    cycle_outside: &'static str,
    auto_shutdown: &'static str,
}

const STRINGS_EN: GuiStrings = GuiStrings {
    title: "Still Here",
    uptime: "Uptime",
    typing: "Typing",
    mouse: "Mouse",
    mouse_mode: "Mouse Mode",
    schedule: "Schedule",
    language_label: "Language",
    hotkey: "Hotkey",
    cycle: "Cycle",
    keystrokes: "Keystrokes",
    mouse_moves: "Mouse Moves",
    user: "User",
    on: "ON",
    off: "OFF",
    on_silent: "ON (silent)",
    subtle: "Subtle",
    wide: "Wide",
    mixed: "Mixed",
    always: "Always",
    active_paused: "Active (paused)",
    idle: "Idle",
    user_active_paused: "User Active \u{2014} Paused",
    footer_hide: "to hide",
    footer_quit: "to quit",
    cycle_idle: "Idle",
    cycle_active: "Active",
    cycle_inactive: "Inactive",
    cycle_long_pause: "Long Pause",
    cycle_lunch: "Lunch",
    cycle_outside: "Outside Hours",
    auto_shutdown: "Auto-Shutdown",
};

const STRINGS_PT: GuiStrings = GuiStrings {
    title: "Still Here",
    uptime: "Ativo h\u{00e1}",
    typing: "Digita\u{00e7}\u{00e3}o",
    mouse: "Mouse",
    mouse_mode: "Modo Mouse",
    schedule: "Hor\u{00e1}rio",
    language_label: "Idioma",
    hotkey: "Atalho",
    cycle: "Ciclo",
    keystrokes: "Teclas",
    mouse_moves: "Mov. Mouse",
    user: "Usu\u{00e1}rio",
    on: "SIM",
    off: "N\u{00c3}O",
    on_silent: "SIM (silencioso)",
    subtle: "Sutil",
    wide: "Amplo",
    mixed: "Misto",
    always: "Sempre",
    active_paused: "Ativo (pausado)",
    idle: "Ocioso",
    user_active_paused: "Usu\u{00e1}rio Ativo \u{2014} Pausado",
    footer_hide: "p/ ocultar",
    footer_quit: "p/ sair",
    cycle_idle: "Ocioso",
    cycle_active: "Ativo",
    cycle_inactive: "Inativo",
    cycle_long_pause: "Pausa Longa",
    cycle_lunch: "Almo\u{00e7}o",
    cycle_outside: "Fora do Hor\u{00e1}rio",
    auto_shutdown: "Auto Desligar",
};

fn gui_strings(lang: &Language) -> &'static GuiStrings {
    match lang {
        Language::PtBr => &STRINGS_PT,
        Language::En => &STRINGS_EN,
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const TIMER_ID: usize = 1;
const TIMER_INTERVAL_MS: u32 = 2000;

/// Extended window style: TOOLWINDOW hides from taskbar/Alt+Tab, TOPMOST keeps on top.
const GUI_EX_STYLE: windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE =
    windows::Win32::UI::WindowsAndMessaging::WINDOW_EX_STYLE(
        WS_EX_TOPMOST.0 | WS_EX_TOOLWINDOW.0,
    );

// Window dimensions (base, before DPI scaling)
const BASE_WIDTH: i32 = 280;
const BASE_HEIGHT: i32 = 392;
const MARGIN: i32 = 16;

// ---------------------------------------------------------------------------
// Theme (light / dark)
// ---------------------------------------------------------------------------

struct Theme {
    bg: u32,
    text: u32,
    label: u32,
    green: u32,
    red: u32,
    yellow: u32,
    separator: u32,
    footer: u32,
    border: u32,
}

// Colors use 0x00BBGGRR format (Win32 COLORREF)
const LIGHT_THEME: Theme = Theme {
    bg: 0x00FFFFFF,         // white
    text: 0x00333333,       // dark gray
    label: 0x00888888,      // medium gray
    green: 0x005EC522,      // #22c55e
    red: 0x004444EF,        // #ef4444
    yellow: 0x0000BFEA,     // #eabf00
    separator: 0x00EBE7E5,  // #e5e7eb
    footer: 0x00AAAAAA,     // #aaaaaa
    border: 0x00CCCCCC,     // #cccccc
};

const DARK_THEME: Theme = Theme {
    bg: 0x002B2B2B,         // #2b2b2b
    text: 0x00E0E0E0,       // #e0e0e0
    label: 0x00999999,      // #999999
    green: 0x005EC522,      // #22c55e
    red: 0x004444EF,        // #ef4444
    yellow: 0x0000BFEA,     // #eabf00
    separator: 0x00444444,  // #444444
    footer: 0x00777777,     // #777777
    border: 0x00555555,     // #555555
};

fn get_theme() -> &'static Theme {
    unsafe {
        if IS_DARK_MODE {
            &DARK_THEME
        } else {
            &LIGHT_THEME
        }
    }
}

// ---------------------------------------------------------------------------
// Global state
// ---------------------------------------------------------------------------

static mut GUI_HWND: HWND = HWND(std::ptr::null_mut());
static mut GUI_START_TIME: Option<Instant> = None;
static mut GUI_CONFIG: Option<Config> = None;
static GUI_VISIBLE: AtomicBool = AtomicBool::new(false);
static mut IS_DARK_MODE: bool = false;

// ---------------------------------------------------------------------------
// Dark mode detection
// ---------------------------------------------------------------------------

/// Reads the Windows system theme from the registry.
/// Returns true if the system is using dark mode.
fn detect_dark_mode() -> bool {
    unsafe {
        let subkey: Vec<u16> =
            "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
                .encode_utf16()
                .collect();
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            0,
            KEY_READ,
            &mut hkey,
        )
        .is_err()
        {
            return false;
        }

        let value_name: Vec<u16> = "AppsUseLightTheme\0".encode_utf16().collect();
        let mut data: u32 = 1;
        let mut data_size: u32 = std::mem::size_of::<u32>() as u32;
        let mut reg_type = REG_VALUE_TYPE::default();
        let result = RegQueryValueExW(
            hkey,
            PCWSTR(value_name.as_ptr()),
            None,
            Some(&mut reg_type),
            Some(&mut data as *mut u32 as *mut u8),
            Some(&mut data_size),
        );
        let _ = RegCloseKey(hkey);

        if result.is_err() {
            return false;
        }
        data == 0 // 0 = dark mode, 1 = light mode
    }
}

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
    // Detect dark mode before creating any window resources
    unsafe {
        IS_DARK_MODE = detect_dark_mode();
    }

    let class = class_name();
    let theme = get_theme();
    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW | CS_DROPSHADOW,
        lpfnWndProc: Some(gui_wndproc),
        hCursor: unsafe { LoadCursorW(None, IDC_ARROW).unwrap_or_default() },
        hbrBackground: unsafe {
            CreateSolidBrush(windows::Win32::Foundation::COLORREF(theme.bg))
        },
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
            GUI_EX_STYLE,
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

fn create_font(size: i32, weight: i32) -> HFONT {
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
    let theme = get_theme();
    if user_paused {
        return theme.yellow;
    }
    match cycle {
        scheduler::CYCLE_ACTIVE => theme.green,
        scheduler::CYCLE_INACTIVE | scheduler::CYCLE_LONG_PAUSE | scheduler::CYCLE_LUNCH => {
            theme.yellow
        }
        scheduler::CYCLE_OUTSIDE_HOURS => theme.red,
        _ => theme.label, // idle/unknown
    }
}

/// Returns the status text for the current state (English, used by tests).
#[cfg(test)]
fn status_text(cycle: u8, user_paused: bool) -> &'static str {
    if user_paused {
        return "User Active \u{2014} Paused";
    }
    scheduler::cycle_label(cycle)
}

/// Returns the localized status text for the current state.
fn status_text_localized(cycle: u8, user_paused: bool, s: &'static GuiStrings) -> &'static str {
    if user_paused {
        return s.user_active_paused;
    }
    cycle_label_localized(cycle, s)
}

/// Returns the localized label for the given cycle constant.
fn cycle_label_localized(cycle: u8, s: &'static GuiStrings) -> &'static str {
    match cycle {
        scheduler::CYCLE_IDLE => s.cycle_idle,
        scheduler::CYCLE_ACTIVE => s.cycle_active,
        scheduler::CYCLE_INACTIVE => s.cycle_inactive,
        scheduler::CYCLE_LONG_PAUSE => s.cycle_long_pause,
        scheduler::CYCLE_LUNCH => s.cycle_lunch,
        scheduler::CYCLE_OUTSIDE_HOURS => s.cycle_outside,
        _ => "—",
    }
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

    let theme = get_theme();
    let scale = get_dpi_scale();
    let pad = scaled(16, scale);
    let row_height = scaled(22, scale);

    // Fonts: title=bold, label=semibold, value=bold, footer=semibold
    let font_title = create_font(scaled(16, scale), 700);
    let font_label = create_font(scaled(13, scale), 600);
    let font_value = create_font(scaled(13, scale), 700);
    let font_small = create_font(scaled(11, scale), 600);

    unsafe {
        SetBkMode(hdc, TRANSPARENT);
    }

    // Get client rect and fill background
    let mut client = RECT::default();
    unsafe {
        let _ = GetClientRect(hwnd, &mut client);
    }
    fill_rect_color(
        hdc,
        client.left,
        client.top,
        client.right,
        client.bottom,
        theme.bg,
    );

    // Draw border
    unsafe {
        let border_brush =
            CreateSolidBrush(windows::Win32::Foundation::COLORREF(theme.border));
        FrameRect(hdc, &client, border_brush);
        let _ = DeleteObject(border_brush);
    }

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

    let s = gui_strings(&config.language);

    let mut y = pad;

    // --- Header: Title ---
    unsafe {
        SelectObject(hdc, HGDIOBJ(font_title.0));
    }
    draw_text(hdc, pad, y, s.title, theme.text);
    y += scaled(24, scale);

    // --- Status line with dot ---
    let dot_radius = scaled(5, scale);
    let dot_color = status_dot_color(cycle, user_paused);
    let status = status_text_localized(cycle, user_paused, s);
    draw_status_dot(hdc, pad, y + scaled(3, scale), dot_radius, dot_color);

    unsafe {
        SelectObject(hdc, HGDIOBJ(font_value.0));
    }
    draw_text(
        hdc,
        pad + dot_radius * 2 + scaled(8, scale),
        y,
        status,
        theme.text,
    );
    y += row_height + scaled(4, scale);

    // --- Separator ---
    fill_rect_color(hdc, pad, y, client.right - pad, y + 1, theme.separator);
    y += scaled(12, scale);

    // --- Info rows ---
    let label_x = pad;
    let value_x = pad + scaled(100, scale);

    // Helper closure: draw a label + value row
    macro_rules! row {
        ($label:expr, $value:expr, $color:expr) => {
            unsafe { SelectObject(hdc, HGDIOBJ(font_label.0)); }
            draw_text(hdc, label_x, y, $label, theme.label);
            unsafe { SelectObject(hdc, HGDIOBJ(font_value.0)); }
            draw_text(hdc, value_x, y, $value, $color);
            y += row_height;
        };
    }

    // Uptime
    let uptime_str = format!("{:02}h {:02}m {:02}s", hours, minutes, seconds);
    row!(s.uptime, &uptime_str, theme.text);

    // Typing
    let (typing_str, typing_color) = if config.typing {
        (s.on, theme.green)
    } else {
        (s.off, theme.red)
    };
    row!(s.typing, typing_str, typing_color);

    // Mouse
    let (mouse_str, mouse_color) = if config.mouse {
        (s.on_silent, theme.green)
    } else {
        (s.off, theme.red)
    };
    row!(s.mouse, mouse_str, mouse_color);

    // Mouse mode
    let mode_str = match config.mouse_mode {
        MouseMode::Subtle => s.subtle,
        MouseMode::Wide => s.wide,
        MouseMode::Mixed => s.mixed,
    };
    row!(s.mouse_mode, mode_str, theme.text);

    // Schedule
    let sched_str = match config.schedule {
        Schedule::Always => s.always.to_string(),
        Schedule::Business => format!("{}-{}", config.schedule_start, config.schedule_end),
    };
    row!(s.schedule, &sched_str, theme.text);

    // Language
    let lang_str = match config.language {
        Language::PtBr => "pt-br",
        Language::En => "en",
    };
    row!(s.language_label, lang_str, theme.text);

    // Hotkey
    row!(s.hotkey, &config.hotkey, theme.text);

    // --- Separator ---
    y += scaled(4, scale);
    fill_rect_color(hdc, pad, y, client.right - pad, y + 1, theme.separator);
    y += scaled(12, scale);

    // Cycle
    let cycle_str = cycle_label_localized(cycle, s);
    row!(s.cycle, cycle_str, theme.text);

    // Keystrokes
    let ks = format_count(input::keystroke_count());
    row!(s.keystrokes, &ks, theme.text);

    // Mouse moves
    let mm = format_count(input::mouse_move_count());
    row!(s.mouse_moves, &mm, theme.text);

    // User status
    let (user_str, user_color) = if user_paused {
        (s.active_paused, theme.yellow)
    } else {
        (s.idle, theme.green)
    };
    row!(s.user, user_str, user_color);

    // Auto-shutdown status
    let (shutdown_str, shutdown_color) = if scheduler::auto_shutdown_enabled() {
        (s.on, theme.green)
    } else {
        (s.off, theme.label)
    };
    row!(s.auto_shutdown, shutdown_str, shutdown_color);

    y += scaled(8, scale);

    // --- Footer ---
    fill_rect_color(hdc, pad, y, client.right - pad, y + 1, theme.separator);
    y += scaled(8, scale);

    unsafe {
        SelectObject(hdc, HGDIOBJ(font_small.0));
    }
    draw_text(
        hdc,
        pad,
        y,
        &format!(
            "{} {}  |  Ctrl+Shift+Q {}",
            config.hotkey, s.footer_hide, s.footer_quit
        ),
        theme.footer,
    );

    // Cleanup fonts
    unsafe {
        let _ = DeleteObject(HGDIOBJ(font_title.0));
        let _ = DeleteObject(HGDIOBJ(font_label.0));
        let _ = DeleteObject(HGDIOBJ(font_value.0));
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
        let theme = get_theme();
        assert_eq!(
            status_dot_color(scheduler::CYCLE_ACTIVE, false),
            theme.green
        );
    }

    #[test]
    fn test_status_dot_color_inactive() {
        let theme = get_theme();
        assert_eq!(
            status_dot_color(scheduler::CYCLE_INACTIVE, false),
            theme.yellow
        );
    }

    #[test]
    fn test_status_dot_color_outside_hours() {
        let theme = get_theme();
        assert_eq!(
            status_dot_color(scheduler::CYCLE_OUTSIDE_HOURS, false),
            theme.red
        );
    }

    #[test]
    fn test_status_dot_color_user_paused_overrides() {
        let theme = get_theme();
        // User paused should always be yellow regardless of cycle
        assert_eq!(
            status_dot_color(scheduler::CYCLE_ACTIVE, true),
            theme.yellow
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

    #[test]
    fn test_detect_dark_mode_returns_bool() {
        // Just verify it doesn't panic — the result depends on system settings
        let _ = detect_dark_mode();
    }

    #[test]
    fn test_light_theme_has_distinct_colors() {
        assert_ne!(LIGHT_THEME.bg, LIGHT_THEME.text);
        assert_ne!(LIGHT_THEME.bg, LIGHT_THEME.border);
        assert_ne!(LIGHT_THEME.label, LIGHT_THEME.text);
    }

    #[test]
    fn test_dark_theme_has_distinct_colors() {
        assert_ne!(DARK_THEME.bg, DARK_THEME.text);
        assert_ne!(DARK_THEME.bg, DARK_THEME.border);
        assert_ne!(DARK_THEME.label, DARK_THEME.text);
    }

    #[test]
    fn test_themes_have_different_backgrounds() {
        assert_ne!(LIGHT_THEME.bg, DARK_THEME.bg);
    }

    #[test]
    fn test_gui_strings_en() {
        let s = gui_strings(&Language::En);
        assert_eq!(s.typing, "Typing");
        assert_eq!(s.on, "ON");
        assert_eq!(s.off, "OFF");
        assert_eq!(s.cycle_active, "Active");
    }

    #[test]
    fn test_gui_strings_pt_br() {
        let s = gui_strings(&Language::PtBr);
        assert_eq!(s.on, "SIM");
        assert_eq!(s.off, "N\u{00c3}O");
        assert_eq!(s.cycle_active, "Ativo");
        assert_eq!(s.always, "Sempre");
    }

    #[test]
    fn test_cycle_label_localized_pt() {
        let s = gui_strings(&Language::PtBr);
        assert_eq!(cycle_label_localized(scheduler::CYCLE_ACTIVE, s), "Ativo");
        assert_eq!(cycle_label_localized(scheduler::CYCLE_LUNCH, s), "Almo\u{00e7}o");
        assert_eq!(cycle_label_localized(scheduler::CYCLE_OUTSIDE_HOURS, s), "Fora do Hor\u{00e1}rio");
    }

    #[test]
    fn test_cycle_label_localized_en() {
        let s = gui_strings(&Language::En);
        assert_eq!(cycle_label_localized(scheduler::CYCLE_ACTIVE, s), "Active");
        assert_eq!(cycle_label_localized(scheduler::CYCLE_LUNCH, s), "Lunch");
    }

    #[test]
    fn test_status_text_localized_paused_pt() {
        let s = gui_strings(&Language::PtBr);
        assert_eq!(
            status_text_localized(scheduler::CYCLE_ACTIVE, true, s),
            "Usu\u{00e1}rio Ativo \u{2014} Pausado"
        );
    }

    #[test]
    fn test_gui_strings_auto_shutdown_en() {
        let s = gui_strings(&Language::En);
        assert_eq!(s.auto_shutdown, "Auto-Shutdown");
    }

    #[test]
    fn test_gui_strings_auto_shutdown_pt() {
        let s = gui_strings(&Language::PtBr);
        assert_eq!(s.auto_shutdown, "Auto Desligar");
    }

    #[test]
    fn test_gui_window_hidden_from_taskbar() {
        // WS_EX_TOOLWINDOW must be set to hide from taskbar and Alt+Tab
        assert_ne!(
            GUI_EX_STYLE.0 & WS_EX_TOOLWINDOW.0,
            0,
            "GUI window must have WS_EX_TOOLWINDOW to stay off the taskbar"
        );
        // WS_EX_APPWINDOW must NOT be set (it forces taskbar appearance)
        const WS_EX_APPWINDOW: u32 = 0x00040000;
        assert_eq!(
            GUI_EX_STYLE.0 & WS_EX_APPWINDOW,
            0,
            "GUI window must NOT have WS_EX_APPWINDOW"
        );
    }

    #[test]
    fn test_status_text_localized_active_en() {
        let s = gui_strings(&Language::En);
        assert_eq!(
            status_text_localized(scheduler::CYCLE_ACTIVE, false, s),
            "Active"
        );
    }
}
