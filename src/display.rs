use crate::system::SysInfo;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::{AnimationDecoder, ImageFormat};
use std::{
    fs,
    io::{self, BufWriter, Cursor, Write},
    path::Path,
};

const CY: &str = "\x1b[36m";
const GR: &str = "\x1b[90m";
const BD: &str = "\x1b[1m";
const RS: &str = "\x1b[0m";

const CHUNK:    usize = 4096;
const IMG_COLS: u32   = 20;
const IMG_ROWS: u32   = 12;
const IMG_GAP:  u32   = 3;

pub enum Mode {
    Gif,
    Png,
    Ascii,
}

fn vlen(s: &str) -> usize {
    let mut n = 0;
    let mut esc = false;
    for c in s.chars() {
        match c {
            '\x1b' => esc = true,
            c if esc => { if c.is_ascii_alphabetic() { esc = false; } }
            _ => n += 1,
        }
    }
    n
}

fn row(key: &str, val: &str) -> String {
    format!("  {CY}{key:<5}{RS} {GR}>{RS} {val}\n")
}

fn color_bar() -> String {
    format!("  \x1b[31m██\x1b[32m██\x1b[33m██\x1b[34m██\x1b[35m██\x1b[36m██\x1b[37m██{RS}\n")
}

fn build_lines(info: &SysInfo) -> Vec<String> {
    let div_w = format!("{}@{}", info.user, info.host).len().max(24);
    let div   = format!("  {GR}{}{RS}\n", "─".repeat(div_w));
    let pkgs  = format!("{} nix  {} hm", info.nix_pkgs, info.hm_pkgs);
    let gen   = format!("{} · {}", info.nix_gen, info.nix_rev);
    vec![
        format!("  {BD}{}{GR}@{RS}{BD}{}{RS}\n", info.user, info.host),
        div.clone(),
        row("os",   &info.os),
        row("ker",  &info.kernel),
        row("up",   &info.uptime),
        row("sh",   &info.shell),
        row("wm",   &info.wm),
        row("term", &info.terminal),
        row("cpu",  &info.cpu),
        row("gpu",  &info.gpu),
        row("mem",  &info.memory),
        row("swap", &info.swap),
        row("disk", &info.disk),
        row("pkgs", &pkgs),
        row("gen",  &gen),
        div,
        color_bar(),
    ]
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

fn render_kitty(output: &mut String, png_bytes: &[u8], lines: &[String]) {
    output.push_str("\x1b[s");
    output.push_str(&kitty_chunk(png_bytes, IMG_COLS, IMG_ROWS));
    output.push_str("\x1b[u");
    let total = IMG_ROWS.max(lines.len() as u32) as usize;
    for i in 0..total {
        
        output.push_str("\x1b[s");
        let right = format!("\x1b[{}C", IMG_COLS * 2 + IMG_GAP);
        output.push_str(&right);
        if let Some(line) = lines.get(i) {
            output.push_str(line);
        }

        output.push_str("\x1b[u");
        output.push_str("\x1b[1B");
    }
    output.push('\n');
}

fn render_ascii(output: &mut String, art: &str, lines: &[String]) {
    let art_lines: Vec<&str> = art.lines().collect();
    let max_w = art_lines.iter().map(|l| vlen(l)).max().unwrap_or(0);
    let total = art_lines.len().max(lines.len());
    for i in 0..total {
        let a   = art_lines.get(i).copied().unwrap_or("");
        let t   = lines.get(i).map(|s| s.trim_end_matches('\n')).unwrap_or("");
        let pad = " ".repeat(max_w.saturating_sub(vlen(a)));
        output.push_str(&format!("{CY}{a}{pad}{RS}   {t}\n"));
    }
}

fn render_plain(output: &mut String, lines: &[String]) {
    for line in lines { output.push_str(line); }
    output.push('\n');
}

pub fn render(info: SysInfo, mode: Mode) {
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    let home = std::env::var("HOME").unwrap_or_default();
    let cfg  = format!("{}/.config/copetch", home);
    let lines = build_lines(&info);
    let mut output = String::with_capacity(4096);
    output.push('\n');

    match mode {
        Mode::Gif => {
            let path = format!("{cfg}/cop.gif");
            if Path::new(&path).exists() {
                match fs::read(&path).ok().and_then(|r| gif_to_png(&r)) {
                    Some(png) => render_kitty(&mut output, &png, &lines),
                    #[allow(non_snake_case)]
                    None => {
                        eprintln!("copetch: failed to decode cop.gif");
                        render_plain(&mut output, &lines);
                    }
                }
            } else {
                eprintln!("copetch: ~/.config/copetch/cop.gif not found");
                render_plain(&mut output, &lines);
            }
        }

        Mode::Png => {
            let path = format!("{cfg}/cop.png");
            if Path::new(&path).exists() {
                match fs::read(&path) {
                    Ok(raw) => render_kitty(&mut output, &raw, &lines),
                    Err(_) => {
                        eprintln!("copetch: failed to read cop.png");
                        render_plain(&mut output, &lines);
                    }
                }
            } else {
                eprintln!("copetch: ~/.config/copetch/cop.png not found");
                render_plain(&mut output, &lines);
            }
        }

        Mode::Ascii => {
            let path = format!("{cfg}/cop.txt");
            if Path::new(&path).exists() {
                match fs::read_to_string(&path) {
                    Ok(art) => render_ascii(&mut output, &art, &lines),
                    Err(_) => {
                        eprintln!("copetch: failed to read cop.txt");
                        render_plain(&mut output, &lines);
                    }
                }
            } else {
                eprintln!("copetch: ~/.config/copetch/cop.txt not found");
                render_plain(&mut output, &lines);
            }
        }
    }

    write!(out, "{output}").unwrap();
    out.flush().unwrap();
}