use crate::config::{self, Module};
use crate::system::SysInfo;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::{AnimationDecoder, ImageFormat};
use std::{
    fs,
    io::{self, BufWriter, Cursor, Write},
    path::Path,
    process::Command,
};

const CHUNK: usize = 4096;

pub enum Mode {
    Gif,
    Png,
    Ascii,
    Text,
}

fn vlen(s: &str) -> usize {
    let mut n = 0;
    let mut esc = false;
    for c in s.chars() {
        match c {
            '\x1b' => esc = true,
            c if esc => {
                if c.is_ascii_alphabetic() {
                    esc = false;
                }
            }
            _ => n += 1,
        }
    }
    n
}

struct Theme {
    key: String,
    accent: String,
    value: String,
    reset: String,
    bold: String,
    separator: String,
    bar: Vec<String>,
}

impl Theme {
    fn from_cfg(colors: &config::ColorConfig) -> Self {
        Self {
            key: resolve_color(&colors.key),
            accent: resolve_color(&colors.accent),
            value: resolve_color(&colors.value),
            reset: resolve_color(&colors.reset),
            bold: resolve_color(&colors.bold),
            separator: resolve_color(&colors.separator),
            bar: colors.bar.iter().map(|c| resolve_color(c)).collect(),
        }
    }
}

fn resolve_color(raw: &str) -> String {
    if raw.is_empty() {
        return String::new();
    }
    if raw.contains("\x1b[") {
        return raw.to_string();
    }
    let code = match raw {
        "reset" => "\x1b[0m",
        "bold" => "\x1b[1m",
        "black" => "\x1b[30m",
        "red" => "\x1b[31m",
        "green" => "\x1b[32m",
        "yellow" => "\x1b[33m",
        "blue" => "\x1b[34m",
        "magenta" => "\x1b[35m",
        "cyan" => "\x1b[36m",
        "white" => "\x1b[37m",
        "bright_black" | "gray" | "grey" => "\x1b[90m",
        "bright_red" => "\x1b[91m",
        "bright_green" => "\x1b[92m",
        "bright_yellow" => "\x1b[93m",
        "bright_blue" => "\x1b[94m",
        "bright_magenta" => "\x1b[95m",
        "bright_cyan" => "\x1b[96m",
        "bright_white" => "\x1b[97m",
        _ => {
            eprintln!("copetch: unknown color '{raw}'");
            ""
        }
    };
    code.to_string()
}

fn row(theme: &Theme, key: &str, val: &str) -> String {
    format!(
        "  {k}{key:<5}{r} {s}>{r} {v}{val}{r}\n",
        k = theme.key,
        r = theme.reset,
        s = theme.separator,
        v = theme.value
    )
}

fn color_bar(theme: &Theme) -> String {
    if theme.bar.is_empty() {
        return String::new();
    }
    let mut s = String::from("  ");
    for c in &theme.bar {
        s.push_str(c);
        s.push_str("██");
    }
    s.push_str(&theme.reset);
    s.push('\n');
    s
}

fn build_lines(info: &SysInfo, cfg: &config::Config, theme: &Theme) -> Vec<String> {
    let div_w = format!("{}@{}", info.user, info.host).len().max(24);
    let div = format!("  {a}{}{r}\n", "─".repeat(div_w), a = theme.accent, r = theme.reset);
    let pkgs = format!("{} nix  {} hm", info.nix_pkgs, info.hm_pkgs);
    let gen = format!("{} · {}", info.nix_gen, info.nix_rev);
    let mut lines = Vec::with_capacity(cfg.modules.len() + 4);

    for module in &cfg.modules {
        match module {
            Module::Title { label } => {
                let title = label
                    .as_deref()
                    .unwrap_or("{user}@{host}")
                    .replace("{user}", &info.user)
                    .replace("{host}", &info.host);
                lines.push(format!(
                    "  {b}{title}{r}\n",
                    b = theme.bold,
                    r = theme.reset
                ));
            }
            Module::Divider => lines.push(div.clone()),
            Module::Os { label } => lines.push(row(theme, label.as_deref().unwrap_or("os"), &info.os)),
            Module::Kernel { label } => lines.push(row(theme, label.as_deref().unwrap_or("ker"), &info.kernel)),
            Module::Uptime { label } => lines.push(row(theme, label.as_deref().unwrap_or("up"), &info.uptime)),
            Module::Shell { label } => lines.push(row(theme, label.as_deref().unwrap_or("sh"), &info.shell)),
            Module::Wm { label } => lines.push(row(theme, label.as_deref().unwrap_or("wm"), &info.wm)),
            Module::Terminal { label } => lines.push(row(theme, label.as_deref().unwrap_or("term"), &info.terminal)),
            Module::Cpu { label } => lines.push(row(theme, label.as_deref().unwrap_or("cpu"), &info.cpu)),
            Module::Gpu { label } => lines.push(row(theme, label.as_deref().unwrap_or("gpu"), &info.gpu)),
            Module::Memory { label } => lines.push(row(theme, label.as_deref().unwrap_or("mem"), &info.memory)),
            Module::Swap { label } => lines.push(row(theme, label.as_deref().unwrap_or("swap"), &info.swap)),
            Module::Disk { label } => lines.push(row(theme, label.as_deref().unwrap_or("disk"), &info.disk)),
            Module::Packages { label } => lines.push(row(theme, label.as_deref().unwrap_or("pkgs"), &pkgs)),
            Module::Generation { label } => lines.push(row(theme, label.as_deref().unwrap_or("gen"), &gen)),
            Module::ColorBar => lines.push(color_bar(theme)),
            Module::Custom { label, command } => {
                let val = run_command(command);
                lines.push(row(theme, label, &val));
            }
        }
    }
    lines
}

