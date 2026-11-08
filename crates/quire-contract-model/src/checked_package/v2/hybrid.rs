//! QSpec FR-481 hybrid node wire shapes and model joins.

use super::graph_storage::GraphIndex;
use super::native_join_budget::PackageReadStage;
use super::stage_storage::{self, Refusal, Site};
use super::{
    CheckedNodeKind, CheckedTerm, HybridForm, LiteralKind,
    RawSemanticNodeV2 as CheckedSemanticNodeV2, ScalarTypeForm, TemporalForm, TermNode, ValueForm,
    WorkMeter,
};
use crate::checked_package::common::{Step, Trail, ValidationFailure};
use crate::checked_package::shared::{
    CheckedNodeId, CheckedNodeIdOrder, CheckedNodePathSegment, CheckedOccurrenceRole,
    CheckedPackageRefusalCause as Cause, CheckedPackageRefusalCode as Code,
};
use ix_trace_rs::string_edge;

/// Closed hybrid schema positions reach thirteen steps (a variable class's
/// literal value). Indices change positions, never depth; term bodies are
/// inspected through their retained document handles, not recursively here.
#[derive(Clone, Copy)]
struct Path {
    steps: [Step<'static>; 14],
    len: usize,
    stage: PackageReadStage,
}
impl Path {
    fn key(mut self, key: &'static str) -> Self {
        self.steps[self.len] = Step::Key(key);
        self.len += 1;
        self
    }
    fn index(mut self, index: usize) -> Self {
        self.steps[self.len] = Step::Index(index);
        self.len += 1;
        self
    }
    fn trail(&self) -> Trail<'_> {
        Trail::Base(&self.steps[..self.len])
    }
    fn holder(&self) -> usize {
        match self.steps[..self.len] {
            [Step::Key("semantic_graph"), Step::Key("nodes"), Step::Index(index), ..] => index,
            _ => unreachable!("hybrid positions are graph-node positions"),
        }
    }
    fn site(&self) -> Site<'_> {
        Site::new(self.stage, self.holder(), "hybrid", self.trail())
    }
}
fn node_path(position: usize) -> Path {
    Path {
        steps: [Step::Index(0); 14],
        len: 0,
        stage: PackageReadStage::Graph,
    }
    .key("semantic_graph")
    .key("nodes")
    .index(position)
}
fn malformed(at: Path, meter: &mut WorkMeter) -> ValidationFailure {
    stage_storage::refuse(
        meter,
        &at.site(),
        Refusal::new(Code::MalformedWire, at.trail()),
    )
}
fn work(meter: &mut WorkMeter, at: Path, amount: u64) -> Result<(), ValidationFailure> {
    stage_storage::work(meter, &at.site(), amount)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HybridVariableClass {
    Clock,
    Hybrid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HybridBindingRole {
    Parameters,
    Kind,
    Trigger,
    Guard,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HybridActivationKind {
    Each,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HybridTimeKind {
    ModelTime,
    ModelSteps,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HybridTemporalOperation {
    Holds,
}

fn term_members(term: &CheckedTerm) -> &[CheckedTerm] {
    match term {
        CheckedTerm::Aggregate { members } => members,
        _ => &[],
    }
}

fn term_value(term: &CheckedTerm) -> Option<&CheckedTerm> {
    match term {
        CheckedTerm::Binding { value, .. } => Some(value),
        _ => None,
    }
}

fn binding_role(term: &CheckedTerm) -> Option<HybridBindingRole> {
    let CheckedTerm::Binding { name, .. } = term else {
        return None;
    };
    decode_word(name)
}

trait ClosedWord: Sized {
    fn from_word(word: &str) -> Option<Self>;
}
impl ClosedWord for HybridVariableClass {
    fn from_word(word: &str) -> Option<Self> {
        match word {
            "clock" => Some(Self::Clock),
            "hybrid" => Some(Self::Hybrid),
            _ => None,
        }
    }
}
impl ClosedWord for HybridBindingRole {
    fn from_word(word: &str) -> Option<Self> {
        match word {
            "parameters" => Some(Self::Parameters),
            "kind" => Some(Self::Kind),
            "trigger" => Some(Self::Trigger),
            "guard" => Some(Self::Guard),
            _ => None,
        }
    }
}
impl ClosedWord for HybridActivationKind {
    fn from_word(word: &str) -> Option<Self> {
        (word == "each").then_some(Self::Each)
    }
}
impl ClosedWord for HybridTimeKind {
    fn from_word(word: &str) -> Option<Self> {
        match word {
            "model-time" => Some(Self::ModelTime),
            "model-steps" => Some(Self::ModelSteps),
            _ => None,
        }
    }
}
impl ClosedWord for HybridTemporalOperation {
    fn from_word(word: &str) -> Option<Self> {
        (word == "quire.op.temporal.holds").then_some(Self::Holds)
    }
}
fn decode_word<T: ClosedWord>(wire: &str) -> Option<T> {
    T::from_word(wire)
}
fn literal_word_as<T: ClosedWord>(term: &CheckedTerm) -> Option<T> {
    let CheckedTerm::Literal { value, .. } = term else {
        return None;
    };
    decode_word(value.as_str()?)
}

fn model_variable_class(term: &CheckedTerm, slot: usize) -> Option<HybridVariableClass> {
    let variables = term_members(term).first().and_then(term_value)?;
    let variable = term_members(variables).get(slot)?;
    let class = term_members(variable).get(1).and_then(term_value)?;
    literal_word_as(class)
}

fn application_arguments(term: &CheckedTerm) -> Option<&[CheckedTerm]> {
    match term {
        CheckedTerm::Application { arguments, .. } => Some(arguments),
        _ => None,
    }
}

fn reference_target(term: &CheckedTerm) -> Option<&CheckedNodeId> {
    match term {
        CheckedTerm::Reference { target } => Some(target),
        _ => None,
    }
}

fn is_rational_one(term: &CheckedTerm) -> bool {
    matches!(
        term,
        CheckedTerm::Literal {
            value_kind: LiteralKind::Rational,
            value,
            ..
        } if value.as_str().is_some_and(|text| text == "1")
    )
}

fn members(value: &CheckedTerm) -> &[CheckedTerm] {
    term_members(value)
}

fn bound(value: &CheckedTerm, index: usize) -> &CheckedTerm {
    term_value(&members(value)[index]).expect("strict hybrid binding")
}

fn target(value: &CheckedTerm) -> Option<&CheckedNodeId> {
    reference_target(value)
}

fn number(value: &CheckedTerm) -> Option<usize> {
    match value {
        CheckedTerm::Literal { value, .. } => value.as_str()?.parse().ok(),
        _ => None,
    }
}

/// GraphIndex's borrowed entry iterator forwards `nth` to its slice iterator,
/// so each probe is constant-time. Its structural comparison may spill only
/// through the actual caller meter, including an unresolved key.
fn index_position(
    index: &GraphIndex<'_>,
    key: &CheckedNodeId,
    meter: &mut WorkMeter,
) -> Result<Option<usize>, ValidationFailure> {
    let (mut low, mut high) = (0, index.len());
    while low < high {
        let middle = low + (high - low) / 2;
        let (candidate, _) = index
            .iter()
            .nth(middle)
            .expect("index length bounds its probe");
        if meter.compare_ids(candidate, key)?.is_lt() {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    let Some((candidate, position)) = index.iter().nth(low) else {
        return Ok(None);
    };
    Ok(meter
        .compare_ids(candidate, key)?
        .is_eq()
        .then_some(*position))
}
fn node_at<'a>(
    target: &CheckedNodeId,
    nodes: &'a [CheckedSemanticNodeV2],
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter,
) -> Result<Option<(usize, &'a CheckedSemanticNodeV2)>, ValidationFailure> {
    Ok(index_position(index, target, meter)?
        .and_then(|position| nodes.get(position).map(|node| (position, node))))
}
fn optional_node<'a>(
    target: Option<&CheckedNodeId>,
    nodes: &'a [CheckedSemanticNodeV2],
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter,
) -> Result<Option<(usize, &'a CheckedSemanticNodeV2)>, ValidationFailure> {
    match target {
        Some(key) => node_at(key, nodes, index, meter),
        None => Ok(None),
    }
}

fn issue(
    code: Code,
    cause: Option<Cause>,
    at: Path,
    node: &CheckedSemanticNodeV2,
    meter: &mut WorkMeter,
) -> ValidationFailure {
    let mut refusal = Refusal::new(code, at.trail()).at_node(&node.node_id);
    refusal.cause = cause;
    stage_storage::refuse(meter, &at.site(), refusal)
}

fn term_type(
    key: &CheckedNodeId,
    required: ScalarTypeForm,
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter,
) -> Result<bool, ValidationFailure> {
    let Some((position, node)) = node_at(key, nodes, index, meter)? else {
        return Ok(false);
    };
    if !matches!(
        kinds.get(position),
        Some(CheckedNodeKind::Expression(_) | CheckedNodeKind::Value(ValueForm::Literal))
    ) {
        return Ok(false);
    }
    Ok(
        index_position(index, &node.semantic_type, meter)?.and_then(|position| kinds.get(position))
            == Some(&CheckedNodeKind::ScalarType(required)),
    )
}
fn rational_or_variable(
    key: &CheckedNodeId,
    variables: &[&CheckedNodeId],
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter,
) -> Result<bool, ValidationFailure> {
    if find_id(variables, key, meter, |key| *key)?.is_some() {
        return Ok(true);
    }
    term_type(key, ScalarTypeForm::Rational, nodes, kinds, index, meter)
}

fn same_id(
    actual: Option<&CheckedNodeId>,
    expected: &CheckedNodeId,
    meter: &mut WorkMeter,
) -> Result<bool, ValidationFailure> {
    match actual {
        Some(actual) => Ok(meter.compare_ids(actual, expected)?.is_eq()),
        None => Ok(false),
    }
}

fn numeric_or_boolean_type(
    type_id: &CheckedNodeId,
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter,
    at: Path,
) -> Result<bool, ValidationFailure> {
    let base = scalar_base_type(type_id, nodes, kinds, index, meter, at)?;
    let Some(base) = base else {
        return Ok(false);
    };
    Ok(index_position(index, base, meter)?
        .and_then(|position| kinds.get(position))
        .is_some_and(|kind| {
            matches!(
                kind,
                CheckedNodeKind::ScalarType(
                    ScalarTypeForm::Boolean
                        | ScalarTypeForm::Integer
                        | ScalarTypeForm::Rational
                        | ScalarTypeForm::Decimal
                        | ScalarTypeForm::Float32
                        | ScalarTypeForm::Float64
                )
            )
        }))
}

fn scalar_base_type<'a>(
    type_id: &'a CheckedNodeId,
    nodes: &'a [CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter,
    at: Path,
) -> Result<Option<&'a CheckedNodeId>, ValidationFailure> {
    meter.at_node(at.holder(), "hybrid");
    let mut current = type_id;
    for _ in 0..=nodes.len() {
        work(meter, at, 1)?;
        let Some((position, node)) = node_at(current, nodes, index, meter)? else {
            return Ok(None);
        };
        match kinds.get(position) {
            Some(CheckedNodeKind::ScalarType(_)) => return Ok(Some(current)),
            Some(CheckedNodeKind::BoundedDomain(_)) => current = &node.semantic_type,
            _ => return Ok(None),
        }
    }
    Ok(None)
}

fn model_pointer(position: usize, binding: usize) -> Path {
    node_path(position)
        .key("body")
        .key("members")
        .index(binding)
        .key("value")
}

fn parameter_name_level(term: &CheckedTerm) -> Option<(&str, usize)> {
    let body = members(term);
    let name = match term_value(body.first()?)? {
        CheckedTerm::Literal { value, .. } => value.as_str()?,
        _ => return None,
    };
    let level = number(term_value(body.get(1)?)?)?;
    Some((name, level))
}

/// A canonical graph key is built from the authenticated node-index DAG;
/// every operand edge goes to another graph node. Its continuation depth is
/// therefore at most the complete graph's node count, including imported
/// dependency nodes. Inline wire paths contain indices, not recursive IDs.
/// This bound is established at the strict validators' graph-stage entry.
struct IdSort<'a> {
    depth_bound: usize,
    order: Option<CheckedNodeIdOrder<'a>>,
}
impl<'a> IdSort<'a> {
    fn new(depth_bound: usize) -> Self {
        Self {
            depth_bound,
            order: None,
        }
    }
    fn prepare(
        &mut self,
        nested: bool,
        meter: &mut WorkMeter,
    ) -> Result<Option<&mut CheckedNodeIdOrder<'a>>, ValidationFailure> {
        if nested && self.order.is_none() {
            meter.reserve(CheckedNodeIdOrder::bytes(self.depth_bound))?;
            self.order = Some(CheckedNodeIdOrder::with_capacity(self.depth_bound));
        }
        Ok(self.order.as_mut())
    }
}
fn sort_id_rows<'a, T: Ord>(
    rows: &mut [(&'a CheckedNodeId, T)],
    meter: &mut WorkMeter,
    sort: &mut IdSort<'a>,
) -> Result<(), ValidationFailure> {
    if rows.len() < 2 {
        return Ok(());
    }
    let nested = rows.iter().any(|(key, _)| {
        key.path
            .iter()
            .any(|step| matches!(step, CheckedNodePathSegment::Node(_)))
    });
    let mut flat = CheckedNodeIdOrder::with_capacity(0);
    let order = sort.prepare(nested, meter)?.unwrap_or(&mut flat);
    rows.sort_unstable_by(|a, b| order.compare_bounded(a.0, b.0).then_with(|| a.1.cmp(&b.1)));
    Ok(())
}
fn sort_ids<'a>(
    rows: &mut [&'a CheckedNodeId],
    meter: &mut WorkMeter,
    sort: &mut IdSort<'a>,
) -> Result<(), ValidationFailure> {
    if rows.len() < 2 {
        return Ok(());
    }
    let nested = rows.iter().any(|key| {
        key.path
            .iter()
            .any(|step| matches!(step, CheckedNodePathSegment::Node(_)))
    });
    let mut flat = CheckedNodeIdOrder::with_capacity(0);
    let order = sort.prepare(nested, meter)?.unwrap_or(&mut flat);
    rows.sort_unstable_by(|a, b| order.compare_bounded(a, b));
    Ok(())
}
fn dedup_ids<T>(
    rows: &mut Vec<T>,
    key: impl Fn(&T) -> &CheckedNodeId,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    if rows.is_empty() {
        return Ok(());
    }
    let mut last = 0;
    for read in 1..rows.len() {
        if !meter
            .compare_ids(key(&rows[last]), key(&rows[read]))?
            .is_eq()
        {
            last += 1;
            rows.swap(last, read);
        }
    }
    rows.truncate(last + 1);
    Ok(())
}
fn lower_id<T>(
    rows: &[T],
    key: &CheckedNodeId,
    meter: &mut WorkMeter,
    key_of: impl Fn(&T) -> &CheckedNodeId,
) -> Result<usize, ValidationFailure> {
    let (mut low, mut high) = (0, rows.len());
    while low < high {
        let middle = low + (high - low) / 2;
        if meter.compare_ids(key_of(&rows[middle]), key)?.is_lt() {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    Ok(low)
}
fn find_id<T>(
    rows: &[T],
    key: &CheckedNodeId,
    meter: &mut WorkMeter,
    key_of: impl Fn(&T) -> &CheckedNodeId,
) -> Result<Option<usize>, ValidationFailure> {
    let at = lower_id(rows, key, meter, &key_of)?;
    if at < rows.len() && meter.compare_ids(key_of(&rows[at]), key)?.is_eq() {
        Ok(Some(at))
    } else {
        Ok(None)
    }
}

/// Fixed sorted lookup keys and reusable visit stamps. Every dependency key
/// is included once, including unresolved keys; each DFS therefore retains
/// its old duplicate suppression and work-unit order without tree insertion.
struct Visits<'a> {
    rows: Vec<(&'a CheckedNodeId, u64)>,
    epoch: u64,
    sort: IdSort<'a>,
}
impl<'a> Visits<'a> {
    fn fork(&self, meter: &mut WorkMeter) -> Result<Self, ValidationFailure> {
        let mut rows = meter.vec(self.rows.len())?;
        rows.extend(self.rows.iter().map(|(key, _)| (*key, 0)));
        Ok(Self {
            rows,
            epoch: 0,
            sort: IdSort::new(self.sort.depth_bound),
        })
    }

    fn new(
        nodes: &'a [CheckedSemanticNodeV2],
        meter: &mut WorkMeter,
    ) -> Result<Self, ValidationFailure> {
        let count = nodes.iter().map(|node| 1 + node.dependencies.len()).sum();
        let mut rows = meter.vec(count)?;
        for node in nodes {
            rows.push((&node.node_id, 0));
            rows.extend(node.dependencies.iter().map(|key| (key, 0)));
        }
        let mut sort = IdSort::new(nodes.len());
        sort_id_rows(&mut rows, meter, &mut sort)?;
        dedup_ids(&mut rows, |row| row.0, meter)?;
        Ok(Self {
            rows,
            epoch: 0,
            sort,
        })
    }
    fn begin<'s>(
        &'s mut self,
        roots: &[&'a CheckedNodeId],
        meter: &mut WorkMeter,
    ) -> Result<Seen<'s, 'a>, ValidationFailure> {
        if self.epoch == u64::MAX {
            for (_, stamp) in &mut self.rows {
                *stamp = 0;
            }
            self.epoch = 0;
        }
        self.epoch += 1;
        let mut missing = 0;
        for &key in roots {
            if find_id(&self.rows, key, meter, |row| row.0)?.is_none() {
                missing += 1;
            }
        }
        let mut extra = meter.vec(missing)?;
        for &key in roots {
            if find_id(&self.rows, key, meter, |row| row.0)?.is_none() {
                extra.push((key, false));
            }
        }
        sort_id_rows(&mut extra, meter, &mut self.sort)?;
        dedup_ids(&mut extra, |row| row.0, meter)?;
        Ok(Seen {
            visits: self,
            extra,
        })
    }
}
struct Seen<'s, 'a> {
    visits: &'s mut Visits<'a>,
    extra: Vec<(&'a CheckedNodeId, bool)>,
}
impl Seen<'_, '_> {
    fn insert(
        &mut self,
        key: &CheckedNodeId,
        meter: &mut WorkMeter,
    ) -> Result<bool, ValidationFailure> {
        match find_id(&self.visits.rows, key, meter, |row| row.0)? {
            Some(slot) => {
                let old = self.visits.rows[slot].1;
                self.visits.rows[slot].1 = self.visits.epoch;
                Ok(old != self.visits.epoch)
            }
            None => {
                let slot = find_id(&self.extra, key, meter, |row| row.0)?
                    .expect("a visited key is a root or a graph dependency");
                let fresh = !self.extra[slot].1;
                self.extra[slot].1 = true;
                Ok(fresh)
            }
        }
    }
}

struct NamedRead<'a> {
    key: (&'a str, usize),
    ty: Option<&'a CheckedNodeId>,
}
struct ReadScope<'a, 's> {
    variables: &'s [&'a CheckedNodeId],
    named_reads: Vec<NamedRead<'a>>,
}
impl ReadScope<'_, '_> {
    fn contains(
        &self,
        key: &CheckedNodeId,
        meter: &mut WorkMeter,
    ) -> Result<bool, ValidationFailure> {
        Ok(find_id(self.variables, key, meter, |key| *key)?.is_some())
    }
    fn named(&self, name: &str, level: usize) -> Option<&CheckedNodeId> {
        self.named_reads
            .binary_search_by_key(&(name, level), |row| row.key)
            .ok()
            .and_then(|slot| self.named_reads[slot].ty)
    }
}
struct ParameterRead<'a> {
    key: (&'a str, usize),
    id: &'a CheckedNodeId,
    ordinal: usize,
}
fn parameter_reads_for<'a>(
    reads: &'a [ParameterRead<'a>],
    key: (&str, usize),
) -> &'a [ParameterRead<'a>] {
    let start = reads.partition_point(|row| row.key < key);
    let end = reads.partition_point(|row| row.key <= key);
    &reads[start..end]
}

#[derive(Clone, Copy)]
struct DecodedNodes<'a> {
    kinds: &'a [CheckedNodeKind],
    terms: &'a [Option<CheckedTerm>],
    index: &'a GraphIndex<'a>,
}

