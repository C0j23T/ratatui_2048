use ratatui::prelude::Color;

pub fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

pub fn color_setter(value: u16) -> Color {
    match value {
        0 => rgb(44, 58, 71),
        2 => rgb(238, 228, 218),
        4 => rgb(237, 224, 200),
        8 => rgb(242, 177, 121),
        16 => rgb(245, 149, 99),
        32 => rgb(246, 124, 96),
        64 => rgb(246, 94, 59),
        128 => rgb(237, 207, 114),
        256 => rgb(237, 204, 97),
        512 => rgb(237, 200, 80),
        1024 => rgb(237, 197, 63),
        _ => rgb(237, 194, 46)
    }
}
