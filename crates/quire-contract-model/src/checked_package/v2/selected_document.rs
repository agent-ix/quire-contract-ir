// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The number of a selected document's JSON view.

use quire_canonical::Number;

/// The numeric value used by the selected document's existing JSON view.
pub(super) fn json_number(number: Number<'_>) -> Option<serde_json::Number> {
    number
        .text()
        .parse::<i64>()
        .map(serde_json::Number::from)
        .or_else(|_| number.text().parse::<u64>().map(serde_json::Number::from))
        .ok()
        .or_else(|| serde_json::Number::from_f64(number.value()))
}
