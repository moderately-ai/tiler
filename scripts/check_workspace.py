#!/usr/bin/env python3
"""Validate Tiler's exact Rust workspace and resolved Cargo boundary."""

from __future__ import annotations

import json
import re
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]

EXPECTED_MEMBERS = (
    "crates/tiler-artifact",
    "crates/tiler-cache",
    "crates/tiler-compiler",
    "crates/tiler-ir",
    "crates/tiler-metal",
    "crates/tiler-metal-aot",
    "crates/tiler-reference",
    "crates/tiler-runtime",
    "prototypes/serial-sum-compile",
    "prototypes/serial-sum-run",
)
EXPECTED_EXCLUDES = (
    "spikes/extensions/operation-api",
    "spikes/extensions/proc-macro-visibility",
    "spikes/indexing/index-access-model",
    "spikes/macro-environment/fixture",
)
EXPECTED_WORKSPACE_PACKAGE = {
    "version": "0.0.0",
    "edition": "2024",
    "license": "MIT OR Apache-2.0",
    "repository": "https://github.com/moderately-ai/tiler",
    "publish": False,
}
EXPECTED_WORKSPACE_DEPENDENCIES: dict[str, object] = {
    "num-bigint": "0.4.6",
    "metal": "0.33.0",
    "num-integer": "0.1.46",
    "num-traits": "0.2.19",
    "tiler-artifact": {"path": "crates/tiler-artifact"},
    "tiler-cache": {"path": "crates/tiler-cache"},
    "tiler-compiler": {"path": "crates/tiler-compiler"},
    "tiler-ir": {"path": "crates/tiler-ir"},
    "tiler-metal": {"path": "crates/tiler-metal"},
    "tiler-metal-aot": {"path": "crates/tiler-metal-aot"},
    "tiler-reference": {"path": "crates/tiler-reference"},
    "tiler-runtime": {"path": "crates/tiler-runtime"},
    "trybuild": "1.0.114",
}
EXPECTED_RUST_LINTS = {"missing_docs": "warn", "unsafe_code": "forbid"}
# Members that deliberately do not inherit `[workspace.lints]`, with the exact
# table each is permitted instead.
#
# The workspace forbids `unsafe_code`, and `forbid` cannot be relaxed by an
# inner attribute at any scope, so a crate that must call an Objective-C API
# cannot inherit it. The runtime proof reaches `MTLBuffer` storage through the
# raw pointer `Buffer::contents` returns, which no Metal binding exposes safely.
#
# The exception is pinned rather than merely allowed, in three ways that matter:
# only the named member may diverge, its table must match *exactly*, and that
# table says `deny` rather than `allow`. So unsafe stays a hard error throughout
# that crate too, except at the individual functions that opt in by name with a
# stated reason — and a later edit widening `deny` to `allow`, or a second crate
# quietly dropping inheritance, fails this check.
UNINHERITED_LINT_MEMBERS = {
    "tiler-prototype-run": {
        "rust": {"missing_docs": "warn", "unsafe_code": "deny"},
        "clippy": {
            "all": {"level": "warn", "priority": -1},
            "pedantic": {"level": "warn", "priority": -1},
        },
    }
}
EXPECTED_CLIPPY_LINTS = {
    "all": {"level": "warn", "priority": -1},
    "pedantic": {"level": "warn", "priority": -1},
}
EXPECTED_RUSTFMT = {"edition": "2024", "max_width": 100}

PACKAGE_DESCRIPTIONS = {
    "tiler-artifact": "Target-neutral artifact and execution contracts for Tiler",
    "tiler-cache": "Cross-process expansion cache protocol for Tiler",
    "tiler-compiler": "Target-independent optimization and scheduling for Tiler",
    "tiler-ir": "Target-independent tensor compiler representations for Tiler",
    "tiler-metal": "Pure structured-kernel-to-Metal-source lowering for Tiler",
    "tiler-metal-aot": "Offline Apple Metal compiler driver for Tiler",
    "tiler-reference": "Target-independent executable reference semantics for Tiler",
    "tiler-runtime": "Device-free artifact loading and validation for Tiler runtimes",
    "tiler-prototype-compile": "Non-published producer for Tiler's serial-Sum value proof",
    "tiler-prototype-run": "Non-published runner for Tiler's serial-Sum value proof",
}
PACKAGE_DIRS = {
    "tiler-artifact": "crates/tiler-artifact",
    "tiler-cache": "crates/tiler-cache",
    "tiler-compiler": "crates/tiler-compiler",
    "tiler-ir": "crates/tiler-ir",
    "tiler-metal": "crates/tiler-metal",
    "tiler-metal-aot": "crates/tiler-metal-aot",
    "tiler-reference": "crates/tiler-reference",
    "tiler-runtime": "crates/tiler-runtime",
    "tiler-prototype-compile": "prototypes/serial-sum-compile",
    "tiler-prototype-run": "prototypes/serial-sum-run",
}


def dependency(
    name: str,
    *,
    kind: str | None = None,
    path: str | None = None,
    requirement: str = "*",
    source: str | None = None,
    target: str | None = None,
) -> dict[str, object]:
    """Build one exact normalized Cargo metadata dependency contract.

    `target` is the `cfg(...)` expression a platform-conditional edge is
    declared under, and `None` means the edge is unconditional. It is pinned
    rather than ignored because narrowing an edge to one platform decides where
    the workspace can be compiled at all: an Apple-only crate declared
    unconditionally makes `cargo check --workspace` impossible on every other
    host, and widening one back is the same change in reverse.
    """
    return {
        "name": name,
        "source": source,
        "req": requirement,
        "kind": kind,
        "rename": None,
        "optional": False,
        "uses_default_features": True,
        "features": [],
        "target": target,
        "registry": None,
        "path": path,
    }


