use super::*;

/// Scans one Rust source file for direct unsafe-code permissions.
pub(super) fn scan_text(path: &str, source: &str) -> Scan {
    let tokens = match lex(path, source) {
        Ok(tokens) => tokens,
        Err(error) => {
            return Scan {
                sites: Sites::new(),
                errors: vec![error],
                loads: Vec::new(),
                builtin_macros: BTreeSet::new(),
                builtin_attributes: BTreeSet::new(),
                builtin_derives: BTreeSet::new(),
                local_macro_rules: BTreeSet::new(),
                proc_macro_exporters: BTreeSet::new(),
                facade_reexports: 0,
                facade_diagnostic_reexports: 0,
                tensor_invocations: BTreeMap::new(),
                rustdoc_tensor_invocations: BTreeMap::new(),
            };
        }
    };
    let mut scan = Scan::default();
    let mut accounted = BTreeSet::new();
    let macro_spans = token_generating_spans(&tokens);
    let depths = curly_depths(path, &tokens, &mut scan.errors);
    let (loads, load_errors) = source_loads(path, &tokens, &macro_spans, &depths);
    scan.loads = loads;
    scan.errors.extend(load_errors);

    let boundary = workspace_macro_language(path, &tokens);
    scan.errors.extend(boundary.errors);
    scan.builtin_macros.extend(boundary.macros);
    scan.builtin_attributes.extend(boundary.attributes);
    scan.builtin_derives.extend(boundary.derives);
    scan.local_macro_rules.extend(boundary.local_macro_rules);
    scan.proc_macro_exporters
        .extend(boundary.proc_macro_exporters);
    scan.facade_reexports += boundary.facade_reexports;
    scan.facade_diagnostic_reexports += boundary.facade_diagnostic_reexports;
    for (invocation_path, count) in boundary.tensor_invocations {
        *scan.tensor_invocations.entry(invocation_path).or_default() += count;
    }
    for (position, token) in tokens.iter().enumerate() {
        if ident(token, "include")
            && !(tokens
                .get(position + 1)
                .is_some_and(|token| punct(token, "!"))
                && tokens.get(position + 2).is_some_and(is_open_delimiter))
        {
            scan.errors.push(format!(
                "unsafe-sites.{path}:{}: the include macro name appears outside direct \
                 `include!(literal)` syntax; imported or aliased include forms have no \
                 lexical source identity",
                token.line,
            ));
        }
    }

    for (start, end) in &macro_spans {
        let occurrences: Vec<usize> = (*start..=*end)
            .filter(|position| ident(&tokens[*position], "unsafe_code"))
            .collect();
        if let Some(position) = occurrences.first() {
            scan.errors.push(format!(
                "unsafe-sites.{path}:{}: unsafe-code permission appears inside a \
                 token-generating macro context; expansion multiplicity has no admitted pin \
                 identity",
                tokens[*position].line,
            ));
        }
        accounted.extend(occurrences);
    }
    let mut index = 0;

    while index < tokens.len() {
        if !punct(&tokens[index], "#") {
            index += 1;
            continue;
        }
        if inside_span(index, &macro_spans) {
            index += 1;
            continue;
        }
        let mut open = index + 1;
        let inner = tokens.get(open).is_some_and(|token| punct(token, "!"));
        if inner {
            open += 1;
        }
        if !tokens.get(open).is_some_and(|token| punct(token, "[")) {
            index += 1;
            continue;
        }
        let Some(end) = matching_delimiter(&tokens, open) else {
            scan.errors.push(format!(
                "unsafe-sites.{path}:{}: an attribute never closes",
                tokens[index].line,
            ));
            break;
        };
        let occurrences: Vec<usize> = (open + 1..end)
            .filter(|position| ident(&tokens[*position], "unsafe_code"))
            .collect();
        if occurrences.is_empty() {
            index = end + 1;
            continue;
        }
        accounted.extend(occurrences.iter().copied());

        if inner && is_doctest_forbid_attribute(&tokens, open, end) {
            index = end + 1;
            continue;
        }
        if inner {
            scan.errors.push(format!(
                "unsafe-sites.{path}:{}: a crate-level unsafe-code allow is outside the admitted \
                 per-item boundary",
                tokens[index].line,
            ));
            index = end + 1;
            continue;
        }
        if !tokens
            .get(open + 1)
            .is_some_and(|token| ident(token, "allow"))
        {
            scan.errors.push(format!(
                "unsafe-sites.{path}:{}: `unsafe_code` appears outside a supported direct \
                `#[allow(...)]`; cfg_attr and other lint attributes fail closed",
                tokens[index].line,
            ));
            index = end + 1;
            continue;
        }
        if depths.get(index).copied().unwrap_or(0) != 0 {
            scan.errors.push(format!(
                "unsafe-sites.{path}:{}: nested permission is outside the file-root pin \
                 boundary; module, impl, and function semantic paths are unsupported",
                tokens[index].line,
            ));
            index = end + 1;
            continue;
        }

        let reason = match direct_allow_reason(path, &tokens, open, end) {
            Ok(reason) => reason,
            Err(error) => {
                scan.errors.push(error);
                index = end + 1;
                continue;
            }
        };
        let (item, _) = match following_function_signature(path, &tokens, end + 1) {
            Ok(item) => item,
            Err(error) => {
                scan.errors.push(error);
                index = end + 1;
                continue;
            }
        };
        let key = (path.to_owned(), item.clone());
        if scan.sites.insert(key, reason).is_some() {
            scan.errors.push(format!(
                "unsafe-sites.{path}: `{item}` carries unsafe-code permission twice",
            ));
        }
        index = end + 1;
    }

    for (position, token) in tokens.iter().enumerate() {
        if ident(token, "unsafe_code") && !accounted.contains(&position) {
            scan.errors.push(format!(
                "unsafe-sites.{path}:{}: `unsafe_code` appears outside a supported direct \
                 `#[allow(...)]` attribute",
                token.line,
            ));
        }
    }
    scan
}

