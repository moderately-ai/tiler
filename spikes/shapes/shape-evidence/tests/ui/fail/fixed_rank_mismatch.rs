use shape_evidence_spike::{Builder, F32, Rank};

fn main() {
    let mut builder = Builder::new();
    let matrix = builder.input::<F32>([2, 3]);
    let vector = builder.input::<F32>([3]);
    let matrix = builder.refine::<_, Rank<2>>(matrix).unwrap();
    let vector = builder.refine::<_, Rank<1>>(vector).unwrap();
    let _ = builder.pointwise_shaped(matrix, vector);
}

