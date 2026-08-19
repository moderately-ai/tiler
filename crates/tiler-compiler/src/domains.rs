//! Every `tiler.`-spelled literal this crate declares, pinned or classified.
//!
//! # The property
//!
//! An identity domain opens a canonical encoding and separates that subject
//! from every other one. Reverting or mistyping it does not make construction
//! fail: it republishes new bytes under an old name, or splits one subject into
//! two names. Digest goldens catch some such moves incidentally, but their
//! failure says only that a digest moved, and 13 of this crate's 25 live domain
//! reverts passed all 803 compiler tests before this module was added.
//!
//! This census makes the spelling itself the subject. Two assertions fail in
//! opposite directions:
//!
//! 1. [`every_tiler_spelled_literal_is_pinned_or_classified`] walks every Rust
//!    source under this crate's `src/` and `tests/` trees. A changed, newly
//!    declared, or unclassified spelling fails at its source location.
//! 2. [`every_pinned_identity_domain_has_its_exact_source_population`] walks the
//!    pin in the other direction. A reverted, removed, duplicated, or moved
//!    domain fails with its expected and observed `src/` and `tests/` counts and
//!    names the table that must move on a deliberate step. Counting the trees
//!    separately keeps a test fixture from masking a missing live declaration.
//!
//! A legitimate domain step therefore costs two edits: the source declaration
//! and its row in [`PINNED_IDENTITY_DOMAINS`]. That is the minimum for an exact
//! byte pin: a version floor would cost no second edit, but could not see a
//! revert to any version at or above the floor.
//!
//! # Why the population is read from source
//!
//! A [`core::mem::variant_count`]-sized enumeration would be incomplete here.
//! Nineteen domains are named constants, but six are inline literals with no
//! constant behind them: the request subject, program alternative, both region
//! identities, and both explain identities. A variant can mirror a constant;
//! it cannot prove it has reached a literal no constant names. The population
//! is therefore the source itself, with explicit floors on both the file walk
//! and the literal census so an empty or truncated walk cannot look green.
//!
//! # Unsupported fragment-derived domains
//!
//! The scanner evaluates one Rust literal token; it does not execute
//! `concat!`, `concat_bytes!`, or `include_bytes!`. At this revision,
//! `rg -n -g '!domains.rs' 'concat!|concat_bytes!|include_bytes!' \
//! crates/tiler-compiler/src crates/tiler-compiler/tests` excludes this
//! self-documentation and finds only the unrelated rendered-trace fixture in
//! `explain.rs`; reading that invocation finds no identity domain. This is an
//! explicit construction boundary: a future identity domain must remain one
//! literal, or the scanner and a fail-capable guard must widen in the same
//! change. Treat any new search hit that constructs an identity as a failed
//! census boundary, not an admitted exception; it must not be accepted merely
//! because this literal census stays green.
//!
//! # Classification is explicit and cannot shadow a pin
//!
//! This crate also declares `tiler.`-spelled provider names, governed keys,
//! diagnostic properties, and test fixtures. Those separate no canonical byte
//! subjects, so they are admitted either exactly or by a narrow namespace in
//! [`ADMITTED_NON_DOMAIN_LITERALS`] and [`ADMITTED_NON_DOMAIN_PREFIXES`]. An
//! unclassified namespace fails rather than disappearing from the census. A
//! NUL-terminated spelling is always an exact-domain candidate and cannot use
//! either admission path, and a prefix broad enough to swallow a pinned domain
//! fails separately.
//!
//! # Self-exclusion
//!
//! The walk must not read this file: its pin table contains every spelling the
//! occurrence assertion seeks, which would let the table satisfy itself. The
//! scanner removes exactly `src/domains.rs` and fails if that path was not
//! found, so renaming the module or weakening the walk cannot silently make the
//! occurrence check vacuous.

use std::path::{Path, PathBuf};

/// One pinned spelling and its exact population outside this module.
#[derive(Clone, Copy, Debug)]
struct PinnedDomain {
    bytes: &'static [u8],
    src_occurrences: usize,
    test_occurrences: usize,
}