/// One file's closed macro-language observations.
pub(super) fn is_doctest_forbid_attribute(tokens: &[Token], open: usize, end: usize) -> bool {
    render_signature(&tokens[open + 1..end]) == "doc(test(attr(forbid(unsafe_code))))"
}

/// Counts exact rustdoc unsafe sentinels that are crate-level inner attributes.
/// A textual copy nested in a module does not govern doctests extracted from
/// the rest of the target root.
/// Whether an inner attribute is exactly the governed rustdoc test-crate
/// sentinel. It strengthens the lint and is checked against Cargo's exact
/// doctest-root population separately.
pub(super) fn root_doctest_sentinel_count(path: &str, source: &str) -> Result<usize, String> {
    let tokens = lex(path, source)?;
    let mut errors = Vec::new();
    let depths = curly_depths(path, &tokens, &mut errors);
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    let mut count = 0;
    for index in 0..tokens.len() {
        if !punct(&tokens[index], "#") || depths.get(index).copied() != Some(0) {
            continue;
        }
        let open = index + 2;
        if !tokens.get(index + 1).is_some_and(|token| punct(token, "!"))
            || !tokens.get(open).is_some_and(|token| punct(token, "["))
        {
            continue;
        }
        let Some(end) = matching_delimiter(&tokens, open) else {
            continue;
        };
        if is_doctest_forbid_attribute(&tokens, open, end) {
            count += 1;
        }
    }
    Ok(count)
}

/// Token-tree spans whose contents can be emitted zero, one, or many times.
///
/// Direct `include!` is excluded: it has its own literal source-loading
/// boundary. Every other visible macro invocation is a token-generating
/// context, and `macro_rules! name { ... }` needs its named-definition shape
/// recognized separately.
pub(super) fn token_generating_spans(tokens: &[Token]) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    for index in 0..tokens.len() {
        if ident(&tokens[index], "macro_rules")
            && tokens.get(index + 1).is_some_and(|token| punct(token, "!"))
            && tokens
                .get(index + 2)
                .is_some_and(|token| identifier_text(token).is_some())
            && tokens.get(index + 3).is_some_and(is_open_delimiter)
        {
            if let Some(end) = matching_delimiter(tokens, index + 3) {
                spans.push((index, end));
            }
            continue;
        }
        if !punct(&tokens[index], "!") || !tokens.get(index + 1).is_some_and(is_open_delimiter) {
            continue;
        }
        if index > 0 && punct(&tokens[index - 1], "#") {
            continue;
        }
        let name = index
            .checked_sub(1)
            .and_then(|position| identifier_text(&tokens[position]));
        if name == Some("include") {
            continue;
        }
        if let Some(end) = matching_delimiter(tokens, index + 1) {
            spans.push((index, end));
        }
    }
    spans.sort_unstable();
    spans
}

