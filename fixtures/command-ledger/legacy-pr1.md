# Frozen PR1 canonical input fixtures

These synthetic binary fixtures were produced with the **unmodified** `quent-core`
from merged PR1 main `b9ef6dbfcc0f26e3e0f0c1ec1fe1061f9b1c2176`, extracted
with `git archive` into an isolated temporary workspace, Rust1.94.0, no dependencies.
They are not production records or secret credentials.

Input is `execution/tests/command.rs`'s existing `command_context(d,11,0,vec![])`
and `ci(888,Value::Null(SqlType::Text),false)`, ledger `command_ledger`, namespace
`CANARY_command_namespace`, durable ID `[150;16]`. Generate with the PR1 Core APIs
`prepare_command_operation`, `canonical_command_key`, `canonical_command_input`.
The fixture-producing program uses the unchanged original PR1 context/INSERT helper
bodies and writes the returned bytes directly, with no JSON/Debug/hash conversion.
Postgres/MySQL have separate QCK1 and QCC1 files. No new format is introduced.

The live PR2 suites seed each fixture into the real installed ledger with the
existing QCR1 dialect/count1 result and a separately seeded business row. Both the
implicit exact1 API and new explicit exact1 API must produce every identical
key/input byte, replay the old saved result, and send zero business operations.
This is compatibility evidence using PR1-produced canonical fixtures, not a claim
that an historical deployed database or arbitrary old application schema was tested.
Do not regenerate these fixtures from the current implementation to fix a failure.
