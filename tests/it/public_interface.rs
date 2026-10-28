use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

use ix_trace_rs::trace;
use quire_contract_model::{
    derive_structural_candidates, CheckedSemanticNodeV2, DiagnosticCode, Std001Code,
    StructuralCandidateError,
};
use serde_json::{json, Value};
use syn::{parse::Parser, punctuated::Punctuated, Token};

const MODEL_ROOT: &str = include_str!("../../crates/quire-contract-model/src/lib.rs");
const BRIDGE_ROOT: &str = include_str!("../../src/lib.rs");
const KANI_ROOT: &str = include_str!("../../src/kani/mod.rs");
const FR_039: &str =
    include_str!("../../spec/kani/functional/FR-039-root-crate-public-interface.md");
const FR_019: &str = include_str!("../../spec/model/functional/FR-019-rust-library-interface.md");
const STD_001: &str = include_str!("../../spec/core/functional/STD-001-diagnostic-registry.md");

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
            if let Some(constant) = object.get("const").and_then(Value::as_str) {
                names.push(constant.to_owned());
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

#[derive(Clone, Debug)]
struct PublicName {
    owner: String,
    name: String,
    value: Option<String>,
}

impl PublicName {
    fn item(owner: &str, name: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            name: name.into(),
            value: None,
        }
    }

    fn value(owner: &str, name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            name: name.into(),
            value: Some(value.into()),
        }
    }
}

struct StringLiterals(Vec<String>);

impl<'ast> syn::visit::Visit<'ast> for StringLiterals {
    fn visit_lit_str(&mut self, literal: &'ast syn::LitStr) {
        self.0.push(literal.value());
    }
}

fn constant_names(owner: &str, name: &str, expression: &syn::Expr, out: &mut Vec<PublicName>) {
    use syn::visit::Visit;
    out.push(PublicName::item(owner, name));
    let mut strings = StringLiterals(Vec::new());
    strings.visit_expr(expression);
    out.extend(
        strings
            .0
            .into_iter()
            .map(|value| PublicName::value(owner, name, value)),
    );
}

fn fields(owner: &str, fields: &syn::Fields, out: &mut Vec<PublicName>) {
    for field in fields {
        if matches!(field.vis, syn::Visibility::Public(_)) {
            if let Some(name) = &field.ident {
                out.push(PublicName::item(owner, name.to_string()));
            }
        }
    }
}

fn exported_paths(tree: &syn::UseTree, prefix: Vec<String>, out: &mut Vec<Vec<String>>) {
    match tree {
        syn::UseTree::Path(path) => {
            let mut prefix = prefix;
            prefix.push(path.ident.to_string());
            exported_paths(&path.tree, prefix, out);
        }
        syn::UseTree::Name(name) => {
            let mut path = prefix;
            path.push(name.ident.to_string());
            out.push(path);
        }
        syn::UseTree::Rename(rename) => {
            let mut path = prefix;
            path.push(rename.ident.to_string());
            out.push(path);
        }
        syn::UseTree::Group(group) => {
            for tree in &group.items {
                exported_paths(tree, prefix.clone(), out);
            }
        }
        syn::UseTree::Glob(_) => {
            let mut path = prefix;
            path.push("*".into());
            out.push(path);
        }
    }
}

fn source_use_aliases(tree: &syn::UseTree, out: &mut Vec<String>) {
    match tree {
        syn::UseTree::Path(path) => source_use_aliases(&path.tree, out),
        syn::UseTree::Name(name) => out.push(name.ident.to_string()),
        syn::UseTree::Rename(rename) => out.push(rename.rename.to_string()),
        syn::UseTree::Group(group) => {
            for tree in &group.items {
                source_use_aliases(tree, out);
            }
        }
        syn::UseTree::Glob(_) => {}
    }
}

fn source_items(file: &syn::File) -> Vec<PublicName> {
    let mut out = Vec::new();
    source_items_in_scope(&file.items, true, &[], &mut out);
    out
}

