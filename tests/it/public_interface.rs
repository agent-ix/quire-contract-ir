use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

use ix_trace_rs::trace;
use quire_contract_model::{
    derive_structural_candidates, CheckedSemanticNodeV2, StructuralCandidateError,
};
use serde_json::{json, Value};

const MODEL_ROOT: &str = include_str!("../../crates/quire-contract-model/src/lib.rs");
const BRIDGE_ROOT: &str = include_str!("../../src/lib.rs");
const FR_019: &str = include_str!("../../spec/model/functional/FR-019-rust-library-interface.md");

/// Forbidden single terms, matched inside any one word of a name.
const FORBIDDEN_PUBLIC_NAME_TERMS: [&str; 16] = [
    "rust",
    "gumbo",
    "aadl",
    "hamr",
    "solver",
    "runtime",
    "usize",
    "isize",
    "pathbuf",
    "filesystem",
    "filepath",
    "ospath",
    "pointerwidth",
    "linux",
    "windows",
    "macos",
];

/// Forbidden adjacent word pairs, so `file_path`, `filePath` and `FilePath`
/// match while `profile_path` does not.
const FORBIDDEN_PUBLIC_NAME_PAIRS: [[&str; 2]; 4] = [
    ["file", "path"],
    ["os", "path"],
    ["file", "system"],
    ["pointer", "width"],
];

