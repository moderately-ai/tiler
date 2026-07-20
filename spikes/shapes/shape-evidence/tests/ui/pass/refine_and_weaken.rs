use shape_evidence_spike::{Builder, Exact, F32, Rank, StaticShapeSpec};

struct Matrix;

impl StaticShapeSpec for Matrix {
    const EXTENTS: &'static [u64] = &[2, 3];
}

fn main() {
    let mut builder = Builder::new();
    let value = builder.input::<F32>([2, 3]);
    let ranked = builder.refine::<_, Rank<2>>(value).unwrap();
    let exact = builder.refine::<_, Exact<Matrix>>(ranked.weaken()).unwrap();
    let _: shape_evidence_spike::Value<F32> = exact.weaken();
}

