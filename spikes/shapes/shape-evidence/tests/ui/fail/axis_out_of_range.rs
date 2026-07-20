use shape_evidence_spike::{Axis, Builder, F32, Rank};

fn main() {
    let mut builder = Builder::new();
    let value = builder.input::<F32>([2, 3]);
    let ranked = builder.refine::<_, Rank<2>>(value).unwrap();
    let _ = builder.reduce_axis(ranked, Axis::<2>);
}

