use eframe::egui;

pub fn ctrl_key_byte(key: &egui::Key) -> Option<u8> {
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

pub fn special_key_bytes(key: &egui::Key) -> Option<&'static [u8]> {
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
