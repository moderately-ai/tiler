//! Every unsafe-code permission in the workspace is one of the four admitted sites.
//!
//! ADR 0079 admits unsafe case by case. The compiler holds the first half of
//! that boundary: fourteen members inherit `unsafe_code = "forbid"`, and the
//! two members that must reach Metal buffer storage restate it as `deny`, so an
//! unsafe operation there still needs a local lint permission. This test holds
//! the other half: the exact `(workspace-relative path, item signature,
//! reason)` of every permission at a source-file-root function.
//!
//! # Why this is a workspace test
//!
//! The first replacement for the deleted Python gate lived in
//! `tiler-conformance` and counted two token substrings under that crate's
//! `src/`. It could not see `prototypes/serial-sum-run`, and a rename plus a
//! replacement inside `device_buffer.rs` preserved both counts. Workspace-wide
//! policy belongs beside `workspace_population.rs` and
//! `workspace_lint_inheritance.rs`, outside both members it enumerates.
//!
//! # Parsing boundary
//!
//! Cargo already exposes the actual workspace packages and target roots, so
//! this test invokes `cargo metadata --locked`. That subcommand resolves manifests but
//! never compiles or runs a target, so it does not recurse into this test. A
//! `Cargo.lock` read alone would not be source-truthful: it does not identify
//! workspace membership or target source paths. The explicit root-member list
//! and metadata package roots must agree exactly, closing Cargo's implicit
//! in-tree path-member rule, every metadata target root must remain inside its
//! owning package. Full dependency metadata also makes a direct proc-macro edge
//! from either reopenable package a clear early diagnostic; it is supporting
//! evidence, not closure, because transitive and re-exported procedural macros
//! need not appear on either direct edge list.
//!
//! The source side has two complementary authorities. A deliberately narrow
//! Rust lexer recognizes line comments, nested block comments, ordinary and
//! raw strings, character literals, Rust Unicode-XID identifiers, and balanced
//! delimiters.
//! Comments and strings are discarded before attributes are examined, so prose
//! and the live doc-comment fixture in `tiler-conformance/src/lib.rs` cannot
//! become sites. A permission is recognized only as a direct `#[allow(...)]`
//! whose comma-separated meta list contains the whole lint name and one
//! ordinary string-literal `reason`.
//! The following item must be a function, and its complete signature is read
//! through the top-level body brace, so wrapped attributes and wrapped
//! signatures are one site. The initial census includes every `.rs` file under
//! every actual package plus every metadata target root regardless of its
//! extension. Literal local `include!` and `#[path]` sources are resolved
//! canonically inside the governed package roots and visited once, so cycles
//! terminate and aliases cannot escape; a permission in either nonstandard
//! loading form is refused because one lexical file can be expanded into more
//! than one semantic site. Computed includes — including `OUT_DIR` generation
//! — imported `include!` aliases in any member, and `#[path]` in a
//! non-`mod.rs` module source whose rustc module-directory rules depend on
//! semantic context fail rather than disappearing.
//!
//! The lexer deliberately retains inactive-source coverage, which compilation
//! cannot supply. For active ordinary and test targets in the two packages that
//! use `unsafe_code = "deny"`, a second test runs Cargo in a deterministic
//! private target directory with rustc's `--force-warn=unsafe-code`, reads the
//! JSON diagnostics for active expanded source, and pins package, Cargo target,
//! target root, unsafe-operation source, and multiplicity. That compiler census
//! catches aliased includes and a source compiled both as a module and another
//! Cargo target. It is deliberately limited to the two `deny` packages.
//! Rustc suppresses unsafe-code diagnostics from external macro expansions even
//! under `--force-warn` and compiler builtins already generate internal unsafe
//! spellings. A separate all-member source-language boundary therefore closes
//! workspace-authored and source-generating authorities: all sixteen source
//! trees and extracted rustdoc Rust are inventoried for the exact private local
//! macro definitions (including multiplicity), the guarded `tensor!` exporter
//! and facade re-export, and
//! the exact facade-owned compiler diagnostic re-export, and the closed
//! compiler/std macro, attribute, and derive vocabularies. The 73
//! fixture invocations and one rustdoc invocation are pinned by exact source
//! identity and per-source multiplicity rather than only by their totals, and
//! Cargo metadata pins the facade dependency spelling to the workspace macro
//! producer and refuses facade dependency bindings named `core` or `std`, which
//! would retarget the diagnostic builtin at its definition site. Custom or
//! path-qualified invocations, workspace-owned local declarative macro exports
//! or re-exports, additional procedural exporters, aliases of the guarded
//! `tiler`, `tiler_macros`, `std`, or `core` macro namespaces, globs, and
//! dynamic macro or attribute names fail closed. Dependency internals and
//! compiler-generated builtin implementation details are not ADR 0079 sites.
//! Because rustdoc extracts documentation tests as separate crates and does not
//! inherit a package lint table, metadata's exact `doctest: true`
//! library/proc-macro population must carry
//! `#![doc(test(attr(forbid(unsafe_code))))]`; the scanner derives and floors
//! that population rather than hand-listing it, and scans line docs, block docs,
//! literal `#[doc]` strings, hidden `#` lines, and pinned macro-generated docs.
//! A literal metavariable may be forwarded only as the entire doc expression
//! from the exact arm that binds it; cooked Rust escapes are decoded before
//! Markdown classification. Raw-string arguments are opaque and therefore
//! refused, documentation nested under `cfg_attr` is unsupported, and
//! `stringify!($name)` requires an exact arm-local `ident`
//! binder and remains admissible only in prose: composition into executable
//! rustdoc is refused because invocation values are not reconstructed.
//! Unsupported doc sources and rustdoc-local source-loading forms fail closed.
//!
//! Everything outside that boundary fails closed. An inner attribute, a
//! `cfg_attr`, another attribute form that contains the lint token, a
//! non-literal reason, a permission within `macro_rules!` or another visible
//! token-generating invocation, a nested module/implementation/function site,
//! an unclosed comment/string/delimiter, or the lint token anywhere outside a
//! recognized direct allow is an error naming the file and line. ADR 0079
//! permits a function or module in principle, but the current population
//! contains source-file-root functions only; teaching this check a deeper semantic-path or
//! expansion identity belongs in the same reviewed change that admits one.
//!
//! The compiler census is one authority on active code in the two reopenable
//! packages. The all-member lexical and rustdoc inventories are the authority
//! on workspace-authored permissions, generators, and invocations, including
//! inactive source and exact reasons; the guarded procedural-macro producer
//! closes its emitted token streams, admitting only the exact facade diagnostic
//! invocation with one literal argument. Workspace lint checks independently keep
//! every ordinary member at `forbid` or `deny`; compiler-created builtin code
//! and dependency-internal expansions remain outside this source-authority
//! contract.

