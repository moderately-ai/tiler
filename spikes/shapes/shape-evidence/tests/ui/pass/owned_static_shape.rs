use shape_evidence_spike::{Builder, F32, ShapedValue, static_shape};

fn main() {
    let mut builder = Builder::new();
    let value = builder.input::<F32>([2, 3]);
    let _: ShapedValue<F32, static_shape!(2, 3)> = builder.refine(value).unwrap();
}