/// Literal local files loaded by compiler source-loading syntax and errors for
/// forms whose resulting source population cannot be enumerated here.
pub(super) fn source_loads(
    path: &str,
    tokens: &[Token],
    macro_spans: &[(usize, usize)],
    depths: &[usize],
) -> (Vec<SourceLoad>, Vec<String>) {
    let mut loads = Vec::new();
    let mut errors = Vec::new();

    for index in 0..tokens.len() {
        if ident(&tokens[index], "include")
            && tokens.get(index + 1).is_some_and(|token| punct(token, "!"))
            && tokens.get(index + 2).is_some_and(is_open_delimiter)
        {
            let line = tokens[index].line;
            if inside_span(index, macro_spans) {
                errors.push(format!(
                    "unsafe-sites.{path}:{line}: include! inside a token-generating macro \
                     context has expansion-dependent source identity",
                ));
                continue;
            }
            let open = index + 2;
            let Some(end) = matching_delimiter(tokens, open) else {
                errors.push(format!(
                    "unsafe-sites.{path}:{line}: include! source expression never closes",
                ));
                continue;
            };
            match &tokens[open + 1..end] {
                [
                    Token {
                        kind: TokenKind::StringLiteral(literal),
                        ..
                    },
                ] if !literal.contains('\\') => loads.push(SourceLoad {
                    kind: "include!",
                    literal: literal.clone(),
                    line,
                }),
                [
                    Token {
                        kind: TokenKind::StringLiteral(_),
                        ..
                    },
                ] => errors.push(format!(
                    "unsafe-sites.{path}:{line}: escaped include! paths are unsupported because \
                     their filesystem identity is not literal",
                )),
                _ => errors.push(format!(
                    "unsafe-sites.{path}:{line}: computed include! is unsupported; generated or \
                     OUT_DIR sources cannot be inventoried",
                )),
            }
        }

        if !punct(&tokens[index], "#") {
            continue;
        }
        let open = index + 1;
        if !tokens.get(open).is_some_and(|token| punct(token, "[")) {
            continue;
        }
        let Some(end) = matching_delimiter(tokens, open) else {
            continue;
        };
        let occurrences: Vec<usize> = (open + 1..end)
            .filter(|position| ident(&tokens[*position], "path"))
            .collect();
        if occurrences.is_empty() {
            continue;
        }
        let line = tokens[index].line;
        if inside_span(index, macro_spans) {
            errors.push(format!(
                "unsafe-sites.{path}:{line}: #[path] inside a token-generating macro context \
                 has expansion-dependent source identity",
            ));
            continue;
        }
        if depths.get(index).copied().unwrap_or(0) != 0 {
            errors.push(format!(
                "unsafe-sites.{path}:{line}: nested #[path] resolution is unsupported; its \
                 compiler-relative module directory is not a literal source-file parent",
            ));
            continue;
        }
        match &tokens[open + 1..end] {
            [
                path_token,
                equals,
                Token {
                    kind: TokenKind::StringLiteral(literal),
                    ..
                },
            ] if ident(path_token, "path") && punct(equals, "=") && !literal.contains('\\') => {
                loads.push(SourceLoad {
                    kind: "#[path]",
                    literal: literal.clone(),
                    line,
                });
            }
            [
                path_token,
                equals,
                Token {
                    kind: TokenKind::StringLiteral(_),
                    ..
                },
            ] if ident(path_token, "path") && punct(equals, "=") => errors.push(format!(
                "unsafe-sites.{path}:{line}: escaped #[path] values are unsupported because \
                 their filesystem identity is not literal",
            )),
            _ => errors.push(format!(
                "unsafe-sites.{path}:{line}: a source-loading `path` appears outside supported \
                 literal #[path = \"...\"] syntax",
            )),
        }
    }
    (loads, errors)
}

/// Curly-brace depth before every token, with unmatched braces reported.
pub(super) fn curly_depths(path: &str, tokens: &[Token], errors: &mut Vec<String>) -> Vec<usize> {
    let mut depths = Vec::with_capacity(tokens.len());
    let mut depth = 0_usize;
    for token in tokens {
        depths.push(depth);
        if punct(token, "{") {
            depth += 1;
        } else if punct(token, "}") {
            if depth == 0 {
                errors.push(format!(
                    "unsafe-sites.{path}:{}: unmatched `}}` in source",
                    token.line,
                ));
            } else {
                depth -= 1;
            }
        }
    }
    if depth != 0 {
        errors.push(format!(
            "unsafe-sites.{path}: source ends with {depth} unclosed `{{` delimiter(s)",
        ));
    }
    depths
}

/// Whether one token position lies in any closed token-generating span.
pub(super) fn inside_span(position: usize, spans: &[(usize, usize)]) -> bool {
    spans
        .iter()
        .any(|(start, end)| *start <= position && position <= *end)
}

/// Whether one token opens a balanced token tree.
pub(super) fn is_open_delimiter(token: &Token) -> bool {
    matches!(punct_text(token), Some("(" | "[" | "{"))
}

