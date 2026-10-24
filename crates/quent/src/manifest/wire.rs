use serde::{Deserialize, Serialize};

macro_rules! fixed_enum {
    ($name:ident {$($variant:ident),+ $(,)?}) => {
        #[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
        #[serde(rename_all = "snake_case")]
        pub(super) enum $name {$($variant),+}
    }
}
fixed_enum!(Dialect { Postgres, Mysql });
fixed_enum!(Coverage {
    Unknown,
    Partial,
    Complete
});
fixed_enum!(Nullability {
    Unknown,
    Nullable,
    Nonnull
});
fixed_enum!(Native {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    Bool,
    Text,
    Unknown,
    Unsupported
});
fixed_enum!(Semantics {
    Unknown,
    Ordinary,
    View,
    Triggers,
    Rls,
    Generated,
    Collation,
    Unsupported
});
fixed_enum!(KeyKind { Primary, Unique });
fixed_enum!(RuleKind {
    Tenant,
    SoftDelete,
    Pii,
    RequiredFilter,
    Readonly,
    Generated
});
fixed_enum!(SourceKind {
    QuentManifestSource
});
fixed_enum!(SchemaKind { QuentSchema });
fixed_enum!(PolicyKind { QuentPolicy });
fixed_enum!(CodeKind { QuentGeneratedCode });
fixed_enum!(LiveKind {
    QuentLiveObservation
});

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Target {
    pub database: String,
    pub address: String,
    pub catalog: String,
    pub dialect: Dialect,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Schema {
    pub kind: SchemaKind,
    pub version: u32,
    pub target: Target,
    pub coverage: Coverage,
    pub tables: Vec<Table>,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Table {
    pub path: Vec<String>,
    pub semantics: Semantics,
    pub columns_coverage: Coverage,
    pub keys_coverage: Coverage,
    pub columns: Vec<Column>,
    pub keys: Vec<Key>,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Column {
    pub name: String,
    pub native: Native,
    pub nullability: Nullability,
    // Omitted = not observed. Explicit JSON null is NOT this state.
    #[serde(
        default,
        deserialize_with = "present_default",
        skip_serializing_if = "Option::is_none"
    )]
    pub default: Option<DefaultValue>,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "state", rename_all = "snake_case")]
pub(super) enum DefaultValue {
    None {},
    Unknown {},
    UnsupportedExpression { expression: String },
    Literal { value: Literal },
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    deny_unknown_fields,
    tag = "type",
    content = "value",
    rename_all = "snake_case"
)]
pub(super) enum Literal {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Bool(bool),
    Text(String),
    Null,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Key {
    pub kind: KeyKind,
    pub columns: Vec<String>,
    pub coverage: Coverage,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Policy {
    pub kind: PolicyKind,
    pub version: u32,
    pub target: Target,
    pub coverage: Coverage,
    pub rules: Vec<Rule>,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Rule {
    pub kind: RuleKind,
    pub table: Vec<String>,
    pub column: String,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Code {
    pub kind: CodeKind,
    pub version: u32,
    pub target: Target,
    pub coverage: Coverage,
    pub generator_version: String,
    pub rust_source: String,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Live {
    pub kind: LiveKind,
    pub version: u32,
    pub target: Target,
    pub coverage: Coverage,
    // Required explicit optional wrappers distinguish unknown from invented values.
    pub observed_at: ObservationTime,
    pub expires_at: ObservationTime,
    pub generation: ObservationTime,
    pub schema: Option<Schema>,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "state", rename_all = "snake_case")]
pub(super) enum ObservationTime {
    Unknown {},
    Claimed { value: u64 },
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Source {
    pub kind: SourceKind,
    pub version: u32,
    pub schema: Option<Schema>,
    pub policy: Option<Policy>,
    pub generated_code: Option<Code>,
    pub live_observation: Option<Live>,
}

fn present_default<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<DefaultValue>, D::Error> {
    DefaultValue::deserialize(d).map(Some)
}
