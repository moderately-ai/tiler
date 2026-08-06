#!/usr/bin/env python3
"""Measure whether the Metal compiler preserves a kernel's emitted evaluation order.

The question is the one [the permitted-divergence oracle
derivation](../../../docs/research/reference/permitted-divergence-oracle.md)
names as refusal class 3: that oracle compares a candidate bitwise against the
*one* realization the physical plan pinned, and the pin holds only if the order
the artifact emits is the order the device executes. Tiler asserts flags today
and consults no fact, so the property is asserted and measured by nothing.

# What this probe does and does not ask

It dispatches two four-contributor add kernels over one operand set whose serial
fold and two-by-two split differ by one ULP — the left-deep chain a serial
reduction's plan would pin, and the split a two-by-two partition would pin — at
every combination of `-fmetal-math-mode` and `-ffp-contract` the offline driver
accepts, and at every `mathMode`/`optimizationLevel` pair `MTLCompileOptions`
exposes. It reports the returned bits against the value each kernel's own
written order names.

**Both kernels are needed and neither alone would do.** A single kernel's result
cannot show that its contributors were *not* regrouped, and the left-deep chain
in particular is the form a compiler canonicalizes toward, so preserving it says
less than it appears to. The split is the perturbation that makes the other
reading possible: the result lane moves when, and only when, the emitted order
moves. Offline, `fold_shape` reads the emitted add tree out of the module and
says so structurally rather than by inference.

**Reassociation is the question; contraction is not.** The two fold kernels
contain adds and nothing else, so `-ffp-contract` has no multiply/add pair to
act on and cannot explain a disagreement in them. That is a structural argument,
so this probe measures it rather than asserting it: every fold case records its
emitted floating-point operation list, and a third kernel — a multiply/add pair
over the same buffer — is dispatched in the same run and in the same flag matrix
as a positive control. If that control never fuses, the contraction axis is not
live in this run, the fold kernels' invariance under it means nothing, and the
producer refuses to publish.

**The offline and runtime halves are separate evidence.** Finding 30 of the
[Apple GPU numerical behaviour
record](../../../docs/research/apple-targets/numerical-behaviour.md) measured
the runtime compiler contracting under `relaxed` and `fast` whatever the offline
selection said, so an offline observation here is not transferable to a
runtime-compiled kernel. Every case key states which compiler produced it and no
verdict is read across.

# Why every operand arrives in a buffer

An immediate is constant-folded, and a compile-time fold is an evaluation order
the *front end* chose rather than one the device executed. The four contributors
therefore come from a device buffer, which is the argument finding 32's sibling
probe already relies on: no stage of either compiler can fold arithmetic whose
operands it cannot see. The execution witness is a second quad whose value is
the same under every order-preserving grouping and differs from all four of its
operands, so a returned witness value is evidence the adds ran without being
evidence about the order under test.

# Run it

From this directory, with an Apple toolchain and a Metal device present:

    python3 order_probe.py                    # print the record to stdout
    python3 order_probe.py --retain           # write results/<date>-<identity>/
    python3 order_probe.py --perturb <name>   # a failure proof; see --help

Nothing here is wired into a repository gate. It installs nothing, downloads
nothing, and writes only inside this directory.
"""

from __future__ import annotations

import argparse
import hashlib
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from datetime import datetime, timezone
from fractions import Fraction
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPOSITORY = HERE.parents[2]
HOST_SOURCE = REPOSITORY / "spikes" / "apple-targets" / "numerical_probe_host.m"

SCHEMA = "tiler.apple-evaluation-order/v1"

# The offline row this probe compiles for. It is the unified MSL 4 selection the
# first macOS Metal compile profile names, so the compilation is the production
# one rather than a neighbouring language standard.
LANGUAGE = "metal4.0"
OFFLINE_TARGET = "air64-apple-macos26.0"
RUNTIME_LANGUAGE = "4.0"
SDK = "macosx"

# Pinned rather than swept. Finding 18 measures this flag inert for multiply,
# add, divide, and a fused multiply-add, which is this probe's whole vocabulary,
# and `precise` is the value the governed row selects.
FP32_FUNCTIONS = "precise"

MATH_MODES = ("safe", "relaxed", "fast")
CONTRACT_SETTINGS = ("off", "on", "fast")
# `-O2` is the governed level and `-O0` is the level finding 19 measures as the
# sole outlier, and it is the interesting one here: at `-O0` the emitted IR is
# measured to keep the written tree, so a device disagreement there would be
# attributable to the AIR-to-ISA stage below the IR this probe can read.
OPTIMIZATION_LEVELS = ("O0", "O2")
# The whole runtime surface. `MTLLibraryOptimizationLevel` offers these two and
# there is no `-ffp-contract` property to sweep (finding 10).
RUNTIME_OPTIMIZATION_LEVELS = ("default", "size")

ENTRY_POINT = "tiler_probe"

# The sentinel `numerical_probe_host.m` seeds its output buffer with. Declared
# here so this producer can hold every reference value to differing from it,
# rather than trusting that no kernel can produce it.
HOST_F32_SENTINEL = 0xDEADBEEF


# --- exact binary32 arithmetic ---------------------------------------------
#
# Every reference value below is computed in exact rationals and rounded once,
# because the candidates differ by one ULP and a host `float` add of two
# binary32 values narrowed back to binary32 double-rounds. Nothing here reads a
# decimal literal: the operands are bit patterns in and bit patterns out.


