//! Simple typings for ANSI escape sequences. Used for nicer terminal output.
//!
//! I created this because I want compile time strings without
//! heap- or stack allocations.
//!
//! Help:
//! - <https://de.wikipedia.org/wiki/ANSI-Escapesequenz>
//! - <https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797>

use core::cell::Cell;
use core::fmt::{
    Display,
    Formatter,
};

/// Constructs the ANSI sequence for colors from the
/// fg/bg property and from the actual color.
macro_rules! ansi_colored {
    (fg, $color: tt) => {
        ansi_escape!(concat!("3", ansi_color_to_id!($color)))
    };
    (bg, $color: tt) => {
        ansi_escape!(concat!("4", ansi_color_to_id!($color)))
    };
}

/// Wraps a ANSI parameter in the ANSI escape sequence delimiters.
macro_rules! ansi_escape {
    ($code: expr) => {
        concat!("\u{1b}[", $code, "m")
    };
}

/// Returns color as token (compile time string).
macro_rules! ansi_color_to_id {
    (black) => {
        0
    };
    (red) => {
        1
    };
    (green) => {
        2
    };
    (yellow) => {
        3
    };
    (blue) => {
        4
    };
    (magenta) => {
        5
    };
    (cyan) => {
        6
    };
    (white) => {
        7
    };
    (default) => {
        8
    };
}

/// Returns text style as token (compile time string).
macro_rules! ansi_text_style_to_id {
    (normal) => {
        0
    };
    (bold) => {
        1
    };
    (dimmed) => {
        2
    };
    (italic) => {
        3
    };
    (underline) => {
        4
    };
    (blink) => {
        5
    };
}

const BG_BLACK: &'static str = ansi_colored!(bg, black);
const BG_RED: &'static str = ansi_colored!(bg, red);
const BG_GREEN: &'static str = ansi_colored!(bg, green);
const BG_YELLOW: &'static str = ansi_colored!(bg, yellow);
const BG_BLUE: &'static str = ansi_colored!(bg, blue);
const BG_MAGENTA: &'static str = ansi_colored!(bg, magenta);
const BG_CYAN: &'static str = ansi_colored!(bg, cyan);
const BG_WHITE: &'static str = ansi_colored!(bg, white);
const BG_DEFAULT: &'static str = ansi_colored!(bg, default);

const FG_BLACK: &'static str = ansi_colored!(fg, black);
const FG_RED: &'static str = ansi_colored!(fg, red);
const FG_GREEN: &'static str = ansi_colored!(fg, green);
const FG_YELLOW: &'static str = ansi_colored!(fg, yellow);
const FG_BLUE: &'static str = ansi_colored!(fg, blue);
const FG_MAGENTA: &'static str = ansi_colored!(fg, magenta);
const FG_CYAN: &'static str = ansi_colored!(fg, cyan);
const FG_WHITE: &'static str = ansi_colored!(fg, white);
const FG_DEFAULT: &'static str = ansi_colored!(fg, default);

const TEXT_STYLE_NORMAL: &'static str = ansi_escape!(ansi_text_style_to_id!(normal));
const TEXT_STYLE_BOLD: &'static str = ansi_escape!(ansi_text_style_to_id!(bold));
const TEXT_STYLE_DIMMED: &'static str = ansi_escape!(ansi_text_style_to_id!(dimmed));
const TEXT_STYLE_ITALIC: &'static str = ansi_escape!(ansi_text_style_to_id!(italic));
const TEXT_STYLE_UNDERLINE: &'static str = ansi_escape!(ansi_text_style_to_id!(underline));
const TEXT_STYLE_BLINK: &'static str = ansi_escape!(ansi_text_style_to_id!(blink));

///
#[derive(Debug)]
pub struct AnsiStyle<'a> {
    text_style: Cell<Option<TextStyle>>,
    foreground_color: Cell<Option<Colored>>,
    background_color: Cell<Option<Colored>>,
    msg: Cell<&'a str>,
}

impl<'a> AnsiStyle<'a> {
    pub const fn new() -> Self {
        Self {
            text_style: Cell::new(None),
            foreground_color: Cell::new(None),
            background_color: Cell::new(None),
            msg: Cell::new(""),
        }
    }

