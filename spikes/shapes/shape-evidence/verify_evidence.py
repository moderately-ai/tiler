#!/usr/bin/env python3
"""Check the retained off-pin shape-evidence diagnostics against their record.

`spikes/shapes/shape-evidence` is the repository's sole off-pin spike. Its six
`trybuild` `.stderr` files were captured on stable Rust 1.89.0, which is not the
toolchain `rust-toolchain.toml` pins, so nothing here recompiles them: re-deriving
the fixtures needs a compiler this checkout does not select, and re-recording them
on the pin would destroy the stable-Rust claim the spike exists to make.

Excluding a workspace from *reproduction* is not the same as leaving its
evidence unchecked. This module is the other half. It compares every retained
diagnostic against a checked-in record without invoking Cargo, so a
`TRYBUILD=overwrite` refresh, a hand edit, a deleted case, an unrecorded case,
or an edit to the source the diagnostics quote fails the repository gate with a
precise reason.

Two things follow from the off-pin posture and make this stricter than the
gated-spike equivalent in `spikes/extensions/run.py`.

Its channel comparison is inverted. That harness requires the recorded channel
to be the one `rust-toolchain.toml` pins, because the gate recompiles its
fixtures and a moved pin must force a fresh run. Here the recorded channel must
*not* be the pin: the whole reason this spike is excluded is that its claim is
about a different compiler, so a record that has drifted onto the pin means the
evidence was re-recorded into meaninglessness. The toolchain the record names is
cross-checked against every place inside the spike that states it.

It also pins the diagnostic-bearing input tree by digest. For a compiled spike,
compilation is the total check and a record only adds attribution; here no
compilation ever happens, so nothing but the record can notice that a fixture
source or `src/lib.rs` changed under a diagnostic that quotes it. Editing any
of those inputs therefore fails until the suite is re-run on the recorded
toolchain and the record is refreshed in the same commit.

**Why this file duplicates its sibling instead of importing one.** Settled by
`share-the-spike-diagnostic-claims-verifier`; recorded here so a third spike's
author does not re-derive it. Measured across this file and
`../nightly-dependent-static-shapes/verify_claims.py`: four functions are
byte-identical apart from the exception class — `read_text`,
`read_pinned_channel`, `locked_package_version`, `fixture_names` — and
`read_record`, `sole_record`, and `main` differ only in a message string. All
seven are file reading with no posture in them. The four that carry posture are
not near-identical and cannot be shared without becoming configurable:
`verify_toolchain` differs in essentially every line (the channel comparison is
inverted, and the two spikes cross-check entirely different things against it),
`verify_failing_case` in 32 lines, `verify_claims` in 23, and
`verify_compiling_case` takes a different signature. Folding those into one
predicate needs about seven flags — comparison direction, digesting,
`source_fragments`, ADR attribution, claim-id uniqueness, singular-versus-plural
case form, empty-list strictness — each of which a later edit can set backwards,
in place of two files that each state their own rule in prose.

Lifting only the seven posture-free helpers was rejected on its own terms: each
would have to take the caller's exception type as a parameter, because both
adversarial suites assert on the spike's own class, so it trades ~70 duplicated
lines for the same number of wrappers and makes the sole custodian of evidence
nothing recompiles no longer readable in one file.

**The trigger, restated.** The original one — "a third spike needs a
retained-claims record" — is already spent and did not fire:
`spikes/extensions/run.py` is that third custodian and is a different design
again, checking many records rather than requiring exactly one. The condition
that should fire instead is a *rule* about what a claim must assert, rather than
a file-reading helper, having to be changed in more than one verifier at once.
That has now happened once: `required_fragment_list` below was copied from the
gated sibling to close a divergence this duplication produced, where an emptied
`required_fragments` type-checked and asserted nothing. A second occurrence is
the evidence for paying the sharing cost.
"""

from __future__ import annotations

import argparse
import ast
import hashlib
import json
import re
import sys
import tomllib
from pathlib import Path

