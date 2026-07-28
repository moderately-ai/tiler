"""Drive the expansion cache with the tools that will actually drive it.

ADR 0050's context sentence is that "Cargo and rust-analyzer may run equivalent
proc-macro expansions concurrently". Every cache measurement before this one used
a harness that calls the cache directly, which is a *model* of that workload. This
driver runs the workload: a real proc macro, expanded by real `cargo` and by a
real `rust-analyzer` proc-macro server, against one shared cache root.

Inputs
------
The Cargo workspace at ``spikes/cache/build-tool-exercise``. Its ``consumer``
crate performs ``INVOCATIONS`` ``resolve!`` expansions per build, over that many
distinct cache keys.

Outputs
-------
One TSV row per scenario on stdout, and with ``--record`` a result fixture under
``spikes/cache/results/``.

Metrics
-------
``builds`` — how many times the expensive closure ran. This is the quantity the
per-key lock exists to suppress, and the only one that distinguishes a working
protocol from a broken one. ``published``/``hit``/``uncached`` are the cache's
own report of each resolution.

Stop conditions
---------------
Every subprocess runs under a deadline. Every scenario declares the exact number
of events it must observe and fails when the count differs, so a scenario that
expanded nothing cannot report success.

Unsupported here
----------------
Multi-editor concurrency, a real LSP session with incremental edits, and
cancellation driven by the editor rather than by a signal. See the research note
for what each would need.
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import shutil
import signal
import subprocess
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path

SPIKE = Path(__file__).resolve().parent
WORKSPACE = SPIKE / "build-tool-exercise"
RESULTS = SPIKE / "results"

#: Invocations one build of the consumer crate performs. Mirrors
#: ``consumer/src/lib.rs``'s ``INVOCATIONS`` and is checked against it, so the
#: two cannot drift apart silently.
INVOCATIONS = 4

TOOLCHAIN = "nightly-2026-07-19"
PIN_SRV = Path(
    f"/Users/tsanterre/.rustup/toolchains/{TOOLCHAIN}-aarch64-apple-darwin"
    "/libexec/rust-analyzer-proc-macro-srv"
)

DEFAULT_DEADLINE = 900


class ScenarioFailure(RuntimeError):
    """A scenario observed something other than what it declared it must."""


@dataclass
class Outcome:
    """What one scenario observed."""

    name: str
    events: list[dict] = field(default_factory=list)
    seconds: float = 0.0
    note: str = ""

    @property
    def builds(self) -> int:
        return sum(1 for event in self.events if event["built"])

    def count(self, outcome: str) -> int:
        return sum(1 for event in self.events if event["outcome"] == outcome)

    @property
    def drivers(self) -> set[str]:
        return {event["driver"] for event in self.events}

    @property
    def processes(self) -> set[int]:
        return {event["pid"] for event in self.events}

    @property
    def working_dirs(self) -> set[str]:
        """The distinct working directories the expansions ran in.

        One value means every driver in the scenario expanded from the same
        directory, which is the ticket's "whether they share a working
        directory" asked of the processes rather than of their launcher.
        """
        return {event["cwd"] for event in self.events}

    @property
    def cross_process_overlaps(self) -> int:
        """Counts expansion windows in different processes that truly intersect.

        Concurrency has to be *observed*, not assumed from the fact that several
        processes were launched together. Three builds that happened to
        serialize and three that genuinely raced produce identical outcome
        counts, so without this the scenario would claim a workload it had not
        reached.
        """
        overlaps = 0
        events = self.events
        for index, left in enumerate(events):
            for right in events[index + 1 :]:
                if left["pid"] == right["pid"]:
                    continue
                starts_first = left["started_ns"] < right["ended_ns"]
                ends_after = right["started_ns"] < left["ended_ns"]
                if starts_first and ends_after:
                    overlaps += 1
        return overlaps

    def row(self) -> str:
        return "\t".join(
            (
                self.name,
                str(len(self.events)),
                str(self.builds),
                str(self.count("published")),
                str(self.count("hit")),
                str(self.count("uncached")),
                str(len(self.processes)),
                str(self.cross_process_overlaps),
                str(len(self.working_dirs)),
                ",".join(sorted(self.drivers)) or "-",
                f"{self.seconds:.1f}",
                self.note,
            )
        )


def read_events(root: Path) -> list[dict]:
    """Reads every recorded event.

    One file per event, so the population is countable. An interleaved append
    log could lose a record without any reader noticing, and a count that can be
    silently short is exactly what makes a uniform pass untrustworthy.
    """
    events = root / "events"
    if not events.is_dir():
        return []
    records = []
    for path in sorted(events.iterdir()):
        if path.suffix != ".json":
            continue
        records.append(json.loads(path.read_text()))
    return records


def reset_events(root: Path) -> None:
    shutil.rmtree(root / "events", ignore_errors=True)
    shutil.rmtree(root / "markers", ignore_errors=True)
    (root / "release").unlink(missing_ok=True)


def await_marker(root: Path, alive: subprocess.Popen, deadline: float = 300.0) -> Path:
    """Waits until some expansion announces that it holds a key lock.

    This is the whole ordering mechanism of the scenarios that need one. The
    expansion writes its marker *before* it waits and removes it after, so a
    driver learns the lock is held by observing state rather than by assuming a
    wall-clock margin was enough — the defect
    `remove-the-wall-clock-race-from-the-cache-kill-harness` is fixing in the
    in-crate harness.
    """
    markers = root / "markers"
    limit = time.monotonic() + deadline
    while time.monotonic() < limit:
        if markers.is_dir():
            seen = [path for path in markers.iterdir() if path.suffix == ".building"]
            if seen:
                return seen[0]
        if alive.poll() is not None:
            raise ScenarioFailure("the process exited before announcing a held lock")
        time.sleep(0.02)
    raise ScenarioFailure("no expansion announced a held lock within the deadline")


def reset_cache(root: Path) -> None:
    shutil.rmtree(root / "cache", ignore_errors=True)


def touch_sources() -> None:
    """Makes Cargo consider the consumer crate dirty.

    Cargo does not know the macro consulted a cache, so nothing about a changed
    cache root invalidates a fingerprint. Re-expansion has to be forced through
    the input Cargo *does* track.
    """
    (WORKSPACE / "consumer" / "src" / "lib.rs").touch()


def cargo_command(target: Path) -> list[str]:
    return [
        "cargo",
        f"+{TOOLCHAIN}",
        "build",
        "-p",
        "exercise-consumer",
        "--target-dir",
        str(target),
    ]


def cargo_environment(root: Path, delay_ms: int = 0) -> dict[str, str]:
    env = dict(os.environ)
    env["TILER_EXERCISE_ROOT"] = str(root)
    env["TILER_EXERCISE_BUILD_DELAY_MS"] = str(delay_ms)
    env.pop("RUSTC_WRAPPER", None)
    return env


def run_cargo(root: Path, target: Path, delay_ms: int = 0, deadline: int = DEFAULT_DEADLINE):
    return subprocess.run(
        cargo_command(target),
        cwd=WORKSPACE,
        env=cargo_environment(root, delay_ms),
        capture_output=True,
        text=True,
        timeout=deadline,
        check=False,
    )


def prebuild(targets: list[Path], root: Path) -> None:
    """Warms each target directory so a measured run compiles only the consumer.

    Done against a throwaway cache root that is discarded afterwards, so the
    warming does not populate the root a scenario is about to judge.
    """
    warm = root / "warmup"
    processes = [
        subprocess.Popen(
            cargo_command(target),
            cwd=WORKSPACE,
            env=cargo_environment(warm),
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        for target in targets
    ]
    for process in processes:
        process.wait(timeout=DEFAULT_DEADLINE)
    shutil.rmtree(warm, ignore_errors=True)


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ScenarioFailure(message)


def expect_events(outcome: Outcome, expected: int) -> None:
    require(
        len(outcome.events) == expected,
        f"{outcome.name}: expected exactly {expected} events, observed {len(outcome.events)}",
    )


# -- scenarios ---------------------------------------------------------------


def scenario_cold(root: Path, target: Path) -> Outcome:
    """One Cargo build against an empty cache."""
    reset_cache(root)
    reset_events(root)
    touch_sources()
    started = time.monotonic()
    result = run_cargo(root, target)
    outcome = Outcome("cargo-cold", read_events(root), time.monotonic() - started)
    require(result.returncode == 0, f"cargo-cold: cargo failed\n{result.stderr[-2000:]}")
    expect_events(outcome, INVOCATIONS)
    require(
        outcome.count("published") == INVOCATIONS,
        f"cargo-cold: every key should publish, saw {outcome.count('published')}",
    )
    require(outcome.builds == INVOCATIONS, "cargo-cold: every key should compile once")
    return outcome


def scenario_warm(root: Path, target: Path) -> Outcome:
    """A second Cargo build against the cache the first one populated."""
    reset_events(root)
    touch_sources()
    started = time.monotonic()
    result = run_cargo(root, target)
    outcome = Outcome("cargo-warm", read_events(root), time.monotonic() - started)
    require(result.returncode == 0, f"cargo-warm: cargo failed\n{result.stderr[-2000:]}")
    expect_events(outcome, INVOCATIONS)
    require(
        outcome.count("hit") == INVOCATIONS,
        f"cargo-warm: every key should hit, saw {outcome.count('hit')}",
    )
    require(outcome.builds == 0, f"cargo-warm: nothing should compile, {outcome.builds} did")
    return outcome


def scenario_concurrent_cargo(root: Path, targets: list[Path]) -> Outcome:
    """Several uncoordinated Cargo builds racing on one cache root.

    Separate target directories, because Cargo takes an exclusive lock on one:
    two builds sharing a target directory serialize and would measure nothing
    about the cache's own exclusion.
    """
    reset_cache(root)
    reset_events(root)
    touch_sources()
    started = time.monotonic()
    processes = [
        subprocess.Popen(
            cargo_command(target),
            cwd=WORKSPACE,
            # A build window wide enough that overlap is unambiguous when it
            # happens. Without it the compile is a few milliseconds and three
            # racing builds may finish in sequence by luck, leaving the scenario
            # unable to say whether it reached concurrency at all.
            env=cargo_environment(root, delay_ms=400),
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
        )
        for target in targets
    ]
    for process in processes:
        process.wait(timeout=DEFAULT_DEADLINE)
    outcome = Outcome(
        f"cargo-concurrent-x{len(targets)}",
        read_events(root),
        time.monotonic() - started,
    )
    for process in processes:
        require(process.returncode == 0, "cargo-concurrent: a build failed")
    expect_events(outcome, INVOCATIONS * len(targets))
    require(
        outcome.builds == INVOCATIONS,
        f"cargo-concurrent: the lock should leave one compile per key, saw {outcome.builds}",
    )
    require(
        len(outcome.processes) == len(targets),
        "cargo-concurrent: each build should expand in its own process",
    )
    require(
        outcome.cross_process_overlaps > 0,
        "cargo-concurrent: no two expansions in different processes overlapped, so this "
        "scenario did not reach the concurrent workload it claims to measure",
    )
    outcome.note = f"{len(outcome.processes)} rustc processes"
    return outcome


def scenario_no_cache(root: Path, targets: list[Path]) -> Outcome:
    """The negative control: the same race with the cache made unusable.

    This scenario exists so that the previous one means something. If the
    driver could not observe duplicate compilation, "one compile per key" would
    be a result the harness produces regardless of what the cache does. Here the
    root is a *file*, so no namespace can be created, every resolution falls open
    to `uncached`, and every process compiles every key.
    """
    unusable = root / "unusable-root"
    shutil.rmtree(unusable, ignore_errors=True)
    unusable.parent.mkdir(parents=True, exist_ok=True)
    unusable.write_text("not a directory")

    reset_events(root)
    touch_sources()
    started = time.monotonic()
    processes = [
        subprocess.Popen(
            cargo_command(target),
            cwd=WORKSPACE,
            # The events still land under `root`; only `cache` is unusable.
            env={
                **cargo_environment(root, delay_ms=400),
                "TILER_EXERCISE_ROOT": str(root),
                "TILER_EXERCISE_CACHE_OVERRIDE": str(unusable),
            },
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
        )
        for target in targets
    ]
    for process in processes:
        process.wait(timeout=DEFAULT_DEADLINE)
    outcome = Outcome(
        f"negative-control-x{len(targets)}",
        read_events(root),
        time.monotonic() - started,
    )
    expect_events(outcome, INVOCATIONS * len(targets))
    require(
        outcome.builds == INVOCATIONS * len(targets),
        "negative-control: with no usable cache every process must compile every key; "
        f"saw {outcome.builds} of {INVOCATIONS * len(targets)}",
    )
    require(
        outcome.count("uncached") == INVOCATIONS * len(targets),
        "negative-control: every resolution must fall open to uncached",
    )
    require(
        outcome.cross_process_overlaps > 0,
        "negative-control: the duplicate work must be concurrent to be the control "
        "for the concurrent scenario",
    )
    outcome.note = "duplicate work is observable"
    return outcome


def scenario_analyzer(root: Path, analyzer: Path) -> Outcome:
    """A real rust-analyzer proc-macro server expanding against the same root."""
    reset_events(root)
    started = time.monotonic()
    env = cargo_environment(root)
    env["RUSTUP_TOOLCHAIN"] = TOOLCHAIN
    result = subprocess.run(
        [
            str(analyzer),
            "analysis-stats",
            "--proc-macro-srv",
            str(PIN_SRV),
            ".",
        ],
        cwd=WORKSPACE,
        env=env,
        capture_output=True,
        text=True,
        timeout=DEFAULT_DEADLINE,
        check=False,
    )
    outcome = Outcome("analyzer", read_events(root), time.monotonic() - started)
    require(result.returncode == 0, f"analyzer: run failed\n{result.stderr[-2000:]}")
    expect_events(outcome, INVOCATIONS)
    require(
        outcome.drivers == {"rust-analyzer-proc-macro-srv"},
        f"analyzer: expansion should happen in the proc-macro server, saw {outcome.drivers}",
    )
    require(
        len(outcome.processes) == 1,
        "analyzer: one long-lived server should expand every invocation",
    )
    outcome.note = "one server process, every invocation"
    return outcome


def scenario_killed_writer(root: Path, target: Path) -> Outcome:
    """Kill a Cargo build while its expansion holds a key lock.

    Ordering is established by observed state, never by a wall-clock margin: the
    build announces itself by creating a marker file *before* it waits, and this
    scenario kills only once that file exists. `harness.rs`'s known defect is a
    50 ms delay used for the same purpose, and this deliberately does not repeat
    it.
    """
    reset_cache(root)
    reset_events(root)
    touch_sources()
    started = time.monotonic()

    process = subprocess.Popen(
        cargo_command(target),
        cwd=WORKSPACE,
        env=cargo_environment(root, delay_ms=60_000),
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True,
    )
    await_marker(root, process)

    # SIGKILL the whole process group, so no destructor runs anywhere: the lock
    # must be released by the operating system closing the descriptor, which is
    # the only recovery rule ADR 0050 relies on.
    os.killpg(os.getpgid(process.pid), signal.SIGKILL)
    process.wait(timeout=60)
    killed_events = len(read_events(root))

    # The survivor must be able to take the same key's lock and publish.
    reset_events(root)
    touch_sources()
    result = run_cargo(root, target)
    outcome = Outcome("killed-writer", read_events(root), time.monotonic() - started)
    require(result.returncode == 0, f"killed-writer: the survivor failed\n{result.stderr[-2000:]}")
    expect_events(outcome, INVOCATIONS)
    require(
        outcome.count("published") + outcome.count("hit") == INVOCATIONS,
        "killed-writer: the survivor must resolve every key",
    )
    require(
        outcome.builds > 0,
        "killed-writer: the killed writer published nothing, so the survivor must compile",
    )
    outcome.note = f"killed after {killed_events} completed expansions"
    return outcome


def start_analyzer(root: Path, analyzer: Path, delay_ms: int) -> subprocess.Popen:
    env = cargo_environment(root, delay_ms)
    env["RUSTUP_TOOLCHAIN"] = TOOLCHAIN
    return subprocess.Popen(
        [str(analyzer), "analysis-stats", "--proc-macro-srv", str(PIN_SRV), "."],
        cwd=WORKSPACE,
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True,
    )


def scenario_interleaved(root: Path, target: Path, analyzer: Path) -> Outcome:
    """A Cargo build and an analyzer server resolving one key at the same time.

    ADR 0050's context sentence taken literally. The two processes share a cache
    root and were never told about each other. Ordering is by observed state: the
    analyzer is allowed to reach an expansion and take a key lock, and only once
    it says so does Cargo start, which is what makes the overlap a fact rather
    than a hope. The analyzer loads its whole crate graph first and Cargo does
    not, so starting them together would reliably *miss* the overlap.
    """
    reset_cache(root)
    reset_events(root)
    touch_sources()
    started = time.monotonic()

    # The analyzer holds each key long enough for Cargo to start, reach its own
    # expansion, and block on the same lock. The window is generous rather than
    # tight because it only has to make the overlap *likely* — the overlap is
    # then *proven* from the recorded windows below, so an inadequate window
    # fails the scenario rather than silently passing it. An earlier version
    # released the analyzer as soon as Cargo was launched, and this check caught
    # that Cargo had not yet reached its expansion.
    analyzer_process = start_analyzer(root, analyzer, delay_ms=10_000)
    await_marker(root, analyzer_process)

    cargo_process = subprocess.Popen(
        cargo_command(target),
        cwd=WORKSPACE,
        env=cargo_environment(root),
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    cargo_process.wait(timeout=DEFAULT_DEADLINE)
    analyzer_process.wait(timeout=DEFAULT_DEADLINE)

    outcome = Outcome("cargo-and-analyzer", read_events(root), time.monotonic() - started)
    expect_events(outcome, INVOCATIONS * 2)
    require(
        outcome.builds == INVOCATIONS,
        f"cargo-and-analyzer: one compile per key across both tools, saw {outcome.builds}",
    )
    require(
        len(outcome.drivers) == 2,
        f"cargo-and-analyzer: both drivers should appear, saw {outcome.drivers}",
    )
    require(
        outcome.cross_process_overlaps > 0,
        "cargo-and-analyzer: the two tools' expansions never overlapped",
    )
    shared = "one working directory" if len(outcome.working_dirs) == 1 else "separate directories"
    outcome.note = f"{'+'.join(sorted(outcome.drivers))}; {shared}"
    return outcome


def scenario_killed_analyzer(root: Path, target: Path, analyzer: Path) -> Outcome:
    """Kill the proc-macro server while its expansion holds a key lock.

    This is the ticket's second obligation: the per-key lock's holder is a
    proc-macro server an editor may kill and restart at any moment. ADR 0050
    relies on exactly one recovery rule — the operating system releases the lock
    when the last descriptor closes — and there is no stale-owner rule to fall
    back on, so a server killed mid-publication either releases the lock or
    wedges the key permanently for every other process on the host.

    `SIGKILL` to the process group, so no destructor runs and no descriptor is
    closed deliberately.
    """
    reset_cache(root)
    reset_events(root)
    touch_sources()
    started = time.monotonic()

    analyzer_process = start_analyzer(root, analyzer, delay_ms=120_000)
    await_marker(root, analyzer_process)
    os.killpg(os.getpgid(analyzer_process.pid), signal.SIGKILL)
    analyzer_process.wait(timeout=120)
    before = len(read_events(root))

    # The survivor must be able to take the same key's lock and publish. If the
    # killed server's lock had leaked, this build would block until the deadline.
    reset_events(root)
    touch_sources()
    result = run_cargo(root, target)
    outcome = Outcome("analyzer-killed-holding-lock", read_events(root), time.monotonic() - started)
    require(
        result.returncode == 0,
        f"analyzer-killed: the survivor failed\n{result.stderr[-2000:]}",
    )
    expect_events(outcome, INVOCATIONS)
    require(
        outcome.count("published") + outcome.count("hit") == INVOCATIONS,
        "analyzer-killed: the survivor must resolve every key",
    )
    outcome.note = f"server killed after {before} completed expansions; lock released"
    return outcome


# -- entry point -------------------------------------------------------------


def environment_note() -> str:
    rustc = subprocess.run(
        ["rustc", f"+{TOOLCHAIN}", "--version"],
        capture_output=True,
        text=True,
        check=False,
    ).stdout.strip()
    return (
        f"host={platform.platform()} machine={platform.machine()} "
        f"cpus={os.cpu_count()} rustc={rustc}"
    )


def analyzer_binary(explicit: str | None) -> Path | None:
    if explicit:
        return Path(explicit)
    found = shutil.which("rust-analyzer")
    return Path(found) if found else None


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", default=None, help="state directory for the run")
    parser.add_argument("--concurrency", type=int, default=3)
    parser.add_argument("--analyzer", default=None, help="path to a rust-analyzer binary")
    parser.add_argument("--skip-analyzer", action="store_true")
    parser.add_argument("--record", default=None, help="write a result fixture with this label")
    arguments = parser.parse_args()

    declared = (WORKSPACE / "consumer" / "src" / "lib.rs").read_text()
    if f"INVOCATIONS: usize = {INVOCATIONS};" not in declared:
        print("the consumer's invocation count and this driver's have drifted", file=sys.stderr)
        return 2

    root = Path(arguments.root).resolve() if arguments.root else WORKSPACE / "exercise-state"
    root.mkdir(parents=True, exist_ok=True)
    targets = [root / f"target-{index}" for index in range(arguments.concurrency)]

    analyzer = None if arguments.skip_analyzer else analyzer_binary(arguments.analyzer)
    if analyzer is not None and not analyzer.exists():
        print(f"no rust-analyzer at {analyzer}", file=sys.stderr)
        return 2
    if analyzer is not None and not PIN_SRV.exists():
        print(f"no pinned proc-macro server at {PIN_SRV}", file=sys.stderr)
        return 2

    print(f"# {environment_note()}", flush=True)
    print(f"# analyzer={analyzer or 'skipped'} proc_macro_srv={PIN_SRV}", flush=True)
    print(
        "scenario\tevents\tbuilds\tpublished\thit\tuncached\tprocesses\toverlaps\tcwds\tdrivers"
        "\tseconds\tnote",
        flush=True,
    )

    prebuild(targets, root)

    outcomes: list[Outcome] = []
    try:
        outcomes.append(scenario_cold(root, targets[0]))
        print(outcomes[-1].row(), flush=True)
        outcomes.append(scenario_warm(root, targets[0]))
        print(outcomes[-1].row(), flush=True)
        outcomes.append(scenario_concurrent_cargo(root, targets))
        print(outcomes[-1].row(), flush=True)
        outcomes.append(scenario_no_cache(root, targets))
        print(outcomes[-1].row(), flush=True)
        outcomes.append(scenario_killed_writer(root, targets[0]))
        print(outcomes[-1].row(), flush=True)
        if analyzer is not None:
            outcomes.append(scenario_analyzer(root, analyzer))
            print(outcomes[-1].row(), flush=True)
            outcomes.append(scenario_interleaved(root, targets[0], analyzer))
            print(outcomes[-1].row(), flush=True)
            outcomes.append(scenario_killed_analyzer(root, targets[0], analyzer))
            print(outcomes[-1].row(), flush=True)
    except ScenarioFailure as failure:
        print(f"FAILED: {failure}", file=sys.stderr)
        return 1

    if arguments.record:
        RESULTS.mkdir(parents=True, exist_ok=True)
        fixture = RESULTS / f"build-tool-exercise-{arguments.record}.tsv"
        lines = [
            f"# {environment_note()}",
            f"# analyzer={analyzer or 'skipped'} proc_macro_srv={PIN_SRV}",
            f"# invocations_per_build={INVOCATIONS} concurrency={arguments.concurrency}",
            "scenario\tevents\tbuilds\tpublished\thit\tuncached\tprocesses\toverlaps\tcwds\tdrivers"
            "\tseconds\tnote",
            *(outcome.row() for outcome in outcomes),
        ]
        fixture.write_text("\n".join(lines) + "\n")
        print(f"# recorded {fixture}", flush=True)

    return 0


if __name__ == "__main__":
    sys.exit(main())
