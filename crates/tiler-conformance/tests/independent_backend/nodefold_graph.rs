//! `tiler.test.nodefold-graph-v1`: this backend's executable representation.
//!
//! # Why this is a node table and not an instruction tape
//!
//! The seam this fixture exercises fixes nothing about a payload's inner shape:
//! `assemble_plan_artifact` treats `code` as opaque bytes, and ADR 0090 item 8
//! makes validating them the backend's own obligation because no check the
//! artifact layer performs can reach them. So the inner shape is a place the
//! seam left a choice, and this backend takes the other one.
//!
//! The design here is a **flat, single-assignment node table with a declarative
//! store plan**. A node's ordinal *is* its result, so there is no destination
//! field, no slot file, and no register allocation; every operand is an earlier
//! ordinal, which the decoder checks, so the table is acyclic and evaluable in
//! one forward pass with no mutable cell that could be read before it is
//! written. Predication is not an instruction: it is one optional guard ordinal
//! on the single store plan, so a decoded entry has exactly one write site by
//! construction rather than by a rule someone has to enforce.
//!
//! That produces validation obligations an instruction tape does not have —
//! [`GraphRefusal::ForwardReference`] and [`GraphRefusal::OperandTypeDisagreement`]
//! are decode-time structural findings here, where a tape discovers the same
//! defects at execution time as an unset or wrongly typed slot, if at all. It
//! also loses expressiveness this fixture does not need and says so: several
//! stores, nested predication, and loops are all outside the vocabulary and are
//! refused by name rather than partially executed.
//!
//! # Little-endian, and why that is not a coin flip
//!
//! The framing is little-endian throughout. A representation's byte order is
//! the backend's to choose and nothing above it reads these bytes, so writing
//! them in the other order from the two payload formats already in the tree is
//! a cheap, checkable demonstration that no reader outside this backend has
//! learned to depend on one.

use std::fmt;

/// Domain separator, matched exactly before any other byte is interpreted.
pub(crate) const GRAPH_DOMAIN: &[u8; 24] = b"tiler.test.nodefold-v1\r\n";

/// Schema version this build writes and is the only one it reads.
pub(crate) const GRAPH_SCHEMA: (u16, u16) = (1, 0);

/// What one node of the table computes.
///
/// Every operand is the ordinal of an earlier node, which is what makes the
/// table a directed acyclic graph in topological order rather than a program
/// with control flow. The variants carry no destination because a node's own
/// ordinal is its result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Node {
    /// The launch's global invocation index, as an index value.
    InvocationIndex,
    /// A literal index value.
    IndexConstant(u64),
    /// A literal `f32`, by bit pattern.
    F32Constant(u32),
    /// Sum of two index nodes.
    IndexAdd(u32, u32),
    /// Product of two index nodes.
    IndexMultiply(u32, u32),
    /// Whether the first index node orders strictly below the second.
    IndexLessThan(u32, u32),
    /// Product of two `f32` nodes, canonicalized on NaN.
    F32Multiply(u32, u32),
    /// Sum of two `f32` nodes, canonicalized on NaN.
    F32Add(u32, u32),
    /// The canonical NaN projection of one `f32` node.
    CanonicalizeF32Nan(u32),
    /// One element read from a declared buffer at an index node's offset.
    Load {
        /// Position of the buffer in the entry's signature order.
        buffer: u32,
        /// Ordinal of the index node giving the element offset.
        offset: u32,
    },
}

/// The value kind a node produces.
///
/// Derived during decode and never transported: a producer that stated it could
/// state it wrongly, and the decoder can compute it from the node table alone.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NodeKind {
    /// An unsigned index.
    Index,
    /// A single-precision float, held as bits.
    F32,
    /// An ordering verdict.
    Bool,
}

/// The single write one entry performs, stated rather than encoded as an instruction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StorePlan {
    /// Ordinal of the boolean node that must hold, when the write is guarded.
    pub(crate) guard: Option<u32>,
    /// Position of the written buffer in the entry's signature order.
    pub(crate) buffer: u32,
    /// Ordinal of the index node giving the element offset.
    pub(crate) offset: u32,
    /// Ordinal of the `f32` node giving the stored value.
    pub(crate) value: u32,
}