/// Reads the reason from one supported direct allow attribute.
pub(super) fn direct_allow_reason(
    path: &str,
    tokens: &[Token],
    open_bracket: usize,
    close_bracket: usize,
) -> Result<String, String> {
    let line = tokens[open_bracket].line;
    let open_paren = open_bracket + 2;
    if !tokens
        .get(open_paren)
        .is_some_and(|token| punct(token, "("))
    {
        return Err(format!(
            "unsafe-sites.{path}:{line}: the allow attribute does not open a meta list",
        ));
    }
    let Some(close_paren) = matching_delimiter(tokens, open_paren) else {
        return Err(format!(
            "unsafe-sites.{path}:{line}: the allow attribute's meta list never closes",
        ));
    };
    if close_paren + 1 != close_bracket {
        return Err(format!(
            "unsafe-sites.{path}:{line}: tokens follow the allow meta list before `]`; this \
             attribute form is unsupported",
        ));
    }

    let mut cursor = open_paren + 1;
    let mut saw_lint = false;
    let mut reason = None;
    while cursor < close_paren {
        if punct(&tokens[cursor], ",") {
            return Err(format!(
                "unsafe-sites.{path}:{line}: the allow list has an empty entry",
            ));
        }
        let entry_line = tokens[cursor].line;
        let Some(mut name) = identifier_text(&tokens[cursor]).map(str::to_owned) else {
            return Err(format!(
                "unsafe-sites.{path}:{entry_line}: an allow entry does not begin with an \
                 identifier; this meta syntax is unsupported",
            ));
        };
        cursor += 1;
        while cursor + 1 < close_paren && punct(&tokens[cursor], "::") {
            let Some(segment) = identifier_text(&tokens[cursor + 1]) else {
                return Err(format!(
                    "unsafe-sites.{path}:{entry_line}: an allow path ends after `::`",
                ));
            };
            name.push_str("::");
            name.push_str(segment);
            cursor += 2;
        }

        if name == "reason" {
            if !tokens.get(cursor).is_some_and(|token| punct(token, "=")) {
                return Err(format!(
                    "unsafe-sites.{path}:{entry_line}: `reason` is not assigned an ordinary \
                     string literal",
                ));
            }
            let Some(Token {
                kind: TokenKind::StringLiteral(value),
                ..
            }) = tokens.get(cursor + 1)
            else {
                return Err(format!(
                    "unsafe-sites.{path}:{entry_line}: `reason` is not an ordinary string \
                     literal; computed and raw forms are unsupported",
                ));
            };
            if reason.replace(value.clone()).is_some() {
                return Err(format!(
                    "unsafe-sites.{path}:{entry_line}: the allow attribute states two reasons",
                ));
            }
            cursor += 2;
        } else if name == "unsafe_code" {
            if saw_lint {
                return Err(format!(
                    "unsafe-sites.{path}:{entry_line}: the allow attribute names unsafe_code \
                     twice",
                ));
            }
            saw_lint = true;
        } else if name.ends_with("::unsafe_code") {
            return Err(format!(
                "unsafe-sites.{path}:{entry_line}: `{name}` is not the whole unsafe-code lint \
                 name",
            ));
        }

        if cursor < close_paren {
            if !punct(&tokens[cursor], ",") {
                return Err(format!(
                    "unsafe-sites.{path}:{}: allow entries must be comma-separated",
                    tokens[cursor].line,
                ));
            }
            cursor += 1;
            if cursor == close_paren {
                break;
            }
        }
    }

    if !saw_lint {
        return Err(format!(
            "unsafe-sites.{path}:{line}: the recognized allow did not contain unsafe_code as a \
             whole lint name",
        ));
    }
    reason.ok_or_else(|| {
        format!(
            "unsafe-sites.{path}:{line}: the unsafe-code permission has no ordinary string \
             `reason` as ADR 0079 requires",
        )
    })
}