impl<'a> DecodedNodes<'a> {
    fn term(&self, position: usize) -> &'a CheckedTerm {
        self.terms
            .get(position)
            .and_then(Option::as_ref)
            .expect("validated hybrid term")
    }
}

fn claim_parameter_reads<'a>(
    roots: Vec<&'a CheckedNodeId>,
    nodes: &'a [CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    decoded: DecodedNodes<'a>,
    meter: &mut WorkMeter,
    at: Path,
    visits: &mut Visits<'a>,
) -> Result<Vec<ParameterRead<'a>>, ValidationFailure> {
    meter.at_node(at.holder(), "hybrid");
    let mut reads = meter.vec(nodes.len())?;
    let mut seen = visits.begin(&roots, meter)?;
    let mut pending = roots;
    while let Some(key) = pending.pop() {
        if !seen.insert(key, meter)? {
            continue;
        }
        work(meter, at, 1)?;
        let Some((position, node)) = node_at(key, nodes, decoded.index, meter)? else {
            continue;
        };
        match kinds.get(position) {
            Some(CheckedNodeKind::Value(ValueForm::Parameter)) => {
                if let Some(key_name) = parameter_name_level(decoded.term(position)) {
                    let ordinal = reads.len();
                    reads.push(ParameterRead {
                        key: key_name,
                        id: key,
                        ordinal,
                    });
                }
            }
            Some(CheckedNodeKind::Function(_)) => {}
            _ => {
                for dependency in &node.dependencies {
                    meter.push(&mut pending, dependency)?;
                }
            }
        }
    }
    reads.sort_unstable_by_key(|row| (row.key, row.ordinal));
    Ok(reads)
}

