use eframe::egui;
use std::io::Write as _;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use crate::grid::TerminalGrid;
use crate::input::{ctrl_key_byte, special_key_bytes};
use crate::theme::{DEFAULT_BG, FONT_SIZE, PADDING};

pub struct TerminalApp {
    pub grid: Arc<Mutex<TerminalGrid>>,
    pub pty_writer: Arc<Mutex<Box<dyn std::io::Write + Send>>>,
    pub running: Arc<AtomicBool>,
}

impl eframe::App for TerminalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.running.load(Ordering::Relaxed) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        ctx.request_repaint_after(std::time::Duration::from_millis(16));

        let mut bytes_to_send: Vec<u8> = Vec::new();
        ctx.input(|i| {
            for event in &i.events {
                match event {
                    egui::Event::Text(text) => {
                        bytes_to_send.extend_from_slice(text.as_bytes());
                    }
                    egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } => {
                        if modifiers.ctrl {
                            if let Some(b) = ctrl_key_byte(key) {
                                bytes_to_send.push(b);
                            }
                        } else if !modifiers.alt && !modifiers.command {
                            if let Some(seq) = special_key_bytes(key) {
                                bytes_to_send.extend_from_slice(seq);
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        if !bytes_to_send.is_empty() {
            if let Ok(mut w) = self.pty_writer.lock() {
                let _ = w.write_all(&bytes_to_send);
            }
        }

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(DEFAULT_BG)
                    .inner_margin(PADDING),
            )
            .show(ctx, |ui| {
                let grid = self.grid.lock().unwrap();
                let font_id = egui::FontId::monospace(FONT_SIZE);

                let char_size = ui.fonts(|f| {
                    f.layout_no_wrap("M".into(), font_id.clone(), egui::Color32::WHITE)
                        .size()
                });
                let cw = char_size.x;
                let lh = char_size.y;

                let origin = ui.min_rect().min;
                let painter = ui.painter();

                for row in 0..grid.rows {
                    let y = origin.y + row as f32 * lh;
                    let mut col = 0;

                    while col < grid.cols {
                        let cell = grid.cells[row][col];
                        let fg = cell.fg;
                        let bg = cell.bg;
                        let start = col;
                        let mut text = String::new();

                        while col < grid.cols
                            && grid.cells[row][col].fg == fg
                            && grid.cells[row][col].bg == bg
                        {
                            text.push(grid.cells[row][col].c);
                            col += 1;
                        }

                        let x = origin.x + start as f32 * cw;
                        let w = (col - start) as f32 * cw;

                        if bg != DEFAULT_BG {
                            painter.rect_filled(
                                egui::Rect::from_min_size(
                                    egui::pos2(x, y),
                                    egui::vec2(w, lh),
                                ),
                                0.0,
                                bg,
                            );
                        }

                        let trimmed = text.trim_end();
                        if !trimmed.is_empty() {
                            painter.text(
                                egui::pos2(x, y),
                                egui::Align2::LEFT_TOP,
                                trimmed,
                                font_id.clone(),
                                fg,
                            );
                        }
                    }
                }

                if grid.cursor_visible {
                    let cx = origin.x + grid.cursor_col as f32 * cw;
                    let cy = origin.y + grid.cursor_row as f32 * lh;
                    painter.rect_filled(
                        egui::Rect::from_min_size(
                            egui::pos2(cx, cy),
                            egui::vec2(cw, lh),
                        ),
                        0.0,
                        egui::Color32::from_rgba_premultiplied(200, 200, 200, 128),
                    );
                }
            });
    }
}
