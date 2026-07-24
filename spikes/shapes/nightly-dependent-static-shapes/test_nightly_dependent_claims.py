"""Gate checks for the gated dependent-static-shape fixtures' recorded claims.

The repository gate collects these: `spikes/shapes/nightly-dependent-static-shapes`
is already in the canonical pytest `testpaths`, so the record check below runs on
every `scripts/check_repository.py` invocation without a `pyproject.toml` change.
Run them alone with

    uv run --locked pytest \
      spikes/shapes/nightly-dependent-static-shapes/test_nightly_dependent_claims.py

The first test is the assertion the gate depends on. Everything after it exists
because a predicate that never refuses anything is not a check: each case copies
the spike, applies one realistic corruption, and requires the exact refusal that
corruption should produce. They run on copies, so nothing here can damage the
retained evidence, and none of them invokes Cargo — reproduction is the Rust
sub-gate's job and this is the semantic half beside it.
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
MODULE_PATH = SPIKE / "verify_claims.py"
SPEC = importlib.util.spec_from_file_location("nightly_dependent_verify_claims", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
verify = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = verify
SPEC.loader.exec_module(verify)

FAIL = verify.FAIL_DIR
PASS = verify.PASS_DIR
# Naming the inventory here rather than deriving it is the point: the record and
# the directory are checked against each other, so only a third, independent
# statement rejects a case deleted from both at once. `conformance/tests/ui.rs`
# names the same inventory for the compiled half.
EXPECTED_FAIL_CASES = [
    f"{FAIL}/forge.rs",
    f"{FAIL}/implement_evidence.rs",
    f"{FAIL}/rank_array_length.rs",
    f"{FAIL}/unequal_shapes.rs",
]
EXPECTED_PASS_CASES = [
    f"{PASS}/cross_crate_identity.rs",
    f"{PASS}/ranks.rs",
]


def test_retained_claims_match_their_fixtures() -> None:
    summary = verify.verify_dependent_shape_claims(SPIKE, verify.read_pinned_channel())

    assert summary["compile_fail_cases"] == EXPECTED_FAIL_CASES
    assert summary["compile_pass_cases"] == EXPECTED_PASS_CASES
    assert summary["decisions"] == ["ADR-0067"]
    # The gated inversion of the off-pin rule: this spike's evidence is only
    # evidence because the gate reproduces it on exactly this compiler.
    assert summary["recorded_channel"] == summary["pinned_channel"]
    assert verify.read_pinned_channel() in verify.declared_measure_toolchains(SPIKE)


def spike_copy(tmp_path: Path) -> Path:
    """Copy the parts of the spike this check reads, leaving build products out."""
    copy = tmp_path / "nightly-dependent-static-shapes"
    shutil.copytree(SPIKE, copy, ignore=shutil.ignore_patterns("target", "raw"))
    return copy


def rewrite_once(path: Path, old: str, new: str) -> None:
    """Replace one unambiguous occurrence, failing if the target text has moved."""
    text = path.read_text(encoding="utf-8")
    assert text.count(old) == 1, f"tamper target {old!r} is not unique in {path.name}"
    path.write_text(text.replace(old, new), encoding="utf-8")


def rewrite_all(path: Path, old: str, new: str) -> None:
    """Replace every occurrence, failing if the target text is absent."""
    text = path.read_text(encoding="utf-8")
    assert old in text, f"tamper target {old!r} is absent from {path.name}"
    path.write_text(text.replace(old, new), encoding="utf-8")


def append_text(path: Path, text: str) -> None:
    """Append to an existing retained file, failing if it is absent."""
    assert path.is_file(), f"tamper target {path.name} is absent"
    path.write_text(path.read_text(encoding="utf-8") + text, encoding="utf-8")


def edit_record(root: Path, mutate: Callable[[dict[str, object]], None]) -> None:
    """Apply one structural edit to the retained record."""
    path = verify.sole_record(root)
    record = json.loads(path.read_text(encoding="utf-8"))
    mutate(record)
    path.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")


def first_failing_claim(record: dict[str, object]) -> dict[str, object]:
    """Return the record's first compile-fail claim."""
    return next(entry for entry in record["claims"] if entry["outcome"] == "fails")


