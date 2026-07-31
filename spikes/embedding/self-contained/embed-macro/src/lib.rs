//! A proc macro whose expansion *carries* a real Tiler artifact, rather than
//! naming one.
//!
//! ADR 0004 (`docs/decisions/0004-inline-macro-aot-bundles.md`) makes each
//! inline invocation a self-contained AOT bundle, and the embedded-artifact
//! cost note (`docs/research/embedding/embedded-artifact-costs.md`) decides
//! the representation: one proc-macro byte-string literal per payload,
//! never one integer literal per byte. This crate is where that claim stops
//! being an argument. An expansion resolves an artifact envelope through the
//! real [`tiler_cache::expansion::ExpansionCache`] and emits the resulting bytes
//! as a single [`Literal::byte_string`] token, so the generated code names no
//! path, opens no file, and reaches nothing outside the tokens it produced.
//!
//! # Why the cache is real and the compilation is not
//!
//! The envelope is produced *beforehand* by `prototypes/serial-sum-compile`,
//! which runs the offline Metal toolchain and writes envelopes carrying genuine
//! compiled `metallib` objects. The macro reads one of those files inside the
//! cache's build closure. Two consequences are the point of the arrangement:
//!
//! - Every cache hit is validated by the real `decode_artifact`, and the bytes
//!   validated carry a real backend object. The build-tool exercise could not
//!   say that — its envelope declared its payload by descriptor — so "a carried
//!   compiled payload" was a named gap there and is closed here.
//! - The file is an input to the *closure*, never to the key. The subject is
//!   composed from the member name alone, so a cache hit needs no file on disk,
//!   which is what makes "delete the artifact, force re-expansion" a question
//!   with an observable answer rather than a tautology.
//!
//! # The environment an expansion reads
//!
//! Every input is stated, and an unstated one is a refusal rather than a
//! default, for the reason `crates/tiler-macros/src/cache_root.rs` gives for
//! the production policy: a root that quietly relocates is a cache that
//! quietly recompiles.
//!
//! | Variable | Meaning |
//! | --- | --- |
//! | `TILER_EMBED_DIR` | absolute directory holding the Tiler-produced envelopes |
//! | `TILER_EMBED_MEMBER_<SLOT>` | the envelope file name this slot embeds |
//! | `TILER_EMBED_CACHE` | absolute expansion-cache root, or `off` |
//! | `TILER_EMBED_CEILING_BYTES` | per-invocation embedding ceiling, default 1 MiB |
//! | `TILER_EMBED_STATE` | optional directory to record one event file per expansion |
//!
//! `TILER_EMBED_CEILING_BYTES` exists so the ceiling refusal can be *driven*.
//! Reaching it with a real payload would need a 1 MiB artifact this slice does
//! not produce, and a refusal no test can reach is a refusal no reader should
//! believe.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use tiler_cache::expansion::{
    ComposedSubject, ExpansionCache, PublishFailure, Resolution, SubjectFacets,
};

/// The directory holding the Tiler-produced envelopes.
const DIRECTORY_VARIABLE: &str = "TILER_EMBED_DIR";
/// The prefix of the per-slot member selector.
const MEMBER_PREFIX: &str = "TILER_EMBED_MEMBER_";
/// The expansion cache root, or [`DISABLE_VALUE`].
const CACHE_VARIABLE: &str = "TILER_EMBED_CACHE";
/// The overridable per-invocation embedding ceiling.
const CEILING_VARIABLE: &str = "TILER_EMBED_CEILING_BYTES";
/// The optional event-recording directory.
const STATE_VARIABLE: &str = "TILER_EMBED_STATE";
/// The one cache-root value that means "expand with no cache".
const DISABLE_VALUE: &str = "off";

/// The per-invocation embedding ceiling, in bytes.
///
/// 1 MiB, which is the figure the embedded-artifact cost note
/// (`docs/research/embedding/embedded-artifact-costs.md`) records as the
/// measured envelope. It is restated here rather than derived,
/// because a spike that computed its own ceiling would be measuring its own
/// arithmetic.
const DEFAULT_CEILING_BYTES: usize = 1 << 20;

/// Embeds one Tiler artifact envelope as a byte-string literal.
///
/// Invoked as `embed!(NAME, "slot");`. `NAME` names the emitted constants and
/// `slot` selects which `TILER_EMBED_MEMBER_<SLOT>` names the envelope, so two
/// crates can be pointed at the same artifact or at different ones without
/// either crate's source changing.
///
/// It expands to three items and nothing else:
///
/// ```text
/// pub const NAME: &[u8] = b"…";
/// pub const NAME_LEN: usize = 32136;
/// pub const NAME_FNV1A: u64 = 0x…;
/// ```
#[proc_macro]
pub fn embed(input: TokenStream) -> TokenStream {
    let region = Span::call_site();
    match expand(input) {
        Ok(expanded) => expanded,
        Err(refusal) => spanned_compile_error(region, &refusal.to_string()),
    }
}

