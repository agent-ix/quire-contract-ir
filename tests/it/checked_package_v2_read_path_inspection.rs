// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Source inspection of the V2 read path (TC-472).
//!
//! FR-038-AC-513 bans, on the path of `CheckedPackageV2::read`, any
//! `serde_json` parser, deserializer or `RawValue`, any `serde_json::Value`,
//! `num-bigint` parse or arithmetic, any stable sort, and any `BTreeMap`,
//! `BTreeSet`, `HashMap` or `serde_json::Map`. The criterion allows no
//! exception, so there is no allowlist: every production syntax item in the read
//! path is inspected, including items after test-only declarations.

use ix_trace_rs::trace;
use std::path::Path;
use syn::visit::{self, Visit};

/// The read path's source files under `checked_package/v2`: the staged readers
/// of the stage table (wire decode, wire checks, node index, lock, model
/// documents, graph validators, source map, extension, diagnostics, retained
/// structures) and the budgeted entry and its charging helpers.
const READ_PATH: &[&str] = &[
    "budgeted_read.rs",
    "read_charge.rs",
    "graph_storage.rs",
    "checked_json.rs",
    "document_de.rs",
    "term_view.rs",
    "raw_body.rs",
    "mod.rs",
    "intake.rs",
    "lossless.rs",
    "flat_wire.rs",
    "typed_terms.rs",
    "node_index.rs",
    "identity.rs",
    "closure_index.rs",
    "operation_catalog.rs",
    "operation_catalog_index.rs",
    "node_lookup.rs",
    "model_members.rs",
    "model_members/charged.rs",
    "model_members/derive.rs",
    "model_members/document.rs",
    "model_members/intake_origin.rs",
    "model_members/option_types.rs",
    "model_members/view.rs",
    "model_fields.rs",
    "model_root.rs",
    "selected_document.rs",
    "dependency_references.rs",
    "model_population_catalog.rs",
    "structural.rs",
    "operations.rs",
    "temporal.rs",
    "timed_model.rs",
    "protocol_body.rs",
    "protocol_locations.rs",
    "natural.rs",
    "refinement.rs",
    "state.rs",
    "step_relation.rs",
    "extension.rs",
    "extension_body.rs",
    "source_map.rs",
    "stage_storage.rs",
    "operation_effects.rs",
    "checked_term_data.rs",
];

/// The banned spellings of FR-038-AC-513.
const BANNED: &[&str] = &[
    "serde_json::from_",
    "serde_json::Value",
    "serde_json::Map",
    "serde_json::value",
    "serde_json::json!",
    "serde_json::Deserializer",
    "serde_json::StreamDeserializer",
    "json!(",
    "RawValue",
    "num_bigint",
    "BigInt",
    "BigUint",
    ".sort(",
    ".sort_by(",
    ".sort_by_key(",
    ".sort_by_cached_key(",
    "BTreeMap",
    "BTreeSet",
    "HashMap",
];

fn test_only(attributes: &[syn::Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.path().is_ident("cfg")
            && attribute
                .parse_args::<syn::Path>()
                .is_ok_and(|path| path.is_ident("test"))
    })
}

#[derive(Default)]
struct Inspection {
    found: Vec<String>,
}

impl Inspection {
    fn check(&mut self, spelling: &str) {
        if let Some(word) = BANNED.iter().find(|word| spelling.contains(**word)) {
            self.found.push(format!("{word}: {spelling}"));
        }
    }

    fn imports(&mut self, prefix: &str, tree: &syn::UseTree) {
        match tree {
            syn::UseTree::Path(path) => {
                self.imports(&format!("{prefix}{}::", path.ident), &path.tree);
            }
            syn::UseTree::Name(name) => self.check(&format!("{prefix}{}", name.ident)),
            syn::UseTree::Rename(rename) => self.check(&format!("{prefix}{}", rename.ident)),
            syn::UseTree::Group(group) => {
                for item in &group.items {
                    self.imports(prefix, item);
                }
            }
            syn::UseTree::Glob(_) => self.check(prefix),
        }
    }
}