impl PinnedDomain {
    const fn new(bytes: &'static [u8], src_occurrences: usize, test_occurrences: usize) -> Self {
        Self {
            bytes,
            src_occurrences,
            test_occurrences,
        }
    }

    const fn expected_occurrences(self, tree: SourceTree) -> usize {
        match tree {
            SourceTree::Src => self.src_occurrences,
            SourceTree::Tests => self.test_occurrences,
        }
    }
}

/// Every identity-domain spelling this crate declares, in exact bytes and counts.
///
/// Sorted by content and free of duplicates, both asserted below. Every row
/// retains its NUL terminator because the pin is over bytes written by the
/// encoder rather than a display name. There are 31 distinct strict domains:
/// 32 occurrences under `src/` because boundary property is restated by a unit
/// test, and 36 across `src/` plus `tests/` because four legality domains are
/// each restated once by an integration test.
const PINNED_IDENTITY_DOMAINS: &[PinnedDomain] = &[
    PinnedDomain::new(b"tiler.compiler.boundary-property-set.v3\0", 2, 0),
    PinnedDomain::new(b"tiler.compiler.fusion-legality-content.v1\0", 1, 0),
    PinnedDomain::new(b"tiler.compiler.fusion-legality-occurrence.v1\0", 1, 0),
    PinnedDomain::new(b"tiler.compiler.index-refinement-content.staged.v1\0", 1, 1),
    PinnedDomain::new(b"tiler.compiler.index-refinement-content.v2\0", 1, 1),
    PinnedDomain::new(
        b"tiler.compiler.index-refinement-occurrence.staged.v1\0",
        1,
        1,
    ),
    PinnedDomain::new(b"tiler.compiler.index-refinement-occurrence.v2\0", 1, 1),
    PinnedDomain::new(b"tiler.compiler.lowering-capability-registry.v2\0", 1, 0),
    PinnedDomain::new(
        b"tiler.compiler.physical-implementation-proposal.v3\0",
        1,
        0,
    ),
    PinnedDomain::new(b"tiler.compiler.region-content.v1\0", 1, 0),
    PinnedDomain::new(b"tiler.compiler.region-cover.v1\0", 1, 0),
    PinnedDomain::new(b"tiler.compiler.region-occurrence.v1\0", 1, 0),
    PinnedDomain::new(b"tiler.compiler.request-subject.v6\0", 1, 0),
    PinnedDomain::new(b"tiler.compiler.selected-physical-plan.v3\0", 1, 0),
    PinnedDomain::new(b"tiler.compiler.selected-physical-portfolio.v1\0", 1, 0),
    PinnedDomain::new(b"tiler.explain.compilation.v1\0", 1, 0),
    PinnedDomain::new(b"tiler.explain.trace.v1\0", 1, 0),
    PinnedDomain::new(b"tiler.program-alternative.v2\0", 1, 0),
    PinnedDomain::new(b"tiler.target-profile.cost-row.v1\0", 1, 0),
    PinnedDomain::new(b"tiler.target-profile.declaration.v11\0", 1, 0),
    PinnedDomain::new(
        b"tiler.target-profile.descriptor.subgroup-realization.v1\0",
        1,
        0,
    ),
    PinnedDomain::new(
        b"tiler.target-profile.descriptor.subgroup-width-query.v1\0",
        1,
        0,
    ),
    PinnedDomain::new(b"tiler.target-profile.descriptor.v11\0", 1, 0),
    PinnedDomain::new(b"tiler.target-profile.dtype-dispatchability.v2\0", 1, 0),
    PinnedDomain::new(b"tiler.target-profile.elementary-realization.v1\0", 1, 0),
    PinnedDomain::new(
        b"tiler.target-profile.evaluation-order-preservation.v1\0",
        1,
        0,
    ),
    PinnedDomain::new(b"tiler.target-profile.fact-sources.v4\0", 1, 0),
    PinnedDomain::new(b"tiler.target-profile.subgroup-realization.v1\0", 1, 0),
    PinnedDomain::new(b"tiler.target-profile.subgroup-width-query.v1\0", 1, 0),
    PinnedDomain::new(
        b"tiler.target-profile.synchronization-realization.v1\0",
        1,
        0,
    ),
    PinnedDomain::new(
        b"tiler.target-profile.workgroup-tree-width-policy.v1\0",
        1,
        0,
    ),
];

