use super::*;

/// Pure comparison states, never ownership, freshness, or permission claims.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RegenerationState {
    NoChange,
    GeneratedChanged,
    Conflict,
    Unsupported,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RegenerationAction {
    None,
    ReviewNewArtifact,
    ResolveLocally,
    RegenerateExplicitVersion,
}
/// Output-only source-free diagnostic. No path, code, diff, or hash is retained.
#[derive(Debug, Serialize)]
pub struct Regeneration {
    kind: &'static str,
    version: u8,
    state: RegenerationState,
    action: RegenerationAction,
    changed_domains: usize,
}
impl Regeneration {
    pub fn state(&self) -> RegenerationState {
        self.state
    }
    pub fn action(&self) -> RegenerationAction {
        self.action
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("fixed comparison")
    }
}
/// Compare exact prior production artifact, proposed artifact, and caller-supplied
/// existing bytes. Missing is distinct from empty. No file is opened or written.
/// Callers must use bounded regular-file input if they choose to read a file.
/// Header/marker/hash claims never establish correspondence; all bytes must match.
pub fn compare(
    previous: &CodeArtifact,
    next: &CodeArtifact,
    existing: Option<&[u8]>,
) -> Regeneration {
    let same_version = previous.revision == next.revision;
    let changed_domains = usize::from(!same_version)
        + usize::from(previous.schema != next.schema)
        + usize::from(previous.policy != next.policy)
        + usize::from(previous.config != next.config);
    let (state, action) = if !same_version {
        (
            RegenerationState::Unsupported,
            RegenerationAction::RegenerateExplicitVersion,
        )
    } else if existing.is_some_and(|b| b.len() > 524_288 || std::str::from_utf8(b).is_err()) {
        (
            RegenerationState::Unsupported,
            RegenerationAction::ResolveLocally,
        )
    } else if existing.is_some_and(|b| b != previous.source.as_bytes()) {
        (
            RegenerationState::Conflict,
            RegenerationAction::ResolveLocally,
        )
    } else if existing.is_some() && previous.source == next.source {
        (RegenerationState::NoChange, RegenerationAction::None)
    } else {
        (
            RegenerationState::GeneratedChanged,
            RegenerationAction::ReviewNewArtifact,
        )
    };
    Regeneration {
        kind: "quent_regeneration_summary",
        version: 1,
        state,
        action,
        changed_domains,
    }
}
