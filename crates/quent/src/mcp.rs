//! Finite MCP 2025-11-25 stdio process server. No DB/provider/Tx capability.
//!
//! `Server` owns one immutable registration for one process/session. The caller
//! supplies current independently; neither a manifest nor transport input can
//! register/replace it. A startup snapshot is not ongoing authentication/freshness.
//! Callers must terminate their process promptly after `serve_stdio` returns:
//! bounded detached I/O workers may still await the OS. No OS scheduling or hard
//! CPU-preemption deadline is promised. See `specs/mcp-v1.md` for finite limits.
use crate::operation_spec::{SpecSource, TypedModelContract};
use quent_core::DatabaseContext;
use serde_json::{Value, json};
use std::fmt;
mod transport;

/// The sole protocol version implemented; initialization negotiates this constant.
pub const PROTOCOL_VERSION: &str = "2025-11-25";
/// Fixed resources; never derived from identifiers or input paths.
pub const RESOURCES: [&str; 4] = [
    "quent://public/context",
    "quent://public/schema",
    "quent://public/manifest-status",
    "quent://public/review-rules",
];
/// Fixed tool register. No arbitrary tool dispatcher or application callback.
pub const TOOLS: [&str; 4] = [
    "quent_plan",
    "quent_compile",
    "quent_review",
    "quent_manifest_status",
];
/// One process-local immutable registration. No public handle or serde conversion.
pub struct Server {
    registration: Option<(TypedModelContract, DatabaseContext)>,
}
impl fmt::Debug for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Server([REDACTED])")
    }
}
/// Input-free session outcome. Neither success nor an MCP result authorizes DB use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Termination {
    Eof,
    Invalid,
    Budget,
    Timeout,
    Io,
}
impl Termination {
    /// CLI process status; EOF is graceful, all refusals are non-success.
    pub fn exit_code(self) -> u8 {
        match self {
            Self::Eof => 0,
            Self::Invalid => 2,
            Self::Budget => 3,
            Self::Timeout => 6,
            Self::Io => 8,
        }
    }
}
impl Server {
    /// No implicit file/config/environment/DB lookup and no trusted registration.
    pub fn offline() -> Self {
        Self { registration: None }
    }
    /// Authenticated application code supplies BOTH existing carriers independently.
    /// They are frozen for this session, not refreshed or manufactured from wire.
    pub fn application_registered(mapping: TypedModelContract, current: DatabaseContext) -> Self {
        Self {
            registration: Some((mapping, current)),
        }
    }
    /// Own stdio for this process. stdout is protocol-only. Return then exit the
    /// process; do not reuse stdin/stdout or start another session in that process.
    pub fn serve_stdio(self) -> Termination {
        transport::serve(self)
    }

