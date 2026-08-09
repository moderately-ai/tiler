use super::*;

#[derive(Debug, Default)]
pub(super) struct MacroLanguageScan {
    pub(super) errors: Vec<String>,
    pub(super) macros: BTreeSet<String>,
    pub(super) attributes: BTreeSet<String>,
    pub(super) derives: BTreeSet<String>,
    pub(super) local_macro_rules: BTreeSet<(String, String)>,
    pub(super) proc_macro_exporters: BTreeSet<String>,
    pub(super) facade_reexports: usize,
    pub(super) facade_diagnostic_reexports: usize,
    pub(super) tensor_invocations: BTreeMap<String, usize>,
}

/// Scans every Rust code block rustdoc can extract from one source file.
pub(super) fn scan_rustdoc_code(path: &str, source: &str) -> MacroLanguageScan {
    let mut scan = MacroLanguageScan::default();
    let mut markdown = line_doc_markdown(source);
    match doc_attribute_markdown(path, source) {
        Ok(attributes) => markdown.extend(attributes),
        Err(errors) => scan.errors.extend(errors),
    }

    let mut block_number = 0;
    for document in markdown {
        let (blocks, errors) = rustdoc_rust_blocks(path, &document);
        scan.errors.extend(errors);
        for block in blocks {
            block_number += 1;
            let block_path = format!("{path}<rustdoc:{block_number}>");
            let tokens = match lex(&block_path, &block) {
                Ok(tokens) => tokens,
                Err(error) => {
                    scan.errors.push(error);
                    continue;
                }
            };
            let found = workspace_macro_language(&block_path, &tokens);
            scan.errors.extend(found.errors);
            scan.errors
                .extend(rustdoc_source_load_errors(&block_path, &tokens));
            scan.macros.extend(found.macros);
            scan.attributes.extend(found.attributes);
            scan.derives.extend(found.derives);
            for (invocation_path, count) in found.tensor_invocations {
                *scan.tensor_invocations.entry(invocation_path).or_default() += count;
            }
        }
    }
    scan
}

/// Refuses source-loading forms whose rustdoc-relative filesystem identity is
/// not enumerated by the ordinary package walk.
pub(super) fn rustdoc_source_load_errors(path: &str, tokens: &[Token]) -> Vec<String> {
    let mut errors = Vec::new();
    for index in 0..tokens.len() {
        if ident(&tokens[index], "include")
            && tokens.get(index + 1).is_some_and(|token| punct(token, "!"))
        {
            errors.push(format!(
                "unsafe-sites.{path}:{}: include! is unsupported in an extracted doctest; its \
                 rustdoc-relative source identity is not enumerated",
                tokens[index].line,
            ));
        }
        if punct(&tokens[index], "#") {
            let mut open = index + 1;
            if tokens.get(open).is_some_and(|token| punct(token, "!")) {
                open += 1;
            }
            if tokens.get(open).is_some_and(|token| punct(token, "["))
                && matching_delimiter(tokens, open).is_some_and(|end| {
                    (open + 1..end).any(|position| ident(&tokens[position], "path"))
                })
            {
                errors.push(format!(
                    "unsafe-sites.{path}:{}: #[path] is unsupported in an extracted doctest; its \
                     rustdoc-relative source identity is not enumerated",
                    tokens[index].line,
                ));
            }
        }
        if ident(&tokens[index], "mod")
            && tokens
                .get(index + 1)
                .is_some_and(|token| identifier_text(token).is_some())
            && tokens.get(index + 2).is_some_and(|token| punct(token, ";"))
        {
            errors.push(format!(
                "unsafe-sites.{path}:{}: out-of-line module load is unsupported in an extracted \
                 doctest; its rustdoc-relative source identity is not enumerated",
                tokens[index].line,
            ));
        }
    }
    errors
}

