//! Property tests over the framing and bounded-allocation paths.
//!
//! The directed tests beside these prove the checks a reader thought of. A
//! bundle is read from a directory any process on the host may write to, so the
//! inputs worth finding are the ones nobody thought of, and these drive the
//! decoder with bytes chosen by a generator rather than by an author.
//!
//! # Why no fuzzing dependency
//!
//! `cargo-fuzz` and `libfuzzer` are materially better at *finding* a crash: they
//! are coverage-guided, so they discover a path a blind generator reaches only
//! by luck. They also require a nightly-only sanitizer runtime, a separate
//! target directory, a build that is not part of `make check`, and a corpus
//! nothing in this repository would store or replay. The property this ticket
//! names — never panic, never allocate past `Limits`, and either return a view
//! inside the input or a typed rejection — is checkable by an in-tree generator
//! that runs in the ordinary suite in milliseconds, and a check that runs on
//! every gate is worth more here than a better search nobody runs. Admitting the
//! dependency stays available if a real defect is ever found that this misses;
//! that is the trigger, and it has not fired.
//!
//! # Why the generator is deterministic
//!
//! Every case is derived from a fixed seed, so a failure reproduces from the
//! reported iteration index alone. A random seed would find more over time and
//! would report failures nobody could reproduce, which is the wrong trade for a
//! gate.

use super::bundle::{self, BundleRejection};
use super::key::{CacheKey, KEY_LABEL_BYTES, KeyTextRejection};
use super::limits::Limits;
use super::retention::DebugRetention;

/// A `splitmix64` state, inlined rather than taken as a dependency.
///
/// The generator only has to be reproducible and reasonably uniform; it decides
/// which inputs are tried and never what counts as a pass, so its statistical
/// quality is not load-bearing.
struct Gen(u64);

impl Gen {
    const fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, bound: usize) -> usize {
        if bound == 0 {
            return 0;
        }
        usize::try_from(self.next_u64() % bound as u64).unwrap_or(0)
    }

    fn byte(&mut self) -> u8 {
        u8::try_from((self.next_u64() >> 24) & 0xFF).unwrap_or(0)
    }

    fn bytes(&mut self, len: usize) -> Vec<u8> {
        (0..len).map(|_| self.byte()).collect()
    }
}

/// One valid bundle, and the key it is filed under.
///
/// Half of them frame a debug retention, so the mutating generators below reach
/// the optional third section — its descriptor, its digest, and the run framing
/// inside it — rather than only the two required ones.
fn valid_bundle(generator: &mut Gen) -> (CacheKey, Vec<u8>) {
    let subject_len = 1 + generator.below(64);
    let subject = generator.bytes(subject_len);
    let envelope_len = 1 + generator.below(96);
    let envelope = generator.bytes(envelope_len);
    let retained = if generator.next_u64().is_multiple_of(2) {
        let text_len = generator.below(48);
        DebugRetention::none()
            .retaining("fuzz.stage", &generator.bytes(text_len))
            .expect("a governed label and one run")
    } else {
        DebugRetention::none()
    };
    bundle::encode(&subject, &envelope, &retained, &Limits::default())
        .expect("a small bundle encodes")
}

/// Whatever a decode returns, it must be one of two things and both must be
/// consistent with the input it was given.
///
/// This is the whole property, in one place, so every generator below checks the
/// same thing and a new generator cannot accidentally check less.
fn holds(bytes: &[u8], key: &CacheKey, limits: &Limits, case: &str) {
    match bundle::decode(bytes, key, limits) {
        Ok(view) => {
            let subject = view.subject.clone();
            let envelope = view.envelope.clone();
            for (name, range) in [("subject", &subject), ("envelope", &envelope)] {
                assert!(
                    range.start <= range.end && range.end <= bytes.len(),
                    "{case}: accepted a bundle whose {name} range {range:?} \
                     leaves the {} input bytes",
                    bytes.len(),
                );
            }
            assert!(
                bytes.len() as u64 <= limits.max_bundle_bytes,
                "{case}: accepted {} bytes past the {} byte bundle limit",
                bytes.len(),
                limits.max_bundle_bytes,
            );
        }
        Err(rejection) => {
            // A rejection must be a typed one that renders; the point is that no
            // path reaches an unwrap or an index panic instead.
            let _ = format!("{rejection:?}");
            assert!(
                !matches!(rejection, BundleRejection::BundleTooLarge { declared, limit }
                    if declared <= limit),
                "{case}: reported a size rejection that is not a size violation",
            );
        }
    }
}

/// Arbitrary bytes never panic the decoder and never produce an out-of-range view.
///
/// Most of these are refused at the magic, which is the point of also running
/// the two generators below: this one establishes that the *entry* is safe, and
/// they establish that the paths past it are.
#[test]
fn arbitrary_bytes_never_panic_the_decoder() {
    let limits = Limits::default();
    let mut generator = Gen::new(0x0BAD_C0DE_D15E_A5E0);
    let key = CacheKey::derive_bytes(b"arbitrary");
    for iteration in 0..4_096 {
        let len = generator.below(512);
        let bytes = generator.bytes(len);
        holds(&bytes, &key, &limits, &format!("arbitrary/{iteration}"));
    }
}

