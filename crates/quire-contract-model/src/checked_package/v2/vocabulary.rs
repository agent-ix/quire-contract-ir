//! The closed `quire.checked-package/v2` vocabularies, each decoded once at
//! intake into its enum.
//!
//! Every closed wire vocabulary the V2 reader admits is named here, and its
//! wire strings are written exactly once, in the `closed_vocabulary!` row
//! that declares the variant. Past intake nothing matches a wire string:
//! code matches these enums, exhaustively and without a catch-all arm, so a
//! member added here is a compile error at every site that must decide what
//! it means.
//!
//! | Vocabulary | Wire member | Enum |
//! | --- | --- | --- |
//! | node family | `semantic_graph.nodes.node_tag` | [`CheckedNodeTag`] |
//! | semantic form, per family | `semantic_graph.nodes.semantic_form` | [`CheckedNodeKind`] over the 13 `*Form` enums |
//! | selection role | `lock.*_selections.role`, `lock.edition.role` | [`CheckedSelectionRole`] |
//! | capability disposition | `capability_report.disposition` | [`CheckedCapabilityDisposition`] |
//!
//! Vocabularies decoded by serde at the wire edge ([`super::CheckedDiagnosticStage`],
//! [`super::CheckedDiagnosticCode`], [`super::CheckedDiagnosticCause`] and
//! `CheckedOccurrenceRole`) are already enums on the wire type and are not
//! repeated here.

/// Declares one closed wire vocabulary: the enum, its exhaustive wire
/// mapping, and every member in schema order.
macro_rules! closed_vocabulary {
    (
        $(#[$meta:meta])*
        $name:ident { $($variant:ident => $wire:literal,)+ }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub enum $name {
            $(
                #[doc = concat!("`", $wire, "`.")]
                $variant,
            )+
        }

        impl $name {
            /// Every member, in schema order.
            pub const ALL: &'static [Self] = &[$(Self::$variant,)+];

            /// The exact wire string.
            pub const fn as_wire(self) -> &'static str {
                match self {
                    $(Self::$variant => $wire,)+
                }
            }

            /// Decodes a wire string; `None` outside the vocabulary.
            pub fn from_wire(wire: &str) -> Option<Self> {
                Self::ALL
                    .iter()
                    .copied()
                    .find(|candidate| candidate.as_wire() == wire)
            }
        }

        impl serde::Serialize for $name {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(self.as_wire())
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let wire = <std::borrow::Cow<'de, str>>::deserialize(deserializer)?;
                Self::from_wire(&wire).ok_or_else(|| {
                    serde::de::Error::unknown_variant(&wire, &[$($wire,)+])
                })
            }
        }
    };
}

closed_vocabulary! {
    /// The closed V2 semantic node family.
    CheckedNodeTag {
        ScalarType => "scalar_type",
        CompositeType => "composite_type",
        BoundedDomain => "bounded_domain",
        Value => "value",
        Expression => "expression",
        Function => "function",
        Model => "model",
        Relation => "relation",
        State => "state",
        Temporal => "temporal",
        Protocol => "protocol",
        Claim => "claim",
        Correspondence => "correspondence",
    }
}

closed_vocabulary! {
    /// The closed `scalar_type` semantic forms.
    ScalarTypeForm {
        Boolean => "boolean",
        Integer => "integer",
        Rational => "rational",
        Decimal => "decimal",
        Float32 => "float32",
        Float64 => "float64",
        Text => "text",
        Dimension => "dimension",
        Unit => "unit",
        Enum => "enum",
    }
}

closed_vocabulary! {
    /// The closed `composite_type` semantic forms.
    CompositeTypeForm {
        Option => "option",
        Sequence => "sequence",
        Set => "set",
        Bag => "bag",
        OrderedSet => "ordered_set",
        Record => "record",
        Tuple => "tuple",
        Alias => "alias",
        Reference => "reference",
    }
}

closed_vocabulary! {
    /// The closed `bounded_domain` semantic forms.
    BoundedDomainForm {
        IntegerRange => "integer_range",
        RationalRange => "rational_range",
        DecimalRange => "decimal_range",
        FloatRounding => "float_rounding",
        TextBounds => "text_bounds",
        CollectionBounds => "collection_bounds",
        ModelPopulation => "model_population",
    }
}

closed_vocabulary! {
    /// The closed `value` semantic forms.
    ValueForm {
        Literal => "literal",
        EnumValue => "enum_value",
        CollectionValue => "collection_value",
        RecordValue => "record_value",
        TupleValue => "tuple_value",
        OptionValue => "option_value",
    }
}