def bits_to_exact(bits: int) -> Fraction:
    """The exact rational value of a binary32 bit pattern."""
    if bits > 0xFFFFFFFF or bits < 0:
        raise ProbeFailure(f"{bits:#x} is not a 32-bit pattern")
    sign = -1 if bits >> 31 else 1
    exponent = (bits >> 23) & 0xFF
    significand = bits & 0x7FFFFF
    if exponent == 0xFF:
        raise ProbeFailure(f"{bits:08x} is an infinity or a NaN, which this probe does not use")
    if exponent == 0:
        return Fraction(sign * significand) * Fraction(1, 2**149)
    return Fraction(sign * (significand + (1 << 23))) * Fraction(2) ** (exponent - 150)


def exact_to_bits(value: Fraction) -> int:
    """Round an exact rational to binary32, to nearest, ties to even."""
    if value == 0:
        return 0
    sign = 0 if value > 0 else 1
    magnitude = abs(value)
    exponent = magnitude.numerator.bit_length() - magnitude.denominator.bit_length()
    while magnitude < Fraction(2) ** exponent:
        exponent -= 1
    while magnitude >= Fraction(2) ** (exponent + 1):
        exponent += 1
    subnormal = exponent < -126
    if subnormal:
        exponent = -126
    scaled = magnitude * Fraction(2) ** (23 - exponent)
    integral = scaled.numerator // scaled.denominator
    remainder = scaled - integral
    if remainder > Fraction(1, 2) or (remainder == Fraction(1, 2) and integral % 2 == 1):
        integral += 1
    if integral >= 1 << 24:
        integral >>= 1
        exponent += 1
        subnormal = False
    if exponent > 127:
        raise ProbeFailure("a reference value overflowed binary32, which this probe does not use")
    if subnormal or integral < 1 << 23:
        # Reached only by a value below the smallest normal. No operand set here
        # produces one, so this raises rather than returning a pattern no case
        # was designed to interpret.
        raise ProbeFailure("a reference value is subnormal, which this probe does not use")
    return (sign << 31) | ((exponent + 127) << 23) | (integral - (1 << 23))


def f32_add(left: int, right: int) -> int:
    return exact_to_bits(bits_to_exact(left) + bits_to_exact(right))


def f32_mul(left: int, right: int) -> int:
    return exact_to_bits(bits_to_exact(left) * bits_to_exact(right))


def f32_fma(a: int, b: int, c: int) -> int:
    """One rounding over the exact product plus the addend."""
    return exact_to_bits(bits_to_exact(a) * bits_to_exact(b) + bits_to_exact(c))


def order_preserving_values(quad: tuple[int, ...]) -> frozenset[int]:
    """Every value an order-preserving regrouping of `quad` can produce.

    This is the permitted set of the reassociation freedom over a fixed leaf
    order, and it is deliberately not an acceptance criterion: it is what makes
    "the returned bits are the written order's bits" a statement with content,
    by naming the values a reordering could have returned instead.
    """

    def groupings(sequence: tuple[int, ...]) -> list[int]:
        if len(sequence) == 1:
            return [sequence[0]]
        produced = []
        for split in range(1, len(sequence)):
            for left in groupings(sequence[:split]):
                for right in groupings(sequence[split:]):
                    produced.append(f32_add(left, right))
        return produced

    return frozenset(groupings(quad))


# --- the operand vector and its quads ---------------------------------------
#
# One vector serves every kernel, because the dispatch host takes one operand
# group per dtype and shares it across the whole manifest. Twelve elements are
# three quads and each kernel folds the quad its lane belongs to, so every lane
# of every case is written and no result is a sentinel the host would refuse.


@dataclass(frozen=True)
class Quad:
    name: str
    purpose: str
    operands: tuple[int, int, int, int]


QUADS = (
    Quad(
        "seed",
        "the four-contributor set whose serial fold and two-by-two split differ by one ULP",
        (0x3F400000, 0x3E800000, 0x33400000, 0x33000000),
    ),
    Quad(
        "witness",
        "powers of two, whose fold is exact under every grouping and differs from every operand",
        (0x3F800000, 0x40000000, 0x40800000, 0x41000000),
    ),
    Quad(
        "contraction",
        "the operand, scale, and bias whose separately rounded and fused multiply-add differ",
        (0x3EB97EF9, 0x3FC00000, 0x3F800000, 0x3F800000),
    ),
)

OPERANDS = tuple(pattern for quad in QUADS for pattern in quad.operands)


# --- the kernels -------------------------------------------------------------


