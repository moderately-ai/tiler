//! Every governed domain separator this crate admits, in one enumerated place.
//!
//! # The property
//!
//! `digest(domain, body)` hashes `domain || body`, which separates two subjects
//! only when no admitted domain is a prefix of another: otherwise a longer domain
//! and a shorter one followed by leading body bytes produce the same pre-image.
//! The same reasoning covers a domain used as a *framing tag* rather than as a
//! digest argument, because such a tag is the leading run of a canonical byte
//! sequence that is digested, compared, or recognised — `ARTIFACT_DOMAIN` is
//! admitted by `RecordedArtifactProgramIdentity::from_bytes` through
//! `starts_with`, so a domain that prefixed it would let another subject's bytes
//! be accepted as an artifact identity with no digest involved at all.
//!
//! One algorithm hashes all of them in one process, so the property is global to
//! the crate rather than local to a container. `docs/artifact-abi.md` records the
//! obligation normatively under "The governed digest".
//!
//! # Why the population is enumerated from a type
//!
//! This module exists because the previous check was a hand-written array of
//! eight beside a hand-written `8`, and it reported success either way. That
//! array has been described by two different figures, both measured at
//! `96dfe333` where this module landed, and they are not in conflict. The check
//! was named `no_governed_domain_of_either_container_prefixes_another`, and
//! against the two containers it named it covered 8 of 11: three domains — the
//! envelope's manifest framing tag and both payload domains — had been added to
//! the crate without being added to it. Against the set the obligation actually
//! ranges over it covered 8 of 18, because the artifact program's seven identity
//! and key domains lay outside the containers it considered at all. Only the
//! second figure is measured against the population this module enumerates, and
//! quoting the first without its scope understates the gap by those seven. A
//! count literal beside a list cannot notice a population that stopped covering
//! its domain.
//!
//! Four independent mechanisms replace it, and each can say *no* on its own:
//!
//! 1. [`GovernedDomain::ALL`] declares its length as
//!    [`variant_count`](core::mem::variant_count), so a variant added to the enum
//!    and left out of the list is an array-length build error at the list, and
//!    [`GovernedDomain::bytes`] is a wildcard-free match, so it is a second build
//!    error there.
//! 2. [`GovernedDomain::pinned_bytes`] restates each member's exact expected
//!    spelling in a wildcard-free match, so changing a live constant without its
//!    pin fails [`every_governed_domain_has_its_exact_pinned_bytes`] with both
//!    values, and widening the enum without a spelling decision fails to build.
//! 3. [`no_governed_domain_of_this_crate_prefixes_another`] is pairwise over
//!    `ALL`. Because a byte string is a prefix of itself, a list that padded its
//!    length by naming one variant twice fails that test rather than passing it.
//! 4. [`every_governed_domain_declared_in_the_source_is_enumerated`] reads this
//!    crate's own sources and requires every declared domain constant to appear
//!    in `ALL`. This is the one that catches the failure that produced this
//!    module: a constant declared in some module and never enumerated anywhere.
//!
//! The per-container counts are asserted against `variant_count` in a `const`
//! block, so widening the enum without deciding which container the new domain
//! belongs to is also a build error.

use crate::{program, proof};
use core::mem::variant_count;
use std::path::{Path, PathBuf};

/// Which of this crate's byte containers a governed domain belongs to.
///
/// The split exists because `docs/artifact-abi.md` states a count per container,
/// and those counts are what went stale. Naming the containers in the type lets
/// the census below check each one separately rather than only the total.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DomainContainer {
    /// The artifact envelope wire form, `tiler.artifact-envelope.*`.
    Envelope,
    /// The proof-case evidence sidecar, `tiler.proof-sidecar.*`.
    ProofSidecar,
    /// The artifact program's canonical identity and key domains.
    ///
    /// Spelled `tiler.artifact-program.*`, with the single exception of the route
    /// requirement's `tiler.artifact.route-requirement.v1`.
    ProgramIdentity,
}

