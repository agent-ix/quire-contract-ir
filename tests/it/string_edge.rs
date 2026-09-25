// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! IR-24: the string-edge gate.
//!
//! A closed vocabulary is decoded to an enum once, at the wire edge; past
//! that edge code matches the enum, exhaustively, so a new variant is a
//! compile error. This test fails when non-test source under `src/kani/` or
//! the model crate's `checked_package/` reads a wire string to decide
//! behaviour anywhere other than a function marked as a wire edge.
//!
//! What counts as a string read: `==`/`!=` against a string literal, a
//! constant or another `as_ref()`/`as_str()`; a `match` arm or `matches!`
//! pattern that is a string literal or constant; `starts_with`/`ends_with`/
//! `strip_prefix`/`strip_suffix`; and any call to a vocabulary's `from_wire`.
//!
//! Two ways to be permitted, both naming a reason:
//!
//! - a wire-edge function carries a `// string-edge: <reason>` comment
//!   directly above its `fn` (between its doc comment and its signature is
//!   fine): it is where the wire text is read and decoded;
//! - a comparison of a user value that selects no behaviour is listed in
//!   [`ALLOWED`] with its reason.
//!
//! A stale entry in either place (a marker or allow-list row whose function
//! no longer has a string read) also fails, so neither list can rot.

use ix_trace_rs::trace;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Directories scanned, relative to the repository root.
const ROOTS: &[&str] = &[
    "src/kani",
    "crates/quire-contract-model/src/checked_package",
];

/// Comparisons of a user value that select no behaviour: `(file, fn, reason)`.
const ALLOWED: &[(&str, &str, &str)] = &[
    (
        "crates/quire-contract-model/src/checked_package/v2/identity.rs",
        "integer_magnitude",
        "lexical check of a decimal integer's text (`0`, a leading `-`); the text is a number, not a vocabulary member",
    ),
    (
        "crates/quire-contract-model/src/checked_package/v2/identity.rs",
        "validate_dimension",
        "rejects a zero exponent by its text; the text is a number, not a vocabulary member",
    ),
    (
        "crates/quire-contract-model/src/checked_package/v2/identity.rs",
        "validate_unit",
        "rejects a zero scale numerator by its text; the text is a number, not a vocabulary member",
    ),
    (
        "crates/quire-contract-model/src/checked_package/v2/identity.rs",
        "is_rational",
        "equality of a rational's numerator and denominator text with a caller's numbers; user values, no behaviour selected",
    ),
    (
        "crates/quire-contract-model/src/checked_package/v2/structural.rs",
        "is_non_negative_integer",
        "lexical check of a decimal integer's text; the text is a number, not a vocabulary member",
    ),
    (
        "crates/quire-contract-model/src/checked_package/v2/structural.rs",
        "is_nonzero_integer",
        "lexical check of a decimal integer's text; the text is a number, not a vocabulary member",
    ),
    (
        "crates/quire-contract-model/src/checked_package/v2/operations.rs",
        "check_mode_type",
        "refuses when the wire mode's value text differs from the value the operand type pins; two opaque values compared, no behaviour selected by either",
    ),
    (
        "crates/quire-contract-model/src/checked_package/v2/operations.rs",
        "check_leaves",
        "reads the `field:<name>` leaf-path syntax and compares a leaf mode's value text with the value its field type pins; a path and opaque values, no vocabulary",
    ),
];

/// A string read found in source.
#[derive(Debug, Eq, PartialEq)]
struct Hit {
    file: String,
    line: usize,
    func: String,
    text: String,
}

/// Result of scanning one source text.
#[derive(Debug, Default)]
struct Scan {
    /// Unmarked string reads, keyed by enclosing function.
    hits: Vec<Hit>,
    /// Functions carrying a `string-edge:` marker, with whether they had a hit.
    edges: BTreeMap<String, (String, bool)>,
}