use std::collections::VecDeque;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

/// One exact permission admitted under ADR 0079.
#[derive(Clone, Copy, Debug)]
struct AdmittedSite {
    path: &'static str,
    item: &'static str,
    reason: &'static str,
}

/// The complete admitted population.
///
/// A count is not enough: one removed site plus one added site still counts
/// four. A path and item are not enough either: ADR 0079 makes the reason part
/// of the permission, so weakening it must be a pin change rather than an edit
/// no mechanical check asks anyone to inspect.
const ADMITTED_SITES: [AdmittedSite; 4] = [
    AdmittedSite {
        path: "crates/tiler-conformance/src/device_buffer.rs",
        item: "pub(crate) fn write_bytes(buffer: &Buffer, bytes: &[u8])",
        reason: concat!(
            "MTLBuffer storage is reachable only through the raw pointer `Buffer::contents` ",
            "returns; no Metal binding in this workspace exposes it safely, and a conformance ",
            "run must place its operands there. The write is bounded by an asserted length ",
            "check against the buffer's own byte length, copies bytes — a type with no ",
            "alignment requirement, no invalid bit pattern, and no destructor — and retains ",
            "no borrow."
        ),
    },
    AdmittedSite {
        path: "crates/tiler-conformance/src/device_buffer.rs",
        item: "pub(crate) fn read_bytes(buffer: &Buffer, len: usize) -> Vec<u8>",
        reason: concat!(
            "the read half of the same constraint: MTLBuffer storage is reachable only through ",
            "`Buffer::contents`, and a conformance run must observe what the device wrote. ",
            "Bounded by an asserted length check, reads bytes, and copies out rather than ",
            "retaining a borrow of device memory."
        ),
    },
    AdmittedSite {
        path: "prototypes/serial-sum-run/src/buffer.rs",
        item: "pub fn write_f32(buffer: &Buffer, values: &[f32])",
        reason: concat!(
            "MTLBuffer storage is reachable only through the raw pointer `Buffer::contents` ",
            "returns; no Metal binding exposes it safely. The write is bounded by an asserted ",
            "length check against the buffer's own byte length, copies a plain-old-data type ",
            "with no destructor, and retains no borrow."
        ),
    },
    AdmittedSite {
        path: "prototypes/serial-sum-run/src/buffer.rs",
        item: "pub fn read_f32(buffer: &Buffer, count: usize) -> Vec<f32>",
        reason: concat!(
            "the read half of the same constraint: MTLBuffer storage is reachable only through ",
            "`Buffer::contents`. Bounded by an asserted length check, reads a plain-old-data ",
            "type, and copies out rather than retaining a borrow of device memory."
        ),
    },
];

/// The smallest workspace source population this walk may report.
///
/// The audited base held 421 tracked Rust files and this test adds one. The
/// exact count is not a second member-population authority; requiring every
/// declared member to contribute a source file catches a missing small member,
/// while this floor catches a walk that lost a substantial subtree and would
/// otherwise report a smaller clean inventory.
const RUST_SOURCE_FILE_FLOOR: usize = 400;

/// The smallest member population the root-manifest reader may derive.
///
/// `workspace_population.rs` pins the exact current set of sixteen. This floor
/// only prevents the unsafe-site scan from becoming vacuous if its independent
/// narrow read of the `members` array stops early.
const MEMBER_POPULATION_FLOOR: usize = 12;

/// The smallest Cargo target population metadata may report.
///
/// The audited amendment base has 63 distinct target roots. The exact paths
/// are always enumerated and duplicate roots are refused; this floor catches a
/// truncated metadata read without becoming a second target manifest.
const TARGET_POPULATION_FLOOR: usize = 50;

/// The audited workspace has thirteen library or proc-macro roots whose
/// extracted documentation tests Cargo compiles as separate crates.
const DOCTEST_ROOT_FLOOR: usize = 12;

/// The one crate-root attribute that makes rustdoc's extracted test crates
/// preserve the workspace's closed unsafe-code boundary.
const DOCTEST_UNSAFE_SENTINEL: &str = "#![doc(test(attr(forbid(unsafe_code))))]";

/// Compiler and standard-library macros used by the current reopenable source
/// population. Every other function-like macro spelling is rejected there:
/// rustc deliberately suppresses unsafe-code lint diagnostics originating in
/// external macro expansions, so an open macro vocabulary would be an open
/// unsafe-code vocabulary too.
const REOPENABLE_BUILTIN_MACROS: [&str; 23] = [
    "assert",
    "assert_eq",
    "assert_ne",
    "cfg",
    "compile_error",
    "concat",
    "debug_assert",
    "debug_assert_eq",
    "env",
    "eprintln",
    "format",
    "include_str",
    "matches",
    "panic",
    "println",
    "stringify",
    "thread_local",
    "todo",
    "unreachable",
    "unimplemented",
    "vec",
    "write",
    "writeln",
];