fn reads_allowed<'a>(
    start: &'a CheckedNodeId,
    allowed: &ReadScope<'_, '_>,
    nodes: &'a [CheckedSemanticNodeV2],
    decoded: DecodedNodes<'a>,
    at: Path,
    source: &CheckedSemanticNodeV2,
    meter: &mut WorkMeter,
    visits: &mut Visits<'a>,
) -> Result<(), ValidationFailure> {
    meter.at_node(at.holder(), "hybrid");
    let kinds = decoded.kinds;
    let index = decoded.index;
    let mut seen = visits.begin(&[start], meter)?;
    let mut pending = meter.vec(1)?;
    pending.push(start);
    let mut invalid_term = false;
    while let Some(key) = pending.pop() {
        if !seen.insert(key, meter)? {
            continue;
        }
        work(meter, at, 1)?;
        let Some((position, node)) = node_at(key, nodes, index, meter)? else {
            return Err(issue(
                Code::MissingDeclaration,
                Some(Cause::MissingName),
                at,
                source,
                meter,
            ));
        };
        match kinds.get(position) {
            Some(CheckedNodeKind::Value(ValueForm::Parameter)) => {
                if !allowed.contains(key, meter)? {
                    let expected = parameter_name_level(decoded.term(position))
                        .and_then(|(name, level)| allowed.named(name, level));
                    match expected {
                        Some(expected)
                            if same_id(
                                scalar_base_type(
                                    &node.semantic_type,
                                    nodes,
                                    kinds,
                                    index,
                                    meter,
                                    at,
                                )?,
                                expected,
                                meter,
                            )? => {}
                        Some(_) => {
                            return Err(issue(
                                Code::IllTyped,
                                Some(Cause::TypeMismatch),
                                at,
                                source,
                                meter,
                            ));
                        }
                        None => {
                            return Err(issue(
                                Code::MissingDeclaration,
                                Some(Cause::MissingName),
                                at,
                                source,
                                meter,
                            ));
                        }
                    }
                }
                // Parameter metadata contains its name and index literals; they
                // are not terms read by the enclosing hybrid expression.
                continue;
            }
            Some(CheckedNodeKind::Function(_)) => {
                // A called function is checked at its call; its own body does not
                // read the enclosing model's parameters.
                if !numeric_or_boolean_type(&node.semantic_type, nodes, kinds, index, meter, at)? {
                    return Err(issue(
                        Code::IllTyped,
                        Some(Cause::TypeMismatch),
                        at,
                        source,
                        meter,
                    ));
                }
                let parameters = members(decoded.term(position))
                    .first()
                    .filter(|member| binding_role(member) == Some(HybridBindingRole::Parameters))
                    .and_then(term_value)
                    .map(members)
                    .unwrap_or(&[]);
                for parameter in parameters {
                    work(meter, at, 1)?;
                    if let Some((position, parameter_node)) =
                        optional_node(target(parameter), nodes, index, meter)?
                    {
                        if kinds.get(position)
                            == Some(&CheckedNodeKind::Value(ValueForm::Parameter))
                            && !numeric_or_boolean_type(
                                &parameter_node.semantic_type,
                                nodes,
                                kinds,
                                index,
                                meter,
                                at,
                            )?
                        {
                            return Err(issue(
                                Code::IllTyped,
                                Some(Cause::TypeMismatch),
                                at,
                                source,
                                meter,
                            ));
                        }
                    }
                }
                continue;
            }
            Some(CheckedNodeKind::Expression(form)) => {
                let operation = match decoded.term(position) {
                    CheckedTerm::Application { operation, .. } => operation.identity(),
                    _ => None,
                };
                let admitted = matches!(
                    form,
                    super::ExpressionForm::Conditional | super::ExpressionForm::Call
                ) || matches!(
                    operation,
                    Some(
                        "quire.op.rational.negate"
                            | "quire.op.rational.add"
                            | "quire.op.rational.sub"
                            | "quire.op.rational.mul"
                            | "quire.op.rational.div"
                            | "quire.op.rational.eq"
                            | "quire.op.rational.ne"
                            | "quire.op.rational.lt"
                            | "quire.op.rational.le"
                            | "quire.op.rational.ge"
                            | "quire.op.rational.gt"
                            | "quire.op.boolean.not"
                            | "quire.op.boolean.and"
                            | "quire.op.boolean.or"
                            | "quire.op.boolean.implies"
                    )
                );
                invalid_term |= !admitted;
            }
            Some(CheckedNodeKind::Value(ValueForm::Literal)) => {
                invalid_term |= !matches!(
                    index_position(index, &node.semantic_type, meter)?
                        .and_then(|position| kinds.get(position)),
                    Some(CheckedNodeKind::ScalarType(
                        ScalarTypeForm::Rational | ScalarTypeForm::Boolean
                    ))
                );
            }
            Some(CheckedNodeKind::Value(_)) => invalid_term = true,
            _ => {}
        }
        for dependency in &node.dependencies {
            meter.push(&mut pending, dependency)?;
        }
    }
    if invalid_term {
        return Err(issue(
            Code::IllTyped,
            Some(Cause::OperatorIneligible),
            at,
            source,
            meter,
        ));
    }
    Ok(())
}

