---
id: SR-202611070951309528
title: IR-1404 PR 972 gap-analysis
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; crates/quire-contract-model/src/checked_package/common.rs,
  crates/quire-contract-model/src/checked_package/terms.rs, crates/quire-contract-model/src/checked_package/v2/claim_terminal.rs,
  crates/quire-contract-model/src/checked_package/v2/contract_package.rs, crates/quire-contract-model/src/checked_package/v2/extension.rs,
  crates/quire-contract-model/src/checked_package/v2/frame.rs, crates/quire-contract-model/src/checked_package/v2/graph_storage.rs,
  crates/quire-contract-model/src/checked_package/v2/identity.rs, crates/quire-contract-model/src/checked_package/v2/lower.rs,
  crates/quire-contract-model/src/checked_package/v2/mod.rs, crates/quire-contract-model/src/checked_package/v2/operations.rs,
  crates/quire-contract-model/src/checked_package/v2/owner.rs, crates/quire-contract-model/src/checked_package/v2/protocol_body.rs,
  crates/quire-contract-model/src/checked_package/v2/stage_storage.rs, crates/quire-contract-model/src/checked_package/v2/state.rs,
  crates/quire-contract-model/src/checked_package/v2/temporal.rs, crates/quire-contract-model/src/checked_package/v2/typed_terms.rs,
  crates/quire-contract-model/src/identity.rs, tests/it/checked_package_v2_recursive_leaves.rs,
  tests/it/public_interface.rs, tests/it/support/qspec_read_proof.rs
review_set: subset
---

## Summary

Ticket: IR-1404. Inspected every hunk across 21 files. Removed private helpers have no remaining callers; cfg(test) methods serve existing unit oracles. ReadInputBinding::new callers are already test-support gated. Owner Deserialize and refuse_bytes bodies move intact. TemporalContext passes the same catalog and lock; meter calls, refusal precedence, locus and caller accounting remain intact. Borrow, clone, iteration and sort rewrites preserve behavior. No new API, duplicate, suppression, gate narrowing, dependency or algorithm. No applicable AssuranceProfile found.

## Verdict

CONDITIONAL. Computed whole-repository matrix: 1029 tagged, 96 untagged, 26 method-without-symbol, 2 tagged-by-ignored-test. These existing gaps remain outside this cleanup; no new scoped gap. Repository-wide assurance is not PASS. Plan completion: not assessed. Optional semantic gap review skipped.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

Supplied workspace Clippy logs inspected: default lib, all targets conformance-cli, and all targets conformance-cli,test-support pass. Scoped fmt, string-edge scanner and independent diff check pass. No broad runtime or aggregate rerun, per dispatch; postmerge aggregate remains lead-owned. AC-511 whole proof, AC-513 static closure and F1 migration remain deferred.

Computed whole-repository matrix: 1029 tagged, 96 untagged, 26 method-without-symbol, 2 tagged-by-ignored-test. These existing gaps remain outside this cleanup; no new scoped gap. Repository-wide assurance is not PASS. Plan completion: not assessed. Optional semantic gap review skipped.

