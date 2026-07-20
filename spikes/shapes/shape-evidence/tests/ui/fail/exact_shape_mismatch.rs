use shape_evidence_spike::{Builder, Exact, F32, StaticShapeSpec};

struct TwoByThree;
impl StaticShapeSpec for TwoByThree {
    const EXTENTS: &'static [u64] = &[2, 3];
}

struct ThreeByTwo;
impl StaticShapeSpec for ThreeByTwo {
    const EXTENTS: &'static [u64] = &[3, 2];
}

fn main() {
    let mut builder = Builder::new();
    let left = builder.input::<F32>([2, 3]);
    let right = builder.input::<F32>([3, 2]);
    let left = builder.refine::<_, Exact<TwoByThree>>(left).unwrap();
    let right = builder.refine::<_, Exact<ThreeByTwo>>(right).unwrap();
    let _ = builder.pointwise_shaped(left, right);
}

