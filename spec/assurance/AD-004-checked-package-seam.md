---
id: AD-004
title: "QSpec to IR seam: the quire.checked-package/v2 reader and lowering"
type: ArchitectureDescription
status: proposed
owner: kreneskyp
system: quire-contract-model checked_package module (reader, lowerer, evidence), its inputs from QSpec's wire contract and QSL's emission, and its output to codegen
relationships:
  - target: ix://agent-ix/quire-contract-ir/AD-001
    type: references
  - target: ix://agent-ix/quire-contract-ir/AD-003
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-040
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-344
    type: references
  - target: ix://agent-ix/quire-contract-ir/FR-035
    type: references
  - target: ix://agent-ix/quire-specification/FR-322
    type: references
  - target: ix://agent-ix/quire-specification/FR-340
    type: references
---
# QSpec to IR seam: the checked-package reader and lowering

The `checked_package` row of the Subsystem Registry lists this AD beside AD-001 and AD-003.
AD-001 gives the reader one paragraph and the versioned-contracts table one row; this AD states
the seam in full and does not restate AD-001. It is one of the seam descriptions of IR-324.

## System Boundary

IR is the strict reader of a wire it does not own. QSpec owns `quire.checked-package/v2`; QSL
produces packages; IR admits or refuses bytes and lowers admitted nodes for backends. This AD
states what the reader takes, what it asserts about identity, what it refuses, who answers when
the producer and the contract disagree, and which disagreements are open.

Out of scope: how QSL emits, what QSpec's text should say (routed), and the Kani profile.

## Views

The seam is described as what crosses it, how identity is asserted, which way dependencies point and who reports each failure.

### What crosses the seam

| Item | Direction | Owner | Notes |
| --- | --- | --- | --- |
| `quire.checked-package/v2` bytes: `contract_version`, `package_id`, `identity_preimage`, `lock`, `semantic_graph`, `source_map`, `diagnostics` | QSL (producer) to IR | QSpec FR-322 and FR-340 to FR-342 define the wire | IR holds no copy of QSpec files; it states the shape it admits in its own reader (`crates/quire-contract-model/src/checked_package/`) |
| `CheckedPackageEvidence`: selected domain package documents (supplied under their `sha256-jcs` digest), dependency packages (already admitted), supported features | caller to IR | IR defines the type | only `lock.sources` rows carry a raw-source byte digest (`quire.source.bytes/v1`), which is not checked against evidence (the `sha256-jcs` digest of a `model_selections` row and the `package_id` of a dependency row are); definition references (`lock` selections, `diagnostics.catalog`, operation laws) are `{authority, identity}` with no revision or digest (FR-038 "Artifact references"); `package_id` is the content identity (`evidence.rs` doc) |
| `quire.checked-operation-catalog/v1`: the closed catalog every V2 `application` term's `operation` is validated against | `quire-verification-contracts` (owner) to IR | `quire-verification-contracts` | IR depends on that crate and owns only the reader; it holds no copy of the catalog (`v2/operation_catalog.rs`) |
| `CheckedPackageReadLimits` (`bounded()` default: 1048576 bytes, 10000 nodes, 100000 edges, 100000 occurrences, 10000 diagnostics, 1000000 validation visits; 🚧 planned: today the default also holds 128 depth, and the depth member and `CheckedPackageLimit::Depth` go with FR-038-AC-117, a breaking change for a consumer across this seam that builds either) | caller to IR | IR | every member finite |
| `CheckedPackageDispatchResult`: `AdmittedV2` / `Refused` / `Incomplete` | IR to caller | IR | typed code and RFC 6901 pointer on refusal |
| `CheckedPackageV2` and its accessors | IR to codegen and QSL | IR | only the reader can build one |
| Lowering records (seven kinds) and `ContractPackage` (`quire.contract-ir.contract-package/v1`) | IR to backends | IR (FR-035) | one record per requested node |

### Versioned contracts and identity assertion

- `contract_version` is read exactly once, before any version-specific decode (`dispatch.rs`:
  `read_checked_package` and `dispatch_value`). The strict parse, the duplicate-member check and
  the canonical-bytes check run first, over the whole document (`common.rs`, `read_value`). It admits `quire.checked-package/v2` only; every
  other version refuses `unknown_contract_version` carrying the version read, and a missing or
  non-string `contract_version` refuses `malformed_wire`. This is a refusal control: IR never
  relabels or widens the input to fit.