/// Lowercase words of a name, split on separators and camelCase boundaries.
fn name_words(name: &str) -> Vec<String> {
    let characters = name.chars().collect::<Vec<_>>();
    let mut words = Vec::<String>::new();
    let mut current = String::new();
    for (index, character) in characters.iter().copied().enumerate() {
        if !character.is_ascii_alphanumeric() {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            continue;
        }
        if character.is_ascii_uppercase() && !current.is_empty() {
            let previous = characters[index - 1];
            let next_is_lower = characters
                .get(index + 1)
                .is_some_and(char::is_ascii_lowercase);
            if !previous.is_ascii_uppercase() || next_is_lower {
                words.push(std::mem::take(&mut current));
            }
        }
        current.push(character.to_ascii_lowercase());
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

fn portable_name_violations<I, S>(names: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut violations = Vec::new();
    for name in names {
        let name = name.as_ref();
        let words = name_words(name);
        for term in FORBIDDEN_PUBLIC_NAME_TERMS {
            if words.iter().any(|word| word.contains(term)) {
                violations.push(format!("{name}: {term}"));
            }
        }
        for pair in FORBIDDEN_PUBLIC_NAME_PAIRS {
            if words
                .windows(2)
                .any(|window| window[0] == pair[0] && window[1] == pair[1])
            {
                violations.push(format!("{name}: {}_{}", pair[0], pair[1]));
            }
        }
    }
    violations
}

/// Source text with comments removed, so a commented-out declaration is
/// neither inventoried nor accepted. String, raw-string and character
/// literals are copied through whole, so comment markers inside them hide
/// nothing; lifetimes are told from character literals by the closing quote.
fn strip_comments(source: &str) -> String {
    let chars = source.chars().collect::<Vec<_>>();
    let mut out = String::with_capacity(source.len());
    let mut index = 0;
    while index < chars.len() {
        let character = chars[index];
        let next = chars.get(index + 1).copied();
        let previous_is_identifier =
            index > 0 && (chars[index - 1].is_ascii_alphanumeric() || chars[index - 1] == '_');
        if character == '/' && next == Some('/') {
            while index < chars.len() && chars[index] != '\n' {
                index += 1;
            }
        } else if character == '/' && next == Some('*') {
            let mut depth = 0_usize;
            while index < chars.len() {
                if chars[index] == '/' && chars.get(index + 1) == Some(&'*') {
                    depth += 1;
                    index += 2;
                } else if chars[index] == '*' && chars.get(index + 1) == Some(&'/') {
                    depth -= 1;
                    index += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    index += 1;
                }
            }
            out.push(' ');
        } else if character == 'r' && !previous_is_identifier && raw_string_len(&chars[index..]) > 0
        {
            let length = raw_string_len(&chars[index..]);
            out.extend(&chars[index..index + length]);
            index += length;
        } else if character == '"' {
            let start = index;
            index += 1;
            while index < chars.len() && chars[index] != '"' {
                index += if chars[index] == '\\' { 2 } else { 1 };
            }
            index = (index + 1).min(chars.len());
            out.extend(&chars[start..index]);
        } else if character == '\'' {
            let literal_end = match (next, chars.get(index + 2)) {
                (Some('\\'), _) => chars[index + 3..]
                    .iter()
                    .position(|candidate| *candidate == '\'')
                    .map(|offset| index + 3 + offset + 1),
                (Some(_), Some('\'')) => Some(index + 3),
                _ => None,
            };
            let end = literal_end.unwrap_or(index + 1);
            out.extend(&chars[index..end]);
            index = end;
        } else {
            out.push(character);
            index += 1;
        }
    }
    out
}

/// Length of a raw string literal (`r"..."`, `r#"..."#`) starting at `rest[0]`,
/// or zero when `rest` does not start one.
fn raw_string_len(rest: &[char]) -> usize {
    let hashes = rest[1..].iter().take_while(|c| **c == '#').count();
    if rest.get(1 + hashes) != Some(&'"') {
        return 0;
    }
    let body = &rest[2 + hashes..];
    for (offset, character) in body.iter().enumerate() {
        if *character == '"'
            && body[offset + 1..].iter().take_while(|c| **c == '#').count() >= hashes
        {
            return 2 + hashes + offset + 1 + hashes;
        }
    }
    rest.len()
}

/// Every top-level `pub ...` statement of a comment-stripped module: a `pub
/// use` runs to its semicolon, any other declaration is its first line.
fn public_statements(source: &str) -> Vec<String> {
    let source = strip_comments(source);
    let mut statements = Vec::new();
    let mut lines = source.lines().peekable();
    while let Some(line) = lines.next() {
        if !line.starts_with("pub ") {
            continue;
        }
        let mut statement = line.to_owned();
        if line.starts_with("pub use ") {
            while !statement.contains(';') {
                statement.push(' ');
                statement.push_str(lines.next().expect("pub use must end with a semicolon"));
            }
        }
        statements.push(statement);
    }
    statements
}

/// Names a `pub use` statement exports; globs, nested groups and any path
/// outside `crate::checked_package` are refused, an alias exports its alias.
fn selected_population_export_names(statement: &str) -> Vec<String> {
    let path = statement
        .strip_prefix("pub use crate::checked_package::")
        .and_then(|path| path.trim_end().strip_suffix(';'))
        .unwrap_or_else(|| panic!("unrecognized selected-population re-export: {statement}"));
    let list = path
        .strip_prefix('{')
        .map(|list| {
            list.strip_suffix('}')
                .unwrap_or_else(|| panic!("unclosed export list: {statement}"))
        })
        .unwrap_or(path);
    list.split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(|entry| {
            let exported = entry.rsplit(" as ").next().unwrap_or(entry).trim();
            assert!(
                !exported.is_empty()
                    && exported
                        .chars()
                        .all(|character| character.is_ascii_alphanumeric() || character == '_'),
                "unrecognized selected-population re-export form `{entry}`: {statement}"
            );
            exported.to_owned()
        })
        .collect()
}

/// The declared name of a `pub` item, skipping function and static qualifiers.
fn public_item_name(declaration: &str) -> String {
    let mut tokens = declaration.split_whitespace().skip(1).peekable();
    while let Some(token) = tokens.peek().copied() {
        let qualifier = matches!(token, "async" | "unsafe" | "extern")
            || token.starts_with('"')
            || (token == "const"
                && matches!(
                    tokens.clone().nth(1),
                    Some("fn" | "unsafe" | "async" | "extern")
                ));
        if !qualifier {
            break;
        }
        tokens.next();
    }
    let kind = tokens.next();
    assert!(
        matches!(
            kind,
            Some("struct" | "enum" | "trait" | "fn" | "type" | "const" | "static" | "union")
        ),
        "unrecognized public source-obligation declaration: {declaration}"
    );
    if kind == Some("static") && tokens.peek() == Some(&"mut") {
        tokens.next();
    }
    tokens
        .next()
        .and_then(|name| {
            name.split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
                .next()
                .filter(|name| !name.is_empty())
        })
        .unwrap_or_else(|| panic!("public declaration has no name: {declaration}"))
        .to_owned()
}

fn collect_schema_names(value: &Value, names: &mut Vec<String>) {
    match value {
        Value::Object(object) => {
            for field in ["properties", "$defs", "definitions"] {
                if let Some(entries) = object.get(field).and_then(Value::as_object) {
                    names.extend(entries.keys().cloned());
                }
            }
            if let Some(title) = object.get("title").and_then(Value::as_str) {
                names.push(title.to_owned());
            }
            if let Some(variants) = object.get("enum").and_then(Value::as_array) {
                names.extend(variants.iter().filter_map(Value::as_str).map(str::to_owned));
            }
            for child in object.values() {
                collect_schema_names(child, names);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_schema_names(item, names);
            }
        }
        _ => {}
    }
}

fn public_schema_names() -> Vec<Vec<String>> {
    let schemas = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("schemas");
    let mut paths = fs::read_dir(schemas)
        .expect("schema directory must be readable")
        .map(|entry| entry.expect("schema entry must be readable").path())
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| name.to_string_lossy().ends_with(".schema.json"))
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let mut names = vec![path
                .file_name()
                .expect("schema path has a filename")
                .to_string_lossy()
                .into_owned()];
            let schema: Value =
                serde_json::from_slice(&fs::read(path).expect("schema must be readable"))
                    .expect("public schema must be JSON");
            collect_schema_names(&schema, &mut names);
            names
        })
        .collect()
}

