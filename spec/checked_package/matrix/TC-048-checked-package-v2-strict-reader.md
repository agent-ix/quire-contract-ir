---
id: TC-048
title: "CheckedPackage V2 strict reader validates structural identities"
type: TC
relationships:
  - target: ix://agent-ix/quire-contract-ir/FR-038
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-064
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-046
    type: verifies
  - target: ix://agent-ix/quire-contract-ir/FR-048
    type: verifies
  - target: ix://agent-ix/quire-specification/TC-217
    type: references
  - target: ix://agent-ix/quire-specification/TC-931
    type: references
  - target: ix://agent-ix/quire-specification/TC-946
    type: references
  - target: ix://agent-ix/quire-specification/TC-947
    type: references
  - target: ix://agent-ix/quire-specification/TC-948
    type: references
  - target: ix://agent-ix/quire-specification/TC-966
    type: references
  - target: ix://agent-ix/quire-specification/TC-367
    type: references
---
# TC-048: CheckedPackage V2 strict reader validates structural identities

## Description

Verify FR-038-AC-1 through FR-038-AC-5 (QSpec FR-322-AC-4, FR-322-AC-8,
FR-322-AC-10 and FR-322-AC-269) against the V2 package fixtures this repository builds from its
own public vocabulary (`tests/it/support/checked_package.rs`), and
read QSpec's `proposals/checked-package-v2/fixtures/adverse.json`
`nominal_body_mutations` NB-01..NB-06 directly from its selected checkout:
compare `malformed_wire` at each recorded body locus for NB-01..NB-05 and admission of the
ordered enum for NB-06. Construct separate schema-valid source nominal bodies
whose owner, declaration, semantic type, dependencies, case or unit meaning
does not join, and compare each applicable semantic refusal after wire validation;
do not copy QSpec fixtures into this repository. Then verify
FR-038-AC-27 through FR-038-AC-30 (QSpec FR-322-AC-29 through FR-322-AC-34
"Model-owned members") against a self-built domain package document, and
FR-038-AC-31 through FR-038-AC-33 (QSpec FR-322-AC-35) against the
`DependencySelection` entries of a self-built package, and
FR-038-AC-293 against recursively derived dependency closures with conflicting complete packages under one name, and
FR-038-AC-35 through FR-038-AC-38 (QSpec FR-322-AC-36 and FR-322-AC-37)
against a self-built dependency package and a package that calls it, and
FR-038-AC-45 (QSpec FR-322-AC-28) against a model-owned nominal declaration and
a model declaration node with its declared structural path and `ModelOwner`, and
FR-038-AC-46 through FR-038-AC-61 (QSpec FR-322 `lock` and `DefinitionRef`)
against the artifact references of a self-built package and the
operation catalog's `law_roles` entries, and FR-038-AC-62 through FR-038-AC-64
(QSpec FR-322 `ModelRef` and `DependencySelection`) against the
`model_root` owners and `dependency_selections` entries of a self-built
package, and FR-038-AC-65 through FR-038-AC-69 against the operation catalog's
words and the applications of a self-built package that name them, and
FR-038-AC-70 through FR-038-AC-72 against self-built compared types that reach
themselves, and FR-038-AC-74 through FR-038-AC-80 against the V2 wire types'
encoding through `quire-canonical` and the reader's canonical-bytes check, and
FR-038-AC-81 through FR-038-AC-88 against the operation identity, law, mode,
member and operand checks of a single application and the key of an application
node, as unit tests of `crates/quire-contract-model/src/checked_package/v2/operations.rs`,
and FR-038-AC-89 through FR-038-AC-95 against structural node-id preservation, canonical bytes, selected model-document integer checks and the lowering byte ceiling; retired identity-only AC-92 has no test obligation, and
FR-038-AC-96 through FR-038-AC-108 (QSpec FR-370, FR-440 and FR-250) against the
temporal, fairness, union and `case` nodes of a self-built package, the
temporal step's placement, clause, profile-fit and interval-bound checks, and
QSpec's own positive fixtures read from the QSpec checkout. FR-038-AC-66 is
retired, its ID not reused (ADR-0056), and has no case. FR-038-AC-109 through
FR-038-AC-111 are against a number whose exact value its RFC 8785 encoding loses
(the `inexact-number` and `inexact-integer` causes of a selected model document,
and the package's own byte-stream refusal). FR-038-AC-112 and FR-038-AC-113 are
against QSpec's own `adverse.json` mutations and `dependency-selection-vectors.json`
identity, read from the QSpec checkout. FR-038-AC-114 through FR-038-AC-118 (IR-495) are against the flat wire of merged QSpec FR-322 "Body grammar": nested and
misplaced applications, the pre-order pointer, the order ahead of identity recomputation,
the absence of a depth limit and of call-stack recursion, and the five body-grammar
mutations. FR-038-AC-119 through FR-038-AC-122 (IR-551) are against the timed interval form of
merged QSpec FR-370 and FR-370-AC-8: the four end variants, rational bounds in lowest
terms, the refusal of a negative, malformed or non-reduced bound, a null upper bound,
an open end with equal bounds and a missing or extra member, and the profile fit.
FR-038-AC-159 through FR-038-AC-164 verify the typed scalar operand accessor
over admitted application nodes and explicit post-admission mutation rows.
FR-038-AC-210 through FR-038-AC-213 have tagged public-reader admission
and adverse controls for the published operation catalog's declared-unit
quantity, aggregate, same-family, literal-kind and rounding-mode rules.
FR-038-AC-222 and FR-038-AC-223 have tagged public-reader controls for the
IEEE literal-family and catalog-result rules of QSpec FR-322-AC-23; IR-742's
tagged reader tests passed.
FR-038-AC-311 through FR-038-AC-317 specify QSpec FR-481's hybrid tag,
structural identity and ordered hybrid reader step. Their public-reader
evidence consumes QSpec TC-966's published vectors from the selected checkout
and compares exact code, cause and locus; QSpec's provider and host trace
checks remain QSpec evidence.
FR-038-AC-242 through FR-038-AC-245 have tagged public-reader controls for
QSpec FR-322-AC-31/67's distinct native String, UUID and Timestamp types,
six comparisons, canonical literal payloads and typed refusals; IR-800's
tagged reader tests passed.
FR-038-AC-224 is the planned structural-type forwarding work control for
operation validation; its private meter probes and public-reader work refusal
are not counted as executed evidence until IR-490 adds tagged tests.
FR-038-AC-225 through FR-038-AC-227 have tagged public-reader controls for
QSpec FR-322's aggregate-bodied `value`/`union_value` application keys. They
use the authentic QSpec positive fixture and its tampered-key adverse controls;
IR-739's tagged reader and conformance tests passed.
FR-038-AC-228 through FR-038-AC-231 are planned public-reader controls for
the catalog's three leaf-source selectors, earlier first-operand family
refusals, nested IEEE ineligibility and result-inner sequence exception. They
do not claim a runtime result until IR-491 implements and runs tagged tests.
FR-038-AC-217 and FR-038-AC-218 (merged QSpec FR-272) are against the
absence of content-mismatch and digest-derived expected/actual identity fields, while independent dependency selection and conflicting-definition refusals retain typed loci. FR-038-AC-215 and FR-038-AC-216 are withdrawn.

## Test Procedure

For FR-038-AC-239 through AC-241, construct local `protocol_clause`, `role` and `control` targets and a closed `protocol_reference` carrier through the public typed graph. Give each carrier its authored structural id, one expression occurrence, exact body target and sole dependency, and the target's semantic type. Read each complete package through the production reader and compare the exact retained target, body, graph and source correspondence. Retarget the carrier with its dependency updated; valid target joins admit under the same structural id, while missing, wrong-form and wrong-type targets refuse with their exact code, cause and `/body/target` pointer. An extra or unequal dependency refuses at `/dependencies` before the target join. Exercise reciprocal carriers in a valid recursion group; retain their producer-assigned structural node IDs, shared non-empty label and exact in-group structural references. Duplicate id, malformed group, forbidden owner/declaration and non-reference body controls refuse at their independent stages. No preimage, key recomputation or package digest is used.

For FR-038-AC-294 and FR-038-AC-295 (IR-785), take that
otherwise admitted public-reader package with one carrier occurrence of
`expression` ordinal 0. Add a second reference to the same local target by
adding ordinal 1 to that carrier and a distinct source-map row naming the
second reference's own source regions. Retain the same structural id and package name before and after; assert both complete packages
admit and preserve their respective occurrences, there is still exactly one carrier, and
the admitted occurrences and source correspondence retain both references.
Add a third distinct ordinal and source-map row to check that admission is
not capped at two occurrences. Keep the body and its target unchanged.
This is a real serialized input to `CheckedPackageV2::read`, not a
post-admission mutation, a compile-only check or an assumed admission.
The positive control must fail if the reader's previous `len > 1` rejection
is restored, and an identity implementation including occurrences must fail
the equality assertions.

From the admitting two-reference package, separately repeat an occurrence
ordinal, repeat a source-map key, empty the occurrence array, replace the
first and then the second occurrence role with a non-`expression` role, and
insert a second actual carrier node with the same `node_id`. Check each
exact code and concrete later-entry or member pointer stated by AC-295.
For the repeated ordinal, retain unique source-map rows so a duplicate row
does not mask the occurrence failure; for the repeated row, retain distinct
occurrence ordinals. Keep canonical encoding and structural ids valid in every mutation, including the added-node control. Do not use an unrelated graph refusal as evidence for
source-map or graph uniqueness. Preserve the existing target/dependency,
closed-body, grouped-cycle and forbidden-member controls of AC-239 through
AC-241.
Producer conformance may additionally use freshly produced bytes through
the producer's public interface, with exact source-region assertions;
neither copied upstream fixtures nor a QSL internal dependency replaces the
local public-reader controls above. The implemented native tests bind
to AC-294/AC-295 with `#[trace]` on `#[test]`. The focused
`checked_package_v2_protocol_reference` module passed all twelve native tests,
including the one/two/three-reference admission and separate adverse controls.
Restoring the prior production cardinality rejection made the new positive
control fail at its two-reference admission with `invalid_semantic_graph`
at the second occurrence; restoring the healthy production implementation
returned all twelve controls to passing. This is focused reader evidence,
not a full matrix or aggregate release-gate result.

For AC-210 through AC-213, start with valid checked V2 packages, not a
post-admission mutation of an in-memory graph. Build a self-typed declared
unit, two values of its type, a Boolean, an Integer, and the selected laws and
members each operation needs. Exercise the operand check of every `quire.op.quantity.*` identity
against its published operand list, changing its first quantity operand to
Boolean; read valid `quantity.add` and `quantity.eq` packages end to end,
then place a declared-unit quantity in `integer.add` argument 0 and change
`quantity.pow`'s integer exponent to a quantity at the operation step. Compare two quantity branches and then a quantity/Integer branch
of `control.if`. Supply valid aggregate terms in the catalogued positions of
`temporal.clause` and `state.clause`, then place an aggregate in
`boolean.and` argument 0 as the family-skip adverse case. Replace the first
`state.clause` aggregate by a Boolean reference separately to check the
earlier structural-shape refusal, not the operation-family check. Supply
literal `value_kind`/`type` pairs for Integer, enum, ordered enum and option,
and mismatches including an integer literal declared at a unit type and a
literal in an `any_value` position so the enclosing
operand check cannot conceal the literal defect. Finally, apply `exact` and
`toward-zero` rounding to otherwise identical valid quantity conversion and
sum applications. Retain structurally valid node ids, update affected references and compare the complete package after each authored change. Compare the public reader's code, cause and exact
argument, literal-type or mode-value pointer; require each positive control
to admit. Keep `compound_unit`, enum aliases and `model_population` out of
these admission claims until FR-322 defines their family behavior.

Dispatch unknown, empty, absent and malformed `contract_version` values, duplicate members, malformed JSON and noncanonical bytes before version-specific checks. Admit every current in-repo positive V2 fixture. Apply every authored adverse structural mutation and compare exact code, cause, pointer and locus. Separately inject unknown members, unsupported capabilities, dangling references and incomplete source maps. Admit at exact byte, node, edge, occurrence, diagnostic and work limits, and refuse or return incomplete one unit below each exact boundary.

For AC-18, use a self-typed body-root literal and nested or other references to the same structural node id. The body-root exemption applies only where QSpec defines it; all other targets must be declared dependencies and resolve. A valid body edit under a retained id is judged by independent body and graph rules.

For AC-45, read a model-owned declaration with owner `{kind: model, identity, node}` and no version. Its selected model-root identity and declaration node join admit; a foreign owner, missing selected document, empty owner value or extra owner member returns the exact owner or wire refusal. Replacing a selected document with a different body under the same name reaches its independent model and member validation rather than a content-key check.

## IR-1141 structural owner and graph controls (FR-038-AC-343)

Read one positive V2 package with declared and derived structural ids. Independently mutate it to duplicate a node id, mismatch a declared path and qualified name, break a derived body shape, select an absent or unselected owner, leave a `reference` target dangling, give a compound unit a wrong shape, and leave a frame `semantic_type` unresolved. For each mutation, keep unrelated ids and bodies valid, read through the public reader, and compare exact code, cause and RFC 6901 pointer. Pair an owner-selection defect with a dependent dangling edge and require the owner refusal; pair a graph-shape defect with an owner or body defect and require the graph refusal. No content-key or nominal-rekey vector is used.

For FR-038-AC-364, use separately authored, otherwise valid ungrouped nodes
with a single `(package,path)` at ordinal 7 and with two nodes sharing that
pair at ordinals 7 and 23. Update every reference to the authored ids and
require admission without a zero, contiguity or sibling-order check. Read
QSpec `proposals/checked-package-v2/node-ordinal-vectors.json`
`uniqueness_cases` NO-05..NO-08 directly from its selected checkout: NO-05
admits the original ordinal 0 and a sibling at 7; NO-06 admits siblings at
3 then 2 before the original at 0, in neither ordinal nor graph order;
NO-07 duplicates the original's ordinal 0 and NO-08 repeats ordinal 4,
each refusing `invalid_semantic_graph` at the recorded later `node_id`.
Apply the vector's declared placement and compare exact code and locus.
Also supply a negative, fractional, string or above-safe-integer ordinal
and require `malformed_wire` at that ordinal. Keep recursion-group membership tied to its shared non-empty string label,
with `node_id.ordinal` producer assigned and each in-group reference naming an
exact structural ID. Permute an otherwise valid group without changing its
IDs, labels or references; require admission and a changed complete graph-array
value. The former group-position and group-reference-ordinal controls are
withdrawn in place because the checked wire has no such carrier. Read QSpec FR-322-AC-274's four positive
group fixtures directly from the selected checkout: `positive-recursive-records`,
`positive-two-owners-a`, `positive-two-owners-b` and
`positive-integer-literal-references`. Each fixture group has a declared
member: assert every member, including an anonymous derived member, shares
that member's declaration path and retains its
producer-assigned structural ordinal; assert admission without deriving that
ordinal from graph position. Read QSpec XP-04
directly from its selected checkout: its second equivalence node's ordinal
1 makes a distinct structural id, while the repeated pair still refuses
`invalid_semantic_graph` for pair meaning, not an ordinal-shape or
duplicate-id defect. Copy no vector into this repository.

## Expected Results

Valid protocol references to clause, role and control nodes admit with exact local targets and one graph collection. Extra or unequal dependencies refuse `invalid_semantic_graph` at `/dependencies`; absent local targets refuse `missing_declaration`/`missing-name` at `/body/target`, even if a dependency package has the same id. Wrong form or type refuses `ill_typed`/`operator-ineligible`. Malformed body, group, owner and occurrence controls refuse at their own earlier stages, and no failure is attributed to a removed identity check.

The positive controls above admit. A wrong quantity operand, an aggregate in
a Boolean operand slot, or a mixed-family `control.if` branch refuses
`ill_typed` with cause `operator-ineligible` at the first offending argument.
A Boolean reference in the first `state.clause` position refuses
`invalid_semantic_graph` at that node's body during structural validation,
before the operation check. A mismatched literal kind/type pair refuses
`ill_typed` with cause `operator-ineligible` at that literal's `type`. A quantity
conversion or sum with a rounding mode that disagrees with its quantity type
refuses `invalid_package` with cause `operation-mode-type-mismatch` at
`operation.mode/value`. No changed control can pass merely because the
reader lacked a family or skipped a default `exact` pin.

An unknown, empty or absent `contract_version` refuses as
`unknown_contract_version`; a malformed `contract_version`, document, or a
duplicate or noncanonical top-level member refuses as `malformed_wire`,
`duplicate_member` or `noncanonical_wire` before any version-specific
decoding. Positive fixtures admit. Current QSpec positive packages admit with their published names, structural ids, complete bodies and source correspondence. Every adverse or injected case returns its independently expected refusal or incomplete result with no partial package. Model owners resolve only through selected model roots and declarations; no selected document is inferred or repaired.

## Native String, UUID and Timestamp admission (FR-038-AC-242 through FR-038-AC-245)

IMPLEMENTED (IR-800; tagged reader tests passed). From a selected Semantic IR document using its
existing `typeRef` member, author a field, parameter and operation result for
each of `ix://quire/native/String`, `ix://quire/native/UUID` and
`ix://quire/native/Timestamp`. Through `CheckedPackageV2::read`, assert the
three distinct, self-typed scalar nodes and their member types, with String
neither a `text` node nor a type requiring a text profile. The selected
document and package remain in their authoritative checkout or are built by
this repository's test builder; do not paste another repository's fixture.

Build a valid package for each of the six catalogued `eq`/`ne` identities.
Give it two operands of its own family, a Boolean result and no law, mode,
member or leaves. Use canonical equal and unequal payloads, including exact
String sequences `\u00e9` and `e\u0301`, UUIDs differing by one hexadecimal
digit, and Timestamp `0`, `1` and `-1`. Assert public-reader admission and
the retained distinct operand and application identities, not an evaluation
result invented by the reader. For **each** of the six admitted identities,
replace argument 0 with an operand of a foreign family, restore it, and then
replace argument 1, keeping the other argument native and every other
catalogued member valid. Use `text` against `string`, `string` against `uuid`
and `uuid` against `timestamp`, yielding twelve separate mutations. Assert
`ill_typed`/`operator-ineligible` at the changed `/arguments/0` or
`/arguments/1` pointer, respectively. Separately apply both
`quire.op.integer.lt` and `quire.op.integer.add` to a pair of `string`
operands, then to `uuid` operands, then to `timestamp` operands: six more
mutations, each refusing `ill_typed`/`operator-ineligible` at
`/arguments/0`. A reader that checks one operand only, coerces a native into
Text or admits a foreign operation fails these controls. Compare the
production catalog with the six closed identities and verify no native
ordering or arithmetic identity was added.

For each native comparison add a selected `text_profile` law and then a
`text_profile` mode, separately, to the otherwise valid package. Assert
`invalid_package`/`operation-law-mismatch` at the law and
`invalid_package`/`operation-mode-mismatch` at the mode. Give
`quire.op.text.eq` a String operand and assert
`invalid_package`/`operation-mode-type-mismatch` at its mode value. Keep the
text-profile selection valid so an earlier unselected-law refusal cannot
satisfy these controls.
With a String operand present, an arity defect or another wrong-family operand
refuses `ill_typed`/`operator-ineligible` at the arguments or first offending
argument before the String-in-text-operation mode refusal.

For each literal kind `string`, `uuid` and `timestamp`, first assign a type
node of another family and a malformed value together; assert
`ill_typed`/`operator-ineligible` at the literal's `type`. With the type
restored, assert `invalid_package`/`invalid-value` at `value` for a non-string
String payload; an uppercase, hyphenless, moved-hyphen and non-hex UUID; and
Timestamp `-0`, `+1`, `01`, empty text, whitespace, a decimal or calendar
form, and each signed-128 endpoint's adjacent exterior value. Admit exact
String Unicode sequences, UUID nil/max and both signed-128 Timestamp
endpoints. Update every dependent reference and retain structurally valid node ids and package name for each case, and compare typed code, cause,
pointer and node locus; no stale-key, schema or resource refusal substitutes
for the specified operation-step outcome. An exact work-limit control also
retains the reader's existing incomplete-before-later-defect precedence.

## IEEE literal and catalog result admission (FR-038-AC-222 and FR-038-AC-223)

Implemented (IR-742). Build otherwise valid V2
packages using the published `quire.op.ieee.to_float32` and
`quire.op.ieee.to_float64` entries, their selected
IEEE law, correctly typed `type_argument`, and genuine Float32 and Float64
type nodes. The Float64→Float32 package with a `float64_bits` source literal
and Float32→Float64 package with a `float32_bits` source literal must each
admit through `CheckedPackageV2::read`.
Then change only a `float32_bits` literal's declared `type` to Float64, and
only a `float64_bits` literal's declared `type` to Float32, in separate fresh
packages. Place each in an otherwise eligible argument position so an
enclosing operand-family rejection cannot stand in for the literal's own
kind/type check. Each must refuse `ill_typed` with cause
`operator-ineligible` at that literal's `/type`, with its containing node as
locus, rather than admit or refuse at the enclosing `/arguments/{i}`.

For each conversion identity, retain the valid destination `type_argument`
but change `result_type` and the enclosing root node's `semantic_type`
together to the source-width type node. Separately change `type_argument`,
`result_type` and that root node's `semantic_type` together to the source-width
node. The root `result_type` equals its node's `semantic_type` in both adverse
packages, so the separate root-mismatch rule cannot decide either case. The first
mutation isolates the catalog's `member` result form; the second isolates its
`member_family` constraint. Each must refuse `ill_typed` with cause
`operator-ineligible` at the application's `/result_type`, with its containing
node as locus. The valid cross-width packages must still admit. Update every affected reference through the public vocabulary before each read; a private helper result or synthetic post-admission graph does not satisfy these cases.
Compare typed code, cause, pointer and locus, without parsing diagnostic prose.
The adverse packages must be valid through earlier wire, structural graph and resource
checks so they distinguish these admission failures from the existing first
refusal order. These cases do not assert an admitted same-width IEEE operation.

Implemented (IR-225; FR-038-AC-247 through FR-038-AC-251). Build fresh otherwise valid
`quire.op.ieee.to_rational` applications for both Float32 and Float64 source
operands, using the catalogue's `member` result form, `type_argument` member
and `member_family: rational` constraint. For each source width, admit the
rational/rational member declaration and `result_type` pair. Then separately
read float/float (the source width in both positions), float/rational and
rational/float pairs. Match the root node's `semantic_type` to each candidate
`result_type` so the root mismatch rule cannot decide the case. Independently update all affected references and compare each complete package value before each public read. Each adverse package must pass earlier wire,
identity and resource checks and refuse `ill_typed`/`operator-ineligible` at
`/semantic_graph/nodes/{i}/body/result_type`, with the application node as
locus: float/rational and rational/float exercise equality (AC-247), while
float/float exercises the resolved-family constraint (AC-248). A valid
`quire.op.quantity.convert` into a `compound_unit` with equal member declaration
and result must admit despite its unresolved family, as FR-038 defers that
family ruling to FR-322 (AC-248). A resolved non-quantity declared type with
equal member and result on an otherwise valid `quantity.convert` must refuse at
`result_type` by its `member_family` constraint (AC-248). Add an otherwise valid
`quire.op.collection.size` public-reader package with a collection-kind operand,
a `model_population` type-argument declaration whose semantic type is its
`Set<Reference<T>>` base, and `result_type` equal to that declaration. It must
remain admitted: this result check skips the unresolved `model_population`
family rather than applying the generic walk's `set` family to the entry's
`integer` member-family constraint (AC-248).

For AC-249, separately remove `declaration`, malform its node id, and replace
it with a `type_argument` key on an otherwise valid type-argument member.
Each refuses `invalid_package`/`operation-member-mismatch` at
`/semantic_graph/nodes/{i}/body/operation/member`, with the application node
as locus. The code PR updates the eleven existing IR fixtures that spell
`member.type_argument` to `member.declaration`; it adds no alternate reader.

For AC-250, start from admitted `quire.op.numeric.narrow` and
`quire.op.quantity.convert` fixtures with `member.declaration == result_type`.
For each identity, change only `result_type` and the enclosing root
`semantic_type` to a distinct valid type while keeping the member declaration
and all other operation requirements valid. Retain structurally valid ids and require `ill_typed`/`operator-ineligible` at application `result_type`, with
the application node as locus. These are two distinct non-IEEE catalogue
entries. Exercise the existing `field` case in
`tests/it/checked_package_v2_model_fields.rs` (`record.project`) and the
`relationship_end` and `operation` cases in
`tests/it/checked_package_v2_model_members.rs` (`model.navigate` and
`model.dispatch_call`). The code PR adds one admitted public-reader package
for `tuple.project` (`position`) and one for `collection.index` (`element`),
since neither kind has an existing IR case. In each new package, make the
member declaration differ from `result_type` and require continued admission;
applying the type-argument equality rule to either kind must fail this check.

For AC-251, read every entry from the published catalogue used by the reader.
Build a fresh, previously admitted package for each identity supported by the
fixture helpers and require continued admission. Compare the remaining identity
set with an explicitly named and counted unavailable set, with its fixture
limitation stated, so a new catalogue entry cannot silently escape the test.
The differential currently has behavioral controls for 56 of 160 entries;
the remaining 104 are classified by the fixture they require: 25 need
control, temporal, state, claim or protocol context; 15 need IEEE profile and typed operands;
3 need selected model declarations; and 61 need other typed operands or member
fixtures. These counts describe this differential's controls, not a complete
baseline for the catalogue.
The admitted controls must include `model.lookup` and `model.all_instances`,
whose result forms are not `member`, and `collection.fold` and
`collection.reduce`, which have no `member_family` and admit when their
declaration equals their result. Require the newly refused `to_rational`
same-width and mismatched pairs and the count, sum, convert and narrow
mismatched-result cases. Verify that selecting by member kind alone breaks the
lookup/all-instances admitted controls, and that selecting an IEEE-only identity
allow-list breaks a non-IEEE mismatched-result control.
For fold and reduce, an unequal result must refuse at `result_type`. A wire
member kind that disagrees with the current entry still refuses earlier at
`operation.member` (AC-69 and AC-98; AC-65 defines the closed member-kind vocabulary).
For a two-defect `quantity.convert` with wrong `result_type` and a true
rounding-mode type defect, require the result refusal first; pair an operand
defect with a wrong `result_type` and require the operand refusal first. The
result check runs after member shape and operand checks and before `result_inner`, state-clause
result, model-member, mode-type and leaf checks (AC-247). Compare typed code,
cause, pointer and locus throughout.

The catalogue's `result: member` entries with `member: type_argument` are
`quire.op.quantity.convert`, `quire.op.ieee.to_float32`,
`quire.op.ieee.to_float64`, `quire.op.ieee.to_rational`,
`quire.op.ieee.from_exact`, `quire.op.reference.upcast`,
`quire.op.numeric.convert`, `quire.op.numeric.convert_rounding`,
`quire.op.numeric.narrow`, `quire.op.rational.narrow`,
`quire.op.collection.fold`, `quire.op.collection.reduce`,
`quire.op.collection.count`, `quire.op.collection.sum.integer`,
`quire.op.collection.sum.rational`, `quire.op.collection.sum.decimal`,
`quire.op.collection.sum.float32`, `quire.op.collection.sum.float64`,
`quire.op.collection.sum.quantity`, `quire.op.collection.size` and
`quire.op.collection.convert`. The five other `result: member` entries are
`quire.op.record.project` (`field`), `quire.op.tuple.project` (`position`),
`quire.op.collection.index` (`element`), `quire.op.model.navigate`
(`relationship_end`) and `quire.op.model.dispatch_call` (`operation`).

## Selections bind by name (FR-038-AC-62 through FR-038-AC-64)

Read a package whose lock has exactly `sources`, `edition`, `profile_selections`, `definition_selections`, `required_features` and `dependency_selections`, with every direct dependency selection exactly `{identity}`. Supply each dependency under a matching nonempty package `name` and require admission. Extra `package_id`, `version`, `digest`, `identity_preimage` or model-selection members refuse `unknown_member` at the first extra member; missing or empty required names refuse at the closed-wire stage. A missing package refuses `missing_import`/`missing-selection`, and unequal complete packages under one selected name through a diamond refuse `invalid_package`/`conflicting-definition`.

## Catalog words (FR-038-AC-65, FR-038-AC-67 through FR-038-AC-69)

Read the production catalog and convert each of `case`, `temporal_formula`,
`temporal_fairness`, `temporal_interval`, `fairness` and `union_arms` from its
wire string to its enum member and back. Compare every operator class, member
kind and constraint kind the catalog declares with the enum's member set. Read
catalog bytes that name an operator class, a member kind and a constraint kind
outside the vocabulary, one at a time. Put an operator outside the closed
operator vocabulary on a package application and read the package.

Name `quire.op.control.case` under operator `unary`, and an identity the catalog
lacks, and read each. Build two defective body-root nodes, one a
`temporal_formula` application in a `function` node and one other operation
defect, in both structural-id orders (AC-67). Put a `temporal_interval` member on
`quire.op.boolean.not` and on `quire.op.temporal.clause`, and a `fairness`
member on `quire.op.boolean.not` and on `quire.op.temporal.holds` (AC-69).

Run the operation check of a `quire.op.temporal.clause` application node as a
unit and, since IR-549, through a package read whose formula node is a
`temporal_formula` application (AC-68): with its law selected, no member and the
six arguments, the first a `reference` to a `value`/`parameter` node of a record
type, with five and seven arguments, with a `text` literal first and a
`reference` to a Boolean node sixth, with a `reference` to a `temporal`/`formula`
node sixth and in a `boolean` operand position and an `any_term` position, and
with a `profile_operator` member and a `fairness` member.

Expected: every word converts both ways and the catalog's sets equal the
enums'; each unreadable catalog returns the typed error of FR-038-AC-58
naming the word; the foreign operator refuses `invalid_semantic_graph` at the
term; the `unary` case refuses `operation-class-mismatch`, the unknown identity
`unknown-operation`; the lower structural id is reported in both orders; the stray
members on `quire.op.boolean.not`, `quire.op.temporal.holds` and
`quire.op.temporal.clause` refuse `operation-member-mismatch` at the member; the
clause passes in the unit and the package, the wrong counts refuse
`operator-ineligible` at `arguments`, the two wrong families at their arguments,
the formula reference fits the sixth operand and the `any_term` position and not
the `boolean` one, and the two members refuse `operation-member-mismatch`.
Compare the whole refusal code, cause and pointer.

## Temporal, fairness and case admission (FR-038-AC-96 through FR-038-AC-108)

Implemented (IR-549 code change) in `tests/it/checked_package_v2_temporal.rs`
(AC-96 through AC-98, AC-100 through AC-105, AC-108), `tests/it/checked_package_v2_union.rs`
(AC-99, AC-101, AC-106), `tests/it/checked_package_v2_catalog_words.rs` (AC-100's
nested and diagnostics cases), the unit tests of `operations.rs` and `temporal.rs`
(AC-96, AC-97, AC-98, AC-103: an `over` outside its clause's `dependencies` is
reachable only at the step, since the dependency join refuses it first in a
package read) and `tests/conformance_qspec/main.rs` (AC-107, run by
`make conformance-qspec` alone). The two-clause cases of AC-102 and AC-108 find
the structural-id order they need by varying the clauses' names, and the adjacent pairs
of one clause are swapped where the two defects' positions can be (the
operator order of a formula tree); `over` and the fairness argument have one
position each.
Build, from this crate's own vocabulary and never from a copy of QSpec's
fixtures, a package whose `temporal`/`formula` nodes apply each of the fifteen
`temporal_formula` identities with their catalogued member and operands under a
clause selecting a bounded profile, a `temporal`/`fairness` node applying
`quire.op.temporal.fair` under a clause selecting `quire.temporal.infinite-trace/v1`,
and a clause whose formula applies `quire.op.temporal.eventually` with the
interval `{0, 3}` over `quire.op.temporal.holds`; read it, then lower it under a
profile that supports the `temporal` tag and under one that does not (AC-96). On
each of the eight interval operators read the member as `{0, 3}`, `{9, 10}`,
`{2, null}`, `null` interval, `{-1, 3}`, `{0, -2}`, `{-5, -2}`, `{3, 0}`, `{10, 9}`, `{2^64+1, 2^64}`,
a `fairness` member, a `null` member, an interval with a third member, an integer
`lower`, and bounds `"1.5"`, `"01"`, `"+1"`, `""` and `"3x"`, and put a member on
`holds`, `not` and the clause (AC-97). Read
the `fairness` member well formed with `weak`/`whole` and `strong`/`each`, then
with `medium`, with `part`, without `name`, with an extra member, `null`, of kind
`temporal_interval`, and on `quire.op.boolean.not` (AC-98). Build the `Shape`
union, the `Shape::Rect(2, 3)` value and the three-arm `case` and read them; then
read the `case` with its arms out of order, an arm omitted, an arm repeated, a binder aggregate of
the wrong count, a binder of the wrong type and an arm body of another type; a
union with a duplicated member and one with a non-type payload; and a union value
naming an absent member, with a wrong payload count and with a payload of the
wrong type (AC-99). Put an application of each of the four classes in a node of
another form, as an element of another application's `arguments`, as a `binding`
value, inside an `aggregate`, and as a root and as a nested term in a diagnostic
entry's `details`, with a `details` reference to a formula, a fairness, a union and a
union value node (the first two refuse, the last two admit; merged QSpec FR-370, #182), a `case` nested in another term, and an `expression` node whose form
contradicts its root operator class (`invalid_semantic_graph`); give a formula,
fairness and clause node an empty `aggregate`
body, a `literal` body and another class's application; reference a formula node
from a `function` body, a fairness argument and a `case` argument, and a fairness
node from a formula argument and a formula operand; read each, then read each
class, node and reference at its own place (AC-100). Check that temporal/formula
and expression/case applications are not
blanket-refused `unsupported_construct`/`expression-form`; the planned IR-661
relationship-navigation additions require the lead-owned existing TC-048 Rust
refusal-enum assertion to be amended, not claimed as executed here. Read a
diagnostics entry whose `code` is `unsupported_construct`, and lower the admitted
`temporal`/`formula` and `expression`/`case` nodes under supporting and
non-supporting profiles, comparing each lowered node's exact structural ID and body
with the admitted node's, and requiring `unsupported` for a profile lacking its tag
(AC-101). Build a placement defect beside a lower-structural-id node with an
unknown identity or a class mismatch, in both structural-id orders, two placement
defects, and, within one clause, each adjacent pair of the order `over`, fairness
resolution, profile fit, bounds, with the two defects' positions swapped; then two
clauses, the lower-structural-id one holding a profile-fit defect and the higher-structural-id
one an `over` defect, in both structural-id orders; and a member `{lower: "1.5", upper:
"0"}` and a wrong-kind member under a bounded and the infinite-trace profile;
`{0, -2}` and `{-1, -3}`, and `{-1, null}` under a bounded profile, under
infinite-trace and beside a placement defect at a lower-structural-id node, each
refused at the negative bound at schema validation (strict wire validation, the flat wire pass),
as is `{"1.5", "0"}` at its malformed bound, none of them `operation-member-mismatch`
(AC-102). Give a clause's
`over` a `value`/`parameter` dependency, one that is no dependency and a
`scalar_type` dependency, and a fairness member an existing operation, a missing
name, an absent declaration, a non-`model` declaration, a name matching two
operations, an operation the node only inherits and a declaring node whose owner
is not recovered (AC-103). Select
`quire.temporal.event-position.false-extension/v1` and
`quire.temporal.infinite-trace/v1` in turn over closed, `{lower, null}` and `null`
intervals and an empty and a non-empty fairness argument, with a second clause
whose tree shares no node with the first (AC-104). Give `holds` a formula
reference and a Boolean reference, `until` one argument, `not` two, `true` one,
`and` a Boolean reference and `not` a formula reference (AC-105). Compare two
`Shape` values, a `Shape` and another union, a union reaching no `text` and the
`Label` union with its leaf list correct, missing a leaf, repeating one and with a
leaf for `Empty`; the recursive unions `IntList` (no `text`, `leaves` empty) and
`TextList` with the leaves `["member:Cons", "position:0"]` and
`["member:Cons", "position:1", "recursion:0"]`, the recursion leaf missing, written
`recursion:1`, and placed at an `IntList` reentry (AC-106; `member:<Ident>` and union
cycles are merged QSpec FR-322-AC-45 and AC-46; the recursion-leaf entry for a
cycle that reaches `text` is merged FR-322 text, #182). Read an empty union and a union value with two
bindings (AC-99, `invalid_semantic_graph` at the body); a fairness `declaration`
naming a `scalar_type` node (AC-103, `invalid_model_binding`/`malformed-declaration`
with path `.../member/declaration` on the fairness node and the target's key as
locus, comparing path and locus for every temporal-step refusal as the "Path and
locus" table of FR-038 lists them); a law with a member's identity under another `authority` (AC-108,
`operation-law-unselected`); a case arm body of unresolvable type; an unreached
formula node with `{3, 0}` (AC-97); and the placement defect in the shape of a
formula application in a `function` node beside an unknown-identity node and beside a
class-mismatch node (AC-102). Run
`make conformance-qspec` with `QUIRE_SPECIFICATION_DIR` set to a QSpec checkout, to
the empty string, to an unset value and to a path without the fixtures directory,
reading `positive-all-families.json`, `positive-clause-operations.json` and
`positive-union-nodes.json` from it, and compare the self-built packages' features
with the fixtures' (AC-107); the same target reads `adverse.json` and applies each of its mutations to
`positive-all-families.json` with no identity refreshed, and each body-grammar mutation's `flattened`
form as a positive control (AC-112), and reads current `dependency-selection-vectors.json`, checking the exact `{identity}` lock rows and supplied package names (AC-113). Run each of AC-112 and AC-113 with the variable
unset, empty and naming a path without the fixtures, with the file missing and not JSON, with a
mutation list absent or empty, with a body-grammar entry lacking `flattened`, and, for AC-112,
with an expected-failure entry naming an id absent from `adverse.json`, one the reader refuses as
recorded, and one the reader refuses with a different code than listed.

Expected: the package of AC-96 admits with its recorded identities and lowers or
returns `unsupported` naming `temporal` as stated; AC-97's admitted members admit,
`{-1, 3}`, `{0, -2}` and `{-5, -2}` and the malformed bounds `"1.5"`, `"01"`, `"+1"`,
`""`, `"3x"` and the JSON integer `0` refuse `invalid_package`/`invalid-value` at the
bound, and `{3, 0}`, `{10, 9}` and `{2^64+1, 2^64}` refuse `invalid_package`/`invalid-value`
at the application; a `null` member on an interval operator refuses
`operation-member-mismatch` at the application, and each other member shape
`operation-member-mismatch` at `operation.member`; AC-98's two well-formed members admit and each other refuses
`operation-member-mismatch` at `operation.member`; the `Shape` package admits and
each defective `case` refuses `ill_typed`/`operator-ineligible` at the `case` node,
the duplicate member `invalid_package`/`duplicate-member` and each defective
union value `ill_typed`/`type-mismatch` at the node; each misplaced application,
node or reference refuses `ill_typed`/`operator-ineligible` as AC-100 locates it
and each is admitted at its own place; the reader's refusal types hold neither
word, the diagnostics entry reads, and the lowered body equals the admitted body;
the placement defect is reported ahead of every other defect in both structural-id
orders, and each later stage ahead of the next; the `over` and fairness cases
refuse `missing_declaration`/`missing-name` or `invalid_model_binding`/
`malformed-declaration` as AC-103 states; the bounded profile refuses a `null`
interval, a `null` `upper` and a non-empty fairness argument
`operation-member-mismatch` and the infinite-trace profile admits them; the
operand cases refuse `ill_typed`/`operator-ineligible` at the operation step; the
`Shape` and `Label` comparisons admit and refuse as AC-106 states; and
`make conformance-qspec` passes with each QSpec fixture admitted with its published
`name` and structural ids and fails when the variable is unset, empty or names a path without
the fixtures; `make test` does not run it, and wiring it into CI is a separate
decision. Selecting `quire.fixture.temporal-profile/v1` (selected nowhere else),
an empty identity and a differently spelled infinite-trace identity as the clause
profile refuses `unknown_profile`/`unsupported-selection`; naming
`quire.package.composed/v1` with the lock selecting it as a `binding_contract` row,
and `quire.protocol.complete/v1` with the lock selecting it as a `protocol_profile`
row, refuses `unknown_profile`/`wrong-selection-role`, while the same two
identities with no such lock row refuse `unsupported-selection` (merged QSpec
FR-370 "Profile check" and FR-370-AC-10, decided from the package alone); a clause with
no `temporal_profile` law or two skips the check; each at the law's
`definition`, first among the clause's checks, and each of the five FR-250
members admits (AC-108). Compare the whole
refusal code, cause and pointer. Every structural mutation of `adverse.json` refuses with its
recorded code, every body-grammar mutation refuses `malformed_wire` as recorded or is listed with
its current refusal and owning ticket, each flattened form is not refused `malformed_wire`, and
the version-free dependency selections retain their exact published identities and the reader joins each supplied package by name (AC-112 and AC-113); each fail-closed
case above fails the run and never skips.

## Parameter and compound-unit nodes (FR-038-AC-22, FR-038-AC-23)

Build a package in QSL FR-092's shapes for `function both(a: Boolean, b:
Boolean): Boolean { a and b }` plus the compound unit `Metre^2`, read it and
lower every node.
Then mutate one parameter, the compound unit or the application node at a
time and re-read. The valid package and the dimensionless compound unit admit.
A parameter body mutation (level, name or binding) refuses
`invalid_semantic_graph` at its `body`; parameter dependency/type defects and
application dependency defects retain their graph or declaration refusals.
Independently mutate compound-unit term order (`RM-CU-ORDER`), a zero or
non-integer exponent, non-unit or non-root unit, repeated unit, and wrong
semantic dimension; each schema-valid derived-shape defect refuses
`invalid_package`/`invalid-derived-shape` at the compound unit's `node_id`.
A wrong dependency list instead refuses `invalid_semantic_graph` at
`/dependencies`, including when the same node also has a derived-shape defect;
assert this precedence. These corrected compound-unit cases are
PLANNED/UNRUN until backed by the IR reader.

## Derived reference target-kind join (FR-038-AC-124)

Build one otherwise valid `composite_type`/`reference` node for each allowed
model target kind (`object_type`, `process`, `state_machine`,
`systems_interface`) and for a relation `relationship` target; assert reader
admission and exact target retention. In each separate negative package,
substitute a builtin `scalar_type`/`integer`, a source-declared function,
a `value`/`literal`, or a derived `composite_type`/`option` target with a valid
structural ID, and update the reference node's `dependencies` to contain
exactly that substituted target in the required order. Give each substituted
node the owner state required for its own kind: the declared function has its
valid joined `SourceOwner`; the builtin scalar, literal value and derived
composite nodes carry no `owner` (QSpec FR-322-AC-51). This keeps schema,
owner and dependency checks passing so the target-kind check is reached.
Each `RM-REF-TARGET` mutation is schema-valid and
refuses `invalid_package`/`invalid-derived-shape` at the reference node's
`node_id` under QSpec FR-322 "Derived type forms and forwarding". Keep the
existing reference body-shape and graph/owner checks independent. These
reader controls are PLANNED/UNRUN.

## Model-owned members (FR-038-AC-27 through FR-038-AC-30)

Build a Semantic IR 2.0.0 document with an object type `Order` declaring the
operation `total(Integer): Integer`, select it in a package's lock and add a
`dispatch_call` on `Order.total` over a `Reference<Order>` receiver. Read it:
it admits. Rename the called operation to one the document does not declare
and read it: it refuses `ill_typed`/`operator-ineligible` at the member's
`name`. Give `Order` a supertype the document does not declare and read it:
the refusal is the document's, `missing_declaration`/`missing-name`, at
the root's `/semantic_graph/nodes/<i>/owner`. Read the document directly: supply none, forged bytes, another identity, and the matching one;
give a node an invalid object id and refer to it as a supertype and a
`typeRef` from nodes on both sides of it; give one field a dangling `typeRef`
and a multiplicity of `lower > upper` in both member orders; name a
relationship as a `typeRef` from a node read before and one read after its
owner; give two nodes no identity. Compare the whole refusal and its pointer
with the expected one. Read with the `work` limit at zero, and with each work
limit one below what the direct read and a member resolution used, and check
the pointer of the `incomplete` outcome resolves in the lock.

## Artifact references and dependency selections (FR-038-AC-31 through FR-038-AC-33, FR-038-AC-46 through FR-038-AC-64)

Construct definition and source references as exact `{authority, identity}` pairs at every lock, diagnostic, source-map and operation-law site. A valid pair admits; missing, empty or wrong-kind members refuse at the exact member. An extra revision, digest, domain or export member refuses `unknown_member`. The reader retains one authoritative lock row and does not compare a second identity-preimage mirror. Two dependency selections sort by UTF-8 identity text, admit with matching supplied package names and refuse duplication or wrong order under the published lock rule. A foreign or absent selected source refuses at the owner join, without inferred replacement.

Supply equal complete packages through two dependency branches and require admission. Change one node body or ordered array in one branch under the same name and require `invalid_package`/`conflicting-definition` with both importer-rooted paths. A selected name with no package refuses `missing_import`/`missing-selection`; no content mismatch or stale-dependency code exists.

For FR-038-AC-61, start from one otherwise admitting source-owned node,
change only its `{authority,identity}` pair to one absent from `lock.sources`,
and require `missing_declaration`/`missing-selection` at that node's
`node_id` after wire validation. In a separate otherwise valid node, replace
its `owner` with a `DefinitionOwner` shape and require `malformed_wire` at
`owner` before the owner join. Neither case supplies a definition-owner
selection path or treats a wire-shape refusal as owner-join evidence.

## Operation identity, laws, mode, member and operands (FR-038-AC-81 through FR-038-AC-88)

Run the operation check of one `application` node as a unit, over a graph of the
node and the type nodes it names, and the application key check of a graph of
application nodes. Name an identity the catalog lacks, and `quire.op.integer.add`
under `unary`; admit `quire.op.integer.add` under `binary` with two plain
operands, and give `operation` a bare string (AC-81).

For AC-81's implemented required-presence controls, independently build a
valid integer-add application package with two correctly typed operands, no
laws or leaves, and both `operation.mode` and `operation.member` explicitly
`null`. Read it through `CheckedPackageV2::read` as the positive control.
Starting afresh from that admitted shape for each adverse case, remove only
`mode`, and separately only `member`, leaving the other present as `null`.
Retain valid structural node ids, rewrite references where necessary, preserve canonical node ordering and compare the complete package value after each mutation. Do not fabricate an identifier or use a
private operation-unit result as the public-reader result. Read each changed
package through the same public reader and compare its exact code, cause and
pointer. Restore the removed member as explicit `null`
to show admission is restored. These controls concern the containing
`operation` object only; do not change leaf-mode behavior. The public-reader
test `operation_mode_and_member_are_required_even_when_null_is_admitted` executes
these controls and also combines each omission with an unknown operation
identity to verify that the shape refusal precedes catalog lookup. The old
production decoder admits the omission and fails this test; restoring the
healthy decoder restores the passing result.

Supply no law to `quire.op.integer.div`, one law to
`quire.op.integer.add`, and to `quire.op.integer.div` a law whose `role` is
`not_integer_division` over a catalogued `integer_division` definition (AC-82).
Give `quire.op.decimal.add` a `null` mode; supply the mode `toward-zero` over a
first operand that is a `reference` to a `decimal_range` binding `rounding`
`nearest-even`, and over a `literal` typed at it (AC-83). Give a `null` member to
`quire.op.quantity.convert` with its `rounding` mode supplied; name an undeclared field in
`quire.op.record.project`; name no node as the `declaration` of
`quire.op.model.reaches_field` (AC-84). Give `quire.op.integer.add` three
arguments, an `integer`-typed and a `boolean`-typed literal first, and
`quire.op.boolean.not` a `quire.op.state.clause` application (AC-85); give
`quire.op.structural.eq` two parameters of one record type and of two, two typed
literals of one type and of two, and two nested applications of one result type
and of two (AC-86). Compare the operand family and the type-shaped predicate of
every kind of the closed node taxonomy with the stated pairs, the
`temporal`/`formula` pair added when the IR-503 code lands and the
`composite_type`/`union` pair when the IR-549 code lands (AC-87). Build an application member in a two-node recursion group with a producer-assigned structural `node_id.ordinal` and exact references to an in-group and an outside node. Verify its ID and references in an aggregate member, binding value and nested arguments; a valid graph permutation retains them and admits. A duplicate ID, malformed structural ordinal or unresolved target refuses under its own rule (AC-88). The former group-reference-position mutations are withdrawn in place.

Expected: the uncatalogued identity refuses `unknown-operation` at
`operation.identity`, the `unary` case `operation-class-mismatch` at `operator`,
and the correct use admits; `operation-law-missing` at `operation.laws`,
`operation-law-mismatch` at `operation.laws/0` and at the law's `role`; the
absent mode `operation-mode-mismatch` at `operation.mode` and the disagreeing
value `operation-mode-type-mismatch` at `operation.mode/value`, for the
`reference` and the `literal` (these are explicit-`null` mode and member
controls, not omitted-member cases); `operation-member-mismatch` at `operation.member`
and `operator-ineligible` at `operation.member.name` and
`operation.member.declaration`; `operator-ineligible` at `arguments` for the
third argument and at the argument for the `boolean` literal and the clause, and
at `arguments/1` for each differing type; every same-type admission admits; the
table equals the stated pairs; the bare string refuses `invalid_semantic_graph`
at `operation`; the shared group label and exact structural references resolve against the authored graph while each producer-assigned structural ID is retained; malformed structural ordinals, duplicate IDs, type-incompatible exact targets and unresolved references refuse at their own locations. Compare the whole refusal code, cause and pointer. The `definition`
of a law of the right role is FR-038-AC-56 and FR-038-AC-57's, and the modes of a
leaf are FR-038-AC-44's.

Expected for the implemented public-reader required-presence controls: the explicit-`null`
integer-add package admits; each of the two independent omissions refuses
`invalid_semantic_graph` with no cause at
`/semantic_graph/nodes/{i}/body/operation`, where `{i}` is the application's
actual index after canonical ordering, rather than at `operation/mode` or
`operation/member`. Restoring the field as explicit `null` admits. The existing
AC-83 required-mode explicit-`null` case continues to refuse
`invalid_package`/`operation-mode-mismatch` at `operation.mode`, and AC-84's
required-member explicit-`null` case continues to refuse
`invalid_package`/`operation-member-mismatch` at `operation.member`; omission
shape failures precede these catalog semantic checks. Earlier package-read
stages retain their existing precedence, and the independent graph and body checks prevent an unrelated earlier failure from satisfying either omission control.

## Typed scalar application operands (FR-038-AC-159 through FR-038-AC-164)

Read an admitted package containing integer add, subtract, multiply and negate
applications. For each, call `scalar_application_operands` with its node id and
an actual occurrence key. Supply, across the applications, references to two
distinct `value`/`parameter` nodes of the same type, a reference to a nested
application result, a reference to a graph literal, and inline integer
literals. Include two occurrences of one application and two inline literal
positions of the same value. Swap an add application's arguments in a
separately admitted package, then call again. An admitted example argument is
`{"term":"literal","type":<integer-type-node-ref>,"value_kind":"integer","value":"7"}`;
unlike `{"term":"reference","target":<literal-node-ref>}`, this inline term
has no child graph node id.

Expected: each result contains exactly the argument count and order, with
zero-based ordinals; parameter and subterm references return their distinct
`GraphChild` node ids and admitted ranges, and both literal forms return
singleton `(7, 7)` for value `7`. The graph literal returns its node id;
each inline literal returns `InlineLiteral` with the application id, supplied
occurrence and position, distinct across either change. Swapping arguments
swaps entries. Supply an unknown id, a non-application node, a different
occurrence key, and a referenced child or range removed by a crate-internal
post-admission mutation; assert the respective typed errors and no partial
result. In separately admitted packages, put a parameter typed at unbounded
`Integer`, then a parameter typed at an `integer_range` with upper endpoint
`170141183460469231731687303715884105728` (one above `i128::MAX`), in an
otherwise eligible add application. Assert `UnboundedRange` for the first and
`RangeOutOfI128` for the second. Also use a graph literal reference and an
inline integer literal with that same out-of-`i128` value as separate operand
cases; each returns `RangeOutOfI128`. Repeat the bound case with lower endpoint
`-170141183460469231731687303715884105729` (one below `i128::MIN`). Assert
each call returns only the typed refusal, without a partial list, narrowed or
saturated endpoint, or panic. The `i128` endpoints themselves return their
exact values. Mutate an admitted application's catalog identity to an unknown
one in the same way, and call an admitted application of a catalogued ineligible
identity; assert `UnknownOperator` and `IneligibleOperator` respectively.
An external API fixture exhaustively matches the public identity and error
enums and consumes every result member without JSON access. Repeated calls
and a package clone return equal values, with package equality unchanged.

## Typed package-authored composite operand projection (FR-038-AC-177 through FR-038-AC-182, FR-038-AC-246)

The accessor is implemented by `composite_application_operands` and has `tc_048_`
composite operand integration tests and narrowly scoped model-unit tests; the
corrected AC-180 18-form evidence below is planned and unrun. Admit structural.eq applications through the
production reader and published catalog using independently authored inputs.
Use authentic parameter and closed graph-composite value references, with two
actual occurrences of one application. Assert the tabled application, occurrence,
ordinals, child identities, type identities and domain fields. Swap arguments
and repeat one parameter without deduplication (AC177).

For AC246, admit otherwise equivalent structural.eq and structural.ne
applications through the public reader, with the same ordered operand children
and an authentic occurrence in each. Call `composite_application_operands` once
per application. Assert `operator` is `CheckedCompositeOperator::Eq` for eq and
`CheckedCompositeOperator::Ne` for ne in the returned operand-bearing result.
Compare the results' operands, ordinals, children, domains and `consumed_work`
for equality, while asserting each result retains its own authentic application
identity and occurrence. In the TC-048 composite-operand fixture, read `operator`
from the returned result and read no body JSON for the operation identity. Extend
the AC-182 public-fields inspection in
`tests/it/checked_package_v2_composite_operands.rs` (`inspect_public_fields`) to
exhaustively match both `CheckedCompositeOperator` variants, with no wildcard arm
and no `#[non_exhaustive]`. This step is implemented (IR-803) by the tagged typed-result and exhaustive
public-fields assertions.

For AC178, compare a supported closed record/tuple/collection/Option graph value
with its original node identity and assert Literal disposition/no positions.
Include a graph value whose member reads a parameter and an application subterm;
assert UnsupportedOperand with exact reason, enclosing ordinal and available
type identity. A closed union-value root and one nested in a closed record value
shall instead refuse UnsupportedDomain with the enclosing ordinal and original
union semantic-type identity, never NonliteralGraphValue or Literal success.
Use a reader-admitted noninteger inline term with structural type
where possible, such as an inline none term typed at Option, to exercise
InlineNonInteger. Defensive inline integer handling may use a crate-internal
post-admission mutation only: the catalog excludes Integer from structural_kind,
so no successful public structural.eq integer-inline fixture is claimed. Assert
InlineInteger refusal/no fabricated identity or value; retain reader admission's
original refusal on a separately authored integer-inline structural.eq input.
Future eligible integer-inline success/value transport requires an owning catalog
and accessor specification, not a test that bypasses admission.

For AC179, use a record with a bounded Sequence of integer-range elements.
Independently enumerate field and element paths from the original admitted type.
Assert exact lower/upper strings, including integer endpoints beyond i128 and
collection maxima beyond u64; preserve collection minimum too. Replace a path's
type in a separately admitted input and assert only its affected descriptor changes.
Repeat with unbounded Integer and collection and assert no invented maximum.

For AC180, build root List with head and an optional-presence tail. Its field
binding value is the prescribed aggregate/optional binding referencing Option<List>.
Assert the field edge retains optional_presence=true, the aggregate/binding add
no index, the actual reentry is [1,0] and the Depth key path is []. Repeat with a
direct field reference to Option<List>, asserting optional_presence=false and the
same path. Root Tree with left/right optional self-fields retains both [0,0] and
[1,0] reentry paths in lexicographic order at a single Depth key []. Contrast two
nonrecursive sibling fields sharing a type: both paths remain, with no Depth.
Cover tuple and Option ordinal edges. Assert each key's actual parameter root.

Enumerate the nine non-IEEE scalar and six non-IEEE bounded-domain table rows
through eligible containing structural types and actual reader-admitted
structural type nodes. Assert Boolean's zero positions, each Whole source, each unbounded
descriptor and each bounded source.
In particular text_bounds yields one Whole at its wrapper, and model_population
as a leaf yields a Node-keyed Whole with no Population key or member walk.
For an unordered Color enum, the admitted enum member order is Blue, Green,
Red with ordered=false, independent of its source spelling. For an ordered enum,
assert ordered=true and its semantic body member order. No accessor output
supplies the expected identifier list. Assert the exact introduced key set for
these fifteen admitted forms. At the isolated private type-walk seam, supply
well-formed `scalar_type/float32`, `scalar_type/float64` and
`bounded_domain/float_rounding` type nodes separately, without an admitted
structural comparison, and assert each tabled Whole source and shape entry,
including suppression of the underlying float Whole under `float_rounding`.
Independently construct otherwise valid public structural.eq and structural.ne
comparisons whose selected compared type reaches each of those three forms;
assert `ill_typed`/`operator-ineligible` at `operation.leaves`, with no admitted
package or accessor call. These public refusals are also AC-231 controls; a
private descriptor result cannot substitute for one.

For AC181, exercise all twelve public error variants and their exact tabled fields.
The changed IneligibleOperator case is implemented (IR-690):
`tc_048_external_error_consumer_retains_authentic_node_and_application_loci`
uses a reader-admitted `boolean.and` application outside eq/ne and asserts its
exact public application-locus refusal. Other existing IR-651 evidence remains unchanged.
Use public admitted inputs for unknown node, nonapplication, missing occurrence,
known ineligible identity outside structural.eq/structural.ne, unsupported operands and
unsupported union projection; retain authentic ordinal/type fields. Crate-internal
mutations may exercise unknown catalog identity, dangling child, malformed domain
or optional wrapper and checked-index conversion ordinarily blocked by admission.
For a reference with a valid but absent target, assert MissingChild and the actual
child id; for an absent or malformed target id, assert MalformedChild with only
the enclosing ordinal, without fabricating or salvaging a child identity.
Do not allocate a multi-billion-element graph to test the conversion helper.
An unanchored forwarding/Option/collection cycle refuses MalformedDomain, not a
fabricated depth. Pair an early operand defect with a later one to assert order.
Measure successful consumed_work, then use its exact value, one less and zero.
Assert the same logical work for equivalent node order and lookup strategies;
expected charges come from the specified semantic visits, not implementation scans.

For AC182, compile an external consumer in the normal workspace test lane. It
exhaustively matches each named public output/key/descriptor/edge/error type and
reads the specified fields without body JSON or a QSL model import. Assert repeated
and clone projections and pre/post package equality. API inspection checks the
absence of drawn bound inputs, incoming-key validator, coverage verdict or obligation
encoder. The structural.ne extension below is verified by separately tagged Ne assertions.

## Bounded integer record literal inside a closed composite (FR-038-AC-232 through FR-038-AC-235)

This IR-691 procedure is planned and unrun. Reuse TC-048 and tag each new
behavioral test with its own FR-038 criterion id. First check and emit from
fresh QSL FR-093 the concrete `record Point { x: Int[0,9]; }` and
`function mk using v(): Point pure { Point { x: 3 } }`. Hand its emitted bytes
and required-feature evidence directly to the production V2 reader. Require
admission and inspect the admitted graph: the record-value binding `x` refers
to one expression/conversion node with body-root `convert`/
`quire.op.numeric.narrow`, `type_argument.declaration`, `result_type` and
`semantic_type` all naming the actual `Int[0,9]` node, and its sole argument
refers to the Integer-typed literal `3`. Retain this producer-origin control
separately from the self-built comparison fixtures; no fixture bytes are copied
from QSL into IR.

1. Independently build and admit structural.eq and structural.ne packages
   comparing that exact shaped record value with itself. Call the public
   accessor at an authentic occurrence and assert the original record
   GraphChild, record semantic type, ordinal, Literal disposition and no
   introduced positions. At the private post-admission seam, assert the same
   for a second record whose bounded integer literal and both canonical range
   endpoints exceed i128; derive expected decimal values from the authored
   inputs, without using the scalar accessor or a fixed-width conversion.
   This private control does not claim public reader admission. Use min and
   max boundary values as separate positive controls. Also author a negative
   range beyond i128 with min `-170141183460469231731687303715884105733`
   and max `-170141183460469231731687303715884105731`; assert both endpoints
   succeed. Derive the expected signed order independently from these authored
   strings, not from the accessor. This verifies AC-232 and the positive half
   of AC-233.
2. At the private post-admission seam, mutate one wrapper, target or literal
   member at a time: operator, identity-adjacent metadata, argument shape,
   member target, result/semantic type, missing range, wrong Integer base,
   malformed `min`/`max` binding or endpoint literal term, an endpoint literal
   whose `type` names Boolean rather than the range's Integer base,
   noncanonical or inverted endpoints, wrong operand literal kind/type/spelling,
   and values one below and one above the target. For the negative range of
   step 1, test outside values `-170141183460469231731687303715884105734`
   and `-170141183460469231731687303715884105730` separately, each still
   beyond i128, and assert refusal; a comparator that strips the sign or sorts
   negative magnitudes lexicographically must fail. Assert `MalformedDomain` with
   the enclosing ordinal and the exact first available type locus specified
   by FR-038. These are defensive accessor controls; do not claim the
   production reader admits a mutated package. A valid min/max control must
   succeed, so merely accepting every `numeric.narrow` cannot pass AC-233.
3. Substitute a parameter, another conversion, an unrelated application or
   a value cycle below the wrapper. Assert the first offending node's actual
   semantic type, enclosing ordinal and `NonliteralGraphValue`, with no
   partial output. Keep direct parameter and closed tuple, Option,
   collection and enum-value controls on their existing paths. Compare the
   public reader's acceptance/refusal of the unmodified packages before and
   after this accessor-only change. This verifies AC-234.
4. Count a small fixture's graph resolutions, node and term/member visits,
   examined range-endpoint bytes and examined literal bytes independently from
   its authored shape. The literal and endpoint bytes are not copied to an output.
   Call at the exact successful work limit, one less and zero; assert the
   resulting consumed-work or `WorkLimit { limit, consumed }` values. Arrange
   an exhausted charge before a malformed wrapper, an out-of-range literal
   and a later operand defect, and assert WorkLimit precedence. Reorder the
   admitted graph and assert equal content and logical work. Inspect the
   public API and source for the unchanged type set and one existing walk/
   meter owner. At the private seam, use positive canonical 4096-digit min,
   literal and max spellings beginning with `1`, `5` and `9` respectively,
   with all remaining digits authored and valid. Their first digit decides
   order, while the meter must charge both complete endpoint byte lengths
   once per logical visit. Assert that the exact work limit succeeds and one
   less returns `WorkLimit` at the first attempted charge. This verifies AC-235.

## Structural inequality operand extension (FR-038-AC-197 through FR-038-AC-201)

The extension procedures are implemented by the TC-048 composite operand tests
and source inspection (IR-690). Existing IR-651 equality execution alone does
not discharge these rows.

1. For AC197, independently author and admit structural.ne inputs through the
   production reader and owning catalog, using supported parameters and closed
   graph composite values. Assert the actual application/selected occurrence,
   ordinals and original child/type identities. Swap arguments, repeat a child,
   select a second authentic occurrence and request an absent occurrence (AC199):
   assert MissingOccurrence with the supplied application and occurrence. Compare
   with the authored source, never an accessor-produced expected value.

2. For AC198, admit corresponding eq/ne applications over the same operand type
   graphs: bounded/unbounded collection and integer fields, optional recursive
   records with separately shared paths, ordered/unordered enum descendants,
   and closed record/tuple/collection/Option values. Independently enumerate the
   expected paths, descriptors and identifiers from the original authored input.
   Compare exact shapes and introduced positions, including decimal values beyond
   fixed-width ranges. Reject any changed/complemented bound or invented maximum;
   preserve union and inline limitations rather than admitting new families.

3. For AC199, apply the existing refusal procedures to ne at nonzero operand
   ordinals: unsupported inline/subterm forms, unsupported union, malformed
   optional/domain structure and two ordered defects. Assert the exact existing
   variant and payload, no partial output. Admission-inaccessible branches use
   explicitly defensive admitted-then-mutated model tests only. A separately
   authored catalog-ineligible input retains the reader's original refusal;
   a known admitted non-eq/ne operation returns IneligibleOperator with its actual
   application id at the accessor. Retarget the existing public AC181/182 error-
   consumer assertion to that identity without removing or weakening its payload
   oracle. The former structural.ne refusal was superseded; the retargeted public
   assertion now verifies IneligibleOperator for an admitted non-eq/ne operation.

4. For AC200, measure successful actual logical work, assert exact-budget success
   and zero/minus-one WorkLimit with exact limit and attempted consumed payload.
   Independently count the existing logical visits/retained bytes for a small
   authored fixture to detect extra or unpaid work. Change admitted node order
   and compare corresponding eq/ne accounting; do not derive the expected visit
   count by invoking the implementation twice.

5. For AC201, inspect the actual production eligibility extension and owner seams:
   retained admission index; operations::resolve_family_with;
   operation_catalog().family_fits; structural::reference_target; shared domain
   walk/meter. Pass only if the existing signature/types and scalar/admission
   semantics remain and no duplicate derivation, per-call uncharged index,
   depth ceiling, encoder or dependency is added. This is source Inspection,
   not runtime or downstream replay evidence.

Expected results: AC197 returns exact authentic positional outputs; AC198
retains corresponding authored domains without inequality-specific alteration;
AC199 preserves typed refusal/first-defect semantics; AC200 preserves exact
logical charges; AC201 finds the single existing owner path. Each missing or
altered predicate fails its own criterion. No downstream CG/QSL replay,
new scalar eligibility, harness draw or whole-domain coverage is claimed.

## Operation leaf count (FR-038-AC-43)

Apply `structural.eq` and `structural.ne` to two parameters of an all-integer
record, and `collection.contains` to a set of integers and an integer, each
with `leaves` empty, and check each admits. Repeat over a record with a nested
`text` field and over a set of `text` with `leaves` empty, and over two `text`
fields with one leaf supplied, and check each refuses `invalid_package` with
cause `operation-law-missing` at `operation.leaves`. Apply `collection.flatten`
from a sequence of sequences of `text` to a sequence of `text` with `leaves`
empty and check it admits, then to a `set`, `bag` and `ordered_set` of `text`
and check each refuses `operation-law-missing`; do the same for
`collection.set` over `text`. Give a record a field naming a node that is not
in the graph, and a type that reaches an option again with no record or tuple
between the two visits, and check each refuses `ill_typed` with cause
`operator-ineligible` at `operation.leaves`. Give a record a field that reaches
the record again through an option and check it is no longer refused
(FR-038-AC-70), and a record whose two optional text fields name one option node
and check it admits. Nest `text` under 40 options and check it refuses
`operation-law-missing`, and nest integers 2000 deep and check the work budget
refuses. Chain 12 record levels of 4 fields each naming the next level and
check it admits within the budget. Put a leaf whose mode disagrees with its
field's pinned rounding beside too few leaves and beside a type naming a node
that is not in the graph, and check the refusals are `operation-law-missing`
and `operator-ineligible`, the leaf count settling before any leaf mode.

## Residual operation leaf sources (FR-038-AC-228 through FR-038-AC-231)

Use the current first-party catalog from `quire-verification-contracts` and
its current QSpec operation fixtures by reference from their authoritative
checkouts; do not copy those files into this repository. Start each case with
an admitted V2 package whose selected operation, lock definitions, source
occurrences, structural node ids, complete graph and package name agree. Assert that
the unchanged package admits through `CheckedPackageV2::read`. For each
mutation below, update affected references while retaining valid structural ids and package name, and assert that the adverse package
passes strict wire and graph checks before its named operation
check. Expected leaves and refusals are test-authored from the declared types
and catalog, never computed by asking the reader twice.

1. For AC-228, use `structural.eq` and `.ne` over an all-integer record and
   a record with one `Text[0, 8; nfc]` field. Independently author the empty
   and one-field leaf lists. Supply the first operand as a `reference` to a
   value node, a `reference` directly to the structural type, a `reference`
   to an expression node whose body root is an `application` with
   `result_type` equal to that node's `semantic_type`, and an option-typed
   `none` literal for an option comparison; keep each second operand the same
   declared type. For the application-bearing expression, use an admitted
   `quire.op.control.if` with structural-typed value branches and a body-root
   `result_type` equal to the structural type compared by the outer operation.
   Assert the exact admission or missing-leaf result of
   AC-228. Then replace only the outer operation's first-operand reference
   with an inline copy of the referenced application's body; assert the V2
   grammar rejects that argument and the public reader refuses `malformed_wire`
   at `arguments/0` before leaf checking. Mutate a declared type and refresh
   identities to show that
   the reader selects the new type and does not use stale first-operand leaves.
2. For AC-229, start from admitted structural comparison and
   `collection.contains` packages. Replace operand 0 separately with a
   well-shaped `binding`, `aggregate`, `frame`, `abstraction_relation` and
   `dependency_reference`. Record the earlier grammar/reference refusal when
   it applies; otherwise assert `ill_typed`/`operator-ineligible` at
   `arguments/0`, never admission through absent type resolution. Construct
   separately a family-fitting first operand whose required `operand:0` or
   `inner:0` type fails only after that earlier check, and assert the typed
   `operation.leaves` refusal with leaves empty and with one supplied entry.
   At a measured exact and one-less structural forwarding work limit, assert
   ordinary decision then typed Work, not an unresolved-type refusal.
3. For AC-230, exercise all seven catalogued `result_inner` identities. For
   each one, give an otherwise valid set-like result over pinned text exactly
   its empty-path text leaf and assert admission; remove it and assert
   `operation-law-missing`. Exercise a valid sequence result over the same
   text where that identity's result form permits one: no leaves admit and
   one supplied leaf refuses `operation-law-mismatch` at its first entry.
   Give a set-like result a shape whose element type fails to resolve only
   after earlier checks, and assert `operator-ineligible` at
   `operation.leaves` with both empty and supplied lists. The selected
   element adds no `inner` prefix; a wrong-prefixed supplied leaf refuses.
4. For AC-231, use genuine Float32 and Float64 type nodes with their current
   QSpec scalar and rounding shapes. In a separately identity-valid package,
   place each under a record, tuple, option and sequence compared by
   `structural.eq` and `.ne`, and under the element of a sequence used by
   `collection.contains`. Both empty leaves and an authored text leaf refuse
   `ill_typed`/`operator-ineligible` at `operation.leaves`. Change only the
   nested IEEE type to Integer, refresh all identities and assert admission
   with exactly empty leaves. A valid `collection.flatten` result of
   `sequence<Float32>` and `sequence<Float64>` admits with no leaves; a
   set, bag or ordered set of an IEEE-bearing element is never used as an
   admission control because QSpec FR-144 forbids its type.

## Exact operation leaves (FR-038-AC-44)

Over a record of two text fields and an option of text, the text type a
`text_bounds` domain binding `nfc`, supply the three derived leaves, each with
one catalogued `text_profile` law the lock selects and the `nfc` mode, and
check it admits; do the same over a tuple, and over a set of text with the
one empty path. Supply two leaves over an all-integer record, one leaf more than
the text leaves, and a leaf to an operation that names no leaf source, and
check each refuses `operation-law-mismatch` at the first extra leaf. Supply
three unrelated paths, a path without its `inner` segment, two leaves swapped
and a `position` segment for a `field` one, and check each refuses
`operation-law-mismatch` at that leaf's `path`. Give a leaf no law, two laws, a
law of another role and an uncatalogued definition, and check each refuses at
that leaf's `laws`. Run the lock without the selection and check
`operation-law-unselected` at the `definition`. Leave a leaf's mode out, give
it another kind, an uncatalogued value, and a catalogued value other than the
pin, and check `operation-mode-mismatch` at `mode`, `mode/kind` and
`mode/value`, then `operation-mode-type-mismatch` at `mode/value`. Make one
text type bind no profile and check `operator-ineligible`, with and without
leaves supplied. Apply `collection.flatten` to a `sequence` of text with one
leaf and with none, and `integer.div` with an unselected law and a leaf, and
check `operation-law-mismatch`, admission, and `operation-law-mismatch` ahead
of the unselected law. Chain 16 levels of 10 fields all
naming the next over text and supply one leaf, and check
`operation-law-missing` comes back at once; make nine of the ten fields
integers and check the one 16-segment path admits and one wrong segment refuses.
Compare the whole refusal code, cause and pointer with the expected one.

## Recursive compared types (FR-038-AC-70 through FR-038-AC-72)

Declare `Node { label: Text[0, 8; nfc]; next?: Node; }` and apply
`structural.eq` to two parameters of it with the `field:label` leaf and the
recursion leaf `field:next`, `inner`, `recursion:0` after it, and check it
admits, and with the text leaf alone and the recursion leaf alone and check each
refuses `operation-law-missing` at `operation.leaves`. Compare `Option<Node>`,
the mutually recursive `A` and `B` at `A` and at `B` in one package, `Two`,
`Tree2`, a declared tuple `Pair` and `Wrap` over `X` and `Y` with the leaves
FR-038-AC-70 lists, recursion leaves included, and check each admits; compare
a `List` of integers by `structural.eq` and `collection.contains` with `leaves`
empty and check both admit. Give `Wrap` its four text leaves alone, and then all
six and a further text leaf, and check `operation-law-missing` and
`operation-law-mismatch` at the seventh entry. Add a second text leaf after
`Node`'s two leaves and check `operation-law-mismatch` at `operation.leaves/2`.
Supply the recursion leaves FR-038-AC-71 lists (before the text leaf, a wrong
segment, a wrong `d`, a second one, one with a law, one with a mode, one at a
reentry of the integer `List`) and check the code, cause and pointer of each; bind no profile on a text type inside the cycle,
unselect a leaf law, and use `T` as an `Option` of itself, a `Sequence` of itself
and a record holding a field of such a type under a work limit of 1000, and check
`operator-ineligible`, `operation-law-unselected` and `operator-ineligible`
(never `incomplete`); check `R { x: Option<R> }` admits. Build the 20000-record
cycle with the byte, node, edge and work limits raised and decide it on a 256 KiB
thread, build the ten all-referencing records and check `incomplete` for `work`
under the default limits, and bisect the work limit of a ring of 12 records, compared with its
12 text leaves and its recursion leaf, to the exact work (FR-038-AC-72). Compare the whole refusal code, cause and pointer
with the expected one.

## QSL-shaped optional record fields (FR-038-AC-151 and FR-038-AC-152)

Implemented by IR-644 in `tests/it/checked_package_v2_recursive_leaves.rs`; the assertions below run as TC-048.

Build a checked package with the QSL-emitted `List` shape: its `next` field's
value is an aggregate containing one `optional` binding whose value references
an `Option<List>` node; the option node references `List`. Compare two `List`
values by `structural.eq`. With an integer field, assert admission with no
leaves. Replace it with selected-profile text and assert admission with the
text leaf followed by `["field:next", "inner", "recursion:0"]`; omit the
recursion leaf and assert `operation-law-missing` at `operation.leaves`. Add a
text sibling after `next` and assert its leaf follows the recursion leaf in
declaration order. Run a direct-reference-to-`Option<List>` control and check that it admits
with the same field and inner path.

For the same otherwise well-formed comparison, replace the `next` value in
fresh packages with an empty aggregate, two `optional` bindings, another
binding name, a non-binding member, and an `optional` binding that does not
reference an option type. Retain the package name and valid structural ids each time.
The flat body grammar admits each mutant. Assert that every one reaches the
operation check and refuses `ill_typed`/`operator-ineligible` at
`operation.leaves`, with no partial leaf derivation. An earlier grammar or graph refusal fails the test.

## Canonical encoding (FR-038-AC-74 through FR-038-AC-80)

For each positive fixture, encode the complete V2 envelope, graph, lock, source-map entries, capability report and diagnostics through `quire_canonical::to_vec`, comparing independently authored canonical JSON bytes for the same complete typed values. Include declared and derived structural ids, recursion groups, all JSON body kinds, non-ASCII strings and supported numeric bounds. A 100000-level in-memory body is encoded and disposed iteratively on a 256 KiB thread stack. At exact and one-byte-below limits, require success and typed byte refusal respectively. A package document containing noncanonical numeric spelling or out-of-order UTF-8 object members refuses `noncanonical_wire` before body and graph checks. No preimage or digest is encoded or compared.

For FR-038-AC-76, compare one independent canonical-byte oracle per public
`CheckedPackageLockV2`, `CheckedSourceMapEntry`, `CheckedCapability`,
`CheckedNodeId` and `CheckedDiagnosticsV2` value from every in-repo positive
fixture. The `CheckedNodeId` oracle includes a nested structural id in `path`
and its exact producer-assigned ordinal; diagnostics include nested object
and array `details` and non-empty `loci`. Do not construct or encode the
removed `CheckedSemanticId` (FR-038-AC-80).

## Structural identity and canonical byte controls (FR-038-AC-89 through FR-038-AC-95)

Read each current QSpec positive package directly from the checkout named by `QUIRE_SPECIFICATION_DIR`. Compare the admitted package name, structural node ids, graph order, complete bodies and owners with its published JSON. A missing checkout, selected domain document, fixture or selected identity fails the run. The reader must not derive an expected node id from a body or expose a package digest. Source-owner distinctions remain visible in their structural package/path components.

Independently construct canonical JSON bytes for a complete V2 envelope and compare with the public writer's bytes. Mutate one semantic body member and then one source-map member separately; each changes only the corresponding bytes and the reader applies its independent body and graph checks. The historical digest and preimage assertions of AC-90 and AC-92 are withdrawn.

For AC-93, supply a selected model document with the authored inexact numeric tokens at `/package/count` and require `noncanonical_wire` with its document pointer. Canonical tokens at the same location are not refused for that cause. For AC-94, a selected document exactly at the byte ceiling admits and one byte below returns typed `incomplete`; repeat with a one-unit work ceiling. For AC-95, drive the public bounded canonical encoder and lowerer at exact and one-byte-below limits using complete typed values. Require typed byte-limit failure with no partial package; no identity preimage or hash is computed.

## Inexact numbers (FR-038-AC-109 through FR-038-AC-111)

FR-038-AC-109 through FR-038-AC-111 are implemented and verified in
`tests/it/checked_package_v2_model_members.rs` and
`tests/it/checked_package_v2_canonical_encoding.rs`, and the existing FR-038-AC-93
tests there carry the cause `inexact-integer`; the `1e400` and `-1e400` mapping
of FR-038-AC-110 (IR-555) is verified through `quire-canonical`'s
`NumberOutOfRange`, `1e-400` and `-1e-400`, which read as zero with their
text kept, through the rule on that text, and the first-fault rule (QSL
FR-056, QSL #625) in `tc_048_the_first_reader_fault_decides_when_faults_coexist`
(IR-573). The causes are those
of quire-specification:FR-271 and FR-272. Read a package that selects a model
document holding `0.1000000000000000000001`, `9007199254740993.5`,
`-0.1000000000000000000001`, `4.9e-324` and `1e-400` at `/package/ratio`, and check `noncanonical_wire` at
`/semantic_graph/nodes/<i>/owner/identity` with `document_pointer` `/package/ratio` and
cause `inexact-number`; repeat with `0.1`, `0.5`, `1.5`, `-0.25`, `5e-324`,
`2.5e-10`, `1.0`, `-0` and `1e2` and check none refuses for it; repeat with the tie texts `1125899906842624.2`, `1500000000000000.2` and `2.9802322387695312e-8` and check none refuses, and with `1125899906842624.3`, `1500000000000000.3` and `2.9802322387695313e-8` and check each refuses `inexact-number`; read two documents differing only in
`0.1` and `0.1000000000000000000001` and check the second refuses
(FR-038-AC-109). Repeat the numbers of FR-038-AC-93 (including the integer value type
upper bound), `9007199254740993.0`, `1e400` and `-1e400` and check cause
`inexact-integer` and `noncanonical_wire`; check `9007199254740992.5` refuses
`inexact-number` and the admitted whole numbers still admit; put an inexact
number and a whole number past 2^53 in one document in both orders and check the
first in document order is named with its own cause; put a number past the
double range after an inexact number (`{"b":0.1000000000000000000001,"a":[1e400]}`,
`{"b":9007199254740993,"a":[1e400]}`) and check the out-of-range number is named
(`/a/0`, `inexact-integer`); read `{"a":1,"a":2,"n":1e400}` and `[1e400` and
check `/n` and `/0` are named, and `[{"a":1,"a":2},1e400]` and
`{"a":1,"a":2,"n":1e-400}` and `[1e400,"<0xFF>"]` and check `malformed_wire`/`malformed-json`, and
`[1e400,"\ud800"]` and check `/0`, the first fault `read` returns deciding
(FR-038-AC-110). Read a
package document holding `0.1000000000000000000001` and `9007199254740993.5` in a
node body and check `noncanonical_wire` with no pointer, no `document_pointer`
and no cause, and `0.1`, `1.2793061557049685`, `1.2106592671318679` and `1.3567384036451073` not refused for it; read the manifest of the crate that holds the
reader and check it declares `serde_json` with the feature `float_roundtrip` (a
source-level check; the package document is the read that goes through
`serde_json`) (FR-038-AC-111).

## Dependency references (FR-038-AC-35 through FR-038-AC-38)

Supply an admitted dependency package by its `name` and declare a function node in it. Read a local function-call application with a `dependency_reference {package, node}` callee naming that package and the exact structural function id; the remote target is not listed as a local dependency. Verify the term is retained verbatim and the call admits. Listing the remote target locally refuses `invalid_semantic_graph`. Change package and node separately: a selected but foreign target, missing selection, missing node, undeclared node and wrong declaration each get their independent join refusal. Malformed shapes, absent members and extra members refuse at the wire stage. Exercise the term in another argument position, aggregate member and body root, plus record, tuple, reference and bounded scalar parameter/result types. Compare exact code, cause and pointer without any rekey step.

## Enum operand family (FR-038-AC-42)

Verify QSpec FR-322's rule that an enum is `ordered_enum` when its selected declaration is ordered and `enum` otherwise, and FR-141-AC-5, in
`tests/it/checked_package_v2_enum_order.rs`. Build a package holding an enum
declaration, its member nodes and a `binary` application of one enum
operation over two operands of that enum, as member literals and as
`value`/`parameter` nodes. Apply `quire.op.enum.lt`, `le`, `gt` and `ge` to an
ordered enum, one listing its members in declaration order and one whose
declaration order is also sorted, and to an unordered enum, one listing its
members sorted. Apply `quire.op.enum.eq` and `ne` to each. Apply `lt` to
operands of two different ordered enums.

Expected: the four ordering operations admit over an ordered enum however its
members are listed; they refuse `ill_typed`/`operator-ineligible` at the first
argument of the application over an unordered enum; `eq` and `ne` admit over
either; `lt` over two different enums refuses `ill_typed`/`operator-ineligible`
at the second argument.

## Selected variant types (FR-038-AC-236 through FR-038-AC-238)

Verify, through `CheckedPackageV2::read` and the tests in
`tests/it/checked_package_v2_model_members.rs` and
`tests/it/checked_package_v2_enum_order.rs`, that a selected Semantic IR model
declaration of a nonempty variant type is admitted and typed exactly. Author a
selected domain package document holding an object type with a variant type
declaration, and a package that selects it.

1. For AC-236, admit a variant declaration whose self-typed node names the
   object type as its model owner. Use the variant as a field, a parameter and a
   result type, and under an optional field and a bounded sequence field; require
   the typed member of each field to name the variant. Remove the owner and
   require `malformed_wire` at the node; name an absent declaration, and then
   a different existing declaration, as the owner and require
   `missing_declaration`/`missing-selection` at the declaration's `node_id`.
2. For AC-237, admit an `enum_value` for each case name in a set holding a name
   with a space, a digit prefix and a non-ASCII character. Admit the same case
   under two different variant declarations and a second case under one
   declaration, and require distinct declared structural value ids. Apply `enum.eq`
   and `enum.ne` to two values of the variant and require admission; apply
   `enum.lt` and `structural.eq` and require `ill_typed`/`operator-ineligible`
   at the first argument.
3. For AC-238, declare no case and require `invalid_model_binding`/
   `malformed-declaration` at the selected model row. Name an absent case in an
   `enum_value` and require `invalid_semantic_graph` at its `case`. Supply a selected document with a different case set and require the affected case join to refuse at its exact member; no staleness refusal is expected. Read an ordinary scalar enum whose cases
   include a non-identifier spelling and require `invalid_semantic_graph`.

Expected: each outcome above holds with the stated code, cause and pointer. The
criteria state these outcomes except two controls the tests add from the
reader's general rules: an absent `owner` refuses `malformed_wire` at the
node, and `enum.lt` and `structural.eq` over the variant refuse
`ill_typed`/`operator-ineligible` at the first argument. Those two are extra
test controls and are not required by the criterion text.

## Protocol node forms (FR-038-AC-281 through FR-038-AC-292; FR-038-AC-324 through FR-038-AC-330)

Use QSpec FR-464, FR-465 and FR-466 and the authored tables in TC-946, TC-947
and TC-948 as the current independent expected outcomes. Build each case into
a V2 package with valid structural node ids and complete bodies. Once QSpec publishes machine-readable
protocol vectors and regenerated fixtures, run them from a
`quire-specification` checkout named by `QUIRE_SPECIFICATION_DIR`. No fixture,
vector or schema is copied into this repository.

1. Admit the seven non-carrier protocol forms and each of the twenty
   `profile_operator` names under the selected
   `quire.protocol.finite-global/v1` row. Refuse an unknown operator, a missing
   or wrong law, a member whose kind is not `profile_operator`, a misplaced
   application and a node with another operator's form. Select
   `quire.protocol.complete/v1` in a separate lock and verify it does not
   satisfy a law naming `finite-global/v1`, or the reverse. Omit the
   `temporal_profile` selection for `await` and `compensate` and require
   `missing_declaration`/`missing-selection` at each node.
2. For each form, admit its fixed named arguments in order, including empty
   sets and absent optionals where allowed. Refuse a missing, extra,
   reordered, misspelled or wrongly shaped binding, a repeated or misordered
   set or sorted record, and a nested protocol application. Reorder an authored `sequence` and an ordered capture list and require changed complete bodies; reformat source and reorder role or other set-backed declarations and require their canonical order and complete bodies to match the authored meaning.
3. Admit valid declaration paths and one-clause membership. Refuse a wrong
   prefix, duplicate or unattached member, double-held control, missing or
   duplicate queue and wrong-clause queue. For each protocol-family form with
   a local body target, omit one existing target from `dependencies` while retaining its structural ids; require `invalid_semantic_graph` at
   that node's `/dependencies` in the graph stage, before a separate
   protocol-body defect. Add an extra dependency and require the same
   graph-stage refusal. Separately name a nonexistent local target in both
   the body and `dependencies`, updating references, and require
   `missing_declaration`/`missing-name` at the target. Compare full
   code, cause, pointer and node locus, using QSpec TC-946 C13/C13a,
   A8/A8a, A9/A9a and OM-09 as independent clause vectors
   (FR-038-AC-285/327).
4. Exercise static and replicated roles, channels with each delivery and
   ordering premise, literal and symbolic queues, finish and terminal,
   thirteen control operators and compensation with `commit` present and
   absent. For each, pair an admitted case with the local adverse forms,
   bounds, type and label cases of TC-947 and TC-948. For compensation, admit
   a commit in one choice case or a non-all parallel branch before the forward
   effect, and refuse a commit that dominates that effect; compare the
   `commit-precedes-effect` reason only for the dominating case.
   For a `terminal when` condition under an `on each` activation, read the
   protocol `over` observation as QSpec TC-947 TM11 does and require admission;
   repeat under `on origin` and require admission. Under `on each`, replace
   that read in turn with the protocol capture of TM10 and the activation
   binder of TM12; require `missing_declaration`/`missing-name` at the terminal
   node with stage `terminal` for each. Under `on origin`, read an initialized
   protocol capture as in TM13 and require admission. Construct fresh,
   identity-consistent packages for these variants and compare the complete
   refusal, including its stage and locus (FR-038-AC-326).
5. Exercise a causal cycle, independent parallel controls, zero-progress and
   progressing repeats; then a binder at each stage, forbidden model-state
   read, inaccessible branch binder and decision fact of another role or not
   individually visible. In each `choice` and `repeat` `visible` entry and
   guard, read the protocol input `over` directly and require
   `ill_typed`/`operator-ineligible` at the read, naming the slot, before
   binder-scope and decision-fact checks. Read a protocol activation
   binder's field that is available at the decision and require
   `wrong_snapshot`/`wrong-observation` with `ineligible-kind` and the retained
   binder and field. In each of those four slots, read the activation binder
   `opened` bare inside a Boolean-rooted expression; repeat with a bare
   receive record binder. Require `ill_typed`/`operator-ineligible` at the
   bare-read node with the slot detail after scope and model-state checks,
   without a field-less `wrong_snapshot` detail. In both operand orders, pair
   the bare activation read with an in-scope ineligible `send` field read and
   require the bare-read refusal first. Read a finish
   binder in each `choice` `visible` entry, `choice` guard, `repeat` `visible`
   entry and `repeat` guard; require `missing_declaration`/`missing-name` at
   the read with `read` `stage` in all four slots. Read a compensation-template
   binder from a run slot and require `missing_declaration`/`missing-name`
   with `read` `foreign`. In one slot pair an `over` read with a binder read
   and require the `over` refusal; pair an out-of-scope binder read with an in-scope
   ineligible binder read and require the out-of-scope refusal. Use QSpec
   TC-948 DF6b and DF7d..DF7h as the independent oracle. Assert the specified
   first refusal and retained stage, owner, slot and reason details rather
   than only a refusal code (FR-038-AC-291/292; QSpec FR-466-AC-14/16).
6. Combine defects in state and protocol, temporal and protocol,
   abstraction and protocol, refinement and protocol, protocol and operation,
   and multiple protocol nodes. Require the FR-322 body-step order and
   FR-464's placement-through-catalog-join order, with ascending structural-id order
   within each stage. Keep the existing `protocol_reference` carrier cases
   green.
7. From an admitted clause with `activation on each (opened)` and one required
   temporal clause, inspect the derived activation relation: it contains the
   clause key, `each` and the trigger key, and the required clause key. Repeat
   with `origin`, no requirement, two source `requires temporal` declarations
   normalized by the producer to one canonical wire set entry, a second
   distinct requirement and an unrelated temporal clause with the same
   clock; require respectively one origin row, no row, one row, two
   structural-id-ordered rows and no extra row. In a separate identity-consistent
   wire mutation, duplicate the same `requirements` set entry; require
   `invalid_semantic_graph` under the clause's set rule and no relation.
   Omit an existing `requirements` or `each` trigger target from
   `dependencies` in separate packages; require graph-stage
   `invalid_semantic_graph` at the clause's `/dependencies` and no
   exposed relation. Separately replace the body target with a nonexistent
   local id, list it in `dependencies`, and require
   `missing_declaration`/`missing-name` at the target with no relation
   (FR-038-AC-324/327; QSpec TC-946 A8/A8a/A9/A9a).
8. Use QSpec TC-310's `CC-01`, `CC-03` to `CC-05`, `CC-08` and `CC-11`
   catalog cases from the selected checkout, updating references and dependencies,
   then comparing complete package values and provenance carriers as each vector requires. Admit
   the complete catalog with separate `fork` and `join` classes on one
   parallel node, including a directly selected dependency's abstract
   protocol. Refuse a foreign or omitted class, wrong node association or
   binder type and an absent source-default memory class at QSpec's code,
   cause and catalog locus. For `CC-11`, require the protocol-body defect
   before the catalog join and the refinement-row defect before both. For
   `CC-10`, two request memory choices leave the admitted catalog and
   complete package value equal (FR-038-AC-325).
9. Build an independent positive package through the public typed writer:
   select concrete model package A with its canonical `model_root` R, owned
   object-type node O and protocol clause C whose `owner_model` is O; also
   select model package B with a different root and object-type node. Put R
   in the refinement's concrete `semantic_type`, not O. Give C a checked
   control class and omit C from the refinement's `steps`, while placing its
   class in the complete concrete catalog. Require public strict read and
   typed read/write/read to admit with the same keys, complete package value,
   root, owner and catalog (FR-064-AC-1/AC-5, FR-038-AC-325). The vector
   SHALL fail if the reader equates R and O or reconstructs only step rows.
10. Update every affected reference and dependency after each owner mutation, then compare complete package values. First retarget C to selected B's object-type node:
    this is a valid reader owner edge, so require admission with a complete
    B catalog and require C's class to be absent from A's catalog. A source
    producer given C declared over A SHALL still emit O; its candidate B
    edge fails producer conformance, a separate check from reader admission.
    In independent packages, retarget C to a selected `state_machine`,
    `process`, `record_value_type`, relation or `model_root` node, then to a
    source-local declaration or a missing local target. Require the typed
    refusal at `owner_model` for each wrong-kind or missing target. A separate
    genuinely unselected `model`/`object_type` target SHALL refuse
    `missing_declaration`/`missing-selection` at its own `node_id` in the
    earlier graph owner join. Pair the wrong-kind edge with earlier refinement
    row and protocol member defects, then with a later same-clause slot defect,
    to verify FR-464-AC-20's phase order. The reader SHALL not infer or
    cross-check ownership from the source `over` parameter.
11. Keeping the positive package's owner join and protocol body valid,
    mutate the concrete catalog separately to omit C's class, add B's
    foreign class, duplicate or reorder an entry, change C's class node or
    binder signature, omit a catalog required by a protocol or memory
    class, and add a catalog to a model-only refinement without those
    classes. Require respectively `invalid_semantic_graph` at the concrete
    array for omission, that code at the extra entry, the earlier refinement
    canonical-order refusal for duplicate or disorder, `invalid_model_binding`/
    `malformed-declaration` at the changed entry, and `invalid_semantic_graph`
    at the catalog member for missing or forbidden presence. Include an
    admitted abstract clause from a directly selected dependency and an
    owned `parallel` with separate `fork` and `join` classes so neither side
    is reduced to step-row membership. Pair a catalog defect in an otherwise structurally valid package with
    an independent protocol-body defect and then an earlier refinement-row
    defect: require the row defect before the body defect, and the body
    defect before the class-catalog graph join (FR-064-AC-5,
    FR-038-AC-284/AC-325; QSpec TC-310/TC-946). Consume QSpec's published
    vectors from the selected checkout; do not copy them into this repo.
12. For owned source `parallel` nodes, derive separate valid catalogs with
    single source defaults `sc`, `tso` and `ra`. Require a node-free, binder-free
    `memory drain` entry with `drain: store-buffer-flush` for each default;
    require `memory flush` with `memory_model: tso` only for `tso`. Keep the
    memory class when no refinement step row names it. In separate packages with valid structural ids, omit a required memory class, add `memory flush` under `sc` or
    `ra`, change the drain model or drain member, and duplicate or reorder a
    memory entry. Compare the exact QSpec FR-375 code, cause and catalog
    pointer, including earlier refinement canonical-order refusal for
    duplicate or disorder and later protocol catalog-join refusal for an
    inventory mismatch. Pair a memory catalog mismatch with an independent
    protocol-body defect to require the body defect first. Two analysis
    requests with different memory overrides SHALL leave the admitted
    catalog and complete package value equal (FR-038-AC-325; QSpec TC-310).
    Also retain QSpec TC-310 `CC-12`'s one-subject multi-default case with
    distinct `memory drain` entries for `sc`, `tso` and `ra` and a `tso`
    `memory flush` entry. Require admission with the three drains in
    `ra`, `sc`, `tso` order followed by the flush, including when two
    `parallel` controls select the same default; the repeated model contributes
    one entry. In separate structurally valid packages, refuse a duplicate (`memory drain`, `memory_model`)
    pair or a declared class spelling at the later duplicate catalog entry,
    and a reordered pair at the first out-of-order entry, in the earlier
    refinement canonical-order step. Omit a distinct default or add an
    unselected one and require the later catalog graph-join refusal.
13. Build public-writer packages for selected concrete and abstract clauses
    activated `on each (x: T)` with non-record `T` and with record `T`.
    Require `activate <protocol>` to name the clause node on the concrete
    side and its clause `RefinementKey` on the abstract side, with `[T]` for
    non-record `T` and one binder type per effective record field. A clause
    activated `on origin` contributes no activate entry. In separate packages with valid structural ids, substitute the trigger node or another clause node for the selected clause,
    a wrong binder type, reversed binder order, a missing activate entry
    and an extra origin entry. Require respectively
    `invalid_model_binding`/`malformed-declaration` at the entry for the
    wrong node or binder, `invalid_semantic_graph` at the catalog array for
    omission, and that code at the extra entry for origin (FR-038-AC-325;
    QSpec FR-375-AC-27, TC-310).
14. For record activation binders, build `Base { a: Boolean; b: Int }`,
    `Mid { c: Int }` and `Leaf { d: Boolean }` with the source inheritance
    chain `Base`, `Mid`, `Leaf`; require binder type keys corresponding to
    `a`, `b`, `c`, `d` in that order. Reverse `Base`'s field declaration order
    and require `b`, `a`, `c`, `d`. For `P { p: Boolean }`,
    `Q { q: Int }` and `Both { r: Boolean }`, compare source
    supertypes `P`, `Q` against `Q`, `P` and require `p`, `q`, `r` against
    `q`, `p`, `r`, irrespective of declaration node-key or IR `types` order.
    In a diamond `Top { t }`, `L { l }`, `R { r }`, `Bot { z }`, require
    `t`, `l`, `r`, `z` with `t` once. Redefine `Base::a` in `Leaf` and
    require the new type of `a` at its inherited position; in a diamond,
    redefine `Top::t` through `L` and require the replacement once at `t`'s
    position. A same-name field inherited without an explicit redefinition
    refuses `ambiguous_declaration`/`ambiguous-name` at the catalog entry's
    `binder_types` pointer
    (`/semantic_graph/nodes/{n}/body/class_catalog/{side}/{i}/binder_types`)
    when its binder record is requested. A wrong or
    reordered binder type in an otherwise unambiguous case refuses
    `invalid_model_binding`/`malformed-declaration` at that entry
    (FR-038-AC-325; QSpec FR-208-AC-18, FR-154-AC-32,
    FR-432-AC-16 through FR-432-AC-19).
15. With a selected abstract model that has an operation but no protocol
    clause, build a protocol refinement whose complete abstract catalog
    contains that operation class although no step row names it. Require
    admission; omit the class and require `invalid_semantic_graph` at the
    abstract catalog array. Add a class from another selected model and
    require `invalid_semantic_graph` at the extra entry. Keep the concrete
    protocol body valid so the catalog join is reached (FR-038-AC-328;
    QSpec FR-375 checked class catalog).
16. Build two admitted protocol clauses with different package-local
    declaration paths, including a clause path longer than one segment.
    Include a role and a nested control whose declaration path contains
    `run` and two ancestor control names. Require the complete alias-free
    FR-432 source class spellings for each clause and member, using the
    complete clause path and the member's authored name while omitting the
    FR-464 fixed kind segment and intervening control ancestors. Include
    QSL-authored declaration shapes `[P, finish, Done]` and
    `[P, run, Main, Par]` with classes `P::Done`, `fork P::Par` and
    `join P::Par`; require admission with their node associations. Construct separate structurally valid packages with a class spelling that drops a clause-path
    segment, uses the other clause's or member's name, or includes the
    fixed kind or an ancestor control name; require
    `invalid_semantic_graph` at the missing class's array or the extra
    entry as applicable (FR-038-AC-329; QSpec FR-322 declaration paths,
    FR-464 qualified names, FR-432 class spellings).
17. Build an admitted compensation template with its FR-432 `cend` class
    and no step row naming that class. Require its template-node association
    in the complete concrete catalog and admission. Construct separate structurally valid packages
    omitting `cend`, adding it under another template node, or adding it to
    a subject with no such template; require respectively
    `invalid_semantic_graph` at the concrete catalog array,
    `invalid_model_binding`/`malformed-declaration` at the changed entry,
    and `invalid_semantic_graph` at the extra entry. Also exercise the
    selected abstract protocol's `cend` class in its abstract catalog
    (FR-038-AC-330; QSpec FR-432 `cend`).

Expected: the positive packages admit with their authored keys, every adverse
vector refuses with the QSpec code, cause and locus, and no runtime workflow
instance, delivery or clock observation is required to read the package.

## Nested repeat progress (FR-038-AC-344 through FR-038-AC-356)

Construct each case as an independently keyed, otherwise valid V2 package and
read it through the public strict reader. Use QSpec FR-464 and TC-946 R13-R24
as the external oracle, without embedding their vectors here. For every
refusal compare the typed code, cause, control key and RFC 6901 locus;
compare the selected-profile detail for a repeat-progress refusal. For every
incomplete result compare limit, consumed work and the nested-control locus.
Keep the enclosing `repeat`'s body free of other
steps except where the control explicitly calls for one.

| Control | Reader assertion |
| --- | --- |
| Nested body is an `attempt`; nested guard is `true`, then a two-visible-atom tautology, each with no maximum and with `maximum: 1` | Admit; the inner `repeat` necessarily steps and supplies the outer body's only step (R13/R14). |
| Same stepping nested body with `maximum: 0`; test both `true` and a one-atom tautology | Refuse `unsupported_construct`/`declaration-form` at the outer `repeat`; the maximum prevents body entry even though the guard exit is infeasible (R15). Still charge the nested valuation before that refusal. |
| Same nested body with each of `a`, `not a`, `a and not a`, `false`, `a or b`, with `visible` listing the used atoms and both absent and positive maxima | Refuse at the outer `repeat` because each guard has a false valuation; the result shall not depend on the maximum or an `exhausted` step (R16). |
| Add a stepping `sequence` sibling on either side of the nested control; put the nested control under a `parallel` join or an `await` success branch | Admit with a feasible nested exit, because the sibling, `parallel` or `await` supplies the necessary step (R17). |
| Put the nested control in one `choice` case and an `attempt` in the other | Admit with an infeasible nested exit; refuse at the outer `repeat` with a feasible exit, since every case must step (R19). |
| Nested body is only a `check`; vary the nested maximum and add an `exhausted` `attempt` in one variant | Refuse at the first nonprogressing `repeat` in tree order, the outer one; `exhausted` supplies no progress (R18). |
| List one `eq(a,b)` expression and use it on both sides of an `or`/`not` tautology; then list `eq(a,b)` and `eq(b,a)` separately and cross the two in the guard | Admit the equal-node case and refuse the reversed-operand case at the outer `repeat`, proving atom identity is node identity without commutative normalization (R20). |
| List a composite Boolean expression `E`, then its component atoms, and use `E or not a or not b` | Refuse at the outer `repeat`: the listed `E` remains an independent atom, not its expanded expression (R22). |
| Use `true` as a referenced Boolean literal, then use a listed atom `a` with a referenced `true` in `a or true` | Admit both; the referenced literal is a constant and contributes no atom (R23). |
| Nest `repeat max 1 while true` around an `attempt`, with an `exhausted` `check`, as the outer body's only step | Admit on every outer iteration; entering from outside resets the inner counter (R24). |

Add work probes that observe the atom order without relying on the final
admit/refuse verdict alone (FR-038-AC-352; QSpec FR-464-AC-22). First, add
one eligible but unread `visible` entry to a nested two-atom tautology.
Its valuation charges SHALL rise from four to eight; omitting unread entries
from atom collection would leave the count at four. Next, keep one visible
atom `a` and use two distinct nonconnective Boolean expression nodes `E1`
and `E2` in the guard, with every field fact they read individually listed
and eligible. Neither expression is itself listed. With *m* visible entries,
the tautology `E1 or not E1 or E2` SHALL charge `2^(m+2)` valuations;
replacing `E2` with another occurrence of the `E1` node SHALL charge
`2^(m+1)`. These counts show that both unlisted expressions are maximal
atoms, their operands are not descended into, and equal node ids collapse
to one atom. For the order probe, independently derive each atom's
ordinal from visible-set order followed by a last-operand-first depth-first
guard walk. Choose a guard false only when atom `a` and `E1` are true and
`E2` is false, such as `not a or (not E1 or E2)`, and assert that the first
false valuation is charged at the one-based little-endian index calculated
from those ordinals. Reorder the inner disjunction as `E2 or not E1`,
keeping the same truth function and node identities but changing discovery
order, and require a different first-false charge count.
Separately, with two visible entries in canonical set order, use
`not a or b`: it SHALL stop after two valuations when `a` is the first atom,
whereas reversing their atom order would take three. At each expected
first-false boundary, set the work limit to the measured base consumption
plus that many valuation charges and require the progress refusal; one
less SHALL return incomplete at the nested control. This distinguishes
visible order, the visible-before-unlisted rule, last-operand-first discovery
and little-endian valuation order.

Measure reader work with a nonbinding limit for the two-atom tautology to get
*V*, total consumption through its fourth valuation charge, and *T*, total
package consumption. A limit of *V* minus 1 SHALL return incomplete
`stage_limit_exceeded`/`work-budget-exceeded` at the inner `repeat`, without
an outer progress refusal. A limit of *V* SHALL pass that valuation and
return the incomplete result at the next later package work charge, located
at that later charged value rather than the inner `repeat`;
*T* SHALL admit. Independently, `a or b` SHALL charge exactly its all-false
valuation before the progress refusal. Repeat the maximum-zero control with a
limit denying its first valuation and require the incomplete result before
its progress refusal. Compare a standalone `repeat` with 30 listed guard
atoms to the same package with `true` as its guard: their total work SHALL
be equal, proving no standalone valuation enumeration (R21).

Pair a causal cycle with a nested guard whose next valuation would exceed
work, and with a nonprogressing outer `repeat`; require the cycle first.
Remove the cycle, retain the denied valuation and nonprogressing outer
control, and require incomplete before progress refusal. Give two outer
repeats nonprogressing bodies and require the first one in control-tree order
after all valuation checks complete (FR-038-AC-356; TC-946 O5/O7).

## Refusal and limit locations (FR-038-AC-24 through FR-038-AC-26)

For every refusal above, compare the whole refusal, including its pointer,
with the expected one. Check that each pointer resolves in the mutated
document. Add an unknown member whose name holds `~` and `/` at the top
level, on a node and on a locked source, and repeat a top-level member whose
name holds both. Dispatch the unknown versions again and read the version
each refusal carries. For each one-over limit, compare the pointer the
`incomplete` outcome carries and check that it resolves.

From an otherwise admitted V2 package with node 0's first occurrence role
`generated`, replace only that role value with the literal string `unknown field`
and read the canonical bytes. Require `malformed_wire` at
`/semantic_graph/nodes/0/occurrences/0/role`, with no admitted package. In
separate mutations of the same admitted package, add a genuine extra top-level
member and change node 0's `dependencies` array to a string. Require
`unknown_member` at the extra member and `malformed_wire` at
`/semantic_graph/nodes/0/dependencies`, respectively; each pointer must
resolve in its own mutated document. The role value's resemblance to decoder
error text cannot change its code, and these controls retain the ordinary
closed-schema code and pointer precedence.

Each refusal points at exactly the value its mutation changed, with `~`
escaped as `~0`, `/` as `~1`, and array elements by index. A missing member
is located at the object that lacks it. Malformed JSON and non-canonical
bytes carry no pointer. Every refusal code is unchanged. Each
`unknown_contract_version` refusal carries the exact string read, including
the empty one. Each one-over limit other than the byte limit names the value
whose charge failed, and that pointer resolves; the byte limit names none.

## Flat wire and no depth limit (FR-038-AC-114 through FR-038-AC-118)

Implemented (IR-495 code) in `tests/it/checked_package_v2_flat_wire.rs`, and AC-118
in `tests/conformance_qspec/main.rs`. In a fresh copy of a self-built package, place each of
`quire.op.function.call`, `quire.op.state.clause` and an application of the
`temporal`, `temporal_formula` and `temporal_fairness` classes inside an
application's `arguments`, as an `aggregate` member and as a `binding` value of
a node body, and read each; put an `aggregate` inside a Group's members, a
`binding` of an `aggregate` inside an `aggregate`'s members, a `binding` whose
value is a `binding`, a `binding` and a Tuple inside a Tuple's members, and a
`binding` as a body root; then write each of those bodies with its composite
subterm as its own node reached by `reference` (AC-114). Nest a `case`
application inside another term; put a non-`case` application at the body root
of a node its class does not place it in; put a `temporal_formula` application
with a nested `case` argument, an aggregate of a `temporal_formula` and then a
`case` application, and an application of another class in a diagnostic's
`details` term, and in a node body a nested `case` at `arguments/0` beside a
nested call at `arguments/1` and the two swapped (AC-115). Leave the authored structural `node_id` unchanged in a package whose body holds a nested
non-`case` application, and in one whose body holds a nested `case`, and read the
first again with the body flattened (AC-116). Scan the source files under
`crates/quire-contract-model/src/checked_package/` (not the v1 modules or the
manifest) for `MAXIMUM_DEPTH`, any `MAX_*DEPTH`, `stacker`, `serde_stacker` and
`on_stack_for`, and check `CheckedPackageReadLimits` and `CheckedPackageLimit` for
a depth member and variant; build a chain of 100000 nodes each referencing the
previous, read it on a thread whose stack is 256 KiB under every limit sized so
that none decides the outcome, lower its last node, and compare its JSON nesting
depth with that of a one-level package; read an otherwise canonical document whose
node body nests a term 300 levels deep on a 256 KiB thread, and the same document
nested 20 levels deep and nested 41 to 61 aggregates deep (up to 126 JSON levels,
within the parse's limit; a `details` term nested 50 deep too) in a debug build
(AC-117). Run `make conformance-qspec` and read the harness's
expected-failure list (AC-118).

Expected: each nested non-`case` application refuses `malformed_wire` at the
nested application, each Group and binding misplacement at that value, and each
flattened form is not refused `malformed_wire`; the nested `case` and the
misplaced body root refuse `ill_typed`/`operator-ineligible` at the pointer
AC-115 gives, the `details` and node-body cases at the first construct in document
pre-order, and the `details` application of another class `malformed_wire`; the
malformed non-`case` package refuses `malformed_wire`, the nested `case` package
`ill_typed`/`operator-ineligible` at the `operator`, and the flattened form
proceeds to independent graph and body checks; the scan finds none of the names and no depth member or
variant, and the 100000-node chain is admitted and lowered with no outcome naming a
depth and no stack overflow; the 300-level document refuses `malformed_wire` with no
pointer at the strict parse and the 20-level one, and each one within the parse's
limit, at the first value outside the body grammar, with no stack overflow; and each of the five
`body_grammar_mutations` refuses `malformed_wire` with the expected-failure list
empty of them.

## Reader peak memory (FR-038-AC-214)

CODE-GATED for IR-575: the isolated 2026-10-10 measurement passed; final repository
gates are pending. In a separate fixture-generator process, build the
same in-repo canonical 100000-node reference chain as AC-117 and save its bytes
in an owned temporary file. In each of three fresh Linux x86_64 reader
processes, load those bytes and prepare the selected-model evidence before
taking baseline VmRSS and VmHWM. If pre-read VmHWM differs from pre-read VmRSS,
record the invalid setup and retry in a fresh process; do not count that run
among the three. Build the reader process with the repository's debug
`cargo test --locked --test` profile and Rust's default System allocator.
For a valid run, start a 2 ms VmRSS sampler and call only
`CheckedPackageV2::read` with limits that
do not decide the outcome, stop the sampler on return and read VmHWM before
dropping the admitted package. Record the input byte count, admitted node count,
pre-call RSS/HWM, sampled peak, post-call RSS/HWM, read time, Linux platform,
compiler version and allocator. For each valid process, compute additional
peak resident memory as `max(sampled peak VmRSS, post-read VmHWM) - pre-read VmRSS`.
Fixture generation, process startup, evidence
construction and lowering are outside the read interval.

Inspect the production read path for direct typed decode and the absence of a
full-document `serde_json::Value` tree or duplicate full-document canonical
byte buffer; `Value` terms inside typed wire nodes are allowed. Run the
existing real-reader adverse tests for duplicate members, noncanonical bytes,
unknown members, refusal precedence and pointers, identities and source-map
content, and compare their exact existing assertions. Keep AC-117's separate
256 KiB-stack admission and lowering test for this same chain.

Expected: all three calls admit the full package; each additional peak is at
most 1.50 GiB (1572864 KiB). Direct decoding preserves the named refusal,
canonicality, identity, source-map and typed-result behavior. An allocation
from input/evidence or lowering cannot be counted as reader use. Requiring
pre-read VmHWM to equal pre-read VmRSS prevents an earlier setup peak from
masking a transient read peak that the sampler misses; post-read VmHWM then
catches that peak even when the sampler does not.

Measured on Linux 6.17.0-20-generic x86_64 with rustc 1.98.1, debug profile
and the default System allocator: the separately generated fixture was
176923194 bytes and each fresh reader admitted 100022 nodes. All three had
pre-read VmHWM equal to VmRSS; their additional peaks were 695712, 695712 and
695776 KiB. The criterion-backed measurement source is the explicit
`ir575_reader_memory` test target.

## Timed interval form (FR-038-AC-119 through FR-038-AC-122)

Implemented (IR-551). Bounds are written `{numerator, denominator}` as `{n, d}`.

Admission (AC-119). Under a clause whose `temporal_profile` law names
`quire.temporal.timed/v1`, on each of the eight interval operators of a
self-built package, read the timed form `{lower, upper, lower_end, upper_end}`
with bounds `{0, 1}` and `{3, 1}` for each of the four end pairs, with `lower`
`{1, 2}`, as the punctual `[3, 3]` (both bounds `{3, 1}`, both ends `closed`), and
read the `null` interval.

Pattern refusals (AC-120). Under `timed/v1`, and again under
`quire.temporal.infinite-trace/v1` and each of the three bounded profiles, read the timed form with
`lower` numerator `"-1"`, `lower` numerator `"01"`, `upper` denominator `"0"`,
`upper` denominator `"-2"`, `lower` a JSON integer, `lower` the string `"3"`, and
`upper` `null`; with `lower` numerator `"-1"` and `upper` denominator `"0"` together;
a package with a non-reduced bound `{2, 4}` in the first node and a pattern
failure in a later node, and the two swapped; and a package with an independent duplicate structural id together with a timed or integer bound outside its pattern.

Lowest terms and bounds (AC-121). Read `lower` `{2, 4}`; `lower` `{2, 4}` with `upper`
numerator `"-1"`; `lower` numerator `"-1"` with `upper` `{2, 4}`; `upper` `{2, 4}`
alone; `lower` `{2, 4}` beside a profile-fit defect in a lower-structural-id node; `(3, 3]`,
`[3, 3)` and `(3, 3)` with both bounds `{3, 1}`; `lower` `{5, 2}` over `upper` `{2, 1}`;
`lower` `{1, 2}` with `upper` `{2, 3}` and the two swapped; `lower`
`{18446744073709551617, 3}` with `upper` `{18446744073709551616, 3}`, both ends
`closed`, and the two swapped; and that beyond-2^64 package (`lower`
`{18446744073709551617, 3}`, `upper` `{18446744073709551616, 3}`) under a work limit
that its GCD or cross-multiplication takes past.

Profile fit and member-set defects (AC-122). Read the timed form with bounds `{0, 1}`
and `{3, 1}` and both ends `closed` under `infinite-trace` and under each of the three
bounded profiles and under `timed/v1`; the form with valid rational bounds and
`lower_end` `"half"`, and with `upper_end` `"half"`; `{lower: "0", upper: "3",
lower_end: "closed"}` and the same with the four ends and a fifth member `extra`, both
with integer-string bounds; the same two with the rational-object bounds of the timed
form; and a clause holding such a member-set defect beside a profile-fit defect in a
higher-structural-id clause.

Expected: AC-119's inputs all admit. AC-120's inputs refuse
`invalid_package`/`invalid-value` at `.../interval/lower` or `.../interval/upper`
(the failing bound, first in member order, so `lower` for the two-failure input), in
strict wire validation and under each profile read, the package with the later pattern
failure refuses at that later node, and the body-edited packages with retained structural ids refuse
`invalid-value` at the bound before a later profile-fit defect. AC-121: `{2, 4}` refuses
`invalid_semantic_graph` at its bound, including beside the profile-fit defect; the
`{2, 4}` lower with a negative upper refuses `invalid-value` at `upper`, the negative
lower with a non-reduced upper `invalid-value` at `lower`, the `{2, 4}` upper alone
`invalid_semantic_graph` at `upper`; `(3, 3]`, `[3, 3)`, `(3, 3)` and `{5, 2}` over
`{2, 1}` refuse `invalid-value` at `/semantic_graph/nodes/{n}/body`; the `{1, 2}`
to `{2, 3}` pair admits and its swap refuses; the beyond-2^64 pair refuses and its swap
admits; and the work-limited read returns `incomplete` naming the work limit. AC-122:
the closed `closed` form refuses `operation-member-mismatch` at the application under
`infinite-trace` and the bounded profiles and admits under `timed/v1`; the two `"half"`
inputs, the missing-end and fifth-member inputs with integer-string bounds refuse
`operation-member-mismatch` at `operation.member`, reported after the higher-structural-id
clause's profile-fit defect; and the two inputs with rational-object bounds refuse
`invalid-value` at `.../interval/lower`.

## FCD relationships and named ends (FR-038-AC-165 through FR-038-AC-173 and FR-038-AC-252 through FR-038-AC-257)

Status: items 1-10 remain **PLANNED / UNRUN** under IR-661; item 11 is
**Implemented** under IR-824 with tagged TC-048 reader tests. Existing TC-048
trace tags alone do not establish the remaining procedures.
They require independently authored minimal Semantic IR 2.0.0 documents and
packages built from this repository's public vocabulary. Do not copy an FCD
schema, fixture, binary or another repository's package into this repository.
The FCD primary-source links in FR-038 "Selected relationship declarations"
identify the shape to implement; the cases below are procedures, not execution
results.

1. Author one selected package of object types `Order` and `Customer`, with a
   relationship owned by `Order` whose identity is
   `ix://example/shop/relationship/Order-billedTo-Customer`, source role
   `billedTo`, source type `ix://example/shop/Order`, target role `bills`, and
   target type `ix://example/shop/Customer`. Supply each end's independently
   authored multiplicity and the FCD-shaped category, composite, direction and
   origin members. Keep fields and an operation on `Order` owner-nested, and
   the operation's parameter nested under the operation. Derive the selection
   evidence and complete package values through the authoritative canonical encoder
   after every document edit. Admit the document and a correctly owned
   `relation`/`relationship` graph declaration. Independently change only the
   relationship identity to an owner-nested, foreign-package and wrong-slot
   identity; change a field and then an operation to the global relationship
   slot. Compare each typed `invalid_model_binding`/`malformed-declaration`
   outcome. Separately key a `model`/`object_type` node with the relationship
   owner: the owner join refuses `missing_declaration`/`missing-selection`,
   as FR-038-AC-155 requires.
2. Read with both roles, then remove only the target role. The selected
   declaration admits in both cases and the latter exposes only the forward
   named end. Change source role `billedTo` to `billedToChanged`, updating the
   relationship's minted identity and all canonical joins: the old name
   refuses `missing_declaration`/`missing-name`, while the new name admits the
   otherwise same forward navigation. Repeat this observation for the target
   role on an eligible finite inverse case. These calls, not a new accessor,
   observe role retention. Separately remove the source role; set each role to
   null, `""`
   and a non-string. Every malformed-role mutation refuses
   `invalid_model_binding`/`malformed-declaration` at the selection row, even
   with no application naming that relationship. Use an arbitrary non-empty
   target role to confirm the reader does not enforce the frontend registry.
3. Mutate each end's type to a missing identity and to the identity of a
   relationship. The missing targets refuse `missing_declaration`/`missing-name`
   and the relationship targets refuse
   `invalid_model_binding`/`malformed-declaration`. Run each with the target
   sorting before and after the referring relationship. Independently make
   an end malformed, remove its type or multiplicity, make a multiplicity
   malformed, and reverse otherwise valid bounds. Also point the source type
   at a valid object type other than the owning type: this refuses
   `invalid_model_binding`/`malformed-declaration`. Compare the declared typed
   refusals and selection-row pointer. Under the planned retention contract, also
   require the actual relationship declaration identity and its valid typed origin,
   distinct from the enclosing owner and any graph-node locus. Compare source
   strings/coordinates/optional ends exactly; compare generated strings and ordered
   input identities exactly without a fabricated span. Missing/malformed origin
   stays absent. These retention assertions remain PLANNED / UNRUN until CODE
   implements them; no row returns a package or changes refusal precedence.
4. Repeat one relationship identity under the same owner and under a second
   owner: each is `invalid_model_binding`/`conflicting-binding`. Remove the
   identity from two relationships: each is a malformed declaration, never
   a conflict. Combine a malformed role, a missing end type and reversed
   multiplicity on one relationship, then restore the role, and compare
   FR-154's earlier-row refusal. Point at a declaration independently refused
   for identity or kind and confirm its own refusal survives in either node
   order. Measure the cumulative work threshold through the successful selection
   stage, keeping other limits sufficient. One below that threshold returns
   `incomplete` at the selection row with no partial admitted package; the
   threshold pays the stage, not necessarily the later complete read. Separately
   measure the total-reader limit in step 8.

5. Build a `quire.op.model.navigate` application with a
   `relationship_end` member whose `declaration` names the relationship graph
   node from step 1 and whose `name` is `billedTo`. The relationship node's
   body is empty and its content-only `ModelOwner` names the selected global
   relationship identity. Give operand 0 type `Reference<Order>` and the
   result type `Reference<Customer>`; with FCD-shaped `source-to-target`
   direction and target multiplicity `[1,1]`, it admits. The source's
   multiplicity is `[0,unbounded]` and does not determine the forward result.
   Remove only the target role and confirm the same forward application still
   admits. On that document, request `bills`, a wholly unknown role, and the
   relationship identity's terminal segment as `name`, independently: each
   refuses `missing_declaration`/`missing-name` at `member.name`. Keep the
   target role absent and combine the unknown name with a wrong receiver,
   disallowed direction and wrong result: missing name still wins. Change
   `declaration` to the valid source object node, then the valid target object
   node: owner recovery succeeds for that object and the new member-kind
   binding check refuses `ill_typed`/`operator-ineligible` at
   `member.declaration`, without rejecting those object nodes' own owners.
   Combine that valid object declaration with an unknown role to confirm
   declaration-kind eligibility precedes role lookup. Independently make the
   relationship node's owner undeclared or wrong-kind and observe the earlier
   existing owner join (FR-038-AC-155). Finally, give both end roles the same
   name and request it: two role matches refuse
   `ambiguous_declaration`/`ambiguous-name`.
6. On an independently authored schema-valid relationship with `bidirectional`
   direction and unordered finite multiplicities, navigate forward over
   `Reference<Order>` and inverse over `Reference<Customer>`, using the same
   relationship graph node and its two role names. Change only the destination
   multiplicity through `[1,1]`, `[0,1]`, and `[2,3]` with unique true and
   false. Compare the derived result nodes against `Reference<U>`,
   `Option<Reference<U>>`, `Set<Reference<U>>[2,3]` and
   `Bag<Reference<U>>[2,3]`, where `U` is Customer forward and Order inverse.
   Vary source and target multiplicities independently to show that the
   opposite destination end, not the receiver end, determines each result.
   These finite bidirectional documents are self-authored contract cases,
   not a claim that today's extraction frontend emits that direction or
   source multiplicity. Add `PriorityOrder extends Order` and
   `PreferredCustomer extends Customer`; a receiver of each subtype admits
   the inherited forward or inverse end and keeps the declared destination
   result node. A third unrelated object type refuses. Change the destination
   type to another valid object type, updating its relationship identity and
   joins, and observe that only the corresponding new result node admits.
   Independently give a forward call the unrelated target receiver
   type and an inverse call the source receiver type, and give a well-typed
   receiver a different result node: each refuses
   `ill_typed`/`operator-ineligible`, respectively at `body.arguments[0]`
   and `body.result_type`.
7. Retain both role names and finite unordered destination multiplicities.
   For each direction, read the corresponding forward and inverse
   applications: `source-to-target` admits only forward, `target-to-source`
   only inverse, and `bidirectional` and `undirected` admit both. A disallowed
   traversal refuses `ill_typed`/`operator-ineligible` at `member.name` even
   when the inverse name exists. With an eligible direction, give the
   destination an unbounded upper and then `ordered: true`: each refuses
   `unsupported_construct`/`expression-form` at `member.name`. Restore the
   exact FCD source-to-target shape, including the unbounded source end, and
   name its present inverse: direction refuses first, never a fabricated
   inverse result type. No row changes reader limits to admit unsupported
   navigation.
8. For each new operation refusal above, compare the typed code, cause,
   RFC 6901 pointer and calling application key. Keep graph dependencies,
   derived identities and other operation members current after each mutation
   so the intended check is reached. Combine owner-recovery, role and receiver
   defects to verify recovery wins; combine wrong receiver, disallowed
   direction, unsupported destination multiplicity and result mismatch to
   verify receiver wins; then repair successive defects to observe direction,
   multiplicity and result comparison in order. Measure the total work of a
   successful complete relationship-end read; the exact measured total admits
   and one less returns `incomplete` at the first unpayable charge in the
   existing stage order without a partial package. Operation validation is
   charged before later graph-body reference edges, so a final argument-edge
   charge may report `/semantic_graph/nodes/<n>/body/arguments/0/target`;
   do not force this total-reader oracle to the selection row. Separately
   retain step 4's selection-stage-minus-one selection-row oracle. These are
   acceptance procedures to execute in the code
   lane, not recorded measurements. External QSpec #191 relationship fixtures
   remain planned and unrun until merged and published by their owner; this
   procedure neither copies them nor tags them as current evidence.

9. Before any application uses the relationship, independently remove
   `direction`, `category`, `composite` and `origin`; set each to null and a
   wrong JSON type; use `diagonal` for direction and `unknown` for category;
   and supply an origin with no admitted branch, both branches, a missing
   required source-locus member or a malformed generated-origin member.
   Each refuses `invalid_model_binding`/`malformed-declaration` at the
   selection row, retaining typed code, cause and pointer under step 3's
   existing refusal contract. Require its authentic available declaration identity;
   missing/malformed origin retains no origin rather than a partial branch or span.
   These added metadata checks remain PLANNED / UNRUN. Valid
   independently authored source and generated origin branches each admit.
   No missing direction becomes source-to-target by default. Combine a
   malformed non-end member with a dangling end type and reversed
   multiplicity to confirm the malformed-declaration row precedes both.
10. Author a self-relationship on `Order` with the same selected relationship
    graph-node mapper, two distinct roles, bidirectional direction and finite
    unordered singleton destination. Read `quire.op.model.reaches` over two
    `Reference<Order>` values with the source role, then the inverse role:
    both admit with Boolean result. Make destination `[0,1]` and admit the
    corresponding optional edge; use `PriorityOrder` subtype operands and
    admit by static-edge conformance. Independently substitute an unrelated
    operand 0, then operand 1, and a heterogeneous destination type: each
    refuses `ill_typed`/`operator-ineligible`, at arguments0, arguments1 and
    member.name respectively. A finite Set or Bag destination on the
    homogeneous edge is no longer refused here: item 11 (FR-038-AC-252 through FR-038-AC-257)
    supersedes that clause of FR-038-AC-172. Remove the requested
    inverse role and name it: missing name wins before operand checks. Name
    a valid object node as member.declaration: the new declaration-kind check
    refuses `ill_typed`/`operator-ineligible` there. Give an otherwise valid
    reaches application a navigation result type instead of Boolean: it
    refuses at body.result_type. Keep both operands' static object owners
    distinct from the relationship graph node; no Reference<relationship>
    substitutes for an object reference. These are static reader procedures,
    not runtime graph execution evidence.
11. Implemented with tagged TC-048 reader tests (FR-038-AC-252 through FR-038-AC-257, IR-824; QSpec FR-322-AC-57 RN-18 and RN-20 through
    RN-27, referenced by id only.) Start from item 10's homogeneous `Order`
    self-relationship and author each variant independently from this
    repository's public vocabulary, retaining valid structural ids and updating references after each edit.
    Patch the destination end multiplicity to finite `{0,3}` unique, then
    `{0,3}` not unique, then `{2,3}`, and read `quire.op.model.reaches` over
    two `Reference<Order>` operands with the source role and then the inverse
    role: every case admits with the same node shape as the single-valued case
    (Reference operands, Boolean `result_type`, no multiplicity or charge
    member). Repeat with a `PriorityOrder` subtype as operand 1: it admits;
    with an unrelated operand 1: `ill_typed`/`operator-ineligible` at
    arguments1. Patch the end to `{0,unbounded}`, then to `{0,3}` ordered:
    each refuses `unsupported_construct`/`expression-form` at member.name. Give
    the end a destination type other than the receiver endpoint's, once
    single-valued and once with upper bound 3: each refuses
    `ill_typed`/`operator-ineligible` at member.name; the same different-type
    end made `{0,unbounded}` refuses `unsupported_construct`/`expression-form`
    (multiplicity before type); a named end against the declared direction
    refuses `ill_typed`/`operator-ineligible` at member.name even when also
    `{0,unbounded}` (direction before multiplicity). Give an admitted
    multi-valued case a navigation result type instead of Boolean: it refuses
    at body.result_type. Last, combine defects in one package, citing QSpec
    FR-322-AC-57's check order (receiver-endpoint operands, then direction,
    then multiplicity, then type): an unrelated operand 1 together with, in
    turn, an end against the declared direction, a `{0,unbounded}` end, a
    `{0,3}` ordered end and an end of another type, each refuses
    `ill_typed`/`operator-ineligible` at arguments1, not at member.name. The
    implementation slice reorders the reader's operand-1 check to match and
    compares operand 1 with the receiver endpoint type, not the destination
    type, so a well-formed other-type end still refuses at member.name; a
    single-defect package is unchanged. These are static reader procedures;
    reachability charges are not asserted here.

Expected: each success and refusal above satisfies FR-038-AC-165 through
FR-038-AC-173 and FR-038-AC-252 through FR-038-AC-257. Compare typed code, cause and pointer, and the calling graph-node
key only
where the existing refusal retains its locus. At the private `admit_selection` return, selected-document declaration refusals
additionally retain authentic declaration identity and valid origin under
FR-038-AC-174/175 and FR-038-AC-186 through FR-038-AC-196, with absence for missing/malformed origin. The existing
model-intake conversion to `ValidationFailure` retains code/path/cause only;
these procedures assert no public metadata extension. Do not parse diagnostic
prose or infer a graph locus from a semantic declaration URI. Item 11 is **Implemented** with tagged TC-048 reader tests. The other planned
outcomes and new retention checks retain their separate status.


### Private intake refusal-origin checks

These Test and Inspection procedures back FR-038-AC-174/175 and
FR-038-AC-186 through FR-038-AC-196. IR-663 implements the private retention
contract. The final-head runtime tests and both Clippy lanes are UNRUN; full
pre-PR and premerge gates remain pending. AC-193/194 are source Inspections.
Existing AC-167/173 trace tags verify refusal/admission semantics only and
supply no evidence for these retention criteria. Exercise the real private model-intake return,
`admit_selection` yielding `SelectionFailure::Refused(SelectionRefusal)`, in
crate-local tests. Author minimal domain-package documents in this repository
and admit their selected bytes through the existing content checks. Follow the
owning FCD origin contract by reference; do not copy foreign schemas or fixtures.
No public reader/dispatch/driver metadata observation is allocated.

1. Cause a declaration reference failure with a valid source origin, then with a
   valid generated origin. At the private intake return after selected-document
   release, require the authentic referring declaration identity and every supplied
   origin member, with unchanged refusal code/cause and selection-row member.
   A nested relationship shall have identity/origin distinct from its parent;
   require the actual relationship's context, never the owner's or missing target's.
   Independently cause an unresolved field type, operation return type and parameter
   type, each with its own identity/origin different from the containing type and
   operation. Require the field, operation and parameter context respectively;
   remove each nested identity/origin to require genuine None rather than owner
   substitution. These are existing intake declaration forms, not new admissions.
   Separately check the existing public model-intake conversion's code, cause and
   selection-row pointer; preserve unrelated public refusal fields and graph locus.
2. Exercise all four end-coordinate presence combinations, positive coordinates
   at the existing selected-document numeric boundary, and otherwise valid origins
   that would fail an invented paired-end or span-order rule. Assert exact values
   and optional presence. Above-bound input shall retain its earlier numeric refusal,
   not acquire declaration metadata or a new origin classification. Vary identity,
   path and version spelling and generated input order/repetition. Assert no span
   on Generated, with typed exact comparison rather than prose parsing.
3. Remove origin; supply both branches, unknown members, invalid members and partial
   source/generated branches. Require None for origin, while retaining any actual
   supplied declaration identity. Missing/non-string identity shall retain None;
   an object-id spelling refused by intake shall remain supplied, not repaired.
   No default string, partial branch or enclosing-owner identity shall pass.
4. Group same-identity type declarations with identical valid origins, distinct
   valid origins, no valid origins, one valid plus missing origin, and one valid
   plus malformed origin. Require the common identity. Retain origin only if
   every candidate is valid and every typed value/presence agrees; otherwise
   require None. Repeat these cases both for conflicting-binding and for a
   malformed-declaration group (bad common object-id spelling or bad candidate
   kind). Permute candidate order without changing origin or first refusal.
   For AC-196, separately repeat a valid relationship identity under two owners with different
   origins: in the existing sequential node/member-path order require the later
   actual offending relationship's metadata, not the earlier node or owner.
   Give that later relationship missing/malformed origin to require None with
   its existing first refusal. This control must not use type-group consensus.

5. Audit every private `SelectionRefusal` constructor and declaration-to-selection
   conversion. Exercise pre-declaration missing bytes, content mismatch and wrong
   selection identity; require metadata absence and original code/cause/member.
   Combine declaration defects with earlier table/member-path defects and bounded
   work exhaustion; assert the same first refusal or distinct Limit/InexactNumber
   result and no partial package. For the exact-budget clause of AC-192, charge
   the retained identity and origin independently from fixed authored records
   (source and generated origins) and require success at exactly that budget and
   the existing work-limit result, with no partial package, at one less; likewise
   require a selected document to succeed at its measured work and return the
   work-limit result at one less. For the inexact-number control, author bytes
   with an origin integer above 2^53 and a declaration defect, then supply them
   through the actual selection-evidence path under the selected document name and under another selected document name. The original typed inexact-number
   result must precede dependency/declaration checks; this is an adverse read,
   not evidence that inexact input admits. Removing available metadata, attaching a foreign
   declaration's metadata, normalizing members or losing owned values on document
   release must fail a typed intake-return assertion.
6. Inspect that the `admit_selection` refusal conversion to `ValidationFailure`
   still carries existing code/path/cause and adds no declaration metadata fields
   to `CheckedPackageRefusal` or the IR-owned dispatch result. Retained `locus`, expected
   node key, contract-version and document-pointer behavior elsewhere remain intact.
   Future concrete typed consumers remain an open owner allocation, not this test's
   promised interface. No second parse, raised ceiling or diagnostic collection
   supplies retention. Source inspection alone cannot close the runtime Tests.
7. For AC-194, inspect that full typed-origin equality actually determines the
   production group retention decision. Follow the owned metadata in
   `SelectionRefusal` through `SelectionFailure::Refused` after `admit_selection`
   releases the document, to the existing `SelectionFailure::Refused(refused)`
   mapping arm in `checked_package/v2/mod.rs`. Pass only if that arm releases the
   actual owner by its end, by explicit ownership teardown or ordinary scoped
   destruction, while preserving code/path/cause. Fail if a field is read solely
   to silence lint, or its read affects neither the group decision nor ownership
   transfer. A blanket dead-code allowance, observer or public metadata field
   also fails. For AC-195, the CODE slice shall run both repository Clippy lanes,
   including the feature-off model-consumer command, and record their actual exits.
   If this ownership lifecycle is unavailable, report the source/API gap rather
   than claim the criterion satisfied. Inspect the named ownership and projection
   in `model_members.rs` and `v2/mod.rs`. Final-head runtime/Clippy checks are
   UNRUN; full gates remain pending.

## Refusal fields and structural node loci (FR-038-AC-183 through FR-038-AC-185)

AC-183 and AC-184 are withdrawn under QSpec #517. For AC-185, read malformed-wire, duplicate-id, unresolved-reference and invalid-body packages through the production reader. Compare code, cause, pointer and authentic structural node locus with independently authored expectations. Pre-graph refusals invent no node locus. The public refusal has no `expected_node_id()` accessor or expected/actual digest pair; clone, Display and serialized forms preserve only the published fields.

## Aggregate-bodied union values (FR-038-AC-225 through FR-038-AC-227)

Read QSpec `positive-union-nodes.json` directly from the selected checkout through the production reader. Verify payload-bearing and nullary union-value nodes preserve their exact structural ids and complete aggregate bodies; separately admit ordinary applications, record values and tuple values. Duplicate a union-value id, break one body reference and make a case/payload invalid in separate copies; assert the specified graph, reference and body refusals. For a valid group with union and nonapplication members, verify its shared non-empty label, producer-assigned structural IDs, and exact in-group and outside references. A valid graph permutation retains every structural ID and reference. Malformed structural ordinal, invalid cycle label, type-incompatible exact target, repeated ID and unresolved target controls refuse at their own graph or wire stage. No content-key or expected-key accessor is involved.

## Dependency selection and diamond conflict (FR-038-AC-217, FR-038-AC-218)

Supply a dependency package whose `name` matches the lock's `{identity}` row and require admission. Omit it and require `missing_import`/`missing-selection` at the row. Supply equal packages through two branches of a diamond and require admission; change one complete JSON body under the same name and require `invalid_package`/`conflicting-definition` with both paths, comparing ordered arrays and object members structurally. Require every refusal to retain code, cause and pointer, with no expected/actual digest pair or content-mismatch cause.

## Structural type forwarding work (FR-038-AC-224)

PLANNED/UNRUN (IR-490). Use a private operation-stage probe with an isolated
meter to resolve a chain containing both `alias` and non-population
`bounded_domain` nodes. Measure the before/after counter for a chain with N
distinct forwarding nodes: the first resolution consumes N units, the same
head and each resolved suffix consume zero more in the same application's
validation, and a second application consumes N again. The terminal node is
uncharged. At exactly N units the first resolution succeeds; at N-1 it returns
the typed Work failure at the forwarding node where the charge fails. Include
a chain with an absent next node and a forwarding cycle to verify resolution
failure remains distinct from budget exhaustion. Do not count the separate
`text_profile` pin or leaf-node visits in this isolated measurement.

At the operation stage, reuse the 300-field/300-alias shared-chain case of
AC-44. Under a nonbinding limit, measure total work W for the ordinary
missing-leaf decision. The same decision holds at W; W-1 returns typed Work.
This missing-leaf case ends after the count pass; measure its structural
forwarding charges for that pass only. Separately supply every derived leaf
with its selected law in an otherwise identical record. Require this lawful-leaf
case to reach the second leaf pass and admit. Under a nonbinding limit measure
its total work L and per-pass structural-forwarding counters: the first pass
charges one unit per distinct forwarding node and the second adds zero for
already resolved nodes. It still charges each pass's leaf visits and the
existing `text_profile` pin independently. Admission holds at L; L-1 returns
typed Work. Preserve the pin
control by verifying that the first lookup of each pin-chain node charges one
unit and repeated pin lookups in the same walk charge zero; measure its
exact/one-less boundary independently of the structural-resolution counter. Build otherwise valid applications
that reach structural resolution through `union_arms`, `inner:0`,
`result_inner`, and a leaf edge. Give each its measured exact remaining budget
and one less; the exact case reaches the ordinary operation decision, while
the one-less case returns the reader's typed `incomplete`/`work` at that
operation. Assert no case turns Work into `ill_typed`/`operator-ineligible`,
a skipped union constraint, or an unchecked leaf. Bind each eventual test to
`FR-038-AC-224`; no runtime result is claimed here.

## Structural application and group assembly (FR-046, FR-048)

PLANNED/UNRUN (IR-1141). Construct a typed `CheckedNodeId { package, path, ordinal }` before read using the public model API; the package name and declared qualified name or derived path are authored by the producer. Encode a complete graph and require the production reader to retain the same id only after strict admission. A body edit with a retained id is judged by the independent body and graph rules; no expected key is computed. AC-2/6 of FR-046 are withdrawn.

For FR-048, assemble owner-free derived nodes, a source-owned declaration, a
model-owned declaration and a mixed recursion group. Verify exact
package/path/producer-assigned structural ordinals, shared group labels, self and internal
reference resolution, outside/dependency references, and typed refusals for
duplicate ID, missing target, malformed ordinal and an independently invalid
body shape or join. A valid body edit under the same structural ID may admit.
Exercise the public encoder's exact canonical byte ceiling and one below,
the reader's exact work limit and one below, and iterative encoding and reader
traversal on a 256 KiB stack with deep in-memory bodies. No candidate helper,
preimage or copied QSpec file is permitted. The compiled public inventory is
tested by TC-058.

## Typed parameter declaration (FR-038-AC-258 through FR-038-AC-261)

Implemented (IR-832) with tagged tests. `CheckedParameterDeclaration` and
`CheckedParameterDeclarationError` are exported and listed in the FR-019 Public
items table; FR-019 lists only exported items. The `MalformedBody` mutations
are crate-internal unit tests beside the accessor, since they need the private
wire. Their fixture is a never-admitted package of admitted shape with its kinds
decoded from the wire, following the `scalar_application_operands` precedent
(a module test can admit a package, as the composite operand tests do, but a
parameter fixture would need its own keyed graph for no added check). Build a self-built
admitted package with a pair of parameters named `x` at levels `0` and `1`, a
parameter at level `10`, and add applications over parameters `a` and `b`, each
typed at `Int[0, 9]` (an `integer_range` `bounded_domain`, since a parameter typed
at unbounded `Integer` makes `scalar_application_operands` refuse
`UnboundedRange`), in both argument orders (separately admitted packages).

Expected: `parameter_declaration` over each parameter node returns its supplied
id, the name literal's text and the level literal's canonical decimal text, with
level `10` returned as `"10"`; the two `x` nodes have distinct ids and levels
`"0"` and `"1"`. Supply an unknown id (`UnknownNode`), a literal, an
application and a function node (`NotParameter`), and a parameter whose body a
crate-internal post-admission mutation lacks the `"name"` binding, gives a
non-text name, gives a non-identifier name, types the name literal at a node
that is not `scalar_type`/`text`, or gives `"level"` the value `"010"`
(`MalformedBody`);
assert the typed error and no partial value. Feed each `GraphChild` id from
`scalar_application_operands` for the add applications to the accessor and
compare the names in ordinal order, then in the swapped package. For an add
with an inline literal operand, destructure its `InlineLiteral` without `..`:
it holds the containing application's id and no operand id, and
`parameter_declaration` over that id refuses `NotParameter`. Do not
construct the expected name or level from the accessor under test. The
compiled public inventory of the two new types is covered by
[TC-058](../../model/matrix/TC-058-model-crate-public-interface.md).

## Typed relationship end of a reaches application (FR-038-AC-262 through FR-038-AC-268)

Implemented (IR-851); tagged tests. The two new types are listed in
the FR-019 Public items table by the code change that exported them; [TC-058](../../model/matrix/TC-058-model-crate-public-interface.md)
compares the compiled exports to that table.

Build admitted packages from the IR-824 reaches fixtures: a homogeneous
self-relationship with direction `bidirectional` (so that both a forward and an
inverse application admit; a `source-to-target` or `target-to-source` direction
admits only one of them, AC-254) whose source end is `{1,1}` with its own role
and whose target end is `{2,3}` unique with role `peer`, one application naming
the source role (forward) and one naming the target role (inverse); further
packages with the destination end `{0,3}` unique, `{0,3}` not unique, `{0,1}`
and `{1,1}`, one whose destination end omits its role, and one whose operand 1 is
a subtype of the receiver endpoint type, with a variant whose graph holds no node
with the supplied destination id. Derive the expected bounds, roles and
relationship id from the domain package document and the application's member;
take the expected `destination_type`
structural id directly from the producer-authored `relationship_end` member,
not from the accessor or a derivation from the document.

Expected: `relationship_end` over each application returns the supplied id, the
relationship id, the destination end's `lower`, `upper`, `unique`, `role` and
`forward`, `ordered` false and `upper` `Some`, and the destination type id
exactly as supplied in the member (also for the subtype operand and the
missing-node variant); the forward and
inverse applications of the unequal relationship return different bounds. Supply
an unknown id (`UnknownNode`) and a navigate application, an `integer.add`
application, the relationship node, an object declaration node, a field-read
node and a literal (`NotReachesApplication`). In crate-internal unit tests
beside the accessor, where the private wire is available, mutate an admitted
application so its member is absent, its member `kind` is not `relationship_end`,
its declaration is not a node id or resolves to no relationship, its name is not
text, matches neither role or matches both roles, and assert `UnresolvedEnd` and
no partial value for each. Call twice and on a clone and compare results, package
equality and work. An external fixture exhaustively matches the error enum and
constructs the result field by field (AC-267). Assert separately (AC-268) that
`model_object_fields` over the object type returns only its fields and that no
`selected_population` item is `pub use`d at the model root or re-exported from the
`quire-contract-ir` root, and that the AC-252 through AC-257 outcomes are
unchanged. Do not take the expected values from the accessor under test.

For QSpec FR-322-AC-270/271, read
`proposals/checked-package-v2/fixtures/positive-operation-identities.json`
and `adverse.json` `destination_type_mutations` DT-01..DT-05 directly from
the selected QSpec checkout, without copying either fixture. The positive
operation fixture's destination node is absent from its graph yet the reader
admits it and `relationship_end` returns the supplied structural id verbatim;
the application does not list the destination id as a dependency. Apply each
published mutation to the same fixture: DT-01 (absent member), DT-02 (string),
DT-03 (extra member) and DT-04 (string ordinal) refuse `malformed_wire` at
the schema step; DT-05 names a node present in the graph, admits and returns
the supplied id verbatim. Keep the existing forward/inverse role, direction,
multiplicity, subtype, and absent-graph controls above.

## Typed population of a reaches application (FR-038-AC-269 through FR-038-AC-280)

The bounded selector path is implemented (IR-852); corrected no-`max` admission and exact-decimal projection are PLANNED/UNRUN. The new error type is listed in the
FR-019 Public items table by the code change that exports it, not by the
specification change; [TC-058](../../model/matrix/TC-058-model-crate-public-interface.md)
compares the compiled exports to that table. The producer rule that admits the
`population` member key is outside this repository; build the fixtures from this
repository's own wire so that the key is present or absent as the case needs.

Build admitted packages from the IR-824 and IR-851 reaches fixtures, adding a
`bounded_domain` / `model_population` node of the receiver end's declared type
(its `max` binding an integer literal, its `semantic_type` a `Set` of
`Reference` of the object type, two `Type` occurrences at ordinals 1 and 4 each
with one source-map row) and the `population` key in the application's
`operation.member`, with the node id in the application's `dependencies`.
Further packages: a second population node with another `max`, a reaches
application with no key, and a package holding a population node that no
application names. Derive the expected binding, ordinal, maximum and object
type from the fixture and the authored structural paths, not from the accessor.

Expected: `reaches_population` returns the selector with the supplied node id and
ordinal 1, and `select_population` over `Admitted` with it returns `Finite` with
the fixture's canonical decimal maximum text, minimum 0 and the object type; an application without the
key returns `Ok(None)`; two applications over one population return equal
selectors and over two populations differ in `binding`. Supply an unknown id
(`UnknownNode`) and a navigate application, an `integer.add` application, the
relationship node, a population node and a literal (`NotReachesApplication`). In
crate-internal unit tests beside the accessor, mutate an admitted application's
`population` to a non-node-id value, to an absent id, to a non-population node
and strip the population node's `Type` occurrences, and assert
`UnresolvedPopulation` and no partial selector. Through the public reader, refuse
a `population` that is a non-node-id (a number, an object, `null`) or names no
node, names a non-population node or a node with no `Type` occurrence, or names a
population over a subtype or another object type (each `ill_typed`/
`operator-ineligible`, never `missing_declaration`), each at the `population`
pointer; admit the
exact type; refuse a navigate application carrying `population`; omit the
population id from `dependencies` and list an unreferenced one (each
`invalid_semantic_graph`, no cause). For AC-279 and QSpec FR-322-AC-74, admit one linked population with empty
aggregate body and no `max`, and assert its selector identifies the exact
node and `Type` occurrence; `select_population(Admitted)` returns
`AdmittedUnbounded` with the package name, binding, occurrence, object type
and source regions, and no finite extent. Admit separate `max` values `0`,
`3`, and canonical decimal string `18446744073709551616` (above `u64::MAX`);
assert `Finite` projects each maximum as exact canonical decimal text with
minimum zero. A negative or non-integer `max`, a second body member, and a
wrong `set<reference<T>>` semantic target each refuse
`invalid_package`/`invalid-derived-shape` at the population node's `node_id`
(`RM-POP-MAX`); a non-binding term retains its earlier graph-term refusal,
and a noncanonical JSON number retains `noncanonical_wire`. Do not reject a
valid maximum on a machine-width limit. These corrected population controls
are PLANNED/UNRUN until backed by the IR reader and projection. Build a navigate
application carrying `population` twice, naming an absent node, an
integer-range node and a valid population node in turn: with the id in
`dependencies` (each refused at the `population` pointer, operation stage) and
without it (refused by the join, `invalid_semantic_graph`, no cause); an
application of another operation carrying the key keeps its outcome. For the
order (AC-280), combine an invalid `population` whose node id is listed in
`dependencies` with an operand 1 that does not conform to the receiver end and
assert the refusal is at the operand; give a reaches application a mode and,
separately, a leaf, with a `population` that is a listed node id (and, separately, an invalid id missing from `dependencies`, refused by the join), and assert the
identity-stage refusal (`operation-mode-mismatch`, a law mismatch) in each case.
For the published QSpec `reaches_population_cases` and
`unbounded_population_cases` vectors (FR-322-AC-73/74), use the QSpec checkout
directly from the IR harness without copying fixtures, and assert the same
admission and refusal outcomes. The harness shall give
each appended node (`Reference`, `Set`, population) exactly one `Type`
occurrence and one matching `source_map` row, because the reader requires the
source map to equal the occurrence set exactly; it shall reuse an existing
`Reference` of the object type instead of appending a duplicate. A package built
without those rows is refused `invalid_source_map` before any population check. Call twice and
on a clone and compare results, package equality and work. An external fixture
exhaustively matches the error enum and binds the result type (AC-278). Do not
take the expected values from the accessor under test.

## Hybrid nodes and claims (FR-038-AC-311 through FR-038-AC-317)

Read `hybrid-node-vectors.json` and `fixtures/positive-all-families.json` from
the selected QSpec checkout, without copying either into this repository.
Feed the valid `Braking`/`SlowEnough`, `Braking2`/`Slows`/`Recent` and `Tank`
graphs through the public V2 reader. Check the admitted typed family/form,
structural node ids, package names, body references, semantic types and dependencies;
`Tank` must admit a variable-to-variable rate and reset value and a
`hybrid`-classed variable with a literal-one rate. Mutate one rate under retained structural ids and check the changed body and unaffected references.

Apply QSpec TC-966's strict-wire mutations with retained structural ids and assert
`malformed_wire` at strict wire validation before body checks. For each
semantic mutation, update dependencies and references while retaining valid structural ids and package name before reading it. Assert the exact code, cause and RFC 6901 locus for repeated
modes, wrong variable type or level, rate count and type, mode and reset
indices, reset conflict, initial entry, foreign read, clock class, orphan
transition, interval/profile mismatch and malformed claim read or fairness.
In a combined-defect package, check precedence of strict wire, graph shape,
temporal, model checks 1 through 6, orphan nodes, claim checks 7 through 9,
protocol and operation. Verify that an unchanged valid hybrid node retains
its key and that the reader never supplies a missing graph node or a second
spelling for the default initial state. The process provider and host trace
checks HY-15 and HY-16 remain QSpec TC-966's evidence, outside this IR
reader test.

## Symbolic clock slots and place presence (FR-038-AC-335 through FR-038-AC-340)

Status: PLANNED/UNRUN (IR-1083). Read QSpec's published
`proposals/checked-package-v2/timed-slot-selector-vectors.json` from the
selected QSpec checkout. Do not store a local copy. Apply its complete
SL-01..SL-08, SR-01..SR-04, GD-01..GD-03 and PR-01..PR-06 cases through the
public V2 reader, updating dependencies and references for a semantic mutation.
Compare exact admission or each vector's expected code and cause. Compare the
RFC 6901 locus suffix recorded by SR-01..04 and PR-04..06. For GD-02, assert `invalid_package`/`malformed-clock-place` at the twin
place's first forged `resolved_path` row
`/semantic_graph/nodes/72/body/members/2/value/members/1`, as fixed by QSpec
FR-322-AC-120, and assert aggregate order passes before the row join. For GD-03, assert `invalid_package`/`timed-node-reference` at
`/semantic_graph/nodes/47/body` with vector suffix `body`, as fixed by QSpec
FR-322-AC-243. This reader control remains PLANNED/UNRUN. Check the reader stage against FR-322's stated order. Keep
strict-wire and graph-shape checks ahead of the timed-model and
operation-body checks.

For AC-335, inspect the admitted template path and selector node identity,
not merely the selector's semantic type. Assert one place node for three
Sequence instances; separate Set, Bag, OrderedSet and nested-Sequence paths;
the exact Text key type node for Map; and the slot-step refusal for a literal,
an expression or the wrong domain type node. For AC-336, admit an empty guard aggregate on a guard-free root. Add one
root-listed guard and verify their complete bodies change while the place
path template and structural ids remain valid. A place `guards` binding refuses at strict wire
validation; an otherwise ordered second place of the same path refuses at its forged
`resolved_path` row as `invalid_package`/`malformed-clock-place`,
and an unlisted guard refuses at its own `body`.

For AC-337 and AC-338, admit `present(other.v)`, `present(self.q[i])` and the
quantifier's `present(e)` with Boolean result and ordered place, owner and slot
arguments. Check that `e` is typed at the slot selector domain, that each
application and its shared place template have the expected independent keys,
and that no presence member is added to another node. Assert exact argument
loci for a missing slot, non-place first argument, wrong owner and wrong slot
type. Include the existing one-operand Option term as an unchanged control.

For AC-339, join `Option<Clock>` and `Sequence<Option<Clock>>` resolved-path
rows to the selected declaration's member type and optional presence. Assert
that Option adds no path step, the terminal inner Clock is accepted, and a
missing or mismatched selected member is refused at the place join.

## Timed aggregate and clause reader (FR-038-AC-342, FR-038-AC-357 through FR-038-AC-362)

Status: PLANNED/UNRUN (IR-1096, IR-1109). Read QSpec's published
`proposals/checked-package-v2/timed-slot-selector-vectors.json` and
`proposals/checked-package-v2/timed-clause-node-vectors.json` through the
selected checkout. Apply `AO-01`..`AO-10` and `CL-01`..`CL-54` to the public
V2 reader with their selected packages; never store local copies. For every
vector compare admission or exact code, cause and RFC 6901 locus. Assert
`AO-01`, `AO-05`, `AO-08`, `CL-01` and each in-key-order `CL` pair admit;
assert all published reverse-order and equal-key cases refuse
`invalid_semantic_graph` with null cause at the first offending member.

For AC-342 and AC-359, test all six root aggregates, guard-part constraint
lists and invariant bound lists with source-region minimum first and QSpec
FR-322-AC-265 structural tie-break second. `AO-02`..`AO-07` distinguish
reversed references, structural-id order, tied starts and an earlier extra region.
`AO-08`..`AO-10` distinguish ordered, reversed and equal place-path keys on
one region. `CL-29`..`CL-53` distinguish constraint, reset and invariant
keys, including comparison rank, rational numeric bound, owner and slot
terms, operation, condition, value, binder order, all invariant bounds and
predicate. `CL-47` changes a shared term's region without changing either
clause's position. Keep an otherwise ordered repeated-place path control that passes aggregate
order and refuses `invalid_package`/`malformed-clock-place` at its forged row
(`/semantic_graph/nodes/59/body/members/2/value/members/1` for TM-12).
Retain a separate
equal-key repeated-reference control that refuses at aggregate order.
`duplicate-clock-place` remains the post-row-join rejection obligation, but
no conforming second node with the same model and path can reach it.

For each remaining root branch—`resets` (binding 4), `guards` (5),
`invariants` (6) and `constraints` (8)—build an admissible two-reference
control with distinct member nodes and strictly increasing full order keys.
For each, reverse the references, update the source-map entry and compare the complete package value, and require `invalid_semantic_graph` with null
cause at `/semantic_graph/nodes/{i}/body/members/{b}/value/members/1`,
using that control's actual graph index `i` and branch index `b`. Retain
the admitted original and verify its exact dependencies, selected
declarations and source-map joins, so a foreign defect cannot pass the
negative control. For each branch also give two distinct member nodes one
minimum region triple and distinct structural keys: their structural order
admits and its reverse refuses at the same second-member locus. Repeat a reference to give two equal full keys (minimum region and
structural tie-break) and require the same refusal at the second member. `AO-02`/`AO-03` already
cover root `clock_places` and `AO-04` covers root `urgencies`; the deleted
place `guards` branch has no aggregate-order control.

For **each** of `AO-02`, `AO-03`, `AO-04`, `AO-06` and `AO-07`, make three
independent mutations that keep its first out-of-order pair. First replace
the timed root's required `occurrences` array with text and require
`malformed_wire` at `/semantic_graph/nodes/{i}/occurrences` before any
graph or order check. Second remove one still-referenced root dependency,
keep all unrelated package members valid and require
`invalid_semantic_graph` with null cause at
`/semantic_graph/nodes/{i}/dependencies` before timed-body checks.
Third duplicate the root's structural id on a later graph node while preserving
the root's order defect and require `invalid_semantic_graph` at the later node
before timed order. With distinct valid ids, retain the matching root source-map
reference and assert the published timed-order refusal. Verify each unmodified AO patch still reaches its published order refusal
after the three precedence probes.

For AC-357, inspect each admitted form and exact binding order in `CL-01`.
Assert `CL-17`..`CL-20`, `CL-27` and `CL-28` refuse at strict wire validation
before any semantic check: empty parts, empty bounds, unknown comparison,
empty part constraints, place `guards` and a conditional parameter reset.
Include an admitted `eq` comparison in a guard and refuse `eq` in an
invariant through `CL-24` at the semantic join stage.

For AC-358, make `CL-05` and `CL-26` compete with an otherwise later per-node
join defect and assert the first unlisted node by structural node-id order refuses at
its own `/body`; use a root-omitted guard as a third form. `CL-04` refuses a
second guard for one anchor at that guard's operation binding. `CL-06`
refuses a listed but unused constraint at the root member. Keep `CL-01` as
an admitted control for both membership paths.

For AC-360 and AC-361, assert the operation-anchor join for `CL-02`, `CL-03`
and state-valid foreign-model `CL-25` plus the state-stage `CL-54` control,
and the exact typed/locus refusals of `CL-07`..`CL-15` and
`CL-21`..`CL-24`. `CL-03` is a reader-level test only while STD-555 leaves
an `execution_point` binding schema-invalid; do not count it as a schema-valid
package. A guard/reset/parameter constraint uses the anchor of its own root
model, while an operation urgency retains model plus operation name. `CL-25`
has a valid `Gadget.ping` anchor with matching context, frame and
`semantic_type`, so the state step passes and the timed join refuses
`invalid_package`/`timed-node-reference` at the operation target. `CL-54`
has a foreign context and `semantic_type` but the first model's frame, and
refuses `invalid_model_binding`/`malformed-declaration` at
`/semantic_graph/nodes/70` in the earlier state step. Check
place owner and selector-domain terms, binder order, frame writes, Boolean
conditions/predicates, Clock parameter declarations, same-operation guard
constraints and upper-only invariant bounds. For each mutated semantic case,
update dependent references and compare complete package values before calling the reader.

For AC-362, inspect the positive dense timed-model and timed-slot fixtures
for the same closed clause forms with valid structural ids, then replay every `CL-01`..`CL-54`
patch on the selected clause fixture. Independently check the staged first
refusal order: strict wire, graph shape, aggregate order,
unlisted timed nodes, per-node joins, model-wide checks, followed by protocol
and operation steps. Use a combined-defect case at each adjacent boundary;
a vector's expected label alone is not evidence of stage order. Quantified
source guard and invariant refusals `QN-01` and `QN-02` remain QSpec producer
controls for STD-554; IR's reader cannot recover a quantifier omitted by a
producer, so TC-048 does not claim that source evidence.

## Admitted operation-view provenance (FR-038-AC-341)

Read V2 operations with populated field projection, member declaration, mode,
law definitions and state-clause kind through the public checked-package
reader. Assert each fallible accessor returns its exact typed admitted value.
Read corresponding admitted operations without each optional fact; assert
admitted `None` or an admitted empty law view. Construct a term through direct
deserialization of the same operation record, with no reader admission, and
assert that each accessor returns typed `NotAdmitted` rather than those absent
values. A malformed declaration, mode or law still refuses at reader admission;
the provenance check does not change that precedence. These controls are
PLANNED/UNRUN under IR-1087.
