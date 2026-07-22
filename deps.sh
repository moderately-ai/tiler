#!/usr/bin/env bash
# Bootstrap or verify Tiler's supported development hosts.
#
# Usage:
#   ./deps.sh          Install missing dependencies and sync the uv environment.
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

toml_integer() {
    local file="$1" section="$2" key="$3"
    awk -v section="$section" -v key="$key" '
        BEGIN { active = (section == "") }
        $0 == "[" section "]" { active = 1; next }
        /^\[/ { active = 0 }
        active && $0 ~ "^" key " = [0-9]+$" {
            count += 1
            value = $0
            sub("^" key " = ", "", value)
        }
        END {
            if (count != 1) exit 1
            print value
        }
    ' "$file"
}

REQUIRED_RUST_TOOLCHAIN="$(toml_string "$ROOT_DIR/rust-toolchain.toml" toolchain channel)" \
    || { printf 'invalid Rust toolchain authority\n' >&2; exit 1; }
uv_requirement="$(toml_string "$ROOT_DIR/pyproject.toml" tool.uv required-version)" \
    || { printf 'invalid uv version authority\n' >&2; exit 1; }
if [[ "$uv_requirement" =~ ^==([0-9]+\.[0-9]+\.[0-9]+)$ ]]; then
    REQUIRED_UV_VERSION="${BASH_REMATCH[1]}"
else
    printf 'uv required-version must be an exact ==X.Y.Z pin\n' >&2
    exit 1
fi
tool_versions_schema="$(toml_integer "$ROOT_DIR/tool-versions.toml" '' schema_version)" \
    || { printf 'invalid tool version authority schema\n' >&2; exit 1; }
[ "$tool_versions_schema" = '1' ] \
    || { printf 'unsupported tool version authority schema: %s\n' "$tool_versions_schema" >&2; exit 1; }
REQUIRED_TICKETSPLEASE_VERSION="$(
    toml_string "$ROOT_DIR/tool-versions.toml" '' ticketsplease
)" || { printf 'invalid ticketsplease version authority\n' >&2; exit 1; }
REQUIRED_TICKETSPLEASE_REV="$(
    toml_string "$ROOT_DIR/tool-versions.toml" '' ticketsplease_rev
)" || { printf 'invalid ticketsplease revision authority\n' >&2; exit 1; }
[[ "$REQUIRED_TICKETSPLEASE_REV" =~ ^[0-9a-f]{40}$ ]] \
    || { printf 'invalid ticketsplease revision authority\n' >&2; exit 1; }
readonly REQUIRED_RUST_TOOLCHAIN REQUIRED_UV_VERSION REQUIRED_TICKETSPLEASE_VERSION
readonly REQUIRED_TICKETSPLEASE_REV
REQUIRED_PYTHON="$(tr -d '[:space:]' < "$ROOT_DIR/.python-version")"
[[ "$REQUIRED_PYTHON" =~ ^3\.11(\.[0-9]+)?$ ]] \
    || { printf 'invalid Python version authority\n' >&2; exit 1; }
readonly REQUIRED_PYTHON
unset uv_requirement tool_versions_schema

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

OS_FAMILY=''
LINUX_ID=''
case "$(uname -s)" in
    Darwin)
        OS_FAMILY='macos'
        ;;
    Linux)
        OS_FAMILY='debian'
        [ -r /etc/os-release ] || die 'Linux host lacks /etc/os-release'
        # shellcheck disable=SC1091
        . /etc/os-release
        LINUX_ID="${ID:-unknown}"
        case " ${ID:-} ${ID_LIKE:-} " in
            *' debian '*|*' ubuntu '*) ;;
            *) die "unsupported Linux distribution: ${ID:-unknown}; only Debian/Ubuntu are supported" ;;
        esac
        ;;
    *)
        die "unsupported operating system: $(uname -s); use macOS or Debian/Ubuntu"
        ;;
esac
readonly OS_FAMILY LINUX_ID

