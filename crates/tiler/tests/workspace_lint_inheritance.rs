//! Every workspace member inherits the workspace lint table, or is one of the
//! members declared here and has a check of its own.
//!
//! `[workspace.lints]` reaches a member only through `[lints] workspace = true`
//! in that member's manifest, so dropping those two lines silently removes the
//! crate from `missing_docs`, from `unsafe_code = "forbid"`, and from every
//! Clippy group the workspace turns on. Nothing about the resulting build looks
//! different: it compiles, it tests, and it passes the complete gate.
//!
//! `scripts/check_workspace.py` used to hold this. Its `UNINHERITED_LINT_MEMBERS`
//! named the one member permitted to diverge and the exact table it could
//! carry, so a second member dropping inheritance failed the gate.
//! [ADR 0079](../../../docs/decisions/0079-permit-unsafe-code-case-by-case-at-named-sites.md)
//! records that `e197176` deleted that script with the rest of the Python
//! tooling and that both halves went with it, leaving `AGENTS.md`'s request to
//! inspect `[lints]` changes — an obligation rather than an instrument. The
//! constant below carries the member half of that name back deliberately.
//!
//! # The two halves, and why only one of them is here
//!
//! *Which* members may diverge is a property of the member set, and it is what
//! this file holds. *What* a diverging member's restated table may say is a
//! property of that member's two manifests, and it is held inside each member
//! by `crates/tiler-conformance/src/lints.rs` — once for the crate that file
//! lives in, and once for `prototypes/serial-sum-run`, whose
//! `tests/lint_table.rs` is a single `#[path]` line running that same module
//! from its own root. One reader, two roots, no second parser.
//!
//! The third test below is the seam between the halves: a member may not be
//! declared here without naming a check that holds its table, and that check
//! must live inside the member and must reach the shared reader. Without it the
//! member half and the table half drift apart, and adding a name to the list
//! becomes enough to exempt a crate from both.
//!
//! # Why the facade crate hosts it
//!
//! Not because the facade has anything to do with lints. `crates/tiler/tests/`
//! is already where a check over the workspace as a whole goes:
//! `workspace_population.rs` holds the member set to a declared list and
//! `dependency_direction.rs` hand-parses `Cargo.lock`, and
//! `labelled_diagnostic.rs` reads a prototype's source across the same frontier
//! this file reads manifests across. The facade is the top of the consumer
//! graph and nothing may depend on it, so a test here can observe every member
//! without an edge that would invert the direction `dependency_direction.rs`
//! exists to keep.
//!
//! `tiler-conformance` was the other candidate and is the wrong one twice over.
//! Its own header says what it owns is cross-layer *executed* evidence — a
//! program built, lowered, compiled, run on a device, and compared against the
//! oracle — and a manifest policy executes nothing and crosses no layer. More
//! than that, it is **one of the two members this file polices**. A census that
//! lives inside a member of the population it enumerates has to describe its
//! own exception, which is the asymmetry that stopped `lints.rs` reaching this
//! property in the first place. `crates/tiler` inherits the workspace table
//! like every other non-exception member, so it observes the partition from
//! outside both sides of it.
//!
//! # The parse, and what it refuses
//!
//! Manifests are read as text because `cargo metadata` does not emit lint
//! tables at all — `cargo metadata --no-deps --format-version 1` at this commit
//! contains no occurrence of `lints`, `missing_docs`, `unsafe_code`,
//! `pedantic`, or `too_many_lines`. `workspace_population.rs` can use it for
//! package names; nothing here can.
//!
//! Only two questions are asked of each manifest, so the grammar needed is far
//! narrower than the lint-table reader's: what the `[workspace] members` array
//! lists, and whether a member declares `[lints]` with `workspace = true` under
//! it. Anything else — a member whose manifest is missing, a `members` array
//! that does not close, a `[lints]` table saying something other than
//! `workspace = true` — is a failure naming the file rather than a skip, and a
//! member population below [`MEMBER_POPULATION_FLOOR`] fails on its own so a
//! scan that stopped finding members cannot report a clean partition of
//! nothing.
//!
//! One spelling is deliberately unrecognized: the dotted `lints.workspace =
//! true` form is legal TOML and no member uses it. A member that adopted it
//! would be reported here as diverging, which is the fail-closed direction —
//! loud and wrong rather than quiet and wrong.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The members permitted not to inherit `[workspace.lints]`, each with the
/// check that holds its restated table to the workspace's.
///
/// Adding an entry here is Tom's decision, not a maintenance edit: ADR 0079
/// item 4 reserves "a second member dropping lint inheritance" to him, and both
/// entries below were taken as decisions rather than as edits —
/// `prototypes/serial-sum-run` at `43f685f` on 2026-07-25, and
/// `crates/tiler-conformance` on 2026-08-07 under
/// `decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access`.
///
/// The second element is not decoration. A member named here is exempt from the
/// inheritance check, so it must be covered by a table check instead, and the
/// third test below is what holds that.
const UNINHERITED_LINT_MEMBERS: [(&str, &str); 2] = [
    (
        "crates/tiler-conformance",
        "crates/tiler-conformance/src/lints.rs",
    ),
    (
        "prototypes/serial-sum-run",
        "prototypes/serial-sum-run/tests/lint_table.rs",
    ),
];