/// Returns the complete signature of the function following an attribute.
pub(super) fn following_function_signature(
    path: &str,
    tokens: &[Token],
    mut cursor: usize,
) -> Result<(String, usize), String> {
    while cursor < tokens.len() && punct(&tokens[cursor], "#") {
        let open = cursor + 1;
        if tokens.get(open).is_some_and(|token| punct(token, "!")) {
            return Err(format!(
                "unsafe-sites.{path}:{}: an inner attribute cannot follow a per-item allow",
                tokens[cursor].line,
            ));
        }
        if !tokens.get(open).is_some_and(|token| punct(token, "[")) {
            break;
        }
        let Some(end) = matching_delimiter(tokens, open) else {
            return Err(format!(
                "unsafe-sites.{path}:{}: a trailing item attribute never closes",
                tokens[cursor].line,
            ));
        };
        cursor = end + 1;
    }
    let start = cursor;
    let line = tokens.get(start).map_or(1, |token| token.line);

    let Some(fn_position) = (start..tokens.len()).find(|position| ident(&tokens[*position], "fn"))
    else {
        return Err(format!(
            "unsafe-sites.{path}:{line}: the unsafe-code permission precedes no function; only \
             the current function-site boundary is supported",
        ));
    };
    for token in &tokens[start..fn_position] {
        let admitted = match &token.kind {
            TokenKind::Ident(_) => matches!(
                identifier_text(token),
                Some(
                    "pub"
                        | "crate"
                        | "self"
                        | "super"
                        | "in"
                        | "const"
                        | "async"
                        | "unsafe"
                        | "extern"
                )
            ),
            TokenKind::StringLiteral(_) => true,
            TokenKind::Punct(value) => matches!(value.as_str(), "(" | ")" | "::"),
        };
        if !admitted {
            return Err(format!(
                "unsafe-sites.{path}:{}: unsupported tokens precede `fn`; the permission may not \
                 name a function item",
                token.line,
            ));
        }
    }

    let mut delimiters = Vec::new();
    for position in start..tokens.len() {
        let token = &tokens[position];
        if punct(token, "{") && delimiters.is_empty() {
            let signature = render_signature(&tokens[start..position]);
            if signature.is_empty() {
                return Err(format!(
                    "unsafe-sites.{path}:{line}: the admitted function has an empty signature",
                ));
            }
            return Ok((signature, position));
        }
        if punct(token, ";") && delimiters.is_empty() {
            return Err(format!(
                "unsafe-sites.{path}:{line}: the admitted function has no body",
            ));
        }
        match punct_text(token) {
            Some("(") => delimiters.push(")"),
            Some("[") => delimiters.push("]"),
            Some("<") => delimiters.push(">"),
            Some(value @ (")" | "]" | ">")) => {
                let expected = delimiters.pop().ok_or_else(|| {
                    format!(
                        "unsafe-sites.{path}:{}: unmatched `{value}` in the admitted signature",
                        token.line,
                    )
                })?;
                if value != expected {
                    return Err(format!(
                        "unsafe-sites.{path}:{}: `{value}` closes a delimiter expecting \
                         `{expected}` in the admitted signature",
                        token.line,
                    ));
                }
            }
            _ => {}
        }
    }
    Err(format!(
        "unsafe-sites.{path}:{line}: the admitted function's body never begins",
    ))
}

/// Compares a scan with the exact admitted population.
pub(super) fn validate_pins(mut scan: Scan, admitted: &[AdmittedSite]) -> Vec<String> {
    let mut expected = Sites::new();
    for site in admitted {
        let key = (site.path.to_owned(), site.item.to_owned());
        assert!(
            expected.insert(key, site.reason.to_owned()).is_none(),
            "the admitted-site table repeats {} `{}`",
            site.path,
            site.item,
        );
    }
    assert!(
        !expected.is_empty(),
        "the admitted-site table is empty; an empty scan would pass vacuously",
    );

    for key in scan.sites.keys().filter(|key| !expected.contains_key(*key)) {
        scan.errors.push(format!(
            "unsafe-sites.{}: `{}` admits unsafe_code and is not pinned; ADR 0079 makes a new \
             site a new decision",
            key.0, key.1,
        ));
    }
    for key in expected.keys().filter(|key| !scan.sites.contains_key(*key)) {
        scan.errors.push(format!(
            "unsafe-sites.{}: pinned site `{}` is gone; remove its pin in the same reviewed \
             change that removes the permission",
            key.0, key.1,
        ));
    }
    for (key, found) in &scan.sites {
        if let Some(pinned) = expected.get(key)
            && found != pinned
        {
            scan.errors.push(format!(
                "unsafe-sites.{}: `{}` states reason {found:?}, pinned as {pinned:?}",
                key.0, key.1,
            ));
        }
    }
    scan.errors.sort();
    scan.errors
}

/// Holds the admitted compiler/std macro language to the exact currently used
/// population so a stale whitelist cannot silently become future expansion
/// authority.
pub(super) fn validate_builtin_populations(scan: &Scan) -> Vec<String> {
    let expected_macros: BTreeSet<String> = REOPENABLE_BUILTIN_MACROS
        .iter()
        .map(|name| (*name).to_owned())
        .collect();
    let expected_attributes: BTreeSet<String> = REOPENABLE_BUILTIN_ATTRIBUTES
        .iter()
        .map(|name| (*name).to_owned())
        .collect();
    let expected_derives: BTreeSet<String> = REOPENABLE_BUILTIN_DERIVES
        .iter()
        .map(|name| (*name).to_owned())
        .collect();
    let mut errors = Vec::new();
    if scan.builtin_macros != expected_macros {
        errors.push(format!(
            "unsafe-site macro census: used compiler/std macros differ from the exact admitted \
             set; found {:?}, expected {expected_macros:?}",
            scan.builtin_macros,
        ));
    }
    if scan.builtin_attributes != expected_attributes {
        errors.push(format!(
            "unsafe-site macro census: used built-in attributes differ from the exact admitted \
             set; found {:?}, expected {expected_attributes:?}",
            scan.builtin_attributes,
        ));
    }
    if scan.builtin_derives != expected_derives {
        errors.push(format!(
            "unsafe-site macro census: used built-in derives differ from the exact admitted set; \
             found {:?}, expected {expected_derives:?}",
            scan.builtin_derives,
        ));
    }
    errors
}

