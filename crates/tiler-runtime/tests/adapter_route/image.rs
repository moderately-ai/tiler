//! The scalar-host backend's executable representation, and the interpreter that runs it.
//!
//! # Why the fixture carries a real format rather than a marker
//!
//! [ADR 0090](../../../../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)
//! item 8 makes payload validation a backend obligation that the artifact layer
//! provably cannot discharge, on the ground that a payload's `code` bytes are
//! opaque to every check the artifact layer performs. A fixture whose payload
//! were a placeholder could not exhibit that: the obligation only becomes
//! visible when there are bytes that decode, bytes that do not, and an artifact
//! layer that accepts both.
//!
//! So `tiler.test.scalar-host-image-v1` is a real, versioned, domain-separated,
//! length-prefixed encoding with its own refusals — and the artifact carrying a
//! damaged one decodes, verifies, and re-derives the same canonical identity as
//! the artifact carrying a sound one, because artifact identity deliberately
//! excludes the emitted object's bytes.
//!
//! # The execution model, stated rather than borrowed from Metal
//!
//! One invocation at a time, in ascending grid index, on the calling thread.
//! There is no workgroup, no subgroup, and no concurrency. The reduction runs in
//! the original axis order the kernel's numerical realization forbids
//! reassociating, so the arithmetic does not depend on invocation order; what
//! does depend on it is only which store lands last where two invocations write
//! one element, and the schedule's write-ownership witness forbids that.

use std::fmt;

use tiler_runtime::load::RoutedEntry;

use tiler_artifact::program::{BindingTarget, BufferAccess};

/// Domain separator of this representation, matched exactly.
///
/// Sixteen bytes rather than a length-prefixed string: the separator is what
/// tells a foreign object from a damaged one, and it must be readable before
/// anything else in the buffer is trusted.
pub const IMAGE_DOMAIN: &[u8; 16] = b"tiler.scalar-img";

/// Schema version of the encoding this build writes and reads.
pub const IMAGE_SCHEMA: (u16, u16) = (1, 0);

/// Bytes of one `f32` element in this representation's storage encoding.
pub const ELEMENT_BYTES: u64 = 4;

/// One executable entry of a scalar image.
///
/// The transports are declared by the *image* and cross-checked against the
/// route rather than derived from it. That is the whole reason this fixture uses
/// a non-identity transport map: a backend that assumed `transport == slot`
/// would bind the right storage to the wrong index on any backend whose mapping
/// is not the identity, and the check that catches it can only be written where
/// both statements are readable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScalarEntry {
    /// The backend's own entry-point symbol, matched against the artifact's.
    pub symbol: String,
    /// Backend transport slot this entry reads its input through.
    pub read_transport: u32,
    /// Backend transport slot this entry writes its output through.
    pub write_transport: u32,
    /// Rows of the input, which is also the output element count.
    pub rows: u32,
    /// Columns of the input, which is the reduction extent.
    pub columns: u32,
    /// Bit pattern of the pointwise scale constant.
    pub scale_bits: u32,
    /// Bit pattern of the pointwise bias constant.
    pub bias_bits: u32,
}

/// One decoded scalar image.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScalarImage {
    /// Executable entries, in declaration order.
    pub entries: Vec<ScalarEntry>,
}

/// Encodes one scalar image into its transported bytes.
#[must_use]
pub fn encode(image: &ScalarImage) -> Vec<u8> {
    let mut bytes = IMAGE_DOMAIN.to_vec();
    bytes.extend_from_slice(&IMAGE_SCHEMA.0.to_le_bytes());
    bytes.extend_from_slice(&IMAGE_SCHEMA.1.to_le_bytes());
    let count = u32::try_from(image.entries.len()).expect("a fixture declares few entries");
    bytes.extend_from_slice(&count.to_le_bytes());
    for entry in &image.entries {
        let symbol = entry.symbol.as_bytes();
        let length = u32::try_from(symbol.len()).expect("a fixture symbol is short");
        bytes.extend_from_slice(&length.to_le_bytes());
        bytes.extend_from_slice(symbol);
        for value in [
            entry.read_transport,
            entry.write_transport,
            entry.rows,
            entry.columns,
            entry.scale_bits,
            entry.bias_bits,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    bytes
}

/// A cursor that refuses rather than panicking when the bytes run out.
struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], ScalarPayloadRefusal> {
        let end = self
            .position
            .checked_add(count)
            .ok_or(ScalarPayloadRefusal::Truncated {
                needed: count,
                available: 0,
            })?;
        if end > self.bytes.len() {
            return Err(ScalarPayloadRefusal::Truncated {
                needed: count,
                available: self.bytes.len() - self.position,
            });
        }
        let taken = &self.bytes[self.position..end];
        self.position = end;
        Ok(taken)
    }

    fn u16(&mut self) -> Result<u16, ScalarPayloadRefusal> {
        let bytes = self.take(2)?;
        Ok(u16::from_le_bytes(bytes.try_into().expect("two bytes")))
    }

    fn u32(&mut self) -> Result<u32, ScalarPayloadRefusal> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes(bytes.try_into().expect("four bytes")))
    }

    const fn remaining(&self) -> usize {
        self.bytes.len() - self.position
    }
}

