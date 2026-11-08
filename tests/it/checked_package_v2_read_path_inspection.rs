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

#[path = "read_path_graph.rs"]
mod root_graph;

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
    aliases: std::collections::BTreeMap<String, String>,
    globs: Vec<String>,
    skip_nested_functions: bool,
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
            syn::UseTree::Name(name) => {
                let path = format!("{prefix}{}", name.ident);
                self.aliases.insert(name.ident.to_string(), path.clone());
                self.check(&path);
            }
            syn::UseTree::Rename(rename) => {
                let path = format!("{prefix}{}", rename.ident);
                self.aliases.insert(rename.rename.to_string(), path.clone());
                self.check(&path);
            }
            syn::UseTree::Group(group) => {
                for item in &group.items {
                    self.imports(prefix, item);
                }
            }
            syn::UseTree::Glob(_) => {
                let prefix = prefix.trim_end_matches("::").to_owned();
                self.check(&prefix);
                if !self.globs.contains(&prefix) {
                    self.globs.push(prefix);
                }
            }
        }
    }
}

impl<'ast> Visit<'ast> for Inspection {
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        if !self.skip_nested_functions {
            visit::visit_item_fn(self, item);
        }
    }
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = call.func.as_ref() {
            if path
                .qself
                .as_ref()
                .is_some_and(|q| matches!(q.ty.as_ref(), syn::Type::Slice(_) | syn::Type::Array(_)))
            {
                if let Some(method) = path.path.segments.last() {
                    self.check(&format!(".{}(", method.ident));
                }
            }
        }
        visit::visit_expr_call(self, call);
    }
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
        let first = path
            .segments
            .first()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();
        if let Some(alias) = self.aliases.get(&first).cloned() {
            self.check(&format!("{}{}", alias, &spelling[first.len()..]));
        }
        for prefix in self.globs.clone() {
            self.check(&format!("{prefix}::{spelling}"));
        }
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
        // Inspect expression-shaped macro arguments as syntax, preserving
        // literals as literals. Other expansions remain unresolved in the
        // rooted graph; token substrings are not proof of a live construct.
        if let Ok(expression) = syn::parse_str::<syn::Expr>(&format!("[{}]", invocation.tokens)) {
            self.visit_expr(&expression);
        } else if let Ok(expression) = syn::parse2::<syn::Expr>(invocation.tokens.clone()) {
            self.visit_expr(&expression);
        }
    }
}

fn banned_lines(file: &str, source: &str) -> Vec<String> {
    let syntax =
        syn::parse_file(source).unwrap_or_else(|error| panic!("read-path syntax {file}: {error}"));
    let mut inspection = Inspection::default();
    for item in &syntax.items {
        if let syn::Item::Use(item) = item {
            if !test_only(&item.attrs) {
                inspection.imports("", &item.tree);
            }
        }
    }
    inspection.found.clear();
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

fn inspect_reached(graph: &root_graph::Graph, roots: &[&str]) -> (Vec<String>, Vec<String>) {
    let reached = graph.reached(roots);
    let mut found = Vec::new();
    for key in &reached.functions {
        let Some(function) = graph.functions.get(key) else {
            continue;
        };
        let mut inspection = Inspection {
            skip_nested_functions: true,
            ..Inspection::default()
        };
        if let Some(imports) = graph.imports.get(&function.module) {
            for (name, path) in imports {
                if name.starts_with('*') {
                    inspection.globs.push(graph.resolve(&function.module, path));
                } else {
                    inspection
                        .aliases
                        .insert(name.clone(), graph.resolve(&function.module, path));
                }
            }
        }
        inspection.visit_signature(&function.signature);
        inspection.visit_block(&function.body);
        found.extend(
            inspection
                .found
                .into_iter()
                .map(|hit| format!("{key}: {hit}")),
        );
    }
    (found, reached.unresolved.into_iter().collect())
}

#[trace("TC-472", "FR-038-AC-513")]
#[test]
fn aliases_globs_reexports_and_slice_ufcs_do_not_hide_banned_calls() {
    for source in [
        "use serde_json as sj; fn read(){sj::from_slice::<()>(b\"{}\");}",
        "use serde_json::*; fn read(){let _:Option<Value>=None;}",
        "pub use serde_json::from_slice as decode; fn read(){decode::<()>(b\"{}\");}",
        "fn read(){let mut rows=[1]; <[_]>::sort(&mut rows);}",
    ] {
        assert!(!banned_lines("fixture", source).is_empty(), "{source}");
    }
    assert!(banned_lines(
        "negative",
        r#"fn reader(){let text="serde_json::Value .sort("; let _=format!("serde_json::Value {}",text);} // serde_json::from_str"#
    )
    .is_empty());
}

#[trace("TC-472", "FR-038-AC-513")]
#[test]
fn a_new_reached_helper_after_cfg_test_is_discovered_without_a_file_list() {
    let graph = root_graph::Graph::from_source(
        r#"
        fn read(){helper::decode();}
        #[cfg(test)] mod tests{fn ignored(){let _:Option<serde_json::Value>=None;}}
        mod helper { pub fn decode(){late();} fn late(){let _=serde_json::from_slice::<()>(b"{}");} }
        fn v1_writer(){let _:Option<serde_json::Value>=None;}
    "#,
    );
    let (hits, unresolved) = inspect_reached(&graph, &["read"]);
    assert!(unresolved.is_empty(), "{unresolved:?}");
    assert_eq!(hits.len(), 1, "{hits:?}");
    assert!(hits[0].starts_with("helper::late:"));
}

#[trace("TC-472", "FR-038-AC-513")]
#[test]
fn unreachable_v1_and_postread_methods_are_not_attributed_to_the_reader() {
    let graph = root_graph::Graph::from_source(
        r#"
        struct Reader;
        impl Reader { fn read(){Self::checked();} fn checked(){} fn postread(){let _=std::collections::BTreeMap::<(),()>::new();} }
        struct V1;
        impl V1 { fn read(){let _=serde_json::from_slice::<()>(b"{}");} }
    "#,
    );
    let (hits, unresolved) = inspect_reached(&graph, &["Reader::read"]);
    assert!(hits.is_empty(), "{hits:?}");
    assert!(unresolved.is_empty(), "{unresolved:?}");
}

#[trace("TC-472", "FR-038-AC-513")]
#[test]
fn ambiguous_local_receiver_is_an_unresolved_proof_edge_not_a_guessed_call() {
    let graph = root_graph::Graph::from_source(
        r#"
        fn read<T>(value:T){value.decode();}
        struct V1;
        impl V1 {fn decode(){let _:Option<serde_json::Value>=None;}}
    "#,
    );
    let (hits, unresolved) = inspect_reached(&graph, &["read"]);
    assert!(
        hits.is_empty(),
        "an unrelated same-named V1 method is not reached: {hits:?}"
    );
    assert_eq!(unresolved.len(), 1, "{unresolved:?}");
    assert!(unresolved[0].contains(".decode"));
}

/// Only bodies reached from the real public entry are inspected. Unknown
/// local receiver/generic dispatch remains a separate, failing proof gap.
#[trace("TC-472", "FR-038-AC-513")]
#[test]
fn the_read_path_uses_no_banned_parser_container_or_sort() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("crates/quire-contract-model/src/lib.rs");
    let graph = root_graph::Graph::from_crate(&root);
    let (found, unresolved) = inspect_reached(
        &graph,
        &[
            "checked_package::v2::CheckedPackageV2::read",
            "checked_package::v2::CheckedPackageV2::read_with_budget",
        ],
    );
    let report = std::env::temp_dir().join(format!(
        "ir1404-f2-read-path-inspection-{}.json",
        std::process::id()
    ));
    std::fs::write(&report,serde_json::to_vec_pretty(&serde_json::json!({"forbidden_syntax_in_resolved_functions":found,"unresolved_static_edges":unresolved})).expect("structured proof report")).expect("write complete proof report");
    assert!(found.is_empty()&&unresolved.is_empty(),"Forbidden syntax in resolved reader functions ({}), sample:\n{}\nUnresolved static proof edges ({}), sample:\n{}\nComplete structured report: {}",found.len(),found.iter().take(20).cloned().collect::<Vec<_>>().join("\n"),unresolved.len(),unresolved.iter().take(20).cloned().collect::<Vec<_>>().join("\n"),report.display());
}

