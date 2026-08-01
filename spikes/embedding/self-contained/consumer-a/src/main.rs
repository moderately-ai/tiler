//! The crate whose compilation embeds an artifact and whose run proves it.
//!
//! One invocation, so one build of this crate is one expansion. The count is
//! fixed and stated here because the driver judges a run by comparing the events
//! it collected against the number it can prove it should have seen — a scenario
//! that silently expanded nothing would otherwise be indistinguishable from one
//! where the expansion hit.
//!
//! Running it is what makes the embedding a fact rather than a compile that
//! happened to succeed: the bytes are read back through
//! [`std::hint::black_box`], summed, and compared against the length and
//! checksum the *expansion* recorded. The driver compares both against the
//! artifact file's own length and checksum, computed independently, so neither
//! side of the claim is checked only by itself.

/// The number of `embed!` invocations one build of this crate performs.
pub const INVOCATIONS: usize = 1;

embed_macro::embed!(EMBEDDED, "a");

fn main() {
    // `black_box` rather than a volatile read: the payload must survive constant
    // folding and dead-code elimination, and the workspace forbids `unsafe`.
    let bytes: &[u8] = std::hint::black_box(EMBEDDED);
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    println!("slot=a len={} fnv1a={hash:016x}", bytes.len());
    assert_eq!(
        bytes.len(),
        EMBEDDED_LEN,
        "the linked payload is not the length the expansion recorded",
    );
    assert_eq!(
        hash, EMBEDDED_FNV1A,
        "the linked payload is not the content the expansion recorded",
    );
}