- Content identity. The reader recomputes `package_id` as the SHA-256 of the RFC 8785 bytes of
  `identity_preimage` under `quire.package.semantic/v2`, requires the preimage's lock members
  to equal the lock and its projection to equal the graph's occurrence-free projection (each
  mismatch refuses `stale_dependency`), and re-derives every nominal node identity (a violation
  refuses `invalid_semantic_graph`). Every one of these identity digests is `quire-canonical`'s
  (FR-038, "Every identity digest is computed through quire-canonical"); a selected model
  document holding a number past 2^53 is refused `noncanonical_wire` (IR's own rule) with a pointer into
  that document, a different refusal from the pointer-free ones about the package's own
  bytes. `package_id` is the package's content identity: it binds a
  package to its content and is the value QSL's replay recomputes later. The wire carries other
  digests (application node keys, nominal identity digests, dependency `package_id`s, the lock's
  raw-source digests; a definition reference carries none) and domain package documents carry `sha256-jcs` digests; these are
  QSpec-owned identities of the package's parts, and IR adds none. A domain package document is
  checked by recomputing its `sha256-jcs` digest from the supplied bytes, never by trusting the
  digest it is supplied under.
- There is no commit id, tool pin or version-tracking record on this seam, and IR checks none.

### Dependency direction

IR depends on nothing of QSpec's code or QSL. QSL depends on the `quire-contract-model` package
(under the dependency name `quire-contract-ir`) and calls `read_checked_package` through it. CG
reaches the reader through IR's root crate, which re-exports the model (see Current state); the
driver reads through the model crate. The direction is QSpec (text) to IR (reader) to QSL, CG,
driver (consumers); no arrow points back to IR. IR's one other inbound edge is the operation
catalog: the reader depends on `quire-verification-contracts` for `quire.checked-operation-catalog/v1`
and does not copy it.

### Failure outcomes and who reports them

| Condition | Reported by | Outcome |
| --- | --- | --- |
| Another version | IR | `unknown_contract_version`, pointer `/contract_version` |
| Missing or non-string `contract_version`, or a document that is not an object | IR | `malformed_wire`, pointer at the member or the root |
| Malformed JSON or non-canonical bytes | IR | `malformed_wire`, `noncanonical_wire`; no pointer (a refusal about the byte stream) |
| Duplicate member, unknown member | IR | `duplicate_member`, `unknown_member`, each at the pointer of the member |
| A selected domain package document or dependency package not supplied | IR | `missing_import` |
| `package_id`, identity preimage, projection, lock or a selected document not matching the content identity it names | IR | `stale_dependency`, at the pointer of the value at fault |
| Bad nominal node identity, unresolved reference, cycle outside one recursion group, other graph-form defects | IR | `invalid_semantic_graph` and the other codes FR-322 fixes, at the pointer of the value at fault |
| Frame, anchor or state-clause body defect | IR | the cause tag FR-322's pairing fixes, with the node key |
| A limit reached | IR | `Incomplete` with the limit, the consumed amount and, except for the byte budget, the pointer of the value whose charge failed |
| Item cannot lower | IR | one of seven record kinds (`lowered`, `unsupported`, `requires_bound`, `invalid_input`, `failed`, `invalid_body`, `body_incomplete`); the last two are never produced for an admitted package |
| QSL emits something IR refuses | IR reports; QSL or QSpec fixes the producer or the text | see precedence below |
| QSpec text and QSpec's reference reader (its checked-package test harness) disagree | QSpec | IR follows the reference reader as its own policy and records a known deviation until QSpec rules |

IR never reports a producer's defect as its own and never admits a package to make a producer
pass.

## Decisions

Two decisions govern the seam: who wins when the producer, the text and the reader disagree, and which statements a test can check.

### Precedence when producer, text and reader disagree

1. QSpec's published text and schema decide what the wire is.
2. Where the text is silent, IR's policy is to match QSpec's reference reader, QSpec's own
   checked-package test harness. This is IR's choice, not a ruling on QSpec's authority; QSpec
   rules and IR follows (IR-483 matches it for cyclic compared types).
3. QSL's emission is evidence of a defect in 1 or 2, or in QSL. IR does not widen the reader to
   fit an emission. The disagreement is filed with QSpec to rule (as STD-129 does for recursive
   compared types).

CG reads what QSL emits and does not hand-build its own shape (IR-324, comment of 2026-09-30,
untrusted ticket text citing IR-480). IR is the admitting gate in between and it follows QSpec,
not the emission.

### Invariants a test can check

Candidate statements (local labels; the repo assigns requirement ids when one is authored). Each
is marked current (the repo already states and backs it), stated (a requirement states it but
no backing test was measured here) or gap (not true today).

- Q-1. Every byte string yields exactly one of admitted, refused or incomplete; never a panic
  (property test over mutated packages). Gap: IR has no property-test or fuzz dependency, so no
  such test exists.
- Q-2. A byte string whose `contract_version` is not `quire.checked-package/v2` refuses
  `unknown_contract_version` before any version-specific decode. Current: FR-038 Description.
- Q-3. The recomputed `package_id` equals the wire's for every admitted package; changing any
  byte of the preimage changes it or refuses. Current: FR-038-AC-4.
- Q-4. A domain package document supplied under another document's digest is refused. Stated:
  `CheckedPackageEvidence` doc; backing test not measured.
- Q-5. Every refusal carries a pointer that resolves in the given document, except a refusal of
  the byte stream. Stated: FR-038 pointer rules; backing test not measured.
- Q-6. Every closed wire vocabulary is decoded into an enum once at intake and every later
  decision matches it exhaustively with no catch-all arm (compile-time). Gap, unmeasured: bodies
  are matched over `serde_json::Value` with `_ =>` arms (for example `v2/frame.rs`), so the
  statement does not hold as worded.
- Q-7. IR defines no replay or witness type (FR-037-AC-6, planned) and the model crate depends on
  no QSL crate (FR-028). FR-037-AC-6 covers neither a terminal type nor a dependency rule.
- Q-8. Each of the five ADR-002 members FR-344 names as unadmitted (supertype list, abstractness
  flag, subsets edge, redefines edge, population node) refuses with a typed code. Current:
  FR-344, TC-222. The other two members are carried and admitted.
- Q-9. A package QSL emits for each node family and semantic form QSL can produce is admitted
  by the reader at the emitted `package_id` (cross-repo, runs in QSL; see Routed gaps). Gap.

## Risks

What is measured today, what is open and with whom, and what is routed.

### Current state and gaps (measured at the commit this AD was written against)

- Reader: `dispatch.rs` (version dispatch), `v2/` (identity, operations, frame, state,
  structural, lowering), `evidence.rs`. FR-038 declares ACs 1 to 15, 17 to 33 and 35 to 43; it
  has no AC-16 or AC-34, a numbering hole and not a gap in behaviour.
- Merged QSpec FR-322 settles STD-125: the reader has no depth limit, because the body grammar
  fixes the JSON depth of every package. FR-038 states the flat wire and no depth limit
  (FR-038-AC-114 through FR-038-AC-118); the reader still charges a capped caller depth limit and
  admits nested applications until IR-495's code lands.
- FR-040 states a known deviation: a field entry on a `model`/`record_value_type` node is joined
  and ordered but its name is not resolved, because QSpec's `ModelDeclarationNode` has no such
  form while FR-340 says the name matches a declared field. Pending a QSpec ruling.
- FR-344: of ADR-002's seven members, two are already carried (operation frame, embedded
  meaning vocabulary) and five (supertype list, abstractness flag, subsets edge, redefines edge,
  population node) have no QSpec wire carrier, so the reader refuses them. Blocked on QSpec.