impl DomainContainer {
    /// Governed domains the envelope admits.
    ///
    /// Mirrored by `docs/artifact-abi.md` under "The governed digest".
    pub(crate) const ENVELOPE: usize = 7;
    /// Governed domains the proof sidecar admits.
    ///
    /// Mirrored by `docs/artifact-abi.md` under "The sidecar's four governed
    /// domains".
    pub(crate) const PROOF_SIDECAR: usize = 4;
    /// Governed domains the artifact program's identity encoding admits.
    pub(crate) const PROGRAM_IDENTITY: usize = 7;
}

/// One governed domain separator admitted by this crate.
///
/// Every constant the crate declares as a domain separator has a variant here,
/// and the census test below is what keeps that true. A variant names the
/// constant rather than restating its bytes, so this enumeration cannot drift
/// from the value the encoder actually writes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GovernedDomain {
    /// Framing tag opening the envelope's canonical manifest bytes.
    EnvelopeManifest,
    /// Digest domain of the envelope manifest carried in the framing header.
    EnvelopeManifestDigest,
    /// Digest domain of one section's exact-content digest.
    EnvelopeSectionDigest,
    /// Digest domain of the external digest over a complete encoded envelope.
    EnvelopeEnvelopeDigest,
    /// Digest domain of the manifest's trailing derived-identity digest.
    EnvelopeIdentityDigest,
    /// Framing tag opening a carried payload's canonical metadata bytes.
    EnvelopePayloadMetadata,
    /// Digest domain of one carried payload's compilation identity.
    EnvelopePayloadIdentity,
    /// Framing tag opening the sidecar's canonical manifest bytes.
    SidecarManifest,
    /// Digest domain of the sidecar manifest carried in the framing header.
    SidecarManifestDigest,
    /// Digest domain of one framed payload's exact-content digest.
    SidecarPayloadDigest,
    /// Framing tag opening the sidecar's canonical identity bytes.
    SidecarIdentity,
    /// Separator opening the canonical artifact-program identity.
    ProgramArtifact,
    /// Separator of one independently compared and serialized stage key.
    ProgramStageKey,
    /// Separator of one carried payload descriptor's canonical key.
    ProgramPayloadKey,
    /// Separator of one selected provider's canonical key.
    ProgramProviderKey,
    /// Separator of one deferred predicate's canonical key.
    ProgramDeferredKey,
    /// Separator opening a delivered-realization record's canonical bytes.
    ProgramDeliveredRealization,
    /// Separator opening one core route requirement's canonical bytes.
    ProgramRouteRequirement,
}

impl GovernedDomain {
    /// Every governed domain this crate admits.
    ///
    /// The declared length is [`variant_count`], so a variant added to the enum
    /// and forgotten here is a build error at this list rather than a population
    /// that quietly stops covering its domain.
    pub(crate) const ALL: [Self; variant_count::<Self>()] = [
        Self::EnvelopeManifest,
        Self::EnvelopeManifestDigest,
        Self::EnvelopeSectionDigest,
        Self::EnvelopeEnvelopeDigest,
        Self::EnvelopeIdentityDigest,
        Self::EnvelopePayloadMetadata,
        Self::EnvelopePayloadIdentity,
        Self::SidecarManifest,
        Self::SidecarManifestDigest,
        Self::SidecarPayloadDigest,
        Self::SidecarIdentity,
        Self::ProgramArtifact,
        Self::ProgramStageKey,
        Self::ProgramPayloadKey,
        Self::ProgramProviderKey,
        Self::ProgramDeferredKey,
        Self::ProgramDeliveredRealization,
        Self::ProgramRouteRequirement,
    ];