/// Complete non-domain literals whose namespace cannot safely be admitted.
///
/// `tiler.compiler` must be exact: admitting that as a prefix would swallow
/// fifteen pinned compiler domains. Sorted and duplicate-free.
const ADMITTED_NON_DOMAIN_LITERALS: &[&[u8]] = &[b"tiler.compiler", b"tiler.pipeline"];

/// Namespaces of `tiler.` literals that separate no canonical byte subjects.
///
/// These are provider identities, governed cost/capability/policy keys,
/// diagnostic properties, and fixtures. Sorted, duplicate-free, and forbidden
/// from prefixing any pinned domain.
const ADMITTED_NON_DOMAIN_PREFIXES: &[&[u8]] = &[
    b"tiler.affinity.",
    b"tiler.algebraic",
    b"tiler.capability.",
    b"tiler.contract.",
    b"tiler.cost.",
    b"tiler.feasibility.",
    b"tiler.governed-",
    b"tiler.normalize",
    b"tiler.prototype-",
    b"tiler.prototype.",
    b"tiler.reduction.",
    b"tiler.region.",
    b"tiler.rules.",
    b"tiler.scalar::",
    b"tiler.selection.",
    b"tiler.some-",
    b"tiler.strict-",
    b"tiler.target.",
    b"tiler.test",
];

/// Fewest Rust source files the recursive walk may find.
///
/// There were 63 across `src/` and `tests/` at this check when the census
/// landed. The scanner then removes this module and reads the remaining 62. A
/// floor allows additions and intentional removals while preventing a broken
/// walk from reporting an empty population as intact.
const MINIMUM_SOURCE_FILES: usize = 50;

/// Fewest `tiler.`-spelled literal occurrences the scanner may recognise.
///
/// It found 138 occurrences in 89 distinct spellings when this census landed.
/// Occurrence checks each pin independently; this floor protects the scanner and
/// the classified non-domain population from a broad recognition regression.
const MINIMUM_TILER_LITERALS: usize = 100;

/// Which crate source tree contains a literal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceTree {
    Src,
    Tests,
}

impl SourceTree {
    const fn name(self) -> &'static str {
        match self {
            Self::Src => "src",
            Self::Tests => "tests",
        }
    }
}

/// One `tiler.`-spelled literal and where the scan read it.
#[derive(Clone, Debug)]
struct FoundLiteral {
    tree: SourceTree,
    path: PathBuf,
    line: usize,
    content: Vec<u8>,
}

/// Every `tiler.`-spelled string or byte-string literal in this crate.
fn scan_crate_sources() -> Vec<FoundLiteral> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_root = root.join("src");
    let tests_root = root.join("tests");
    let mut files = Vec::new();
    collect_rust_sources(&src_root, &mut files);
    collect_rust_sources(&tests_root, &mut files);
    files.sort();
    assert!(
        files.len() >= MINIMUM_SOURCE_FILES,
        "the walk found {} `.rs` file(s) across `{}` and `{}`, fewer than the floor of \
         {MINIMUM_SOURCE_FILES}. A walk that stopped finding files reports an empty population \
         as intact, so this is a verdict about the walk rather than the crate.",
        files.len(),
        root.join("src").display(),
        root.join("tests").display(),
    );

    let this_module = root.join("src").join("domains.rs");
    let before = files.len();
    files.retain(|path| path != &this_module);
    assert!(
        files.len() + 1 == before,
        "the walk did not find this module at `{}`, so it removed nothing. The pin table in \
         that file restates every spelling it pins; reading it would let \
         `PINNED_IDENTITY_DOMAINS` satisfy its own occurrence assertion.",
        this_module.display(),
    );

    let mut found = Vec::new();
    for path in &files {
        let tree = if path.starts_with(&src_root) {
            SourceTree::Src
        } else {
            assert!(
                path.starts_with(&tests_root),
                "the crate-source walk returned `{}` outside `{}` and `{}`",
                path.display(),
                src_root.display(),
                tests_root.display(),
            );
            SourceTree::Tests
        };
        let text = std::fs::read_to_string(path).expect("a crate source file is readable");
        read_source_literals(tree, path, &text, &mut found);
    }
    assert!(
        found.len() >= MINIMUM_TILER_LITERALS,
        "the scan read {} `tiler.`-spelled literal(s) across {} source file(s), fewer than the \
         floor of {MINIMUM_TILER_LITERALS}. The scanner has stopped recognising literals it \
         once read, so this is a verdict about the scan rather than the crate.",
        found.len(),
        files.len(),
    );
    found
}