/// Expands one invocation, or returns the refusal a consumer will read.
fn expand(input: TokenStream) -> Result<TokenStream, EmbedRefusal> {
    let (name, slot) = parse(input)?;
    let variable = format!("{MEMBER_PREFIX}{}", slot.to_uppercase());
    let member = stated(&variable)?;
    let directory = PathBuf::from(stated(DIRECTORY_VARIABLE)?);
    let path = directory.join(&member);

    let started = unix_nanos();
    let mut built = false;
    let outcome = resolve(&member, &path, &mut built)?;
    let (bytes, label) = outcome;

    let ceiling = ceiling();
    if bytes.len() > ceiling {
        return Err(EmbedRefusal::CeilingExceeded {
            member,
            bytes: bytes.len(),
            ceiling,
        });
    }

    record_event(&Event {
        slot: &slot,
        member: &member,
        label,
        built,
        bytes: bytes.len(),
        started,
        ended: unix_nanos(),
    });

    Ok(emit(&name, &bytes))
}

/// Reads `NAME, "slot"` off the invocation.
fn parse(input: TokenStream) -> Result<(String, String), EmbedRefusal> {
    let trees: Vec<TokenTree> = input.into_iter().collect();
    let malformed = || EmbedRefusal::Malformed {
        found: trees
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(" "),
    };
    let [TokenTree::Ident(name), TokenTree::Punct(comma), TokenTree::Literal(slot)] =
        trees.as_slice()
    else {
        return Err(malformed());
    };
    if comma.as_char() != ',' {
        return Err(malformed());
    }
    // `Literal` has no accessor for a string literal's value, so the quotes come
    // off the rendered form. A slot is an ASCII identifier by construction here,
    // and anything else fails the `is_ascii_alphanumeric` guard below rather
    // than reaching an environment lookup.
    let rendered = slot.to_string();
    let slot = rendered
        .strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
        .ok_or_else(malformed)?;
    if slot.is_empty() || !slot.chars().all(|character| character.is_ascii_alphanumeric()) {
        return Err(malformed());
    }
    Ok((name.to_string(), slot.to_owned()))
}

/// Resolves the envelope through the cache, or reports why it could not.
fn resolve(
    member: &str,
    path: &Path,
    built: &mut bool,
) -> Result<(Vec<u8>, &'static str), EmbedRefusal> {
    // The subject is a function of the member name alone. Digesting the file
    // would make a hit require the file, which is precisely the dependency this
    // spike exists to show the expansion does not have.
    let backend = format!("tiler.embed.member.{member}").into_bytes();
    let program = b"tiler.embed.artifact-program.v1".to_vec();
    let runs: [&[u8]; 1] = [&backend];
    let subject = ComposedSubject::compose(&SubjectFacets {
        backend_compilations: &runs,
        artifact_program: &program,
    })
    .expect("both facets are non-empty");

    let cache = cache_root()?;
    let Some(cache) = cache else {
        // `off`: read the file every time and embed without storing anything.
        let bytes = fs::read(path).map_err(|cause| EmbedRefusal::MemberUnavailable {
            member: member.to_owned(),
            path: path.display().to_string(),
            cause: cause.to_string(),
            cached: false,
        })?;
        *built = true;
        return Ok((bytes, "uncached"));
    };

    let cache = ExpansionCache::open(cache);
    match cache.get_or_publish(&subject, || {
        *built = true;
        fs::read(path).map_err(MemberUnreadable)
    }) {
        Ok(Resolution::Hit { entry, .. }) => Ok((entry.envelope_bytes().to_vec(), "hit")),
        Ok(Resolution::Published { entry, .. }) => {
            Ok((entry.envelope_bytes().to_vec(), "published"))
        }
        Ok(Resolution::Uncached { envelope, .. }) => Ok((envelope, "uncached")),
        Err(PublishFailure::Build(MemberUnreadable(cause))) => {
            Err(EmbedRefusal::MemberUnavailable {
                member: member.to_owned(),
                path: path.display().to_string(),
                cause: cause.to_string(),
                cached: true,
            })
        }
        Err(PublishFailure::Artifact(failure)) => Err(EmbedRefusal::InvalidArtifact {
            member: member.to_owned(),
            path: path.display().to_string(),
            cause: format!("{failure:?}"),
        }),
    }
}