SPIKE = Path(__file__).resolve().parent
REPOSITORY = SPIKE.parents[2]
TOOLCHAIN_MANIFEST = REPOSITORY / "rust-toolchain.toml"
SCHEMA = "tiler-shape-evidence-diagnostics/v1"
FAIL_DIR = "tests/ui/fail"
PASS_DIR = "tests/ui/pass"
# Exactly the inputs that determine what the retained diagnostics say. The
# measurement harness, its shell wrappers, and the generated `src/bin` workloads
# are deliberately outside this set: they are compiled by the suite but cannot
# change a compile-fail diagnostic, and freezing them would fail the gate on
# unrelated harness work.
INPUT_PATTERNS = (
    "Cargo.lock",
    "Cargo.toml",
    "src/lib.rs",
    "tests/ui.rs",
    f"{FAIL_DIR}/*.rs",
    f"{FAIL_DIR}/*.stderr",
    f"{PASS_DIR}/*.rs",
    f"{PASS_DIR}/*.stderr",
)
MAX_RECORD_BYTES = 256 << 10
MAX_INPUT_BYTES = 1 << 20
MAX_TOTAL_INPUT_BYTES = 8 << 20
ERROR_CODE = re.compile(r"^error\[(E\d{4})\]", re.MULTILINE)
CARGO_SELECTOR = re.compile(r"\bcargo \+(\S+)")


class EvidenceFailure(RuntimeError):
    """A retained off-pin diagnostic no longer agrees with its record."""


def read_text(path: Path, description: str) -> str:
    """Read one checked-in file, failing closed on anything unreadable."""
    try:
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as error:
        raise EvidenceFailure(f"cannot read {description} at {path.name}: {error}") from error


def read_pinned_channel(manifest: Path = TOOLCHAIN_MANIFEST) -> str:
    """Read the repository's sole Rust toolchain authority."""
    try:
        channel = tomllib.loads(read_text(manifest, "the toolchain manifest"))["toolchain"][
            "channel"
        ]
    except (tomllib.TOMLDecodeError, KeyError, TypeError) as error:
        raise EvidenceFailure(f"cannot read the pinned toolchain channel: {error}") from error
    if not isinstance(channel, str) or not channel:
        raise EvidenceFailure(f"pinned toolchain channel is not a string: {channel!r}")
    return channel


def declared_measure_toolchain(root: Path) -> str:
    """Read the toolchain selector the spike's measurement harness executes with.

    `measure.py`'s `TOOLCHAIN` is the spike's executable statement of which
    compiler it is about, and it is the site `AGENTS.md` and the gate's off-pin
    exclusion both cite. Parse it rather than matching text, so a value that only
    appears in a comment or a docstring cannot answer for the real constant.
    """
    source = read_text(root / "measure.py", "the measurement harness")
    try:
        module = ast.parse(source)
    except SyntaxError as error:
        raise EvidenceFailure(f"measure.py does not parse: {error}") from error
    found = [
        node.value.value
        for node in module.body
        if isinstance(node, ast.Assign)
        and isinstance(node.value, ast.Constant)
        and isinstance(node.value.value, str)
        and any(
            isinstance(target, ast.Name) and target.id == "TOOLCHAIN" for target in node.targets
        )
    ]
    if len(found) != 1:
        raise EvidenceFailure(
            f"measure.py must declare exactly one module-level TOOLCHAIN string, found {len(found)}"
        )
    return found[0]


def manifest_rust_version(root: Path) -> str:
    """Read the spike manifest's declared Rust version."""
    try:
        manifest = tomllib.loads(read_text(root / "Cargo.toml", "the spike manifest"))
        version = manifest["package"]["rust-version"]
    except (tomllib.TOMLDecodeError, KeyError, TypeError) as error:
        raise EvidenceFailure(f"cannot read the spike's rust-version: {error}") from error
    if not isinstance(version, str) or not version:
        raise EvidenceFailure(f"the spike's rust-version is not a string: {version!r}")
    return version


def locked_package_version(root: Path, name: str) -> str:
    """Read one exact dependency version from the spike's own lockfile."""
    try:
        lock = tomllib.loads(read_text(root / "Cargo.lock", "the spike lockfile"))
        packages = lock["package"]
    except (tomllib.TOMLDecodeError, KeyError, TypeError) as error:
        raise EvidenceFailure(f"cannot read the spike lockfile: {error}") from error
    versions = [
        entry["version"]
        for entry in packages
        if isinstance(entry, dict) and entry.get("name") == name
    ]
    if len(versions) != 1 or not isinstance(versions[0], str):
        raise EvidenceFailure(f"the spike lockfile does not pin exactly one {name}: {versions!r}")
    return versions[0]


