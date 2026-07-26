"""Gate checks for the retained off-pin shape-evidence diagnostics.

The repository gate collects these: `spikes/shapes/shape-evidence` is in the
canonical pytest `testpaths`, so the record check below runs on every
`scripts/check_repository.py` invocation without a `pyproject.toml` change. Run
them alone with

    uv run --with pytest pytestspikes/shapes/shape-evidence/test_shape_evidence_record.py

The first test is the assertion the gate depends on. Everything after it exists
because a predicate that never refuses anything is not a check: each case copies
the spike, applies one realistic corruption, and requires the exact refusal that
corruption should produce. They run on copies, so nothing here can damage the
retained evidence.
"""

from __future__ import annotations

import importlib.util
import json
import re
import shutil
import sys
from collections.abc import Callable
from pathlib import Path

import pytest

SPIKE = Path(__file__).resolve().parent
MODULE_PATH = SPIKE / "verify_evidence.py"
SPEC = importlib.util.spec_from_file_location("shape_evidence_verify", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
verify = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = verify
SPEC.loader.exec_module(verify)

FAIL = verify.FAIL_DIR
PASS = verify.PASS_DIR
# Naming the inventory here rather than deriving it is the point: the record and
# the directory are checked against each other, so only a third, independent
# statement rejects a case deleted from both at once.
EXPECTED_FAIL_CASES = [
    f"{FAIL}/axis_out_of_range.rs",
    f"{FAIL}/duplicate_axes.rs",
    f"{FAIL}/exact_shape_mismatch.rs",
    f"{FAIL}/fixed_rank_mismatch.rs",
    f"{FAIL}/forge.rs",
    f"{FAIL}/implement_evidence.rs",
]
EXPECTED_PASS_CASES = [
    f"{PASS}/owned_static_shape.rs",
    f"{PASS}/refine_and_weaken.rs",
    f"{PASS}/typed_axes.rs",
]


def test_retained_diagnostics_match_their_record() -> None:
    summary = verify.verify_shape_evidence(SPIKE, verify.read_pinned_channel())

    assert summary["compile_fail_cases"] == EXPECTED_FAIL_CASES
    assert summary["compile_pass_cases"] == EXPECTED_PASS_CASES
    assert summary["recorded_channel"] == verify.declared_measure_toolchain(SPIKE)
    assert summary["recorded_channel"] != summary["pinned_channel"]


def spike_copy(tmp_path: Path) -> Path:
    """Copy the parts of the spike this check reads, leaving build products out."""
    copy = tmp_path / "shape-evidence"
    shutil.copytree(SPIKE, copy, ignore=shutil.ignore_patterns("target", "raw", "bin"))
    return copy


def rewrite_once(path: Path, old: str, new: str) -> None:
    """Replace one unambiguous occurrence, failing if the target text has moved."""
    text = path.read_text(encoding="utf-8")
    assert text.count(old) == 1, f"tamper target {old!r} is not unique in {path.name}"
    path.write_text(text.replace(old, new), encoding="utf-8")


def append_text(path: Path, text: str) -> None:
    """Append to an existing retained file, failing if it is absent."""
    assert path.is_file(), f"tamper target {path.name} is absent"
    path.write_text(path.read_text(encoding="utf-8") + text, encoding="utf-8")


def rename_recorded_diagnostic_code(root: Path) -> None:
    """Re-record a diagnostic's message text while leaving its recorded code behind.

    This is the mutation `diagnostic_codes` exists for. The retained message and
    the record's `first_line` agree afterwards, so the separately recorded code
    is the only field left that still says which error the claim is about.
    """
    rewrite_once(root / FAIL / "implement_evidence.stderr", "error[E0277]", "error[E0091]")
    rewrite_once(verify.sole_record(root), "error[E0277]", "error[E0091]")


def duplicate_record(root: Path) -> None:
    """Add a second record, which would leave two claims over the same bytes."""
    record = verify.sole_record(root)
    shutil.copyfile(record, record.with_name("second.json"))


def move_record_onto_the_pin(root: Path) -> None:
    """Re-record the evidence as if it belonged to the repository's own pin."""
    pinned = verify.read_pinned_channel()
    rewrite_once(verify.sole_record(root), '"channel": "1.89.0"', f'"channel": "{pinned}"')


def drop_claim_field(field: str) -> Callable[[Path], None]:
    """Remove one recorded field from the first compile-fail claim.

    A record can decay by asserting less rather than by asserting something
    wrong, and a defaulted field is how that stays invisible.
    """

    def mutate(root: Path) -> None:
        path = verify.sole_record(root)
        record = json.loads(path.read_text(encoding="utf-8"))
        claim = next(entry for entry in record["claims"] if entry["outcome"] == "fails")
        del claim[field]
        path.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")

    return mutate


def empty_claim_field(field: str) -> Callable[[Path], None]:
    """Empty one recorded field on the first compile-fail claim without removing it.

    The quieter half of `drop_claim_field`, and the one a type check alone lets
    through: `[]` is a list of strings, so a claim emptied rather than deleted
    goes on satisfying every structural predicate while asserting nothing. This
    is the form the gated sibling spike already refused and this one did not.
    """

    def mutate(root: Path) -> None:
        path = verify.sole_record(root)
        record = json.loads(path.read_text(encoding="utf-8"))
        claim = next(entry for entry in record["claims"] if entry["outcome"] == "fails")
        claim[field] = []
        path.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")

    return mutate


TAMPERS: tuple[tuple[str, Callable[[Path], object], str], ...] = (
    (
        "no record at all",
        lambda root: shutil.rmtree(root / "results"),
        "must retain exactly one diagnostics record, found 0",
    ),
    ("a second record", duplicate_record, "must retain exactly one diagnostics record, found 2"),
    (
        "a record under another schema",
        lambda root: rewrite_once(verify.sole_record(root), verify.SCHEMA, "wrong/v1"),
        f"is not a {verify.SCHEMA} record",
    ),
    (
        "evidence re-recorded onto the repository pin",
        move_record_onto_the_pin,
        "now records the repository pin",
    ),
    (
        "a record naming a toolchain the harness does not",
        lambda root: rewrite_once(
            verify.sole_record(root), '"channel": "1.89.0"', '"channel": "1.90.0"'
        ),
        "records toolchain 1.90.0, but measure.py declares TOOLCHAIN 1.89.0",
    ),
    (
        "a harness moved to another toolchain",
        lambda root: rewrite_once(
            root / "measure.py", 'TOOLCHAIN = "1.89.0"', 'TOOLCHAIN = "1.90"'
        ),
        "records toolchain 1.89.0, but measure.py declares TOOLCHAIN 1.90",
    ),
    (
        "a manifest moved to another release series",
        lambda root: rewrite_once(
            root / "Cargo.toml", 'rust-version = "1.89"', 'rust-version = "1.90"'
        ),
        "but Cargo.toml declares rust-version 1.90",
    ),
    (
        "a README documenting another selector",
        lambda root: append_text(root / "README.md", "\nOr run `cargo +1.90.0 test`.\n"),
        "README documents cargo selectors",
    ),
    (
        "a lockfile moved to another trybuild",
        lambda root: rewrite_once(
            root / "Cargo.lock", 'version = "1.0.118"', 'version = "1.0.119"'
        ),
        "but Cargo.lock pins 1.0.119",
    ),
    (
        "a changed diagnostic first line",
        lambda root: rewrite_once(
            root / FAIL / "fixed_rank_mismatch.stderr",
            "error[E0308]: mismatched types",
            "error[E0308]: incompatible types",
        ),
        "now reports 'error[E0308]: incompatible types'",
    ),
    (
        "a message re-recorded under a different diagnostic code",
        rename_recorded_diagnostic_code,
        "now reports diagnostic codes ['E0091'], recorded ['E0277']",
    ),
    (
        "a claim that dropped its diagnostic codes",
        drop_claim_field("diagnostic_codes"),
        "tests/ui/fail/axis_out_of_range.rs records no diagnostic_codes list",
    ),
    (
        "a claim that dropped its required fragments",
        drop_claim_field("required_fragments"),
        "tests/ui/fail/axis_out_of_range.rs records no required_fragments list",
    ),
    (
        "a claim that emptied its diagnostic codes",
        empty_claim_field("diagnostic_codes"),
        "tests/ui/fail/axis_out_of_range.rs records an empty diagnostic_codes list",
    ),
    (
        "a claim that emptied its required fragments",
        empty_claim_field("required_fragments"),
        "tests/ui/fail/axis_out_of_range.rs records an empty required_fragments list",
    ),
    (
        "a dropped required fragment",
        lambda root: rewrite_once(
            root / FAIL / "implement_evidence.stderr", 'is a "sealed trait"', 'is a "closed trait"'
        ),
        "no longer reports '`ShapeEvidence` is a \"sealed trait\"'",
    ),
    (
        "a forbidden fragment appearing",
        lambda root: append_text(
            root / FAIL / "fixed_rank_mismatch.stderr",
            "   = note: Exact<Matrix> was also considered\n",
        ),
        "now reports 'Exact<', which it must not",
    ),
    (
        "a deleted retained diagnostic",
        lambda root: (root / FAIL / "forge.stderr").unlink(),
        "fixture retains no diagnostic: tests/ui/fail/forge.rs",
    ),
    (
        "a fixture present without a record",
        lambda root: (root / FAIL / "unrecorded.rs").write_text("fn main() {}\n", encoding="utf-8"),
        "fixtures ['tests/ui/fail/unrecorded.rs'] are present without a record",
    ),
    (
        "a record whose fixture is missing",
        lambda root: (root / FAIL / "forge.rs").unlink(),
        "recorded fixture is missing: tests/ui/fail/forge.rs",
    ),
    (
        "a compiling fixture claiming a failure",
        lambda root: (root / PASS / "typed_axes.stderr").write_text(
            "error: invented\n", encoding="utf-8"
        ),
        "a compiling fixture must retain no diagnostic: tests/ui/pass/typed_axes.rs",
    ),
    (
        "an orphaned retained diagnostic",
        lambda root: (root / FAIL / "orphan.stderr").write_text(
            "error: invented\n", encoding="utf-8"
        ),
        "orphaned retained diagnostic orphan.stderr",
    ),
    (
        "an edit to the source a diagnostic quotes",
        lambda root: rewrite_once(
            root / "src" / "lib.rs",
            "//! Stable-Rust model of non-authoritative shape evidence.",
            "//! Edited after the diagnostics were captured.",
        ),
        "inputs ['src/lib.rs'] no longer match their recorded digest",
    ),
    (
        "a record carrying no input digests",
        lambda root: rewrite_once(verify.sole_record(root), '"inputs": {', '"unchecked_inputs": {'),
        "records no input digests",
    ),
)


@pytest.mark.parametrize(
    ("mutate", "expected"),
    [pytest.param(mutate, expected, id=name) for name, mutate, expected in TAMPERS],
)
def test_tampering_is_rejected(
    tmp_path: Path, mutate: Callable[[Path], object], expected: str
) -> None:
    root = spike_copy(tmp_path)
    verify.verify_shape_evidence(root, verify.read_pinned_channel())

    mutate(root)

    with pytest.raises(verify.EvidenceFailure, match=re.escape(expected)):
        verify.verify_shape_evidence(root, verify.read_pinned_channel())