#[trace("TC-472", "FR-038-AC-513")]
#[test]
fn unshadowed_prelude_constructors_resolve_without_skipping_shadowed_helpers() {
    let graph = root_graph::Graph::from_source(
        r#"
        struct Unrelated;
        impl Unrelated {fn new(){let _=serde_json::from_slice::<()>(b"{}");}}
        fn read(){let _=Vec::<u8>::new();let _:Result<(),()>=Ok(());let _:Option<()>=Some(());let _=Err::<(),()>(());}
    "#,
    );
    let (hits, unresolved) = inspect_reached(&graph, &["read"]);
    assert!(hits.is_empty(), "{hits:?}");
    assert!(unresolved.is_empty(), "{unresolved:?}");
    for source in [
        r#"struct Vec;impl Vec{fn new(){let _=std::collections::BTreeMap::<(),()>::new();}}fn read(){Vec::new();}"#,
        r#"fn read(){Err();}fn Err(){let _=serde_json::from_slice::<()>(b"{}");}"#,
    ] {
        let graph = root_graph::Graph::from_source(source);
        let (hits, unresolved) = inspect_reached(&graph, &["read"]);
        assert_eq!(
            hits.len(),
            1,
            "shadowed names still reach their actual helper: {hits:?}"
        );
        assert!(unresolved.is_empty(), "{unresolved:?}");
    }
}

#[trace("TC-472", "FR-038-AC-513")]
#[test]
fn implicit_drop_and_overloaded_operators_cannot_escape_reachability() {
    let graph = root_graph::Graph::from_source(
        r#"
        struct Guard;
        impl Drop for Guard { fn drop(&mut self){ helper(); } }
        fn helper(){let _=serde_json::from_slice::<()>(b"{}");}
        fn read(){let _guard=Guard;}
        fn v1(){let _guard=Guard;}
        fn clean(){}
        fn operators(a:Unknown,b:Unknown){let _=a+b;let _=a[0];for _ in b {}}
    "#,
    );
    let (hits, unresolved) = inspect_reached(&graph, &["read"]);
    assert!(
        hits.iter().any(|hit| hit.starts_with("helper:")),
        "{hits:?}"
    );
    assert!(!unresolved.is_empty());
    let (hits, unresolved) = inspect_reached(&graph, &["clean"]);
    assert!(hits.is_empty() && unresolved.is_empty());
    let (_, unresolved) = inspect_reached(&graph, &["operators"]);
    for kind in ["operator dispatch", "index dispatch", "IntoIterator"] {
        assert!(
            unresolved.iter().any(|edge| edge.contains(kind)),
            "{unresolved:?}"
        );
    }
}
