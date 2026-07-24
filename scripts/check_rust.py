#!/usr/bin/env python3
"""Run Tiler's fail-closed Rust workspace gate."""

from __future__ import annotations

import hashlib
import os
import pwd
import re
import shlex
import subprocess
import sys
import tomllib
from pathlib import Path

import check_workspace

ROOT = Path(__file__).resolve().parents[1]
SHAPE_ROOT = ROOT / "spikes/shapes/nightly-dependent-static-shapes"
VISIBILITY_ROOT = ROOT / "spikes/extensions/non-exhaustive-visibility"
SPIKES = ROOT / "spikes"
# A spike Cargo workspace is compiled by this gate when it retains a
# compiler-produced golden artifact — a `trybuild` `.stderr` — that a governed
# document cites, *and* that artifact was captured on the toolchain
# `rust-toolchain.toml` pins. Reproducing such a fixture is the only check that
# compares a retained diagnostic against the code that is supposed to produce
# it; every other predicate over these directories compares a record to a file
# that a source edit can silently invalidate.
GATED_SPIKE_WORKSPACES = (SHAPE_ROOT, VISIBILITY_ROOT)
# A spike whose evidence is deliberately tied to a *different* toolchain cannot
# join the set above. Re-deriving it needs a compiler this gate has no
# authority to install, and re-recording it on the pin would destroy the claim
# the spike exists to make. Its diagnostics stay retained evidence, verified
# against their record rather than reproduced, and the exclusion is named here
# so that "not compiled" is a decision with a reason instead of an omission.
OFF_PIN_SPIKE_WORKSPACES = {ROOT / "spikes/shapes/shape-evidence": "stable 1.89.0"}
LOCKFILES = (
    ROOT / "Cargo.lock",
    SHAPE_ROOT / "Cargo.lock",
    VISIBILITY_ROOT / "Cargo.lock",
)

FORBIDDEN_ENVIRONMENT = {
    "RUSTUP_TOOLCHAIN",
    "RUSTUP_HOME",
    "RUSTFLAGS",
    "RUSTC_BOOTSTRAP",
    "RUSTC",
    "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "RUSTDOC",
    "RUSTDOCFLAGS",
    "RUSTFMT",
    "CLIPPY_DRIVER",
    "CARGO_ENCODED_RUSTFLAGS",
    "CARGO_BUILD_TARGET",
    "CARGO_BUILD_RUSTC",
    "CARGO_BUILD_RUSTC_WRAPPER",
    "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER",
    "CARGO_BUILD_RUSTDOC",
    "CARGO_BUILD_RUSTDOCFLAGS",
    "CARGO_BUILD_RUSTFLAGS",
    "CARGO_INCREMENTAL",
}
FORBIDDEN_CARGO_ENVIRONMENT = re.compile(
    r"^CARGO_(?:ALIAS_|PROFILE_|TARGET_.*_(?:RUNNER|RUSTFLAGS|LINKER)$)"
)
ALLOWED_CARGO_CONFIG = {
    "net": {"retry", "git-fetch-with-cli", "offline"},
    "http": {
        "proxy",
        "timeout",
        "cainfo",
        "check-revoke",
        "multiplexing",
        "user-agent",
        "debug",
        "ssl-version",
        "low-speed-limit",
    },
    "term": {"quiet", "verbose", "color", "hyperlinks", "unicode", "progress"},
    "unstable": {"gc"},
}


class GateFailure(RuntimeError):
    """The Rust gate could not establish its complete success contract."""


def account_home() -> Path:
    """Return the supported Unix account home independently of ambient HOME."""
    return Path(pwd.getpwuid(os.getuid()).pw_dir).resolve()


def digest(path: Path) -> str:
    """Return the exact SHA-256 identity of one required lockfile."""
    try:
        return hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError as error:
        raise GateFailure(f"lockfile.missing: {path}: {error}") from error


def snapshot_lockfiles() -> dict[Path, str]:
    """Snapshot every Cargo lock governed by this gate."""
    return {path: digest(path) for path in LOCKFILES}