def readme_cargo_selectors(root: Path) -> set[str]:
    """Collect every toolchain selector the spike's README tells a reader to run."""
    return set(CARGO_SELECTOR.findall(read_text(root / "README.md", "the spike README")))


def sole_record(root: Path) -> Path:
    """Return the spike's single retained record.

    Exactly one is required. The record pins the retained bytes by digest, so two
    records would be two claims over the same files with nothing saying which
    governs when they disagree.
    """
    records = sorted((root / "results").glob("*.json"))
    if len(records) != 1:
        raise EvidenceFailure(
            f"the spike must retain exactly one diagnostics record, found {len(records)}"
        )
    return records[0]


def read_record(path: Path) -> dict[str, object]:
    """Load the retained record under a fixed size budget."""
    try:
        if path.stat().st_size > MAX_RECORD_BYTES:
            raise EvidenceFailure(f"{path.name} exceeds {MAX_RECORD_BYTES} bytes")
        record = json.loads(read_text(path, "the diagnostics record"))
    except (OSError, json.JSONDecodeError) as error:
        raise EvidenceFailure(f"{path.name} is not a readable JSON record: {error}") from error
    if not isinstance(record, dict) or record.get("schema") != SCHEMA:
        raise EvidenceFailure(f"{path.name} is not a {SCHEMA} record")
    return record


def verify_toolchain(root: Path, label: str, record: dict[str, object], pinned: str) -> str:
    """Require the record's toolchain to be off-pin and named consistently.

    A record that has silently moved onto the repository pin is the failure this
    spike's whole posture exists to prevent, so it is rejected before the weaker
    agreement checks; the remaining comparisons keep the harness, the manifest,
    and the README from drifting away from the compiler the evidence belongs to.
    """
    toolchain = record.get("toolchain")
    if not isinstance(toolchain, dict):
        raise EvidenceFailure(f"{label} records no toolchain")
    channel = toolchain.get("channel")
    if not isinstance(channel, str) or not channel:
        raise EvidenceFailure(f"{label} records no toolchain channel")
    if channel == pinned:
        raise EvidenceFailure(
            f"{label} now records the repository pin {pinned}; this spike is excluded from "
            "Rust-gate compilation precisely because its evidence is off-pin, so evidence "
            "recorded on the pin is no longer the stable-Rust claim the spike exists to make"
        )
    declared = declared_measure_toolchain(root)
    if channel != declared:
        raise EvidenceFailure(
            f"{label} records toolchain {channel}, but measure.py declares TOOLCHAIN {declared}; "
            "the record and the harness must name one compiler"
        )
    rust_version = manifest_rust_version(root)
    if channel != rust_version and not channel.startswith(f"{rust_version}."):
        raise EvidenceFailure(
            f"{label} records toolchain {channel}, but Cargo.toml declares rust-version "
            f"{rust_version}; the retained diagnostics are evidence for one release series"
        )
    selectors = readme_cargo_selectors(root)
    if selectors != {channel}:
        raise EvidenceFailure(
            f"README documents cargo selectors {sorted(selectors)}, but the retained diagnostics "
            f"were captured on {channel}; the documented reproduction must be the recorded one"
        )
    locked = locked_package_version(root, "trybuild")
    if toolchain.get("trybuild_version") != locked:
        raise EvidenceFailure(
            f"{label} records trybuild {toolchain.get('trybuild_version')!r}, but Cargo.lock pins "
            f"{locked}; trybuild normalizes the retained diagnostics, so another version is "
            "another measurement"
        )
    return channel


