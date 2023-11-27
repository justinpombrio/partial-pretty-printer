#[derive(Debug, Clone, Copy)]
pub struct BasicStyle {
    pub color: Color,
    pub bold: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    White,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
}
