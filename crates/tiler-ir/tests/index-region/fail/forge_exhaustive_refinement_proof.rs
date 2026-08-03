use tiler_ir::index::IndexDomainProofEvidence;

fn main() {
    let _forged = IndexDomainProofEvidence::ExhaustiveFinite {
        points: 1,
        derivation: Box::from(&b"caller-authored"[..]),
    };
}
