use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    DefinednessObligationKind, DependencyIdentity, DependencyKind, DependencyName,
    DependencySource, Diagnostic, DiagnosticCode, ExecutionPoint, RequirementRef, SourceSpan,
    StateObservation,
};

pub const MAX_EXPRESSION_NODES: u32 = 10_000;
pub const MAX_EXPRESSION_DEPTH: u32 = 256;
pub const MAX_TEXT_LENGTH: u32 = 1_048_576;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct SymbolName(String);

impl SymbolName {
    pub fn new(value: impl Into<String>) -> Result<Self, Diagnostic> {
        let value = value.into();
        let mut chars = value.chars();
        let valid = matches!(chars.next(), Some(first) if first.is_ascii_alphabetic())
            && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'));
        if valid {
            Ok(Self(value))
        } else {
            Err(Diagnostic::error(
                DiagnosticCode::InvalidIdentifier,
                "symbol name violates the contract identifier grammar",
                "declaration.name",
            ))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegerDomain {
    Signed,
    Unsigned,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OverflowPolicy {
    Reject,
    Saturate,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct IntegerType {
    domain: IntegerDomain,
    minimum: i64,
    maximum: i64,
    overflow: OverflowPolicy,
}

impl IntegerType {
    pub fn new(
        domain: IntegerDomain,
        minimum: i64,
        maximum: i64,
        overflow: OverflowPolicy,
    ) -> Result<Self, Diagnostic> {
        if minimum > maximum || (domain == IntegerDomain::Unsigned && minimum < 0) {
            Err(Diagnostic::error(
                DiagnosticCode::InvalidNumericBounds,
                "integer bounds are unordered or violate the unsigned domain",
                "type.integer.bounds",
            ))
        } else {
            Ok(Self {
                domain,
                minimum,
                maximum,
                overflow,
            })
        }
    }

    pub const fn domain(&self) -> IntegerDomain {
        self.domain
    }

    pub const fn minimum(&self) -> i64 {
        self.minimum
    }

    pub const fn maximum(&self) -> i64 {
        self.maximum
    }

    pub const fn overflow(&self) -> OverflowPolicy {
        self.overflow
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct RationalType {
    numerator_minimum: i64,
    numerator_maximum: i64,
    maximum_denominator: u64,
}

impl RationalType {
    pub fn new(
        numerator_minimum: i64,
        numerator_maximum: i64,
        maximum_denominator: u64,
    ) -> Result<Self, Diagnostic> {
        if numerator_minimum > numerator_maximum
            || maximum_denominator == 0
            || maximum_denominator > i64::MAX as u64
        {
            Err(Diagnostic::error(
                DiagnosticCode::InvalidNumericBounds,
                "rational numerator or denominator bounds are invalid",
                "type.rational.bounds",
            ))
        } else {
            Ok(Self {
                numerator_minimum,
                numerator_maximum,
                maximum_denominator,
            })
        }
    }

    pub const fn numerator_minimum(&self) -> i64 {
        self.numerator_minimum
    }

    pub const fn numerator_maximum(&self) -> i64 {
        self.numerator_maximum
    }

    pub const fn maximum_denominator(&self) -> u64 {
        self.maximum_denominator
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct CollectionType {
    element: Box<ValueType>,
    maximum_items: u32,
}

impl CollectionType {
    pub fn new(element: ValueType, maximum_items: u32) -> Result<Self, Diagnostic> {
        if maximum_items == 0 {
            Err(Diagnostic::error(
                DiagnosticCode::UnboundedCollection,
                "collection maximum must be positive",
                "type.collection.maximum_items",
            ))
        } else {
            Ok(Self {
                element: Box::new(element),
                maximum_items,
            })
        }
    }

    pub fn element(&self) -> &ValueType {
        &self.element
    }

    pub const fn maximum_items(&self) -> u32 {
        self.maximum_items
    }

    pub fn index_type(&self) -> IntegerType {
        IntegerType {
            domain: IntegerDomain::Unsigned,
            minimum: 0,
            maximum: i64::from(self.maximum_items),
            overflow: OverflowPolicy::Reject,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ValueType {
    Boolean,
    Integer { value: IntegerType },
    Rational { value: RationalType },
    Text,
    Enum { name: SymbolName },
    Record { name: SymbolName },
    Option { value: Box<ValueType> },
    Collection { value: CollectionType },
}

impl ValueType {
    pub fn integer(value: IntegerType) -> Self {
        Self::Integer { value }
    }

    pub fn rational(value: RationalType) -> Self {
        Self::Rational { value }
    }

    pub fn option(value: ValueType) -> Self {
        Self::Option {
            value: Box::new(value),
        }
    }

    pub fn collection(value: CollectionType) -> Self {
        Self::Collection { value }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EnumVariantDeclaration {
    name: SymbolName,
    source: SourceSpan,
}

impl EnumVariantDeclaration {
    pub fn new(name: SymbolName, source: SourceSpan) -> Self {
        Self { name, source }
    }

    pub fn name(&self) -> &SymbolName {
        &self.name
    }

    pub fn source(&self) -> &SourceSpan {
        &self.source
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EnumDeclaration {
    name: SymbolName,
    source: SourceSpan,
    variants: Vec<EnumVariantDeclaration>,
}

impl EnumDeclaration {
    pub fn new(
        name: SymbolName,
        source: SourceSpan,
        variants: Vec<EnumVariantDeclaration>,
    ) -> Result<Self, Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        if variants.is_empty() {
            diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::EmptyEnum,
                    "enum must contain a variant",
                    "enum",
                )
                .at_span(&source),
            );
        }
        let mut seen = BTreeSet::new();
        for variant in &variants {
            if !seen.insert(variant.name.clone()) {
                diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::DuplicateVariant,
                        "enum variant name is duplicated",
                        "enum.variants",
                    )
                    .at_span(&variant.source),
                );
            }
        }
        if diagnostics.is_empty() {
            Ok(Self {
                name,
                source,
                variants,
            })
        } else {
            Err(diagnostics)
        }
    }

    pub fn name(&self) -> &SymbolName {
        &self.name
    }

    pub fn source(&self) -> &SourceSpan {
        &self.source
    }

    pub fn variants(&self) -> &[EnumVariantDeclaration] {
        &self.variants
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RecordFieldDeclaration {
    name: SymbolName,
    value_type: ValueType,
    source: SourceSpan,
}

impl RecordFieldDeclaration {
    pub fn new(name: SymbolName, value_type: ValueType, source: SourceSpan) -> Self {
        Self {
            name,
            value_type,
            source,
        }
    }

    pub fn name(&self) -> &SymbolName {
        &self.name
    }

    pub fn value_type(&self) -> &ValueType {
        &self.value_type
    }

    pub fn source(&self) -> &SourceSpan {
        &self.source
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RecordDeclaration {
    name: SymbolName,
    source: SourceSpan,
    fields: Vec<RecordFieldDeclaration>,
}

impl RecordDeclaration {
    pub fn new(
        name: SymbolName,
        source: SourceSpan,
        fields: Vec<RecordFieldDeclaration>,
    ) -> Result<Self, Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        let mut seen = BTreeSet::new();
        for field in &fields {
            if !seen.insert(field.name.clone()) {
                diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::DuplicateField,
                        "record field name is duplicated",
                        "record.fields",
                    )
                    .at_span(&field.source),
                );
            }
        }
        if diagnostics.is_empty() {
            Ok(Self {
                name,
                source,
                fields,
            })
        } else {
            Err(diagnostics)
        }
    }

    pub fn name(&self) -> &SymbolName {
        &self.name
    }

    pub fn source(&self) -> &SourceSpan {
        &self.source
    }

    pub fn fields(&self) -> &[RecordFieldDeclaration] {
        &self.fields
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TypeDeclaration {
    Enum { declaration: EnumDeclaration },
    Record { declaration: RecordDeclaration },
}

impl TypeDeclaration {
    pub fn name(&self) -> &SymbolName {
        match self {
            Self::Enum { declaration } => declaration.name(),
            Self::Record { declaration } => declaration.name(),
        }
    }

    pub fn source(&self) -> &SourceSpan {
        match self {
            Self::Enum { declaration } => declaration.source(),
            Self::Record { declaration } => declaration.source(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueDeclarationKind {
    Input,
    State,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ValueDeclaration {
    name: SymbolName,
    kind: ValueDeclarationKind,
    value_type: ValueType,
    source: SourceSpan,
}

impl ValueDeclaration {
    pub fn new(
        name: SymbolName,
        kind: ValueDeclarationKind,
        value_type: ValueType,
        source: SourceSpan,
    ) -> Self {
        Self {
            name,
            kind,
            value_type,
            source,
        }
    }

    pub fn name(&self) -> &SymbolName {
        &self.name
    }

    pub const fn kind(&self) -> ValueDeclarationKind {
        self.kind
    }

    pub fn value_type(&self) -> &ValueType {
        &self.value_type
    }

    pub fn source(&self) -> &SourceSpan {
        &self.source
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FunctionParameter {
    name: SymbolName,
    value_type: ValueType,
    source: SourceSpan,
}

impl FunctionParameter {
    pub fn new(name: SymbolName, value_type: ValueType, source: SourceSpan) -> Self {
        Self {
            name,
            value_type,
            source,
        }
    }

    pub fn name(&self) -> &SymbolName {
        &self.name
    }

    pub fn value_type(&self) -> &ValueType {
        &self.value_type
    }

    pub fn source(&self) -> &SourceSpan {
        &self.source
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PureFunctionDeclaration {
    name: SymbolName,
    parameters: Vec<FunctionParameter>,
    result_type: ValueType,
    source: SourceSpan,
}

impl PureFunctionDeclaration {
    pub fn new(
        name: SymbolName,
        parameters: Vec<FunctionParameter>,
        result_type: ValueType,
        source: SourceSpan,
    ) -> Result<Self, Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        let mut seen = BTreeSet::new();
        for parameter in &parameters {
            if !seen.insert(parameter.name.clone()) {
                diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::DuplicateParameter,
                        "function parameter name is duplicated",
                        "function.parameters",
                    )
                    .at_span(&parameter.source),
                );
            }
        }
        if diagnostics.is_empty() {
            Ok(Self {
                name,
                parameters,
                result_type,
                source,
            })
        } else {
            Err(diagnostics)
        }
    }

    pub fn name(&self) -> &SymbolName {
        &self.name
    }

    pub fn parameters(&self) -> &[FunctionParameter] {
        &self.parameters
    }

    pub fn result_type(&self) -> &ValueType {
        &self.result_type
    }

    pub fn source(&self) -> &SourceSpan {
        &self.source
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DeclarationEnvironment {
    owner: RequirementRef,
    types: Vec<TypeDeclaration>,
    values: Vec<ValueDeclaration>,
    functions: Vec<PureFunctionDeclaration>,
}

impl DeclarationEnvironment {
    pub fn new(
        owner: RequirementRef,
        types: Vec<TypeDeclaration>,
        values: Vec<ValueDeclaration>,
        functions: Vec<PureFunctionDeclaration>,
    ) -> Result<Self, Vec<Diagnostic>> {
        let candidate = Self {
            owner,
            types,
            values,
            functions,
        };
        if let Some(diagnostic) = candidate.preflight_semantic_input() {
            return Err(vec![diagnostic]);
        }
        let diagnostics = candidate.validate();
        if diagnostics.is_empty() {
            Ok(candidate)
        } else {
            Err(diagnostics)
        }
    }

    pub fn owner(&self) -> &RequirementRef {
        &self.owner
    }

    pub fn types(&self) -> &[TypeDeclaration] {
        &self.types
    }

    pub fn values(&self) -> &[ValueDeclaration] {
        &self.values
    }

    pub fn functions(&self) -> &[PureFunctionDeclaration] {
        &self.functions
    }

    fn preflight_semantic_input(&self) -> Option<Diagnostic> {
        if [self.types.len(), self.values.len(), self.functions.len()]
            .into_iter()
            .any(|length| length > crate::MAX_SEMANTIC_COLLECTION_ITEMS as usize)
        {
            return Some(semantic_input_too_large("declarations"));
        }
        let mut nodes = 0_u32;
        let mut value_types = Vec::new();
        for declaration in &self.types {
            nodes = nodes.saturating_add(1);
            match declaration {
                TypeDeclaration::Enum { declaration } => {
                    if declaration.variants.len() > crate::MAX_SEMANTIC_COLLECTION_ITEMS as usize {
                        return Some(semantic_input_too_large("types.enum.variants"));
                    }
                    nodes = nodes.saturating_add(
                        u32::try_from(declaration.variants.len()).unwrap_or(u32::MAX),
                    );
                }
                TypeDeclaration::Record { declaration } => {
                    if declaration.fields.len() > crate::MAX_SEMANTIC_COLLECTION_ITEMS as usize {
                        return Some(semantic_input_too_large("types.record.fields"));
                    }
                    nodes = nodes.saturating_add(
                        u32::try_from(declaration.fields.len()).unwrap_or(u32::MAX),
                    );
                    value_types.extend(
                        declaration
                            .fields
                            .iter()
                            .map(|field| (&field.value_type, 1_u32)),
                    );
                }
            }
        }
        for declaration in &self.values {
            nodes = nodes.saturating_add(1);
            value_types.push((&declaration.value_type, 1));
        }
        for declaration in &self.functions {
            if declaration.parameters.len() > crate::MAX_SEMANTIC_COLLECTION_ITEMS as usize {
                return Some(semantic_input_too_large("functions.parameters"));
            }
            nodes = nodes.saturating_add(
                u32::try_from(declaration.parameters.len() + 1).unwrap_or(u32::MAX),
            );
            value_types.push((&declaration.result_type, 1));
            value_types.extend(
                declaration
                    .parameters
                    .iter()
                    .map(|parameter| (&parameter.value_type, 1)),
            );
        }
        while let Some((value_type, depth)) = value_types.pop() {
            if depth > crate::MAX_SEMANTIC_DEPTH {
                return Some(semantic_input_too_large("type.depth"));
            }
            nodes = nodes.saturating_add(1);
            if nodes > crate::MAX_SEMANTIC_NODES {
                return Some(semantic_input_too_large("semantic.nodes"));
            }
            match value_type {
                ValueType::Option { value } => value_types.push((value, depth + 1)),
                ValueType::Collection { value } => {
                    value_types.push((value.element(), depth + 1));
                }
                _ => {}
            }
        }
        (nodes > crate::MAX_SEMANTIC_NODES).then(|| semantic_input_too_large("semantic.nodes"))
    }

    fn validate(&self) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut type_names = BTreeMap::new();
        for declaration in &self.types {
            if type_names
                .insert(declaration.name().clone(), declaration)
                .is_some()
            {
                diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::DuplicateTypeDeclaration,
                        "type declaration name is duplicated",
                        "types",
                    )
                    .at_span(declaration.source()),
                );
            }
        }
        let mut value_names = BTreeSet::new();
        for declaration in &self.values {
            if !value_names.insert(declaration.name.clone()) {
                diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::DuplicateValueDeclaration,
                        "value declaration name is duplicated",
                        "values",
                    )
                    .at_span(&declaration.source),
                );
            }
        }
        let mut function_names = BTreeSet::new();
        for declaration in &self.functions {
            if !function_names.insert(declaration.name.clone()) {
                diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::DuplicateFunctionDeclaration,
                        "function declaration name is duplicated",
                        "functions",
                    )
                    .at_span(&declaration.source),
                );
            }
        }
        if !diagnostics.is_empty() {
            return diagnostics;
        }
        for declaration in &self.types {
            if let TypeDeclaration::Record { declaration } = declaration {
                for field in &declaration.fields {
                    if let Err(diagnostic) = self.validate_type(&field.value_type, &field.source) {
                        diagnostics.push(diagnostic);
                    }
                }
            }
        }
        for declaration in &self.values {
            if let Err(diagnostic) =
                self.validate_type(&declaration.value_type, &declaration.source)
            {
                diagnostics.push(diagnostic);
            }
        }
        for declaration in &self.functions {
            for parameter in &declaration.parameters {
                if let Err(diagnostic) =
                    self.validate_type(&parameter.value_type, &parameter.source)
                {
                    diagnostics.push(diagnostic);
                }
            }
            if let Err(diagnostic) =
                self.validate_type(&declaration.result_type, &declaration.source)
            {
                diagnostics.push(diagnostic);
            }
        }
        if diagnostics.is_empty() {
            if let Some(diagnostic) = self.detect_record_cycle() {
                diagnostics.push(diagnostic);
            }
        }
        diagnostics
    }

    fn validate_type(&self, value_type: &ValueType, span: &SourceSpan) -> Result<(), Diagnostic> {
        match value_type {
            ValueType::Enum { name } => match self.type_declaration(name) {
                Some(TypeDeclaration::Enum { .. }) => Ok(()),
                _ => Err(orphaned_type(span)),
            },
            ValueType::Record { name } => match self.type_declaration(name) {
                Some(TypeDeclaration::Record { .. }) => Ok(()),
                _ => Err(orphaned_type(span)),
            },
            ValueType::Option { value } => self.validate_type(value, span),
            ValueType::Collection { value } => self.validate_type(value.element(), span),
            _ => Ok(()),
        }
    }

    fn detect_record_cycle(&self) -> Option<Diagnostic> {
        let mut visited = BTreeSet::new();
        for declaration in &self.types {
            if let TypeDeclaration::Record { declaration } = declaration {
                let mut active = BTreeSet::new();
                if let Some(field_span) =
                    self.visit_record(&declaration.name, &mut active, &mut visited)
                {
                    return Some(
                        Diagnostic::error(
                            DiagnosticCode::RecursiveType,
                            "record containment graph contains a cycle",
                            "types.record.fields",
                        )
                        .at_span(&field_span),
                    );
                }
            }
        }
        None
    }

    fn visit_record(
        &self,
        name: &SymbolName,
        active: &mut BTreeSet<SymbolName>,
        visited: &mut BTreeSet<SymbolName>,
    ) -> Option<SourceSpan> {
        if visited.contains(name) {
            return None;
        }
        active.insert(name.clone());
        if let Some(record) = self.record_declaration(name) {
            for field in &record.fields {
                let mut referenced = Vec::new();
                record_references(&field.value_type, &mut referenced);
                for next in referenced {
                    if active.contains(&next) {
                        active.remove(name);
                        return Some(field.source.clone());
                    }
                    if let Some(span) = self.visit_record(&next, active, visited) {
                        active.remove(name);
                        return Some(span);
                    }
                }
            }
        }
        active.remove(name);
        visited.insert(name.clone());
        None
    }

    fn type_declaration(&self, name: &SymbolName) -> Option<&TypeDeclaration> {
        self.types
            .iter()
            .find(|declaration| declaration.name() == name)
    }

    fn record_declaration(&self, name: &SymbolName) -> Option<&RecordDeclaration> {
        match self.type_declaration(name) {
            Some(TypeDeclaration::Record { declaration }) => Some(declaration),
            _ => None,
        }
    }

    fn enum_declaration(&self, name: &SymbolName) -> Option<&EnumDeclaration> {
        match self.type_declaration(name) {
            Some(TypeDeclaration::Enum { declaration }) => Some(declaration),
            _ => None,
        }
    }

    fn value_declaration(&self, name: &SymbolName) -> Option<&ValueDeclaration> {
        self.values
            .iter()
            .find(|declaration| &declaration.name == name)
    }

    fn function_declaration(&self, name: &SymbolName) -> Option<&PureFunctionDeclaration> {
        self.functions
            .iter()
            .find(|declaration| &declaration.name == name)
    }

    pub fn check_expression(
        &self,
        expression: &Expression,
        expected: &ValueType,
        execution_point: &ExecutionPoint,
        clause_root: bool,
    ) -> Result<TypedExpression, Vec<Diagnostic>> {
        let maximum_depth = preflight(expression)?;
        let check = || {
            let mut counter = 0_u32;
            check_node(self, expression, execution_point, &[], &[], &mut counter)
        };
        let checked = if maximum_depth < MAX_EXPRESSION_DEPTH as usize / 2 {
            check()
        } else {
            std::thread::scope(|scope| {
                match std::thread::Builder::new()
                    .name("contract-expression-check".to_owned())
                    .stack_size(8 * 1024 * 1024)
                    .spawn_scoped(scope, check)
                {
                    Ok(handle) => match handle.join() {
                        Ok(result) => result,
                        Err(payload) => std::panic::resume_unwind(payload),
                    },
                    Err(_) => Err(single(
                        expression,
                        DiagnosticCode::ExpressionTooLarge,
                        "expression cannot be validated within the bounded stack resource",
                    )),
                }
            })
        }?;
        if clause_root && checked.value_type != ValueType::Boolean {
            return Err(vec![Diagnostic::error(
                DiagnosticCode::NonBooleanClauseRoot,
                "executable clause root must be Boolean",
                "expression.root",
            )
            .at_span(&expression.source)]);
        }
        if &checked.value_type != expected {
            return Err(vec![Diagnostic::error(
                DiagnosticCode::ResultTypeMismatch,
                "expression result differs from expected type",
                "expression.root",
            )
            .at_span(&expression.source)]);
        }
        Ok(TypedExpression {
            expression: expression.clone(),
            value_type: checked.value_type,
            nodes: checked.nodes,
            obligations: checked.obligations,
            dependencies: checked.dependencies,
        })
    }
}

fn orphaned_type(span: &SourceSpan) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::OrphanedTypeReference,
        "named type declaration does not resolve",
        "type.name",
    )
    .at_span(span)
}

fn semantic_input_too_large(path: &'static str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::SemanticInputTooLarge,
        "semantic input exceeds a fixed validation limit",
        path,
    )
}

fn record_references(value_type: &ValueType, output: &mut Vec<SymbolName>) {
    match value_type {
        ValueType::Record { name } => output.push(name.clone()),
        ValueType::Option { value } => record_references(value, output),
        ValueType::Collection { value } => record_references(value.element(), output),
        _ => {}
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NumericOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BooleanOperator {
    ShortCircuitAnd,
    ShortCircuitOr,
    TotalAnd,
    TotalOr,
    Implication,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QuantifierKind {
    ForAll,
    Exists,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QuantifierDomain {
    Elements,
    Indices,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RecordLiteralField {
    name: SymbolName,
    value: Expression,
}

impl RecordLiteralField {
    pub fn new(name: SymbolName, value: Expression) -> Self {
        Self { name, value }
    }

    pub fn name(&self) -> &SymbolName {
        &self.name
    }

    pub fn value(&self) -> &Expression {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Expression {
    kind: ExpressionKind,
    source: SourceSpan,
}

impl Expression {
    pub fn new(mut kind: ExpressionKind, source: SourceSpan) -> Self {
        if let ExpressionKind::RationalLiteral {
            numerator,
            denominator,
            ..
        } = &mut kind
        {
            if *denominator > 0 {
                let divisor = gcd(numerator.unsigned_abs(), *denominator as u64) as i64;
                *numerator /= divisor;
                *denominator /= divisor;
            }
        }
        Self { kind, source }
    }

    pub fn kind(&self) -> &ExpressionKind {
        &self.kind
    }

    pub fn source(&self) -> &SourceSpan {
        &self.source
    }
}

impl Drop for Expression {
    fn drop(&mut self) {
        let mut pending = owned_children(std::mem::replace(
            &mut self.kind,
            ExpressionKind::BooleanLiteral { value: false },
        ));
        while let Some(mut expression) = pending.pop() {
            pending.extend(owned_children(std::mem::replace(
                &mut expression.kind,
                ExpressionKind::BooleanLiteral { value: false },
            )));
        }
    }
}

fn owned_children(kind: ExpressionKind) -> Vec<Expression> {
    match kind {
        ExpressionKind::OptionSome { value, .. } => vec![*value],
        ExpressionKind::RecordLiteral { fields, .. } => {
            fields.into_iter().map(|field| field.value).collect()
        }
        ExpressionKind::CollectionLiteral { items, .. }
        | ExpressionKind::Call {
            arguments: items, ..
        } => items,
        ExpressionKind::FieldAccess { base, .. }
        | ExpressionKind::IsPresent { option: base }
        | ExpressionKind::Unwrap { option: base }
        | ExpressionKind::Length { collection: base }
        | ExpressionKind::NumericNegate { operand: base }
        | ExpressionKind::BooleanNot { operand: base } => vec![*base],
        ExpressionKind::Index { collection, index }
        | ExpressionKind::Numeric {
            left: collection,
            right: index,
            ..
        }
        | ExpressionKind::Compare {
            left: collection,
            right: index,
            ..
        }
        | ExpressionKind::Boolean {
            left: collection,
            right: index,
            ..
        } => vec![*collection, *index],
        ExpressionKind::Quantifier {
            collection,
            predicate,
            ..
        } => vec![*collection, *predicate],
        ExpressionKind::BooleanLiteral { .. }
        | ExpressionKind::IntegerLiteral { .. }
        | ExpressionKind::RationalLiteral { .. }
        | ExpressionKind::TextLiteral { .. }
        | ExpressionKind::EnumLiteral { .. }
        | ExpressionKind::OptionNone { .. }
        | ExpressionKind::ValueReference { .. }
        | ExpressionKind::LocalReference { .. } => Vec::new(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "node", rename_all = "snake_case")]
pub enum ExpressionKind {
    BooleanLiteral {
        value: bool,
    },
    IntegerLiteral {
        value: i64,
        value_type: IntegerType,
    },
    RationalLiteral {
        numerator: i64,
        denominator: i64,
        value_type: RationalType,
    },
    TextLiteral {
        value: String,
    },
    EnumLiteral {
        enumeration: SymbolName,
        variant: SymbolName,
    },
    OptionNone {
        value_type: ValueType,
    },
    OptionSome {
        value_type: ValueType,
        value: Box<Expression>,
    },
    RecordLiteral {
        record: SymbolName,
        fields: Vec<RecordLiteralField>,
    },
    CollectionLiteral {
        value_type: CollectionType,
        items: Vec<Expression>,
    },
    ValueReference {
        name: SymbolName,
        observation: StateObservation,
    },
    LocalReference {
        name: SymbolName,
    },
    FieldAccess {
        base: Box<Expression>,
        field: SymbolName,
    },
    IsPresent {
        option: Box<Expression>,
    },
    Unwrap {
        option: Box<Expression>,
    },
    Length {
        collection: Box<Expression>,
    },
    Index {
        collection: Box<Expression>,
        index: Box<Expression>,
    },
    Call {
        function: SymbolName,
        arguments: Vec<Expression>,
    },
    Numeric {
        operator: NumericOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    NumericNegate {
        operand: Box<Expression>,
    },
    Compare {
        operator: ComparisonOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    BooleanNot {
        operand: Box<Expression>,
    },
    Boolean {
        operator: BooleanOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Quantifier {
        quantifier: QuantifierKind,
        domain: QuantifierDomain,
        collection: Box<Expression>,
        local: SymbolName,
        local_source: SourceSpan,
        predicate: Box<Expression>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TypedNode {
    index: u32,
    value_type: ValueType,
    source: SourceSpan,
}

impl TypedNode {
    pub const fn index(&self) -> u32 {
        self.index
    }

    pub fn value_type(&self) -> &ValueType {
        &self.value_type
    }

    pub fn source(&self) -> &SourceSpan {
        &self.source
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DischargedObligation {
    kind: DefinednessObligationKind,
    subject: String,
    source: SourceSpan,
    proof_span: SourceSpan,
}

impl DischargedObligation {
    pub const fn kind(&self) -> DefinednessObligationKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn source(&self) -> &SourceSpan {
        &self.source
    }

    pub fn proof_span(&self) -> &SourceSpan {
        &self.proof_span
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TypedExpression {
    expression: Expression,
    value_type: ValueType,
    nodes: Vec<TypedNode>,
    obligations: Vec<DischargedObligation>,
    dependencies: BTreeSet<DependencyIdentity>,
}

impl TypedExpression {
    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn value_type(&self) -> &ValueType {
        &self.value_type
    }

    pub fn nodes(&self) -> &[TypedNode] {
        &self.nodes
    }

    pub fn obligations(&self) -> &[DischargedObligation] {
        &self.obligations
    }

    pub fn dependencies(&self) -> &BTreeSet<DependencyIdentity> {
        &self.dependencies
    }
}

impl DependencySource for TypedExpression {
    fn visit_dependencies(&self, visitor: &mut dyn FnMut(&DependencyIdentity)) {
        for dependency in &self.dependencies {
            visitor(dependency);
        }
    }
}

#[derive(Clone)]
struct LocalBinding {
    name: SymbolName,
    value_type: ValueType,
    source: SourceSpan,
}

#[derive(Clone)]
struct Fact {
    kind: FactKind,
    source: SourceSpan,
    generation: u32,
    authored_order: u32,
}

#[derive(Clone, Eq, PartialEq)]
enum FactKind {
    Present(String),
    NonZero(String),
    Lower(String, ScalarBound, bool),
    Upper(String, ScalarBound, bool),
    IndexBound { index: String, collection: String },
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ScalarBound {
    Integer(i64),
    Rational(i128, i128),
}

#[derive(Clone)]
enum NumericRange {
    Integer(Vec<(i128, i128)>),
    Rational {
        numerator_ranges: Vec<(i128, i128)>,
        denominator_max: i128,
        exact: Option<(i128, i128)>,
    },
}

struct Checked {
    value_type: ValueType,
    range: Option<NumericRange>,
    range_proof: Option<RangeProof>,
    nodes: Vec<TypedNode>,
    obligations: Vec<DischargedObligation>,
    dependencies: BTreeSet<DependencyIdentity>,
}

#[derive(Clone)]
struct RangeProof {
    span: SourceSpan,
    guard_generation: Option<u32>,
    authored_order: u32,
}

#[derive(Clone, Copy)]
struct CheckInputs<'a> {
    execution_point: &'a ExecutionPoint,
    locals: &'a [LocalBinding],
    facts: &'a [Fact],
}

#[derive(Clone, Copy)]
struct QuantifierInputs<'a> {
    domain: QuantifierDomain,
    collection: &'a Expression,
    local: &'a SymbolName,
    local_source: &'a SourceSpan,
    predicate: &'a Expression,
}

fn check_node(
    environment: &DeclarationEnvironment,
    expression: &Expression,
    execution_point: &ExecutionPoint,
    locals: &[LocalBinding],
    facts: &[Fact],
    counter: &mut u32,
) -> Result<Checked, Vec<Diagnostic>> {
    let index = *counter;
    *counter += 1;
    let mut checked = check_kind(
        environment,
        expression,
        execution_point,
        locals,
        facts,
        counter,
    )?;
    checked.nodes.insert(
        0,
        TypedNode {
            index,
            value_type: checked.value_type.clone(),
            source: expression.source.clone(),
        },
    );
    Ok(checked)
}

fn check_kind(
    environment: &DeclarationEnvironment,
    expression: &Expression,
    execution_point: &ExecutionPoint,
    locals: &[LocalBinding],
    facts: &[Fact],
    counter: &mut u32,
) -> Result<Checked, Vec<Diagnostic>> {
    match &expression.kind {
        ExpressionKind::BooleanLiteral { .. } => Ok(leaf(ValueType::Boolean, None)),
        ExpressionKind::IntegerLiteral { value, value_type } => {
            if *value < value_type.minimum || *value > value_type.maximum {
                Err(single(
                    expression,
                    DiagnosticCode::InvalidNumericBounds,
                    "integer literal is outside its named type",
                ))
            } else {
                Ok(with_range_proof(
                    leaf(
                        ValueType::integer(value_type.clone()),
                        Some(NumericRange::Integer(vec![(
                            i128::from(*value),
                            i128::from(*value),
                        )])),
                    ),
                    declaration_range_proof(&expression.source),
                ))
            }
        }
        ExpressionKind::RationalLiteral {
            numerator,
            denominator,
            value_type,
        } => check_rational_literal(expression, *numerator, *denominator, value_type),
        ExpressionKind::TextLiteral { value } => {
            if value.chars().count() > MAX_TEXT_LENGTH as usize {
                Err(single(
                    expression,
                    DiagnosticCode::TextBoundExceeded,
                    "text literal exceeds the public scalar-value limit",
                ))
            } else {
                Ok(leaf(ValueType::Text, None))
            }
        }
        ExpressionKind::EnumLiteral {
            enumeration,
            variant,
        } => check_enum_literal(environment, expression, enumeration, variant),
        ExpressionKind::OptionNone { value_type } => match value_type {
            ValueType::Option { .. } => Ok(leaf(value_type.clone(), None)),
            _ => Err(single(
                expression,
                DiagnosticCode::IllTypedExpression,
                "option-none must name an option type",
            )),
        },
        ExpressionKind::OptionSome { value_type, value } => check_option_some(
            environment,
            expression,
            value_type,
            value,
            execution_point,
            locals,
            facts,
            counter,
        ),
        ExpressionKind::RecordLiteral { record, fields } => check_record_literal(
            environment,
            expression,
            record,
            fields,
            execution_point,
            locals,
            facts,
            counter,
        ),
        ExpressionKind::CollectionLiteral { value_type, items } => check_collection_literal(
            environment,
            expression,
            value_type,
            items,
            execution_point,
            locals,
            facts,
            counter,
        ),
        ExpressionKind::ValueReference { name, observation } => check_value_reference(
            environment,
            expression,
            name,
            *observation,
            execution_point,
            facts,
        ),
        ExpressionKind::LocalReference { name } => {
            check_local_reference(expression, name, locals, facts)
        }
        ExpressionKind::FieldAccess { base, field } => check_field_access(
            environment,
            expression,
            base,
            field,
            execution_point,
            locals,
            facts,
            counter,
        ),
        ExpressionKind::IsPresent { option } => check_is_present(
            environment,
            expression,
            option,
            execution_point,
            locals,
            facts,
            counter,
        ),
        ExpressionKind::Unwrap { option } => check_unwrap(
            environment,
            expression,
            option,
            execution_point,
            locals,
            facts,
            counter,
        ),
        ExpressionKind::Length { collection } => check_length(
            environment,
            expression,
            collection,
            execution_point,
            locals,
            facts,
            counter,
        ),
        ExpressionKind::Index { collection, index } => check_index(
            environment,
            expression,
            collection,
            index,
            execution_point,
            locals,
            facts,
            counter,
        ),
        ExpressionKind::Call {
            function,
            arguments,
        } => check_call(
            environment,
            expression,
            function,
            arguments,
            execution_point,
            locals,
            facts,
            counter,
        ),
        ExpressionKind::Numeric {
            operator,
            left,
            right,
        } => check_numeric(
            environment,
            expression,
            *operator,
            left,
            right,
            CheckInputs {
                execution_point,
                locals,
                facts,
            },
            counter,
        ),
        ExpressionKind::NumericNegate { operand } => check_numeric_negate(
            environment,
            expression,
            operand,
            execution_point,
            locals,
            facts,
            counter,
        ),
        ExpressionKind::Compare {
            operator,
            left,
            right,
        } => check_compare(
            environment,
            expression,
            *operator,
            left,
            right,
            CheckInputs {
                execution_point,
                locals,
                facts,
            },
            counter,
        ),
        ExpressionKind::BooleanNot { operand } => check_boolean_not(
            environment,
            expression,
            operand,
            execution_point,
            locals,
            facts,
            counter,
        ),
        ExpressionKind::Boolean {
            operator,
            left,
            right,
        } => check_boolean(
            environment,
            expression,
            *operator,
            left,
            right,
            CheckInputs {
                execution_point,
                locals,
                facts,
            },
            counter,
        ),
        ExpressionKind::Quantifier {
            domain,
            collection,
            local,
            local_source,
            predicate,
            ..
        } => check_quantifier(
            environment,
            expression,
            QuantifierInputs {
                domain: *domain,
                collection,
                local,
                local_source,
                predicate,
            },
            CheckInputs {
                execution_point,
                locals,
                facts,
            },
            counter,
        ),
    }
}

fn leaf(value_type: ValueType, range: Option<NumericRange>) -> Checked {
    Checked {
        value_type,
        range,
        range_proof: None,
        nodes: Vec::new(),
        obligations: Vec::new(),
        dependencies: BTreeSet::new(),
    }
}

fn with_range_proof(mut checked: Checked, proof: RangeProof) -> Checked {
    checked.range_proof = checked.range.as_ref().map(|_| proof);
    checked
}

fn declaration_range_proof(span: &SourceSpan) -> RangeProof {
    RangeProof {
        span: span.clone(),
        guard_generation: None,
        authored_order: 0,
    }
}

fn single(expression: &Expression, code: DiagnosticCode, message: &str) -> Vec<Diagnostic> {
    vec![Diagnostic::error(code, message, "expression").at_span(&expression.source)]
}

fn undefined(
    expression: &Expression,
    kind: DefinednessObligationKind,
    message: &str,
) -> Vec<Diagnostic> {
    vec![
        Diagnostic::error(DiagnosticCode::PotentiallyUndefined, message, "expression")
            .at_span(&expression.source)
            .with_obligation(kind),
    ]
}

fn check_rational_literal(
    expression: &Expression,
    numerator: i64,
    denominator: i64,
    value_type: &RationalType,
) -> Result<Checked, Vec<Diagnostic>> {
    if denominator <= 0 {
        return Err(single(
            expression,
            DiagnosticCode::InvalidNumericBounds,
            "rational denominator must be positive",
        ));
    }
    let divisor = gcd(numerator.unsigned_abs(), denominator as u64);
    let normalized_numerator = i128::from(numerator) / i128::from(divisor);
    let normalized_denominator = i128::from(denominator) / i128::from(divisor);
    if normalized_numerator < i128::from(value_type.numerator_minimum)
        || normalized_numerator > i128::from(value_type.numerator_maximum)
        || normalized_denominator > i128::from(value_type.maximum_denominator)
    {
        Err(single(
            expression,
            DiagnosticCode::InvalidNumericBounds,
            "normalized rational literal is outside its named type",
        ))
    } else {
        Ok(with_range_proof(
            leaf(
                ValueType::rational(value_type.clone()),
                Some(NumericRange::Rational {
                    numerator_ranges: vec![(normalized_numerator, normalized_numerator)],
                    denominator_max: normalized_denominator,
                    exact: Some((normalized_numerator, normalized_denominator)),
                }),
            ),
            declaration_range_proof(&expression.source),
        ))
    }
}

fn gcd(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left.max(1)
}

fn check_enum_literal(
    environment: &DeclarationEnvironment,
    expression: &Expression,
    enumeration: &SymbolName,
    variant: &SymbolName,
) -> Result<Checked, Vec<Diagnostic>> {
    let Some(declaration) = environment.enum_declaration(enumeration) else {
        return Err(vec![orphaned_type(&expression.source)]);
    };
    if !declaration
        .variants
        .iter()
        .any(|item| &item.name == variant)
    {
        return Err(single(
            expression,
            DiagnosticCode::IllTypedExpression,
            "enum variant does not exist",
        ));
    }
    let mut checked = leaf(
        ValueType::Enum {
            name: enumeration.clone(),
        },
        None,
    );
    checked.dependencies.insert(dependency(
        environment,
        DependencyKind::EnumVariant,
        None,
        [enumeration, variant],
    )?);
    Ok(checked)
}

fn check_option_some(
    environment: &DeclarationEnvironment,
    expression: &Expression,
    value_type: &ValueType,
    value: &Expression,
    execution_point: &ExecutionPoint,
    locals: &[LocalBinding],
    facts: &[Fact],
    counter: &mut u32,
) -> Result<Checked, Vec<Diagnostic>> {
    let ValueType::Option { value: inner } = value_type else {
        return Err(single(
            expression,
            DiagnosticCode::IllTypedExpression,
            "option-some must name an option type",
        ));
    };
    let child = check_node(environment, value, execution_point, locals, facts, counter)?;
    if &child.value_type != inner.as_ref() {
        return Err(single(
            expression,
            DiagnosticCode::IllTypedExpression,
            "option value type differs from its option type",
        ));
    }
    Ok(with_children(value_type.clone(), None, [child]))
}

fn check_record_literal(
    environment: &DeclarationEnvironment,
    expression: &Expression,
    record: &SymbolName,
    fields: &[RecordLiteralField],
    execution_point: &ExecutionPoint,
    locals: &[LocalBinding],
    facts: &[Fact],
    counter: &mut u32,
) -> Result<Checked, Vec<Diagnostic>> {
    let mut names = BTreeSet::new();
    for field in fields {
        if !names.insert(field.name.clone()) {
            return Err(vec![Diagnostic::error(
                DiagnosticCode::DuplicateField,
                "record literal field is duplicated",
                "expression.record.fields",
            )
            .at_span(&field.value.source)]);
        }
    }
    let Some(declaration) = environment.record_declaration(record) else {
        return Err(vec![orphaned_type(&expression.source)]);
    };
    if fields.len() != declaration.fields.len() {
        return Err(single(
            expression,
            DiagnosticCode::IllTypedExpression,
            "record literal must provide every field exactly once",
        ));
    }
    let mut children = Vec::new();
    let mut diagnostics = Vec::new();
    for field in fields {
        let Some(declared) = declaration
            .fields
            .iter()
            .find(|candidate| candidate.name == field.name)
        else {
            diagnostics.extend(single(
                &field.value,
                DiagnosticCode::IllTypedExpression,
                "record literal contains an unknown field",
            ));
            continue;
        };
        match check_node(
            environment,
            &field.value,
            execution_point,
            locals,
            facts,
            counter,
        ) {
            Ok(checked) if checked.value_type == declared.value_type => children.push(checked),
            Ok(_) => diagnostics.extend(single(
                &field.value,
                DiagnosticCode::IllTypedExpression,
                "record field value has the wrong type",
            )),
            Err(mut errors) => diagnostics.append(&mut errors),
        }
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let mut checked = with_child_vec(
        ValueType::Record {
            name: record.clone(),
        },
        None,
        children,
    );
    for field in fields {
        checked.dependencies.insert(dependency(
            environment,
            DependencyKind::Field,
            None,
            [record, &field.name],
        )?);
    }
    Ok(checked)
}

fn check_collection_literal(
    environment: &DeclarationEnvironment,
    expression: &Expression,
    value_type: &CollectionType,
    items: &[Expression],
    execution_point: &ExecutionPoint,
    locals: &[LocalBinding],
    facts: &[Fact],
    counter: &mut u32,
) -> Result<Checked, Vec<Diagnostic>> {
    if items.len() > value_type.maximum_items as usize {
        return Err(single(
            expression,
            DiagnosticCode::CollectionBoundExceeded,
            "collection literal exceeds its maximum item count",
        ));
    }
    let mut children = Vec::new();
    let mut diagnostics = Vec::new();
    for item in items {
        match check_node(environment, item, execution_point, locals, facts, counter) {
            Ok(checked) if checked.value_type == *value_type.element => children.push(checked),
            Ok(_) => diagnostics.extend(single(
                item,
                DiagnosticCode::IllTypedExpression,
                "collection item has the wrong type",
            )),
            Err(mut errors) => diagnostics.append(&mut errors),
        }
    }
    if diagnostics.is_empty() {
        Ok(with_child_vec(
            ValueType::collection(value_type.clone()),
            None,
            children,
        ))
    } else {
        Err(diagnostics)
    }
}

fn check_value_reference(
    environment: &DeclarationEnvironment,
    expression: &Expression,
    name: &SymbolName,
    observation: StateObservation,
    execution_point: &ExecutionPoint,
    facts: &[Fact],
) -> Result<Checked, Vec<Diagnostic>> {
    let Some(declaration) = environment.value_declaration(name) else {
        return Err(single(
            expression,
            DiagnosticCode::OrphanedValueReference,
            "value declaration does not resolve",
        ));
    };
    let allowed = match declaration.kind {
        ValueDeclarationKind::Input => observation == StateObservation::Current,
        ValueDeclarationKind::State => observation_allowed(execution_point, observation),
    };
    if !allowed {
        return Err(single(
            expression,
            DiagnosticCode::InvalidStateObservation,
            "observation is not permitted at this execution point",
        ));
    }
    let kind = match declaration.kind {
        ValueDeclarationKind::Input => DependencyKind::Input,
        ValueDeclarationKind::State => DependencyKind::State,
    };
    let mut checked = leaf(
        declaration.value_type.clone(),
        numeric_range(&declaration.value_type),
    );
    let key = semantic_key(expression);
    checked.range = refine_range(checked.range, &key, facts);
    checked.range_proof = checked.range.as_ref().map(|_| {
        range_fact_proof(facts, &key)
            .unwrap_or_else(|| declaration_range_proof(&declaration.source))
    });
    checked
        .dependencies
        .insert(dependency(environment, kind, Some(observation), [name])?);
    Ok(checked)
}

fn observation_allowed(point: &ExecutionPoint, observation: StateObservation) -> bool {
    match point {
        ExecutionPoint::Initialization { .. } => matches!(
            observation,
            StateObservation::Current | StateObservation::Post
        ),
        ExecutionPoint::Handler { .. } | ExecutionPoint::Post { .. } => true,
        ExecutionPoint::Pre { .. } => matches!(
            observation,
            StateObservation::Current | StateObservation::Pre
        ),
    }
}

fn check_local_reference(
    expression: &Expression,
    name: &SymbolName,
    locals: &[LocalBinding],
    facts: &[Fact],
) -> Result<Checked, Vec<Diagnostic>> {
    let Some(binding) = locals.iter().rev().find(|binding| &binding.name == name) else {
        return Err(single(
            expression,
            DiagnosticCode::InvalidScope,
            "local reference is outside its quantifier scope",
        ));
    };
    let key = semantic_key(expression);
    Ok(with_range_proof(
        leaf(
            binding.value_type.clone(),
            refine_range(numeric_range(&binding.value_type), &key, facts),
        ),
        range_fact_proof(facts, &key).unwrap_or_else(|| declaration_range_proof(&binding.source)),
    ))
}

fn check_field_access(
    environment: &DeclarationEnvironment,
    expression: &Expression,
    base: &Expression,
    field: &SymbolName,
    execution_point: &ExecutionPoint,
    locals: &[LocalBinding],
    facts: &[Fact],
    counter: &mut u32,
) -> Result<Checked, Vec<Diagnostic>> {
    let child = check_node(environment, base, execution_point, locals, facts, counter)?;
    let ValueType::Record { name } = &child.value_type else {
        return Err(single(
            expression,
            DiagnosticCode::IllTypedExpression,
            "field access requires a record",
        ));
    };
    let record_name = name.clone();
    let Some(declaration) = environment.record_declaration(name) else {
        return Err(vec![orphaned_type(&expression.source)]);
    };
    let Some(declared_field) = declaration
        .fields
        .iter()
        .find(|candidate| &candidate.name == field)
    else {
        return Err(single(
            expression,
            DiagnosticCode::IllTypedExpression,
            "record field does not exist",
        ));
    };
    let result_type = declared_field.value_type.clone();
    let mut checked = with_children(
        result_type.clone(),
        refine_range(
            numeric_range(&result_type),
            &semantic_key(expression),
            facts,
        ),
        [child],
    );
    checked.range_proof = checked.range.as_ref().map(|_| {
        range_fact_proof(facts, &semantic_key(expression))
            .unwrap_or_else(|| declaration_range_proof(&declared_field.source))
    });
    checked.dependencies.insert(dependency(
        environment,
        DependencyKind::Field,
        None,
        [&record_name, field],
    )?);
    Ok(checked)
}

fn check_is_present(
    environment: &DeclarationEnvironment,
    expression: &Expression,
    option: &Expression,
    execution_point: &ExecutionPoint,
    locals: &[LocalBinding],
    facts: &[Fact],
    counter: &mut u32,
) -> Result<Checked, Vec<Diagnostic>> {
    let child = check_node(environment, option, execution_point, locals, facts, counter)?;
    if !matches!(child.value_type, ValueType::Option { .. }) {
        return Err(single(
            expression,
            DiagnosticCode::IllTypedExpression,
            "is-present requires an option",
        ));
    }
    Ok(with_children(ValueType::Boolean, None, [child]))
}

fn check_unwrap(
    environment: &DeclarationEnvironment,
    expression: &Expression,
    option: &Expression,
    execution_point: &ExecutionPoint,
    locals: &[LocalBinding],
    facts: &[Fact],
    counter: &mut u32,
) -> Result<Checked, Vec<Diagnostic>> {
    let child = check_node(environment, option, execution_point, locals, facts, counter)?;
    let ValueType::Option { value } = &child.value_type else {
        return Err(single(
            expression,
            DiagnosticCode::IllTypedExpression,
            "unwrap requires an option",
        ));
    };
    let key = semantic_key(option);
    let proof = if matches!(option.kind, ExpressionKind::OptionSome { .. }) {
        Some(option.source.clone())
    } else {
        fact_span(facts, &FactKind::Present(key.clone()))
    };
    let Some(proof_span) = proof else {
        return Err(undefined(
            expression,
            DefinednessObligationKind::OptionPresence,
            "option presence is not established",
        ));
    };
    let result_type = value.as_ref().clone();
    let mut checked = with_children(
        result_type.clone(),
        refine_range(
            numeric_range(&result_type),
            &semantic_key(expression),
            facts,
        ),
        [child],
    );
    checked.range_proof = checked.range.as_ref().map(|_| {
        range_fact_proof(facts, &semantic_key(expression)).unwrap_or_else(|| RangeProof {
            span: proof_span.clone(),
            guard_generation: None,
            authored_order: 0,
        })
    });
    checked.obligations.push(obligation(
        expression,
        DefinednessObligationKind::OptionPresence,
        key,
        proof_span,
    ));
    Ok(checked)
}

fn check_length(
    environment: &DeclarationEnvironment,
    expression: &Expression,
    collection: &Expression,
    execution_point: &ExecutionPoint,
    locals: &[LocalBinding],
    facts: &[Fact],
    counter: &mut u32,
) -> Result<Checked, Vec<Diagnostic>> {
    let child = check_node(
        environment,
        collection,
        execution_point,
        locals,
        facts,
        counter,
    )?;
    let ValueType::Collection { value } = &child.value_type else {
        return Err(single(
            expression,
            DiagnosticCode::IllTypedExpression,
            "length requires a collection",
        ));
    };
    let value_type = ValueType::integer(value.index_type());
    let range = refine_range(numeric_range(&value_type), &semantic_key(expression), facts);
    let mut checked = with_children(value_type, range, [child]);
    checked.range_proof = checked.range.as_ref().map(|_| {
        range_fact_proof(facts, &semantic_key(expression))
            .unwrap_or_else(|| declaration_range_proof(&collection.source))
    });
    Ok(checked)
}

fn check_index(
    environment: &DeclarationEnvironment,
    expression: &Expression,
    collection: &Expression,
    index: &Expression,
    execution_point: &ExecutionPoint,
    locals: &[LocalBinding],
    facts: &[Fact],
    counter: &mut u32,
) -> Result<Checked, Vec<Diagnostic>> {
    let collection_checked = check_node(
        environment,
        collection,
        execution_point,
        locals,
        facts,
        counter,
    );
    let index_checked = check_node(environment, index, execution_point, locals, facts, counter);
    let (collection_checked, index_checked) = pair(collection_checked, index_checked)?;
    let ValueType::Collection { value } = &collection_checked.value_type else {
        return Err(single(
            expression,
            DiagnosticCode::IllTypedExpression,
            "index requires a collection",
        ));
    };
    if index_checked.value_type != ValueType::integer(value.index_type()) {
        return Err(single(
            index,
            DiagnosticCode::IllTypedExpression,
            "index has the wrong derived type",
        ));
    }
    let index_key = semantic_key(index);
    let collection_key = semantic_key(collection);
    let static_bound = match (&collection.kind, &index_checked.range) {
        (ExpressionKind::CollectionLiteral { items, .. }, Some(NumericRange::Integer(ranges))) => {
            ranges
                .iter()
                .all(|(min, max)| *min >= 0 && *max < items.len() as i128)
        }
        _ => false,
    };
    let proof = if static_bound {
        Some(collection.source.clone())
    } else {
        fact_span(
            facts,
            &FactKind::IndexBound {
                index: index_key.clone(),
                collection: collection_key,
            },
        )
    };
    let Some(proof_span) = proof else {
        return Err(undefined(
            expression,
            DefinednessObligationKind::IndexInBounds,
            "collection index bounds are not established",
        ));
    };
    let result_type = value.element.as_ref().clone();
    let mut checked = with_children(
        result_type.clone(),
        refine_range(
            numeric_range(&result_type),
            &semantic_key(expression),
            facts,
        ),
        [collection_checked, index_checked],
    );
    checked.range_proof = checked.range.as_ref().map(|_| {
        range_fact_proof(facts, &semantic_key(expression)).unwrap_or_else(|| RangeProof {
            span: proof_span.clone(),
            guard_generation: None,
            authored_order: 0,
        })
    });
    checked.obligations.push(obligation(
        expression,
        DefinednessObligationKind::IndexInBounds,
        index_key,
        proof_span,
    ));
    Ok(checked)
}

fn check_call(
    environment: &DeclarationEnvironment,
    expression: &Expression,
    function: &SymbolName,
    arguments: &[Expression],
    execution_point: &ExecutionPoint,
    locals: &[LocalBinding],
    facts: &[Fact],
    counter: &mut u32,
) -> Result<Checked, Vec<Diagnostic>> {
    let Some(declaration) = environment.function_declaration(function) else {
        return Err(single(
            expression,
            DiagnosticCode::OrphanedFunctionReference,
            "pure function does not resolve",
        ));
    };
    if arguments.len() != declaration.parameters.len() {
        return Err(single(
            expression,
            DiagnosticCode::ArityMismatch,
            "pure function argument count differs",
        ));
    }
    let mut children = Vec::new();
    let mut diagnostics = Vec::new();
    for (argument, parameter) in arguments.iter().zip(&declaration.parameters) {
        match check_node(
            environment,
            argument,
            execution_point,
            locals,
            facts,
            counter,
        ) {
            Ok(checked) if checked.value_type == parameter.value_type => children.push(checked),
            Ok(_) => diagnostics.extend(single(
                argument,
                DiagnosticCode::IllTypedExpression,
                "pure function argument has the wrong type",
            )),
            Err(mut errors) => diagnostics.append(&mut errors),
        }
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let result_type = declaration.result_type.clone();
    let mut checked = with_child_vec(
        result_type.clone(),
        refine_range(
            numeric_range(&result_type),
            &semantic_key(expression),
            facts,
        ),
        children,
    );
    checked.range_proof = checked.range.as_ref().map(|_| {
        range_fact_proof(facts, &semantic_key(expression))
            .unwrap_or_else(|| declaration_range_proof(&declaration.source))
    });
    checked.dependencies.insert(dependency(
        environment,
        DependencyKind::PureFunction,
        None,
        [function],
    )?);
    Ok(checked)
}

fn check_numeric(
    environment: &DeclarationEnvironment,
    expression: &Expression,
    operator: NumericOperator,
    left: &Expression,
    right: &Expression,
    inputs: CheckInputs<'_>,
    counter: &mut u32,
) -> Result<Checked, Vec<Diagnostic>> {
    let left_checked = check_node(
        environment,
        left,
        inputs.execution_point,
        inputs.locals,
        inputs.facts,
        counter,
    );
    let right_checked = check_node(
        environment,
        right,
        inputs.execution_point,
        inputs.locals,
        inputs.facts,
        counter,
    );
    let (left_checked, right_checked) = pair(left_checked, right_checked)?;
    if left_checked.value_type != right_checked.value_type {
        return Err(single(
            expression,
            DiagnosticCode::IllTypedExpression,
            "numeric operand types differ",
        ));
    }
    match left_checked.value_type.clone() {
        ValueType::Integer { value } => {
            check_integer_operator(expression, operator, &value, left_checked, right_checked)
        }
        ValueType::Rational { value } if operator != NumericOperator::Remainder => {
            check_rational_operator(expression, operator, &value, left_checked, right_checked)
        }
        _ => Err(single(
            expression,
            DiagnosticCode::IllTypedExpression,
            "operator requires compatible numeric operands",
        )),
    }
}

fn check_integer_operator(
    expression: &Expression,
    operator: NumericOperator,
    value_type: &IntegerType,
    left: Checked,
    right: Checked,
) -> Result<Checked, Vec<Diagnostic>> {
    let Some(NumericRange::Integer(left_ranges)) = &left.range else {
        unreachable!()
    };
    let Some(NumericRange::Integer(right_ranges)) = &right.range else {
        unreachable!()
    };
    let right_proof = right
        .range_proof
        .as_ref()
        .map(|proof| proof.span.clone())
        .unwrap_or_else(|| expression.source.clone());
    let range_proof = [left.range_proof.as_ref(), right.range_proof.as_ref()]
        .into_iter()
        .flatten()
        .cloned()
        .max_by(compare_range_proof)
        .map(|proof| proof.span)
        .unwrap_or_else(|| expression.source.clone());
    let mut obligations = Vec::new();
    if matches!(
        operator,
        NumericOperator::Divide | NumericOperator::Remainder
    ) {
        if contains_zero(right_ranges) {
            return Err(undefined(
                expression,
                DefinednessObligationKind::NonZeroDivisor,
                "divisor may be zero",
            ));
        }
        obligations.push(obligation(
            expression,
            DefinednessObligationKind::NonZeroDivisor,
            "divisor".to_owned(),
            right_proof,
        ));
    }
    let Some(ranges) = integer_ranges(operator, left_ranges, right_ranges) else {
        return Err(undefined(
            expression,
            DefinednessObligationKind::CheckedRange,
            "integer interval computation exceeded checked intermediates",
        ));
    };
    let within = ranges.iter().all(|(min, max)| {
        *min >= i128::from(value_type.minimum) && *max <= i128::from(value_type.maximum)
    });
    let range_required = value_type.overflow == OverflowPolicy::Reject;
    if range_required && !within {
        return Err(undefined(
            expression,
            DefinednessObligationKind::CheckedRange,
            "integer result may exceed its named bounds",
        ));
    }
    if range_required {
        obligations.push(obligation(
            expression,
            DefinednessObligationKind::CheckedRange,
            semantic_key(expression),
            range_proof,
        ));
    }
    let range = if within {
        ranges
    } else {
        vec![(
            i128::from(value_type.minimum),
            i128::from(value_type.maximum),
        )]
    };
    let mut checked = with_children(
        ValueType::integer(value_type.clone()),
        Some(NumericRange::Integer(range)),
        [left, right],
    );
    checked.obligations.extend(obligations);
    Ok(checked)
}

fn integer_ranges(
    operator: NumericOperator,
    left: &[(i128, i128)],
    right: &[(i128, i128)],
) -> Option<Vec<(i128, i128)>> {
    let mut output = Vec::new();
    for &(left_min, left_max) in left {
        for &(right_min, right_max) in right {
            let range = match operator {
                NumericOperator::Add => (
                    left_min.checked_add(right_min)?,
                    left_max.checked_add(right_max)?,
                ),
                NumericOperator::Subtract => (
                    left_min.checked_sub(right_max)?,
                    left_max.checked_sub(right_min)?,
                ),
                NumericOperator::Multiply => extrema([
                    left_min.checked_mul(right_min)?,
                    left_min.checked_mul(right_max)?,
                    left_max.checked_mul(right_min)?,
                    left_max.checked_mul(right_max)?,
                ]),
                NumericOperator::Divide => extrema([
                    left_min.checked_div(right_min)?,
                    left_min.checked_div(right_max)?,
                    left_max.checked_div(right_min)?,
                    left_max.checked_div(right_max)?,
                ]),
                NumericOperator::Remainder => {
                    let magnitude = right_min.abs().max(right_max.abs()).saturating_sub(1);
                    (-magnitude, magnitude)
                }
            };
            output.push(range);
        }
    }
    Some(output)
}

fn extrema(values: [i128; 4]) -> (i128, i128) {
    (*values.iter().min().unwrap(), *values.iter().max().unwrap())
}

fn check_rational_operator(
    expression: &Expression,
    operator: NumericOperator,
    value_type: &RationalType,
    left: Checked,
    right: Checked,
) -> Result<Checked, Vec<Diagnostic>> {
    let Some(NumericRange::Rational {
        numerator_ranges: left_ranges,
        denominator_max: left_den,
        exact: left_exact,
    }) = left.range.clone()
    else {
        unreachable!()
    };
    let right_proof = right
        .range_proof
        .as_ref()
        .map(|proof| proof.span.clone())
        .unwrap_or_else(|| expression.source.clone());
    let range_proof = [left.range_proof.as_ref(), right.range_proof.as_ref()]
        .into_iter()
        .flatten()
        .cloned()
        .max_by(compare_range_proof)
        .map(|proof| proof.span)
        .unwrap_or_else(|| expression.source.clone());
    let Some(NumericRange::Rational {
        numerator_ranges: right_ranges,
        denominator_max: right_den,
        exact: right_exact,
    }) = right.range.clone()
    else {
        unreachable!()
    };
    let mut obligations = Vec::new();
    if operator == NumericOperator::Divide && contains_zero(&right_ranges) {
        return Err(undefined(
            expression,
            DefinednessObligationKind::NonZeroDivisor,
            "rational divisor may be zero",
        ));
    }
    if operator == NumericOperator::Divide {
        obligations.push(obligation(
            expression,
            DefinednessObligationKind::NonZeroDivisor,
            "divisor".to_owned(),
            right_proof,
        ));
    }
    let computed = match (left_exact, right_exact) {
        (Some(left), Some(right)) => {
            rational_exact_result(operator, left, right).map(|(numerator, denominator)| {
                (
                    vec![(numerator, numerator)],
                    denominator,
                    Some((numerator, denominator)),
                )
            })
        }
        _ => rational_range_results(operator, &left_ranges, left_den, &right_ranges, right_den)
            .map(|(ranges, denominator)| (ranges, denominator, None)),
    };
    let Some((numerator_ranges, denominator_max, exact)) = computed else {
        return Err(undefined(
            expression,
            DefinednessObligationKind::CheckedRange,
            "rational interval computation exceeded checked intermediates",
        ));
    };
    if !numerator_ranges.iter().all(|(minimum, maximum)| {
        *minimum >= i128::from(value_type.numerator_minimum)
            && *maximum <= i128::from(value_type.numerator_maximum)
    }) || denominator_max > i128::from(value_type.maximum_denominator)
    {
        return Err(undefined(
            expression,
            DefinednessObligationKind::CheckedRange,
            "rational result may exceed its named bounds",
        ));
    }
    obligations.push(obligation(
        expression,
        DefinednessObligationKind::CheckedRange,
        semantic_key(expression),
        range_proof,
    ));
    let mut checked = with_children(
        ValueType::rational(value_type.clone()),
        Some(NumericRange::Rational {
            numerator_ranges,
            denominator_max,
            exact,
        }),
        [left, right],
    );
    checked.obligations.extend(obligations);
    Ok(checked)
}

fn rational_range_results(
    operator: NumericOperator,
    left_ranges: &[(i128, i128)],
    left_denominator: i128,
    right_ranges: &[(i128, i128)],
    right_denominator: i128,
) -> Option<(Vec<(i128, i128)>, i128)> {
    let mut output = Vec::new();
    let mut denominator_maximum = 1_i128;
    for &left in left_ranges {
        for &right in right_ranges {
            let (minimum, maximum, denominator) = rational_worst_case(
                operator,
                (left.0, left.1, left_denominator),
                (right.0, right.1, right_denominator),
            )?;
            output.push((minimum, maximum));
            denominator_maximum = denominator_maximum.max(denominator);
        }
    }
    Some((output, denominator_maximum))
}

fn rational_exact_result(
    operator: NumericOperator,
    left: (i128, i128),
    right: (i128, i128),
) -> Option<(i128, i128)> {
    let (left_numerator, left_denominator) = left;
    let (right_numerator, right_denominator) = right;
    let (numerator, denominator) = match operator {
        NumericOperator::Add => (
            left_numerator
                .checked_mul(right_denominator)?
                .checked_add(right_numerator.checked_mul(left_denominator)?)?,
            left_denominator.checked_mul(right_denominator)?,
        ),
        NumericOperator::Subtract => (
            left_numerator
                .checked_mul(right_denominator)?
                .checked_sub(right_numerator.checked_mul(left_denominator)?)?,
            left_denominator.checked_mul(right_denominator)?,
        ),
        NumericOperator::Multiply => (
            left_numerator.checked_mul(right_numerator)?,
            left_denominator.checked_mul(right_denominator)?,
        ),
        NumericOperator::Divide => (
            left_numerator.checked_mul(right_denominator)?,
            left_denominator.checked_mul(right_numerator)?,
        ),
        NumericOperator::Remainder => return None,
    };
    normalize_rational(numerator, denominator)
}

fn normalize_rational(numerator: i128, denominator: i128) -> Option<(i128, i128)> {
    if denominator == 0 {
        return None;
    }
    let (numerator, denominator) = if denominator < 0 {
        (numerator.checked_neg()?, denominator.checked_neg()?)
    } else {
        (numerator, denominator)
    };
    let divisor = gcd_u128(numerator.unsigned_abs(), denominator as u128);
    Some((
        numerator / i128::try_from(divisor).ok()?,
        denominator / i128::try_from(divisor).ok()?,
    ))
}

fn gcd_u128(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left.max(1)
}

fn rational_worst_case(
    operator: NumericOperator,
    left: (i128, i128, i128),
    right: (i128, i128, i128),
) -> Option<(i128, i128, i128)> {
    let (left_min, left_max, left_denominator) = left;
    let (right_min, right_max, right_denominator) = right;
    let left_scaled = scale_by_positive_denominator(left_min, left_max, right_denominator)?;
    let right_scaled = scale_by_positive_denominator(right_min, right_max, left_denominator)?;
    match operator {
        NumericOperator::Add => Some((
            left_scaled.0.checked_add(right_scaled.0)?,
            left_scaled.1.checked_add(right_scaled.1)?,
            left_denominator.checked_mul(right_denominator)?,
        )),
        NumericOperator::Subtract => Some((
            left_scaled.0.checked_sub(right_scaled.1)?,
            left_scaled.1.checked_sub(right_scaled.0)?,
            left_denominator.checked_mul(right_denominator)?,
        )),
        NumericOperator::Multiply => {
            let (minimum, maximum) = extrema([
                left_min.checked_mul(right_min)?,
                left_min.checked_mul(right_max)?,
                left_max.checked_mul(right_min)?,
                left_max.checked_mul(right_max)?,
            ]);
            Some((
                minimum,
                maximum,
                left_denominator.checked_mul(right_denominator)?,
            ))
        }
        NumericOperator::Divide => {
            let denominator = left_denominator
                .checked_mul(right_min.checked_abs()?.max(right_max.checked_abs()?))?;
            if right_max < 0 {
                Some((
                    left_scaled.1.checked_neg()?,
                    left_scaled.0.checked_neg()?,
                    denominator,
                ))
            } else {
                Some((left_scaled.0, left_scaled.1, denominator))
            }
        }
        NumericOperator::Remainder => None,
    }
}

fn scale_by_positive_denominator(
    minimum: i128,
    maximum: i128,
    denominator_maximum: i128,
) -> Option<(i128, i128)> {
    Some(extrema([
        minimum,
        maximum,
        minimum.checked_mul(denominator_maximum)?,
        maximum.checked_mul(denominator_maximum)?,
    ]))
}

fn check_numeric_negate(
    environment: &DeclarationEnvironment,
    expression: &Expression,
    operand: &Expression,
    execution_point: &ExecutionPoint,
    locals: &[LocalBinding],
    facts: &[Fact],
    counter: &mut u32,
) -> Result<Checked, Vec<Diagnostic>> {
    let child = check_node(
        environment,
        operand,
        execution_point,
        locals,
        facts,
        counter,
    )?;
    let range_proof = child
        .range_proof
        .as_ref()
        .map(|proof| proof.span.clone())
        .unwrap_or_else(|| expression.source.clone());
    match (&child.value_type, &child.range) {
        (ValueType::Integer { value }, Some(NumericRange::Integer(ranges)))
            if value.domain == IntegerDomain::Signed =>
        {
            let overflow = value.overflow;
            let Some(mut result) = ranges
                .iter()
                .map(|(min, max)| Some((max.checked_neg()?, min.checked_neg()?)))
                .collect::<Option<Vec<_>>>()
            else {
                return Err(undefined(
                    expression,
                    DefinednessObligationKind::CheckedRange,
                    "negation exceeded checked intermediates",
                ));
            };
            let within = result.iter().all(|(min, max)| {
                *min >= i128::from(value.minimum) && *max <= i128::from(value.maximum)
            });
            if overflow == OverflowPolicy::Reject && !within {
                return Err(undefined(
                    expression,
                    DefinednessObligationKind::CheckedRange,
                    "negation may exceed its named bounds",
                ));
            }
            if overflow == OverflowPolicy::Saturate && !within {
                let minimum = i128::from(value.minimum);
                let maximum = i128::from(value.maximum);
                result = result
                    .into_iter()
                    .map(|(min, max)| (min.clamp(minimum, maximum), max.clamp(minimum, maximum)))
                    .collect();
            }
            let ty = child.value_type.clone();
            let mut checked = with_children(ty, Some(NumericRange::Integer(result)), [child]);
            if overflow == OverflowPolicy::Reject {
                checked.obligations.push(obligation(
                    expression,
                    DefinednessObligationKind::CheckedRange,
                    semantic_key(expression),
                    range_proof,
                ));
            }
            Ok(checked)
        }
        (
            ValueType::Rational { value },
            Some(NumericRange::Rational {
                numerator_ranges,
                denominator_max,
                exact,
            }),
        ) => {
            let Some(negated_ranges) = numerator_ranges
                .iter()
                .map(|(minimum, maximum)| Some((maximum.checked_neg()?, minimum.checked_neg()?)))
                .collect::<Option<Vec<_>>>()
            else {
                return Err(undefined(
                    expression,
                    DefinednessObligationKind::CheckedRange,
                    "rational negation exceeded checked intermediates",
                ));
            };
            if !negated_ranges.iter().all(|(minimum, maximum)| {
                *minimum >= i128::from(value.numerator_minimum)
                    && *maximum <= i128::from(value.numerator_maximum)
            }) {
                return Err(undefined(
                    expression,
                    DefinednessObligationKind::CheckedRange,
                    "rational negation may exceed its named bounds",
                ));
            }
            let range = NumericRange::Rational {
                numerator_ranges: negated_ranges,
                denominator_max: *denominator_max,
                exact: exact.map(|(numerator, denominator)| (-numerator, denominator)),
            };
            let ty = ValueType::rational(value.clone());
            let mut checked = with_children(ty, Some(range), [child]);
            checked.obligations.push(obligation(
                expression,
                DefinednessObligationKind::CheckedRange,
                semantic_key(expression),
                range_proof,
            ));
            Ok(checked)
        }
        _ => Err(single(
            expression,
            DiagnosticCode::IllTypedExpression,
            "numeric negate requires a signed integer or rational",
        )),
    }
}

fn check_compare(
    environment: &DeclarationEnvironment,
    expression: &Expression,
    operator: ComparisonOperator,
    left: &Expression,
    right: &Expression,
    inputs: CheckInputs<'_>,
    counter: &mut u32,
) -> Result<Checked, Vec<Diagnostic>> {
    let checked = pair(
        check_node(
            environment,
            left,
            inputs.execution_point,
            inputs.locals,
            inputs.facts,
            counter,
        ),
        check_node(
            environment,
            right,
            inputs.execution_point,
            inputs.locals,
            inputs.facts,
            counter,
        ),
    )?;
    if checked.0.value_type != checked.1.value_type {
        return Err(single(
            expression,
            DiagnosticCode::IllTypedExpression,
            "comparison operand types differ",
        ));
    }
    let ordered = matches!(
        checked.0.value_type,
        ValueType::Integer { .. } | ValueType::Rational { .. } | ValueType::Text
    );
    if !matches!(
        operator,
        ComparisonOperator::Equal | ComparisonOperator::NotEqual
    ) && !ordered
    {
        return Err(single(
            expression,
            DiagnosticCode::IllTypedExpression,
            "ordering requires an ordered scalar",
        ));
    }
    Ok(with_children(
        ValueType::Boolean,
        None,
        [checked.0, checked.1],
    ))
}

fn check_boolean_not(
    environment: &DeclarationEnvironment,
    expression: &Expression,
    operand: &Expression,
    execution_point: &ExecutionPoint,
    locals: &[LocalBinding],
    facts: &[Fact],
    counter: &mut u32,
) -> Result<Checked, Vec<Diagnostic>> {
    let child = check_node(
        environment,
        operand,
        execution_point,
        locals,
        facts,
        counter,
    )?;
    if child.value_type != ValueType::Boolean {
        Err(single(
            expression,
            DiagnosticCode::IllTypedExpression,
            "Boolean negate requires Boolean",
        ))
    } else {
        Ok(with_children(ValueType::Boolean, None, [child]))
    }
}

fn check_boolean(
    environment: &DeclarationEnvironment,
    expression: &Expression,
    operator: BooleanOperator,
    left: &Expression,
    right: &Expression,
    inputs: CheckInputs<'_>,
    counter: &mut u32,
) -> Result<Checked, Vec<Diagnostic>> {
    let left_checked = check_node(
        environment,
        left,
        inputs.execution_point,
        inputs.locals,
        inputs.facts,
        counter,
    );
    let right_facts = match operator {
        BooleanOperator::ShortCircuitAnd | BooleanOperator::Implication => {
            facts_with(inputs.facts, guard_facts(left, true))
        }
        BooleanOperator::ShortCircuitOr => facts_with(inputs.facts, guard_facts(left, false)),
        BooleanOperator::TotalAnd | BooleanOperator::TotalOr => inputs.facts.to_vec(),
    };
    let right_checked = check_node(
        environment,
        right,
        inputs.execution_point,
        inputs.locals,
        &right_facts,
        counter,
    );
    let (left_checked, right_checked) = pair(left_checked, right_checked)?;
    if left_checked.value_type != ValueType::Boolean
        || right_checked.value_type != ValueType::Boolean
    {
        return Err(single(
            expression,
            DiagnosticCode::IllTypedExpression,
            "Boolean operator requires Boolean operands",
        ));
    }
    Ok(with_children(
        ValueType::Boolean,
        None,
        [left_checked, right_checked],
    ))
}

fn check_quantifier(
    environment: &DeclarationEnvironment,
    expression: &Expression,
    quantifier: QuantifierInputs<'_>,
    inputs: CheckInputs<'_>,
    counter: &mut u32,
) -> Result<Checked, Vec<Diagnostic>> {
    if inputs
        .locals
        .iter()
        .any(|binding| &binding.name == quantifier.local)
    {
        return Err(vec![Diagnostic::error(
            DiagnosticCode::InvalidScope,
            "quantifier local repeats an enclosing local",
            "expression.quantifier.local",
        )
        .at_span(quantifier.local_source)]);
    }
    let collection_checked = check_node(
        environment,
        quantifier.collection,
        inputs.execution_point,
        inputs.locals,
        inputs.facts,
        counter,
    )?;
    let ValueType::Collection { value } = &collection_checked.value_type else {
        return Err(single(
            expression,
            DiagnosticCode::IllTypedExpression,
            "quantifier domain requires a bounded collection",
        ));
    };
    let local_type = match quantifier.domain {
        QuantifierDomain::Elements => value.element.as_ref().clone(),
        QuantifierDomain::Indices => ValueType::integer(value.index_type()),
    };
    let index_collection = (quantifier.domain == QuantifierDomain::Indices)
        .then(|| semantic_key(quantifier.collection));
    let binding = LocalBinding {
        name: quantifier.local.clone(),
        value_type: local_type,
        source: quantifier.local_source.clone(),
    };
    let mut nested_locals = inputs.locals.to_vec();
    nested_locals.push(binding);
    let mut nested_facts = inputs.facts.to_vec();
    if let Some(collection_key) = index_collection {
        nested_facts.push(Fact {
            kind: FactKind::IndexBound {
                index: format!("local:{}", symbol_key(quantifier.local)),
                collection: collection_key,
            },
            source: quantifier.local_source.clone(),
            generation: next_fact_generation(inputs.facts),
            authored_order: 0,
        });
    }
    let predicate_checked = check_node(
        environment,
        quantifier.predicate,
        inputs.execution_point,
        &nested_locals,
        &nested_facts,
        counter,
    )?;
    if predicate_checked.value_type != ValueType::Boolean {
        return Err(single(
            quantifier.predicate,
            DiagnosticCode::IllTypedExpression,
            "quantifier predicate must be Boolean",
        ));
    }
    Ok(with_children(
        ValueType::Boolean,
        None,
        [collection_checked, predicate_checked],
    ))
}

fn with_children<const N: usize>(
    value_type: ValueType,
    range: Option<NumericRange>,
    children: [Checked; N],
) -> Checked {
    with_child_vec(value_type, range, Vec::from(children))
}

fn with_child_vec(
    value_type: ValueType,
    range: Option<NumericRange>,
    children: Vec<Checked>,
) -> Checked {
    let mut nodes = Vec::new();
    let mut obligations = Vec::new();
    let mut dependencies = BTreeSet::new();
    let range_proof = children
        .iter()
        .filter_map(|child| child.range_proof.as_ref())
        .cloned()
        .max_by(compare_range_proof);
    for mut child in children {
        nodes.append(&mut child.nodes);
        obligations.append(&mut child.obligations);
        dependencies.append(&mut child.dependencies);
    }
    Checked {
        value_type,
        range,
        range_proof,
        nodes,
        obligations,
        dependencies,
    }
}

fn compare_range_proof(left: &RangeProof, right: &RangeProof) -> std::cmp::Ordering {
    match (left.guard_generation, right.guard_generation) {
        (Some(left_generation), Some(right_generation)) => left_generation
            .cmp(&right_generation)
            .then_with(|| right.authored_order.cmp(&left.authored_order)),
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (None, None) => right.authored_order.cmp(&left.authored_order),
    }
}

fn pair(
    left: Result<Checked, Vec<Diagnostic>>,
    right: Result<Checked, Vec<Diagnostic>>,
) -> Result<(Checked, Checked), Vec<Diagnostic>> {
    match (left, right) {
        (Ok(left), Ok(right)) => Ok((left, right)),
        (Err(mut left), Err(mut right)) => {
            left.append(&mut right);
            Err(left)
        }
        (Err(errors), Ok(_)) | (Ok(_), Err(errors)) => Err(errors),
    }
}

fn dependency<const N: usize>(
    environment: &DeclarationEnvironment,
    kind: DependencyKind,
    observation: Option<StateObservation>,
    path: [&SymbolName; N],
) -> Result<DependencyIdentity, Vec<Diagnostic>> {
    let path = path
        .into_iter()
        .map(|name| DependencyName::new(name.as_str()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|diagnostic| vec![diagnostic])?;
    let result = match observation {
        Some(observation) => {
            DependencyIdentity::new_observed(environment.owner.clone(), kind, observation, path)
        }
        None => DependencyIdentity::new(environment.owner.clone(), kind, path),
    };
    result.map_err(|diagnostic| vec![diagnostic])
}

fn numeric_range(value_type: &ValueType) -> Option<NumericRange> {
    match value_type {
        ValueType::Integer { value } => Some(NumericRange::Integer(vec![(
            i128::from(value.minimum),
            i128::from(value.maximum),
        )])),
        ValueType::Rational { value } => Some(NumericRange::Rational {
            numerator_ranges: vec![(
                i128::from(value.numerator_minimum),
                i128::from(value.numerator_maximum),
            )],
            denominator_max: i128::from(value.maximum_denominator),
            exact: None,
        }),
        _ => None,
    }
}

fn refine_range(
    range: Option<NumericRange>,
    subject: &str,
    facts: &[Fact],
) -> Option<NumericRange> {
    match range {
        Some(NumericRange::Integer(ranges)) => Some(NumericRange::Integer(refine_integer_ranges(
            ranges, subject, facts,
        ))),
        Some(NumericRange::Rational {
            numerator_ranges,
            denominator_max,
            exact,
        }) => {
            let ranges = refine_rational_ranges(numerator_ranges, denominator_max, subject, facts);
            let exact = exact.filter(|(numerator, _)| {
                ranges
                    .iter()
                    .any(|(minimum, maximum)| numerator >= minimum && numerator <= maximum)
            });
            Some(NumericRange::Rational {
                numerator_ranges: ranges,
                denominator_max,
                exact,
            })
        }
        None => None,
    }
}

fn refine_integer_ranges(
    mut ranges: Vec<(i128, i128)>,
    subject: &str,
    facts: &[Fact],
) -> Vec<(i128, i128)> {
    for fact in facts {
        match &fact.kind {
            FactKind::NonZero(candidate) if candidate == subject => {
                ranges = ranges
                    .into_iter()
                    .flat_map(|(min, max)| {
                        let mut parts = Vec::new();
                        if min <= -1 {
                            parts.push((min, max.min(-1)));
                        }
                        if max >= 1 {
                            parts.push((min.max(1), max));
                        }
                        parts
                    })
                    .collect();
            }
            FactKind::Lower(candidate, ScalarBound::Integer(value), inclusive)
                if candidate == subject =>
            {
                let bound = i128::from(*value) + i128::from(!*inclusive);
                ranges = ranges
                    .into_iter()
                    .filter_map(|(min, max)| (max >= bound).then_some((min.max(bound), max)))
                    .collect();
            }
            FactKind::Upper(candidate, ScalarBound::Integer(value), inclusive)
                if candidate == subject =>
            {
                let bound = i128::from(*value) - i128::from(!*inclusive);
                ranges = ranges
                    .into_iter()
                    .filter_map(|(min, max)| (min <= bound).then_some((min, max.min(bound))))
                    .collect();
            }
            _ => {}
        }
    }
    ranges
}

fn refine_rational_ranges(
    mut ranges: Vec<(i128, i128)>,
    denominator_maximum: i128,
    subject: &str,
    facts: &[Fact],
) -> Vec<(i128, i128)> {
    for fact in facts {
        match &fact.kind {
            FactKind::NonZero(candidate) if candidate == subject => {
                ranges = ranges
                    .into_iter()
                    .flat_map(|(minimum, maximum)| {
                        let mut parts = Vec::new();
                        if minimum <= -1 {
                            parts.push((minimum, maximum.min(-1)));
                        }
                        if maximum >= 1 {
                            parts.push((minimum.max(1), maximum));
                        }
                        parts
                    })
                    .collect();
            }
            FactKind::Lower(
                candidate,
                ScalarBound::Rational(numerator, denominator),
                inclusive,
            ) if candidate == subject => {
                let bound = rational_numerator_bound(
                    *numerator,
                    *denominator,
                    denominator_maximum,
                    true,
                    *inclusive,
                );
                ranges = ranges
                    .into_iter()
                    .filter_map(|(minimum, maximum)| {
                        (maximum >= bound).then_some((minimum.max(bound), maximum))
                    })
                    .collect();
            }
            FactKind::Upper(
                candidate,
                ScalarBound::Rational(numerator, denominator),
                inclusive,
            ) if candidate == subject => {
                let bound = rational_numerator_bound(
                    *numerator,
                    *denominator,
                    denominator_maximum,
                    false,
                    *inclusive,
                );
                ranges = ranges
                    .into_iter()
                    .filter_map(|(minimum, maximum)| {
                        (minimum <= bound).then_some((minimum, maximum.min(bound)))
                    })
                    .collect();
            }
            _ => {}
        }
    }
    ranges
}

fn rational_numerator_bound(
    numerator: i128,
    denominator: i128,
    denominator_maximum: i128,
    lower: bool,
    inclusive: bool,
) -> i128 {
    [1_i128, denominator_maximum]
        .into_iter()
        .map(|candidate_denominator| {
            let product = numerator * candidate_denominator;
            let floor = product.div_euclid(denominator);
            let ceiling = floor + i128::from(product.rem_euclid(denominator) != 0);
            match (lower, inclusive) {
                (true, true) => ceiling,
                (true, false) => floor + 1,
                (false, true) => floor,
                (false, false) => ceiling - 1,
            }
        })
        .reduce(|left, right| {
            if lower {
                left.min(right)
            } else {
                left.max(right)
            }
        })
        .unwrap_or(0)
}

fn contains_zero(ranges: &[(i128, i128)]) -> bool {
    ranges.iter().any(|(min, max)| *min <= 0 && *max >= 0)
}

fn fact_span(facts: &[Fact], expected: &FactKind) -> Option<SourceSpan> {
    fact_proof(facts, |fact| &fact.kind == expected).map(|proof| proof.span)
}

fn range_fact_proof(facts: &[Fact], subject: &str) -> Option<RangeProof> {
    fact_proof(facts, |fact| match &fact.kind {
        FactKind::NonZero(candidate)
        | FactKind::Lower(candidate, ..)
        | FactKind::Upper(candidate, ..) => candidate == subject,
        FactKind::Present(_) | FactKind::IndexBound { .. } => false,
    })
}

fn fact_proof(facts: &[Fact], predicate: impl Fn(&Fact) -> bool) -> Option<RangeProof> {
    facts
        .iter()
        .filter(|fact| predicate(fact))
        .map(|fact| RangeProof {
            span: fact.source.clone(),
            guard_generation: Some(fact.generation),
            authored_order: fact.authored_order,
        })
        .max_by(compare_range_proof)
}

fn obligation(
    expression: &Expression,
    kind: DefinednessObligationKind,
    subject: String,
    proof_span: SourceSpan,
) -> DischargedObligation {
    DischargedObligation {
        kind,
        subject,
        source: expression.source.clone(),
        proof_span,
    }
}

fn facts_with(existing: &[Fact], mut added: Vec<Fact>) -> Vec<Fact> {
    let generation = next_fact_generation(existing);
    for (order, fact) in added.iter_mut().enumerate() {
        fact.generation = generation;
        fact.authored_order = order as u32;
    }
    let mut facts = existing.to_vec();
    facts.append(&mut added);
    facts
}

fn next_fact_generation(existing: &[Fact]) -> u32 {
    existing
        .iter()
        .map(|fact| fact.generation)
        .max()
        .unwrap_or(0)
        .saturating_add(1)
}

fn guard_facts(expression: &Expression, truth: bool) -> Vec<Fact> {
    let mut facts = Vec::new();
    match &expression.kind {
        ExpressionKind::IsPresent { option } if truth => facts.push(Fact {
            kind: FactKind::Present(semantic_key(option)),
            source: expression.source.clone(),
            generation: 0,
            authored_order: 0,
        }),
        ExpressionKind::BooleanNot { operand } => facts.extend(guard_facts(operand, !truth)),
        ExpressionKind::Compare {
            operator,
            left,
            right,
        } => {
            comparison_facts(
                *operator,
                left,
                right,
                truth,
                &expression.source,
                &mut facts,
            );
        }
        ExpressionKind::Boolean {
            operator,
            left,
            right,
        } if truth
            && matches!(
                operator,
                BooleanOperator::ShortCircuitAnd | BooleanOperator::TotalAnd
            ) =>
        {
            facts.extend(guard_facts(left, true));
            facts.extend(guard_facts(right, true));
        }
        ExpressionKind::Boolean {
            operator,
            left,
            right,
        } if !truth
            && matches!(
                operator,
                BooleanOperator::ShortCircuitOr | BooleanOperator::TotalOr
            ) =>
        {
            facts.extend(guard_facts(left, false));
            facts.extend(guard_facts(right, false));
        }
        _ => {}
    }
    facts
}

fn comparison_facts(
    operator: ComparisonOperator,
    left: &Expression,
    right: &Expression,
    truth: bool,
    source: &SourceSpan,
    output: &mut Vec<Fact>,
) {
    if let Some(value) = numeric_literal(right) {
        let kind = comparison_literal_fact(operator, left, value, truth);
        if let Some(kind) = kind {
            output.push(Fact {
                kind,
                source: source.clone(),
                generation: 0,
                authored_order: 0,
            });
        }
    }
    if let Some(value) = numeric_literal(left) {
        let kind = comparison_literal_fact(reverse_comparison(operator), right, value, truth);
        if let Some(kind) = kind {
            output.push(Fact {
                kind,
                source: source.clone(),
                generation: 0,
                authored_order: 0,
            });
        }
    }
    let index_bound = match (&left.kind, &right.kind, operator, truth) {
        (_, ExpressionKind::Length { collection }, ComparisonOperator::Less, true)
        | (_, ExpressionKind::Length { collection }, ComparisonOperator::GreaterEqual, false) => {
            Some((left, collection.as_ref()))
        }
        (ExpressionKind::Length { collection }, _, ComparisonOperator::Greater, true)
        | (ExpressionKind::Length { collection }, _, ComparisonOperator::LessEqual, false) => {
            Some((right, collection.as_ref()))
        }
        _ => None,
    };
    if let Some((index, collection)) = index_bound {
        output.push(Fact {
            kind: FactKind::IndexBound {
                index: semantic_key(index),
                collection: semantic_key(collection),
            },
            source: source.clone(),
            generation: 0,
            authored_order: 0,
        });
    }
}

fn comparison_literal_fact(
    operator: ComparisonOperator,
    subject: &Expression,
    value: ScalarBound,
    truth: bool,
) -> Option<FactKind> {
    let subject = semantic_key(subject);
    match (operator, truth) {
        (ComparisonOperator::NotEqual, true) | (ComparisonOperator::Equal, false)
            if scalar_is_zero(value) =>
        {
            Some(FactKind::NonZero(subject))
        }
        (ComparisonOperator::Greater, true) | (ComparisonOperator::LessEqual, false) => {
            Some(FactKind::Lower(subject, value, false))
        }
        (ComparisonOperator::GreaterEqual, true) | (ComparisonOperator::Less, false) => {
            Some(FactKind::Lower(subject, value, true))
        }
        (ComparisonOperator::Less, true) | (ComparisonOperator::GreaterEqual, false) => {
            Some(FactKind::Upper(subject, value, false))
        }
        (ComparisonOperator::LessEqual, true) | (ComparisonOperator::Greater, false) => {
            Some(FactKind::Upper(subject, value, true))
        }
        _ => None,
    }
}

const fn reverse_comparison(operator: ComparisonOperator) -> ComparisonOperator {
    match operator {
        ComparisonOperator::Equal => ComparisonOperator::Equal,
        ComparisonOperator::NotEqual => ComparisonOperator::NotEqual,
        ComparisonOperator::Less => ComparisonOperator::Greater,
        ComparisonOperator::LessEqual => ComparisonOperator::GreaterEqual,
        ComparisonOperator::Greater => ComparisonOperator::Less,
        ComparisonOperator::GreaterEqual => ComparisonOperator::LessEqual,
    }
}

const fn scalar_is_zero(value: ScalarBound) -> bool {
    match value {
        ScalarBound::Integer(value) => value == 0,
        ScalarBound::Rational(numerator, _) => numerator == 0,
    }
}

fn numeric_literal(expression: &Expression) -> Option<ScalarBound> {
    match expression.kind {
        ExpressionKind::IntegerLiteral { value, .. } => Some(ScalarBound::Integer(value)),
        ExpressionKind::RationalLiteral {
            numerator,
            denominator,
            ..
        } if denominator > 0 => Some(ScalarBound::Rational(
            i128::from(numerator),
            i128::from(denominator),
        )),
        _ => None,
    }
}

fn semantic_key(expression: &Expression) -> String {
    match &expression.kind {
        ExpressionKind::BooleanLiteral { value } => format!("bool:{value}"),
        ExpressionKind::IntegerLiteral { value, value_type } => {
            format!("integer:{value}:{}", value_type_key_integer(value_type))
        }
        ExpressionKind::RationalLiteral {
            numerator,
            denominator,
            value_type,
        } => {
            if *denominator > 0 {
                let divisor = gcd(numerator.unsigned_abs(), *denominator as u64) as i64;
                format!(
                    "rational:{}/{}:{}",
                    numerator / divisor,
                    denominator / divisor,
                    rational_type_key(value_type)
                )
            } else {
                format!(
                    "rational:{numerator}/{denominator}:{}",
                    rational_type_key(value_type)
                )
            }
        }
        ExpressionKind::TextLiteral { value } => format!("text:{}:{value}", value.len()),
        ExpressionKind::EnumLiteral {
            enumeration,
            variant,
        } => format!("enum:{}:{}", symbol_key(enumeration), symbol_key(variant)),
        ExpressionKind::OptionNone { value_type } => {
            format!("none:{}", value_type_key(value_type))
        }
        ExpressionKind::OptionSome { value_type, value } => format!(
            "some:{}:{}",
            value_type_key(value_type),
            semantic_key(value)
        ),
        ExpressionKind::RecordLiteral { record, fields } => format!(
            "record:{}:[{}]",
            symbol_key(record),
            fields
                .iter()
                .map(|field| format!("{}={}", symbol_key(&field.name), semantic_key(&field.value)))
                .collect::<Vec<_>>()
                .join(",")
        ),
        ExpressionKind::CollectionLiteral { value_type, items } => format!(
            "collection:{}:[{}]",
            collection_type_key(value_type),
            items.iter().map(semantic_key).collect::<Vec<_>>().join(",")
        ),
        ExpressionKind::ValueReference { name, observation } => {
            format!(
                "value:{}:{}",
                symbol_key(name),
                observation_key(*observation)
            )
        }
        ExpressionKind::LocalReference { name } => format!("local:{}", symbol_key(name)),
        ExpressionKind::FieldAccess { base, field } => {
            format!("field:{}:{}", semantic_key(base), symbol_key(field))
        }
        ExpressionKind::IsPresent { option } => format!("present:{}", semantic_key(option)),
        ExpressionKind::Unwrap { option } => format!("unwrap:{}", semantic_key(option)),
        ExpressionKind::Index { collection, index } => {
            format!("index:{}:{}", semantic_key(collection), semantic_key(index))
        }
        ExpressionKind::Length { collection } => format!("length:{}", semantic_key(collection)),
        ExpressionKind::Call {
            function,
            arguments,
        } => format!(
            "call:{}({})",
            symbol_key(function),
            arguments
                .iter()
                .map(semantic_key)
                .collect::<Vec<_>>()
                .join(",")
        ),
        ExpressionKind::Numeric {
            operator,
            left,
            right,
        } => format!(
            "numeric:{}:{}:{}",
            numeric_operator_key(*operator),
            semantic_key(left),
            semantic_key(right)
        ),
        ExpressionKind::NumericNegate { operand } => format!("negate:{}", semantic_key(operand)),
        ExpressionKind::Compare {
            operator,
            left,
            right,
        } => format!(
            "compare:{}:{}:{}",
            comparison_operator_key(*operator),
            semantic_key(left),
            semantic_key(right)
        ),
        ExpressionKind::BooleanNot { operand } => format!("not:{}", semantic_key(operand)),
        ExpressionKind::Boolean {
            operator,
            left,
            right,
        } => format!(
            "boolean:{}:{}:{}",
            boolean_operator_key(*operator),
            semantic_key(left),
            semantic_key(right)
        ),
        ExpressionKind::Quantifier {
            quantifier,
            domain,
            collection,
            local,
            predicate,
            ..
        } => format!(
            "quantifier:{}:{}:{}:{}:{}",
            quantifier_key(*quantifier),
            quantifier_domain_key(*domain),
            semantic_key(collection),
            symbol_key(local),
            semantic_key(predicate)
        ),
    }
}

fn symbol_key(name: &SymbolName) -> String {
    format!("{}:{}", name.as_str().len(), name.as_str())
}

fn value_type_key(value_type: &ValueType) -> String {
    match value_type {
        ValueType::Boolean => "boolean".to_owned(),
        ValueType::Integer { value } => value_type_key_integer(value),
        ValueType::Rational { value } => rational_type_key(value),
        ValueType::Text => "text".to_owned(),
        ValueType::Enum { name } => format!("enum:{}", symbol_key(name)),
        ValueType::Record { name } => format!("record:{}", symbol_key(name)),
        ValueType::Option { value } => format!("option:{}", value_type_key(value)),
        ValueType::Collection { value } => collection_type_key(value),
    }
}

fn value_type_key_integer(value: &IntegerType) -> String {
    format!(
        "integer:{}:{}:{}:{}",
        match value.domain {
            IntegerDomain::Signed => "signed",
            IntegerDomain::Unsigned => "unsigned",
        },
        value.minimum,
        value.maximum,
        match value.overflow {
            OverflowPolicy::Reject => "reject",
            OverflowPolicy::Saturate => "saturate",
        }
    )
}

fn rational_type_key(value: &RationalType) -> String {
    format!(
        "rational:{}:{}:{}",
        value.numerator_minimum, value.numerator_maximum, value.maximum_denominator
    )
}

fn collection_type_key(value: &CollectionType) -> String {
    format!(
        "collection:{}:{}",
        value.maximum_items,
        value_type_key(&value.element)
    )
}

const fn observation_key(value: StateObservation) -> &'static str {
    match value {
        StateObservation::Current => "current",
        StateObservation::Pre => "pre",
        StateObservation::Post => "post",
    }
}

const fn numeric_operator_key(value: NumericOperator) -> &'static str {
    match value {
        NumericOperator::Add => "add",
        NumericOperator::Subtract => "subtract",
        NumericOperator::Multiply => "multiply",
        NumericOperator::Divide => "divide",
        NumericOperator::Remainder => "remainder",
    }
}

const fn comparison_operator_key(value: ComparisonOperator) -> &'static str {
    match value {
        ComparisonOperator::Equal => "equal",
        ComparisonOperator::NotEqual => "not_equal",
        ComparisonOperator::Less => "less",
        ComparisonOperator::LessEqual => "less_equal",
        ComparisonOperator::Greater => "greater",
        ComparisonOperator::GreaterEqual => "greater_equal",
    }
}

const fn boolean_operator_key(value: BooleanOperator) -> &'static str {
    match value {
        BooleanOperator::ShortCircuitAnd => "short_circuit_and",
        BooleanOperator::ShortCircuitOr => "short_circuit_or",
        BooleanOperator::TotalAnd => "total_and",
        BooleanOperator::TotalOr => "total_or",
        BooleanOperator::Implication => "implication",
    }
}

const fn quantifier_key(value: QuantifierKind) -> &'static str {
    match value {
        QuantifierKind::ForAll => "for_all",
        QuantifierKind::Exists => "exists",
    }
}

const fn quantifier_domain_key(value: QuantifierDomain) -> &'static str {
    match value {
        QuantifierDomain::Elements => "elements",
        QuantifierDomain::Indices => "indices",
    }
}

fn preflight(expression: &Expression) -> Result<usize, Vec<Diagnostic>> {
    let mut stack = vec![(expression, 1_usize)];
    let mut nodes = 0_usize;
    let mut maximum_depth = 0_usize;
    while let Some((node, depth)) = stack.pop() {
        nodes += 1;
        maximum_depth = maximum_depth.max(depth);
        if nodes > MAX_EXPRESSION_NODES as usize || depth > MAX_EXPRESSION_DEPTH as usize {
            return Err(single(
                node,
                DiagnosticCode::ExpressionTooLarge,
                "expression exceeds the public node or depth limit",
            ));
        }
        for child in children(node).into_iter().rev() {
            stack.push((child, depth + 1));
        }
    }
    Ok(maximum_depth)
}

fn children(expression: &Expression) -> Vec<&Expression> {
    match &expression.kind {
        ExpressionKind::OptionSome { value, .. } => vec![value],
        ExpressionKind::RecordLiteral { fields, .. } => {
            fields.iter().map(|field| &field.value).collect()
        }
        ExpressionKind::CollectionLiteral { items, .. }
        | ExpressionKind::Call {
            arguments: items, ..
        } => items.iter().collect(),
        ExpressionKind::FieldAccess { base, .. }
        | ExpressionKind::IsPresent { option: base }
        | ExpressionKind::Unwrap { option: base }
        | ExpressionKind::Length { collection: base }
        | ExpressionKind::NumericNegate { operand: base }
        | ExpressionKind::BooleanNot { operand: base } => vec![base],
        ExpressionKind::Index { collection, index }
        | ExpressionKind::Numeric {
            left: collection,
            right: index,
            ..
        }
        | ExpressionKind::Compare {
            left: collection,
            right: index,
            ..
        }
        | ExpressionKind::Boolean {
            left: collection,
            right: index,
            ..
        } => vec![collection, index],
        ExpressionKind::Quantifier {
            collection,
            predicate,
            ..
        } => vec![collection, predicate],
        _ => Vec::new(),
    }
}