/// Holds workspace-owned macro producers and the facade route to their exact
/// current identities.
pub(super) fn validate_macro_authorities(scan: &Scan) -> Vec<String> {
    let expected_local: BTreeSet<(String, String)> = WORKSPACE_LOCAL_MACRO_RULES
        .iter()
        .map(|(path, name)| ((*path).to_owned(), (*name).to_owned()))
        .collect();
    let expected_exporters = BTreeSet::from(["#[proc_macro] pub fn tensor".to_owned()]);
    let mut errors = Vec::new();
    if scan.local_macro_rules != expected_local {
        errors.push(format!(
            "unsafe-site macro producer census: private macro_rules! definitions differ; found \
             {:?}, expected {expected_local:?}",
            scan.local_macro_rules,
        ));
    }
    if scan.proc_macro_exporters != expected_exporters {
        errors.push(format!(
            "unsafe-site proc-macro exporter census changed; found {:?}, expected \
             {expected_exporters:?}",
            scan.proc_macro_exporters,
        ));
    }
    if scan.facade_reexports != 1 {
        errors.push(format!(
            "unsafe-site proc-macro facade census: found {} exact \
             `pub use tiler_macros::tensor` re-export(s), expected 1",
            scan.facade_reexports,
        ));
    }
    if scan.facade_diagnostic_reexports != 1 {
        errors.push(format!(
            "unsafe-site compiler diagnostic facade census: found {} exact \
             `pub use core::compile_error as __tiler_compile_error` re-export(s), expected 1",
            scan.facade_diagnostic_reexports,
        ));
    }
    let expected_fixture_invocations: BTreeMap<String, usize> = TENSOR_FIXTURE_INVOCATION_PINS
        .iter()
        .map(|(path, count)| ((*path).to_owned(), *count))
        .collect();
    if scan.tensor_invocations != expected_fixture_invocations {
        errors.push(format!(
            "unsafe-site guarded fixture tensor invocation identities changed: {}",
            invocation_map_difference(&scan.tensor_invocations, &expected_fixture_invocations),
        ));
    }
    let fixture_invocations: usize = scan.tensor_invocations.values().sum();
    if fixture_invocations != TENSOR_FIXTURE_INVOCATION_COUNT {
        errors.push(format!(
            "unsafe-site guarded fixture tensor invocation census changed; found \
             {fixture_invocations}, expected {TENSOR_FIXTURE_INVOCATION_COUNT}",
        ));
    }
    let expected_rustdoc_invocations: BTreeMap<String, usize> = TENSOR_RUSTDOC_INVOCATION_PINS
        .iter()
        .map(|(path, count)| ((*path).to_owned(), *count))
        .collect();
    if scan.rustdoc_tensor_invocations != expected_rustdoc_invocations {
        errors.push(format!(
            "unsafe-site guarded rustdoc tensor invocation identities changed: {}",
            invocation_map_difference(
                &scan.rustdoc_tensor_invocations,
                &expected_rustdoc_invocations,
            ),
        ));
    }
    let rustdoc_invocations: usize = scan.rustdoc_tensor_invocations.values().sum();
    if rustdoc_invocations != TENSOR_RUSTDOC_INVOCATION_COUNT {
        errors.push(format!(
            "unsafe-site guarded rustdoc tensor invocation census changed; found \
             {rustdoc_invocations}, expected \
             {TENSOR_RUSTDOC_INVOCATION_COUNT}",
        ));
    }
    errors
}

/// Reports path additions, removals, and per-path count changes explicitly.
fn invocation_map_difference(
    found: &BTreeMap<String, usize>,
    expected: &BTreeMap<String, usize>,
) -> String {
    let keys: BTreeSet<&str> = found
        .keys()
        .chain(expected.keys())
        .map(String::as_str)
        .collect();
    keys.into_iter()
        .filter_map(|path| {
            let found_count = found.get(path);
            let expected_count = expected.get(path);
            (found_count != expected_count).then(|| {
                format!(
                    "{path}: found {}, expected {}",
                    found_count.map_or("absent".to_owned(), usize::to_string),
                    expected_count.map_or("absent".to_owned(), usize::to_string),
                )
            })
        })
        .collect::<Vec<_>>()
        .join("; ")
}

