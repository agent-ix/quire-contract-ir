// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! The raw wire body bridge: the body of a node before the typed decode is a
//! `serde_json::Value`, and the declaration check asks whether it is the empty
//! aggregate. Nothing here is charged; the whole module is removed when the
//! node bodies move to `CheckedJson` (group A).

use super::model_members::EmptyAggregateBody;
use serde_json::{json, Value};

impl EmptyAggregateBody for Value {
    fn is_empty_aggregate(&self) -> bool {
        *self == json!({"term": "aggregate", "members": []})
    }
}
