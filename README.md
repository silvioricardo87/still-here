# Still Here

[![Build & Release](https://github.com/silvioricardo87/still-here/actions/workflows/build.yml/badge.svg)](https://github.com/silvioricardo87/still-here/actions/workflows/build.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-stable-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/Platform-Windows%2011%2B-blue.svg)](https://www.microsoft.com/windows)
[![GitHub release](https://img.shields.io/github/v/release/silvioricardo87/still-here)](https://github.com/silvioricardo87/still-here/releases/latest)

A stealth Windows application that simulates human computer activity — typing, mouse movement, and system interaction — to prevent screen lock and keep presence indicators active.

Ships as `still-here.exe` — rename the binary (e.g. to `WinServiceHost.exe`, `Notepad.exe`, etc.) if you want it to blend in with system processes. Runs as a local user with no admin privileges required.

## Features

- **Native GUI overlay** — modern light-styled popup window (Segoe UI, rounded corners, drop shadow) positioned in the bottom-right corner, always on top, DPI-aware
- **Realistic typing** from 180+ embedded phrases (pt-br and en) covering emails, meeting notes, project updates, code snippets, and more
- **Human-like typos** (~8% rate) with immediate correction
- **Natural mouse movement** — subtle jitter, wide Bezier curves, or mixed mode
- **User idle detection** — automatically pauses when you're actively using the computer, resumes when you step away
- **Activity cycles** — alternates between active typing, idle periods, and long pauses throughout the day
- **Business hours scheduling** — Mon-Fri 9:00-18:00 by default, with configurable work hours, active days, and lunch break
- **Sleep prevention** — continuous `SetThreadExecutionState` keep-alive every 30 seconds
- **Global hotkey** (Ctrl+Shift+F9) to toggle GUI visibility, with automatic fallback chain
- **Persistent config** — save settings to `%TEMP%\sth.dat` and reuse across sessions
- **Live statistics** — session counters for keystrokes and mouse moves, current cycle state, user activity status

## Requirements

- Windows 11+ (or Windows 10 with recent updates)
- Rust toolchain (stable)

## Build

```bash
cargo build --release
```

The optimized binary is at `target/release/still-here.exe` (stripped, LTO-enabled, size-optimized).

## Usage

```bash
# Run hidden (default) — press Ctrl+Shift+F9 to toggle GUI, Q or Ctrl+C to quit
still-here.exe

# Run with GUI visible (useful for development)
still-here.exe --visible

# Mouse only, no typing
still-here.exe --no-typing

# Active 24/7 instead of business hours only
still-here.exe --schedule always

# English phrases, subtle mouse, custom hours
still-here.exe --language en --mouse-mode subtle --schedule-start 08:00 --schedule-end 17:00

# Save current settings for next run
still-here.exe --save-config
```

### CLI Options

| Flag | Description | Default |
|------|-------------|---------|
| `--visible` | Show GUI overlay window | hidden |
| `--no-typing` | Disable keyboard simulation | typing on |
| `--no-mouse` | Disable mouse simulation | mouse on |
| `--mouse-mode <MODE>` | `subtle`, `wide`, or `mixed` | `mixed` |
| `--schedule <MODE>` | `business` or `always` | `business` |
| `--schedule-start <HH:MM>` | Work start time | `09:00` |
| `--schedule-end <HH:MM>` | Work end time | `18:00` |
| `--schedule-days <DAYS>` | Active days (comma-separated) | `mon,tue,wed,thu,fri` |
| `--lunch-start <HH:MM>` | Lunch break start | `13:00` |
| `--lunch-duration <MIN>` | Lunch break length in minutes | `60` |
| `--language <LANG>` | `pt-br` or `en` | `pt-br` |
| `--hotkey <HOTKEY>` | Global toggle hotkey | `Ctrl+Shift+F9` |
| `--save-config` | Persist settings to disk and exit | -- |

## GUI Overlay

The status window is a native Win32 popup (no external GUI frameworks). The header shows the title and current app version; the body displays:

- **Status** with colored indicator dot (green = active, yellow = inactive/paused, red = outside hours)
- **Uptime**, **Typing**, **Mouse**, **Mouse Mode**, **Schedule**, **Language**, **Hotkey**
- **Cycle** — current activity state (Active, Inactive, Long Pause, Lunch, Outside Hours)
- **Keystrokes** and **Mouse Moves** — session counters
- **User** — idle detection status (Idle / Active with simulation paused)

The window is always on top, hidden from Alt+Tab and the taskbar, and refreshes every 2 seconds. Toggle with the global hotkey or start with `--visible`.

## Architecture

Single-process, multi-threaded application coordinating shutdown via a `static AtomicBool`:

| Thread | Role |
|--------|------|
| Main | Win32 message pump — hotkey events, GUI `WM_PAINT`/`WM_TIMER`, and `WM_QUIT` |
| Activity | Scheduler loop — typing/mouse cycles, business hours |
| Console input | Blocks on `ReadConsoleInput`, posts quit on Q press |
| Keep-alive | `SetThreadExecutionState` every 30s |

### Modules

| Module | Responsibility |
|--------|---------------|
| `main.rs` | Entry point, thread orchestration, DPI awareness, Ctrl+C handler |
| `gui.rs` | Win32 overlay window — registration, creation, GDI painting, show/hide/toggle |
| `config.rs` | CLI parsing (clap), binary config load/save |
| `dictionary.rs` | Embedded phrase libraries, random selection |
| `input.rs` | Keyboard simulation (SendInput/Unicode), mouse movement, idle detection, session counters |
| `stealth.rs` | Console hide, hotkey registration, keep-alive, console input handling |
| `scheduler.rs` | Activity cycles, business hours, lunch detection, cycle state broadcasting |

### Key Design Decisions

- **Native Win32 GUI** — uses GDI painting with the existing `windows` crate, zero additional dependencies
- **DPI-aware** — scales window size and fonts based on monitor DPI via `SetProcessDpiAwarenessContext`
- **Unicode input** — uses `SendInput` with `KEYEVENTF_UNICODE` and `wScan` (not `wVk`), so typing works regardless of active keyboard layout
- **No chrono** — time handling uses `std::time::SystemTime` with Win32 `GetLocalTime` for correct timezone support
- **Static shutdown flag** — `static AtomicBool` (not `Arc`) because the Ctrl+C handler is a C callback that cannot capture state
- **Atomic shared state** — cycle state and session counters use `AtomicU8`/`AtomicU32` for lock-free cross-thread communication
- **Hotkey fallback** — primary hotkey -> Ctrl+Shift+F10 -> Ctrl+Shift+F11 -> fatal error

## Testing

```bash
cargo test
```

100 unit tests cover all pure logic: config parsing, CLI merging, schedule evaluation, hotkey parsing, timing ranges, dictionary selection, GUI state mapping, format helpers, and session counters. Win32 API calls are verified manually since they require a live Windows session.

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.
