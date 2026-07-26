#!/usr/bin/env python3
"""Run Tiler's fail-closed Rust workspace gate."""

from __future__ import annotations

import hashlib
import os
import pwd
import re
import shlex
import shutil
import subprocess
import sys
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
# Packages that only mean anything on an Apple host: one links Metal, the other
# drives `xcrun`. They are skipped rather than made to compile everywhere,
# because a Metal proof has no non-Apple behaviour worth building — gating their
# contents would spread a host condition through code that is Apple-specific
# from top to bottom. Every target-independent crate is still built and tested
# on both profiles, which is what the target-independence claim rests on.
APPLE_ONLY_PACKAGES = ("tiler-prototype-compile", "tiler-prototype-run")
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


def _ignored(paths: list[Path]) -> set[str]:
    """The subset of `paths` git ignores, empty outside a git work tree.

    Return codes: 0 means some paths are ignored and are printed; 1 means none
    are; 128 (outside a repository) or a missing git means the question could
    not be asked, so nothing is treated as ignored and the filesystem stands as
    the only authority — which is what the synthetic trees the integrity tests
    build rely on.
    """
    if not paths:
        return set()
    try:
        completed = subprocess.run(
            ["git", "check-ignore", "--stdin", "-z"],
            cwd=ROOT,
            input="\0".join(str(path) for path in paths),
            capture_output=True,
            text=True,
            check=False,
        )
    except OSError:
        return set()
    if completed.returncode not in (0, 1):
        return set()
    return {name for name in completed.stdout.split("\0") if name}


def retained_diagnostics(root: Path) -> list[Path]:
    """Enumerate every checked-in compiler diagnostic retained under a tree.

    A retained diagnostic is checked-in evidence. A regenerable `.stderr` a
    harness writes under a gitignored run-output directory — `local-results/`,
    say — is not, so a gitignored path is excluded; a compiler scratch tree
    under `target/` is excluded the same way. Outside a git work tree nothing is
    known-ignored and every `.stderr` is retained.
    """
    candidates = sorted(
        path for path in root.rglob("*.stderr") if "target" not in path.relative_to(root).parts
    )
    ignored = _ignored(candidates)
    return [path for path in candidates if str(path) not in ignored]


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
    child_environment = sanitized_environment(source_environment)
    rustup = shutil.which("rustup", path=child_environment["PATH"])
    if rustup is None:
        raise GateFailure("toolchain.missing: rustup is not on the governed PATH")
    locks = snapshot_lockfiles()
    skipped = [] if sys.platform == "darwin" else APPLE_ONLY_PACKAGES
    try:
        validate_workspace(rustup, toolchain, child_environment)
        excluded = [argument for package in skipped for argument in ("--exclude", package)]
        if skipped:
            print(f"+ skipping Apple-only packages on {sys.platform}: {', '.join(skipped)}")
        run(
            cargo_command(rustup, toolchain, "fmt", "--all", "--check"),
            environment=child_environment,
        )
        run(
            cargo_command(
                rustup,
                toolchain,
                "check",
                "--workspace",
                "--all-targets",
                "--locked",
                *excluded,
            ),
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
                *excluded,
                "--",
                "-D",
                "warnings",
            ),
            environment=child_environment,
        )
        run(
            cargo_command(rustup, toolchain, "test", "--workspace", "--locked", *excluded),
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
            cargo_command(
                rustup, toolchain, "doc", "--workspace", "--no-deps", "--locked", *excluded
            ),
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
