#!/bin/sh
# Install the latest prebuilt ranger-rs release.
#
#   ./install.sh                 install to ~/.local/bin
#   PREFIX=/opt/bin ./install.sh install to a custom prefix
#   ./install.sh --system        install to /usr/local/bin (uses sudo)
#   NO_COLOR=1 ./install.sh      disable colored output
set -eu

usage() {
    sed -n '2,7p' "$0" | sed 's/^# \{0,1\}//'
}

prefix="${PREFIX:-$HOME/.local/bin}"
sudo=""

for arg in "$@"; do
    case "$arg" in
        --system) prefix="/usr/local/bin"; sudo="sudo" ;;
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