impl<'ast> Visit<'ast> for Inspection {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        let attributes: &[syn::Attribute] = match item {
            syn::Item::Const(item) => &item.attrs,
            syn::Item::Enum(item) => &item.attrs,
            syn::Item::ExternCrate(item) => &item.attrs,
            syn::Item::Fn(item) => &item.attrs,
            syn::Item::ForeignMod(item) => &item.attrs,
            syn::Item::Impl(item) => &item.attrs,
            syn::Item::Macro(item) => &item.attrs,
            syn::Item::Mod(item) => &item.attrs,
            syn::Item::Static(item) => &item.attrs,
            syn::Item::Struct(item) => &item.attrs,
            syn::Item::Trait(item) => &item.attrs,
            syn::Item::TraitAlias(item) => &item.attrs,
            syn::Item::Type(item) => &item.attrs,
            syn::Item::Union(item) => &item.attrs,
            syn::Item::Use(item) => &item.attrs,
            _ => &[][..],
        };
        if !test_only(attributes) {
            visit::visit_item(self, item);
        }
    }

    fn visit_impl_item(&mut self, item: &'ast syn::ImplItem) {
        let attributes: &[syn::Attribute] = match item {
            syn::ImplItem::Const(item) => &item.attrs,
            syn::ImplItem::Fn(item) => &item.attrs,
            syn::ImplItem::Type(item) => &item.attrs,
            syn::ImplItem::Macro(item) => &item.attrs,
            _ => &[][..],
        };
        if !test_only(attributes) {
            visit::visit_impl_item(self, item);
        }
    }

    fn visit_trait_item(&mut self, item: &'ast syn::TraitItem) {
        let attributes: &[syn::Attribute] = match item {
            syn::TraitItem::Const(item) => &item.attrs,
            syn::TraitItem::Fn(item) => &item.attrs,
            syn::TraitItem::Type(item) => &item.attrs,
            syn::TraitItem::Macro(item) => &item.attrs,
            _ => &[][..],
        };
        if !test_only(attributes) {
            visit::visit_trait_item(self, item);
        }
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        self.imports("", &item.tree);
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        let spelling = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        self.check(&spelling);
        visit::visit_path(self, path);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        self.check(&format!(".{}(", call.method));
        visit::visit_expr_method_call(self, call);
    }

    fn visit_macro(&mut self, invocation: &'ast syn::Macro) {
        let path = invocation
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        self.check(&format!("{path}!("));
        visit::visit_macro(self, invocation);
        // Macro bodies are token streams, not syntax visited by syn.
        self.check(
            &invocation
                .tokens
                .to_string()
                .split_whitespace()
                .collect::<String>(),
        );
    }
}

fn banned_lines(file: &str, source: &str) -> Vec<String> {
    let syntax =
        syn::parse_file(source).unwrap_or_else(|error| panic!("read-path syntax {file}: {error}"));
    let mut inspection = Inspection::default();
    inspection.visit_file(&syntax);
    inspection
        .found
        .into_iter()
        .map(|hit| format!("{file}: {hit}"))
        .collect()
}

/// Test-only items do not hide later production items or methods.
#[trace("TC-472", "FR-038-AC-513")]
#[test]
fn inspection_continues_after_test_only_items() {
    let source = r#"
        #[cfg(test)] use serde_json::Value;
        #[cfg(test)] fn helper() { let _ = serde_json::from_str::<()>("{}"); }
        #[cfg(test)] mod tests { fn helper() { let _ = std::collections::BTreeMap::<(), ()>::new(); } }
        fn production() {
            let _ = serde_json::from_str::<()>("{}");
            let _ = serde_json::json!({});
        }
        use std::collections::{BTreeMap, BTreeSet as Set};
        struct Reader;
        impl Reader {
            #[cfg(test)] fn helper() { let _ = serde_json::from_str::<()>("{}"); }
            fn read() { let mut rows = [1]; rows.sort(); }
        }
    "#;
    let hits = banned_lines("fixture.rs", source);
    assert_eq!(hits.len(), 5, "{hits:?}");
    for word in [
        "serde_json::from_",
        "serde_json::json!",
        "BTreeMap",
        "BTreeSet",
        ".sort(",
    ] {
        assert!(
            hits.iter().any(|hit| hit.contains(word)),
            "{word}: {hits:?}"
        );
    }
}

/// The V2 read path contains no banned parser, tree, big-integer, stable sort
/// or ordered/hashed container.
#[trace("TC-472", "FR-038-AC-513")]
#[test]
fn the_read_path_uses_no_banned_parser_container_or_sort() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("crates/quire-contract-model/src/checked_package/v2");
    let mut found = Vec::new();
    for file in READ_PATH {
        let source = std::fs::read_to_string(root.join(file))
            .unwrap_or_else(|error| panic!("read-path file {file}: {error}"));
        found.extend(banned_lines(file, &source));
    }
    assert!(
        found.is_empty(),
        "{} banned occurrences on the read path:\n{}",
        found.len(),
        found.join("\n")
    );
}