fn source_items_in_scope(
    items: &[syn::Item],
    public_scope: bool,
    inherited_exports: &[Vec<String>],
    out: &mut Vec<PublicName>,
) {
    use syn::{ImplItem, Item, TraitItem, Visibility};
    let definitions = items
        .iter()
        .filter_map(|item| {
            let Item::Macro(item) = item else { return None };
            item.mac.path.is_ident("macro_rules").then(|| {
                (
                    item.ident.as_ref().expect("macro_rules name").to_string(),
                    item.mac.tokens.to_string(),
                )
            })
        })
        .collect::<BTreeMap<_, _>>();
    let mut exports = inherited_exports.to_vec();
    if public_scope {
        for item in items {
            if let Item::Use(item) = item {
                if matches!(item.vis, Visibility::Public(_)) {
                    exported_paths(&item.tree, Vec::new(), &mut exports);
                    let mut aliases = Vec::new();
                    source_use_aliases(&item.tree, &mut aliases);
                    out.extend(aliases.into_iter().map(|alias| PublicName::item("", alias)));
                }
            }
        }
    }
    let visible = |name: &str, visibility: &Visibility| {
        (public_scope && matches!(visibility, Visibility::Public(_)))
            || exports
                .iter()
                .any(|path| path.len() == 1 && (path[0] == name || path[0] == "*"))
    };
    for item in items {
        match item {
            Item::Mod(item) => {
                let name = item.ident.to_string();
                let nested_exports = exports
                    .iter()
                    .filter(|path| path.first() == Some(&name) && path.len() > 1)
                    .map(|path| path[1..].to_vec())
                    .collect::<Vec<_>>();
                if let Some((_, nested)) = &item.content {
                    source_items_in_scope(
                        nested,
                        (public_scope && matches!(item.vis, Visibility::Public(_)))
                            || nested_exports.iter().any(|path| path == &["*"]),
                        &nested_exports,
                        out,
                    );
                }
            }
            Item::Struct(item) if visible(&item.ident.to_string(), &item.vis) => {
                let owner = item.ident.to_string();
                out.push(PublicName::item("", &owner));
                fields(&owner, &item.fields, out);
            }
            Item::Union(item) if visible(&item.ident.to_string(), &item.vis) => {
                let owner = item.ident.to_string();
                out.push(PublicName::item("", &owner));
                for field in &item.fields.named {
                    if matches!(field.vis, Visibility::Public(_)) {
                        out.push(PublicName::item(
                            &owner,
                            field.ident.as_ref().unwrap().to_string(),
                        ));
                    }
                }
            }
            Item::Enum(item) if visible(&item.ident.to_string(), &item.vis) => {
                let owner = item.ident.to_string();
                out.push(PublicName::item("", &owner));
                for variant in &item.variants {
                    out.push(PublicName::item(&owner, variant.ident.to_string()));
                    fields(&owner, &variant.fields, out);
                }
            }
            Item::Trait(item) if visible(&item.ident.to_string(), &item.vis) => {
                let owner = item.ident.to_string();
                out.push(PublicName::item("", &owner));
                for member in &item.items {
                    match member {
                        TraitItem::Fn(member) => {
                            out.push(PublicName::item(&owner, member.sig.ident.to_string()))
                        }
                        TraitItem::Const(member) => {
                            if let Some((_, expression)) = &member.default {
                                constant_names(&owner, &member.ident.to_string(), expression, out);
                            } else {
                                out.push(PublicName::item(&owner, member.ident.to_string()));
                            }
                        }
                        TraitItem::Type(member) => {
                            out.push(PublicName::item(&owner, member.ident.to_string()))
                        }
                        _ => panic!("unrecognized exported trait member in {owner}"),
                    }
                }
            }
            Item::Impl(item) if item.trait_.is_none() => {
                let owner = match item.self_ty.as_ref() {
                    syn::Type::Path(path) => path.path.segments.last().unwrap().ident.to_string(),
                    _ => continue,
                };
                if !public_scope
                    && !exports
                        .iter()
                        .any(|path| path.len() == 1 && path[0] == owner)
                {
                    continue;
                }
                for member in &item.items {
                    match member {
                        ImplItem::Fn(member) if matches!(member.vis, Visibility::Public(_)) => {
                            out.push(PublicName::item(&owner, member.sig.ident.to_string()))
                        }
                        ImplItem::Const(member) if matches!(member.vis, Visibility::Public(_)) => {
                            constant_names(&owner, &member.ident.to_string(), &member.expr, out)
                        }
                        ImplItem::Type(member) if matches!(member.vis, Visibility::Public(_)) => {
                            out.push(PublicName::item(&owner, member.ident.to_string()))
                        }
                        _ => {}
                    }
                }
            }
            Item::Const(item) if visible(&item.ident.to_string(), &item.vis) => {
                constant_names("", &item.ident.to_string(), &item.expr, out)
            }
            Item::Static(item) if visible(&item.ident.to_string(), &item.vis) => {
                constant_names("", &item.ident.to_string(), &item.expr, out)
            }
            Item::Fn(item) if visible(&item.sig.ident.to_string(), &item.vis) => {
                out.push(PublicName::item("", item.sig.ident.to_string()))
            }
            Item::Type(item) if visible(&item.ident.to_string(), &item.vis) => {
                out.push(PublicName::item("", item.ident.to_string()))
            }
            Item::TraitAlias(item) if visible(&item.ident.to_string(), &item.vis) => {
                out.push(PublicName::item("", item.ident.to_string()))
            }
            Item::Macro(item) => macro_items(item, &definitions, out),
            Item::Use(_)
            | Item::Impl(_)
            | Item::Fn(_)
            | Item::Struct(_)
            | Item::Enum(_)
            | Item::Union(_)
            | Item::Const(_)
            | Item::Static(_)
            | Item::Type(_)
            | Item::Trait(_)
            | Item::TraitAlias(_) => {}
            Item::ExternCrate(item) if visible(&item.ident.to_string(), &item.vis) => {
                out.push(PublicName::item("", item.ident.to_string()));
            }
            Item::ExternCrate(_) => {}
            _ => panic!("unrecognized declaration kind in source inventory"),
        }
    }
}

