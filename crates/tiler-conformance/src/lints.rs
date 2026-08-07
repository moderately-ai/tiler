//! This crate's uninherited lint table, held against the workspace's.
//!
//! # Why this crate restates a table instead of inheriting it
//!
//! The workspace sets the unsafe-code lint to `forbid`, and the rustc
//! lint-level rule is that `forbid` cannot be relaxed by an inner attribute at
//! any scope — not on a module, not on a function, not on a block. The two
//! functions in `device_buffer` that must reach `MTLBuffer` storage need a
//! named per-site allow, so this crate drops `[lints] workspace = true` and
//! restates the whole table with `deny` in that one place. `Cargo.toml`'s own
//! note records Tom's decision of 2026-08-07 and the rule it sets.
//!
//! # The cost that decision did not name
//!
//! **A restated table drifts.** A lint added, tightened, or removed
//! workspace-wide reaches every other member through inheritance and does not
//! reach this one, and nothing failed when the two diverged.
//! [ADR 0079](../../../docs/decisions/0079-permit-unsafe-code-case-by-case-at-named-sites.md)
//! records the same gap and how it opened: `scripts/check_workspace.py` pinned
//! the diverging member's exact table until `e197176` deleted it with the rest
//! of the Python tooling, and since then this crate's `deny` could be widened
//! to `allow`, or a lint added or dropped from either side, with no check
//! failing. `AGENTS.md` asks a reviewer to inspect `[lints]` changes, which is
//! an obligation rather than an instrument.
//!
//! # What this module holds, and the larger property it does not
//!
//! It reads both manifests and compares them, so **this crate's** table
//! diverging from the workspace's by anything other than the one intended lint
//! level is a red test.
//!
//! It is not the workspace-wide pin that `UNINHERITED_LINT_MEMBERS` was, and
//! must not be read as one. Two properties stay unheld, both of them
//! workspace-scoped and neither reachable from inside one member's test:
//!
//! - **A third member dropping `[lints] workspace = true`** still goes
//!   unlinted with nothing failing. Two members diverge today — this crate and
//!   `prototypes/serial-sum-run` — and a member that stops inheriting is
//!   visible only in the diff that does it.
//! - **`prototypes/serial-sum-run`'s own table** has no check of any kind. Its
//!   divergence is the same shape as this one and its `deny` may still be
//!   widened to `allow` silently.
//!
//! Both belong to a check over the member set, which is
//! `crates/tiler/tests/workspace_population.rs`'s neighbourhood rather than
//! this crate's: what this crate owns is cross-layer executed evidence, and
//! policing every member's manifest from here would make it the place a test
//! goes when nobody decided where it belongs.
//!
//! # Derived rather than restated
//!
//! The expectation below is one line: the section-qualified name of the single
//! lint that may differ, and the two levels it may differ by. Every other lint
//! name and level is read out of the two files. A check that listed the
//! expected table would be a *third* copy of it, drifting alongside the second
//! and needing an edit every time the workspace gained a lint — which is the
//! failure this module exists to remove, reintroduced one layer up.
//!
//! # The parse is deliberately fail-closed
//!
//! A manifest table is a multi-line construct and its entries may be too, so
//! the scan accumulates physical lines into one logical entry until quotes and
//! braces balance rather than assuming an entry is a line. Anything it cannot
//! read — an entry with no `=`, one that never closes, one that runs past
//! [`MAX_ENTRY_LINES`], a duplicate key — panics naming the file and line
//! instead of being skipped, because a skipped entry is an invisible
//! difference. And the population is counted and floored, since a scan that
//! parsed nothing would otherwise compare two empty tables and report no drift.
//!
//! Two limits are worth stating rather than discovering. Only basic
//! (double-quoted) strings are tracked, so a level spelled as a TOML literal
//! string on one side alone is reported as a difference; and values are
//! compared as text with whitespace outside strings removed, so reordering an
//! inline table's keys on one side alone is reported as a difference too. Both
//! are false positives in the safe direction, and both name the exact entry.

use std::collections::BTreeMap;
use std::path::Path;

