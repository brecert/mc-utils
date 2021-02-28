use craftping::{self, Chat};
use css_color_parser2::Color as CssColor;
use owo_colors::{DynColors, OwoColorize, Style};

pub fn minecraft_color_to_hex(color: &str) -> Option<String> {
    match color {
        "black" => Some(String::from("#000000")),
        "dark_blue" => Some(String::from("#0000aa")),
        "dark_green" => Some(String::from("#00aa00")),
        "dark_aqua" => Some(String::from("#00aaaa")),
        "dark_red" => Some(String::from("#aa0000")),
        "dark_purple" => Some(String::from("#aa00aa")),
        "gold" => Some(String::from("#ffaa00")),
        "gray" => Some(String::from("#aaaaaa")),
        "dark_gray" => Some(String::from("#555555")),
        "blue" => Some(String::from("#5555ff")),
        "green" => Some(String::from("#55ff55")),
        "aqua" => Some(String::from("#55ffff")),
        "red" => Some(String::from("#ff5555")),
        "light_purple" => Some(String::from("#ff55ff")),
        "yellow" => Some(String::from("#ffff55")),
        "white" => Some(String::from("#ffffff")),
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
            .unwrap_or(color)
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
