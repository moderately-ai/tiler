//! The durable identity record, as the consumer reads it.
//!
//! # What this record is, and what it is not
//!
//! It is the configuration a consumer is built against: the governed keys and
//! the digests over canonical bytes that say *which* artifact this consumer
//! expects and *which* target it will execute on. It is not a second source of
//! truth about the artifact. Every field it carries is also declared by the
//! envelope, and [`crate`]'s first case compares the two field by field — so a
//! record that disagreed with the bytes would fail rather than be believed.
//!
//! The parser is deliberately narrow: `key = value` per line, values that are
//! either governed text or lower-case hexadecimal, and one repeated `entry` key
//! for the payload's entry mapping. A line it does not understand is a parse
//! failure rather than a silently dropped field, because a record that lost a
//! join subject would make the join look like it had fewer of them.

use std::collections::BTreeMap;

/// One payload entry mapping, as the record states it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntryMapping {
    /// Governed backend entry key, as raw bytes.
    pub key: Vec<u8>,
    /// The backend's own entry-point symbol inside the carried object.
    pub symbol: String,
}

/// The durable record written beside one transported envelope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sidecar {
    /// Which variant of the producer's behaviour wrote this.
    pub variant: String,
    /// How the producer's cache orchestration resolved: published, hit, or bypassed.
    pub resolution: String,
    /// Process identifier of the producer that wrote it.
    pub producer_pid: String,
    /// Canonical identity of the artifact program, as raw bytes.
    pub artifact_identity: Vec<u8>,
    /// Composed cache subject the artifact resolved under, empty when bypassed.
    pub cache_subject: Vec<u8>,
    /// Governed backend family the payload declares.
    pub backend: String,
    /// Governed executable representation the payload declares.
    pub representation: String,
    /// Digest of the compilation subject the payload's metadata identifies.
    pub payload_digest: Vec<u8>,
    /// Governed key of the profile the payload was built for.
    pub payload_compatibility_key: String,
    /// Exact descriptor identity of that profile.
    pub payload_compatibility_descriptor: Vec<u8>,
    /// Governed key of the profile the packaged variant was assessed against.
    pub target_profile_key: String,
    /// Exact descriptor identity of that profile.
    pub target_profile_descriptor: Vec<u8>,
    /// The payload's entry mapping, in the order the record states it.
    pub entries: Vec<EntryMapping>,
}

impl Sidecar {
    /// Parses one record.
    ///
    /// # Panics
    ///
    /// Panics on a malformed line, an unknown key, a missing key, or a
    /// hexadecimal value that is not one. Each is a producer defect, and a
    /// parser that tolerated any of them would report a join with fewer subjects
    /// than the producer wrote.
    #[must_use]
    pub fn parse(record: &str) -> Self {
        let mut fields: BTreeMap<&str, &str> = BTreeMap::new();
        let mut entries = Vec::new();
        for line in record.lines().filter(|line| !line.is_empty()) {
            let (key, value) = line
                .split_once(" = ")
                .unwrap_or_else(|| panic!("the record line {line:?} is not `key = value`"));
            if key == "entry" {
                let (entry_key, symbol) = value
                    .split_once(' ')
                    .unwrap_or_else(|| panic!("the entry line {value:?} names no symbol"));
                entries.push(EntryMapping {
                    key: unhex(entry_key),
                    symbol: symbol.to_owned(),
                });
                continue;
            }
            assert!(
                fields.insert(key, value).is_none(),
                "the record repeats the key {key:?}",
            );
        }
        // Every key the producer can write, checked so an unknown one is a
        // failure. Without this, a renamed join subject would simply stop being
        // read and every case below would keep passing on the ones that remain.
        let known: Vec<&str> = REQUIRED.iter().chain(OPTIONAL.iter()).copied().collect();
        for key in fields.keys() {
            assert!(
                known.contains(key),
                "the record carries the unknown key {key:?}; this parser reads {known:?}",
            );
        }
        let text = |key: &str| -> String {
            (*fields
                .get(key)
                .unwrap_or_else(|| panic!("the record omits {key:?}")))
            .to_owned()
        };
        let bytes = |key: &str| -> Vec<u8> { unhex(&text(key)) };
        Self {
            variant: text("variant"),
            resolution: text("resolution"),
            producer_pid: text("producer-pid"),
            artifact_identity: bytes("artifact-identity"),
            cache_subject: bytes("cache-subject"),
            backend: text("backend"),
            representation: text("representation"),
            payload_digest: bytes("payload-digest"),
            payload_compatibility_key: text("payload-compatibility-key"),
            payload_compatibility_descriptor: bytes("payload-compatibility-descriptor"),
            target_profile_key: text("target-profile-key"),
            target_profile_descriptor: bytes("target-profile-descriptor"),
            entries,
        }
    }
}

/// Keys the producer writes only for some variants, or that no case reads.
const OPTIONAL: [&str; 3] = ["cache-bypass", "payload-schema", "execution-policy"];

/// Every key this parser requires the record to carry.
const REQUIRED: [&str; 12] = [
    "variant",
    "resolution",
    "producer-pid",
    "artifact-identity",
    "cache-subject",
    "backend",
    "representation",
    "payload-digest",
    "payload-compatibility-key",
    "payload-compatibility-descriptor",
    "target-profile-key",
    "target-profile-descriptor",
];

/// Decodes one lower-case hexadecimal run.
fn unhex(value: &str) -> Vec<u8> {
    assert!(
        value.len().is_multiple_of(2),
        "the hexadecimal value {value:?} has an odd length",
    );
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let text = std::str::from_utf8(pair).expect("a hexadecimal pair is ASCII");
            u8::from_str_radix(text, 16)
                .unwrap_or_else(|_| panic!("{text:?} is not a hexadecimal byte"))
        })
        .collect()
}
