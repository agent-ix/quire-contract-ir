---
id: SR-202611070951309526
title: IR-1404 PR970 gap-analysis
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-contract-ir; PR970; 19 V2 Rust files; marker and formatting
  diff
review_set: subset
---

## Summary

Ticket: IR-1404. The diff contains only 45 existing no-argument string_edge markers and two rustfmt hunks across 19 files. Inspected every marker target: wire decoding, closed spelling parsing, member/identifier projections, canonical number and type syntax, or semantic operation-name dispatch. The dependency macro explicitly returns the unchanged item for empty arguments. No scanner rule, allowlist, API, body behavior, dependency, suppression or CI change occurs. No vendoring or new duplicate implementation. No applicable AssuranceProfile was found.

## Verdict

CONDITIONAL. No new scoped assurance gap; repository assurance is CONDITIONAL because the existing computed matrix has 96 untagged criteria, 1 tagged-by-ignored-test and 26 method-without-symbol (1030 tagged). This is not a whole-repository PASS or a new marker-PR blocker.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Examined Scope

```yaml
scope:
- id: crates/quire-contract-model/src/checked_package/v2/document_de.rs:679
  path: crates/quire-contract-model/src/checked_package/v2/document_de.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\n    fn deserialize_newtype_struct<V: Visitor<'de>>(\n\
    \        self,\n        name: &'static str,\n        visitor: V,\n    ) -> Result<V::Value,\
    \ Error> {\n        if name == CLOSED_ENUM_TOKEN {\n            let node = super::TermNode::from_ref(self.node);\n\
    \            let tag = node\n                .get(\"kind\")\n                .and_then(super::TermNode::as_str)\n\
    \                .ok_or(Error::Invalid)?;\n            return visitor.visit_enum(ClosedEnum\
    \ { parent: self, tag });\n        }\n        if name == super::typed_terms::OPERATION_TOKEN\
    \ {\n            let context = self.cont"
- id: crates/quire-contract-model/src/checked_package/v2/document_de.rs:1051
  path: crates/quire-contract-model/src/checked_package/v2/document_de.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\n    fn next_key_seed<K: DeserializeSeed<'de>>(\n\
    \        &mut self,\n        seed: K,\n    ) -> Result<Option<K::Value>, Error>\
    \ {\n        let Some((name, value)) = self\n            .members\n          \
    \  .by_ref()\n            .find(|(name, _)| !(self.skip_kind && *name == \"kind\"\
    ))\n        else {\n            return Ok(None);\n        };\n        self.value\
    \ = Some(value);\n        self.name = name;\n        self.parent.context.push(Segment::Key(name));\n\
    \        let decoded = seed.deserialize(KeyDe { name });\n        if let Err(error)\
    \ = &decoded {\n            self.parent.context."
- id: crates/quire-contract-model/src/checked_package/v2/document_de.rs:1073
  path: crates/quire-contract-model/src/checked_package/v2/document_de.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\n    fn next_value_seed<T: DeserializeSeed<'de>>(&mut\
    \ self, seed: T) -> Result<T::Value, Error> {\n        let value = self\n    \
    \        .value\n            .take()\n            .ok_or_else(|| de::Error::custom(\"\
    a value without a member name\"))?;\n        let slots = self.path_slots && self.name\
    \ == \"path\";\n        let decoded = seed.deserialize(ValueDe {\n           \
    \ slots,\n            ..self.parent.child_with(value, false)\n        });\n  \
    \      if let Err(error) = &decoded {\n            self.parent.context.record_error(error);\n\
    \        }\n        self.parent.context.pop();\n  "
- id: crates/quire-contract-model/src/checked_package/v2/guarantee_composition.rs:133
  path: crates/quire-contract-model/src/checked_package/v2/guarantee_composition.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\nfn kind_of(spelling: &str) -> Option<CheckedFairnessKind>\
    \ {\n    match spelling {\n        \"weak\" => Some(CheckedFairnessKind::Weak),\n\
    \        \"strong\" => Some(CheckedFairnessKind::Strong),\n        _ => None,\n\
    \    }\n}\n\n#[ix_trace_rs::string_edge]\nfn granularity_of(spelling: &str) ->\
    \ Option<CheckedFairnessGranularity> {\n    match spelling {\n        \"whole\"\
    \ => Some(CheckedFairnessGranularity::Whole),\n        \"each\" => Some(CheckedFairnessGranularity::Each),\n\
    \        _ => None,\n    }\n}\n\nfn read_supply(\n    value: TermNode<'_>,\n \
    \   at: &Path<'static>,\n    ctx: &Parse<'_, '_>"
- id: crates/quire-contract-model/src/checked_package/v2/guarantee_composition.rs:142
  path: crates/quire-contract-model/src/checked_package/v2/guarantee_composition.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\nfn granularity_of(spelling: &str) -> Option<CheckedFairnessGranularity>\
    \ {\n    match spelling {\n        \"whole\" => Some(CheckedFairnessGranularity::Whole),\n\
    \        \"each\" => Some(CheckedFairnessGranularity::Each),\n        _ => None,\n\
    \    }\n}\n\nfn read_supply(\n    value: TermNode<'_>,\n    at: &Path<'static>,\n\
    \    ctx: &Parse<'_, '_>,\n) -> Result<CheckedFairnessSupply, Defect> {\n    let\
    \ object = closed(\n        value,\n        &[\"kind\", \"granularity\", \"context\"\
    , \"operation\"],\n        at,\n        ctx,\n    )?;\n    let fail = || ctx.invalid(at);\n\
    \    Ok(CheckedFairnessSupply {"
- id: crates/quire-contract-model/src/checked_package/v2/guarantee_composition.rs:494
  path: crates/quire-contract-model/src/checked_package/v2/guarantee_composition.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\nfn selects_infinite_trace<Body>(\n    clause:\
    \ &CheckedSemanticNodeV2<Body>,\n    graph: &StepGraph<'_, '_, Body>,\n    terms:\
    \ &[Option<CheckedTerm>],\n) -> bool {\n    let Some(CheckedTerm::Application\
    \ { operation, .. }) = graph\n        .index\n        .get(&clause.node_id)\n\
    \        .and_then(|&p| terms.get(p))\n        .and_then(Option::as_ref)\n   \
    \ else {\n        return false;\n    };\n    operation.record().laws.iter().any(|law|\
    \ {\n        law.role.as_ref() == \"temporal_profile\"\n            && law.definition.identity.as_ref()\
    \ == INFINITE_TRACE\n    })\n}\n\n/// The identity of t"
- id: crates/quire-contract-model/src/checked_package/v2/guarantee_composition.rs:863
  path: crates/quire-contract-model/src/checked_package/v2/guarantee_composition.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\nfn clause_is_liveness(\n    package: &CheckedPackageV2,\n\
    \    clause: &CheckedNodeId,\n    meter: &mut WorkMeter,\n    at: &Path<'static>,\n\
    ) -> Result<bool, ValidationFailure> {\n    let Some(CheckedTerm::Application\
    \ {\n        arguments,\n        operation,\n        ..\n    }) = package.term(clause)\n\
    \    else {\n        return Ok(false);\n    };\n    let infinite = operation.read().ok().is_some_and(|read|\
    \ {\n        read.laws\n            .iter()\n            .any(|law| law.definition.identity.as_ref()\
    \ == INFINITE_TRACE)\n    });\n    if !infinite {\n        return Ok(false);\n\
    \    }\n  "
- id: crates/quire-contract-model/src/checked_package/v2/hybrid.rs:127
  path: crates/quire-contract-model/src/checked_package/v2/hybrid.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\n    fn from_word(word: &str) -> Option<Self>\
    \ {\n        match word {\n            \"clock\" => Some(Self::Clock),\n     \
    \       \"hybrid\" => Some(Self::Hybrid),\n            _ => None,\n        }\n\
    \    }\n}\nimpl ClosedWord for HybridBindingRole {\n    #[ix_trace_rs::string_edge]\n\
    \    fn from_word(word: &str) -> Option<Self> {\n        match word {\n      \
    \      \"parameters\" => Some(Self::Parameters),\n            \"kind\" => Some(Self::Kind),\n\
    \            \"trigger\" => Some(Self::Trigger),\n            \"guard\" => Some(Self::Guard),\n\
    \            _ => None,\n        }\n    }\n}\nimpl ClosedWord "
- id: crates/quire-contract-model/src/checked_package/v2/hybrid.rs:137
  path: crates/quire-contract-model/src/checked_package/v2/hybrid.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\n    fn from_word(word: &str) -> Option<Self>\
    \ {\n        match word {\n            \"parameters\" => Some(Self::Parameters),\n\
    \            \"kind\" => Some(Self::Kind),\n            \"trigger\" => Some(Self::Trigger),\n\
    \            \"guard\" => Some(Self::Guard),\n            _ => None,\n       \
    \ }\n    }\n}\nimpl ClosedWord for HybridActivationKind {\n    #[ix_trace_rs::string_edge]\n\
    \    fn from_word(word: &str) -> Option<Self> {\n        (word == \"each\").then_some(Self::Each)\n\
    \    }\n}\nimpl ClosedWord for HybridTimeKind {\n    #[ix_trace_rs::string_edge]\n\
    \    fn from_word(word: &str) -> Optio"
- id: crates/quire-contract-model/src/checked_package/v2/hybrid.rs:149
  path: crates/quire-contract-model/src/checked_package/v2/hybrid.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\n    fn from_word(word: &str) -> Option<Self>\
    \ {\n        (word == \"each\").then_some(Self::Each)\n    }\n}\nimpl ClosedWord\
    \ for HybridTimeKind {\n    #[ix_trace_rs::string_edge]\n    fn from_word(word:\
    \ &str) -> Option<Self> {\n        match word {\n            \"model-time\" =>\
    \ Some(Self::ModelTime),\n            \"model-steps\" => Some(Self::ModelSteps),\n\
    \            _ => None,\n        }\n    }\n}\nimpl ClosedWord for HybridTemporalOperation\
    \ {\n    #[ix_trace_rs::string_edge]\n    fn from_word(word: &str) -> Option<Self>\
    \ {\n        (word == \"quire.op.temporal.holds\").then_some(Self::Hol"
- id: crates/quire-contract-model/src/checked_package/v2/hybrid.rs:155
  path: crates/quire-contract-model/src/checked_package/v2/hybrid.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\n    fn from_word(word: &str) -> Option<Self>\
    \ {\n        match word {\n            \"model-time\" => Some(Self::ModelTime),\n\
    \            \"model-steps\" => Some(Self::ModelSteps),\n            _ => None,\n\
    \        }\n    }\n}\nimpl ClosedWord for HybridTemporalOperation {\n    #[ix_trace_rs::string_edge]\n\
    \    fn from_word(word: &str) -> Option<Self> {\n        (word == \"quire.op.temporal.holds\"\
    ).then_some(Self::Holds)\n    }\n}\nfn decode_word<T: ClosedWord>(wire: &str)\
    \ -> Option<T> {\n    T::from_word(wire)\n}\nfn literal_word_as<T: ClosedWord>(term:\
    \ &CheckedTerm) -> Option<T> {\n    let "
- id: crates/quire-contract-model/src/checked_package/v2/hybrid.rs:165
  path: crates/quire-contract-model/src/checked_package/v2/hybrid.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\n    fn from_word(word: &str) -> Option<Self>\
    \ {\n        (word == \"quire.op.temporal.holds\").then_some(Self::Holds)\n  \
    \  }\n}\nfn decode_word<T: ClosedWord>(wire: &str) -> Option<T> {\n    T::from_word(wire)\n\
    }\nfn literal_word_as<T: ClosedWord>(term: &CheckedTerm) -> Option<T> {\n    let\
    \ CheckedTerm::Literal { value, .. } = term else {\n        return None;\n   \
    \ };\n    decode_word(value.as_str()?)\n}\n\nfn model_variable_class(term: &CheckedTerm,\
    \ slot: usize) -> Option<HybridVariableClass> {\n    let variables = term_members(term).first().and_then(term_value)?;\n\
    \    let variable ="
- id: crates/quire-contract-model/src/checked_package/v2/identity.rs:1404
  path: crates/quire-contract-model/src/checked_package/v2/identity.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\n    fn string_member(self, name: &str) ->\
    \ Option<&'a str> {\n        match self {\n            Self::Document(body) =>\
    \ body?.0.get(name)?.as_str(),\n            Self::Typed(super::CheckedTerm::Literal\
    \ { value, .. }) if name == \"value\" => {\n                value.as_str()\n \
    \           }\n            Self::Typed(_) => None,\n        }\n    }\n\n    ///\
    \ Whether the body is exactly the enum literal of `case` of the enum\n    ///\
    \ `declaration`: its `type` spells that node ID, read through the node's\n   \
    \ /// operand row with the ID's allocations charged.\n    fn is_enum_literal(\n\
    \    "
- id: crates/quire-contract-model/src/checked_package/v2/identity.rs:2245
  path: crates/quire-contract-model/src/checked_package/v2/identity.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\npub(super) fn duration_unit_key<B>(\n   \
    \ nodes: &[CheckedSemanticNodeV2<B>],\n    terms: &[Option<super::CheckedTerm>],\n\
    \    name: &str,\n    meter: &mut WorkMeter,\n) -> Result<Option<CheckedNodeId>,\
    \ ValidationFailure> {\n    meter.charge(u64::try_from(nodes.len()).unwrap_or(u64::MAX),\
    \ || {\n        JsonPointer::root().key(\"semantic_graph\").key(\"nodes\")\n \
    \   })?;\n    let Some((position, node)) = nodes.iter().enumerate().find(|(_,\
    \ node)| {\n        node.declaration.as_ref().is_some_and(|declaration| {\n  \
    \          name.strip_prefix(\"ix://quire/unit/\")\n                .is_some"
- id: crates/quire-contract-model/src/checked_package/v2/intake.rs:35
  path: crates/quire-contract-model/src/checked_package/v2/intake.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\nfn key_ordinal_node(value: TermNode<'_>)\
    \ -> bool {\n    const MAX_SAFE: &str = \"9007199254740991\";\n    if value\n\
    \        .as_u64()\n        .is_some_and(|number| number <= 9_007_199_254_740_991)\n\
    \    {\n        return true;\n    }\n    let Some(text) = value.as_str() else\
    \ {\n        return false;\n    };\n    !text.is_empty()\n        && !text.starts_with('0')\n\
    \        && text.bytes().all(|byte| byte.is_ascii_digit())\n        && (text.len()\
    \ > MAX_SAFE.len() || (text.len() == MAX_SAFE.len() && text > MAX_SAFE))\n}\n\n\
    fn position_member_shape(body: TermNode<'_>) -> bool {\n    if bod"
- id: crates/quire-contract-model/src/checked_package/v2/intake.rs:334
  path: crates/quire-contract-model/src/checked_package/v2/intake.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\nfn classify_wire_fault(\n    document: &Document,\n\
    \    mut code: CheckedPackageRefusalCode,\n    original: &[Step<'_>],\n    meter:\
    \ &mut super::WorkMeter,\n) -> ValidationFailure {\n    use CheckedPackageRefusalCode::{MalformedWire,\
    \ UnknownMember};\n    const EXTRA: usize = 2;\n    let mut steps = [Step::Index(0);\
    \ super::document_de::TRACKED_DEPTH + EXTRA];\n    steps[..original.len()].copy_from_slice(original);\n\
    \    let mut length = original.len();\n    let index = original\n        .iter()\n\
    \        .filter_map(|step| match step {\n            Step::Index(index) => Some(*index),"
- id: crates/quire-contract-model/src/checked_package/v2/intake.rs:459
  path: crates/quire-contract-model/src/checked_package/v2/intake.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\nfn invalid_clock_field(document: &Document,\
    \ at: &[Step<'_>]) -> Option<&'static str> {\n    if !matches!(at.first(), Some(Step::Key(\"\
    temporal_bindings\")))\n        || !matches!(at.last(), Some(Step::Key(\"clock\"\
    )))\n        || at.len() < 2\n    {\n        return None;\n    }\n    let clock\
    \ = term_at_steps(document, at)?;\n    let field = match clock.get(\"kind\")?.as_str()?\
    \ {\n        \"event_position\" => \"sequence_authority\",\n        \"timestamped_event\"\
    \ => \"timestamp_unit\",\n        \"fixed_sample\" => \"unit\",\n        _ =>\
    \ return None,\n    };\n    clock\n        .get(field)\n        "
- id: crates/quire-contract-model/src/checked_package/v2/intake.rs:482
  path: crates/quire-contract-model/src/checked_package/v2/intake.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\nfn invalid_node_id_suffix(\n    document:\
    \ &Document,\n    at: &[Step<'_>],\n) -> Option<(Step<'static>, Option<Step<'static>>)>\
    \ {\n    const MAX_SAFE: u64 = 9_007_199_254_740_991;\n    let id = term_at_steps(document,\
    \ at)?;\n    if id.get(\"package\").is_none() || id.get(\"path\").is_none() {\n\
    \        return None;\n    }\n    if id\n        .get(\"ordinal\")\n        .is_some_and(|ordinal|\
    \ ordinal.as_u64().is_none_or(|value| value > MAX_SAFE))\n    {\n        return\
    \ Some((Step::Key(\"ordinal\"), None));\n    }\n    for (index, segment) in id.get(\"\
    path\")?.items()?.enumerate() {\n        if "
- id: crates/quire-contract-model/src/checked_package/v2/intake.rs:551
  path: crates/quire-contract-model/src/checked_package/v2/intake.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\npub(super) fn check_lossless_source(\n  \
    \  document: &Document,\n    wire: &RawCheckedPackageWireV2,\n    budget: &mut\
    \ NativeJoinBudget,\n) -> Result<(), ValidationFailure> {\n    let mut meter =\
    \ super::WorkMeter::over(budget);\n    let meter = &mut meter;\n    let root =\
    \ TermNode::from_ref(document.root());\n    let root_at = super::stage_storage::Path::root();\n\
    \    compare_root(\n        root,\n        \"capability_report\",\n        &wire.capability_report,\n\
    \        &[],\n        meter,\n    )?;\n    compare_root(root, \"contract_version\"\
    , &wire.contract_version, &[], meter)?;\n    c"
- id: crates/quire-contract-model/src/checked_package/v2/lock_tables.rs:279
  path: crates/quire-contract-model/src/checked_package/v2/lock_tables.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\nfn unit_name<B>(node: &super::CheckedSemanticNodeV2<B>)\
    \ -> Option<UnitName<'_>> {\n    if CheckedNodeTag::from_wire(&node.node_tag)\n\
    \        .and_then(|tag| CheckedNodeKind::decode(tag, &node.semantic_form))\n\
    \        != Some(CheckedNodeKind::ScalarType(ScalarTypeForm::Unit))\n    {\n \
    \       return None;\n    }\n    match (&node.owner, &node.declaration) {\n  \
    \      (Some(CheckedNodeOwner::Source { .. }), Some(declaration)) => {\n     \
    \       Some(UnitName::Source(&declaration.qualified_name))\n        }\n     \
    \   (\n            Some(CheckedNodeOwner::Model {\n                identi"
- id: crates/quire-contract-model/src/checked_package/v2/mod.rs:3092
  path: crates/quire-contract-model/src/checked_package/v2/mod.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\nfn declaration_name<'a>(\n    node: &'a RawSemanticNodeV2,\n\
    \    kind: Option<&CheckedNodeKind>,\n) -> Option<(u8, DeclarationName<'a>)> {\n\
    \    if matches!(kind, Some(CheckedNodeKind::Protocol(_))) {\n        return None;\n\
    \    }\n    let namespace = match (node.node_tag.as_ref(), node.semantic_form.as_ref())\
    \ {\n        (\"scalar_type\", \"dimension\") => 0,\n        (\"scalar_type\"\
    , \"unit\") => 1,\n        _ => 2,\n    };\n    let name = match (&node.declaration,\
    \ &node.owner, namespace) {\n        (Some(declaration), _, _) => DeclarationName::Source(&declaration.qualified_name),\n\
    \      "
- id: crates/quire-contract-model/src/checked_package/v2/model_members.rs:331
  path: crates/quire-contract-model/src/checked_package/v2/model_members.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\n    fn admit(name: &str) -> Option<Admitted<'_>>\
    \ {\n        if let Some(inner) = name.strip_prefix(\"Map<\").and_then(|s| s.strip_suffix('>'))\
    \ {\n            let (key, value) = inner.split_once(\", \")?;\n            if\
    \ key.is_empty() || value.is_empty() || value.contains(',') {\n              \
    \  return None;\n            }\n            return Some(Admitted::Map(key, value));\n\
    \        }\n        if let Some(unit) = name.strip_prefix(\"ix://quire/unit/\"\
    ) {\n            return (!unit.is_empty()).then_some(Admitted::Unit(name));\n\
    \        }\n        if let Some(native) = name.strip_pref"
- id: crates/quire-contract-model/src/checked_package/v2/model_members.rs:354
  path: crates/quire-contract-model/src/checked_package/v2/model_members.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\n    fn is_unit_name(name: &str) -> bool {\n\
    \        name.strip_prefix(\"Map<\")\n            .and_then(|s| s.strip_suffix('>'))\n\
    \            .is_none()\n            && name\n                .strip_prefix(\"\
    ix://quire/unit/\")\n                .is_some_and(|unit| !unit.is_empty())\n \
    \   }\n}\n\n/// Whether `text` is the canonical decimal spelling of `number`.\n\
    fn decimal_text_is(number: i64, text: &str) -> bool {\n    use std::fmt::Write\
    \ as _;\n    struct Compare<'t>(&'t str);\n    impl std::fmt::Write for Compare<'_>\
    \ {\n        fn write_str(&mut self, part: &str) -> std::fmt::Result {\n "
- id: crates/quire-contract-model/src/checked_package/v2/model_members.rs:5556
  path: crates/quire-contract-model/src/checked_package/v2/model_members.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\nfn valid_relationship_identity(package: &str,\
    \ identity: &str) -> bool {\n    identity\n        .strip_prefix(\"ix://\")\n\
    \        .and_then(|rest| rest.strip_prefix(package))\n        .and_then(|rest|\
    \ rest.strip_prefix(\"/relationship/\"))\n        .is_some_and(|name| {\n    \
    \        !name.is_empty() && !name.contains('/') && valid_semantic_identity(identity)\n\
    \        })\n}\n\n/// One end of a relationship as the document spells it.\nstruct\
    \ RelationshipEndView<'v> {\n    role: Option<&'v str>,\n    type_ref: &'v str,\n\
    \    multiplicity: Multiplicity,\n}\n\nfn relationship_end(value: View<'"
- id: crates/quire-contract-model/src/checked_package/v2/model_members/document.rs:280
  path: crates/quire-contract-model/src/checked_package/v2/model_members/document.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\n    fn of(text: &'t str) -> Self {\n    \
    \    let (negative, unsigned) = match text.strip_prefix('-') {\n            Some(unsigned)\
    \ => (true, unsigned),\n            None => (false, text),\n        };\n     \
    \   let (mantissa, exponent) = unsigned.split_once(['e', 'E']).unwrap_or((unsigned,\
    \ \"0\"));\n        let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa,\
    \ \"\"));\n        // The digits are `whole` then `fraction`, without leading\
    \ zeros.\n        let (head, tail) = match whole.trim_start_matches('0') {\n \
    \           \"\" => (\"\", fraction.trim_start_matches('0')),\n "
- id: crates/quire-contract-model/src/checked_package/v2/node_index.rs:1105
  path: crates/quire-contract-model/src/checked_package/v2/node_index.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\n    fn enter(\n        &mut self,\n     \
    \   value: NodeRef<'d>,\n        walk: Walk,\n    ) -> Result<Option<Frame<'d>>,\
    \ ValidationFailure> {\n        match value.node() {\n            Node::Object(members)\
    \ => {\n                self.check_dependency_reference(&members)?;\n        \
    \        self.record_segments(&members, walk)?;\n                // FR-375's external\
    \ key is scoped by its outer package,\n                // regardless of the package\
    \ named inside its node ID.\n                let is_external_key = members.len()\
    \ == 2\n                    && members.clone().any(|(name, "
- id: crates/quire-contract-model/src/checked_package/v2/node_index.rs:1146
  path: crates/quire-contract-model/src/checked_package/v2/node_index.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\n    fn check_dependency_reference(\n    \
    \    &mut self,\n        members: &Members<'d>,\n    ) -> Result<(), ValidationFailure>\
    \ {\n        let is_reference = members.clone().any(|(name, value)| {\n      \
    \      name == \"term\" && matches!(value.node(), Node::String(\"dependency_reference\"\
    ))\n        });\n        if !is_reference {\n            return Ok(());\n    \
    \    }\n        let Some((_, node)) = members.clone().find(|(name, _)| *name ==\
    \ \"node\") else {\n            return Ok(());\n        };\n        let Node::Object(node_members)\
    \ = node.node() else {\n            return Err(malfor"
- id: crates/quire-contract-model/src/checked_package/v2/node_index.rs:1178
  path: crates/quire-contract-model/src/checked_package/v2/node_index.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\n    fn record_segments(\n        &mut self,\n\
    \        members: &Members<'d>,\n        walk: Walk,\n    ) -> Result<(), ValidationFailure>\
    \ {\n        if members.len() != 3\n            || ![\"package\", \"path\", \"\
    ordinal\"]\n                .iter()\n                .all(|wanted| members.clone().any(|(name,\
    \ _)| name == *wanted))\n        {\n            return Ok(());\n        }\n  \
    \      let Some((_, path)) = members.clone().find(|(name, _)| *name == \"path\"\
    ) else {\n            return Ok(());\n        };\n        let Node::Array(segments)\
    \ = path.node() else {\n            return Err(malfor"
- id: crates/quire-contract-model/src/checked_package/v2/operations.rs:126
  path: crates/quire-contract-model/src/checked_package/v2/operations.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\n    fn member_key(&self, field: &str) ->\
    \ Option<&CheckedNodeId> {\n        match field {\n            \"declaration\"\
    \ => self.member_ids.declaration.as_ref(),\n            \"context\" => self.member_ids.context.as_ref(),\n\
    \            \"destination_type\" => self.member_ids.destination_type.as_ref(),\n\
    \            \"population\" => self.member_ids.population.as_ref(),\n        \
    \    _ => None,\n        }\n    }\n\n    fn copy_laws(\n        laws: &[OperationLawWire],\n\
    \        meter: &mut WorkMeter,\n    ) -> Result<Vec<OperationLawWire>, ValidationFailure>\
    \ {\n        let mut copied = meter.v"
- id: crates/quire-contract-model/src/checked_package/v2/protocol_body.rs:898
  path: crates/quire-contract-model/src/checked_package/v2/protocol_body.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\nfn validate_envelope<'a>(\n    form: ProtocolForm,\n\
    \    body: &'a CheckedTerm,\n    semantic_type: &CheckedNodeId,\n    selected_profile:\
    \ Option<&CheckedArtifactRef>,\n) -> Result<(&'a str, &'a [CheckedTerm]), ProtocolBodyDefect>\
    \ {\n    use ProtocolBodyDefect as D;\n    let CheckedTerm::Application {\n  \
    \      operator: ApplicationOperator::ProtocolControl,\n        operation,\n \
    \       result_type,\n        arguments,\n    } = body\n    else {\n        return\
    \ Err(D::OperatorIneligible);\n    };\n    if result_type != semantic_type ||\
    \ operation.identity() != Some(\"quire.op.protocol.c"
- id: crates/quire-contract-model/src/checked_package/v2/schedulability.rs:528
  path: crates/quire-contract-model/src/checked_package/v2/schedulability.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\npub(super) fn rational_parts_are_canonical(\n\
    \    numerator: &str,\n    denominator: &str,\n    meter: &mut WorkMeter,\n  \
    \  at: Site<'_>,\n) -> Result<bool, ValidationFailure> {\n    let positive = |text:\
    \ &str| matches!(text.as_bytes(), [first, rest @ ..] if (b'1'..=b'9').contains(first)\
    \ && rest.iter().all(u8::is_ascii_digit));\n    let magnitude = numerator.strip_prefix('-').unwrap_or(numerator);\n\
    \    if !(numerator == \"0\" || positive(magnitude)) || !positive(denominator)\
    \ {\n        return Ok(false);\n    }\n    super::natural::coprime(magnitude,\
    \ denominator, meter, at)\n}\n\nfn "
- id: crates/quire-contract-model/src/checked_package/v2/structural.rs:890
  path: crates/quire-contract-model/src/checked_package/v2/structural.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\npub(super) fn validate_derived_type_shapes(\n\
    \    catalog: &super::operation_catalog::OperationCatalog,\n    nodes: &[CheckedSemanticNodeV2],\n\
    \    kinds: &[CheckedNodeKind],\n    terms: &[Option<CheckedTerm>],\n    index:\
    \ &GraphIndex<'_>,\n    structural_keys: &[std::sync::Arc<CheckedNodeId>],\n \
    \   models: &[super::model_members::DomainModel],\n    meter: &mut WorkMeter,\n\
    ) -> Result<(), ValidationFailure> {\n    let graph = Graph {\n        nodes,\n\
    \        kinds,\n        index,\n        models: Some(models),\n    };\n    for\
    \ (&node_id, &position) in index {\n        let (Some(node)"
- id: crates/quire-contract-model/src/checked_package/v2/structural.rs:1785
  path: crates/quire-contract-model/src/checked_package/v2/structural.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\nfn is_closed_execution_point(point: TermNode<'_>)\
    \ -> bool {\n    let Some(kind) = point.get(\"kind\").and_then(TermNode::as_str)\
    \ else {\n        return false;\n    };\n    let field = match kind {\n      \
    \  \"initialization\" | \"handler\" => \"name\",\n        \"pre\" | \"post\" =>\
    \ \"operation\",\n        _ => return false,\n    };\n    point.exact_members(&[\"\
    kind\", field])\n        && point\n            .get(field)\n            .and_then(TermNode::as_str)\n\
    \            .is_some_and(|name| !name.is_empty())\n}\n\n/// A `state`/`operation_anchor`\
    \ body (QSpec FR-342), borrowed from its term.\npub(su"
- id: crates/quire-contract-model/src/checked_package/v2/temporal.rs:162
  path: crates/quire-contract-model/src/checked_package/v2/temporal.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\n    fn parse_charged(text: &str, meter: &mut\
    \ WorkMeter) -> Result<Option<Self>, ValidationFailure> {\n        let Some((negative,\
    \ digits)) = Self::parts(text) else {\n            return Ok(None);\n        };\n\
    \        Ok(Some(Self {\n            negative,\n            zero: digits == \"\
    0\",\n            digits: meter.boxed_str(digits)?,\n        }))\n    }\n\n  \
    \  /// The magnitude's decimal digits.\n    fn digits(&self) -> &str {\n     \
    \   &self.digits\n    }\n\n    /// Whether this is `0`.\n    fn is_zero(&self)\
    \ -> bool {\n        self.zero\n    }\n}\n\n/// Compare borrowed, canonical signe"
- id: crates/quire-contract-model/src/checked_package/v2/temporal.rs:334
  path: crates/quire-contract-model/src/checked_package/v2/temporal.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\npub(super) fn interval_bound_outside_pattern_node(operation:\
    \ TermNode<'_>) -> Option<&'static str> {\n    let member = operation.get(\"member\"\
    )?;\n    if OperationMemberKind::from_wire(member.get(\"kind\")?.as_str()?)\n\
    \        != Some(OperationMemberKind::TemporalInterval)\n    {\n        return\
    \ None;\n    }\n    let interval = member.get(\"interval\")?;\n    if !interval.is_object()\
    \ {\n        return None;\n    }\n    let timed = interval.exact_members(&[\"\
    lower\", \"upper\", \"lower_end\", \"upper_end\"]);\n    let integer = interval.exact_members(&[\"\
    lower\", \"upper\"]);\n    [\"lower\", \"uppe"
- id: crates/quire-contract-model/src/checked_package/v2/temporal.rs:510
  path: crates/quire-contract-model/src/checked_package/v2/temporal.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\n    fn integer(value: Option<TermNode<'_>>)\
    \ -> bool {\n        value.and_then(TermNode::as_str).is_some_and(|text| {\n \
    \           let digits = text.strip_prefix('-').unwrap_or(text);\n           \
    \ match digits.as_bytes() {\n                [b'0'] => text == \"0\",\n      \
    \          [first, rest @ ..] => {\n                    (b'1'..=b'9').contains(first)\
    \ && rest.iter().all(u8::is_ascii_digit)\n                }\n                []\
    \ => false,\n            }\n        })\n    }\n    #[ix_trace_rs::string_edge]\n\
    \    fn rational(value: TermNode<'_>) -> bool {\n        value.exact_members("
- id: crates/quire-contract-model/src/checked_package/v2/temporal.rs:523
  path: crates/quire-contract-model/src/checked_package/v2/temporal.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\n    fn rational(value: TermNode<'_>) -> bool\
    \ {\n        value.exact_members(&[\"numerator\", \"denominator\"])\n        \
    \    && integer(value.get(\"numerator\"))\n            && integer(value.get(\"\
    denominator\"))\n            && value\n                .get(\"numerator\")\n \
    \               .and_then(TermNode::as_str)\n                .is_some_and(|text|\
    \ !text.starts_with('-'))\n            && value\n                .get(\"denominator\"\
    )\n                .and_then(TermNode::as_str)\n                .is_some_and(|text|\
    \ text != \"0\" && !text.starts_with('-'))\n    }\n    match kind {\n        Oper"
- id: crates/quire-contract-model/src/checked_package/v2/temporal.rs:1635
  path: crates/quire-contract-model/src/checked_package/v2/temporal.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\nfn is_clock_unit_path(path: Option<&[CheckedNodePathSegment]>,\
    \ unit: Option<&str>) -> bool {\n    use CheckedNodePathSegment::Name;\n    match\
    \ (path, unit) {\n        (None, None) => true,\n        (Some([Name(kind), Name(form),\
    \ Name(name)]), Some(unit)) => {\n            kind.as_ref() == \"scalar_type\"\
    \ && form.as_ref() == \"unit\" && name.as_ref() == unit\n        }\n        _\
    \ => false,\n    }\n}\n\n#[ix_trace_rs::string_edge]\nfn check_aggregate(\n  \
    \  formula: usize,\n    profile: Option<TemporalProfile>,\n    clock_unit: Option<&str>,\n\
    \    graph: &StepGraph<'_, '_>,\n    terms: &[Op"
- id: crates/quire-contract-model/src/checked_package/v2/temporal_binding.rs:288
  path: crates/quire-contract-model/src/checked_package/v2/temporal_binding.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\nfn is_operation_leaf(leaf: TermNode<'_>)\
    \ -> bool {\n    let Some(members) = leaf.members() else {\n        return false;\n\
    \    };\n    let (mut path, mut laws) = (false, false);\n    for (name, value)\
    \ in members {\n        let fine = match name {\n            \"path\" => {\n \
    \               path = true;\n                value\n                    .items()\n\
    \                    .is_some_and(|mut items| items.all(|item| item.as_str().is_some()))\n\
    \            }\n            \"laws\" => {\n                laws = true;\n    \
    \            value\n                    .items()\n                    .i"
- id: crates/quire-contract-model/src/checked_package/v2/term_view.rs:520
  path: crates/quire-contract-model/src/checked_package/v2/term_view.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\n    fn segment_form(self) -> Option<SegmentForm<'d>>\
    \ {\n        if let Some(name) = self.as_str() {\n            return Some(SegmentForm::Name(name));\n\
    \        }\n        if self.exact_members(&[\"package\", \"node\"]) {\n      \
    \      return Some(SegmentForm::Dependency(\n                self.get(\"package\"\
    )?.as_str()?,\n                self.get(\"node\")?.as_u64()?,\n            ));\n\
    \        }\n        let mut members = self.members()?;\n        let ((name, value),\
    \ None) = (members.next()?, members.next()) else {\n            return None;\n\
    \        };\n        let index = value.as_u64()?"
- id: crates/quire-contract-model/src/checked_package/v2/term_view.rs:1071
  path: crates/quire-contract-model/src/checked_package/v2/term_view.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\nfn decode_term<'d>(\n    context: &DecodeContext<'d>,\n\
    \    document: &'d Arc<Document>,\n    term: TermNode<'d>,\n) -> Result<CheckedTerm,\
    \ TermConversionCause> {\n    if let Some(kind @ (\"implementation_guarantee\"\
    \ | \"component_composition\")) =\n        term.get(\"term\").and_then(TermNode::as_str)\n\
    \    {\n        let shape = if kind == \"implementation_guarantee\" {\n      \
    \      FlatShape::Guarantee\n        } else {\n            FlatShape::Composition\n\
    \        };\n        if !flat_shape(term, shape) {\n            return Err(TermConversionCause::MalformedMember\
    \ {\n                mem"
- id: crates/quire-contract-model/src/checked_package/v2/timed_model.rs:388
  path: crates/quire-contract-model/src/checked_package/v2/timed_model.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\nfn check_wire_shape(\n    node: &CheckedSemanticNodeV2,\n\
    \    form: TimedModelForm,\n    position: usize,\n    budget: &mut NativeJoinBudget,\n\
    \    failure_at: &mut Path,\n) -> Result<(), WireFailure> {\n    let site = Site::new(super::PackageReadStage::WireChecks,\
    \ position, \"timed_model\");\n    let mut at = node_path(position);\n    at.stage\
    \ = super::PackageReadStage::WireChecks;\n    if node.recursion_group.is_some()\
    \ {\n        return Err(malformed(at.key(\"recursion_group\"), failure_at));\n\
    \    }\n    let root = form == TimedModelForm::Model;\n    if root {\n       \
    \ if node.owner.i"
- id: crates/quire-contract-model/src/checked_package/v2/typed_terms.rs:129
  path: crates/quire-contract-model/src/checked_package/v2/typed_terms.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\n    fn child(self, name: &str, node: quire_canonical::NodeRef<'a>)\
    \ -> Self {\n        Self {\n            node,\n            ids: self.ids,\n \
    \           remapped: self.remapped,\n            scope: if matches!(self.scope,\
    \ MemberScope::Root) && name == \"target\" {\n                MemberScope::Target\n\
    \            } else {\n                MemberScope::Raw\n            },\n    \
    \    }\n    }\n}\nimpl Serialize for MemberProjection<'_> {\n    fn serialize<S:\
    \ Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {\n        if let\
    \ Some(remapped) = self.remapped {\n            if let"
- id: crates/quire-contract-model/src/checked_package/v2/typed_terms.rs:396
  path: crates/quire-contract-model/src/checked_package/v2/typed_terms.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\n    pub(super) fn member_id(&self, field:\
    \ &str) -> Option<&CheckedNodeId> {\n        match field {\n            \"declaration\"\
    \ => self.member_ids.declaration.as_ref(),\n            \"context\" => self.member_ids.context.as_ref(),\n\
    \            \"destination_type\" => self.member_ids.destination_type.as_ref(),\n\
    \            \"population\" => self.member_ids.population.as_ref(),\n        \
    \    _ => None,\n        }\n    }\n}\n\n#[derive(Clone, Debug, Eq, PartialEq,\
    \ Serialize, Deserialize, FixedShape)]\n#[serde(deny_unknown_fields)]\npub(super)\
    \ struct CheckedOperationLawWire {\n    pub(super) "
- id: crates/quire-contract-model/src/checked_package/v2/typed_terms.rs:783
  path: crates/quire-contract-model/src/checked_package/v2/typed_terms.rs
  role: examined
  excerpt: "#[ix_trace_rs::string_edge]\n    pub(super) fn from_validated_node(\n\
    \        value: super::TermNode<'_>,\n        ids: &CheckedOperationMemberIds,\n\
    \        meter: &mut super::WorkMeter,\n    ) -> Result<Option<Self>, crate::checked_package::common::ValidationFailure>\
    \ {\n        use super::TermNode;\n        macro_rules! need {\n            ($value:expr)\
    \ => {\n                match $value {\n                    Some(value) => value,\n\
    \                    None => return Ok(None),\n                }\n           \
    \ };\n        }\n        let kind = need!(value.get(\"kind\").and_then(TermNode::as_str));\n\
    \        Ok(Som"
```

## Coverage

Plan completion: not assessed. Optional semantic gap review skipped. Full-repository computed matrix inspected using Quire 0.38.1; no execution evidence was read. Reverse mapping of changed existing behavior remains FR-038 strict V2 consumption; marker-only changes add no behavior requiring a new test binding. No test binding changed. No source or test stubs introduced.

## Verification and Limits

Reviewed exact frozen PR 970 source. git diff --check passed. Supplied scanner logs report 102 before and zero after; scoped Rust 1.98.1 rustfmt check log is clean. No compilation/test/aggregate repetition for marker-only changes; preceding source workspace compile evidence is inherited, not rerun. Existing aggregate, Clippy, coverage and integration failures remain separate baseline work. Deferred AC-511 whole-read physical proof, AC-513 static closure and F1 body migration are not claimed complete.