CRATES_IO = "registry+https://github.com/rust-lang/crates.io-index"
EXPECTED_DEPENDENCIES = {
    "tiler-artifact": [dependency("tiler-ir", path="crates/tiler-ir")],
    # The expansion cache's single edge is a decided property in the same sense
    # as the driver's empty closure and the loader's single edge (ADR 0082). It
    # reaches `tiler-artifact` for exactly two things ADR 0050 requires and a
    # storage protocol cannot supply itself: the governed digest
    # `tiler.digest.sha-256.v1`, which validates a stored bundle's section
    # digests, and `decode_artifact`, which re-proves the carried envelope's
    # manifest, section digests, and canonical identity on every hit. A local
    # hash function would make it a second identity authority over one subject,
    # which is what made the previous owner assignment unsatisfiable.
    "tiler-cache": [dependency("tiler-artifact", path="crates/tiler-artifact")],
    "tiler-compiler": [
        dependency("tiler-ir", path="crates/tiler-ir"),
        dependency("tiler-reference", kind="dev", path="crates/tiler-reference"),
    ],
    "tiler-ir": [
        dependency("num-bigint", requirement="^0.4.6", source=CRATES_IO),
        dependency("num-integer", requirement="^0.1.46", source=CRATES_IO),
        dependency("num-traits", requirement="^0.2.19", source=CRATES_IO),
        dependency("trybuild", kind="dev", requirement="^1.0.114", source=CRATES_IO),
    ],
    # The offline driver edge is deliberately development-only: `tiler-metal`
    # emits source and must not acquire Apple tool discovery at build time, and
    # keeping the edge out of the normal graph leaves the eventual
    # `tiler-metal-aot` -> `tiler-metal` direction available.
    "tiler-metal": [
        dependency("tiler-artifact", path="crates/tiler-artifact"),
        dependency("tiler-ir", path="crates/tiler-ir"),
        dependency("tiler-metal-aot", kind="dev", path="crates/tiler-metal-aot"),
    ],
    "tiler-metal-aot": [],
    "tiler-reference": [dependency("tiler-ir", path="crates/tiler-ir")],
    # The device-free loader's closure is a decided property, not an accident of
    # ordering (ADR 0081). It is the whole substance of the crate: a loader that
    # acquired `tiler-compiler` could rebuild a plan instead of validating one,
    # and one that acquired a platform binding would stop being decidable
    # without hardware. `tiler-ir` is absent as a *direct* edge deliberately —
    # everything the loader names is an artifact-layer type — even though
    # `tiler-artifact` links it transitively.
    "tiler-runtime": [dependency("tiler-artifact", path="crates/tiler-artifact")],
    "tiler-prototype-compile": [
        dependency("tiler-artifact", path="crates/tiler-artifact"),
        dependency("tiler-compiler", path="crates/tiler-compiler"),
        dependency("tiler-ir", path="crates/tiler-ir"),
        dependency("tiler-metal", path="crates/tiler-metal"),
        # The producer is the one component that sees the emitter's and the
        # driver's target vocabularies at once. Neither backend crate may depend
        # on the other, so the production translation between them can only live
        # here; `tiler_metal::target_correspondence` records that its
        # orchestrator inherits the obligation its tests state.
        dependency("tiler-metal-aot", path="crates/tiler-metal-aot"),
        dependency("tiler-reference", path="crates/tiler-reference"),
    ],
    "tiler-prototype-run": [
        # The runtime proof is the one member that talks to a device, so it is
        # the one member with a Metal binding. It is conditional because `metal`
        # links Apple frameworks and cannot build anywhere else; declaring it
        # unconditionally made `cargo check --workspace` structurally impossible
        # on the supported GNU Linux host, which is the profile that carries the
        # evidence that the compiler core is target-independent.
        dependency(
            "metal",
            requirement="^0.33.0",
            source=CRATES_IO,
            target='cfg(target_os = "macos")',
        ),
        dependency("tiler-artifact", path="crates/tiler-artifact"),
        dependency("tiler-compiler", path="crates/tiler-compiler"),
        dependency("tiler-ir", path="crates/tiler-ir"),
        dependency("tiler-metal", path="crates/tiler-metal"),
        dependency("tiler-metal-aot", path="crates/tiler-metal-aot"),
        dependency("tiler-reference", path="crates/tiler-reference"),
        # The runtime proof dispatches twice: once from bytes this process
        # compiled, and once from an artifact envelope the producer wrote to a
        # file. The second path is the whole point of the edge, and the first is
        # retained as the control that tells an envelope defect from a compiler
        # defect. Only this member has both a device and a loader, which is why
        # the edge lands here rather than widening `tiler-runtime`'s own closure
        # — that closure stays exactly `tiler-artifact` (ADR 0081).
        dependency("tiler-runtime", path="crates/tiler-runtime"),
    ],
}

# Every site admitted under ADR 0079, keyed by `(package-relative path, item)`
# with the exact `reason` the attribute states.
#
# ADR 0079 permits unsafe "case by case, at an individual function or module",
# and says in terms that a third site is a new decision rather than an
# application of the existing one. `UNINHERITED_LINT_MEMBERS` above pins the
# *crate* half of that boundary; this table pins the *site* half, which is the
# half the record's Consequences name as unchecked.
#
# The pin is a `(path, item, reason)` triple rather than a count or a bare
# location. A count passes when a site moves or when one is added while another
# is deleted. A location alone checks where the permission sits but not what it
# claims, and the `reason` is the whole substance of ADR 0079 item 3's second
# condition — weakening a justification is exactly the change a reviewer is
# meant to see. The accepted cost is that renaming a function, changing its
# signature, or rewording a reason churns this table; each of those genuinely
# changes what was admitted.
ADMITTED_UNSAFE_SITES: dict[tuple[str, str], str] = {
    (
        "prototypes/serial-sum-run/src/buffer.rs",
        "pub fn write_f32(buffer: &Buffer, values: &[f32])",
    ): (
        "MTLBuffer storage is reachable only through the raw pointer "
        "`Buffer::contents` returns; no Metal binding exposes it safely. The write "
        "is bounded by an asserted length check against the buffer's own byte "
        "length, copies a plain-old-data type with no destructor, and retains no "
        "borrow."
    ),
    (
        "prototypes/serial-sum-run/src/buffer.rs",
        "pub fn read_f32(buffer: &Buffer, count: usize) -> Vec<f32>",
    ): (
        "the read half of the same constraint: MTLBuffer storage is reachable only "
        "through `Buffer::contents`. Bounded by an asserted length check, reads a "
        "plain-old-data type, and copies out rather than retaining a borrow of "
        "device memory."
    ),
}