```yaml
scope:
- id: crates/quire-contract-model/src/checked_package/common.rs
  path: crates/quire-contract-model/src/checked_package/common.rs
  role: examined
  excerpt: "/// Runs a test oracle over a `Value` with an [`Ids`] that charges nothing.\n\
    #[cfg(test)]\n    #[cfg(test)]\n    #[cfg(test)]"
- id: crates/quire-contract-model/src/checked_package/terms.rs
  path: crates/quire-contract-model/src/checked_package/terms.rs
  role: examined
  excerpt: Removed unused private helpers/imports; all diff hunks examined.
- id: crates/quire-contract-model/src/checked_package/v2/claim_terminal.rs
  path: crates/quire-contract-model/src/checked_package/v2/claim_terminal.rs
  role: examined
  excerpt: '                .cloned();'
- id: crates/quire-contract-model/src/checked_package/v2/contract_package.rs
  path: crates/quire-contract-model/src/checked_package/v2/contract_package.rs
  role: examined
  excerpt: "        _limits: CheckedPackageReadLimits,\n        for (position, (_node,\
    \ kind)) in self.semantic_graph.nodes.iter().zip(&kinds).enumerate() {"
- id: crates/quire-contract-model/src/checked_package/v2/extension.rs
  path: crates/quire-contract-model/src/checked_package/v2/extension.rs
  role: examined
  excerpt: "            index.sort_unstable_by_key(|(left, _)| *left);\n         \
    \   index.sort_unstable_by_key(|(left, _)| *left);"
- id: crates/quire-contract-model/src/checked_package/v2/frame.rs
  path: crates/quire-contract-model/src/checked_package/v2/frame.rs
  role: examined
  excerpt: '    #[cfg(test)]'
- id: crates/quire-contract-model/src/checked_package/v2/graph_storage.rs
  path: crates/quire-contract-model/src/checked_package/v2/graph_storage.rs
  role: examined
  excerpt: Removed unused private helpers/imports; all diff hunks examined.
- id: crates/quire-contract-model/src/checked_package/v2/identity.rs
  path: crates/quire-contract-model/src/checked_package/v2/identity.rs
  role: examined
  excerpt: "                    at,\n            at,"
- id: crates/quire-contract-model/src/checked_package/v2/lower.rs
  path: crates/quire-contract-model/src/checked_package/v2/lower.rs
  role: examined
  excerpt: "#[cfg(test)]\n    for fact in facts.iter() {\n                       \
    \         ReferenceMember::Target | ReferenceMember::ResultType => {\n       \
    \                             EdgeRole::Typing\n                             \
    \   }"
- id: crates/quire-contract-model/src/checked_package/v2/mod.rs
  path: crates/quire-contract-model/src/checked_package/v2/mod.rs
  role: examined
  excerpt: '                ReferenceMember::Target | ReferenceMember::ResultType
    => lower::EdgeRole::Typing,'
- id: crates/quire-contract-model/src/checked_package/v2/operations.rs
  path: crates/quire-contract-model/src/checked_package/v2/operations.rs
  role: examined
  excerpt: "    let source_unit = carried_unit_and_mode(source_type, graph)\n    \
    \    .or_else(|| bounded_unit(source_type, graph))\n            let position =\
    \ *graph.index.get(source_type)?;\n        .and_then(|type_id| owners.variant_family(type_id));\n\
    \                || resolve_family_with_lookup(type_node, nodes, kinds, index,\
    \ fast_index),\n                        type_node, nodes, kinds, index, fast_index,\
    \ owners,\n        type_id,\n        referenced_type(binder).and_then(|target|\
    \ graph.index.get(target).copied())\n    if conforms_to_type(&node.semantic_type,\
    \ Some(root_type), graph, owners, meter)? {\n    let"
- id: crates/quire-contract-model/src/checked_package/v2/owner.rs
  path: crates/quire-contract-model/src/checked_package/v2/owner.rs
  role: examined
  excerpt: "    CheckedOccurrenceRole, CheckedPackageRefusalCause, CheckedPackageRefusalCode,\n\
    impl<'de> Deserialize<'de> for CheckedNodeOwner {\n    fn deserialize<D: serde::Deserializer<'de>>(deserializer:\
    \ D) -> Result<Self, D::Error> {\n        #[derive(Deserialize)]\n        #[serde(tag\
    \ = \"kind\", rename_all = \"snake_case\", deny_unknown_fields)]\n        enum\
    \ Wire {\n            Source {\n                authority: Box<str>,\n       \
    \         identity: Box<str>,\n            },\n            Model {\n         \
    \       identity: Box<str>,\n                node: Box<str>,\n            },\n\
    \            ModelRoot {\n         "
- id: crates/quire-contract-model/src/checked_package/v2/protocol_body.rs
  path: crates/quire-contract-model/src/checked_package/v2/protocol_body.rs
  role: examined
  excerpt: Removed unused private helpers/imports; all diff hunks examined.
- id: crates/quire-contract-model/src/checked_package/v2/stage_storage.rs
  path: crates/quire-contract-model/src/checked_package/v2/stage_storage.rs
  role: examined
  excerpt: "/// Pays the boxed byte-stream refusal before constructing it, without\
    \ a path.\npub(super) fn refuse_bytes(\n    meter: &mut WorkMeter<'_>,\n    site:\
    \ &Site<'_>,\n    code: CheckedPackageRefusalCode,\n) -> ValidationFailure {\n\
    \    if let Err(stop) = diagnostic(\n        meter,\n        site,\n        size_of::<CheckedPackageRefusal>()\
    \ + size_of::<CheckedPackageRefusalLocation>(),\n        CheckedPackagePendingOutcome::Refused(code),\n\
    \    ) {\n        return stop;\n    }\n    ValidationFailure::refused_bytes(code)\n\
    }\n"
- id: crates/quire-contract-model/src/checked_package/v2/state.rs
  path: crates/quire-contract-model/src/checked_package/v2/state.rs
  role: examined
  excerpt: "    let (context, context_kind) =\n        join(anchor, body.context,\
    \ is_operation_context, graph).map_err(|refusal| {\n                body.context,\n\
    \        body.frame,\n        model_refusal(refusal, target_at(2).key(\"value\"\
    ).key(\"target\"), body.frame)\n                .get(body.frame)\n        body.anchor,\n\
    \            body.anchor,\n        body.anchor,\n    .map_err(|refusal| model_refusal(refusal,\
    \ target_at(0), body.anchor))?;\n        .get(body.anchor)\n            body.anchor,\n\
    \        body.subject,\n    .map_err(|refusal| model_refusal(refusal, target_at(1),\
    \ body.subject))?;\n        .get(body.subj"
- id: crates/quire-contract-model/src/checked_package/v2/temporal.rs
  path: crates/quire-contract-model/src/checked_package/v2/temporal.rs
  role: examined
  excerpt: "    #[cfg(test)]\n                &context,\n    context: &TemporalContext<'_>,\n\
    \    let catalog = context.catalog;\n    let lock = context.lock;\n          \
    \  graph.node(over).is_some_and(|(parameter, kind)| {\n        let refusal = match\
    \ graph.node(over) {\n            Some(_) if !node.dependencies.contains(over)\
    \ => {\n    let Some(&position) = graph.index.get(target) else {\n        let\
    \ alias_package = alias_root.and_then(|root| match graph.node(root)? {\n     \
    \   .and_then(|target| graph.index.get(target).copied())"
- id: crates/quire-contract-model/src/checked_package/v2/typed_terms.rs
  path: crates/quire-contract-model/src/checked_package/v2/typed_terms.rs
  role: examined
  excerpt: "                identity: fixture.identity,\n    Binding { _value: &'a\
    \ Value },\n                    _value: wire.field(TermField::Value)?,"
- id: crates/quire-contract-model/src/identity.rs
  path: crates/quire-contract-model/src/identity.rs
  role: examined
  excerpt: '    #[cfg(test)]'
- id: tests/it/checked_package_v2_recursive_leaves.rs
  path: tests/it/checked_package_v2_recursive_leaves.rs
  role: examined
  excerpt: '    CheckedPackageV2ReadResult, CompleteLoweringProfileV2, ContractDependencies,'
- id: tests/it/public_interface.rs
  path: tests/it/public_interface.rs
  role: examined
  excerpt: "        CheckedPackageRefusalCode, CheckedPackageV2ReadResult,\n    use\
    \ quire_contract_model::{CheckedPackageReadLimits, CheckedPackageV2ReadResult,\
    \ CheckedTerm};"
- id: tests/it/support/qspec_read_proof.rs
  path: tests/it/support/qspec_read_proof.rs
  role: examined
  excerpt: '    #[cfg(feature = "test-support")]'
```

## Final-head continuation

Final-head continuation: PASS for PR 972 cleanup. Compared the refreshed candidate with current main and the previously reviewed candidate: production files are identical to the prior reviewed candidate; only inherited PR 973 test changes plus one guarantee-composition rustfmt hunk differ. The new hunk preserves call arguments, evidence construction and assertion; layout and optional trailing comma only. Final PR scope is 22 files. Whole-workspace fmt log is clean; independent diff check passed. No new finding. Prior method verdict and deferred repository assurance limits remain unchanged. Runtime/aggregate remains postmerge lead-owned.

Examined: tests/it/checked_package_v2_guarantee_composition.rs:1346, role: examined, excerpt: `let reread = crate::read_and_account(&canonical_bytes, CheckedPackageReadLimits::bounded(), &{`.
