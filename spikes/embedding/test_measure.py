"""Failure-mode tests for the embedding measurement harness."""

from __future__ import annotations

import importlib.util
import shutil
import sys
import time
from pathlib import Path

import pytest

MODULE_PATH = Path(__file__).with_name("measure.py")
REPOSITORY_ROOT = MODULE_PATH.parents[2]
LEGACY_FIXTURE = REPOSITORY_ROOT / "docs/research/embedding/measurements/2026-07-20-macos-arm64"
SPEC = importlib.util.spec_from_file_location("embedding_measure", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
measure = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = measure
SPEC.loader.exec_module(measure)


def test_parse_time_requires_both_metrics(tmp_path: Path) -> None:
    timing = tmp_path / "time.txt"
    timing.write_text("        0.42 real\n  123456 maximum resident set size\n")

    assert measure.parse_time(timing) == (0.42, 123456)

    timing.write_text("        0.42 real\n")
    with pytest.raises(measure.MeasurementFailure, match="peak-RSS"):
        measure.parse_time(timing)


def test_parse_macho_sections_rejects_missing_required_section() -> None:
    with pytest.raises(measure.MeasurementFailure, match="unparseable"):
        measure.parse_macho_sections("not a size report")

    with pytest.raises(measure.MeasurementFailure, match="__TEXT,__const"):
        measure.parse_macho_sections("Segment __TEXT: 10\nSection __text: 10\n")


def test_run_logged_enforces_deadline(tmp_path: Path) -> None:
    with pytest.raises(measure.MeasurementFailure, match="exceeded 1s"):
        measure.run_logged(
            [sys.executable, "-c", "import time; time.sleep(30)"],
            tmp_path / "stdout",
            tmp_path / "stderr",
            1,
        )


def test_run_logged_enforces_expired_overall_deadline(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setattr(measure, "HARNESS_DEADLINE", time.monotonic() - 1)
    with pytest.raises(measure.MeasurementFailure, match="overall harness deadline"):
        measure.run_logged(
            [sys.executable, "-c", "pass"],
            tmp_path / "stdout",
            tmp_path / "stderr",
            30,
        )


def test_overall_timeout_interrupts_non_subprocess_work() -> None:
    with pytest.raises(measure.MeasurementFailure, match="overall deadline"):
        measure.overall_timeout_handler(0, None)


def test_run_logged_enforces_output_limit(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setattr(measure, "MAX_CAPTURE_BYTES", 64)
    with pytest.raises(measure.MeasurementFailure, match="exceed.*64"):
        measure.run_logged(
            [sys.executable, "-c", "print('x' * 1000)"],
            tmp_path / "stdout",
            tmp_path / "stderr",
            10,
        )


def test_validate_rows_rejects_incomplete_success() -> None:
    with pytest.raises(measure.MeasurementFailure, match="missing metrics"):
        measure.validate_rows([{"wall_seconds": 0.1}], freshness=False)


def test_verify_retained_rejects_missing_fixture(tmp_path: Path) -> None:
    with pytest.raises(measure.MeasurementFailure, match="missing retained fixture"):
        measure.verify_retained(tmp_path)


def test_verify_retained_rejects_digest_mismatch(tmp_path: Path) -> None:
    fixture = tmp_path / "legacy"
    shutil.copytree(LEGACY_FIXTURE, fixture)
    metadata = fixture / "metadata.json"
    metadata.write_text(metadata.read_text() + "\n")

    with pytest.raises(measure.MeasurementFailure, match="digest mismatch"):
        measure.verify_retained(fixture)


def test_source_identity_changes_with_generated_input(tmp_path: Path) -> None:
    (tmp_path / "src").mkdir()
    source = tmp_path / "src/lib.rs"
    source.write_text("pub fn value() -> u8 { 1 }\n")
    (tmp_path / "Cargo.toml").write_text("[package]\nname='probe'\nversion='0.0.0'\n")
    first = measure.source_identity(tmp_path)

    source.write_text("pub fn value() -> u8 { 2 }\n")
    second = measure.source_identity(tmp_path)

    assert first["sha256"] != second["sha256"]


def test_evidence_identity_excludes_work_and_completion(tmp_path: Path) -> None:
    (tmp_path / "raw").mkdir()
    (tmp_path / "raw/stdout").write_text("retained")
    (tmp_path / "work").mkdir()
    (tmp_path / "work/generated.rs").write_text("temporary")
    (tmp_path / "complete.json").write_text("stale")

    records = measure.evidence_identity(tmp_path)

    assert [record["path"] for record in records] == ["raw/stdout"]
