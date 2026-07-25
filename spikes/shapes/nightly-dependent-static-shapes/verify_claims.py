#!/usr/bin/env python3
"""Check the gated dependent-static-shape fixtures against their recorded claims.

`scripts/check_rust.py` names this workspace in `GATED_SPIKE_WORKSPACES`, so the
Rust gate compiles it on the pinned nightly on every run and `trybuild` compares
each fixture against the `.stderr` beside it byte for byte. `verify_fixture_coverage`
then requires the run transcript to name every case, and
`retained_fixture_inventory_is_complete` in `conformance/tests/ui.rs` requires the
inventory on disk to equal a named list. Between them, a diagnostic that no longer
matches its fixture, a glob that stopped matching, and a case deleted from the tree
all fail.

Compilation still proves only that a fixture and its diagnostic agree. It does not
prove the agreed diagnostic is the claim ADR 0067 relies on. This module is that
other half: a record beside the fixtures states, per case, which error the case
exists to demonstrate, which ADR clause it is attributed to, and which text the
fixture itself must still contain. A case weakened until it fails for an unrelated
reason and refreshed with `TRYBUILD=overwrite` in the same commit compiles cleanly
and is refused here.

Two rules differ deliberately from the off-pin form in
`spikes/shapes/shape-evidence/verify_evidence.py`, and both follow from being
compiled rather than excluded.

The channel comparison is inverted. That spike requires its recorded channel *not*
to be the repository pin, because its claim is about a compiler the gate cannot
run. Here the recorded channel must *equal* the pin: the gate recompiles these
fixtures with exactly the toolchain `rust-toolchain.toml` names, so a record naming
another compiler describes a run that no longer happens, and a pin migration must
force the claims to be re-derived rather than inherited.

Nothing is pinned by digest. The off-pin record digests its fixture sources and
`.stderr` bytes because no compilation ever re-derives them. Here compilation
re-derives every `.stderr` on every gate invocation, so digesting one pins nothing
a rebuild would not already catch, and a legitimate pin migration regenerates them
all. The fixture *sources* are a different question: a source edit the compiler does
not echo leaves the retained diagnostic byte-identical, so reproduction cannot see
it. That is what `source_fragments` is for. It states what the fixture must still
contain to be the case it is recorded as, which survives cosmetic edits that a
digest would reject and refuses semantic ones that a digest would only report as
"something changed".

**Why this file duplicates its sibling instead of importing one.** Settled by
`share-the-spike-diagnostic-claims-verifier`, and recorded at length in
`../shape-evidence/verify_evidence.py`'s module docstring rather than twice. In
short: the seven functions the two files genuinely share are file reading with no
posture in them, and sharing them would mean threading each caller's exception
type through every one, because both adversarial suites assert on their own
class. The four that carry posture — `verify_toolchain`, `verify_claims`,
`verify_failing_case`, `verify_compiling_case` — are not near-identical and would
need about seven flags to unify, each settable backwards by a later edit. The
trigger for revisiting is a *rule* about what a claim must assert, rather than a
file-reading helper, having to change in more than one verifier at once;
`required_fragment_list` here was copied to the off-pin spike for exactly that
reason once, and a second occurrence is the evidence for paying the cost.
"""

from __future__ import annotations

import argparse
import ast
import json
import re
import sys
import tomllib
from pathlib import Path

SPIKE = Path(__file__).resolve().parent
REPOSITORY = SPIKE.parents[2]
TOOLCHAIN_MANIFEST = REPOSITORY / "rust-toolchain.toml"
DECISIONS = REPOSITORY / "docs" / "decisions"
SCHEMA = "tiler-nightly-dependent-shapes-diagnostics/v1"
FAIL_DIR = "conformance/tests/ui/fail"
PASS_DIR = "conformance/tests/ui/pass"
MAX_RECORD_BYTES = 256 << 10
ERROR_CODE = re.compile(r"^error\[(E\d{4})\]", re.MULTILINE)
DATED_NIGHTLY = re.compile(r"nightly-\d{4}-\d{2}-\d{2}")
DECISION_REFERENCE = re.compile(r"^ADR-(\d{4})$")


class ClaimFailure(RuntimeError):
    """A gated fixture no longer demonstrates the claim recorded for it."""


def read_text(path: Path, description: str) -> str:
    """Read one checked-in file, failing closed on anything unreadable."""
    try:
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as error:
        raise ClaimFailure(f"cannot read {description} at {path.name}: {error}") from error


def read_pinned_channel(manifest: Path = TOOLCHAIN_MANIFEST) -> str:
    """Read the repository's sole Rust toolchain authority."""
    try:
        channel = tomllib.loads(read_text(manifest, "the toolchain manifest"))["toolchain"][
            "channel"
        ]
    except (tomllib.TOMLDecodeError, KeyError, TypeError) as error:
        raise ClaimFailure(f"cannot read the pinned toolchain channel: {error}") from error
    if not isinstance(channel, str) or not channel:
        raise ClaimFailure(f"pinned toolchain channel is not a string: {channel!r}")
    return channel


