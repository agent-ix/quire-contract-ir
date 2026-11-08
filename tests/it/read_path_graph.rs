// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Fail-closed syntactic reachability for the reader's local call edges.
//! Unresolved local receiver/indirect dispatch is a proof gap, never a reason
//! to scan every unrelated method with the same name or to omit the edge.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use syn::visit::{self, Visit};

#[derive(Clone)]
pub struct Function {
    pub module: String,
    pub owner: Option<String>,
    pub signature: syn::Signature,
    pub body: syn::Block,
}

#[derive(Default)]
pub struct Graph {
    pub functions: BTreeMap<String, Function>,
    pub imports: BTreeMap<String, BTreeMap<String, String>>,
    methods: BTreeSet<String>,
    types: BTreeSet<String>,
    duplicate: BTreeSet<String>,
}

fn test_only(attrs: &[syn::Attribute]) -> bool {
    super::test_only(attrs)
}

fn path_text(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn imports(prefix: &str, tree: &syn::UseTree, out: &mut BTreeMap<String, String>) {
    match tree {
        syn::UseTree::Path(p) => imports(&format!("{prefix}{}::", p.ident), &p.tree, out),
        syn::UseTree::Name(n) => {
            out.insert(n.ident.to_string(), format!("{prefix}{}", n.ident));
        }
        syn::UseTree::Rename(n) => {
            out.insert(n.rename.to_string(), format!("{prefix}{}", n.ident));
        }
        syn::UseTree::Group(g) => {
            for t in &g.items {
                imports(prefix, t, out)
            }
        }
        syn::UseTree::Glob(_) => {
            out.insert(
                format!("*{prefix}"),
                prefix.trim_end_matches("::").to_owned(),
            );
        }
    }
}

fn absolute(module: &str, path: &str) -> String {
    let mut base = module
        .split("::")
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    let mut parts = path.split("::").peekable();
    match parts.peek().copied() {
        Some("crate") => {
            base.clear();
            parts.next();
        }
        Some("self") => {
            parts.next();
        }
        Some("super") => {
            while parts.peek() == Some(&"super") {
                base.pop();
                parts.next();
            }
        }
        _ => {}
    }
    base.extend(parts);
    base.join("::")
}

// Rust 2021 prelude name resolution, applied only when the module has no
// shadowing item or explicit import. These bindings are not scan exceptions.
#[derive(Clone, Copy)]
enum PreludeBinding {
    Some,
    None,
    Ok,
    Err,
    Vec,
    String,
    Box,
    Option,
    Result,
}
impl PreludeBinding {
    fn named(name: &str) -> Option<Self> {
        match name {
            "Some" => Some(Self::Some),
            "None" => Some(Self::None),
            "Ok" => Some(Self::Ok),
            "Err" => Some(Self::Err),
            "Vec" => Some(Self::Vec),
            "String" => Some(Self::String),
            "Box" => Some(Self::Box),
            "Option" => Some(Self::Option),
            "Result" => Some(Self::Result),
            _ => None,
        }
    }
    fn path(self) -> &'static str {
        match self {
            Self::Some => "core::option::Option::Some",
            Self::None => "core::option::Option::None",
            Self::Ok => "core::result::Result::Ok",
            Self::Err => "core::result::Result::Err",
            Self::Vec => "std::vec::Vec",
            Self::String => "std::string::String",
            Self::Box => "std::boxed::Box",
            Self::Option => "core::option::Option",
            Self::Result => "core::result::Result",
        }
    }
}
fn standard_variant(path: &str) -> bool {
    matches!(
        path,
        "core::result::Result::Ok"
            | "core::result::Result::Err"
            | "core::option::Option::Some"
            | "core::option::Option::None"
    )
}
fn external_standard(path: &str) -> bool {
    matches!(path.split("::").next(), Some("std" | "core" | "alloc"))
}