@dataclass(frozen=True)
class Kernel:
    name: str
    purpose: str
    #: The body statements over `v4`..`v7`, the quad's four loaded contributors,
    #: rendered into the Metal emitter's per-statement output shape.
    statements: tuple[str, ...]
    result: str
    #: What the emitted floating-point operation list must be for an offline
    #: case to be admissible. Layer one of the guard.
    expected_operations: tuple[str, ...]
    fold: bool
    #: How many of its quad's four operands this kernel reads. The control is a
    #: three-operand multiply-add, so it loads three: `-Wall -Werror` refuses an
    #: unused load, and a load written only to keep two kernels' prologues
    #: identical would be a statement the compiler is free to delete.
    contributors: int = 4

    def source(self) -> str:
        lines = [
            "#include <metal_stdlib>",
            "using namespace metal;",
            "",
            f"kernel void {ENTRY_POINT}(",
            "        device const float *b0 [[buffer(0)]],",
            "        device float *b1 [[buffer(1)]],",
            "        uint tiler_global_invocation_index [[thread_position_in_grid]]) {",
            "    ulong v0 = ulong(tiler_global_invocation_index);",
            f"    ulong v1 = {len(OPERANDS)}ul;",
            "    bool v2 = v0 < v1;",
            "    if (v2) {",
            "        ulong v3 = (v0 / 4ul) * 4ul;",
        ]
        lines += [
            f"        float v{4 + offset} = b0[v3 + {offset}ul];"
            for offset in range(self.contributors)
        ]
        lines += [f"        {statement}" for statement in self.statements]
        lines += [f"        b1[v0] = {self.result};", "    }", "}", ""]
        return "\n".join(lines)


SERIAL_STATEMENTS = (
    "float v8 = v4 + v5;",
    "float v9 = v8 + v6;",
    "float v10 = v9 + v7;",
)
SPLIT_STATEMENTS = (
    "float v8 = v4 + v5;",
    "float v9 = v6 + v7;",
    "float v10 = v8 + v9;",
)

KERNELS = (
    Kernel(
        "serial_fold4",
        "the left-deep chain ((a+b)+c)+d, the written order the oracle would pin",
        SERIAL_STATEMENTS,
        "v10",
        ("fadd", "fadd", "fadd"),
        fold=True,
    ),
    Kernel(
        "split_fold4",
        "the legal alternative (a+b)+(c+d), the perturbation that shows the seed discriminates",
        SPLIT_STATEMENTS,
        "v10",
        ("fadd", "fadd", "fadd"),
        fold=True,
    ),
    Kernel(
        "contraction_control",
        "a*b+c, the positive control that keeps the contraction axis live in this run",
        ("float v7 = v4 * v5;", "float v8 = v7 + v6;"),
        "v8",
        ("fadd", "fmul"),
        fold=False,
        contributors=3,
    ),
)

DECLARED_KERNELS = 3
DECLARED_OFFLINE_CASES = 54  # 3 kernels x 3 math modes x 3 contraction settings x 2 levels
DECLARED_RUNTIME_CASES = 18  # 3 kernels x 3 math modes x 2 optimization levels


class ProbeFailure(RuntimeError):
    """A defect or an unmet prerequisite. Never a quiet skip."""


# --- references --------------------------------------------------------------


@dataclass(frozen=True)
class Reference:
    """What one kernel may return for one quad, and which value the source wrote."""

    written: int
    #: Values a legal alternative could return instead. Empty when the freedom
    #: under test cannot move this quad's value, which is what makes it a
    #: witness rather than a probe.
    alternatives: frozenset[int]

    def verdict(self, observed: int) -> str:
        if not self.alternatives:
            return "as-written-and-freedom-independent" if observed == self.written else "unexpected"
        if observed == self.written:
            return "as-written"
        if observed in self.alternatives:
            return "diverged"
        return "unexpected"


def fold_reference(kernel: Kernel, quad: Quad) -> Reference:
    a, b, c, d = quad.operands
    written = f32_add(f32_add(f32_add(a, b), c), d)
    if kernel.statements == SPLIT_STATEMENTS:
        written = f32_add(f32_add(a, b), f32_add(c, d))
    return Reference(written, order_preserving_values(quad.operands) - {written})


def contraction_reference(quad: Quad) -> Reference:
    a, b, c, _ = quad.operands
    written = f32_add(f32_mul(a, b), c)
    fused = f32_fma(a, b, c)
    return Reference(written, frozenset({fused}) - {written})


def references() -> dict[tuple[str, str], Reference]:
    table: dict[tuple[str, str], Reference] = {}
    for kernel in KERNELS:
        for quad in QUADS:
            reference = (
                fold_reference(kernel, quad) if kernel.fold else contraction_reference(quad)
            )
            for value in {reference.written} | reference.alternatives:
                if value == HOST_F32_SENTINEL:
                    raise ProbeFailure(
                        f"{kernel.name}/{quad.name} can return the host's unwritten-element "
                        "sentinel, so an undispatched lane would be indistinguishable from a "
                        "measured one"
                    )
            table[(kernel.name, quad.name)] = reference
    return table


# --- toolchain ---------------------------------------------------------------


def run(command: list[str], **kwargs) -> subprocess.CompletedProcess:
    return subprocess.run(command, capture_output=True, text=True, check=False, **kwargs)


def required(command: list[str], stage: str) -> str:
    result = run(command)
    if result.returncode != 0:
        raise ProbeFailure(f"{stage} failed: {(result.stderr or result.stdout).strip()}")
    return result.stdout.strip()


@dataclass(frozen=True)
class Toolchain:
    metal_version: str
    metal_installed_dir: str
    sdk_version: str
    sdk_build: str
    xcode: str
    xcode_path: str
    os_build: str
    os_version: str
    architecture: str