/// Reads every `tiler.`-spelled literal with enough Rust lexical state to skip prose.
///
/// This is a file walk rather than a line matcher: block comments nest, and raw
/// and cooked strings can cross lines. Keeping that state prevents either a
/// comment containing `//` or a multiline literal from hiding later live code.
fn read_source_literals(
    tree: SourceTree,
    path: &Path,
    source: &str,
    found: &mut Vec<FoundLiteral>,
) {
    let bytes = source.as_bytes();
    let mut cursor = 0;
    let mut line = 1;
    while cursor < bytes.len() {
        if bytes[cursor..].starts_with(b"//") {
            let after = bytes[cursor..]
                .iter()
                .position(|byte| *byte == b'\n')
                .map_or(bytes.len(), |offset| cursor + offset);
            advance(&mut cursor, after, bytes, &mut line);
            continue;
        }
        if bytes[cursor..].starts_with(b"/*") {
            let after = block_comment_end(bytes, cursor).unwrap_or_else(|| {
                panic!(
                    "{}:{line}: a block comment is unterminated; the census cannot distinguish \
                     later declarations from comment prose",
                    path.display(),
                )
            });
            advance(&mut cursor, after, bytes, &mut line);
            continue;
        }
        if let Some(raw) = raw_string_bounds(bytes, cursor) {
            let opening_line = line;
            assert!(
                raw.terminated,
                "{}:{opening_line}: a raw string is unterminated; the census cannot delimit its \
                 bytes",
                path.display(),
            );
            let mut content = bytes[raw.body_start..raw.body_end].to_vec();
            raw.kind.append_implicit_terminator(&mut content);
            if content.starts_with(b"tiler.") {
                found.push(FoundLiteral {
                    tree,
                    path: path.to_path_buf(),
                    line: opening_line,
                    content,
                });
            }
            advance(&mut cursor, raw.after, bytes, &mut line);
            continue;
        }
        if bytes[cursor] == b'\''
            && let Some(after) = char_literal_end(source, cursor)
        {
            advance(&mut cursor, after, bytes, &mut line);
            continue;
        }
        if bytes[cursor] == b'"' {
            let opening_line = line;
            let body_start = cursor + 1;
            let body_end = cooked_string_end(bytes, body_start).unwrap_or_else(|| {
                panic!(
                    "{}:{opening_line}: a cooked string is unterminated; the census cannot \
                     delimit its bytes",
                    path.display(),
                )
            });
            let body = &source[body_start..body_end];
            let mut content = unescape(path, opening_line, body);
            let kind = if cursor > 0 && bytes[cursor - 1] == b'c' {
                StringLiteralKind::C
            } else {
                StringLiteralKind::Rust
            };
            kind.append_implicit_terminator(&mut content);
            if content.starts_with(b"tiler.") {
                found.push(FoundLiteral {
                    tree,
                    path: path.to_path_buf(),
                    line: opening_line,
                    content,
                });
            }
            advance(&mut cursor, body_end + 1, bytes, &mut line);
            continue;
        }
        let next = cursor + 1;
        advance(&mut cursor, next, bytes, &mut line);
    }
}

/// Whether Rust implicitly terminates a string literal's evaluated bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StringLiteralKind {
    Rust,
    C,
}

impl StringLiteralKind {
    fn append_implicit_terminator(self, content: &mut Vec<u8>) {
        if self == Self::C {
            content.push(0);
        }
    }
}

