"""Measure whether a byte-literal embedding is genuinely self-contained.

`docs/research/embedding/embedded-artifact-costs.md` decided the representation —
one proc-macro byte-string literal per payload — and measured what it costs. It
did not establish the property the representation was chosen *for*: that the
expanded code carries the artifact rather than referring to it. This driver
establishes that, and measures how the two build tools behave around it.

Inputs
------
The Cargo workspace at ``spikes/embedding/self-contained``, and real artifact
envelopes produced by ``prototypes/serial-sum-compile``, which runs the offline
Metal toolchain and writes envelopes carrying genuine compiled ``metallib``
objects. The driver runs that producer itself, so a run reconstructs everything
it needs under its own run root.

Outputs
-------
One TSV row per scenario on stdout, the exact rendered text of every diagnostic
class, and with ``--record`` result fixtures under ``spikes/embedding/results/``.

Metrics
-------
``expansions`` — how many times the macro ran, counted from one event file per
expansion. ``built`` — how many of those had to read the envelope off disk,
which is the quantity the expansion cache exists to suppress.
``published``/``hit``/``uncached`` are the cache's own report.

How a scenario can fail
-----------------------
Every deletion is *proved*: the same path is required to hold files before the
deletion and to hold none afterwards, so a mistyped path fails on the
before-check rather than passing on the after-check. Every scenario declares the
exact number of expansions it must observe. Every concurrent scenario requires
observed overlap between windows in different processes. ``test_self_contained``
covers the predicates against inputs that must be rejected.

Unsupported here
----------------
A real LSP session with incremental edits, hosts other than the measured one,
release-profile and linker-folding questions (the cost note owns those), and any
payload larger than the ~47 KiB this slice's producer emits.
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path

SPIKE = Path(__file__).resolve().parent
WORKSPACE = SPIKE / "self-contained"
REPO = SPIKE.parent.parent
RESULTS = SPIKE / "results"

#: The pinned toolchain, and the second one the toolchain-change axis moves to.
#: Both must already be installed: this driver never installs, selects, or
#: mutates a toolchain, and records the axis as an unreached boundary instead.
PIN = "nightly-2026-07-19"
ALTERNATE = "nightly-2026-07-20"

#: The proc-macro server that performs expansion for the editor. The pin ships
#: it even though it ships no full ``rust-analyzer``; pointing an off-pin
#: analyzer at it is what keeps the expanding process the pinned one.
def proc_macro_srv(toolchain: str) -> Path:
    return Path(
        f"{Path.home()}/.rustup/toolchains/{toolchain}-aarch64-apple-darwin"
        "/libexec/rust-analyzer-proc-macro-srv"
    )


PIN_SRV = proc_macro_srv(PIN)

#: Envelope members this slice embeds. Both are real, and they differ, which is
#: what makes "repeated" and "unique" across crates two different scenarios.
MEMBER_ONE = "serial-sum.tiler.nontrivial.selected"
MEMBER_TWO = "serial-sum.tiler.singleton.materialized"

#: Expansions one build of one consumer crate performs. Mirrors each consumer's
#: own ``INVOCATIONS`` and is checked against it, so the two cannot drift.
INVOCATIONS = 1
CONSUMERS = ("embed-consumer-a", "embed-consumer-b")

#: The per-invocation embedding ceiling the macro enforces, restated from
#: ``docs/research/embedding/embedded-artifact-costs.md``.
CEILING_BYTES = 1 << 20

#: Features rustc's own expanded output needs to compile again. The expansion
#: of `println!` and `assert_eq!` names internal `std` items; nothing about the
#: *payload* needs them, and the standalone crate declares them so that the
#: byte literal can be rebuilt exactly as rustc rendered it.
EXPANDED_FEATURES = "prelude_import, panic_internals, print_internals"

#: Code forms that must not appear in an expanded consumer. Each is a way the
#: expansion could reach outside itself for the payload, which is the property
#: under test.
#:
#: Written as the *call* forms rather than as bare words on purpose: a doc
#: comment naming the macro is not a reference to it, and the first version of
#: this list rejected the fixture's own documentation. The structural half of
#: the claim is stronger anyway and is enforced separately — the generated crate
#: declares no dependency and is built `--offline` from an empty target
#: directory, so an expansion that still named `embed_macro` could not link.
FORBIDDEN_IN_EXPANSION = (
    "include_bytes!",
    "include_str!",
    "env!",
    "option_env!",
    "std::fs",
    "embed_macro::",
)

DEFAULT_DEADLINE = 1800


class ScenarioFailure(RuntimeError):
    """A scenario observed something other than what it declared it must."""


@dataclass
class Outcome:
    """What one scenario observed."""

    name: str
    driver: str = "cargo"
    toolchain: str = PIN
    crates: int = 1
    events: list[dict] = field(default_factory=list)
    seconds: float = 0.0
    payload_bytes: int = 0
    note: str = ""

    @property
    def built(self) -> int:
        return sum(1 for event in self.events if event["built"])

    def count(self, outcome: str) -> int:
        return sum(1 for event in self.events if event["outcome"] == outcome)

    @property
    def processes(self) -> set[int]:
        return {event["pid"] for event in self.events}

    @property
    def drivers(self) -> set[str]:
        return {event["driver"] for event in self.events}

    @property
    def working_dirs(self) -> set[str]:
        return {event["cwd"] for event in self.events}

    @property
    def cross_process_overlaps(self) -> int:
        """Counts expansion windows in different processes that truly intersect.

        Concurrency has to be observed, not assumed from the fact that two
        crates were named on one command line. Two crates that happened to
        serialize and two that genuinely raced produce identical outcome counts.
        """
        overlaps = 0
        for index, left in enumerate(self.events):
            for right in self.events[index + 1 :]:
                if left["pid"] == right["pid"]:
                    continue
                if left["started_ns"] < right["ended_ns"] and right["started_ns"] < left["ended_ns"]:
                    overlaps += 1
        return overlaps

    def row(self) -> str:
        return "\t".join(
            (
                self.name,
                self.driver,
                self.toolchain,
                str(self.crates),
                str(len(self.events)),
                str(self.built),
                str(self.count("published")),
                str(self.count("hit")),
                str(self.count("uncached")),
                str(len(self.processes)),
                str(self.cross_process_overlaps),
                str(len(self.working_dirs)),
                f"{self.seconds:.1f}",
                str(self.payload_bytes),
                self.note,
            )
        )


HEADER = (
    "scenario\tdriver\ttoolchain\tcrates\texpansions\tbuilt\tpublished\thit\tuncached"
    "\tprocesses\toverlaps\tcwds\tseconds\tbytes\tnote"
)


# -- checks that can say no ---------------------------------------------------


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ScenarioFailure(message)


def census(root: Path) -> int:
    """Counts regular files under `root`, or zero when it does not exist.

    Named and counted rather than answered yes/no, because "no files here" and
    "this check did not run" are the two answers a survey must never confuse.
    """
    if not root.exists():
        return 0
    return sum(1 for path in root.rglob("*") if path.is_file())


def delete_tree_provably(label: str, root: Path) -> int:
    """Deletes `root` and proves the path deleted is the one that held files.

    A check that passes because the path was wrong proves nothing, so the same
    path is required to hold at least one file *before* the deletion — a typo
    fails there, before anything is removed — and to hold none afterwards.
    Returns how many files were removed, so a caller can report a quantity
    rather than an adjective.
    """
    before = census(root)
    require(
        before > 0,
        f"{label}: {root} held no files before deletion, so deleting it would prove nothing; "
        "this is what a mistyped path looks like",
    )
    shutil.rmtree(root)
    after = census(root)
    require(
        after == 0 and not root.exists(),
        f"{label}: {root} still holds {after} file(s) after deletion",
    )
    return before


def expansion_is_self_contained(source: str, run_root: Path) -> list[str]:
    """Returns every reason an expanded consumer is not self-contained.

    Empty means the expansion carries its payload. The list form is deliberate:
    a boolean would make one satisfied condition look like all of them.
    """
    reasons = []
    if not re.search(r'b"', source):
        reasons.append("no byte-string literal in the expansion")
    for forbidden in FORBIDDEN_IN_EXPANSION:
        if forbidden in source:
            reasons.append(f"expansion names `{forbidden}`")
    for path in (str(run_root), str(run_root / "artifacts"), str(run_root / "cache")):
        if path in source:
            reasons.append(f"expansion names the run path `{path}`")
    return reasons


def byte_literal(source: str) -> str:
    """Returns the one byte-string literal in an expansion, refusing any other count.

    Scanned with escape handling rather than by a regular expression, because a
    payload's own bytes include quotes and backslashes once escaped, and a naive
    scan would end the literal early and report a length that is not the
    literal's.

    Exactly one is required, and that is the representation claim itself: the
    cost note decided one `Literal::byte_string` token per payload over one
    integer literal per byte, and an expansion carrying 36,838 separate tokens
    would satisfy every other check in this driver.
    """
    literals = []
    index = 0
    while (start := source.find('b"', index)) != -1:
        cursor = start + 2
        while cursor < len(source):
            if source[cursor] == "\\":
                cursor += 2
                continue
            if source[cursor] == '"':
                break
            cursor += 1
        require(cursor < len(source), "an unterminated byte-string literal in the expansion")
        literals.append(source[start : cursor + 1])
        index = cursor + 1
    require(
        len(literals) == 1,
        f"the expansion holds {len(literals)} byte-string literals; the accepted representation "
        "is exactly one per payload",
    )
    return literals[0]


def fnv1a(data: bytes) -> int:
    """The checksum the consumer recomputes at run time.

    Computed here from the artifact file itself, so the length and content the
    program prints are checked against the producer's bytes rather than only
    against the numbers the expansion recorded alongside them.
    """
    value = 0xCBF29CE484222325
    for byte in data:
        value ^= byte
        value = (value * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return value


# -- run state ----------------------------------------------------------------


@dataclass
class Run:
    """Everything one invocation of this driver owns on disk."""

    root: Path

    @property
    def artifacts(self) -> Path:
        return self.root / "artifacts"

    @property
    def cache(self) -> Path:
        return self.root / "cache"

    @property
    def state(self) -> Path:
        return self.root / "state"

    @property
    def events(self) -> Path:
        return self.state / "events"

    def target(self, name: str) -> Path:
        return self.root / f"target-{name}"

    def reset_events(self) -> None:
        shutil.rmtree(self.events, ignore_errors=True)

    def read_events(self) -> list[dict]:
        if not self.events.is_dir():
            return []
        return [
            json.loads(path.read_text())
            for path in sorted(self.events.iterdir())
            if path.suffix == ".json"
        ]

    def environment(
        self,
        *,
        member_a: str = MEMBER_ONE,
        member_b: str = MEMBER_ONE,
        cache: str | None = None,
        ceiling: int | None = None,
        directory: str | None = None,
    ) -> dict[str, str]:
        env = dict(os.environ)
        env["TILER_EMBED_DIR"] = directory if directory is not None else str(self.artifacts)
        env["TILER_EMBED_MEMBER_A"] = member_a
        env["TILER_EMBED_MEMBER_B"] = member_b
        env["TILER_EMBED_CACHE"] = cache if cache is not None else str(self.cache)
        env["TILER_EMBED_STATE"] = str(self.state)
        if ceiling is not None:
            env["TILER_EMBED_CEILING_BYTES"] = str(ceiling)
        else:
            env.pop("TILER_EMBED_CEILING_BYTES", None)
        env.pop("RUSTC_WRAPPER", None)
        return env


def produce_artifacts(run: Run) -> dict[str, int]:
    """Runs the real producer, so the run reconstructs its own inputs.

    `prototypes/serial-sum-compile` emits envelopes carrying compiled `metallib`
    objects through the offline Metal toolchain. Nothing here fabricates bytes:
    an embedding measured against a synthetic payload would not be evidence
    about the artifacts this project actually produces.
    """
    run.artifacts.mkdir(parents=True, exist_ok=True)
    result = subprocess.run(
        [
            "cargo",
            f"+{PIN}",
            "run",
            "--quiet",
            "--offline",
            "-p",
            "tiler-prototype-compile",
            "--",
            "--out",
            str(run.artifacts / "serial-sum.tiler"),
        ],
        cwd=REPO,
        capture_output=True,
        text=True,
        timeout=DEFAULT_DEADLINE,
        check=False,
    )
    require(
        result.returncode == 0,
        f"the producer failed; the Metal toolchain is a prerequisite\n{result.stderr[-2000:]}",
    )
    sizes = {}
    for member in (MEMBER_ONE, MEMBER_TWO):
        path = run.artifacts / member
        require(path.is_file(), f"the producer wrote no {member}")
        sizes[member] = path.stat().st_size
    return sizes


def ensure_lockfile() -> None:
    """Resolves the fixture's dependencies once, so every build can be offline.

    The lockfile is generated rather than tracked because it is a product of the
    repository's own pinned dependency set, which the fixture reaches by path.
    This is the one step that may touch the network, and it does nothing when a
    lockfile is already present.
    """
    if (WORKSPACE / "Cargo.lock").is_file():
        return
    result = subprocess.run(
        ["cargo", f"+{PIN}", "generate-lockfile"],
        cwd=WORKSPACE,
        capture_output=True,
        text=True,
        timeout=DEFAULT_DEADLINE,
        check=False,
    )
    require(result.returncode == 0, f"could not resolve the fixture\n{result.stderr[-2000:]}")


def prebuild(run: Run, targets: tuple[str, ...]) -> None:
    """Compiles the fixture's dependencies into each target directory.

    Done against a throwaway cache root that is discarded afterwards, so warming
    does not populate the root a scenario is about to judge, and so a scenario's
    recorded seconds measure the consumer's build rather than `tiler-ir`'s.
    """
    warm = run.root / "warmup"
    for target in targets:
        cargo_build(
            run,
            target,
            packages=CONSUMERS,
            env=run.environment(cache=str(warm)),
            label=f"prebuild-{target}",
        )
    shutil.rmtree(warm, ignore_errors=True)


def touch_sources() -> None:
    """Makes Cargo consider both consumer crates dirty.

    Cargo does not know an expansion consulted a cache, so nothing about a
    changed or deleted cache root invalidates a fingerprint. Re-expansion has to
    be forced through the input Cargo *does* track, and that asymmetry is itself
    one of this measurement's results.
    """
    for consumer in ("consumer-a", "consumer-b"):
        (WORKSPACE / consumer / "src" / "main.rs").touch()


def cargo_build(
    run: Run,
    target: str,
    *,
    toolchain: str = PIN,
    packages: tuple[str, ...] = (CONSUMERS[0],),
    env: dict[str, str] | None = None,
    check: bool = True,
    label: str = "cargo",
) -> subprocess.CompletedProcess:
    command = ["cargo", f"+{toolchain}", "build", "--offline", "--target-dir", str(run.target(target))]
    for package in packages:
        command += ["-p", package]
    result = subprocess.run(
        command,
        cwd=WORKSPACE,
        env=env if env is not None else run.environment(),
        capture_output=True,
        text=True,
        timeout=DEFAULT_DEADLINE,
        check=False,
    )
    if check:
        require(result.returncode == 0, f"{label}: cargo failed\n{result.stderr[-3000:]}")
    return result


def run_consumer(run: Run, target: str, package: str) -> str:
    """Runs a built consumer and returns its one output line."""
    binary = run.target(target) / "debug" / package
    require(binary.is_file(), f"{package}: no binary at {binary}")
    result = subprocess.run(
        [str(binary)],
        capture_output=True,
        text=True,
        timeout=300,
        check=False,
    )
    require(
        result.returncode == 0,
        f"{package}: the run failed\n{result.stdout}\n{result.stderr}",
    )
    return result.stdout.strip()


def require_carries(line: str, slot: str, oracle: tuple[int, int]) -> int:
    """Checks one consumer's output against the artifact file's own bytes.

    The expansion records a length and a checksum beside the payload, and the
    program compares the linked bytes against those. That catches corruption
    between emission and link, and nothing else — both numbers come from the
    macro. This is the independent half: `oracle` was computed from the producer's
    file by this driver, so agreement means the bytes in the binary are the bytes
    the producer wrote.
    """
    match = re.fullmatch(rf"slot={slot} len=(\d+) fnv1a=([0-9a-f]{{16}})", line)
    require(match is not None, f"unrecognized consumer output `{line}`")
    assert match is not None
    length, digest = int(match.group(1)), int(match.group(2), 16)
    require(
        (length, digest) == oracle,
        f"slot {slot} carries len={length} fnv1a={digest:016x}, "
        f"but the producer wrote len={oracle[0]} fnv1a={oracle[1]:016x}",
    )
    return length


# -- scenarios ----------------------------------------------------------------


def scenario_cold(run: Run, oracle: tuple[int, int]) -> Outcome:
    """One build against an empty cache: the artifact is read and published."""
    shutil.rmtree(run.cache, ignore_errors=True)
    run.reset_events()
    touch_sources()
    started = time.monotonic()
    cargo_build(run, "main", label="embedding-cold")
    outcome = Outcome("embedding-cold", events=run.read_events(), seconds=time.monotonic() - started)
    require(len(outcome.events) == INVOCATIONS, f"embedding-cold: {len(outcome.events)} expansions")
    require(outcome.count("published") == INVOCATIONS, "embedding-cold: the key should publish")
    require(outcome.built == INVOCATIONS, "embedding-cold: the envelope should be read once")
    outcome.payload_bytes = require_carries(
        run_consumer(run, "main", CONSUMERS[0]), "a", oracle
    )
    outcome.note = "envelope read from disk and published"
    return outcome


def scenario_warm(run: Run, oracle: tuple[int, int]) -> Outcome:
    """A second expansion against the populated cache: no disk read."""
    run.reset_events()
    touch_sources()
    started = time.monotonic()
    cargo_build(run, "main", label="embedding-warm")
    outcome = Outcome("embedding-warm", events=run.read_events(), seconds=time.monotonic() - started)
    require(len(outcome.events) == INVOCATIONS, f"embedding-warm: {len(outcome.events)} expansions")
    require(outcome.count("hit") == INVOCATIONS, "embedding-warm: the key should hit")
    require(outcome.built == 0, f"embedding-warm: {outcome.built} disk reads, expected none")
    outcome.payload_bytes = require_carries(
        run_consumer(run, "main", CONSUMERS[0]), "a", oracle
    )
    outcome.note = "validated cache hit; envelope not read"
    return outcome


def scenario_artifacts_deleted(run: Run, oracle: tuple[int, int]) -> Outcome:
    """Delete every Tiler-produced artifact, then force a re-expansion.

    The cache survives, so a *hit* must stand in for the file. This separates
    the two halves of the deletion question: here the cache is what makes the
    expansion possible, and in `scenario_cache_deleted` it is what is gone.
    """
    run.reset_events()
    removed = delete_tree_provably("artifacts-deleted", run.artifacts)
    touch_sources()
    started = time.monotonic()
    cargo_build(run, "main", label="artifacts-deleted-reexpand")
    outcome = Outcome(
        "artifacts-deleted-reexpand", events=run.read_events(), seconds=time.monotonic() - started
    )
    require(len(outcome.events) == INVOCATIONS, "artifacts-deleted-reexpand: wrong expansion count")
    require(
        outcome.count("hit") == INVOCATIONS,
        "artifacts-deleted-reexpand: only a cache hit can stand in for the deleted file",
    )
    require(outcome.built == 0, "artifacts-deleted-reexpand: nothing could have been read")
    outcome.payload_bytes = require_carries(
        run_consumer(run, "main", CONSUMERS[0]), "a", oracle
    )
    outcome.note = f"{removed} Tiler-produced file(s) deleted; hit stood in"
    return outcome


def scenario_cache_deleted_no_reexpand(run: Run, oracle: tuple[int, int]) -> Outcome:
    """Delete the whole cache root, then build and run without touching sources.

    The load-bearing half of "self-contained". If a deleted cache broke this
    build, the embedding was a reference wearing a literal's clothes.
    """
    run.reset_events()
    removed = delete_tree_provably("cache-deleted", run.cache)
    started = time.monotonic()
    cargo_build(run, "main", label="cache-deleted-no-reexpand")
    outcome = Outcome(
        "cache-deleted-no-reexpand", events=run.read_events(), seconds=time.monotonic() - started
    )
    require(
        len(outcome.events) == 0,
        f"cache-deleted-no-reexpand: {len(outcome.events)} expansions ran, so this scenario "
        "measured a re-expansion rather than the already-expanded code",
    )
    outcome.payload_bytes = require_carries(
        run_consumer(run, "main", CONSUMERS[0]), "a", oracle
    )
    outcome.note = (
        f"{removed} cache file(s) deleted, envelopes already gone; no expansion, run unaffected"
    )
    return outcome


def scenario_cache_deleted_reexpand(run: Run, oracle: tuple[int, int]) -> Outcome:
    """Cache gone, artifacts restored: a forced re-expansion republishes."""
    run.reset_events()
    produce_artifacts(run)
    touch_sources()
    started = time.monotonic()
    cargo_build(run, "main", label="cache-deleted-reexpand")
    outcome = Outcome(
        "cache-deleted-reexpand", events=run.read_events(), seconds=time.monotonic() - started
    )
    require(len(outcome.events) == INVOCATIONS, "cache-deleted-reexpand: wrong expansion count")
    require(
        outcome.count("published") == INVOCATIONS,
        "cache-deleted-reexpand: an empty cache must republish",
    )
    outcome.payload_bytes = require_carries(
        run_consumer(run, "main", CONSUMERS[0]), "a", oracle
    )
    outcome.note = "republished from the restored envelope"
    return outcome


def scenario_axis_source_edit(run: Run, oracle: tuple[int, int]) -> list[Outcome]:
    """Axis 1: a source edit, cold and warm.

    Cold is a fresh target directory against an empty cache; warm is the same
    edit with both populated. Recorded as two rows because they answer different
    questions — whether an edit re-expands at all, and what an expansion costs
    once the cache holds the answer.
    """
    shutil.rmtree(run.target("edit"), ignore_errors=True)
    shutil.rmtree(run.cache, ignore_errors=True)
    run.reset_events()
    touch_sources()
    started = time.monotonic()
    cargo_build(run, "edit", label="axis-source-edit-cold")
    cold = Outcome(
        "axis-source-edit-cold", events=run.read_events(), seconds=time.monotonic() - started
    )
    require(len(cold.events) == INVOCATIONS, "axis-source-edit-cold: wrong expansion count")
    require(cold.built == INVOCATIONS, "axis-source-edit-cold: the envelope should be read")
    cold.payload_bytes = require_carries(run_consumer(run, "edit", CONSUMERS[0]), "a", oracle)
    cold.note = "fresh target directory, empty cache"

    run.reset_events()
    touch_sources()
    started = time.monotonic()
    cargo_build(run, "edit", label="axis-source-edit-warm")
    warm = Outcome(
        "axis-source-edit-warm", events=run.read_events(), seconds=time.monotonic() - started
    )
    require(len(warm.events) == INVOCATIONS, "axis-source-edit-warm: wrong expansion count")
    require(
        warm.built == 0,
        "axis-source-edit-warm: an edit must re-expand, but the cache must spare the disk read",
    )
    warm.payload_bytes = require_carries(run_consumer(run, "edit", CONSUMERS[0]), "a", oracle)
    warm.note = "edit re-expands; cache spares the read"
    return [cold, warm]


def scenario_axis_toolchain(run: Run, oracle: tuple[int, int]) -> Outcome:
    """Axis 2: the same sources built by a different rustc.

    No source changed, so anything that re-expands did so because Cargo's
    fingerprint carries the compiler. The cache subject does not, which is what
    makes the two counts in this row differ.
    """
    run.reset_events()
    started = time.monotonic()
    cargo_build(run, "edit", toolchain=ALTERNATE, label="axis-toolchain-change")
    outcome = Outcome(
        "axis-toolchain-change",
        toolchain=ALTERNATE,
        events=run.read_events(),
        seconds=time.monotonic() - started,
    )
    require(
        len(outcome.events) == INVOCATIONS,
        f"axis-toolchain-change: {len(outcome.events)} expansions; a changed compiler should "
        "re-expand exactly as a changed source does",
    )
    require(
        outcome.built == 0,
        "axis-toolchain-change: the cache subject does not carry the compiler, so the envelope "
        "must not be read again",
    )
    outcome.payload_bytes = require_carries(run_consumer(run, "edit", CONSUMERS[0]), "a", oracle)
    outcome.note = f"{PIN} -> {ALTERNATE}; same target directory, no source edit"
    return outcome


def scenario_axis_cross_crate(run: Run, oracle: tuple[int, int], *, repeated: bool) -> Outcome:
    """Axes 3 and 4: two crates embedding the same or different artifacts."""
    name = "axis-repeated-across-crates" if repeated else "axis-unique-across-crates"
    member_b = MEMBER_ONE if repeated else MEMBER_TWO
    shutil.rmtree(run.target("cross"), ignore_errors=True)
    shutil.rmtree(run.cache, ignore_errors=True)
    run.reset_events()
    touch_sources()
    env = run.environment(member_a=MEMBER_ONE, member_b=member_b)
    started = time.monotonic()
    cargo_build(run, "cross", packages=CONSUMERS, env=env, label=name)
    outcome = Outcome(
        name, crates=2, events=run.read_events(), seconds=time.monotonic() - started
    )
    expected = INVOCATIONS * 2
    require(len(outcome.events) == expected, f"{name}: {len(outcome.events)} expansions")
    require(
        len(outcome.processes) == 2,
        f"{name}: each crate should expand in its own rustc process, saw {len(outcome.processes)}",
    )
    if repeated:
        require(
            outcome.built == 1,
            f"{name}: one artifact, so exactly one crate should read it; {outcome.built} did",
        )
        outcome.note = "one envelope read across two crates"
    else:
        require(
            outcome.built == 2,
            f"{name}: two artifacts, so both crates must read; {outcome.built} did",
        )
        outcome.note = "two envelopes, two reads, no contention"
    outcome.payload_bytes = require_carries(run_consumer(run, "cross", CONSUMERS[0]), "a", oracle)
    run_consumer(run, "cross", CONSUMERS[1])
    return outcome


def scenario_analyzer(
    run: Run,
    analyzer: Path,
    name: str,
    *,
    member_b: str,
    warm: bool,
    toolchain: str = PIN,
    expect_built: int,
    note: str,
) -> Outcome:
    """One rust-analyzer proc-macro server expanding both consumer crates.

    `analysis-stats` loads the project and expands once per invocation, in one
    long-lived `rust-analyzer-proc-macro-srv` process. "Cold" and "warm" here are
    properties of the *cache*, not of a target directory: the analyzer keeps no
    Cargo fingerprint, so it re-expands on every run regardless.
    """
    if not warm:
        shutil.rmtree(run.cache, ignore_errors=True)
    run.reset_events()
    server = proc_macro_srv(toolchain)
    require(server.exists(), f"{name}: no proc-macro server at {server}")
    env = run.environment(member_a=MEMBER_ONE, member_b=member_b)
    env["RUSTUP_TOOLCHAIN"] = toolchain
    started = time.monotonic()
    result = subprocess.run(
        [str(analyzer), "analysis-stats", "--proc-macro-srv", str(server), "."],
        cwd=WORKSPACE,
        env=env,
        capture_output=True,
        text=True,
        timeout=DEFAULT_DEADLINE,
        check=False,
    )
    outcome = Outcome(
        name,
        driver="rust-analyzer",
        toolchain=toolchain,
        crates=2,
        events=run.read_events(),
        seconds=time.monotonic() - started,
        note=note,
    )
    require(result.returncode == 0, f"{name}: the analyzer failed\n{result.stderr[-2000:]}")
    require(
        len(outcome.events) == INVOCATIONS * 2,
        f"{name}: {len(outcome.events)} expansions, expected {INVOCATIONS * 2}",
    )
    require(
        outcome.drivers == {"rust-analyzer-proc-macro-srv"},
        f"{name}: expansion must happen in the proc-macro server, saw {outcome.drivers}",
    )
    require(
        len(outcome.processes) == 1,
        f"{name}: one long-lived server should expand every invocation",
    )
    require(
        outcome.built == expect_built,
        f"{name}: expected {expect_built} envelope read(s), saw {outcome.built}",
    )
    outcome.payload_bytes = max(event["bytes"] for event in outcome.events)
    return outcome


def scenario_standalone(run: Run, oracle: tuple[int, int]) -> tuple[Outcome, Path]:
    """The whole of item 1: rustc's own expansion, rebuilt with everything gone.

    `-Zunpretty=expanded` is rustc rendering the tokens it was handed, so the
    byte literal in this file is the one the macro emitted rather than one this
    driver reconstructed. The generated crate declares no dependency, names no
    proc macro, and is then built from an empty target directory with every
    Tiler-produced file — every envelope, every sidecar, and the whole cache
    root — deleted from the filesystem.
    """
    run.reset_events()
    touch_sources()
    started = time.monotonic()
    env = run.environment()
    result = subprocess.run(
        [
            "cargo",
            f"+{PIN}",
            "rustc",
            "--quiet",
            "--offline",
            "-p",
            CONSUMERS[0],
            "--target-dir",
            str(run.target("expand")),
            "--",
            "-Zunpretty=expanded",
        ],
        cwd=WORKSPACE,
        env=env,
        capture_output=True,
        text=True,
        timeout=DEFAULT_DEADLINE,
        check=False,
    )
    require(result.returncode == 0, f"standalone: expansion failed\n{result.stderr[-3000:]}")
    expanded = result.stdout
    reasons = expansion_is_self_contained(expanded, run.root)
    require(not reasons, "standalone: the expansion is not self-contained: " + "; ".join(reasons))
    literal = byte_literal(expanded)

    crate = run.root / "standalone"
    shutil.rmtree(crate, ignore_errors=True)
    (crate / "src").mkdir(parents=True)
    (crate / "Cargo.toml").write_text(
        "[package]\n"
        'name = "embed-standalone"\n'
        'version = "0.0.0"\n'
        'edition = "2024"\n'
        "publish = false\n\n"
        "[[bin]]\n"
        'name = "embed-standalone"\n'
        'path = "src/main.rs"\n\n'
        "# Deliberately empty. The crate's whole claim is that it needs nothing.\n"
        "[dependencies]\n"
    )
    # rustc emits its own `#![feature(prelude_import)]` first; replace that line
    # with the complete set the rendered `std` macro expansions need.
    body = expanded.split("\n", 1)[1]
    (crate / "src" / "main.rs").write_text(f"#![feature({EXPANDED_FEATURES})]\n{body}")

    # Everything Tiler produced, gone — and proved gone before the build runs.
    removed = delete_tree_provably("standalone-artifacts", run.artifacts)
    removed += delete_tree_provably("standalone-cache", run.cache)

    build = subprocess.run(
        [
            "cargo",
            f"+{PIN}",
            "run",
            "--quiet",
            "--offline",
            "--target-dir",
            str(run.target("standalone")),
        ],
        cwd=crate,
        capture_output=True,
        text=True,
        timeout=DEFAULT_DEADLINE,
        check=False,
    )
    require(
        build.returncode == 0,
        f"standalone: the dependency-free crate failed to build or run\n{build.stderr[-3000:]}",
    )
    outcome = Outcome(
        "standalone-cold-everything-deleted",
        events=run.read_events(),
        seconds=time.monotonic() - started,
    )
    require(
        len(outcome.events) == INVOCATIONS,
        "standalone: exactly the one expansion that produced the source should be recorded",
    )
    outcome.payload_bytes = require_carries(build.stdout.strip(), "a", oracle)
    outcome.note = (
        f"{removed} Tiler-produced file(s) deleted; one byte-string literal of "
        f"{len(literal)} source byte(s) for {outcome.payload_bytes} payload byte(s); "
        "no dependency, no proc macro"
    )
    return outcome, crate / "src" / "main.rs"


# -- diagnostics --------------------------------------------------------------


def rendered_diagnostic(run: Run, label: str, env: dict[str, str]) -> str:
    """Builds under a deliberately broken input and returns rustc's exact text.

    Every class here must *fail*. A refusal that cannot be reached is a refusal
    no reader should believe, which is why the ceiling is overridable and why
    the invalid-artifact class truncates a real envelope rather than being
    asserted from the type.
    """
    touch_sources()
    result = cargo_build(run, "diagnostic", env=env, check=False, label=label)
    require(
        result.returncode != 0,
        f"diagnostic {label}: the build succeeded, so this failure class is unreachable "
        "and the text below would describe nothing",
    )
    # Verbatim from the first `error` line onward. An earlier version filtered
    # by line shape and silently dropped the numbered source line, so the
    # recorded text showed a caret under nothing — a rendering no consumer would
    # ever see, published as though it were the one they do.
    lines = result.stderr.splitlines()
    start = next(
        (index for index, line in enumerate(lines) if line.startswith("error")),
        None,
    )
    require(start is not None, f"diagnostic {label}: cargo failed without an `error` line")
    assert start is not None
    return "\n".join(lines[start:]).rstrip()


def collect_diagnostics(run: Run) -> dict[str, str]:
    """The exact rendered text a consumer sees, one entry per failure class."""
    produce_artifacts(run)
    shutil.rmtree(run.cache, ignore_errors=True)
    rendered = {}

    missing_directory = run.environment()
    missing_directory.pop("TILER_EMBED_DIR")
    rendered["directory-unstated"] = rendered_diagnostic(
        run, "directory-unstated", missing_directory
    )

    missing_slot = run.environment()
    missing_slot.pop("TILER_EMBED_MEMBER_A")
    rendered["slot-unstated"] = rendered_diagnostic(run, "slot-unstated", missing_slot)

    missing_cache = run.environment()
    missing_cache.pop("TILER_EMBED_CACHE")
    rendered["cache-root-unstated"] = rendered_diagnostic(
        run, "cache-root-unstated", missing_cache
    )

    rendered["cache-root-relative"] = rendered_diagnostic(
        run, "cache-root-relative", run.environment(cache="relative/cache")
    )

    # Both the envelope and any cache entry standing in for it are gone, which
    # is the one state in which an expansion genuinely cannot proceed.
    absent = run.root / "absent-artifacts"
    shutil.rmtree(run.cache, ignore_errors=True)
    rendered["member-unavailable"] = rendered_diagnostic(
        run, "member-unavailable", run.environment(directory=str(absent))
    )

    # A real envelope truncated to half its length: the cache's own validator
    # refuses it, so this class proves the validation is not decorative.
    truncated = run.root / "truncated"
    truncated.mkdir(parents=True, exist_ok=True)
    whole = (run.artifacts / MEMBER_ONE).read_bytes()
    (truncated / MEMBER_ONE).write_bytes(whole[: len(whole) // 2])
    shutil.rmtree(run.cache, ignore_errors=True)
    rendered["invalid-artifact"] = rendered_diagnostic(
        run, "invalid-artifact", run.environment(directory=str(truncated))
    )

    shutil.rmtree(run.cache, ignore_errors=True)
    rendered["ceiling-exceeded"] = rendered_diagnostic(
        run, "ceiling-exceeded", run.environment(ceiling=1024)
    )
    return rendered


# -- entry point --------------------------------------------------------------


def environment_note(toolchains: dict[str, str]) -> str:
    versions = " ".join(f"{name}={version}" for name, version in toolchains.items())
    return (
        f"host={platform.platform()} machine={platform.machine()} "
        f"cpus={os.cpu_count()} {versions}"
    )


def rustc_version(toolchain: str) -> str:
    result = subprocess.run(
        ["rustc", f"+{toolchain}", "--version"],
        capture_output=True,
        text=True,
        check=False,
    )
    return result.stdout.strip() or "unavailable"


def analyzer_binary(explicit: str | None) -> Path | None:
    """Finds a full `rust-analyzer`, which is not the same thing as a proxy.

    `shutil.which` resolves the rustup shim, and the shim resolves *this
    directory's* toolchain — the pin, which declares `profile = "minimal"` and
    carries no analyzer, so the shim exists and fails when run. Every installed
    toolchain is asked instead, and only a real binary is returned. The
    expanding process is the pin's [`PIN_SRV`] either way; the LSP half is what
    varies, and the run records which binary it was.
    """
    if explicit:
        path = Path(explicit)
        return path if path.exists() else None
    listed = subprocess.run(
        ["rustup", "toolchain", "list"], capture_output=True, text=True, check=False
    )
    for line in listed.stdout.splitlines():
        toolchain = line.split()[0] if line.split() else ""
        if not toolchain:
            continue
        found = subprocess.run(
            ["rustup", "which", "--toolchain", toolchain, "rust-analyzer"],
            capture_output=True,
            text=True,
            check=False,
        )
        candidate = Path(found.stdout.strip()) if found.returncode == 0 else None
        if candidate is not None and candidate.exists():
            return candidate
    return None


def check_declared_invocations() -> None:
    for consumer in ("consumer-a", "consumer-b"):
        source = (WORKSPACE / consumer / "src" / "main.rs").read_text()
        if f"INVOCATIONS: usize = {INVOCATIONS};" not in source:
            raise ScenarioFailure(
                f"{consumer}'s declared invocation count and this driver's have drifted"
            )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", default=None, help="run directory; a fresh one is made by default")
    parser.add_argument("--analyzer", default=None, help="path to a rust-analyzer binary")
    parser.add_argument("--skip-analyzer", action="store_true")
    parser.add_argument("--record", default=None, help="write result fixtures with this label")
    parser.add_argument("--keep-root", action="store_true", help="do not remove the run root")
    arguments = parser.parse_args()

    try:
        check_declared_invocations()
    except ScenarioFailure as failure:
        print(f"FAILED: {failure}", file=sys.stderr)
        return 2

    root = (
        Path(arguments.root).resolve()
        if arguments.root
        else Path(os.environ.get("TMPDIR", "/tmp")).resolve()
        / f"tiler-embedding-self-contained-{os.getpid()}"
    )
    shutil.rmtree(root, ignore_errors=True)
    root.mkdir(parents=True)
    run = Run(root)

    analyzer = None
    if not arguments.skip_analyzer:
        analyzer = analyzer_binary(arguments.analyzer)
        if analyzer is None:
            print(
                "FAILED: no rust-analyzer binary. The pin declares `profile = \"minimal\"` and "
                "ships none; installing one would be a toolchain mutation this driver will not "
                "make. Pass --analyzer <path>, or --skip-analyzer and record the analyzer axes "
                "as unreached boundaries.",
                file=sys.stderr,
            )
            return 2
        if not PIN_SRV.exists():
            print(f"no pinned proc-macro server at {PIN_SRV}", file=sys.stderr)
            return 2

    toolchains = {PIN: rustc_version(PIN), ALTERNATE: rustc_version(ALTERNATE)}
    if toolchains[ALTERNATE] == "unavailable":
        print(
            f"FAILED: the toolchain-change axis needs `{ALTERNATE}` installed; this driver never "
            "installs one. Record that axis as an unreached boundary instead.",
            file=sys.stderr,
        )
        return 2

    print(f"# {environment_note(toolchains)}", flush=True)
    print(f"# analyzer={analyzer or 'skipped'} proc_macro_srv={PIN_SRV}", flush=True)
    print(f"# run_root={root}", flush=True)

    outcomes: list[Outcome] = []
    diagnostics: dict[str, str] = {}
    sizes: dict[str, int] = {}
    expanded_path: Path | None = None
    try:
        ensure_lockfile()
        sizes = produce_artifacts(run)
        oracle_bytes = (run.artifacts / MEMBER_ONE).read_bytes()
        oracle = (len(oracle_bytes), fnv1a(oracle_bytes))
        members = " ".join(f"{name}={size}B" for name, size in sizes.items())
        print(f"# members {members} ceiling={CEILING_BYTES}B", flush=True)
        prebuild(run, ("main", "cross", "diagnostic", "expand"))
        print(HEADER, flush=True)

        def record(outcome: Outcome) -> None:
            outcomes.append(outcome)
            print(outcome.row(), flush=True)

        record(scenario_cold(run, oracle))
        record(scenario_warm(run, oracle))
        record(scenario_artifacts_deleted(run, oracle))
        record(scenario_cache_deleted_no_reexpand(run, oracle))
        record(scenario_cache_deleted_reexpand(run, oracle))
        for outcome in scenario_axis_source_edit(run, oracle):
            record(outcome)
        record(scenario_axis_toolchain(run, oracle))
        record(scenario_axis_cross_crate(run, oracle, repeated=True))
        record(scenario_axis_cross_crate(run, oracle, repeated=False))
        if analyzer is not None:
            record(
                scenario_analyzer(
                    run,
                    analyzer,
                    "axis-analyzer-repeated-cold",
                    member_b=MEMBER_ONE,
                    warm=False,
                    expect_built=1,
                    note="one server, two crates, one envelope read",
                )
            )
            record(
                scenario_analyzer(
                    run,
                    analyzer,
                    "axis-analyzer-repeated-warm",
                    member_b=MEMBER_ONE,
                    warm=True,
                    expect_built=0,
                    note="both invocations hit; no envelope read",
                )
            )
            record(
                scenario_analyzer(
                    run,
                    analyzer,
                    "axis-analyzer-unique-cold",
                    member_b=MEMBER_TWO,
                    warm=False,
                    expect_built=2,
                    note="two envelopes, two reads, one server process",
                )
            )
            record(
                scenario_analyzer(
                    run,
                    analyzer,
                    "axis-analyzer-toolchain-change",
                    member_b=MEMBER_TWO,
                    warm=True,
                    toolchain=ALTERNATE,
                    expect_built=0,
                    note=f"{PIN} -> {ALTERNATE} server; entries published by the pin still hit",
                )
            )
        produce_artifacts(run)
        standalone, expanded_path = scenario_standalone(run, oracle)
        record(standalone)
        diagnostics = collect_diagnostics(run)
    except ScenarioFailure as failure:
        print(f"FAILED: {failure}", file=sys.stderr)
        return 1
    finally:
        touch_sources()

    print("", flush=True)
    for label, text in diagnostics.items():
        print(f"## {label}\n{text}\n", flush=True)

    if arguments.record:
        RESULTS.mkdir(parents=True, exist_ok=True)
        fixture = RESULTS / f"self-contained-embedding-{arguments.record}.tsv"
        expanded_bytes = expanded_path.stat().st_size if expanded_path else 0
        members = " ".join(f"{name}={size}B" for name, size in sizes.items())
        fixture.write_text(
            "\n".join(
                [
                    f"# {environment_note(toolchains)}",
                    f"# analyzer={analyzer or 'skipped'} proc_macro_srv={PIN_SRV}",
                    f"# members {members} ceiling={CEILING_BYTES}B "
                    f"standalone_crate_source={expanded_bytes}B",
                    HEADER,
                    *(outcome.row() for outcome in outcomes),
                ]
            )
            + "\n"
        )
        print(f"# recorded {fixture}", flush=True)

        text = RESULTS / f"self-contained-diagnostics-{arguments.record}.txt"
        text.write_text(
            "\n".join(
                [
                    f"# {environment_note(toolchains)}",
                    "# Exact rendered text, one block per failure class. Every class was reached",
                    "# by a build that had to fail; a class whose build succeeded fails the run.",
                    "",
                    *(f"## {label}\n{body}\n" for label, body in diagnostics.items()),
                ]
            )
        )
        print(f"# recorded {text}", flush=True)

    if not arguments.keep_root:
        shutil.rmtree(root, ignore_errors=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