def required_fragment_list(label: str, case: str, claim: dict[str, object], key: str) -> list[str]:
    """Read one fragment list, refusing a claim that decayed into asserting nothing.

    Absent and empty are the same failure. A claim whose `required_fragments`
    became `[]` still type-checks as a list, so a check that only rejects a
    missing key keeps passing while the claim asserts nothing beyond an error
    code that several cases here share. `forbidden_fragments` is deliberately
    not read through this: two cases legitimately forbid nothing, because they
    share no diagnostic code with another case.
    """
    fragments = claim.get(key)
    if not isinstance(fragments, list) or not all(isinstance(item, str) for item in fragments):
        raise EvidenceFailure(f"{label}: {case} records no {key} list")
    if not fragments:
        raise EvidenceFailure(f"{label}: {case} records an empty {key} list")
    return list(fragments)


def claim_cases(label: str, claim: dict[str, object]) -> list[str]:
    """Return the fixture paths one claim names, in either singular or plural form."""
    raw = claim["case"] if "case" in claim else claim.get("cases")
    cases = [raw] if isinstance(raw, str) else raw
    if not isinstance(cases, list) or not cases or not all(isinstance(c, str) and c for c in cases):
        raise EvidenceFailure(f"{label}: claim {claim.get('id')!r} names no fixture")
    return [str(case) for case in cases]


def verify_failing_case(root: Path, label: str, claim: dict[str, object], case: str) -> None:
    """Require one compile-fail fixture to still retain its recorded diagnostic.

    The recorded codes are checked against the whole file rather than the first
    line alone, so a fixture that gained or lost a second error is rejected even
    when its opening error is unchanged, and a re-recording that rewrote the
    message text in agreement with the record still has a separately recorded
    code to disagree with.
    """
    source = root / case
    if not source.is_file():
        raise EvidenceFailure(f"{label}: recorded fixture is missing: {case}")
    retained = source.with_suffix(".stderr")
    if not retained.is_file():
        raise EvidenceFailure(f"{label}: fixture retains no diagnostic: {case}")
    text = read_text(retained, "a retained diagnostic")
    lines = text.splitlines()
    first = lines[0] if lines else ""
    if first != claim.get("first_line"):
        raise EvidenceFailure(
            f"{label}: {case} now reports {first!r}, recorded {claim.get('first_line')!r}"
        )
    recorded_codes = required_fragment_list(label, case, claim, "diagnostic_codes")
    codes = ERROR_CODE.findall(text)
    if codes != recorded_codes:
        raise EvidenceFailure(
            f"{label}: {case} now reports diagnostic codes {codes}, recorded {recorded_codes}"
        )
    if not first.startswith(f"error[{recorded_codes[0]}]"):
        raise EvidenceFailure(f"{label}: {case} no longer reports {recorded_codes[0]} first")
    for fragment in required_fragment_list(label, case, claim, "required_fragments"):
        if fragment not in text:
            raise EvidenceFailure(f"{label}: {case} no longer reports {fragment!r}")
    forbidden = claim.get("forbidden_fragments")
    if not isinstance(forbidden, list) or not all(isinstance(item, str) for item in forbidden):
        raise EvidenceFailure(f"{label}: {case} records no forbidden_fragments list")
    for fragment in forbidden:
        if fragment in text:
            raise EvidenceFailure(f"{label}: {case} now reports {fragment!r}, which it must not")


def verify_compiling_case(root: Path, label: str, case: str) -> None:
    """Require one compiling fixture to exist and to claim no expected failure."""
    source = root / case
    if not source.is_file():
        raise EvidenceFailure(f"{label}: recorded fixture is missing: {case}")
    if source.with_suffix(".stderr").exists():
        raise EvidenceFailure(f"{label}: a compiling fixture must retain no diagnostic: {case}")


def fixture_names(root: Path, directory: str) -> set[str]:
    """Enumerate the trybuild fixtures actually present in one case directory."""
    return {f"{directory}/{path.name}" for path in (root / directory).glob("*.rs")}