/// The one manifest reader every declared exception's table check runs.
///
/// Named here so that a second exception cannot be admitted with a table check
/// that reimplements the parse. Two readers of the same two files disagree
/// eventually, and the disagreement is invisible until the manifests are
/// already apart.
const SHARED_LINT_TABLE_READER: &str = "crates/tiler-conformance/src/lints.rs";

/// The smallest member population this scan may derive from the root manifest.
///
/// **A floor, four below the current population of sixteen.**
/// `workspace_population.rs` is the authority for the exact count and pins it
/// against `cargo metadata`; restating sixteen here would be a third copy of
/// one number. What this floor is for is different: the partition below is
/// vacuously clean over an empty member set, so a `members` array this scan
/// stopped reading would report no drift and pass.
const MEMBER_POPULATION_FLOOR: usize = 12;

/// How a member's manifest answers the inheritance question.
#[derive(Debug)]
enum Inheritance {
    /// `[lints]` declares `workspace = true`.
    Inherits,
    /// It does not, with what the manifest says instead.
    Diverges(String),
}

/// The member set is read from the root manifest and is not empty.
///
/// Separate from the partition for the reason `lints.rs` keeps its census
/// separate from its comparison: the two fail for unrelated reasons, and a scan
/// that silently found nothing would make the partition pass vacuously rather
/// than fail.
#[test]
fn the_member_set_is_read_from_the_root_manifest_and_is_not_empty() {
    let root = workspace_root();
    let members = members(&root);

    eprintln!(
        "member census: the root manifest lists {} member(s): {members:?}",
        members.len(),
    );

    assert!(
        members.len() >= MEMBER_POPULATION_FLOOR,
        "the root manifest lists {} member(s) and the floor is {MEMBER_POPULATION_FLOOR}. Either \
         the workspace genuinely shrank this far — which belongs in the same commit as the \
         removals, beside `workspace_population.rs` — or this scan has stopped reading the \
         `members` array, in which case the partition beside it is partitioning nothing and \
         cannot say no.",
        members.len(),
    );

    for member in &members {
        let manifest = root.join(member).join("Cargo.toml");
        assert!(
            manifest.is_file(),
            "the root manifest lists `{member}` as a member but {} is not a file, so this scan \
             cannot read what that member declares and would otherwise leave it out of the \
             partition entirely",
            manifest.display(),
        );
    }
}