impl Graph {
    pub fn from_source(source: &str) -> Self {
        let mut graph = Self::default();
        let syntax = syn::parse_file(source).expect("reachability fixture syntax");
        graph.items("", Path::new("."), Path::new("."), &syntax.items, false);
        graph
    }
    pub fn from_crate(root: &Path) -> Self {
        let mut graph = Self::default();
        let source = std::fs::read_to_string(root).expect("crate source");
        graph.items(
            "",
            root.parent().expect("crate source directory"),
            root.parent().expect("crate source directory"),
            &syn::parse_file(&source).expect("crate syntax").items,
            true,
        );
        graph
    }
    fn items(
        &mut self,
        module: &str,
        directory: &Path,
        source_directory: &Path,
        items: &[syn::Item],
        load_files: bool,
    ) {
        for item in items {
            if let syn::Item::Mod(m) = item {
                if !test_only(&m.attrs) {
                    let child = if module.is_empty() {
                        m.ident.to_string()
                    } else {
                        format!("{module}::{}", m.ident)
                    };
                    self.imports.entry(child).or_default();
                }
            }
        }
        for item in items {
            let ident = match item {
                syn::Item::Struct(t) if !test_only(&t.attrs) => Some(&t.ident),
                syn::Item::Enum(t) if !test_only(&t.attrs) => Some(&t.ident),
                syn::Item::Type(t) if !test_only(&t.attrs) => Some(&t.ident),
                _ => None,
            };
            if let Some(ident) = ident {
                self.types.insert(if module.is_empty() {
                    ident.to_string()
                } else {
                    format!("{module}::{ident}")
                });
            }
        }
        let aliases = self.imports.entry(module.into()).or_default();
        for item in items {
            if let syn::Item::Use(item) = item {
                if !test_only(&item.attrs) {
                    imports("", &item.tree, aliases);
                }
            }
        }
        for item in items {
            match item {
                syn::Item::Struct(t) if !test_only(&t.attrs) => {
                    self.types.insert(if module.is_empty() {
                        t.ident.to_string()
                    } else {
                        format!("{module}::{}", t.ident)
                    });
                }
                syn::Item::Enum(t) if !test_only(&t.attrs) => {
                    self.types.insert(if module.is_empty() {
                        t.ident.to_string()
                    } else {
                        format!("{module}::{}", t.ident)
                    });
                }
                syn::Item::Trait(t) if !test_only(&t.attrs) => {
                    for item in &t.items {
                        if let syn::TraitItem::Fn(f) = item {
                            if !test_only(&f.attrs) {
                                self.methods.insert(f.sig.ident.to_string());
                            }
                        }
                    }
                }
                syn::Item::Fn(f) if !test_only(&f.attrs) => self.insert(
                    format!(
                        "{}{}",
                        if module.is_empty() {
                            String::new()
                        } else {
                            format!("{module}::")
                        },
                        f.sig.ident
                    ),
                    Function {
                        module: module.into(),
                        owner: None,
                        signature: f.sig.clone(),
                        body: (*f.block).clone(),
                    },
                ),
                syn::Item::Impl(i) if !test_only(&i.attrs) => {
                    for item in &i.items {
                        if let syn::ImplItem::Fn(f) = item {
                            if !test_only(&f.attrs) {
                                self.methods.insert(f.sig.ident.to_string());
                            }
                        }
                    }
                    let syn::Type::Path(ty) = i.self_ty.as_ref() else {
                        continue;
                    };
                    let owner = self.resolve(module, &path_text(&ty.path));
                    for item in &i.items {
                        if let syn::ImplItem::Fn(f) = item {
                            if !test_only(&f.attrs) {
                                self.methods.insert(f.sig.ident.to_string());
                                self.insert(
                                    format!("{owner}::{}", f.sig.ident),
                                    Function {
                                        module: module.into(),
                                        owner: Some(owner.clone()),
                                        signature: f.sig.clone(),
                                        body: f.block.clone(),
                                    },
                                );
                            }
                        }
                    }
                }
                syn::Item::Mod(m) if !test_only(&m.attrs) => {
                    let name = m.ident.to_string();
                    let child = if module.is_empty() {
                        name.clone()
                    } else {
                        format!("{module}::{name}")
                    };
                    if let Some((_, items)) = &m.content {
                        self.items(
                            &child,
                            &directory.join(&name),
                            &directory.join(&name),
                            items,
                            load_files,
                        );
                    } else if load_files {
                        let custom = m.attrs.iter().find_map(|a| {
                            if !a.path().is_ident("path") {
                                return None;
                            }
                            let syn::Meta::NameValue(n) = &a.meta else {
                                return None;
                            };
                            let syn::Expr::Lit(l) = &n.value else {
                                return None;
                            };
                            let syn::Lit::Str(s) = &l.lit else {
                                return None;
                            };
                            Some(source_directory.join(s.value()))
                        });
                        let path = custom.unwrap_or_else(|| {
                            let file = directory.join(format!("{name}.rs"));
                            if file.exists() {
                                file
                            } else {
                                directory.join(&name).join("mod.rs")
                            }
                        });
                        let source = std::fs::read_to_string(&path)
                            .unwrap_or_else(|e| panic!("module {}: {e}", path.display()));
                        let base = if path.file_name().is_some_and(|n| n == "mod.rs") {
                            path.parent().expect("module directory").to_owned()
                        } else {
                            path.parent()
                                .expect("module directory")
                                .join(path.file_stem().expect("module name"))
                        };
                        self.items(
                            &child,
                            &base,
                            path.parent().expect("source directory"),
                            &syn::parse_file(&source).expect("module syntax").items,
                            true,
                        );
                    }
                }
                _ => {}
            }
        }
    }
    fn insert(&mut self, key: String, function: Function) {
        if self.functions.insert(key.clone(), function).is_some() {
            self.duplicate.insert(key);
        }
    }
    pub fn resolve(&self, module: &str, path: &str) -> String {
        let first = path.split("::").next().unwrap_or("");
        if matches!(first, "crate" | "self" | "super") {
            return self.follow(absolute(module, path));
        }
        if let Some(alias) = self.imports.get(module).and_then(|m| m.get(first)) {
            let suffix = &path[first.len()..];
            return self.follow(format!("{}{suffix}", self.import_target(module, alias)));
        }
        let local = if module.is_empty() {
            path.into()
        } else {
            format!("{module}::{path}")
        };
        let local_first = if module.is_empty() {
            first.to_owned()
        } else {
            format!("{module}::{first}")
        };
        if self.functions.contains_key(&local) || self.types.contains(&local_first) {
            return local;
        }
        if let Some(binding) = PreludeBinding::named(first) {
            return format!("{}{}", binding.path(), &path[first.len()..]);
        }
        if !path.contains("::") {
            local
        } else {
            path.into()
        }
    }
    fn import_target(&self, module: &str, path: &str) -> String {
        let first = path.split("::").next().unwrap_or("");
        if matches!(first, "crate" | "self" | "super") {
            return absolute(module, path);
        }
        let local = if module.is_empty() {
            path.to_owned()
        } else {
            format!("{module}::{path}")
        };
        let child = if module.is_empty() {
            first.to_owned()
        } else {
            format!("{module}::{first}")
        };
        if self.imports.contains_key(&child) {
            local
        } else {
            path.to_owned()
        }
    }
    fn follow(&self, mut path: String) -> String {
        let mut seen = BTreeSet::new();
        while seen.insert(path.clone()) {
            let mut rewrite = None;
            let parts = path.split("::").collect::<Vec<_>>();
            for i in (0..parts.len()).rev() {
                let module = parts[..i].join("::");
                if let Some(alias) = self.imports.get(&module).and_then(|m| m.get(parts[i])) {
                    let suffix = if i + 1 == parts.len() {
                        String::new()
                    } else {
                        format!("::{}", parts[i + 1..].join("::"))
                    };
                    rewrite = Some(format!("{}{suffix}", self.import_target(&module, alias)));
                    break;
                }
            }
            match rewrite {
                Some(next) if next != path => path = next,
                _ => break,
            }
        }
        path
    }
    pub fn reached(&self, roots: &[&str]) -> Reachability {
        let mut result = Reachability::default();
        let mut pending = roots.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        while let Some(key) = pending.pop() {
            if !result.functions.insert(key.clone()) {
                continue;
            }
            let Some(function) = self.functions.get(&key) else {
                result
                    .unresolved
                    .insert(format!("missing root/local function {key}"));
                continue;
            };
            if self.duplicate.contains(&key) {
                result
                    .unresolved
                    .insert(format!("ambiguous trait/impl function {key}"));
                result.functions.remove(&key);
                continue;
            }
            let mut calls = Calls {
                graph: self,
                function,
                edges: BTreeSet::new(),
                unresolved: BTreeSet::new(),
                bindings: BTreeMap::new(),
            };
            for arg in &function.signature.inputs {
                if let syn::FnArg::Typed(arg) = arg {
                    if let syn::Pat::Ident(p) = arg.pat.as_ref() {
                        if let Some(ty) = calls.type_name(&arg.ty) {
                            calls.bindings.insert(p.ident.to_string(), ty);
                        }
                    }
                }
            }
            calls.visit_block(&function.body);
            pending.extend(calls.edges);
            result.unresolved.extend(
                calls
                    .unresolved
                    .into_iter()
                    .map(|edge| format!("{key}: {edge}")),
            );
        }
        result
    }
}

