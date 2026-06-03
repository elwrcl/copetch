use serde::Deserialize;
use std::{env, fs, path::PathBuf};

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub display: DisplayConfig,
    pub image: ImageConfig,
    pub logo: LogoConfig,
    pub colors: ColorConfig,
    pub modules: Vec<Module>,
}

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct DisplayConfig {
    pub mode: String,
}

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct ImageConfig {
    pub cols: u32,
    pub rows: u32,
    pub gap: u32,
}

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct LogoConfig {
    pub path: Option<String>,
}

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct ColorConfig {
    pub key: String,
    pub accent: String,
    pub value: String,
    pub reset: String,
    pub bold: String,
    pub bar: Vec<String>,
    pub separator: String,
}

#[derive(Clone, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Module {
    Title { label: Option<String> },
    Divider,
    Os { label: Option<String> },
    Kernel { label: Option<String> },
    Uptime { label: Option<String> },
    Shell { label: Option<String> },
    Wm { label: Option<String> },
    Terminal { label: Option<String> },
    Cpu { label: Option<String> },
    Gpu { label: Option<String> },
    Memory { label: Option<String> },
    Swap { label: Option<String> },
    Disk { label: Option<String> },
    Packages { label: Option<String> },
    Generation { label: Option<String> },
    ColorBar,
    Custom { label: String, command: String },
}

impl Default for Config {
    fn default() -> Self {
        Self {
            display: DisplayConfig::default(),
            image: ImageConfig::default(),
            logo: LogoConfig::default(),
            colors: ColorConfig::default(),
            modules: default_modules(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self { mode: "ascii".into() }
    }
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self { cols: 20, rows: 12, gap: 3 }
    }
}

impl Default for LogoConfig {
    fn default() -> Self {
        Self { path: None }
    }
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            key: "cyan".into(),
            accent: "bright_black".into(),
            value: "white".into(),
            reset: "reset".into(),
            bold: "bold".into(),
            separator: "bright_black".into(),
            bar: vec![
                "red".into(),
                "green".into(),
                "yellow".into(),
                "blue".into(),
                "magenta".into(),
                "cyan".into(),
                "white".into(),
            ],
        }
    }
}

fn default_modules() -> Vec<Module> {
    vec![
        Module::Title { label: None },
        Module::Divider,
        Module::Os { label: Some("os".into()) },
        Module::Kernel { label: Some("ker".into()) },
        Module::Uptime { label: Some("up".into()) },
        Module::Shell { label: Some("sh".into()) },
        Module::Wm { label: Some("wm".into()) },
        Module::Terminal { label: Some("term".into()) },
        Module::Cpu { label: Some("cpu".into()) },
        Module::Gpu { label: Some("gpu".into()) },
        Module::Memory { label: Some("mem".into()) },
        Module::Swap { label: Some("swap".into()) },
        Module::Disk { label: Some("disk".into()) },
        Module::Packages { label: Some("pkgs".into()) },
        Module::Generation { label: Some("gen".into()) },
        Module::Divider,
        Module::ColorBar,
    ]
}

pub fn load() -> Config {
    let mut paths = Vec::<PathBuf>::new();
    paths.push(PathBuf::from("./config/copetch/config.toml"));
    if let Ok(home) = env::var("HOME") {
        paths.push(PathBuf::from(format!("{home}/.config/copetch/config.toml")));
    }

    for path in paths {
        if let Ok(text) = fs::read_to_string(&path) {
            match toml::from_str::<Config>(&text) {
                Ok(cfg) => return cfg,
                Err(err) => {
                    eprintln!("copetch: failed to parse {}: {err}", path.display());
                    return Config::default();
                }
            }
        }
    }

    Config::default()
}