- IR-483 (merged): empty operation `leaves` is admitted for a compared type with no text leaf,
  and a cyclic or unresolved compared type is refused as operator-ineligible, matching
  QSpec's reference reader (its checked-package test harness). That QSL emits `leaves: []` for
  a cyclic type with no text field and a `recursion:<d>` leaf for a cyclic type that reaches
  text, which QSpec has not adopted and the published leaf schema rejects, is STD-129's text
  (untrusted, not re-measured here); IR's refusal was re-measured. Open at QSpec: STD-129.
- Text admission is a codegen term: CG's oracle matrix records text admission as refused today
  (its FR-014 rows). Whether any catalogued `numeric.convert` admits text is an owner decision
  that is pending with the owner. It is recorded as open, not routed to QSpec and not decided
  here; the operation catalog it would touch is owned by `quire-verification-contracts`.
- IR's own tests build packages in-repo from IR's own vocabulary
  (`tests/it/support/checked_package.rs`; FR-038 matrix row). That cannot see a shape QSL emits
  and IR's builder does not. The IR-reader test over a real QSL emission lives in QSL
  (`tests/it/config_version_spine.rs`, `tc_469_step_6_the_emitted_package_admits_via_i04`), for
  one spine fixture. IR cannot hold a QSL-emitted fixture without copying QSL's output into IR,
  so the cross-repo corpus belongs in QSL, which already depends on the model crate. That
  corpus is tracked as QSL-353 (as relayed, untrusted). The cross-repo test in quire-integration
  that the relayed review text mentions concerns the replay seam, not this one, and does not
  apply here.