/// Collects consecutive line-doc comments as Markdown documents.
///
/// This intentionally over-approximates a doc marker appearing at the start of
/// a raw-string line: scanning prose as Markdown can only add refusals, while
/// failing to collect an actual doc comment could hide compiler input.
pub(super) fn line_doc_markdown(source: &str) -> Vec<String> {
    let mut documents = Vec::new();
    let mut current = String::new();
    for line in source.lines() {
        let trimmed = line.trim_start();
        let content = if let Some(content) = trimmed.strip_prefix("//!") {
            Some(content)
        } else if trimmed.starts_with("////") {
            None
        } else {
            trimmed.strip_prefix("///")
        };
        if let Some(content) = content {
            let content = content.strip_prefix(' ').unwrap_or(content);
            current.push_str(content);
            current.push('\n');
        } else if !current.is_empty() {
            documents.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        documents.push(current);
    }
    documents.extend(block_doc_markdown(source));
    documents
}

/// Collects nested Rust block-doc comments while excluding strings, character
/// literals, raw strings, and line comments with the same state boundaries as
/// the ordinary source lexer.
pub(super) fn block_doc_markdown(source: &str) -> Vec<String> {
    let mut documents = Vec::new();
    let mut cursor = 0;
    while cursor < source.len() {
        let tail = &source[cursor..];
        if tail.starts_with("//") {
            cursor += tail.find('\n').unwrap_or(tail.len());
            continue;
        }
        if let Some((end, _)) = raw_string_span(source, cursor) {
            cursor = end;
            continue;
        }
        let character = next_char(source, cursor);
        if character == '"' {
            cursor = ordinary_string(source, cursor).map_or(source.len(), |(end, _, _)| end);
            continue;
        }
        if character == '\''
            && let Some(end) = character_literal_end(source, cursor)
        {
            cursor = end;
            continue;
        }
        if !tail.starts_with("/*") {
            cursor += character.len_utf8();
            continue;
        }

        let start = cursor;
        let doc = tail.starts_with("/**") && !tail.starts_with("/***") || tail.starts_with("/*!");
        let mut position = start + 2;
        let content_start = if doc { start + 3 } else { position };
        let mut depth = 1_usize;
        while position < source.len() && depth != 0 {
            if source[position..].starts_with("/*") {
                depth += 1;
                position += 2;
            } else if source[position..].starts_with("*/") {
                depth -= 1;
                position += 2;
            } else {
                position += next_char(source, position).len_utf8();
            }
        }
        if depth != 0 {
            break;
        }
        if doc {
            let content_end = position - 2;
            let mut cleaned = String::new();
            for line in source[content_start..content_end].lines() {
                let line = line.trim_start();
                let line = line.strip_prefix('*').unwrap_or(line);
                let line = line.strip_prefix(' ').unwrap_or(line);
                cleaned.push_str(line);
                cleaned.push('\n');
            }
            documents.push(cleaned);
        }
        cursor = position;
    }
    documents
}

/// Collects literal `#[doc = "..."]` Markdown and refuses documentation whose
/// source is not locally enumerable. Pinned local macro templates may forward
/// a whole cooked-string literal doc value or concatenate cooked literals and
/// arm-local stringified identifiers in prose. Stringified values in rustdoc
/// code and opaque raw strings fail closed because this scanner does not expand
/// or reconstruct them. Every cooked literal invocation argument is also
/// scanned so forwarded docs cannot hide a generated doctest.
pub(super) fn doc_attribute_markdown(path: &str, source: &str) -> Result<Vec<String>, Vec<String>> {
    let tokens = lex(path, source).map_err(|error| vec![error])?;
    let macro_spans = token_generating_spans(&tokens);
    let mut markdown = Vec::new();
    let mut errors = Vec::new();
    let local_names: BTreeSet<&str> = WORKSPACE_LOCAL_MACRO_RULES
        .iter()
        .map(|(_, name)| *name)
        .collect();
    for (index, token) in tokens.iter().enumerate() {
        if punct(token, "!")
            && tokens.get(index + 1).is_some_and(is_open_delimiter)
            && index > 0
            && identifier_text(&tokens[index - 1]).is_some_and(|name| local_names.contains(name))
            && let Some(end) = matching_delimiter(&tokens, index + 1)
        {
            for raw in tokens[index + 2..end]
                .iter()
                .filter(|token| punct(token, "<raw-string>"))
            {
                errors.push(format!(
                    "unsafe-sites.{path}:{}: raw-string macro argument is unsupported for a \
                     pinned documentation-generating macro; rustdoc input must be a cooked \
                     string literal",
                    raw.line,
                ));
            }
            for (line, value) in tokens[index + 2..end].iter().filter_map(|token| {
                if let TokenKind::StringLiteral(value) = &token.kind {
                    Some((token.line, value.as_str()))
                } else {
                    None
                }
            }) {
                match cook_doc_literal(value) {
                    Ok(value) => markdown.push(value),
                    Err(detail) => errors.push(format!(
                        "unsafe-sites.{path}:{line}: unsupported documentation string escape: \
                         {detail}",
                    )),
                }
            }
        }
        if !punct(token, "#") {
            continue;
        }
        let mut open = index + 1;
        if tokens.get(open).is_some_and(|token| punct(token, "!")) {
            open += 1;
        }
        if !tokens.get(open).is_some_and(|token| punct(token, "[")) {
            continue;
        }
        let Some(end) = matching_delimiter(&tokens, open) else {
            continue;
        };
        let attribute = &tokens[open + 1..end];
        if cfg_attr_contains_doc_meta(attribute) {
            errors.push(format!(
                "unsafe-sites.{path}:{}: documentation nested in cfg_attr is unsupported; \
                 rustdoc input must use a directly enumerable doc attribute",
                token.line,
            ));
            continue;
        }
        if !attribute.first().is_some_and(|token| ident(token, "doc"))
            || !attribute.get(1).is_some_and(|token| punct(token, "="))
        {
            continue;
        }
        match &attribute[2..] {
            [
                Token {
                    kind: TokenKind::StringLiteral(value),
                    ..
                },
            ] => match cook_doc_literal(value) {
                Ok(value) => markdown.push(value),
                Err(detail) => errors.push(format!(
                    "unsafe-sites.{path}:{}: unsupported documentation string escape: {detail}",
                    token.line,
                )),
            },
            expression if inside_span(index, &macro_spans) => {
                let Some(has_stringify) =
                    enumerable_macro_doc_expression(expression, &tokens, open + 3)
                else {
                    errors.push(format!(
                        "unsafe-sites.{path}:{}: dynamic documentation source is unsupported; \
                         rustdoc-extracted code must be literal or recursively enumerable",
                        token.line,
                    ));
                    continue;
                };
                let mut cooked = String::new();
                for literal in expression.iter().filter_map(|token| match &token.kind {
                    TokenKind::StringLiteral(value) => Some(value.as_str()),
                    _ => None,
                }) {
                    match cook_doc_literal(literal) {
                        Ok(value) => cooked.push_str(&value),
                        Err(detail) => errors.push(format!(
                            "unsafe-sites.{path}:{}: unsupported documentation string escape: \
                             {detail}",
                            token.line,
                        )),
                    }
                }
                if has_stringify {
                    let (blocks, markdown_errors) = rustdoc_rust_blocks(path, &cooked);
                    if !blocks.is_empty() || !markdown_errors.is_empty() {
                        errors.push(format!(
                            "unsafe-sites.{path}:{}: stringify-composed rustdoc code is \
                             unsupported; stringified invocation values are not reconstructed",
                            token.line,
                        ));
                        errors.extend(markdown_errors);
                        continue;
                    }
                }
                markdown.push(cooked);
            }
            _ => errors.push(format!(
                "unsafe-sites.{path}:{}: dynamic documentation source is unsupported; \
                 rustdoc-extracted code must be literal or recursively enumerable",
                token.line,
            )),
        }
    }
    markdown.sort();
    markdown.dedup();
    if errors.is_empty() {
        Ok(markdown)
    } else {
        Err(errors)
    }
}

/// Whether one `cfg_attr` can synthesize a `doc` meta item, including through
/// another nested `cfg_attr`. The predicate before the first comma is skipped;
/// every later `doc` identifier is conservatively a documentation authority.
fn cfg_attr_contains_doc_meta(attribute: &[Token]) -> bool {
    if !attribute
        .first()
        .is_some_and(|token| ident(token, "cfg_attr"))
        || !attribute.get(1).is_some_and(is_open_delimiter)
    {
        return false;
    }
    let Some(close) = matching_delimiter(attribute, 1) else {
        return true;
    };
    let mut position = 2;
    while position < close && !punct(&attribute[position], ",") {
        if is_open_delimiter(&attribute[position]) {
            let Some(end) = matching_delimiter(attribute, position) else {
                return true;
            };
            position = end + 1;
        } else {
            position += 1;
        }
    }
    if position == close {
        return true;
    }
    attribute[position.saturating_add(1)..close]
        .iter()
        .any(|token| ident(token, "doc"))
}

/// Decodes the cooked-string escapes rustc applies before a `#[doc]` value
/// reaches rustdoc. Unknown or malformed escapes fail closed.
fn cook_doc_literal(value: &str) -> Result<String, String> {
    let mut chars = value.chars().peekable();
    let mut cooked = String::new();
    while let Some(character) = chars.next() {
        if character != '\\' {
            cooked.push(character);
            continue;
        }
        let Some(escape) = chars.next() else {
            return Err("trailing backslash".to_owned());
        };
        match escape {
            '\\' => cooked.push('\\'),
            '"' => cooked.push('"'),
            '\'' => cooked.push('\''),
            'n' => cooked.push('\n'),
            '\r' if chars.peek() == Some(&'\n') => {
                chars.next();
                while chars.peek().is_some_and(|next| next.is_whitespace()) {
                    chars.next();
                }
            }
            'r' => cooked.push('\r'),
            't' => cooked.push('\t'),
            '0' => cooked.push('\0'),
            '\n' => {
                while chars.peek().is_some_and(|next| next.is_whitespace()) {
                    chars.next();
                }
            }
            'x' => {
                let digits: String = chars.by_ref().take(2).collect();
                if digits.len() != 2 || !digits.chars().all(|digit| digit.is_ascii_hexdigit()) {
                    return Err(format!("malformed ASCII escape `\\x{digits}`"));
                }
                let byte = u8::from_str_radix(&digits, 16)
                    .map_err(|error| format!("invalid ASCII escape `\\x{digits}`: {error}"))?;
                if !byte.is_ascii() {
                    return Err(format!("non-ASCII string escape `\\x{digits}`"));
                }
                cooked.push(char::from(byte));
            }
            'u' => {
                if chars.next() != Some('{') {
                    return Err("Unicode escape has no opening brace".to_owned());
                }
                let mut digits = String::new();
                loop {
                    match chars.next() {
                        Some('}') => break,
                        Some(character) => digits.push(character),
                        None => return Err("Unicode escape has no closing brace".to_owned()),
                    }
                }
                if digits.is_empty()
                    || digits.len() > 6
                    || !digits.chars().all(|digit| digit.is_ascii_hexdigit())
                {
                    return Err(format!("malformed Unicode escape `\\u{{{digits}}}`"));
                }
                let scalar = u32::from_str_radix(&digits, 16)
                    .map_err(|error| format!("invalid Unicode escape: {error}"))?;
                let character = char::from_u32(scalar)
                    .ok_or_else(|| format!("Unicode escape is not a scalar: {scalar:#x}"))?;
                cooked.push(character);
            }
            other => return Err(format!("unknown escape `\\{other}`")),
        }
    }
    Ok(cooked)
}

/// Classifies a pinned local macro's generated doc expression as either one
/// directly forwarded arm-local literal or a literal/stringify-only
/// composition. The result reports whether the expression contains
/// stringification, whose invocation value this scanner cannot reconstruct.
pub(super) fn enumerable_macro_doc_expression(
    expression: &[Token],
    all_tokens: &[Token],
    expression_start: usize,
) -> Option<bool> {
    let mut has_stringify = false;
    for (index, token) in expression.iter().enumerate() {
        if punct(token, "!") && expression.get(index + 1).is_some_and(is_open_delimiter) {
            let name = index
                .checked_sub(1)
                .and_then(|position| identifier_text(&expression[position]))?;
            if !matches!(name, "concat" | "stringify") {
                return None;
            }
            if name == "stringify" {
                let open = index + 1;
                let close = matching_delimiter(expression, open)?;
                let [dollar, binding] = &expression[open + 1..close] else {
                    return None;
                };
                let binding = identifier_text(binding)?;
                if !punct(dollar, "$")
                    || !macro_arm_declares_fragment(all_tokens, expression_start, binding, "ident")
                {
                    return None;
                }
                has_stringify = true;
            }
        }
        if let Some(name) = identifier_text(token)
            && matches!(name, "include" | "include_str" | "env" | "option_env")
        {
            return None;
        }
        if punct(token, "<raw-string>") {
            return None;
        }
        if punct(token, "$") {
            let name = expression.get(index + 1).and_then(identifier_text)?;
            let stringified = index >= 3
                && ident(&expression[index - 3], "stringify")
                && punct(&expression[index - 2], "!")
                && punct(&expression[index - 1], "(")
                && expression
                    .get(index + 2)
                    .is_some_and(|token| punct(token, ")"))
                && matching_delimiter(expression, index - 1) == Some(index + 2)
                && macro_arm_declares_fragment(all_tokens, expression_start, name, "ident");
            let direct_literal = expression.len() == 2
                && index == 0
                && macro_arm_declares_fragment(all_tokens, expression_start, name, "literal");
            if !direct_literal && !stringified {
                return None;
            }
        }
    }
    Some(has_stringify)
}

/// Whether the exact macro arm emitting this doc expression binds `$name` with
/// the required fragment kind. A same-named binder in another arm proves
/// nothing.
fn macro_arm_declares_fragment(
    tokens: &[Token],
    expression_start: usize,
    name: &str,
    fragment: &str,
) -> bool {
    let Some(arrow) = (0..expression_start)
        .filter(|index| punct(&tokens[*index], "=>"))
        .filter(|arrow| {
            let open = *arrow + 1;
            matching_delimiter(tokens, open)
                .is_some_and(|close| open < expression_start && expression_start < close)
        })
        .max()
    else {
        return false;
    };
    let Some(pattern_open) = (0..arrow)
        .rev()
        .find(|open| matching_delimiter(tokens, *open) == arrow.checked_sub(1))
    else {
        return false;
    };
    tokens[pattern_open + 1..arrow - 1]
        .windows(4)
        .any(|window| {
            punct(&window[0], "$")
                && ident(&window[1], name)
                && punct(&window[2], ":")
                && ident(&window[3], fragment)
        })
}

/// Extracts fenced and indented Rust blocks under rustdoc's current Markdown
/// conventions, failing closed on an unknown fence language.
pub(super) fn rustdoc_rust_blocks(path: &str, markdown: &str) -> (Vec<String>, Vec<String>) {
    let lines: Vec<&str> = markdown.lines().collect();
    let mut blocks = Vec::new();
    let mut errors = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let trimmed = lines[index].trim_start();
        let mut nested = trimmed;
        let mut in_blockquote = false;
        while let Some(blockquote) = nested.strip_prefix('>') {
            in_blockquote = true;
            nested = blockquote.strip_prefix(' ').unwrap_or(blockquote);
        }
        if in_blockquote
            && (nested.starts_with("```")
                || nested.starts_with("~~~")
                || nested.starts_with("    ")
                || nested.starts_with('\t'))
        {
            errors.push(format!(
                "unsafe-sites.{path}: rustdoc code in a blockquote container is unsupported; \
                 compiler-input extraction must fail closed",
            ));
        }
        if !trimmed.starts_with("```")
            && !trimmed.starts_with("~~~")
            && (trimmed.contains("```") || trimmed.contains("~~~"))
        {
            errors.push(format!(
                "unsafe-sites.{path}: rustdoc fence marker in an unsupported container or \
                 position; compiler-input extraction must fail closed",
            ));
        }
        let marker = if trimmed.starts_with("```") {
            Some('`')
        } else if trimmed.starts_with("~~~") {
            Some('~')
        } else {
            None
        };
        if let Some(marker) = marker {
            let width = trimmed
                .chars()
                .take_while(|character| *character == marker)
                .count();
            let info = trimmed[width..].trim();
            let rust = if let Some(rust) = rustdoc_fence_language(info) {
                rust
            } else {
                errors.push(format!(
                    "unsafe-sites.{path}: unsupported rustdoc fence language `{info}`; \
                     compiler-input classification must fail closed",
                ));
                false
            };
            index += 1;
            let mut block = String::new();
            let mut closed = false;
            while index < lines.len() {
                let closing = lines[index].trim_start();
                if closing
                    .chars()
                    .take_while(|character| *character == marker)
                    .count()
                    >= width
                    && closing.chars().all(|character| character == marker)
                {
                    closed = true;
                    break;
                }
                if rust {
                    block.push_str(rustdoc_code_line(lines[index]));
                    block.push('\n');
                }
                index += 1;
            }
            if !closed {
                errors.push(format!(
                    "unsafe-sites.{path}: rustdoc fence `{}` never closes",
                    marker.to_string().repeat(width),
                ));
                break;
            }
            if rust {
                blocks.push(block);
            }
            index += 1;
            continue;
        }

        if lines[index].starts_with("    ") || lines[index].starts_with('\t') {
            let mut block = String::new();
            while index < lines.len()
                && (lines[index].starts_with("    ")
                    || lines[index].starts_with('\t')
                    || lines[index].trim().is_empty())
            {
                block.push_str(rustdoc_code_line(
                    lines[index]
                        .strip_prefix("    ")
                        .or_else(|| lines[index].strip_prefix('\t'))
                        .unwrap_or(""),
                ));
                block.push('\n');
                index += 1;
            }
            blocks.push(block);
            continue;
        }
        index += 1;
    }
    (blocks, errors)
}