def resolve_toolchain() -> Toolchain:
    for tool in ("xcrun", "xcode-select", "sw_vers", "uname"):
        if shutil.which(tool) is None:
            raise ProbeFailure(f"{tool} is not on PATH; this probe needs an Apple toolchain")
    version = required(["xcrun", "--sdk", SDK, "metal", "--version"], "metal --version")
    metal_version = ""
    installed = ""
    for line in version.splitlines():
        if line.startswith("Apple metal version"):
            metal_version = line.strip()
        if line.startswith("InstalledDir:"):
            installed = line.split(":", 1)[1].strip()
    if not metal_version:
        raise ProbeFailure(f"could not read a Metal compiler version from: {version!r}")
    xcode = required(["xcodebuild", "-version"], "xcodebuild -version").replace("\n", " ")
    return Toolchain(
        metal_version=metal_version,
        metal_installed_dir=installed,
        sdk_version=required(["xcrun", "--sdk", SDK, "--show-sdk-version"], "sdk version"),
        sdk_build=required(["xcrun", "--sdk", SDK, "--show-sdk-build-version"], "sdk build"),
        xcode=xcode,
        xcode_path=required(["xcode-select", "-p"], "xcode-select -p"),
        os_build=required(["sw_vers", "-buildVersion"], "sw_vers"),
        os_version=required(["sw_vers", "-productVersion"], "sw_vers"),
        architecture=required(["uname", "-m"], "uname -m"),
    )


def compiler_build_tag(toolchain: Toolchain) -> str:
    """The `metalfe-…` build inside the version string, for the record directory name."""
    for token in toolchain.metal_version.replace("(", " ").replace(")", " ").split():
        if token.startswith("metalfe-"):
            return token
    raise ProbeFailure(f"no metalfe build in {toolchain.metal_version!r}")


# --- offline compilation ------------------------------------------------------

FLOAT_OPCODES = ("fmul", "fadd", "fsub", "fdiv", "fneg", "fpext", "fptrunc")


def operation_list(ir: str) -> tuple[str, ...]:
    """Every floating-point operation the emitted module's function bodies carry.

    Named calls are matched as well as bare opcodes because this front end
    spells a fused multiply-add `air.fma.f32` rather than `llvm.fma.f32`; the
    numerical-behaviour record retracted an operation count once for naming only
    the LLVM spellings, and a fusion reported as an absence is the one direction
    a reader acts on.
    """
    operations: list[str] = []
    inside = False
    for line in ir.splitlines():
        if line.startswith("define "):
            inside = True
            continue
        if line.startswith("}"):
            inside = False
            continue
        if not inside:
            continue
        called = None
        for marker in ("@air.", "@llvm."):
            index = line.find(marker)
            if index != -1:
                tail = line[index + 1 :]
                called = tail.split("(")[0].strip()
                break
        if called is not None:
            operations.append(called)
            continue
        for opcode in FLOAT_OPCODES:
            if f" {opcode} " in line:
                operations.append(opcode)
                break
    return tuple(sorted(operations))


def render_operations(operations: tuple[str, ...]) -> str:
    return ";".join(operations) if operations else "none"


def entry_body(ir: str) -> list[str]:
    """The instruction lines of the module's one entry point."""
    body: list[str] = []
    inside = False
    for line in ir.splitlines():
        if line.startswith("define "):
            inside = True
            continue
        if line.startswith("}"):
            inside = False
            continue
        if inside:
            body.append(line.strip())
    return body


def _defined(body: list[str]) -> dict[str, str]:
    """Every `%name = …` in the body, mapped to the text on the right."""
    definitions: dict[str, str] = {}
    for line in body:
        name, separator, rest = line.partition(" = ")
        if separator and name.startswith("%"):
            definitions[name] = rest
    return definitions


def _buffer_slot(name: str, definitions: dict[str, str]) -> int | None:
    """Which operand slot a loaded value came from, or `None` if unresolvable.

    Resolves the two shapes the front end emits for `b0[v3 + k]` after
    optimization: the base index, and the base with a small constant or-ed in.
    Anything else returns `None`, so an unrecognized addressing form is recorded
    as unresolved rather than guessed at.
    """
    load = definitions.get(name, "")
    if not load.startswith("load float, float addrspace(1)*"):
        return None
    pointer = load.split("addrspace(1)*")[1].split(",")[0].strip()
    gep = definitions.get(pointer, "")
    if "getelementptr" not in gep:
        return None
    index = gep.rsplit("i64 ", 1)[-1].strip()
    if not index.startswith("%"):
        return None
    computed = definitions.get(index, "")
    if computed.startswith("and i64"):
        return 0
    if computed.startswith("or i64"):
        constant = computed.rsplit(",", 1)[-1].strip()
        return int(constant) if constant.isdigit() else None
    return None


FAST_MATH_FLAGS = ("fast", "reassoc", "nnan", "ninf", "nsz", "arcp", "contract", "afn")


def fast_math_licences(ir: str) -> str:
    """The fast-math licence set the emitted floating-point operations carry.

    Recorded so a reordering is attributable to the licences the module actually
    granted rather than to the driver flags that were requested. Finding 1 of the
    numerical-behaviour record measures those two as different things: the same
    `-fmetal-math-mode` renders different licence sets at different `-ffp-contract`
    settings, and the module-level `air.compile.*` names summarize neither.
    """
    sets: set[str] = set()
    for line in entry_body(ir):
        rest = line.partition(" = ")[2]
        if not rest.startswith(("fadd ", "fmul ", "fsub ", "fdiv ")):
            continue
        words = rest.split()[1:]
        granted = tuple(word for word in words if word in FAST_MATH_FLAGS)
        sets.add(" ".join(granted) if granted else "none")
    return ";".join(sorted(sets)) if sets else "none"


