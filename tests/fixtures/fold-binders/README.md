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
