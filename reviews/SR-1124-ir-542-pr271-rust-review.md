---
id: SR-1124
title: "rust review of PR 271: inexact-number scan in model_members.rs and the new refusal causes (IR-542)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-contract-ir@cb01b8277f92404d729f696baa91d8cfdd441695; git diff origin/main...cb01b82 (Rust: crates/quire-contract-model/src/checked_package/{common.rs,shared.rs,v2/mod.rs,v2/model_members.rs}, tests/it/checked_package_v2_{model_members,canonical_encoding,temporal}.rs; crates/quire-contract-model/Cargo.toml)"
review_set: subset
---
# SR-1124: rust review of PR 271: inexact-number scan in model_members.rs and the new refusal causes (IR-542)

## Summary

Ticket: IR-542. This is the rust-review lane of the code review. The schema has no `rust-review` analysis value, so the frontmatter records `code-review`. This review applies the rust-review checklist to the Rust diff of PR 271, with the repository's own conventions taking precedence. I checked idioms, panic surface, integer conversions, resource bounds on untrusted input, error mapping, test conventions and tracking tags, and the gates.

Measured in a throwaway worktree at cb01b82, since removed:

- `make fmt-check` passes.
- `make lint` passes: clippy `-D warnings` on the workspace, all targets, and on the model crate alone.
- The touched integration tests pass, 66 of them.

Checklist results:

- **Panic surface.** The new code has no `unwrap`, `expect`, indexing or `as` casts. Every integer conversion is `i64::try_from(..).unwrap_or(i64::MAX)` or `usize::try_from(..).unwrap_or(0)`, and the exponent arithmetic saturates. Padding in `is_whole_past_2_pow_53` is reached only when length + scale == 16, so the `"0".repeat(padding)` length is at most 15.
- **Untrusted input bounds.** The scan is linear in the document. It walks from the existing explicit heap stack, so a document of any depth is safe on any thread stack. Each number costs a few small heap allocations: the concatenated digits, the owned `digits` String, and the writer's `Vec`. The document byte limit bounds the total, so this is acceptable.
- **Error mapping.** A `Writer::number` error maps to `InexactNumber`. It is unreachable, because `quire_canonical::read` yields only finite doubles and 64 bytes is more than any double's text needs. The comment says this plainly. That makes it a closed refusal, not a silent fallback.
- **Types and naming.** `SelectionFailure::NumberPast2Pow53` is renamed to `InexactNumber { document_pointer, cause }`, and the one match site in v2/mod.rs is updated. `Spelling` derives `Debug, Eq, PartialEq` and holds canonical digits, so derived equality is value equality. The enum's new variants carry doc comments that cite QSpec FR-272, which matches the existing variants' style.
- **Tests.** The new tests follow the `tc_048_*` convention with `#[trace("TC-048", ...)]` and doc-comment tags. They assert whole structs, not single fields. The temporal module's exhaustive `cause_word` match gains the two arms. The tag targets are covered in SR-1125.
- **Manifest.** `serde_json` keeps its exact `=1.0.151` pin and gains a feature. A comment states why.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | `String::from_utf8_lossy(&written)` decodes bytes that are ASCII by construction, which makes it the wrong tool. If the invariant ever broke, the lossy decode would hide it by substituting U+FFFD and then refusing `inexact-number` for an unrelated reason. Use `std::str::from_utf8(&written)` and send `Err` down the same refusal path as the write error just above it. That keeps one closed failure path with no silent substitution. | crates/quire-contract-model/src/checked_package/v2/model_members.rs:1276 |

## Verdict

The Rust is idiomatic and panic-free. It saturates its arithmetic on untrusted exponents, and its allocation is bounded by the document byte limit. The single low finding is a nit about decoding the scratch writer's output. No unsafe code, blocking or locking is involved.

## Dispositions

Round 1 was reviewed at 3a34916a52918fb7ae1c1c96df4e40842b829c88. make fmt-check and make lint (both clippy lanes, -D warnings) pass.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed 3a34916 | `let exact = wrote && std::str::from_utf8(&written).is_ok_and(\|written\| spelled == Spelling::of(written)); (!exact).then_some(Cause::InexactNumber)`. A write error and a non-UTF-8 output now take the same closed refusal path, with no lossy substitution. |