/// The build closure's failure: the named envelope could not be read.
struct MemberUnreadable(std::io::Error);

impl fmt::Display for MemberUnreadable {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Resolves the stated cache root, or `None` when the consumer disabled it.
fn cache_root() -> Result<Option<PathBuf>, EmbedRefusal> {
    let stated = stated(CACHE_VARIABLE)?;
    if stated == DISABLE_VALUE {
        return Ok(None);
    }
    let root = PathBuf::from(stated);
    if !root.is_absolute() {
        return Err(EmbedRefusal::CacheRootRelative {
            value: root.display().to_string(),
        });
    }
    Ok(Some(root))
}

/// Reads one required variable, refusing rather than defaulting.
fn stated(variable: &str) -> Result<String, EmbedRefusal> {
    std::env::var(variable)
        .ok()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| EmbedRefusal::Unstated {
            variable: variable.to_owned(),
        })
}

/// The per-invocation ceiling in force for this expansion.
fn ceiling() -> usize {
    std::env::var(CEILING_VARIABLE)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_CEILING_BYTES)
}

/// Why an invocation did not expand.
///
/// One variant per failure class a consumer can reach, and every rendering names
/// what to change. A refusal that only says "embedding failed" spends the one
/// place the consumer is looking.
enum EmbedRefusal {
    /// A required input is unstated or empty.
    Unstated { variable: String },
    /// The cache root is stated but relative.
    CacheRootRelative { value: String },
    /// The named envelope could not be read, and no cache entry stood in for it.
    MemberUnavailable {
        member: String,
        path: String,
        cause: String,
        cached: bool,
    },
    /// The bytes read are not a decodable Tiler artifact.
    InvalidArtifact {
        member: String,
        path: String,
        cause: String,
    },
    /// The payload is larger than this invocation may embed.
    CeilingExceeded {
        member: String,
        bytes: usize,
        ceiling: usize,
    },
    /// The invocation is not `embed!(NAME, "slot")`.
    Malformed { found: String },
}

impl fmt::Display for EmbedRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // The cache variable states its own remedy, so the shared sentence
            // would tell a reader to set the variable they already failed to
            // set and then offer the same variable as the alternative.
            Self::Unstated { variable } if variable == CACHE_VARIABLE => write!(
                formatter,
                "`embed!` requires `{CACHE_VARIABLE}` to be set to a non-empty value, and will not \
                 substitute a default: a cache root that arrives unstated is a cache that quietly \
                 relocates, and a developer sees only that builds became slow. Set it to an \
                 absolute directory path only you can write, or to `{DISABLE_VALUE}` to expand \
                 without a cache",
            ),
            Self::Unstated { variable } => write!(
                formatter,
                "`embed!` requires `{variable}` to be set to a non-empty value, and will not \
                 substitute a default: an embedding that silently changed which artifact it \
                 carried would produce a consumer binary nobody named. Set `{variable}`, or set \
                 `{CACHE_VARIABLE}` to `{DISABLE_VALUE}` to expand without a cache",
            ),
            Self::CacheRootRelative { value } => write!(
                formatter,
                "`{CACHE_VARIABLE}` is set to `{value}`, which is not an absolute path. A proc \
                 macro runs in the build tool's working directory rather than yours, and `cargo` \
                 and `rust-analyzer` need not agree on it, so a relative root would name different \
                 directories in one project. Set `{CACHE_VARIABLE}` to an absolute directory path \
                 only you can write, or to `{DISABLE_VALUE}` to expand without a cache",
            ),
            Self::MemberUnavailable {
                member,
                path,
                cause,
                cached,
            } => write!(
                formatter,
                "`embed!` cannot carry `{member}`: {path} could not be read ({cause}){}. The \
                 artifact is an input to this expansion, so it must exist the first time a build \
                 expands this invocation; it is not needed afterwards, because the bytes are \
                 already in the expanded code. Re-run the producer, or point `{DIRECTORY_VARIABLE}` \
                 at a directory that holds it",
                if *cached {
                    ", and no cache entry stood in for it"
                } else {
                    ""
                },
            ),
            Self::InvalidArtifact {
                member,
                path,
                cause,
            } => write!(
                formatter,
                "`embed!` read {path} for `{member}`, but those bytes are not a decodable Tiler \
                 artifact ({cause}); embedding them would put a payload in the consumer's binary \
                 that no runtime could accept. Re-run the producer that writes it",
            ),
            Self::CeilingExceeded {
                member,
                bytes,
                ceiling,
            } => write!(
                formatter,
                "`embed!` refuses to carry `{member}`: it is {bytes} bytes and this invocation's \
                 ceiling is {ceiling} bytes. The ceiling is a measured product bound rather than a \
                 Rust or linker limit, and every emitted copy counts against it, so crossing it is \
                 an explicit decision with a new measurement behind it. Raise \
                 `{CEILING_VARIABLE}`, or split the region so each invocation carries less",
            ),
            Self::Malformed { found } => write!(
                formatter,
                "`embed!` takes an identifier and a slot string, as `embed!(NAME, \"a\")`; found \
                 `{found}`",
            ),
        }
    }
}

