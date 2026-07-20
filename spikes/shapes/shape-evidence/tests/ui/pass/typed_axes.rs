use shape_evidence_spike::{Axes2, Builder, F32, Rank};

fn main() {
    let mut builder = Builder::new();
    let value = builder.input::<F32>([2, 3, 4]);
    let ranked = builder.refine::<_, Rank<3>>(value).unwrap();
    let _ = builder.reduce_axes2(ranked, Axes2::<0, 2>).unwrap();
}