def fold_shape(ir: str) -> tuple[str, str]:
    """The emitted add tree, and its classification.

    This is the direct structural reading of the evaluation order the module
    carries, which the returned bits can only reach by inference. It is layer one
    of the guard doing a second job: an opcode multiset cannot distinguish a
    preserved tree from a rearranged one, because rearranging three adds leaves
    three adds.
    """
    body = entry_body(ir)
    definitions = _defined(body)
    adds: list[tuple[str, str, str]] = []
    for line in body:
        name, separator, rest = line.partition(" = ")
        if not separator or not rest.startswith("fadd "):
            continue
        operands = rest.rsplit("float ", 1)[-1]
        left, _, right = operands.partition(", ")
        adds.append((name, left.strip(), right.strip()))
    if not adds:
        return "none", "no-adds"

    labels: dict[str, str] = {}
    anonymous = 0

    def label(operand: str) -> str:
        nonlocal anonymous
        if operand in labels:
            return labels[operand]
        slot = _buffer_slot(operand, definitions)
        labels[operand] = f"a{slot}" if slot is not None else f"L{anonymous}"
        if slot is None:
            anonymous += 1
        return labels[operand]

    rendered: list[str] = []
    for index, (name, left, right) in enumerate(adds):
        labels[name] = f"t{index}"
        rendered.append(f"t{index}={label(left)}+{label(right)}")
    shape = ";".join(rendered)

    references = [
        (operand.startswith("t") for operand in (labels[left], labels[right]))
        for _, left, right in adds
    ]
    internal = sum(1 for pair in references for value in pair if value)
    if len(adds) != 3:
        return shape, "other"
    if internal == 0:
        # Every add reads a value this reader cannot connect to another add,
        # which is what `-O0` produces: each intermediate round-trips through a
        # stack slot, so the tree exists in the program and not in the SSA form.
        return shape, "unresolved-through-stack-slots"
    if rendered[1].startswith("t1=t0+") and rendered[2].startswith("t2=t1+"):
        return shape, "serial"
    if rendered[2] in ("t2=t0+t1", "t2=t1+t0"):
        return shape, "split"
    return shape, "other"


def offline_flags(mode: str, contract: str, level: str) -> list[str]:
    return [
        "-x",
        "metal",
        f"-std={LANGUAGE}",
        "-target",
        OFFLINE_TARGET,
        f"-{level}",
        f"-fmetal-math-mode={mode}",
        f"-fmetal-math-fp32-functions={FP32_FUNCTIONS}",
        f"-ffp-contract={contract}",
        "-Wall",
        "-Werror",
    ]


def compile_offline(source: Path, flags: list[str], workspace: Path, key: str) -> tuple[str, Path]:
    """Emit the module's IR and link a metallib, from the identical source bytes."""
    ir = run(["xcrun", "--sdk", SDK, "metal", *flags, "-S", "-emit-llvm", str(source), "-o", "-"])
    if ir.returncode != 0:
        raise ProbeFailure(f"{key}: emitting LLVM IR failed: {ir.stderr.strip()}")
    air = workspace / f"{key}.air"
    compiled = run(["xcrun", "--sdk", SDK, "metal", *flags, "-c", str(source), "-o", str(air)])
    if compiled.returncode != 0:
        raise ProbeFailure(f"{key}: compiling to AIR failed: {compiled.stderr.strip()}")
    library = workspace / f"{key}.metallib"
    linked = run(["xcrun", "--sdk", SDK, "metallib", str(air), "-o", str(library)])
    if linked.returncode != 0:
        raise ProbeFailure(f"{key}: linking failed: {linked.stderr.strip()}")
    return ir.stdout, library


# --- dispatch -----------------------------------------------------------------


@dataclass
class Observation:
    key: str
    kernel: str
    path: str
    #: `None` on the runtime path, where there is no emitted module to read.
    #: Never an empty tuple, which would assert a measured absence of
    #: arithmetic — the distinction the numerical-behaviour record encodes.
    operations: tuple[str, ...] | None
    #: The emitted add tree and its classification, for an offline fold case.
    #: `None` on the runtime path and for the control, for the same reason
    #: `operations` is: the question was not asked rather than answered empty.
    shape: tuple[str, str] | None
    #: The fast-math licence set the emitted operations carry. `None` on the
    #: runtime path, which emits no module to read.
    licences: str | None
    applied: str | None
    results: tuple[int, ...]
    expected_operations: tuple[str, ...]


def build_host(destination: Path) -> None:
    result = run(
        [
            "xcrun",
            "--sdk",
            SDK,
            "clang",
            "-fobjc-arc",
            "-O0",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-framework",
            "Metal",
            "-framework",
            "Foundation",
            str(HOST_SOURCE),
            "-o",
            str(destination),
        ]
    )
    if result.returncode != 0:
        raise ProbeFailure(f"the dispatch host did not build: {result.stderr.strip()}")


