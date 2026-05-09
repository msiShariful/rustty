use eframe::egui;
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::io::{Read as _, Write as _};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

const FONT_SIZE: f32 = 15.0;
const PADDING: f32 = 6.0;
const DEFAULT_FG: egui::Color32 = egui::Color32::from_rgb(204, 204, 204);
const DEFAULT_BG: egui::Color32 = egui::Color32::from_rgb(30, 30, 30);

fn color_256(idx: u16) -> egui::Color32 {
    let standard: [(u8, u8, u8); 16] = [
        (0, 0, 0),
        (205, 49, 49),
        (13, 188, 121),
        (229, 229, 16),
        (36, 114, 200),
        (188, 63, 188),
        (17, 168, 205),
        (204, 204, 204),
        (118, 118, 118),
        (241, 76, 76),
        (35, 209, 139),
        (245, 245, 67),
        (59, 142, 234),
        (214, 112, 214),
        (41, 184, 219),
        (229, 229, 229),
    ];
    if idx < 16 {
        let (r, g, b) = standard[idx as usize];
        egui::Color32::from_rgb(r, g, b)
    } else if idx < 232 {
        let i = idx - 16;
        let r = (i / 36) as u8;
        let g = ((i % 36) / 6) as u8;
        let b = (i % 6) as u8;
        let v = |c: u8| if c == 0 { 0 } else { 55 + 40 * c };
        egui::Color32::from_rgb(v(r), v(g), v(b))
    } else {
        let v = 8 + 10 * (idx - 232) as u8;
        egui::Color32::from_rgb(v, v, v)
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Cell {
    c: char,
    fg: egui::Color32,
    bg: egui::Color32,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            fg: DEFAULT_FG,
            bg: DEFAULT_BG,
        }
    }
}

struct TerminalGrid {
    cells: Vec<Vec<Cell>>,
    cursor_row: usize,
    cursor_col: usize,
    rows: usize,
    cols: usize,
    fg: egui::Color32,
    bg: egui::Color32,
    scroll_top: usize,
    scroll_bottom: usize,
    saved_cursor: (usize, usize),
    cursor_visible: bool,
}

impl TerminalGrid {
    fn new(rows: usize, cols: usize) -> Self {
        Self {
            cells: vec![vec![Cell::default(); cols]; rows],
            cursor_row: 0,
            cursor_col: 0,
            rows,
            cols,
            fg: DEFAULT_FG,
            bg: DEFAULT_BG,
            scroll_top: 0,
            scroll_bottom: rows - 1,
            saved_cursor: (0, 0),
            cursor_visible: true,
        }
    }

    fn scroll_up(&mut self) {
        for r in self.scroll_top..self.scroll_bottom {
            self.cells[r] = self.cells[r + 1].clone();
        }
        self.cells[self.scroll_bottom] = vec![Cell::default(); self.cols];
    }

    fn scroll_down(&mut self) {
        for r in (self.scroll_top + 1..=self.scroll_bottom).rev() {
            self.cells[r] = self.cells[r - 1].clone();
        }
        self.cells[self.scroll_top] = vec![Cell::default(); self.cols];
    }

    fn put_char(&mut self, c: char) {
        if self.cursor_col >= self.cols {
            self.cursor_col = 0;
            self.cursor_row += 1;
        }
        if self.cursor_row > self.scroll_bottom {
            self.scroll_up();
            self.cursor_row = self.scroll_bottom;
        }
        if self.cursor_row < self.rows && self.cursor_col < self.cols {
            self.cells[self.cursor_row][self.cursor_col] = Cell {
                c,
                fg: self.fg,
                bg: self.bg,
            };
            self.cursor_col += 1;
        }
    }
}