/// The smallest lint population either manifest may declare.
///
/// **A floor, one below the current population of five**, which is the level
/// that makes the smallest collapse fail rather than only a total one: the
/// tables hold `missing_docs` and the unsafe-code lint under `rust`, and
/// `all`, `pedantic`, and `too_many_lines` under `clippy`. Its real work is
/// against a scan that stopped recognising entries — two empty tables compare
/// equal, so a parse that found nothing would report no drift and pass.
///
/// Raising it with the population is the ordinary edit. Lowering it is a claim
/// that the workspace lint set genuinely shrank, and belongs in the same commit
/// as the removal.
const LINT_POPULATION_FLOOR: usize = 4;

/// The most physical lines one logical manifest entry may span.
///
/// A bound on how far a malformed entry can swallow the file. TOML puts an
/// inline table on one line, so anything wrapping this far is a multi-line
/// string or a construct this scan was not written for, and either is a
/// deliberate failure rather than a silent read.
const MAX_ENTRY_LINES: usize = 8;

/// One manifest's lint tables: section name, then lint name, then level text.
type LintTables = BTreeMap<String, BTreeMap<String, String>>;

/// The one difference between the two tables that is not drift.
///
/// The section-qualified lint name, the workspace's level, and this crate's,
/// each as the text a manifest spells it in. The name is assembled from two
/// pieces so that `crate::bf16_vertical::tests::the_unsafe_site_population_is_the_two_named_ones`,
/// which scans this crate's sources for the attribute token, cannot match this
/// file — the same reason `crate::portability` assembles its needles.
fn permitted_divergence() -> (String, String, String) {
    (
        format!("rust.{}{}", "unsafe", "_code"),
        "\"forbid\"".to_owned(),
        "\"deny\"".to_owned(),
    )
}

/// Both lint tables are read from their manifests rather than assumed.
///
/// Named and counted separately from the comparison because the two fail for
/// unrelated reasons: this one goes red when the scan stops finding lints, and
/// the comparison goes red when it finds them and they disagree. A scan that
/// silently found nothing would make the comparison pass vacuously, so it is
/// the precondition rather than a part of it.
#[test]
fn both_lint_tables_are_read_from_their_manifests_rather_than_assumed() {
    let (workspace, mine) = both_lint_tables();

    for (label, tables) in [("the workspace", &workspace), ("this crate", &mine)] {
        assert!(
            !tables.is_empty(),
            "the scan derived no lint section at all from {label}'s manifest. Either the table is \
             gone — which is a change to what this crate is held to and belongs in the diff that \
             makes it — or this parse has stopped recognising a section header, in which case the \
             comparison beside it is comparing two empty maps and cannot say no.",
        );
        for (section, entries) in tables {
            assert!(
                !entries.is_empty(),
                "{label}'s lint section `{section}` parsed as empty. An empty section is either a \
                 table someone left behind or an entry form this scan cannot read; both are \
                 differences the comparison would not see.",
            );
        }
    }

    let workspace_total: usize = workspace.values().map(BTreeMap::len).sum();
    let mine_total: usize = mine.values().map(BTreeMap::len).sum();
    eprintln!(
        "lint census: the workspace declares {workspace_total} lint(s) across {:?}; this crate \
         declares {mine_total} across {:?}",
        workspace.keys().collect::<Vec<_>>(),
        mine.keys().collect::<Vec<_>>(),
    );

    for (label, total) in [
        ("the workspace", workspace_total),
        ("this crate", mine_total),
    ] {
        assert!(
            total >= LINT_POPULATION_FLOOR,
            "{label} declares {total} lint(s) and the floor is {LINT_POPULATION_FLOOR}. A \
             population this small is either a real relaxation of what the workspace enforces or \
             a scan that has stopped reading entries, and both are worth stopping for.",
        );
    }
}