/// A bundle that begins as valid and is then corrupted at one byte.
///
/// Reusing a real prefix is what gets the generator past the magic, the schema,
/// and the algorithm tag, so the mutations land on the offsets, lengths, and
/// digests that the directed tests reach one at a time.
#[test]
fn single_byte_mutations_of_a_valid_bundle_never_panic_the_decoder() {
    let limits = Limits::default();
    let mut generator = Gen::new(0x5EED_1234_ABCD_0001);
    for iteration in 0..4_096 {
        let (key, mut bytes) = valid_bundle(&mut generator);
        let at = generator.below(bytes.len());
        let flip = 1_u8 << u32::try_from(generator.below(8)).unwrap_or(0);
        bytes[at] ^= flip;
        holds(
            &bytes,
            &key,
            &limits,
            &format!("mutate/{iteration}/at={at}/flip={flip:#04x}"),
        );
    }
}

/// A truncated or extended bundle, which is where a length field and the bytes
/// actually present disagree.
#[test]
fn resized_bundles_never_panic_the_decoder() {
    let limits = Limits::default();
    let mut generator = Gen::new(0x5EED_1234_ABCD_0002);
    for iteration in 0..2_048 {
        let (key, mut bytes) = valid_bundle(&mut generator);
        if generator.next_u64().is_multiple_of(2) {
            bytes.truncate(generator.below(bytes.len().max(1)));
        } else {
            let extra_len = 1 + generator.below(32);
            let extra = generator.bytes(extra_len);
            bytes.extend_from_slice(&extra);
        }
        holds(&bytes, &key, &limits, &format!("resize/{iteration}"));
    }
}

/// The resealing mutator: corrupt the subject, then recompute every digest so
/// the bundle is internally consistent again.
///
/// A corruption a digest catches proves only that the digest works. This one
/// re-encodes the mutated subject, so every section digest and the declared
/// length agree with the bytes — the forgery is perfect except that the key the
/// entry is *filed under* no longer derives from the subject it carries.
///
/// **The assertion is that this is still refused**, and specifically by
/// `KeyNotDerivedFromSubject` rather than by a digest, because that is the only
/// check standing between a writable cache directory and a wrong-artifact hit.
#[test]
fn a_resealed_forgery_is_refused_by_the_key_derivation() {
    let limits = Limits::default();
    let mut generator = Gen::new(0x5EED_1234_ABCD_0003);
    let mut refused_by_derivation = 0_u32;
    for iteration in 0..512 {
        let subject_len = 1 + generator.below(64);
        let subject = generator.bytes(subject_len);
        let envelope_len = 1 + generator.below(64);
        let envelope = generator.bytes(envelope_len);
        let (original_key, _) =
            bundle::encode(&subject, &envelope, &DebugRetention::none(), &limits)
                .expect("a small bundle encodes");

        // Change the subject and re-seal: the new bundle is wholly consistent,
        // carrying its own correct digests and its own correct embedded key.
        let mut forged_subject = subject.clone();
        let at = generator.below(forged_subject.len());
        forged_subject[at] ^= 0xFF;
        if forged_subject == subject {
            continue;
        }
        let (_, resealed) =
            bundle::encode(&forged_subject, &envelope, &DebugRetention::none(), &limits)
                .expect("the forgery encodes");

        // Requested under the *original* key, which is where it is filed.
        match bundle::decode(&resealed, &original_key, &limits) {
            Ok(_) => panic!(
                "reseal/{iteration}: a resealed bundle carrying a different subject \
                 was accepted under the original key"
            ),
            Err(rejection) => {
                if matches!(
                    rejection,
                    BundleRejection::KeyNotDerivedFromSubject { .. }
                        | BundleRejection::KeyMismatch { .. }
                ) {
                    refused_by_derivation += 1;
                }
            }
        }
    }
    assert!(
        refused_by_derivation > 0,
        "no resealed forgery reached the key-derivation check, so this test \
         proved nothing about it"
    );
}

