//! Finite syntax observations, never type identity, policy facts or execution.
//! No macro/cfg/module/type resolution, compiler invocation or trusted capture.
//!
//! Output cannot be restored or converted into authority.
//! ```compile_fail
//! let _: quent::source_review::Review = serde_json::from_str("{}").unwrap();
//! ```
//! ```compile_fail
//! let r = quent::source_review::review(b"fn main() {}").unwrap();
//! let _: quent_core::ScopeFacts = r.into();
//! ```
use serde::Serialize;
use std::fmt;
use syn::{
    Expr, Item,
    visit::{self, Visit},
};

/// Pre-parser ceilings count raw bytes INCLUDING strings/comments. This is an
/// intentionally conservative envelope, not a Rust lexer or brace-depth proof.
/// Every token consumes a non-whitespace byte; every recursive delimiter,
/// generic, unary or operator spelling consumes punctuation. Quoted/comment
/// punctuation can only cause earlier refusal, never bypass these ceilings.
pub const MAX_BYTES: usize = 65_536;
pub const MAX_PARSE_BYTES: usize = 4_096;
pub const MAX_TOKEN_BYTES: usize = 512;
pub const MAX_PUNCTUATION: usize = 48;
pub const MAX_GROUP_BYTES: usize = 16;
pub const MAX_ITEMS: usize = 16;
pub const MAX_NODES: usize = 512;
pub const MAX_OPERATIONS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Error {
    Invalid,
    Budget,
}
impl Error {
    pub fn code(self) -> &'static str {
        match self {
            Self::Invalid => "QUENT_REVIEW_INVALID",
            Self::Budget => "QUENT_REVIEW_BUDGET",
        }
    }
    pub fn exit_code(self) -> u8 {
        2
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}
impl std::error::Error for Error {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Diagnostic {
    IdentityUnverified,
    ContextUnverified,
    NoCandidates,
    MacroUnexpanded,
    AttributeUnevaluated,
    NamesUnresolved,
    ControlFlowUnresolved,
    ReceiverUnresolved,
    ArgumentsUnverified,
    UnsupportedSyntax,
}
/// Ten distinct fixed codes maximum, deduplicated without discarding coverage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Precision {
    syntax: &'static str,
    candidate: &'static str,
    identity: &'static str,
    sql: &'static str,
    native: &'static str,
    model: &'static str,
    binding: &'static str,
    policy: &'static str,
    tenant: &'static str,
    freshness: &'static str,
    runtime: &'static str,
    capture: &'static str,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Candidate {
    family: &'static str,
    filter: &'static str,
    missing_where: &'static str,
}
/// Independent output kind/version; existing quent_review v1 is unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Review {
    kind: &'static str,
    version: u8,
    state: &'static str,
    code: &'static str,
    coverage: &'static str,
    risk: &'static str,
    precision: Precision,
    items: usize,
    nodes: usize,
    operations: Vec<Candidate>,
    diagnostics: Vec<Diagnostic>,
}
impl Review {
    pub fn exit_code(&self) -> u8 {
        if self.state == "unsupported" { 4 } else { 3 }
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("fixed source output")
    }
    pub fn to_pretty(&self) -> String {
        serde_json::to_string_pretty(self).expect("fixed source output")
    }
}
impl fmt::Display for Review {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_json())
    }
}