closed_vocabulary! {
    /// The closed `expression` semantic forms.
    ExpressionForm {
        Reference => "reference",
        Call => "call",
        Unary => "unary",
        Binary => "binary",
        Conditional => "conditional",
        Let => "let",
        Quantify => "quantify",
        Collection => "collection",
        Conversion => "conversion",
        Query => "query",
        PreRead => "pre_read",
        PresenceRead => "presence_read",
        ValueRead => "value_read",
        Deref => "deref",
        Reachability => "reachability",
    }
}

closed_vocabulary! {
    /// The closed `function` semantic forms.
    FunctionForm {
        PureFunction => "pure_function",
        Predicate => "predicate",
        RecursiveFunction => "recursive_function",
    }
}

closed_vocabulary! {
    /// The closed `model` semantic forms.
    ModelForm {
        ModelImport => "model_import",
        ObjectType => "object_type",
        ValueType => "value_type",
        VariantType => "variant_type",
        RecordValueType => "record_value_type",
        EventType => "event_type",
        StateMachine => "state_machine",
        Process => "process",
        PersistenceInterface => "persistence_interface",
        Namespace => "namespace",
        FieldDeclaration => "field_declaration",
        OperationDeclaration => "operation_declaration",
        ClauseMemberDeclaration => "clause_member_declaration",
        SystemsInterface => "systems_interface",
        SystemsPart => "systems_part",
        SystemsPort => "systems_port",
        SystemsConnection => "systems_connection",
        SystemsAllocation => "systems_allocation",
    }
}

closed_vocabulary! {
    /// The closed `relation` semantic forms.
    RelationForm {
        Relationship => "relationship",
        Population => "population",
        Membership => "membership",
        CausalRelation => "causal_relation",
    }
}

closed_vocabulary! {
    /// The closed `state` semantic forms.
    StateForm {
        StateClause => "state_clause",
        Frame => "frame",
        Transition => "transition",
        OperationAnchor => "operation_anchor",
        Snapshot => "snapshot",
    }
}

closed_vocabulary! {
    /// The closed `temporal` semantic forms.
    TemporalForm {
        TemporalClause => "temporal_clause",
        Formula => "formula",
        Clock => "clock",
        Window => "window",
        Activation => "activation",
        Deadline => "deadline",
    }
}

closed_vocabulary! {
    /// The closed `protocol` semantic forms.
    ProtocolForm {
        ProtocolClause => "protocol_clause",
        Role => "role",
        Channel => "channel",
        Queue => "queue",
        Control => "control",
        Obligation => "obligation",
        Compensation => "compensation",
    }
}

closed_vocabulary! {
    /// The closed `claim` semantic forms.
    ClaimForm {
        VerificationClaim => "verification_claim",
        AnalysisClaim => "analysis_claim",
        Hyperproperty => "hyperproperty",
        SynthesisRequest => "synthesis_request",
    }
}

closed_vocabulary! {
    /// The closed `correspondence` semantic forms.
    CorrespondenceForm {
        SourceLocus => "source_locus",
        ModelCorrespondence => "model_correspondence",
        BindingRole => "binding_role",
        ProfileCorrespondence => "profile_correspondence",
    }
}

closed_vocabulary! {
    /// The closed V2 lock selection role.
    CheckedSelectionRole {
        Language => "language",
        Edition => "edition",
        Profile => "profile",
        Dependency => "dependency",
        BindingContract => "binding_contract",
        TemporalProfile => "temporal_profile",
        ProtocolProfile => "protocol_profile",
    }
}

closed_vocabulary! {
    /// The closed V2 capability-report disposition.
    CheckedCapabilityDisposition {
        Available => "available",
        Unimplemented => "unimplemented",
        Unsupported => "unsupported",
    }
}

/// A node's family together with its form, decoded once at intake. The
/// family fixes which form vocabulary applies, so a form of the wrong family
/// is unrepresentable.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CheckedNodeKind {
    /// A `scalar_type` node.
    ScalarType(ScalarTypeForm),
    /// A `composite_type` node.
    CompositeType(CompositeTypeForm),
    /// A `bounded_domain` node.
    BoundedDomain(BoundedDomainForm),
    /// A `value` node.
    Value(ValueForm),
    /// An `expression` node.
    Expression(ExpressionForm),
    /// A `function` node.
    Function(FunctionForm),
    /// A `model` node.
    Model(ModelForm),
    /// A `relation` node.
    Relation(RelationForm),
    /// A `state` node.
    State(StateForm),
    /// A `temporal` node.
    Temporal(TemporalForm),
    /// A `protocol` node.
    Protocol(ProtocolForm),
    /// A `claim` node.
    Claim(ClaimForm),
    /// A `correspondence` node.
    Correspondence(CorrespondenceForm),
}