/// Applies rustdoc's hidden-line transform before lexing the extracted crate.
pub(super) fn rustdoc_code_line(line: &str) -> &str {
    let indent = line.len() - line.trim_start().len();
    let trimmed = &line[indent..];
    if let Some(hidden) = trimmed.strip_prefix("# ") {
        hidden
    } else if let Some(escaped) = trimmed.strip_prefix("##") {
        escaped.strip_prefix(' ').unwrap_or(escaped)
    } else if trimmed == "#" {
        ""
    } else {
        line
    }
}

/// Classifies one rustdoc fence info string as Rust or an exact prose language.
pub(super) fn rustdoc_fence_language(info: &str) -> Option<bool> {
    if info.is_empty() {
        return Some(true);
    }
    let first = info.split([',', ' ']).next().unwrap_or(info);
    if matches!(
        first,
        "rust" | "compile_fail" | "no_run" | "should_panic" | "ignore"
    ) || first.starts_with("edition")
        || first.starts_with("ignore-")
    {
        Some(true)
    } else if matches!(first, "text" | "sh") {
        Some(false)
    } else {
        None
    }
}

/// Enforces and inventories the closed macro/attribute language of packages
/// where item-level attributes can reopen unsafe code.
pub(super) fn workspace_macro_language(path: &str, tokens: &[Token]) -> MacroLanguageScan {
    let mut scan = MacroLanguageScan::default();
    let mut invoked = BTreeSet::new();
    let mut definition_bangs = BTreeSet::new();
    let local_names: BTreeSet<&str> = WORKSPACE_LOCAL_MACRO_RULES
        .iter()
        .map(|(_, name)| *name)
        .collect();
    let imports_tensor = has_exact_tensor_import(tokens);

    for index in 0..tokens.len() {
        if ident(&tokens[index], "extern")
            && tokens
                .get(index + 1)
                .is_some_and(|token| ident(token, "crate"))
        {
            scan.errors.push(format!(
                "unsafe-sites.{path}:{}: extern crate declarations and aliases are unsupported; \
                 they can shadow classified macro and attribute namespaces",
                tokens[index].line,
            ));
        }
        if ident(&tokens[index], "macro")
            && tokens
                .get(index + 1)
                .is_some_and(|token| identifier_text(token).is_some())
            && tokens
                .get(index + 2)
                .is_some_and(|token| punct(token, "(") || punct(token, "{"))
        {
            scan.errors.push(format!(
                "unsafe-sites.{path}:{}: macro-2.0 definition `{}` is unsupported regardless of \
                 visibility; the exact local producer population uses pinned private \
                 macro_rules! definitions",
                tokens[index].line,
                identifier_text(&tokens[index + 1]).unwrap_or("<dynamic>"),
            ));
        }
        if ident(&tokens[index], "macro_rules")
            && tokens.get(index + 1).is_some_and(|token| punct(token, "!"))
        {
            definition_bangs.insert(index + 1);
            let Some(name) = tokens.get(index + 2).and_then(identifier_text) else {
                scan.errors.push(format!(
                    "unsafe-sites.{path}:{}: macro_rules! definition has no literal identifier",
                    tokens[index].line,
                ));
                continue;
            };
            let identity = (path.to_owned(), name.to_owned());
            if WORKSPACE_LOCAL_MACRO_RULES.contains(&(path, name)) {
                if !scan.local_macro_rules.insert(identity) {
                    scan.errors.push(format!(
                        "unsafe-sites.{path}:{}: duplicate pinned macro_rules! definition \
                         `{name}`; producer identity includes exact multiplicity",
                        tokens[index].line,
                    ));
                }
            } else {
                scan.errors.push(format!(
                    "unsafe-sites.{path}:{}: unpinned macro_rules! definition `{name}`; local \
                     token producers are an exact private population",
                    tokens[index].line,
                ));
            }
            continue;
        }
        if ident(&tokens[index], "pub")
            && tokens
                .get(index + 1)
                .is_some_and(|token| ident(token, "macro"))
        {
            scan.errors.push(format!(
                "unsafe-sites.{path}:{}: public declarative macros are unsupported; the exact \
                 local macro_rules! population is private and non-exported",
                tokens[index].line,
            ));
        }
        if !punct(&tokens[index], "!") || !tokens.get(index + 1).is_some_and(is_open_delimiter) {
            continue;
        }
        if definition_bangs.contains(&index) {
            continue;
        }
        if index > 0 && punct(&tokens[index - 1], "#") {
            continue;
        }
        let Some(name_position) = index.checked_sub(1) else {
            continue;
        };
        let Some(name) = identifier_text(&tokens[name_position]) else {
            continue;
        };
        if name_position > 0 && punct(&tokens[name_position - 1], "$") {
            scan.errors.push(format!(
                "unsafe-sites.{path}:{}: dynamic macro invocation name `${name}!` is unsupported; \
                 emitted macro authority must have a literal classified name",
                tokens[name_position].line,
            ));
            continue;
        }
        if is_rust_keyword(name) && !is_raw_identifier(&tokens[name_position]) {
            continue;
        }
        if is_exact_tensor_path(tokens, name_position) {
            *scan.tensor_invocations.entry(path.to_owned()).or_default() += 1;
            continue;
        }
        if is_exact_facade_compile_error_path(tokens, name_position) {
            scan.macros.insert("compile_error".to_owned());
            continue;
        }
        if name_position > 0 && punct(&tokens[name_position - 1], "::") {
            scan.errors.push(format!(
                "unsafe-sites.{path}:{}: path-qualified macro invocation `{name}!` is \
                 unsupported in the workspace source-language boundary",
                tokens[name_position].line,
            ));
            continue;
        }
        if name == "tensor" && imports_tensor {
            invoked.insert(name);
            *scan.tensor_invocations.entry(path.to_owned()).or_default() += 1;
            continue;
        }
        if WORKSPACE_LOCAL_MACRO_RULES.contains(&(path, name)) {
            invoked.insert(name);
            continue;
        }
        if name != "include" && !REOPENABLE_BUILTIN_MACROS.contains(&name) {
            scan.errors.push(format!(
                "unsafe-sites.{path}:{}: custom macro invocation `{name}!` is unsupported in the \
                 workspace; external procedural-macro expansions suppress unsafe-code \
                 diagnostics",
                tokens[name_position].line,
            ));
            continue;
        }
        invoked.insert(name);
        scan.macros.insert(name.to_owned());
    }

    for index in 0..tokens.len() {
        if !ident(&tokens[index], "use") {
            continue;
        }
        let Some(end) = top_level_semicolon(tokens, index + 1) else {
            scan.errors.push(format!(
                "unsafe-sites.{path}:{}: a use declaration never reaches a top-level semicolon",
                tokens[index].line,
            ));
            continue;
        };
        let tree = &tokens[index + 1..end];
        let exact_tensor_import = render_signature(tree) == "tiler::tensor";
        let exact_facade_reexport = index > 0
            && ident(&tokens[index - 1], "pub")
            && render_signature(tree) == "tiler_macros::tensor";
        let exact_facade_diagnostic_reexport = index > 0
            && ident(&tokens[index - 1], "pub")
            && render_signature(tree) == "core::compile_error as __tiler_compile_error";
        if exact_facade_reexport {
            if path == "crates/tiler/src/lib.rs" {
                scan.facade_reexports += 1;
            } else {
                scan.errors.push(format!(
                    "unsafe-sites.{path}:{}: the tensor procedural macro may be re-exported only \
                     once from the owning facade",
                    tokens[index].line,
                ));
            }
        }
        if exact_facade_diagnostic_reexport {
            if path == "crates/tiler/src/lib.rs" {
                scan.facade_diagnostic_reexports += 1;
            } else {
                scan.errors.push(format!(
                    "unsafe-sites.{path}:{}: the compiler diagnostic builtin may be re-exported \
                     only once from the owning facade",
                    tokens[index].line,
                ));
            }
        }
        if tree.iter().any(|token| punct(token, "*"))
            && !matches!(
                render_signature(tree).as_str(),
                "super::*" | "tiler_compiler::target::*"
            )
        {
            scan.errors.push(format!(
                "unsafe-sites.{path}:{}: external or non-parent glob use is unsupported because \
                 it can import an untracked macro name",
                tokens[index].line,
            ));
        }
        for name in &local_names {
            if use_tree_imports_name(tree, name) {
                scan.errors.push(format!(
                    "unsafe-sites.{path}:{}: use declaration imports or re-exports pinned local \
                     macro name `{name}`; local macro authorities must remain private to their \
                     pinned source file",
                    tokens[index].line,
                ));
            }
        }
        for namespace in GUARDED_MACRO_NAMESPACES {
            if use_tree_binds_name(tree, namespace) {
                scan.errors.push(format!(
                    "unsafe-sites.{path}:{}: use declaration binds guarded macro namespace \
                     `{namespace}`; aliases cannot establish an admitted qualified macro identity",
                    tokens[index].line,
                ));
            }
        }
        let guarded_bindings = invoked
            .iter()
            .copied()
            .chain(REOPENABLE_BUILTIN_MACROS)
            .chain(
                REOPENABLE_BUILTIN_ATTRIBUTES
                    .into_iter()
                    .filter(|name| !name.contains("::")),
            )
            .chain(REOPENABLE_BUILTIN_DERIVES)
            .chain(local_names.iter().copied())
            .chain(["tensor"])
            .collect::<BTreeSet<_>>();
        for name in guarded_bindings {
            let exact_non_macro_import = (name == "tensor"
                && (exact_tensor_import || exact_facade_reexport))
                || (name == "compile_error" && exact_facade_diagnostic_reexport)
                || (name == "env" && render_signature(tree) == "std::env")
                || (name == "Debug" && render_signature(tree) == "std::fmt::Debug");
            if use_tree_binds_name(tree, name) && !exact_non_macro_import {
                scan.errors.push(format!(
                    "unsafe-sites.{path}:{}: use declaration binds classified macro, attribute, \
                     or derive name `{name}`; imports and shadowing of admitted expansion names \
                     are unsupported",
                    tokens[index].line,
                ));
            }
        }
    }

    for index in 0..tokens.len() {
        if !punct(&tokens[index], "#") {
            continue;
        }
        let mut open = index + 1;
        if tokens.get(open).is_some_and(|token| punct(token, "!")) {
            open += 1;
        }
        if !tokens.get(open).is_some_and(|token| punct(token, "[")) {
            if tokens
                .get(open)
                .is_some_and(|token| punct(token, "$") || identifier_text(token).is_some())
            {
                scan.errors.push(format!(
                    "unsafe-sites.{path}:{}: dynamic attribute name emission is unsupported; \
                     attributes must have literal classified names",
                    tokens[index].line,
                ));
            }
            continue;
        }
        let Some(end) = matching_delimiter(tokens, open) else {
            continue;
        };
        let attribute_name = tokens.get(open + 1).and_then(identifier_text);
        if matches!(
            attribute_name,
            Some("proc_macro" | "proc_macro_attribute" | "proc_macro_derive")
        ) {
            if attribute_name == Some("proc_macro")
                && path == "crates/tiler-macros/src/lib.rs"
                && tokens.get(end + 1).is_some_and(|token| ident(token, "pub"))
                && tokens.get(end + 2).is_some_and(|token| ident(token, "fn"))
                && tokens
                    .get(end + 3)
                    .is_some_and(|token| ident(token, "tensor"))
            {
                scan.proc_macro_exporters
                    .insert("#[proc_macro] pub fn tensor".to_owned());
                scan.attributes.insert("proc_macro".to_owned());
                scan.errors.extend(tensor_exporter_guard_errors(
                    path,
                    tokens,
                    end + 1,
                    tokens[index].line,
                ));
            } else {
                scan.errors.push(format!(
                    "unsafe-sites.{path}:{}: unguarded procedural-macro exporter \
                     `#[{}]`; the exact exporter population is `#[proc_macro] pub fn tensor`",
                    tokens[index].line,
                    attribute_name.unwrap_or("<unknown>"),
                ));
            }
            continue;
        }
        validate_reopenable_attribute(path, &tokens[open + 1..end], tokens[index].line, &mut scan);
    }
    scan
}