    /// Returns the exact separator bytes this domain names.
    ///
    /// Wildcard-free, so a variant added without a value is an `E0004` here.
    pub(crate) const fn bytes(self) -> &'static [u8] {
        match self {
            Self::EnvelopeManifest => program::MANIFEST_DOMAIN,
            Self::EnvelopeManifestDigest => program::MANIFEST_DIGEST_DOMAIN,
            Self::EnvelopeSectionDigest => program::SECTION_DIGEST_DOMAIN,
            Self::EnvelopeEnvelopeDigest => program::ENVELOPE_DIGEST_DOMAIN,
            Self::EnvelopeIdentityDigest => program::IDENTITY_DIGEST_DOMAIN,
            Self::EnvelopePayloadMetadata => program::PAYLOAD_METADATA_DOMAIN,
            Self::EnvelopePayloadIdentity => program::PAYLOAD_IDENTITY_DOMAIN,
            Self::SidecarManifest => proof::MANIFEST_DOMAIN,
            Self::SidecarManifestDigest => proof::MANIFEST_DIGEST_DOMAIN,
            Self::SidecarPayloadDigest => proof::PAYLOAD_DIGEST_DOMAIN,
            Self::SidecarIdentity => proof::IDENTITY_DOMAIN,
            Self::ProgramArtifact => program::ARTIFACT_DOMAIN,
            Self::ProgramStageKey => program::STAGE_KEY_DOMAIN,
            Self::ProgramPayloadKey => program::PAYLOAD_KEY_DOMAIN,
            Self::ProgramProviderKey => program::PROVIDER_KEY_DOMAIN,
            Self::ProgramDeferredKey => program::DEFERRED_KEY_DOMAIN,
            Self::ProgramDeliveredRealization => program::DELIVERED_REALIZATION_DOMAIN,
            Self::ProgramRouteRequirement => program::ROUTE_REQUIREMENT_DOMAIN,
        }
    }

    /// Returns the independently pinned separator bytes this domain must keep.
    ///
    /// This mapping deliberately restates the values rather than returning the
    /// live constants as [`Self::bytes`] does. A legitimate version step costs
    /// the live declaration edit plus its one arm here; an accidental spelling
    /// change leaves the pin behind for the test below to report. The match is
    /// wildcard-free so a new variant cannot compile without an exact-byte
    /// decision.
    const fn pinned_bytes(self) -> &'static [u8] {
        match self {
            Self::EnvelopeManifest => b"tiler.artifact-envelope.manifest.v1\0",
            Self::EnvelopeManifestDigest => b"tiler.artifact-envelope.manifest-digest.v1\0",
            Self::EnvelopeSectionDigest => b"tiler.artifact-envelope.section-digest.v1\0",
            Self::EnvelopeEnvelopeDigest => b"tiler.artifact-envelope.envelope-digest.v1\0",
            Self::EnvelopeIdentityDigest => b"tiler.artifact-envelope.identity-digest.v1\0",
            Self::EnvelopePayloadMetadata => b"tiler.artifact-envelope.payload-metadata.v1\0",
            Self::EnvelopePayloadIdentity => b"tiler.artifact-envelope.payload-identity.v1\0",
            Self::SidecarManifest => b"tiler.proof-sidecar.manifest.v1\0",
            Self::SidecarManifestDigest => b"tiler.proof-sidecar.manifest-digest.v1\0",
            Self::SidecarPayloadDigest => b"tiler.proof-sidecar.payload-digest.v1\0",
            Self::SidecarIdentity => b"tiler.proof-sidecar.identity.v1\0",
            Self::ProgramArtifact => b"tiler.artifact-program.v17\0",
            Self::ProgramStageKey => b"tiler.artifact-program.stage.v3\0",
            Self::ProgramPayloadKey => b"tiler.artifact-program.payload.v1\0",
            Self::ProgramProviderKey => b"tiler.artifact-program.provider.v2\0",
            Self::ProgramDeferredKey => b"tiler.artifact-program.deferred.v2\0",
            Self::ProgramDeliveredRealization => {
                b"tiler.artifact-program.delivered-realization.v2\0"
            }
            Self::ProgramRouteRequirement => b"tiler.artifact.route-requirement.v1\0",
        }
    }

    /// Returns the container that admits this domain.
    ///
    /// Wildcard-free for the same reason [`Self::bytes`] is.
    pub(crate) const fn container(self) -> DomainContainer {
        match self {
            Self::EnvelopeManifest
            | Self::EnvelopeManifestDigest
            | Self::EnvelopeSectionDigest
            | Self::EnvelopeEnvelopeDigest
            | Self::EnvelopeIdentityDigest
            | Self::EnvelopePayloadMetadata
            | Self::EnvelopePayloadIdentity => DomainContainer::Envelope,
            Self::SidecarManifest
            | Self::SidecarManifestDigest
            | Self::SidecarPayloadDigest
            | Self::SidecarIdentity => DomainContainer::ProofSidecar,
            Self::ProgramArtifact
            | Self::ProgramStageKey
            | Self::ProgramPayloadKey
            | Self::ProgramProviderKey
            | Self::ProgramDeferredKey
            | Self::ProgramDeliveredRealization
            | Self::ProgramRouteRequirement => DomainContainer::ProgramIdentity,
        }
    }

    /// Returns every domain the given container admits.
    pub(crate) fn of(container: DomainContainer) -> Vec<Self> {
        Self::ALL
            .into_iter()
            .filter(|domain| domain.container() == container)
            .collect()
    }
}

