#!/bin/sh
# Install the latest prebuilt ranger-rs release.
#
#   ./install.sh                 install to ~/.local/bin
#   PREFIX=/opt/bin ./install.sh install to a custom prefix
#   ./install.sh --system        install to /usr/local/bin (uses sudo)
#   ./install.sh --nvim          also install the bundled Neovim config
#   ./install.sh --no-nvim       skip the Neovim config, no prompt
#   NO_COLOR=1 ./install.sh      disable colored output
set -eu

usage() {
    sed -n '2,9p' "$0" | sed 's/^# \{0,1\}//'
}

prefix="${PREFIX:-$HOME/.local/bin}"
sudo=""
nvim_choice="ask"

for arg in "$@"; do
    case "$arg" in
        --system) prefix="/usr/local/bin"; sudo="sudo" ;;
        --nvim) nvim_choice="yes" ;;
        --no-nvim) nvim_choice="no" ;;
        -h|--help) usage; exit 0 ;;
        *) printf 'install.sh: unknown option: %s\n' "$arg" >&2; exit 1 ;;
    esac
done

color=0
if [ -t 1 ] && [ -z "${NO_COLOR+x}" ] && [ "${TERM:-dumb}" != "dumb" ]; then
    color=1
fi
if [ "$color" = 1 ]; then
    accent=$(printf '\033[38;5;208m')
    muted=$(printf '\033[38;5;244m')
    green=$(printf '\033[38;5;114m')
    red=$(printf '\033[38;5;203m')
    bold=$(printf '\033[1m')
    reset=$(printf '\033[0m')
else
    accent= muted= green= red= bold= reset=
fi

utf8=0
case "${LC_ALL:-${LC_CTYPE:-${LANG:-}}}" in
    *UTF-8*|*utf8*|*UTF8*) utf8=1 ;;
esac
if [ "$utf8" = 1 ]; then
    arrow="→"
    check="✓"
    cross="×"
    rule="────────────────────────────────────────"
else
    arrow="->"
    check="ok"
    cross="x"
    rule="----------------------------------------"
fi

step() {
    printf '  %s%s%s %-10s %s\n' "$accent" "$arrow" "$reset" "$1" "$2"
}

ok() {
    printf '  %s%s%s %-10s %s\n' "$green" "$check" "$reset" "$1" "$2"
}

note_line() {
    printf '  %s%s%s\n' "$muted" "$*" "$reset"
}

# Install the bundled Neovim config, keeping the previous one safe.
install_nvim_config() {
    src=$1
    files=$(find "$src" -type f | wc -l | tr -d ' ')
    backup=""
    if [ -d "$nvim_dir" ] && [ -n "$(ls -A "$nvim_dir" 2>/dev/null)" ]; then
        backup="$nvim_dir.bak.$(date +%Y%m%d-%H%M%S)"
        if ! mv "$nvim_dir" "$backup"; then
            return 1
        fi
    fi
    if ! mkdir -p "$nvim_dir" || ! cp -R "$src/." "$nvim_dir/"; then
        if [ -n "$backup" ]; then
            note_line "previous config is at $backup"
        fi
        return 1
    fi
    ok "installed" "Neovim config ($files files)"
    if [ -n "$backup" ]; then
        note_line "previous config saved to $backup"
    fi
    return 0
}

die() {
    printf '  %s%s%s %-10s %s\n' "$red" "$cross" "$reset" "error" "$*" >&2
    exit 1
}

as_root() {
    if [ -n "$sudo" ]; then
        "$sudo" "$@"
    else
        "$@"
    fi
}

human_size() {
    bytes=$1
    if [ "$bytes" -ge 1048576 ]; then
        printf '%s.%s MB' "$((bytes / 1048576))" "$(((bytes % 1048576) * 10 / 1048576))"
    elif [ "$bytes" -ge 1024 ]; then
        printf '%s KB' "$((bytes / 1024))"
    else
        printf '%s B' "$bytes"
    fi
}

ask() {
    [ -t 0 ] || return 1
    printf '  %s?%s %s [y/N] ' "$accent" "$reset" "$1"
    if ! read -r reply; then
        printf '\n'
        return 1
    fi
    case "$reply" in
        [yY]|[yY][eE][sS]) return 0 ;;
        *) return 1 ;;
    esac
}

# Locate the bundled Neovim config: a checkout keeps it next to this script,
# a standalone run has to fetch the same source archive the release came from.
fetch_config() {
    spec=$1
    slot=$2
    if ! curl -fsSL -o "$tmp/nvim-$slot.tar.gz" \
        "https://github.com/adiraj66132/ranger-rs/archive/refs/$spec.tar.gz"; then
        return 1
    fi
    dir="$tmp/nvim-$slot"
    mkdir -p "$dir"
    if ! tar -xzf "$tmp/nvim-$slot.tar.gz" -C "$dir"; then
        return 1
    fi
    for candidate in "$dir"/ranger-rs-*/nvim; do
        if [ -f "$candidate/init.lua" ]; then
            printf '%s\n' "$candidate"
            return 0
        fi
    done
    return 1
}

