//! Every unsafe-code permission in the workspace is one of the four admitted sites.
//!
//! ADR 0079 admits unsafe case by case. The compiler holds the first half of
//! that boundary: fourteen members inherit `unsafe_code = "forbid"`, and the
//! two members that must reach Metal buffer storage restate it as `deny`, so an
//! unsafe operation there still needs a local lint permission. This test holds
//! the other half: the exact `(workspace-relative path, item signature,
//! reason)` of every permission.
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
//! This is a deliberately narrow Rust lexer, not a Rust parser. It recognizes
//! line comments, nested block comments, ordinary and raw strings, character
//! literals, identifiers, and balanced delimiters. Comments and strings are
//! discarded before attributes are examined, so prose and the live doc-comment
//! fixture in `tiler-conformance/src/lib.rs` cannot become sites. A permission
//! is recognized only as a direct `#[allow(...)]` whose comma-separated meta
//! list contains the whole lint name and one ordinary string-literal `reason`.
//! The following item must be a function, and its complete signature is read
//! through the top-level body brace, so wrapped attributes and wrapped
//! signatures are one site.
//!
//! Everything outside that boundary fails closed. An inner attribute, a
//! `cfg_attr`, another attribute form that contains the lint token, a
//! non-literal reason, an admitted module or implementation, an unclosed
//! comment/string/delimiter, or the lint token anywhere outside a recognized
//! direct allow is an error naming the file and line. ADR 0079 permits a
//! function or module in principle, but the current population contains
//! functions only; teaching this check another item form belongs in the same
//! reviewed change that admits one.
//!
//! Only permissions are inventoried. The compiler remains the authority that
//! makes an unsafe operation without one fail: the workspace-wide lint checks
//! keep every member at `forbid` or `deny`, and this test does not restate that
//! table.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

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

/// One found site's exact item and reason, keyed by path and signature.
type Sites = BTreeMap<(String, String), String>;