/// Built-in attributes present in the current reopenable source population.
const REOPENABLE_BUILTIN_ATTRIBUTES: [&str; 19] = [
    "allow",
    "cfg",
    "cfg_attr",
    "cold",
    "default",
    "derive",
    "doc",
    "expect",
    "feature",
    "ignore",
    "must_use",
    "non_exhaustive",
    "path",
    "proc_macro",
    "repr",
    "rustfmt::skip",
    "should_panic",
    "test",
    "track_caller",
];

/// Compiler-built-in derives present in the reopenable source population.
const REOPENABLE_BUILTIN_DERIVES: [&str; 9] = [
    "Clone",
    "Copy",
    "Debug",
    "Default",
    "Eq",
    "Hash",
    "Ord",
    "PartialEq",
    "PartialOrd",
];

/// Namespace owners used by the admitted qualified macro and facade spellings.
/// A source alias with one of these bound names could preserve the token path
/// while changing which producer rustc resolves.
const GUARDED_MACRO_NAMESPACES: [&str; 4] = ["core", "std", "tiler", "tiler_macros"];

/// The exact private declarative-macro producer population in member source.
const WORKSPACE_LOCAL_MACRO_RULES: [(&str, &str); 15] = [
    (
        "crates/tiler-artifact/src/program/handles.rs",
        "draft_handle",
    ),
    (
        "crates/tiler-artifact/src/program/codec/model.rs",
        "received_subject",
    ),
    (
        "crates/tiler-artifact/src/program/codec/decode.rs",
        "tag_reader",
    ),
    ("crates/tiler-ir/src/kernel/handles.rs", "draft_handle"),
    ("crates/tiler-ir/src/kernel/handles.rs", "verified_handle"),
    ("crates/tiler-artifact/src/program/keys.rs", "governed_key"),
    (
        "crates/tiler-artifact/src/program/keys.rs",
        "opaque_identity",
    ),
    (
        "crates/tiler-artifact/src/proof/model.rs",
        "received_subject",
    ),
    (
        "crates/tiler-artifact/src/proof/codec.rs",
        "from_subject_bytes",
    ),
    ("crates/tiler-ir/src/index/handles.rs", "draft_handle"),
    ("crates/tiler-ir/src/index/handles.rs", "verified_handle"),
    ("crates/tiler-metal/src/emit.rs", "emit"),
    ("crates/tiler-ir/src/program/handles.rs", "draft_handle"),
    ("crates/tiler-compiler/src/explain.rs", "key_type"),
    (
        "crates/tiler-ir/src/semantic/accuracy/contract.rs",
        "spelled_rule",
    ),
];

/// Exact guarded `tensor!` invocation identities in ordinary and rustdoc source.
const TENSOR_FIXTURE_INVOCATION_COUNT: usize = 73;
const TENSOR_RUSTDOC_INVOCATION_COUNT: usize = 1;
const TENSOR_FIXTURE_INVOCATION_PINS: [(&str, usize); 13] = [
    (
        "crates/tiler/tests/facade/fail/contract_statement_diagnostics.rs",
        7,
    ),
    (
        "crates/tiler/tests/facade/fail/deliver_selects_an_artifact_family.rs",
        4,
    ),
    (
        "crates/tiler/tests/facade/fail/deliver_statement_diagnostics.rs",
        8,
    ),
    (
        "crates/tiler/tests/facade/fail/generated_operand_reference_spans.rs",
        1,
    ),
    (
        "crates/tiler/tests/facade/fail/reduction_diagnostics.rs",
        13,
    ),
    (
        "crates/tiler/tests/facade/fail/region_meaning_diagnostics.rs",
        7,
    ),
    (
        "crates/tiler/tests/facade/fail/region_syntax_diagnostics.rs",
        9,
    ),
    (
        "crates/tiler/tests/facade/pass/deliver_compiles_embeds_and_routes.rs",
        4,
    ),
    (
        "crates/tiler/tests/facade/pass/deliver_states_fallback_only.rs",
        3,
    ),
    (
        "crates/tiler/tests/facade/pass/inline_region_dispatches.rs",
        2,
    ),
    (
        "crates/tiler/tests/facade/pass/inline_region_executes.rs",
        11,
    ),
    (
        "crates/tiler/tests/facade/pass/inline_region_refuses_an_undispatchable_dtype.rs",
        1,
    ),
    (
        "crates/tiler/tests/facade/pass/reexport_and_generated_path.rs",
        3,
    ),
];
const TENSOR_RUSTDOC_INVOCATION_PINS: [(&str, usize); 1] =
    [("crates/tiler/src/lib.rs<rustdoc:1>", 1)];

/// One compiler-expanded unsafe operation identity and its expected
/// multiplicity across Cargo's ordinary and test compilations.
#[derive(Clone, Copy, Debug)]
struct ExpandedUnsafePin {
    package: &'static str,
    target: &'static str,
    target_source: &'static str,
    operation_source: &'static str,
    count: usize,
}

const EXPANDED_UNSAFE_PINS: [ExpandedUnsafePin; 2] = [
    ExpandedUnsafePin {
        package: "tiler-conformance",
        target: "tiler_conformance",
        target_source: "crates/tiler-conformance/src/lib.rs",
        operation_source: "crates/tiler-conformance/src/device_buffer.rs",
        count: 2,
    },
    ExpandedUnsafePin {
        package: "tiler-prototype-run",
        target: "tiler-prototype-run",
        target_source: "prototypes/serial-sum-run/src/main.rs",
        operation_source: "prototypes/serial-sum-run/src/buffer.rs",
        count: 4,
    },
];

/// One found site's exact item and reason, keyed by path and signature.
type Sites = BTreeMap<(String, String), String>;

/// Expanded unsafe operation counts keyed by package, target, target root, and
/// compiler-reported operation source.
type ExpandedOperations = BTreeMap<(String, String, String, String), usize>;