/// Decodes one scalar image from a carried payload's opaque bytes.
///
/// # Errors
///
/// Returns the first [`ScalarPayloadRefusal`] the bytes provoke. Every one of
/// them is a check the artifact layer cannot perform, which is why this runs
/// here.
pub fn decode(bytes: &[u8]) -> Result<ScalarImage, ScalarPayloadRefusal> {
    let mut cursor = Cursor::new(bytes);
    let domain = cursor.take(IMAGE_DOMAIN.len())?;
    if domain != IMAGE_DOMAIN.as_slice() {
        return Err(ScalarPayloadRefusal::ForeignDomain {
            observed: domain.to_vec(),
        });
    }
    let major = cursor.u16()?;
    let minor = cursor.u16()?;
    if (major, minor) != IMAGE_SCHEMA {
        return Err(ScalarPayloadRefusal::UnsupportedSchema { major, minor });
    }
    let count = cursor.u32()?;
    let mut entries = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let length = cursor.u32()? as usize;
        let symbol = String::from_utf8(cursor.take(length)?.to_vec())
            .map_err(|_| ScalarPayloadRefusal::NonUtf8Symbol)?;
        entries.push(ScalarEntry {
            symbol,
            read_transport: cursor.u32()?,
            write_transport: cursor.u32()?,
            rows: cursor.u32()?,
            columns: cursor.u32()?,
            scale_bits: cursor.u32()?,
            bias_bits: cursor.u32()?,
        });
    }
    // Trailing bytes are refused rather than ignored. Ignoring them would admit
    // an object whose tail says something this build did not read, which is
    // indistinguishable from a later schema that this build must refuse.
    if cursor.remaining() != 0 {
        return Err(ScalarPayloadRefusal::TrailingBytes {
            extra: cursor.remaining(),
        });
    }
    Ok(ScalarImage { entries })
}

impl ScalarImage {
    /// Resolves and checks the image entry realizing one routed entry.
    ///
    /// This is the second half of the backend's payload obligation: decoding
    /// proves the bytes are *an* image, and this proves the image is the one the
    /// artifact's entry names and that it addresses the storage the route
    /// published. Both halves run while the preflight is still held.
    ///
    /// # Errors
    ///
    /// Returns the first [`ScalarPayloadRefusal`] the pairing provokes.
    pub fn entry_for(
        &self,
        routed: &RoutedEntry<'_>,
    ) -> Result<&ScalarEntry, ScalarPayloadRefusal> {
        let symbol = routed.entry_symbol();
        let entry = self
            .entries
            .iter()
            .find(|candidate| candidate.symbol == symbol)
            .ok_or_else(|| ScalarPayloadRefusal::SymbolAbsent {
                symbol: symbol.to_owned(),
            })?;

        let bindings = routed.bindings();
        let transports = u32::try_from(bindings.len()).expect("a fixture entry has few bindings");
        for (role, transport) in [
            ("read", entry.read_transport),
            ("write", entry.write_transport),
        ] {
            if transport >= transports {
                return Err(ScalarPayloadRefusal::TransportOutOfRange {
                    role,
                    transport,
                    bindings: bindings.len(),
                });
            }
        }

        // The route says which transport each ABI slot occupies and what access
        // mode that slot has. The image says which transport it reads and which
        // it writes. Agreement is checked rather than assumed, because an image
        // that wrote through the read binding would corrupt a caller's input and
        // return plausible values.
        let placement = |wanted: u32, access: BufferAccess, role: &'static str| {
            bindings
                .iter()
                .find(|binding| binding.transport_slot() == wanted)
                .filter(|binding| binding.binding().access() == access)
                .ok_or(ScalarPayloadRefusal::AccessModeMismatch {
                    role,
                    transport: wanted,
                })
        };
        let read = placement(entry.read_transport, BufferAccess::Read, "read")?;
        let write = placement(entry.write_transport, BufferAccess::Write, "write")?;