/// Blanks comments, and string/char literal contents when `keep_strings` is
/// false, carrying string state across lines.
fn strip(line: &str, in_string: &mut bool, keep_strings: bool) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if *in_string {
            if c == '\\' {
                if keep_strings {
                    out.push(c);
                    if let Some(next) = chars.get(i + 1) {
                        out.push(*next);
                    }
                }
                i += 2;
                continue;
            }
            if c == '"' {
                *in_string = false;
                out.push('"');
            } else if keep_strings {
                out.push(c);
            }
            i += 1;
            continue;
        }
        if c == '/' && chars.get(i + 1) == Some(&'/') {
            break;
        }
        if c == '"' {
            *in_string = true;
            out.push('"');
            i += 1;
            continue;
        }
        if c == '\'' {
            // A char literal is `'x'` or `'\..'`; a lifetime is not closed.
            let close = if chars.get(i + 1) == Some(&'\\') {
                chars[i + 2..]
                    .iter()
                    .position(|&ch| ch == '\'')
                    .map(|p| i + 2 + p)
            } else if chars.get(i + 2) == Some(&'\'') {
                Some(i + 2)
            } else {
                None
            };
            if let Some(close) = close {
                out.push_str("''");
                i = close + 1;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    out
}

fn is_const_ident(word: &str) -> bool {
    word.len() >= 2
        && word.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        && word
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
}

fn leading_word(text: &str) -> &str {
    let end = text
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(text.len());
    &text[..end]
}

/// Whether `code` (comments stripped, literals kept) reads a string.
fn reads_a_string(code: &str) -> bool {
    let trimmed = code.trim();
    // Comparison operators.
    for op in ["==", "!="] {
        let mut from = 0;
        while let Some(found) = code[from..].find(op) {
            let at = from + found;
            let left = code[..at].trim_end();
            let right = code[at + 2..].trim_start();
            from = at + 2;
            if right.starts_with('=') {
                continue;
            }
            let literal_side = right.starts_with('"')
                || right.starts_with("Some(\"")
                || left.ends_with('"')
                || left.ends_with("\")");
            let const_side = is_const_ident(leading_word(right));
            let as_ref_both = (left.ends_with(".as_ref()") || left.ends_with(".as_str()"))
                && (right.contains(".as_ref()") || right.contains(".as_str()"));
            if literal_side || const_side || as_ref_both {
                return true;
            }
        }
    }
    // Match arms and `matches!` patterns.
    if trimmed.contains("matches!(") && trimmed.contains('"') {
        return true;
    }
    let arm_start = trimmed.starts_with('"')
        || trimmed.starts_with("Some(\"")
        || trimmed.starts_with("| \"")
        || is_const_ident(leading_word(trimmed));
    if arm_start && (trimmed.contains("=>") || trimmed.ends_with('|') || trimmed.starts_with("| "))
    {
        // A constant-headed line is only an arm when it has the arrow.
        if !is_const_ident(leading_word(trimmed)) || trimmed.contains("=>") {
            return true;
        }
    }
    if trimmed.starts_with("match ")
        && (trimmed.contains(".as_str()") || trimmed.contains("Value::as_str"))
    {
        return true;
    }
    // Prefix/suffix tests and decode calls.
    [
        ".starts_with(",
        ".ends_with(",
        ".strip_prefix(",
        ".strip_suffix(",
        "from_wire(",
    ]
    .iter()
    .any(|needle| code.contains(needle))
}

/// The name of the function declared on `line`, if any.
fn fn_name(line: &str) -> Option<String> {
    let at = line.find("fn ")?;
    let before = &line[..at];
    if !(before.is_empty() || before.ends_with(' ') || before.ends_with('(')) {
        return None;
    }
    let name = leading_word(&line[at + 3..]);
    (!name.is_empty()).then(|| name.to_owned())
}

fn indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

/// Scans one source text; `test_module_names` receives each out-of-line
/// `#[cfg(test)] mod name;` it declares.
fn scan_source(file: &str, source: &str, test_module_names: &mut BTreeSet<String>) -> Scan {
    let raw: Vec<&str> = source.lines().collect();
    let mut in_string = false;
    let mut in_string_blank = false;
    let mut code: Vec<String> = Vec::new();
    let mut blank: Vec<String> = Vec::new();
    for line in &raw {
        code.push(strip(line, &mut in_string, true));
        blank.push(strip(line, &mut in_string_blank, false));
    }

    // Lines belonging to `#[cfg(test)]` items are not production code.
    let mut skipped = vec![false; raw.len()];
    let mut n = 0;
    while n < raw.len() {
        if !raw[n].trim_start().starts_with("#[cfg(test") {
            n += 1;
            continue;
        }
        let start = n;
        let mut item = n + 1;
        while item < raw.len() && raw[item].trim_start().starts_with("#[") {
            item += 1;
        }
        let header = raw.get(item).map_or("", |line| line.trim());
        if let Some(rest) = header
            .strip_prefix("mod ")
            .or_else(|| header.strip_prefix("pub mod "))
        {
            if let Some(name) = rest.strip_suffix(';') {
                test_module_names.insert(name.trim().to_owned());
            }
        }
        let mut depth: i64 = 0;
        let mut opened = false;
        let mut end = item;
        while end < raw.len() {
            for c in blank[end].chars() {
                match c {
                    '{' => {
                        depth += 1;
                        opened = true;
                    }
                    '}' => depth -= 1,
                    _ => {}
                }
            }
            let done = (opened && depth <= 0) || (!opened && blank[end].trim_end().ends_with(';'));
            if done {
                break;
            }
            end += 1;
        }
        for flag in skipped
            .iter_mut()
            .take(end.min(raw.len() - 1) + 1)
            .skip(start)
        {
            *flag = true;
        }
        n = end + 1;
    }

    let mut scan = Scan::default();
    let mut marked: BTreeMap<String, String> = BTreeMap::new();
    let mut hit_functions: BTreeSet<String> = BTreeSet::new();
    for (index, text) in code.iter().enumerate() {
        if skipped[index] {
            continue;
        }
        // Register a wire-edge marker for each function that carries one.
        if let Some(name) = fn_name(text) {
            let mut up = index;
            while up > 0 {
                let above = raw[up - 1].trim_start();
                if above.starts_with("///") || above.starts_with("//") || above.starts_with("#[") {
                    if let Some(reason) = above.split("string-edge:").nth(1) {
                        marked.insert(name.clone(), reason.trim().to_owned());
                    }
                    up -= 1;
                } else {
                    break;
                }
            }
        }
        if !reads_a_string(text) {
            continue;
        }
        let here = indent(raw[index]);
        let func = if fn_name(text).is_some() {
            fn_name(text)
        } else {
            (0..index)
                .rev()
                .find(|&up| !skipped[up] && fn_name(&code[up]).is_some() && indent(raw[up]) < here)
                .and_then(|up| fn_name(&code[up]))
        }
        .unwrap_or_else(|| "<item>".to_owned());
        hit_functions.insert(func.clone());
        if marked.contains_key(&func) {
            continue;
        }
        scan.hits.push(Hit {
            file: file.to_owned(),
            line: index + 1,
            func,
            text: raw[index].trim().to_owned(),
        });
    }
    for (func, reason) in marked {
        let used = hit_functions.contains(&func);
        scan.edges.insert(func, (reason, used));
    }
    scan
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("read {}: {error}", dir.display()))
        .map(|entry| entry.expect("directory entry").path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// Every unmarked string read in `ROOTS`, plus stale markers.
fn scan_repository() -> (Vec<Hit>, Vec<String>) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for dir in ROOTS {
        collect_files(&root.join(dir), &mut files);
    }
    let mut test_modules = BTreeSet::new();
    let mut scans = Vec::new();
    for path in &files {
        let relative = path
            .strip_prefix(root)
            .expect("under the root")
            .to_string_lossy()
            .replace('\\', "/");
        let source = fs::read_to_string(path).expect("source");
        scans.push((
            path.clone(),
            relative.clone(),
            scan_source(&relative, &source, &mut test_modules),
        ));
    }
    let mut hits = Vec::new();
    let mut stale = Vec::new();
    for (path, relative, scan) in scans {
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if stem == "tests" || test_modules.contains(stem) {
            continue;
        }
        for (func, (reason, used)) in scan.edges {
            if reason.is_empty() {
                stale.push(format!(
                    "{relative}: `{func}` has a string-edge marker with no reason"
                ));
            }
            if !used {
                stale.push(format!(
                    "{relative}: `{func}` is marked string-edge but reads no string"
                ));
            }
        }
        hits.extend(scan.hits);
    }
    (hits, stale)
}

/// Splits hits into those the allow-list covers and those it does not, and
/// reports allow-list rows that matched nothing.
fn apply_allow_list(hits: Vec<Hit>) -> (Vec<Hit>, Vec<String>) {
    let mut used = BTreeSet::new();
    let mut remaining = Vec::new();
    for hit in hits {
        match ALLOWED
            .iter()
            .position(|(file, func, _)| *file == hit.file && *func == hit.func)
        {
            Some(row) => {
                used.insert(row);
            }
            None => remaining.push(hit),
        }
    }
    let mut stale = Vec::new();
    for (row, (file, func, reason)) in ALLOWED.iter().enumerate() {
        if reason.trim().is_empty() {
            stale.push(format!("allow-list row {file}::{func} has no reason"));
        }
        if !used.contains(&row) {
            stale.push(format!(
                "allow-list row {file}::{func} matches no string read"
            ));
        }
    }
    (remaining, stale)
}

/// Tracing: TC-048
/// ACs: FR-038-AC-34
#[trace("TC-048", "FR-038-AC-34")]
#[test]
fn tc_048_no_wire_string_is_matched_after_intake() {
    let (hits, marker_problems) = scan_repository();
    let (unlisted, allow_problems) = apply_allow_list(hits);
    let mut report = String::new();
    for hit in &unlisted {
        report.push_str(&format!(
            "{}:{} in `{}`: {}\n",
            hit.file, hit.line, hit.func, hit.text
        ));
    }
    for problem in marker_problems.iter().chain(&allow_problems) {
        report.push_str(problem);
        report.push('\n');
    }
    assert!(
        report.is_empty(),
        "a wire string is read after intake, or a string-edge marker / allow-list row is \
         stale. Decode it to its enum at the wire edge, or mark the edge function \
         `// string-edge: <reason>`:\n{report}"
    );
}

/// Tracing: TC-048
/// ACs: FR-038-AC-34
#[trace("TC-048", "FR-038-AC-34")]
#[test]
fn tc_048_the_scan_detects_each_kind_of_string_read() {
    let cases: &[(&str, &str)] = &[
        ("comparison to a literal", "fn f(k: &str) -> bool {\n    k == \"a\"\n}\n"),
        ("comparison to a constant", "fn f(k: &str) -> bool {\n    k != VERSION\n}\n"),
        (
            "two as_ref reads",
            "fn f(a: &Box<str>, b: &Box<str>) -> bool {\n    a.as_ref() == b.as_ref()\n}\n",
        ),
        (
            "match arm",
            "fn f(k: &str) -> u8 {\n    match k {\n        \"a\" => 1,\n        _ => 0,\n    }\n}\n",
        ),
        (
            "or-pattern arm",
            "fn f(k: &str) -> u8 {\n    match k {\n        \"a\" | \"b\" => 1,\n        _ => 0,\n    }\n}\n",
        ),
        (
            "matches! pattern",
            "fn f(k: &str) -> bool {\n    matches!(k, \"a\" | \"b\")\n}\n",
        ),
        ("prefix test", "fn f(k: &str) -> bool {\n    k.starts_with(\"x\")\n}\n"),
        ("downstream decode", "fn f(k: &str) {\n    let _ = Role::from_wire(k);\n}\n"),
        (
            "Option-wrapped literal",
            "fn f(k: Option<&str>) -> bool {\n    k != Some(\"a\")\n}\n",
        ),
    ];
    for (name, source) in cases {
        let scan = scan_source("case.rs", source, &mut BTreeSet::new());
        assert!(!scan.hits.is_empty(), "the scan missed a {name}");
        assert_eq!(scan.hits[0].func, "f", "the {name} names its function");
    }
}

/// Tracing: TC-048
/// ACs: FR-038-AC-34
#[trace("TC-048", "FR-038-AC-34")]
#[test]
fn tc_048_the_scan_ignores_enums_comments_tests_and_marked_edges() {
    let quiet: &[(&str, &str)] = &[
        (
            "enum comparison",
            "fn f(a: Kind, b: Kind) -> bool {\n    a == Kind::X && Some(a) != None\n}\n",
        ),
        (
            "comment and log text",
            "fn f() {\n    // k == \"a\" in prose\n    let _ = \"a == b\";\n}\n",
        ),
        (
            "cfg(test) module",
            "#[cfg(test)]\nmod tests {\n    fn f(k: &str) -> bool {\n        k == \"a\"\n    }\n}\n",
        ),
        (
            "marked edge",
            "// string-edge: reads the wire tag\nfn f(k: &str) -> bool {\n    k == \"a\"\n}\n",
        ),
        (
            "marked edge under a doc comment",
            "/// Docs.\n// string-edge: reads the wire tag\n#[inline]\nfn f(k: &str) -> bool {\n    k == \"a\"\n}\n",
        ),
    ];
    for (name, source) in quiet {
        let scan = scan_source("case.rs", source, &mut BTreeSet::new());
        assert!(
            scan.hits.is_empty(),
            "the scan flagged {name}: {:?}",
            scan.hits
        );
    }
    let mut modules = BTreeSet::new();
    scan_source("case.rs", "#[cfg(test)]\nmod vectors;\n", &mut modules);
    assert!(
        modules.contains("vectors"),
        "an out-of-line test module is recorded"
    );
    let marked = scan_source(
        "case.rs",
        "// string-edge: reads the wire tag\nfn f() {}\n",
        &mut BTreeSet::new(),
    );
    assert_eq!(
        marked.edges.get("f"),
        Some(&("reads the wire tag".to_owned(), false)),
        "a marker on a function that reads no string is reported as unused"
    );
}
