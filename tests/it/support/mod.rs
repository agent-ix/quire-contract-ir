// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Shared fixtures and helpers for the merged `it` integration test binary.
//!
//! Each submodule used to be duplicated (via `#[path = "support/…"]`) into
//! every integration-test binary that needed it, so each binary linked its
//! own private copy. Now there is exactly one copy per submodule, shared by
//! every test module declared in `tests/it/main.rs`.

pub mod checked_package;
pub mod native_protocol;
pub mod result_fixture;
pub mod temporal_fixture;
pub mod v2_handoff;
