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
//! the rule. Stating a convention did not hold it, so a mechanical check
//! replaced the prose.
//!
//! # Where the rule is enforced
//!
//! **In `scripts/check_workspace.py`, over every workspace member.** Its
//! `FRAMING_SITE_CITATIONS` table pins each permitted definition as a
//! `(path, signature, reason)` triple with a citation the site's own
//! documentation must carry, so adding, moving, renaming, or unciting one fails
//! the repository gate until the pin is updated in the same change.
//!
//! A crate-local test lived here first and was retired rather than kept as a
//! faster loop, on evidence rather than tidiness. It walked one crate, so seven
//! copies in `tiler-compiler` and `tiler-reference` were outside its reach by
//! construction. It matched a list of four helper names, so `tiler-compiler`'s
//! `encode_count` was outside its reach by spelling. And it searched for
//! `.len() as u64`, so three open-coded copies *in this crate* — in
//! `index/integer.rs` and `semantic/operation.rs` — were outside its reach for
//! writing the same bytes as `u64::try_from(…).expect(…).to_be_bytes()`. Keeping
//! it would have meant two recognizers for one rule, whose divergence is the
//! failure this module exists to prevent.
//!
//! The workspace check recognizes a *shape* — a `&mut Vec<u8>` sink plus one
//! `usize`, `&[u8]`, or `&str` payload, or one statement that both reads a
//! length and writes it as fixed-width bytes — because every name list tried so
//! far has been incomplete. Its two stated blind spots, a framing method and a
//! length bound to a local before it is written, are recorded in its own
//! docstring rather than here.
//!
//! It reads production code only. `shape/env.rs` and `tiler-compiler`'s
//! `feasibility.rs` each assert an identity opens with its domain's length by
//! spelling the eight-byte prefix out by hand, and that independence from the
//! encoder is exactly what would catch this module changing the framing width —
//! a test written with the encoder's own helper could not.
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
/// The workspace's **sole definition of canonical length framing**. Every crate
/// that can reach `tiler-ir` calls this and defines none of its own;
/// `scripts/check_workspace.py` pins the two crates that cannot reach it.
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
/// Built from [`push_len`] rather than framing again, so the two are **one
/// primitive pair** and not two rules that have to be kept agreeing.
///
/// # Panics
///
/// Panics under the same unreachable condition as [`push_len`].
pub fn push_slice(bytes: &mut Vec<u8>, value: &[u8]) {
    push_len(bytes, value.len());
    bytes.extend_from_slice(value);
}
