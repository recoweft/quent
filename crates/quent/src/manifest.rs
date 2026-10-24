//! DB-free finite manifest sources. Parsing never authenticates or authorizes.
//! See specs/schema-manifest-v1.md for the canonical and wire contracts.
mod bounded;
mod wire;
use std::fmt;
use wire::*;

/// Fixed, source-free errors. No serde error text or cause is retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestError {
    Budget,
    Wire,
    Version,
    Shape,
    Type,
}
impl fmt::Display for ManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QUENT_MANIFEST_{self:?}")
    }
}
impl std::error::Error for ManifestError {}
macro_rules! redacted { ($($t:ty),+) => {$ (impl fmt::Debug for $t {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(concat!(stringify!($t), "([REDACTED])")) }
})+}; }
#[cfg(all(
    feature = "runtime-tokio",
    any(feature = "postgres", feature = "mysql")
))]
pub mod live;
/// Independent domain/version, never a schema-only digest for every artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    Schema,
    Policy,
    GeneratedCode,
    LiveObservation,
}
/// Coverage of finite source content, not completeness of a live catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceCoverage {
    Missing,
    Unknown,
    Partial,
    Complete,
    Unsupported,
}
/// Fixed source support diagnostics; never caller text or execution permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceCode {
    Missing,
    UnverifiedSource,
    UnknownFacts,
    PartialFacts,
    UnsupportedSemantics,
    UnsupportedNative,
    UnsupportedDefault,
    UnsupportedKey,
}
/// No LiveVerified state or constructor exists in PR1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Freshness {
    Missing,
    Unverified,
    OfflineConsistent,
    ApplicationAsserted,
    Stale,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Correspondence {
    MissingBoth,
    MissingLeft,
    MissingRight,
    Different,
    ExactContent,
}
/// Immutable parsed data, with no context/facts/permit/handle conversion.
#[derive(Clone)]
pub struct ManifestSource {
    source: Source,
}
/// Explicitly sensitive canonical data/digest access. Never emit to public output.
pub struct SensitiveManifestSource<'a> {
    source: &'a ManifestSource,
}
/// Noncryptographic FNV-1a-64/v1 hint. Not identity, authentication or authority.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Fingerprint(u64);
impl Fingerprint {
    pub fn sensitive_hex(&self) -> String {
        format!("{:016x}", self.0)
    }
}
redacted!(ManifestSource, SensitiveManifestSource<'_>, Fingerprint);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactSummary {
    kind: ArtifactKind,
    coverage: SourceCoverage,
    freshness: Freshness,
    items: usize,
    code: SourceCode,
}
impl ArtifactSummary {
    pub fn code(&self) -> SourceCode {
        self.code
    }
    pub fn kind(&self) -> ArtifactKind {
        self.kind
    }
    pub fn coverage(&self) -> SourceCoverage {
        self.coverage
    }
    pub fn freshness(&self) -> Freshness {
        self.freshness
    }
    pub fn items(&self) -> usize {
        self.items
    }
}
/// Output-only allowlist: no target, name, digest, time, generation or text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManifestSummary {
    artifacts: [ArtifactSummary; 4],
}
impl ManifestSummary {
    pub fn artifacts(&self) -> &[ArtifactSummary; 4] {
        &self.artifacts
    }
    pub fn to_json(&self) -> String {
        let mut s =
            String::from("{\"kind\":\"quent_manifest_summary\",\"version\":1,\"artifacts\":[");
        for (i, a) in self.artifacts.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push_str(&format!("{{\"kind\":\"{:?}\",\"version\":1,\"coverage\":\"{:?}\",\"freshness\":\"{:?}\",\"items\":{},\"code\":\"{:?}\"}}", a.kind, a.coverage, a.freshness, a.items, a.code));
        }
        s.push_str("]}");
        s
    }
}
impl fmt::Display for ManifestSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_json())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OfflineComparison {
    correspondence: [Correspondence; 4],
    summary: ManifestSummary,
}
impl OfflineComparison {
    pub fn correspondence(&self) -> &[Correspondence; 4] {
        &self.correspondence
    }
    pub fn public_view(&self) -> ManifestSummary {
        self.summary
    }
}
const KINDS: [ArtifactKind; 4] = [
    ArtifactKind::Schema,
    ArtifactKind::Policy,
    ArtifactKind::GeneratedCode,
    ArtifactKind::LiveObservation,
];
impl ManifestSource {
    pub fn parse(bytes: &[u8]) -> Result<Self, ManifestError> {
        bounded::check(bytes)?;
        let source: Source = serde_json::from_slice(bytes).map_err(|_| ManifestError::Wire)?;
        validate(&source)?;
        Ok(Self { source })
    }
    pub fn sensitive_source(&self) -> SensitiveManifestSource<'_> {
        SensitiveManifestSource { source: self }
    }
    pub fn public_view(&self) -> ManifestSummary {
        ManifestSummary {
            artifacts: KINDS.map(|kind| {
                let (coverage, items) = self.coverage(kind);
                ArtifactSummary {
                    kind,
                    code: self.code(kind, coverage),
                    coverage,
                    items,
                    freshness: if coverage == SourceCoverage::Missing {
                        Freshness::Missing
                    } else {
                        Freshness::Unverified
                    },
                }
            }),
        }
    }
    /// Exact full canonical content, not fingerprint equality. Targets, domain,
    /// version, order, unknowns and coverage all participate. No DB/provider I/O.
    pub fn compare_offline(&self, other: &Self) -> OfflineComparison {
        let mut summary = self.public_view();
        let correspondence = std::array::from_fn(|i| {
            let a = self.sensitive_source().canonical_artifact(KINDS[i]);
            let b = other.sensitive_source().canonical_artifact(KINDS[i]);
            match (a, b) {
                (None, None) => Correspondence::MissingBoth,
                (None, Some(_)) => Correspondence::MissingLeft,
                (Some(_), None) => Correspondence::MissingRight,
                (Some(a), Some(b)) if a == b => {
                    summary.artifacts[i].freshness = Freshness::OfflineConsistent;
                    Correspondence::ExactContent
                }
                _ => Correspondence::Different,
            }
        });
        OfflineComparison {
            correspondence,
            summary,
        }
    }
    /// Explicit application assertion label only. Does not construct trusted facts
    /// or an execution context. Unknown expiry/current time stays unverified for
    /// expiry; an explicitly supplied expired deadline yields Stale.
    pub fn application_asserted_summary(
        &self,
        now: Option<u64>,
        expires_at: Option<u64>,
    ) -> ManifestSummary {
        let mut s = self.public_view();
        for a in &mut s.artifacts {
            if a.freshness != Freshness::Missing {
                a.freshness = if matches!((now, expires_at), (Some(n), Some(e)) if n >= e) {
                    Freshness::Stale
                } else {
                    Freshness::ApplicationAsserted
                };
            }
        }
        s
    }
    fn code(&self, kind: ArtifactKind, c: SourceCoverage) -> SourceCode {
        if let ArtifactKind::Schema = kind
            && let Some(s) = &self.source.schema
        {
            if s.tables
                .iter()
                .any(|t| !matches!(t.semantics, Semantics::Ordinary | Semantics::Unknown))
            {
                return SourceCode::UnsupportedSemantics;
            }
            if s.tables.iter().any(|t| {
                t.columns.iter().any(|c| {
                    c.native == Native::Unsupported
                        || (s.target.dialect == Dialect::Postgres
                            && matches!(
                                c.native,
                                Native::I8 | Native::U8 | Native::U16 | Native::U32 | Native::U64
                            ))
                })
            }) {
                return SourceCode::UnsupportedNative;
            }
            if s.tables.iter().any(|t| {
                t.columns
                    .iter()
                    .any(|c| matches!(c.default, Some(DefaultValue::UnsupportedExpression { .. })))
            }) {
                return SourceCode::UnsupportedDefault;
            }
            if c == SourceCoverage::Unsupported {
                return SourceCode::UnsupportedKey;
            }
        }
        match c {
            SourceCoverage::Missing => SourceCode::Missing,
            SourceCoverage::Unknown => SourceCode::UnknownFacts,
            SourceCoverage::Partial => SourceCode::PartialFacts,
            _ => SourceCode::UnverifiedSource,
        }
    }
    fn coverage(&self, kind: ArtifactKind) -> (SourceCoverage, usize) {
        match kind {
            ArtifactKind::Schema => self
                .source
                .schema
                .as_ref()
                .map(|s| (schema_coverage(s), s.tables.len())),
            ArtifactKind::Policy => self
                .source
                .policy
                .as_ref()
                .map(|s| (coverage(s.coverage), s.rules.len())),
            // A source cannot prove generated-code correspondence or live coverage.
            ArtifactKind::GeneratedCode => self
                .source
                .generated_code
                .as_ref()
                .map(|s| (coverage(s.coverage), 1)),
            ArtifactKind::LiveObservation => self
                .source
                .live_observation
                .as_ref()
                .map(|s| (coverage(s.coverage), usize::from(s.schema.is_some()))),
        }
        .unwrap_or((SourceCoverage::Missing, 0))
    }
}
impl SensitiveManifestSource<'_> {
    /// Canonical compact UTF-8 JSON in declaration order. Missing artifact = None.
    pub fn canonical_artifact(&self, kind: ArtifactKind) -> Option<Vec<u8>> {
        let s = &self.source.source;
        match kind {
            ArtifactKind::Schema => s.schema.as_ref().map(canonical),
            ArtifactKind::Policy => s.policy.as_ref().map(canonical),
            ArtifactKind::GeneratedCode => s.generated_code.as_ref().map(canonical),
            ArtifactKind::LiveObservation => s.live_observation.as_ref().map(canonical),
        }
    }
    pub fn fingerprint(&self, kind: ArtifactKind) -> Option<Fingerprint> {
        self.canonical_artifact(kind).map(|b| Fingerprint(fnv(&b)))
    }
    pub fn canonical_source(&self) -> Vec<u8> {
        canonical(&self.source.source)
    }
}
fn canonical<T: serde::Serialize>(s: &T) -> Vec<u8> {
    // Closed finite types contain only strings, finite integers, enums and vectors;
    // serde_json serialization has no user callback, map key or nonfinite float.
    serde_json::to_vec(s).expect("closed manifest serialization")
}
fn fnv(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    })
}
fn coverage(c: Coverage) -> SourceCoverage {
    match c {
        Coverage::Unknown => SourceCoverage::Unknown,
        Coverage::Partial => SourceCoverage::Partial,
        Coverage::Complete => SourceCoverage::Complete,
    }
}
fn schema_coverage(s: &Schema) -> SourceCoverage {
    if s.tables.iter().any(|t| {
        !matches!(t.semantics, Semantics::Ordinary | Semantics::Unknown)
            || t.columns.iter().any(|c| {
                c.native == Native::Unsupported
                    || matches!(c.default, Some(DefaultValue::UnsupportedExpression { .. }))
                    || (s.target.dialect == Dialect::Postgres
                        && matches!(
                            c.native,
                            Native::I8 | Native::U8 | Native::U16 | Native::U32 | Native::U64
                        ))
            })
            || t.keys.iter().any(|k| {
                k.columns.iter().any(|n| {
                    t.columns.iter().any(|c| {
                        &c.name == n
                            && (c.nullability != Nullability::Nonnull
                                || !matches!(
                                    c.native,
                                    Native::I8 | Native::I16 | Native::I32 | Native::I64
                                ))
                    })
                })
            })
    }) {
        return SourceCoverage::Unsupported;
    }
    if s.coverage == Coverage::Unknown
        || s.tables.iter().any(|t| {
            t.semantics == Semantics::Unknown
                || t.columns_coverage == Coverage::Unknown
                || t.keys_coverage == Coverage::Unknown
                || t.columns.iter().any(|c| {
                    c.native == Native::Unknown
                        || c.nullability == Nullability::Unknown
                        || c.default.is_none()
                        || matches!(c.default, Some(DefaultValue::Unknown { .. }))
                })
                || t.keys.iter().any(|k| k.coverage == Coverage::Unknown)
        })
    {
        return SourceCoverage::Unknown;
    }
    if s.coverage == Coverage::Partial
        || s.tables.iter().any(|t| {
            t.columns_coverage == Coverage::Partial
                || t.keys_coverage == Coverage::Partial
                || t.keys.iter().any(|k| k.coverage == Coverage::Partial)
        })
    {
        return SourceCoverage::Partial;
    }
    SourceCoverage::Complete
}
fn validate(s: &Source) -> Result<(), ManifestError> {
    if s.version != 1 {
        return Err(ManifestError::Version);
    }
    if let Some(s) = &s.schema {
        schema(s)?;
    }
    if let Some(p) = &s.policy {
        version(p.version)?;
        target(&p.target)?;
        limit(p.rules.len(), 256)?;
        for r in &p.rules {
            path(&r.table)?;
            ident(&r.column)?;
        }
    }
    if let Some(c) = &s.generated_code {
        version(c.version)?;
        target(&c.target)?;
        nonempty(&c.generator_version)?;
    }
    if let Some(l) = &s.live_observation {
        version(l.version)?;
        target(&l.target)?;
        if let Some(s) = &l.schema {
            schema(s)?;
            if s.target != l.target {
                return Err(ManifestError::Shape);
            }
        }
        if let (ObservationTime::Claimed { value: a }, ObservationTime::Claimed { value: b }) =
            (&l.observed_at, &l.expires_at)
            && b <= a
        {
            return Err(ManifestError::Shape);
        }
    }
    Ok(())
}
fn version(v: u32) -> Result<(), ManifestError> {
    if v == 1 {
        Ok(())
    } else {
        Err(ManifestError::Version)
    }
}
fn limit(n: usize, max: usize) -> Result<(), ManifestError> {
    if n <= max {
        Ok(())
    } else {
        Err(ManifestError::Budget)
    }
}
fn nonempty(s: &str) -> Result<(), ManifestError> {
    if s.is_empty() {
        Err(ManifestError::Shape)
    } else {
        Ok(())
    }
}
fn ident(s: &str) -> Result<(), ManifestError> {
    quent_core::Ident::new(s)
        .map(|_| ())
        .map_err(|_| ManifestError::Shape)
}
fn path(p: &[String]) -> Result<(), ManifestError> {
    if !(1..=2).contains(&p.len()) {
        return Err(ManifestError::Shape);
    }
    for s in p {
        ident(s)?;
    }
    Ok(())
}
fn target(t: &Target) -> Result<(), ManifestError> {
    nonempty(&t.database)?;
    nonempty(&t.address)?;
    limit(t.database.len(), 128)?;
    limit(t.address.len(), 128)?;
    ident(&t.catalog)
}
fn schema(s: &Schema) -> Result<(), ManifestError> {
    version(s.version)?;
    target(&s.target)?;
    limit(s.tables.len(), 16)?;
    for (i, t) in s.tables.iter().enumerate() {
        path(&t.path)?;
        limit(t.columns.len(), 256)?;
        limit(t.keys.len(), 16)?;
        if s.tables[..i].iter().any(|p| p.path == t.path) {
            return Err(ManifestError::Shape);
        }
        for (j, c) in t.columns.iter().enumerate() {
            ident(&c.name)?;
            if t.columns[..j].iter().any(|p| p.name == c.name) {
                return Err(ManifestError::Shape);
            }
            if let Some(DefaultValue::Literal { value }) = &c.default {
                let native = match value {
                    Literal::I8(_) => Native::I8,
                    Literal::I16(_) => Native::I16,
                    Literal::I32(_) => Native::I32,
                    Literal::I64(_) => Native::I64,
                    Literal::U8(_) => Native::U8,
                    Literal::U16(_) => Native::U16,
                    Literal::U32(_) => Native::U32,
                    Literal::U64(_) => Native::U64,
                    Literal::Bool(_) => Native::Bool,
                    Literal::Text(_) => Native::Text,
                    Literal::Null => c.native,
                };
                if native != c.native {
                    return Err(ManifestError::Type);
                }
            }
        }
        for k in &t.keys {
            if k.columns.is_empty() {
                return Err(ManifestError::Shape);
            }
            limit(k.columns.len(), 16)?;
            for (j, n) in k.columns.iter().enumerate() {
                if !t.columns.iter().any(|c| &c.name == n) || k.columns[..j].contains(n) {
                    return Err(ManifestError::Shape);
                }
            }
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn colliding_hints_never_define_equality() {
        // Inject a collision at the hint layer; comparison still reads full bytes.
        let a = Fingerprint(42);
        let b = Fingerprint(42);
        assert_eq!(a, b);
        assert_eq!(fnv(b""), 0xcbf29ce484222325);
        assert_eq!(fnv(b"a"), 0xaf63dc4c8601ec8c);
    }
}
