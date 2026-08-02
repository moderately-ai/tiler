#!/usr/bin/env python3
"""Whether `#pragma METAL fp contract(off)` survives to the Metal *runtime* compiler.

Finding 30 of the [numerical-behaviour record](../../../docs/research/apple-targets/numerical-behaviour.md)
measures the runtime compiler contracting a written multiply/add pair under
`mathMode = Relaxed` and `Fast` at `f32`, `f16`, and `bf16` alike, and finding 10
records that `MTLCompileOptions` exposes no contraction property to turn that off
with. Finding 10's last paragraph records a source-level pragma that *is* accepted
offline and *does* remove the `contract` fast-math flag from the emitted IR — and
records equally deliberately that the numerical probe did not use it, because
changing the source bytes would destroy the byte-identical offline/runtime pairing
that whole comparison rests on.

So the pragma's effect on the runtime path was unmeasured rather than known to be
absent, and the difference decides a contract: if the pragma is a defence, a BF16
program can be given an unfused guarantee on this row by *emitting different
source*, which is ADR 0076's `SupportedWithExactEmulation` shape; if it is not,
`docs/numerical-semantics.md`'s refusal stands as written.

# Why this is a sibling probe and not an axis on the numerical probe

The numerical probe's whole contraction comparison is byte-identical source
through two compilers. A pragma variant is, by construction, *not* byte-identical
source — it is the one perturbation that comparison forbids. Folding it in would
either break the pairing or force a second source per case that the pairing then
has to except, and it would move `probe.harness_sha256` in all four retained
numerical records for a question none of them asks. `aot-runtime-compiler-observer`
and `code-domain-integer-decode` are the precedents for a sibling that shares this
host row, the dispatch host, and nothing else.

# What it reuses rather than re-derives, and why that matters

The control source is `numerical_probe.Kernel.source()` for the very kernel each
retained record measures, so the unperturbed neighbour in every run is the same
bytes finding 30 measured. The discriminating scales are read from those kernel
definitions — `0x3FBE` at `bf16` and `0x3E02` at `f16`, one ulp from 1.5 apiece —
rather than restated here, because finding 28 records that the obvious `x * 1.5 + 1.0`
spelling discriminates on no operand of either narrow vector and would report the
opposite conclusion while its execution witness reported `executed`. The two
candidate results are derived per operand by `numerical_probe.evaluate`, which is
what makes `probe.candidates.*` checkable arithmetic rather than a copied literal.

# The control is the failure proof

A run in which the *unperturbed* neighbour does not return the fused value
establishes nothing about the pragma: whatever suppressed the fusion there is not
the pragma, because the pragma is not in those bytes. Every cell therefore
dispatches the control and the pragma variant in one host invocation, on one
device and one queue, and a control that does not fuse — or whose execution
witness does not report `executed` — refuses the whole run and publishes nothing.
`--perturb-control` applies the pragma to the control as well and is the one-line
way to watch that refusal fire.

# The guard layer this path does not have

`newLibraryWithSource:options:` returns an opaque `MTLLibrary`, so the emitted
module cannot be read and the numerical probe's first guard layer is unavailable
here exactly as it is there. What replaces it is the execution witness on every
case, and an *offline* companion compilation of the identical two sources under
`-ffp-contract=fast`, whose emitted flags say whether the pragma is live at all in
this exact spelling and placement on this row. That companion is offline evidence
about the offline compiler and is never evidence about the runtime one; it exists
so that a negative runtime result cannot be confused with a misplaced pragma.
"""

from __future__ import annotations

import argparse
import hashlib
import os
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass, replace
from datetime import UTC, datetime
from pathlib import Path

HERE = Path(__file__).resolve().parent
SIBLING = HERE.parent
REPOSITORY = HERE.parents[2]

sys.path.insert(0, str(SIBLING))

import numerical_probe as sibling  # noqa: E402

