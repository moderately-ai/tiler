use shape_evidence_spike::{F32, Rank, ShapedValue, Value};

fn main() {
    let value = Value::<F32> {
        graph: 1,
        index: 0,
        marker: std::marker::PhantomData,
    };
    let _ = ShapedValue::<F32, Rank<2>> {
        value,
        evidence: std::marker::PhantomData,
    };
}

