#!/usr/bin/env bash
# Bootstrap or verify Tiler's supported development hosts.
#
# Usage:
#   ./deps.sh          Install missing dependencies.
#   ./deps.sh --check  Verify without changing the host or project environment.
#   ./deps.sh --help   Show this help.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly ROOT_DIR

toml_string() {
    local file="$1" section="$2" key="$3"
    awk -v section="$section" -v key="$key" '
        BEGIN { active = (section == "") }
        $0 == "[" section "]" { active = 1; next }
        /^\[/ { active = 0 }
        active && $0 ~ "^" key " = \"[^\"]+\"$" {
            count += 1
            value = $0
            sub("^" key " = \"", "", value)
            sub("\"$", "", value)
        }
        END {
            if (count != 1) exit 1
            print value
        }
    ' "$file"
}

toml_string_array() {
    local file="$1" section="$2" key="$3"
    awk -v section="$section" -v key="$key" '
        BEGIN { active = (section == "") }
        $0 == "[" section "]" { active = 1; next }
        /^\[/ { active = 0 }
        active && $0 ~ "^" key " = \\[" {
            count += 1
            value = $0
            sub("^" key " = \\[", "", value)
            sub("\\]$", "", value)
            gsub("[\" ]", "", value)
            gsub(",", " ", value)
        }
        END {
            if (count != 1 || value == "") exit 1
            print value
        }
    ' "$file"
}

REQUIRED_RUST_TOOLCHAIN="$(toml_string "$ROOT_DIR/rust-toolchain.toml" toolchain channel)" \
    || { printf 'invalid Rust toolchain authority\n' >&2; exit 1; }
# Read rather than restate: the manifest is the sole component authority, and a
# copy here would install a different set than the gate requires.
REQUIRED_RUST_COMPONENTS="$(toml_string_array "$ROOT_DIR/rust-toolchain.toml" toolchain components)" \
    || { printf 'invalid Rust component authority\n' >&2; exit 1; }
readonly REQUIRED_RUST_TOOLCHAIN

CHECK_ONLY=0
for argument in "$@"; do
    case "$argument" in
        --check) CHECK_ONLY=1 ;;
        -h|--help)
            sed -n '2,8p' "$0" | sed 's/^# //; s/^#$//'
            exit 0
            ;;
        *)
            printf 'unknown argument: %s (run with --help)\n' "$argument" >&2
            exit 2
            ;;
    esac
done
readonly CHECK_ONLY

if [ -t 1 ]; then
    readonly C_GREEN=$'\033[32m'
    readonly C_RED=$'\033[31m'
    readonly C_YELLOW=$'\033[33m'
    readonly C_BLUE=$'\033[34m'
    readonly C_RESET=$'\033[0m'
else
    readonly C_GREEN=''
    readonly C_RED=''
    readonly C_YELLOW=''
    readonly C_BLUE=''
    readonly C_RESET=''
fi

info() { printf '%s==>%s %s\n' "$C_BLUE" "$C_RESET" "$1"; }
ok() { printf '  %s[ok]%s %s\n' "$C_GREEN" "$C_RESET" "$1"; }
warn() { printf '  %s[warn]%s %s\n' "$C_YELLOW" "$C_RESET" "$1"; }
die() { printf '  %s[fail]%s %s\n' "$C_RED" "$C_RESET" "$1" >&2; exit 1; }
have() { command -v "$1" >/dev/null 2>&1; }

[ "$(uname -s)" = 'Darwin' ] \
    || die "unsupported operating system: $(uname -s); Tiler develops on macOS"

ensure_system_packages() {
    info 'system packages (macos)'
    have brew || die 'Homebrew is required; install it from https://brew.sh'
    local package
    local missing=()
    for package in pkg-config shellcheck; do
        brew list --formula "$package" >/dev/null 2>&1 || missing+=("$package")
    done
    if [ "${#missing[@]}" -eq 0 ]; then
        ok 'Homebrew development packages present'
        return
    fi
    if [ "$CHECK_ONLY" -eq 1 ]; then
        die "missing Homebrew formulae: ${missing[*]}"
    fi
    info "installing Homebrew formulae: ${missing[*]}"
    brew install "${missing[@]}"
}

load_cargo_path() {
    if [ -f "$HOME/.cargo/env" ]; then
        # shellcheck disable=SC1091
        . "$HOME/.cargo/env"
    fi
}

ensure_rust() {
    info 'Rust toolchain'
    load_cargo_path
    if ! have rustup; then
        if [ "$CHECK_ONLY" -eq 1 ]; then
            die 'rustup is missing; run ./deps.sh to install it'
        fi
        have curl || die 'curl is required to install rustup'
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
            | sh -s -- -y --profile minimal --default-toolchain none
        load_cargo_path
    fi

    if [ "$CHECK_ONLY" -eq 0 ]; then
        rustup toolchain install "$REQUIRED_RUST_TOOLCHAIN" \
            --profile minimal --component "${REQUIRED_RUST_COMPONENTS// /,}"
    fi
    rustup toolchain list | grep -q "^${REQUIRED_RUST_TOOLCHAIN}" \
        || die "Rust $REQUIRED_RUST_TOOLCHAIN is not installed"
    for component in $REQUIRED_RUST_COMPONENTS; do
        rustup component list --toolchain "$REQUIRED_RUST_TOOLCHAIN" --installed \
            | grep -qE "^${component}(-|$)" \
            || die "$component is missing for Rust $REQUIRED_RUST_TOOLCHAIN"
    done
    ok "$(rustup run "$REQUIRED_RUST_TOOLCHAIN" rustc --version)"
}

