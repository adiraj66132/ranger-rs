# Contributing to ranger-rs

Thanks for helping out. ranger-rs is a Rust reimplementation of
[ranger](https://github.com/ranger/ranger), the Vim-powered console file
manager — fast startup, ranger's keybindings, no Python.

## Before you start

- Small fixes and docs: open a pull request directly.
- Larger changes or new features: open an issue first so we can agree on the
  approach before you write code.
- You'll need Rust (stable) and cargo: <https://rustup.rs>.

## Build, run, test

```sh
cargo build
cargo run -- /path/to/directory   # omit the path to start in the current directory
cargo test
cargo build --release             # optimized binary at target/release/ranger-rs
```

CI runs `cargo build` and `cargo test` for every push and pull request to
`master`; both must pass.

## Project layout

| File | Responsibility |
| --- | --- |
| `src/main.rs` | terminal setup and event loop |
| `src/app.rs` | application state, key handling, command parsing and execution |
| `src/ui.rs` | rendering and layout |
| `src/fs.rs` | directory reading and entry metadata |
| `src/preview.rs` | file previews, including external previewers |
| `src/info.rs` | the `Tab` info panel |
| `src/editor.rs` | editor detection and launching |
| `src/theme.rs` | built-in themes |
| `src/config.rs` | config loading |
| `src/plugin.rs` | the Lua plugin API |

## Common changes

- **Keybinding or command** — handle it in `src/app.rs` (`handle_key`,
  `execute_input`). Update the key/command tables in `README.md` and
  `docs/index.html` in the same pull request.
- **Theme** — add a `Theme` constant in `src/theme.rs` and append it to the
  `THEMES` array. The `T` picker and `:theme` pick it up automatically; add the
  name to the theme list in the README.
- **External previewer** — config only (`[[scope]]` rules in `config.toml`); see
  `config.example.toml`. No code change is needed unless the built-in previewer
  misbehaves.
- **Lua API** — `src/plugin.rs`. Keep `plugins/example.lua` working. The plugin
  API is marked "not tested yet" in the README; if you test it, update that note.

## Style

- Format with `cargo fmt` (default settings; there is no `rustfmt.toml`).
- Prefer the standard library or an existing dependency over a new crate. A new
  dependency needs a reason in the pull request.
- Keep unit tests in-module (`#[cfg(test)]`), next to the code they test. New
  non-trivial logic — parsing, path handling, config — should come with a test.
- Match the surrounding code's names, messages, and structure.

## Docs

`README.md` is the source of truth. `docs/index.html` is the wiki published at
<https://adiraj66132.github.io/ranger-rs/> and mirrors the README — update both
when user-facing behavior changes. A `shortcut:` comment in the HTML notes this.

## Pull requests

- One logical change per pull request; keep the diff focused.
- Use lowercase imperative commit messages, matching the history (`add rr
  shortcut`, `fix release job checkout`).
- Say what changed, why, and how you tested it (platform and terminal).

## Releases (maintainers)

Pushing a `v*` tag builds Linux (x86_64/arm64) and macOS (Intel/Apple Silicon)
binaries in GitHub Actions and creates the release. `install.sh` downloads from
`releases/latest`, so every release needs a tag.

## License

ranger-rs is MIT-licensed; contributions are accepted under the same license.