fn quoted_names(text: &str) -> impl Iterator<Item = &str> {
    text.split('`').skip(1).step_by(2)
}

fn expected_items() -> BTreeMap<String, BTreeSet<String>> {
    let (_, after_heading) = FR_019
        .split_once("### Public items")
        .expect("FR-019 public items table must exist");
    let (table, _) = after_heading
        .split_once("## Acceptance Criteria")
        .expect("FR-019 acceptance criteria must follow the table");
    let mut items = BTreeMap::<String, BTreeSet<String>>::new();
    for row in table.lines().filter(|line| line.starts_with("| `")) {
        let cells: Vec<_> = row.split('|').collect();
        let module = quoted_names(cells[1])
            .next()
            .expect("public item row must name a module")
            .split_whitespace()
            .next()
            .expect("module name must be present");
        let names = quoted_names(cells[2]).map(|name| {
            if module == "crate_root" {
                name.strip_prefix("module ").unwrap_or(name).to_owned()
            } else {
                name.to_owned()
            }
        });
        items.entry(module.into()).or_default().extend(names);
    }
    items
}

fn exported_items() -> BTreeMap<String, BTreeSet<String>> {
    let mut items = BTreeMap::<String, BTreeSet<String>>::new();
    let mut exports = MODEL_ROOT.split("pub use ").skip(1);
    for export in exports.by_ref() {
        let statement = export
            .split(';')
            .next()
            .expect("export must end with semicolon");
        assert!(
            !statement.contains('*'),
            "model root must not glob-export: {statement}"
        );
        if statement.starts_with("output_mapping::MappingAllocationPoint") {
            continue;
        }
        let (module, names) = statement
            .split_once("::{")
            .expect("default-feature model exports must name items in braces");
        let names = names.strip_suffix('}').expect("item list must close");
        for name in names
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            assert!(
                items.entry(module.into()).or_default().insert(name.into()),
                "duplicate export: {module}::{name}"
            );
        }
    }
    let mut sources =
        vec![PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("crates/quire-contract-model/src")];
    let mut exported_macros = BTreeSet::new();
    while let Some(path) = sources.pop() {
        for entry in fs::read_dir(path).expect("model source directory must be readable") {
            let path = entry
                .expect("source directory entry must be readable")
                .path();
            if path.is_dir() {
                sources.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let source = fs::read_to_string(&path).expect("model source file must be readable");
                for macro_export in source.split("#[macro_export]").skip(1) {
                    assert_eq!(
                        path.file_name().and_then(|name| name.to_str()),
                        Some("code.rs"),
                        "FR-019 lists exported macros under code"
                    );
                    let declaration = macro_export
                        .trim_start()
                        .strip_prefix("macro_rules! ")
                        .expect("exported macro must have an adjacent macro_rules declaration");
                    let name = declaration
                        .split(|character: char| {
                            !character.is_ascii_alphanumeric() && character != '_'
                        })
                        .next()
                        .expect("exported macro must have a name");
                    assert!(
                        exported_macros.insert(name.to_owned()),
                        "duplicate exported macro: {name}"
                    );
                }
            }
        }
    }
    items
        .entry("code".into())
        .or_default()
        .extend(exported_macros);
    let root = strip_comments(MODEL_ROOT);
    if root
        .lines()
        .any(|line| line.trim() == "pub mod source_obligation;")
    {
        items
            .entry("crate_root".into())
            .or_default()
            .insert("source_obligation".into());
        let source = fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("crates/quire-contract-model/src/source_obligation.rs"),
        )
        .expect("source-obligation module must be readable");
        let module_items = public_statements(&source)
            .into_iter()
            .map(|declaration| public_item_name(&declaration));
        items
            .entry("source_obligation".into())
            .or_default()
            .extend(module_items);
    }
    if root
        .lines()
        .any(|line| line.trim() == "pub mod selected_population;")
    {
        items
            .entry("crate_root".into())
            .or_default()
            .insert("selected_population".into());
        let source = fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("crates/quire-contract-model/src/selected_population.rs"),
        )
        .expect("selected-population facade must be readable");
        let names = public_statements(&source)
            .iter()
            .flat_map(|statement| selected_population_export_names(statement))
            .collect::<Vec<_>>();
        items
            .entry("selected_population".into())
            .or_default()
            .extend(names);
    }
    items
}

