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
//! replaced the prose. That check is now gone too, which is the subject of the
//! next section.
//!
//! # Where the rule is enforced
//!
//! **Nowhere mechanically. It is held by review of the diff that adds an
//! encoder.** A `scripts/check_workspace.py` pass once pinned every permitted
//! framing definition in a `FRAMING_SITE_CITATIONS` table; that table went in
//! `0b31488`, and the Python gate itself in `e197176`, which replaced it with
//! the `Makefile` of cargo commands. Neither has a successor, so nothing fails
//! today when a new encoder frames a length by hand.
//!
//! The history is kept because it says what a replacement would have to do. A
//! crate-local test came first: it walked one crate, so copies in
//! `tiler-compiler` and `tiler-reference` were outside its reach by
//! construction; it matched a list of four helper names, so `tiler-compiler`'s
//! `encode_count` was outside its reach by spelling; and it searched for
//! `.len() as u64`, so open-coded copies *in this crate* were outside its reach
//! for writing the same bytes as `u64::try_from(…).expect(…).to_be_bytes()`.
//! The workspace pass that replaced it recognized a *shape* — a `&mut Vec<u8>`
//! sink plus one `usize`, `&[u8]`, or `&str` payload, or one statement that
//! both reads a length and writes it as fixed-width bytes — because every name
//! list tried had been incomplete. A future check needs that property; a name
//! list reintroduces the blind spot it was built to close.
//!
//! One live check does bear on the framing, and it is not a recognizer.
//! `shape/env.rs` and `tiler-compiler`'s `feasibility.rs` each assert that an
//! identity opens with its domain's length by spelling the eight-byte prefix
//! out by hand. That independence from the encoder is exactly what would catch
//! this module changing the framing *width* — a test written with the encoder's
//! own helper could not. It says nothing about a second definition appearing
//! elsewhere, which is the failure this module exists to prevent.
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
/// that can reach `tiler-ir` is expected to call this and define none of its
/// own. Nothing checks that expectation any more — see the module documentation
/// — so a diff adding an encoder is where it is kept.
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
    // Reserved as one request because the two `extend_from_slice` calls below
    // would otherwise each test capacity and each be able to trigger a separate
    // reallocation-and-move of the whole buffer. A sampling profile of the
    // compile loop put this function at 8.93% of active self time, spread over
    // twenty-odd encoders with no dominant caller, so the growth is systemic to
    // the primitive rather than to any one encoder. The reserved amount is
    // exact, not an estimate.
    bytes.reserve(8 + value.len());
    push_len(bytes, value.len());
    bytes.extend_from_slice(value);
}