/// One buffer of an entry's signature, in signature order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GraphBuffer {
    /// Whether this backend writes through the buffer.
    pub(crate) write: bool,
    /// Elements the entry may address through it.
    pub(crate) element_count: u64,
}

/// One executable entry of a nodefold graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GraphEntry {
    /// This backend's own entry-point symbol.
    pub(crate) symbol: String,
    /// The canonical NaN bit pattern this entry's arithmetic projects onto.
    pub(crate) canonical_nan: u32,
    /// Buffers the entry binds, in signature order.
    pub(crate) buffers: Vec<GraphBuffer>,
    /// The node table, in topological order.
    pub(crate) nodes: Vec<Node>,
    /// The one write this entry performs.
    pub(crate) store: StorePlan,
}

/// A decoded nodefold graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Graph {
    /// Executable entries, in declaration order.
    pub(crate) entries: Vec<GraphEntry>,
}

impl Graph {
    /// Returns the entry a payload mapping's symbol names.
    pub(crate) fn entry_for(&self, symbol: &str) -> Option<&GraphEntry> {
        self.entries.iter().find(|entry| entry.symbol == symbol)
    }
}

/// Why a byte run is not a nodefold graph this build executes.
///
/// Ordered as the decoder reaches them. A foreign object, an unread schema, a
/// damaged run, and a structurally impossible table are four findings with four
/// remedies, and a backend that collapsed them would leave a host unable to say
/// which it holds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GraphRefusal {
    /// The leading bytes are not this representation's domain separator.
    ForeignDomain,
    /// The declared schema is not the one this build reads.
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
    /// A node tag is not one this schema names.
    UnknownNodeTag(u8),
    /// An operand names its own node or a later one.
    ///
    /// The structural invariant of this representation: an operand must be an
    /// earlier ordinal. Without it the table is not evaluable in one pass, and
    /// a graph carrying a cycle would be discovered — if at all — as a read of
    /// a value nothing has computed.
    ///
    /// **`node` is the reading site, and for the store plan that site is one
    /// past the last ordinal.** The store plan is evaluated after every node
    /// and has no ordinal of its own, so reporting it as `nodes.len()` says
    /// "after the table" in the same coordinate the rest of the message uses. A
    /// reader who looks up that ordinal will not find a node, which is the
    /// correct answer rather than a rotted one.
    ForwardReference {
        /// The node whose operand reaches forward.
        node: u32,
        /// The operand ordinal it named.
        operand: u32,
    },
    /// An operand's node produces a kind the consuming node cannot take.
    ///
    /// `node` follows the convention [`Self::ForwardReference`] states: the
    /// store plan is reported at the ordinal one past the table.
    OperandTypeDisagreement {
        /// The node whose operand disagrees.
        node: u32,
        /// The kind the consuming node requires.
        required: NodeKind,
        /// The kind the operand node produces.
        found: NodeKind,
    },
    /// The store plan names a buffer the signature does not declare.
    UndeclaredStoreBuffer(u32),
    /// A load names a buffer the signature does not declare.
    UndeclaredLoadBuffer(u32),
    /// An entry declares no buffer, so no binding could be placed.
    EmptySignature,
    /// The store plan names a buffer this entry does not write through.
    StoreThroughReadBuffer(u32),
}

