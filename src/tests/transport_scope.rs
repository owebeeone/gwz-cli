//! The transport release plan's amendment 2, TR2.11, with its verdict's
//! Safety-1 residual: the requests `transport_meta` runs inside
//! `with_local_transport` are exactly the operations whose gwz-core handler
//! scopes a transport with `with_transport`. Since TR2.11 a backend without a
//! host context takes libgit2's native route, so a network operation missing
//! from those arms would leave the transport silently; an extra arm builds a
//! runtime for an operation that opens no connection.
//!
//! The test reads source text, gwz-core's from the checkout beside this one, so
//! it runs in the ordinary build and still covers the arms that only the
//! candidate switch compiles.
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[test]
fn transport_meta_arms_equal_the_operations_that_call_with_transport() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dispatch =
        code_only(&std::fs::read_to_string(root.join("src/globalargs/dispatch.rs")).unwrap());
    let arms = names_after(
        braced_after(&dispatch, "fn transport_meta("),
        "CliRequest::",
    );
    let core = root.join("../gwz-core/src");
    let mut operations = BTreeSet::new();
    for file in production_files(&core) {
        let text = without_test_code(code_only(&std::fs::read_to_string(&file).unwrap()));
        for (call, _) in text.match_indices(".with_transport(") {
            // A handler takes its operation's `request: crate::<Kind>Request`,
            // and `CliRequest` names its variants by the same kinds.
            let kinds: BTreeSet<String> =
                names_after(&text[enclosing_fn(&text, call)..call], "request: crate::")
                    .iter()
                    .filter_map(|name| name.strip_suffix("Request"))
                    .map(str::to_owned)
                    .collect();
            assert_eq!(
                kinds.len(),
                1,
                "{}: a with_transport call outside one operation's handler: {kinds:?}",
                file.display()
            );
            operations.extend(kinds);
        }
    }
    assert!(
        !operations.is_empty(),
        "no with_transport call found under {}",
        core.display()
    );
    assert_eq!(
        arms, operations,
        "transport_meta's arms (left) differ from the operations whose gwz-core handler \
         calls with_transport (right)"
    );
}

/// The text inside the first braced block after `marker`.
fn braced_after<'a>(text: &'a str, marker: &str) -> &'a str {
    let start = text
        .find(marker)
        .unwrap_or_else(|| panic!("no {marker} in the source"));
    let open = start + text[start..].find('{').expect("a block after the marker");
    &text[open + 1..block_end(text, open)]
}

/// The index of the brace that closes the block opened at `open`.
fn block_end(text: &str, open: usize) -> usize {
    let mut depth = 0usize;
    for (index, byte) in text.bytes().enumerate().skip(open) {
        if byte == b'{' {
            depth += 1;
        } else if byte == b'}' {
            depth -= 1;
            if depth == 0 {
                return index;
            }
        }
    }
    panic!("unbalanced braces after byte {open}");
}

/// Each identifier that directly follows `prefix` in `text`.
fn names_after(text: &str, prefix: &str) -> BTreeSet<String> {
    text.match_indices(prefix)
        .map(|(at, _)| {
            text[at + prefix.len()..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect()
        })
        .collect()
}

/// Where the last function item that starts before `at` begins.
fn enclosing_fn(text: &str, at: usize) -> usize {
    text[..at]
        .match_indices("fn ")
        .map(|(index, _)| index)
        .filter(|index| {
            *index == 0 || text[..*index].ends_with(|c: char| c.is_whitespace() || c == ')')
        })
        .last()
        .expect("a with_transport call inside a function")
}

/// gwz-core's Rust sources, less those named as tests: any file or directory
/// whose name contains `test`.
fn production_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if name.contains("test") {
            continue;
        }
        if path.is_dir() {
            files.extend(production_files(&path));
        } else if name.ends_with(".rs") {
            files.push(path);
        }
    }
    files
}

/// `code` without the blocks that `#[cfg(test)]` or `#[cfg(all(test, ...))]`
/// gates, `cfg_if!` branches among them. A gated item that ends at `;` before
/// any block keeps its text.
fn without_test_code(mut code: String) -> String {
    for marker in ["#[cfg(test)]", "#[cfg(all(test,"] {
        while let Some(at) = code.find(marker) {
            let rest = &code[at..];
            match (rest.find('{'), rest.find(';')) {
                (Some(brace), semicolon) if semicolon.is_none_or(|end| brace < end) => {
                    let end = block_end(&code, at + brace);
                    code.replace_range(at..=end, "");
                }
                _ => {
                    code.replace_range(at..at + marker.len(), "");
                }
            }
        }
    }
    code
}

/// `text` with every comment and every string, byte string or character
/// literal blanked to spaces, byte for byte, so that offsets, braces and
/// searches see only code.
fn code_only(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut code = bytes.to_vec();
    let mut at = 0;
    while at < bytes.len() {
        let rest = &bytes[at..];
        let blanked = if rest.starts_with(b"//") {
            Some(at + rest.iter().position(|&b| b == b'\n').unwrap_or(rest.len()))
        } else if rest.starts_with(b"/*") {
            let close = rest[2..].windows(2).position(|w| w == b"*/");
            Some(at + 2 + close.map_or(rest.len() - 2, |end| end + 2))
        } else if bytes[at] == b'"' {
            let mut end = at + 1;
            while end < bytes.len() && bytes[end] != b'"' {
                end += if bytes[end] == b'\\' { 2 } else { 1 };
            }
            Some(end + 1)
        } else if bytes[at] == b'\'' {
            char_literal_end(text, at)
        } else {
            raw_string_end(bytes, at)
        };
        match blanked {
            Some(end) => {
                let end = end.min(bytes.len());
                for byte in &mut code[at..end] {
                    if *byte != b'\n' {
                        *byte = b' ';
                    }
                }
                at = end;
            }
            None => {
                at += 1;
            }
        }
    }
    String::from_utf8(code).expect("literals blank whole characters")
}

/// The end of a raw string literal (`r"…"`, `r#"…"#`, `br"…"`) that starts at
/// `at`, if one does.
fn raw_string_end(bytes: &[u8], at: usize) -> Option<usize> {
    if at > 0 && (bytes[at - 1].is_ascii_alphanumeric() || bytes[at - 1] == b'_') {
        return None;
    }
    let start = at + usize::from(bytes[at] == b'b');
    if bytes.get(start) != Some(&b'r') {
        return None;
    }
    let hashes = bytes[start + 1..]
        .iter()
        .take_while(|&&b| b == b'#')
        .count();
    if bytes.get(start + 1 + hashes) != Some(&b'"') {
        return None;
    }
    let mut close = vec![b'"'];
    close.extend(std::iter::repeat_n(b'#', hashes));
    let body = start + 2 + hashes;
    let found = bytes[body..]
        .windows(close.len())
        .position(|w| w == close.as_slice());
    Some(found.map_or(bytes.len(), |end| body + end + close.len()))
}

/// The end of a character literal that starts at `at`, or `None` for a
/// lifetime or a label.
fn char_literal_end(text: &str, at: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    if bytes.get(at + 1) == Some(&b'\\') {
        let close = bytes.get(at + 3..)?.iter().position(|&b| b == b'\'')?;
        return Some(at + 3 + close + 1);
    }
    let width = text[at + 1..].chars().next()?.len_utf8();
    (bytes.get(at + 1 + width) == Some(&b'\'')).then_some(at + 2 + width)
}