# Every definition of canonical length framing the workspace is permitted,
# keyed by `(package-relative path, item signature)`.
#
# Identity in this workspace is a digest over a canonical encoding, so two
# encoders disagreeing by one byte name the same subject with two different
# identities and nothing downstream can tell that from two genuinely different
# subjects. `tiler_ir::identity` therefore owns the framing, and a crate that
# can reach it has no permitted copy of its own.
#
# The pin exists because a *convention* did not hold this. `tiler_ir::identity`
# stated the rule in its own module documentation while five copies grew inside
# that crate; two more then grew in `tiler-compiler`; and the crate-local test
# that replaced the prose could not see any other crate. This table is the
# workspace-wide form, and it is a table rather than a bare "zero copies
# outside `tiler-ir`" rule because one crate genuinely may not import the
# framing: ADR 0077 item 2 pins `tiler-metal-aot`'s dependency closure empty, so
# its copy is forced by an accepted decision rather than being a defect.
#
# Each value states why the site is admitted, and names a `citation` the site's
# own documentation must contain. The reason is Python-side and the citation is
# source-verified, which is the one deliberate difference from
# `ADMITTED_UNSAFE_SITES` above: an admitted `unsafe` site carries a
# `reason = "..."` attribute the compiler already requires, and a framing helper
# has no such attribute — only a doc comment, whose rustdoc structure would
# churn this table on edits that change nothing about the admission. Pinning the
# citation keeps the load-bearing half checked: a copy whose documentation stops
# naming the decision that forces it stops being admitted.
FRAMING_SITE_CITATIONS: dict[tuple[str, str], tuple[str, str]] = {
    (
        "crates/tiler-ir/src/identity.rs",
        "pub fn push_len(bytes: &mut Vec<u8>, len: usize)",
    ): (
        "the workspace's sole canonical length framing. Every crate that can reach "
        "`tiler-ir` calls this and defines none of its own.",
        "sole definition of canonical length framing",
    ),
    (
        "crates/tiler-ir/src/identity.rs",
        "pub fn push_slice(bytes: &mut Vec<u8>, value: &[u8])",
    ): (
        "the length-prefixed byte run built from `push_len`, admitted beside it as "
        "one primitive pair rather than a second framing.",
        "one primitive pair",
    ),
    (
        "crates/tiler-cache/src/expansion/subject.rs",
        "fn push_count(bytes: &mut Vec<u8>, count: usize)",
    ): (
        "forced by ADR 0082 item 2, which decides this crate's closure is exactly "
        "`tiler-artifact` — and states in terms that `tiler-ir` is 'an edge this "
        "record decides the crate does not have'. `ComposedSubject` is a genuine "
        "canonical identity preimage, so this is the same admission as the driver's "
        "and not a classification of the framing as something else.",
        "ADR 0082 item 2",
    ),
    (
        "crates/tiler-cache/src/expansion/subject.rs",
        "fn push_run(bytes: &mut Vec<u8>, run: &[u8])",
    ): (
        "the length-prefixed run built from the `push_count` above, admitted with it "
        "under the same ADR 0082 item 2 closure.",
        "Admitted alongside",
    ),
    (
        "crates/tiler-artifact/src/program/codec/encode.rs",
        "bytes.extend_from_slice(&ordinal(envelope.sections().len()).to_be_bytes());",
    ): (
        "not canonical identity framing but a four-byte field of the envelope's "
        "fixed-width header, read back as `cursor.u32()` by `decode.rs` and sized by "
        "the `u32` ordinal space the envelope's tables are indexed in. Widening it "
        "to the eight-byte framing beside it on the previous line would change the "
        "artifact ABI, not remove a duplicate.",
        "not `tiler_ir::identity` framing",
    ),
    (
        "crates/tiler-metal-aot/src/identity.rs",
        "pub(crate) fn push_len(bytes: &mut Vec<u8>, len: usize)",
    ): (
        "forced by ADR 0077 item 2, which pins this crate's dependency closure "
        "empty: it declares no workspace dependency, so `tiler_ir::identity` is not "
        "importable here and the framing has to be restated. Admitted once, so a "
        "second copy in this crate is a diff someone must look at — which is how "
        "`family.rs` came to frame its own subject in four bytes while this one "
        "framed the compilation subject in eight.",
        "ADR 0077 item 2",
    ),
    (
        "crates/tiler-metal-aot/src/identity.rs",
        "pub(crate) fn push_str(bytes: &mut Vec<u8>, value: &str)",
    ): (
        "the textual run built from the `push_len` above, admitted with it under the "
        "same ADR 0077 item 2 closure.",
        "Admitted alongside",
    ),
    (
        "crates/tiler-cache/src/expansion/bundle.rs",
        "bytes.extend_from_slice(&(sections.len() as u64).to_be_bytes());",
    ): (
        "not canonical identity framing but a fixed-offset field of a decodable "
        "container: it is written at `SECTION_COUNT_AT` and read back by "
        "`read_u64`, the section bytes it counts are located by explicit descriptor "
        "offsets rather than by following a prefix, and the bundle bytes are never "
        "digested into an identity. `tiler-cache` therefore does not acquire "
        "`tiler-ir` to write it; its closure stays exactly `tiler-artifact` under "
        "ADR 0082, for the same reason `tiler-runtime` keeps `tiler-ir` out as a "
        "direct edge.",
        "not `tiler_ir::identity` framing",
    ),
}

# A canonical-encoding primitive's structural shape: a `&mut Vec<u8>` sink plus
# exactly one payload it frames.
#
# Recognizing the *signature* rather than a name is deliberate. Every copy this
# check exists to catch was found by name, and every name list has been
# incomplete: the crate-local test that preceded this one listed `push_len`,
# `push_slice`, `encode_len`, and `encode_bytes`, and `tiler-compiler`'s
# `region.rs` held an `encode_count` none of them matched.
FRAMING_SINK = "&mutVec<u8>"
FRAMING_PAYLOADS = frozenset({"usize", "&[u8]", "&str"})
FRAMING_FN = re.compile(r"\bfn\s+[A-Za-z_][A-Za-z0-9_]*\s*[(<]")
# The helper-less shape: a length converted and written in one statement. The
# fifth copy inside `tiler-ir` was written this way, with no helper at all.
FRAMING_LENGTH_SOURCE = re.compile(r"\.(?:len|rank)\s*\(\s*\)")
FRAMING_FIXED_WIDTH = re.compile(r"\bto_(?:be|le)_bytes\s*\(\s*\)")
CFG_TEST_ATTRIBUTE = re.compile(r"#\[cfg\(test\)\]")
MODULE_DECLARATION = re.compile(r"\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;")
RAW_STRING_OPENING = re.compile(r"b?r(#*)\"")
STRING_OPENING = re.compile(r"b?\"")
CHAR_LITERAL = re.compile(r"b?'(?:\\[^']*|[^'\\])'")

# `unsafe_code` as a whole token, so `unsafe_codegen` or a longer identifier
# does not match.
UNSAFE_CODE_TOKEN = re.compile(r"\bunsafe_code\b")
ALLOW_ATTRIBUTE_START = re.compile(r"^#!?\[allow\(")
ATTRIBUTE_START = re.compile(r"^#!?\[")
REASON_LITERAL = re.compile(r'\breason\s*=\s*"((?:[^"\\]|\\.)*)"')
STRING_LITERAL = re.compile(r'"(?:[^"\\]|\\.)*"')


def expected_member_manifest(name: str) -> dict[str, object]:
    """Return the complete authored manifest contract for one package."""
    manifest: dict[str, object] = {
        "package": {
            "name": name,
            "description": PACKAGE_DESCRIPTIONS[name],
            "version": {"workspace": True},
            "edition": {"workspace": True},
            "license": {"workspace": True},
            "repository": {"workspace": True},
            "publish": {"workspace": True},
        }
    }
    if name.startswith("tiler-prototype-"):
        manifest["bin"] = [
            {
                "name": name,
                "path": "src/main.rs",
                "test": True,
                "doc": True,
            }
        ]
    else:
        manifest["lib"] = {"test": True, "doctest": True, "doc": True}
    normal = {
        item["name"]: {"workspace": True}
        for item in EXPECTED_DEPENDENCIES[name]
        if item["kind"] is None and item["target"] is None
    }
    development = {
        item["name"]: {"workspace": True}
        for item in EXPECTED_DEPENDENCIES[name]
        if item["kind"] == "dev" and item["target"] is None
    }
    # A platform-conditional edge is authored under `[target.<cfg>.dependencies]`
    # rather than beside the unconditional ones, so the expected manifest has to
    # reproduce that nesting; the resolved comparison below still sees one flat
    # edge list carrying the `cfg` on the edge itself.
    conditional: dict[str, object] = {}
    for item in EXPECTED_DEPENDENCIES[name]:
        if item["target"] is None:
            continue
        table = "dev-dependencies" if item["kind"] == "dev" else "dependencies"
        section = conditional.setdefault(str(item["target"]), {})
        section.setdefault(table, {})[item["name"]] = {"workspace": True}  # type: ignore[index]
    if normal:
        manifest["dependencies"] = normal
    if development:
        manifest["dev-dependencies"] = development
    if conditional:
        manifest["target"] = conditional
    manifest["lints"] = UNINHERITED_LINT_MEMBERS.get(name, {"workspace": True})
    return manifest


