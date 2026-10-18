// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The QSL-emitted selected-population fixture runs in this explicit lane.

#[expect(
    dead_code,
    reason = "this explicit lane uses two helpers from the shared default-test module"
)]
#[path = "../it/support/mod.rs"]
mod support;

#[path = "../it/selected_population.rs"]
mod selected_population;