/// Every member inherits the workspace lint table except the declared ones.
#[test]
fn every_member_inherits_the_workspace_lint_table_except_the_declared_exceptions() {
    let root = workspace_root();
    let members = members(&root);

    let mut inheriting = Vec::new();
    let mut diverging = Vec::new();
    for member in &members {
        let manifest = root.join(member).join("Cargo.toml");
        match inheritance(&read(&manifest), &manifest) {
            Inheritance::Inherits => inheriting.push(member.clone()),
            Inheritance::Diverges(reason) => diverging.push((member.clone(), reason)),
        }
    }

    eprintln!(
        "lint inheritance census: {} of {} member(s) inherit; {} diverge: {diverging:?}",
        inheriting.len(),
        members.len(),
        diverging.len(),
    );

    let declared: BTreeSet<&str> = UNINHERITED_LINT_MEMBERS
        .iter()
        .map(|(member, _)| *member)
        .collect();
    assert_eq!(
        declared.len(),
        UNINHERITED_LINT_MEMBERS.len(),
        "the declared exception list names one member twice",
    );

    let actual: BTreeSet<&str> = diverging
        .iter()
        .map(|(member, _)| member.as_str())
        .collect();

    let undeclared: Vec<&(String, String)> = diverging
        .iter()
        .filter(|(member, _)| !declared.contains(member.as_str()))
        .collect();
    let absent: Vec<&&str> = declared
        .iter()
        .filter(|member| !actual.contains(**member))
        .collect();

    assert!(
        undeclared.is_empty(),
        "member(s) that do not inherit `[workspace.lints]` and are not a declared exception: \
         {undeclared:?}.\n\nA member that stops inheriting stops being linted — no \
         `missing_docs`, no `unsafe_code = \"forbid\"`, no Clippy group — and nothing about the \
         build looks different. ADR 0079 item 4 reserves a member dropping lint inheritance to \
         Tom, so restore `[lints] workspace = true` rather than adding a name here. If he did \
         decide it, the name goes in `UNINHERITED_LINT_MEMBERS` together with a check over that \
         member's restated table, which is what the test below requires.",
    );
    assert!(
        absent.is_empty(),
        "declared exception(s) that now inherit `[workspace.lints]` after all: {absent:?}.\n\nThe \
         exception is over, so it goes rather than being kept alive: delete the entry here and \
         the member's table check with it. An exception nobody exercises is one nobody rereads.",
    );
}

/// Each declared exception carries a table check, inside it, on the one reader.
///
/// The member half above exempts a named member from inheritance; without this
/// the exemption would be the whole of what naming it costs, and a member could
/// be excused from the workspace table without anything holding its restated
/// one. ADR 0079 argues for exactly this seam — that a pinned path must lie
/// inside the member permitted to diverge — as what keeps the two halves from
/// drifting apart.
#[test]
fn each_declared_exception_has_a_table_check_inside_it_on_the_shared_reader() {
    let root = workspace_root();

    let reader = root.join(SHARED_LINT_TABLE_READER);
    assert!(
        reader.is_file(),
        "the shared manifest reader {} is gone, so every table check named below is either \
         broken or has quietly grown a parse of its own",
        reader.display(),
    );

    for (member, check) in UNINHERITED_LINT_MEMBERS {
        assert!(
            check.starts_with(&format!("{member}/")),
            "`{check}` is named as `{member}`'s table check but does not lie inside it. A check \
             outside the member it governs is one that can be deleted with neither the member nor \
             its divergence changing.",
        );

        let path = root.join(check);
        assert!(
            path.is_file(),
            "`{member}` is declared exempt from lint inheritance and its table check {} is not a \
             file, so nothing holds its restated table to the workspace's at all",
            path.display(),
        );

        // The reader lives inside one of the two members it serves, so for that
        // member it *is* the check and cannot be asked to name itself.
        if check == SHARED_LINT_TABLE_READER {
            continue;
        }

        let text = read(&path);
        assert!(
            text.contains(SHARED_LINT_TABLE_READER),
            "`{check}` does not name `{SHARED_LINT_TABLE_READER}`, so `{member}`'s table is \
             either unchecked or checked by a second reader of the same two files. A second \
             reader drifts against the first exactly as the two manifests drift without either — \
             the same failure one layer up. Run the shared module from this member's root with a \
             `#[path]` declaration, as `prototypes/serial-sum-run/tests/lint_table.rs` does.",
        );
    }
}

/// The workspace root, two levels above this crate's manifest directory.
fn workspace_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("this crate's manifest directory sits two levels below the workspace root")
        .to_path_buf();
    let manifest = root.join("Cargo.toml");
    assert!(
        read(&manifest).contains("[workspace]"),
        "{} declares no workspace, so walking up two directories from this crate no longer \
         reaches the root manifest and every member read below would come from the wrong file",
        manifest.display(),
    );
    root
}