/// One raw string's content and closing delimiter.
#[derive(Clone, Copy, Debug)]
struct RawStringBounds {
    body_start: usize,
    body_end: usize,
    after: usize,
    terminated: bool,
    kind: StringLiteralKind,
}

/// Returns the bounds of a raw string beginning at `cursor`, if one begins.
fn raw_string_bounds(bytes: &[u8], cursor: usize) -> Option<RawStringBounds> {
    let (raw_prefix, kind) = match (bytes.get(cursor), bytes.get(cursor + 1)) {
        (Some(b'r'), _) => (cursor, StringLiteralKind::Rust),
        (Some(b'b'), Some(b'r')) => (cursor + 1, StringLiteralKind::Rust),
        (Some(b'c'), Some(b'r')) => (cursor + 1, StringLiteralKind::C),
        _ => return None,
    };

    let mut opening_quote = raw_prefix + 1;
    while bytes.get(opening_quote) == Some(&b'#') {
        opening_quote += 1;
    }
    if bytes.get(opening_quote) != Some(&b'"') {
        return None;
    }
    let hashes = opening_quote - raw_prefix - 1;
    let mut closing_quote = opening_quote + 1;
    while closing_quote < bytes.len() {
        if bytes[closing_quote] == b'"'
            && bytes.get(closing_quote + 1..closing_quote + 1 + hashes)
                == Some(&bytes[raw_prefix + 1..opening_quote])
        {
            return Some(RawStringBounds {
                body_start: opening_quote + 1,
                body_end: closing_quote,
                after: closing_quote + 1 + hashes,
                terminated: true,
                kind,
            });
        }
        closing_quote += 1;
    }
    Some(RawStringBounds {
        body_start: opening_quote + 1,
        body_end: bytes.len(),
        after: bytes.len(),
        terminated: false,
        kind,
    })
}

/// Returns the closing quote of a cooked string.
fn cooked_string_end(bytes: &[u8], mut cursor: usize) -> Option<usize> {
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\\' => cursor = (cursor + 2).min(bytes.len()),
            b'"' => return Some(cursor),
            _ => cursor += 1,
        }
    }
    None
}

/// Returns the byte after a character literal, or `None` for a lifetime.
fn char_literal_end(source: &str, opening: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let content = opening + 1;
    let after_content = if bytes.get(content) == Some(&b'\\') {
        match bytes.get(content + 1) {
            Some(b'x') => content + 4,
            Some(b'u') if bytes.get(content + 2) == Some(&b'{') => {
                bytes[content + 3..].iter().position(|byte| *byte == b'}')? + content + 4
            }
            Some(_) => content + 2,
            None => return None,
        }
    } else {
        let character = source[content..].chars().next()?;
        if character == '\n' || character == '\r' || character == '\'' {
            return None;
        }
        content + character.len_utf8()
    };
    (bytes.get(after_content) == Some(&b'\'')).then_some(after_content + 1)
}

/// Returns the byte after a nested block comment, if it closes.
fn block_comment_end(bytes: &[u8], opening: usize) -> Option<usize> {
    let mut depth = 1_usize;
    let mut cursor = opening + 2;
    while cursor < bytes.len() {
        if bytes[cursor..].starts_with(b"/*") {
            depth += 1;
            cursor += 2;
        } else if bytes[cursor..].starts_with(b"*/") {
            depth -= 1;
            cursor += 2;
            if depth == 0 {
                return Some(cursor);
            }
        } else {
            cursor += 1;
        }
    }
    None
}

/// Moves a byte cursor and carries its one-based source line with it.
fn advance(cursor: &mut usize, after: usize, bytes: &[u8], line: &mut usize) {
    for byte in &bytes[*cursor..after] {
        if *byte == b'\n' {
            *line += 1;
        }
    }
    *cursor = after;
}

