//! Style choices, such as color, bolding, and underlining.

mod color_theme;

pub use color_theme::{ColorTheme, Rgb};

// Just the components that can be chosen by Notations.
/// Styling to apply to text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    pub color: Color,
    pub bold: bool,
    // TODO: How widespread is terminal support for underlining?
    pub underlined: bool,
    pub reversed: bool,
}

/// The overall style to render text to the terminal.
/// If `reversed`, swap the foreground and background.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShadedStyle {
    pub color: Color,
    pub bold: bool,
    pub underlined: bool,
    pub reversed: bool,
    pub shade: Shade,
}

/// The foreground color of some text (or if reversed the background color).
///
/// This uses the [Base16](http://chriskempson.com/projects/base16/) colortheme definitions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    /// Default Background
    Base00,
    /// Lighter Background (Used for status bars)
    Base01,
    /// Selection Background, was shade2
    Base02,
    /// Comments, Invisibles, Line Highlighting
    Base03,
    /// Dark Foreground (Used for status bars)
    Base04,
    /// Default Foreground, Caret, Delimiters, Operators
    Base05,
    /// Light Foreground (Not often used)
    Base06,
    /// Light Background (Not often used)
    Base07,
    /// Variables, XML Tags, Markup Link Text, Markup Lists, Diff Deleted
    Base08,
    /// Integers, Boolean, Constants, XML Attributes, Markup Link Url
    Base09,
    /// Classes, Markup Bold, Search Text Background
    Base0A,
    /// Strings, Inherited Class, Markup Code, Diff Inserted
    Base0B,
    /// Support, Regular Expressions, Escape Characters, Markup Quotes
    Base0C,
    /// Functions, Methods, Attribute IDs, Headings
    Base0D,
    /// Keywords, Storage, Selector, Markup Italic, Diff Changed
    Base0E,
    /// Deprecated, Opening/Closing Embedded Language Tags, e.g. <?php ?>
    Base0F,
}

/// How dark the background is, or if reversed how dark the foreground is.
///
/// Only 0, 1, and 2+ are distinguished (subject to change).
/// 0 is brightest (most highlighted), and 2+ is black (least highlighted).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shade(pub usize);

impl Style {
    /// Typically, ordinary white on black.
    pub fn plain() -> Style {
        Style {
            color: Color::Base05,
            bold: false,
            underlined: false,
            reversed: false,
        }
    }

    pub fn underlined(self) -> Style {
        Style {
            underlined: true,
            ..self
        }
    }

    pub fn bold(self) -> Style {
        Style { bold: true, ..self }
    }

    pub fn reversed(self) -> Style {
        Style {
            reversed: true,
            ..self
        }
    }

    pub fn color(self, color: Color) -> Style {
        Style { color, ..self }
    }
}

impl Default for Style {
    fn default() -> Self {
        Style::plain()
    }
}

impl ShadedStyle {
    pub fn new(style: Style, shade: Shade) -> Self {
        Self {
            color: style.color,
            underlined: style.underlined,
            bold: style.bold,
            reversed: style.reversed,
            shade,
        }
    }

    pub fn plain() -> Self {
        Self::new(Style::plain(), Shade::background())
    }
}

impl Shade {
    /// Typically pure black, the most ordinary shade.
    pub fn background() -> Shade {
        Shade(usize::max_value())
    }
}

impl Default for Shade {
    fn default() -> Self {
        Shade::background()
    }
}

impl From<Shade> for Color {
    fn from(shade: Shade) -> Color {
        use Color::*;
        match shade {
            Shade(0) => Base03,
            Shade(1) => Base02,
            Shade(2) => Base01,
            _ => Base00,
        }
    }
}