def duplicate_record(root: Path) -> None:
    """Add a second record, which would leave two attributions of the same fixtures."""
    record = verify.sole_record(root)
    shutil.copyfile(record, record.with_name("second.json"))


def weaken_fixture_and_overwrite_its_diagnostic(root: Path) -> None:
    """Reproduce the exact commit this record exists to refuse.

    A fixture is weakened until it fails for an unrelated reason, `TRYBUILD=overwrite`
    rewrites the `.stderr` to whatever the compiler now says, and the recorded first
    line is refreshed to agree. Fixture, diagnostic, and record are then mutually
    consistent, so compilation passes and only the separately recorded code objects.

    Both replacements below were produced by the governed compiler, not written by
    hand: at `f57e23b` this fixture edit followed by `TRYBUILD=overwrite` on
    `nightly-2026-07-19` emitted exactly this E0425 diagnostic into the `.stderr`
    and the suite reported `test result: ok`.
    """
    (root / FAIL / "rank_array_length.rs").write_text(
        "use nightly_shape_api::StaticShape;\n"
        "\n"
        "type Invalid = StaticShape<2, { [2, 3] }>;\n"
        "\n"
        "fn main() {\n"
        "    let _ = std::mem::size_of::<Undefined>();\n"
        "}\n",
        encoding="utf-8",
    )
    (root / FAIL / "rank_array_length.stderr").write_text(
        "error[E0412]: cannot find type `Undefined` in this scope\n"
        " --> tests/ui/fail/rank_array_length.rs:6:33\n"
        "  |\n"
        "6 |     let _ = std::mem::size_of::<Undefined>();\n"
        "  |                                 ^^^^^^^^^ not found in this scope\n",
        encoding="utf-8",
    )
    rewrite_once(
        verify.sole_record(root),
        '"first_line": "error[E0308]: mismatched types",\n'
        '      "diagnostic_codes": [\n'
        '        "E0308"\n'
        "      ],\n"
        '      "required_fragments": [\n'
        '        "tests/ui/fail/rank_array_length.rs:3:33"',
        '"first_line": "error[E0412]: cannot find type `Undefined` in this scope",\n'
        '      "diagnostic_codes": [\n'
        '        "E0308"\n'
        "      ],\n"
        '      "required_fragments": [\n'
        '        "tests/ui/fail/rank_array_length.rs:3:33"',
    )


def weaken_compiling_fixture(root: Path) -> None:
    """Drop the rank-64 probe from the arbitrary-rank case.

    Measured on `nightly-2026-07-19` at `f57e23b`: after exactly this edit the
    gated suite still reports `2 passed`, `retained_fixture_inventory_is_complete`
    included, and every trybuild case still resolves. Nothing the Rust gate
    compiles can see it, because a compile-pass case retains no diagnostic for a
    source change to contradict, so ADR 0067's arbitrary-rank claim would keep a
    green gate while being demonstrated only up to rank two.
    """
    (root / PASS / "ranks.rs").write_text(
        "use nightly_shape_api::StaticShape;\n"
        "\n"
        "type Scalar = StaticShape<0, { [] }>;\n"
        "type Vector = StaticShape<1, { [8] }>;\n"
        "type Matrix = StaticShape<2, { [2, 3] }>;\n"
        "\n"
        "fn main() {\n"
        "    assert_eq!(std::mem::size_of::<Scalar>(), 0);\n"
        "    assert_eq!(std::mem::size_of::<Vector>(), 0);\n"
        "    assert_eq!(std::mem::size_of::<Matrix>(), 0);\n"
        "}\n",
        encoding="utf-8",
    )