/// Compiles only while the three per-container counts account for every variant.
///
/// This is the half that makes `docs/artifact-abi.md`'s per-container counts
/// maintainable: a domain added to the enum without a container count moving is a
/// build error here, which is where a reader is told a documented number has to
/// change. The runtime census below then checks that the split itself is right,
/// because a total can be correct while two containers are wrong by one each.
const _: () = {
    assert!(
        GovernedDomain::ALL.len() == variant_count::<GovernedDomain>(),
        "GovernedDomain::ALL must name every governed domain this crate admits",
    );
    assert!(
        DomainContainer::ENVELOPE
            + DomainContainer::PROOF_SIDECAR
            + DomainContainer::PROGRAM_IDENTITY
            == variant_count::<GovernedDomain>(),
        "the per-container governed-domain counts must account for every variant",
    );
};

/// Every governed domain retains its independently stated exact bytes.
///
/// What it takes for this check to say *no*: change any live domain declaration
/// without moving its one arm in [`GovernedDomain::pinned_bytes`]. The failure
/// names the member and prints both byte strings so the required second edit is
/// located rather than hunted.
#[test]
fn every_governed_domain_has_its_exact_pinned_bytes() {
    for domain in GovernedDomain::ALL {
        let expected = domain.pinned_bytes();
        let observed = domain.bytes();
        let expected_text = String::from_utf8_lossy(expected);
        let observed_text = String::from_utf8_lossy(observed);
        assert_eq!(
            observed, expected,
            "{domain:?}'s exact domain bytes moved:\n  expected bytes: {expected_text:?}\n  observed bytes: \
             {observed_text:?}\nA deliberate domain step costs the live declaration edit plus this member's \
             one `GovernedDomain::pinned_bytes` arm edit.",
        );
    }
}

