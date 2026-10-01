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
| `CheckedPackageEvidence`: selected domain package documents (supplied under their `sha256-jcs` digest), dependency packages (already admitted), supported features | caller to IR | IR defines the type | the lock's raw-artifact digests are not checked against evidence; `package_id` is the content identity (`evidence.rs` doc) |
| `CheckedPackageReadLimits` (`bounded()` default: 1048576 bytes, 128 depth, 10000 nodes, 100000 edges, 100000 occurrences, 10000 diagnostics, 1000000 validation visits) | caller to IR | IR | every member finite |
| `CheckedPackageDispatchResult`: `AdmittedV2` / `Refused` / `Incomplete` | IR to caller | IR | typed code and RFC 6901 pointer on refusal |
| `CheckedPackageV2` and its accessors | IR to codegen and QSL | IR | only the reader can build one |
| Lowering records (seven kinds) and `ContractPackage` (`quire.contract-ir.contract-package/v1`) | IR to backends | IR (FR-035) | one record per requested node |

### Versioned contracts and identity assertion

- `contract_version` is read exactly once, before any other decode (`dispatch.rs`:
  `read_checked_package` and `dispatch_value`). It admits `quire.checked-package/v2` only; every
  other version refuses `unknown_contract_version` carrying the version read, and a missing or
  non-string `contract_version` refuses `malformed_wire`. This is a refusal control: IR never
  relabels or widens the input to fit.
- Content identity. The reader recomputes `package_id` as the SHA-256 of the RFC 8785 bytes of
  `identity_preimage` under `quire.package.semantic/v2`, requires the preimage's lock members
  to equal the lock and its projection to equal the graph's occurrence-free projection (each
  mismatch refuses `stale_dependency`), and re-derives every nominal node identity (a violation
  refuses `invalid_semantic_graph`). `package_id` is the one canonical content-identity digest on
  this seam: it binds a package to its content and is the value QSL's replay recomputes later.
  A domain package document is checked the same way, by recomputing its `sha256-jcs` digest
  from the supplied bytes, never by trusting the digest it is supplied under.
- There is no commit id, tool pin or version-tracking record on this seam, and IR checks none.

### Dependency direction

IR depends on nothing of QSpec's code or QSL. QSL depends on the `quire-contract-model` package
(under the dependency name `quire-contract-ir`) and calls `read_checked_package` through it. CG
reaches the reader through IR's root crate, which re-exports the model (see Current state); the
driver reads through the model crate. The direction is QSpec (text) to IR (reader) to QSL, CG,
driver (consumers); no arrow points back to IR.

### Failure outcomes and who reports them

| Condition | Reported by | Outcome |
| --- | --- | --- |
| Another version | IR | `unknown_contract_version`, pointer `/contract_version` |
| Missing or non-string `contract_version`, or a document that is not an object | IR | `malformed_wire`, pointer at the member or the root |
| Malformed JSON, non-canonical bytes, duplicate member, unknown member | IR | `malformed_wire`, `noncanonical_wire`, `duplicate_member`, `unknown_member`; no pointer for the byte stream |
| `package_id`, identity preimage, projection, lock or a selected document not matching the content identity it names | IR | `stale_dependency`, at the pointer of the value at fault |
| Bad nominal node identity, unresolved reference, cycle outside one recursion group, other graph-form defects | IR | `invalid_semantic_graph` and the other codes FR-322 fixes, at the pointer of the value at fault |
| Frame, anchor or state-clause body defect | IR | the cause tag FR-322's pairing fixes, with the node key |
| A limit reached | IR | `Incomplete` with the limit, the consumed amount and, except for the byte budget, the pointer of the value whose charge failed |
| Item cannot lower | IR | one of seven record kinds (`lowered`, `unsupported`, `requires_bound`, `invalid_input`, `failed`, `invalid_body`, `body_incomplete`); the last two are never produced for an admitted package |
| QSL emits something IR refuses | IR reports; QSL or QSpec fixes the producer or the text | see precedence below |
| QSpec text and its reference reader disagree | QSpec | IR follows the reference reader and records a known deviation until QSpec rules |

IR never reports a producer's defect as its own and never admits a package to make a producer
pass.

## Decisions

Two decisions govern the seam: who wins when the producer, the text and the reader disagree, and which statements a test can check.

### Precedence when producer, text and reader disagree

1. QSpec's published text and schema decide what the wire is.
2. QSpec's reference reader decides where the text is silent. IR matches it (IR-483 matches
   it for cyclic compared types).
3. QSL's emission is evidence of a defect in 1 or 2, or in QSL. IR does not widen the reader to
   fit an emission. The disagreement is filed with QSpec to rule (as STD-129 does for recursive
   compared types).

CG reads what QSL emits and does not hand-build its own shape (IR-324, comment of 2026-09-30,
untrusted ticket text citing IR-480). IR is the admitting gate in between and it follows QSpec,
not the emission.

### Invariants a test can check

Candidate statements (local labels; the repo assigns requirement ids when one is authored).

- Q-1. Every byte string yields exactly one of admitted, refused or incomplete; never a panic
  (property test over mutated packages).
- Q-2. A byte string whose `contract_version` is not `quire.checked-package/v2` refuses
  `unknown_contract_version` before any version-specific decode.
- Q-3. The recomputed `package_id` equals the wire's for every admitted package; changing any
  byte of the preimage changes it or refuses.
- Q-4. A domain package document supplied under another document's digest is refused.
- Q-5. Every refusal carries a pointer that resolves in the given document, except a refusal of
  the byte stream.