/// Proves the sole procedural-macro entry point has one guarded final return
/// expression and no explicit early return.
pub(super) fn tensor_exporter_guard_errors(
    path: &str,
    tokens: &[Token],
    signature_start: usize,
    line: usize,
) -> Vec<String> {
    let Some(body_open) = (signature_start..tokens.len()).find(|index| punct(&tokens[*index], "{"))
    else {
        return vec![format!(
            "unsafe-sites.{path}:{line}: tensor proc-macro exporter has no body"
        )];
    };
    let Some(body_close) = matching_delimiter(tokens, body_open) else {
        return vec![format!(
            "unsafe-sites.{path}:{line}: tensor proc-macro exporter body never closes"
        )];
    };
    let body = &tokens[body_open + 1..body_close];
    let guarded: Vec<usize> = body
        .iter()
        .enumerate()
        .filter_map(|(index, token)| ident(token, "guarded_emission").then_some(index))
        .collect();
    let explicit_returns = body.iter().filter(|token| ident(token, "return")).count();
    let final_guarded = guarded.first().copied().is_some_and(|guard| {
        guarded.len() == 1
            && body.get(guard + 1).is_some_and(|token| punct(token, "("))
            && matching_delimiter(body, guard + 1) == Some(body.len() - 1)
    });
    if explicit_returns == 0 && final_guarded {
        Vec::new()
    } else {
        vec![format!(
            "unsafe-sites.{path}:{line}: tensor proc-macro exporter must have exactly one final \
             `guarded_emission(...)` return expression and no explicit early return; found {} \
             guard call(s) and {explicit_returns} return token(s)",
            guarded.len(),
        )]
    }
}