/// No governed domain of this crate is a prefix of another.
///
/// The authority for the no-prefix obligation `docs/artifact-abi.md` states. It
/// is over the crate's whole admitted set rather than per container, because one
/// algorithm hashes every container in one process: a check confined to one would
/// report a separation it had not established.
///
/// Across crates the obligation is discharged by a spelling argument rather than
/// a check, because no crate holds the union. `tiler-ir` does not depend on this
/// crate and cannot see its domains at all; this crate does depend on `tiler-ir`,
/// which is the direction that would allow a union check, but finds no
/// enumeration to range over, because that crate keeps its population in a
/// private `PINNED_IDENTITY_DOMAINS`; and `tiler-digest`, which owns the
/// algorithm, deliberately knows no subject domain at all.
///
/// This comment used to argue that the two namespaces are disjoint by
/// construction: "every domain the shared IR admits opens `tiler.ir.`". That
/// claim is retired, and quoted here so the retired wording stays greppable — a
/// later search for it lands in this note rather than in a live premise. It was
/// never true at any commit. `EXPR_DOMAIN`, spelled
/// `tiler.artifact-program.abi-expr.v1`, moved into `tiler-ir` at `d1a95e18` on
/// 2026-07-25, and this sentence was written at `96dfe333` on 2026-08-08, so the
/// shared IR has always admitted a domain inside this crate's own
/// `tiler.artifact` prefix, with most of its domains spelled outside `tiler.ir.`
/// altogether. The first differing byte after `tiler.` does not separate the two
/// sets either: that domain and this crate's `ARTIFACT_DOMAIN` agree through the
/// whole of `tiler.artifact-program.`.
///
/// What separates them is each domain's terminator rather than its namespace.
/// Every domain [`GovernedDomain`] enumerates ends in a NUL that occurs nowhere
/// else in it, both asserted below, so one of them can prefix a longer byte
/// string only where that string carries a NUL at an interior position. Read at
/// this commit, `crates/tiler-ir/src/domains.rs` pins no spelling that does:
/// its terminated spellings carry the NUL only at the end, and the three it
/// spells without one carry no NUL at all and open `tiler.contract.` or
/// `tiler.scalar`, neither of which any domain enumerated here extends. What
/// the terminator leaves open is exact equality, which would be one spelling
/// shared by two crates rather than a prefix relation between two spellings.
///
/// Only this crate's half of that argument is checkable here, and it is checked:
/// a domain spelled outside this crate's established prefixes, or carrying any
/// NUL but its terminator, breaks a test rather than silently invalidating a
/// paragraph.
#[test]
fn no_governed_domain_of_this_crate_prefixes_another() {
    let domains = GovernedDomain::ALL;
    for (index, left) in domains.iter().enumerate() {
        for right in domains.iter().skip(index + 1) {
            // Equal strings are prefixes of each other, so this also rejects a
            // list that padded its length by naming one variant twice.
            assert!(
                !left.bytes().starts_with(right.bytes())
                    && !right.bytes().starts_with(left.bytes()),
                "the governed domain {left:?} and {right:?} are in a prefix relation: \
                 {:?} against {:?}",
                String::from_utf8_lossy(left.bytes()),
                String::from_utf8_lossy(right.bytes()),
            );
        }
    }

    for domain in domains {
        let bytes = domain.bytes();
        assert!(
            bytes.starts_with(b"tiler.artifact") || bytes.starts_with(b"tiler.proof-sidecar."),
            "{domain:?} is spelled {:?}, outside this crate's established prefixes. The \
             cross-crate half of the no-prefix obligation is a spelling argument rather than a \
             check, and a domain outside these prefixes breaks that argument rather than merely \
             weakening it.",
            String::from_utf8_lossy(bytes),
        );
        assert_eq!(
            bytes.last(),
            Some(&0),
            "{domain:?} is spelled {:?} and does not end with its NUL terminator. The terminator \
             is what keeps one domain from prefixing another that extends its name — \
             `…manifest` would prefix `…manifest-digest.v1` without it.",
            String::from_utf8_lossy(bytes),
        );
        // The first NUL being the last byte is the same property as "the
        // terminator is the only NUL", stated without counting the rest.
        assert_eq!(
            bytes.iter().position(|byte| *byte == 0),
            Some(bytes.len() - 1),
            "{domain:?} is spelled {:?} and carries a NUL that is not its terminator. The \
             cross-crate half of the no-prefix obligation rests on the terminator being the only \
             NUL a domain contains: that is what confines this domain to prefixing byte strings \
             which carry a NUL at an interior position, and no domain the shared IR pins does. An \
             interior NUL here reopens the case that argument closes.",
            String::from_utf8_lossy(bytes),
        );
    }
}