def dispatch(host: Path, manifest: Path) -> tuple[dict[str, list[int]], dict[str, str], dict[str, str]]:
    """Run the whole manifest in one host invocation and parse what came back."""
    operands = ",".join(f"{pattern:08x}" for pattern in OPERANDS)
    result = run([str(host), "batch", str(manifest), f"f32={operands}"])
    if result.returncode == 3:
        raise ProbeFailure("no default Metal device resolved; this probe needs a GPU")
    if result.returncode != 0:
        raise ProbeFailure(
            f"the dispatch host exited {result.returncode}: {result.stderr.strip()}"
        )
    results: dict[str, list[int]] = {}
    applied: dict[str, str] = {}
    environment: dict[str, str] = {}
    current = None
    for line in result.stdout.splitlines():
        key, _, value = line.partition("=")
        if key == "case":
            current = value
            results[current] = []
        elif key == "applied" and current is not None:
            applied[current] = value
        elif key == "result" and current is not None:
            results[current].append(int(value, 16))
        elif key in ("device", "registry-id", "gpu-family-apple9"):
            environment[key] = value
        elif key == "runtime-compiler-image":
            # Every matching image, not the first: the host reports the whole
            # loaded population and naming one of several would be an
            # attribution the measurement did not make.
            existing = environment.get("runtime-compiler-image")
            environment["runtime-compiler-image"] = (
                f"{existing} {value}" if existing else value
            )
    return results, applied, environment


# --- verdicts -----------------------------------------------------------------


def quad_observation(results: tuple[int, ...], index: int) -> int | None:
    """The one value a quad's four lanes returned, or `None` if they disagree.

    Every lane of a quad computes the identical expression over the identical
    operands, so a disagreement is a per-lane nondeterminism this probe must
    refuse rather than average.
    """
    lanes = set(results[index * 4 : index * 4 + 4])
    return lanes.pop() if len(lanes) == 1 else None


# --- the record ---------------------------------------------------------------


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def repository_revision() -> str:
    result = run(["git", "-C", str(REPOSITORY), "rev-parse", "HEAD"])
    if result.returncode != 0:
        raise ProbeFailure("the repository revision could not be read")
    return result.stdout.strip()


def measure(perturb: str | None) -> tuple[list[tuple[str, str]], dict[str, str], dict[str, str]]:
    toolchain = resolve_toolchain()
    table = references()

    kernels = list(KERNELS)
    if perturb == "deleted-arithmetic":
        # The failure proof for admissibility. Both fold kernels become
        # pass-throughs while still declaring the three adds their references
        # are derived from, which is the shape finding 7 of the
        # numerical-behaviour record exists to refuse: a returned pattern from a
        # kernel whose arithmetic did not run is evidence about nothing.
        kernels[0:2] = [
            Kernel(kernel.name, kernel.purpose, (), "v4", kernel.expected_operations, True, 1)
            for kernel in KERNELS[0:2]
        ]
    if perturb == "written-order":
        # The failure proof for the metric itself: the serial kernel's body is
        # replaced by the split one's while its reference stays the serial
        # value, which is exactly what a compiler that reordered the chain
        # would have produced.
        kernels[0] = Kernel(
            KERNELS[0].name,
            KERNELS[0].purpose,
            SPLIT_STATEMENTS,
            KERNELS[0].result,
            KERNELS[0].expected_operations,
            fold=True,
            contributors=KERNELS[0].contributors,
        )
    workspace = Path(tempfile.mkdtemp(prefix="tiler-evaluation-order-"))
    sources: list[Path] = []
    try:
        for kernel in kernels:
            path = workspace / f"{kernel.name}.metal"
            path.write_text(kernel.source(), encoding="utf-8")
            sources.append(path)

        manifest_lines: list[str] = []
        observations: list[Observation] = []
        for kernel, source in zip(kernels, sources):
            for mode in MATH_MODES:
                for contract in CONTRACT_SETTINGS:
                    for level in OPTIMIZATION_LEVELS:
                        # The failure proof for the separation compiles the
                        # control in every cell under a selection that cannot
                        # fuse, so nothing fuses and the run must refuse rather
                        # than report an invariance it cannot attribute.
                        effective = contract
                        if perturb == "dead-contraction-axis" and not kernel.fold:
                            effective = "off"
                        key = f"offline.{kernel.name}.{mode}.{level}.contract-{contract}"
                        ir, library = compile_offline(
                            source, offline_flags(mode, effective, level), workspace, key
                        )
                        observations.append(
                            Observation(
                                key,
                                kernel.name,
                                "offline",
                                operation_list(ir),
                                fold_shape(ir) if kernel.fold else None,
                                fast_math_licences(ir),
                                None,
                                (),
                                kernel.expected_operations,
                            )
                        )
                        manifest_lines.append(
                            f"{key}\tf32\tlibrary\t{library}\t{ENTRY_POINT}"
                        )
        for kernel, source in zip(kernels, sources):
            for mode in MATH_MODES:
                for level in RUNTIME_OPTIMIZATION_LEVELS:
                    key = f"runtime.{kernel.name}.{mode}.opt-{level}"
                    # `MTLCompileOptions` has no contraction property, so the
                    # only selection that can keep the control from fusing on
                    # this path is the math mode.
                    effective_mode = mode
                    if perturb == "dead-contraction-axis" and not kernel.fold:
                        effective_mode = "safe"
                    options = (
                        f"math={effective_mode},fpfun={FP32_FUNCTIONS},"
                        f"lang={RUNTIME_LANGUAGE},opt={level}"
                    )
                    observations.append(
                        Observation(
                            key,
                            kernel.name,
                            "runtime",
                            None,
                            None,
                            None,
                            None,
                            (),
                            kernel.expected_operations,
                        )
                    )
                    manifest_lines.append(
                        f"{key}\tf32\tsource\t{source}\t{ENTRY_POINT}\t{options}"
                    )

        offline_count = sum(1 for entry in observations if entry.path == "offline")
        runtime_count = sum(1 for entry in observations if entry.path == "runtime")
        if offline_count != DECLARED_OFFLINE_CASES or runtime_count != DECLARED_RUNTIME_CASES:
            raise ProbeFailure(
                f"population mismatch: built {offline_count} offline and {runtime_count} "
                f"runtime cases, expected {DECLARED_OFFLINE_CASES} and {DECLARED_RUNTIME_CASES}"
            )

        manifest = workspace / "manifest.tsv"
        manifest.write_text("\n".join(manifest_lines) + "\n", encoding="utf-8")
        host = workspace / "numerical_probe_host"
        build_host(host)
        results, applied, device_environment = dispatch(host, manifest)

        for entry in observations:
            if entry.key not in results:
                raise ProbeFailure(f"the host returned no results for {entry.key}")
            entry.results = tuple(results[entry.key])
            entry.applied = applied.get(entry.key)
            if len(entry.results) != len(OPERANDS):
                raise ProbeFailure(
                    f"{entry.key} returned {len(entry.results)} results, expected {len(OPERANDS)}"
                )

        rows, summary = classify(observations, table)
        retained_sources = {source.name: source.read_text(encoding="utf-8") for source in sources}
        environment = {
            "environment.date_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            "environment.os_version": toolchain.os_version,
            "environment.os_build": toolchain.os_build,
            "environment.architecture": toolchain.architecture,
            "environment.xcode": toolchain.xcode,
            "environment.xcode_path": toolchain.xcode_path,
            "environment.sdk": f"{SDK} {toolchain.sdk_version} build {toolchain.sdk_build}",
            "environment.offline_compiler": toolchain.metal_version,
            "environment.offline_compiler_installed_dir": toolchain.metal_installed_dir,
            "environment.runtime_compiler_image": device_environment.get(
                "runtime-compiler-image", "unrecovered"
            ),
            "environment.device": device_environment.get("device", "unrecovered"),
            "environment.device_registry_id": device_environment.get("registry-id", "unrecovered"),
            "environment.device_family_apple9": device_environment.get(
                "gpu-family-apple9", "unrecovered"
            ),
        }
        environment.update(summary)
        return rows, environment, retained_sources
    finally:
        shutil.rmtree(workspace, ignore_errors=True)