/// Builds the three items one invocation expands to.
///
/// The payload is one [`Literal::byte_string`] token, spliced into the stream
/// rather than rendered into text and re-lexed, so what reaches rustc is
/// unambiguously the single-token representation the cost note decided on.
fn emit(name: &str, bytes: &[u8]) -> TokenStream {
    let mut expanded = TokenStream::new();
    expanded.extend(lex(&format!(
        "#[doc = \"The artifact envelope this invocation carries.\"] pub const {name}: &[u8] ="
    )));
    expanded.extend([TokenTree::Literal(Literal::byte_string(bytes))]);
    expanded.extend(lex(";"));
    expanded.extend(lex(&format!(
        "#[doc = \"The carried envelope's length.\"] pub const {name}_LEN: usize = {};",
        bytes.len(),
    )));
    expanded.extend(lex(&format!(
        "#[doc = \"FNV-1a over the carried envelope, so a run can prove it holds those bytes.\"] \
         pub const {name}_FNV1A: u64 = {};",
        fnv1a(bytes),
    )));
    expanded
}

/// FNV-1a over the payload.
///
/// A checksum rather than a digest, because the consumer recomputes it at run
/// time and must do so with no dependency at all — the expanded crate's whole
/// claim is that it needs nothing.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// Lexes generated scaffolding at the call site.
fn lex(source: &str) -> TokenStream {
    source.parse().expect("the emitted scaffolding lexes")
}

/// One recorded expansion.
struct Event<'text> {
    slot: &'text str,
    member: &'text str,
    label: &'static str,
    built: bool,
    bytes: usize,
    started: u128,
    ended: u128,
}

/// Writes one event as its own file, when a state directory is stated.
///
/// One file per event rather than appended lines, because several uncoordinated
/// processes write here at once and a driver must be able to *count* the
/// population it is judging. Interleaved appends can lose that count silently.
fn record_event(event: &Event<'_>) {
    let Some(state) = std::env::var_os(STATE_VARIABLE) else {
        return;
    };
    let events = PathBuf::from(state).join("events");
    if fs::create_dir_all(&events).is_err() {
        return;
    }
    let path = events.join(format!("{}.{}.json", event.ended, process::id()));
    let record = format!(
        concat!(
            "{{\"slot\":\"{}\",\"member\":\"{}\",\"outcome\":\"{}\",\"built\":{},",
            "\"bytes\":{},\"pid\":{},\"driver\":\"{}\",\"cwd\":{:?},",
            "\"started_ns\":{},\"ended_ns\":{}}}"
        ),
        event.slot,
        event.member,
        event.label,
        event.built,
        event.bytes,
        process::id(),
        driver_name(),
        working_directory(),
        event.started,
        event.ended,
    );
    let _ = fs::write(path, record);
}

/// Names the executable performing this expansion.
///
/// `CARGO_PKG_NAME` does not distinguish the drivers — `rust-analyzer` populates
/// a proc macro's environment from the crate graph it loaded, so that variable is
/// present under both and a macro reading it concludes "cargo" in both. The host
/// executable is the signal that works, and this is the measurement the
/// build-tool exercise (`docs/research/cache/build-tool-exercise.md`) recorded.
fn driver_name() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| "unknown".to_owned())
}

/// The working directory the expansion ran in.
fn working_directory() -> String {
    std::env::current_dir()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "unknown".to_owned())
}

/// Reads the wall clock as nanoseconds since the Unix epoch.
fn unix_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_nanos())
}

/// Builds `compile_error! { "<message>" }` with every token carrying `span`.
fn spanned_compile_error(span: Span, message: &str) -> TokenStream {
    let mut literal = Literal::string(message);
    literal.set_span(span);

    let mut body = TokenStream::new();
    body.extend([TokenTree::Literal(literal)]);

    let mut bang = Punct::new('!', Spacing::Alone);
    bang.set_span(span);

    let mut group = Group::new(Delimiter::Brace, body);
    group.set_span(span);

    let mut expanded = TokenStream::new();
    expanded.extend([
        TokenTree::Ident(Ident::new("compile_error", span)),
        TokenTree::Punct(bang),
        TokenTree::Group(group),
    ]);
    expanded
}