def verify_lockfiles(before: dict[Path, str]) -> None:
    """Reject any Cargo command that mutated a governed lockfile."""
    changed = [
        str(path.relative_to(ROOT)) if path.is_relative_to(ROOT) else str(path)
        for path, value in before.items()
        if digest(path) != value
    ]
    if changed:
        raise GateFailure(f"lockfile.mutated: Cargo changed {changed}")


def retained_diagnostics(root: Path) -> list[Path]:
    """Enumerate every checked-in compiler diagnostic retained under a tree."""
    return sorted(
        path for path in root.rglob("*.stderr") if "target" not in path.relative_to(root).parts
    )


def ui_fixtures(workspace: Path) -> list[Path]:
    """Enumerate the `trybuild` cases a spike workspace is expected to exercise."""
    return sorted(
        path
        for path in workspace.rglob("tests/ui/*/*.rs")
        if "target" not in path.relative_to(workspace).parts
    )


def validate_spike_evidence_custody() -> None:
    """Require every retained spike diagnostic to have a decided gate posture.

    A `.stderr` checked in beside a fixture is a positive claim about what a
    compiler emits, and it outlives whatever produced it: the file stays on
    disk unchanged when the source beside it is edited. Only a compilation
    compares the two. This predicate is therefore the admission rule made
    mechanical — a directory that gains such a claim is either compiled here
    or is recorded as off-pin with the toolchain its evidence belongs to, and
    a third, unexamined state cannot be reached by adding a file.
    """
    for workspace, channel in OFF_PIN_SPIKE_WORKSPACES.items():
        if not retained_diagnostics(workspace):
            raise GateFailure(
                f"spike.stale-exclusion: {workspace.relative_to(ROOT)} is recorded as retaining "
                f"{channel} evidence this gate cannot reproduce, but retains no diagnostic; "
                "remove the exclusion"
            )
    governed = (*GATED_SPIKE_WORKSPACES, *OFF_PIN_SPIKE_WORKSPACES)
    ungoverned = [
        str(path.relative_to(ROOT))
        for path in retained_diagnostics(SPIKES)
        if not any(path.is_relative_to(workspace) for workspace in governed)
    ]
    if ungoverned:
        raise GateFailure(
            f"spike.ungoverned-evidence: {ungoverned} retain a compiler diagnostic outside every "
            "workspace this gate compiles or explicitly excludes; admit the workspace to "
            "GATED_SPIKE_WORKSPACES, or record the toolchain its evidence is tied to in "
            "OFF_PIN_SPIKE_WORKSPACES"
        )


def verify_fixture_coverage(workspace: Path, transcript: str) -> None:
    """Reject a spike run that reported success without exercising its fixtures.

    `trybuild` resolves its cases from a glob, so a suite whose fixtures moved
    or whose glob stopped matching reports an ordinary passing test having
    compiled nothing. Nothing else in the run distinguishes that from real
    agreement, and for a workspace whose only content is the fixture set the
    silent loss is total, so the transcript must name every retained case.
    """
    fixtures = ui_fixtures(workspace)
    if not fixtures:
        raise GateFailure(f"spike.fixtures: {workspace.relative_to(ROOT)} retains no trybuild case")
    missing = [
        str(path.relative_to(ROOT))
        for path in fixtures
        if "/".join(path.parts[-4:]) not in transcript
    ]
    if missing:
        raise GateFailure(f"spike.fixtures: {missing} were not exercised by the run")


def hostile_environment(environment: dict[str, str]) -> list[str]:
    """Return ambient controls capable of weakening or redirecting Rust checks."""
    return sorted(
        name
        for name in environment
        if name in FORBIDDEN_ENVIRONMENT or FORBIDDEN_CARGO_ENVIRONMENT.match(name)
    )