/// Whether this file imports the facade's one guarded macro under its public
/// unqualified spelling.
pub(super) fn has_exact_tensor_import(tokens: &[Token]) -> bool {
    (0..tokens.len()).any(|index| {
        ident(&tokens[index], "use")
            && top_level_semicolon(tokens, index + 1)
                .is_some_and(|end| render_signature(&tokens[index + 1..end]) == "tiler::tensor")
    })
}

/// Whether one invocation path is exactly `tiler::tensor!`.
pub(super) fn is_exact_tensor_path(tokens: &[Token], name: usize) -> bool {
    name >= 2
        && ident(&tokens[name], "tensor")
        && punct(&tokens[name - 1], "::")
        && ident(&tokens[name - 2], "tiler")
        && (name == 2 || !punct(&tokens[name - 3], "::"))
}

/// Whether one invocation path is exactly the facade-owned compiler builtin.
pub(super) fn is_exact_facade_compile_error_path(tokens: &[Token], name: usize) -> bool {
    if name < 5 {
        return false;
    }
    let start = name - 5;
    ident(&tokens[name], "__tiler_compile_error")
        && punct(&tokens[name - 1], "::")
        && ident(&tokens[name - 2], "__private")
        && punct(&tokens[name - 3], "::")
        && ident(&tokens[name - 4], "tiler")
        && punct(&tokens[start], "::")
        && exact_source_path_start(tokens, start)
}