/// Reads one manifest, naming it if it cannot be read.
fn read(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{} is readable: {error}", path.display()))
}

/// The member paths the root manifest's `[workspace] members` array lists.
///
/// The array is accumulated until it closes rather than read a line at a time,
/// so a reflow to one line or to several reads the same. `exclude` sits in the
/// same table and is not this array; the key is matched exactly.
fn members(root: &Path) -> Vec<String> {
    let manifest = root.join("Cargo.toml");
    let text = read(&manifest);
    let lines: Vec<&str> = text.lines().collect();

    let mut in_workspace = false;
    for (index, line) in lines.iter().enumerate() {
        let trimmed = code(line).trim().to_owned();
        if trimmed.starts_with('[') {
            in_workspace = trimmed == "[workspace]";
            continue;
        }
        if !in_workspace {
            continue;
        }
        let Some(value) = trimmed.strip_prefix("members") else {
            continue;
        };
        let Some(value) = value.trim_start().strip_prefix('=') else {
            continue;
        };

        let mut array = value.to_owned();
        let mut cursor = index;
        while !array.contains(']') {
            cursor += 1;
            assert!(
                cursor < lines.len(),
                "{}:{}: the `members` array never closes, so the member set this scan derived is \
                 whatever it had read when the file ran out",
                manifest.display(),
                index + 1,
            );
            array.push('\n');
            array.push_str(&code(lines[cursor]));
        }
        return quoted(&array);
    }

    panic!(
        "{} has no `members` key under `[workspace]`, so this scan derived no member set and \
         every partition over it would be vacuously clean",
        manifest.display(),
    );
}

/// How one member's manifest answers the inheritance question.
///
/// `[lints]` is the table inheritance is spelled in, and the only body admitted
/// under it is `workspace = true`. `[lints.rust]` and `[lints.clippy]` are
/// different headers and are not it — declaring them is exactly what a
/// diverging member does.
fn inheritance(manifest: &str, path: &Path) -> Inheritance {
    let mut body: Option<Vec<String>> = None;
    let mut header = 0;
    for (index, line) in manifest.lines().enumerate() {
        let trimmed = code(line).trim().to_owned();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('[') {
            if trimmed == "[lints]" {
                assert!(
                    body.is_none(),
                    "{}:{}: `[lints]` is declared twice, so one of the two bodies is invisible to \
                     this scan",
                    path.display(),
                    index + 1,
                );
                body = Some(Vec::new());
                header = index + 1;
            } else if body.is_some() {
                break;
            }
            continue;
        }
        if let Some(entries) = body.as_mut() {
            entries.push(trimmed);
        }
    }

    let Some(entries) = body else {
        return Inheritance::Diverges("declares no `[lints]` table".to_owned());
    };

    let joined = entries.join("; ");
    let normalized: String = joined.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        !normalized.is_empty(),
        "{}:{}: `[lints]` is declared with an empty body, which is neither inheritance nor a \
         restated table and is not a shape this scan was written to read",
        path.display(),
        header,
    );
    if normalized == "workspace=true" {
        Inheritance::Inherits
    } else {
        Inheritance::Diverges(format!("`[lints]` says `{joined}`"))
    }
}

/// The code part of one line, with any trailing comment removed.
///
/// A `#` inside a double-quoted string is not a comment. No manifest here puts
/// one there, and tracking it costs three lines.
fn code(line: &str) -> String {
    let mut out = String::new();
    let mut in_string = false;
    for character in line.chars() {
        match character {
            '"' => in_string = !in_string,
            '#' if !in_string => return out,
            _ => {}
        }
        out.push(character);
    }
    out
}

/// Every double-quoted string in one accumulated array.
fn quoted(array: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut current: Option<String> = None;
    for character in array.chars() {
        match (character, current.as_mut()) {
            ('"', None) => current = Some(String::new()),
            ('"', Some(_)) => values.push(current.take().expect("a string is open")),
            (_, Some(value)) => value.push(character),
            (_, None) => {}
        }
    }
    assert!(
        current.is_none(),
        "the `members` array holds an unterminated string, so the member set below is whatever \
         had been read when the quote opened",
    );
    values
}
