# ranger-rs

A fast, Vim-powered terminal file manager written in Rust.

ranger-rs is a Rust reimplementation of [ranger](https://github.com/ranger/ranger), the Vim-powered console file manager. It keeps ranger's keybindings and ideas but is rebuilt from scratch in Rust — no Python at startup.

Inspired by:
- **ranger** — the original Python file manager
- **yazi** — the floating info panel (`Tab`)

## Get started

```sh
# Clone
git clone git@github.com:adiraj66132/ranger-rs.git
cd ranger-rs

# Build
cargo build --release

# Run
./target/release/ranger-rs /path/to/directory
```

Or install:

```sh
install -m755 target/release/ranger-rs ~/.local/bin/ranger-rs
ranger-rs
```

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
| `:` | Command line |
| `T` | Theme picker |
| `?` | Show key hints |
| `q` | Quit |

## Commands

Type them after `:` or `/`:

| Command | Action |
| --- | --- |
| `:q` | Quit |
| `:cd <path>` | Change directory |
| `:mkdir <name>` | Create directory |
| `:touch <name>` | Create file |
| `:open` | Open selected file |
| `:reload` | Reload directory |
| `:hidden` | Toggle hidden files |
| `:pwd` | Show current path |
| `:theme <name>` | Switch theme |
| `:themes` | List themes |

## Editors

Text files open in the first editor found on `PATH`:

1. `nvim`
2. `vim`
3. `vi`
4. `nano`
5. `emacs`

If only `nano` or `emacs` is available, you'll see a suggestion to install vim or nvim.

Binary files (images, PDFs, etc.) open with `xdg-open` (`open` on macOS).

## Config

Create `~/.config/ranger-rs/config.toml` (see `config.example.toml`):

```toml
theme = "catppuccin-mocha"
show_hidden = false
preview_lines = 200
```

## Themes

Built-in themes: `default`, `catppuccin-mocha`, `catppuccin-macchiato`, `catppuccin-frappe`, `catppuccin-latte`, `tokyo-night`, `tokyo-night-storm`, `tokyo-night-day`, `nord`, `dracula`, `gruvbox`.

Press `T` or run `:theme <name>` to switch. The picker shows a live preview.

## Plugins

Lua plugins go in `~/.config/ranger-rs/plugins/*.lua`. See `plugins/example.lua`.

```lua
ranger.register_command("hello", function(args)
    ranger.notify("hello " .. args .. " from " .. ranger.cwd())
end)
ranger.register_key("H", "hello")
```

## Build status

Tests:

```sh
cargo test
```

License: [MIT](LICENSE)
