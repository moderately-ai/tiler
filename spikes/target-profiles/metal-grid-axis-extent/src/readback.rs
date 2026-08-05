//! The one place this spike needs `unsafe`, and nothing else.
//!
//! Metal exposes no C API for buffer storage: `MTLBuffer`'s contents are
//! reachable only through the raw pointer `Buffer::contents` returns, so moving
//! results out of device memory cannot be expressed safely by any Rust binding.
//! Device creation, library loading, pipeline construction, encoding, dispatch,
//! and status inspection are all safe calls and stay in [`crate::main`].
//!
//! This crate denies `unsafe_code`, so this site opts in by name with its own
//! reason, following ADR 0079's four conditions and the precedent
//! `prototypes/serial-sum-run/src/buffer.rs` set for the identical constraint.
//!
//! **The invariant both functions rely on.** A buffer's length in bytes is fixed
//! at allocation, and `metal::Buffer::contents` returns a pointer valid for that
//! whole length while the buffer is alive. Each function asserts the byte length
//! it is about to touch against the buffer's own reported length, so a caller
//! that mismatched the two gets a panic before any pointer is dereferenced
//! rather than a silent access past the mapping.

use metal::Buffer;

/// Byte width of one `u32`.
const U32_BYTES: u64 = 4;

/// Fills the first `count` slots of `buffer` with `poison`.
///
/// Called before every dispatch so that "the invocation did not run" and "the
/// invocation wrote the expected value" are distinguishable observations. A
/// buffer left holding a previous rung's results would make a short dispatch
/// look complete.
///
/// # Panics
///
/// Panics when `buffer` is shorter than `count` requires.
#[expect(
    unsafe_code,
    reason = "MTLBuffer storage is reachable only through the raw pointer `Buffer::contents` returns; no Metal binding exposes it safely. The write is bounded by an asserted length check against the buffer's own reported byte length, writes a plain-old-data type with no destructor and no invalid bit patterns, and retains no borrow."
)]
pub fn poison(buffer: &Buffer, count: usize, poison: u32) {
    let required = u64::try_from(count).expect("an element count fits a u64") * U32_BYTES;
    assert!(
        buffer.length() >= required,
        "buffer holds {} bytes, the fill needs {required}",
        buffer.length(),
    );
    // SAFETY: `contents()` returns a pointer valid for `buffer.length()` bytes
    // for as long as `buffer` is alive, and `buffer` is borrowed for this call.
    // The assertion above proves the destination spans at least `required`
    // bytes. `u32` is `Copy`, has no invalid bit patterns and no destructor, so
    // writing into uninitialized Metal storage is well defined.
    unsafe {
        let start = buffer.contents().cast::<u32>();
        for index in 0..count {
            start.add(index).write(poison);
        }
    }
}

/// Copies `destination.len()` `u32` values out of `buffer`, starting at `offset`.
///
/// Range-addressed rather than whole-buffer, and reusing the caller's
/// allocation, so verifying the widest rung costs one chunk of host memory
/// instead of a second copy of a gigabyte-scale device buffer.
///
/// # Panics
///
/// Panics when `buffer` is shorter than the requested range, for the same reason
/// [`poison`] does.
#[expect(
    unsafe_code,
    reason = "the read half of the same constraint: MTLBuffer storage is reachable only through `Buffer::contents`. Bounded by an asserted length check against the buffer's own reported byte length, reads a plain-old-data type, and copies out rather than retaining a borrow of device memory."
)]
pub fn read_u32_into(buffer: &Buffer, offset: usize, destination: &mut [u32]) {
    let end = offset
        .checked_add(destination.len())
        .expect("the requested range does not overflow");
    let required = u64::try_from(end).expect("an element count fits a u64") * U32_BYTES;
    assert!(
        buffer.length() >= required,
        "buffer holds {} bytes, the read needs {required}",
        buffer.length(),
    );
    // SAFETY: as in `poison`, with the direction reversed. The assertion above
    // proves the source spans at least `offset + destination.len()` elements, so
    // the offset pointer and the whole copy stay inside one live allocation. The
    // destination is the caller's own slice, a distinct allocation from device
    // memory, so the two cannot overlap. `u32` is a plain-old-data type whose
    // every bit pattern is valid.
    unsafe {
        std::ptr::copy_nonoverlapping(
            buffer.contents().cast::<u32>().add(offset),
            destination.as_mut_ptr(),
            destination.len(),
        );
    }
}