/// Each container admits exactly the number of domains the contract records.
///
/// The `const` block above pins the *total* against `variant_count`; this pins
/// the *split*, which a total cannot. Both numbers appear in
/// `docs/artifact-abi.md`, so a failure here names the document that has to move.
#[test]
fn each_container_admits_the_number_of_domains_the_contract_records() {
    for (container, expected) in [
        (DomainContainer::Envelope, DomainContainer::ENVELOPE),
        (
            DomainContainer::ProofSidecar,
            DomainContainer::PROOF_SIDECAR,
        ),
        (
            DomainContainer::ProgramIdentity,
            DomainContainer::PROGRAM_IDENTITY,
        ),
    ] {
        let found = GovernedDomain::of(container);
        assert_eq!(
            found.len(),
            expected,
            "{container:?} admits {} governed domain(s) and `docs/artifact-abi.md` records \
             {expected}. The census is {:?}.",
            found.len(),
            found
                .iter()
                .map(|domain| String::from_utf8_lossy(domain.bytes()).into_owned())
                .collect::<Vec<_>>(),
        );
    }
}

/// Every domain constant declared in this crate's sources is enumerated above.
///
/// This is the check that reaches the failure the module header describes: a
/// constant declared in some module and added to no list. The enum's
/// `variant_count` sizing cannot see that case, because nothing about a new
/// constant obliges anyone to add a variant.
///
/// What it would take for this to say *no*: declare a governed domain constant
/// anywhere under `src/` — a name ending in `_DOMAIN`, typed `&[u8]` — and do not
/// add it to [`GovernedDomain::ALL`]. That case is reached by the perturbation
/// recorded on this module's ticket.
///
/// The scan is written to fail loudly rather than quietly. It requires a plausible
/// file count, so a walk that stopped finding files cannot report an empty
/// population as an intact one; it requires the declared type to be exactly
/// `&[u8]`, so a differently-spelled declaration reddens rather than being
/// skipped; and it accepts a byte literal on the following line, because
/// `DELIVERED_REALIZATION_DOMAIN` is declared that way and a single-line matcher
/// would silently miss it.
#[test]
fn every_governed_domain_declared_in_the_source_is_enumerated() {
    // Assembled at run time so this scanner does not match its own source.
    let needle = format!("_DOMAIN{}", ":");

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect_rust_sources(&root, &mut files);
    files.sort();
    assert!(
        files.len() >= 25,
        "the scan found {} source file(s), which is fewer than this crate has; a walk that \
         stopped finding files would report an empty population as an intact one",
        files.len(),
    );

    let mut declared: Vec<(PathBuf, String, Vec<u8>)> = Vec::new();
    for path in &files {
        let text = std::fs::read_to_string(path).expect("a crate source file is readable");
        let mut from = 0_usize;
        while let Some(offset) = text[from..].find(needle.as_str()) {
            let at = from + offset;
            from = at + needle.len();

            // Stepping past the boundary character by its UTF-8 width rather
            // than by one: these files carry `…` and other multi-byte characters
            // in their documentation, and `boundary + 1` slices inside one.
            let name_start = text[..at]
                .char_indices()
                .rev()
                .find(|(_, character)| !character.is_alphanumeric() && *character != '_')
                .map_or(0, |(boundary, character)| boundary + character.len_utf8());
            let name = &text[name_start..at + "_DOMAIN".len()];

            let rest = &text[at + needle.len()..];
            let Some(equals) = rest.find('=') else {
                continue;
            };
            let declared_type = rest[..equals].trim();
            // A `use` item or a doc reference reaches the `=` of some later
            // statement; a declaration has exactly the governed type between the
            // colon and the `=`. Anything else that got this far is reported
            // rather than skipped, because "not recognised" and "not present"
            // must not look the same.
            if !declared_type.contains("[u8]") {
                continue;
            }
            assert_eq!(
                declared_type,
                "&[u8]",
                "{}: `{name}` is declared with type `{declared_type}`. This scan reads the \
                 governed spelling `&[u8]`; a domain declared another way would be skipped \
                 silently, so teach the scan about it rather than leaving it uncounted.",
                path.display(),
            );

            let body = &rest[equals + 1..];
            let opening = body.find("b\"").unwrap_or_else(|| {
                panic!(
                    "{}: `{name}` is declared `&[u8]` with no byte-string literal after its `=`",
                    path.display(),
                )
            });
            assert!(
                body[..opening].trim().is_empty(),
                "{}: `{name}` has {:?} between its `=` and its byte-string literal. The scan \
                 pairs a declaration with the next literal, so anything in between means it \
                 could be pairing the wrong one.",
                path.display(),
                &body[..opening],
            );
            let literal = &body[opening + 2..];
            let closing = literal.find('"').unwrap_or_else(|| {
                panic!(
                    "{}: `{name}`'s byte-string literal is unterminated",
                    path.display()
                )
            });
            declared.push((path.clone(), name.to_owned(), unescape(&literal[..closing])));
        }
    }

    assert!(
        declared.len() >= GovernedDomain::ALL.len(),
        "the scan recognised only {} domain declaration(s) across {} source file(s), fewer than \
         the {} this crate enumerates. The scan has stopped recognising declarations it once \
         read, so its verdict is about the scan rather than about the crate.",
        declared.len(),
        files.len(),
        GovernedDomain::ALL.len(),
    );

    let enumerated: Vec<&[u8]> = GovernedDomain::ALL
        .iter()
        .map(|domain| domain.bytes())
        .collect();
    for (path, name, bytes) in &declared {
        assert!(
            enumerated.contains(&bytes.as_slice()),
            "{}: `{name}` declares the governed domain {:?}, which `GovernedDomain::ALL` does not \
             enumerate. Every domain this crate admits owes a variant there, because the \
             no-prefix obligation is over the whole admitted set and a domain missing from the \
             enumeration is a collision the check cannot see.",
            path.display(),
            String::from_utf8_lossy(bytes),
        );
    }

    for domain in GovernedDomain::ALL {
        assert!(
            declared
                .iter()
                .any(|(_, _, bytes)| bytes.as_slice() == domain.bytes()),
            "{domain:?} is enumerated as {:?} but no declaration in this crate's sources produces \
             those bytes. Either the constant moved out of the crate or the scan stopped reading \
             its declaration; both make this census weaker than it reports.",
            String::from_utf8_lossy(domain.bytes()),
        );
    }
}

/// Resolves the escapes admitted in a governed domain's byte-string literal.
///
/// Only `\0` occurs today, and an unrecognised escape panics rather than being
/// passed through: a literal this function misread would be compared against the
/// enumeration as the wrong bytes, which is a false verdict rather than a gap.
fn unescape(literal: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(literal.len());
    let mut characters = literal.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            let mut buffer = [0_u8; 4];
            bytes.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
            continue;
        }
        match characters.next() {
            Some('0') => bytes.push(0),
            Some(other) => panic!(
                "the governed domain literal {literal:?} carries the escape `\\{other}`, which \
                 this scan does not resolve; teach it the escape rather than comparing the wrong \
                 bytes"
            ),
            None => panic!("the governed domain literal {literal:?} ends in a trailing backslash"),
        }
    }
    bytes
}

/// Collects every Rust source file under one directory, recursively.
fn collect_rust_sources(directory: &Path, into: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(directory).expect("the crate's source directory is readable");
    for entry in entries {
        let path = entry.expect("a directory entry is readable").path();
        if path.is_dir() {
            collect_rust_sources(&path, into);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            into.push(path);
        }
    }
}
