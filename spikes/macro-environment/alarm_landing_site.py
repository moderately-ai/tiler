#!/usr/bin/env python3
"""Census where a harness's overall alarm lands relative to its capture pipes.

`capture` reports the same expired deadline with two different messages
depending on where the `SIGALRM` interrupts it. Inside `selector.select` the
handler's `ProbeFailure` is caught and re-raised as `command exceeded deadline`;
anywhere past the streaming loop it propagates unmodified as the harness's own
`overall deadline`. Which one a test observes is therefore a property of the
alarm's landing site, and this driver measures that site directly instead of
inferring it from the message.

- **`--construction pre-armed`** arms a fixed timer before the child is spawned,
  so the landing site is decided by a race against the child interpreter's
  startup. That is the construction this driver exists to disqualify: the
  quantity it races is set by the environment, not by the harness.
- **`--construction drain-armed`** arms the same timer from the `unregister`
  that empties the selector map, which is the moment both capture pipes reach
  end of file. `capture`'s loop condition is already false there, so no further
  `selector.select` call exists for the signal to land in.

Both constructions instrument the selector, so both also report the spawn-to-
drain latency the pre-armed margin has to out-run. Passing a different
`--interpreter` reproduces the ambient-`PATH` failure directly: a `python3` that
resolves to a pyenv shim starts far slower than the interpreter running the
tests.

    spikes/macro-environment/alarm_landing_site.py \
      --module spikes/macro-environment/probe.py --construction drain-armed
"""

from __future__ import annotations

import argparse
import collections
import importlib.util
import selectors
import statistics
import sys
import tempfile
import time
import traceback
from pathlib import Path

RECORD_PID = "import os, pathlib, sys\npathlib.Path(sys.argv[1]).write_text(str(os.getpid()))\n"
CLOSE_PIPES_AND_SLEEP = RECORD_PID + "import os, time\nos.close(1)\nos.close(2)\ntime.sleep(30)\n"
DEFAULT_MARGIN_SECONDS = 0.2
DEFAULT_TRIALS = 20


def load(path: Path) -> object:
    """Load one harness revision by path without importing its package."""
    spec = importlib.util.spec_from_file_location("harness_under_demonstration", path)
    if spec is None or spec.loader is None:
        raise SystemExit(f"cannot load harness: {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def instrumented_selector(
    module: object, drained: list[float], arm_after: float | None
) -> type[selectors.BaseSelector]:
    """Observe the drain, and on request arm the alarm from it rather than before it."""

    class Instrumented(selectors.DefaultSelector):
        def unregister(self, fileobj: object) -> selectors.SelectorKey:
            key = super().unregister(fileobj)
            if not self.get_map():
                drained.append(time.monotonic())
                if arm_after is not None:
                    module.signal.setitimer(module.signal.ITIMER_REAL, arm_after)
            return key

    return Instrumented


def landing_site(error: BaseException) -> str:
    """Name the call chain the alarm interrupted, dropping this driver's own frame."""
    frames = traceback.extract_tb(error.__traceback__)
    return " -> ".join(f"{Path(frame.filename).name}:{frame.name}" for frame in frames[1:])


def trial(
    module: object, interpreter: str, margin: float, pre_armed: bool
) -> tuple[str, str, float | None]:
    """Run one capture to its deadline and report site, message, and drain latency."""
    drained: list[float] = []
    original = selectors.DefaultSelector
    selectors.DefaultSelector = instrumented_selector(
        module, drained, None if pre_armed else margin
    )
    try:
        with tempfile.TemporaryDirectory(prefix="tiler-alarm-landing-") as scratch:
            command = [interpreter, "-c", CLOSE_PIPES_AND_SLEEP, str(Path(scratch) / "pid")]
            module.start_deadline()
            if pre_armed:
                module.signal.setitimer(module.signal.ITIMER_REAL, margin)
            spawned = time.monotonic()
            try:
                module.capture(command)
            except module.ProbeFailure as error:
                return landing_site(error), str(error), drained[0] - spawned if drained else None
            finally:
                module.signal.setitimer(module.signal.ITIMER_REAL, 0)
            return "NO FAILURE", "the harness returned successfully", None
    finally:
        selectors.DefaultSelector = original


def classify(message: str) -> str:
    """Reduce a failure to the class the assertion distinguishes, not its command spelling."""
    for known in ("overall deadline", "command exceeded deadline"):
        if known in message:
            return known
    return message


def report(name: str, census: collections.Counter[str]) -> None:
    for value, count in census.most_common():
        print(f"{count:4d}  {name}: {value}")


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--module", type=Path, required=True)
    parser.add_argument("--construction", choices=("pre-armed", "drain-armed"), required=True)
    parser.add_argument("--trials", type=int, default=DEFAULT_TRIALS)
    parser.add_argument("--interpreter", default=sys.executable)
    parser.add_argument("--margin", type=float, default=DEFAULT_MARGIN_SECONDS)
    options = parser.parse_args(sys.argv[1:] if arguments is None else arguments)
    if options.trials < 1:
        raise SystemExit("--trials must be at least 1")
    module = load(options.module)
    pre_armed = options.construction == "pre-armed"

    sites: collections.Counter[str] = collections.Counter()
    verdicts: collections.Counter[str] = collections.Counter()
    latencies: list[float] = []
    for _ in range(options.trials):
        site, message, latency = trial(module, options.interpreter, options.margin, pre_armed)
        sites[site] += 1
        verdicts[classify(message)] += 1
        if latency is not None:
            latencies.append(latency)

    print(f"{options.construction}, {options.trials} trials, margin {options.margin}s")
    print(f"interpreter: {options.interpreter}")
    report("site", sites)
    report("message", verdicts)
    if latencies:
        print(
            f"spawn to both pipes closed: min={min(latencies) * 1000:.1f}ms "
            f"median={statistics.median(latencies) * 1000:.1f}ms "
            f"max={max(latencies) * 1000:.1f}ms n={len(latencies)}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