/// The output of one scan, including parsing failures.
#[derive(Debug, Default)]
struct Scan {
    sites: Sites,
    errors: Vec<String>,
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

#[test]
fn the_workspace_unsafe_sites_are_exactly_the_four_admitted_ones() {
    let root = workspace_root();
    let members = workspace_members(&root);
    assert!(
        members.len() >= MEMBER_POPULATION_FLOOR,
        "unsafe-site census: the root manifest yielded {} member(s), below the floor of \
         {MEMBER_POPULATION_FLOOR}; a scan over a collapsed member set cannot say no",
        members.len(),
    );

    let sources = workspace_sources(&root, &members);
    assert!(
        sources.len() >= RUST_SOURCE_FILE_FLOOR,
        "unsafe-site census: found {} Rust source file(s) across {} member(s), below the floor \
         of {RUST_SOURCE_FILE_FLOOR}; a shrunken walk cannot report a clean inventory",
        sources.len(),
        members.len(),
    );

    let scan = scan_files(&root, &sources);
    let violations = validate_pins(scan, &ADMITTED_SITES);
    eprintln!(
        "unsafe-site census: {} Rust source file(s) across {} member(s); {} admitted site(s): \
         {:?}",
        sources.len(),
        members.len(),
        ADMITTED_SITES.len(),
        ADMITTED_SITES
            .iter()
            .map(|site| (site.path, site.item))
            .collect::<Vec<_>>(),
    );
    assert!(
        violations.is_empty(),
        "workspace unsafe-site inventory failed:\n{}",
        violations.join("\n"),
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

/// The workspace root, two levels above the facade crate.
fn workspace_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("the facade crate sits two levels below the workspace root")
        .to_path_buf();
    let manifest = root.join("Cargo.toml");
    let text = read(&manifest);
    assert!(
        text.contains("[workspace]"),
        "{} declares no workspace",
        manifest.display(),
    );
    root
}

/// The member paths declared by the root manifest.
///
/// This narrow parser intentionally recognizes only the table-and-array form
/// the repository uses. A dotted key or non-string array entry fails instead
/// of silently shrinking the source population.
fn workspace_members(root: &Path) -> Vec<String> {
    let manifest = root.join("Cargo.toml");
    let text = read(&manifest);
    let lines: Vec<&str> = text.lines().collect();
    let mut in_workspace = false;

    for (index, line) in lines.iter().enumerate() {
        let trimmed = manifest_code(line).trim().to_owned();
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
        let value = value.trim_start().strip_prefix('=').unwrap_or_else(|| {
            panic!(
                "{}:{}: `members` has no `=` and cannot be read",
                manifest.display(),
                index + 1,
            )
        });

        let mut array = value.to_owned();
        let mut cursor = index;
        while !array.contains(']') {
            cursor += 1;
            assert!(
                cursor < lines.len(),
                "{}:{}: the `members` array never closes",
                manifest.display(),
                index + 1,
            );
            array.push('\n');
            array.push_str(&manifest_code(lines[cursor]));
        }
        let members = quoted_values(&array, &manifest, index + 1);
        let unique: BTreeSet<&str> = members.iter().map(String::as_str).collect();
        assert_eq!(
            unique.len(),
            members.len(),
            "{}:{}: the member list repeats a path",
            manifest.display(),
            index + 1,
        );
        return members;
    }

    panic!(
        "{} has no `members` key under `[workspace]`; the unsafe-site scan has no roots",
        manifest.display(),
    );
}

/// Collects every Rust source beneath every declared member.
fn workspace_sources(root: &Path, members: &[String]) -> Vec<PathBuf> {
    let mut all = Vec::new();
    for member in members {
        let directory = root.join(member);
        assert!(
            directory.is_dir(),
            "workspace member `{member}` is not a directory, so its unsafe sites cannot be read",
        );
        let mut sources = Vec::new();
        collect_rust_sources(&directory, &mut sources);
        assert!(
            !sources.is_empty(),
            "workspace member `{member}` contributes no Rust source file; a member omitted from \
             the walk would otherwise look safely empty",
        );
        all.extend(sources);
    }
    all.sort();
    all.dedup();
    all
}

/// Recursively collects Rust source and rejects symlinks at the scan boundary.
fn collect_rust_sources(directory: &Path, into: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("{} is readable: {error}", directory.display()));
    for entry in entries {
        let entry = entry.expect("a workspace source directory entry is readable");
        let path = entry.path();
        let kind = entry
            .file_type()
            .unwrap_or_else(|error| panic!("{} has a readable file type: {error}", path.display()));
        assert!(
            !kind.is_symlink(),
            "{} is a symlink inside a workspace member; following it could escape or duplicate \
             the governed source population",
            path.display(),
        );
        if kind.is_dir() {
            collect_rust_sources(&path, into);
        } else if kind.is_file()
            && path.extension().and_then(|extension| extension.to_str()) == Some("rs")
        {
            into.push(path);
        }
    }
}

/// Scans every source path.
fn scan_files(root: &Path, sources: &[PathBuf]) -> Scan {
    let mut whole = Scan::default();
    for source in sources {
        let relative = source
            .strip_prefix(root)
            .expect("a member source lies under the workspace root")
            .to_string_lossy()
            .replace('\\', "/");
        let text = read(source);
        let scan = scan_text(&relative, &text);
        whole.errors.extend(scan.errors);
        for (key, reason) in scan.sites {
            if whole.sites.insert(key.clone(), reason).is_some() {
                whole.errors.push(format!(
                    "unsafe-sites.{}: `{}` is reported twice",
                    key.0, key.1,
                ));
            }
        }
    }
    whole
}

/// Scans one Rust source file for direct unsafe-code permissions.
fn scan_text(path: &str, source: &str) -> Scan {
    let tokens = match lex(path, source) {
        Ok(tokens) => tokens,
        Err(error) => {
            return Scan {
                sites: Sites::new(),
                errors: vec![error],
            };
        }
    };
    let mut scan = Scan::default();
    let mut accounted = BTreeSet::new();
    let mut index = 0;

    while index < tokens.len() {
        if !punct(&tokens[index], "#") {
            index += 1;
            continue;
        }
        let mut open = index + 1;
        let inner = tokens.get(open).is_some_and(|token| punct(token, "!"));
        if inner {
            open += 1;
        }
        if !tokens.get(open).is_some_and(|token| punct(token, "[")) {
            index += 1;
            continue;
        }
        let Some(end) = matching_delimiter(&tokens, open) else {
            scan.errors.push(format!(
                "unsafe-sites.{path}:{}: an attribute never closes",
                tokens[index].line,
            ));
            break;
        };
        let occurrences: Vec<usize> = (open + 1..end)
            .filter(|position| ident(&tokens[*position], "unsafe_code"))
            .collect();
        if occurrences.is_empty() {
            index = end + 1;
            continue;
        }
        accounted.extend(occurrences.iter().copied());

        if inner {
            scan.errors.push(format!(
                "unsafe-sites.{path}:{}: a crate-level unsafe-code allow is outside the admitted \
                 per-item boundary",
                tokens[index].line,
            ));
            index = end + 1;
            continue;
        }
        if !tokens
            .get(open + 1)
            .is_some_and(|token| ident(token, "allow"))
        {
            scan.errors.push(format!(
                "unsafe-sites.{path}:{}: `unsafe_code` appears outside a supported direct \
                `#[allow(...)]`; cfg_attr and other lint attributes fail closed",
                tokens[index].line,
            ));
            index = end + 1;
            continue;
        }

        let reason = match direct_allow_reason(path, &tokens, open, end) {
            Ok(reason) => reason,
            Err(error) => {
                scan.errors.push(error);
                index = end + 1;
                continue;
            }
        };
        let (item, _) = match following_function_signature(path, &tokens, end + 1) {
            Ok(item) => item,
            Err(error) => {
                scan.errors.push(error);
                index = end + 1;
                continue;
            }
        };
        let key = (path.to_owned(), item.clone());
        if scan.sites.insert(key, reason).is_some() {
            scan.errors.push(format!(
                "unsafe-sites.{path}: `{item}` carries unsafe-code permission twice",
            ));
        }
        index = end + 1;
    }

    for (position, token) in tokens.iter().enumerate() {
        if ident(token, "unsafe_code") && !accounted.contains(&position) {
            scan.errors.push(format!(
                "unsafe-sites.{path}:{}: `unsafe_code` appears outside a supported direct \
                 `#[allow(...)]` attribute",
                token.line,
            ));
        }
    }
    scan
}

/// Reads the reason from one supported direct allow attribute.
fn direct_allow_reason(
    path: &str,
    tokens: &[Token],
    open_bracket: usize,
    close_bracket: usize,
) -> Result<String, String> {
    let line = tokens[open_bracket].line;
    let open_paren = open_bracket + 2;
    if !tokens
        .get(open_paren)
        .is_some_and(|token| punct(token, "("))
    {
        return Err(format!(
            "unsafe-sites.{path}:{line}: the allow attribute does not open a meta list",
        ));
    }
    let Some(close_paren) = matching_delimiter(tokens, open_paren) else {
        return Err(format!(
            "unsafe-sites.{path}:{line}: the allow attribute's meta list never closes",
        ));
    };
    if close_paren + 1 != close_bracket {
        return Err(format!(
            "unsafe-sites.{path}:{line}: tokens follow the allow meta list before `]`; this \
             attribute form is unsupported",
        ));
    }

    let mut cursor = open_paren + 1;
    let mut saw_lint = false;
    let mut reason = None;
    while cursor < close_paren {
        if punct(&tokens[cursor], ",") {
            return Err(format!(
                "unsafe-sites.{path}:{line}: the allow list has an empty entry",
            ));
        }
        let entry_line = tokens[cursor].line;
        let Some(mut name) = identifier_text(&tokens[cursor]).map(str::to_owned) else {
            return Err(format!(
                "unsafe-sites.{path}:{entry_line}: an allow entry does not begin with an \
                 identifier; this meta syntax is unsupported",
            ));
        };
        cursor += 1;
        while cursor + 1 < close_paren && punct(&tokens[cursor], "::") {
            let Some(segment) = identifier_text(&tokens[cursor + 1]) else {
                return Err(format!(
                    "unsafe-sites.{path}:{entry_line}: an allow path ends after `::`",
                ));
            };
            name.push_str("::");
            name.push_str(segment);
            cursor += 2;
        }

        if name == "reason" {
            if !tokens.get(cursor).is_some_and(|token| punct(token, "=")) {
                return Err(format!(
                    "unsafe-sites.{path}:{entry_line}: `reason` is not assigned an ordinary \
                     string literal",
                ));
            }
            let Some(Token {
                kind: TokenKind::StringLiteral(value),
                ..
            }) = tokens.get(cursor + 1)
            else {
                return Err(format!(
                    "unsafe-sites.{path}:{entry_line}: `reason` is not an ordinary string \
                     literal; computed and raw forms are unsupported",
                ));
            };
            if reason.replace(value.clone()).is_some() {
                return Err(format!(
                    "unsafe-sites.{path}:{entry_line}: the allow attribute states two reasons",
                ));
            }
            cursor += 2;
        } else if name == "unsafe_code" {
            if saw_lint {
                return Err(format!(
                    "unsafe-sites.{path}:{entry_line}: the allow attribute names unsafe_code \
                     twice",
                ));
            }
            saw_lint = true;
        } else if name.ends_with("::unsafe_code") {
            return Err(format!(
                "unsafe-sites.{path}:{entry_line}: `{name}` is not the whole unsafe-code lint \
                 name",
            ));
        }

        if cursor < close_paren {
            if !punct(&tokens[cursor], ",") {
                return Err(format!(
                    "unsafe-sites.{path}:{}: allow entries must be comma-separated",
                    tokens[cursor].line,
                ));
            }
            cursor += 1;
            if cursor == close_paren {
                break;
            }
        }
    }

    if !saw_lint {
        return Err(format!(
            "unsafe-sites.{path}:{line}: the recognized allow did not contain unsafe_code as a \
             whole lint name",
        ));
    }
    reason.ok_or_else(|| {
        format!(
            "unsafe-sites.{path}:{line}: the unsafe-code permission has no ordinary string \
             `reason` as ADR 0079 requires",
        )
    })
}

/// Returns the complete signature of the function following an attribute.
fn following_function_signature(
    path: &str,
    tokens: &[Token],
    mut cursor: usize,
) -> Result<(String, usize), String> {
    while cursor < tokens.len() && punct(&tokens[cursor], "#") {
        let open = cursor + 1;
        if tokens.get(open).is_some_and(|token| punct(token, "!")) {
            return Err(format!(
                "unsafe-sites.{path}:{}: an inner attribute cannot follow a per-item allow",
                tokens[cursor].line,
            ));
        }
        if !tokens.get(open).is_some_and(|token| punct(token, "[")) {
            break;
        }
        let Some(end) = matching_delimiter(tokens, open) else {
            return Err(format!(
                "unsafe-sites.{path}:{}: a trailing item attribute never closes",
                tokens[cursor].line,
            ));
        };
        cursor = end + 1;
    }
    let start = cursor;
    let line = tokens.get(start).map_or(1, |token| token.line);

    let Some(fn_position) = (start..tokens.len()).find(|position| ident(&tokens[*position], "fn"))
    else {
        return Err(format!(
            "unsafe-sites.{path}:{line}: the unsafe-code permission precedes no function; only \
             the current function-site boundary is supported",
        ));
    };
    for token in &tokens[start..fn_position] {
        let admitted = match &token.kind {
            TokenKind::Ident(name) => matches!(
                name.as_str(),
                "pub" | "crate" | "self" | "super" | "in" | "const" | "async" | "unsafe" | "extern"
            ),
            TokenKind::StringLiteral(_) => true,
            TokenKind::Punct(value) => matches!(value.as_str(), "(" | ")" | "::"),
        };
        if !admitted {
            return Err(format!(
                "unsafe-sites.{path}:{}: unsupported tokens precede `fn`; the permission may not \
                 name a function item",
                token.line,
            ));
        }
    }

    let mut delimiters = Vec::new();
    for position in start..tokens.len() {
        let token = &tokens[position];
        if punct(token, "{") && delimiters.is_empty() {
            let signature = render_signature(&tokens[start..position]);
            if signature.is_empty() {
                return Err(format!(
                    "unsafe-sites.{path}:{line}: the admitted function has an empty signature",
                ));
            }
            return Ok((signature, position));
        }
        if punct(token, ";") && delimiters.is_empty() {
            return Err(format!(
                "unsafe-sites.{path}:{line}: the admitted function has no body",
            ));
        }
        match punct_text(token) {
            Some("(") => delimiters.push(")"),
            Some("[") => delimiters.push("]"),
            Some("<") => delimiters.push(">"),
            Some(value @ (")" | "]" | ">")) => {
                let expected = delimiters.pop().ok_or_else(|| {
                    format!(
                        "unsafe-sites.{path}:{}: unmatched `{value}` in the admitted signature",
                        token.line,
                    )
                })?;
                if value != expected {
                    return Err(format!(
                        "unsafe-sites.{path}:{}: `{value}` closes a delimiter expecting \
                         `{expected}` in the admitted signature",
                        token.line,
                    ));
                }
            }
            _ => {}
        }
    }
    Err(format!(
        "unsafe-sites.{path}:{line}: the admitted function's body never begins",
    ))
}

/// Compares a scan with the exact admitted population.
fn validate_pins(mut scan: Scan, admitted: &[AdmittedSite]) -> Vec<String> {
    let mut expected = Sites::new();
    for site in admitted {
        let key = (site.path.to_owned(), site.item.to_owned());
        assert!(
            expected.insert(key, site.reason.to_owned()).is_none(),
            "the admitted-site table repeats {} `{}`",
            site.path,
            site.item,
        );
    }
    assert!(
        !expected.is_empty(),
        "the admitted-site table is empty; an empty scan would pass vacuously",
    );

    for key in scan.sites.keys().filter(|key| !expected.contains_key(*key)) {
        scan.errors.push(format!(
            "unsafe-sites.{}: `{}` admits unsafe_code and is not pinned; ADR 0079 makes a new \
             site a new decision",
            key.0, key.1,
        ));
    }
    for key in expected.keys().filter(|key| !scan.sites.contains_key(*key)) {
        scan.errors.push(format!(
            "unsafe-sites.{}: pinned site `{}` is gone; remove its pin in the same reviewed \
             change that removes the permission",
            key.0, key.1,
        ));
    }
    for (key, found) in &scan.sites {
        if let Some(pinned) = expected.get(key)
            && found != pinned
        {
            scan.errors.push(format!(
                "unsafe-sites.{}: `{}` states reason {found:?}, pinned as {pinned:?}",
                key.0, key.1,
            ));
        }
    }
    scan.errors.sort();
    scan.errors
}

/// Lexes the Rust constructs relevant to attributes, dropping comments and
/// string-like prose before the lint name can be observed.
fn lex(path: &str, source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut index = 0;
    let mut line = 1;

    while index < source.len() {
        let tail = &source[index..];
        if tail.starts_with("//") {
            if let Some(end) = tail.find('\n') {
                index += end;
            } else {
                break;
            }
            continue;
        }
        if tail.starts_with("/*") {
            let start_line = line;
            index += 2;
            let mut depth = 1_usize;
            while index < source.len() && depth != 0 {
                let rest = &source[index..];
                if rest.starts_with("/*") {
                    depth += 1;
                    index += 2;
                } else if rest.starts_with("*/") {
                    depth -= 1;
                    index += 2;
                } else {
                    let character = next_char(source, index);
                    if character == '\n' {
                        line += 1;
                    }
                    index += character.len_utf8();
                }
            }
            if depth != 0 {
                return Err(format!(
                    "unsafe-sites.{path}:{start_line}: a block comment never closes",
                ));
            }
            continue;
        }
        if let Some((end, newlines)) = raw_string_span(source, index) {
            tokens.push(Token {
                kind: TokenKind::Punct("<raw-string>".to_owned()),
                line,
            });
            line += newlines;
            index = end;
            continue;
        }

        let character = next_char(source, index);
        if character.is_whitespace() {
            if character == '\n' {
                line += 1;
            }
            index += character.len_utf8();
            continue;
        }
        if character == '"' {
            let start_line = line;
            let (end, content, newlines) =
                ordinary_string(source, index).ok_or_else(|| {
                    format!(
                        "unsafe-sites.{path}:{start_line}: an ordinary string literal never closes",
                    )
                })?;
            tokens.push(Token {
                kind: TokenKind::StringLiteral(content),
                line,
            });
            line += newlines;
            index = end;
            continue;
        }
        if character == '\''
            && let Some(end) = character_literal_end(source, index)
        {
            line += source[index..end].matches('\n').count();
            index = end;
            continue;
        }
        if is_ident_start(character) {
            let start = index;
            index += character.len_utf8();
            while index < source.len() {
                let next = next_char(source, index);
                if !is_ident_continue(next) {
                    break;
                }
                index += next.len_utf8();
            }
            tokens.push(Token {
                kind: TokenKind::Ident(source[start..index].to_owned()),
                line,
            });
            continue;
        }

        let (punctuation, width) = if tail.starts_with("::") {
            ("::", 2)
        } else if tail.starts_with("->") {
            ("->", 2)
        } else if tail.starts_with("=>") {
            ("=>", 2)
        } else {
            (
                &source[index..index + character.len_utf8()],
                character.len_utf8(),
            )
        };
        tokens.push(Token {
            kind: TokenKind::Punct(punctuation.to_owned()),
            line,
        });
        index += width;
    }
    Ok(tokens)
}

/// The exclusive span and newline count of a raw string beginning at `start`.
fn raw_string_span(source: &str, start: usize) -> Option<(usize, usize)> {
    let tail = &source[start..];
    let prefix = if tail.starts_with("br") || tail.starts_with("cr") {
        2
    } else if tail.starts_with('r') {
        1
    } else {
        return None;
    };
    let mut cursor = start + prefix;
    let mut hashes = 0;
    while source[cursor..].starts_with('#') {
        hashes += 1;
        cursor += 1;
    }
    if !source[cursor..].starts_with('"') {
        return None;
    }
    cursor += 1;
    let closing = format!("\"{}", "#".repeat(hashes));
    let rest = &source[cursor..];
    let relative = rest.find(&closing)?;
    let end = cursor + relative + closing.len();
    Some((end, source[start..end].matches('\n').count()))
}

/// An ordinary string's exclusive end, raw content, and newline count.
fn ordinary_string(source: &str, start: usize) -> Option<(usize, String, usize)> {
    let mut cursor = start + 1;
    let content_start = cursor;
    let mut escaped = false;
    while cursor < source.len() {
        let character = next_char(source, cursor);
        if !escaped && character == '"' {
            let content = source[content_start..cursor].to_owned();
            let end = cursor + 1;
            return Some((end, content, source[start..end].matches('\n').count()));
        }
        escaped = !escaped && character == '\\';
        cursor += character.len_utf8();
    }
    None
}

/// The exclusive end of a character literal, or `None` for a lifetime tick.
fn character_literal_end(source: &str, start: usize) -> Option<usize> {
    let mut cursor = start + 1;
    if cursor >= source.len() {
        return None;
    }
    let first = next_char(source, cursor);
    if first == '\\' {
        cursor += 1;
        if cursor >= source.len() {
            return None;
        }
        cursor += next_char(source, cursor).len_utf8();
    } else {
        cursor += first.len_utf8();
    }
    source[cursor..].starts_with('\'').then_some(cursor + 1)
}

/// The closing delimiter for the one opening token, respecting nesting.
fn matching_delimiter(tokens: &[Token], open: usize) -> Option<usize> {
    let first = punct_text(tokens.get(open)?)?;
    let expected = match first {
        "(" => ")",
        "[" => "]",
        "{" => "}",
        _ => return None,
    };
    let mut stack = vec![expected];
    for (position, token) in tokens.iter().enumerate().skip(open + 1) {
        match punct_text(token) {
            Some("(") => stack.push(")"),
            Some("[") => stack.push("]"),
            Some("{") => stack.push("}"),
            Some(value @ (")" | "]" | "}")) => {
                if stack.pop()? != value {
                    return None;
                }
                if stack.is_empty() {
                    return Some(position);
                }
            }
            _ => {}
        }
    }
    None
}

/// Renders a stable, human-readable item signature from lexed tokens.
fn render_signature(tokens: &[Token]) -> String {
    let mut rendered = String::new();
    let mut previous: Option<String> = None;
    for (index, token) in tokens.iter().enumerate() {
        let current = match &token.kind {
            TokenKind::Ident(value) | TokenKind::Punct(value) => value.clone(),
            TokenKind::StringLiteral(value) => format!("\"{value}\""),
        };
        if current == "," && tokens.get(index + 1).is_some_and(|next| punct(next, ")")) {
            continue;
        }
        let tight_before = matches!(
            current.as_str(),
            ")" | "]" | ">" | "," | ";" | ":" | "::" | "(" | "[" | "<" | "."
        );
        let tight_after_previous = previous
            .as_deref()
            .is_some_and(|value| matches!(value, "(" | "[" | "<" | "::" | "&" | "'" | "."));
        if !rendered.is_empty() && !tight_before && !tight_after_previous {
            rendered.push(' ');
        }
        rendered.push_str(&current);
        previous = Some(current);
    }
    rendered
}

/// Reads a UTF-8 file or fails naming it.
fn read(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{} is readable UTF-8: {error}", path.display()))
}

/// The non-comment part of one manifest line.
fn manifest_code(line: &str) -> String {
    let mut code = String::new();
    let mut in_string = false;
    for character in line.chars() {
        match character {
            '"' => in_string = !in_string,
            '#' if !in_string => break,
            _ => {}
        }
        code.push(character);
    }
    code
}

/// Every double-quoted value in the root member array.
fn quoted_values(array: &str, path: &Path, line: usize) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = None;
    for character in array.chars() {
        match (character, current.as_mut()) {
            ('"', None) => current = Some(String::new()),
            ('"', Some(_)) => values.push(current.take().expect("a member string is open")),
            (_, Some(value)) => value.push(character),
            (_, None) => {}
        }
    }
    assert!(
        current.is_none(),
        "{}:{line}: the member array contains an unterminated string",
        path.display(),
    );
    assert!(
        !values.is_empty(),
        "{}:{line}: the member array contains no string paths",
        path.display(),
    );
    values
}

