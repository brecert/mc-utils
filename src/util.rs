use craftping::{self, Chat};
use css_color_parser2::Color as CssColor;
use owo_colors::{DynColors, OwoColorize, Style};

use std::borrow::Cow;

pub fn minecraft_color_to_hex(color: &str) -> Option<Cow<str>> {
    match color {
        "black" => Some(Cow::from("#000000")),
        "dark_blue" => Some(Cow::from("#0000aa")),
        "dark_green" => Some(Cow::from("#00aa00")),
        "dark_aqua" => Some(Cow::from("#00aaaa")),
        "dark_red" => Some(Cow::from("#aa0000")),
        "dark_purple" => Some(Cow::from("#aa00aa")),
        "gold" => Some(Cow::from("#ffaa00")),
        "gray" => Some(Cow::from("#aaaaaa")),
        "dark_gray" => Some(Cow::from("#555555")),
        "blue" => Some(Cow::from("#5555ff")),
        "green" => Some(Cow::from("#55ff55")),
        "aqua" => Some(Cow::from("#55ffff")),
        "red" => Some(Cow::from("#ff5555")),
        "light_purple" => Some(Cow::from("#ff55ff")),
        "yellow" => Some(Cow::from("#ffff55")),
        "white" => Some(Cow::from("#ffffff")),
        _ => None,
    }
}

pub fn tty_style_chat(chat: &Chat) -> String {
    let mut style = Style::new();
    if chat.bold {
        style = style.bold();
    }
    if chat.italic {
        style = style.italic();
    }
    if chat.underlined {
        style = style.underline();
    }
    if chat.strikethrough {
        style = style.strikethrough();
    }
    if chat.obfuscated {
        style = style.blink();
    }
    if let Some(color) = chat.color.clone() {
        let css = minecraft_color_to_hex(&color)
            .unwrap_or_else(|| Cow::from(&color))
            .parse::<CssColor>()
            .expect("Unable to parse web color");

        style = style.color(DynColors::Rgb(css.r, css.g, css.b));
    }
    format!(
        "{}{}",
        chat.text.style(style),
        chat.extra.iter().map(tty_style_chat).collect::<String>()
    )
}
