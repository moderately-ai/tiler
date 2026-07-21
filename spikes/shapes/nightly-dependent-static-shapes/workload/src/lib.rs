//! Host for generated dependent-static-shape compile workloads.

/// Forces one shape type through monomorphization without retaining a value.
#[inline(never)]
pub fn touch<T>() {
    std::hint::black_box(std::mem::size_of::<T>());
}
