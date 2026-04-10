use std::{env, fs, process::Command, thread};

pub struct SysInfo {
    pub user: String,
    pub host: String,
    pub os: String,
    pub kernel: String,
    pub uptime: String,
    pub memory: String,
    pub swap: String,
    pub shell: String,
    pub wm: String,
    pub terminal: String,
    pub cpu: String,
    pub gpu: String,
    pub disk: String,
    pub nix_pkgs: usize,
    pub hm_pkgs: usize,
    pub nix_gen: String,
    pub nix_rev: String,
}

pub fn collect() -> SysInfo {
    let gpu_h  = thread::spawn(get_gpu);
    let disk_h = thread::spawn(|| get_disk("/"));

    let user = env::var("USER").unwrap_or_else(|_| "user".into());
    let host = read("/proc/sys/kernel/hostname").trim().to_string();
    let os   = get_os();
    let kernel = read("/proc/sys/kernel/osrelease").trim().to_string();
    let uptime = get_uptime();
    let (memory, swap) = get_mem_swap();
    let shell    = get_shell();
    let wm       = get_wm();
    let terminal = get_terminal();
    let cpu      = get_cpu();
    let (nix_pkgs, hm_pkgs) = get_packages(&user);
    let nix_gen  = get_nix_gen();
    let nix_rev  = get_nix_rev();

    SysInfo {
        user, host, os, kernel, uptime, memory, swap,
        shell, wm, terminal, cpu,
        gpu:  gpu_h.join().unwrap_or_else(|_| "unknown".into()),
        disk: disk_h.join().unwrap_or_else(|_| "N/A".into()),
        nix_pkgs, hm_pkgs, nix_gen, nix_rev,
    }
}

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

fn get_os() -> String {
    for line in read("/etc/os-release").lines() {
        if let Some(v) = line.strip_prefix("PRETTY_NAME=") {
            return v.trim_matches('"').to_string();
        }
    }
    "NixOS".into()
}

fn get_nix_rev() -> String {
    for line in read("/etc/os-release").lines() {
        if let Some(v) = line.strip_prefix("BUILD_ID=") {
            let v = v.trim_matches('"');
            if let Some(rev) = v.rsplit('.').next() {
                return rev[..rev.len().min(8)].to_string();
            }
            return v[..v.len().min(8)].to_string();
        }
    }
    if let Ok(v) = fs::read_to_string("/run/current-system/nixos-version") {
        if let Some(rev) = v.trim().rsplit('.').next() {
            return rev[..rev.len().min(8)].to_string();
        }
    }
    "unknown".into()
}

fn get_uptime() -> String {
    let s: f64 = read("/proc/uptime")
        .split_whitespace().next()
        .and_then(|x| x.parse().ok())
        .unwrap_or(0.0);
    let h = (s / 3600.0) as u64;
    let m = ((s % 3600.0) / 60.0) as u64;
    if h > 0 { format!("{}h {}m", h, m) } else { format!("{}m", m) }
}

fn get_mem_swap() -> (String, String) {
    let content = read("/proc/meminfo");
    let (mut mt, mut ma, mut st, mut sf) = (0u64, 0u64, 0u64, 0u64);
    let mut found = 0u8;
    for line in content.lines() {
        if      line.starts_with("MemTotal:")    { mt = kb(line); found += 1; }
        else if line.starts_with("MemAvailable:"){ ma = kb(line); found += 1; }
        else if line.starts_with("SwapTotal:")   { st = kb(line); found += 1; }
        else if line.starts_with("SwapFree:")    { sf = kb(line); found += 1; }
        if found == 4 { break; }
    }
    let gib = |v: u64| v as f64 / 1_048_576.0;
    let mem  = format!("{:.1} / {:.1} GiB", gib(mt.saturating_sub(ma)), gib(mt));
    let swap = if st == 0 {
        "none".into()
    } else {
        format!("{:.1} / {:.1} GiB", gib(st.saturating_sub(sf)), gib(st))
    };
    (mem, swap)
}

fn kb(line: &str) -> u64 {
    line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0)
}

fn get_shell() -> String {
    env::var("SHELL").unwrap_or_default()
        .rsplit('/').next().unwrap_or("sh").to_string()
}

fn get_wm() -> String {
    env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| env::var("DESKTOP_SESSION"))
        .unwrap_or_else(|_| "tty".into())
}

fn get_terminal() -> String {
    if env::var("GHOSTTY_RESOURCES_DIR").is_ok() { return "Ghostty".into(); }
    env::var("TERM_PROGRAM")
        .or_else(|_| env::var("TERM"))
        .unwrap_or_else(|_| "unknown".into())
}

fn get_cpu() -> String {
    for line in read("/proc/cpuinfo").lines() {
        if line.starts_with("model name") {
            if let Some(v) = line.splitn(2, ':').nth(1) {
                return v.trim()
                    .replace("(R)", "").replace("(TM)", "")
                    .replace(" Processor", "").replace(" CPU", "");
            }
        }
    }
    "unknown".into()
}

fn get_gpu() -> String {
    let Ok(out) = Command::new("lspci").output() else { return "unknown".into(); };
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        if line.contains("VGA compatible controller") || line.contains("3D controller") {
            if let Some(part) = line.splitn(2, ": ").nth(1) {
                let mut name = part
                    .replace(" Corporation", "")
                    .replace(" Advanced Micro Devices, Inc. [AMD/ATI]", "AMD")
                    .replace("3rd Gen Core processor Graphics Controller", "HD 4000");
                if let Some(i) = name.find(" (rev") { name.truncate(i); }
                return name.trim().to_string();
            }
        }
    }
    "unknown".into()
}

fn get_disk(path: &str) -> String {
    let Ok(out) = Command::new("df").args(["-h", path]).output() else {
        return "N/A".into();
    };
    let stdout = String::from_utf8_lossy(&out.stdout);
    if let Some(line) = stdout.lines().nth(1) {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() >= 5 {
            return format!("{} / {} ({})", f[2], f[1], f[4]);
        }
    }
    "N/A".into()
}

fn get_packages(user: &str) -> (usize, usize) {
    let nix = fs::read_dir("/run/current-system/sw/bin")
        .map(|d| d.filter_map(Result::ok).count()).unwrap_or(0);
    let hm = fs::read_dir(format!("/etc/profiles/per-user/{}/bin", user))
        .map(|d| d.filter_map(Result::ok).count()).unwrap_or(0);
    (nix, hm)
}

fn get_nix_gen() -> String {
    fs::read_link("/nix/var/nix/profiles/system")
        .ok()
        .and_then(|p| {
            p.to_string_lossy().split('-').nth(1).map(|s| s.to_string())
        })
        .unwrap_or_else(|| "?".into())
}