/// The source character beginning at one byte boundary.
fn next_char(source: &str, index: usize) -> char {
    source[index..]
        .chars()
        .next()
        .expect("the lexer index is inside the source")
}

/// Whether one character can begin an identifier relevant to this scan.
fn is_ident_start(character: char) -> bool {
    character == '_' || character.is_alphabetic()
}

/// Whether one character can continue an identifier relevant to this scan.
fn is_ident_continue(character: char) -> bool {
    character == '_' || character.is_alphanumeric()
}

/// Whether a token is one exact identifier.
fn ident(token: &Token, expected: &str) -> bool {
    matches!(&token.kind, TokenKind::Ident(value) if value == expected)
}

/// One identifier token's text.
fn identifier_text(token: &Token) -> Option<&str> {
    match &token.kind {
        TokenKind::Ident(value) => Some(value),
        TokenKind::StringLiteral(_) | TokenKind::Punct(_) => None,
    }
}

/// Whether a token is one exact punctuation token.
fn punct(token: &Token, expected: &str) -> bool {
    punct_text(token) == Some(expected)
}

/// One punctuation token's text.
fn punct_text(token: &Token) -> Option<&str> {
    match &token.kind {
        TokenKind::Punct(value) => Some(value),
        TokenKind::Ident(_) | TokenKind::StringLiteral(_) => None,
    }
}
