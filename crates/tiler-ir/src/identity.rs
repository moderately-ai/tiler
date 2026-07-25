//! The canonical byte-encoding primitives every identity derivation shares.
//!
//! Identity in this workspace is a digest over a canonical encoding, so two
//! encoders that disagree by one byte name the same subject with two different
//! identities — and nothing downstream can tell that from two genuinely
//! different subjects. The primitives that decide that framing therefore need
//! exactly one definition, not one per encoder that happens to agree today.
//!
//! Before `relocate-abi-expressions-into-tiler-ir` there were four: kernel
//! identity, program identity, and ABI expression identity each carried a
//! private copy, and the artifact codec imported a fourth path to one of them.
//! They already disagreed in form — the kernel copy narrowed with an `as` cast
//! where the others used a checked conversion. On the 64-bit little-endian
//! address space the Rust gate admits, `usize` is `u64` and the two emit the
//! same eight bytes, so the divergence was latent rather than live. That is the
//! hazard: a silent digest change is invisible in review and indistinguishable
//! from a real one in a cache.
//!
//! `finish-consolidating-tiler-ir-length-framing` then found five more inside
//! this crate — in `schedule/model.rs`, `semantic/{types,registry,identity}.rs`,
//! and `index/scalar.rs` — after this module already existed and already said
//! the rule. Stating a convention did not hold it, which is why this module's
//! `length_framing_has_exactly_one_definition_in_this_crate` test now checks it
//! mechanically instead.
//!
//! # The framing rule
//!
//! A length prefix precedes variable-width content, fixed at eight bytes
//! big-endian. Fixed width means the prefix cannot itself be ambiguous, and
//! prefixing means no concatenation of fields is: without it, `("ab", "c")` and
//! `("a", "bc")` would encode identically.
//!
//! Big-endian is a canonical-form choice, not a host concern. These bytes are
//! hashed and compared, never loaded as integers, so the ordering only has to
//! be *stated* and stable.

/// Appends the fixed-width canonical framing prefix for `len` items.
///
/// Callers that follow this with the content itself should use [`push_slice`]
/// instead. This is for the cases where the content is not a byte run — a
/// shape's extents, or an arena's nodes — and the count still has to be framed.
///
/// # Panics
///
/// Panics when `len` exceeds `u64::MAX`, which is unreachable on the 64-bit
/// address spaces the Rust gate admits. The conversion is checked rather than
/// cast so that a future 128-bit host fails loudly here instead of silently
/// truncating a length and colliding two distinct subjects onto one identity.
pub fn push_len(bytes: &mut Vec<u8>, len: usize) {
    bytes.extend_from_slice(
        &u64::try_from(len)
            .expect("supported usize fits u64")
            .to_be_bytes(),
    );
}

/// Appends one length-prefixed byte run to a canonical encoding.
///
/// # Panics
///
/// Panics under the same unreachable condition as [`push_len`].
pub fn push_slice(bytes: &mut Vec<u8>, value: &[u8]) {
    push_len(bytes, value.len());
    bytes.extend_from_slice(value);
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    /// The two shapes a sixth copy of the framing has taken in this crate.
    ///
    /// Both are drawn from copies that actually existed, not from what one
    /// might imagine: four of the five were a private helper pair, and the
    /// fifth wrote the cast inline with no helper at all.
    const FORBIDDEN: [&str; 6] = [
        "fn push_len(",
        "fn push_slice(",
        "fn encode_len(",
        "fn encode_bytes(",
        ".len() as u64",
        ".rank() as u64",
    ];

    fn rust_sources(directory: &Path, found: &mut Vec<PathBuf>) {
        let entries =
            std::fs::read_dir(directory).expect("the crate's own source tree is readable");
        for entry in entries {
            let path = entry.expect("a readable directory entry").path();
            if path.is_dir() {
                rust_sources(&path, found);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                found.push(path);
            }
        }
    }

    /// No file but this one defines or open-codes canonical length framing.
    ///
    /// The module documentation records why prose was not enough: this module
    /// existed, and said the rule, while five copies grew anyway. Two of them
    /// were missed even by the ticket that set out to remove them, because it
    /// searched for *importers* of this module rather than for definitions —
    /// `grep -rn "crate::identity"` finds who complies, never who does not.
    ///
    /// **Bound of this check, stated so it is not mistaken for more.** It reads
    /// each file only up to its first `#[cfg(test)]` line, which is where every
    /// module in this crate puts its tests. It therefore governs production
    /// encoders and deliberately leaves test expectations alone — `shape/env.rs`
    /// asserts its identity begins with the domain's length by spelling the
    /// eight-byte prefix out independently, and that independence is exactly
    /// what would catch this module changing the framing width. A test that
    /// checked the encoder with the encoder's own helper could not.
    #[test]
    fn length_framing_has_exactly_one_definition_in_this_crate() {
        let source_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut sources = Vec::new();
        rust_sources(&source_root, &mut sources);
        assert!(
            sources.len() > 10,
            "the scan found {} files, which is too few to have walked the crate",
            sources.len(),
        );

        let mut offenders = Vec::new();
        for path in sources {
            if path == source_root.join("identity.rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("a readable Rust source file");
            let production = text
                .split_once("#[cfg(test)]")
                .map_or(text.as_str(), |(before, _)| before);
            for (number, line) in production.lines().enumerate() {
                for pattern in FORBIDDEN {
                    if line.contains(pattern) {
                        offenders.push(format!("{}:{}: {}", path.display(), number + 1, pattern));
                    }
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "canonical length framing has one definition, in `crate::identity`. \
             Use `push_len`/`push_slice` at these sites instead:\n{}",
            offenders.join("\n"),
        );
    }
}