def outside_string_literals(line: str) -> str:
    """Return `line` with every double-quoted run removed.

    Brackets are counted to find where an attribute ends, and a `reason` string
    may legitimately contain one. Counting them would end the group early and
    report a confusing violation of a site that is in fact well formed.

    Bounded to a literal that opens and closes on one line, which is the form
    both admitted sites use and the form `rustfmt` preserves — it does not
    reflow string literals unless `format_strings` is enabled, and
    `rustfmt.toml` sets only `edition` and `max_width`. A literal deliberately
    written across lines would be miscounted, and the resulting failure is a
    stopped gate rather than an accepted site.
    """
    return STRING_LITERAL.sub("", line)


def attribute_span(lines: list[str], start: int) -> int:
    """Return the exclusive end index of the attribute beginning at `start`.

    An attribute is accumulated until its bracket balance closes, so a
    multi-line attribute is one span — the form a substring search over single
    lines splits and misses.
    """
    depth = 0
    cursor = start
    while cursor < len(lines):
        bare = outside_string_literals(lines[cursor])
        depth += bare.count("[") + bare.count("(")
        depth -= bare.count("]") + bare.count(")")
        cursor += 1
        if depth <= 0:
            break
    return cursor


def attribute_groups(lines: list[str]) -> list[tuple[int, int, str]]:
    """Return every `#[allow(...)]`/`#![allow(...)]` group as `(start, end, text)`.

    Indices are 0-based and `end` is exclusive.
    """
    groups: list[tuple[int, int, str]] = []
    index = 0
    while index < len(lines):
        if not ALLOW_ATTRIBUTE_START.match(lines[index].strip()):
            index += 1
            continue
        end = attribute_span(lines, index)
        groups.append((index, end, "\n".join(lines[index:end])))
        index = end
    return groups


def following_item(lines: list[str], start: int) -> str | None:
    """Return the normalized signature of the item an attribute group precedes.

    Further attributes, doc comments, ordinary comments, and blank lines are
    skipped, so a site carrying `#[must_use]` beneath its `#[allow]` still names
    its function; an attribute skipped this way is skipped by its whole span, so
    a multi-line one does not leave its continuation mistaken for a signature. A
    trailing brace is dropped and interior whitespace collapsed, so the pin does
    not churn on reformatting that leaves the signature unchanged.
    """
    cursor = start
    while cursor < len(lines):
        stripped = lines[cursor].strip()
        if not stripped or stripped.startswith("//"):
            cursor += 1
            continue
        if ATTRIBUTE_START.match(stripped):
            cursor = attribute_span(lines, cursor)
            continue
        return " ".join(stripped.removesuffix("{").split())
    return None


def scan_unsafe_allow_sites(
    root: Path, package_dirs: dict[str, str]
) -> tuple[dict[tuple[str, str], str], list[str]]:
    """Return every admitted-unsafe site found under the governed packages.

    The scan is textual and its limits are stated rather than left to be
    discovered. It recognizes `unsafe_code` only inside an `#[allow(...)]` or
    `#![allow(...)]` group that begins a line, and it ignores the token on a
    line-comment line so this file's own prose and `buffer.rs`'s module
    documentation do not register as sites. Every *other* occurrence is reported
    as unaccounted-for, which is the fail-closed direction: a `cfg_attr`, a
    macro-generated attribute, a block comment, or a string literal holding the
    token stops the gate until someone decides what it is, rather than passing
    unseen.

    Spike workspaces are deliberately out of range. They are Cargo workspaces
    excluded from this one, none is a shipping component, and the three that
    mention the lint at all declare `#![forbid(unsafe_code)]`.

    Only `#[allow]` sites are pinned, because the compiler already guarantees
    they are the complete set: ADR 0079 item 2 keeps `unsafe_code` at `deny` or
    `forbid` in every member, so an `unsafe` block that no attribute admits does
    not build.
    """
    sites: dict[tuple[str, str], str] = {}
    errors: list[str] = []
    for package_dir in sorted(set(package_dirs.values())):
        base = root / package_dir
        for source in sorted(base.rglob("*.rs")):
            relative_path = source.relative_to(root).as_posix()
            try:
                lines = source.read_text(encoding="utf-8").splitlines()
            except (OSError, UnicodeError) as error:
                errors.append(f"unsafe-sites.{relative_path}: cannot read: {error}")
                continue
            accounted: set[int] = set()
            for start, end, text in attribute_groups(lines):
                if not UNSAFE_CODE_TOKEN.search(text):
                    continue
                accounted.update(range(start, end))
                item = following_item(lines, end)
                if item is None:
                    errors.append(
                        f"unsafe-sites.{relative_path}:{start + 1}: the attribute admits "
                        "unsafe_code but precedes no item"
                    )
                    continue
                reason = REASON_LITERAL.search(text)
                if reason is None:
                    errors.append(
                        f"unsafe-sites.{relative_path}:{start + 1}: `{item}` admits "
                        "unsafe_code without the `reason` ADR 0079 item 3 requires"
                    )
                    continue
                key = (relative_path, item)
                if key in sites:
                    errors.append(
                        f"unsafe-sites.{relative_path}: `{item}` admits unsafe_code twice"
                    )
                    continue
                sites[key] = " ".join(reason.group(1).split())
            for number, line in enumerate(lines):
                if number in accounted or line.strip().startswith("//"):
                    continue
                if UNSAFE_CODE_TOKEN.search(line):
                    errors.append(
                        f"unsafe-sites.{relative_path}:{number + 1}: `unsafe_code` appears "
                        "outside a recognized `#[allow(...)]` attribute"
                    )
    return sites, errors