/// This crate's lint table differs from the workspace's by exactly one level.
#[test]
fn this_crates_lint_table_differs_from_the_workspace_by_exactly_one_level() {
    let (workspace, mine) = both_lint_tables();
    let workspace = flatten(&workspace);
    let mine = flatten(&mine);

    let mut differences = Vec::new();
    for (lint, level) in &workspace {
        match mine.get(lint) {
            None => differences.push(format!(
                "{lint}: the workspace declares {level} and this crate declares nothing"
            )),
            Some(ours) if ours != level => differences.push(format!(
                "{lint}: the workspace declares {level} and this crate declares {ours}"
            )),
            Some(_) => {}
        }
    }
    for (lint, level) in &mine {
        if !workspace.contains_key(lint) {
            differences.push(format!(
                "{lint}: this crate declares {level} and the workspace declares nothing"
            ));
        }
    }
    differences.sort();

    let (lint, workspace_level, our_level) = permitted_divergence();
    let intended = vec![format!(
        "{lint}: the workspace declares {workspace_level} and this crate declares {our_level}"
    )];

    assert_eq!(
        differences, intended,
        "this crate restates the workspace lint table because it cannot inherit one entry of it, \
         and the difference must stay that one entry. What the two manifests differ by is on the \
         left and what they may differ by is on the right.\n\nAn *extra* difference is drift: \
         inheritance carried a workspace lint change to every other member and not to this one. \
         Copy the change across rather than recording it here, and do not make the tables match \
         by weakening a lint — matching by weakening is the failure this check exists to \
         prevent.\n\nA *missing* difference means the exception itself moved. If this crate no \
         longer needs the relaxed level, the whole divergence goes: restore `[lints] workspace = \
         true`, delete the restated tables, and delete this module — the exception is not \
         something to keep alive by editing the expectation. If the level changed to something \
         else, that is Tom's decision under `decide-the-conformance-crate-s-unsafe-lint-level-\
         for-device-buffer-access`, and a crate-level allow is refused outright by it.",
    );
}

/// Reads the workspace's lint tables and this crate's, in that order.
///
/// The workspace root is found by walking up from this crate's manifest
/// directory and then checked against what it should contain, so a layout
/// change surfaces as a named failure rather than as a comparison against
/// whatever file that path happened to reach.
fn both_lint_tables() -> (LintTables, LintTables) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("this crate's manifest directory sits two levels below the workspace root");

    let root_manifest = root.join("Cargo.toml");
    let root_text = read(&root_manifest);
    assert!(
        root_text.contains("[workspace]"),
        "{} declares no workspace, so walking up two directories from this crate no longer \
         reaches the root manifest and every table read below would come from the wrong file",
        root_manifest.display(),
    );

    let member = manifest_dir
        .strip_prefix(root)
        .expect("this crate's directory lies under the workspace root")
        .to_string_lossy()
        .replace('\\', "/");
    assert!(
        root_text.contains(&format!("\"{member}\"")),
        "{} does not list \"{member}\" among its members, so the manifest this comparison treats \
         as this crate's governing workspace does not govern it",
        root_manifest.display(),
    );

    let crate_manifest = manifest_dir.join("Cargo.toml");
    let crate_text = read(&crate_manifest);

    (
        lint_tables(&root_text, "workspace.lints", &root_manifest),
        lint_tables(&crate_text, "lints", &crate_manifest),
    )
}

/// Reads one manifest, naming it if it cannot be read.
fn read(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{} is readable: {error}", path.display()))
}