def declared_measure_toolchains(root: Path) -> tuple[str, ...]:
    """Read the toolchains the spike's measurement harness actually executes.

    `measure.py`'s `TOOLCHAINS` is the spike's executable statement of which
    compilers it covers: the governed pin and whichever adjacent nightly a
    migration probe compared against. Parse it rather than matching text, so a
    value appearing only in a comment cannot answer for the real constant.
    """
    source = read_text(root / "measure.py", "the measurement harness")
    try:
        module = ast.parse(source)
    except SyntaxError as error:
        raise ClaimFailure(f"measure.py does not parse: {error}") from error
    found = [
        node.value
        for node in module.body
        if isinstance(node, ast.Assign)
        and isinstance(node.value, ast.Tuple)
        and any(
            isinstance(target, ast.Name) and target.id == "TOOLCHAINS" for target in node.targets
        )
    ]
    if len(found) != 1:
        raise ClaimFailure(
            f"measure.py must declare exactly one module-level TOOLCHAINS tuple, found {len(found)}"
        )
    values = [
        element.value
        for element in found[0].elts
        if isinstance(element, ast.Constant) and isinstance(element.value, str)
    ]
    if len(values) != len(found[0].elts) or not values:
        raise ClaimFailure("measure.py TOOLCHAINS must be a non-empty tuple of string literals")
    return tuple(values)


def documented_toolchains(root: Path) -> set[str]:
    """Collect every dated nightly the spike README tells a reader to run."""
    return set(DATED_NIGHTLY.findall(read_text(root / "README.md", "the spike README")))


def locked_package_version(root: Path, name: str) -> str:
    """Read one exact dependency version from the spike's own lockfile."""
    try:
        lock = tomllib.loads(read_text(root / "Cargo.lock", "the spike lockfile"))
        packages = lock["package"]
    except (tomllib.TOMLDecodeError, KeyError, TypeError) as error:
        raise ClaimFailure(f"cannot read the spike lockfile: {error}") from error
    versions = [
        entry["version"]
        for entry in packages
        if isinstance(entry, dict) and entry.get("name") == name
    ]
    if len(versions) != 1 or not isinstance(versions[0], str):
        raise ClaimFailure(f"the spike lockfile does not pin exactly one {name}: {versions!r}")
    return versions[0]


def sole_record(root: Path) -> Path:
    """Return the spike's single retained claims record.

    Exactly one is required. Two records would be two attributions of the same
    fixtures with nothing saying which governs when they disagree.
    """
    records = sorted((root / "results").glob("*.json"))
    if len(records) != 1:
        raise ClaimFailure(f"the spike must retain exactly one claims record, found {len(records)}")
    return records[0]


def read_record(path: Path) -> dict[str, object]:
    """Load the retained record under a fixed size budget."""
    try:
        if path.stat().st_size > MAX_RECORD_BYTES:
            raise ClaimFailure(f"{path.name} exceeds {MAX_RECORD_BYTES} bytes")
        record = json.loads(read_text(path, "the claims record"))
    except (OSError, json.JSONDecodeError) as error:
        raise ClaimFailure(f"{path.name} is not a readable JSON record: {error}") from error
    if not isinstance(record, dict) or record.get("schema") != SCHEMA:
        raise ClaimFailure(f"{path.name} is not a {SCHEMA} record")
    return record


def verify_toolchain(root: Path, label: str, record: dict[str, object], pinned: str) -> str:
    """Require the record to describe a run of the compiler the gate actually uses.

    The equality is the whole point of the gated posture. `scripts/check_rust.py`
    invokes `check.sh` with the channel `rust-toolchain.toml` names, so a record
    naming any other compiler is a record of a run that no longer happens, and a
    pin migration has to re-derive these claims instead of inheriting them.
    """
    toolchain = record.get("toolchain")
    if not isinstance(toolchain, dict):
        raise ClaimFailure(f"{label} records no toolchain")
    channel = toolchain.get("channel")
    if not isinstance(channel, str) or not channel:
        raise ClaimFailure(f"{label} records no toolchain channel")
    if channel != pinned:
        raise ClaimFailure(
            f"{label} records toolchain {channel}, but rust-toolchain.toml pins {pinned}; the "
            "Rust gate reproduces these fixtures on the pin, so re-run the suite on it and "
            "re-record the claims before reusing the conclusion"
        )
    declared = declared_measure_toolchains(root)
    if pinned not in declared:
        raise ClaimFailure(
            f"measure.py measures {list(declared)}, which does not include the pinned {pinned}; "
            "the harness and the record must cover one governed compiler"
        )
    documented = documented_toolchains(root)
    if pinned not in documented:
        raise ClaimFailure(
            f"README documents nightlies {sorted(documented)}, which does not include the pinned "
            f"{pinned}; the documented reproduction must be the recorded one"
        )
    locked = locked_package_version(root, "trybuild")
    if toolchain.get("trybuild_version") != locked:
        raise ClaimFailure(
            f"{label} records trybuild {toolchain.get('trybuild_version')!r}, but Cargo.lock pins "
            f"{locked}; trybuild normalizes the retained diagnostics, so another version is "
            "another measurement"
        )
    return channel


