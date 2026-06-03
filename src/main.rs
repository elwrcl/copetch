mod config;
mod display;
mod system;

fn main() {
    let cfg = config::load();
    let arg = std::env::args().nth(1).unwrap_or_default();
    let mode = match arg.as_str() {
        "--gif"   => display::Mode::Gif,
        "--png"   => display::Mode::Png,
        "--ascii" => display::Mode::Ascii,
        "--text"  => display::Mode::Text,
        _ => match cfg.display.mode.as_str() {
            "gif"  => display::Mode::Gif,
            "png"  => display::Mode::Png,
            "text" => display::Mode::Text,
            _      => display::Mode::Ascii,
        },
    };
    display::render(system::collect(), mode, &cfg);
}