def classify(
    observations: list[Observation], table: dict[tuple[str, str], Reference]
) -> tuple[list[tuple[str, str]], dict[str, str]]:
    rows: list[tuple[str, str]] = []
    counted: dict[str, int] = {}
    shapes: dict[str, int] = {}
    for entry in sorted(observations, key=lambda item: item.key):
        if entry.operations is not None:
            # Layer one. An operation list that is not the one the source names
            # means the module under test is not the program this probe wrote,
            # and no verdict below is about it.
            if entry.operations != tuple(sorted(entry.expected_operations)):
                raise ProbeFailure(
                    f"{entry.key}: emitted {render_operations(entry.operations)}, where this "
                    f"kernel declares "
                    f"{render_operations(tuple(sorted(entry.expected_operations)))}"
                )
            rows.append(
                (f"case.{entry.key}.float_operations", render_operations(entry.operations))
            )
        if entry.shape is not None:
            shape, tree = entry.shape
            rows.append((f"case.{entry.key}.fold_shape", f"{shape},tree={tree}"))
            shapes[f"{entry.kernel}:{tree}"] = shapes.get(f"{entry.kernel}:{tree}", 0) + 1
        if entry.licences is not None:
            rows.append((f"case.{entry.key}.fast_math_licences", entry.licences))
        if entry.applied is not None:
            rows.append((f"case.{entry.key}.applied", entry.applied))
        rows.append(
            (
                f"case.{entry.key}.results",
                " ".join(f"{value:08x}" for value in entry.results),
            )
        )
        for index, quad in enumerate(QUADS):
            observed = quad_observation(entry.results, index)
            if observed is None:
                raise ProbeFailure(
                    f"{entry.key}: the four lanes of quad {quad.name} disagree, which is a "
                    "per-lane nondeterminism this probe refuses to publish"
                )
            reference = table[(entry.kernel, quad.name)]
            verdict = reference.verdict(observed)
            rows.append(
                (
                    f"case.{entry.key}.verdict.{quad.name}",
                    f"observed={observed:08x},written={reference.written:08x},verdict={verdict}",
                )
            )
            if verdict == "unexpected":
                raise ProbeFailure(
                    f"{entry.key}: quad {quad.name} returned {observed:08x}, which is neither "
                    f"the written order's {reference.written:08x} nor any value the freedom "
                    "under test admits"
                )
            if quad.name == "witness" and verdict != "as-written-and-freedom-independent":
                raise ProbeFailure(
                    f"{entry.key}: the execution witness returned {observed:08x}, so this case "
                    "supports no claim about what its arithmetic did"
                )
            probed = "seed" if entry.kernel != "contraction_control" else "contraction"
            if quad.name == probed:
                name = f"summary.{entry.kernel}.{entry.path}.{verdict}"
                counted[name] = counted.get(name, 0) + 1

    fused = sum(
        count
        for name, count in counted.items()
        if name.startswith("summary.contraction_control.") and name.endswith(".diverged")
    )
    if fused == 0:
        raise ProbeFailure(
            "the contraction control never fused, so the contraction axis is not live in this "
            "run and the fold kernels' invariance under it attributes to nothing"
        )
    summary = {name: str(count) for name, count in sorted(counted.items())}
    for name, count in sorted(shapes.items()):
        kernel, tree = name.split(":")
        summary[f"summary.emitted_tree.{kernel}.{tree}"] = str(count)
    return rows, summary