def verify_claims(root: Path, label: str, record: dict[str, object]) -> dict[str, list[str]]:
    """Check every recorded claim and require the fixture sets to agree both ways."""
    claims = record.get("claims")
    if not isinstance(claims, list) or not claims:
        raise EvidenceFailure(f"{label} records no claims")
    recorded_fail: set[str] = set()
    recorded_pass: set[str] = set()
    for claim in claims:
        if not isinstance(claim, dict):
            raise EvidenceFailure(f"{label} contains a malformed claim")
        cases = claim_cases(label, claim)
        outcome = claim.get("outcome")
        if outcome == "fails":
            if len(cases) != 1:
                raise EvidenceFailure(f"{label}: a failing claim names exactly one fixture")
            verify_failing_case(root, label, claim, cases[0])
            recorded_fail.add(cases[0])
        elif outcome == "compiles":
            for case in cases:
                verify_compiling_case(root, label, case)
                recorded_pass.add(case)
        else:
            raise EvidenceFailure(f"{label}: unknown claim outcome {outcome!r}")
    for directory, recorded in ((FAIL_DIR, recorded_fail), (PASS_DIR, recorded_pass)):
        present = fixture_names(root, directory)
        if present != recorded:
            raise EvidenceFailure(
                f"{label}: {directory} fixtures {sorted(present ^ recorded)} are present without a "
                "record or recorded without a fixture"
            )
    for retained in (root / FAIL_DIR).glob("*.stderr"):
        if not retained.with_suffix(".rs").is_file():
            raise EvidenceFailure(f"{label}: orphaned retained diagnostic {retained.name}")
    return {
        "compile_fail_cases": sorted(recorded_fail),
        "compile_pass_cases": sorted(recorded_pass),
    }


def hash_inputs(root: Path) -> dict[str, dict[str, object]]:
    """Digest exactly the inputs that determine the retained diagnostics."""
    paths = sorted({path for pattern in INPUT_PATTERNS for path in root.glob(pattern)})
    total = 0
    inputs: dict[str, dict[str, object]] = {}
    for path in paths:
        try:
            contents = path.read_bytes()
        except OSError as error:
            raise EvidenceFailure(f"cannot read recorded input {path.name}: {error}") from error
        if len(contents) > MAX_INPUT_BYTES:
            raise EvidenceFailure(f"{path.name} exceeds {MAX_INPUT_BYTES} bytes")
        total += len(contents)
        if total > MAX_TOTAL_INPUT_BYTES:
            raise EvidenceFailure(f"recorded inputs exceed {MAX_TOTAL_INPUT_BYTES} bytes")
        inputs[path.relative_to(root).as_posix()] = {
            "bytes": len(contents),
            "sha256": hashlib.sha256(contents).hexdigest(),
        }
    return inputs


def verify_inputs(root: Path, label: str, record: dict[str, object]) -> None:
    """Require the diagnostic-bearing source tree to be exactly what was recorded.

    Nothing recompiles this spike, so a diagnostic that quotes `src/lib.rs` or a
    fixture stays on disk verbatim after the code beneath it changes. Only a
    digest notices that, and the fix is a fresh run on the recorded toolchain
    rather than a record edit.
    """
    recorded = record.get("inputs")
    if not isinstance(recorded, dict) or not recorded:
        raise EvidenceFailure(f"{label} records no input digests")
    present = hash_inputs(root)
    difference = sorted(set(present) ^ set(recorded))
    if difference:
        raise EvidenceFailure(
            f"{label}: inputs {difference} are present without a recorded digest or recorded "
            "without a file"
        )
    changed = sorted(name for name in present if present[name] != recorded[name])
    if changed:
        raise EvidenceFailure(
            f"{label}: inputs {changed} no longer match their recorded digest; the retained "
            "diagnostics describe a different source tree, so re-run the suite on the recorded "
            "toolchain and re-record in the same commit"
        )


def verify_shape_evidence(root: Path, pinned: str) -> dict[str, object]:
    """Check every retained off-pin diagnostic against the record beside it.

    Ordered from the most specific failure to the most general so that a reader
    gets the narrowest true explanation: the toolchain the evidence belongs to,
    then each recorded diagnostic, then the fixture inventory, and only then the
    digest that catches an edit no recorded claim happens to cover.
    """
    path = sole_record(root)
    label = path.name
    record = read_record(path)
    channel = verify_toolchain(root, label, record, pinned)
    summary = verify_claims(root, label, record)
    verify_inputs(root, label, record)
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
    summary = verify_shape_evidence(SPIKE, read_pinned_channel())
    print(json.dumps(summary, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except EvidenceFailure as error:
        print(f"shape-evidence record check failed: {error}", file=sys.stderr)
        raise SystemExit(1) from error