impl CheckedNodeKind {
    /// Decodes a wire form under an already decoded family; `None` when the
    /// form is not one of that family's forms.
    pub fn decode(tag: CheckedNodeTag, form: &str) -> Option<Self> {
        match tag {
            CheckedNodeTag::ScalarType => ScalarTypeForm::from_wire(form).map(Self::ScalarType),
            CheckedNodeTag::CompositeType => {
                CompositeTypeForm::from_wire(form).map(Self::CompositeType)
            }
            CheckedNodeTag::BoundedDomain => {
                BoundedDomainForm::from_wire(form).map(Self::BoundedDomain)
            }
            CheckedNodeTag::Value => ValueForm::from_wire(form).map(Self::Value),
            CheckedNodeTag::Expression => ExpressionForm::from_wire(form).map(Self::Expression),
            CheckedNodeTag::Function => FunctionForm::from_wire(form).map(Self::Function),
            CheckedNodeTag::Model => ModelForm::from_wire(form).map(Self::Model),
            CheckedNodeTag::Relation => RelationForm::from_wire(form).map(Self::Relation),
            CheckedNodeTag::State => StateForm::from_wire(form).map(Self::State),
            CheckedNodeTag::Temporal => TemporalForm::from_wire(form).map(Self::Temporal),
            CheckedNodeTag::Protocol => ProtocolForm::from_wire(form).map(Self::Protocol),
            CheckedNodeTag::Claim => ClaimForm::from_wire(form).map(Self::Claim),
            CheckedNodeTag::Correspondence => {
                CorrespondenceForm::from_wire(form).map(Self::Correspondence)
            }
        }
    }

    /// The node's family.
    pub const fn tag(self) -> CheckedNodeTag {
        match self {
            Self::ScalarType(_) => CheckedNodeTag::ScalarType,
            Self::CompositeType(_) => CheckedNodeTag::CompositeType,
            Self::BoundedDomain(_) => CheckedNodeTag::BoundedDomain,
            Self::Value(_) => CheckedNodeTag::Value,
            Self::Expression(_) => CheckedNodeTag::Expression,
            Self::Function(_) => CheckedNodeTag::Function,
            Self::Model(_) => CheckedNodeTag::Model,
            Self::Relation(_) => CheckedNodeTag::Relation,
            Self::State(_) => CheckedNodeTag::State,
            Self::Temporal(_) => CheckedNodeTag::Temporal,
            Self::Protocol(_) => CheckedNodeTag::Protocol,
            Self::Claim(_) => CheckedNodeTag::Claim,
            Self::Correspondence(_) => CheckedNodeTag::Correspondence,
        }
    }

    /// The exact wire form.
    pub const fn form_wire(self) -> &'static str {
        match self {
            Self::ScalarType(form) => form.as_wire(),
            Self::CompositeType(form) => form.as_wire(),
            Self::BoundedDomain(form) => form.as_wire(),
            Self::Value(form) => form.as_wire(),
            Self::Expression(form) => form.as_wire(),
            Self::Function(form) => form.as_wire(),
            Self::Model(form) => form.as_wire(),
            Self::Relation(form) => form.as_wire(),
            Self::State(form) => form.as_wire(),
            Self::Temporal(form) => form.as_wire(),
            Self::Protocol(form) => form.as_wire(),
            Self::Claim(form) => form.as_wire(),
            Self::Correspondence(form) => form.as_wire(),
        }
    }

    /// Every kind, family by family in schema order, forms in schema order.
    pub fn all() -> Vec<Self> {
        let mut kinds = Vec::new();
        kinds.extend(ScalarTypeForm::ALL.iter().copied().map(Self::ScalarType));
        kinds.extend(
            CompositeTypeForm::ALL
                .iter()
                .copied()
                .map(Self::CompositeType),
        );
        kinds.extend(
            BoundedDomainForm::ALL
                .iter()
                .copied()
                .map(Self::BoundedDomain),
        );
        kinds.extend(ValueForm::ALL.iter().copied().map(Self::Value));
        kinds.extend(ExpressionForm::ALL.iter().copied().map(Self::Expression));
        kinds.extend(FunctionForm::ALL.iter().copied().map(Self::Function));
        kinds.extend(ModelForm::ALL.iter().copied().map(Self::Model));
        kinds.extend(RelationForm::ALL.iter().copied().map(Self::Relation));
        kinds.extend(StateForm::ALL.iter().copied().map(Self::State));
        kinds.extend(TemporalForm::ALL.iter().copied().map(Self::Temporal));
        kinds.extend(ProtocolForm::ALL.iter().copied().map(Self::Protocol));
        kinds.extend(ClaimForm::ALL.iter().copied().map(Self::Claim));
        kinds.extend(
            CorrespondenceForm::ALL
                .iter()
                .copied()
                .map(Self::Correspondence),
        );
        kinds
    }
}