def verify_attribution(label: str, claim: dict[str, object]) -> str:
    """Require a claim to name a decision that exists and the clause it demonstrates.

    This resolves the reference, not the wording. Checking that the quoted clause
    still reads as recorded is cross-document quotation validation, which
    `detect-stale-cross-document-quotations` owns for the whole corpus; resolving
    the reference is what stops a renumbered or withdrawn decision from leaving a
    fixture attributed to nothing.
    """
    reference = claim.get("adr")
    if not isinstance(reference, str):
        raise ClaimFailure(f"{label}: claim {claim.get('id')!r} records no adr attribution")
    match = DECISION_REFERENCE.fullmatch(reference)
    if match is None:
        raise ClaimFailure(f"{label}: {reference!r} is not an ADR-NNNN reference")
    documents = sorted(DECISIONS.glob(f"{match.group(1)}-*.md"))
    if len(documents) != 1:
        raise ClaimFailure(
            f"{label}: {reference} resolves to {len(documents)} decision documents; a fixture "
            "must be attributed to exactly one accepted decision"
        )
    clause = claim.get("adr_clause")
    if not isinstance(clause, str) or not clause.strip():
        raise ClaimFailure(
            f"{label}: claim {claim.get('id')!r} names {reference} but no clause within it"
        )
    return reference


def required_fragment_list(label: str, case: str, claim: dict[str, object], key: str) -> list[str]:
    """Read one fragment list, refusing a claim that decayed into asserting nothing."""
    fragments = claim.get(key)
    if not isinstance(fragments, list) or not all(isinstance(item, str) for item in fragments):
        raise ClaimFailure(f"{label}: {case} records no {key} list")
    if not fragments:
        raise ClaimFailure(f"{label}: {case} records an empty {key} list")
    return list(fragments)


def verify_source(root: Path, label: str, claim: dict[str, object], case: str) -> None:
    """Require the fixture source to still contain what makes it this case.

    Compilation cannot see an edit the compiler does not echo. Changing a type
    alias a diagnostic never quotes, or deleting the higher ranks a compile-pass
    case exists to cover, leaves the whole gated suite green — both were measured
    on the governed pin — so the source claim is checked here rather than inferred
    from the diagnostic.
    """
    text = read_text(root / case, "a fixture source")
    for fragment in required_fragment_list(label, case, claim, "source_fragments"):
        if fragment not in text:
            raise ClaimFailure(
                f"{label}: {case} no longer contains {fragment!r}, so it is no longer the case "
                "this claim is recorded for"
            )


def verify_failing_case(root: Path, label: str, claim: dict[str, object], case: str) -> None:
    """Require one compile-fail fixture to still demonstrate its recorded error.

    The recorded codes are compared as the whole ordered sequence the file emits
    rather than as the first line alone, so a case that gained or lost a second
    error is refused even when its opening error is unchanged, and a
    `TRYBUILD=overwrite` refresh that rewrote the message text in agreement with
    the record still has a separately recorded code to disagree with.
    """
    source = root / case
    if not source.is_file():
        raise ClaimFailure(f"{label}: recorded fixture is missing: {case}")
    retained = source.with_suffix(".stderr")
    if not retained.is_file():
        raise ClaimFailure(f"{label}: fixture retains no diagnostic: {case}")
    text = read_text(retained, "a retained diagnostic")
    lines = text.splitlines()
    first = lines[0] if lines else ""
    if first != claim.get("first_line"):
        raise ClaimFailure(
            f"{label}: {case} now reports {first!r}, recorded {claim.get('first_line')!r}"
        )
    recorded_codes = required_fragment_list(label, case, claim, "diagnostic_codes")
    codes = ERROR_CODE.findall(text)
    if codes != recorded_codes:
        raise ClaimFailure(
            f"{label}: {case} now reports diagnostic codes {codes}, recorded {recorded_codes}"
        )
    if not first.startswith(f"error[{recorded_codes[0]}]"):
        raise ClaimFailure(f"{label}: {case} no longer reports {recorded_codes[0]} first")
    for fragment in required_fragment_list(label, case, claim, "required_fragments"):
        if fragment not in text:
            raise ClaimFailure(f"{label}: {case} no longer reports {fragment!r}")
    forbidden = claim.get("forbidden_fragments")
    if not isinstance(forbidden, list) or not all(isinstance(item, str) for item in forbidden):
        raise ClaimFailure(f"{label}: {case} records no forbidden_fragments list")
    for fragment in forbidden:
        if fragment in text:
            raise ClaimFailure(f"{label}: {case} now reports {fragment!r}, which it must not")
    verify_source(root, label, claim, case)