def validate_unsafe_site_pins(
    root: Path,
    *,
    package_dirs: dict[str, str] | None = None,
    admitted: dict[tuple[str, str], str] | None = None,
    diverging_members: dict[str, object] | None = None,
) -> list[str]:
    """Return typed violations of the per-site half of ADR 0079."""
    package_dirs = PACKAGE_DIRS if package_dirs is None else package_dirs
    admitted = ADMITTED_UNSAFE_SITES if admitted is None else admitted
    diverging = UNINHERITED_LINT_MEMBERS if diverging_members is None else diverging_members

    found, errors = scan_unsafe_allow_sites(root, package_dirs)
    for key in sorted(found.keys() - admitted.keys()):
        errors.append(
            f"unsafe-sites.{key[0]}: `{key[1]}` admits unsafe_code and is not pinned; "
            "ADR 0079 makes a new site a new decision"
        )
    for key in sorted(admitted.keys() - found.keys()):
        errors.append(
            f"unsafe-sites.{key[0]}: pinned site `{key[1]}` is gone; remove its pin in the "
            "same change that removes it"
        )
    for key in sorted(found.keys() & admitted.keys()):
        if found[key] != admitted[key]:
            errors.append(
                f"unsafe-sites.{key[0]}: `{key[1]}` states reason {found[key]!r}, pinned "
                f"as {admitted[key]!r}"
            )

    # A site can only compile inside a member that replaced the workspace
    # `forbid`, so a pin naming any other package records a permission that does
    # not exist. Checking it here keeps ADR 0079's crate half and site half from
    # drifting apart.
    permitted_dirs = {package_dirs[name] for name in diverging if name in package_dirs}
    for path, item in sorted(admitted):
        if not any(path.startswith(f"{directory}/") for directory in permitted_dirs):
            errors.append(
                f"unsafe-sites.{path}: `{item}` is pinned in a package that inherits the "
                'workspace `unsafe_code = "forbid"`, where no allow attribute can apply'
            )
    return errors


def rust_code_only(text: str) -> str:
    """Return `text` with comment and literal *contents* replaced by spaces.

    Offsets and line breaks are preserved, so a match found in the result can be
    sliced out of the original. Everything downstream — brace balancing, `fn`
    recognition, and the length-write search — runs over this rather than raw
    source, so a `#[cfg(test)]` quoted in prose, a `.len()` inside a doc comment,
    or a brace inside a string literal cannot steer the scan.

    Rust's lexical cases are handled explicitly: line comments, *nesting* block
    comments, byte and ordinary strings, raw strings at any hash count, and
    character literals. The last needs care because `'` also opens a lifetime:
    a tick is treated as a literal only when a complete one closes it, so
    `&'static str` is left as code rather than swallowing to the next quote.
    """
    result: list[str] = []
    index = 0
    length = len(text)

    def blanked(span: str) -> str:
        return "".join(" " if character != "\n" else "\n" for character in span)

    while index < length:
        character = text[index]
        if text.startswith("//", index):
            end = text.find("\n", index)
            end = length if end < 0 else end
            result.append(blanked(text[index:end]))
            index = end
        elif text.startswith("/*", index):
            depth = 0
            cursor = index
            while cursor < length:
                if text.startswith("/*", cursor):
                    depth += 1
                    cursor += 2
                elif text.startswith("*/", cursor):
                    depth -= 1
                    cursor += 2
                    if depth == 0:
                        break
                else:
                    cursor += 1
            result.append(blanked(text[index:cursor]))
            index = cursor
        elif (raw := RAW_STRING_OPENING.match(text, index)) is not None:
            terminator = '"' + raw.group(1)
            end = text.find(terminator, raw.end())
            end = length if end < 0 else end + len(terminator)
            result.append(blanked(text[index:end]))
            index = end
        elif (quoted := STRING_OPENING.match(text, index)) is not None:
            cursor = quoted.end()
            while cursor < length and text[cursor] != '"':
                cursor += 2 if text[cursor] == "\\" else 1
            end = min(cursor + 1, length)
            result.append(blanked(text[index:end]))
            index = end
        elif (literal := CHAR_LITERAL.match(text, index)) is not None:
            # Only a *complete* character literal blanks; a bare tick is a
            # lifetime, so `&'static str` stays code instead of swallowing
            # source to the next quote.
            result.append(blanked(literal.group()))
            index = literal.end()
        else:
            result.append(character)
            index += 1
    return "".join(result)


def balanced_end(code: str, start: int, opening: str, closing: str) -> int:
    """Return the index just past the group `code[start]` opens, or `len(code)`."""
    depth = 0
    cursor = start
    while cursor < len(code):
        if code[cursor] == opening:
            depth += 1
        elif code[cursor] == closing:
            depth -= 1
            if depth == 0:
                return cursor + 1
        cursor += 1
    return len(code)


def without_test_items(code: str) -> str:
    """Return `code` with every `#[cfg(test)]` item blanked out.

    The span is found by balancing the item's own braces rather than by cutting
    the file at its first `#[cfg(test)]` line. The cheaper cut is what the
    crate-local test this check replaces did, and it carries a bound that does
    not survive being generalized: production code placed *after* a test module
    in the same file would stop being scanned, and `tiler-compiler`'s
    `pipeline.rs` already holds two `#[cfg(test)]` modules with source between
    them.

    Test code is excluded rather than checked because two assertions depend on
    it. `tiler-ir`'s `shape/env.rs` and `tiler-compiler`'s `feasibility.rs` each
    assert an identity begins with its domain's length by spelling the eight-byte
    prefix out by hand, and that independence from the encoder is exactly what
    would catch the framing width changing — a test written with the encoder's
    own helper could not. They are skipped for that reason, not by accident.
    """
    result = code
    while (attribute := CFG_TEST_ATTRIBUTE.search(result)) is not None:
        cursor = attribute.end()
        while cursor < len(result) and result[cursor] not in "{;":
            cursor += 1
        end = (
            len(result)
            if cursor >= len(result)
            else (balanced_end(result, cursor, "{", "}") if result[cursor] == "{" else cursor + 1)
        )
        span = result[attribute.start() : end]
        result = (
            result[: attribute.start()]
            + "".join(" " if c != "\n" else "\n" for c in span)
            + result[end:]
        )
    return result


def module_directory(source: Path) -> Path:
    """Return the directory a module's `mod name;` declarations resolve against."""
    if source.stem in {"lib", "main", "mod"}:
        return source.parent
    return source.parent / source.stem


def test_only_sources(sources: list[Path]) -> tuple[set[Path], list[str]]:
    """Return every source reachable only under `#[cfg(test)]`, and any failure.

    A whole-file test module — `#[cfg(test)] mod tests;` beside `tests.rs` — is
    invisible from inside the file it names, which holds no `#[cfg(test)]` of its
    own. Nine such files exist across the workspace, so resolving the declaration
    is required rather than an edge case. A declaration whose file cannot be
    found stops the gate instead of leaving the file scanned as production.
    """
    excluded: set[Path] = set()
    errors: list[str] = []
    for source in sources:
        code = rust_code_only(source.read_text(encoding="utf-8"))
        for attribute in CFG_TEST_ATTRIBUTE.finditer(code):
            declaration = MODULE_DECLARATION.match(code, attribute.end())
            if declaration is None:
                continue
            directory = module_directory(source)
            name = declaration.group(1)
            single = directory / f"{name}.rs"
            nested = directory / name / "mod.rs"
            if single.is_file():
                excluded.add(single)
            elif nested.is_file():
                excluded.update(path for path in sources if path.is_relative_to(directory / name))
            else:
                errors.append(
                    f"length-framing.{source}: `#[cfg(test)] mod {name};` resolves to no file"
                )
    return excluded, errors


