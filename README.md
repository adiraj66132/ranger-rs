# ranger-rs

A fast, Vim-powered terminal file manager written in Rust. It is a clean-room
take on ranger's ideas, not a port: own config format, own plugin runtime, no
Python at startup.

The original Python ranger lives at `/home/adiraj/git/ranger` for reference and
is intentionally untouched.

## Status

Early working version. Navigation, previews, commands, scope rules, and Lua
plugins work. File operations beyond mkdir/touch, tabs, bookmarks, tags, search,
VCS integration, and image previews are not implemented yet.

Requires Linux or macOS (it uses Unix permissions and `xdg-open`/`open`).

## Build and run

```sh
cargo build --release
./target/release/ranger-rs [PATH]
```

Install by copying the binary somewhere on `$PATH`:

```sh
install -m755 target/release/ranger-rs ~/.local/bin/ranger-rs
```

## Keys

| Key | Action |
| --- | --- |
| `j` / `k` | Move down / up |
| `Ctrl-d` / `Ctrl-u` | Move 10 down / up |
| `g` / `G` | Top / bottom |
| `h` / `l` | Parent directory / enter directory or open file |
| `Enter` | Enter directory or open file |
| `.` | Toggle hidden files |
| `r` | Reload directory |
| `~` | Go home |
| `:` | Command line |
| `?` | Show key hints |
| `q` | Quit |

## Commands

`:q`, `:cd [path]`, `:mkdir <path>`, `:touch <path>`, `:open`, `:reload`,
`:hidden`, `:pwd`, `:help`, plus any command registered by a plugin.

## Config

`~/.config/ranger-rs/config.toml` (or `$XDG_CONFIG_HOME/ranger-rs/config.toml`).
See `config.example.toml`.

```toml
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

## Tests

```sh
cargo test
```