def verify_compiling_case(root: Path, label: str, claim: dict[str, object], case: str) -> None:
    """Require one compile-pass fixture to exist, expect no failure, and still cover its claim."""
    source = root / case
    if not source.is_file():
        raise ClaimFailure(f"{label}: recorded fixture is missing: {case}")
    if source.with_suffix(".stderr").exists():
        raise ClaimFailure(f"{label}: a compiling fixture must retain no diagnostic: {case}")
    verify_source(root, label, claim, case)


def fixture_names(root: Path, directory: str) -> set[str]:
    """Enumerate the trybuild fixtures actually present in one case directory."""
    return {f"{directory}/{path.name}" for path in (root / directory).glob("*.rs")}


def claim_case(label: str, claim: dict[str, object]) -> str:
    """Return the single fixture one claim names.

    Every claim here names exactly one fixture, unlike the off-pin record's
    plural compile-pass form, because `source_fragments` attaches to a specific
    file and a shared list would not say which case had to contain what.
    """
    case = claim.get("case")
    if not isinstance(case, str) or not case:
        raise ClaimFailure(f"{label}: claim {claim.get('id')!r} names no fixture")
    return case


def verify_claims(root: Path, label: str, record: dict[str, object]) -> dict[str, list[str]]:
    """Check every recorded claim and require the fixture sets to agree both ways."""
    claims = record.get("claims")
    if not isinstance(claims, list) or not claims:
        raise ClaimFailure(f"{label} records no claims")
    recorded_fail: set[str] = set()
    recorded_pass: set[str] = set()
    identifiers: set[str] = set()
    decisions: set[str] = set()
    for claim in claims:
        if not isinstance(claim, dict):
            raise ClaimFailure(f"{label} contains a malformed claim")
        identifier = claim.get("id")
        if not isinstance(identifier, str) or not identifier:
            raise ClaimFailure(f"{label} contains a claim with no id")
        if identifier in identifiers:
            raise ClaimFailure(f"{label}: claim id {identifier!r} is recorded twice")
        identifiers.add(identifier)
        decisions.add(verify_attribution(label, claim))
        case = claim_case(label, claim)
        outcome = claim.get("outcome")
        if outcome == "fails":
            verify_failing_case(root, label, claim, case)
            recorded_fail.add(case)
        elif outcome == "compiles":
            verify_compiling_case(root, label, claim, case)
            recorded_pass.add(case)
        else:
            raise ClaimFailure(f"{label}: unknown claim outcome {outcome!r}")
    for directory, recorded in ((FAIL_DIR, recorded_fail), (PASS_DIR, recorded_pass)):
        present = fixture_names(root, directory)
        if present != recorded:
            raise ClaimFailure(
                f"{label}: {directory} fixtures {sorted(present ^ recorded)} are present without a "
                "record or recorded without a fixture"
            )
    for retained in (root / FAIL_DIR).glob("*.stderr"):
        if not retained.with_suffix(".rs").is_file():
            raise ClaimFailure(f"{label}: orphaned retained diagnostic {retained.name}")
    return {
        "compile_fail_cases": sorted(recorded_fail),
        "compile_pass_cases": sorted(recorded_pass),
        "decisions": sorted(decisions),
    }


def verify_dependent_shape_claims(root: Path, pinned: str) -> dict[str, object]:
    """Check every gated fixture against the claim recorded beside it.

    Ordered from the most general failure to the most specific so a reader gets
    the narrowest true explanation: first whether the record describes the
    compiler the gate runs at all, then each case's diagnostic, its attribution,
    and the source text that makes it that case.
    """
    path = sole_record(root)
    label = path.name
    record = read_record(path)
    channel = verify_toolchain(root, label, record, pinned)
    summary = verify_claims(root, label, record)
    toolchain = record["toolchain"]
    return {
        "record": label,
        "pinned_channel": pinned,
        "recorded_channel": channel,
        "rustc_version": toolchain.get("rustc_version"),
        "rustc_commit_hash": toolchain.get("rustc_commit_hash"),
        **summary,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.parse_args()
    summary = verify_dependent_shape_claims(SPIKE, read_pinned_channel())
    print(json.dumps(summary, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ClaimFailure as error:
        print(f"dependent-shape claims check failed: {error}", file=sys.stderr)
        raise SystemExit(1) from error