/// Resolves Rust's cooked-string escapes before namespace recognition.
///
/// Decoding before asking whether the bytes start with `tiler.` is
/// load-bearing: `\x74iler.` and a prefix split by backslash-newline both
/// evaluate to that namespace. An unknown or malformed escape fails closed
/// instead of letting a spelling disappear from the census.
fn unescape(path: &Path, line: usize, literal: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(literal.len());
    let mut characters = literal.chars().peekable();
    while let Some(character) = characters.next() {
        if character != '\\' {
            let mut buffer = [0_u8; 4];
            bytes.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
            continue;
        }
        match characters.next() {
            Some('0') => bytes.push(0),
            Some('n') => bytes.push(b'\n'),
            Some('r') => bytes.push(b'\r'),
            Some('t') => bytes.push(b'\t'),
            Some('\\') => bytes.push(b'\\'),
            Some('\'') => bytes.push(b'\''),
            Some('"') => bytes.push(b'"'),
            Some('x') => {
                let upper = characters.next().and_then(hex_digit).unwrap_or_else(|| {
                    panic!(
                        "{}:{line}: the literal {literal:?} has a malformed `\\xNN` escape; the \
                         census cannot decide its evaluated bytes",
                        path.display(),
                    )
                });
                let lower = characters.next().and_then(hex_digit).unwrap_or_else(|| {
                    panic!(
                        "{}:{line}: the literal {literal:?} has a malformed `\\xNN` escape; the \
                         census cannot decide its evaluated bytes",
                        path.display(),
                    )
                });
                bytes.push(u8::try_from(upper * 16 + lower).expect("two hex digits fit in a byte"));
            }
            Some('u') => {
                assert!(
                    characters.next() == Some('{'),
                    "{}:{line}: the literal {literal:?} has a malformed Unicode escape; the \
                     census cannot decide its evaluated bytes",
                    path.display(),
                );
                let mut value = 0_u32;
                let mut digits = 0_usize;
                loop {
                    match characters.next() {
                        Some('}') if digits > 0 => break,
                        Some('_') => {}
                        Some(character) if hex_digit(character).is_some() => {
                            value = value
                                .checked_mul(16)
                                .and_then(|value| value.checked_add(hex_digit(character).unwrap()))
                                .unwrap_or_else(|| {
                                    panic!(
                                        "{}:{line}: the literal {literal:?} has an overflowing \
                                         Unicode escape; the census cannot decide its evaluated \
                                         bytes",
                                        path.display(),
                                    )
                                });
                            digits += 1;
                        }
                        _ => panic!(
                            "{}:{line}: the literal {literal:?} has a malformed Unicode escape; \
                             the census cannot decide its evaluated bytes",
                            path.display(),
                        ),
                    }
                }
                let scalar = char::from_u32(value).unwrap_or_else(|| {
                    panic!(
                        "{}:{line}: the literal {literal:?} has a non-scalar Unicode escape; the \
                         census cannot decide its evaluated bytes",
                        path.display(),
                    )
                });
                let mut buffer = [0_u8; 4];
                bytes.extend_from_slice(scalar.encode_utf8(&mut buffer).as_bytes());
            }
            Some('\n') => {
                while characters
                    .peek()
                    .is_some_and(|character| character.is_whitespace())
                {
                    characters.next();
                }
            }
            Some('\r') => {
                assert!(
                    characters.next() == Some('\n'),
                    "{}:{line}: the literal {literal:?} has a bare carriage return escape; the \
                     census cannot decide its evaluated bytes",
                    path.display(),
                );
                while characters
                    .peek()
                    .is_some_and(|character| character.is_whitespace())
                {
                    characters.next();
                }
            }
            Some(other) => panic!(
                "{}:{}: the literal {literal:?} carries the escape `\\{other}`, which this \
                scanner does not resolve; teach it the escape rather than comparing the wrong \
                 bytes",
                path.display(),
                line,
            ),
            None => panic!(
                "{}:{}: the literal {literal:?} ends in a trailing backslash",
                path.display(),
                line,
            ),
        }
    }
    bytes
}

/// Interprets one hexadecimal digit.
fn hex_digit(character: char) -> Option<u32> {
    character.to_digit(16)
}

