# ranger-rs

[![Release](https://img.shields.io/github/v/release/adiraj66132/ranger-rs)](https://github.com/adiraj66132/ranger-rs/releases/latest)
[![CI](https://github.com/adiraj66132/ranger-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/adiraj66132/ranger-rs/actions/workflows/rust.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A fast, Vim-powered terminal file manager written in Rust.

ranger-rs is a Rust reimplementation of [ranger](https://github.com/ranger/ranger), the Vim-powered console file manager. It keeps ranger's three-column layout and Vim keybindings, but is rebuilt from scratch in Rust — no Python at startup.

Inspired by:

- **ranger** — the original Python file manager
- **yazi** — the floating info panel (`Tab`)

## Features

- Borderless, dense three-column layout: parent directory, current directory, preview.
- Vim-style navigation and commands.
- `ls -F`-style type markers (`dir/`, `link@`, `exec*`) and right-aligned sizes.
- Text and directory previews, plus configurable external previewers.
- Files that look like text open in your editor; everything else opens with the system opener.
- Floating info panel with size, owner, permissions, timestamps, and MIME type.
- Help overlay for every key and command.
- Eleven built-in themes with a live theme picker.
- Lua plugins for custom commands and single-key bindings.
- No Python runtime.

## Requirements

- Linux or macOS. The code uses Unix filesystem APIs.
- When building from source: Rust stable and a C compiler (Lua is vendored).
- `curl` and `tar` for the prebuilt installer.
- `xdg-open` on Linux or `open` on macOS to open non-text files.

## Install

### Prebuilt binary

```sh
curl -fsSL https://raw.githubusercontent.com/adiraj66132/ranger-rs/master/install.sh | sh
```

This downloads the latest release and installs `ranger-rs` plus the `rr` shortcut to `~/.local/bin`. The installer prints a compact step-by-step summary; set `NO_COLOR=1` to disable color, or run `./install.sh --help` for options.

Prebuilt targets:

- Linux x86_64 and arm64
- macOS Intel and Apple Silicon

From a checkout, `./install.sh --system` installs to `/usr/local/bin`, and `PREFIX=/opt/bin ./install.sh` installs to a custom location.

### Optional: Neovim config

The installer can also set up the bundled Neovim config. It asks first, and only when it has a terminal to ask on, so piping the script into `sh` never touches your editor:

```text
  ? install the bundled Neovim config to ~/.config/nvim? [y/N]
```

Answer `y`, or pass `--nvim` to skip the question. `--no-nvim` skips it outright. The config is LazyVim-based and lives in [`nvim/`](nvim/) in this repository.

If `~/.config/nvim` already exists, it is moved aside to `~/.config/nvim.bak.YYYYMMDD-HHMMSS` before anything is written — the installer never overwrites an existing editor config in place. Dotfiles such as `.ryoku-lazyvim` are included in the copy.

Neovim itself is not installed for you. If `nvim` is missing from `PATH`, the installer says so and leaves the config in place.

### From source

```sh
git clone https://github.com/adiraj66132/ranger-rs.git
cd ranger-rs
cargo build --release
./target/release/ranger-rs /path/to/directory
```

To install the build you just made:

```sh
install -m755 target/release/ranger-rs ~/.local/bin/ranger-rs
```

Every `v*` tag is built and published by GitHub Actions. See the [releases page](https://github.com/adiraj66132/ranger-rs/releases) for binaries and notes.

## Usage

```sh
ranger-rs [PATH]
```

With no path, ranger-rs starts in the current directory. `--help` and `--version` print information and exit.

Press `?` inside the app for the help overlay. Hidden files are off by default; toggle them with `.`.

## Layout

The interface is borderless and built for density:

```text
 ~/Work/git                    ~/Work/git/ranger-rs          app.rs
──────────────────────────────────────────────────────────────────────
 ..                           .github/                      1  use crate::...
 dotfiles/                    docs/                         2  use crate::editor;
 ranger-rs/                   plugins/                      3  use crate::fs::{...
 scratch/                     src/                          4
 > scratch/                   target/                       5
──────────────────────────────────────────────────────────────────────
 j/k move  h/l dirs  . hidden  tab info  : command  ? help   app.rs  2.1K
```

- A thin rule separates the header from the listing; columns are separated by single-cell dividers, not boxes.
- One accent color marks directories and the selected row. Everything else is neutral text.
- The selected row uses a subtle background plus a `>` marker, so selection does not depend on color.
- Rows use `ls -F`-style markers (`dir/`, `link@`, `exec*`) and right-aligned sizes. Directories show `-` instead of a size.
- The parent column drops first on narrow terminals, then the preview. The current column always stays usable.
- The footer keeps the key hints visible. Status messages replace them for about four seconds and then expire.

## Keys

| Key | Action |
| --- | --- |
| `j` / `k` | Move down / up |
| `Ctrl-d` / `Ctrl-u` | Jump 10 down / up |
| `g` / `G` | Top / bottom |
| `h` / `l` | Parent directory / enter directory or open file |
| `Enter` | Enter directory or open file |
| `Tab` | Info panel (size, owner, MIME type, timestamps) |
| `.` | Toggle hidden files |
| `r` | Reload directory |
| `~` | Go to home |
| `:` or `/` | Open the command line |
| `T` | Theme picker |
| `?` | Open the help overlay (`j`/`k` scrolls, `?` closes) |
| `q` / `Ctrl-c` | Quit |

In the theme picker, `j`/`k` or the arrow keys cycle, `g`/`G` jump to the ends, `Enter` saves, and `Esc`/`q` cancels. Each row has a small swatch so the palette is visible without repainting the whole row. On the command line, `Esc` cancels and `Ctrl-u` clears the input.

## Commands

Type them after `:` or `/`:

| Command | Action |
| --- | --- |
| `:q`, `:quit` | Quit |
| `:cd [path]` | Change directory (no path: home) |
| `:mkdir <path>` | Create a directory |
| `:touch <path>` | Create a file, or leave an existing file untouched |
| `:open` | Open the selected entry |
| `:reload` | Reload the current directory |
| `:hidden`, `:toggle_hidden` | Toggle hidden files |
| `:pwd` | Show the current path |
| `:help` | Open the help overlay |
| `:theme [name]` | Switch theme (no name: open the picker) |
| `:themes` | List built-in themes |
| `:<name>` | Run a plugin command |

Relative paths are resolved against the current directory, and a leading `~` expands to your home directory.

## Editors

Files that look like text open in the first editor found on `PATH`:

1. `nvim`
2. `vim`
3. `vi`
4. `nano`
5. `emacs`

If only `nano` or `emacs` is available, you'll see a suggestion to install vim or nvim.

Files that do not look like text (images, PDFs, archives, and other binaries) open with `xdg-open` on Linux or `open` on macOS.

## Config

Create `~/.config/ranger-rs/config.toml`, or `$XDG_CONFIG_HOME/ranger-rs/config.toml` when `XDG_CONFIG_HOME` is set. See [`config.example.toml`](config.example.toml).

| Key | Default | Meaning |
| --- | --- | --- |
| `theme` | `"catppuccin-mocha"` | Built-in theme name |
| `show_hidden` | `false` | Show dotfiles |
| `preview_lines` | `200` | Maximum lines in the preview pane |
| `preview_bytes` | `65536` | Maximum bytes read for text previews |

```toml
theme = "catppuccin-mocha"
show_hidden = false
preview_lines = 200
preview_bytes = 65536
```

### External previewers

Add `[[scope]]` rules to run a command for matching files. Rules are checked top to bottom and the first matching rule wins. The selected path is available as `$RANGER_RS_PATH`. If the command fails or times out, the built-in preview is used instead.

```toml
[[scope]]
match = "*.pdf"
command = 'pdftotext -l 2 "$RANGER_RS_PATH" -'
timeout_ms = 1500

[[scope]]
match = "*.zip"
command = 'unzip -l "$RANGER_RS_PATH"'
timeout_ms = 1500
```

`timeout_ms` defaults to 1500.

## Themes

Built-in themes: `default`, `catppuccin-mocha`, `catppuccin-macchiato`, `catppuccin-frappe`, `catppuccin-latte`, `tokyo-night`, `tokyo-night-storm`, `tokyo-night-day`, `nord`, `dracula`, `gruvbox`.

Press `T` or run `:theme <name>` to switch. The picker shows a live preview; confirming a theme saves it to `config.toml`.

## Plugins

Lua plugins go in `~/.config/ranger-rs/plugins/*.lua` and are loaded at startup. See [`plugins/example.lua`](plugins/example.lua).

```lua
ranger.register_command("hello", function(args)
    local name = args ~= "" and args or "world"
    ranger.notify("hello " .. name .. " from " .. ranger.cwd())
end)
ranger.register_key("H", "hello")
```

Run a registered command as `:hello`, or press its key in normal mode.

Available API:

| Function | Meaning |
| --- | --- |
| `ranger.register_command(name, fn)` | Register a command; `fn` receives the argument string |
| `ranger.register_key(key, command)` | Bind the first character of `key` to a command |
| `ranger.notify(message)` | Show a message in the status line |
| `ranger.cd(path)` | Change directory |
| `ranger.quit()` | Quit |
| `ranger.reload()` | Reload the current directory |
| `ranger.toggle_hidden()` | Toggle hidden files |
| `ranger.mkdir(path)` | Create a directory |
| `ranger.touch(path)` | Create a file if it does not exist |
| `ranger.cwd()` | Return the current directory |

Commands registered by a plugin can be invoked from the command line, and keys are checked after the built-in bindings.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets
cargo test
```

CI runs the build and test suite for pushes and pull requests to `master`. See [CONTRIBUTING.md](CONTRIBUTING.md) for project layout, style, and release notes.

## License

[MIT](LICENSE)