ensure_nextest() {
    info 'cargo-nextest'
    # `make test` runs the suite through nextest, so a checkout without it fails
    # the gate rather than merely losing a convenience. Presence is the whole
    # requirement; no version is pinned or asserted.
    if ! have cargo-nextest; then
        [ "$CHECK_ONLY" -eq 1 ] && die 'cargo-nextest is missing; run ./deps.sh'
        rustup run "$REQUIRED_RUST_TOOLCHAIN" cargo install cargo-nextest --locked
    fi
    have cargo-nextest || die 'cargo-nextest is still missing after installation'
    ok "$(cargo-nextest nextest --version)"
}

ensure_tkt_alias() {
    local managed_bin="$HOME/.local/bin"
    local managed_alias="$managed_bin/tkt"
    export PATH="$managed_bin:$PATH"
    hash -r
    if [ -e "$managed_alias" ] && [ ! -L "$managed_alias" ]; then
        die "$managed_alias is user-owned and cannot be replaced with the managed tkt alias"
    fi
    local ticketsplease_path
    ticketsplease_path="$(command -v ticketsplease)"
    if [ ! -L "$managed_alias" ] || [ "$(readlink "$managed_alias")" != "$ticketsplease_path" ]; then
        if [ "$CHECK_ONLY" -eq 1 ]; then
            die 'the managed tkt alias is missing or stale; run ./deps.sh to repair it'
        fi
        mkdir -p "$managed_bin"
        local temporary_alias="$managed_alias.tmp.$$"
        ln -s "$ticketsplease_path" "$temporary_alias"
        mv -f "$temporary_alias" "$managed_alias"
        hash -r
    fi
    [ "$(command -v tkt)" = "$managed_alias" ] \
        || die "tkt does not resolve through the managed alias $managed_alias"
}

ensure_ticketsplease() {
    info 'ticketsplease'
    # Presence is the whole requirement. No version or revision is pinned, so an
    # install here tracks the upstream default branch.
    export PATH="$HOME/.local/bin:$PATH"
    hash -r
    if ! have ticketsplease; then
        [ "$CHECK_ONLY" -eq 1 ] && die 'ticketsplease is missing; run ./deps.sh'
        rustup run "$REQUIRED_RUST_TOOLCHAIN" cargo install \
            --git https://github.com/moderately-ai/ticketsplease \
            --locked --force --root "$HOME/.local" ticketsplease-cli
        hash -r
    fi
    have ticketsplease || die 'ticketsplease is still missing after installation'
    ensure_tkt_alias

    if [ "$CHECK_ONLY" -eq 0 ]; then
        ticketsplease skill sync >/dev/null
        ticketsplease skill install --repo "$ROOT_DIR" --harness codex --format json >/dev/null
        ticketsplease skill install --repo "$ROOT_DIR" --harness claude --format json >/dev/null
    fi
    [ -r "$ROOT_DIR/.agents/skills/ticketsplease/SKILL.md" ] \
        || die 'the Codex/cross-tool ticketsplease skill link is missing; run ./deps.sh'
    [ -r "$ROOT_DIR/.claude/skills/ticketsplease/SKILL.md" ] \
        || die 'the Claude ticketsplease skill link is missing; run ./deps.sh'
    ticketsplease doctor --repo "$ROOT_DIR" --format json >/dev/null
    ok "$(ticketsplease --version)"
}

ensure_metal_toolchain() {
    info 'Apple Metal toolchain'
    have xcode-select || die 'xcode-select is missing; install Xcode from Apple'
    if ! xcode-select -p >/dev/null 2>&1; then
        if [ "$CHECK_ONLY" -eq 0 ]; then
            xcode-select --install >/dev/null 2>&1 || true
        fi
        die 'Apple developer tools are not selected; complete Xcode installation and rerun ./deps.sh'
    fi
    have xcrun || die 'xcrun is missing from the selected Apple developer tools'
    xcrun -sdk macosx --find metal >/dev/null 2>&1 \
        || die 'Metal compiler is unavailable; install/select full Xcode and its Metal toolchain'
    xcrun -sdk macosx --find metallib >/dev/null 2>&1 \
        || die 'metallib is unavailable; install/select full Xcode and its Metal toolchain'
    ok "Metal SDK $(xcrun -sdk macosx --show-sdk-version)"
}

verify_tools() {
    info 'tool versions'
    shellcheck --version | head -n 2
    make --version | head -n 1
    rustup run "$REQUIRED_RUST_TOOLCHAIN" cargo --version
    # Unqualified, to show that `rust-toolchain.toml` resolves the pin without a
    # wrapper. Everything in the Makefile relies on exactly this.
    cargo --version
    cargo-nextest nextest --version
    ticketsplease --version
}

main() {
    cd "$ROOT_DIR"
    local mode='install'
    [ "$CHECK_ONLY" -eq 1 ] && mode='check-only'
    printf 'tiler dependencies (%s)\n' "$mode"
    ensure_system_packages
    ensure_rust
    ensure_nextest
    ensure_ticketsplease
    ensure_metal_toolchain
    verify_tools
    printf '%sdevelopment dependencies are ready%s\n' "$C_GREEN" "$C_RESET"
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
    main "$@"
fi