/// Whether an absolute source path begins at a classified item/expression
/// boundary rather than as the suffix of a longer path.
fn exact_source_path_start(tokens: &[Token], start: usize) -> bool {
    start == 0
        || tokens
            .get(start - 1)
            .is_some_and(|previous| punct(previous, ";") || punct(previous, "{"))
        || (tokens
            .get(start - 1)
            .is_some_and(|previous| punct(previous, "]"))
            && (0..start - 1).rev().any(|open| {
                punct(&tokens[open], "[")
                    && matching_delimiter(tokens, open) == Some(start - 1)
                    && open > 0
                    && punct(&tokens[open - 1], "#")
            }))
}

/// Rust keywords cannot name a function-like macro. A following `!` belongs to
/// an expression such as `if !(...)`, not to a macro invocation.
pub(super) fn is_rust_keyword(name: &str) -> bool {
    matches!(
        name,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
            | "try"
            | "gen"
    )
}

/// The first semicolon outside a use tree's nested delimiters.
pub(super) fn top_level_semicolon(tokens: &[Token], start: usize) -> Option<usize> {
    let mut delimiters = Vec::new();
    for (position, token) in tokens.iter().enumerate().skip(start) {
        match punct_text(token) {
            Some("(") => delimiters.push(")"),
            Some("[") => delimiters.push("]"),
            Some("{") => delimiters.push("}"),
            Some(value @ (")" | "]" | "}")) => {
                if delimiters.pop() != Some(value) {
                    return None;
                }
            }
            Some(";") if delimiters.is_empty() => return Some(position),
            _ => {}
        }
    }
    None
}

