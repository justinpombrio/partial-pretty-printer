use crate::pretty_printing::Style;

#[derive(Debug, Clone, Copy, Default)]
pub struct BasicStyle {
    pub color: Color,
    pub bold: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Color {
    #[default]
    White,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
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

impl Style for BasicStyle {
    fn combine(outer_style: &BasicStyle, inner_style: &BasicStyle) -> BasicStyle {
        BasicStyle {
            color: inner_style.color,
            bold: outer_style.bold || inner_style.bold,
        }
    }
}

impl From<&'static str> for BasicStyle {
    fn from(label: &'static str) -> Self {
        use Color::*;

        match label {
            "white" => BasicStyle::new().color(White),
            "bold_white" => BasicStyle::new().color(White).bold(),
            "black" => BasicStyle::new().color(Black),
            "bold_black" => BasicStyle::new().color(Black).bold(),
            "red" => BasicStyle::new().color(Red),
            "bold_red" => BasicStyle::new().color(Red).bold(),
            "green" => BasicStyle::new().color(Green),
            "bold_green" => BasicStyle::new().color(Green).bold(),
            "yellow" => BasicStyle::new().color(Yellow),
            "bold_yellow" => BasicStyle::new().color(Yellow).bold(),
            "blue" => BasicStyle::new().color(Blue),
            "bold_blue" => BasicStyle::new().color(Blue).bold(),
            "magenta" => BasicStyle::new().color(Magenta),
            "bold_magenta" => BasicStyle::new().color(Magenta).bold(),
            "cyan" => BasicStyle::new().color(Cyan),
            "bold_cyan" => BasicStyle::new().color(Cyan).bold(),
            _ => BasicStyle::new(),
        }
    }
}
