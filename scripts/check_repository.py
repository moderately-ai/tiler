#!/usr/bin/env python3
"""Run the complete repository-owned contributor and CI validation gate."""

from __future__ import annotations

import argparse
import hashlib
import os
import pwd
import shlex
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PYPROJECT = ROOT / "pyproject.toml"
PYTHON_VERSION = ROOT / ".python-version"
LOCKS = (ROOT / "Cargo.lock", ROOT / "uv.lock")
EXPECTED_PYTEST_PATHS = [
    "scripts/tests",
    "spikes/apple-targets",
    "spikes/embedding",
    "spikes/macro-environment",
    "spikes/numerics/sound_accuracy",
    "spikes/runtime",
    "spikes/shapes/nightly-dependent-static-shapes",
    "spikes/shapes/shape-evidence",
]
# Ambient variables that would change what the gate builds or where it looks,
# stripped so a developer's shell cannot quietly alter the verdict. This is
# hygiene, not a security boundary: anyone who can set these can also edit the
# gate.
HOSTILE_PREFIXES = ("UV_", "PYTEST_", "CARGO_TARGET_", "CLIPPY_")
HOSTILE_EXACT = {
    "CARGO_BUILD_TARGET",
    "CARGO_ENCODED_RUSTFLAGS",
    "PYTHONHOME",
    "PYTHONPATH",
    "RUSTC",
    "RUSTC_WRAPPER",
    "RUSTDOC",
    "RUSTDOCFLAGS",
    "RUSTFLAGS",
    "RUSTFMT",
    "RUSTUP_TOOLCHAIN",
    "SHELLCHECK_OPTS",
    "VIRTUAL_ENV",
}


class GateFailure(RuntimeError):
    """A repository validation invariant was violated."""


def account_home() -> Path:
    """Return the supported Unix account home independently of ambient HOME."""
    return Path(pwd.getpwuid(os.getuid()).pw_dir).resolve()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise GateFailure(message)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def sanitized_environment(source: dict[str, str] | None = None) -> dict[str, str]:
    """Remove ambient controls that can redirect or disable governed tools."""
    environment = dict(os.environ if source is None else source)
    for name in list(environment):
        if name in HOSTILE_EXACT or name.startswith(HOSTILE_PREFIXES):
            del environment[name]
    home = account_home()
    environment.update(
        {
            "HOME": str(home),
            "PATH": os.pathsep.join(
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
            ),
            "PYTEST_DISABLE_PLUGIN_AUTOLOAD": "1",
            "PYTHONHASHSEED": "0",
        }
    )
    return environment


def run(command: list[str], *, environment: dict[str, str], capture: bool = False) -> str:
    print(f"+ {shlex.join(command)}", flush=True)
    result = subprocess.run(
        command,
        cwd=ROOT,
        env=environment,
        check=True,
        text=True,
        stdout=subprocess.PIPE if capture else None,
    )
    return result.stdout if capture else ""


def source_files(suffix: str) -> list[str]:
    """Discover checked-in and pending repository sources outside generated trees."""
    found = []
    for base in (ROOT / "scripts", ROOT / "spikes"):
        for path in base.rglob(f"*{suffix}"):
            relative = path.relative_to(ROOT)
            if path.is_file() and not {"target", "__pycache__"} & set(relative.parts):
                found.append(relative.as_posix())
    if suffix == ".sh" and (ROOT / "deps.sh").is_file():
        found.append("deps.sh")
    return sorted(found)


def shell_dialect(script: str) -> str:
    """Return the shell a checked-in script declares in its shebang."""
    shebang = (ROOT / script).read_text(encoding="utf-8").splitlines()[0]
    for name in ("zsh", "bash"):
        if shebang.endswith(name):
            return name
    return "sh"


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.parse_args(arguments)
    require(
        Path(sys.prefix).resolve() == (ROOT / ".venv").resolve(),
        "gate must run in this checkout's .venv",
    )
    environment = sanitized_environment()
    before = {path: digest(path) for path in LOCKS}

    run(["uv", "--project", str(ROOT), "--no-config", "lock", "--check"], environment=environment)
    run(
        ["uv", "--project", str(ROOT), "--no-config", "sync", "--locked", "--check"],
        environment=environment,
    )
    ruff = str(ROOT / ".venv/bin/ruff")
    python = str(ROOT / ".venv/bin/python")
    run([ruff, "format", "--check"], environment=environment)
    run([ruff, "check"], environment=environment)
    run(
        [python, "-m", "pytest", "-c", str(PYPROJECT), *EXPECTED_PYTEST_PATHS],
        environment=environment,
    )
    run([python, "scripts/docs.py", "validate"], environment=environment)
    run([python, "scripts/check_ci.py"], environment=environment)

    for script in source_files(".sh"):
        shell = shell_dialect(script)
        # zsh is not a ShellCheck dialect; a syntax check is all it gets.
        if shell != "zsh":
            run(
                ["shellcheck", "--severity", "style", "--shell", shell, script],
                environment=environment,
            )
        run([shell, "-n", script], environment=environment)
    run(["ticketsplease", "lint"], environment=environment)
    run([python, "scripts/check_rust.py"], environment=environment)

    after = {path: digest(path) for path in LOCKS}
    require(before == after, "a repository lockfile changed during validation")
    print("complete repository validation passed")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (GateFailure, OSError, subprocess.CalledProcessError) as error:
        print(f"repository validation failed: {error}", file=sys.stderr)
        raise SystemExit(1) from error