/// Lexes the Rust constructs relevant to attributes, dropping comments and
/// string-like prose before the lint name can be observed.
pub(super) fn lex(path: &str, source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut index = 0;
    let mut line = 1;

    while index < source.len() {
        let tail = &source[index..];
        if tail.starts_with("//") {
            if let Some(end) = tail.find('\n') {
                index += end;
            } else {
                break;
            }
            continue;
        }
        if tail.starts_with("/*") {
            let start_line = line;
            index += 2;
            let mut depth = 1_usize;
            while index < source.len() && depth != 0 {
                let rest = &source[index..];
                if rest.starts_with("/*") {
                    depth += 1;
                    index += 2;
                } else if rest.starts_with("*/") {
                    depth -= 1;
                    index += 2;
                } else {
                    let character = next_char(source, index);
                    if character == '\n' {
                        line += 1;
                    }
                    index += character.len_utf8();
                }
            }
            if depth != 0 {
                return Err(format!(
                    "unsafe-sites.{path}:{start_line}: a block comment never closes",
                ));
            }
            continue;
        }
        if let Some((end, newlines)) = raw_string_span(source, index) {
            tokens.push(Token {
                kind: TokenKind::Punct("<raw-string>".to_owned()),
                line,
            });
            line += newlines;
            index = end;
            continue;
        }

        if tail.starts_with("r#")
            && source
                .get(index + 2..)
                .and_then(|rest| rest.chars().next())
                .is_some_and(is_ident_start)
        {
            let start = index;
            index += 2;
            while index < source.len() {
                let next = next_char(source, index);
                if !is_ident_continue(next) {
                    break;
                }
                index += next.len_utf8();
            }
            tokens.push(Token {
                kind: TokenKind::Ident(source[start..index].to_owned()),
                line,
            });
            continue;
        }

        let character = next_char(source, index);
        if character.is_whitespace() {
            if character == '\n' {
                line += 1;
            }
            index += character.len_utf8();
            continue;
        }
        if character == '"' {
            let start_line = line;
            let (end, content, newlines) =
                ordinary_string(source, index).ok_or_else(|| {
                    format!(
                        "unsafe-sites.{path}:{start_line}: an ordinary string literal never closes",
                    )
                })?;
            tokens.push(Token {
                kind: TokenKind::StringLiteral(content),
                line,
            });
            line += newlines;
            index = end;
            continue;
        }
        if character == '\''
            && let Some(end) = character_literal_end(source, index)
        {
            line += source[index..end].matches('\n').count();
            index = end;
            continue;
        }
        if is_ident_start(character) {
            let start = index;
            index += character.len_utf8();
            while index < source.len() {
                let next = next_char(source, index);
                if !is_ident_continue(next) {
                    break;
                }
                index += next.len_utf8();
            }
            tokens.push(Token {
                kind: TokenKind::Ident(source[start..index].to_owned()),
                line,
            });
            continue;
        }

        let (punctuation, width) = if tail.starts_with("::") {
            ("::", 2)
        } else if tail.starts_with("->") {
            ("->", 2)
        } else if tail.starts_with("=>") {
            ("=>", 2)
        } else {
            (
                &source[index..index + character.len_utf8()],
                character.len_utf8(),
            )
        };
        tokens.push(Token {
            kind: TokenKind::Punct(punctuation.to_owned()),
            line,
        });
        index += width;
    }
    Ok(tokens)
}

/// The exclusive span and newline count of a raw string beginning at `start`.
pub(super) fn raw_string_span(source: &str, start: usize) -> Option<(usize, usize)> {
    let tail = &source[start..];
    let prefix = if tail.starts_with("br") || tail.starts_with("cr") {
        2
    } else if tail.starts_with('r') {
        1
    } else {
        return None;
    };
    let mut cursor = start + prefix;
    let mut hashes = 0;
    while source[cursor..].starts_with('#') {
        hashes += 1;
        cursor += 1;
    }
    if !source[cursor..].starts_with('"') {
        return None;
    }
    cursor += 1;
    let closing = format!("\"{}", "#".repeat(hashes));
    let rest = &source[cursor..];
    let relative = rest.find(&closing)?;
    let end = cursor + relative + closing.len();
    Some((end, source[start..end].matches('\n').count()))
}

/// An ordinary string's exclusive end, raw content, and newline count.
pub(super) fn ordinary_string(source: &str, start: usize) -> Option<(usize, String, usize)> {
    let mut cursor = start + 1;
    let content_start = cursor;
    let mut escaped = false;
    while cursor < source.len() {
        let character = next_char(source, cursor);
        if !escaped && character == '"' {
            let content = source[content_start..cursor].to_owned();
            let end = cursor + 1;
            return Some((end, content, source[start..end].matches('\n').count()));
        }
        escaped = !escaped && character == '\\';
        cursor += character.len_utf8();
    }
    None
}

/// The exclusive end of a character literal, or `None` for a lifetime tick.
pub(super) fn character_literal_end(source: &str, start: usize) -> Option<usize> {
    let mut cursor = start + 1;
    if cursor >= source.len() {
        return None;
    }
    let first = next_char(source, cursor);
    if first == '\\' {
        cursor += 1;
        if cursor >= source.len() {
            return None;
        }
        cursor += next_char(source, cursor).len_utf8();
    } else {
        cursor += first.len_utf8();
    }
    source[cursor..].starts_with('\'').then_some(cursor + 1)
}

