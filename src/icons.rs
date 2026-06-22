use ratatui::style::Color;

// Same icon set as nvim-web-devicons (Nerd Fonts)
pub fn icon_for(name: &str, is_dir: bool) -> (&'static str, Color) {
    if is_dir {
        if name == ".." {
            return ("\u{f07b} ", Color::DarkGray); // folder (dim, going back)
        }
        return ("\u{f07b} ", Color::Yellow); // folder
    }

    let ext = std::path::Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match ext.to_lowercase().as_str() {
        "rs"                     => ("\u{e7a8} ", Color::Red),
        "py" | "pyw"             => ("\u{e73c} ", Color::Yellow),
        "js" | "mjs" | "cjs"    => ("\u{e74e} ", Color::Yellow),
        "ts" | "mts"             => ("\u{e628} ", Color::Cyan),
        "jsx"                    => ("\u{e7ba} ", Color::Cyan),
        "tsx"                    => ("\u{e7ba} ", Color::Cyan),
        "lua"                    => ("\u{e620} ", Color::Blue),
        "go"                     => ("\u{e627} ", Color::Cyan),
        "c" | "h"                => ("\u{e61e} ", Color::Blue),
        "cpp" | "cc" | "hpp"     => ("\u{e61d} ", Color::Blue),
        "cs"                     => ("\u{e648} ", Color::Magenta),
        "java"                   => ("\u{e738} ", Color::Red),
        "rb"                     => ("\u{e739} ", Color::Red),
        "php"                    => ("\u{e73d} ", Color::Magenta),
        "swift"                  => ("\u{e755} ", Color::Red),
        "kt" | "kts"             => ("\u{e634} ", Color::Magenta),
        "html" | "htm"           => ("\u{e736} ", Color::Red),
        "css"                    => ("\u{e749} ", Color::Blue),
        "scss" | "sass"          => ("\u{e603} ", Color::Magenta),
        "md" | "markdown"        => ("\u{e609} ", Color::White),
        "json"                   => ("\u{e60b} ", Color::Yellow),
        "toml"                   => ("\u{e60b} ", Color::Gray),
        "yaml" | "yml"           => ("\u{e60b} ", Color::Red),
        "xml"                    => ("\u{e60b} ", Color::Red),
        "sh" | "bash" | "zsh" | "fish" => ("\u{e795} ", Color::Green),
        "sql"                    => ("\u{e706} ", Color::Cyan),
        "dockerfile"             => ("\u{e650} ", Color::Cyan),
        "txt"                    => ("\u{f15c} ", Color::Gray),
        "pdf"                    => ("\u{f1c1} ", Color::Red),
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" => ("\u{f1c5} ", Color::Magenta),
        "zip" | "tar" | "gz" | "rar" => ("\u{f1c6} ", Color::Yellow),
        _                        => ("\u{f15b} ", Color::Gray),
    }
}
