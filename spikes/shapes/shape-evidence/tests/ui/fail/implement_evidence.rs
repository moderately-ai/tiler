use shape_evidence_spike::ShapeEvidence;

struct Forged;

impl ShapeEvidence for Forged {
    fn matches(_: &[u64]) -> bool {
        true
    }
}

fn main() {}