/// The output of one scan, including parsing failures.
#[derive(Debug, Default)]
struct Scan {
    sites: Sites,
    errors: Vec<String>,
    loads: Vec<SourceLoad>,
    builtin_macros: BTreeSet<String>,
    builtin_attributes: BTreeSet<String>,
    builtin_derives: BTreeSet<String>,
    local_macro_rules: BTreeSet<(String, String)>,
    proc_macro_exporters: BTreeSet<String>,
    facade_reexports: usize,
    facade_diagnostic_reexports: usize,
    tensor_invocations: BTreeMap<String, usize>,
    rustdoc_tensor_invocations: BTreeMap<String, usize>,
}

/// Cargo's actual governed source roots.
#[derive(Debug)]
struct WorkspacePopulation {
    member_roots: Vec<PathBuf>,
    target_roots: Vec<PathBuf>,
    target_count: usize,
    doctest_roots: Vec<PathBuf>,
    doctest_package_roots: Vec<PathBuf>,
    reopenable_packages: Vec<ReopenablePackage>,
}

/// One workspace package whose local lint table can reopen unsafe code only at
/// individually reasoned sites.
#[derive(Clone, Debug)]
struct ReopenablePackage {
    id: String,
    name: String,
    targets: BTreeSet<(String, String)>,
}

/// One literal compiler source-loading edge found in a source file.
#[derive(Clone, Debug, Eq, PartialEq)]
struct SourceLoad {
    kind: &'static str,
    literal: String,
    line: usize,
}

/// A lexed Rust token with the source location used by diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Token {
    kind: TokenKind,
    line: usize,
}

/// The token classes the admitted-attribute grammar needs.
#[derive(Clone, Debug, Eq, PartialEq)]
enum TokenKind {
    Ident(String),
    StringLiteral(String),
    Punct(String),
}

#[path = "workspace_unsafe_sites_support/metadata.rs"]
mod metadata;
pub(crate) use metadata::{
    metadata_array, metadata_string, relative_display, scan_files, workspace_population,
    workspace_root, workspace_sources,
};

#[path = "workspace_unsafe_sites_support/compiler_census.rs"]
mod compiler_census;
pub(crate) use compiler_census::expanded_unsafe_operations;

#[path = "workspace_unsafe_sites_support/macro_boundary.rs"]
mod macro_boundary;
pub(crate) use macro_boundary::{
    doc_attribute_markdown, rustdoc_rust_blocks, scan_rustdoc_code, workspace_macro_language,
};

#[path = "workspace_unsafe_sites_support/syntax.rs"]
mod syntax;
pub(crate) use syntax::{
    character_literal_end, ident, identifier_text, inside_span, is_open_delimiter,
    is_raw_identifier, lex, manifest_code, matching_delimiter, next_char, ordinary_string, punct,
    punct_text, quoted_values, raw_string_span, read, render_signature,
    root_doctest_sentinel_count, scan_text, token_generating_spans, validate_builtin_populations,
    validate_macro_authorities, validate_pins,
};

#[test]
fn the_workspace_unsafe_sites_are_exactly_the_four_admitted_ones() {
    let root = workspace_root();
    let population = workspace_population(&root)
        .unwrap_or_else(|error| panic!("unsafe-site source population failed:\n{error}"));
    let sources = workspace_sources(&population.member_roots, &population.target_roots);
    assert!(
        sources.len() >= RUST_SOURCE_FILE_FLOOR,
        "unsafe-site census: found {} Rust source file(s) across {} member(s), below the floor \
         of {RUST_SOURCE_FILE_FLOOR}; a shrunken walk cannot report a clean inventory",
        sources.len(),
        population.member_roots.len(),
    );

    let (mut scan, source_count) = scan_files(
        &root,
        &population.member_roots,
        &population.target_roots,
        &population.doctest_package_roots,
        &sources,
    );
    let population_errors = validate_builtin_populations(&scan);
    scan.errors.extend(population_errors);
    let authority_errors = validate_macro_authorities(&scan);
    scan.errors.extend(authority_errors);
    let tensor_invocations: usize = scan.tensor_invocations.values().sum();
    let rustdoc_tensor_invocations: usize = scan.rustdoc_tensor_invocations.values().sum();
    eprintln!(
        "unsafe-site census: {source_count} source file(s), {} Cargo target(s), {} doctest \
         root(s), {} package(s), {} fixture tensor invocation(s), and {} rustdoc tensor \
         invocation(s); \
         {} admitted site(s): {:?}",
        population.target_count,
        population.doctest_roots.len(),
        population.member_roots.len(),
        tensor_invocations,
        rustdoc_tensor_invocations,
        ADMITTED_SITES.len(),
        ADMITTED_SITES
            .iter()
            .map(|site| (site.path, site.item))
            .collect::<Vec<_>>(),
    );
    eprintln!(
        "guarded tensor invocation identities: fixtures {:?}; rustdoc {:?}",
        scan.tensor_invocations, scan.rustdoc_tensor_invocations,
    );
    let violations = validate_pins(scan, &ADMITTED_SITES);
    assert!(
        violations.is_empty(),
        "workspace unsafe-site inventory failed:\n{}",
        violations.join("\n"),
    );
}

#[test]
fn compiler_expansion_has_exactly_the_pinned_unsafe_operation_population() {
    let root = workspace_root();
    let population = workspace_population(&root)
        .unwrap_or_else(|error| panic!("expanded unsafe-site population failed:\n{error}"));
    let found = expanded_unsafe_operations(&root, &population)
        .unwrap_or_else(|error| panic!("expanded unsafe-site audit failed:\n{error}"));
    let expected: BTreeMap<_, _> = EXPANDED_UNSAFE_PINS
        .iter()
        .map(|pin| {
            (
                (
                    pin.package.to_owned(),
                    pin.target.to_owned(),
                    pin.target_source.to_owned(),
                    pin.operation_source.to_owned(),
                ),
                pin.count,
            )
        })
        .collect();
    eprintln!(
        "expanded unsafe-site census: {} compiler diagnostic(s) across {} target/source \
         identity pair(s), {} metadata target(s) reached, and {} reopenable package(s): \
         {found:?}",
        found.values().sum::<usize>(),
        found.len(),
        population
            .reopenable_packages
            .iter()
            .map(|package| package.targets.len())
            .sum::<usize>(),
        population.reopenable_packages.len(),
    );
    assert_eq!(
        found, expected,
        "compiler-expanded unsafe operation population changed; package, Cargo target, target \
         root, operation source, and compilation multiplicity are all pinned",
    );
}

