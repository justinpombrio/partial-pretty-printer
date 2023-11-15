//! Text styles like color and boldness, for mediums that support it.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    pub color: Color,
    pub bold: bool,
    pub underlined: bool,
    /// Swap the foreground and background
    pub reversed: bool,
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
    pub fn colored(color: Color) -> Style {
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
