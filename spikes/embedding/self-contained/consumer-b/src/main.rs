//! A second embedding crate, identical to `consumer-a` but for its slot.
//!
//! It is a separate crate rather than a second invocation because the two axes
//! it exists for are about crate boundaries: whether a second *crate* embedding
//! the same artifact reuses the first crate's compilation, and whether two
//! crates embedding different artifacts behave as the cache protocol expects
//! when Cargo builds them at once. A second invocation in one crate would
//! measure neither — Cargo gives one `rustc` process per crate, so concurrency
//! arrives between crates and never within one.

/// The number of `embed!` invocations one build of this crate performs.
pub const INVOCATIONS: usize = 1;

embed_macro::embed!(EMBEDDED, "b");

fn main() {
    let bytes: &[u8] = std::hint::black_box(EMBEDDED);
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    println!("slot=b len={} fnv1a={hash:016x}", bytes.len());
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