#[test]
fn a_wrapped_attribute_and_signature_are_one_reached_site() {
    let source = concat!(
        "//! A doc comment naming `#[allow(unsafe_code)]` is prose.\n",
        "#[allow(\n",
        "    unsafe_code,\n",
        "    reason = \"wrapped reason\"\n",
        ")]\n",
        "#[must_use]\n",
        "pub fn write(\n",
        "    buffer: &Buffer,\n",
        "    values: &[f32],\n",
        ") -> Vec<f32> {\n",
        "    Vec::new()\n",
        "}\n",
    );
    let pin = [AdmittedSite {
        path: "crates/planted/src/lib.rs",
        item: "pub fn write(buffer: &Buffer, values: &[f32]) -> Vec<f32>",
        reason: "wrapped reason",
    }];

    let scan = scan_text("crates/planted/src/lib.rs", source);
    assert_eq!(scan.sites.len(), 1, "the wrapped attribute was not reached");
    let violations = validate_pins(scan, &pin);
    assert!(
        violations.is_empty(),
        "the wrapped attribute or signature did not normalize to its pin:\n{}",
        violations.join("\n"),
    );
}

#[test]
fn an_added_site_fails_until_it_is_pinned() {
    let source = format!(
        "{}{}",
        planted_site("write", "admitted reason"),
        planted_site("read", "second reason"),
    );
    let violations = planted_violations(&source, "admitted reason");
    assert!(
        violations.iter().any(|error| error
            .contains("`pub fn read(buffer: &Buffer)` admits unsafe_code and is not pinned")),
        "addition failure:\n{}",
        violations.join("\n"),
    );
}

#[test]
fn a_moved_site_fails_even_when_the_count_is_unchanged() {
    let source = planted_site("moved", "admitted reason");
    let violations = planted_violations(&source, "admitted reason");
    assert!(
        violations.iter().any(|error| error
            .contains("`pub fn moved(buffer: &Buffer)` admits unsafe_code and is not pinned")),
        "move addition half:\n{}",
        violations.join("\n"),
    );
    assert!(
        violations
            .iter()
            .any(|error| error.contains("pinned site `pub fn write(buffer: &Buffer)` is gone")),
        "move removal half:\n{}",
        violations.join("\n"),
    );
}

#[test]
fn a_removed_site_fails_until_its_pin_is_removed() {
    let violations = planted_violations("pub fn write(buffer: &Buffer) {}\n", "reason");
    assert!(
        violations
            .iter()
            .any(|error| error.contains("pinned site `pub fn write(buffer: &Buffer)` is gone")),
        "removal failure:\n{}",
        violations.join("\n"),
    );
}

#[test]
fn a_changed_reason_fails_although_the_path_and_item_are_unchanged() {
    let source = planted_site("write", "weakened");
    let violations = planted_violations(&source, "admitted reason");
    assert!(
        violations.iter().any(|error| {
            error.contains("states reason \"weakened\", pinned as \"admitted reason\"")
        }),
        "reason failure:\n{}",
        violations.join("\n"),
    );
}

#[test]
fn prose_and_strings_do_not_count_as_sites() {
    let source = concat!(
        "//! `#[allow(unsafe_code)]` in a doc comment.\n",
        "/* #[allow(unsafe_code)] in a nested /* block */ comment. */\n",
        "const PROSE: &str = \"#[allow(unsafe_code)]\";\n",
        "const RAW: &str = r#\"#[allow(unsafe_code)]\"#;\n",
    );
    let scan = scan_text("crates/planted/src/lib.rs", source);
    assert!(scan.errors.is_empty(), "{:?}", scan.errors);
    assert!(scan.sites.is_empty(), "{:?}", scan.sites);
}

#[test]
fn unsupported_permission_syntax_fails_closed() {
    let source = concat!(
        "#[cfg_attr(target_os = \"macos\", allow(unsafe_code))]\n",
        "pub fn write(buffer: &Buffer) {}\n",
    );
    let scan = scan_text("crates/planted/src/lib.rs", source);
    assert!(
        scan.errors
            .iter()
            .any(|error| error.contains("outside a supported direct `#[allow(...)]`")),
        "unsupported-syntax failure: {:?}",
        scan.errors,
    );
}

#[test]
fn the_same_pin_moved_into_a_module_is_refused() {
    let source = format!(
        "mod moved {{\n{}\n}}\n",
        planted_site("write", "admitted reason")
    );
    let scan = scan_text("crates/planted/src/lib.rs", &source);
    assert!(
        scan.errors
            .iter()
            .any(|error| error.contains("nested permission is outside the file-root pin boundary")),
        "nested-move failure: {:?}",
        scan.errors,
    );
    assert!(
        scan.sites.is_empty(),
        "a nested site was inventoried: {:?}",
        scan.sites
    );
}

#[test]
fn a_permission_in_a_macro_template_is_refused_before_expansion_multiplies_it() {
    let source = concat!(
        "macro_rules! emit_site {\n",
        "    ($module:ident) => {\n",
        "        mod $module {\n",
        "            #[allow(unsafe_code, reason = \"template reason\")]\n",
        "            pub fn write(buffer: &Buffer) {}\n",
        "        }\n",
        "    };\n",
        "}\n",
        "emit_site!(first);\n",
        "emit_site!(second);\n",
    );
    let scan = scan_text("crates/planted/src/lib.rs", source);
    assert!(
        scan.errors.iter().any(|error| error
            .contains("unsafe-code permission appears inside a token-generating macro context")),
        "macro-template failure: {:?}",
        scan.errors,
    );
    assert!(
        scan.sites.is_empty(),
        "a macro template became one lexical site"
    );
}