        // The image's own element arithmetic against the route's declared
        // accessible ranges. These are two independent statements — the route's
        // range is derived from the packaged program's ABI and the extents are
        // the image's — so a disagreement is a real artifact defect rather than
        // a restatement, and refusing here is what keeps the interpreter's own
        // bounds check unreachable in a sound route.
        let read_bytes = u64::from(entry.rows) * u64::from(entry.columns) * ELEMENT_BYTES;
        if read_bytes > read.accessible_bytes() {
            return Err(ScalarPayloadRefusal::UndersizedAccess {
                role: "read",
                declared: read_bytes,
                routed: read.accessible_bytes(),
            });
        }
        let write_bytes = u64::from(entry.rows) * ELEMENT_BYTES;
        if write_bytes > write.accessible_bytes() {
            return Err(ScalarPayloadRefusal::UndersizedAccess {
                role: "write",
                declared: write_bytes,
                routed: write.accessible_bytes(),
            });
        }

        Ok(entry)
    }
}

/// Why this backend will not execute a carried payload.
///
/// Every variant is a check the artifact layer performed no part of. Each is
/// reached before the routing commit, so each leaves a fallback permitted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScalarPayloadRefusal {
    /// The object's leading bytes are not this representation's separator.
    ForeignDomain {
        /// The bytes that were there instead.
        observed: Vec<u8>,
    },
    /// The object declares a schema this build does not read.
    UnsupportedSchema {
        /// Declared major version.
        major: u16,
        /// Declared minor version.
        minor: u16,
    },
    /// The object ended before a field it declared.
    Truncated {
        /// Bytes the next field needed.
        needed: usize,
        /// Bytes that remained.
        available: usize,
    },
    /// The object carried bytes after its last declared field.
    TrailingBytes {
        /// How many bytes were left over.
        extra: usize,
    },
    /// An entry symbol is not valid UTF-8.
    NonUtf8Symbol,
    /// The object declares no entry under the symbol the artifact names.
    SymbolAbsent {
        /// The symbol the routed entry published.
        symbol: String,
    },
    /// The object addresses a transport slot the entry does not have.
    TransportOutOfRange {
        /// Which access the out-of-range transport was declared for.
        role: &'static str,
        /// The transport slot the object declared.
        transport: u32,
        /// How many ABI bindings the routed entry declares.
        bindings: usize,
    },
    /// The object addresses a transport whose routed access mode is the other one.
    AccessModeMismatch {
        /// Which access the object declared for that transport.
        role: &'static str,
        /// The transport slot the object declared.
        transport: u32,
    },
    /// The object's own extents exceed the byte range the route published.
    UndersizedAccess {
        /// Which access disagreed.
        role: &'static str,
        /// Bytes the object's extents require.
        declared: u64,
        /// Bytes the route says are reachable.
        routed: u64,
    },
}

impl fmt::Display for ScalarPayloadRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ForeignDomain { observed } => write!(
                formatter,
                "scalar-host.payload.domain: the object opens with {observed:02x?} and this \
                 representation opens with {IMAGE_DOMAIN:02x?}",
            ),
            Self::UnsupportedSchema { major, minor } => write!(
                formatter,
                "scalar-host.payload.schema: the object declares {major}.{minor} and this build \
                 reads {}.{}",
                IMAGE_SCHEMA.0, IMAGE_SCHEMA.1,
            ),
            Self::Truncated { needed, available } => write!(
                formatter,
                "scalar-host.payload.truncated: a field needs {needed} byte(s) and {available} \
                 remain",
            ),
            Self::TrailingBytes { extra } => write!(
                formatter,
                "scalar-host.payload.trailing: {extra} byte(s) follow the last declared field",
            ),
            Self::NonUtf8Symbol => formatter.write_str(
                "scalar-host.payload.symbol-encoding: an entry symbol is not valid UTF-8",
            ),
            Self::SymbolAbsent { symbol } => write!(
                formatter,
                "scalar-host.payload.symbol: the object declares no entry named {symbol:?}",
            ),
            Self::TransportOutOfRange {
                role,
                transport,
                bindings,
            } => write!(
                formatter,
                "scalar-host.payload.transport: the object's {role} transport {transport} is \
                 outside the entry's {bindings} binding(s)",
            ),
            Self::AccessModeMismatch { role, transport } => write!(
                formatter,
                "scalar-host.payload.access-mode: the object declares transport {transport} as \
                 its {role} and the route does not",
            ),
            Self::UndersizedAccess {
                role,
                declared,
                routed,
            } => write!(
                formatter,
                "scalar-host.payload.range: the object's {role} extents need {declared} byte(s) \
                 and the route publishes {routed}",
            ),
        }
    }
}

impl std::error::Error for ScalarPayloadRefusal {}

/// One routed ABI slot resolved to host storage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Placement {
    /// Index of the host allocation backing this slot.
    pub allocation: usize,
    /// First addressed byte of the bound value within that allocation.
    pub offset: u64,
    /// Bytes the route requires be reachable from that offset.
    pub bytes: u64,
}