ensure_brew_packages() {
    have brew || die 'Homebrew is required on macOS; install it from https://brew.sh'
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

apt_command() {
    if [ "$(id -u)" -eq 0 ]; then
        apt-get "$@"
    else
        have sudo || die 'sudo is required to install Debian/Ubuntu packages'
        sudo apt-get "$@"
    fi
}

ensure_apt_packages() {
    have apt-get || die 'apt-get is required on Debian/Ubuntu'
    local package
    local packages=(build-essential ca-certificates curl git pkg-config shellcheck time zsh)
    local missing=()
    for package in "${packages[@]}"; do
        dpkg-query -W -f='${Status}' "$package" 2>/dev/null | grep -q 'ok installed' || missing+=("$package")
    done
    if [ "${#missing[@]}" -eq 0 ]; then
        ok 'Debian/Ubuntu development packages present'
        return
    fi
    if [ "$CHECK_ONLY" -eq 1 ]; then
        die "missing Debian/Ubuntu packages: ${missing[*]}"
    fi
    info "installing Debian/Ubuntu packages: ${missing[*]}"
    apt_command update
    apt_command install -y "${missing[@]}"
}

ensure_system_packages() {
    info "system packages ($OS_FAMILY${LINUX_ID:+/$LINUX_ID})"
    case "$OS_FAMILY" in
        macos) ensure_brew_packages ;;
        debian) ensure_apt_packages ;;
    esac
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
            --profile minimal --component clippy,rustfmt
    fi
    rustup toolchain list | grep -q "^${REQUIRED_RUST_TOOLCHAIN}" \
        || die "Rust $REQUIRED_RUST_TOOLCHAIN is not installed"
    for component in clippy rustfmt; do
        rustup component list --toolchain "$REQUIRED_RUST_TOOLCHAIN" --installed \
            | grep -qE "^${component}(-|$)" \
            || die "$component is missing for Rust $REQUIRED_RUST_TOOLCHAIN"
    done
    ok "$(rustup run "$REQUIRED_RUST_TOOLCHAIN" rustc --version)"
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
    [ "$(tkt --version | awk '{print $2}')" = "$REQUIRED_TICKETSPLEASE_VERSION" ] \
        || die "tkt does not resolve to ticketsplease $REQUIRED_TICKETSPLEASE_VERSION"
}

ensure_ticketsplease() {
    info 'ticketsplease'
    local current=''
    local revision_receipt="$HOME/.local/share/tiler/ticketsplease-revision"
    local installed_revision=''
    export PATH="$HOME/.local/bin:$PATH"
    hash -r
    if have ticketsplease; then
        current="$(ticketsplease --version | awk '{print $2}')"
    fi
    if [ -f "$revision_receipt" ]; then
        installed_revision="$(tr -d '[:space:]' < "$revision_receipt")"
    fi
    if [ "$current" != "$REQUIRED_TICKETSPLEASE_VERSION" ] \
        || [ "$installed_revision" != "$REQUIRED_TICKETSPLEASE_REV" ]; then
        if [ "$CHECK_ONLY" -eq 1 ]; then
            die "ticketsplease ${current:-missing} revision ${installed_revision:-unknown}; run ./deps.sh"
        fi
        rustup run "$REQUIRED_RUST_TOOLCHAIN" cargo install \
            --git https://github.com/moderately-ai/ticketsplease \
            --rev "$REQUIRED_TICKETSPLEASE_REV" \
            --locked --force --root "$HOME/.local" ticketsplease-cli
        mkdir -p "$(dirname "$revision_receipt")"
        local receipt_temp
        receipt_temp="$(mktemp "${revision_receipt}.XXXXXX")"
        printf '%s\n' "$REQUIRED_TICKETSPLEASE_REV" > "$receipt_temp"
        mv -f "$receipt_temp" "$revision_receipt"
        hash -r
    fi

    current="$(ticketsplease --version | awk '{print $2}')"
    [ "$current" = "$REQUIRED_TICKETSPLEASE_VERSION" ] \
        || die "ticketsplease $current does not match $REQUIRED_TICKETSPLEASE_VERSION"
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
    ok "ticketsplease $current"
}

