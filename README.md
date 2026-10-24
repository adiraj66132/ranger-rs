# ranger-rs

A fast, Vim-powered terminal file manager written in Rust.

ranger-rs is a Rust reimplementation of [ranger](https://github.com/ranger/ranger),
the classic Vim-powered console file manager, with a modern async UI and a
Yazi-inspired floating info panel. It is a clean-room rewrite — no Python at
startup, no ranger source copied — built for speed and extensibility.

The original Python ranger lives at `/home/adiraj/git/ranger` for reference and
is intentionally untouched.

## Features

| Feature | Description |
| --- | --- |
| **Vim keys** | `hjkl`, `g/G`, `Ctrl-d/u`, `~`, `.`, `q` — ranger's keybindings, ported |
| **Three-pane UI** | Parent directory, current directory, and preview side-by-side |
| **Async previews** | Scope rules with timeout; slow previews never block the UI |
| **Floating info panel** | `Tab` opens a Yazi-style popup: size, owner, MIME type, timestamps, permissions, path |
| **Editor detection** | `nvim` → `vim` → `vi` → `nano` → `emacs`, with TUI suspend/resume |
| **Built-in themes** | `catppuccin-*`, `tokyo-night-*`, `nord`, `dracula`, `gruvbox`, `default` |
| **Theme picker** | `T` or `:theme` — live preview, saves to `config.toml` |
| **Lua plugins** | `~/.config/ranger-rs/plugins/*.lua` via embedded `mlua` runtime |
| **Commands** | `:cd`, `:mkdir`, `:touch`, `:open`, `:reload`, `:hidden`, `:pwd`, `:theme` |
| **Config** | TOML config at `~/.config/ranger-rs/config.toml` |
| **Zero-cost theming** | Themes resolved once at compile time; no per-frame lookup cost |

## Keys

| Key | Action |
| --- | --- |
| `j` / `k` | Move down / up |
| `Ctrl-d` / `Ctrl-u` | Move 10 down / up |
| `g` / `G` | Top / bottom |
| `h` / `l` | Parent directory / enter directory or open file |
| `Enter` | Enter directory or open file (editor for text files) |
| `Tab` | Info panel floating popup (size, owner, MIME type, timestamps) |
| `.` | Toggle hidden files |
| `r` | Reload directory |
| `~` | Go home |
| `:` | Command line |
| `/` | Command line (so `/theme` works) |
| `T` | Theme picker |
| `?` | Show key hints |
| `q` | Quit |

## Commands

`:q`, `:cd [path]`, `:mkdir <path>`, `:touch <path>`, `:open`, `:reload`,
`:hidden`, `:pwd`, `:help`, `:theme [name]`, `:themes`, plus any command
registered by a plugin. `:theme` with no name opens the picker; `/theme` works
too.

## Editors

Opening a text file (`l` or `Enter`) suspends the UI and launches the first
editor found on `PATH`, in this order:

1. `nvim`
2. `vim`
3. `vi`
4. `nano`
5. `emacs` (including Doom Emacs)

When only `nano` or `emacs` is available, the status line suggests installing
vim or nvim for the native experience. Binary and media files still go through
`xdg-open` (or `open` on macOS). Detection is cached after the first file open.

## Config

`~/.config/ranger-rs/config.toml` (or `$XDG_CONFIG_HOME/ranger-rs/config.toml`).
See `config.example.toml`.

```toml
theme = "catppuccin-mocha"
show_hidden = false
preview_lines = 200
preview_bytes = 65536

[[scope]]
match = "*.pdf"
command = 'pdftotext -l 2 "$RANGER_RS_PATH" -'
timeout_ms = 1500
```

Scope rules run asynchronously with a timeout, so a slow preview never blocks
the UI. Built-in previews handle directories, text, and binary files.

## Themes

Built in: `default`, `catppuccin-mocha`, `catppuccin-macchiato`,
`catppuccin-frappe`, `catppuccin-latte`, `tokyo-night`, `tokyo-night-storm`,
`tokyo-night-day`, `nord`, `dracula`, `gruvbox`.

Press `T` (or run `/theme`) to open the theme picker. `j`/`k` preview each
theme live, `Enter` saves the choice to `config.toml`, and `Esc` restores the
previous theme. `:theme <name>` jumps straight to a theme and saves it;
`:themes` lists the built-ins.

The picker preserves comments and every other setting in `config.toml`.
Themes are static compile-time palettes resolved once at startup. Rendering
reads the resolved `Style`s directly, so switching themes adds no per-frame
lookups or allocation.

## Plugins

Lua plugins live in `~/.config/ranger-rs/plugins/*.lua` and load at startup.
See `plugins/example.lua`.

```lua
ranger.register_command("hello", function(args)
    ranger.notify("hello " .. args .. " from " .. ranger.cwd())
end)
ranger.register_key("H", "hello")
```

Plugin keys are single characters and do not override built-in keys.

API:

- `ranger.register_command(name, fn)`
- `ranger.register_key(key, command)`
- `ranger.notify(message)`
- `ranger.cd(path)`
- `ranger.mkdir(path)`
- `ranger.touch(path)`
- `ranger.reload()`
- `ranger.toggle_hidden()`
- `ranger.quit()`
- `ranger.cwd()`

Plugin errors are reported in the status line; a broken plugin does not prevent
startup.

## Origin & inspiration

ranger-rs is inspired by:

- **[ranger](https://github.com/ranger/ranger)** — the Vim-powered console
  file manager this project reimplements in Rust. The original Python codebase
  is kept at `/home/adiraj/git/ranger` for reference.
- **[yazi](https://github.com/sxyazi/yazi)** — the floating info panel
  (size, owner, MIME type, timestamps) on `Tab` is Yazi-style.

## Build and run

Requires Rust. Linux and macOS are supported.

```sh
cargo build --release
./target/release/ranger-rs [PATH]
```

Install by copying the binary somewhere on `$PATH`:

```sh
install -m755 target/release/ranger-rs ~/.local/bin/ranger-rs
```

## Tests

```sh
cargo test
```

## License

[MIT License](LICENSE)