- QSpec ids: two QSpec documents carry `id: FR-341`: "checked-package state clause body" and
  "infinite-trace result disposition". IR FR-040 disambiguates by title.
- IR's root crate has `pub use quire_contract_model::*` (`src/lib.rs:12`) while AD-001 says the
  root re-exports nothing from the model; CG imports `CheckedNodeId` and `CheckedPackageV2`
  through that glob. The root crate also still exports `KaniProviderResult` and
  `KaniProviderRecord` (`src/kani/mod.rs:23`) although FR-039 lists them as not part of its
  interface, and the kani outcome code strings cross as free strings. IR-347 (reopened) already
  carries the glob removal, the `KaniProvider*` removal and the free-string cause codes. This AD
  files nothing new for them and only notes that any consumer of this reader meets the glob.

### Open questions

| Question | Owner | Recommendation | Cost of the alternative |
| --- | --- | --- | --- |
| Recursive compared type: refuse (matches the reference reader, which IR follows) or 0 leaves for no-text cyclic types; adopt `recursion:<d>` leaves | QSpec (STD-129) | IR does not change before the ruling | IR widening first would diverge from the reference reader |
| `record_value_type` field names | QSpec | rule that a field entry on that form names no declared field, or add the form to `ModelDeclarationNode` | IR keeps a documented deviation |
| Text admission for `numeric.convert` | the owner (decision pending; not routed) | decide, then state it where the operation catalog and FR-322 can carry it | consumer paths with no admitted input |
| Five ADR-002 members have no wire carrier | QSpec | publish carriers or state they are out of v2 | reader refuses forever |
| Duplicate `FR-341` ids in QSpec | QSpec | renumber one | citations by id are ambiguous |

No compatibility layer is proposed. IR admits one version and refuses the rest.

### Routed gaps

Needs stated to owners, not decisions. Ids are the routing ids of IR-324; they are not
requirement ids.

To QSL (QSL reviews these rows):

| Id | Stated need |
| --- | --- |
| R-Q6 | An emission-to-admission test in QSL over every node family and form QSL can emit, reading through the model crate. IR cannot hold QSL output without copying it. As relayed (untrusted), tracked as QSL-353. |

To QSpec:

| Id | Stated need |
| --- | --- |
| R-S1 | Rule on recursive compared types and `recursion:<d>` leaves (STD-129). |
| R-S8 | Reconcile FR-440 "Reader joins", which decides a nested `case` at the operation step, with FR-322 "Body grammar", whose first-in-document-pre-order rule needs it decided in the same walk as `malformed_wire`. IR reads it at strict wire validation (FR-038-AC-115, FR-038-AC-116). |
| R-S4 | Renumber one of the two `FR-341` documents. |
| R-S5 | Rule on `record_value_type` field entries (FR-340). |
| R-S7 | Publish wire carriers for the five ADR-002 members or state them out of v2. |

The routing ids R-S2 and R-S3 (the FR-331 provider envelope and the obligation-identity digest
domain) belong to the replay and evidence seams and are carried in CG's AD-003, not here. R-S8
is the owner decision on text admission above, not a QSpec routing.

IR-owned items, defined here because other seam ADs cite them:

| Id | Stated need | Where it is tracked |
| --- | --- | --- |
| R-I1 | Delete `KaniProviderResult` and `KaniProviderRecord` and the `provider_result` map from the root crate (`src/kani/outcome.rs`, `src/kani/mod.rs:23`); they duplicate the terminal map CG owns. | IR-347 reopened scope |
| R-I2 | Export cause-code constants or a typed cause enum: the kani cause codes cross as bare strings that consumers re-spell. | Overlaps IR-347's free-string cause codes |
| R-I3 | Remove `pub use quire_contract_model::*` (`src/lib.rs:12`) together with CG adding a direct `quire-contract-model` dependency; CG imports model types through the glob. | IR-347 reopened scope |

No new ticket is filed for them.