/// Joins every hybrid model to its variables, modes, transitions and initial entries.
/// The strict wire and key stages have already authenticated each node body.
pub(super) fn validate_models(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut sort = IdSort::new(nodes.len());
    let decoded = DecodedNodes {
        kinds,
        terms,
        index,
    };
    meter.at_node(0, "hybrid");
    let mut reached = meter.vec::<u8>(0)?;
    let mut visits: Option<Visits<'_>> = None;
    for &position in index.values() {
        if kinds.get(position) != Some(&CheckedNodeKind::Hybrid(HybridForm::Model)) {
            continue;
        }
        meter.at_node(position, "hybrid");
        let model = &nodes[position];
        let model_term = decoded.term(position);
        work(meter, node_path(position), 1)?;
        if reached.is_empty() {
            meter.at_node(0, "hybrid");
            reached = meter.vec(nodes.len())?;
            reached.resize(nodes.len(), 0_u8);
            meter.at_node(position, "hybrid");
        }
        let variables_body = bound(model_term, 0);
        let modes_body = bound(model_term, 1);
        let mode_refs = members(modes_body);
        let mut mode_first = meter.vec(mode_refs.len())?;
        mode_first.extend(
            mode_refs
                .iter()
                .enumerate()
                .filter_map(|(slot, term)| target(term).map(|key| (key, slot))),
        );
        sort_id_rows(&mut mode_first, meter, &mut sort)?;
        for (slot, mode) in mode_refs.iter().enumerate() {
            let at = model_pointer(position, 1).key("members").index(slot);
            let Some(mode_id) = target(mode) else {
                return Err(malformed(at, meter));
            };
            let first = lower_id(&mode_first, mode_id, meter, |row| row.0)?;
            if mode_first[first].1 != slot {
                return Err(issue(Code::InvalidSemanticGraph, None, at, model, meter));
            }
        }
        if members(model_term).len() == 3 {
            let initial = bound(model_term, 2);
            if members(initial).len() == 1 {
                let item = &members(initial)[0];
                if let Some(condition) = target(bound(item, 1)) {
                    if number(bound(item, 0)) == Some(0)
                        && node_at(condition, nodes, index, meter)?.is_some_and(|(position, _)| {
                            matches!(
                                terms.get(position).and_then(Option::as_ref),
                                Some(CheckedTerm::Literal {
                                    value_kind: LiteralKind::Boolean,
                                    value,
                                    ..
                                }) if value.as_bool() == Some(true)
                            )
                        })
                    {
                        return Err(issue(
                            Code::InvalidSemanticGraph,
                            None,
                            model_pointer(position, 2),
                            model,
                            meter,
                        ));
                    }
                }
            }
        }
        let variable_entries = members(variables_body);
        let mut variable_first = meter.vec(variable_entries.len())?;
        let mut name_first = meter.vec(variable_entries.len())?;
        for (slot, variable) in variable_entries.iter().enumerate() {
            if let Some(key) = target(bound(variable, 0)) {
                variable_first.push((key, slot));
                if let Some((target_position, _)) = node_at(key, nodes, index, meter)? {
                    if let Some((name, _)) = parameter_name_level(decoded.term(target_position)) {
                        name_first.push((name, slot));
                    }
                }
            }
        }
        sort_id_rows(&mut variable_first, meter, &mut sort)?;
        name_first.sort_unstable();
        let mut variables = meter.vec(variable_entries.len())?;
        let mut classes = meter.vec(variable_entries.len())?;
        for (slot, variable) in members(variables_body).iter().enumerate() {
            let at = model_pointer(position, 0).key("members").index(slot);
            let Some(key) = target(bound(variable, 0)) else {
                return Err(malformed(at, meter));
            };
            let class = terms
                .get(position)
                .and_then(Option::as_ref)
                .and_then(|term| model_variable_class(term, slot));
            let valid = if let Some((target_position, node)) = node_at(key, nodes, index, meter)? {
                kinds.get(target_position) == Some(&CheckedNodeKind::Value(ValueForm::Parameter))
                    && index_position(index, &node.semantic_type, meter)?
                        .and_then(|position| kinds.get(position))
                        == Some(&CheckedNodeKind::ScalarType(ScalarTypeForm::Rational))
                    && parameter_name_level(decoded.term(target_position)).is_some_and(
                        |(name, level)| {
                            let first =
                                name_first.partition_point(|(candidate, _)| *candidate < name);
                            level == slot && name_first[first].1 == slot
                        },
                    )
            } else {
                false
            };
            let first = lower_id(&variable_first, key, meter, |row| row.0)?;
            if !valid || variable_first[first].1 != slot {
                return Err(issue(
                    Code::InvalidModelBinding,
                    Some(Cause::MalformedDeclaration),
                    at.key("members").index(0).key("value").key("target"),
                    model,
                    meter,
                ));
            }
            variables.push(key);
            classes.push((
                key,
                class == Some(HybridVariableClass::Clock),
                at.key("members").index(1).key("value").key("value"),
            ));
        }
        sort_ids(&mut variables, meter, &mut sort)?;
        let mut rates_by_mode = meter.vec(mode_refs.len())?;
        for (slot, mode_ref) in mode_refs.iter().enumerate() {
            let at = model_pointer(position, 1).key("members").index(slot);
            let Some(key) = target(mode_ref) else {
                return Err(malformed(at, meter));
            };
            let Some((mode_position, mode)) = node_at(key, nodes, index, meter)? else {
                return Err(issue(
                    Code::InvalidModelBinding,
                    Some(Cause::MalformedDeclaration),
                    at.key("target"),
                    model,
                    meter,
                ));
            };
            let declared = model
                .declaration
                .as_ref()
                .map(|declaration| declaration.qualified_name.as_slice())
                .unwrap_or(&[]);
            let mode_name = mode
                .declaration
                .as_ref()
                .map(|declaration| declaration.qualified_name.as_slice())
                .unwrap_or(&[]);
            if kinds.get(mode_position) != Some(&CheckedNodeKind::Hybrid(HybridForm::Mode))
                || mode_name.len() != declared.len() + 1
                || !mode_name.starts_with(declared)
                || mode.owner != model.owner
            {
                return Err(issue(
                    Code::InvalidModelBinding,
                    Some(Cause::MalformedDeclaration),
                    at.key("target"),
                    model,
                    meter,
                ));
            }
            reached[mode_position] |= 1;
            meter.at_node(mode_position, "hybrid");
            let mode_term = decoded.term(mode_position);
            let rates = members(bound(mode_term, 1));
            if rates.len() != variables.len() {
                return Err(issue(
                    Code::InvalidModelBinding,
                    Some(Cause::MalformedDeclaration),
                    model_pointer(mode_position, 1),
                    mode,
                    meter,
                ));
            }
            let invariant = target(bound(mode_term, 0)).expect("strict hybrid reference");
            if !term_type(
                invariant,
                ScalarTypeForm::Boolean,
                nodes,
                kinds,
                index,
                meter,
            )? {
                return Err(issue(
                    Code::IllTyped,
                    Some(Cause::NonBooleanRoot),
                    model_pointer(mode_position, 0).key("target"),
                    mode,
                    meter,
                ));
            }
            let mut rate_keys = meter.vec(rates.len())?;
            for (rate_slot, rate) in rates.iter().enumerate() {
                let key = target(rate).expect("strict hybrid reference");
                if !rational_or_variable(key, &variables, nodes, kinds, index, meter)? {
                    return Err(issue(
                        Code::IllTyped,
                        Some(Cause::TypeMismatch),
                        model_pointer(mode_position, 1)
                            .key("members")
                            .index(rate_slot)
                            .key("target"),
                        mode,
                        meter,
                    ));
                }
                rate_keys.push(key);
            }
            rates_by_mode.push(rate_keys);
            for (transition_slot, transition_ref) in members(bound(mode_term, 2)).iter().enumerate()
            {
                let at = model_pointer(mode_position, 2)
                    .key("members")
                    .index(transition_slot);
                let key = target(transition_ref).expect("strict hybrid reference");
                let Some((transition_position, transition)) = node_at(key, nodes, index, meter)?
                else {
                    return Err(issue(
                        Code::InvalidModelBinding,
                        Some(Cause::MalformedDeclaration),
                        at.key("target"),
                        mode,
                        meter,
                    ));
                };
                if kinds.get(transition_position)
                    != Some(&CheckedNodeKind::Hybrid(HybridForm::Transition))
                {
                    return Err(issue(
                        Code::InvalidModelBinding,
                        Some(Cause::MalformedDeclaration),
                        at.key("target"),
                        mode,
                        meter,
                    ));
                }
                reached[transition_position] |= 2;
                meter.at_node(transition_position, "hybrid");
                let transition_term = decoded.term(transition_position);
                let target_mode = number(bound(transition_term, 0)).unwrap_or(usize::MAX);
                if target_mode >= mode_refs.len() {
                    return Err(issue(
                        Code::MissingDeclaration,
                        Some(Cause::MissingName),
                        model_pointer(transition_position, 0),
                        transition,
                        meter,
                    ));
                }
                let guard = target(bound(transition_term, 1)).expect("strict hybrid reference");
                if !term_type(guard, ScalarTypeForm::Boolean, nodes, kinds, index, meter)? {
                    return Err(issue(
                        Code::IllTyped,
                        Some(Cause::NonBooleanRoot),
                        model_pointer(transition_position, 1).key("target"),
                        transition,
                        meter,
                    ));
                }
                let mut reset_flags = meter.vec(variables.len())?;
                reset_flags.resize(variables.len(), 0_u8);
                for (reset_slot, reset) in members(bound(transition_term, 2)).iter().enumerate() {
                    let at = model_pointer(transition_position, 2)
                        .key("members")
                        .index(reset_slot);
                    let variable = number(bound(reset, 0)).unwrap_or(usize::MAX);
                    if variable >= variables.len() {
                        return Err(issue(
                            Code::MissingDeclaration,
                            Some(Cause::MissingName),
                            at.key("members").index(0).key("value"),
                            transition,
                            meter,
                        ));
                    }
                    let assignment = members(reset).len() == 2;
                    if (assignment && reset_flags[variable] != 0)
                        || (!assignment && reset_flags[variable] & 1 != 0)
                    {
                        return Err(issue(
                            Code::InvalidPackage,
                            Some(Cause::ConflictingDefinition),
                            at,
                            transition,
                            meter,
                        ));
                    }
                    reset_flags[variable] |= if assignment { 1 } else { 2 };
                    let value = target(bound(reset, 1)).expect("strict hybrid reference");
                    if !rational_or_variable(value, &variables, nodes, kinds, index, meter)? {
                        return Err(issue(
                            Code::IllTyped,
                            Some(Cause::TypeMismatch),
                            model_pointer(transition_position, 2)
                                .key("members")
                                .index(reset_slot)
                                .key("members")
                                .index(1)
                                .key("value")
                                .key("target"),
                            transition,
                            meter,
                        ));
                    }
                }
            }
        }
        if let Some(initial) = members(model_term).get(2).map(|_| bound(model_term, 2)) {
            for (slot, item) in members(initial).iter().enumerate() {
                let at = model_pointer(position, 2).key("members").index(slot);
                let mode = number(bound(item, 0)).unwrap_or(usize::MAX);
                if mode >= mode_refs.len() {
                    return Err(issue(
                        Code::MissingDeclaration,
                        Some(Cause::MissingName),
                        at.key("members").index(0).key("value"),
                        model,
                        meter,
                    ));
                }
                let condition = target(bound(item, 1)).expect("strict hybrid reference");
                if !term_type(
                    condition,
                    ScalarTypeForm::Boolean,
                    nodes,
                    kinds,
                    index,
                    meter,
                )? {
                    return Err(issue(
                        Code::IllTyped,
                        Some(Cause::NonBooleanRoot),
                        at.key("members").index(1).key("value").key("target"),
                        model,
                        meter,
                    ));
                }
            }
        }
        // QSpec FR-481 reader check 5: the referenced term and its dependency
        // closure may read only this model's continuous variables.
        let model_scope = ReadScope {
            variables: &variables,
            named_reads: meter.vec(0)?,
        };
        let visits = match &mut visits {
            Some(visits) => visits,
            slot @ None => slot.insert(Visits::new(nodes, meter)?),
        };
        for mode_ref in mode_refs {
            let Some(mode_key) = target(mode_ref) else {
                continue;
            };
            let Some(mode_position) = index_position(index, mode_key, meter)? else {
                continue;
            };
            let mode = &nodes[mode_position];
            let mode_term = decoded.term(mode_position);
            for binding_index in [0, 1] {
                let term_refs = if binding_index == 0 {
                    std::slice::from_ref(bound(mode_term, 0))
                } else {
                    members(bound(mode_term, 1))
                };
                for (slot, term) in term_refs.iter().enumerate() {
                    if let Some(key) = target(term) {
                        reads_allowed(
                            key,
                            &model_scope,
                            nodes,
                            decoded,
                            if binding_index == 0 {
                                model_pointer(mode_position, 0)
                            } else {
                                model_pointer(mode_position, 1).key("members").index(slot)
                            },
                            mode,
                            meter,
                            visits,
                        )?;
                    }
                }
            }
            for (transition_slot, transition_ref) in members(bound(mode_term, 2)).iter().enumerate()
            {
                let Some(key) = target(transition_ref) else {
                    continue;
                };
                let Some((transition_position, transition)) = node_at(key, nodes, index, meter)?
                else {
                    continue;
                };
                let transition_term = decoded.term(transition_position);
                if let Some(guard) = target(bound(transition_term, 1)) {
                    reads_allowed(
                        guard,
                        &model_scope,
                        nodes,
                        decoded,
                        model_pointer(mode_position, 2)
                            .key("members")
                            .index(transition_slot),
                        mode,
                        meter,
                        visits,
                    )?;
                }
                for (reset_slot, reset) in members(bound(transition_term, 2)).iter().enumerate() {
                    if let Some(value) = target(bound(reset, 1)) {
                        reads_allowed(
                            value,
                            &model_scope,
                            nodes,
                            decoded,
                            model_pointer(transition_position, 2)
                                .key("members")
                                .index(reset_slot),
                            transition,
                            meter,
                            visits,
                        )?;
                    }
                }
            }
        }
        if members(model_term).len() == 3 {
            for (slot, initial) in members(bound(model_term, 2)).iter().enumerate() {
                if let Some(key) = target(bound(initial, 1)) {
                    reads_allowed(
                        key,
                        &model_scope,
                        nodes,
                        decoded,
                        model_pointer(position, 2).key("members").index(slot),
                        model,
                        meter,
                        visits,
                    )?;
                }
            }
        }
        for (slot, (_, clock, at)) in classes.iter().enumerate() {
            if !*clock {
                continue;
            }
            let mut invalid = false;
            for rates in &rates_by_mode {
                invalid = node_at(rates[slot], nodes, index, meter)?.is_none_or(|(position, _)| {
                    kinds.get(position) != Some(&CheckedNodeKind::Value(ValueForm::Literal))
                        || !terms
                            .get(position)
                            .and_then(Option::as_ref)
                            .is_some_and(is_rational_one)
                });
                if invalid {
                    break;
                }
            }
            if invalid {
                return Err(issue(
                    Code::InvalidPackage,
                    Some(Cause::InvalidValue),
                    *at,
                    model,
                    meter,
                ));
            }
        }
        if classes.iter().all(|(_, clock, _)| *clock) {
            return Err(issue(
                Code::InvalidPackage,
                Some(Cause::InvalidValue),
                model_pointer(position, 0),
                model,
                meter,
            ));
        }
    }
    for (_, &position) in index {
        match kinds.get(position) {
            Some(CheckedNodeKind::Hybrid(HybridForm::Mode))
                if reached.get(position).copied().unwrap_or(0) & 1 == 0 =>
            {
                return Err(issue(
                    Code::InvalidSemanticGraph,
                    None,
                    node_path(position),
                    &nodes[position],
                    meter,
                ));
            }
            Some(CheckedNodeKind::Hybrid(HybridForm::Transition))
                if reached.get(position).copied().unwrap_or(0) & 2 == 0 =>
            {
                return Err(issue(
                    Code::InvalidSemanticGraph,
                    None,
                    node_path(position),
                    &nodes[position],
                    meter,
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

/// Checks temporal clauses whose over parameter is typed by a hybrid model.
/// The ordinary temporal step has already admitted placement and profile fit.
#[ix_trace_rs::string_edge]
pub(super) fn validate_claims(
    nodes: &[CheckedSemanticNodeV2],
    kinds: &[CheckedNodeKind],
    terms: &[Option<CheckedTerm>],
    index: &GraphIndex<'_>,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let mut sort = IdSort::new(nodes.len());
    let decoded = DecodedNodes {
        kinds,
        terms,
        index,
    };
    let mut visits: Option<Visits<'_>> = None;
    let mut formula_visits: Option<Visits<'_>> = None;
    for &position in index.values() {
        if kinds.get(position) != Some(&CheckedNodeKind::Temporal(TemporalForm::TemporalClause)) {
            continue;
        }
        meter.at_node(position, "hybrid");
        let claim = &nodes[position];
        let Some(arguments) = terms
            .get(position)
            .and_then(Option::as_ref)
            .and_then(application_arguments)
        else {
            continue;
        };
        let Some(over) = arguments.first().and_then(reference_target) else {
            continue;
        };
        let Some((_, over_node)) = node_at(over, nodes, index, meter)? else {
            continue;
        };
        let Some((model_position, model)) = node_at(&over_node.semantic_type, nodes, index, meter)?
        else {
            continue;
        };
        if kinds.get(model_position) != Some(&CheckedNodeKind::Hybrid(HybridForm::Model)) {
            continue;
        }
        meter.at_node(position, "hybrid");
        work(meter, node_path(position), 1)?;
        if arguments.len() != 6 {
            continue;
        }
        let argument_at = |slot| node_path(position).key("body").key("arguments").index(slot);
        if arguments
            .get(1)
            .and_then(literal_word_as::<HybridTimeKind>)
            .is_none()
        {
            return Err(issue(
                Code::InvalidPackage,
                Some(Cause::InvalidValue),
                argument_at(1),
                claim,
                meter,
            ));
        }
        let activation = terms
            .get(position)
            .and_then(Option::as_ref)
            .and_then(application_arguments)
            .and_then(|arguments| arguments.get(2))
            .map(term_members)
            .unwrap_or(&[]);
        let each = activation.first().is_some_and(|kind| {
            binding_role(kind) == Some(HybridBindingRole::Kind)
                && term_value(kind).and_then(literal_word_as::<HybridActivationKind>)
                    == Some(HybridActivationKind::Each)
        });
        if each {
            let trigger = activation
                .iter()
                .find(|member| binding_role(member) == Some(HybridBindingRole::Trigger))
                .and_then(term_value)
                .and_then(reference_target);
            let valid = if let Some((target_position, node)) =
                optional_node(trigger, nodes, index, meter)?
            {
                kinds.get(target_position) == Some(&CheckedNodeKind::Value(ValueForm::Parameter))
                    && meter
                        .compare_ids(&node.semantic_type, &model.node_id)?
                        .is_eq()
                    && parameter_name_level(decoded.term(target_position))
                        .is_some_and(|(_, level)| level == 1)
            } else {
                false
            };
            if !valid {
                return Err(issue(
                    Code::IllTyped,
                    Some(Cause::TypeMismatch),
                    argument_at(2).key("members"),
                    claim,
                    meter,
                ));
            }
        }
        let model_term = decoded.term(model_position);
        let variable_entries = members(bound(model_term, 0));
        let captures = term_members(&arguments[3]);
        let mut variables = meter.vec(variable_entries.len())?;
        variables.extend(
            variable_entries
                .iter()
                .filter_map(|entry| target(bound(entry, 0))),
        );
        sort_ids(&mut variables, meter, &mut sort)?;
        dedup_ids(&mut variables, |key| *key, meter)?;
        let mut named_reads =
            meter.vec(if each { variable_entries.len() } else { 0 } + captures.len())?;
        if each {
            for (slot, entry) in variable_entries.iter().enumerate() {
                let Some(variable) = target(bound(entry, 0)) else {
                    continue;
                };
                let Some((variable_position, variable_node)) =
                    node_at(variable, nodes, index, meter)?
                else {
                    continue;
                };
                let Some((name, _)) = parameter_name_level(decoded.term(variable_position)) else {
                    continue;
                };
                named_reads.push(NamedRead {
                    key: (name, variable_entries.len() + slot),
                    ty: Some(&variable_node.semantic_type),
                });
            }
        }
        for (slot, capture) in captures.iter().enumerate() {
            let name = match capture {
                CheckedTerm::Binding { name, .. } => name.as_ref(),
                _ => "",
            };
            named_reads.push(NamedRead {
                key: (name, 2 * variable_entries.len() + slot),
                ty: None,
            });
        }
        named_reads.sort_unstable_by_key(|row| row.key);
        let mut allowed = ReadScope {
            variables: &variables,
            named_reads,
        };
        let visits = match &mut visits {
            Some(visits) => visits,
            slot @ None => slot.insert(Visits::new(nodes, meter)?),
        };
        if let Some((slot, guard)) = activation
            .iter()
            .enumerate()
            .find(|(_, member)| binding_role(member) == Some(HybridBindingRole::Guard))
        {
            if let Some(key) = term_value(guard).and_then(reference_target) {
                let at = argument_at(2).key("members").index(slot);
                if !term_type(key, ScalarTypeForm::Boolean, nodes, kinds, index, meter)? {
                    return Err(issue(
                        Code::IllTyped,
                        Some(Cause::TypeMismatch),
                        at,
                        claim,
                        meter,
                    ));
                }
                reads_allowed(key, &allowed, nodes, decoded, at, claim, meter, visits)?;
            }
        }
        let formula_root = arguments.get(5).and_then(reference_target);
        let mut claim_roots = meter.vec(captures.len() + usize::from(formula_root.is_some()))?;
        claim_roots.extend(formula_root);
        claim_roots.extend(
            captures
                .iter()
                .filter_map(|capture| term_value(capture).and_then(reference_target)),
        );
        let parameter_reads = claim_parameter_reads(
            claim_roots,
            nodes,
            kinds,
            decoded,
            meter,
            argument_at(3),
            visits,
        )?;
        for (slot, capture) in captures.iter().enumerate() {
            let Some(value) = term_value(capture).and_then(reference_target) else {
                continue;
            };
            let at = argument_at(3).key("members").index(slot);
            let initializer = node_at(value, nodes, index, meter)?;
            if !initializer.is_some_and(|(position, _)| {
                matches!(
                    kinds.get(position),
                    Some(
                        CheckedNodeKind::Expression(_)
                            | CheckedNodeKind::Value(ValueForm::Literal | ValueForm::Parameter)
                    )
                )
            }) {
                return Err(issue(
                    Code::IllTyped,
                    Some(Cause::TypeMismatch),
                    at,
                    claim,
                    meter,
                ));
            }
            let Some(initializer_type) = initializer
                .map(|(_, node)| {
                    scalar_base_type(&node.semantic_type, nodes, kinds, index, meter, at)
                })
                .transpose()?
                .flatten()
            else {
                return Err(issue(
                    Code::IllTyped,
                    Some(Cause::TypeMismatch),
                    at,
                    claim,
                    meter,
                ));
            };
            let capture_name = match capture {
                CheckedTerm::Binding { name, .. } => name.as_ref(),
                _ => "",
            };
            let capture_level = 2 * variable_entries.len() + slot;
            {
                for read in parameter_reads_for(&parameter_reads, (capture_name, capture_level)) {
                    let Some((_, read_node)) = node_at(read.id, nodes, index, meter)? else {
                        continue;
                    };
                    if !same_id(
                        scalar_base_type(&read_node.semantic_type, nodes, kinds, index, meter, at)?,
                        initializer_type,
                        meter,
                    )? {
                        return Err(issue(
                            Code::IllTyped,
                            Some(Cause::TypeMismatch),
                            at,
                            claim,
                            meter,
                        ));
                    }
                }
            }
            reads_allowed(value, &allowed, nodes, decoded, at, claim, meter, visits)?;
            let slot = allowed
                .named_reads
                .binary_search_by_key(&(capture_name, capture_level), |row| row.key)
                .expect("the capture name was indexed before traversal");
            allowed.named_reads[slot].ty = Some(initializer_type);
        }
        let mode_refs = members(bound(model_term, 1));
        let mut mode_keys = meter.vec(mode_refs.len())?;
        mode_keys.extend(mode_refs.iter().filter_map(target));
        for fair_ref in term_members(&arguments[4]) {
            let Some(fair_key) = reference_target(fair_ref) else {
                continue;
            };
            let Some((fair_position, fair)) = node_at(fair_key, nodes, index, meter)? else {
                continue;
            };
            let CheckedTerm::Application { operation, .. } = decoded.term(fair_position) else {
                continue;
            };
            let Some(super::CheckedFairnessTarget::HybridTransition { declaration, name }) =
                operation.hybrid_target_for_join(meter)?
            else {
                return Err(issue(
                    Code::MissingDeclaration,
                    Some(Cause::MissingName),
                    node_path(fair_position)
                        .key("body")
                        .key("operation")
                        .key("member")
                        .key("target")
                        .key("name"),
                    fair,
                    meter,
                ));
            };
            let mut resolved = false;
            for mode_key in &mode_keys {
                let Some((mode_position, mode)) = node_at(mode_key, nodes, index, meter)? else {
                    continue;
                };
                let mode_name = mode
                    .declaration
                    .as_ref()
                    .and_then(|decl| decl.qualified_name.last())
                    .map(AsRef::as_ref);
                if meter.compare_ids(&declaration, &model.node_id)?.is_eq()
                    && mode_name == Some(name.as_ref())
                    && !members(bound(decoded.term(mode_position), 2)).is_empty()
                {
                    resolved = true;
                }
                if meter.compare_ids(&declaration, mode_key)?.is_eq() {
                    for transition_ref in members(bound(decoded.term(mode_position), 2)) {
                        let Some(transition_key) = target(transition_ref) else {
                            continue;
                        };
                        let Some((transition_position, _)) =
                            node_at(transition_key, nodes, index, meter)?
                        else {
                            continue;
                        };
                        let Some(target_slot) = number(bound(decoded.term(transition_position), 0))
                        else {
                            continue;
                        };
                        if optional_node(mode_keys.get(target_slot).copied(), nodes, index, meter)?
                            .and_then(|(_, target_mode)| target_mode.declaration.as_ref())
                            .and_then(|decl| decl.qualified_name.last())
                            .is_some_and(|part| part.as_ref() == name.as_ref())
                        {
                            resolved = true;
                        }
                    }
                }
            }
            if !resolved {
                return Err(issue(
                    Code::MissingDeclaration,
                    Some(Cause::MissingName),
                    node_path(fair_position)
                        .key("body")
                        .key("operation")
                        .key("member")
                        .key("target")
                        .key("name"),
                    fair,
                    meter,
                ));
            }
        }
        let mut pending = meter.vec(usize::from(formula_root.is_some()))?;
        pending.extend(formula_root);
        let formula_visits = match &mut formula_visits {
            Some(visits) => visits,
            slot @ None => slot.insert(visits.fork(meter)?),
        };
        let mut seen = formula_visits.begin(&pending, meter)?;
        while let Some(formula_key) = pending.pop() {
            if !seen.insert(formula_key, meter)? {
                continue;
            }
            let Some((formula_position, formula)) = node_at(formula_key, nodes, index, meter)?
            else {
                continue;
            };
            if kinds.get(formula_position)
                != Some(&CheckedNodeKind::Temporal(TemporalForm::Formula))
            {
                continue;
            }
            let operation = terms
                .get(formula_position)
                .and_then(Option::as_ref)
                .and_then(|term| match term {
                    CheckedTerm::Application { operation, .. } => {
                        operation.identity().and_then(decode_word)
                    }
                    _ => None,
                });
            if operation == Some(HybridTemporalOperation::Holds) {
                if let Some(operand) = application_arguments(decoded.term(formula_position))
                    .and_then(|arguments| arguments.first())
                    .and_then(reference_target)
                {
                    reads_allowed(
                        operand,
                        &allowed,
                        nodes,
                        decoded,
                        node_path(formula_position),
                        formula,
                        meter,
                        visits,
                    )?;
                }
            }
            for dependency in &formula.dependencies {
                meter.push(&mut pending, dependency)?;
            }
        }
    }
    Ok(())
}

fn aggregate<'d>(
    value: TermNode<'d>,
    at: Path,
    meter: &mut WorkMeter,
) -> Result<super::TermItems<'d>, ValidationFailure> {
    if value.kind() != Some(super::BodyTerm::Aggregate)
        || !value.exact_members(&["term", "members"])
    {
        return Err(malformed(at, meter));
    }
    value
        .get("members")
        .and_then(TermNode::items)
        .ok_or_else(|| malformed(at.key("members"), meter))
}

fn binding<'d>(
    value: TermNode<'d>,
    name: &str,
    at: Path,
    meter: &mut WorkMeter,
) -> Result<TermNode<'d>, ValidationFailure> {
    if value.kind() != Some(super::BodyTerm::Binding)
        || !value.exact_members(&["term", "name", "value"])
    {
        return Err(malformed(at, meter));
    }
    match value.view() {
        Some(super::TermView::Binding {
            name: actual,
            value,
        }) if actual == name => Ok(value),
        _ => Err(malformed(at, meter)),
    }
}

fn entry<'d>(
    members: super::TermItems<'d>,
    index: usize,
    name: &str,
    at: Path,
    meter: &mut WorkMeter,
) -> Result<(TermNode<'d>, Path), ValidationFailure> {
    let member_at = at.key("members").index(index);
    let member = members
        .clone()
        .nth(index)
        .ok_or_else(|| malformed(member_at, meter))?;
    Ok((
        binding(member, name, member_at, meter)?,
        member_at.key("value"),
    ))
}

fn reference(
    value: TermNode<'_>,
    at: Path,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    match value.view() {
        Some(super::TermView::Reference { .. }) => Ok(()),
        _ => Err(malformed(at, meter)),
    }
}

fn literal<'d>(
    value: TermNode<'d>,
    kind: &str,
    at: Path,
    meter: &mut WorkMeter,
) -> Result<&'d str, ValidationFailure> {
    match value.view() {
        Some(super::TermView::Literal {
            value_kind, value, ..
        }) if Some(value_kind) == LiteralKind::from_wire(kind) => value
            .as_str()
            .ok_or_else(|| malformed(at.key("value"), meter)),
        _ => Err(malformed(at, meter)),
    }
}