impl vte::Perform for TerminalGrid {
    fn print(&mut self, c: char) {
        self.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            0x08 => {
                self.cursor_col = self.cursor_col.saturating_sub(1);
            }
            0x09 => {
                self.cursor_col = ((self.cursor_col / 8) + 1) * 8;
                if self.cursor_col >= self.cols {
                    self.cursor_col = self.cols - 1;
                }
            }
            0x0A | 0x0B | 0x0C => {
                self.cursor_row += 1;
                if self.cursor_row > self.scroll_bottom {
                    self.scroll_up();
                    self.cursor_row = self.scroll_bottom;
                }
            }
            0x0D => {
                self.cursor_col = 0;
            }
            _ => {}
        }
    }

    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        if intermediates == [b'?'] {
            let ps: Vec<u16> = params.iter().map(|p| p[0]).collect();
            if action == 'h' || action == 'l' {
                let enable = action == 'h';
                for &p in &ps {
                    if p == 25 {
                        self.cursor_visible = enable;
                    }
                }
            }
            return;
        }
        if !intermediates.is_empty() {
            return;
        }

        let ps: Vec<u16> = params.iter().map(|p| p[0]).collect();

        match action {
            'A' => {
                let n = ps.first().copied().unwrap_or(1).max(1) as usize;
                self.cursor_row = self.cursor_row.saturating_sub(n);
            }
            'B' => {
                let n = ps.first().copied().unwrap_or(1).max(1) as usize;
                self.cursor_row = (self.cursor_row + n).min(self.rows - 1);
            }
            'C' => {
                let n = ps.first().copied().unwrap_or(1).max(1) as usize;
                self.cursor_col = (self.cursor_col + n).min(self.cols - 1);
            }
            'D' => {
                let n = ps.first().copied().unwrap_or(1).max(1) as usize;
                self.cursor_col = self.cursor_col.saturating_sub(n);
            }
            'H' | 'f' => {
                let row = ps.first().copied().unwrap_or(1).max(1) as usize - 1;
                let col = ps.get(1).copied().unwrap_or(1).max(1) as usize - 1;
                self.cursor_row = row.min(self.rows - 1);
                self.cursor_col = col.min(self.cols - 1);
            }
            'J' => {
                let mode = ps.first().copied().unwrap_or(0);
                match mode {
                    0 => {
                        for c in self.cursor_col..self.cols {
                            self.cells[self.cursor_row][c] = Cell::default();
                        }
                        for r in (self.cursor_row + 1)..self.rows {
                            self.cells[r] = vec![Cell::default(); self.cols];
                        }
                    }
                    1 => {
                        for r in 0..self.cursor_row {
                            self.cells[r] = vec![Cell::default(); self.cols];
                        }
                        for c in 0..=self.cursor_col.min(self.cols - 1) {
                            self.cells[self.cursor_row][c] = Cell::default();
                        }
                    }
                    2 | 3 => {
                        for r in 0..self.rows {
                            self.cells[r] = vec![Cell::default(); self.cols];
                        }
                    }
                    _ => {}
                }
            }
            'K' => {
                let mode = ps.first().copied().unwrap_or(0);
                match mode {
                    0 => {
                        for c in self.cursor_col..self.cols {
                            self.cells[self.cursor_row][c] = Cell::default();
                        }
                    }
                    1 => {
                        for c in 0..=self.cursor_col.min(self.cols - 1) {
                            self.cells[self.cursor_row][c] = Cell::default();
                        }
                    }
                    2 => {
                        self.cells[self.cursor_row] = vec![Cell::default(); self.cols];
                    }
                    _ => {}
                }
            }
            'm' => {
                if ps.is_empty() {
                    self.fg = DEFAULT_FG;
                    self.bg = DEFAULT_BG;
                    return;
                }
                let mut i = 0;
                while i < ps.len() {
                    match ps[i] {
                        0 => {
                            self.fg = DEFAULT_FG;
                            self.bg = DEFAULT_BG;
                        }
                        1 | 22 => {}
                        30 => self.fg = egui::Color32::from_rgb(0, 0, 0),
                        31 => self.fg = egui::Color32::from_rgb(205, 49, 49),
                        32 => self.fg = egui::Color32::from_rgb(13, 188, 121),
                        33 => self.fg = egui::Color32::from_rgb(229, 229, 16),
                        34 => self.fg = egui::Color32::from_rgb(36, 114, 200),
                        35 => self.fg = egui::Color32::from_rgb(188, 63, 188),
                        36 => self.fg = egui::Color32::from_rgb(17, 168, 205),
                        37 | 39 => self.fg = DEFAULT_FG,
                        40 => self.bg = egui::Color32::from_rgb(0, 0, 0),
                        41 => self.bg = egui::Color32::from_rgb(205, 49, 49),
                        42 => self.bg = egui::Color32::from_rgb(13, 188, 121),
                        43 => self.bg = egui::Color32::from_rgb(229, 229, 16),
                        44 => self.bg = egui::Color32::from_rgb(36, 114, 200),
                        45 => self.bg = egui::Color32::from_rgb(188, 63, 188),
                        46 => self.bg = egui::Color32::from_rgb(17, 168, 205),
                        47 => self.bg = egui::Color32::from_rgb(204, 204, 204),
                        49 => self.bg = DEFAULT_BG,
                        90 => self.fg = egui::Color32::from_rgb(118, 118, 118),
                        91 => self.fg = egui::Color32::from_rgb(241, 76, 76),
                        92 => self.fg = egui::Color32::from_rgb(35, 209, 139),
                        93 => self.fg = egui::Color32::from_rgb(245, 245, 67),
                        94 => self.fg = egui::Color32::from_rgb(59, 142, 234),
                        95 => self.fg = egui::Color32::from_rgb(214, 112, 214),
                        96 => self.fg = egui::Color32::from_rgb(41, 184, 219),
                        97 => self.fg = egui::Color32::from_rgb(229, 229, 229),
                        100 => self.bg = egui::Color32::from_rgb(118, 118, 118),
                        101 => self.bg = egui::Color32::from_rgb(241, 76, 76),
                        102 => self.bg = egui::Color32::from_rgb(35, 209, 139),
                        103 => self.bg = egui::Color32::from_rgb(245, 245, 67),
                        104 => self.bg = egui::Color32::from_rgb(59, 142, 234),
                        105 => self.bg = egui::Color32::from_rgb(214, 112, 214),
                        106 => self.bg = egui::Color32::from_rgb(41, 184, 219),
                        107 => self.bg = egui::Color32::from_rgb(229, 229, 229),
                        38 if ps.get(i + 1) == Some(&5) => {
                            if let Some(&idx) = ps.get(i + 2) {
                                self.fg = color_256(idx);
                                i += 2;
                            }
                        }
                        48 if ps.get(i + 1) == Some(&5) => {
                            if let Some(&idx) = ps.get(i + 2) {
                                self.bg = color_256(idx);
                                i += 2;
                            }
                        }
                        38 if ps.get(i + 1) == Some(&2) => {
                            if let (Some(&r), Some(&g), Some(&b)) =
                                (ps.get(i + 2), ps.get(i + 3), ps.get(i + 4))
                            {
                                self.fg = egui::Color32::from_rgb(r as u8, g as u8, b as u8);
                                i += 4;
                            }
                        }
                        48 if ps.get(i + 1) == Some(&2) => {
                            if let (Some(&r), Some(&g), Some(&b)) =
                                (ps.get(i + 2), ps.get(i + 3), ps.get(i + 4))
                            {
                                self.bg = egui::Color32::from_rgb(r as u8, g as u8, b as u8);
                                i += 4;
                            }
                        }
                        _ => {}
                    }
                    i += 1;
                }
            }
            'r' => {
                let top = ps.first().copied().unwrap_or(1).max(1) as usize - 1;
                let bot = ps
                    .get(1)
                    .copied()
                    .unwrap_or(self.rows as u16)
                    .max(1) as usize
                    - 1;
                self.scroll_top = top.min(self.rows - 1);
                self.scroll_bottom = bot.min(self.rows - 1);
                self.cursor_row = 0;
                self.cursor_col = 0;
            }
            'L' => {
                let n = ps.first().copied().unwrap_or(1).max(1) as usize;
                for _ in 0..n {
                    if self.cursor_row <= self.scroll_bottom {
                        self.cells.remove(self.scroll_bottom);
                        self.cells
                            .insert(self.cursor_row, vec![Cell::default(); self.cols]);
                    }
                }
            }
            'M' => {
                let n = ps.first().copied().unwrap_or(1).max(1) as usize;
                for _ in 0..n {
                    if self.cursor_row <= self.scroll_bottom {
                        self.cells.remove(self.cursor_row);
                        self.cells
                            .insert(self.scroll_bottom, vec![Cell::default(); self.cols]);
                    }
                }
            }
            'P' => {
                let n = ps.first().copied().unwrap_or(1).max(1) as usize;
                let row = &mut self.cells[self.cursor_row];
                for _ in 0..n {
                    if self.cursor_col < self.cols {
                        row.remove(self.cursor_col);
                        row.push(Cell::default());
                    }
                }
            }
            '@' => {
                let n = ps.first().copied().unwrap_or(1).max(1) as usize;
                let row = &mut self.cells[self.cursor_row];
                for _ in 0..n {
                    if self.cursor_col < self.cols {
                        row.insert(self.cursor_col, Cell::default());
                        row.truncate(self.cols);
                    }
                }
            }
            'd' => {
                let row = ps.first().copied().unwrap_or(1).max(1) as usize - 1;
                self.cursor_row = row.min(self.rows - 1);
            }
            'G' => {
                let col = ps.first().copied().unwrap_or(1).max(1) as usize - 1;
                self.cursor_col = col.min(self.cols - 1);
            }
            'X' => {
                let n = ps.first().copied().unwrap_or(1).max(1) as usize;
                for c in self.cursor_col..(self.cursor_col + n).min(self.cols) {
                    self.cells[self.cursor_row][c] = Cell::default();
                }
            }
            'S' => {
                let n = ps.first().copied().unwrap_or(1).max(1) as usize;
                for _ in 0..n {
                    self.scroll_up();
                }
            }
            'T' => {
                let n = ps.first().copied().unwrap_or(1).max(1) as usize;
                for _ in 0..n {
                    self.scroll_down();
                }
            }
            's' => {
                self.saved_cursor = (self.cursor_row, self.cursor_col);
            }
            'u' => {
                let (r, c) = self.saved_cursor;
                self.cursor_row = r;
                self.cursor_col = c;
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, byte: u8) {
        match byte {
            b'M' => {
                if self.cursor_row == self.scroll_top {
                    self.scroll_down();
                } else {
                    self.cursor_row = self.cursor_row.saturating_sub(1);
                }
            }
            b'7' => {
                self.saved_cursor = (self.cursor_row, self.cursor_col);
            }
            b'8' => {
                let (r, c) = self.saved_cursor;
                self.cursor_row = r;
                self.cursor_col = c;
            }
            _ => {}
        }
    }
}