nvim_source() {
    if [ -n "$script_dir" ] && [ -f "$script_dir/nvim/init.lua" ]; then
        printf '%s\n' "$script_dir/nvim"
        return 0
    fi
    # The tagged archive only carries the config from the release that added
    # it, so fall back to master for older tags.
    if [ -n "$version" ] && fetch_config "tags/v$version" tag; then
        return 0
    fi
    fetch_config "heads/master" master
}

case "$(uname -s):$(uname -m)" in
    Linux:x86_64|Linux:amd64)  target=x86_64-unknown-linux-gnu;  platform="linux x86_64" ;;
    Linux:aarch64|Linux:arm64) target=aarch64-unknown-linux-gnu; platform="linux arm64" ;;
    Darwin:x86_64)             target=x86_64-apple-darwin;       platform="macOS x86_64" ;;
    Darwin:arm64)              target=aarch64-apple-darwin;      platform="macOS arm64" ;;
    *) die "unsupported platform: $(uname -s) $(uname -m)" ;;
esac

command -v curl >/dev/null 2>&1 || die "curl not found"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# Empty when the script arrived on stdin, so a stray ./nvim in the current
# directory can never be mistaken for the bundled config.
script_dir=""
if [ -f "$0" ]; then
    script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
fi

printf '\n'
printf '  %s%sranger-rs%s\n' "$bold" "$accent" "$reset"
printf '  %sa fast, vim-powered file manager%s\n' "$muted" "$reset"
printf '  %s%s%s\n\n' "$muted" "$rule" "$reset"

step "platform" "$platform"
step "download" "ranger-rs-$target.tar.gz"

url="https://github.com/adiraj66132/ranger-rs/releases/latest/download/ranger-rs-$target.tar.gz"
if ! curl -fsSL -o "$tmp/ranger-rs.tar.gz" "$url"; then
    die "download failed: $url"
fi
bytes=$(wc -c < "$tmp/ranger-rs.tar.gz" | tr -d ' ')
ok "fetched" "$(human_size "$bytes")"

if ! tar -xzf "$tmp/ranger-rs.tar.gz" -C "$tmp"; then
    die "could not unpack the archive"
fi
[ -f "$tmp/ranger-rs" ] || die "archive did not contain ranger-rs"

version=""
if output=$("$tmp/ranger-rs" --version 2>/dev/null); then
    case "$output" in
        "ranger-rs "*) version=${output#ranger-rs } ;;
    esac
fi

step "install" "$prefix"
if ! as_root install -d "$prefix"; then
    die "could not create $prefix"
fi
if ! as_root install -m755 "$tmp/ranger-rs" "$prefix/ranger-rs"; then
    die "could not install ranger-rs"
fi
if ! as_root ln -sf ranger-rs "$prefix/rr"; then
    die "could not create the rr shortcut"
fi
ok "installed" "ranger-rs${version:+ $version}"

# --- optional: Neovim config ------------------------------------------------
nvim_dir="$HOME/.config/nvim"
install_nvim=0
case "$nvim_choice" in
    yes) install_nvim=1 ;;
    no) install_nvim=0 ;;
    *)
        if [ -t 0 ]; then
            printf '\n'
            if ask "install the bundled Neovim config to $nvim_dir?"; then
                install_nvim=1
            fi
        else
            printf '\n'
            note_line "note  Neovim config not installed (run with --nvim)"
        fi
        ;;
esac

if [ "$install_nvim" = 1 ]; then
    printf '\n'
    step "editor" "$nvim_dir"
    if src=$(nvim_source); then
        if ! install_nvim_config "$src"; then
            note_line "could not install the Neovim config"
        elif ! command -v nvim >/dev/null 2>&1; then
            note_line "nvim is not on PATH; install Neovim to use this config"
        fi
    else
        note_line "could not find the bundled Neovim config"
    fi
fi

printf '\n'
printf '  run %srr%s or %sranger-rs%s to start\n' "$accent" "$reset" "$accent" "$reset"

case ":$PATH:" in
    *":$prefix:"*) ;;
    *)
        printf '\n'
        printf '  %snote%s %s is not on PATH\n' "$muted" "$reset" "$prefix"
        printf '       add: %sexport PATH="%s:$PATH"%s\n' "$muted" "$prefix" "$reset"
        ;;
esac

printf '\n'