/// The closing delimiter for the one opening token, respecting nesting.
pub(super) fn matching_delimiter(tokens: &[Token], open: usize) -> Option<usize> {
    let first = punct_text(tokens.get(open)?)?;
    let expected = match first {
        "(" => ")",
        "[" => "]",
        "{" => "}",
        _ => return None,
    };
    let mut stack = vec![expected];
    for (position, token) in tokens.iter().enumerate().skip(open + 1) {
        match punct_text(token) {
            Some("(") => stack.push(")"),
            Some("[") => stack.push("]"),
            Some("{") => stack.push("}"),
            Some(value @ (")" | "]" | "}")) => {
                if stack.pop()? != value {
                    return None;
                }
                if stack.is_empty() {
                    return Some(position);
                }
            }
            _ => {}
        }
    }
    None
}

/// Renders a stable, human-readable item signature from lexed tokens.
pub(super) fn render_signature(tokens: &[Token]) -> String {
    let mut rendered = String::new();
    let mut previous: Option<String> = None;
    for (index, token) in tokens.iter().enumerate() {
        let current = match &token.kind {
            TokenKind::Ident(value) => value.strip_prefix("r#").unwrap_or(value).to_owned(),
            TokenKind::Punct(value) => value.clone(),
            TokenKind::StringLiteral(value) => format!("\"{value}\""),
        };
        if current == "," && tokens.get(index + 1).is_some_and(|next| punct(next, ")")) {
            continue;
        }
        let tight_before = matches!(
            current.as_str(),
            ")" | "]" | ">" | "," | ";" | ":" | "::" | "(" | "[" | "<" | "."
        );
        let tight_after_previous = previous
            .as_deref()
            .is_some_and(|value| matches!(value, "(" | "[" | "<" | "::" | "&" | "'" | "."));
        if !rendered.is_empty() && !tight_before && !tight_after_previous {
            rendered.push(' ');
        }
        rendered.push_str(&current);
        previous = Some(current);
    }
    rendered
}

/// Reads a UTF-8 file or fails naming it.
pub(super) fn read(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{} is readable UTF-8: {error}", path.display()))
}

/// The non-comment part of one manifest line.
pub(super) fn manifest_code(line: &str) -> String {
    let mut code = String::new();
    let mut in_string = false;
    for character in line.chars() {
        match character {
            '"' => in_string = !in_string,
            '#' if !in_string => break,
            _ => {}
        }
        code.push(character);
    }
    code
}

/// Every double-quoted value in the root member array.
pub(super) fn quoted_values(array: &str, path: &Path, line: usize) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = None;
    for character in array.chars() {
        match (character, current.as_mut()) {
            ('"', None) => current = Some(String::new()),
            ('"', Some(_)) => values.push(current.take().expect("a member string is open")),
            (_, Some(value)) => value.push(character),
            (_, None) => {}
        }
    }
    assert!(
        current.is_none(),
        "{}:{line}: the member array contains an unterminated string",
        path.display(),
    );
    assert!(
        !values.is_empty(),
        "{}:{line}: the member array contains no string paths",
        path.display(),
    );
    values
}

/// The source character beginning at one byte boundary.
pub(super) fn next_char(source: &str, index: usize) -> char {
    source[index..]
        .chars()
        .next()
        .expect("the lexer index is inside the source")
}

/// Whether one character can begin an identifier relevant to this scan.
pub(super) fn is_ident_start(character: char) -> bool {
    character == '_' || unicode_ident::is_xid_start(character)
}

/// Whether one character can continue an identifier relevant to this scan.
pub(super) fn is_ident_continue(character: char) -> bool {
    unicode_ident::is_xid_continue(character)
}

/// Whether a token is one exact identifier.
pub(super) fn ident(token: &Token, expected: &str) -> bool {
    identifier_text(token) == Some(expected)
}

/// One identifier token's text.
pub(super) fn identifier_text(token: &Token) -> Option<&str> {
    match &token.kind {
        TokenKind::Ident(value) => Some(value.strip_prefix("r#").unwrap_or(value)),
        TokenKind::StringLiteral(_) | TokenKind::Punct(_) => None,
    }
}

/// Whether an identifier used Rust's raw `r#name` spelling.
pub(super) fn is_raw_identifier(token: &Token) -> bool {
    matches!(&token.kind, TokenKind::Ident(value) if value.starts_with("r#"))
}

/// Whether a token is one exact punctuation token.
pub(super) fn punct(token: &Token, expected: &str) -> bool {
    punct_text(token) == Some(expected)
}

/// One punctuation token's text.
pub(super) fn punct_text(token: &Token) -> Option<&str> {
    match &token.kind {
        TokenKind::Punct(value) => Some(value),
        TokenKind::Ident(_) | TokenKind::StringLiteral(_) => None,
    }
}
