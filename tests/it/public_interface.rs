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

/// Forbidden terms. A term matches inside any one word of a name, or as the
/// concatenation of adjacent words that starts with the term and reaches into
/// the last word, so `PathBuf`, `path_bufs`, `macOS`, `PointerWidth64`,
/// `file_path` and `filePath` match while `profile_path` does not.
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
            let in_word = words.iter().any(|word| word.contains(term));
            let across_words = (0..words.len()).any(|start| {
                (start + 2..=words.len()).any(|end| {
                    let before_last = words[start..end - 1].concat();
                    before_last.len() < term.len() && words[start..end].concat().starts_with(term)
                })
            });
            if in_word || across_words {
                violations.push(format!("{name}: {term}"));
            }
        }
    }
    violations
}

/// Public modules of the model crate named by the merged TC-456 procedure; a
/// newly declared public module fails the inventory until it is added here and
/// its names are checked.
const CURRENT_PUBLIC_MODULES: [&str; 2] = ["selected_population", "source_obligation"];

/// Parses one model source file; a file that does not parse fails the test.
fn parse_model_source(source: &str, what: &str) -> syn::File {
    syn::parse_file(source).unwrap_or_else(|error| panic!("{what} must parse: {error}"))
}

fn parse_model_module(module: &str) -> syn::File {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("crates/quire-contract-model/src")
        .join(format!("{module}.rs"));
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("public module {module} must be readable: {error}"));
    parse_model_source(&source, module)
}

/// `(module path, exported name)` for every leaf of a `use` tree; an alias
/// exports its alias and a glob is refused.
fn use_tree_exports(tree: &syn::UseTree, prefix: &[String], out: &mut Vec<(String, String)>) {
    match tree {
        syn::UseTree::Path(path) => {
            let mut next = prefix.to_vec();
            next.push(path.ident.to_string());
            use_tree_exports(&path.tree, &next, out);
        }
        syn::UseTree::Name(name) => out.push((prefix.join("::"), name.ident.to_string())),
        syn::UseTree::Rename(rename) => out.push((prefix.join("::"), rename.rename.to_string())),
        syn::UseTree::Glob(_) => panic!("public glob re-export: {}::*", prefix.join("::")),
        syn::UseTree::Group(group) => {
            for item in &group.items {
                use_tree_exports(item, prefix, out);
            }
        }
    }
}

/// The public declarations of a source file. Items that are not `pub` (inherited
/// or `pub(...)` restricted) are not public API. A public declaration of an
/// unrecognized kind, an inline module, or a macro invocation fails.
enum PublicDeclaration {
    Module(String),
    Export { module: String, name: String },
    Item(String),
}

fn public_declarations(file: &syn::File) -> Vec<PublicDeclaration> {
    use syn::{Item, Visibility};
    let mut out = Vec::new();
    for item in &file.items {
        let (visibility, name) = match item {
            Item::Use(item) => {
                if matches!(item.vis, Visibility::Public(_)) {
                    let mut exports = Vec::new();
                    use_tree_exports(&item.tree, &[], &mut exports);
                    out.extend(
                        exports
                            .into_iter()
                            .map(|(module, name)| PublicDeclaration::Export { module, name }),
                    );
                }
                continue;
            }
            Item::Mod(item) => {
                if matches!(item.vis, Visibility::Public(_)) {
                    assert!(
                        !item.attrs.iter().any(|attr| attr.path().is_ident("path")),
                        "public module {} redirected by #[path]",
                        item.ident
                    );
                    assert!(
                        item.content.is_none(),
                        "inline public module: {}",
                        item.ident
                    );
                    out.push(PublicDeclaration::Module(item.ident.to_string()));
                }
                continue;
            }
            Item::Fn(item) => (&item.vis, item.sig.ident.to_string()),
            Item::Struct(item) => (&item.vis, item.ident.to_string()),
            Item::Enum(item) => (&item.vis, item.ident.to_string()),
            Item::Union(item) => (&item.vis, item.ident.to_string()),
            Item::Const(item) => (&item.vis, item.ident.to_string()),
            Item::Static(item) => (&item.vis, item.ident.to_string()),
            Item::Trait(item) => (&item.vis, item.ident.to_string()),
            Item::TraitAlias(item) => (&item.vis, item.ident.to_string()),
            Item::Type(item) => (&item.vis, item.ident.to_string()),
            Item::ExternCrate(item) => (&item.vis, item.ident.to_string()),
            Item::Impl(_) => continue,
            Item::Macro(item) => {
                assert!(
                    item.mac.path.is_ident("macro_rules"),
                    "macro invocation can declare unreviewed public items"
                );
                continue;
            }
            _ => panic!("unrecognized item kind in a public-name inventory"),
        };
        if matches!(visibility, Visibility::Public(_)) {
            out.push(PublicDeclaration::Item(name));
        }
    }
    out
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
    let root = parse_model_source(MODEL_ROOT, "model crate root");
    let mut public_modules = BTreeSet::new();
    for declaration in public_declarations(&root) {
        match declaration {
            PublicDeclaration::Module(module) => {
                assert!(
                    CURRENT_PUBLIC_MODULES.contains(&module.as_str()),
                    "unrecognized public model module: {module}"
                );
                public_modules.insert(module);
            }
            PublicDeclaration::Export { module, name } => {
                if module == "output_mapping" && name == "MappingAllocationPoint" {
                    continue;
                }
                assert!(
                    items
                        .entry(module.clone())
                        .or_default()
                        .insert(name.clone()),
                    "duplicate export: {module}::{name}"
                );
            }
            PublicDeclaration::Item(name) => {
                panic!("unrecognized public model-root declaration: {name}")
            }
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
    for module in public_modules {
        let names = public_declarations(&parse_model_module(&module))
            .into_iter()
            .map(|declaration| match declaration {
                PublicDeclaration::Item(name) | PublicDeclaration::Export { name, .. } => name,
                PublicDeclaration::Module(nested) => {
                    panic!("nested public module in {module}: {nested}")
                }
            })
            .collect::<Vec<_>>();
        items
            .entry("crate_root".into())
            .or_default()
            .insert(module.clone());
        items.entry(module).or_default().extend(names);
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
    assert_eq!(
        portable_name_violations(["PathBuf", "path_buf", "MacOS", "macOS", "mac_os"]).len(),
        5
    );
    assert_eq!(
        portable_name_violations(["RunTime", "run_time", "USize"]).len(),
        3
    );
    assert_eq!(
        portable_name_violations([
            "PointerWidth64",
            "PathBufs",
            "path_bufs",
            "RunTimes",
            "run_times",
            "MacOSX"
        ])
        .len(),
        6
    );
    assert!(portable_name_violations(["profile_path", "repos_path", "profile_system"]).is_empty());
    assert_eq!(
        portable_name_violations(["filePath", "FilePath", "osPath", "file_system"]).len(),
        4
    );

    let model_items = exported_items();
    let inventoried_modules = model_items
        .get("crate_root")
        .expect("public modules must be inventoried")
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        inventoried_modules,
        CURRENT_PUBLIC_MODULES.into_iter().collect::<BTreeSet<_>>()
    );
    for module in inventoried_modules {
        assert!(
            model_items
                .get(module)
                .is_some_and(|items| !items.is_empty()),
            "public module {module} must have an item inventory"
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