/// Collects every Rust source file under one directory recursively.
fn collect_rust_sources(directory: &Path, into: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(directory).expect("the source directory is readable");
    for entry in entries {
        let path = entry.expect("a directory entry is readable").path();
        if path.is_dir() {
            collect_rust_sources(&path, into);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            into.push(path);
        }
    }
}

/// Renders one spelling the way a Rust source literal writes it.
fn render(content: &[u8]) -> String {
    let mut rendered = String::new();
    for byte in content {
        if *byte == 0 {
            rendered.push_str("\\0");
        } else {
            rendered.push(char::from(*byte));
        }
    }
    rendered
}

/// Lexical prose cannot hide a later declaration or manufacture one.
#[test]
fn source_scanner_tracks_rust_comments_strings_and_character_literals() {
    let source = r##"
fn probe() {
    sink(b"\x74iler.live.after-hex-escape.v1\0");
    sink("\u{74}iler.live.after-unicode-escape.v1\0");
    sink(b"t\
        iler.live.after-prefix-continuation.v1\0");
    sink(c"tiler.target.live.cooked-c-domain.v1");
    sink(cr#"tiler.target.live.raw-c-domain.v1"#);
    sink(r"tiler.live.raw-rust-string.v1");
    let _ = "https://example.invalid"; sink(b"tiler.live.after-url.v1\0");
    /* // */ sink(b"tiler.live.after-block-comment.v1\0");
    /* outer /* // */ comment */ sink(b"tiler.live.after-nested-comment.v1\0");
    let _ = '"'; sink(b"tiler.live.after-quote-character.v1\0");
    let _ = r#"
        "tiler.prose.inside-multiline-raw-string.v1\0"
        // still raw-string content
    "#;
    sink(b"tiler.live.after-multiline-raw.v1\0");
    let _ = "continued \
        // still cooked-string content";
    sink(b"tiler.live.after-multiline-cooked.v1\0");
    // sink(b"tiler.prose.inside-line-comment.v1\0");
    /* sink(b"tiler.prose.inside-block-comment.v1\0"); */
}
"##;
    let mut found = Vec::new();
    read_source_literals(
        SourceTree::Src,
        Path::new("scanner-probe.rs"),
        source,
        &mut found,
    );
    let spellings: Vec<String> = found
        .iter()
        .map(|literal| render(&literal.content))
        .collect();
    assert_eq!(
        spellings,
        [
            "tiler.live.after-hex-escape.v1\\0",
            "tiler.live.after-unicode-escape.v1\\0",
            "tiler.live.after-prefix-continuation.v1\\0",
            "tiler.target.live.cooked-c-domain.v1\\0",
            "tiler.target.live.raw-c-domain.v1\\0",
            "tiler.live.raw-rust-string.v1",
            "tiler.live.after-url.v1\\0",
            "tiler.live.after-block-comment.v1\\0",
            "tiler.live.after-nested-comment.v1\\0",
            "tiler.live.after-quote-character.v1\\0",
            "tiler.live.after-multiline-raw.v1\\0",
            "tiler.live.after-multiline-cooked.v1\\0",
        ]
    );
}

/// Every source literal is an exact domain pin or an explicit non-domain.
#[test]
fn every_tiler_spelled_literal_is_pinned_or_classified() {
    let found = scan_crate_sources();
    for literal in &found {
        let pinned = PINNED_IDENTITY_DOMAINS
            .iter()
            .any(|pinned| pinned.bytes == literal.content);
        let admitted_literal = ADMITTED_NON_DOMAIN_LITERALS.contains(&literal.content.as_slice());
        let admitted_prefix = ADMITTED_NON_DOMAIN_PREFIXES
            .iter()
            .any(|prefix| literal.content.starts_with(prefix));
        let exact_domain_candidate = literal.content.last() == Some(&0);
        assert!(
            pinned || (!exact_domain_candidate && (admitted_literal || admitted_prefix)),
            "{}:{}: the literal `{}` is neither pinned in `src/domains.rs`'s \
             `PINNED_IDENTITY_DOMAINS` nor classified by its \
             `ADMITTED_NON_DOMAIN_LITERALS` or `ADMITTED_NON_DOMAIN_PREFIXES`. A NUL-terminated \
             literal is always an exact-domain candidate and must be pinned even inside an \
             admitted namespace. If an identity domain stepped, move its pin row with the \
             source edit; if this non-NUL spelling separates no canonical byte subjects, \
             classify it explicitly.",
            literal.path.display(),
            literal.line,
            render(&literal.content),
        );
    }
}

/// Every exact domain pin has its expected live and test populations.
#[test]
fn every_pinned_identity_domain_has_its_exact_source_population() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let found = scan_crate_sources();
    let mut mismatches = Vec::new();
    for pinned in PINNED_IDENTITY_DOMAINS {
        for tree in [SourceTree::Src, SourceTree::Tests] {
            let locations: Vec<String> = found
                .iter()
                .filter(|literal| literal.tree == tree && literal.content == pinned.bytes)
                .map(|literal| format!("{}:{}", literal.path.display(), literal.line))
                .collect();
            let expected = pinned.expected_occurrences(tree);
            if locations.len() != expected {
                mismatches.push(format!(
                    "`{}` in `{}/`: expected {expected} occurrence(s), found {} at [{}]",
                    render(pinned.bytes),
                    tree.name(),
                    locations.len(),
                    locations.join(", "),
                ));
            }
        }
    }
    assert!(
        mismatches.is_empty(),
        "`{}`'s `PINNED_IDENTITY_DOMAINS` occurrence census changed:\n{}\nA live declaration \
         cannot be supplied by a same-spelled test fixture: `src/` and `tests/` are counted \
         independently. Either a source domain stepped without its pin row, it was reverted, \
         duplicated, or the declaration moved between trees. A deliberate step costs the source \
         edit plus its row edit in that table; the scan read {} `tiler.` literal(s), so this is \
         about exact populations rather than an empty walk.",
        root.join("src/domains.rs").display(),
        mismatches.join("\n"),
        found.len(),
    );
}

