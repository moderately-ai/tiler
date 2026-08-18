//! The only module in this spike that may contain `unsafe`: **two sites**,
//! the operand write and the result read, following the shape and the ADR 0079
//! reasoning of `crates/tiler-conformance/src/device_buffer.rs` — Metal exposes
//! buffer storage only through the raw pointer `Buffer::contents` returns, and
//! no binding in this workspace exposes it safely.

use metal::Buffer;

/// Writes `bytes` into the front of `buffer`'s storage.
///
/// # Panics
///
/// Panics when `buffer` is shorter than `bytes`, so an allocation mistake is an
/// attributable failure rather than a write past the mapping.
#[allow(
    unsafe_code,
    reason = "MTLBuffer storage is reachable only through the raw pointer `Buffer::contents` returns; the write is bounded by an asserted length check against the buffer's own byte length, copies bytes — no alignment requirement, no invalid bit pattern, no destructor — and retains no borrow"
)]
pub fn write_bytes(buffer: &Buffer, bytes: &[u8]) {
    let required = u64::try_from(bytes.len()).expect("a slice length fits a u64");
    assert!(
        buffer.length() >= required,
        "buffer holds {} bytes, the write needs {required}",
        buffer.length(),
    );
    // SAFETY: `contents()` is valid for `buffer.length()` bytes while `buffer`
    // is alive and borrowed for this call; the assertion above proves the
    // destination spans at least `required` bytes; source and destination are
    // distinct allocations, so they cannot overlap.
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer.contents().cast::<u8>(), bytes.len());
    }
}

/// Reads the first `len` bytes of `buffer`'s storage.
///
/// # Panics
///
/// Panics when `buffer` is shorter than `len`, for the same reason
/// [`write_bytes`] does.
#[allow(
    unsafe_code,
    reason = "the read half of the same constraint: bounded by an asserted length check, reads bytes, and copies out rather than retaining a borrow of device memory; the GPU write is ordered before this read by the caller's wait_until_completed"
)]
#[must_use]
pub fn read_bytes(buffer: &Buffer, len: usize) -> Vec<u8> {
    let required = u64::try_from(len).expect("a byte length fits a u64");
    assert!(
        buffer.length() >= required,
        "buffer holds {} bytes, the read needs {required}",
        buffer.length(),
    );
    let mut bytes = vec![0_u8; len];
    // SAFETY: as in `write_bytes`, direction reversed; the source spans at
    // least `required` bytes by the assertion and the destination is a fresh
    // `Vec` of exactly `len` elements in a distinct allocation.
    unsafe {
        std::ptr::copy_nonoverlapping(buffer.contents().cast::<u8>(), bytes.as_mut_ptr(), len);
    }
    bytes
}
