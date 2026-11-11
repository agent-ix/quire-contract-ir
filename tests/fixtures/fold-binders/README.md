# Fold binder producer fixture

`integer-fold.json` is genuine QSL checked-package output from
`qsl-package/src/emit/tests.rs`, `fold_with_both_authored_binders_reads_back`.
It was emitted without modification from this Agent-IX regression source:

```qsl
language "ix:native" edition "1-draft";
profile v = "quire.model.complete/v1";
type Total = Integer;
function accumulate using v(xs: Sequence<Integer>): Integer pure {
    fold<Total>(acc, x in xs: acc + x, identity: 0)
}
```

The producer uses library name `pkg`, source authority `a` and identity `u`.
There are no selected model documents or dependency packages. The caller
supports `quire.value.complete/v1`. The bytes exercise the strict public IR
reader; no QSL implementation or schema is embedded. This Agent-IX-authored
test output is maintained with the repository's AGPL-3.0-or-later tests.

`monitor-base.json` preserves the original complete 1,114-node checked package
exported from the failing `qsl-analyze/tests/it/monitor_authority.rs`
`genuine_tick_authority_retains_complete_original_values_and_same_emission`
reproduction before monitor-plan attachment. No semantic section was removed or
rewritten. Its fold at node 359 has accumulator `a`, element `x`, and a record
step of type `ActsAcc`. The source uses `monitor_root::authored` with
`always (holds(c.versionNumber = 2) implies eventually[0 ms, 3 ms] holds(c.versionNumber = 3))`
and changes the time declaration to
`time M tick M::ConfigVersion::attemptUpdate period 2 ms unit ms;`.
The surrounding functions are QSL's genuine generated monitor entries.

`config-version-domain.json` is a domain model document authored natively in
this repository (it is not a copy of any other repository's file). It declares
the `example/config-version` package: a `ConfigVersion` object type with
`versionNumber` and `parent` fields and an `attemptUpdate` operation, so the
monitor-base tick source resolves its model and operation names. Only this
evidence document and `quire.value.complete/v1` support are supplied; the tests
do not rewrite the original checked-package bytes. It is maintained with the
repository's AGPL-3.0-or-later tests.
