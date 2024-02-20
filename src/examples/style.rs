#[derive(Debug, Clone, Copy, Default)]
pub struct BasicStyle {
    pub color: Color,
    pub bold: bool,
}

impl BasicStyle {
    pub fn new() -> BasicStyle {
        BasicStyle {
            color: Color::White,
            bold: false,
        }
    }

    pub fn color(self, color: Color) -> Self {
        BasicStyle {
            color,
            bold: self.bold,
        }
    }

    pub fn bold(self) -> Self {
        BasicStyle {
            color: self.color,
            bold: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Color {
    #[default]
    White,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
}