struct CodeRow {
    name: syn::Ident,
    value: syn::LitStr,
}

struct ClosedVocabulary {
    visibility: syn::Visibility,
    name: syn::Ident,
    rows: Punctuated<CodeRow, Token![,]>,
}

impl syn::parse::Parse for ClosedVocabulary {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let _attributes = input.call(syn::Attribute::parse_outer)?;
        let visibility = input.parse()?;
        let name = input.parse()?;
        let content;
        syn::braced!(content in input);
        let rows = content.parse_terminated(CodeRow::parse, Token![,])?;
        Ok(Self {
            visibility,
            name,
            rows,
        })
    }
}

impl syn::parse::Parse for CodeRow {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![=>]>()?;
        let value = input.parse()?;
        Ok(Self { name, value })
    }
}

fn code_rows(item: &syn::ItemMacro) -> Vec<CodeRow> {
    Punctuated::<CodeRow, Token![,]>::parse_terminated
        .parse2(item.mac.tokens.clone())
        .unwrap_or_else(|error| {
            panic!(
                "{} rows must parse: {error}",
                item.mac.path.segments.last().unwrap().ident
            )
        })
        .into_iter()
        .collect()
}

fn macro_static_public_names(definition: &str) -> BTreeSet<String> {
    let tokens = definition.split_whitespace().collect::<Vec<_>>();
    let mut names = BTreeSet::new();
    for (index, token) in tokens.iter().enumerate() {
        if *token != "pub" {
            continue;
        }
        let mut next = index + 1;
        while matches!(
            tokens.get(next),
            Some(&"const" | &"fn" | &"struct" | &"enum" | &"type" | &"trait" | &"static" | &"mut")
        ) {
            next += 1;
        }
        if let Some(name) = tokens.get(next) {
            if *name != "$"
                && name
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '_')
            {
                names.insert((*name).to_owned());
            }
        }
    }
    names
}

fn macro_items(
    item: &syn::ItemMacro,
    definitions: &BTreeMap<String, String>,
    out: &mut Vec<PublicName>,
) {
    let name = item.mac.path.segments.last().unwrap().ident.to_string();
    let first = out.len();
    match name.as_str() {
        "diagnostic_codes" | "mapping_error_codes" => {
            let owner = if name == "diagnostic_codes" {
                "DiagnosticCode"
            } else {
                "MappingRequestErrorCode"
            };
            out.push(PublicName::item("", owner));
            out.extend(
                ["ALL", "as_str"]
                    .into_iter()
                    .map(|name| PublicName::item(owner, name)),
            );
            if owner == "MappingRequestErrorCode" {
                out.push(PublicName::item(owner, "from_code"));
            }
            for row in code_rows(item) {
                out.push(PublicName::item(owner, row.name.to_string()));
                out.push(PublicName::value(owner, "as_str", row.value.value()));
                out.push(PublicName::value(owner, "ALL", row.value.value()));
            }
        }
        "registered_codes" => {
            out.push(PublicName::item("Std001Code", "REGISTERED"));
            for row in code_rows(item) {
                let name = row.name.to_string();
                let value = row.value.value();
                out.push(PublicName::value("Std001Code", name, value.clone()));
                out.push(PublicName::value("Std001Code", "REGISTERED", value));
            }
        }
        "identifier_type"
        | "positive_revision"
        | "raw_digest_type"
        | "source_selection_type"
        | "qualified_code_type" => {
            let owner = item
                .mac
                .tokens
                .to_string()
                .split([',', ' '])
                .find(|token| !token.is_empty())
                .expect("generated type name")
                .to_owned();
            out.push(PublicName::item("", &owner));
            let members: &[&str] = match name.as_str() {
                "identifier_type" => &["new", "as_str"],
                "positive_revision" => &["new", "get"],
                "raw_digest_type" => &["from_bytes", "digest", "as_bytes", "parse"],
                "source_selection_type" => &["new", "identity", "revision", "digest"],
                "qualified_code_type" => {
                    &["new", "owner", "contract", "revision", "digest", "code"]
                }
                _ => unreachable!(),
            };
            out.extend(
                members
                    .iter()
                    .map(|member| PublicName::item(&owner, *member)),
            );
        }
        "closed_vocabulary" => {
            let vocabulary: ClosedVocabulary =
                syn::parse2(item.mac.tokens.clone()).expect("closed vocabulary must parse");
            if matches!(vocabulary.visibility, syn::Visibility::Public(_)) {
                let owner = vocabulary.name.to_string();
                out.push(PublicName::item("", &owner));
                out.extend(
                    ["ALL", "as_wire", "from_wire"]
                        .into_iter()
                        .map(|name| PublicName::item(&owner, name)),
                );
                for row in vocabulary.rows {
                    out.push(PublicName::item(&owner, row.name.to_string()));
                    out.push(PublicName::value(&owner, "as_wire", row.value.value()));
                }
            }
        }
        "macro_rules" => {
            let definition = item.mac.tokens.to_string();
            if definition.contains("pub ") {
                let known = [
                    "diagnostic_codes",
                    "mapping_error_codes",
                    "registered_codes",
                    "identifier_type",
                    "positive_revision",
                    "raw_digest_type",
                    "source_selection_type",
                    "qualified_code_type",
                    "closed_vocabulary",
                ];
                let defined = item.ident.as_ref().expect("macro_rules name").to_string();
                assert!(
                    known.contains(&defined.as_str()),
                    "macro_rules! {defined} can generate unreviewed public API"
                );
            }
        }
        _ => {}
    }
    if name != "macro_rules" {
        if let Some(definition) = definitions.get(&name).filter(|_| out.len() > first) {
            let extracted = out[first..]
                .iter()
                .map(|item| item.name.as_str())
                .collect::<BTreeSet<_>>();
            for public_name in macro_static_public_names(definition) {
                assert!(
                    extracted.contains(public_name.as_str()),
                    "macro-expanded public item {name}::{public_name} escaped inventory"
                );
            }
        }
    }
}