impl fmt::Display for GraphRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ForeignDomain => formatter.write_str("not a tiler.test nodefold graph"),
            Self::UnsupportedSchema { major, minor } => {
                write!(formatter, "nodefold schema {major}.{minor} is not read")
            }
            Self::Truncated => formatter.write_str("nodefold graph ended inside a field"),
            Self::TrailingBytes => formatter.write_str("nodefold graph carries trailing bytes"),
            Self::MalformedSymbol => formatter.write_str("nodefold entry symbol is malformed"),
            Self::UnknownNodeTag(tag) => {
                write!(formatter, "nodefold node tag 0x{tag:02x} is not named")
            }
            Self::ForwardReference { node, operand } => write!(
                formatter,
                "nodefold node {node} reads operand {operand}, which is not an earlier node",
            ),
            Self::OperandTypeDisagreement {
                node,
                required,
                found,
            } => write!(
                formatter,
                "nodefold node {node} requires a {required:?} operand and reads a {found:?} one",
            ),
            Self::UndeclaredStoreBuffer(buffer) => write!(
                formatter,
                "nodefold store plan names buffer {buffer}, which the signature does not declare",
            ),
            Self::UndeclaredLoadBuffer(buffer) => write!(
                formatter,
                "nodefold load names buffer {buffer}, which the signature does not declare",
            ),
            Self::EmptySignature => formatter.write_str("nodefold entry declares no buffer"),
            Self::StoreThroughReadBuffer(buffer) => write!(
                formatter,
                "nodefold store plan writes through read-only buffer {buffer}",
            ),
        }
    }
}

impl std::error::Error for GraphRefusal {}

/// Encodes one graph into its transported bytes.
pub(crate) fn encode(graph: &Graph) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(GRAPH_DOMAIN);
    bytes.extend_from_slice(&GRAPH_SCHEMA.0.to_le_bytes());
    bytes.extend_from_slice(&GRAPH_SCHEMA.1.to_le_bytes());
    put_u32(&mut bytes, count(graph.entries.len()));
    for entry in &graph.entries {
        let symbol = entry.symbol.as_bytes();
        put_u32(&mut bytes, count(symbol.len()));
        bytes.extend_from_slice(symbol);
        put_u32(&mut bytes, entry.canonical_nan);
        put_u32(&mut bytes, count(entry.buffers.len()));
        for buffer in &entry.buffers {
            bytes.push(u8::from(buffer.write));
            put_u64(&mut bytes, buffer.element_count);
        }
        put_u32(&mut bytes, count(entry.nodes.len()));
        for node in &entry.nodes {
            put_node(&mut bytes, *node);
        }
        match entry.store.guard {
            None => bytes.push(0),
            Some(guard) => {
                bytes.push(1);
                put_u32(&mut bytes, guard);
            }
        }
        put_u32(&mut bytes, entry.store.buffer);
        put_u32(&mut bytes, entry.store.offset);
        put_u32(&mut bytes, entry.store.value);
    }
    bytes
}

/// Decodes one graph from bytes, trusting nothing about their origin.
///
/// The kind table is rebuilt here rather than transported, and every operand is
/// checked against it, so a graph that decodes is one this backend can evaluate
/// in a single forward pass without a runtime type test.
pub(crate) fn decode(bytes: &[u8]) -> Result<Graph, GraphRefusal> {
    let mut reader = Reader { bytes, at: 0 };
    if reader.take(GRAPH_DOMAIN.len())? != GRAPH_DOMAIN.as_slice() {
        return Err(GraphRefusal::ForeignDomain);
    }
    let major = reader.u16()?;
    let minor = reader.u16()?;
    if (major, minor) != GRAPH_SCHEMA {
        return Err(GraphRefusal::UnsupportedSchema { major, minor });
    }
    let entries = reader.u32()?;
    let mut decoded = Vec::with_capacity(entries as usize);
    for _ in 0..entries {
        decoded.push(decode_entry(&mut reader)?);
    }
    if reader.at != bytes.len() {
        return Err(GraphRefusal::TrailingBytes);
    }
    Ok(Graph { entries: decoded })
}

