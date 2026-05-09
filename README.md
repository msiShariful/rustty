# my-terminal

A GUI terminal emulator written in Rust. Opens its own native window with a shell session, full ANSI color support, and cursor rendering.

## Dependencies

- **eframe** — native GUI window and rendering via egui
- **portable-pty** — cross-platform PTY creation and management
- **vte** — ANSI/VT escape sequence parsing

## Features

- Native GUI window (not console-based)
- 80×24 terminal grid with block cursor
- ANSI color support: 16 standard, 256 extended, and 24-bit true color
- Escape sequence handling: cursor movement, erase, scroll, insert/delete, SGR styling
- Keyboard input: printable characters, special keys, and Ctrl+A–Z
- Spawns your default `$SHELL`

## Build & Run

```sh
cargo build
cargo run
```