uv_version() {
    uv --version | awk '{print $2}'
}

install_uv_standalone() {
    have curl || die 'curl is required to install uv'
    curl -LsSf "https://astral.sh/uv/${REQUIRED_UV_VERSION}/install.sh" | sh
    export PATH="$HOME/.local/bin:$PATH"
}

sanitize_uv_environment() {
    local name
    while IFS= read -r name; do
        unset "$name"
    done < <(compgen -A variable UV_)
}

ensure_uv() {
    info 'uv'
    export PATH="$HOME/.local/bin:$PATH"
    if ! have uv; then
        if [ "$CHECK_ONLY" -eq 1 ]; then
            die 'uv is missing; run ./deps.sh to install it'
        fi
        if [ "$OS_FAMILY" = 'macos' ]; then
            brew install uv
        else
            install_uv_standalone
        fi
    fi

    local current
    current="$(uv_version)"
    if [ "$current" != "$REQUIRED_UV_VERSION" ]; then
        if [ "$CHECK_ONLY" -eq 1 ]; then
            die "uv $current does not match required $REQUIRED_UV_VERSION"
        fi
        if ! uv self update "$REQUIRED_UV_VERSION"; then
            install_uv_standalone
        fi
        current="$(uv_version)"
        [ "$current" = "$REQUIRED_UV_VERSION" ] \
            || die "uv $current does not match $REQUIRED_UV_VERSION after installation"
    fi
    ok "uv $current"
}

ensure_python_environment() {
    info 'locked Python development environment'
    cd "$ROOT_DIR"
    uv --project "$ROOT_DIR" --no-config lock --check
    if [ "$CHECK_ONLY" -eq 1 ]; then
        uv --project "$ROOT_DIR" --no-config sync --locked --check
        [ -x .venv/bin/python ] || die 'the project environment is missing; run ./deps.sh'
        [ -x .venv/bin/pytest ] || die 'pytest is missing from the locked environment; run ./deps.sh'
        [ -x .venv/bin/ruff ] || die 'Ruff is missing from the locked environment; run ./deps.sh'
        ok "$(.venv/bin/python --version) with the locked development dependencies"
    else
        uv --project "$ROOT_DIR" --no-config python install "$REQUIRED_PYTHON"
        uv --project "$ROOT_DIR" --no-config sync --locked
        ok "$(uv --project "$ROOT_DIR" --no-config run --locked python --version) with the locked development dependencies"
    fi
}

ensure_metal_toolchain() {
    [ "$OS_FAMILY" = 'macos' ] || {
        warn 'Metal toolchain is macOS-only; Linux supports target-independent development'
        return
    }
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
    rustup run "$REQUIRED_RUST_TOOLCHAIN" cargo --version
    ticketsplease --version
    uv --version
    if [ "$CHECK_ONLY" -eq 1 ]; then
        .venv/bin/python --version
        .venv/bin/python -c 'import mpmath; print("mpmath", mpmath.__version__)'
        .venv/bin/pytest --version
        .venv/bin/ruff --version
    else
        uv --project "$ROOT_DIR" --no-config run --locked python --version
        uv --project "$ROOT_DIR" --no-config run --locked python \
            -c 'import mpmath; print("mpmath", mpmath.__version__)'
        uv --project "$ROOT_DIR" --no-config run --locked pytest --version
        uv --project "$ROOT_DIR" --no-config run --locked ruff --version
    fi
}

main() {
    cd "$ROOT_DIR"
    sanitize_uv_environment
    local mode='install'
    [ "$CHECK_ONLY" -eq 1 ] && mode='check-only'
    printf 'tiler dependencies (%s)\n' "$mode"
    ensure_system_packages
    ensure_rust
    ensure_ticketsplease
    ensure_uv
    ensure_python_environment
    ensure_metal_toolchain
    verify_tools
    printf '%sdevelopment dependencies are ready%s\n' "$C_GREEN" "$C_RESET"
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
    main "$@"
fi