fn decode_entry(reader: &mut Reader<'_>) -> Result<GraphEntry, GraphRefusal> {
    let length = reader.u32()? as usize;
    let symbol =
        std::str::from_utf8(reader.take(length)?).map_err(|_| GraphRefusal::MalformedSymbol)?;
    if symbol.is_empty() {
        return Err(GraphRefusal::MalformedSymbol);
    }
    let symbol = symbol.to_owned();
    let canonical_nan = reader.u32()?;
    let buffer_count = reader.u32()?;
    if buffer_count == 0 {
        return Err(GraphRefusal::EmptySignature);
    }
    let mut buffers = Vec::with_capacity(buffer_count as usize);
    for _ in 0..buffer_count {
        let write = reader.u8()? != 0;
        buffers.push(GraphBuffer {
            write,
            element_count: reader.u64()?,
        });
    }
    let node_count = reader.u32()?;
    let mut nodes = Vec::with_capacity(node_count as usize);
    let mut kinds: Vec<NodeKind> = Vec::with_capacity(node_count as usize);
    for ordinal in 0..node_count {
        let node = decode_node(reader)?;
        kinds.push(check_node(ordinal, node, &kinds, &buffers)?);
        nodes.push(node);
    }
    let guard = if reader.u8()? == 0 {
        None
    } else {
        Some(reader.u32()?)
    };
    let store = StorePlan {
        guard,
        buffer: reader.u32()?,
        offset: reader.u32()?,
        value: reader.u32()?,
    };
    check_store(node_count, store, &kinds, &buffers)?;
    Ok(GraphEntry {
        symbol,
        canonical_nan,
        buffers,
        nodes,
        store,
    })
}

fn decode_node(reader: &mut Reader<'_>) -> Result<Node, GraphRefusal> {
    Ok(match reader.u8()? {
        0x10 => Node::InvocationIndex,
        0x11 => Node::IndexConstant(reader.u64()?),
        0x12 => Node::F32Constant(reader.u32()?),
        0x13 => Node::IndexAdd(reader.u32()?, reader.u32()?),
        0x14 => Node::IndexMultiply(reader.u32()?, reader.u32()?),
        0x15 => Node::IndexLessThan(reader.u32()?, reader.u32()?),
        0x16 => Node::F32Multiply(reader.u32()?, reader.u32()?),
        0x17 => Node::F32Add(reader.u32()?, reader.u32()?),
        0x18 => Node::CanonicalizeF32Nan(reader.u32()?),
        0x19 => Node::Load {
            buffer: reader.u32()?,
            offset: reader.u32()?,
        },
        tag => return Err(GraphRefusal::UnknownNodeTag(tag)),
    })
}

fn put_node(bytes: &mut Vec<u8>, node: Node) {
    match node {
        Node::InvocationIndex => bytes.push(0x10),
        Node::IndexConstant(value) => {
            bytes.push(0x11);
            put_u64(bytes, value);
        }
        Node::F32Constant(bits) => {
            bytes.push(0x12);
            put_u32(bytes, bits);
        }
        Node::IndexAdd(lhs, rhs) => {
            bytes.push(0x13);
            put_u32(bytes, lhs);
            put_u32(bytes, rhs);
        }
        Node::IndexMultiply(lhs, rhs) => {
            bytes.push(0x14);
            put_u32(bytes, lhs);
            put_u32(bytes, rhs);
        }
        Node::IndexLessThan(lhs, rhs) => {
            bytes.push(0x15);
            put_u32(bytes, lhs);
            put_u32(bytes, rhs);
        }
        Node::F32Multiply(lhs, rhs) => {
            bytes.push(0x16);
            put_u32(bytes, lhs);
            put_u32(bytes, rhs);
        }
        Node::F32Add(lhs, rhs) => {
            bytes.push(0x17);
            put_u32(bytes, lhs);
            put_u32(bytes, rhs);
        }
        Node::CanonicalizeF32Nan(source) => {
            bytes.push(0x18);
            put_u32(bytes, source);
        }
        Node::Load { buffer, offset } => {
            bytes.push(0x19);
            put_u32(bytes, buffer);
            put_u32(bytes, offset);
        }
    }
}