#[derive(Default)]
pub struct Reachability {
    pub functions: BTreeSet<String>,
    pub unresolved: BTreeSet<String>,
}

struct Calls<'a> {
    graph: &'a Graph,
    function: &'a Function,
    edges: BTreeSet<String>,
    unresolved: BTreeSet<String>,
    bindings: BTreeMap<String, String>,
}

impl Calls<'_> {
    fn type_name(&self, ty: &syn::Type) -> Option<String> {
        match ty {
            syn::Type::Reference(r) => self.type_name(&r.elem),
            syn::Type::Path(p) => Some(
                self.graph
                    .resolve(&self.function.module, &path_text(&p.path)),
            ),
            _ => None,
        }
    }
    fn expression_type(&self, e: &syn::Expr) -> Option<String> {
        match e {
            syn::Expr::Reference(r) => self.expression_type(&r.expr),
            syn::Expr::Path(p) if p.path.is_ident("self") => self.function.owner.clone(),
            syn::Expr::Path(p) => p
                .path
                .get_ident()
                .and_then(|n| self.bindings.get(&n.to_string()).cloned()),
            syn::Expr::Call(c) => {
                let syn::Expr::Path(path) = c.func.as_ref() else {
                    return None;
                };
                let target = self.target(&path.path);
                if let Some(function) = self.graph.functions.get(&target) {
                    match &function.signature.output {
                        syn::ReturnType::Type(_, ty) => {
                            if matches!(ty.as_ref(),syn::Type::Path(p) if p.path.is_ident("Self")) {
                                function.owner.clone()
                            } else {
                                fn declared(
                                    graph: &Graph,
                                    module: &str,
                                    ty: &syn::Type,
                                ) -> Option<String> {
                                    match ty {
                                        syn::Type::Reference(r) => declared(graph, module, &r.elem),
                                        syn::Type::Path(p) => {
                                            Some(graph.resolve(module, &path_text(&p.path)))
                                        }
                                        _ => None,
                                    }
                                }
                                declared(self.graph, &function.module, ty)
                            }
                        }
                        syn::ReturnType::Default => None,
                    }
                } else {
                    let constructor = self
                        .graph
                        .resolve(&self.function.module, &path_text(&path.path));
                    self.graph
                        .types
                        .contains(&constructor)
                        .then_some(constructor)
                }
            }
            _ => None,
        }
    }
    fn target(&self, path: &syn::Path) -> String {
        let text = path_text(path);
        if let Some(suffix) = text.strip_prefix("Self::") {
            if let Some(owner) = &self.function.owner {
                return format!("{owner}::{suffix}");
            }
        }
        self.graph.resolve(&self.function.module, &text)
    }
}