fn preflight(bytes: &[u8]) -> Result<&str, Error> {
    if bytes.len() > MAX_BYTES || bytes.len() > MAX_PARSE_BYTES {
        return Err(Error::Budget);
    }
    let s = std::str::from_utf8(bytes).map_err(|_| Error::Invalid)?;
    let significant = bytes.iter().filter(|b| !b.is_ascii_whitespace()).count();
    let punctuation = bytes.iter().filter(|b| b.is_ascii_punctuation()).count();
    let groups = bytes.iter().filter(|b| b"()[]{}".contains(b)).count();
    if significant > MAX_TOKEN_BYTES || punctuation > MAX_PUNCTUATION || groups > MAX_GROUP_BYTES {
        return Err(Error::Budget);
    }
    Ok(s)
}
/// Bounded parse of ONE explicitly supplied UTF-8 file. The only retained data
/// are fixed codes and structural counts. syn errors/spans/tokens are discarded.
pub fn review(bytes: &[u8]) -> Result<Review, Error> {
    let source = preflight(bytes)?;
    let file = syn::parse_file(source).map_err(|_| Error::Invalid)?;
    let mut scan = Scan {
        result: Review {
            kind: "quent_rust_review",
            version: 1,
            state: "unknown",
            code: "no_candidates",
            coverage: "unknown",
            risk: "unknown",
            precision: Precision {
                syntax: "parsed",
                candidate: "syntax_only",
                identity: "unverified",
                sql: "unverified",
                native: "unverified",
                model: "unverified",
                binding: "unverified",
                policy: "unverified",
                tenant: "unverified",
                freshness: "unverified",
                runtime: "unverified",
                capture: "not_implemented",
            },
            items: 0,
            nodes: 0,
            operations: vec![],
            diagnostics: vec![],
        },
        budget: false,
        chain_receiver: false,
    };
    scan.add(Diagnostic::IdentityUnverified);
    scan.add(Diagnostic::ContextUnverified);
    scan.visit_file(&file);
    if scan.budget {
        return Err(Error::Budget);
    }
    if !scan.result.operations.is_empty() {
        scan.result.state = "partial";
        scan.result.coverage = "partial";
        scan.result.code = "syntax_candidates";
    } else {
        scan.add(Diagnostic::NoCandidates);
        if scan.result.items != 0 || !file.attrs.is_empty() || !file.items.is_empty() {
            scan.result.state = "unsupported";
            scan.result.coverage = "unsupported";
            scan.result.code = "unsupported";
        }
    }
    Ok(scan.result)
}
struct Scan {
    result: Review,
    budget: bool,
    chain_receiver: bool,
}
impl Scan {
    fn add(&mut self, d: Diagnostic) {
        if !self.result.diagnostics.contains(&d) {
            self.result.diagnostics.push(d);
        }
    }
}
// Only the direct spelling is recognized, with optional leading ::. It does not
// establish crate/module/type identity, even without a visible shadow declaration.
fn root(expr: &Expr) -> Option<&'static str> {
    let Expr::Call(call) = expr else { return None };
    let Expr::Path(path) = call.func.as_ref() else {
        return None;
    };
    if path.qself.is_some() || path.path.segments.len() != 3 {
        return None;
    }
    let mut s = path.path.segments.iter();
    let a = s.next()?;
    let b = s.next()?;
    let c = s.next()?;
    if a.ident != "quent_core"
        || b.ident != "Draft"
        || !path.path.segments.iter().all(|s| s.arguments.is_empty())
    {
        return None;
    }
    match c.ident.to_string().as_str() {
        "select" if call.args.len() == 2 => Some("select"),
        "insert" if call.args.len() == 2 => Some("insert"),
        "update" if call.args.len() == 2 => Some("update"),
        "delete" if call.args.len() == 1 => Some("delete"),
        _ => None,
    }
}
fn chain(mut expr: &Expr) -> Option<Candidate> {
    let mut filter = false;
    let mut count = false;
    while let Expr::MethodCall(m) = expr {
        if m.turbofish.is_some() {
            return None;
        }
        match m.method.to_string().as_str() {
            "filter" if m.args.len() == 1 => filter = true,
            "order" | "limit" if m.args.len() == 1 => (),
            "count" if m.args.is_empty() => count = true,
            _ => return None,
        }
        expr = &m.receiver;
    }
    let mut family = root(expr)?;
    if count {
        if family != "select" {
            return None;
        }
        family = "count";
    }
    Some(Candidate {
        family,
        filter: if filter {
            "present_syntax"
        } else {
            "absent_syntax"
        },
        missing_where: if matches!(family, "update" | "delete") && !filter {
            "candidate"
        } else {
            "not_established"
        },
    })
}
impl<'ast> Visit<'ast> for Scan {
    fn visit_item(&mut self, item: &'ast Item) {
        self.result.items += 1;
        if self.result.items > MAX_ITEMS {
            self.budget = true;
            return;
        }
        match item {
            Item::Fn(_) => (),
            Item::Use(_)
            | Item::Mod(_)
            | Item::Type(_)
            | Item::Struct(_)
            | Item::Enum(_)
            | Item::ExternCrate(_)
            | Item::Trait(_)
            | Item::Impl(_) => self.add(Diagnostic::NamesUnresolved),
            Item::Macro(_) => self.add(Diagnostic::MacroUnexpanded),
            _ => self.add(Diagnostic::UnsupportedSyntax),
        }
        visit::visit_item(self, item);
    }
    fn visit_attribute(&mut self, _: &'ast syn::Attribute) {
        self.add(Diagnostic::AttributeUnevaluated);
    }
    fn visit_macro(&mut self, _: &'ast syn::Macro) {
        self.add(Diagnostic::MacroUnexpanded);
    }
    fn visit_expr(&mut self, expr: &'ast Expr) {
        self.result.nodes += 1;
        if self.result.nodes > MAX_NODES {
            self.budget = true;
            return;
        }
        let in_receiver = self.chain_receiver;
        self.chain_receiver = false;
        if !in_receiver && let Some(c) = chain(expr) {
            if self.result.operations.len() == MAX_OPERATIONS {
                self.budget = true;
                return;
            }
            self.result.operations.push(c);
            self.add(Diagnostic::ArgumentsUnverified);
        }
        match expr {
            Expr::MethodCall(m) => {
                if chain(expr).is_none() {
                    self.add(Diagnostic::ReceiverUnresolved);
                }
                // Suppress duplicate roots ONLY along this receiver spine. All
                // arguments and their nested candidates are visited independently.
                self.chain_receiver = chain(expr).is_some();
                self.visit_expr(&m.receiver);
                self.chain_receiver = false;
                for a in &m.args {
                    self.visit_expr(a);
                }
                for a in &m.attrs {
                    self.visit_attribute(a);
                }
                if let Some(t) = &m.turbofish {
                    self.add(Diagnostic::UnsupportedSyntax);
                    self.visit_angle_bracketed_generic_arguments(t);
                }
                return;
            }
            Expr::Call(_) if root(expr).is_none() => self.add(Diagnostic::ReceiverUnresolved),
            Expr::If(_)
            | Expr::Match(_)
            | Expr::Loop(_)
            | Expr::While(_)
            | Expr::ForLoop(_)
            | Expr::Closure(_)
            | Expr::Async(_)
            | Expr::Await(_)
            | Expr::Try(_)
            | Expr::Return(_)
            | Expr::Break(_) => self.add(Diagnostic::ControlFlowUnresolved),
            Expr::Path(_) => self.add(Diagnostic::NamesUnresolved),
            Expr::Lit(_)
            | Expr::Block(_)
            | Expr::Call(_)
            | Expr::Tuple(_)
            | Expr::Array(_)
            | Expr::Paren(_)
            | Expr::Macro(_) => (),
            _ => self.add(Diagnostic::UnsupportedSyntax),
        }
        visit::visit_expr(self, expr);
    }
}

#[cfg(feature = "review-ci")]
impl Review {
    pub(crate) fn ci_codes(&self) -> Vec<crate::review_ci::Code> {
        use crate::review_ci::Code as C;
        let mut codes = vec![C::ReviewIncomplete];
        if self.state == "unsupported" {
            codes.push(C::Unsupported);
        }
        for d in &self.diagnostics {
            codes.push(match d {
                Diagnostic::IdentityUnverified => C::IdentityUnverified,
                Diagnostic::ContextUnverified => C::ContextUnverified,
                Diagnostic::NoCandidates => C::NoCandidates,
                Diagnostic::MacroUnexpanded => C::MacroUnexpanded,
                Diagnostic::AttributeUnevaluated => C::AttributeUnevaluated,
                Diagnostic::NamesUnresolved => C::NamesUnresolved,
                Diagnostic::ControlFlowUnresolved => C::ControlFlowUnresolved,
                Diagnostic::ReceiverUnresolved => C::ReceiverUnresolved,
                Diagnostic::ArgumentsUnverified => C::ArgumentsUnverified,
                Diagnostic::UnsupportedSyntax => C::UnsupportedSyntax,
            });
        }
        codes
    }
}