/// Whether one use tree binds the supplied unqualified name.
pub(super) fn use_tree_binds_name(tokens: &[Token], name: &str) -> bool {
    for (position, token) in tokens.iter().enumerate() {
        if ident(token, "as")
            && tokens
                .get(position + 1)
                .is_some_and(|candidate| ident(candidate, name))
        {
            return true;
        }
        if !ident(token, name)
            || tokens
                .get(position + 1)
                .is_some_and(|next| ident(next, "as"))
        {
            continue;
        }
        if position + 1 == tokens.len()
            || tokens
                .get(position + 1)
                .is_some_and(|next| punct(next, ",") || punct(next, "}"))
        {
            return true;
        }
    }
    false
}

/// Whether a use tree imports the supplied source name, including under an
/// alias. A path module with the same spelling does not count when another
/// segment follows it.
fn use_tree_imports_name(tokens: &[Token], name: &str) -> bool {
    tokens.iter().enumerate().any(|(position, token)| {
        ident(token, name)
            && (position + 1 == tokens.len()
                || tokens
                    .get(position + 1)
                    .is_some_and(|next| ident(next, "as") || punct(next, ",") || punct(next, "}")))
    })
}

/// Validates one outer/inner attribute or one nested `cfg_attr` entry.
pub(super) fn validate_reopenable_attribute(
    path: &str,
    tokens: &[Token],
    line: usize,
    scan: &mut MacroLanguageScan,
) {
    if tokens.first().is_some_and(|token| punct(token, "$")) {
        scan.errors.push(format!(
            "unsafe-sites.{path}:{line}: dynamic attribute name emission is unsupported; \
             attributes must have literal classified names",
        ));
        return;
    }
    let Some(name) = tokens.first().and_then(identifier_text) else {
        scan.errors.push(format!(
            "unsafe-sites.{path}:{line}: attribute has no plain classified name in the \
             workspace source-language boundary",
        ));
        return;
    };
    if render_signature(tokens) == "rustfmt::skip" {
        scan.attributes.insert("rustfmt::skip".to_owned());
        return;
    }
    if tokens.get(1).is_some_and(|token| punct(token, "::")) {
        scan.errors.push(format!(
            "unsafe-sites.{path}:{line}: path-qualified/custom attribute `{name}` is unsupported \
             in the workspace source-language boundary",
        ));
        return;
    }
    if !REOPENABLE_BUILTIN_ATTRIBUTES.contains(&name) {
        scan.errors.push(format!(
            "unsafe-sites.{path}:{line}: custom attribute `{name}` is unsupported in the \
             workspace source-language boundary",
        ));
        return;
    }
    scan.attributes.insert(name.to_owned());
    if !matches!(name, "derive" | "cfg_attr") {
        return;
    }
    let Some(open) = tokens.get(1).filter(|token| punct(token, "(")).map(|_| 1) else {
        scan.errors.push(format!(
            "unsafe-sites.{path}:{line}: built-in `{name}` attribute has unsupported non-list \
             syntax",
        ));
        return;
    };
    let Some(close) = matching_delimiter(tokens, open) else {
        scan.errors.push(format!(
            "unsafe-sites.{path}:{line}: built-in `{name}` attribute never closes",
        ));
        return;
    };
    if close + 1 != tokens.len() {
        scan.errors.push(format!(
            "unsafe-sites.{path}:{line}: tokens follow the built-in `{name}` attribute list",
        ));
        return;
    }
    let entries = top_level_comma_ranges(tokens, open + 1, close);
    if name == "derive" {
        for (start, end) in entries {
            let entry = &tokens[start..end];
            if entry.len() != 1
                || entry
                    .first()
                    .and_then(identifier_text)
                    .is_none_or(|derive| !REOPENABLE_BUILTIN_DERIVES.contains(&derive))
            {
                scan.errors.push(format!(
                    "unsafe-sites.{path}:{line}: custom or path-qualified derive `{}` is \
                     unsupported in the workspace source-language boundary",
                    render_signature(entry),
                ));
            } else if let Some(derive) = entry.first().and_then(identifier_text) {
                scan.derives.insert(derive.to_owned());
            }
        }
        return;
    }
    if entries.len() < 2 {
        scan.errors.push(format!(
            "unsafe-sites.{path}:{line}: cfg_attr contains no nested built-in attribute",
        ));
        return;
    }
    for (start, end) in entries.into_iter().skip(1) {
        validate_reopenable_attribute(path, &tokens[start..end], line, scan);
    }
}

/// Half-open token ranges separated by commas outside nested delimiters.
pub(super) fn top_level_comma_ranges(
    tokens: &[Token],
    start: usize,
    end: usize,
) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut entry_start = start;
    let mut delimiters = Vec::new();
    for (position, token) in tokens.iter().enumerate().take(end).skip(start) {
        match punct_text(token) {
            Some("(") => delimiters.push(")"),
            Some("[") => delimiters.push("]"),
            Some("{") => delimiters.push("}"),
            Some(value @ (")" | "]" | "}")) => {
                let _ = delimiters.pop().filter(|expected| expected == &value);
            }
            Some(",") if delimiters.is_empty() => {
                if entry_start != position {
                    ranges.push((entry_start, position));
                }
                entry_start = position + 1;
            }
            _ => {}
        }
    }
    if entry_start != end {
        ranges.push((entry_start, end));
    }
    ranges
}
