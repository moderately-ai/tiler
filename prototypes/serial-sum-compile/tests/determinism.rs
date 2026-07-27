//! Two producer runs, in two processes, agree byte for byte.
//!
//! # Why this is an integration test and not a unit case
//!
//! It began as one: the same in-process helper called twice, asserting the
//! envelope, the sidecar, and the artifact identity matched. That is a weaker
//! claim than its name, and weaker than what the ticket asks for. Two calls in
//! one process share an address space and a hash seed by construction, so a
//! value derived from either is identical in both by definition rather than by
//! design — the case could not fail for the reason it existed to detect.
//!
//! Only a second *process* re-randomizes those. Running the real binary also
//! covers what a helper cannot: argument handling, the order the two files are
//! written in, and the publication path a consumer actually gets.
//!
//! `env!("CARGO_BIN_EXE_…")` and `CARGO_TARGET_TMPDIR` are set for integration
//! targets and not for unit tests inside a `[[bin]]`, which is the mechanical
//! reason this file exists rather than a preference.
//!
//! # What byte equality here does and does not establish
//!
//! Every output-affecting input is supposed to be represented in identity or
//! provenance. Agreement across two processes is evidence for that on this
//! host, with this toolchain, for this program. It is not a portability claim:
//! two hosts, two SDK versions, or two `metallib` linkers are a different
//! measurement, and `metallib_byte_reproducibility_is_measured_and_recorded`
//! in the producer records what is known about the last of those.

use std::path::{Path, PathBuf};
use std::process::Command;

use tiler_artifact::proof::decode_proof_sidecar;

/// The proof matrix this producer publishes, named exactly as it names them.
///
/// Duplicated from the producer rather than imported for the reason
/// [`sidecar_path`] gives: `serial-sum-compile` is a `[[bin]]` with no library
/// target, so this test links nothing from it. That duplication is why the
/// count is asserted below — a test that reads whatever it finds cannot tell a
/// producer that published six members from one that published one.
const REDUCTION_CLASSES: [&str; 3] = ["empty-domain", "singleton", "nontrivial"];
const PLAN_ROLES: [&str; 2] = ["selected", "materialized"];

/// One published pair, read back from disk exactly as a consumer would find it.
struct Published {
    member: String,
    envelope: Vec<u8>,
    sidecar: Vec<u8>,
}

#[test]
fn two_producer_processes_agree_on_envelope_sidecar_and_identity() {
    let first = produce("first");
    let second = produce("second");

    // Named and counted, not merely zipped. A `zip` over two short lists is
    // vacuously true, and this test's whole subject is a matrix whose size is
    // the claim: six members, one per reduction class per plan role.
    let expected = REDUCTION_CLASSES.len() * PLAN_ROLES.len();
    assert_eq!(
        first.len(),
        expected,
        "the producer published {} members, not the {expected} the proof matrix declares",
        first.len(),
    );
    assert_eq!(
        second.len(),
        expected,
        "the two runs published different counts"
    );

    for (first, second) in first.iter().zip(&second) {
        assert_eq!(
            first.member, second.member,
            "the two runs published different members"
        );
        let member = &first.member;
        assert_eq!(
            first.envelope, second.envelope,
            "{member}: envelope bytes vary between two producer processes",
        );
        assert_eq!(
            first.sidecar, second.sidecar,
            "{member}: sidecar bytes vary between two producer processes",
        );

        // Asserted from the decoded record rather than inferred from the byte
        // equality above. The identity is the field a consumer takes as
        // normative, and reading it back is what proves the equal bytes carry an
        // equal claim rather than merely being equal bytes.
        let first_identity = decode_proof_sidecar(&first.sidecar)
            .expect("the first run's sidecar decodes")
            .artifact_identity_bytes()
            .to_vec();
        let second_identity = decode_proof_sidecar(&second.sidecar)
            .expect("the second run's sidecar decodes")
            .artifact_identity_bytes()
            .to_vec();
        assert_eq!(
            first_identity, second_identity,
            "{member}: artifact identity varies between two producer processes",
        );

        // The published pair still binds after a round trip through the
        // filesystem, and it binds to *this* run's envelope. A producer that
        // wrote a sidecar describing the other run's bytes would satisfy every
        // equality above.
        decode_proof_sidecar(&second.sidecar)
            .expect("the second run's sidecar decodes")
            .bind_to_envelope(&second.envelope)
            .expect("the sidecar names the envelope published beside it");
    }

    // Distinct members must not be byte-equal. Every assertion above compares
    // one run against the other, so a producer that wrote the same program six
    // times under six names would pass all of them.
    for window in first.windows(2) {
        assert_ne!(
            window[0].envelope, window[1].envelope,
            "{} and {} published identical envelopes; the matrix is not covering distinct programs",
            window[0].member, window[1].member,
        );
    }
}

/// Runs the real producer once into its own directory and reads both files.
///
/// A directory per run rather than a filename per run: the envelope path is an
/// input to nothing the producer derives, and giving each run the same file
/// name under a different parent keeps it that way instead of quietly making
/// the two invocations differ in an argument.
fn produce(label: &str) -> Vec<Published> {
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(label);
    // Removed rather than merely created. A leftover file from an earlier run
    // satisfies a `read` that the current producer never wrote, which is exactly
    // how this test passed in a reused `target/` while failing in every fresh
    // checkout after the producer moved to suffixed member names.
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("the run directory is created");
    let base = directory.join("serial-sum.tiler");

    let output = Command::new(env!("CARGO_BIN_EXE_tiler-prototype-compile"))
        .arg("--out")
        .arg(&base)
        .output()
        .expect("the producer binary runs");
    assert!(
        output.status.success(),
        "the producer failed ({}):\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr),
    );

    let mut published = Vec::new();
    for class in REDUCTION_CLASSES {
        for role in PLAN_ROLES {
            let envelope = member_path(&base, class, role);
            let member = Published {
                member: format!("{class}.{role}"),
                envelope: read(&envelope),
                sidecar: read(&sidecar_path(&envelope)),
            };
            // Two empty files are equal to each other. Every assertion in this
            // file compares one run's bytes against another's, and none of them
            // can tell a producer that published nothing from one that published
            // the same thing twice, so the non-vacuity is stated here rather
            // than assumed there.
            assert!(
                !member.envelope.is_empty() && !member.sidecar.is_empty(),
                "the {label} run published an empty file for {}",
                member.member,
            );
            published.push(member);
        }
    }
    published
}

/// The envelope path this producer writes for one member of the proof matrix.
///
/// Derived here for the same reason [`sidecar_path`] is: nothing links from the
/// producer binary, so both sides pin the derivation independently and the
/// producer's own `proof_member` is the other half.
fn member_path(base: &Path, class: &str, role: &str) -> PathBuf {
    let mut name = base.as_os_str().to_owned();
    name.push(format!(".{class}.{role}"));
    PathBuf::from(name)
}

/// The sidecar path this producer writes beside an envelope.
///
/// Derived here rather than imported: `prototypes/serial-sum-compile` is a
/// binary with no library target, so this test links nothing from it and pins
/// the suffix the same way `prototypes/serial-sum-run` does. The producer's own
/// `the_sidecar_suffix_is_the_one_the_runner_opens` is the other half.
fn sidecar_path(envelope: &Path) -> PathBuf {
    let mut name = envelope.as_os_str().to_owned();
    name.push(".proof");
    PathBuf::from(name)
}

fn read(path: &Path) -> Vec<u8> {
    std::fs::read(path)
        .unwrap_or_else(|cause| panic!("{} could not be read: {cause}", path.display()))
}
