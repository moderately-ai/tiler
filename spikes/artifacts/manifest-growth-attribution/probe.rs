//! Prints one zero-object artifact envelope's fixed content, attributed.
//!
//! Copied into an extracted worktree by `probe.sh` as a second binary of
//! `spikes/cache/hot-path-efficiency/harness`, whose `envelope.rs` module it
//! reaches by path. That harness is the fixture because its envelope carries
//! **zero** object bytes, so its whole length is fixed content and no part of
//! any figure below is a compiled object.
//!
//! Every quantity is read through the public
//! [`tiler_artifact::program::decode_artifact`] view rather than by parsing the
//! wire, so a framing change between two probed commits cannot silently be
//! attributed to content. The manifest length is the one derived column:
//! `total - 69 - sum(section bytes + 12)`, which is the two-end parse
//! `docs/research/cache/hot-path-efficiency.md` Section 9.1 states.

#[path = "../envelope.rs"]
mod envelope;

fn main() {
    let factory = envelope::EnvelopeFactory::new();
    let bytes = factory.exactly(factory.base_bytes());
    let total = bytes.len();
    let decoded =
        tiler_artifact::program::decode_artifact(&bytes).expect("the fixture envelope decodes");
    let mut framed = 0usize;
    let mut parts: Vec<String> = Vec::new();
    for section in decoded.sections() {
        framed += section.bytes().len() + 12;
        parts.push(format!("{:?}={}", section.purpose(), section.bytes().len()));
    }
    // Re-derived from the decoded content rather than read off the wire, which
    // is what `DecodedArtifact::identity` documents. The manifest's own trailing
    // run is byte-equal to it, because a decode that reached this point already
    // proved the two agree.
    let identity = decoded.identity().as_bytes().len();
    println!(
        "PROBE\ttotal={total}\tmanifest={}\tidentity={identity}\tvariants={}\tpayloads={}\tsections={}",
        total - 69 - framed,
        decoded.variant_count(),
        decoded.payloads().len(),
        parts.join(","),
    );
}