/// Collects every lint table under one section prefix.
///
/// `prefix` is the section path the tables hang from — `workspace.lints` in the
/// root manifest and `lints` in a member's — and the returned keys are what
/// follows it, so both manifests yield the same section names for the same
/// lint tools.
fn lint_tables(manifest: &str, prefix: &str, path: &Path) -> LintTables {
    let lines: Vec<&str> = manifest.lines().collect();
    let qualified = format!("{prefix}.");
    let mut tables = LintTables::new();
    let mut section: Option<String> = None;
    let mut index = 0;

    while index < lines.len() {
        let trimmed = lines[index].trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            index += 1;
            continue;
        }

        if trimmed.starts_with('[') {
            let header = strip_comment(trimmed, &mut false);
            let header = header.trim();
            let name = header
                .strip_prefix('[')
                .and_then(|rest| rest.strip_suffix(']'))
                .unwrap_or_else(|| {
                    panic!(
                        "{}:{}: a table header this scan cannot read: {header}. A header it \
                         misreads is a table it silently drops.",
                        path.display(),
                        index + 1,
                    )
                });
            assert_ne!(
                name,
                prefix,
                "{}:{}: `[{prefix}]` declares the lint table as a whole, which this comparison is \
                 not written for — in a member it is how inheritance is spelled. If this crate \
                 inherits again the divergence is over and this module goes with it.",
                path.display(),
                index + 1,
            );
            section = name.strip_prefix(&qualified).map(str::to_owned);
            index += 1;
            continue;
        }

        let Some(name) = section.clone() else {
            index += 1;
            continue;
        };

        let start = index;
        let mut in_string = false;
        let mut entry = String::new();
        loop {
            assert!(
                index < lines.len(),
                "{}:{}: an entry that never closes its quotes or braces, so the scan reached the \
                 end of the file still reading it",
                path.display(),
                start + 1,
            );
            assert!(
                index - start < MAX_ENTRY_LINES,
                "{}:{}: an entry spanning more than {MAX_ENTRY_LINES} lines, which is past what \
                 this scan was written to read",
                path.display(),
                start + 1,
            );
            entry.push_str(&strip_comment(lines[index], &mut in_string));
            entry.push('\n');
            index += 1;
            if balanced(&entry) {
                break;
            }
        }

        let (lint, level) = entry.split_once('=').unwrap_or_else(|| {
            panic!(
                "{}:{}: a lint entry with no `=`, which this scan cannot read as a level: \
                 {}",
                path.display(),
                start + 1,
                entry.trim(),
            )
        });
        let lint = lint.trim().to_owned();
        let level = normalize(level);
        assert!(
            !lint.is_empty() && !level.is_empty(),
            "{}:{}: a lint entry with an empty name or level",
            path.display(),
            start + 1,
        );
        let replaced = tables.entry(name).or_default().insert(lint.clone(), level);
        assert!(
            replaced.is_none(),
            "{}:{}: `{lint}` is declared twice in one lint table, so one of the two levels is \
             invisible to this comparison",
            path.display(),
            start + 1,
        );
    }

    tables
}

/// Flattens the sections into section-qualified lint names.
fn flatten(tables: &LintTables) -> BTreeMap<String, String> {
    tables
        .iter()
        .flat_map(|(section, entries)| {
            entries
                .iter()
                .map(move |(lint, level)| (format!("{section}.{lint}"), level.clone()))
        })
        .collect()
}

/// Returns the code part of one physical line, advancing the string state.
///
/// The state is carried across the lines of one entry rather than reset per
/// line, which is what keeps a `#` inside a multi-line string from being read
/// as the start of a comment — and, in the common direction, keeps a trailing
/// comment that mentions a bracket from unbalancing the entry it follows.
fn strip_comment(line: &str, in_string: &mut bool) -> String {
    let mut code = String::new();
    for character in line.chars() {
        match character {
            '"' => *in_string = !*in_string,
            '#' if !*in_string => return code,
            _ => {}
        }
        code.push(character);
    }
    code
}

/// Whether an accumulated entry has closed every quote and bracket it opened.
fn balanced(entry: &str) -> bool {
    let mut in_string = false;
    let mut depth = 0_i32;
    for character in entry.chars() {
        if character == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        match character {
            '{' | '[' => depth += 1,
            '}' | ']' => depth -= 1,
            _ => {}
        }
    }
    !in_string && depth == 0
}

/// Removes whitespace outside strings so formatting is not read as drift.
fn normalize(level: &str) -> String {
    let mut normalized = String::new();
    let mut in_string = false;
    for character in level.chars() {
        if character == '"' {
            in_string = !in_string;
        } else if !in_string && character.is_whitespace() {
            continue;
        }
        normalized.push(character);
    }
    normalized
}