impl<'ast> Visit<'ast> for Calls<'_> {
    // A nested declaration is not called merely because its parent is called.
    fn visit_item_fn(&mut self, _: &'ast syn::ItemFn) {}
    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let Some(init) = &local.init {
            let (name, ty) = match &local.pat {
                syn::Pat::Type(p) => {
                    if let syn::Pat::Ident(n) = p.pat.as_ref() {
                        (Some(n.ident.to_string()), self.type_name(&p.ty))
                    } else {
                        (None, None)
                    }
                }
                syn::Pat::Ident(n) => (Some(n.ident.to_string()), self.expression_type(&init.expr)),
                _ => (None, None),
            };
            if let (Some(name), Some(ty)) = (name, ty) {
                self.bindings.insert(name, ty);
            }
        }
        visit::visit_local(self, local);
    }
    fn visit_macro(&mut self, invocation: &'ast syn::Macro) {
        self.unresolved.insert(format!(
            "unresolved macro expansion {}",
            path_text(&invocation.path)
        ));
    }
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = call.func.as_ref() {
            let target = self.target(&path.path);
            if self.graph.functions.contains_key(&target) {
                self.edges.insert(target);
            } else if !standard_variant(&target) {
                if call
                    .args
                    .iter()
                    .filter_map(|a| self.expression_type(a))
                    .any(|ty| self.graph.types.contains(&ty))
                {
                    self.unresolved.insert(format!(
                        "external call {target} may dispatch a reader-local trait implementation"
                    ));
                }
                let last = path
                    .path
                    .segments
                    .last()
                    .expect("call path")
                    .ident
                    .to_string();
                if (!external_standard(&target) && self.graph.methods.contains(&last))
                    || target.starts_with("checked_package::")
                {
                    self.unresolved
                        .insert(format!("unresolved associated/indirect call {target}"));
                }
            }
        } else {
            self.unresolved.insert("indirect expression call".into());
        }
        visit::visit_expr_call(self, call);
    }
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let method = call.method.to_string();
        if self.graph.methods.contains(&method) {
            if let Some(receiver) = self.expression_type(&call.receiver) {
                let target = format!("{receiver}::{method}");
                if self.graph.functions.contains_key(&target) {
                    self.edges.insert(target);
                } else {
                    self.unresolved.insert(format!(
                        "local method .{method} on unresolved receiver {receiver}"
                    ));
                }
            } else {
                self.unresolved.insert(format!(
                    "local method .{method} on a receiver with unresolved type"
                ));
            }
        }
        visit::visit_expr_method_call(self, call);
    }
}