def statement_spans(code: str) -> list[tuple[int, int]]:
    """Return each top-level statement's `(start, end)` offsets in `code`.

    Statements rather than lines, because `rustfmt` wraps at 100 columns and one
    copy of the framing already spans four lines: `encode_explain_shape` in
    `tiler-compiler`'s `request.rs` reads `shape.rank()` on one line and writes
    `to_be_bytes()` on another, which a per-line search cannot see.

    A span ends at a `;` or a brace outside every paren and bracket. Only parens
    and brackets carry depth, so the `;` in an array type such as `[u8; 4]` does
    not end a span while a brace does — which keeps a span from running across
    unrelated items and joining one item's `.len()` to another's `to_be_bytes`.
    Every write this recognizer targets is an argument inside a call, so no
    framing statement is split by that rule.
    """
    spans: list[tuple[int, int]] = []
    depth = 0
    start = 0
    for index, character in enumerate(code):
        if character in "([":
            depth += 1
        elif character in ")]":
            depth -= 1
        elif character in ";{}" and depth == 0:
            spans.append((start, index + 1))
            start = index + 1
    if start < len(code):
        spans.append((start, len(code)))
    return spans


def framing_payload(parameters: str) -> bool:
    """Return whether a parameter list is a byte sink plus one framed payload."""
    depth = 0
    fields: list[str] = []
    current: list[str] = []
    for character in parameters:
        if character in "(<[":
            depth += 1
        elif character in ")>]":
            depth -= 1
        if character == "," and depth == 0:
            fields.append("".join(current))
            current = []
            continue
        current.append(character)
    fields.append("".join(current))
    typed = [field.split(":", 1)[1] for field in fields if ":" in field]
    if len(typed) != len(fields) or len(typed) != 2:
        return False
    sink = "".join(typed[0].split())
    payload = "".join(typed[1].split())
    return sink == FRAMING_SINK and payload in FRAMING_PAYLOADS


def item_documentation(lines: list[str], line_number: int) -> str:
    """Return the justification above the site at `line_number`, normalized.

    Doc comments and ordinary comments both count, because an admitted site is
    sometimes a statement rather than an item and a statement carries no doc
    comment. Attribute lines are stepped over so a site carrying `#[must_use]`
    beneath its documentation still finds it.
    """
    collected: list[str] = []
    cursor = line_number - 1
    while cursor >= 0:
        stripped = lines[cursor].strip()
        if stripped.startswith("//"):
            collected.append(stripped.removeprefix("//").removeprefix("/").strip())
        elif stripped.startswith("#[") or stripped.endswith(","):
            pass
        else:
            break
        cursor -= 1
    return " ".join(reversed(collected))


def frames_in_body(code: str, parameters_end: int) -> bool:
    """Whether the function body starting after `parameters_end` frames a length.

    The sink-plus-payload signature is necessary but not sufficient. It is also
    the shape of any ordinary serializer helper, and matching on it alone
    rejected a `fn f(bytes: &mut Vec<u8>, value: &[u8])` whose whole body was
    one `extend_from_slice` -- a function that frames nothing -- telling its
    author to use a framing primitive instead. Every copy this check exists to
    catch either converts a length to fixed-width bytes or reads one to hand to
    something that does, so the body must show one of those two halves.

    A declaration with no body (a trait method) frames nothing here; the
    implementation is what gets scanned.
    """
    body_start = code.find("{", parameters_end)
    terminator = code.find(";", parameters_end)
    if body_start < 0 or (terminator >= 0 and terminator < body_start):
        return False
    body = code[body_start : balanced_end(code, body_start, "{", "}")]
    return bool(FRAMING_LENGTH_SOURCE.search(body) or FRAMING_FIXED_WIDTH.search(body))


def scan_length_framing_sites(
    root: Path, package_dirs: dict[str, str]
) -> tuple[dict[tuple[str, str], str], list[str], dict[str, int]]:
    """Return every canonical length-framing site, failures, and the population.

    The population is returned rather than inferred so a caller can reject a scan
    that reached nothing. This check exists partly because a sweep for these
    copies returned six clean results after `zsh` expanded an unquoted
    `--include` glob: finding nothing is also what a broken search does, so a
    predicate whose only success signal is an empty result set has an unreachable
    failure path.

    Two recognizers run over production code:

    - a `fn` whose parameters are a `&mut Vec<u8>` sink and exactly one `usize`,
      `&[u8]`, or `&str` payload — the structural shape of every copy found so
      far, recognized without a name list because every name list has been
      incomplete; and
    - one statement that both reads a `.len()`/`.rank()` and writes it with
      `to_be_bytes`/`to_le_bytes` — the helper-less shape one `tiler-ir` copy
      took.

    Two bounds are stated rather than left to be discovered. A framing helper
    written as a method, or over a sink type other than `&mut Vec<u8>`, is not
    matched by the first recognizer. A length bound to a local by one statement
    and written by another is not matched by the second; `bundle.rs` writes its
    section-descriptor lengths that way, and they are classified in
    `FRAMING_SITE_CITATIONS` as container fields on the same evidence as the
    section count that *is* matched.
    """
    found: dict[tuple[str, str], str] = {}
    errors: list[str] = []
    population: dict[str, int] = {}
    for package, package_dir in sorted(package_dirs.items()):
        sources = sorted((root / package_dir / "src").rglob("*.rs"))
        excluded, resolution_errors = test_only_sources(sources)
        errors.extend(resolution_errors)
        production = [source for source in sources if source not in excluded]
        population[package] = len(production)
        for source in production:
            relative_path = source.relative_to(root).as_posix()
            text = source.read_text(encoding="utf-8")
            lines = text.splitlines()
            code = without_test_items(rust_code_only(text))
            for keyword in FRAMING_FN.finditer(code):
                opening = code.find("(", keyword.start())
                if opening < 0:
                    continue
                end = balanced_end(code, opening, "(", ")")
                if not framing_payload(code[opening + 1 : end - 1]):
                    continue
                if not frames_in_body(code, end):
                    continue
                line_number = code.count("\n", 0, keyword.start())
                start = code.rfind("\n", 0, keyword.start()) + 1
                subject = " ".join(text[start:end].split())
                key = (relative_path, subject)
                if key in found:
                    errors.append(f"length-framing.{relative_path}: `{subject}` is defined twice")
                    continue
                found[key] = item_documentation(lines, line_number)
            for start, end in statement_spans(code):
                statement = code[start:end]
                if not FRAMING_LENGTH_SOURCE.search(statement):
                    continue
                if not FRAMING_FIXED_WIDTH.search(statement):
                    continue
                # A span opens where the previous one closed, so it carries any
                # comment between them. Comments are already blanked in `code`,
                # so its first non-space column is the statement's real start and
                # the subject is sliced from the original text there — keeping
                # the literals a blanked slice would have lost.
                start += len(statement) - len(statement.lstrip())
                subject = " ".join(text[start:end].split())
                key = (relative_path, subject)
                if key not in found:
                    found[key] = item_documentation(lines, code.count("\n", 0, start))
    return found, errors, population