SCHEMA = "tiler.apple-contraction-pragma-runtime/v1"

QUESTION = (
    "whether a source-level contraction pragma survives newLibraryWithSource to the Metal "
    "runtime compiler, measured against an unperturbed neighbour that must still fuse"
)

PRAGMA_LINE = "#pragma METAL fp contract(off)\n"
"""The complete inserted text, newline included, and the only difference between the two sources.

`#pragma clang fp contract(off)` is the other spelling finding 10 records as
accepted. It is not swept here: one spelling answers the question this ticket
asks, and a second would double every cell to compare two mechanisms rather than
to measure one. The unswept spelling is a stated boundary in the README.
"""

ANCHOR = "using namespace metal;\n"
"""Where the pragma goes, chosen because it is the only file-scope position every generated source has.

`#pragma METAL fp` and `#pragma clang fp` are accepted at file scope or at the
start of a compound statement. The generated kernels open a nested `if` block, so
a block-scope placement would sit inside the guarded region and would be a second
variable between the two sources rather than one. File scope after the anchor
covers the whole translation unit and is the placement the offline companion
compilation below confirms is live.
"""

SUBJECT_KERNELS = (
    "contraction_pair_bf16",
    "contraction_pair",
    "contraction_pair_f16",
)
"""The three widths, `bf16` first because it is the width the contract question is about.

`f32` and `f16` are not decoration: finding 30's result is width-independent, so a
pragma that worked at only one width would be a finding rather than a convenience.
"""

RUNTIME_MATH_MODES = ("relaxed", "fast")
"""The two modes under which finding 30 measures the runtime compiler fusing.

`safe` is omitted deliberately. The runtime path already returns the separately
rounded value there (finding 30), so a pragma cell under `safe` could not
distinguish a defence from the behaviour without one.
"""

RUNTIME_OPTIMIZATIONS = ("default", "size")
"""Both `MTLLibraryOptimizationLevel` values, which is the whole runtime axis."""

OFFLINE_OPTIMIZATION = "2"
OFFLINE_FP_CONTRACT = "fast"
"""The offline companion's setting: the one under which finding 10 measured the pragma removing `contract`."""

CONTROL = "control"
PRAGMA = "pragma"
VARIANTS = (CONTROL, PRAGMA)

UNFUSED = "unfused"
FUSED = "fused"
NEITHER = "neither-candidate"

PROFILE = sibling.Profile(
    name="contraction-pragma-macos-msl31",
    schema=SCHEMA,
    msl_version=sibling.MSL_VERSION,
    runtime_language=sibling.RUNTIME_LANGUAGE,
    families=(sibling.LEGACY_PROFILE.family(sibling.HOST_FAMILY),),
    dtypes=sibling.DTYPES,
)
"""The macOS half of the row finding 30 was measured on, and nothing wider.

Same MSL version, same offline target, same runtime language. The iOS Simulator
is out of scope here for a measured reason rather than a convenient one: it
refuses to create a `bfloat` pipeline at all on this row (finding 26), so the
width the question is about cannot be dispatched there.
"""


class ProbeRefusal(RuntimeError):
    """A condition under which this run establishes nothing and must publish nothing."""


