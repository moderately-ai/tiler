//! The one place this proof needs `unsafe`, and nothing else.
//!
//! Metal exposes no C API: `MTLBuffer`'s storage is reached through a raw
//! pointer, so moving `f32` operands in and results out cannot be expressed
//! safely by any Rust binding. Device creation, library loading, pipeline
//! construction, encoding, and dispatch are all safe calls and stay in
//! [`crate::proof`].
//!
//! The crate denies `unsafe_code`, so every site that needs it opts in by name
//! with its own reason. Two functions do; they are the complete extent of
//! unsafe code in the runtime proof, and both are straight-line copies with no
//! branching, no arithmetic on the pointer beyond the length the caller states,
//! and no retained borrows.
//!
//! **The invariant both rely on.** A buffer's length in bytes is fixed at
//! allocation from the same element count the caller passes here, and
//! `metal::Buffer::contents` returns a pointer valid for that whole length
//! while the buffer is alive. Each function therefore asserts the byte length
//! it is about to touch against the buffer's own reported length, so a caller
//! that mismatched the two gets a panic before any pointer is dereferenced
//! rather than a silent out-of-bounds copy.

use metal::Buffer;

/// Byte width of one `f32`.
const F32_BYTES: u64 = 4;

/// Writes `values` into `buffer`'s storage.
///
/// # Panics
///
/// Panics when `buffer` is shorter than `values` requires. The check is
/// deliberate: it converts a caller's allocation mistake into an immediate,
/// attributable failure instead of a write past the mapping.
#[allow(
    unsafe_code,
    reason = "MTLBuffer storage is reachable only through the raw pointer `Buffer::contents` returns; no Metal binding exposes it safely. The write is bounded by an asserted length check against the buffer's own byte length, copies a plain-old-data type with no destructor, and retains no borrow."
)]
pub fn write_f32(buffer: &Buffer, values: &[f32]) {
    let required = u64::try_from(values.len()).expect("a slice length fits a u64") * F32_BYTES;
    assert!(
        buffer.length() >= required,
        "buffer holds {} bytes, the write needs {required}",
        buffer.length(),
    );
    // SAFETY: `contents()` returns a pointer valid for `buffer.length()` bytes
    // for as long as `buffer` is alive, and `buffer` is borrowed for this call.
    // The assertion above proves the destination spans at least `required`
    // bytes. `f32` is `Copy` with no invalid bit patterns and no destructor, so
    // a byte copy into uninitialized Metal storage is well defined. Source and
    // destination are distinct allocations, so they cannot overlap.
    unsafe {
        std::ptr::copy_nonoverlapping(
            values.as_ptr(),
            buffer.contents().cast::<f32>(),
            values.len(),
        );
    }
}

/// Reads `count` `f32` values out of `buffer`'s storage.
///
/// # Panics
///
/// Panics when `buffer` is shorter than `count` requires, for the same reason
/// [`write_f32`] does.
#[allow(
    unsafe_code,
    reason = "the read half of the same constraint: MTLBuffer storage is reachable only through `Buffer::contents`. Bounded by an asserted length check, reads a plain-old-data type, and copies out rather than retaining a borrow of device memory."
)]
#[must_use]
pub fn read_f32(buffer: &Buffer, count: usize) -> Vec<f32> {
    let required = u64::try_from(count).expect("an element count fits a u64") * F32_BYTES;
    assert!(
        buffer.length() >= required,
        "buffer holds {} bytes, the read needs {required}",
        buffer.length(),
    );
    let mut values = vec![0.0_f32; count];
    // SAFETY: as in `write_f32`, with the direction reversed. The source spans
    // at least `required` bytes by the assertion, the destination is a freshly
    // allocated `Vec` of exactly `count` elements, and the two are distinct
    // allocations. The GPU write that produced these bytes is ordered before
    // this read by the caller's `wait_until_completed`.
    unsafe {
        std::ptr::copy_nonoverlapping(buffer.contents().cast::<f32>(), values.as_mut_ptr(), count);
    }
    values
}