    pub fn msg(self, msg: &'a str) -> Self {
        self.msg.replace(msg);
        self
    }

    pub fn foreground_color(self, color: Color) -> Self {
        self.foreground_color.replace(Some(Colored::Fg(color)));
        self
    }

    pub fn background_color(self, color: Color) -> Self {
        self.background_color.replace(Some(Colored::Bg(color)));
        self
    }

    pub fn text_style(self, text_style: TextStyle) -> Self {
        self.text_style.replace(Some(text_style));
        self
    }
}

impl<'a> Display for AnsiStyle<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        // we need the options because otherwise the default values
        // for unset properties reset us all styles
        if let Some(st) = self.text_style.get() {
            write!(f, "{}", st)?;
        }
        if let Some(st) = self.foreground_color.get() {
            write!(f, "{}", st)?;
        }
        if let Some(st) = self.background_color.get() {
            write!(f, "{}", st)?;
        }
        // actual message is wrapped by ANSI sequences
        write!(f, "{}", self.msg.get())?;
        write!(f, "{}", Colored::Reset)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TextStyle {
    Normal,
    Bold,
    Dimmed,
    Italic,
    Underline,
    Blink,
}

impl TextStyle {
    /// Return the ANSI escape sequence.
    const fn sequence(self) -> &'static str {
        match self {
            TextStyle::Normal => TEXT_STYLE_NORMAL,
            TextStyle::Bold => TEXT_STYLE_BOLD,
            TextStyle::Dimmed => TEXT_STYLE_DIMMED,
            TextStyle::Italic => TEXT_STYLE_ITALIC,
            TextStyle::Underline => TEXT_STYLE_UNDERLINE,
            TextStyle::Blink => TEXT_STYLE_BLINK,
        }
    }
}

impl Display for TextStyle {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.sequence())
    }
}

/// ANSI defines the default colors as either a pair
/// of foreground-color or background-color. We can't
/// print a color directly therefore and always must specify
/// the "layer"/dimension/z-index.
#[derive(Copy, Clone, Debug)]
pub enum Colored {
    Fg(Color),
    Bg(Color),
    Reset,
}

impl Colored {
    /// Return the ANSI escape sequence.
    pub const fn sequence(self) -> &'static str {
        match self {
            Colored::Fg(c) => match c {
                Color::Black => FG_BLACK,
                Color::Red => FG_RED,
                Color::Green => FG_GREEN,
                Color::Yellow => FG_YELLOW,
                Color::Blue => FG_BLUE,
                Color::Magenta => FG_MAGENTA,
                Color::Cyan => FG_CYAN,
                Color::White => FG_WHITE,
                Color::Default => FG_DEFAULT,
            },
            Colored::Bg(c) => match c {
                Color::Black => BG_BLACK,
                Color::Red => BG_RED,
                Color::Green => BG_GREEN,
                Color::Yellow => BG_YELLOW,
                Color::Blue => BG_BLUE,
                Color::Magenta => BG_MAGENTA,
                Color::Cyan => BG_CYAN,
                Color::White => BG_WHITE,
                Color::Default => BG_DEFAULT,
            },
            Colored::Reset => ansi_escape!("0"),
        }
    }
}

impl Default for Colored {
    fn default() -> Self {
        Self::Reset
    }
}

impl Display for Colored {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.sequence())
    }
}

/// Helper struct for [`Colored`].
#[derive(Copy, Clone, Debug)]
pub enum Color {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,
    Default = 9,
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;

    #[test]
    fn test_ansi() {
        println!(
            "Normal {}Red {}Normal {}{}GREEN_BG{} {}Italic{}",
            FG_RED,
            FG_DEFAULT,
            BG_GREEN,
            FG_WHITE,
            BG_DEFAULT,
            TEXT_STYLE_ITALIC,
            TEXT_STYLE_NORMAL
        );

        let my_str = String::from("foo");
        let my_str_slice = my_str.as_str();
        let style = AnsiStyle::new()
            .text_style(TextStyle::Underline)
            .background_color(Color::Blue)
            .foreground_color(Color::Red)
            .msg(my_str_slice);
        println!("bar and {}", style);
    }
}
