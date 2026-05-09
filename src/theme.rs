use eframe::egui;

pub const FONT_SIZE: f32 = 15.0;
pub const PADDING: f32 = 6.0;
pub const DEFAULT_FG: egui::Color32 = egui::Color32::from_rgb(204, 204, 204);
pub const DEFAULT_BG: egui::Color32 = egui::Color32::from_rgb(30, 30, 30);

const STANDARD_COLORS: [(u8, u8, u8); 16] = [
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

pub fn color_256(idx: u16) -> egui::Color32 {
    if idx < 16 {
        let (r, g, b) = STANDARD_COLORS[idx as usize];
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