def validate_length_framing_pins(
    root: Path,
    *,
    package_dirs: dict[str, str] | None = None,
    admitted: dict[tuple[str, str], tuple[str, str]] | None = None,
) -> list[str]:
    """Return typed violations of the one-definition-per-permitted-crate rule."""
    package_dirs = PACKAGE_DIRS if package_dirs is None else package_dirs
    admitted = FRAMING_SITE_CITATIONS if admitted is None else admitted

    found, errors, population = scan_length_framing_sites(root, package_dirs)

    # The population half: a scan that read nothing reports no violations for the
    # same reason a clean tree does, so the counts are checked before the sites.
    if not population:
        errors.append(
            "length-framing.population: no package was scanned, so the sites below are "
            "a comparison against nothing"
        )
    for package in sorted(package for package, count in population.items() if count == 0):
        errors.append(
            f"length-framing.population: `{package}` contributed no production source; "
            "a package that reads as empty has not been checked"
        )
    if not admitted:
        errors.append(
            "length-framing.population: the admitted table is empty, so every site would "
            "be reported and none could be right"
        )

    for key in sorted(found.keys() - admitted.keys()):
        errors.append(
            f"length-framing.{key[0]}: `{key[1]}` frames a length and is not admitted. "
            "Use `tiler_ir::identity::{push_len, push_slice}` instead, or pin the site with "
            "the decision that forbids reaching them"
        )
    for key in sorted(admitted.keys() - found.keys()):
        errors.append(
            f"length-framing.{key[0]}: pinned site `{key[1]}` is gone; remove its pin in the "
            "same change that removes it"
        )
    for key in sorted(found.keys() & admitted.keys()):
        citation = admitted[key][1]
        if citation not in found[key]:
            errors.append(
                f"length-framing.{key[0]}: `{key[1]}` is admitted on the strength of "
                f"{citation!r}, which its own documentation no longer states"
            )
    return errors


def load_toml(path: Path) -> dict[str, Any]:
    """Load one UTF-8 TOML document."""
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, tomllib.TOMLDecodeError) as error:
        raise ValueError(f"cannot parse {path}: {error}") from error


def relative(root: Path, value: str | None) -> str | None:
    """Normalize a metadata path relative to the workspace."""
    if value is None:
        return None
    try:
        return Path(value).resolve().relative_to(root.resolve()).as_posix()
    except ValueError:
        return value


def normalize_dependency(root: Path, raw: dict[str, object]) -> dict[str, object]:
    """Select every output-affecting dependency field from Cargo metadata."""
    return {
        "name": raw.get("name"),
        "source": raw.get("source"),
        "req": raw.get("req"),
        "kind": raw.get("kind"),
        "rename": raw.get("rename"),
        "optional": raw.get("optional"),
        "uses_default_features": raw.get("uses_default_features"),
        "features": raw.get("features"),
        "target": raw.get("target"),
        "registry": raw.get("registry"),
        "path": relative(root, raw.get("path") if isinstance(raw.get("path"), str) else None),
    }


def normalize_target(root: Path, raw: dict[str, object]) -> dict[str, object]:
    """Select the governed target-role fields from Cargo metadata."""
    return {
        "name": raw.get("name"),
        "kind": raw.get("kind"),
        "crate_types": raw.get("crate_types"),
        "src_path": relative(
            root, raw.get("src_path") if isinstance(raw.get("src_path"), str) else None
        ),
        "edition": raw.get("edition"),
        "doc": raw.get("doc"),
        "doctest": raw.get("doctest"),
        "test": raw.get("test"),
    }


def expected_primary_target(name: str) -> dict[str, object]:
    """Return the one library or binary target a governed package must declare.

    Only this target is pinned exactly. Its `doc`, `doctest`, and `test` flags
    are the load-bearing part: switching any of them off silently removes a
    whole class of checking from a package that still looks governed.
    """
    package_dir = PACKAGE_DIRS[name]
    if name.startswith("tiler-prototype-"):
        return {
            "name": name,
            "kind": ["bin"],
            "crate_types": ["bin"],
            "src_path": f"{package_dir}/src/main.rs",
            "edition": "2024",
            "doc": True,
            "doctest": False,
            "test": True,
        }
    return {
        "name": name.replace("-", "_"),
        "kind": ["lib"],
        "crate_types": ["lib"],
        "src_path": f"{package_dir}/src/lib.rs",
        "edition": "2024",
        "doc": True,
        "doctest": True,
        "test": True,
    }


def validate_targets(name: str, actual: list[dict[str, object]] | None) -> list[str]:
    """Check one package's targets without pinning which tests exist.

    The set is deliberately open. Pinning it made adding an ordinary
    `tests/*.rs` to a crate an edit to this file, which taxes the writing of
    Tiler's own tests to catch nothing: a new integration test that Cargo
    discovers and runs is not a defect, and the wall-of-dict rejection it
    produced named no cause.

    What is checked is what a reviewer cannot see: the primary target exactly,
    and that no discovered integration test has been switched off. `test =
    false` on a `tests/*.rs` target leaves the file present, compiling, and
    never run -- which reads exactly like a passing suite.
    """
    if actual is None:
        return [f"package.{name}.targets: metadata declared no target list"]
    primary = expected_primary_target(name)
    matched = [target for target in actual if target["kind"] == primary["kind"]]
    if matched != [primary]:
        return [f"package.{name}.targets: expected primary {primary!r}, got {matched!r}"]
    errors = []
    for target in actual:
        if target["kind"] == primary["kind"]:
            continue
        if target["edition"] != "2024":
            errors.append(
                f"package.{name}.targets: {target['name']!r} declares edition "
                f"{target['edition']!r}, not the workspace edition"
            )
        if target["kind"] == ["test"] and target["test"] is not True:
            errors.append(
                f"package.{name}.targets: integration test {target['name']!r} sets "
                "test = false and is compiled but never run"
            )
    return errors