def sanitized_environment(environment: dict[str, str]) -> dict[str, str]:
    """Build the exact child environment after rejecting hostile controls."""
    hostile = hostile_environment(environment)
    if hostile:
        raise GateFailure(f"environment.hostile: unset {hostile}")
    home = account_home()
    if "HOME" in environment and Path(environment["HOME"]).resolve() != home:
        raise GateFailure(f"environment.hostile: HOME must identify the account home {home}")
    result = environment.copy()
    result.pop("CARGO_HOME", None)
    result.pop("CARGO_TARGET_DIR", None)
    result["HOME"] = str(home)
    result["CARGO_HOME"] = str(home / ".cargo")
    result["CARGO_TARGET_DIR"] = str(ROOT / "target")
    result["PATH"] = os.pathsep.join(
        str(path)
        for path in (
            ROOT / ".venv/bin",
            home / ".cargo/bin",
            home / ".local/bin",
            Path("/opt/homebrew/bin"),
            Path("/usr/local/bin"),
            Path("/usr/bin"),
            Path("/bin"),
        )
        if path.is_dir()
    )
    return result


def cargo_config_paths(environment: dict[str, str]) -> list[Path]:
    """Enumerate Cargo configuration files visible from every governed workspace."""
    homes = [account_home() / ".cargo"]
    ambient_home = environment.get("CARGO_HOME")
    if ambient_home:
        homes.append(Path(ambient_home).expanduser())
    directories: list[Path] = []
    for workspace in (ROOT, SHAPE_ROOT, VISIBILITY_ROOT):
        for directory in (workspace, *workspace.parents):
            if directory not in directories:
                directories.append(directory)
    candidates = [
        *(
            directory / ".cargo" / name
            for directory in directories
            for name in ("config", "config.toml")
        ),
        *(home / name for home in homes for name in ("config", "config.toml")),
    ]
    unique: list[Path] = []
    seen: set[Path] = set()
    for path in candidates:
        resolved = path.resolve()
        if resolved not in seen and path.is_file():
            seen.add(resolved)
            unique.append(path)
    return unique


def validate_cargo_configs(environment: dict[str, str]) -> None:
    """Reject Cargo configuration capable of changing compiled code or execution."""
    for path in cargo_config_paths(environment):
        try:
            config = tomllib.loads(path.read_text(encoding="utf-8"))
        except (OSError, UnicodeError, tomllib.TOMLDecodeError) as error:
            raise GateFailure(f"cargo-config.invalid: {path}: {error}") from error
        for section, value in config.items():
            allowed_keys = ALLOWED_CARGO_CONFIG.get(section)
            if allowed_keys is None or not isinstance(value, dict):
                raise GateFailure(f"cargo-config.hostile: unsupported [{section}] in {path}")
            extra = set(value) - allowed_keys
            if extra:
                raise GateFailure(
                    f"cargo-config.hostile: unsupported {section} keys {sorted(extra)} in {path}"
                )


def run(
    command: list[str],
    *,
    environment: dict[str, str],
    cwd: Path = ROOT,
    capture: bool = False,
    combined: bool = False,
) -> subprocess.CompletedProcess[str]:
    """Run one checked command in the governed environment.

    `capture` keeps the child's streams apart for a caller that parses one of
    them. `combined` instead folds standard error into standard output and
    tees the merged stream, which is what inspecting a compilation requires:
    Cargo, the test harness, and `trybuild` split their reporting across both
    streams, so a predicate over one stream alone reads a partial transcript.
    Teeing rather than buffering keeps a long cold build's progress visible,
    and the transcript is attached to a failure so that a compile step's
    diagnostics survive being read by the gate.
    """
    print(f"+ {shlex.join(command)}", flush=True)
    if combined:
        return tee(command, environment=environment, cwd=cwd)
    try:
        return subprocess.run(
            command,
            cwd=cwd,
            env=environment,
            check=True,
            capture_output=capture,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError) as error:
        raise GateFailure(f"command.failed: {shlex.join(command)}: {error}") from error


