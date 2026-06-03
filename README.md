Basic rust fetch for nix.

## Config

Copetch reads TOML config from (first match wins):

1. `./config/copetch/config.toml`
2. `~/.config/copetch/config.toml`

Minimal example:

```toml
[display]
mode = "ascii" # ascii | png | gif | text

[image]
cols = 20
rows = 12
gap  = 3

[logo]
path = "./config/copetch/cop.txt"

[colors]
key = "cyan"
accent = "bright_black"
value = "white"
separator = "bright_black"
reset = "reset"
bold = "bold"
bar = ["red", "green", "yellow", "blue", "magenta", "cyan", "white"]

[[modules]]
type = "title"

[[modules]]
type = "divider"

[[modules]]
type = "os"
label = "os"

[[modules]]
type = "kernel"
label = "ker"

[[modules]]
type = "uptime"
label = "up"

[[modules]]
type = "shell"
label = "sh"

[[modules]]
type = "wm"
label = "wm"

[[modules]]
type = "terminal"
label = "term"

[[modules]]
type = "cpu"
label = "cpu"

[[modules]]
type = "gpu"
label = "gpu"

[[modules]]
type = "memory"
label = "mem"

[[modules]]
type = "swap"
label = "swap"

[[modules]]
type = "disk"
label = "disk"

[[modules]]
type = "packages"
label = "pkgs"

[[modules]]
type = "generation"
label = "gen"

[[modules]]
type = "divider"

[[modules]]
type = "color-bar"

[[modules]]
type = "custom"
label = "ip"
command = "hostname -I | awk '{print $1}'"
```
