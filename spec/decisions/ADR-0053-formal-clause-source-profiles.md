---
id: ADR-0053
title: "Proposed bounded formal-clause source profiles"
type: ADR
status: proposed
owner: kreneskyp
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-012
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-013
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-014
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-015
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/FR-016
    type: depends_on
---
# ADR-0053: Proposed bounded formal-clause source profiles

## Status

**Proposed; owner decision absent.** Prepared for
[issue #53](https://github.com/agent-ix/quire-contract-ir/issues/53), not an
implementation authorization or a claim that any source frontend is qualified.
The [owner ruling on #52](https://github.com/agent-ix/quire-contract-ir/issues/52#issuecomment-5532708641)
explicitly makes the earlier OCL/SysML/FRET research input to this decision,
not the decision itself. Only `@kreneskyp` may accept, amend, reject, or defer it.
The proposed profile names below are not registered or published identities.

| Decision field | State |
|---|---|
| Recommendation | OCL 2.4-based non-temporal bounded profile first |
| Later sequence | FRETish/tl-* temporal profile, then SysML/KerML invariant/import profile; separate qualification gates |
| Human decision, timestamp, rationale | Not recorded |
| Independent review | Pending coordinator review |
| Implementation, native parser qualification, hosted CI | Not performed or authorized by this packet |

## Context

TypeSpec remains the structural schema source. A frontend consumes resolved,
compiled **domain content** from filament-core-data #36, not raw Markdown,
runtime Rust types, or envelope-only `SemanticObject` records. Module-owned
`FieldDecl`, `TypeRef`, operation, bound, and identity declarations must describe
the values a clause names. A field that merely says `int` or `Dict[str, Any]`
does not supply that environment.

Quire extracts static authored declarations. The contract frontend parses and
types its selected language, binds compiled declarations, and submits explicit
expressions to the closed IR. The domain producer executes checks; Quoin retains
native structured results; ix-flow retains human decisions. No JVM, solver,
language runtime, or producer invocation is added to Quire or Quoin.

FR-014 has no temporal expression node and FR-013 has no identity-bearing object
reference or recursive record type. A source notation does not create either
capability. Temporal syntax must go through a separately specified tl-* bridge;
object navigation remains gated by #54's reference/type-environment contract.

## Decision proposed for approval

Adopt `ocl24-bounded-v1` as the first **OCL-based subset**, not a claim of full
OCL conformance. Initially admit named invariants and operation pre/postconditions
whose complete accepted evaluation is two-valued and statically defined.
Reject unsupported constructs with their authored span and the protected IR
construct; never emit an approximation or erase a rejected clause.

Reserve `fretish-bounded-ticks-v1` and `sysml2-bounded-invariant-v1` as later
candidate profiles. Their descriptions below are reviewable target contracts,
not enabled frontends. Existing manual/Test/Inspection/Analysis/Demonstration
obligations remain valid; language selection does not mandate migration.
JSON Schema remains structural shape. SMT-LIB, WhyML, and Boogie remain derived
backend encodings, not additional editable clause authorities.

### One clause, one language, one editable authority

A formal clause has one stable clause identity and one authored source:
either one language-tagged inline fence or one explicitly referenced local file
region, never both. The reference binds exact bytes by SHA-256, source revision,
and byte span; paths alone and floating URLs cannot identify the source.
All clauses in a multi-clause file are individually identified and span-bound.
Replacing an inline clause with a file reference is an explicit source change.

Source bytes, selected language/profile, clause/requirement revision, compiled
declaration closure, adapter/tool/library pins, and anchor binding are explicit
derivation inputs. Duplicate authorities, stale source digests, ambiguous names,
and conflicting compiled declarations refuse before lowering. Generated OCL,
SysML, schemas, and solver text are read-only projections. No prose-to-formula
inference, automatic header migration, or second hand-authored model is implied.
Source provenance stays separate from FR-016's source-free semantic digest;
changing a language/profile must not be hidden by an equal expression digest.
The later frontend result contract owns those fields; this ADR creates no
generic evidence envelope.

### Eligibility is not proof or a migration obligation

A criterion is eligible for this tier only when its author can identify a
precise property, finite typed domain, named evaluation/operation boundary,
and selected supported language/profile without changing the criterion's
meaning. Eligibility may be proposed before a source clause exists; it is not
authorship, parsing, typing, lowering, backend support, or proof. A structural
inspection, operational procedure, or unformalized criterion keeps its
Inspection/Test/Analysis/Demonstration obligation and an explicit reason why
this formalization tier is not applicable or not yet available.

#58 must enumerate the criterion population before filtering and preserve
eligible/authored/parsed/typechecked/lowered/backend-supported states plus
proved, rejected, inconclusive, unavailable, and not-applicable distinctions.
A source or tool refusal is never silently removed from that population.
The #58 ticket's older "shared evidence contract" wording requires reconciliation
with PGM-01-R08/R09: a native domain result may be retained by Quoin, but this
packet authorizes no replacement generic evidence envelope or second store.

### Shared bounded semantic admission

These are proposed **frontend restrictions**, not changes to the IR's accepted
types. Every bound must originate in an explicit reviewed domain declaration
or named profile parameter; a backend's native width is not an authored bound.

| Concern | Proposed admission rule | IR construct protected |
|---|---|---|
| Numeric domain | Inclusive finite integer bounds within signed 64-bit representability; `reject` overflow only. Exact bounded rationals normalize before checking numerator and denominator bounds. No float, NaN, infinity, saturation, silent widening, or implicit integer-to-rational conversion. | FR-013 integer/rational; FR-014 equal complete operand types; FR-015 checked range |
| Literals/arithmetic | Contextually type a literal only when one exact declared operand type determines it and it fits. Admit `+`, `-`, `*`, unary numeric negate only with all intermediate ranges proved. Rational `/` additionally needs an independently proved nonzero divisor. Integer `/`, `div`, and `mod` are deferred: OCL result-type/rounding correspondence is not assumed. | FR-014 numeric operators; FR-015 nonzero and range obligations |
| Presence/null/invalid | IR option-none represents one explicitly declared missing-value channel. If authored absence and explicit null are distinct, preserve both through a qualified encoding or reject the field; never collapse them into one option. Invalid is a rejected evaluation possibility, not false or option-none. | FR-013 option; FR-015 definedness; FR-014 Boolean root |
| Value containment | Acyclic, fully typed records only; exact owner/field paths. Scalar/enum/text equality is admitted; record/object identity equality is deferred rather than replaced with structural equality. | FR-013 containment graph; FR-014 field access/equality |
| References | An opaque UUID may be compared as its declared scalar representation, but cannot become an object or foreign-key lookup. No `parent_id.version_number`, recursive inline parent record, ambient object store, or `allInstances()`. | FR-013 closed types/no cycles; FR-012 exact identity; FR-014 field-owner dependencies |
| Collections | Only explicitly ordered, duplicate-preserving finite sequences map to the IR collection. Maximum length must be declared and at most 10000 in this frontend. Set, Bag, OrderedSet, unbounded ranges, map/filter/flatten/closure and implicit collection conversions are deferred. | FR-013 bounded collection; FR-014 ordered items/quantifier domains; FR-016 sequence identity |
| Calls | First profile admits no user-defined calls, recursion, reflection, I/O, or ambient extension library. Declared pure calls may be added only after their executable semantics, qualification pins, and bound/definedness contracts are supplied. A signature alone is not an evaluator. | FR-013 pure-function declarations; FR-014 exact call signature; FR-015 call-result bounds |
| Result | Exactly a statically defined Boolean root. No nullable Boolean, partial result, exception recovery, or truthiness coercion. | FR-014 Boolean clause root; FR-015 obligations |

### Separate #54 reference and bounds decision

The current #54 acceptance example, `parent: ConfigVersion[0..1]` with an
Integer having only `min 1`, is not implementable by inserting an optional
recursive record into FR-013: the record graph must be acyclic and a minimum
alone is not a finite integer bound. Record this as a design gate, not a
failing fixture to repair by erasing the relationship or inventing a maximum.

Two explicit later alternatives require owner choice and separate specification:

1. **Recommended investigation: finite identity-preserving reference
   environment.** Keep identity-typed reference values distinct from records;
   supply an explicit, digest-bound finite object population and lookup
   contract. Validate identity uniqueness, target closure, optional absence,
   missing targets, and observation-specific object versions. Bound population,
   navigation depth, cycles, and lookup cost. A concrete implementation may
   lower qualified lookups to existing finite constructs, but only with an
   independently proved correspondence; an uninterpreted pure call or a UUID
   cast is not that correspondence. Unresolved external stores are refused.
2. **Alternative: add a first-class reference construct to the contract IR.**
   This requires a separately versioned type, wire/schema, dependency,
   canonicalization, runtime/codegen/solver and conformance change. It is not
   an extension a frontend may smuggle into FR-013's closed enum.

Neither alternative is admitted by this first source profile. Scalar and
acyclic-record binding can proceed once approved without pretending the
`ParentPrecedes` case works. Finite integer maxima, rational denominator bounds,
collection lengths and object-population bounds must be explicitly reviewed
model/profile data; no default inferred from `int`, UUID shape, current sample
values, backend width, or a source `min` annotation satisfies this gate.

### Evaluation instants and operation binding

Each executable clause requires a named anchor and an exact operation or
observation binding. A context type name by itself is insufficient.

| Source clause | IR anchor | Source `self` without `@pre` | Source `@pre` |
|---|---|---|---|
| Invariant | Named initialization or handler observation | Explicitly bound `current` snapshot | Refused |
| Operation precondition | Named pre anchor for the exact operation | `pre` snapshot | Refused as outside this source subset |
| Operation postcondition | Named post anchor for that same operation | `post` snapshot | `pre` snapshot of that operation |

Snapshots are immutable values bound by the producer; the frontend does not read
application state. Parameters are declared IR inputs observed at `current`.
A result, if used, is an explicitly declared post-only input of the operation's
result type; exposing it in a precondition or invariant is a binding error.
The exact parameter/result visibility and snapshot roles must be checked by the
#54 adapter, since the generic IR input declaration does not encode operations.
Missing signatures, omitted anchors, mismatched operations, `@pre` on a
parameter, and cross-observation guard reuse refuse (FR-012 anchor compatibility,
FR-014 observation policy, FR-015 exact-subject facts).

An invariant at one handler observation does not imply it holds at every
application state. A postcondition comparing one field does not establish full
immutability or absence of an update API.

### `ocl24-bounded-v1`: exact initial source subset

Admit an optional qualified package context and named `context T inv`,
`context T::operation(...) pre`, or `post` clauses. Context and signature names
must resolve uniquely through the compiled closure. Expressions admit Boolean,
integer, exact-decimal rational, text, and qualified enum literals; parentheses;
`self`/parameter/result references; acyclic non-optional record field paths;
the arithmetic admitted above; scalar `=`, `<>`, `<`, `<=`, `>`, `>=`; `not`,
`and`, `or`, `implies`; and the sequence operations below. No implicit casts,
overload guessing, type inference from observed application values, or new types
defined inside a clause.

For an explicitly optional value, comparison to `null` and
`oclIsUndefined()` may lower to option presence/absence **only after** proving
that invalid cannot occur in the operand. `oclIsInvalid()`, invalid literals,
unwrapping an optional value, navigation through it, and `oclAsType` refuse in
the first profile. A guarded optional navigation is deliberately not yet
accepted. `if/then/else`, `let`, `def`, `derive`, `init`, message expressions,
`allInstances`, tuples, user helper definitions, ambient `import`/`include`/
`library`, and every unnamed extension are outside the initial subset.
These exclusions protect FR-014's closed expression vocabulary and FR-015's
statically established definedness; source constructs cannot be replaced by
uninterpreted pure calls or truthy literals just to fit that vocabulary.

Sequence `size`, `isEmpty`, `notEmpty`, `forAll(x | predicate)`, and
`exists(x | predicate)` map to length, zero comparisons, and element-domain
quantifiers. Locals obey FR-014 scoping; an empty sequence has universal true
and existential false. `at` is deferred: source one-based indexing must not
silently become the IR's zero-based index. Sequence equality, construction,
and iterator operations other than the two named quantifiers are deferred.

**Boolean evaluation is deliberately stricter than accepting an OCL truth
table by resemblance.** `and`/`or` lower to total IR conjunction/disjunction.
Both operands must pass definedness under incoming declaration facts, with no
cross-operand guard facts. `implies` lowers to total `or(not(left), right)` and
requires the same independently defined operands. `not` preserves Boolean
negation. This gives the ordinary two-valued result on the admitted domain;
it does not equate OCL's null/invalid cases with IR short-circuit variants.
For example, a possible division by zero behind `x <> 0 implies ...` refuses
in this first subset rather than borrowing a short-circuit proof. Broadening
to safe conditional equivalence requires a separate reviewed correspondence
rule and differential fixtures. The same restriction applies inside quantifiers.

An OCL parser accepting a larger language does not enlarge this subset. Its AST
must pass an explicit allowlist/type/binding check before any IR is emitted.
No normative OCL grammar or upstream implementation is copied into this packet.
The behavioral language reference is [OMG OCL 2.4](https://www.omg.org/spec/OCL/2.4/PDF);
the subset and rejection rules here are independently proposed design choices.

### `fretish-bounded-ticks-v1`: later temporal candidate

The first candidate is a named, sampled, finite obligation at an explicit
trigger anchor: a Boolean response within a literal nonnegative bound in
discrete **ticks**. Each atomic variable resolves to a compiled, total Boolean
IR predicate and exact input/state-observation identity. No prose-derived
variables, opaque names, physical-time conversion, past operators, unbounded
operators, or SysML temporal-library stubs are admitted.

Proposed endpoints are closed `[0,H]`, tick zero is the trigger observation,
and an observation at H is in range. Require at least H+1 consecutive samples
for a complete evaluation window; a shorter window is **incomplete**, not a
successful check. The future tl-* bridge must qualify these choices with its
owner before enabling the profile. This is an admission rule for complete
windows, not a claim about every possible finite-prefix evaluator semantics.

Proposed maxima are H=10000 ticks and 10000 sampled variables, further bounded
by the aggregate IR budgets below. Boolean atoms have no null/invalid state;
missing samples are incomplete input. Temporal negation/connectives, if later
enabled, require the tl-* semantics and resource contract, not FR-014's static
Boolean operators reused as temporal ones. Standalone invariant and operation
postcondition clause kinds are outside this candidate, although their total
predicates may become explicitly bound atoms.

NASA's documented `ft-fin` CLI output can include a `LAST` alternative. Never
discard that alternative or relabel the output as strong bounded MLTL.
The future bridge must compare parsed native formulas with the proposed
complete-window semantics; unsupported operators, horizon mismatch, or an
unproved translation refuse. Temporal expressions cannot be inserted into a
made-up FR-014 node; #57 must specify their separate tl-* representation and
exact predicate-binding contract.

### `sysml2-bounded-invariant-v1`: later invariant candidate

The first candidate permits a named `assert constraint` containing a Boolean
literal or one scalar comparison over literals and qualified non-optional
record-field paths, in an explicitly bound model context. Structure is a
generated view of the compiled domain package, not a second authored model.
The only proposed comparison spellings are `==`, `!=`, `<`, `<=`, `>`, `>=`;
exact parser/type correspondence remains a qualification gate. No temporal,
operation pre/post, `@pre`, user functions, recursive navigation, implicit
collections, or structural redefinition is admitted.

Null/empty sequence, `??`, arithmetic, and Boolean connectives are deferred
until their native semantics are individually matched. A candidate scalar
comparison is admitted only with two total, equally typed singleton values;
an empty or multi-valued operand refuses, not false. This avoids equating the
source language's value multiplicity with an IR option or collection by name.
An opaque `rep language "ocl"` body is not a SysML evaluation of OCL; an OCL
clause would instead be separately selected and owned by the OCL frontend.

### Resource admission and canonical results

Proposed source limits are 65536 UTF-8 bytes per clause, 1048576 bytes across a
package's formal sources, 256 clauses, and source AST nesting at most 128.
Reject invalid UTF-8 and resource breaches before recursive external-parser
work wherever possible; external parser allocation/stack/time containment is
an additional adapter qualification gate, not guaranteed by a byte cap.

Lowered inputs still obey FR-014's 10000 expression nodes/depth 256 and
FR-019's aggregate 25000 semantic nodes/depth 256/10000 collection entries.
Budgets apply to the complete package/binding operation, not independently to
each clause; splitting a package must not multiply them. Literal text remains
bounded by FR-013's 1048576 Unicode scalars as well as the source-byte budget.
Reject the first over-limit construct without partial executable output.

To prevent finite-but-impractical nested quantification, propose a maximum
product of nested sequence bounds of 4096 and at most 1000000 statically
estimated expression-node visits per predicate evaluation. This is a stricter
frontend admission limit; it is not currently an IR or runtime guarantee.
Unprovable evaluation cost refuses. Future adapters must specify watchdog,
memory ceiling, cancellation and descendant-process containment separately;
a timeout is unavailable/incomplete evidence, never a false or proved clause.

Only successful typing, binding and definedness produce FR-016 canonical
bytes. No canonical semantic null, float, guessed declaration, silent numeric
coercion, ambient library identity, or partially lowered clause is emitted.

## Worked examples and expected acceptance

These are **synthetic/adapted profile probes**, not clauses found in the real
ConfigVersion specification. The original config-service FR-006 at
`6de6cd98718a8c8b516434172fb9b2dd4fe18978` has SHA-256
`b395b2c001744672a2a5f50e5ca9d975b90f764cb199e64e3ac5a77b7b62ca8a`.
It declares an unbounded `int`, nullable UUID `parent_id`, and opaque
`Dict[str, Any]`; its three criteria concern persistence-table mapping,
immutability, and JSONB storage. It has no formal fence, positive-version rule,
operation signature, or bounded temporal requirement.

For these probes only, propose a synthetic ConfigVersion **value** projection
with required `version_number: Integer[0,10000], overflow=reject`, a declared
`attemptUpdate(): Boolean` operation with pre/post snapshots, and separately
declared Boolean `accepted`/`persisted` observations. Bound 0 deliberately
permits a negative invariant witness; defining the domain as `[1,10000]` would
make that invariant true by construction. No production model or fixture is
changed, and this projection omits rather than pretends to type opaque data.

| Probe | OCL first-profile expectation | Later SysML candidate | Later FRETish candidate |
|---|---|---|---|
| P1: version is at least 1 at a named handler observation | Accept `context ConfigVersion inv VersionPositive: self.version_number >= 1` with the explicit synthetic domain | Accept candidate `assert constraint VersionPositive { version_number >= 1 }` in the generated bound context | Reject as a standalone invariant clause kind; a separately bound total predicate may later be an atom |
| P2: version number unchanged across synthetic attemptUpdate | Accept `context ConfigVersion::attemptUpdate(): Boolean post VersionUnchanged: self.version_number = self.version_number@pre` with the exact operation/anchors | Reject postcondition/`@pre`: no admitted mapping | Reject operation pre/post observation syntax; do not invent a temporal equivalence |
| P3: from an accepted trigger, persisted within 3 ticks | Reject temporal expression: no FR-014 temporal node | Reject temporal construct/stub | Accept candidate source `config_service shall within 3 ticks satisfy persisted` only with the explicit accepted anchor and qualified complete-window bridge |

"Accept" means the proposed semantic subset should accept after owner approval,
tool qualification and implementation. No row reports a native parser run or
existing production acceptance. Both later profiles remain disabled.

Expected independent controls: P1 at version 0 is false and at 1 true; P2 with
pre=4/post=5 is false and pre=4/post=4 true; P3 with first response at tick 3 is
true, first response at tick 4 is false over a complete four-sample window,
and fewer than four samples is incomplete. A response at tick 0 is in range.
Retain these as separately adjudicated oracle fixtures before implementing
the corresponding lowering; do not author compiler and oracle expectations
together as a way to pass.

Additional mandatory rejection/control pairs are listed in the companion
[qualification checklist](../../plan/issue-53-formal-profile-qualification.md).
P2 concerns one field only and cannot prove FR-006-AC-2's full immutability.
Neither P1 nor P3 is authored by FR-006. None establishes its table mapping or
JSONB criterion; their original verification methods remain in force.

## Tool candidates, licenses, and invocation boundaries

Source pins are research candidates, **not** exact binary/toolchain locks or
qualification receipts. No listed parser/typechecker was installed or run.
Each adapter remains outside the IR semantic crate and outside customer runtime.
License observations identify upstream notices; they do not decide legal
compatibility or authorize incorporation under PGM-01-R04/R05.

| Candidate | Immutable source candidate and observed license | Real documented invocation boundary | Still missing before implementation qualification |
|---|---|---|---|
| Eclipse OCL 6.24.0 Pivot/Complete OCL | Tag object `c3d6e5d2da8780b57946e5fafa0e817e2ec66d70`, commit `75f5bd7a5e53b0f57efacf7d69b52e825f419de7`; top-level LICENSE is EPL-2.0 | Java API setup `CompleteOCLStandaloneSetup.doSetup()` and resource-set model loading; this is not a supplied one-command standalone OCL validator | Exact Java, EMF/Xtext/OSGi/JAR and standard-library closure, hashes, adapter source/entrypoint, generated metamodel mapping and bounded AST export. Published setup docs name an OCL-2.5 standard library: OCL-2.4 correspondence must be established, never inferred from the product version. |
| NASA FRET 3.1.0 | Commit `58db455be35182a015e607232d9f4e3c86731932`; pinned README declares Apache-2.0 and links LICENSE.pdf (Git blob `7087df48b14c02dbd7d1b92eeef5a5960b147896`, 139221 bytes) | In `fret-electron`: `npm run --silent start-cli -- formalize -l ft-fin '<sentence>'`; documented logics are ft-inf/ft-fin/pt, not a documented MLTL CLI switch | Node/npm/dependency lock, all runtime/library artifacts and hashes, license inventory including LICENSE.pdf, parsed formula output boundary, exact tl-* schema/source pins, temporal translation and complete-window differential qualification |
| SysML v2 Pilot 2025-09 | Commit `0d5552b4ea440ce011905d73565e435fcac70382`; pinned LICENSE is LGPL v3 and README's source header specifies LGPL-3.0-or-later, **not** the unpinned EPL assumption in earlier research | Pinned README describes Java 21/Eclipse 2025-03 and `mvn clean package`; this is a build command, not a qualified headless validation invocation | Exact Java/Maven/target-platform/JAR/library closure, license inventory, stable SysML 2.0/KerML 1.0 library match, public parser/typechecker entrypoint, structured diagnostic/AST adapter, artifact hashes and native tests |

The stable SysML candidate is preferred for investigation over the newer
2026-07 pilot checkpoint (kernel 0.61.0 with 2.1/1.1-development libraries);
"latest" must not silently redefine the source language baseline. Eclipse
Complete OCL import/include/library extensions are forbidden by this profile,
even if the tool accepts them. No upstream grammar or generated parser is
vendored or adapted before the required origin/license review.

Sources inspected: [Eclipse Complete OCL documentation](https://help.eclipse.org/latest/topic/org.eclipse.ocl.doc/help/CompleteOCL.html),
[Pivot standalone setup](https://help.eclipse.org/latest/topic/org.eclipse.ocl.doc/help/PivotStandalone.html),
[pinned Eclipse license](https://github.com/eclipse-ocl/org.eclipse.ocl/blob/75f5bd7a5e53b0f57efacf7d69b52e825f419de7/LICENSE),
[pinned FRET CLI](https://github.com/NASA-SW-VnV/fret/blob/58db455be35182a015e607232d9f4e3c86731932/fret-electron/docs/_media/cli/cli.md),
[pinned FRET README](https://github.com/NASA-SW-VnV/fret/blob/58db455be35182a015e607232d9f4e3c86731932/README.md),
[pinned SysML README](https://github.com/Systems-Modeling/SysML-v2-Pilot-Implementation/blob/0d5552b4ea440ce011905d73565e435fcac70382/README.adoc),
and [pinned SysML license](https://github.com/Systems-Modeling/SysML-v2-Pilot-Implementation/blob/0d5552b4ea440ce011905d73565e435fcac70382/LICENSE).

## Consequences and alternatives

OCL-first supports useful typed non-temporal constraints without making the
reference environment, temporal semantics, or an entire SysML ecosystem a false
prerequisite for scalar work. Its deliberately strict first subset rejects
familiar guarded partial expressions; an accepted parser spelling does not
promise executable profile support. Full OCL, SysML-first structure migration,
automatic EARS formalization, and treating solver encodings as source would
all introduce larger authority or semantic questions and are not recommended.

The #53 owner decision and independent review remain mandatory before #54–#58
implementation. #54 must define the declaration/bounds/identity bridge, including
whether bounded references are a later IR extension or explicitly supplied
identity-preserving dereference environment. A UUID-to-record substitution is
not an acceptable shortcut. Native language qualification can proceed only in
a separately authorized followup; this packet runs no producer or hosted CI.

## Owner decision requested

Approve, amend, or defer the **direction**: OCL-based two-valued, statically
defined scalar/finite-sequence subset first; total Boolean evaluation with no
guarded partial navigation initially; the proposed source/resource bounds;
separate reference and temporal design/qualification gates; later FRETish then
SysML candidates. Approval of direction does not approve any unqualified tool,
third-party incorporation, publication, application migration, or release.
