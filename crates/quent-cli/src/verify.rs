//! Explicit CLI workflow; fixed diagnostics only, no ambient connection fallback.
use quent::manifest::{ArtifactKind, ManifestSource, SourceCoverage};
use std::{
    io::{Read, Write},
    path::Path,
};
fn fail(code: u8, message: &'static str) -> u8 {
    let _ = std::io::stderr().write_all(message.as_bytes());
    code
}
fn read(path: &Path, max: usize) -> Result<Vec<u8>, u8> {
    let f = open_input(path)?;
    if !f
        .metadata()
        .map_err(|_| fail(8, "QUENT_VERIFY_IO\n"))?
        .is_file()
    {
        return Err(fail(2, "QUENT_VERIFY_INVALID\n"));
    }
    let mut b = Vec::new();
    f.take((max + 1) as u64)
        .read_to_end(&mut b)
        .map_err(|_| fail(8, "QUENT_VERIFY_IO\n"))?;
    if b.len() > max {
        return Err(fail(2, "QUENT_VERIFY_BUDGET\n"));
    }
    Ok(b)
}
// Check the object actually opened, never a path stat followed by a different open.
// NONBLOCK prevents a writerless FIFO from hanging before the regular-file check.
#[cfg(unix)]
fn open_input(path: &Path) -> Result<std::fs::File, u8> {
    use rustix::fs::{Mode, OFlags, open};
    open(
        path,
        OFlags::RDONLY | OFlags::NONBLOCK | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map(std::fs::File::from)
    .map_err(|_| fail(8, "QUENT_VERIFY_IO\n"))
}
#[cfg(windows)]
fn open_input(path: &Path) -> Result<std::fs::File, u8> {
    use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
    // Open the reparse point itself; do not follow a final symlink. Only a
    // regular file proceeds to Read (pipe/device metadata errors refuse).
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(0x0020_0000) // FILE_FLAG_OPEN_REPARSE_POINT
        .open(path)
        .map_err(|_| fail(8, "QUENT_VERIFY_IO\n"))?;
    let metadata = file.metadata().map_err(|_| fail(8, "QUENT_VERIFY_IO\n"))?;
    if metadata.file_attributes() & (0x400 | 0x40) != 0 {
        // Reparse points and device attributes are outside this input contract.
        return Err(fail(2, "QUENT_VERIFY_INVALID\n"));
    }
    Ok(file)
}
#[cfg(not(any(unix, windows)))]
fn open_input(_: &Path) -> Result<std::fs::File, u8> {
    Err(fail(4, "QUENT_VERIFY_UNSUPPORTED\n"))
}
pub(super) fn run() -> Option<u8> {
    let mut args = std::env::args_os();
    let _ = args.next();
    let a = args.next()?;
    #[cfg(all(
        feature = "runtime-tokio",
        any(feature = "postgres", feature = "mysql")
    ))]
    if a == "--quent-verify-worker" {
        return Some(worker(args));
    }
    if a != "manifest" {
        return None;
    }
    if args.next().as_deref() != Some(std::ffi::OsStr::new("verify")) {
        return None;
    }
    Some(command(args).unwrap_or_else(|e| e))
}
fn command(mut args: impl Iterator<Item = std::ffi::OsString>) -> Result<u8, u8> {
    let mode = args
        .next()
        .ok_or_else(|| fail(2, "QUENT_VERIFY_INVALID\n"))?;
    // Exact positional grammar deliberately refuses unknown, duplicate and extra args.
    let path = args
        .next()
        .ok_or_else(|| fail(2, "QUENT_VERIFY_INVALID\n"))?;
    let second = args
        .next()
        .ok_or_else(|| fail(2, "QUENT_VERIFY_INVALID\n"))?;
    if args.next().is_some() || (mode != "--offline" && mode != "--live") {
        return Err(fail(2, "QUENT_VERIFY_INVALID\n"));
    }
    if path.as_encoded_bytes().len() > 4096 || second.as_encoded_bytes().len() > 4096 {
        return Err(fail(2, "QUENT_VERIFY_BUDGET\n"));
    }
    let bytes = read(Path::new(&path), 65_536)?;
    let source = ManifestSource::parse(&bytes).map_err(|_| fail(2, "QUENT_VERIFY_INVALID\n"))?;
    if mode == "--offline" {
        let other = ManifestSource::parse(&read(Path::new(&second), 65_536)?)
            .map_err(|_| fail(2, "QUENT_VERIFY_INVALID\n"))?;
        let equal = source.sensitive_source().canonical_source()
            == other.sensitive_source().canonical_source();
        let summary = source.public_view();
        let coverage = summary.artifacts()[0].coverage();
        let unsupported = summary
            .artifacts()
            .iter()
            .any(|a| a.coverage() == SourceCoverage::Unsupported);
        let incomplete = summary.artifacts().iter().any(|a| {
            matches!(
                a.coverage(),
                SourceCoverage::Unknown | SourceCoverage::Partial
            )
        });
        let state = if !equal {
            "Different"
        } else if unsupported {
            "Unsupported"
        } else if coverage == SourceCoverage::Missing {
            "Missing"
        } else if incomplete || coverage != SourceCoverage::Complete {
            "Unknown"
        } else {
            "OfflineConsistent"
        };
        let output = format!(
            "{{\"kind\":\"quent_verify_offline\",\"version\":1,\"state\":\"{state}\",\"live\":\"Unverified\"}}\n"
        );
        std::io::stdout()
            .write_all(output.as_bytes())
            .map_err(|_| fail(8, "QUENT_VERIFY_IO\n"))?;
        return Ok(match state {
            "OfflineConsistent" => 0,
            "Different" => 5,
            "Unsupported" => 4,
            _ => 3,
        });
    }
    #[cfg(all(
        feature = "runtime-tokio",
        any(feature = "postgres", feature = "mysql")
    ))]
    {
        isolated(&bytes, &read(Path::new(&second), 8192)?)
    }
    #[cfg(not(all(
        feature = "runtime-tokio",
        any(feature = "postgres", feature = "mysql")
    )))]
    {
        let _ = ArtifactKind::Schema;
        Err(fail(4, "QUENT_VERIFY_UNSUPPORTED\n"))
    }
}
// SQLx PG options construction consults PG* defaults even without pgpass. Live CLI
// runs in a child with a closed bootstrap environment, so no inherited PGOPTIONS/credentials/
// certificate paths or implicit environment configuration can influence it.
#[cfg(all(
    feature = "runtime-tokio",
    any(feature = "postgres", feature = "mysql")
))]
fn isolated(source: &[u8], config: &[u8]) -> Result<u8, u8> {
    use std::process::{Command, Stdio};
    let exe = std::env::current_exe().map_err(|_| fail(8, "QUENT_VERIFY_IO\n"))?;
    let mut child = Command::new(exe)
        .arg("--quent-verify-worker")
        .env_clear()
        // Prevent SQLx's constructor from consulting whoami/default socket paths.
        // These nonsecret constants are overridden by the validated explicit config.
        .env("PGHOST", "127.0.0.1")
        .env("PGPORT", "1")
        .env("PGUSER", "quent_manifest_worker")
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|_| fail(8, "QUENT_VERIFY_IO\n"))?;
    let mut send = || -> std::io::Result<()> {
        let mut input = child.stdin.take().ok_or(std::io::ErrorKind::BrokenPipe)?;
        input.write_all(&(source.len() as u32).to_le_bytes())?;
        input.write_all(source)?;
        input.write_all(config)
    };
    if send().is_err() {
        let _ = child.kill();
        let _ = child.wait();
        return Err(fail(8, "QUENT_VERIFY_IO\n"));
    }
    let code = child
        .wait()
        .map_err(|_| fail(8, "QUENT_VERIFY_IO\n"))?
        .code();
    match code {
        Some(c @ (0 | 2 | 3 | 4 | 5 | 6 | 8)) => Ok(c as u8),
        _ => Err(fail(8, "QUENT_VERIFY_IO\n")),
    }
}
#[cfg(all(
    feature = "runtime-tokio",
    any(feature = "postgres", feature = "mysql")
))]
fn worker(mut args: impl Iterator<Item = std::ffi::OsString>) -> u8 {
    // Closed bootstrap environment; never inherit or discover application config.
    if args.next().is_some() {
        return fail(2, "QUENT_VERIFY_INVALID\n");
    }
    let mut mask = 0u8;
    let mut count = 0;
    for (key, value) in std::env::vars_os() {
        count += 1;
        let (bit, expected) = match key.to_str() {
            Some("PGHOST") => (1, "127.0.0.1"),
            Some("PGPORT") => (2, "1"),
            Some("PGUSER") => (4, "quent_manifest_worker"),
            _ => return fail(2, "QUENT_VERIFY_INVALID\n"),
        };
        if value != expected || count > 3 {
            return fail(2, "QUENT_VERIFY_INVALID\n");
        }
        mask |= bit;
    }
    if mask != 7 {
        return fail(2, "QUENT_VERIFY_INVALID\n");
    }
    let mut bytes = Vec::new();
    if std::io::stdin()
        .take(73_733)
        .read_to_end(&mut bytes)
        .is_err()
        || bytes.len() < 4
        || bytes.len() > 73_732
    {
        return fail(2, "QUENT_VERIFY_INVALID\n");
    }
    let n = u32::from_le_bytes(bytes[..4].try_into().unwrap()) as usize;
    if n > 65_536 || n + 4 > bytes.len() || bytes.len() - n - 4 > 8192 {
        return fail(2, "QUENT_VERIFY_INVALID\n");
    }
    live(&bytes[4..n + 4], &bytes[n + 4..]).unwrap_or_else(|e| e)
}
#[cfg(all(
    feature = "runtime-tokio",
    any(feature = "postgres", feature = "mysql")
))]
fn live(bytes: &[u8], config: &[u8]) -> Result<u8, u8> {
    use quent::manifest::live::*;
    use quent_core::*;
    use std::time::Duration;
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Config {
        transport: String,
        dialect: String,
        host: String,
        port: u16,
        user: String,
        password: String,
        database: String,
        address: String,
        catalog: String,
        namespace: String,
        tables: Vec<String>,
        generation: u64,
        timeout_ms: u64,
        ttl_ns: u64,
    }
    let cfg: Config =
        serde_json::from_slice(config).map_err(|_| fail(2, "QUENT_VERIFY_INVALID\n"))?;
    if cfg.transport != "tcp_plaintext" {
        return Err(fail(4, "QUENT_VERIFY_UNSUPPORTED\n"));
    }
    if cfg.host.is_empty()
        || cfg.host.len() > 128
        || !cfg
            .host
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".-:".contains(&b))
        || cfg.port == 0
        || cfg.user.is_empty()
        || cfg.user.len() > 128
        || cfg.password.len() > 4096
        || cfg.timeout_ms == 0
        || cfg.timeout_ms > 60_000
        || cfg.ttl_ns == 0
        || cfg.ttl_ns > 300_000_000_000
    {
        return Err(fail(2, "QUENT_VERIFY_INVALID\n"));
    }
    let dialect = match cfg.dialect.as_str() {
        "postgres" => Dialect::Postgres,
        "mysql" => Dialect::Mysql,
        _ => return Err(fail(4, "QUENT_VERIFY_UNSUPPORTED\n")),
    };
    let source = ManifestSource::parse(bytes).map_err(|_| fail(2, "QUENT_VERIFY_INVALID\n"))?;
    let scope = quent::manifest::live::Scope::new(
        &cfg.namespace,
        &cfg.tables.iter().map(String::as_str).collect::<Vec<_>>(),
    )
    .map_err(|_| fail(2, "QUENT_VERIFY_INVALID\n"))?;
    let build = || -> Result<DatabaseContext, ContextError> {
        let binding = DatabaseBinding::application_supplied(
            DatabaseIdentity::new(cfg.database.clone())?,
            TargetAddress::new(cfg.address.clone())?,
            Ident::new(cfg.catalog.clone()).map_err(|_| ContextError::Identity)?,
            dialect,
        );
        let generation = Generation::new(cfg.generation)?;
        let policy = PolicySet::new(
            binding.clone(),
            PolicyIdentity::new("cli_observation")?,
            generation,
            vec![],
        )?;
        DatabaseContext::new(
            Settings::new(
                binding.clone(),
                SettingsIdentity::new("cli_observation")?,
                generation,
                policy,
            )?,
            ApplicationContext::application_supplied(binding, None, None, None)?,
        )
    };
    let context = build().map_err(|_| fail(2, "QUENT_VERIFY_INVALID\n"))?;
    // Validate the source/explicit configuration and complete scope BEFORE connect.
    let target = context.sensitive_source();
    let (db, address, catalog, d) = target.target();
    let canonical = source
        .sensitive_source()
        .canonical_artifact(ArtifactKind::Schema)
        .ok_or_else(|| fail(3, "QUENT_VERIFY_MISSING\n"))?;
    let value: serde_json::Value =
        serde_json::from_slice(&canonical).map_err(|_| fail(2, "QUENT_VERIFY_INVALID\n"))?;
    if cfg.address != format!("{}:{}", cfg.host, cfg.port)
        || value["target"]["database"] != db
        || value["target"]["address"] != address
        || value["target"]["catalog"] != catalog.sensitive_text()
        || value["target"]["dialect"] != cfg.dialect
        || (d == Dialect::Mysql && cfg.namespace != cfg.catalog)
    {
        return Err(fail(2, "QUENT_VERIFY_TARGET\n"));
    }
    let expected = value["tables"]
        .as_array()
        .ok_or_else(|| fail(2, "QUENT_VERIFY_INVALID\n"))?;
    if expected.len() != cfg.tables.len()
        || expected.iter().any(|t| {
            t["path"].as_array().is_none_or(|p| {
                p.len() != 2 || p[0] != cfg.namespace || !cfg.tables.iter().any(|n| p[1] == *n)
            })
        })
    {
        return Err(fail(2, "QUENT_VERIFY_TARGET\n"));
    }
    let budget = Budget {
        queries: 65,
        rows: 8192,
        bytes: 1_048_576,
        string_bytes: 4096,
        tables: 16,
        columns: 256,
        keys: 16,
        branches: 256,
        timeout: Duration::from_millis(cfg.timeout_ms),
    };
    let ttl = Duration::from_nanos(cfg.ttl_ns);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| fail(8, "QUENT_VERIFY_IO\n"))?;
    let result: Result<VerifySummary, VerifyError> = runtime.block_on(async {
        match dialect {
            #[cfg(feature = "postgres")]
            Dialect::Postgres => {
                let options = sqlx::postgres::PgConnectOptions::new_without_pgpass()
                    .host(&cfg.host)
                    .port(cfg.port)
                    .username(&cfg.user)
                    .password(&cfg.password)
                    .database(&cfg.catalog)
                    .ssl_mode(sqlx::postgres::PgSslMode::Disable)
                    .application_name("quent_manifest_verify")
                    .options([("search_path", cfg.namespace.as_str())]);
                let mut reader =
                    PostgresReader::connect(options, context.clone(), budget.timeout).await?;
                let observation = reader
                    .observe(&context, &scope, &source, budget, ttl)
                    .await?;
                Ok(reader.compare(&context, &scope, &source, &observation))
            }
            #[cfg(feature = "mysql")]
            Dialect::Mysql => {
                let options = sqlx::mysql::MySqlConnectOptions::new()
                    .host(&cfg.host)
                    .port(cfg.port)
                    .username(&cfg.user)
                    .database(&cfg.catalog)
                    .ssl_mode(sqlx::mysql::MySqlSslMode::Disabled);
                let options = if cfg.password.is_empty() {
                    options
                } else {
                    options.password(&cfg.password)
                };
                let mut reader =
                    MysqlReader::connect(options, context.clone(), budget.timeout).await?;
                let observation = reader
                    .observe(&context, &scope, &source, budget, ttl)
                    .await?;
                Ok(reader.compare(&context, &scope, &source, &observation))
            }
            #[allow(unreachable_patterns)]
            _ => Err(VerifyError::Unsupported),
        }
    });
    let summary = result.map_err(|e| match e {
        VerifyError::Invalid | VerifyError::Target | VerifyError::Budget => {
            fail(2, "QUENT_VERIFY_INVALID\n")
        }
        VerifyError::Missing => fail(3, "QUENT_VERIFY_MISSING\n"),
        VerifyError::Unknown => fail(3, "QUENT_VERIFY_UNKNOWN\n"),
        VerifyError::Unsupported => fail(4, "QUENT_VERIFY_UNSUPPORTED\n"),
        VerifyError::Expired | VerifyError::Clock => fail(6, "QUENT_VERIFY_STALE\n"),
        VerifyError::Connect => fail(8, "QUENT_VERIFY_CONNECT\n"),
        VerifyError::Io => fail(8, "QUENT_VERIFY_IO\n"),
    })?;
    std::io::stdout()
        .write_all(format!("{}\n", summary.to_json()).as_bytes())
        .map_err(|_| fail(8, "QUENT_VERIFY_IO\n"))?;
    Ok(match summary.verdict() {
        Verdict::Matched => 0,
        Verdict::Drift => 5,
        Verdict::Unsupported => 4,
        Verdict::Stale => 6,
        _ => 3,
    })
}