#[test]
fn literal_source_loads_are_reported_and_computed_includes_fail_closed() {
    let literal = scan_text(
        "crates/planted/src/lib.rs",
        "include!(\"hidden.inc\");\n#[path = \"module.inc\"]\nmod hidden;\n",
    );
    assert!(literal.errors.is_empty(), "{:?}", literal.errors);
    assert_eq!(
        literal.loads,
        [
            SourceLoad {
                kind: "include!",
                literal: "hidden.inc".to_owned(),
                line: 1,
            },
            SourceLoad {
                kind: "#[path]",
                literal: "module.inc".to_owned(),
                line: 2,
            },
        ],
    );

    let generated = scan_text(
        "crates/planted/src/lib.rs",
        "include!(concat!(env!(\"OUT_DIR\"), \"/generated.rs\"));\n",
    );
    assert!(
        generated.errors.iter().any(|error| error.contains(
            "computed include! is unsupported; generated or OUT_DIR sources cannot be inventoried"
        )),
        "computed-include failure: {:?}",
        generated.errors,
    );

    let nested_path = scan_text(
        "crates/planted/src/lib.rs",
        "mod nested {\n    #[path = \"hidden.inc\"]\n    mod hidden;\n}\n",
    );
    assert!(
        nested_path
            .errors
            .iter()
            .any(|error| error.contains("nested #[path] resolution is unsupported")),
        "nested-path failure: {:?}",
        nested_path.errors,
    );
}

#[test]
fn an_imported_include_alias_is_refused_in_a_reopenable_package() {
    let scan = scan_text(
        "crates/planted/src/lib.rs",
        "use std::include as imported_include;\nimported_include!(\"hidden.inc\");\n",
    );
    assert!(
        scan.errors.iter().any(|error| error
            .contains("the include macro name appears outside direct `include!(literal)` syntax")),
        "imported-include failure: {:?}",
        scan.errors,
    );
}

#[test]
fn the_workspace_macro_language_is_closed_over_classified_expansions() {
    let cases = [
        (
            "macro_rules! format { () => { 0 } }\n",
            "unpinned macro_rules! definition `format`",
        ),
        (
            "probe::emit!();\n",
            "path-qualified macro invocation `emit!` is unsupported",
        ),
        (
            "emit!();\n",
            "custom macro invocation `emit!` is unsupported",
        ),
        (
            "use tiler::tensor as ℘;\n℘! {}\n",
            "custom macro invocation `℘!` is unsupported",
        ),
        (
            "use probe::emit as format;\nformat!(\"probe\");\n",
            "use declaration binds classified macro, attribute, or derive name `format`",
        ),
        ("use probe::*;\n", "glob use is unsupported"),
        (
            "#[probe]\nfn item() {}\n",
            "custom attribute `probe` is unsupported",
        ),
        (
            "#[derive(Clone, probe::Emit)]\nstruct Item;\n",
            "custom or path-qualified derive `probe::Emit` is unsupported",
        ),
        (
            "#[cfg_attr(test, probe)]\nfn item() {}\n",
            "custom attribute `probe` is unsupported",
        ),
        (
            "macro probe() {}\n",
            "macro-2.0 definition `probe` is unsupported regardless of visibility",
        ),
        (
            "pub(crate) macro probe() {}\n",
            "macro-2.0 definition `probe` is unsupported regardless of visibility",
        ),
        (
            "#[proc_macro]\npub fn second(input: TokenStream) -> TokenStream { input }\n",
            "unguarded procedural-macro exporter `#[proc_macro]`",
        ),
        (
            "use evil::test;\n#[test]\nfn item() {}\n",
            "use declaration binds classified macro, attribute, or derive name `test`",
        ),
        (
            "use evil::Debug;\n#[derive(Debug)]\nstruct Item;\n",
            "use declaration binds classified macro, attribute, or derive name `Debug`",
        ),
        (
            "use evil::compile_error;\ncompile_error!(\"probe\");\n",
            "use declaration binds classified macro, attribute, or derive name `compile_error`",
        ),
        (
            "extern crate evil as tiler;\ntiler::tensor! {}\n",
            "extern crate declarations and aliases are unsupported",
        ),
        (
            "use evil as tiler;\ntiler::tensor! {}\n",
            "use declaration binds guarded macro namespace `tiler`",
        ),
        (
            "use evil as r#tiler;\ntiler::tensor! {}\n",
            "use declaration binds guarded macro namespace `tiler`",
        ),
        (
            "evil::tiler::__private::__tiler_compile_error!(\"probe\");\n",
            "path-qualified macro invocation `__tiler_compile_error!`",
        ),
        (
            "use evil as std;\nuse std::env;\nenv!(\"PROBE\");\n",
            "use declaration binds guarded macro namespace `std`",
        ),
        (
            "use evil as core;\n::core::compile_error!(\"probe\");\n",
            "use declaration binds guarded macro namespace `core`",
        ),
        (
            "use evil as tiler_macros;\npub use tiler_macros::tensor;\n",
            "use declaration binds guarded macro namespace `tiler_macros`",
        ),
        (
            "use elsewhere::draft_handle as hidden;\n",
            "use declaration imports or re-exports pinned local macro name `draft_handle`",
        ),
        (
            "#[macro_export]\nmacro_rules! emitted { () => {} }\n",
            "custom attribute `macro_export` is unsupported",
        ),
    ];
    for (source, expected) in cases {
        let scan = scan_text("crates/planted/src/lib.rs", source);
        assert!(
            scan.errors.iter().any(|error| error.contains(expected)),
            "macro-language failure `{expected}` was absent: {:?}",
            scan.errors,
        );
    }

    let duplicate_local = scan_text(
        "crates/tiler-ir/src/index/handles.rs",
        concat!(
            "macro_rules! draft_handle { () => {} }\n",
            "macro_rules! draft_handle { () => {} }\n",
        ),
    );
    assert!(
        duplicate_local.errors.iter().any(|error| error.contains(
            "duplicate pinned macro_rules! definition `draft_handle`; producer identity includes \
             exact multiplicity"
        )),
        "duplicate local producer failure: {:?}",
        duplicate_local.errors,
    );

    let builtins = scan_text(
        "crates/planted/src/lib.rs",
        concat!(
            "#[derive(Clone, Copy, Debug, Eq, PartialEq)]\n",
            "struct Item;\n",
            "#[cfg_attr(test, allow(dead_code, reason = \"test-only\"))]\n",
            "fn item() { assert_eq!(format!(\"{}\", 1), \"1\"); }\n",
        ),
    );
    assert!(
        builtins.errors.is_empty(),
        "classified builtins were refused: {:?}",
        builtins.errors,
    );
}