def validate_manifest_contract(root: Path, metadata: dict[str, object]) -> list[str]:
    """Return typed violations of the exact workspace contract."""
    errors: list[str] = []
    manifest = load_toml(root / "Cargo.toml")
    rustfmt = load_toml(root / "rustfmt.toml")
    workspace = manifest.get("workspace", {})
    if not isinstance(workspace, dict):
        return ["workspace.root: [workspace] is missing"]

    checks = (
        (workspace.get("members"), list(EXPECTED_MEMBERS), "workspace.members"),
        (workspace.get("exclude"), list(EXPECTED_EXCLUDES), "workspace.exclude"),
        (workspace.get("resolver"), "3", "workspace.resolver"),
        (workspace.get("package"), EXPECTED_WORKSPACE_PACKAGE, "workspace.package"),
        (
            workspace.get("dependencies"),
            EXPECTED_WORKSPACE_DEPENDENCIES,
            "workspace.dependencies",
        ),
        (
            workspace.get("lints", {}).get("rust")
            if isinstance(workspace.get("lints"), dict)
            else None,
            EXPECTED_RUST_LINTS,
            "workspace.lints.rust",
        ),
        (
            workspace.get("lints", {}).get("clippy")
            if isinstance(workspace.get("lints"), dict)
            else None,
            EXPECTED_CLIPPY_LINTS,
            "workspace.lints.clippy",
        ),
        (rustfmt, EXPECTED_RUSTFMT, "rustfmt.config"),
    )
    for actual, expected, label in checks:
        if actual != expected:
            errors.append(f"{label}: expected {expected!r}, got {actual!r}")

    if set(manifest) != {"workspace", "profile"}:
        errors.append(
            f"workspace.root-tables: expected ['profile', 'workspace'], got {sorted(manifest)}"
        )
    if set(workspace) != {
        "members",
        "exclude",
        "resolver",
        "package",
        "dependencies",
        "lints",
    }:
        errors.append(f"workspace.tables: unexpected keys {sorted(workspace)}")

    expected_profiles = {
        "dev": {
            "debug": "line-tables-only",
            "split-debuginfo": "unpacked",
            "package": {"*": {"opt-level": 1}},
        }
    }
    if manifest.get("profile") != expected_profiles:
        errors.append(f"workspace.profiles: unexpected contract {manifest.get('profile')!r}")

    packages_raw = metadata.get("packages")
    if not isinstance(packages_raw, list):
        return [*errors, "workspace.metadata: packages are missing"]
    packages = {
        package.get("name"): package for package in packages_raw if isinstance(package, dict)
    }
    if set(packages) != set(PACKAGE_DIRS):
        errors.append(
            f"workspace.packages: expected {sorted(PACKAGE_DIRS)}, got {sorted(packages)}"
        )

    for name, package_dir in PACKAGE_DIRS.items():
        package = packages.get(name)
        if not isinstance(package, dict):
            continue
        package_manifest = load_toml(root / package_dir / "Cargo.toml")
        expected_authored_manifest = expected_member_manifest(name)
        if package_manifest != expected_authored_manifest:
            errors.append(
                f"package.{name}.manifest: expected {expected_authored_manifest!r}, got "
                f"{package_manifest!r}"
            )
        expected_lints = UNINHERITED_LINT_MEMBERS.get(name, {"workspace": True})
        if package_manifest.get("lints") != expected_lints:
            errors.append(
                f"package.{name}.lints: expected {expected_lints!r}, got "
                f"{package_manifest.get('lints')!r}"
            )

        package_fields = {
            "version": package.get("version"),
            "edition": package.get("edition"),
            "license": package.get("license"),
            "repository": package.get("repository"),
            "publish": package.get("publish"),
            "rust_version": package.get("rust_version"),
            "features": package.get("features"),
            "links": package.get("links"),
            "default_run": package.get("default_run"),
        }
        expected_fields = {
            "version": "0.0.0",
            "edition": "2024",
            "license": "MIT OR Apache-2.0",
            "repository": "https://github.com/moderately-ai/tiler",
            "publish": [],
            "rust_version": None,
            "features": {},
            "links": None,
            "default_run": None,
        }
        if package_fields != expected_fields:
            errors.append(
                f"package.{name}.resolved-fields: expected {expected_fields!r}, got "
                f"{package_fields!r}"
            )

        raw_dependencies = package.get("dependencies")
        actual_dependencies = (
            sorted(
                (normalize_dependency(root, item) for item in raw_dependencies),
                key=lambda item: (str(item["name"]), str(item["kind"])),
            )
            if isinstance(raw_dependencies, list)
            else None
        )
        expected_dependencies = sorted(
            EXPECTED_DEPENDENCIES[name],
            key=lambda item: (str(item["name"]), str(item["kind"])),
        )
        if actual_dependencies != expected_dependencies:
            errors.append(
                f"package.{name}.dependencies: expected {expected_dependencies!r}, got "
                f"{actual_dependencies!r}"
            )

        raw_targets = package.get("targets")
        actual_targets = (
            sorted(
                (normalize_target(root, item) for item in raw_targets),
                key=lambda item: (str(item["kind"]), str(item["name"])),
            )
            if isinstance(raw_targets, list)
            else None
        )
        errors.extend(validate_targets(name, actual_targets))
    return errors


def cargo_metadata(root: Path, toolchain: str) -> dict[str, object]:
    """Read locked metadata through the exact selected toolchain."""
    result = subprocess.run(
        [
            "rustup",
            "run",
            toolchain,
            "cargo",
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--locked",
        ],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    )
    parsed = json.loads(result.stdout)
    if not isinstance(parsed, dict):
        raise ValueError("cargo metadata did not return an object")
    return parsed


def configured_toolchain(root: Path) -> str:
    """Return the repository's sole exact dated Rust toolchain pin."""
    toolchain = load_toml(root / "rust-toolchain.toml").get("toolchain")
    if not isinstance(toolchain, dict):
        raise ValueError("rust-toolchain.toml: [toolchain] is missing")
    if toolchain.get("profile") != "minimal":
        raise ValueError("rust-toolchain.toml: profile must be 'minimal'")
    # `rust-src` is load-bearing for the retained `trybuild` goldens rather than
    # for compilation: its presence decides whether a const-eval panic renders
    # the offending `core` line, so dropping it makes the byte-compared `.stderr`
    # files depend on the host instead of on the pin.
    if toolchain.get("components") != ["clippy", "rust-src", "rustfmt"]:
        raise ValueError(
            "rust-toolchain.toml: components must be ['clippy', 'rust-src', 'rustfmt']"
        )
    channel = toolchain.get("channel")
    if not isinstance(channel, str) or not channel.startswith("nightly-"):
        raise ValueError("rust-toolchain.toml: channel must be an exact dated nightly")
    date = channel.removeprefix("nightly-")
    try:
        year, month, day = (int(part) for part in date.split("-"))
    except ValueError as error:
        raise ValueError("rust-toolchain.toml: malformed dated nightly") from error
    if not (2020 <= year <= 9999 and 1 <= month <= 12 and 1 <= day <= 31):
        raise ValueError("rust-toolchain.toml: malformed dated nightly")
    return channel


def main() -> int:
    try:
        toolchain = configured_toolchain(ROOT)
        metadata = cargo_metadata(ROOT, toolchain)
        # The site pins read Rust source rather than manifests or resolved
        # metadata, so they are a separate phase composed here instead of a
        # clause of the manifest contract.
        errors = (
            validate_manifest_contract(ROOT, metadata)
            + validate_unsafe_site_pins(ROOT)
            + validate_length_framing_pins(ROOT)
        )
    except (OSError, ValueError, subprocess.CalledProcessError, json.JSONDecodeError) as error:
        print(f"workspace.validation: {error}", file=sys.stderr)
        return 1
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print("Rust workspace boundary passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
