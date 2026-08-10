//! Kani harnesses over guarded copies of Tiler's canonical slice framing.
//!
//! `tiler-ir` does not compile under Kani 0.67.0's bundled nightly. Therefore
//! [`push_len`] and [`push_slice`] below are copies, not calls into the live
//! crate. `guard.sh` independently compares exactly these two copies with
//! `crates/tiler-ir/src/identity.rs`; the unrelated stale resource copies in the
//! predecessor spike are not in this guard's population.
//!
//! A proof here proves these copies over byte runs of at most
//! [`MAX_SYMBOLIC_BYTES`] bytes. It is evidence about the live primitives only
//! after `guard.sh` succeeds, and nothing in the repository gate forces that
//! script or these harnesses to run.

#![allow(dead_code)]

/// Largest byte-run length quantified by the two proof harnesses.
pub const MAX_SYMBOLIC_BYTES: usize = 4;

/// @source: crates/tiler-ir/src/identity.rs :: push_len
pub fn push_len(bytes: &mut Vec<u8>, len: usize) {
    bytes.extend_from_slice(
        &u64::try_from(len)
            .expect("supported usize fits u64")
            .to_be_bytes(),
    );
}

/// @source: crates/tiler-ir/src/identity.rs :: push_slice
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

#[cfg(kani)]
mod proofs {
    use super::*;

    fn encode(run: &[u8; MAX_SYMBOLIC_BYTES], len: usize) -> Vec<u8> {
        let mut encoded = Vec::new();
        push_slice(&mut encoded, &run[..len]);
        encoded
    }

    /// Injective over every byte run whose length is at most four.
    ///
    /// Bytes beyond each symbolic length are storage padding, not part of the
    /// input slice, and are deliberately excluded from the equality conclusion.
    /// Unwind 13 covers the eight-byte prefix plus four payload bytes with one
    /// spare iteration; Kani's unwinding assertion must discharge for the
    /// result to count.
    #[kani::proof]
    #[kani::unwind(13)]
    fn push_slice_injective_len_4() {
        let run_a: [u8; MAX_SYMBOLIC_BYTES] = kani::any();
        let run_b: [u8; MAX_SYMBOLIC_BYTES] = kani::any();
        let len_a: usize = kani::any();
        let len_b: usize = kani::any();
        kani::assume(len_a <= MAX_SYMBOLIC_BYTES);
        kani::assume(len_b <= MAX_SYMBOLIC_BYTES);

        let encoded_a = encode(&run_a, len_a);
        let encoded_b = encode(&run_b, len_b);
        if encoded_a == encoded_b {
            assert!(len_a == len_b, "equal framing must carry equal lengths");
            for index in 0..MAX_SYMBOLIC_BYTES {
                if index < len_a {
                    assert!(
                        run_a[index] == run_b[index],
                        "equal framing must carry equal active bytes"
                    );
                }
            }
        }
    }

    /// Prefix-free over every byte run whose length is at most four.
    ///
    /// Neither complete encoding may be a strict prefix of the other. The
    /// quantified run bound belongs to this model check, not to the framing
    /// construction: the fixed-width length prefix is prefix-free for every
    /// representable slice length.
    #[kani::proof]
    #[kani::unwind(13)]
    fn push_slice_prefix_free_len_4() {
        let run_a: [u8; MAX_SYMBOLIC_BYTES] = kani::any();
        let run_b: [u8; MAX_SYMBOLIC_BYTES] = kani::any();
        let len_a: usize = kani::any();
        let len_b: usize = kani::any();
        kani::assume(len_a <= MAX_SYMBOLIC_BYTES);
        kani::assume(len_b <= MAX_SYMBOLIC_BYTES);

        let encoded_a = encode(&run_a, len_a);
        let encoded_b = encode(&run_b, len_b);
        assert!(
            encoded_a.len() >= encoded_b.len() || !encoded_b.starts_with(&encoded_a),
            "one framed byte run is a strict prefix of another"
        );
        assert!(
            encoded_b.len() >= encoded_a.len() || !encoded_a.starts_with(&encoded_b),
            "one framed byte run is a strict prefix of another"
        );
    }
}