fn ctrl_key_byte(key: &egui::Key) -> Option<u8> {
    let b = match key {
        egui::Key::A => 1,
        egui::Key::B => 2,
        egui::Key::C => 3,
        egui::Key::D => 4,
        egui::Key::E => 5,
        egui::Key::F => 6,
        egui::Key::G => 7,
        egui::Key::H => 8,
        egui::Key::I => 9,
        egui::Key::J => 10,
        egui::Key::K => 11,
        egui::Key::L => 12,
        egui::Key::M => 13,
        egui::Key::N => 14,
        egui::Key::O => 15,
        egui::Key::P => 16,
        egui::Key::Q => 17,
        egui::Key::R => 18,
        egui::Key::S => 19,
        egui::Key::T => 20,
        egui::Key::U => 21,
        egui::Key::V => 22,
        egui::Key::W => 23,
        egui::Key::X => 24,
        egui::Key::Y => 25,
        egui::Key::Z => 26,
        _ => return None,
    };
    Some(b)
}

fn special_key_bytes(key: &egui::Key) -> Option<&'static [u8]> {
    match key {
        egui::Key::Enter => Some(b"\r"),
        egui::Key::Backspace => Some(&[0x7f]),
        egui::Key::Tab => Some(b"\t"),
        egui::Key::Escape => Some(&[0x1b]),
        egui::Key::ArrowUp => Some(b"\x1b[A"),
        egui::Key::ArrowDown => Some(b"\x1b[B"),
        egui::Key::ArrowRight => Some(b"\x1b[C"),
        egui::Key::ArrowLeft => Some(b"\x1b[D"),
        egui::Key::Home => Some(b"\x1b[H"),
        egui::Key::End => Some(b"\x1b[F"),
        egui::Key::Delete => Some(b"\x1b[3~"),
        egui::Key::PageUp => Some(b"\x1b[5~"),
        egui::Key::PageDown => Some(b"\x1b[6~"),
        _ => None,
    }
}

struct TerminalApp {
    grid: Arc<Mutex<TerminalGrid>>,
    pty_writer: Arc<Mutex<Box<dyn std::io::Write + Send>>>,
    running: Arc<AtomicBool>,
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
    let _child = pair.slave.spawn_command(CommandBuilder::new(shell)).expect("failed to spawn shell");
    drop(pair.slave);

    let pty_writer: Arc<Mutex<Box<dyn std::io::Write + Send>>> =
        Arc::new(Mutex::new(pair.master.take_writer().expect("failed to get pty writer")));
    let mut pty_reader = pair.master.try_clone_reader().expect("failed to get pty reader");
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

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([700.0, 460.0])
            .with_title("Rustty"),
        ..Default::default()
    };

    eframe::run_native(
        "Rustty",
        options,
        Box::new(move |_cc| Ok(Box::new(TerminalApp {
            grid,
            pty_writer,
            running,
        }))),
    )
    .expect("failed to start eframe");
}