const CODE_TABLES: [&str; 7] = [
    "Identity and Reference Codes",
    "Type and Expression Codes",
    "Version, Canonicalization and Coverage Codes",
    "Semantic Limit Codes",
    "Typed Code Form",
    "Bounded Kani Cause Codes",
    "Bounded Kani Outcome Error Code",
];

fn std001_code_rows(document: &str) -> BTreeMap<String, BTreeSet<String>> {
    let mut tables = BTreeMap::<String, BTreeSet<String>>::new();
    let mut active: Option<&str> = None;
    for line in document.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            active = CODE_TABLES.contains(&heading).then_some(heading);
            if let Some(heading) = active {
                assert!(
                    tables.insert(heading.to_owned(), BTreeSet::new()).is_none(),
                    "duplicate STD-001 code table: {heading}"
                );
            }
        } else if let (Some(heading), Some(rest)) = (active, line.strip_prefix("| `")) {
            let (code, _) = rest
                .split_once('`')
                .expect("Code row must close its code literal");
            assert!(
                tables.get_mut(heading).unwrap().insert(code.to_owned()),
                "duplicate STD-001 code row: {code}"
            );
        }
    }
    assert_eq!(
        tables.len(),
        CODE_TABLES.len(),
        "all seven STD-001 code tables must exist"
    );
    assert!(
        tables.values().all(|codes| !codes.is_empty()),
        "STD-001 code table is empty"
    );
    tables
}

fn registry_difference(tables: &BTreeMap<String, BTreeSet<String>>) -> Vec<String> {
    let diagnostic = DiagnosticCode::ALL
        .iter()
        .map(|code| code.as_str().to_owned())
        .collect::<BTreeSet<_>>();
    let all = tables
        .values()
        .flat_map(|table| table.iter().cloned())
        .collect::<BTreeSet<_>>();
    assert!(
        diagnostic.is_subset(&all),
        "DiagnosticCode::ALL must appear in STD-001"
    );
    all.difference(&diagnostic).cloned().collect()
}

