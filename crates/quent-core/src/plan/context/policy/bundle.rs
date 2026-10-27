//! Fixed three-record bundle. Pure preparation only; no execution authority.
use super::*;

/// Application-only role declarations, never a general tenant remapping API.
/// Link columns must be distinct nonnullable Text columns. The caller supplies
/// stable record identity, payload, time and ordering as ordinary native values.
pub struct CommandCompanion {
    operation: Operation,
    tenant: Column,
    record_id: Column,
    command_id: Column,
    namespace: Column,
    subject: Column,
}
impl CommandCompanion {
    pub fn application_supplied(
        operation: Operation,
        tenant: Column,
        record_id: Column,
        command_id: Column,
        namespace: Column,
        subject: Column,
    ) -> Self {
        Self {
            operation,
            tenant,
            record_id,
            command_id,
            namespace,
            subject,
        }
    }
    fn copy_input(&self) -> Self {
        Self {
            operation: self.operation.clone(),
            tenant: self.tenant.clone(),
            record_id: self.record_id.clone(),
            command_id: self.command_id.clone(),
            namespace: self.namespace.clone(),
            subject: self.subject.clone(),
        }
    }
    fn roles(&self) -> [&Column; 5] {
        [
            &self.tenant,
            &self.record_id,
            &self.command_id,
            &self.namespace,
            &self.subject,
        ]
    }
}
/// Exactly one business operation, one audit INSERT, then one outbox INSERT.
/// No companion-only, variable-length or branching bundle exists.
pub struct CommandBundle {
    business: Operation,
    audit: CommandCompanion,
    outbox: CommandCompanion,
}
impl CommandBundle {
    pub fn new(business: Operation, audit: CommandCompanion, outbox: CommandCompanion) -> Self {
        Self {
            business,
            audit,
            outbox,
        }
    }
    pub fn sensitive_business(&self) -> &Operation {
        &self.business
    }
    fn copy_input(&self) -> Self {
        Self {
            business: self.business.clone(),
            audit: self.audit.copy_input(),
            outbox: self.outbox.copy_input(),
        }
    }
    pub fn same_bounded_input(&self, current: &Self) -> PolicyResult<bool> {
        if !self.business.same_bounded_input(&current.business)? {
            return Ok(false);
        }
        for (a, b) in [
            (&self.audit, &current.audit),
            (&self.outbox, &current.outbox),
        ] {
            if a.roles() != b.roles() || !a.operation.same_bounded_input(&b.operation)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
/// Non-executable retained pure preparation. Parts remain non-executable Core
/// snapshots; only the facade's private consumed carrier can bind a session.
pub struct CommandBundlePreparation {
    original: CommandBundle,
    records: [PolicyPreparation; 3],
}
impl CommandBundlePreparation {
    pub fn sensitive_records(&self) -> &[PolicyPreparation; 3] {
        &self.records
    }
    pub fn into_parts(self) -> (CommandBundle, [PolicyPreparation; 3]) {
        (self.original, self.records)
    }
    pub fn canonical_input(&self, ledger: &Table) -> PolicyResult<CommandCanonical> {
        use crate::plan::command_encoding::Encoder;
        let mut e = Encoder::new(b"QCC2");
        e.table(ledger)?;
        let context = &self.records[0].snapshot;
        context.encode_command_context(&mut e)?;
        e.column(
            &context
                .application
                .tenant
                .as_ref()
                .ok_or(PolicyError::MissingTenant)?
                .column,
        )?;
        for (index, p) in self.records.iter().enumerate() {
            e.number(index as u64)?; // fixed role and send ordering
            e.number(1)?; // fixed expected count
            if index > 0 {
                let c = if index == 1 {
                    &self.original.audit
                } else {
                    &self.original.outbox
                };
                e.columns(&c.roles().into_iter().cloned().collect::<Vec<_>>())?;
                e.value(&context.application.tenant.as_ref().unwrap().value)?;
            }
            e.operation(&p.original)?;
            let r = &p.plan.record;
            e.operation(&r.metadata.operation)?;
            e.bytes(r.sql.as_bytes())?;
            e.values(&r.args)?;
            e.boolean(r.caller_missing_where)?;
            e.boolean(r.insert_proof.is_some())?;
        }
        Ok(CommandCanonical(e.finish()))
    }
}
// Binary durable bytes are stored as canonical lowercase hex in supported Text
// link columns. Equality here is native exact bytes, never database collation.
fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(DIGITS[(b >> 4) as usize] as char);
        s.push(DIGITS[(b & 15) as usize] as char);
    }
    s
}
impl PreparationContext {
    pub fn prepare_command_bundle(
        &self,
        bundle: &CommandBundle,
        namespace: &[u8],
        id: &[u8; 16],
    ) -> PolicyResult<CommandBundlePreparation> {
        if namespace.is_empty() || namespace.len() > 128 {
            return Err(PolicyError::Budget);
        }
        let business = self.prepare_command_operation(&bundle.business)?;
        let source = self
            .snapshot
            .application
            .tenant
            .as_ref()
            .ok_or(PolicyError::MissingTenant)?;
        let subject = self
            .snapshot
            .application
            .subject
            .as_ref()
            .ok_or(PolicyError::Identity)?;
        let bt = &bundle.business.0.table;
        let at = &bundle.audit.operation.0.table;
        let ot = &bundle.outbox.operation.0.table;
        if bt == at || bt == ot || at == ot {
            return Err(PolicyError::Identity);
        }
        let prepare = |c: &CommandCompanion| -> PolicyResult<PolicyPreparation> {
            c.operation.check_budget()?;
            let d = &c.operation.0;
            if !matches!(d.body, Body::Insert(_))
                || !canonical_table(&d.table)
                || d.table.path[0] != bt.path[0]
                || c.tenant.table != d.table
                || c.tenant.nullable
                || c.tenant.sql_type != source.column.sql_type
                || !matches!(
                    c.tenant.sql_type,
                    SqlType::I8 | SqlType::I16 | SqlType::I32 | SqlType::I64
                )
                || self
                    .snapshot
                    .settings
                    .policies
                    .declarations
                    .iter()
                    .any(|p| p.column() == &c.tenant)
            {
                return Err(PolicyError::Unsupported);
            }
            let roles = c.roles();
            for (i, column) in roles.iter().enumerate() {
                if column.table != d.table
                    || column.nullable
                    || roles[..i].contains(column)
                    || (i > 0 && column.sql_type != SqlType::Text)
                {
                    return Err(PolicyError::Unsupported);
                }
            }
            let mut p = self.prepare_with_mapping(&c.operation, Some(&c.tenant))?;
            check_command_shape(&mut p)?;
            let Body::Insert(data) = &p.plan.record.metadata.operation.0.body else {
                return Err(PolicyError::Unsupported);
            };
            let value = |column: &Column| data.iter().find(|(c, _)| c == column).map(|(_, v)| v);
            if !matches!(value(&c.record_id),Some(Input::Value(Value::Text(s))) if !s.is_empty())
                || value(&c.command_id) != Some(&Input::Value(Value::Text(hex(id))))
                || value(&c.namespace) != Some(&Input::Value(Value::Text(hex(namespace))))
                || value(&c.subject) != Some(&Input::Value(Value::Text(subject.0.clone())))
            {
                return Err(PolicyError::Identity);
            }
            Ok(p)
        };
        let audit = prepare(&bundle.audit)?;
        let outbox = prepare(&bundle.outbox)?;
        Ok(CommandBundlePreparation {
            original: bundle.copy_input(),
            records: [business, audit, outbox],
        })
    }
}
redacted!(CommandCompanion, CommandBundle, CommandBundlePreparation);
