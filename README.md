# my-terminal

A minimal terminal emulator written in Rust.

Uses a PTY (pseudo-terminal) to spawn a shell and forwards keyboard input/output between the host terminal and the child shell process.

## Dependencies

- **portable-pty** — cross-platform PTY creation and management
- **crossterm** — raw mode, keyboard events, alternate screen
- **vte** — terminal escape sequence parsing (planned)
- **tokio** — async runtime

## Build & Run

```sh
cargo build
cargo run
```

Press `Ctrl+D` or `Ctrl+C` to exit.