#[test]
fn dynamic_macro_and_attribute_names_are_refused_inside_a_pinned_template() {
    let source = concat!(
        "macro_rules! emit {\n",
        "    ($format:ident, $attr:ident) => {\n",
        "        $format!();\n",
        "        #$attr[item]\n",
        "    };\n",
        "}\n",
    );
    let scan = scan_text("crates/tiler-metal/src/emit.rs", source);
    assert!(
        scan.errors
            .iter()
            .any(|error| error.contains("dynamic macro invocation name `$format!`")),
        "dynamic macro-name failure: {:?}",
        scan.errors,
    );
    assert!(
        scan.errors
            .iter()
            .any(|error| error.contains("dynamic attribute name emission is unsupported")),
        "dynamic attribute-name failure: {:?}",
        scan.errors,
    );
}

#[test]
fn block_doc_rust_is_scanned_after_an_unmatched_comment_marker_in_a_string() {
    let source = concat!(
        "const MASK: &str = \"/*\";\n",
        "/**\n",
        "```rust\n",
        "evil::emit!();\n",
        "```\n",
        "*/\n",
        "pub fn item() {}\n",
    );
    let scan = scan_rustdoc_code("crates/planted/src/lib.rs", source);
    assert!(
        scan.errors
            .iter()
            .any(|error| error.contains("path-qualified macro invocation `emit!`")),
        "block-doc macro failure: {:?}",
        scan.errors,
    );
}