def tee(
    command: list[str],
    *,
    environment: dict[str, str],
    cwd: Path,
) -> subprocess.CompletedProcess[str]:
    """Run one checked command, echoing and returning its merged output."""
    lines: list[str] = []
    try:
        with subprocess.Popen(
            command,
            cwd=cwd,
            env=environment,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
        ) as process:
            if process.stdout is None:
                raise GateFailure(f"command.failed: {shlex.join(command)}: capture pipe is missing")
            for line in process.stdout:
                print(line, end="", flush=True)
                lines.append(line)
    except OSError as error:
        raise GateFailure(f"command.failed: {shlex.join(command)}: {error}") from error
    transcript = "".join(lines)
    if process.returncode != 0:
        raise GateFailure(
            f"command.failed: {shlex.join(command)}: exit status {process.returncode}\n{transcript}"
        )
    return subprocess.CompletedProcess(command, process.returncode, transcript, "")


def rustup_command(rustup: str, toolchain: str, executable: str, *arguments: str) -> list[str]:
    """Select one component from the repository's exact rustup toolchain."""
    return [rustup, "run", toolchain, executable, *arguments]


def cargo_command(rustup: str, toolchain: str, *arguments: str) -> list[str]:
    """Select Cargo from the exact toolchain with supplied arguments."""
    return rustup_command(rustup, toolchain, "cargo", *arguments)


def governed_rustup() -> str:
    """Resolve rustup only from the bootstrap-owned installation location."""
    path = account_home() / ".cargo/bin/rustup"
    if not path.is_file() or not os.access(path, os.X_OK):
        raise GateFailure(f"toolchain.missing: governed rustup is missing: {path}")
    # rustup is a multicall binary: preserving argv[0] as `rustup` is semantic.
    return str(path)


def parse_cfg(output: str) -> set[str]:
    """Parse strict, duplicate-free `rustc --print cfg` output."""
    lines = output.splitlines()
    if not lines or any(not line or line.strip() != line for line in lines):
        raise GateFailure("host.unsupported: malformed rustc cfg output")
    if len(lines) != len(set(lines)):
        raise GateFailure("host.unsupported: duplicate rustc cfg output")
    return set(lines)


def one_cfg(cfg: set[str], prefix: str) -> str:
    """Require one exact valued cfg entry."""
    values = sorted(value for value in cfg if value.startswith(prefix))
    if len(values) != 1:
        raise GateFailure(f"host.unsupported: expected one {prefix}, got {values}")
    return values[0]


def validate_host(cfg: set[str]) -> None:
    """Enforce the target-independent host profile actually supported by Tiler."""
    operating_system = one_cfg(cfg, 'target_os="')
    architecture = one_cfg(cfg, 'target_arch="')
    environment = one_cfg(cfg, 'target_env="')
    pointer_width = one_cfg(cfg, 'target_pointer_width="')
    endian = one_cfg(cfg, 'target_endian="')
    supported_pairs = {
        ('target_os="macos"', 'target_arch="aarch64"'),
        ('target_os="linux"', 'target_arch="x86_64"'),
    }
    if (operating_system, architecture) not in supported_pairs:
        raise GateFailure(
            f"host.unsupported: unproved host pair {operating_system}, {architecture}"
        )
    expected_environment = (
        'target_env=""' if operating_system == 'target_os="macos"' else 'target_env="gnu"'
    )
    if environment != expected_environment:
        raise GateFailure(f"host.unsupported: target environment {environment}")
    if pointer_width != 'target_pointer_width="64"' or endian != 'target_endian="little"':
        raise GateFailure(
            f"host.unsupported: requires 64-bit little-endian, got {pointer_width}, {endian}"
        )
    if 'target_has_atomic="64"' not in cfg:
        raise GateFailure("host.unsupported: tiler-ir requires native 64-bit atomics")


def verify_toolchain(rustup: str, toolchain: str, environment: dict[str, str]) -> None:
    """Require every compiler component and validate the selected host profile."""
    commands = (
        ("rustc", "-vV"),
        ("cargo", "-Vv"),
        ("rustfmt", "--version"),
        ("clippy-driver", "--version"),
        ("rustdoc", "--version"),
    )
    for executable, argument in commands:
        selected = run(
            [rustup, "which", "--toolchain", toolchain, executable],
            environment=environment,
            capture=True,
        ).stdout.strip()
        if toolchain not in selected or not Path(selected).is_file():
            raise GateFailure(
                f"toolchain.identity: {executable} did not resolve inside {toolchain}: {selected!r}"
            )
        result = run(
            rustup_command(rustup, toolchain, executable, argument),
            environment=environment,
            capture=True,
        )
        output = result.stdout.strip()
        if not output:
            raise GateFailure(f"toolchain.component: {executable} returned no version")
    cfg_result = run(
        rustup_command(rustup, toolchain, "rustc", "--print", "cfg"),
        environment=environment,
        capture=True,
    )
    validate_host(parse_cfg(cfg_result.stdout))


