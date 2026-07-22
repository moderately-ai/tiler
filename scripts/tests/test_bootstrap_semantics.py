import os
import subprocess
import tomllib
from pathlib import Path

ROOT = Path(__file__).parents[2]
DEPS = ROOT / "deps.sh"


def run_bash(script: str, *, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["bash", "-c", script],
        cwd=ROOT,
        env=env,
        check=False,
        capture_output=True,
        text=True,
    )


def test_bootstrap_reads_versions_from_repository_authorities():
    result = run_bash(
        f"source {DEPS!s}; "
        'printf \'%s|%s|%s|%s|%s\' "$REQUIRED_RUST_TOOLCHAIN" "$REQUIRED_UV_VERSION" '
        '"$REQUIRED_TICKETSPLEASE_VERSION" "$REQUIRED_TICKETSPLEASE_REV" "$REQUIRED_PYTHON"'
    )

    rust = tomllib.loads((ROOT / "rust-toolchain.toml").read_text(encoding="utf-8"))
    project = tomllib.loads((ROOT / "pyproject.toml").read_text(encoding="utf-8"))
    tools = tomllib.loads((ROOT / "tool-versions.toml").read_text(encoding="utf-8"))
    expected = "|".join(
        (
            rust["toolchain"]["channel"],
            project["tool"]["uv"]["required-version"].removeprefix("=="),
            tools["ticketsplease"],
            tools["ticketsplease_rev"],
            (ROOT / ".python-version").read_text(encoding="utf-8").strip(),
        )
    )

    assert result.returncode == 0, result.stderr
    assert result.stdout == expected


def test_bootstrap_clears_inherited_uv_controls():
    env = os.environ | {
        "UV_PROJECT": "/untrusted/project",
        "UV_NO_PROJECT": "1",
        "UV_NO_SYNC": "1",
    }
    result = run_bash(
        f"source {DEPS!s}; sanitize_uv_environment; "
        "[[ -z ${UV_PROJECT+x} && -z ${UV_NO_PROJECT+x} && -z ${UV_NO_SYNC+x} ]]",
        env=env,
    )

    assert result.returncode == 0, result.stderr


def write_ticketsplease(path: Path) -> None:
    path.write_text(
        "#!/bin/sh\nif [ \"${1:-}\" = --version ]; then printf 'ticketsplease 0.11.0\\n'; fi\n",
        encoding="utf-8",
    )
    path.chmod(0o755)


def test_bootstrap_repairs_a_stale_managed_tkt_alias(tmp_path: Path):
    home = tmp_path / "home"
    managed_bin = home / ".local" / "bin"
    managed_bin.mkdir(parents=True)
    (managed_bin / "tkt").symlink_to("missing-ticketsplease")
    mock_bin = tmp_path / "bin"
    mock_bin.mkdir()
    write_ticketsplease(mock_bin / "ticketsplease")
    env = os.environ | {"HOME": str(home), "PATH": f"{mock_bin}:{os.environ['PATH']}"}

    result = run_bash(f"source {DEPS!s}; ensure_tkt_alias", env=env)

    assert result.returncode == 0, result.stderr
    assert (managed_bin / "tkt").resolve() == (mock_bin / "ticketsplease").resolve()


def test_bootstrap_rejects_a_user_owned_tkt_collision(tmp_path: Path):
    home = tmp_path / "home"
    managed_bin = home / ".local" / "bin"
    managed_bin.mkdir(parents=True)
    write_ticketsplease(managed_bin / "tkt")
    mock_bin = tmp_path / "bin"
    mock_bin.mkdir()
    write_ticketsplease(mock_bin / "ticketsplease")
    env = os.environ | {"HOME": str(home), "PATH": f"{mock_bin}:{os.environ['PATH']}"}

    result = run_bash(f"source {DEPS!s}; ensure_tkt_alias", env=env)

    assert result.returncode == 1
    assert "user-owned" in result.stderr