#[trace("TC-058", "FR-019-AC-5", "FR-052-AC-1")]
#[test]
fn tc_058_model_root_exports_exactly_the_documented_public_items() {
    assert_eq!(exported_items(), expected_items());
    assert_eq!(
        MODEL_ROOT
            .lines()
            .filter(|line| line.starts_with("pub ")
                && !line.starts_with("pub use ")
                && !line.starts_with("pub mod source_obligation;")
                && !line.starts_with("pub mod selected_population;"))
            .count(),
        0,
        "model root must expose items only through named exports"
    );
    assert!(MODEL_ROOT
        .lines()
        .any(|line| line == "pub mod source_obligation;"));
    assert!(MODEL_ROOT
        .lines()
        .any(|line| line == "pub mod selected_population;"));
}

#[trace("TC-456", "NFR-002-AC-1")]
#[test]
fn tc_456_public_schema_and_model_names_are_portable() {
    let forbidden_schema = json!({
        "title": "RuntimeHandle",
        "properties": { "pointer_width": { "enum": ["GumboNode"] } }
    });
    let mut forbidden_names = Vec::new();
    collect_schema_names(&forbidden_schema, &mut forbidden_names);
    assert_eq!(portable_name_violations(forbidden_names).len(), 3);
    assert!(portable_name_violations(["jsonpointer"]).is_empty());
    assert!(portable_name_violations(["profile_path", "repos_path", "profile_system"]).is_empty());
    assert_eq!(
        portable_name_violations(["filePath", "FilePath", "osPath", "file_system"]).len(),
        4
    );

    let model_items = exported_items();
    let root = strip_comments(MODEL_ROOT);
    let public_modules = root
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub mod ")?.strip_suffix(';'))
        .collect::<BTreeSet<_>>();
    let inventoried_modules = model_items
        .get("crate_root")
        .expect("public modules must be inventoried")
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    assert_eq!(public_modules, inventoried_modules);
    for module in public_modules {
        assert!(
            model_items
                .get(module)
                .is_some_and(|items| !items.is_empty()),
            "public module {module} must have an item inventory"
        );
    }
    for declaration in public_statements(MODEL_ROOT) {
        assert!(
            (declaration.starts_with("pub mod ") && declaration.trim_end().ends_with(';'))
                || declaration.starts_with("pub use "),
            "unrecognized public model-root declaration: {declaration}"
        );
    }
    let model_names = model_items
        .into_iter()
        .flat_map(|(module, items)| std::iter::once(module).chain(items));
    let schema_names = public_schema_names();
    assert!(!schema_names.is_empty(), "public schemas must be present");
    let violations =
        portable_name_violations(model_names.chain(schema_names.into_iter().flatten()));
    assert!(
        violations.is_empty(),
        "forbidden public names: {violations:?}"
    );
}

