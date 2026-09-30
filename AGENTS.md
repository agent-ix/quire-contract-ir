See CLAUDE.md.

## Hash / digest / pin antipattern: do not introduce

Hashes, digests, SHAs, pins, checksum catalogs and records that track files, versions or
tools are an antipattern and have been removed from this repository. Do not introduce
any new use of them. If you find one, remove it as part of the change. The only hash
that stays is a canonical identity digest that binds a proof to the exact content it
proved. Package versions live in Cargo.toml / package.json and their lockfiles only;
reports name the app version they ran.
