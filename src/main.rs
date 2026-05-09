mod app;
mod grid;
mod input;
mod theme;

use eframe::egui;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::io::Read as _;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use app::TerminalApp;
use grid::TerminalGrid;

fn main() {
    let rows: u16 = 24;
    let cols: u16 = 80;

    let grid = Arc::new(Mutex::new(TerminalGrid::new(rows as usize, cols as usize)));
    let running = Arc::new(AtomicBool::new(true));

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("failed to open pty");

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".into());
    let _child = pair
        .slave
        .spawn_command(CommandBuilder::new(shell))
        .expect("failed to spawn shell");
    drop(pair.slave);

    let pty_writer: Arc<Mutex<Box<dyn std::io::Write + Send>>> = Arc::new(Mutex::new(
        pair.master
            .take_writer()
            .expect("failed to get pty writer"),
    ));
    let mut pty_reader = pair
        .master
        .try_clone_reader()
        .expect("failed to get pty reader");
    drop(pair.master);

    let grid_r = grid.clone();
    let running_r = running.clone();
    std::thread::spawn(move || {
        let mut parser = vte::Parser::new();
        let mut buf = [0u8; 4096];
        loop {
            match pty_reader.read(&mut buf) {
                Ok(0) | Err(_) => {
                    running_r.store(false, Ordering::Relaxed);
                    break;
                }
                Ok(n) => {
                    let mut g = grid_r.lock().unwrap();
                    for &b in &buf[..n] {
                        parser.advance(&mut *g, b);
                    }
                }
            }
        }
    });

    let icon = image::load_from_memory(include_bytes!("../rustty.png"))
        .expect("failed to load icon")
        .to_rgba8();
    let (w, h) = icon.dimensions();
    let icon_data = egui::IconData {
        rgba: icon.into_raw(),
        width: w,
        height: h,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([700.0, 460.0])
            .with_title("Rustty")
            .with_icon(icon_data),
        ..Default::default()
    };

    eframe::run_native(
        "Rustty",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(TerminalApp {
                grid,
                pty_writer,
                running,
            }))
        }),
    )
    .expect("failed to start eframe");
}