def emit(rows: list[tuple[str, str]], environment: dict[str, str]) -> str:
    table = references()
    header = [
        ("schema", SCHEMA),
        ("probe.repository_base_revision", repository_revision()),
        ("probe.producer_sha256", digest(Path(__file__).resolve())),
        ("probe.host_source_sha256", digest(HOST_SOURCE)),
        ("probe.language", LANGUAGE),
        ("probe.offline_target", OFFLINE_TARGET),
        ("probe.runtime_language", RUNTIME_LANGUAGE),
        ("probe.fp32_functions", FP32_FUNCTIONS),
        ("probe.math_modes", ",".join(MATH_MODES)),
        ("probe.contract_settings", ",".join(CONTRACT_SETTINGS)),
        ("probe.optimization_levels", ",".join(OPTIMIZATION_LEVELS)),
        ("probe.runtime_optimization_levels", ",".join(RUNTIME_OPTIMIZATION_LEVELS)),
        ("probe.operands", ",".join(f"{pattern:08x}" for pattern in OPERANDS)),
        ("probe.quads", ",".join(quad.name for quad in QUADS)),
        ("probe.kernels", ",".join(kernel.name for kernel in KERNELS)),
    ]
    for name, value in sorted(environment.items()):
        header.append((name, value))
    for kernel in KERNELS:
        for quad in QUADS:
            reference = table[(kernel.name, quad.name)]
            alternatives = ",".join(sorted(f"{value:08x}" for value in reference.alternatives))
            header.append(
                (
                    f"reference.{kernel.name}.{quad.name}",
                    f"written={reference.written:08x},"
                    f"alternatives={alternatives if alternatives else 'none'}",
                )
            )
    lines = ["key\tvalue"]
    lines += [f"{name}\t{value}" for name, value in header + rows]
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--retain",
        action="store_true",
        help="write results/<date>-<identity>/ with the record and the exact kernel sources",
    )
    parser.add_argument(
        "--perturb",
        choices=("written-order", "dead-contraction-axis", "deleted-arithmetic"),
        help=(
            "run a failure proof rather than a measurement, printing the demonstration to "
            "stderr and no record: 'written-order' emits the split chain under the serial "
            "kernel's name, so the metric must report it diverging from the order that "
            "kernel's reference names; 'dead-contraction-axis' compiles the control under a "
            "selection that cannot fuse in any cell, so the run must refuse to publish"
        ),
    )
    arguments = parser.parse_args()
    if arguments.perturb and arguments.retain:
        parser.error("a perturbed run is a failure proof and must not be retained as a record")
    try:
        rows, environment, sources = measure(arguments.perturb)
        record = emit(rows, environment)
    except ProbeFailure as failure:
        print(f"order_probe: {failure}", file=sys.stderr)
        return 1
    if arguments.perturb in ("dead-contraction-axis", "deleted-arithmetic"):
        # Reaching here means the run published despite the perturbation, which
        # is the perturbation not firing: the gate it exercises is a refusal, so
        # a successful demonstration exits through the `ProbeFailure` above.
        print(
            f"order_probe: the {arguments.perturb} perturbation did not fire, so this run "
            "does not demonstrate that the guard it exercises can refuse",
            file=sys.stderr,
        )
        return 1
    if arguments.perturb == "written-order":
        # Unperturbed, `serial_fold4` is `as-written` in every case: it is the
        # form this compiler canonicalizes to. Perturbed, its emitted order is
        # the split and its reference is still the serial value, so a metric
        # that can detect a reordering must report it diverging.
        diverged = sum(
            int(count)
            for name, count in environment.items()
            if name.startswith("summary.serial_fold4.") and name.endswith(".diverged")
        )
        print(
            f"order_probe: the written-order perturbation made serial_fold4 diverge in "
            f"{diverged} cases, where the unperturbed run reports 0",
            file=sys.stderr,
        )
        return 0 if diverged > 0 else 1
    if not arguments.retain:
        sys.stdout.write(record)
        return 0
    toolchain = resolve_toolchain()
    stamp = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    identity = (
        f"{stamp}-evaluation-order-macos{toolchain.os_version.split('.')[0]}"
        f"-msl4-{compiler_build_tag(toolchain)}"
    )
    destination = HERE / "results" / identity
    if destination.exists():
        raise SystemExit(f"order_probe: {destination} already exists; move it before retaining")
    staged = destination.with_name(destination.name + ".staging")
    (staged / "sources").mkdir(parents=True)
    for name, text in sorted(sources.items()):
        (staged / "sources" / name).write_text(text, encoding="utf-8")
    (staged / "record.tsv").write_text(record, encoding="utf-8")
    staged.rename(destination)
    print(f"order_probe: retained {destination.relative_to(REPOSITORY)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