def run(command: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(command, capture_output=True, text=True, check=False)


def first_line(text: str) -> str:
    return text.splitlines()[0].strip() if text.strip() else ""


def flat(text: str) -> str:
    """Collapse whitespace so a diagnostic cannot split one record row into two."""
    return " ".join(text.split())


def digest_bytes(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def perturbed(source: str) -> str:
    """Return `source` with the pragma inserted, refusing anything but an exact single anchor.

    The whole probe's claim is "unchanged except the pragma", so the insertion is
    the one place a silent difference could enter. An anchor that appears zero or
    twice means the generated shape moved underneath this probe, and continuing
    would produce two sources differing in something nobody stated.
    """
    if source.count(ANCHOR) != 1:
        raise ProbeRefusal(
            f"the generated source carries {source.count(ANCHOR)} anchors, not exactly one"
        )
    return source.replace(ANCHOR, ANCHOR + PRAGMA_LINE)


@dataclass(frozen=True)
class Subject:
    """One kernel under test, with both its sources and both its derived candidates."""

    kernel: sibling.Kernel
    control_source: str
    pragma_source: str
    discriminating: int
    unfused: int
    fused: int

    @property
    def name(self) -> str:
        return self.kernel.name

    @property
    def dtype(self) -> sibling.Dtype:
        return self.kernel.dtype

    def source(self, variant: str) -> str:
        return self.control_source if variant == CONTROL else self.pragma_source

    def classify(self, results: tuple[int, ...]) -> str:
        """Name which of the two derived candidates the discriminating lane returned."""
        observed = results[self.dtype.operands.index(self.discriminating)]
        if observed == self.unfused:
            return UNFUSED
        if observed == self.fused:
            return FUSED
        return NEITHER


def subject(name: str, *, perturb_control: bool) -> Subject:
    """Resolve one kernel into its two sources and its two derived candidate results.

    The candidates are derived rather than stated, and the operand that separates
    them is *found* rather than assumed to be a particular index. A kernel whose
    two candidates agree on every operand cannot measure contraction at all —
    which is exactly the degeneracy finding 28 records for the obvious `1.5`
    spelling — so that case refuses here rather than producing eight lanes of
    agreement and a confident wrong conclusion.
    """
    kernel = sibling.BY_NAME[name]
    dtype = kernel.dtype
    single_rounding = replace(kernel, fused=True)
    separating = [
        operand
        for operand in dtype.operands
        if sibling.evaluate(kernel, operand, flushes=False)
        != sibling.evaluate(single_rounding, operand, flushes=False)
    ]
    if len(separating) != 1:
        raise ProbeRefusal(
            f"{name}: {len(separating)} operands separate single from double rounding; this probe "
            f"reads exactly one discriminating lane"
        )
    operand = separating[0]
    control_source = kernel.source()
    return Subject(
        kernel=kernel,
        control_source=perturbed(control_source) if perturb_control else control_source,
        pragma_source=perturbed(control_source),
        discriminating=operand,
        unfused=sibling.evaluate(kernel, operand, flushes=False),
        fused=sibling.evaluate(single_rounding, operand, flushes=False),
    )


@dataclass(frozen=True)
class OfflineObservation:
    """What the offline companion compilation of one source emitted."""

    operations: tuple[sibling.FloatOperation, ...]
    options: tuple[str, ...]
    diagnostics: str


def compile_offline(source: Path, destination: Path, math_mode: str) -> OfflineObservation:
    """Compile one source to LLVM IR under `-ffp-contract=fast`, refusing any diagnostic.

    `-Wall -Werror` is what turns "the pragma is accepted" into a checked property
    rather than an unread stderr: an unknown or misplaced pragma warns, and a
    warning is an error here. The `-ffp-contract=fast` selection is the one under
    which the flag the pragma removes is otherwise present, so a control that
    emitted no `contract` flag would mean the companion measured nothing.
    """
    family = PROFILE.family(sibling.HOST_FAMILY)
    configuration = sibling.Configuration(
        math_mode=math_mode,
        optimization=OFFLINE_OPTIMIZATION,
        fp_contract=OFFLINE_FP_CONTRACT,
    )
    command = [
        "xcrun",
        "--sdk",
        family.sdk,
        "metal",
        *PROFILE.offline_flags(family.name, configuration),
        "-Wall",
        "-Werror",
        "-S",
        "-emit-llvm",
        str(source),
        "-o",
        str(destination),
    ]
    result = run(command)
    if result.returncode != 0:
        raise ProbeRefusal(f"metal refused {source.name} under {math_mode}: {flat(result.stderr)}")
    ir = destination.read_text()
    return OfflineObservation(
        operations=sibling.float_operations(ir),
        options=sibling.compile_options(ir),
        diagnostics=flat(result.stderr) or "none",
    )


@dataclass(frozen=True)
class Reported:
    """One dispatched manifest entry, or the exact reason it produced no results."""

    results: tuple[int, ...] | None
    applied_options: str | None
    refusal: str | None


@dataclass(frozen=True)
class Dispatched:
    """One whole host invocation."""

    device: str
    registry_id: str
    apple9: str
    images: tuple[str, ...]
    entries: dict[str, Reported]


def dispatch(host: Path, manifest: Path, expected: dict[str, sibling.Dtype]) -> Dispatched:
    """Run the dispatch host once and parse its `key=value` lines, keeping partial results.

    This does not reuse `numerical_probe.dispatch_batch`, and the difference is
    the negative result. That function raises on a nonzero exit, which is right
    for a gate that must not publish a partial matrix; here a runtime compilation
    the pragma made fail *is the measurement*, and it must be recorded with the
    control that ran before it in the same process rather than collapsed into one
    exception. The manifest is therefore ordered control-first, so the control's
    lines are already on stdout when a pragma entry fails.
    """
    command = [str(host), "batch", str(manifest), *sibling.operand_arguments(PROFILE)]
    result = run(command)
    if result.returncode == 3:
        raise ProbeRefusal(flat(result.stderr) or "no default Metal device resolved")
    if result.returncode == 2:
        raise ProbeRefusal(f"the dispatch host rejected the manifest: {flat(result.stderr)}")

    device, registry, apple9 = "", "", ""
    images: list[str] = []
    entries: dict[str, Reported] = {}
    key, applied, values = "", None, []

    def close() -> None:
        if not key:
            return
        dtype = expected[key]
        if len(values) != len(dtype.operands):
            raise ProbeRefusal(
                f"{key} returned {len(values)} results, expected {len(dtype.operands)}"
            )
        entries[key] = Reported(tuple(values), applied, None)

    for line in result.stdout.splitlines():
        name, _, value = line.partition("=")
        if name == "device":
            device = value
        elif name == "registry-id":
            registry = value
        elif name == "gpu-family-apple9":
            apple9 = value
        elif name == "runtime-compiler-image":
            images.append(value)
        elif name == "case":
            close()
            if value not in expected:
                raise ProbeRefusal(f"{value} was reported but is not in the manifest")
            key, applied, values = value, None, []
        elif name == "applied":
            applied = value
        elif name == "result":
            values.append(int(value, 16))
    close()

    # A nonzero exit that is neither a usage error nor an absent device means an
    # entry failed after the ones before it had printed. The unreported entries
    # are exactly the ones the host never completed, and the host's stderr names
    # the stage — which for this probe is the interesting outcome, so it is
    # attached to each of them verbatim rather than raised.
    if result.returncode != 0:
        reason = flat(result.stderr) or f"the dispatch host exited {result.returncode}"
        for name in expected:
            if name not in entries:
                entries[name] = Reported(None, None, reason)
    if not entries:
        raise ProbeRefusal(f"the dispatch host reported no case at all: {flat(result.stderr)}")
    return Dispatched(device, registry, apple9 or "unreported", tuple(sorted(set(images))), entries)


@dataclass(frozen=True)
class Cell:
    """One (width, math mode, optimization level) pair of a control and a pragma dispatch."""

    subject: Subject
    math_mode: str
    optimization: str
    reported: dict[str, Reported]

    @property
    def key(self) -> str:
        return f"{self.subject.name}.{self.math_mode}.opt-{self.optimization}"

    def witness(self, variant: str) -> str:
        reported = self.reported[variant]
        if reported.results is None:
            return "no-dispatch"
        dtype = self.subject.dtype
        observed = reported.results[dtype.operands.index(self.subject.kernel.witness.operand)]
        return sibling.witness_status(self.subject.kernel.witness, observed).value

    def verdict(self, variant: str) -> str:
        reported = self.reported[variant]
        if reported.results is None:
            return "no-dispatch"
        status = self.witness(variant)
        if status != sibling.WitnessStatus.EXECUTED.value:
            # The same rule the sibling harness applies: an observation whose
            # arithmetic cannot be shown to have run supports no claim about that
            # arithmetic, whatever the claim is about.
            return f"inadmissible-{status}"
        return self.subject.classify(reported.results)


def measure(
    host: Path,
    work: Path,
    subjects: tuple[Subject, ...],
) -> tuple[dict[str, OfflineObservation], tuple[Cell, ...], Dispatched]:
    """Compile the offline companion, then dispatch every cell, control first in each."""
    sources = work / "sources"
    sources.mkdir(parents=True, exist_ok=True)
    written: dict[tuple[str, str], Path] = {}
    for item in subjects:
        for variant in VARIANTS:
            path = sources / f"{item.name}.{variant}.metal"
            path.write_text(item.source(variant))
            written[(item.name, variant)] = path

    offline: dict[str, OfflineObservation] = {}
    for item in subjects:
        for math_mode in RUNTIME_MATH_MODES:
            for variant in VARIANTS:
                destination = work / f"{item.name}.{variant}.{math_mode}.ll"
                offline[f"{item.name}.{math_mode}.{variant}"] = compile_offline(
                    written[(item.name, variant)], destination, math_mode
                )

    cells: list[Cell] = []
    last: Dispatched | None = None
    for item in subjects:
        for math_mode in RUNTIME_MATH_MODES:
            for optimization in RUNTIME_OPTIMIZATIONS:
                configuration = sibling.RuntimeConfiguration(
                    math_mode=math_mode, optimization=optimization
                )
                options = PROFILE.runtime_options(configuration)
                lines, expected = [], {}
                # Control first, always. The control's result is the evidence the
                # pragma's result is read against, and an entry that fails takes
                # every entry after it with it.
                for variant in VARIANTS:
                    key = f"{item.name}.{math_mode}.opt-{optimization}.{variant}"
                    lines.append(
                        "\t".join(
                            (
                                key,
                                item.dtype.name,
                                "source",
                                str(written[(item.name, variant)]),
                                sibling.ENTRY_POINT,
                                options,
                            )
                        )
                    )
                    expected[key] = item.dtype
                manifest = work / f"{item.name}.{math_mode}.{optimization}.manifest.tsv"
                manifest.write_text("\n".join(lines) + "\n")
                dispatched = dispatch(host, manifest, expected)
                last = dispatched
                cells.append(
                    Cell(
                        subject=item,
                        math_mode=math_mode,
                        optimization=optimization,
                        reported={
                            variant: dispatched.entries[
                                f"{item.name}.{math_mode}.opt-{optimization}.{variant}"
                            ]
                            for variant in VARIANTS
                        },
                    )
                )
    assert last is not None
    return offline, tuple(cells), last


def require_controls(cells: tuple[Cell, ...]) -> None:
    """Refuse the run unless every unperturbed neighbour fused with a reporting witness.

    This is the probe's own failure proof and not a formality. A run in which the
    control did not fuse says nothing about the pragma, because whatever
    suppressed the fusion is not in the control's bytes; publishing such a run
    would put a pragma column next to a control column that agrees with it for an
    unrelated reason.
    """
    complaints = [
        f"{cell.key}: the control returned {cell.verdict(CONTROL)}, not {FUSED}"
        for cell in cells
        if cell.verdict(CONTROL) != FUSED
    ]
    if complaints:
        raise ProbeRefusal(
            f"{len(complaints)} of {len(cells)} controls did not fuse, so this run establishes "
            f"nothing about the pragma: " + "; ".join(complaints)
        )


def environment_rows(toolchain: sibling.Toolchain, dispatched: Dispatched) -> list[tuple[str, str]]:
    """The exact host row every value below is qualified by."""
    family = PROFILE.family(sibling.HOST_FAMILY)
    sdk = toolchain.sdks[family.sdk]
    xcode = run(["xcodebuild", "-version"])
    rows = [
        ("environment.date_utc", datetime.now(UTC).strftime("%Y-%m-%dT%H:%M:%SZ")),
        ("environment.os_version", first_line(run(["sw_vers", "-productVersion"]).stdout)),
        ("environment.os_build", first_line(run(["sw_vers", "-buildVersion"]).stdout)),
        ("environment.machine", first_line(run(["uname", "-m"]).stdout)),
        ("environment.xcode", flat(xcode.stdout) if xcode.returncode == 0 else "unreported"),
        ("environment.metal_platform", family.metal_platform),
        ("environment.sdk", sdk.name),
        ("environment.sdk_version", sdk.version),
        ("environment.sdk_build", sdk.build),
        ("environment.requested_target", family.target),
        ("environment.metal_version", sdk.metal_version),
        ("environment.metallib_version", sdk.metallib_version),
        ("environment.execution", family.execution.value),
        ("environment.device", dispatched.device),
        ("environment.device_registry_id", dispatched.registry_id),
        ("environment.device_apple9", dispatched.apple9),
        ("environment.runtime_compiler_build", sibling.compiler_build(dispatched.images)),
    ]
    rows += [
        (f"environment.runtime_compiler_image.{index}", image)
        for index, image in enumerate(dispatched.images)
    ]
    return rows


def record_rows(
    toolchain: sibling.Toolchain,
    subjects: tuple[Subject, ...],
    offline: dict[str, OfflineObservation],
    cells: tuple[Cell, ...],
    dispatched: Dispatched,
) -> list[tuple[str, str]]:
    revision = run(["git", "-C", str(REPOSITORY), "rev-parse", "HEAD"])
    family = PROFILE.family(sibling.HOST_FAMILY)
    configuration = sibling.Configuration(
        math_mode="<mode>",
        optimization=OFFLINE_OPTIMIZATION,
        fp_contract=OFFLINE_FP_CONTRACT,
    )
    rows: list[tuple[str, str]] = [
        ("schema", SCHEMA),
        ("probe.question", QUESTION),
        ("probe.profile", PROFILE.name),
        ("probe.repository_base_revision", first_line(revision.stdout) or "unreported"),
        ("probe.harness_sha256", digest_bytes(Path(__file__).resolve().read_bytes())),
        ("probe.numerical_harness_sha256", sibling.digest(SIBLING / "numerical_probe.py")),
        ("probe.host_source_sha256", sibling.digest(sibling.HOST_SOURCE)),
        ("probe.entry_point", sibling.ENTRY_POINT),
        ("probe.family", family.name),
        ("probe.msl_version", PROFILE.msl_version),
        ("probe.pragma_text", PRAGMA_LINE.rstrip("\n")),
        ("probe.pragma_anchor", ANCHOR.rstrip("\n")),
        (
            "probe.pragma_insertion",
            f"exactly one line, {len(PRAGMA_LINE.encode())} bytes including its newline, inserted "
            f"immediately after the single anchor line; every other byte of the control source is "
            f"unchanged",
        ),
        ("probe.control_requirement", f"every control cell must return {FUSED} with an executed witness"),
        (
            "probe.offline_companion_flags",
            " ".join(PROFILE.offline_flags(family.name, configuration)) + " -Wall -Werror -S -emit-llvm",
        ),
        (
            "probe.offline_companion_scope",
            "offline evidence that the pragma is live in this spelling and placement; never "
            "evidence about the runtime compiler",
        ),
        ("probe.runtime_math_modes", " ".join(RUNTIME_MATH_MODES)),
        ("probe.runtime_optimizations", " ".join(RUNTIME_OPTIMIZATIONS)),
        ("probe.runtime_fixed_options", f"lang={PROFILE.runtime_language},fpfun={sibling.DEFAULT_FP32_FUNCTIONS}"),
        (
            "probe.runtime_guard_layer",
            f"{sibling.EXECUTION_WITNESS} only; newLibraryWithSource returns an opaque MTLLibrary "
            f"and no emitted module can be read",
        ),
        (
            "probe.runtime_diagnostic_channel",
            "unobserved on success: the dispatch host reads newLibraryWithSource's error only when "
            "the library is nil, so a warning-free runtime acceptance is not established here",
        ),
        ("probe.kernels", " ".join(item.name for item in subjects)),
    ]
    rows += [
        (f"probe.operands.{item.dtype.name}", " ".join(item.dtype.render(v) for v in item.dtype.operands))
        for item in subjects
    ]
    for item in subjects:
        dtype, witness = item.dtype, item.kernel.witness
        rows += [
            (f"probe.candidate.{item.name}.dtype", dtype.name),
            (f"probe.candidate.{item.name}.scale", dtype.render(item.kernel.steps[0].constant)),
            (f"probe.candidate.{item.name}.bias", dtype.render(item.kernel.steps[1].constant)),
            (f"probe.candidate.{item.name}.discriminating_operand", dtype.render(item.discriminating)),
            (f"probe.candidate.{item.name}.unfused", dtype.render(item.unfused)),
            (f"probe.candidate.{item.name}.fused", dtype.render(item.fused)),
            (f"probe.candidate.{item.name}.witness_operand", dtype.render(witness.operand)),
            (f"probe.candidate.{item.name}.witness_executed", dtype.render(witness.executed)),
            (f"probe.candidate.{item.name}.witness_deleted", dtype.render(witness.deleted)),
        ]
    rows += environment_rows(toolchain, dispatched)
    for item in subjects:
        for variant in VARIANTS:
            rows.append(
                (
                    f"source.{item.name}.{variant}.sha256",
                    digest_bytes(item.source(variant).encode()),
                )
            )
    for key in sorted(offline):
        observation = offline[key]
        rows += [
            (
                f"offline.{key}.float_operations",
                " ".join(
                    f"{operation.opcode}[{' '.join(operation.flags)}]"
                    for operation in observation.operations
                )
                or "none",
            ),
            (f"offline.{key}.compile_options", " ".join(observation.options) or "none"),
            (f"offline.{key}.diagnostics", observation.diagnostics),
        ]
    for cell in cells:
        for variant in VARIANTS:
            reported = cell.reported[variant]
            prefix = f"case.{cell.key}.{variant}"
            if reported.refusal is not None:
                rows.append((f"{prefix}.refusal", reported.refusal))
                continue
            assert reported.results is not None
            rows += [
                (f"{prefix}.applied_options", reported.applied_options or "unreported"),
                (
                    f"{prefix}.results",
                    " ".join(cell.subject.dtype.render(value) for value in reported.results),
                ),
                (f"{prefix}.witness", cell.witness(variant)),
                (f"{prefix}.verdict", cell.verdict(variant)),
            ]
    verdicts = [cell.verdict(PRAGMA) for cell in cells]
    rows += [
        ("summary.cells", str(len(cells))),
        ("summary.control_fused", str(sum(1 for cell in cells if cell.verdict(CONTROL) == FUSED))),
        ("summary.pragma_unfused", str(sum(1 for verdict in verdicts if verdict == UNFUSED))),
        ("summary.pragma_fused", str(sum(1 for verdict in verdicts if verdict == FUSED))),
        (
            "summary.pragma_other",
            str(sum(1 for verdict in verdicts if verdict not in (UNFUSED, FUSED))),
        ),
    ]
    return rows


def write_result(
    destination: Path,
    rows: list[tuple[str, str]],
    subjects: tuple[Subject, ...],
) -> None:
    """Stage the record, its retained sources, and their manifest, then publish atomically."""
    for key, value in rows:
        if "\t" in key or "\t" in value or "\n" in key or "\n" in value:
            raise ProbeRefusal(f"row {key!r} carries a tab or newline and would corrupt the record")
    staging = Path(tempfile.mkdtemp(prefix=".staging-", dir=str(destination.parent)))
    try:
        sources = staging / "sources"
        sources.mkdir()
        manifest = [
            ("schema", "tiler.apple-contraction-pragma-runtime-input-manifest/v1"),
            ("profile", PROFILE.name),
            ("msl_version", PROFILE.msl_version),
            ("runtime_language", PROFILE.runtime_language),
            (
                "input.spikes/apple-targets/contraction-pragma-runtime-probe/pragma_probe.py",
                digest_bytes(Path(__file__).resolve().read_bytes()),
            ),
            (
                "input.spikes/apple-targets/numerical_probe.py",
                sibling.digest(SIBLING / "numerical_probe.py"),
            ),
            (
                "input.spikes/apple-targets/numerical_probe_host.m",
                sibling.digest(sibling.HOST_SOURCE),
            ),
        ]
        for item in subjects:
            for variant in VARIANTS:
                name = f"{item.name}.{variant}.metal"
                (sources / name).write_text(item.source(variant))
                manifest.append((f"source.sources/{name}", digest_bytes(item.source(variant).encode())))
        (staging / "record.tsv").write_text(
            "".join(f"{key}\t{value}\n" for key, value in rows)
        )
        (staging / "input-manifest.tsv").write_text(
            "".join(f"{key}\t{value}\n" for key, value in manifest)
        )
        if destination.exists():
            shutil.rmtree(destination)
        os.replace(staging, destination)
    finally:
        if staging.exists():
            shutil.rmtree(staging, ignore_errors=True)


def main(arguments: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--result-dir", type=Path, required=True, help="where to publish the record")
    parser.add_argument("--work-dir", type=Path, help="keep the generated sources, IR, and manifests")
    parser.add_argument(
        "--perturb-control",
        action="store_true",
        help="apply the pragma to the control too; the run must then refuse and publish nothing",
    )
    parsed = parser.parse_args(arguments)

    holder = None
    if parsed.work_dir is None:
        holder = tempfile.TemporaryDirectory(prefix="tiler-pragma-probe-")
        work = Path(holder.name)
    else:
        work = parsed.work_dir
        work.mkdir(parents=True, exist_ok=True)
    try:
        toolchain = sibling.resolve(PROFILE)
        host = work / "pragma_probe_host"
        toolchain.build_host(host, PROFILE.family(sibling.HOST_FAMILY).sdk)
        subjects = tuple(
            subject(name, perturb_control=parsed.perturb_control) for name in SUBJECT_KERNELS
        )
        offline, cells, dispatched = measure(host, work, subjects)
        require_controls(cells)
        rows = record_rows(toolchain, subjects, offline, cells, dispatched)
        parsed.result_dir.parent.mkdir(parents=True, exist_ok=True)
        write_result(parsed.result_dir, rows, subjects)
    except sibling.ProbeUnavailable as unavailable:
        print(f"pragma_probe: unavailable: {unavailable}", file=sys.stderr)
        return 1
    except (ProbeRefusal, sibling.ProbeFailure) as refused:
        print(f"pragma_probe: refused: {refused}", file=sys.stderr)
        return 1
    finally:
        if holder is not None:
            holder.cleanup()

    for cell in cells:
        print(
            f"{cell.key}\tcontrol={cell.verdict(CONTROL)}\tpragma={cell.verdict(PRAGMA)}"
            f"\twitness={cell.witness(PRAGMA)}"
        )
    print(f"published {parsed.result_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
