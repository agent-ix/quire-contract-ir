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
        "crates/quire-contract-model/src/checked_package/common.rs",
        "decode_closed",
        "classifies a serde decode failure by its message text to choose between the unknown-member and malformed-wire refusals; the message is serde's own, not a wire vocabulary",
    ),
    (
        "crates/quire-contract-model/src/checked_package/v2/identity.rs",
        "locate_preimage_failure",
        "localises an error: reads the preimage's own `version` text to pick the decoder that names the member at fault; duplicates the preimage-version vocabulary and selects only a pointer, never an admission",
    ),
    (
        "crates/quire-contract-model/src/checked_package/v2/identity.rs",
        "locate_owner_failure",
        "localises an error: reads the owner's `kind` text to pick the member set that names the member at fault; selects only a pointer, never an admission",
    ),
    (
        "crates/quire-contract-model/src/checked_package/v2/mod.rs",
        "classify_dependency_entry_shape",
        "parses the reader's own refusal pointer to localise an error",
    ),
    (
        "crates/quire-contract-model/src/checked_package/v2/mod.rs",
        "is_typed_preimage_position",
        "parses the reader's own refusal pointer to localise an error",
    ),
    (
        "crates/quire-contract-model/src/checked_package/v2/operations.rs",
        "check_model_member",
        "compares two node-key digest texts; digests, not vocabulary",
    ),
    (
        "crates/quire-contract-model/src/checked_package/v2/operations.rs",
        "check_field_member",
        "looks a declared member up by its name; the name is the user's, no behaviour selected",
    ),
    (
        "crates/quire-contract-model/src/checked_package/v2/operations.rs",
        "record_field_type",
        "looks a record field up by its name; the name is the user's, no behaviour selected",
    ),
    (
        "crates/quire-contract-model/src/checked_package/v2/operations.rs",
        "type_pin",
        "finds the binding a type declares for a mode kind by comparing the binding's name with the decoded kind's wire text; a name lookup, no behaviour selected",
    ),
    (
        "crates/quire-contract-model/src/checked_package/v2/structural.rs",
        "binding",
        "looks a binding up by its name; the name is the user's, no behaviour selected",
    ),
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
            if literal_side
                || const_side
                || as_ref_both
                || derived_from_text(operand_left(left))
                || derived_from_text(operand_right(right))
            {
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
    // `let`/`if let`/`while let` with a string in the pattern.
    if let Some(at) = code.find("let ") {
        if let Some(eq) = code[at..].find(" = ") {
            if code[at..at + eq].contains('"') {
                return true;
            }
        }
    }
    // Membership of a string in a literal list, or a substring test.
    if (code.contains("].contains(") && code.contains('"')) || code.contains(".contains(\"") {
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

/// Whether an operand was read as text from the wire, or is a vocabulary
/// member's wire text, rather than a decoded enum.
fn derived_from_text(operand: &str) -> bool {
    operand.contains(".as_str()")
        || operand.contains("Value::as_str")
        || operand.contains(".as_wire()")
}

/// The tail of `left` that belongs to the comparison's left operand.
fn operand_left(left: &str) -> &str {
    let cut = ["&&", "||", "if ", "{", "return "]
        .iter()
        .filter_map(|separator| left.rfind(separator).map(|at| at + separator.len()))
        .max()
        .unwrap_or(0);
    &left[cut..]
}

/// The head of `right` that belongs to the comparison's right operand.
fn operand_right(right: &str) -> &str {
    let cut = ["&&", "||", " {", ";"]
        .iter()
        .filter_map(|separator| right.find(separator))
        .min()
        .unwrap_or(right.len());
    &right[..cut]
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

/// The type an `impl` header names, if `line` is one.
fn impl_type(line: &str) -> Option<String> {
    let rest = line.trim_start().strip_prefix("impl")?;
    if !(rest.starts_with(' ') || rest.starts_with('<')) {
        return None;
    }
    // Skip generics after `impl`, then take the type after `for` if present.
    let header = rest.split('{').next().unwrap_or(rest);
    let target = header.rsplit(" for ").next().unwrap_or(header);
    let target = if header.contains(" for ") {
        target
    } else if let Some(end) = header.rfind('>') {
        // `impl<T> Type` names the text after the generics.
        if header.trim_start().starts_with('<') {
            &header[end + 1..]
        } else {
            header
        }
    } else {
        header
    };
    let name: String = target
        .trim()
        .chars()
        .take_while(|c| !c.is_whitespace() && *c != '<' && *c != '{')
        .collect();
    (!name.is_empty()).then_some(name)
}

/// `Type::name` for a method, `name` for a free function; `index` is the
/// line of the function's declaration.
fn qualified_fn(index: usize, raw: &[&str], code: &[String], skipped: &[bool]) -> Option<String> {
    let name = fn_name(&code[index])?;
    let here = indent(raw[index]);
    let owner = (0..index).rev().find(|&up| {
        let trimmed = raw[up].trim_start();
        !skipped[up]
            && !trimmed.is_empty()
            && !trimmed.starts_with("//")
            && !trimmed.starts_with("#[")
            && !trimmed.starts_with('}')
            && indent(raw[up]) < here
    });
    match owner.and_then(|up| impl_type(raw[up])) {
        Some(owner) => Some(format!("{owner}::{name}")),
        None => Some(name),
    }
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
        if let Some(name) = qualified_fn(index, &raw, &code, &skipped) {
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
            qualified_fn(index, &raw, &code, &skipped)
        } else {
            (0..index)
                .rev()
                .find(|&up| !skipped[up] && fn_name(&code[up]).is_some() && indent(raw[up]) < here)
                .and_then(|up| qualified_fn(up, &raw, &code, &skipped))
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
    let mut scans = Vec::new();
    // Files that are `#[cfg(test)] mod x;` of the file that declares them.
    let mut test_files: BTreeSet<PathBuf> = BTreeSet::new();
    for path in &files {
        let relative = path
            .strip_prefix(root)
            .expect("under the root")
            .to_string_lossy()
            .replace('\\', "/");
        let source = fs::read_to_string(path).expect("source");
        let mut declared = BTreeSet::new();
        let scan = scan_source(&relative, &source, &mut declared);
        let dir = path.parent().expect("a file has a directory");
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        // `mod x;` in `a.rs` is `x.rs` beside it when `a` is `mod`/`lib`/`main`,
        // else `a/x.rs`; either may be `x/mod.rs`.
        let homes = if matches!(stem, "mod" | "lib" | "main") {
            vec![dir.to_path_buf()]
        } else {
            vec![dir.join(stem)]
        };
        for name in declared {
            for home in &homes {
                test_files.insert(home.join(format!("{name}.rs")));
                test_files.insert(home.join(&name).join("mod.rs"));
            }
        }
        scans.push((path.clone(), relative, scan));
    }
    let mut hits = Vec::new();
    let mut stale = Vec::new();
    for (path, relative, scan) in scans {
        if test_files.contains(&path) {
            continue;
        }
        stale.extend(edge_problems(&relative, &scan));
        hits.extend(scan.hits);
    }
    (hits, stale)
}

/// Markers with no reason, and markers on a function that reads no string.
fn edge_problems(file: &str, scan: &Scan) -> Vec<String> {
    let mut problems = Vec::new();
    for (func, (reason, used)) in &scan.edges {
        if reason.is_empty() {
            problems.push(format!(
                "{file}: `{func}` has a string-edge marker with no reason"
            ));
        }
        if !used {
            problems.push(format!(
                "{file}: `{func}` is marked string-edge but reads no string"
            ));
        }
    }
    problems
}

/// Splits hits into those the allow-list covers and those it does not, and
/// reports allow-list rows that matched nothing.
fn apply_allow_list(hits: Vec<Hit>, allowed: &[(&str, &str, &str)]) -> (Vec<Hit>, Vec<String>) {
    let mut used = BTreeSet::new();
    let mut remaining = Vec::new();
    for hit in hits {
        match allowed
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
    for (row, (file, func, reason)) in allowed.iter().enumerate() {
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
    let (unlisted, allow_problems) = apply_allow_list(hits, ALLOWED);
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
            "let-else with a string pattern",
            "fn f(x: &Value) {\n    let Some(\"reference\") = x.get(\"term\").and_then(Value::as_str) else {\n        return;\n    };\n}\n",
        ),
        (
            "if-let with a string pattern",
            "fn f(x: Option<&str>) {\n    if let Some(\"a\") = x {}\n}\n",
        ),
        (
            "text compared with an as_wire result",
            "fn f(x: &Value, k: Kind) -> bool {\n    x.get(\"kind\").and_then(Value::as_str) != Some(k.as_wire())\n}\n",
        ),
        (
            "as_str compared with a variable",
            "fn f(a: &String, b: &str) -> bool {\n    a.as_str() == b\n}\n",
        ),
        (
            "membership in a literal list",
            "fn f(k: &str) -> bool {\n    [\"a\", \"b\"].contains(&k)\n}\n",
        ),
        (
            "substring test against a literal",
            "fn f(k: &str) -> bool {\n    k.contains(\"a\")\n}\n",
        ),
        (
            "constant match arm",
            "fn f(k: &str) -> u8 {\n    match k {\n        VERSION => 1,\n        _ => 0,\n    }\n}\n",
        ),
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

/// Tracing: TC-048
/// ACs: FR-038-AC-34
#[trace("TC-048", "FR-038-AC-34")]
#[test]
fn tc_048_stale_allow_list_rows_and_empty_marker_reasons_fail() {
    let hit = |func: &str| Hit {
        file: "a.rs".to_owned(),
        line: 1,
        func: func.to_owned(),
        text: String::new(),
    };
    let allowed: &[(&str, &str, &str)] = &[
        ("a.rs", "read", "a user value"),
        ("a.rs", "gone", "a function that no longer reads a string"),
        ("a.rs", "silent", ""),
    ];
    let (unlisted, problems) = apply_allow_list(vec![hit("read"), hit("other")], allowed);
    assert_eq!(unlisted, vec![hit("other")], "an unlisted read is reported");
    assert!(
        problems
            .iter()
            .any(|p| p.contains("gone") && p.contains("matches no string read")),
        "a stale allow-list row is reported: {problems:?}"
    );
    assert!(
        problems
            .iter()
            .any(|p| p.contains("silent") && p.contains("no reason")),
        "an allow-list row with no reason is reported: {problems:?}"
    );

    let empty = scan_source(
        "case.rs",
        "// string-edge:\nfn f(k: &str) -> bool {\n    k == \"a\"\n}\n",
        &mut BTreeSet::new(),
    );
    let problems = edge_problems("case.rs", &empty);
    assert!(
        problems.iter().any(|p| p.contains("no reason")),
        "a marker with an empty reason is reported: {problems:?}"
    );
}

/// Tracing: TC-048
/// ACs: FR-038-AC-34
#[trace("TC-048", "FR-038-AC-34")]
#[test]
fn tc_048_a_marker_covers_its_own_method_not_a_namesake() {
    let source = "impl A {\n    // string-edge: decodes A\n    fn decode(k: &str) -> bool {\n        k == \"a\"\n    }\n}\nimpl B {\n    fn decode(k: &str) -> bool {\n        k == \"b\"\n    }\n}\n";
    let scan = scan_source("case.rs", source, &mut BTreeSet::new());
    let funcs: Vec<&str> = scan.hits.iter().map(|hit| hit.func.as_str()).collect();
    assert_eq!(
        funcs,
        ["B::decode"],
        "only the unmarked namesake is reported"
    );
}