fn run_command(command: &str) -> String {
    let output = Command::new("sh").arg("-c").arg(command).output();
    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            text.lines().next().unwrap_or("N/A").trim().to_string()
        }
        Ok(out) => {
            let msg = String::from_utf8_lossy(&out.stderr);
            eprintln!("copetch: command failed: {command}: {}", msg.trim());
            "N/A".into()
        }
        Err(err) => {
            eprintln!("copetch: command failed: {command}: {err}");
            "N/A".into()
        }
    }
}

fn kitty_chunk(png_bytes: &[u8], cols: u32, rows: u32) -> String {
    let b64 = STANDARD.encode(png_bytes);
    let chunks: Vec<&[u8]> = b64.as_bytes().chunks(CHUNK).collect();
    let total = chunks.len();
    let mut s = String::new();
    for (i, chunk) in chunks.iter().enumerate() {
        let more = u8::from(i + 1 < total);
        let c = std::str::from_utf8(chunk).unwrap();
        if i == 0 {
            s.push_str(&format!(
                "\x1b_Ga=T,t=d,f=100,c={cols},r={rows},q=2,m={more};{c}\x1b\\"
            ));
        } else {
            s.push_str(&format!("\x1b_Gm={more};{c}\x1b\\"));
        }
    }
    s
}

fn gif_to_png(raw: &[u8]) -> Option<Vec<u8>> {
    let decoder = image::codecs::gif::GifDecoder::new(Cursor::new(raw)).ok()?;
    let mut frames = decoder.into_frames();
    let frame = frames.next()?.ok()?;
    let img = frame.into_buffer();
    let mut png_bytes: Vec<u8> = Vec::new();
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)
        .ok()?;
    Some(png_bytes)
}

fn render_kitty(
    output: &mut String,
    png_bytes: &[u8],
    lines: &[String],
    img: &config::ImageConfig,
) {
    output.push_str("\x1b[s");
    output.push_str(&kitty_chunk(png_bytes, img.cols, img.rows));
    output.push_str("\x1b[u");
    let total = img.rows.max(lines.len() as u32) as usize;
    for i in 0..total {
        output.push_str("\x1b[s");
        let right = format!("\x1b[{}C", img.cols * 2 + img.gap);
        output.push_str(&right);
        if let Some(line) = lines.get(i) {
            output.push_str(line);
        }
        output.push_str("\x1b[u");
        output.push_str("\x1b[1B");
    }
    output.push('\n');
}

fn render_ascii(output: &mut String, art: &str, lines: &[String], theme: &Theme) {
    let art_lines: Vec<&str> = art.lines().collect();
    let max_w = art_lines.iter().map(|l| vlen(l)).max().unwrap_or(0);
    let total = art_lines.len().max(lines.len());
    for i in 0..total {
        let a = art_lines.get(i).copied().unwrap_or("");
        let t = lines.get(i).map(|s| s.trim_end_matches('\n')).unwrap_or("");
        let pad = " ".repeat(max_w.saturating_sub(vlen(a)));
        output.push_str(&format!(
            "{k}{a}{pad}{r}   {t}\n",
            k = theme.key,
            r = theme.reset
        ));
    }
}

fn render_plain(output: &mut String, lines: &[String]) {
    for line in lines {
        output.push_str(line);
    }
    output.push('\n');
}

fn logo_path(mode: &Mode, cfg: &config::Config) -> String {
    if let Some(path) = &cfg.logo.path {
        return path.clone();
    }
    let home = std::env::var("HOME").unwrap_or_default();
    match mode {
        Mode::Gif => format!("{home}/.config/copetch/cop.gif"),
        Mode::Png => format!("{home}/.config/copetch/cop.png"),
        Mode::Ascii => format!("{home}/.config/copetch/cop.txt"),
        Mode::Text => String::new(),
    }
}

pub fn render(info: SysInfo, mode: Mode, cfg: &config::Config) {
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    let theme = Theme::from_cfg(&cfg.colors);
    let lines = build_lines(&info, cfg, &theme);
    let mut output = String::with_capacity(4096);
    output.push('\n');

    match mode {
        Mode::Gif => {
            let path = logo_path(&mode, cfg);
            if Path::new(&path).exists() {
                match fs::read(&path).ok().and_then(|r| gif_to_png(&r)) {
                    Some(png) => render_kitty(&mut output, &png, &lines, &cfg.image),
                    #[allow(non_snake_case)]
                    None => {
                        eprintln!("copetch: failed to decode {path}");
                        render_plain(&mut output, &lines);
                    }
                }
            } else {
                eprintln!("copetch: {path} not found");
                render_plain(&mut output, &lines);
            }
        }

        Mode::Png => {
            let path = logo_path(&mode, cfg);
            if Path::new(&path).exists() {
                match fs::read(&path) {
                    Ok(raw) => render_kitty(&mut output, &raw, &lines, &cfg.image),
                    Err(_) => {
                        eprintln!("copetch: failed to read {path}");
                        render_plain(&mut output, &lines);
                    }
                }
            } else {
                eprintln!("copetch: {path} not found");
                render_plain(&mut output, &lines);
            }
        }

        Mode::Ascii => {
            let path = logo_path(&mode, cfg);
            if Path::new(&path).exists() {
                match fs::read_to_string(&path) {
                    Ok(art) => render_ascii(&mut output, &art, &lines, &theme),
                    Err(_) => {
                        eprintln!("copetch: failed to read {path}");
                        render_plain(&mut output, &lines);
                    }
                }
            } else {
                eprintln!("copetch: {path} not found");
                render_plain(&mut output, &lines);
            }
        }

        Mode::Text => render_plain(&mut output, &lines),
    }

    write!(out, "{output}").unwrap();
    out.flush().unwrap();
}
