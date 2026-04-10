mod display;
mod system;

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_default();
    let mode = match mode.as_str() {
        "--gif"   => display::Mode::Gif,
        "--png"   => display::Mode::Png,
        "--ascii" => display::Mode::Ascii,
        _         => display::Mode::Ascii,
    };
    let info = system::collect();
    display::render(info, mode);
}