/// Why a committed dispatch did not complete.
///
/// Reached **after** the routing commit, so none of these is a fallback: each is
/// reported. Every one fails closed rather than leaving whatever the output
/// storage happened to hold.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionFault {
    /// An invocation addressed an element outside its placement.
    ///
    /// Unreachable through a sound route — [`ScalarImage::entry_for`] refuses an
    /// image whose extents exceed the published range before the commit — and
    /// retained as the check that makes a widened image format a refusal here
    /// rather than a read of whatever storage follows.
    OutOfRange {
        /// Which access escaped its range.
        role: &'static str,
        /// Byte the invocation addressed.
        byte: u64,
        /// Bytes the placement reaches.
        bytes: u64,
    },
    /// The run did not reach every invocation the route launched.
    Incomplete {
        /// Invocations that ran.
        executed: u64,
        /// Invocations the routed launch declared.
        expected: u64,
    },
}

impl fmt::Display for ExecutionFault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfRange { role, byte, bytes } => write!(
                formatter,
                "scalar-host.execute.range: a {role} addressed byte {byte} of a {bytes}-byte \
                 placement",
            ),
            Self::Incomplete { executed, expected } => write!(
                formatter,
                "scalar-host.execute.incomplete: {executed} of {expected} invocation(s) ran, and \
                 the routing commit forbids another route",
            ),
        }
    }
}

impl std::error::Error for ExecutionFault {}

/// Runs one scalar entry over its launch grid on the calling thread.
///
/// `halt_after` stops the run early. It exists so that the post-commit failure
/// path is reachable in a test at all: every condition this interpreter can
/// detect on its own is refused before the commit by design, which is correct
/// and would otherwise leave the reported-not-retried path untested. The Metal
/// proof synthesizes command-buffer statuses for the same reason.
///
/// # Errors
///
/// Returns the [`ExecutionFault`] the run produced. Nothing here is a fallback.
///
/// # Panics
///
/// Panics if a placement names an allocation that was not supplied.
/// [`crate::adapter::ScalarHostAdapter`] allocates from the route before the
/// commit, so a missing allocation is a defect in the adapter rather than a
/// route it should have refused.
pub fn execute(
    entry: &ScalarEntry,
    read: Placement,
    write: Placement,
    allocations: &mut [Vec<u8>],
    grid_threads: u64,
    halt_after: Option<u64>,
) -> Result<u64, ExecutionFault> {
    let scale = f32::from_bits(entry.scale_bits);
    let bias = f32::from_bits(entry.bias_bits);
    let columns = u64::from(entry.columns);

    // Copied out before the first store, so that a route binding one allocation
    // to both slots reads the operand rather than what this run has written.
    let source = {
        let allocation = &allocations[read.allocation];
        let start = usize::try_from(read.offset).expect("a fixture offset is small");
        let end = start + usize::try_from(read.bytes).expect("a fixture range is small");
        allocation[start..end].to_vec()
    };

    let mut executed = 0_u64;
    for row in 0..grid_threads {
        if halt_after.is_some_and(|limit| executed >= limit) {
            break;
        }
        let mut accumulator = 0.0_f32;
        for column in 0..columns {
            let element = row * columns + column;
            let byte = element * ELEMENT_BYTES;
            if byte + ELEMENT_BYTES > read.bytes {
                return Err(ExecutionFault::OutOfRange {
                    role: "read",
                    byte,
                    bytes: read.bytes,
                });
            }
            let start = usize::try_from(byte).expect("a fixture range is small");
            let operand = f32::from_bits(u32::from_le_bytes(
                source[start..start + 4].try_into().expect("four bytes"),
            ));
            // Serial, in ascending contributor order, with no reassociation:
            // the kernel's declared numerical realization forbids it, and this
            // is the whole of what makes the comparison against the reference a
            // result rather than a coincidence.
            let contribution = operand * scale + bias;
            accumulator = if column == 0 {
                contribution
            } else {
                accumulator + contribution
            };
        }
        let byte = row * ELEMENT_BYTES;
        if byte + ELEMENT_BYTES > write.bytes {
            return Err(ExecutionFault::OutOfRange {
                role: "write",
                byte,
                bytes: write.bytes,
            });
        }
        let start = usize::try_from(write.offset + byte).expect("a fixture range is small");
        allocations[write.allocation][start..start + 4]
            .copy_from_slice(&accumulator.to_bits().to_le_bytes());
        executed += 1;
    }
    Ok(executed)
}

/// Returns whether a routed binding addresses a named program input.
///
/// The one binding class whose storage the *caller* supplies. Everything else a
/// route names is storage the adapter allocates for itself, which is why the two
/// are separated at the planning stage rather than filled uniformly.
#[must_use]
pub const fn addresses_program_input(target: BindingTarget<'_>) -> bool {
    matches!(target, BindingTarget::ProgramInput(_))
}
