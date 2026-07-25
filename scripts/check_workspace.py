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
) -> dict[str, object]:
    """Build one exact normalized Cargo metadata dependency contract."""
    return {
        "name": name,
        "source": source,
        "req": requirement,
        "kind": kind,
        "rename": None,
        "optional": False,
        "uses_default_features": True,
        "features": [],
        "target": None,
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
        # the one member with a Metal binding.
        dependency("metal", requirement="^0.33.0", source=CRATES_IO),
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

# `unsafe_code` as a whole token, so `unsafe_codegen` or a longer identifier
# does not match.
UNSAFE_CODE_TOKEN = re.compile(r"\bunsafe_code\b")
ALLOW_ATTRIBUTE_START = re.compile(r"^#!?\[allow\(")
ATTRIBUTE_START = re.compile(r"^#!?\[")
REASON_LITERAL = re.compile(r'\breason\s*=\s*"((?:[^"\\]|\\.)*)"')
STRING_LITERAL = re.compile(r'"(?:[^"\\]|\\.)*"')

EXPECTED_TESTS = {
    "tiler-ir": {
        "index_region": "crates/tiler-ir/tests/index_region.rs",
        "index_region_ui": "crates/tiler-ir/tests/index_region_ui.rs",
        "shape_evidence": "crates/tiler-ir/tests/shape_evidence.rs",
        "shape_evidence_ui": "crates/tiler-ir/tests/shape_evidence_ui.rs",
        "typed_handles": "crates/tiler-ir/tests/typed_handles.rs",
    },
    "tiler-reference": {
        "index_region_oracle": "crates/tiler-reference/tests/index_region_oracle.rs",
        "serial_sum_slice": "crates/tiler-reference/tests/serial_sum_slice.rs",
    },
}


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
        if item["kind"] is None
    }
    development = {
        item["name"]: {"workspace": True}
        for item in EXPECTED_DEPENDENCIES[name]
        if item["kind"] == "dev"
    }
    if normal:
        manifest["dependencies"] = normal
    if development:
        manifest["dev-dependencies"] = development
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


def expected_targets(name: str) -> list[dict[str, object]]:
    """Return the exact target set for one governed package."""
    package_dir = PACKAGE_DIRS[name]
    if name.startswith("tiler-prototype-"):
        targets = [
            {
                "name": name,
                "kind": ["bin"],
                "crate_types": ["bin"],
                "src_path": f"{package_dir}/src/main.rs",
                "edition": "2024",
                "doc": True,
                "doctest": False,
                "test": True,
            }
        ]
    else:
        targets = [
            {
                "name": name.replace("-", "_"),
                "kind": ["lib"],
                "crate_types": ["lib"],
                "src_path": f"{package_dir}/src/lib.rs",
                "edition": "2024",
                "doc": True,
                "doctest": True,
                "test": True,
            }
        ]
    for test_name, test_path in EXPECTED_TESTS.get(name, {}).items():
        targets.append(
            {
                "name": test_name,
                "kind": ["test"],
                "crate_types": ["bin"],
                "src_path": test_path,
                "edition": "2024",
                "doc": False,
                "doctest": False,
                "test": True,
            }
        )
    return sorted(targets, key=lambda target: (str(target["kind"]), str(target["name"])))


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
        target_contract = expected_targets(name)
        if actual_targets != target_contract:
            errors.append(
                f"package.{name}.targets: expected {target_contract!r}, got {actual_targets!r}"
            )
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
    if toolchain.get("components") != ["clippy", "rustfmt"]:
        raise ValueError("rust-toolchain.toml: components must be ['clippy', 'rustfmt']")
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
        errors = validate_manifest_contract(ROOT, metadata) + validate_unsafe_site_pins(ROOT)
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
