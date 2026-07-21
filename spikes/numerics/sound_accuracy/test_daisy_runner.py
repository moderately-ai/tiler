"""Fail-closed tests for the Daisy result adapter."""

from __future__ import annotations

from pathlib import Path

import pytest
from daisy_runner import (
    MAX_FIELD_CHARS,
    MAX_PROVENANCE_BYTES,
    MAX_RESULT_ROWS,
    Unknown,
    analyzer_provenance,
    bounded_analyzer_provenance,
    main,
    parse_results,
    run_profile,
)


def test_parse_results_requires_complete_finite_rows(tmp_path: Path) -> None:
    result = tmp_path / "result.csv"
    result.write_text("first;1e-6;;[-1, 1];4\nsecond;2e-6;3e-6;[0, 2];5\n")

    parsed = parse_results(result, ("first", "second"), "test-profile")

    assert parsed["first"]["absolute_error"] == "1e-6"
    assert parsed["second"]["relative_error"] == "3e-6"


@pytest.mark.parametrize(
    ("contents", "reason"),
    [
        ("first;not-a-number;;[-1, 1];4\n", "analyzer_diagnostic"),
        ("first;1e-6;;[1, -1];4\n", "analyzer_diagnostic"),
        ("first;1e-6;;[-1, 1];4\nfirst;1e-6;;[-1, 1];4\n", "analyzer_diagnostic"),
        ("", "missing_result"),
    ],
)
def test_parse_results_fails_closed(tmp_path: Path, contents: str, reason: str) -> None:
    result = tmp_path / "result.csv"
    result.write_text(contents)

    with pytest.raises(Unknown) as raised:
        parse_results(result, ("first",), "test-profile")

    assert raised.value.reason == reason


def test_parse_results_bounds_rows_and_fields(tmp_path: Path) -> None:
    result = tmp_path / "result.csv"
    result.write_text("\n" * (MAX_RESULT_ROWS + 1))
    with pytest.raises(Unknown) as rows:
        parse_results(result, ("first",), "test-profile")
    assert rows.value.reason == "analyzer_resource_limit"

    result.write_text(f"first;{'1' * (MAX_FIELD_CHARS + 1)};;[-1, 1];4\n")
    with pytest.raises(Unknown) as field:
        parse_results(result, ("first",), "test-profile")
    assert field.value.reason == "analyzer_diagnostic"


def test_analyzer_provenance_binds_launcher_classpath_java_and_inputs(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    classpath = tmp_path / "classes"
    classpath.mkdir()
    (classpath / "Main.class").write_bytes(b"class bytes")
    launcher = tmp_path / "daisy"
    launcher.write_text(f'#!/bin/sh\nSCALACLASSPATH="{classpath}"\n')
    java = tmp_path / "java"
    java.write_bytes(b"java bytes")
    monkeypatch.setattr("daisy_runner.shutil.which", lambda _name: str(java))
    for name in ("daisy_runner.py", "scalar_regions.scala", "mixed-precision.txt"):
        (tmp_path / name).write_text(name)

    provenance = analyzer_provenance(tmp_path, tmp_path, "a" * 40)

    assert provenance["classpath_files"] == 1
    assert provenance["source_revision"] == "a" * 40
    assert provenance["launcher_sha256"]
    assert provenance["java_sha256"]


def test_analyzer_provenance_rejects_missing_classpath_entry(tmp_path: Path) -> None:
    (tmp_path / "daisy").write_text(f'#!/bin/sh\nSCALACLASSPATH="{tmp_path / "missing.jar"}"\n')

    with pytest.raises(Unknown) as raised:
        analyzer_provenance(tmp_path, tmp_path, "a" * 40)

    assert raised.value.reason == "analyzer_provenance"


def test_analyzer_provenance_rejects_sparse_file_before_reading(tmp_path: Path) -> None:
    oversized = tmp_path / "oversized.jar"
    with oversized.open("wb") as target:
        target.truncate(MAX_PROVENANCE_BYTES + 1)
    (tmp_path / "daisy").write_text(f'#!/bin/sh\nSCALACLASSPATH="{oversized}"\n')

    with pytest.raises(Unknown, match="provenance budget") as raised:
        analyzer_provenance(tmp_path, tmp_path, "a" * 40)

    assert raised.value.reason == "analyzer_provenance"


def test_run_profile_kills_timed_out_process_group(tmp_path: Path) -> None:
    output = tmp_path / "output"
    output.mkdir()
    daisy = tmp_path / "daisy"
    daisy.write_text("#!/bin/sh\nsleep 30\n")
    daisy.chmod(0o755)

    with pytest.raises(Unknown, match="exceeded 1 seconds") as raised:
        run_profile(tmp_path, tmp_path, 1, "test-profile", ("first",))

    assert raised.value.reason == "analyzer_timeout"


def write_fake_daisy(root: Path, body: str) -> None:
    """Write a tiny executable that stands in for Daisy's generated launcher."""
    daisy = root / "daisy"
    daisy.write_text(f"#!/usr/bin/env python3\n{body}")
    daisy.chmod(0o755)


def test_run_profile_accepts_one_complete_silent_result(tmp_path: Path) -> None:
    (tmp_path / "output").mkdir()
    write_fake_daisy(
        tmp_path,
        """import pathlib, sys
result = next(arg.split('=', 1)[1] for arg in sys.argv if arg.startswith('--results-csv='))
(pathlib.Path('output') / result).write_text('first;1e-6;;[-1, 1];4\\n')
""",
    )

    result = run_profile(tmp_path, tmp_path, 5, "test-profile", ("first",))

    assert result["first"]["absolute_error"] == "1e-6"


@pytest.mark.parametrize(
    ("body", "reason"),
    [
        ("print('nominal failure hidden by tee')\n", "analyzer_diagnostic"),
        ("raise SystemExit(3)\n", "analyzer_diagnostic"),
        ("pass\n", "missing_result"),
    ],
)
def test_run_profile_fails_closed_through_external_process(
    tmp_path: Path, body: str, reason: str
) -> None:
    (tmp_path / "output").mkdir()
    write_fake_daisy(tmp_path, body)

    with pytest.raises(Unknown) as raised:
        run_profile(tmp_path, tmp_path, 5, "test-profile", ("first",))

    assert raised.value.reason == reason


def test_provenance_collection_has_wall_clock_deadline(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    monkeypatch.setattr("daisy_runner.PROVENANCE_TIMEOUT_SECONDS", 0.001)

    with pytest.raises(Unknown) as raised:
        bounded_analyzer_provenance(tmp_path, tmp_path, "a" * 40)

    assert raised.value.reason == "analyzer_timeout"


def test_main_records_checked_revision_and_deadline(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
    tmp_path: Path,
) -> None:
    revision = "a" * 40
    monkeypatch.setattr(
        "daisy_runner.run_profile",
        lambda _root, _spike, _timeout, _profile, expected, _extra: {
            name: {
                "absolute_error": "0",
                "relative_error": "",
                "real_range": "[0, 1]",
            }
            for name in expected
        },
    )
    monkeypatch.setattr(
        "daisy_runner.bounded_analyzer_provenance",
        lambda _root, _spike, checked_revision: {
            "name": "Daisy",
            "source_revision": checked_revision,
        },
    )
    monkeypatch.setattr(
        "daisy_runner.checked_revision", lambda _root, checked_revision: checked_revision
    )

    status = main([str(tmp_path), str(tmp_path), "17", revision])

    assert status == 0
    output = capsys.readouterr().out
    assert f'"source_revision": "{revision}"' in output
    assert '"timeout_seconds_per_profile": 17' in output
