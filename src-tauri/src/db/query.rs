//! Free-form SQL execution. Input may contain multiple statements; we split
//! at top-level `;` (quote/dollar-quote/comment aware) and run each over the
//! extended protocol for full column-type metadata, falling back to the
//! simple protocol for statements that cannot be prepared (SET, BEGIN,
//! VACUUM, CREATE EXTENSION, ...).

use std::time::Instant;

use futures_util::{pin_mut, TryStreamExt};
use tokio_postgres::types::ToSql;
use tokio_postgres::SimpleQueryMessage;

use crate::db::pool::NoticeSink;
use crate::db::value;
use crate::error::AppResult;
use crate::models::{SqlExecResult, StatementResult};

pub const DEFAULT_MAX_ROWS: usize = 10_000;

fn drain_notices(sink: &NoticeSink) -> Vec<String> {
    std::mem::take(&mut *sink.lock().unwrap())
}

/// Execute against an already-checked-out client (so the caller can hold its
/// `cancel_token()` for the duration). `geom`/`geog` are the connection's
/// PostGIS OIDs; `notices` is the pool's NOTICE sink, drained per statement.
pub async fn run_sql(
    client: &tokio_postgres::Client,
    geom: Option<u32>,
    geog: Option<u32>,
    sql: &str,
    max_rows: usize,
    notices: &NoticeSink,
) -> AppResult<SqlExecResult> {
    let mut statements = Vec::new();

    for stmt_sql in split_sql(sql) {
        let s = stmt_sql.trim();
        if s.is_empty() {
            continue;
        }
        let started = Instant::now();

        match client.prepare(s).await {
            Ok(prepared) => {
                if prepared.columns().is_empty() {
                    let n = client.execute(&prepared, &[]).await?;
                    statements.push(StatementResult {
                        columns: vec![],
                        rows: vec![],
                        row_count: n,
                        command_tag: format!("{} {}", verb(s), n),
                        elapsed_ms: ms(started),
                        truncated: false,
                        notices: drain_notices(notices),
                    });
                } else {
                    let cols: Vec<_> = prepared
                        .columns()
                        .iter()
                        .map(|c| {
                            value::column_meta(
                                c.name(),
                                c.type_(),
                                geom,
                                geog,
                            )
                        })
                        .collect();
                    // Stream rows so a huge result set cannot exhaust memory;
                    // stop consuming once the cap is hit (dropping the stream
                    // discards whatever the server still has buffered).
                    let col_types: Vec<_> =
                        prepared.columns().iter().map(|c| c.type_().clone()).collect();
                    let stream = client
                        .query_raw(&prepared, Vec::<&(dyn ToSql + Sync)>::new())
                        .await?;
                    pin_mut!(stream);
                    let mut data: Vec<Vec<Option<String>>> = Vec::new();
                    let mut truncated = false;
                    while let Some(r) = stream.try_next().await? {
                        if data.len() >= max_rows {
                            truncated = true;
                            break;
                        }
                        let mut cells = Vec::with_capacity(cols.len());
                        for (i, ty) in col_types.iter().enumerate() {
                            cells.push(value::cell(&r, i, ty, geom, geog));
                        }
                        data.push(cells);
                    }
                    let rc = data.len() as u64;
                    statements.push(StatementResult {
                        columns: cols,
                        rows: data,
                        row_count: rc,
                        command_tag: format!("SELECT {rc}"),
                        elapsed_ms: ms(started),
                        truncated,
                        notices: drain_notices(notices),
                    });
                }
            }
            Err(_) => {
                // Unpreparable utility statement — run via simple protocol.
                let msgs = client.simple_query(s).await?;
                let mut tag = String::from("OK");
                for m in &msgs {
                    if let SimpleQueryMessage::CommandComplete(n) = m {
                        tag = format!("{} {}", verb(s), n);
                    }
                }
                statements.push(StatementResult {
                    columns: vec![],
                    rows: vec![],
                    row_count: 0,
                    command_tag: tag,
                    elapsed_ms: ms(started),
                    truncated: false,
                    notices: drain_notices(notices),
                });
            }
        }
    }

    Ok(SqlExecResult { statements })
}

fn ms(t: Instant) -> u64 {
    t.elapsed().as_millis() as u64
}

fn verb(sql: &str) -> String {
    sql.split_whitespace()
        .next()
        .unwrap_or("OK")
        .to_uppercase()
}

/// Split on top-level `;`, ignoring delimiters inside single/double quotes,
/// dollar-quoted bodies, and line/block comments.
fn split_sql(sql: &str) -> Vec<String> {
    let b = sql.as_bytes();
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    let n = b.len();

    while i < n {
        let c = b[i];
        match c {
            b'\'' | b'"' => {
                let q = c;
                i += 1;
                while i < n {
                    if b[i] == q {
                        // doubled quote = escaped quote
                        if i + 1 < n && b[i + 1] == q {
                            i += 2;
                            continue;
                        }
                        break;
                    }
                    i += 1;
                }
            }
            b'-' if i + 1 < n && b[i + 1] == b'-' => {
                while i < n && b[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < n && b[i + 1] == b'*' => {
                i += 2;
                while i + 1 < n && !(b[i] == b'*' && b[i + 1] == b'/') {
                    i += 1;
                }
                i += 1;
            }
            b'$' => {
                // dollar-quote tag: $tag$ ... $tag$
                if let Some(tag_end) = find_dollar_tag(b, i) {
                    let tag = &b[i..=tag_end];
                    i = tag_end + 1;
                    while i + tag.len() <= n && &b[i..i + tag.len()] != tag {
                        i += 1;
                    }
                    i += tag.len().saturating_sub(1);
                }
            }
            b';' => {
                out.push(sql[start..i].to_string());
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    if start < n {
        out.push(sql[start..].to_string());
    }
    out
}

/// At position `i` (a `$`), return the index of the closing `$` of the tag,
/// if `b[i..]` opens a valid `$tag$` / `$$`.
fn find_dollar_tag(b: &[u8], i: usize) -> Option<usize> {
    let mut j = i + 1;
    while j < b.len() {
        match b[j] {
            b'$' => return Some(j),
            x if x == b'_' || x.is_ascii_alphanumeric() => j += 1,
            _ => return None,
        }
    }
    None
}