/// Checks one node's operands against the kinds already derived, and returns its own.
fn check_node(
    ordinal: u32,
    node: Node,
    kinds: &[NodeKind],
    buffers: &[GraphBuffer],
) -> Result<NodeKind, GraphRefusal> {
    let operand = |operand: u32, required: NodeKind| -> Result<(), GraphRefusal> {
        let found = *kinds
            .get(operand as usize)
            .ok_or(GraphRefusal::ForwardReference {
                node: ordinal,
                operand,
            })?;
        if found == required {
            Ok(())
        } else {
            Err(GraphRefusal::OperandTypeDisagreement {
                node: ordinal,
                required,
                found,
            })
        }
    };
    Ok(match node {
        Node::InvocationIndex | Node::IndexConstant(_) => NodeKind::Index,
        Node::F32Constant(_) => NodeKind::F32,
        Node::IndexAdd(lhs, rhs) | Node::IndexMultiply(lhs, rhs) => {
            operand(lhs, NodeKind::Index)?;
            operand(rhs, NodeKind::Index)?;
            NodeKind::Index
        }
        Node::IndexLessThan(lhs, rhs) => {
            operand(lhs, NodeKind::Index)?;
            operand(rhs, NodeKind::Index)?;
            NodeKind::Bool
        }
        Node::F32Multiply(lhs, rhs) | Node::F32Add(lhs, rhs) => {
            operand(lhs, NodeKind::F32)?;
            operand(rhs, NodeKind::F32)?;
            NodeKind::F32
        }
        Node::CanonicalizeF32Nan(source) => {
            operand(source, NodeKind::F32)?;
            NodeKind::F32
        }
        Node::Load { buffer, offset } => {
            if buffers.get(buffer as usize).is_none() {
                return Err(GraphRefusal::UndeclaredLoadBuffer(buffer));
            }
            operand(offset, NodeKind::Index)?;
            NodeKind::F32
        }
    })
}

/// Checks the store plan against the derived kinds and the declared signature.
fn check_store(
    node_count: u32,
    store: StorePlan,
    kinds: &[NodeKind],
    buffers: &[GraphBuffer],
) -> Result<(), GraphRefusal> {
    let operand = |operand: u32, required: NodeKind| -> Result<(), GraphRefusal> {
        let found = *kinds
            .get(operand as usize)
            .ok_or(GraphRefusal::ForwardReference {
                node: node_count,
                operand,
            })?;
        if found == required {
            Ok(())
        } else {
            Err(GraphRefusal::OperandTypeDisagreement {
                node: node_count,
                required,
                found,
            })
        }
    };
    if let Some(guard) = store.guard {
        operand(guard, NodeKind::Bool)?;
    }
    operand(store.offset, NodeKind::Index)?;
    operand(store.value, NodeKind::F32)?;
    let target = buffers
        .get(store.buffer as usize)
        .ok_or(GraphRefusal::UndeclaredStoreBuffer(store.buffer))?;
    if target.write {
        Ok(())
    } else {
        Err(GraphRefusal::StoreThroughReadBuffer(store.buffer))
    }
}

fn count(value: usize) -> u32 {
    u32::try_from(value).expect("a bounded nodefold population fits u32")
}

fn put_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn put_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

struct Reader<'bytes> {
    bytes: &'bytes [u8],
    at: usize,
}

impl<'bytes> Reader<'bytes> {
    fn take(&mut self, length: usize) -> Result<&'bytes [u8], GraphRefusal> {
        let end = self.at.checked_add(length).ok_or(GraphRefusal::Truncated)?;
        let run = self
            .bytes
            .get(self.at..end)
            .ok_or(GraphRefusal::Truncated)?;
        self.at = end;
        Ok(run)
    }

    fn u8(&mut self) -> Result<u8, GraphRefusal> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, GraphRefusal> {
        let run: [u8; 2] = self
            .take(2)?
            .try_into()
            .expect("the reader returned the requested width");
        Ok(u16::from_le_bytes(run))
    }

    fn u32(&mut self) -> Result<u32, GraphRefusal> {
        let run: [u8; 4] = self
            .take(4)?
            .try_into()
            .expect("the reader returned the requested width");
        Ok(u32::from_le_bytes(run))
    }

    fn u64(&mut self) -> Result<u64, GraphRefusal> {
        let run: [u8; 8] = self
            .take(8)?
            .try_into()
            .expect("the reader returned the requested width");
        Ok(u64::from_le_bytes(run))
    }
}
