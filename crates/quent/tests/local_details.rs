#![cfg(feature = "local-details")]
use quent::local_details::*;
use quent_core::*;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
#[cfg(feature = "output-tracing")]
mod output_capture;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "quent-detail-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let b = fs::DirBuilder::new();
        #[cfg(unix)]
        let b = {
            use std::os::unix::fs::DirBuilderExt;
            let mut b = b;
            b.mode(0o700);
            b
        };
        b.create(&p).unwrap();
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
        assert!(!self.0.exists());
    }
}
fn plan(d: Dialect) -> Plan {
    let t = Table::new(
        vec![
            Ident::new("CANARY_schema").unwrap(),
            Ident::new("CANARY_table").unwrap(),
        ],
        None,
    )
    .unwrap();
    let mut values = vec![
        Value::I16(i16::MIN),
        Value::I32(i32::MIN),
        Value::I64(i64::MIN),
        Value::Bool(false),
        Value::Text("CANARY\n\r\t\u{1b}\\\"日本語".into()),
    ];
    if d == Dialect::Mysql {
        values.extend([
            Value::I8(i8::MIN),
            Value::U8(u8::MAX),
            Value::U16(u16::MAX),
            Value::U32(u32::MAX),
            Value::U64(u64::MAX),
        ]);
    }
    values.extend(values.clone().iter().map(|v| Value::Null(v.sql_type())));
    let data = values
        .into_iter()
        .enumerate()
        .map(|(i, v)| {
            (
                t.column(
                    Ident::new(format!("CANARY_col{i}")).unwrap(),
                    v.sql_type(),
                    true,
                ),
                Input::Value(v),
            )
        })
        .collect();
    Draft::insert(t, data).freeze().finalize(d).unwrap()
}
fn export(path: &Path, p: &Plan) -> Result<CreatedLocalDetail, DetailError> {
    create_new(
        LocalOptIn::acknowledge_sensitive_local_file(),
        Surface::LocalApplication,
        path,
        p.sensitive_source(),
    )
}
#[cfg(target_os = "linux")]
fn in_ci() -> bool {
    std::env::var_os("CI").is_some() || std::env::var_os("GITHUB_ACTIONS").is_some()
}
#[test]
fn surface_denial_is_finite_and_precedes_path_or_source_output() {
    let temp = Temp::new();
    let p = plan(Dialect::Mysql);
    for s in [Surface::Ci, Surface::Mcp, Surface::Remote] {
        let path = temp.0.join("CANARY_path");
        let e = create_new(
            LocalOptIn::acknowledge_sensitive_local_file(),
            s,
            &path,
            p.sensitive_source(),
        )
        .unwrap_err();
        assert_eq!(e, DetailError::SurfaceDenied);
        assert!(!path.exists());
        assert!(std::error::Error::source(&e).is_none());
        assert!(!format!("{e} {e:?}").contains("CANARY"));
    }
}
#[cfg(target_os = "linux")]
#[test]
fn actual_new_file_exact_source_typed_values_mode_and_unchanged_gate() {
    #[cfg(feature = "output-tracing")]
    let capture = output_capture::Capture::default();
    #[cfg(feature = "output-tracing")]
    let _subscriber = tracing::subscriber::set_default(capture.clone());
    let temp = Temp::new();
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let p = plan(d);
        let path = temp.0.join(format!("{d:?}"));
        let before = p.public_view().to_json();
        if in_ci() {
            assert_eq!(
                export(&path, &p).unwrap_err(),
                DetailError::EnvironmentDenied
            );
            assert!(!path.exists());
            continue;
        }
        export(&path, &p).unwrap();
        use std::os::unix::fs::MetadataExt;
        let m = fs::metadata(&path).unwrap();
        assert_eq!(m.mode() & 0o7777, 0o600);
        assert_eq!(m.uid(), rustix::process::geteuid().as_raw());
        assert_eq!(m.nlink(), 1);
        let bytes = fs::read(&path).unwrap();
        let j: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(j["kind"] == "quent_local_source" && j["version"] == 1);
        assert!(j["sql"] == p.sensitive_source().sql());
        let args = j["args"].as_array().unwrap();
        assert_eq!(args.len(), p.sensitive_source().args().len());
        for (v, j) in p.sensitive_source().args().iter().zip(args) {
            let (ty, value) = match v {
                Value::I8(n) => ("i8", serde_json::json!(n.to_string())),
                Value::I16(n) => ("i16", serde_json::json!(n.to_string())),
                Value::I32(n) => ("i32", serde_json::json!(n.to_string())),
                Value::I64(n) => ("i64", serde_json::json!(n.to_string())),
                Value::U8(n) => ("u8", serde_json::json!(n.to_string())),
                Value::U16(n) => ("u16", serde_json::json!(n.to_string())),
                Value::U32(n) => ("u32", serde_json::json!(n.to_string())),
                Value::U64(n) => ("u64", serde_json::json!(n.to_string())),
                Value::Bool(n) => ("bool", serde_json::json!(n)),
                Value::Text(n) => ("text", serde_json::json!(n)),
                Value::Null(t) => (
                    match t {
                        SqlType::I8 => "i8",
                        SqlType::I16 => "i16",
                        SqlType::I32 => "i32",
                        SqlType::I64 => "i64",
                        SqlType::U8 => "u8",
                        SqlType::U16 => "u16",
                        SqlType::U32 => "u32",
                        SqlType::U64 => "u64",
                        SqlType::Bool => "bool",
                        SqlType::Text => "text",
                    },
                    serde_json::Value::Null,
                ),
            };
            assert!(j == &serde_json::json!({"type":ty,"value":value}));
        }
        let expected_data = args.iter().enumerate().map(|(i,v)| serde_json::json!({"column":{"table":{"path":["CANARY_schema","CANARY_table"],"alias":null},"name":format!("CANARY_col{i}"),"type":v["type"],"nullable":true},"input":v})).collect::<Vec<_>>();
        assert!(
            j["metadata"]
                == serde_json::json!({"table":{"path":["CANARY_schema","CANARY_table"],"alias":null},"family":"insert","projection":null,"predicate":null,"write_data":expected_data,"order":[],"limit":null,"version":null,"caller_missing_where":false})
        );
        assert!(!String::from_utf8_lossy(&bytes).contains("[REDACTED]"));
        assert_eq!(before, p.public_view().to_json());
        assert_eq!(export(&path, &p).unwrap_err(), DetailError::CreateRefused);
        assert!(fs::read(&path).unwrap() == bytes);
        #[cfg(feature = "output-tracing")]
        capture.assert_safe(0);
    }
}
#[cfg(target_os = "linux")]
#[test]
fn rejects_existing_symlink_directory_fifo_device_relative_parent_and_racing_create() {
    if in_ci() {
        return;
    } // product CI-denial is tested above; private primitives run in CI.
    use std::os::unix::fs::{PermissionsExt, symlink};
    let temp = Temp::new();
    let p = plan(Dialect::Postgres);
    let original = temp.0.join("original");
    fs::write(&original, b"unchanged").unwrap();
    let link = temp.0.join("symlink");
    symlink(&original, &link).unwrap();
    let dir = temp.0.join("directory");
    fs::create_dir(&dir).unwrap();
    let fifo = temp.0.join("fifo");
    rustix::fs::mkfifoat(rustix::fs::CWD, &fifo, rustix::fs::Mode::RUSR).unwrap();
    for path in [&original, &link, &dir, &fifo] {
        assert_eq!(export(path, &p).unwrap_err(), DetailError::CreateRefused);
    }
    assert!(fs::read(&original).unwrap() == b"unchanged");
    let parent_link = temp.0.join("parent_link");
    symlink(&dir, &parent_link).unwrap();
    for path in [
        PathBuf::from("CANARY_relative"),
        temp.0.join("../CANARY_escape"),
        parent_link.join("new"),
        temp.0.join("missing/new"),
        PathBuf::from("/dev/null"),
    ] {
        assert!(export(&path, &p).is_err());
    }
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o777)).unwrap();
    assert_eq!(
        export(&dir.join("new"), &p).unwrap_err(),
        DetailError::ProtectionFailed
    );
    assert!(!dir.join("new").exists());
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
    let race = temp.0.join("race");
    let barrier = std::sync::Barrier::new(8);
    let results = std::thread::scope(|s| {
        let handles = (0..8)
            .map(|_| {
                let path = &race;
                let b = &barrier;
                s.spawn(move || {
                    b.wait();
                    export(path, &plan(Dialect::Mysql))
                })
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert!(
        results
            .iter()
            .filter_map(|r| r.as_ref().err())
            .all(|e| *e == DetailError::CreateRefused)
    );
    let j: serde_json::Value = serde_json::from_slice(&fs::read(&race).unwrap()).unwrap();
    assert!(j["kind"] == "quent_local_source");
}
#[cfg(not(target_os = "linux"))]
#[test]
fn unsupported_os_never_creates_or_writes_a_path() {
    let temp = Temp::new();
    let path = temp.0.join("CANARY_path");
    let e = export(&path, &plan(Dialect::Mysql)).unwrap_err();
    assert!(matches!(
        e,
        DetailError::UnsupportedOs | DetailError::EnvironmentDenied
    ));
    assert!(!path.exists());
}