#[trace("TC-058", "FR-019-AC-5")]
#[test]
fn tc_058_fault_injection_adds_only_its_named_export() {
    let gate =
        "#[cfg(feature = \"fault-injection\")]\npub use output_mapping::MappingAllocationPoint;";
    assert!(MODEL_ROOT.contains(gate));
    assert_eq!(
        MODEL_ROOT
            .lines()
            .filter(|line| *line == "pub use output_mapping::MappingAllocationPoint;")
            .count(),
        1
    );
}

#[trace("TC-058", "FR-019-AC-5")]
#[test]
fn tc_058_root_crate_exposes_only_its_kani_module() {
    let public: Vec<_> = BRIDGE_ROOT
        .lines()
        .filter(|line| line.starts_with("pub "))
        .collect();
    assert_eq!(public, ["pub mod kani;"]);
}

/// The external consumer can call the borrowed graph API and distinguish
/// every refusal by its variant, without reading error messages.
#[trace("TC-058", "FR-048-AC-1")]
#[test]
fn tc_058_structural_candidate_api_is_typed_and_closed() {
    let draft: CheckedSemanticNodeV2 = serde_json::from_value(json!({
        "node_id": {"domain": "quire.checked-semantic-node/v1", "digest": "draft"},
        "schema_version": "quire.checked-semantic-graph/v2",
        "node_tag": "scalar_type", "semantic_form": "boolean",
        "semantic_type": {"domain": "quire.checked-semantic-node/v1", "digest": "draft"},
        "dependencies": [], "occurrences": [{"role": "generated", "ordinal": 0}],
        "body": {"term": "aggregate", "members": []},
    }))
    .expect("typed draft");
    let candidates =
        derive_structural_candidates(std::slice::from_ref(&draft), 4096, 10).expect("candidate");
    assert_eq!(candidates.nodes().len(), 1);
    assert_eq!(candidates.nodes()[0].position(), 0);
    assert_eq!(
        candidates.nodes()[0].candidate_node_id().domain.as_ref(),
        "quire.checked-semantic-node/v1"
    );
    assert!(candidates.groups().is_empty());
    let repeated = [draft.clone(), draft];
    let error = derive_structural_candidates(&repeated, 4096, 10).expect_err("repeat");
    let kind = match error {
        StructuralCandidateError::RepeatedKey {
            first_position,
            repeated_position,
            ..
        } => {
            assert_eq!((first_position, repeated_position), (0, 1));
            "repeated"
        }
        StructuralCandidateError::InvalidTagForm { .. } => "tag-form",
        StructuralCandidateError::InvalidClosedShape { .. } => "shape",
        StructuralCandidateError::UnsupportedGroupMember { .. } => "group-member",
        StructuralCandidateError::CounterOverflow { .. } => "overflow",
        StructuralCandidateError::WorkExhausted { .. } => "work",
        StructuralCandidateError::CanonicalEncoding { .. } => "canonical",
        StructuralCandidateError::ApplicationLocal { .. } => "application-local",
    };
    assert_eq!(kind, "repeated");
}