fn registered_code_violations(
    names: &[PublicName],
    tables: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<String> {
    let expected = registry_difference(tables);
    let source_values = names
        .iter()
        .filter(|name| name.owner == "Std001Code" && name.name == "REGISTERED")
        .filter_map(|name| name.value.as_deref())
        .collect::<Vec<_>>();
    let actual = Std001Code::REGISTERED;
    let mut violations = Vec::new();
    if actual
        .iter()
        .map(|value| (*value).to_owned())
        .collect::<Vec<_>>()
        != expected
    {
        violations.push(format!(
            "Std001Code::REGISTERED differs from STD-001: {expected:?}"
        ));
    }
    if source_values != actual {
        violations.push("Std001Code::REGISTERED source differs from compiled array".into());
    }
    for code in &expected {
        let constant = code.to_ascii_uppercase();
        let matches = names
            .iter()
            .filter(|name| {
                name.owner == "Std001Code"
                    && name.name == constant
                    && name.value.as_deref() == Some(code)
            })
            .count();
        if matches != 1 {
            violations.push(format!(
                "Std001Code::{constant} must equal {code} exactly once"
            ));
        }
    }
    violations
}

fn inventory_violations(names: &[PublicName]) -> Vec<String> {
    let mut violations = Vec::new();
    for entry in names {
        for name in std::iter::once(entry.name.as_str()).chain(entry.value.as_deref()) {
            violations.extend(
                portable_name_violations([name])
                    .into_iter()
                    .map(|violation| format!("{}::{}: {violation}", entry.owner, entry.name)),
            );
        }
    }
    violations
}

fn model_source_inventory(root: &std::path::Path) -> Vec<PublicName> {
    let mut pending = vec![root.to_path_buf()];
    let mut names = Vec::new();
    while let Some(path) = pending.pop() {
        for entry in fs::read_dir(path).expect("model source directory") {
            let path = entry.expect("model source entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let source = fs::read_to_string(&path).expect("model source file");
                names.extend(source_items(&parse_model_source(
                    &source,
                    &path.display().to_string(),
                )));
            }
        }
    }
    names
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
                if (module == "output_mapping" && name == "MappingAllocationPoint")
                    || (module == "checked_package"
                        && [
                            "CheckedApplicationOperationFixture",
                            "CheckedApplicationOperationFixtureError",
                        ]
                        .contains(&name.as_str()))
                {
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
    let _: Option<quire_contract_model::MappingAllocationPoint> = None;
    let tables = std001_code_rows(STD_001);
    let source_root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("crates/quire-contract-model/src");
    let mut names = model_source_inventory(&source_root);
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
    names.extend(model_items.into_iter().flat_map(|(module, items)| {
        std::iter::once(module)
            .chain(items)
            .map(|name| PublicName::item("", name))
    }));
    for declaration in public_declarations(&parse_model_source(MODEL_ROOT, "model root")) {
        if let PublicDeclaration::Export { name, .. } = declaration {
            names.push(PublicName::item("", name));
        }
    }
    let schema_names = public_schema_names();
    assert!(!schema_names.is_empty(), "public schemas must be present");
    names.extend(
        schema_names
            .into_iter()
            .flatten()
            .map(|name| PublicName::item("schema", name)),
    );
    let violations = inventory_violations(&names);
    assert!(
        violations.is_empty(),
        "forbidden public names or values: {violations:?}"
    );
    assert!(
        registered_code_violations(&names, &tables).is_empty(),
        "Std001Code constants and REGISTERED must match all STD-001 non-diagnostic rows"
    );

    let mut benign = Vec::new();
    collect_schema_names(&json!({"properties": {"jsonpointer": {}}}), &mut benign);
    assert!(inventory_violations(
        &benign
            .into_iter()
            .map(|name| PublicName::item("schema", name))
            .collect::<Vec<_>>(),
    )
    .is_empty());

    for schema in [
        json!({"title": "RuntimeHandle"}),
        json!({"properties": {"pointer_width": {}}}),
        json!({"enum": ["GumboNode"]}),
        json!({"const": "solver_runtime"}),
    ] {
        let mut extracted = Vec::new();
        collect_schema_names(&schema, &mut extracted);
        assert!(
            !inventory_violations(
                &extracted
                    .into_iter()
                    .map(|name| PublicName::item("schema", name))
                    .collect::<Vec<_>>(),
            )
            .is_empty(),
            "schema adverse control escaped: {schema}"
        );
    }

    for source in [
        "pub struct FeatureExport; impl FeatureExport { pub fn solver_runtime(&self) {} }",
        "pub struct Model { pub pointer_width: u8 }",
        "pub enum Kind { GumboNode }",
        "pub struct Model; impl Model { pub fn runtime(&self) {} }",
        "pub struct Model; impl Model { pub const SOLVER: &'static str = \"solver\"; }",
        "pub trait CanonicalBody { fn runtime(&self); const SOLVER: &'static str = \"solver\"; }",
        "diagnostic_codes! { SolverLeak => \"solver_leak\" }",
        "mod allocation { pub enum Point { Safe, SolverRuntimeBuffer } } #[cfg(feature = \"fault-injection\")] pub use allocation::Point;",
        "pub struct Std001Code; impl Std001Code { pub const EXTRA_SOLVER: &'static str = \"solver\"; }",
    ] {
        let extracted = source_items(&parse_model_source(source, "Rust adverse control"));
        assert!(!inventory_violations(&extracted).is_empty(),
            "Rust adverse control escaped: {source}");
    }
}

#[trace("TC-456")]
#[test]
fn tc_456_name_matcher_covers_case_boundaries_and_near_misses() {
    for name in [
        "PathBuf",
        "PathBufs",
        "path_buf",
        "path_bufs",
        "MacOS",
        "macOS",
        "mac_os",
        "RunTime",
        "RunTimes",
        "run_time",
        "run_times",
        "USize",
        "PointerWidth64",
        "pointer_widths",
        "MacOSX",
        "filePath",
        "FilePath",
        "osPath",
        "file_system",
    ] {
        assert!(
            !portable_name_violations([name]).is_empty(),
            "missed {name}"
        );
    }
    assert!(portable_name_violations([
        "profile_path",
        "profilePath",
        "repos_path",
        "reposPath",
        "profile_system",
        "profileSystem",
        "jsonpointer"
    ])
    .is_empty());
    assert_eq!(name_words("filePath"), ["file", "path"]);
}

#[trace("TC-456")]
#[test]
fn tc_456_gated_reexport_checks_inline_enum_variants() {
    let _: Option<quire_contract_model::MappingAllocationPoint> = None;
    let source = parse_model_source(
        "mod allocation { pub enum Point { Safe, SolverRuntimeBuffer } } #[cfg(feature = \"fault-injection\")] pub use allocation::Point;",
        "gated re-export control",
    );
    let names = source_items(&source);
    assert!(names
        .iter()
        .any(|name| name.owner == "Point" && name.name == "SolverRuntimeBuffer"));
    assert!(!inventory_violations(&names).is_empty());
}

#[trace("TC-456")]
#[test]
fn tc_456_reexported_private_type_impl_is_inventoried() {
    let source = parse_model_source(
        "mod allocation { pub struct Point; impl Point { pub fn solver_probe(self) {} } } pub use allocation::Point;",
        "re-exported impl control",
    );
    let names = source_items(&source);
    assert!(names
        .iter()
        .any(|name| name.owner == "Point" && name.name == "solver_probe"));
    assert!(!inventory_violations(&names).is_empty());
}

#[trace("TC-456")]
#[test]
fn tc_456_public_inline_module_is_inventoried() {
    let source = parse_model_source(
        "pub mod api { pub fn solver_probe() {} }",
        "public inline module control",
    );
    let names = source_items(&source);
    assert!(names.iter().any(|name| name.name == "solver_probe"));
    assert!(!inventory_violations(&names).is_empty());
}

#[trace("TC-456")]
#[test]
fn tc_456_glob_reexport_is_inventoried() {
    let source = parse_model_source(
        "mod api { pub fn solver_probe() {} } pub use api::*;",
        "glob re-export control",
    );
    let names = source_items(&source);
    assert!(names.iter().any(|name| name.name == "solver_probe"));
    assert!(!inventory_violations(&names).is_empty());
}

#[trace("TC-456")]
#[test]
fn tc_456_renamed_reexport_checks_source_name() {
    let source = parse_model_source(
        "mod api { pub fn solver_probe() {} } pub use api::solver_probe as probe;",
        "renamed re-export control",
    );
    let names = source_items(&source);
    assert!(names.iter().any(|name| name.name == "solver_probe"));
    assert!(!inventory_violations(&names).is_empty());
}

#[trace("TC-456")]
#[test]
fn tc_456_non_root_reexport_alias_is_inventoried() {
    let directory = std::env::temp_dir().join(format!("tc456-alias-{}", std::process::id()));
    fs::create_dir(&directory).expect("create non-root source fixture directory");
    fs::write(
        directory.join("api.rs"),
        "mod api { pub fn probe() {} } pub use api::probe as solver_probe;",
    )
    .expect("write non-root source fixture");
    let names = model_source_inventory(&directory);
    fs::remove_dir_all(&directory).expect("remove non-root source fixture directory");
    assert!(names.iter().any(|name| name.name == "solver_probe"));
    assert!(!inventory_violations(&names).is_empty());
}

#[trace("TC-456")]
#[test]
fn tc_456_unrecognized_public_declaration_fails_closed() {
    let source = parse_model_source(
        "extern \"C\" { pub fn runtime(); }",
        "foreign declaration control",
    );
    assert!(std::panic::catch_unwind(|| source_items(&source)).is_err());
}

#[trace("TC-456")]
#[test]
fn tc_456_unrecognized_exported_trait_member_fails_closed() {
    let source = parse_model_source(
        "pub trait CanonicalBody { unknown_member!(); }",
        "trait member control",
    );
    assert!(std::panic::catch_unwind(|| source_items(&source)).is_err());
}

#[trace("TC-456")]
#[test]
fn tc_456_constant_values_and_trait_functions_reach_the_inventory() {
    for (source, owner, name) in [
        (
            "pub struct Model; impl Model { pub const LABEL: &'static str = \"solver\"; }",
            "Model",
            "LABEL",
        ),
        (
            "pub trait CanonicalBody { fn runtime(&self); }",
            "CanonicalBody",
            "runtime",
        ),
    ] {
        let names = source_items(&parse_model_source(source, "member control"));
        assert!(names
            .iter()
            .any(|item| item.owner == owner && item.name == name));
        assert!(
            !inventory_violations(&names).is_empty(),
            "member escaped: {source}"
        );
    }
}

#[trace("TC-456")]
#[test]
fn tc_456_retired_solver_code_is_rejected_without_an_allowlist() {
    for (name, source) in [
        (
            "KANI_SOLVER_ABSENT",
            "pub struct Std001Code; impl Std001Code { pub const KANI_SOLVER_ABSENT: &'static str = \"kani_solver_absent\"; }",
        ),
        (
            "REGISTERED",
            "pub struct Std001Code; impl Std001Code { pub const REGISTERED: &'static [&'static str] = &[\"kani_solver_absent\"]; }",
        ),
    ] {
        let names = source_items(&parse_model_source(source, "retired code control"));
        assert!(
            !inventory_violations(&names).is_empty(),
            "retired code admitted through {name}"
        );
    }
}

#[trace("TC-456")]
#[test]
fn tc_456_macro_body_cannot_add_an_uninventoried_public_member() {
    let source = parse_model_source(
        "macro_rules! identifier_type { ($name:ident) => { pub struct $name; impl $name { pub fn solver_escape() {} } } } identifier_type!(Safe);",
        "macro escape control",
    );
    let result = std::panic::catch_unwind(|| source_items(&source));
    assert!(
        result.is_err(),
        "macro body public member escaped inventory"
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

/// Trace: FR-019-AC-16, TC-058.
#[trace("TC-058", "FR-019-AC-16")]
#[test]
fn tc_058_test_support_exports_are_independent_of_fault_injection() {
    let fixture_export = "#[cfg(feature = \"test-support\")]\npub use checked_package::{\n    CheckedApplicationOperationFixture, CheckedApplicationOperationFixtureError,\n};";
    assert!(MODEL_ROOT.contains(fixture_export));
    assert_eq!(exported_items(), expected_items());
}

/// Trace: FR-019-AC-15, TC-058.
#[cfg(feature = "test-support")]
#[trace("TC-058", "FR-019-AC-15")]
#[test]
fn tc_058_fixture_operation_does_not_bypass_strict_package_admission() {
    use crate::support::checked_package::{
        canonical, evidence_for, pointer, refresh_identity, rekey_application_node, v2_all_families,
    };
    use quire_contract_model::{
        CheckedApplicationOperation, CheckedApplicationOperationFixture, CheckedPackageReadLimits,
        CheckedPackageRefusalCode, CheckedPackageV2, CheckedPackageV2ReadResult,
    };

    let fixture = CheckedApplicationOperation::from_test_fixture(
        CheckedApplicationOperationFixture::default(),
    )
    .expect("intentional absent identity is a valid typed fixture");
    let operation = serde_json::to_value(&fixture).expect("fixture serializes as operation wire");
    assert!(operation.get("test_fixture").is_none());
    let mut package = v2_all_families();
    let position = package["semantic_graph"]["nodes"]
        .as_array()
        .expect("nodes")
        .iter()
        .position(|node| node["node_tag"] == "function" && node["body"]["term"] == "application")
        .expect("function call application");
    package["semantic_graph"]["nodes"][position]["body"]["operation"] = operation;
    rekey_application_node(&mut package, position);
    refresh_identity(&mut package);
    let result = CheckedPackageV2::read(
        &canonical(&package),
        CheckedPackageReadLimits::bounded(),
        &evidence_for(&package),
    );
    let CheckedPackageV2ReadResult::Refused(refusal) = result else {
        panic!("missing operation identity must be refused: {result:?}");
    };
    assert_eq!(refusal.code, CheckedPackageRefusalCode::MalformedWire);
    assert_eq!(
        refusal.path,
        Some(pointer(&format!(
            "/semantic_graph/nodes/{position}/body/operation/identity"
        )))
    );
}

/// Trace: FR-039-AC-1, FR-039-AC-2, FR-039-AC-3, FR-037-AC-6, TC-055.
#[trace("TC-055", "FR-039-AC-1", "FR-039-AC-2", "FR-039-AC-3", "FR-037-AC-6")]
#[test]
fn tc_055_root_public_inventory_matches_fr_039() {
    let root = parse_model_source(BRIDGE_ROOT, "IR root");
    let declarations = public_declarations(&root);
    assert_eq!(
        declarations.len(),
        1,
        "IR root must have one public declaration"
    );
    assert!(matches!(&declarations[0], PublicDeclaration::Module(name) if name == "kani"));
    let (table, _) = FR_039
        .split_once("### Public items")
        .expect("FR-039 public items")
        .1
        .split_once("An item's own public fields")
        .expect("FR-039 table end");
    let expected: BTreeSet<String> = table
        .lines()
        .filter(|line| line.starts_with("| `kani`"))
        .flat_map(|line| {
            let column = line.split('|').nth(2).expect("public items column");
            column
                .replace("constant ", "")
                .replace("types ", "")
                .replace("; ", ", ")
                .split(',')
                .map(|item| item.trim().trim_matches('`').to_owned())
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(
        expected.len(),
        20,
        "FR-039 table parser must cover every item"
    );
    let kani = parse_model_source(KANI_ROOT, "Kani root");
    for item in &kani.items {
        if let syn::Item::Mod(module) = item {
            assert!(
                !["replay", "witness", "arithmetic", "collections", "objects"]
                    .contains(&module.ident.to_string().as_str()),
                "retired Kani module {} is declared",
                module.ident
            );
        }
    }
    let mut actual = BTreeSet::new();
    for declaration in public_declarations(&kani) {
        match declaration {
            PublicDeclaration::Export { module, name } => {
                assert!(
                    matches!(module.as_str(), "abi" | "dispatch" | "outcome" | "profile"),
                    "unexpected export module: {module}"
                );
                let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("src/kani")
                    .join(format!("{module}.rs"));
                let source = fs::read_to_string(path).expect("export source must exist");
                let leaf = parse_model_source(&source, &module);
                let leaf_has_item = public_declarations(&leaf).iter().any(
                    |item| matches!(item, PublicDeclaration::Item(item_name) if item_name == &name),
                );
                assert!(leaf_has_item, "{module}::{name} must be public in its leaf");
                assert!(actual.insert(name), "duplicate Kani export");
            }
            PublicDeclaration::Item(name) => assert!(actual.insert(name), "duplicate Kani item"),
            PublicDeclaration::Module(name) => panic!("unexpected public Kani module: {name}"),
        }
    }
    assert_eq!(
        actual, expected,
        "transitive root public items differ from FR-039"
    );
}

fn exports_macro(file: &syn::File) -> bool {
    use syn::visit::Visit;

    struct ExportedMacro(bool);
    impl<'ast> Visit<'ast> for ExportedMacro {
        fn visit_item_macro(&mut self, item: &'ast syn::ItemMacro) {
            self.0 |= item
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("macro_export"));
            syn::visit::visit_item_macro(self, item);
        }
    }
    let mut visitor = ExportedMacro(false);
    visitor.visit_file(file);
    visitor.0
}

/// Trace: FR-039-AC-1, TC-055.
#[trace("TC-055", "FR-039-AC-1")]
#[test]
fn tc_055_nested_exported_macro_is_rejected_by_inventory() {
    let nested = parse_model_source(
        "mod nested { #[macro_export] macro_rules! leaked { () => {}; } }",
        "nested macro control",
    );
    assert!(exports_macro(&nested));
    let ordinary = parse_model_source(
        "mod nested { macro_rules! private { () => {}; } }",
        "private macro control",
    );
    assert!(!exports_macro(&ordinary));
}

/// Trace: FR-039-AC-2, FR-039-AC-3, FR-037-AC-6, TC-055.
#[trace("TC-055", "FR-039-AC-2", "FR-039-AC-3", "FR-037-AC-6")]
#[test]
fn tc_055_no_foreign_runtime_or_replay_surface_in_source() {
    let root_source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let model_source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("crates/quire-contract-model/src");
    let mut pending = vec![root_source.clone(), model_source.clone()];
    let mut scanned = Vec::new();
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(dir).expect("source directory") {
            let path = entry.expect("source entry").path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            if path.extension().is_none_or(|extension| extension != "rs") {
                continue;
            }
            let source = fs::read_to_string(&path).expect("source file");
            let mut violations = Vec::new();
            if path.starts_with(&root_source) {
                let parsed = parse_model_source(&source, &path.display().to_string());
                if exports_macro(&parsed) {
                    violations.push("exports an uninventoried macro".to_owned());
                }
            }
            let code = source
                .lines()
                .filter(|line| !line.trim_start().starts_with("//"))
                .collect::<Vec<_>>()
                .join("\n");
            for forbidden in [
                "quire_spec_language::runtime",
                "qsl_replay::",
                "TerminalValue",
                "replay_with_native_runtime",
            ] {
                if code.contains(forbidden) {
                    violations.push(format!("contains {forbidden}"));
                }
            }
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("file name");
            if path
                .parent()
                .is_some_and(|parent| parent.ends_with("src/kani"))
                && [
                    "replay.rs",
                    "witness.rs",
                    "arithmetic.rs",
                    "collections.rs",
                    "objects.rs",
                ]
                .contains(&name)
            {
                violations.push(format!("retired Kani module {name}"));
            }
            scanned.push((path, violations));
        }
    }
    assert!(
        scanned
            .iter()
            .any(|(path, _)| path.starts_with(&root_source)),
        "root source must be scanned"
    );
    assert!(
        scanned
            .iter()
            .any(|(path, _)| path.starts_with(&model_source)),
        "model source must be scanned"
    );
    let failures = scanned
        .iter()
        .flat_map(|(path, violations)| {
            violations
                .iter()
                .map(move |violation| format!("{}: {violation}", path.display()))
        })
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
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
    let draft: quire_contract_model::CheckedSemanticNodeV2<Value> =
        serde_json::from_value(serde_json::to_value(draft).expect("draft wire"))
            .expect("raw draft");
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

/// The timed form is available through the model's documented public root.
#[trace("TC-058", "TC-048")]
#[test]
fn tc_058_timed_model_form_is_available_from_the_public_model_root() {
    let kind = quire_contract_model::CheckedNodeKind::TimedModel(
        quire_contract_model::TimedModelForm::Source,
    );
    assert_eq!(kind.tag(), quire_contract_model::CheckedNodeTag::TimedModel);
}