def validate_workspace(rustup: str, toolchain: str, environment: dict[str, str]) -> None:
    """Validate exact locked Cargo metadata without trusting later compilation."""
    metadata_result = run(
        cargo_command(
            rustup,
            toolchain,
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--locked",
        ),
        environment=environment,
        capture=True,
    )
    try:
        import json

        metadata = json.loads(metadata_result.stdout)
    except json.JSONDecodeError as error:
        raise GateFailure(f"workspace.metadata: malformed JSON: {error}") from error
    errors = check_workspace.validate_manifest_contract(ROOT, metadata)
    if errors:
        raise GateFailure("workspace.contract:\n" + "\n".join(errors))


def run_gate(environment: dict[str, str] | None = None) -> None:
    """Run the complete Rust gate or raise one typed failure."""
    source_environment = os.environ.copy() if environment is None else environment.copy()
    toolchain = check_workspace.configured_toolchain(ROOT)
    validate_spike_evidence_custody()
    validate_cargo_configs(source_environment)
    child_environment = sanitized_environment(source_environment)
    rustup = governed_rustup()
    locks = snapshot_lockfiles()
    try:
        verify_toolchain(rustup, toolchain, child_environment)
        validate_workspace(rustup, toolchain, child_environment)
        run(
            cargo_command(rustup, toolchain, "fmt", "--all", "--check"),
            environment=child_environment,
        )
        run(
            cargo_command(rustup, toolchain, "check", "--workspace", "--all-targets", "--locked"),
            environment=child_environment,
        )
        run(
            cargo_command(
                rustup,
                toolchain,
                "clippy",
                "--workspace",
                "--all-targets",
                "--locked",
                "--",
                "-D",
                "warnings",
            ),
            environment=child_environment,
        )
        run(
            cargo_command(rustup, toolchain, "test", "--workspace", "--locked"),
            environment=child_environment,
        )
        run(
            cargo_command(
                rustup,
                toolchain,
                "test",
                "--release",
                "--locked",
                "-p",
                "tiler-reference",
                "-p",
                "tiler-compiler",
            ),
            environment=child_environment,
        )
        doc_environment = child_environment | {"RUSTDOCFLAGS": "-D warnings"}
        run(
            cargo_command(rustup, toolchain, "doc", "--workspace", "--no-deps", "--locked"),
            environment=doc_environment,
        )
        shapes = run(
            ["/bin/bash", str(SHAPE_ROOT / "check.sh"), rustup, toolchain],
            environment=child_environment | {"CARGO_TARGET_DIR": str(SHAPE_ROOT / "target")},
            combined=True,
        )
        verify_fixture_coverage(SHAPE_ROOT, shapes.stdout)
        # The recorded procedure of the retained measurement, run under the
        # pin rather than under whichever toolchain a directory search finds.
        visibility = run(
            cargo_command(
                rustup,
                toolchain,
                "test",
                "--locked",
                "--manifest-path",
                str(VISIBILITY_ROOT / "Cargo.toml"),
            ),
            environment=child_environment | {"CARGO_TARGET_DIR": str(VISIBILITY_ROOT / "target")},
            combined=True,
        )
        verify_fixture_coverage(VISIBILITY_ROOT, visibility.stdout)
    finally:
        verify_lockfiles(locks)


def main() -> int:
    try:
        run_gate()
    except (GateFailure, ValueError) as error:
        print(f"Rust gate failed: {error}", file=sys.stderr)
        return 1
    print("Rust gate passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
