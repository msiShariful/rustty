use eframe::egui;

use crate::theme::{DEFAULT_BG, DEFAULT_FG, color_256};

#[derive(Clone, Copy, PartialEq)]
pub struct Cell {
    pub c: char,
    pub fg: egui::Color32,
    pub bg: egui::Color32,
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

pub struct TerminalGrid {
    pub cells: Vec<Vec<Cell>>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub rows: usize,
    pub cols: usize,
    pub cursor_visible: bool,
    fg: egui::Color32,
    bg: egui::Color32,
    scroll_top: usize,
    scroll_bottom: usize,
    saved_cursor: (usize, usize),
}

impl TerminalGrid {
    pub fn new(rows: usize, cols: usize) -> Self {
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

    fn apply_sgr(&mut self, ps: &[u16]) {
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
            'm' => self.apply_sgr(&ps),
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
