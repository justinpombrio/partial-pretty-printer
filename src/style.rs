//! Text styles like color and boldness, for mediums that support it.
//!
//! For exposition, the descriptions here are written assuming that the background is black and the
//! foreground is white, but that is not required.

use self::Color::*;

/// Just the components that can be chosen by Notations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    pub color: Color,
    pub bold: bool,
    pub underlined: bool,
    pub reversed: bool,
}

/// The overall style to render text.
/// If `reversed`, swap the foreground and background.
// TODO: How widespread is terminal support for underlining?
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
/// Only 0, 1, and 2+ are distinguished at the moment.
/// 0 is lightest (most highlighted), and 2+ is black (least highlighted).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shade(pub u8);

impl Style {
    /// Typically, ordinary white on black.
    pub fn plain() -> Self {
        Style {
            color: Color::Base05,
            bold: false,
            underlined: false,
            reversed: false,
        }
    }

    /// Ordinary colored text.
    pub fn color(color: Color) -> Style {
        Style {
            color,
            bold: false,
            underlined: false,
            reversed: false,
        }
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
            bold: style.bold,
            underlined: style.underlined,
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
        Shade(u8::max_value())
    }

    /// Cursor highlight color; typically dark gray.
    pub fn highlight() -> Shade {
        Shade(0)
    }
}

impl Default for Shade {
    fn default() -> Self {
        Shade::background()
    }
}

impl From<Shade> for Color {
    fn from(shade: Shade) -> Color {
        match shade {
            Shade(0) => Base03,
            Shade(1) => Base02,
            Shade(2) => Base01,
            _ => Base00,
        }
    }
}