- Q-6. Every closed wire vocabulary is decoded into an enum once at intake and every later
  decision matches it exhaustively with no catch-all arm (compile-time).
- Q-7. IR defines no replay, witness or terminal type and depends on no QSL crate (existing:
  FR-037-AC-6).
- Q-8. Each of the seven ADR-002 members FR-344 names as unadmitted refuses with a typed code
  (existing: FR-344, TC-222).
- Q-9. A package QSL emits for each node family and semantic form QSL can produce is admitted
  by the reader at the emitted `package_id` (cross-repo, runs in QSL; see Routed gaps).

## Risks

What is measured today, what is open and with whom, and what is routed.

### Current state and gaps (measured at the commit this AD was written against)

- Reader: `dispatch.rs` (version dispatch), `v2/` (identity, operations, frame, state,
  structural, lowering), `evidence.rs`. FR-038 declares ACs 1 to 15, 17 to 33 and 35 to 43; it
  has no AC-16 or AC-34, a numbering hole and not a gap in behaviour.
- FR-038 states a known deviation: the depth ceiling is capped at 16,384
  (`CheckedPackageReadLimits::MAXIMUM_DEPTH`) although FR-322 and FR-038-AC-3 charge the caller's
  limit as given, pending STD-125.
- FR-040 states a known deviation: a field entry on a `model`/`record_value_type` node is joined
  and ordered but its name is not resolved, because QSpec's `ModelDeclarationNode` has no such
  form while FR-340 says the name matches a declared field. Pending a QSpec ruling.
- FR-344: of ADR-002's seven members, two are already carried (operation frame, embedded
  meaning vocabulary) and five (supertype list, abstractness flag, subsets edge, redefines edge,
  population node) have no QSpec wire carrier, so the reader refuses them. Blocked on QSpec.
- IR-483 (merged): empty operation `leaves` is admitted for a compared type with no text leaf,
  and a cyclic or unresolved compared type is refused as operator-ineligible, matching the
  QSpec reference reader. That QSL emits `leaves: []` for a cyclic type with no text field and a
  `recursion:<d>` leaf for a cyclic type that reaches text, which QSpec has not adopted and the
  published leaf schema rejects, is STD-129's text (untrusted, not re-measured here); IR's
  refusal was re-measured. Open at QSpec: STD-129.
- Text admission: IR refuses every text-admission node. Any consumer path that needs a
  text-admitted `numeric.convert` is therefore reachable from no admitted package. The owner
  decision on whether any catalogued `convert` admits text is pending; recorded as open and not
  decided here.
- IR's own tests build packages in-repo from IR's own vocabulary
  (`tests/it/support/checked_package.rs`; FR-038 matrix row). That cannot see a shape QSL emits
  and IR's builder does not. The IR-reader test over a real QSL emission lives in QSL
  (`tests/it/config_version_spine.rs`, `tc_469_step_6_the_emitted_package_admits_via_i04`), for
  one spine fixture. IR cannot hold a QSL-emitted fixture without copying QSL's output into IR,
  so the cross-repo corpus belongs in QSL, which already depends on the model crate.
- QSpec ids: on QSpec `origin/main` two documents carry `id: FR-341` (the checked-package state
  clause body under `spec/objects/interfaces/` and the infinite-trace result disposition under
  `spec/objects/temporal/`). IR FR-040 disambiguates by spelling the file path.
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
| Recursive compared type: refuse (matches the reference reader and IR) or 0 leaves for no-text cyclic types; adopt `recursion:<d>` leaves | QSpec (STD-129) | IR does not change before the ruling | IR widening first would diverge from the reference reader |
| Depth ceiling: caller's limit as given, or capped | QSpec (STD-125) | whichever QSpec rules; remove the deviation note then | a stated deviation stays in an FR |
| `record_value_type` field names | QSpec | rule that a field entry on that form names no declared field, or add the form to `ModelDeclarationNode` | IR keeps a documented deviation |
| Text admission for `numeric.convert` | owner decision | state it in FR-322 or the operation catalog | consumer paths with no admitted input |
| Five ADR-002 members have no wire carrier | QSpec | publish carriers or state they are out of v2 | reader refuses forever |
| Duplicate `FR-341` ids in QSpec | QSpec | renumber one | citations by id are ambiguous |

No compatibility layer is proposed. IR admits one version and refuses the rest.

### Routed gaps

Needs stated to owners, not decisions. Ids are the routing ids of IR-324; they are not
requirement ids.

To QSL (QSL reviews these rows):

| Id | Stated need |
| --- | --- |
| R-Q6 | An emission-to-admission test in QSL over every node family and form QSL can emit, reading through the model crate. IR cannot hold QSL output without copying it. |

To QSpec:

| Id | Stated need |
| --- | --- |
| R-S1 | Rule on recursive compared types and `recursion:<d>` leaves (STD-129). |
| R-S2 | FR-331-AC-8 wording of "SUCCESS check": the count is the backend adapter's. |
| R-S3 | Whether the obligation identity needs a digest domain name. |
| R-S4 | Renumber one of the two `FR-341` documents. |
| R-S5 | Rule on `record_value_type` field entries (FR-340). |
| R-S6 | Rule on the depth ceiling (STD-125). |
| R-S7 | Publish wire carriers for the five ADR-002 members or state them out of v2. |
| R-S8 | State which catalogued `numeric.convert` admits text, if any. |

IR-owned items R-I1 and R-I3 are in IR-347's reopened scope and R-I2 overlaps IR-347's
free-string cause codes; no new ticket is filed for them.
