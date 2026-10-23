---
id: FR-056
title: "Consume the authoritative string-edge scan"
type: FR
relationships:
  - target: ix://agent-ix/quire-contract-ir/StR-003
    type: traces_to
  - target: ix://agent-ix/quire-contract-ir/FR-028
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/AD-004
    type: references
  - target: ix://agent-ix/quire-spec-language/FR-064
    type: depends_on
  - target: ix://agent-ix/ix-trace-rs/FR-004
    type: depends_on
---
# FR-056: Consume the authoritative string-edge scan

## Description

When a developer runs `make lint`, the Contract IR repository shall run the
authoritative external `qsl-string-edge` command over both production source
roots and propagate its failure. [QSL FR-064](ix://agent-ix/quire-spec-language/FR-064)
owns the detector, strict rules format and command interface. This requirement
owns the IR consumer invocation, edge inventory and falsifying controls under
IR-9, without a second detector or a QSL Cargo dependency.

## Inputs

- The repository base directory and the nonoverlapping crate source roots `src`
  and `crates/quire-contract-model/src`.
- One checked-in rules file in the strict FR-064 JSON format.
- The installed authoritative `qsl-string-edge` executable. An explicit executable
  path may select the same command for local installation or qualification.
- Function markers imported directly from the neutral `ix-trace-rs` crate.

## Outputs

The normal lint gate reports the command's source-relative file, one-based line
and enclosing-item diagnostics. A successful scan has exit zero. A completed
scan with violations has exit one; request, rule, read or parse errors have exit
two. A missing executable is a failing invocation, not a skipped check.

## Behavior

The repository shall invoke the command with the repository base, both explicit
crate roots and its checked-in rules file. `make lint` shall depend on this
invocation through ordinary Make failure propagation; the existing CI lint
prerequisite shall remain in place. The gate shall not accept a partial report,
ignored exit, conditional skip, success fallback or filtered detector output as
a completed clean scan.

The repository shall obtain `string_edge` directly from `ix-trace-rs` in every
package whose production functions use the marker. The neutral owner's
[FR-004](ix://agent-ix/ix-trace-rs/FR-004) defines its argument-free identity
expansion and compile error for arguments. IR shall define no replacement,
local copy or compatibility re-export of that marker. The IR repository shall not add the external scanner as a normal, build,
optional or dev Cargo dependency of either package; [FR-028](../../model/functional/FR-028-separate-cycle-free-contract-model.md)
remains unchanged.

A marker shall identify only a reviewed intake conversion from external text to
an owned typed value, or its typed refusal. It shall not exempt an entire file,
crate or a broad validator's post-intake decisions. A macro-generated conversion
requires review of the actual emitted function; a syntactic scan does not expand
Rust macros or prove their compiler resolution.

The rules shall not suppress post-intake closed-vocabulary dispatch. FR-064's
branch-gating allow-list rejection remains authoritative. Open caller identities
and selection keys are not closed vocabularies merely because they are strings:
the positive control shall preserve a comparison between two caller-supplied
keys without treating it as a wire-enum decode. A rule for a detector-visible
open-text comparison is admissible only at the non-branching sinks allowed by
FR-064; no branch-gating exception is introduced here.

When the scan exposes an existing decision, its owning source/specification
slice shall classify and remedy it before the normal gate can pass. No baseline
allowance, broad marker or deleted root may turn an unresolved violation into a
clean report. Raw untrusted-byte grammar checks remain distinct from decisions
on an already admitted node. Typed-term and lowering migrations retain their
existing semantic and first-refusal obligations.

## Scope Limits

The detector recognizes the syntactic cases defined by FR-064, including literal
strings, same-crate string constants and supported string methods. It does not
resolve variable-to-variable string comparisons, external constants, indirect
dataflow or macro expansions. The open-key positive control verifies absence of
a false report; it does not establish general dataflow classification. A clean
scan does not establish every closed vocabulary has one intake conversion or
that all later matches are exhaustive. The broader Q-6 gap in
[AD-004](../../assurance/AD-004-checked-package-seam.md) retains its own scope.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-056-AC-1 | The normal `make lint` dependency graph invokes the real external command with the repository base, exactly the two nonoverlapping production roots and the checked-in strict rules. A clean controlled consumer completes the invocation; the ordinary `ci` target retains `lint`. | Test |
| FR-056-AC-2 | A real unmarked post-intake `match` on a closed node-tag wire spelling, introduced in a controlled consumer production file, makes that same gate entry point fail with the command's exit-one violation and expected file/item diagnostic. Restoring the enum-based control makes the scan pass. | Test |
| FR-056-AC-3 | A genuine intake function marked with `ix_trace_rs::string_edge` compiles against the authoritative neutral crate and preserves its conversion/refusal behavior. The command accepts this marked edge while still reporting an adjacent unmarked closed-string decision. A look-alike qualified marker and comment-only marker do not exempt the adverse decision. | Test |
| FR-056-AC-4 | An unchanged comparison of two supplied identity/selection keys passes the controlled scan, and an allowed detector-visible non-branching open-text comparison passes only with its valid rule. Removing that rule reports it; attempting to allow-list a branch-gating wire decision is a rule error rather than a clean scan. These controls state the detector's variable-to-variable limitation. | Test |
| FR-056-AC-5 | A missing executable, malformed rules, missing source root or malformed Rust source makes the normal scan prerequisite fail. Request/rule/read/parse errors preserve the real command's exit-two diagnostic, and no path reports clean after failure or partial execution. | Test |
| FR-056-AC-6 | Every production package that marks an intake function resolves the neutral marker from `ix-trace-rs`, and Cargo metadata confirms neither IR package adds a dependency in any kind on QSL or the scanner. Existing cycle-free metadata and intake conformance controls remain intact. | Test |
| FR-056-AC-7 | Each marked function is an actual intake conversion; the checked-in rules contain no branch-dispatch suppression, file/crate exemption or copied scanner logic. Existing reported post-intake decisions retain their owning remediation until removed from the real report. | Inspection |
| FR-056-AC-8 | The actual repository command completes with no unresolved occurrences across both roots, without removing a root, filtering diagnostics, introducing a local marker or weakening FR-064's rule checks. A fixture-only success cannot satisfy this criterion. | Test |

## Qualification Status

PLANNED: this specification precedes consumer implementation. The authoritative
marker owner and reusable CLI already exist. Initial source inventory exposes
unresolved occurrences; this document does not qualify a clean repository scan,
replace their semantic migration or claim Q-6 complete. Integration and final
review must retain that distinction.

## Dependencies

- [TC-460](../matrix/TC-460-string-edge-consumer-gate.md) defines consumer controls.
- [FR-028](../../model/functional/FR-028-separate-cycle-free-contract-model.md)
  owns the dependency boundary.
- [QSL FR-064](ix://agent-ix/quire-spec-language/FR-064) owns the detector and
  documented `cargo install --git https://github.com/agent-ix/quire-spec-language --branch main qsl-string-edge`
  route. Installation is tooling setup, not an IR Cargo dependency.
- [ix-trace-rs FR-004](ix://agent-ix/ix-trace-rs/FR-004) owns the marker.
