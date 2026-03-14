# Still Here

A stealth Windows application that simulates human computer activity — typing, mouse movement, and system interaction — to prevent screen lock and keep presence indicators active.

Compiles to `WinServiceHost.exe` to blend with system processes. Runs as a local user with no admin privileges required.

## Features

- **Realistic typing** from 180+ embedded phrases (pt-br and en) covering emails, meeting notes, project updates, code snippets, and more
- **Human-like typos** (~8% rate) with immediate correction
- **Natural mouse movement** — subtle jitter, wide Bezier curves, or mixed mode
- **User idle detection** — automatically pauses when you're actively using the computer, resumes when you step away
- **Activity cycles** — alternates between active typing, idle periods, and long pauses throughout the day
- **Business hours scheduling** — Mon-Fri 9:00-18:00 by default, with configurable work hours, active days, and lunch break
- **Sleep prevention** — continuous `SetThreadExecutionState` keep-alive every 30 seconds
- **Global hotkey** (Ctrl+Shift+F9) to toggle console visibility, with automatic fallback chain
- **Persistent config** — save settings to `%TEMP%\wsh.dat` and reuse across sessions

## Requirements

- Windows 11+ (or Windows 10 with recent updates)
- Rust toolchain (stable)

## Build

```bash
cargo build --release
```

The optimized binary is at `target/release/WinServiceHost.exe` (stripped, LTO-enabled, size-optimized).

## Usage

```bash
# Run hidden (default) — press Ctrl+Shift+F9 to toggle console, Q or Ctrl+C to quit
WinServiceHost.exe

# Run with console visible (useful for development)
WinServiceHost.exe --visible

# Mouse only, no typing
WinServiceHost.exe --no-typing

# Active 24/7 instead of business hours only
WinServiceHost.exe --schedule always

# English phrases, subtle mouse, custom hours
WinServiceHost.exe --language en --mouse-mode subtle --schedule-start 08:00 --schedule-end 17:00

# Save current settings for next run
WinServiceHost.exe --save-config
```

### CLI Options

| Flag | Description | Default |
|------|-------------|---------|
| `--visible` | Show console window | hidden |
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

## Architecture

Single-process, five threads coordinating shutdown via a `static AtomicBool`:

| Thread | Role |
|--------|------|
| Main | Win32 message pump — hotkey events and `WM_QUIT` |
| Activity | Scheduler loop — typing/mouse cycles, business hours |
| Console input | Blocks on `ReadConsoleInput`, posts quit on Q press |
| Keep-alive | `SetThreadExecutionState` every 30s |
| Display | Refreshes console status panel every 2s (when visible) |

### Modules

| Module | Responsibility |
|--------|---------------|
| `main.rs` | Entry point, thread orchestration, Ctrl+C handler |
| `config.rs` | CLI parsing (clap), binary config load/save |
| `dictionary.rs` | Embedded phrase libraries, random selection |
| `input.rs` | Keyboard simulation (SendInput/Unicode), mouse movement, typo logic, idle detection |
| `stealth.rs` | Window hide/show, hotkey registration, keep-alive, console status display |
| `scheduler.rs` | Activity cycles, business hours, lunch detection |

### Key Design Decisions

- **Unicode input** — uses `SendInput` with `KEYEVENTF_UNICODE` and `wScan` (not `wVk`), so typing works regardless of active keyboard layout
- **No chrono** — time handling uses `std::time::SystemTime` with Win32 `GetLocalTime` for correct timezone support
- **Static shutdown flag** — `static AtomicBool` (not `Arc`) because the Ctrl+C handler is a C callback that cannot capture state
- **Hotkey fallback** — primary hotkey -> Ctrl+Shift+F10 -> Ctrl+Shift+F11 -> fatal error

## Testing

```bash
cargo test
```

Unit tests cover all pure logic: config parsing, schedule evaluation, hotkey parsing, timing ranges, dictionary selection, and Bezier curves. Win32 API calls are verified manually since they require a live Windows session.

## License

Private use only.
