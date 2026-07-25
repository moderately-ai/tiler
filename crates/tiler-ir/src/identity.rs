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