#[test]
fn unsupported_rustdoc_code_containers_and_forwarded_doc_sources_fail_closed() {
    for markdown in [
        "> ```rust\nevil::emit!();\n> ```\n",
        ">     evil::emit!();\n",
    ] {
        let (_, errors) = rustdoc_rust_blocks("crates/planted/src/lib.rs", markdown);
        assert!(
            errors.iter().any(
                |error| error.contains("rustdoc code in a blockquote container is unsupported")
            ),
            "blockquote rustdoc failure: {errors:?}",
        );
    }

    let (_, errors) = rustdoc_rust_blocks(
        "crates/planted/src/lib.rs",
        "- ```rust\n  evil::emit!();\n  ```\n",
    );
    assert!(
        errors
            .iter()
            .any(|error| error
                .contains("rustdoc fence marker in an unsupported container or position")),
        "list-container rustdoc failure: {errors:?}",
    );

    let (blocks, errors) = rustdoc_rust_blocks("crates/planted/src/lib.rs", "\tevil::emit!();\n");
    assert!(errors.is_empty(), "tab-indented rustdoc errors: {errors:?}");
    assert_eq!(blocks, ["evil::emit!();\n"]);

    let dynamic = doc_attribute_markdown(
        "crates/tiler-artifact/src/program/handles.rs",
        concat!(
            "macro_rules! draft_handle {\n",
            "    ($docs:tt) => { #[doc = $docs] pub struct Item; };\n",
            "}\n",
            "draft_handle!(include_str!(\"hidden.md\"));\n",
        ),
    );
    let errors = dynamic.expect_err("forwarded non-literal doc source must fail closed");
    assert!(
        errors
            .iter()
            .any(|error| error.contains("dynamic documentation source is unsupported")),
        "forwarded doc-source failure: {errors:?}",
    );

    let fragmented = doc_attribute_markdown(
        "crates/tiler-artifact/src/program/handles.rs",
        concat!(
            "macro_rules! draft_handle {\n",
            "    ($a:literal, $b:literal, $c:literal) => {\n",
            "        #[doc = concat!($a, $b, $c)] pub struct Item;\n",
            "    };\n",
            "}\n",
            "draft_handle!(\"`\", \"``rust\\nevil::emit!();\\n\", \"```\");\n",
        ),
    );
    let errors = fragmented.expect_err("forwarded doc composition must fail closed");
    assert!(
        errors
            .iter()
            .any(|error| error.contains("dynamic documentation source is unsupported")),
        "fragmented forwarded-doc failure: {errors:?}",
    );

    let raw_template_doc = doc_attribute_markdown(
        "crates/tiler-artifact/src/program/handles.rs",
        concat!(
            "macro_rules! draft_handle {\n",
            "    () => { #[doc = concat!(r#\"```rust\\nevil::emit!();\\n```\"#)] pub struct Item; };\n",
            "}\n",
            "draft_handle!();\n",
        ),
    );
    let errors = raw_template_doc.expect_err("raw template docs are opaque to the lexer");
    assert!(
        errors
            .iter()
            .any(|error| error.contains("dynamic documentation source is unsupported")),
        "raw template-doc failure: {errors:?}",
    );

    let cross_arm = doc_attribute_markdown(
        "crates/tiler-artifact/src/program/handles.rs",
        concat!(
            "macro_rules! draft_handle {\n",
            "    ($docs:literal) => {};\n",
            "    ($docs:expr) => { #[doc = $docs] pub struct Item; };\n",
            "}\n",
            "draft_handle!(include_str!(\"hidden.md\"));\n",
        ),
    );
    let errors = cross_arm.expect_err("another arm's literal binder proves nothing");
    assert!(
        errors
            .iter()
            .any(|error| error.contains("dynamic documentation source is unsupported")),
        "cross-arm forwarded-doc failure: {errors:?}",
    );

    let comma_cross_arm = doc_attribute_markdown(
        "crates/tiler-artifact/src/program/handles.rs",
        concat!(
            "macro_rules! draft_handle {\n",
            "    (($docs:literal)) => {},\n",
            "    ($docs:expr) => { #[doc = $docs] pub struct Item; },\n",
            "}\n",
            "draft_handle!(include_str!(\"hidden.md\"));\n",
        ),
    );
    let errors = comma_cross_arm.expect_err("comma-separated arms do not share binders");
    assert!(
        errors
            .iter()
            .any(|error| error.contains("dynamic documentation source is unsupported")),
        "comma-separated cross-arm forwarded-doc failure: {errors:?}",
    );

    let escaped_fence = scan_rustdoc_code(
        "crates/planted/src/lib.rs",
        r#"#[doc = "\x60\x60\x60rust\nevil::emit!();\n\x60\x60\x60"]
pub struct Item;
"#,
    );
    assert!(
        escaped_fence
            .errors
            .iter()
            .any(|error| error.contains("path-qualified macro invocation `emit!`")),
        "escaped cooked-string rustdoc failure: {:?}",
        escaped_fence.errors,
    );

    let stringified_macro_name = scan_rustdoc_code(
        "crates/tiler-ir/src/index/handles.rs",
        concat!(
            "macro_rules! define_handle {\n",
            "    ($name:tt) => {\n",
            "        #[doc = concat!(\"```rust\\nstd::\", stringify!($name), \"!();\\n```\")]\n",
            "        pub struct Item;\n",
            "    };\n",
            "}\n",
            "define_handle!(println);\n",
        ),
    );
    let errors = stringified_macro_name.errors;
    assert!(
        errors
            .iter()
            .any(|error| error.contains("dynamic documentation source is unsupported")),
        "non-ident stringify binder failure: {errors:?}",
    );

    let stringified_rustdoc_code = scan_rustdoc_code(
        "crates/tiler-ir/src/index/handles.rs",
        concat!(
            "macro_rules! define_handle {\n",
            "    ($name:ident) => {\n",
            "        #[doc = concat!(\"```rust\\nstd::\", stringify!($name), \
             \"!();\\n```\")]\n",
            "        pub struct Item;\n",
            "    };\n",
            "}\n",
            "define_handle!(println);\n",
        ),
    );
    assert!(
        stringified_rustdoc_code
            .errors
            .iter()
            .any(|error| error.contains(
                "stringify-composed rustdoc code is unsupported; stringified invocation values are \
             not reconstructed"
            )),
        "stringified rustdoc-code failure: {:?}",
        stringified_rustdoc_code.errors,
    );

    let raw_forwarded_doc = doc_attribute_markdown(
        "crates/tiler-artifact/src/program/handles.rs",
        concat!(
            "macro_rules! draft_handle {\n",
            "    ($docs:literal) => { #[doc = $docs] pub struct Item; };\n",
            "}\n",
            "draft_handle!(r#\"```rust\\nevil::emit!();\\n```\"#);\n",
        ),
    );
    let errors = raw_forwarded_doc.expect_err("raw forwarded docs are opaque to the lexer");
    assert!(
        errors.iter().any(|error| error.contains(
            "raw-string macro argument is unsupported for a pinned documentation-generating \
             macro"
        )),
        "raw forwarded-doc failure: {errors:?}",
    );

    let nested_cfg_attr_doc = scan_rustdoc_code(
        "crates/planted/src/lib.rs",
        "#[cfg_attr(doc, doc = \"```rust\\nevil::emit!();\\n```\")]\npub struct Item;\n",
    );
    assert!(
        nested_cfg_attr_doc
            .errors
            .iter()
            .any(|error| error.contains(
                "documentation nested in cfg_attr is unsupported; rustdoc input must use a directly \
                 enumerable doc attribute"
            )),
        "cfg_attr-generated rustdoc failure: {:?}",
        nested_cfg_attr_doc.errors,
    );

    let recursively_nested_cfg_attr_doc = scan_rustdoc_code(
        "crates/planted/src/lib.rs",
        concat!(
            "#[cfg_attr(doc, cfg_attr(doc, doc = \
             \"```rust\\nevil::emit!();\\n```\"))]\n",
            "pub struct Item;\n",
        ),
    );
    assert!(
        recursively_nested_cfg_attr_doc
            .errors
            .iter()
            .any(|error| error.contains("documentation nested in cfg_attr is unsupported")),
        "recursively nested cfg_attr rustdoc failure: {:?}",
        recursively_nested_cfg_attr_doc.errors,
    );
}

#[test]
fn rustdoc_source_loading_forms_fail_closed() {
    let source = concat!(
        "//! ```rust\n",
        "//! include!(\"hidden.rs\");\n",
        "//! #[path = \"hidden.rs\"] mod hidden_path;\n",
        "//! mod hidden_module;\n",
        "//! ```\n",
    );
    let scan = scan_rustdoc_code("crates/planted/src/lib.rs", source);
    for expected in [
        "include! is unsupported in an extracted doctest",
        "#[path] is unsupported in an extracted doctest",
        "out-of-line module load is unsupported in an extracted doctest",
    ] {
        assert!(
            scan.errors.iter().any(|error| error.contains(expected)),
            "rustdoc source-load failure `{expected}` was absent: {:?}",
            scan.errors,
        );
    }
}

/// One synthetic direct permission.
fn planted_site(item: &str, reason: &str) -> String {
    format!(
        "#[allow(\n    unsafe_code,\n    reason = \"{reason}\"\n)]\npub fn {item}(buffer: \
         &Buffer) {{}}\n"
    )
}

/// Violations for a synthetic one-site population.
fn planted_violations(source: &str, reason: &'static str) -> Vec<String> {
    let path = "crates/planted/src/lib.rs";
    let pin = [AdmittedSite {
        path,
        item: "pub fn write(buffer: &Buffer)",
        reason,
    }];
    validate_pins(scan_text(path, source), &pin)
}
