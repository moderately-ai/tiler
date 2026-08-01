//! `tiler.test.scalar-host-image-v1`: this backend's executable representation.
//!
//! # Why the payload is a real format rather than a marker
//!
//! [ADR 0090](../../../../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)
//! item 8 makes payload validation a backend obligation the artifact layer
//! provably cannot discharge, because a payload's `code` bytes are opaque to
//! every check that layer performs. A placeholder payload could not exhibit
//! that. So this is a real domain-separated, versioned, length-prefixed encoding
//! with its own refusals, and the artifact carrying a damaged image verifies,
//! encodes, decodes, and re-derives the *same* canonical identity as the
//! artifact carrying a sound one — which is exactly why the backend has to look.
//!
//! The transports are deliberately not the identity. Slot 0 is the read binding
//! and slot 1 the write binding, and the image places them at transports `[1,
//! 0]`. A backend that assumed a slot occupies the transport of the same number
//! would bind the input where the output goes and nothing else in the stack
//! would notice.

use std::fmt;

/// Domain separator of this representation, matched exactly before anything else.
pub const IMAGE_DOMAIN: &[u8; 16] = b"tiler.scalar-img";

/// Schema version this build writes and reads.
pub const IMAGE_SCHEMA: (u16, u16) = (1, 0);

/// One executable entry of a scalar image.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScalarEntry {
    /// This backend's own entry-point symbol.
    pub symbol: String,
    /// Backend transport slot each ABI binding occupies, in slot order.
    pub transports: Vec<u32>,
    /// Elements the entry's launch covers.
    pub work_items: u64,
}

/// One decoded scalar image.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScalarImage {
    /// Executable entries, in declaration order.
    pub entries: Vec<ScalarEntry>,
}

/// Why a byte run is not a scalar image this build can execute.
///
/// Exhaustive and ordered as the decoder checks them: a foreign object, a
/// version this build cannot read, and a damaged one are three different
/// findings with three different remedies, and collapsing them would leave a
/// host unable to say which it has.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScalarImageRefusal {
    /// The leading bytes are not this representation's domain separator.
    ForeignDomain,
    /// The declared schema is not one this build reads.
    UnsupportedSchema {
        /// Declared major version.
        major: u16,
        /// Declared minor version.
        minor: u16,
    },
    /// The run ended inside a field.
    Truncated,
    /// Bytes remain after the last declared entry.
    TrailingBytes,
    /// A declared symbol is empty or not UTF-8.
    MalformedSymbol,
    /// An entry declares no transport slot, so no binding could be placed.
    EmptyTransports,
    /// Two bindings of one entry claim the same transport slot.
    AliasedTransport,
}

impl fmt::Display for ScalarImageRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ForeignDomain => formatter.write_str("not a tiler.test scalar-host image"),
            Self::UnsupportedSchema { major, minor } => {
                write!(formatter, "scalar image schema {major}.{minor} is not read")
            }
            Self::Truncated => formatter.write_str("scalar image ended inside a field"),
            Self::TrailingBytes => formatter.write_str("scalar image carries trailing bytes"),
            Self::MalformedSymbol => formatter.write_str("scalar image entry symbol is malformed"),
            Self::EmptyTransports => formatter.write_str("scalar image entry places no binding"),
            Self::AliasedTransport => {
                formatter.write_str("scalar image entry aliases one transport slot")
            }
        }
    }
}

/// Encodes one scalar image into its transported bytes.
#[must_use]
pub fn encode(image: &ScalarImage) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(IMAGE_DOMAIN);
    bytes.extend_from_slice(&IMAGE_SCHEMA.0.to_be_bytes());
    bytes.extend_from_slice(&IMAGE_SCHEMA.1.to_be_bytes());
    bytes.extend_from_slice(
        &u32::try_from(image.entries.len())
            .expect("a small entry count")
            .to_be_bytes(),
    );
    for entry in &image.entries {
        let symbol = entry.symbol.as_bytes();
        bytes.extend_from_slice(
            &u32::try_from(symbol.len())
                .expect("a short symbol")
                .to_be_bytes(),
        );
        bytes.extend_from_slice(symbol);
        bytes.extend_from_slice(
            &u32::try_from(entry.transports.len())
                .expect("a small transport count")
                .to_be_bytes(),
        );
        for transport in &entry.transports {
            bytes.extend_from_slice(&transport.to_be_bytes());
        }
        bytes.extend_from_slice(&entry.work_items.to_be_bytes());
    }
    bytes
}

/// Decodes one scalar image from bytes, trusting nothing about their origin.
///
/// # Errors
///
/// Returns the first refusal in the order the decoder checks them.
pub fn decode(bytes: &[u8]) -> Result<ScalarImage, ScalarImageRefusal> {
    let mut reader = Reader { bytes, at: 0 };
    if reader.take(IMAGE_DOMAIN.len())? != IMAGE_DOMAIN.as_slice() {
        return Err(ScalarImageRefusal::ForeignDomain);
    }
    let major = reader.u16()?;
    let minor = reader.u16()?;
    if (major, minor) != IMAGE_SCHEMA {
        return Err(ScalarImageRefusal::UnsupportedSchema { major, minor });
    }
    let count = reader.u32()?;
    let mut entries = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let length = reader.u32()? as usize;
        let symbol = std::str::from_utf8(reader.take(length)?)
            .map_err(|_| ScalarImageRefusal::MalformedSymbol)?;
        if symbol.is_empty() {
            return Err(ScalarImageRefusal::MalformedSymbol);
        }
        let transport_count = reader.u32()? as usize;
        if transport_count == 0 {
            return Err(ScalarImageRefusal::EmptyTransports);
        }
        let mut transports = Vec::with_capacity(transport_count);
        for _ in 0..transport_count {
            transports.push(reader.u32()?);
        }
        let mut sorted = transports.clone();
        sorted.sort_unstable();
        sorted.dedup();
        if sorted.len() != transports.len() {
            return Err(ScalarImageRefusal::AliasedTransport);
        }
        entries.push(ScalarEntry {
            symbol: symbol.to_owned(),
            transports,
            work_items: reader.u64()?,
        });
    }
    if reader.at != bytes.len() {
        return Err(ScalarImageRefusal::TrailingBytes);
    }
    Ok(ScalarImage { entries })
}

struct Reader<'bytes> {
    bytes: &'bytes [u8],
    at: usize,
}

impl<'bytes> Reader<'bytes> {
    fn take(&mut self, length: usize) -> Result<&'bytes [u8], ScalarImageRefusal> {
        let end = self
            .at
            .checked_add(length)
            .ok_or(ScalarImageRefusal::Truncated)?;
        let run = self
            .bytes
            .get(self.at..end)
            .ok_or(ScalarImageRefusal::Truncated)?;
        self.at = end;
        Ok(run)
    }

    fn u16(&mut self) -> Result<u16, ScalarImageRefusal> {
        let run: [u8; 2] = self
            .take(2)?
            .try_into()
            .expect("the reader returned the requested width");
        Ok(u16::from_be_bytes(run))
    }

    fn u32(&mut self) -> Result<u32, ScalarImageRefusal> {
        let run: [u8; 4] = self
            .take(4)?
            .try_into()
            .expect("the reader returned the requested width");
        Ok(u32::from_be_bytes(run))
    }

    fn u64(&mut self) -> Result<u64, ScalarImageRefusal> {
        let run: [u8; 8] = self
            .take(8)?
            .try_into()
            .expect("the reader returned the requested width");
        Ok(u64::from_be_bytes(run))
    }
}