/// All census tables are sorted by content and free of duplicates.
#[test]
fn census_tables_are_sorted_and_free_of_duplicates() {
    for pair in PINNED_IDENTITY_DOMAINS.windows(2) {
        assert!(
            pair[0].bytes < pair[1].bytes,
            "PINNED_IDENTITY_DOMAINS is out of order or repeats itself at `{}` and `{}`.",
            render(pair[0].bytes),
            render(pair[1].bytes),
        );
    }
    for (name, table) in [
        ("ADMITTED_NON_DOMAIN_LITERALS", ADMITTED_NON_DOMAIN_LITERALS),
        ("ADMITTED_NON_DOMAIN_PREFIXES", ADMITTED_NON_DOMAIN_PREFIXES),
    ] {
        for pair in table.windows(2) {
            assert!(
                pair[0] < pair[1],
                "{name} is out of order or repeats itself at `{}` and `{}`.",
                render(pair[0]),
                render(pair[1]),
            );
        }
    }
}

/// No non-domain classification can admit an exact pinned domain.
#[test]
fn no_non_domain_classification_swallows_a_pinned_domain() {
    for admitted in ADMITTED_NON_DOMAIN_LITERALS {
        for pinned in PINNED_IDENTITY_DOMAINS {
            assert!(
                *admitted != pinned.bytes,
                "the admitted non-domain literal `{}` equals the pinned identity domain `{}`, \
                 so the same spelling is classified both ways. Remove the non-domain row rather \
                 than leaving the exact pin ambiguous.",
                render(admitted),
                render(pinned.bytes),
            );
        }
    }
    for prefix in ADMITTED_NON_DOMAIN_PREFIXES {
        for pinned in PINNED_IDENTITY_DOMAINS {
            assert!(
                !pinned.bytes.starts_with(prefix),
                "the admitted non-domain prefix `{}` covers the pinned identity domain `{}`, so \
                 that domain's exact spelling is no longer compared against its row in \
                 `PINNED_IDENTITY_DOMAINS`. Narrow the prefix instead of leaving the pin \
                 unenforced.",
                render(prefix),
                render(pinned.bytes),
            );
        }
    }
}