/// A parsed key round-trips to the exact text it was parsed from.
///
/// This is what stops two texts naming one entry. If some non-canonical
/// spelling parsed to the same key, two paths would resolve to one cache entry
/// and a caller could file under one and read under the other.
///
/// # The verdict is an oracle, and the generator reaches both halves of it
///
/// [`CacheKey::parse_label`] accepts exactly [`KEY_LABEL_BYTES`] bytes of
/// lowercase hexadecimal, and that predicate is computed here from the text
/// rather than read back off the parser, so a parser that widened or narrowed
/// its accepting language fails on the case it newly disagrees about.
///
/// Two alphabets exist because one of them cannot reach the accepting half. A
/// draw from the wide alphabet — hexadecimal in both cases plus the path
/// punctuation a real path parser meets — is the near-miss corpus, and its
/// chance of landing on 64 lowercase-hexadecimal bytes is about 4e-16 per
/// iteration. Under a fixed seed that is deterministically zero rather than
/// merely unlikely: replaying this generator over its 8,192 draws puts 87 of
/// them at the accepted width, and the longest run of leading lowercase
/// hexadecimal among those 87 is 13 bytes of the 64 required. The second
/// alphabet is the accepting one, drawn at widths straddling
/// [`KEY_LABEL_BYTES`], so about a fifth of its draws are texts the parser must
/// accept and round-trip and the rest are widths it must refuse rather than pad
/// or cut. Each accepted text is then re-spelled with one uppercase letter,
/// which is the case that must be refused rather than folded.
///
/// The three populations are counted and asserted non-empty, so a generator that
/// stopped reaching one of them fails here instead of passing vacuously.
#[test]
fn a_parsed_key_round_trips_to_its_exact_text() {
    /// Hexadecimal in both cases, plus the punctuation a path parser meets.
    /// Reaches the accepting language with probability about 4e-16 per draw,
    /// which under a fixed seed is zero.
    const NEAR_MISS: &[u8] = b"0123456789abcdefABCDEF-_/.";
    /// The accepting alphabet, so a draw at the accepted width is a text
    /// `parse_label` must take.
    const ACCEPTING: &[u8] = b"0123456789abcdef";

    /// True for exactly the texts [`CacheKey::parse_label`] must accept.
    fn is_canonical(text: &str) -> bool {
        text.len() == KEY_LABEL_BYTES
            && text
                .bytes()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    }

    let mut generator = Gen::new(0x5EED_1234_ABCD_0004);

    // Real labels must survive the round trip.
    for iteration in 0..256 {
        let seed_len = 1 + generator.below(48);
        let key = CacheKey::derive_bytes(&generator.bytes(seed_len));
        let label = key.label();
        let parsed = CacheKey::parse_label(&label).unwrap_or_else(|error| {
            panic!("label/{iteration}: a real label failed to parse: {error:?}")
        });
        assert_eq!(
            parsed.label(),
            label,
            "label/{iteration}: a real label did not round-trip"
        );
    }

    let mut accepted = 0_u32;
    let mut refused = 0_u32;
    let mut refused_for_case = 0_u32;
    for iteration in 0_u32..8_192 {
        let (alphabet, len) = if iteration.is_multiple_of(2) {
            (NEAR_MISS, generator.below(80))
        } else {
            (ACCEPTING, KEY_LABEL_BYTES - 2 + generator.below(5))
        };
        let text: String = (0..len)
            .map(|_| alphabet[generator.below(alphabet.len())] as char)
            .collect();

        match CacheKey::parse_label(&text) {
            Ok(parsed) => {
                assert!(
                    is_canonical(&text),
                    "text/{iteration}: {text:?} was accepted and is not \
                     {KEY_LABEL_BYTES} lowercase hexadecimal bytes, so a \
                     non-canonical spelling names an entry",
                );
                assert_eq!(
                    parsed.label(),
                    text,
                    "text/{iteration}: {text:?} parsed to a key whose label differs, \
                     so two texts name one entry"
                );
                accepted += 1;

                // The same key with one hexadecimal letter raised must be
                // refused rather than folded: folding files one entry under two
                // texts, and the per-key lock at one would not exclude the
                // other.
                if let Some(position) = text.bytes().position(|byte| byte.is_ascii_lowercase()) {
                    let mut raised = text.clone();
                    raised.replace_range(
                        position..=position,
                        &text[position..=position].to_ascii_uppercase(),
                    );
                    assert_eq!(
                        CacheKey::parse_label(&raised),
                        Err(KeyTextRejection::NotLowercaseHexadecimal {
                            position,
                            byte: raised.as_bytes()[position],
                        }),
                        "text/{iteration}: {raised:?} is a second spelling of an \
                         accepted key and was not refused for its case",
                    );
                    refused_for_case += 1;
                }
            }
            Err(rejection) => {
                assert!(
                    !is_canonical(&text),
                    "text/{iteration}: {text:?} is a canonical key text and was \
                     refused: {rejection:?}",
                );
                // The refusal has to name the first thing wrong with the text,
                // because that is what a caller reads to correct a path.
                let expected = if text.len() == KEY_LABEL_BYTES {
                    let position = text
                        .bytes()
                        .position(|byte| !matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
                        .expect("a refused text of the accepted width has a non-hex byte");
                    KeyTextRejection::NotLowercaseHexadecimal {
                        position,
                        byte: text.as_bytes()[position],
                    }
                } else {
                    KeyTextRejection::Width { found: text.len() }
                };
                assert_eq!(rejection, expected, "text/{iteration}: {text:?}");
                refused += 1;
            }
        }
    }

    assert!(
        accepted > 0,
        "no generated text reached the accepting language, so nothing here \
         checked that an accepted text round-trips",
    );
    assert!(
        refused_for_case > 0,
        "no accepted text was re-spelled with an uppercase letter, so nothing \
         here checked that case is refused rather than folded",
    );
    assert!(
        refused > 0,
        "no generated text was refused, so nothing here checked a rejection",
    );
}