def edit_unquoted_fixture_line(root: Path) -> None:
    """Change a compile-fail fixture on a line its diagnostic never quotes.

    Measured on `nightly-2026-07-19` at `f57e23b`: with `forge.rs` rewritten this
    way the gated suite reports `2 passed`, all six trybuild cases resolve, and
    `forge.stderr` is byte-identical to the retained one — the privacy error names
    lines 12 to 14 and the alias sits on line 5, so nothing the compiler emits
    moves. The case would still demonstrate field privacy, but no longer for the
    matrix evidence the record attributes it to, and only a source claim notices.
    """
    rewrite_once(
        root / FAIL / "forge.rs",
        "type Matrix = StaticShape<2, { [2, 3] }>;",
        "type Matrix = StaticShape<0, { [] }>;",
    )


def drop_claim_field(field: str) -> Callable[[Path], None]:
    """Remove one recorded field from the first compile-fail claim.

    A record can decay by asserting less rather than by asserting something wrong,
    and a defaulted field is how that stays invisible.
    """

    def mutate(root: Path) -> None:
        edit_record(root, lambda record: first_failing_claim(record).pop(field))

    return mutate


def empty_claim_field(field: str) -> Callable[[Path], None]:
    """Leave one recorded field present but asserting nothing."""

    def mutate(root: Path) -> None:
        edit_record(root, lambda record: first_failing_claim(record).__setitem__(field, []))

    return mutate


def reattribute_first_claim(value: object) -> Callable[[Path], None]:
    """Point the first compile-fail claim's attribution somewhere else."""

    def mutate(root: Path) -> None:
        edit_record(root, lambda record: first_failing_claim(record).__setitem__("adr", value))

    return mutate


def duplicate_claim_id(root: Path) -> None:
    """Give two claims one identity, so one of them silently answers for both."""

    def mutate(record: dict[str, object]) -> None:
        claims = record["claims"]
        claims[1]["id"] = claims[0]["id"]

    edit_record(root, mutate)


