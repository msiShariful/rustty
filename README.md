# rustty

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

## Project Structure

```
src/
├── main.rs    — Entry point: PTY setup, reader thread, eframe launch
├── app.rs     — TerminalApp and eframe rendering/input dispatch
├── grid.rs    — Cell, TerminalGrid, and VTE ANSI escape sequence handling
├── input.rs   — Keyboard-to-byte mapping (Ctrl keys, special keys)
└── theme.rs   — Color constants and 256-color palette
```

## Build & Run

```sh
cargo build
cargo run
```

## Build macOS .dmg

Requires [cargo-bundle](https://github.com/nickelc/cargo-bundle):

```sh
cargo install cargo-bundle
```

Then run the build script:

```sh
./build-dmg.sh
```

This will:

1. Compile a release binary
2. Create a `Rustty.app` bundle
3. Package it into `Rustty.dmg`

If [create-dmg](https://github.com/create-dmg/create-dmg) is installed (`brew install create-dmg`), the DMG will include a drag-to-Applications layout. Otherwise it falls back to `hdiutil`.

> **Note:** The DMG is built for your current architecture (Intel or Apple Silicon). It is not code-signed or notarized, so recipients may need to right-click → Open on first launch.