#[string_edge]
fn integer(value: TermNode<'_>, at: Path, meter: &mut WorkMeter) -> Result<(), ValidationFailure> {
    let text = literal(value, "integer", at, meter)?;
    let digits = text.strip_prefix('-').unwrap_or(text);
    if digits.is_empty()
        || (digits.starts_with('0') && digits.len() > 1)
        || text == "-0"
        || !digits.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(malformed(at.key("value"), meter));
    }
    Ok(())
}

fn list_of_references(
    value: TermNode<'_>,
    at: Path,
    minimum: usize,
    meter: &mut WorkMeter,
) -> Result<(), ValidationFailure> {
    let members = aggregate(value, at, meter)?;
    if members.len() < minimum {
        return Err(malformed(at.key("members"), meter));
    }
    for (index, member) in members.enumerate() {
        reference(member, at.key("members").index(index), meter)?;
    }
    Ok(())
}

/// The published FR-481 body and occurrence schema, before package identity.
pub(super) fn check_wire(
    node: &CheckedSemanticNodeV2,
    form: HybridForm,
    position: usize,
    budget: &mut super::NativeJoinBudget,
) -> Result<(), ValidationFailure> {
    let mut meter = WorkMeter::over(budget);
    let meter = &mut meter;
    let mut at = node_path(position);
    at.stage = PackageReadStage::WireChecks;
    if node.recursion_group.is_some() {
        return Err(malformed(at.key("recursion_group"), meter));
    }
    let expected_role = match form {
        HybridForm::Model | HybridForm::Mode => {
            if node.declaration.is_none() {
                return Err(malformed(at.key("declaration"), meter));
            }
            if node.owner.is_none() {
                return Err(malformed(at.key("owner"), meter));
            }
            if node.occurrences.len() != 1 {
                return Err(malformed(at.key("occurrences"), meter));
            }
            CheckedOccurrenceRole::Declaration
        }
        HybridForm::Transition => {
            if node.declaration.is_some() {
                return Err(malformed(at.key("declaration"), meter));
            }
            if node.owner.is_some() {
                return Err(malformed(at.key("owner"), meter));
            }
            if node.occurrences.is_empty() {
                return Err(malformed(at.key("occurrences"), meter));
            }
            CheckedOccurrenceRole::Expression
        }
    };
    for (index, occurrence) in node.occurrences.iter().enumerate() {
        if occurrence.role != expected_role {
            return Err(malformed(
                at.key("occurrences").index(index).key("role"),
                meter,
            ));
        }
    }
    let body_at = at.key("body");
    let members = aggregate(TermNode::of(&node.body), body_at, meter)?;
    match form {
        HybridForm::Model => {
            if !matches!(members.len(), 2 | 3) {
                return Err(malformed(body_at.key("members"), meter));
            }
            let (variables, variables_at) = entry(members.clone(), 0, "variables", body_at, meter)?;
            let variables = aggregate(variables, variables_at, meter)?;
            if variables.len() == 0 {
                return Err(malformed(variables_at.key("members"), meter));
            }
            for (index, variable) in variables.enumerate() {
                let at = variables_at.key("members").index(index);
                let members = aggregate(variable, at, meter)?;
                if members.len() != 2 {
                    return Err(malformed(at.key("members"), meter));
                }
                let (parameter, parameter_at) = entry(members.clone(), 0, "parameter", at, meter)?;
                reference(parameter, parameter_at, meter)?;
                let (class, class_at) = entry(members.clone(), 1, "class", at, meter)?;
                if !matches!(literal(class, "text", class_at, meter)?, "clock" | "hybrid") {
                    return Err(malformed(class_at.key("value"), meter));
                }
            }
            let (modes, modes_at) = entry(members.clone(), 1, "modes", body_at, meter)?;
            list_of_references(modes, modes_at, 1, meter)?;
            if members.len() == 3 {
                let (initial, initial_at) = entry(members.clone(), 2, "initial", body_at, meter)?;
                let initial = aggregate(initial, initial_at, meter)?;
                if initial.len() == 0 {
                    return Err(malformed(initial_at.key("members"), meter));
                }
                for (index, item) in initial.enumerate() {
                    let at = initial_at.key("members").index(index);
                    let item = aggregate(item, at, meter)?;
                    if item.len() != 2 {
                        return Err(malformed(at.key("members"), meter));
                    }
                    let (mode, mode_at) = entry(item.clone(), 0, "mode", at, meter)?;
                    integer(mode, mode_at, meter)?;
                    let (condition, condition_at) = entry(item.clone(), 1, "condition", at, meter)?;
                    reference(condition, condition_at, meter)?;
                }
            }
        }
        HybridForm::Mode => {
            if members.len() != 3 {
                return Err(malformed(body_at.key("members"), meter));
            }
            let (invariant, invariant_at) = entry(members.clone(), 0, "invariant", body_at, meter)?;
            reference(invariant, invariant_at, meter)?;
            let (rates, rates_at) = entry(members.clone(), 1, "rates", body_at, meter)?;
            list_of_references(rates, rates_at, 1, meter)?;
            let (transitions, transitions_at) =
                entry(members.clone(), 2, "transitions", body_at, meter)?;
            list_of_references(transitions, transitions_at, 0, meter)?;
        }
        HybridForm::Transition => {
            if members.len() != 3 {
                return Err(malformed(body_at.key("members"), meter));
            }
            let (target, target_at) = entry(members.clone(), 0, "target", body_at, meter)?;
            integer(target, target_at, meter)?;
            let (guard, guard_at) = entry(members.clone(), 1, "guard", body_at, meter)?;
            reference(guard, guard_at, meter)?;
            let (resets, resets_at) = entry(members.clone(), 2, "resets", body_at, meter)?;
            for (index, reset) in aggregate(resets, resets_at, meter)?.enumerate() {
                let at = resets_at.key("members").index(index);
                let reset = aggregate(reset, at, meter)?;
                if !matches!(reset.len(), 2 | 3) {
                    return Err(malformed(at.key("members"), meter));
                }
                let (variable, variable_at) = entry(reset.clone(), 0, "variable", at, meter)?;
                integer(variable, variable_at, meter)?;
                let (value, value_at) = entry(reset.clone(), 1, "value", at, meter)?;
                reference(value, value_at, meter)?;
                if reset.len() == 3 {
                    let (relation, relation_at) = entry(reset.clone(), 2, "relation", at, meter)?;
                    if !matches!(
                        literal(relation, "text", relation_at, meter)?,
                        "ne" | "lt" | "le" | "ge" | "gt"
                    ) {
                        return Err(malformed(relation_at.key("value"), meter));
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod budget_tests {
    use super::super::{
        ApplicationOperator, CheckedApplicationOperation, CheckedDeclaration, CheckedJson,
        CheckedNodeOwner, NativeJoinBudget,
    };
    use super::*;
    use crate::checked_package::shared::{CheckedNodePathSegment as Segment, CheckedOccurrence};
    use ix_trace_rs::trace;
    use std::sync::Arc;

    struct Fixture {
        nodes: Vec<CheckedSemanticNodeV2>,
        kinds: Vec<CheckedNodeKind>,
        terms: Vec<Option<CheckedTerm>>,
    }
    fn id(index: usize) -> CheckedNodeId {
        CheckedNodeId {
            package: "pkg".into(),
            path: vec![Segment::Name(format!("n{index:04}").into())],
            ordinal: 0,
        }
    }
    fn reference(index: usize) -> CheckedTerm {
        CheckedTerm::reference(id(index))
    }
    fn text(value: &str) -> CheckedTerm {
        CheckedTerm::literal_text(id(2), value)
    }
    fn integer(value: usize) -> CheckedTerm {
        CheckedTerm::Literal {
            type_id: id(3),
            value_kind: LiteralKind::Integer,
            value: CheckedJson::scalar(&value.to_string()),
        }
    }
    fn binding(name: &str, value: CheckedTerm) -> CheckedTerm {
        CheckedTerm::binding(name, value)
    }
    fn aggregate(values: Vec<CheckedTerm>) -> CheckedTerm {
        CheckedTerm::aggregate(values)
    }
    fn parameter(name: &str, level: usize) -> CheckedTerm {
        aggregate(vec![
            binding("name", text(name)),
            binding("level", integer(level)),
        ])
    }
    fn application(
        identity: &str,
        operator: ApplicationOperator,
        arguments: Vec<CheckedTerm>,
    ) -> CheckedTerm {
        let operation: CheckedApplicationOperation = serde_json::from_value(serde_json::json!({
            "identity":identity,"laws":[],"mode":null,"member":null,"leaves":[]
        }))
        .unwrap();
        CheckedTerm::Application {
            operator,
            operation,
            result_type: id(1),
            arguments,
        }
    }
    fn node(
        index: usize,
        kind: CheckedNodeKind,
        type_index: usize,
        term: &CheckedTerm,
        dependencies: &[usize],
        declaration: Option<&[&str]>,
    ) -> CheckedSemanticNodeV2 {
        let hybrid_decl = matches!(
            kind,
            CheckedNodeKind::Hybrid(HybridForm::Model | HybridForm::Mode)
        );
        CheckedSemanticNodeV2 {
            node_id: id(index),
            schema_version: "quire.checked-semantic-graph/v2".into(),
            node_tag: kind.tag().as_wire().into(),
            semantic_form: kind.form_wire().into(),
            semantic_type: id(type_index),
            dependencies: dependencies.iter().map(|i| id(*i)).collect(),
            occurrences: vec![CheckedOccurrence {
                role: if hybrid_decl {
                    CheckedOccurrenceRole::Declaration
                } else {
                    CheckedOccurrenceRole::Expression
                },
                ordinal: 0,
            }],
            recursion_group: None,
            declaration: declaration.map(|names| CheckedDeclaration {
                qualified_name: names.iter().map(|name| (*name).into()).collect(),
            }),
            owner: hybrid_decl.then(|| CheckedNodeOwner::ModelRoot {
                identity: "domain".into(),
            }),
            owner_model: None,
            body: CheckedJson::from_serializable(term).unwrap(),
        }
    }
    fn fixture(claim: bool) -> Fixture {
        let kinds = vec![
            CheckedNodeKind::ScalarType(ScalarTypeForm::Rational),
            CheckedNodeKind::ScalarType(ScalarTypeForm::Boolean),
            CheckedNodeKind::ScalarType(ScalarTypeForm::Text),
            CheckedNodeKind::ScalarType(ScalarTypeForm::Integer),
            CheckedNodeKind::Value(ValueForm::Parameter),
            CheckedNodeKind::Value(ValueForm::Literal),
            CheckedNodeKind::Value(ValueForm::Literal),
            CheckedNodeKind::Hybrid(HybridForm::Model),
            CheckedNodeKind::Hybrid(HybridForm::Mode),
            CheckedNodeKind::Hybrid(HybridForm::Transition),
        ];
        let terms = vec![
            aggregate(vec![]),
            aggregate(vec![]),
            aggregate(vec![]),
            aggregate(vec![]),
            parameter("x", 0),
            CheckedTerm::Literal {
                type_id: id(1),
                value_kind: LiteralKind::Boolean,
                value: CheckedJson::scalar(&true),
            },
            CheckedTerm::literal_rational(id(0), "1"),
            aggregate(vec![
                binding(
                    "variables",
                    aggregate(vec![aggregate(vec![
                        binding("parameter", reference(4)),
                        binding("class", text("hybrid")),
                    ])]),
                ),
                binding("modes", aggregate(vec![reference(8)])),
            ]),
            aggregate(vec![
                binding("invariant", reference(5)),
                binding("rates", aggregate(vec![reference(6)])),
                binding("transitions", aggregate(vec![reference(9)])),
            ]),
            aggregate(vec![
                binding("target", integer(0)),
                binding("guard", reference(5)),
                binding(
                    "resets",
                    aggregate(vec![aggregate(vec![
                        binding("variable", integer(0)),
                        binding("value", reference(6)),
                    ])]),
                ),
            ]),
        ];
        let types = [0, 1, 2, 3, 0, 1, 0, 7, 7, 7];
        let deps: &[&[usize]] = &[
            &[],
            &[],
            &[],
            &[],
            &[2, 3],
            &[1],
            &[0],
            &[2, 4, 8],
            &[5, 6, 9],
            &[3, 5, 6],
        ];
        let nodes = (0..terms.len())
            .map(|i| {
                node(
                    i,
                    kinds[i],
                    types[i],
                    &terms[i],
                    deps[i],
                    match i {
                        7 => Some(&["Tank"][..]),
                        8 => Some(&["Tank", "Open"][..]),
                        _ => None,
                    },
                )
            })
            .collect();
        let mut f = Fixture {
            nodes,
            kinds,
            terms: terms.into_iter().map(Some).collect(),
        };
        if claim {
            for (kind, ty, term, deps) in [
                (
                    CheckedNodeKind::Temporal(TemporalForm::Formula),
                    1,
                    application(
                        "quire.op.temporal.holds",
                        ApplicationOperator::TemporalFormula,
                        vec![reference(11)],
                    ),
                    vec![1, 11],
                ),
                (
                    CheckedNodeKind::Value(ValueForm::Parameter),
                    0,
                    parameter("captured", 2),
                    vec![2, 3],
                ),
                (
                    CheckedNodeKind::Value(ValueForm::Parameter),
                    7,
                    parameter("over", 0),
                    vec![2, 3],
                ),
                (
                    CheckedNodeKind::Value(ValueForm::Parameter),
                    7,
                    parameter("trigger", 1),
                    vec![2, 3],
                ),
                (
                    CheckedNodeKind::Temporal(TemporalForm::TemporalClause),
                    1,
                    application(
                        "quire.op.temporal.clause",
                        ApplicationOperator::Temporal,
                        vec![
                            reference(12),
                            text("model-time"),
                            aggregate(vec![
                                binding("kind", text("each")),
                                binding("trigger", reference(13)),
                            ]),
                            aggregate(vec![binding("captured", reference(6))]),
                            aggregate(vec![]),
                            reference(10),
                        ],
                    ),
                    vec![1, 2, 6, 10, 12, 13],
                ),
            ] {
                let index = f.nodes.len();
                f.nodes.push(node(index, kind, ty, &term, &deps, None));
                f.kinds.push(kind);
                f.terms.push(Some(term));
            }
        }
        f
    }
    fn run(
        f: &Fixture,
        storage: u64,
        work_limit: u64,
    ) -> (Result<(), ValidationFailure>, u64, NativeJoinBudget) {
        let index: GraphIndex<'_> = f
            .nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (&node.node_id, i))
            .collect();
        let mut budget = NativeJoinBudget::new(work_limit, 0).with_read_storage_limit(storage);
        let mut result = None;
        let measured = allocation_counter::measure(|| {
            result = Some((|| {
                for (i, kind) in f.kinds.iter().enumerate() {
                    if let CheckedNodeKind::Hybrid(form) = kind {
                        check_wire(&f.nodes[i], *form, i, &mut budget)?;
                    }
                }
                let mut meter = WorkMeter::over(&mut budget);
                validate_models(&f.nodes, &f.kinds, &f.terms, &index, &mut meter)?;
                validate_claims(&f.nodes, &f.kinds, &f.terms, &index, &mut meter)
            })());
        });
        (result.unwrap(), measured.bytes_total, budget)
    }

    #[trace("TC-472", "FR-038-AC-511")]
    #[test]
    fn hybrid_actual_model_and_claim_chain_prepays_every_block() {
        for claim in [false, true] {
            let f = fixture(claim);
            let (result, allocated, budget) = run(&f, u64::MAX, u64::MAX);
            result.expect("a model's own variable and an ordered typed capture are admitted");
            let required = budget.read_storage_consumed();
            assert_eq!(allocated, required);
            assert_eq!(budget.consumed().work, if claim { 21 } else { 9 });
            let (exact, allocated, budget) = run(&f, required, u64::MAX);
            exact.unwrap();
            assert_eq!(allocated, required);
            assert_eq!(budget.read_storage_consumed(), required);
            let (short, allocated, budget) = run(&f, required - 1, u64::MAX);
            assert!(matches!(short, Err(ValidationFailure::Budget(_))));
            assert_eq!(allocated, budget.read_storage_consumed());
            assert!(allocated < required);
            let (zero, allocated, budget) = run(&f, 0, u64::MAX);
            assert!(matches!(zero, Err(ValidationFailure::Budget(_))));
            assert_eq!((allocated, budget.read_storage_consumed()), (0, 0));
        }
    }

    #[trace("TC-048", "FR-038-AC-508")]
    #[test]
    fn hybrid_exact_work_and_last_visit_denial_keep_their_points() {
        let f = fixture(true);
        let (exact, _, budget) = run(&f, u64::MAX, 21);
        exact.unwrap();
        assert_eq!(budget.consumed().work, 21);
        let (short, allocated, budget) = run(&f, u64::MAX, 20);
        let Err(ValidationFailure::Budget(stop)) = short else {
            panic!("last scalar-base visit must refuse")
        };
        assert_eq!(stop.amount, 1);
        assert_eq!(stop.path.unwrap().as_str(), "/semantic_graph/nodes/10");
        assert_eq!(budget.consumed().work, 20);
        assert_eq!(allocated, budget.read_storage_consumed());
        let (zero, allocated, budget) = run(&f, u64::MAX, 0);
        let Err(ValidationFailure::Budget(stop)) = zero else {
            panic!("the model's first visit must refuse")
        };
        assert_eq!(stop.path.unwrap().as_str(), "/semantic_graph/nodes/7");
        assert_eq!(budget.consumed().work, 0);
        assert_eq!(allocated, budget.read_storage_consumed());
    }
    fn set_modes(f: &mut Fixture, modes: &[usize]) {
        let CheckedTerm::Aggregate { members } = f.terms[7].as_mut().unwrap() else {
            unreachable!()
        };
        let CheckedTerm::Binding { value, .. } = &mut members[1] else {
            unreachable!()
        };
        **value = aggregate(modes.iter().map(|i| reference(*i)).collect());
        f.nodes[7].body = CheckedJson::from_serializable(f.terms[7].as_ref().unwrap()).unwrap();
    }
    fn set_class(f: &mut Fixture, name: &str) {
        let CheckedTerm::Aggregate { members } = f.terms[7].as_mut().unwrap() else {
            unreachable!()
        };
        let CheckedTerm::Binding { value, .. } = &mut members[0] else {
            unreachable!()
        };
        let CheckedTerm::Aggregate { members } = value.as_mut() else {
            unreachable!()
        };
        let CheckedTerm::Aggregate { members } = &mut members[0] else {
            unreachable!()
        };
        let CheckedTerm::Binding { value, .. } = &mut members[1] else {
            unreachable!()
        };
        **value = text(name);
        f.nodes[7].body = CheckedJson::from_serializable(f.terms[7].as_ref().unwrap()).unwrap();
    }

    #[trace("TC-048", "FR-038-AC-313")]
    #[test]
    fn hybrid_duplicate_first_ordinal_precedes_later_sorted_key_and_clock_defect() {
        let mut f = fixture(false);
        let term = f.terms[8].as_ref().unwrap().clone();
        let kind = CheckedNodeKind::Hybrid(HybridForm::Mode);
        f.nodes.push(node(
            10,
            kind,
            7,
            &term,
            &[5, 6, 9],
            Some(&["Tank", "Second"]),
        ));
        f.kinds.push(kind);
        f.terms.push(Some(term));
        set_modes(&mut f, &[10, 8, 10, 8]);
        set_class(&mut f, "clock");
        let (failure, allocated, budget) = run(&f, u64::MAX, u64::MAX);
        let Err(ValidationFailure::Refused(refusal)) = failure else {
            panic!("first repeated mode must refuse")
        };
        assert_eq!(refusal.code, Code::InvalidSemanticGraph);
        assert_eq!(
            refusal.primary_path().unwrap().as_str(),
            "/semantic_graph/nodes/7/body/members/1/value/members/2"
        );
        assert_eq!(allocated, budget.read_storage_consumed());
        assert_eq!(budget.consumed().work, 1);
        let required = budget.read_storage_consumed();
        assert!(matches!(
            run(&f, required, u64::MAX).0,
            Err(ValidationFailure::Refused(_))
        ));
        let (short, allocated, budget) = run(&f, required - 1, u64::MAX);
        assert!(matches!(short, Err(ValidationFailure::Budget(_))));
        assert_eq!(allocated, budget.read_storage_consumed());
    }

    #[trace("TC-472", "FR-038-AC-511")]
    #[test]
    fn hybrid_thirteen_step_wire_fault_is_exact_and_precedes_graph_work() {
        let mut f = fixture(false);
        set_class(&mut f, "continuous");
        let (failure, allocated, budget) = run(&f, u64::MAX, u64::MAX);
        let Err(ValidationFailure::Refused(refusal)) = failure else {
            panic!("closed class word must refuse")
        };
        let path = "/semantic_graph/nodes/7/body/members/0/value/members/0/members/1/value/value";
        assert_eq!(refusal.code, Code::MalformedWire);
        assert_eq!(refusal.primary_path().unwrap().as_str(), path);
        assert_eq!(allocated, budget.read_storage_consumed());
        assert_eq!(budget.consumed().work, 0);
        let required = budget.read_storage_consumed();
        let (short, allocated, budget) = run(&f, required - 1, u64::MAX);
        assert!(matches!(short, Err(ValidationFailure::Budget(_))));
        assert_eq!((allocated, budget.read_storage_consumed()), (0, 0));
        assert_eq!(
            budget.first_unpaid_locus().unwrap().source,
            super::super::NativeJoinLocusSource::PackageRead(PackageReadStage::WireChecks, 7)
        );
        assert!(matches!(
            run(&f, required, u64::MAX).0,
            Err(ValidationFailure::Refused(_))
        ));
    }

    fn nested(depth: usize, leaf: Segment, ordinal: u64) -> CheckedNodeId {
        let mut key = CheckedNodeId {
            package: "dependency".into(),
            path: vec![leaf],
            ordinal,
        };
        for _ in 0..depth {
            key = CheckedNodeId {
                package: "dependency".into(),
                path: vec![Segment::Name("prefix".into()), Segment::Node(Arc::new(key))],
                ordinal,
            };
        }
        key
    }

    #[trace("TC-472", "FR-038-AC-511")]
    #[test]
    fn hybrid_prepaid_sort_preserves_ord_and_reuses_scratch_without_allocator_requests() {
        let keys: Vec<_> = [0, 1, 2, 31, 32, 64]
            .into_iter()
            .flat_map(|depth| {
                [
                    Segment::Name("a".into()),
                    Segment::Name("b".into()),
                    Segment::Index(1),
                    Segment::Operand(2),
                ]
                .into_iter()
                .map(move |leaf| nested(depth, leaf, (depth % 2) as u64))
            })
            .collect();
        let mut expected: Vec<_> = keys.iter().collect();
        expected.sort_unstable();
        let mut rows: Vec<_> = keys.iter().rev().collect();
        let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(u64::MAX);
        let mut sort = IdSort::new(65);
        {
            let mut meter = WorkMeter::over(&mut budget);
            meter.at_node(0, "hybrid");
            // Preparation is paid outside the measured infallible sort/drop.
            sort.prepare(true, &mut meter).unwrap();
        }
        let required = budget.read_storage_consumed();
        assert_eq!(required as usize, CheckedNodeIdOrder::bytes(65));
        let measured = allocation_counter::measure(|| {
            let mut meter = WorkMeter::over(&mut budget);
            sort_ids(&mut rows, &mut meter, &mut sort).unwrap();
            rows.reverse();
            sort_ids(&mut rows, &mut meter, &mut sort).unwrap();
            drop(sort);
        });
        assert_eq!(measured.bytes_total, 0, "including reusable scratch drop");
        assert_eq!(budget.read_storage_consumed(), required);
        assert_eq!(rows, expected);
        for limit in [0, required - 1] {
            let mut rows: Vec<_> = keys.iter().rev().collect();
            let before = rows.clone();
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut result = None;
            let measured = allocation_counter::measure(|| {
                let mut meter = WorkMeter::over(&mut budget);
                meter.at_node(0, "hybrid");
                result = Some(sort_ids(&mut rows, &mut meter, &mut IdSort::new(65)));
            });
            assert!(matches!(result.unwrap(), Err(ValidationFailure::Budget(_))));
            assert_eq!(rows, before, "denial precedes the sort");
            assert_eq!(measured.bytes_total, budget.read_storage_consumed());
            assert!(budget.read_storage_consumed() <= limit);
        }
        let flat = [id(2), id(0), id(1)];
        let mut rows: Vec<_> = flat.iter().collect();
        let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(0);
        let measured = allocation_counter::measure(|| {
            sort_ids(
                &mut rows,
                &mut WorkMeter::over(&mut budget),
                &mut IdSort::new(flat.len()),
            )
            .unwrap();
        });
        assert_eq!(
            (measured.bytes_total, budget.read_storage_consumed()),
            (0, 0)
        );
        assert_eq!(rows, [&flat[1], &flat[2], &flat[0]]);
    }

    /// The producer's original flat body handles stay retained while the
    /// native graph/typed terms acquire their authenticated DAG identities.
    fn deep_fixture(depth: usize) -> Fixture {
        let mut f = fixture(true);
        let old: Vec<_> = f.nodes.iter().map(|node| node.node_id.clone()).collect();
        let mut prefixes = Vec::new();
        let mut roots = Vec::new();
        for branch in ["a", "b"] {
            let mut key = Arc::new(CheckedNodeId {
                package: "dependency".into(),
                path: vec![Segment::Name(branch.into())],
                ordinal: 0,
            });
            prefixes.push(key.clone());
            for _ in 0..depth {
                key = Arc::new(CheckedNodeId {
                    package: "dependency".into(),
                    path: vec![Segment::Node(key)],
                    ordinal: 0,
                });
                prefixes.push(key.clone());
            }
            roots.push(key);
        }
        let replacements: Vec<_> = old
            .iter()
            .enumerate()
            .map(|(i, key)| CheckedNodeId {
                package: key.package.clone(),
                path: vec![
                    Segment::Node(roots[i % 2].clone()),
                    Segment::Name(format!("n{i:04}").into()),
                ],
                ordinal: 0,
            })
            .collect();
        let map = |key: &CheckedNodeId| {
            old.iter()
                .position(|old| old == key)
                .map(|i| replacements[i].clone())
        };
        for node in &mut f.nodes {
            node.node_id = map(&node.node_id).unwrap();
            node.semantic_type = map(&node.semantic_type).unwrap();
            for key in &mut node.dependencies {
                *key = map(key).unwrap();
            }
        }
        for term in f.terms.iter_mut().flatten() {
            term.map_node_ids(&map);
        }
        for key in prefixes {
            let kind = CheckedNodeKind::ScalarType(ScalarTypeForm::Rational);
            let term = aggregate(vec![]);
            let mut added = node(f.nodes.len(), kind, 0, &term, &[], None);
            added.node_id = (*key).clone();
            added.semantic_type = replacements[0].clone();
            f.nodes.push(added);
            f.kinds.push(kind);
            f.terms.push(Some(term));
        }
        for node in &f.nodes {
            for segment in &node.node_id.path {
                if let Segment::Node(child) = segment {
                    assert!(
                        f.nodes.iter().any(|node| node.node_id == **child),
                        "every dependency/model operand is a graph key"
                    );
                }
            }
        }
        f
    }

    #[trace("TC-472", "FR-038-AC-511")]
    #[test]
    fn hybrid_complete_deep_dependency_dag_pays_comparison_scratch_and_returned_drop() {
        let run = |limit| {
            let f = deep_fixture(64);
            let mut budget = NativeJoinBudget::new(u64::MAX, 0).with_read_storage_limit(limit);
            let mut returned = None;
            let mut phases = [(0_u64, 0_u64); 5];
            let measured = allocation_counter::measure(|| {
                let result = (|| {
                    let mut meter = WorkMeter::over(&mut budget);
                    meter.at_node(0, "hybrid");
                    let mut index = None;
                    let before = meter.budget_and_site().unwrap().0.read_storage_consumed();
                    phases[0].0 = allocation_counter::measure(|| {
                        index = Some(GraphIndex::build(
                            f.nodes.iter().map(|node| &node.node_id),
                            &mut meter,
                        ));
                    })
                    .bytes_total;
                    phases[0].1 =
                        meter.budget_and_site().unwrap().0.read_storage_consumed() - before;
                    let index = index.unwrap()?;
                    let mut outcome = None;
                    let before = meter.budget_and_site().unwrap().0.read_storage_consumed();
                    phases[1].0 = allocation_counter::measure(|| {
                        outcome = Some(validate_models(
                            &f.nodes, &f.kinds, &f.terms, &index, &mut meter,
                        ));
                    })
                    .bytes_total;
                    phases[1].1 =
                        meter.budget_and_site().unwrap().0.read_storage_consumed() - before;
                    outcome.take().unwrap()?;
                    let before = meter.budget_and_site().unwrap().0.read_storage_consumed();
                    phases[2].0 = allocation_counter::measure(|| {
                        outcome = Some(validate_claims(
                            &f.nodes, &f.kinds, &f.terms, &index, &mut meter,
                        ));
                    })
                    .bytes_total;
                    phases[2].1 =
                        meter.budget_and_site().unwrap().0.read_storage_consumed() - before;
                    outcome.unwrap()
                })();
                returned = Some(result);
            });
            let result = returned.unwrap();
            let success = result.is_ok();
            assert!(success || matches!(result, Err(ValidationFailure::Budget(_))));
            // Returned outcomes and externally prepared fixture ownership are
            // separate observations, outside the actual during-call total.
            phases[3].0 = allocation_counter::measure(|| drop(result)).bytes_total;
            phases[4].0 = allocation_counter::measure(|| drop(f)).bytes_total;
            assert_eq!(phases[3].0, 0, "returned outcome drop requests no heap");
            (success, measured.bytes_total, budget, phases)
        };
        let (success, allocated, budget, phases) = run(u64::MAX);
        assert!(success);
        let required = budget.read_storage_consumed();
        assert_eq!(allocated, required, "phase allocated/charged: {phases:?}");
        assert_eq!(budget.consumed().work, 21);
        let (success, allocated, budget, phases) = run(required);
        assert!(success);
        assert_eq!(
            (allocated, budget.read_storage_consumed()),
            (required, required),
            "{phases:?}"
        );
        let (success, allocated, budget, phases) = run(required - 1);
        assert!(!success);
        assert_eq!(allocated, budget.read_storage_consumed(), "{phases:?}");
        let (success, allocated, budget, phases) = run(0);
        assert!(!success);
        assert_eq!(
            (allocated, budget.read_storage_consumed()),
            (0, 0),
            "{phases:?}"
        );
    }
}