    fn resource(&self, uri: &str) -> Option<Value> {
        let registered = self.registration.is_some();
        Some(match uri {
            "quent://public/context" => {
                json!({"kind":"quent_mcp_context","version":1,"registration":if registered {"application_asserted"} else {"missing"},"freshness":"unverified","runtime":"unverified","execution":"unavailable"})
            }
            "quent://public/schema" => {
                let (columns, complete) = self
                    .registration
                    .as_ref()
                    .map(|(m, _)| m.mcp_counts())
                    .unwrap_or((0, false));
                json!({"kind":"quent_mcp_schema","version":1,"models":usize::from(registered),"columns":columns,"coverage":if complete {"application_asserted"} else {"unknown"},"freshness":"unverified"})
            }
            "quent://public/manifest-status" => {
                json!({"kind":"quent_mcp_manifest_status","version":1,"schema":"missing","policy":"missing","generated_code":"missing","live":"unverified"})
            }
            "quent://public/review-rules" => {
                json!({"kind":"quent_mcp_rules","version":1,"mode":"strict","authority":"none","unsupported":"refuse","runtime":"unverified","rules":["read_only","offline_only","public_output_only","application_context_required","no_execution","no_permit","no_live_freshness"]})
            }
            _ => return None,
        })
    }
    fn call(&self, name: &str, arguments: &Value) -> Result<Value, Fault> {
        let args = object(arguments, &["source"], &["source"])?;
        let source = args["source"].as_str().ok_or(Fault::Params)?;
        if source.len() > 4096 {
            return Err(Fault::Params);
        }
        // Strings are source bytes, NEVER paths/URIs/SQL/DSNs or command lines.
        let result = match name {
            "quent_plan" | "quent_compile" => {
                let (mapping, current) = self.registration.as_ref().ok_or(Fault::MissingContext)?;
                let s = SpecSource::parse(source.as_bytes()).map_err(|_| Fault::Input)?;
                s.compile(mapping, current)
                    .map_err(|_| Fault::Input)?
                    .public_view()
                    .to_json()
            }
            "quent_review" => {
                if let Some((m, c)) = &self.registration {
                    let s = SpecSource::parse(source.as_bytes()).map_err(|_| Fault::Input)?;
                    crate::review::typed_spec(s, m, c)
                        .map_err(|_| Fault::Input)?
                        .to_json()
                } else {
                    crate::review::operation_spec(source.as_bytes())
                        .map_err(|_| Fault::Input)?
                        .to_json()
                }
            }
            "quent_manifest_status" => crate::review::manifest(source.as_bytes())
                .map_err(|_| Fault::Input)?
                .to_json(),
            _ => return Err(Fault::Params),
        };
        // Only retained output-only public serializers reach MCP content.
        Ok(json!({"content":[{"type":"text","text":result}],"isError":false}))
    }
    fn request(&self, method: &str, params: &Value, phase: &mut Phase) -> Result<Value, Fault> {
        if method == "initialize" {
            if *phase != Phase::New {
                return Err(Fault::State);
            }
            let p = object(
                params,
                &["protocolVersion", "capabilities", "clientInfo"],
                &["protocolVersion", "capabilities", "clientInfo"],
            )?;
            if !p["protocolVersion"].is_string() {
                return Err(Fault::Params);
            }
            // Client information/capabilities are bounded and discarded, never
            // reflected, used to connect, or treated as application authentication.
            object(&p["capabilities"], &[], &[])?;
            let c = object(&p["clientInfo"], &["name", "version"], &["name", "version"])?;
            if !c["name"].is_string() || !c["version"].is_string() {
                return Err(Fault::Params);
            }
            *phase = Phase::Initialized;
            return Ok(
                json!({"protocolVersion":PROTOCOL_VERSION,"capabilities":{"resources":{},"tools":{}},"serverInfo":{"name":"quent-readonly","version":"0.1.0"}}),
            );
        }
        if method == "ping" {
            object(params, &[], &[])?;
            return Ok(json!({}));
        }
        if *phase != Phase::Ready {
            return Err(Fault::State);
        }
        match method {
            "tools/list" => {
                object(params, &[], &[])?;
                Ok(
                    json!({"tools":TOOLS.iter().map(|name|json!({"name":name,"description":match *name {"quent_review"=>"Offline spec review; typed with application registration, syntax-only otherwise.","quent_manifest_status"=>"Offline manifest public summary; never registration or live verification.",_=>"Typed read compilation requires application registration; unavailable in unregistered CLI."},"inputSchema":{"type":"object","properties":{"source":{"type":"string","maxLength":4096}},"required":["source"],"additionalProperties":false},"annotations":{"readOnlyHint":true,"destructiveHint":false,"idempotentHint":true,"openWorldHint":false}})).collect::<Vec<_>>()}),
                )
            }
            "resources/list" => {
                object(params, &[], &[])?;
                Ok(
                    json!({"resources":RESOURCES.iter().map(|uri|json!({"uri":uri,"name":match *uri {"quent://public/context"=>"context","quent://public/schema"=>"schema","quent://public/manifest-status"=>"manifest-status",_=>"review-rules"},"mimeType":"application/json"})).collect::<Vec<_>>()}),
                )
            }
            "resources/templates/list" => {
                object(params, &[], &[])?;
                Ok(json!({"resourceTemplates":[]}))
            }
            "resources/read" => {
                let p = object(params, &["uri"], &["uri"])?;
                let uri = p["uri"].as_str().ok_or(Fault::Params)?;
                // Select constant URI for output rather than copying caller bytes.
                let fixed = RESOURCES
                    .into_iter()
                    .find(|u| *u == uri)
                    .ok_or(Fault::Resource)?;
                let value = self.resource(fixed).ok_or(Fault::Resource)?;
                Ok(
                    json!({"contents":[{"uri":fixed,"mimeType":"application/json","text":value.to_string()}]}),
                )
            }
            "tools/call" => {
                let p = object(params, &["name", "arguments"], &["name", "arguments"])?;
                self.call(p["name"].as_str().ok_or(Fault::Params)?, &p["arguments"])
            }
            _ => Err(Fault::Method),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    New,
    Initialized,
    Ready,
}
#[derive(Clone, Copy)]
enum Fault {
    Params,
    State,
    Method,
    Resource,
    MissingContext,
    Input,
}
impl Fault {
    fn wire(self) -> Value {
        let (code, message) = match self {
            Self::Method => (-32601, "method_not_found"),
            Self::Resource => (-32002, "resource_not_found"),
            Self::MissingContext => (-32000, "missing_application_context"),
            Self::Input => (-32602, "input_refused"),
            Self::State => (-32600, "invalid_state"),
            Self::Params => (-32602, "invalid_params"),
        };
        json!({"code":code,"message":message})
    }
}
fn object<'a>(
    v: &'a Value,
    allowed: &[&str],
    required: &[&str],
) -> Result<&'a serde_json::Map<String, Value>, Fault> {
    let o = v.as_object().ok_or(Fault::Params)?;
    if o.keys().any(|k| !allowed.contains(&k.as_str()))
        || required.iter().any(|k| !o.contains_key(*k))
    {
        return Err(Fault::Params);
    }
    Ok(o)
}