TAMPERS: tuple[tuple[str, Callable[[Path], object], str], ...] = (
    (
        "no record at all",
        lambda root: shutil.rmtree(root / "results"),
        "must retain exactly one claims record, found 0",
    ),
    ("a second record", duplicate_record, "must retain exactly one claims record, found 2"),
    (
        "a record under another schema",
        lambda root: rewrite_once(verify.sole_record(root), verify.SCHEMA, "wrong/v1"),
        f"is not a {verify.SCHEMA} record",
    ),
    (
        "evidence re-recorded off the repository pin",
        lambda root: rewrite_once(
            verify.sole_record(root),
            '"channel": "nightly-2026-07-19"',
            '"channel": "nightly-2026-07-20"',
        ),
        "records toolchain nightly-2026-07-20, but rust-toolchain.toml pins nightly-2026-07-19",
    ),
    (
        "a harness that no longer measures the pin",
        lambda root: rewrite_once(
            root / "measure.py",
            'TOOLCHAINS = ("nightly-2026-07-19", "nightly-2026-07-20")',
            'TOOLCHAINS = ("nightly-2026-07-20",)',
        ),
        "measure.py measures ['nightly-2026-07-20'], which does not include the pinned "
        "nightly-2026-07-19",
    ),
    (
        "a README that no longer documents the pin",
        lambda root: rewrite_all(root / "README.md", "nightly-2026-07-19", "nightly-2026-07-21"),
        "which does not include the pinned nightly-2026-07-19",
    ),
    (
        "a lockfile moved to another trybuild",
        lambda root: rewrite_once(
            root / "Cargo.lock", 'version = "1.0.118"', 'version = "1.0.119"'
        ),
        "but Cargo.lock pins 1.0.119",
    ),
    (
        "a fixture weakened and its diagnostic refreshed in the same commit",
        weaken_fixture_and_overwrite_its_diagnostic,
        "now reports diagnostic codes ['E0412'], recorded ['E0308']",
    ),
    (
        "a changed diagnostic first line",
        lambda root: rewrite_once(
            root / FAIL / "forge.stderr",
            "of struct `ShapedValue` are private",
            "of struct `ShapedValue` are inaccessible",
        ),
        "are inaccessible'",
    ),
    (
        "a claim that dropped its diagnostic codes",
        drop_claim_field("diagnostic_codes"),
        f"{FAIL}/forge.rs records no diagnostic_codes list",
    ),
    (
        "a claim that dropped its required fragments",
        drop_claim_field("required_fragments"),
        f"{FAIL}/forge.rs records no required_fragments list",
    ),
    (
        "a claim that dropped its source fragments",
        drop_claim_field("source_fragments"),
        f"{FAIL}/forge.rs records no source_fragments list",
    ),
    (
        "a claim that dropped its forbidden fragments",
        drop_claim_field("forbidden_fragments"),
        f"{FAIL}/forge.rs records no forbidden_fragments list",
    ),
    (
        "a fragment list decayed to empty",
        empty_claim_field("required_fragments"),
        f"{FAIL}/forge.rs records an empty required_fragments list",
    ),
    (
        "a dropped required fragment",
        lambda root: rewrite_once(
            root / FAIL / "implement_evidence.stderr",
            'is a "sealed trait"',
            'is a "closed trait"',
        ),
        "no longer reports '`ShapeEvidence` is a \"sealed trait\"'",
    ),
    (
        "a forbidden fragment appearing",
        lambda root: append_text(
            root / FAIL / "rank_array_length.stderr",
            "  |     arguments to this function are incorrect\n",
        ),
        "now reports 'arguments to this function are incorrect', which it must not",
    ),
    (
        "a deleted retained diagnostic",
        lambda root: (root / FAIL / "forge.stderr").unlink(),
        f"fixture retains no diagnostic: {FAIL}/forge.rs",
    ),
    (
        "a fixture present without a record",
        lambda root: (root / FAIL / "unrecorded.rs").write_text("fn main() {}\n", encoding="utf-8"),
        f"fixtures ['{FAIL}/unrecorded.rs'] are present without a record",
    ),
    (
        "a record whose fixture is missing",
        lambda root: (root / FAIL / "forge.rs").unlink(),
        f"recorded fixture is missing: {FAIL}/forge.rs",
    ),
    (
        "a compiling fixture claiming a failure",
        lambda root: (root / PASS / "ranks.stderr").write_text(
            "error: invented\n", encoding="utf-8"
        ),
        f"a compiling fixture must retain no diagnostic: {PASS}/ranks.rs",
    ),
    (
        "an orphaned retained diagnostic",
        lambda root: (root / FAIL / "orphan.stderr").write_text(
            "error: invented\n", encoding="utf-8"
        ),
        "orphaned retained diagnostic orphan.stderr",
    ),
    (
        "a compile-fail fixture edited where its diagnostic does not quote it",
        edit_unquoted_fixture_line,
        f"{FAIL}/forge.rs no longer contains 'type Matrix = StaticShape<2, {{ [2, 3] }}>;'",
    ),
    (
        "a compile-pass fixture weakened below its claim",
        weaken_compiling_fixture,
        f"{PASS}/ranks.rs no longer contains 'type Rank64 = StaticShape<64, {{ [1; 64] }}>;'",
    ),
    (
        "a claim attributed to a decision that does not exist",
        reattribute_first_claim("ADR-9999"),
        "ADR-9999 resolves to 0 decision documents",
    ),
    (
        "a claim that dropped its attribution",
        drop_claim_field("adr"),
        "records no adr attribution",
    ),
    (
        "a claim naming a decision but no clause within it",
        drop_claim_field("adr_clause"),
        "names ADR-0067 but no clause within it",
    ),
    (
        "two claims sharing one identity",
        duplicate_claim_id,
        "is recorded twice",
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
    verify.verify_dependent_shape_claims(root, verify.read_pinned_channel())

    mutate(root)

    with pytest.raises(verify.ClaimFailure, match=re.escape(expected)):
        verify.verify_dependent_shape_claims(root, verify.read_pinned_channel())
