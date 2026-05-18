//! Generic Postgres value -> display string. The frontend only ever sees
//! `Option<String>`; this module is the single place that knows Postgres
//! types. It must be **total**: an unknown/undecodable value degrades to a
//! hex dump, never a panic.
//!
//! `tokio_postgres::types::Type` is a struct (not an enum), so it cannot be
//! used in match patterns — we compare with `==` instead.

use std::error::Error;

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use rust_decimal::Decimal;
use tokio_postgres::types::{FromSql, Type};
use tokio_postgres::Row;
use uuid::Uuid;

use crate::models::{ColumnMeta, RenderHint};

/// Catch-all: capture the raw wire bytes for any type we don't decode natively.
struct Raw(Vec<u8>);

impl<'a> FromSql<'a> for Raw {
    fn from_sql(_: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        Ok(Raw(raw.to_vec()))
    }
    fn accepts(_: &Type) -> bool {
        true
    }
}

fn hex_prefixed(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2 + 2);
    s.push_str("\\x");
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

fn raw_fallback(row: &Row, i: usize) -> Option<String> {
    match row.try_get::<_, Option<Raw>>(i) {
        Ok(Some(r)) => Some(hex_prefixed(&r.0)),
        Ok(None) => None,
        Err(_) => Some("<undecodable>".into()),
    }
}

fn fmt_array<T>(items: Vec<Option<T>>, f: impl Fn(&T) -> String) -> String {
    let parts: Vec<String> = items
        .iter()
        .map(|o| o.as_ref().map(&f).unwrap_or_else(|| "NULL".into()))
        .collect();
    format!("{{{}}}", parts.join(","))
}

/// Format one cell. `geom`/`geog` are the resolved PostGIS OIDs for this
/// connection, used to recognise extension types by OID.
pub fn cell(
    row: &Row,
    i: usize,
    ty: &Type,
    geom: Option<u32>,
    geog: Option<u32>,
) -> Option<String> {
    macro_rules! get {
        ($t:ty, $fmt:expr) => {
            match row.try_get::<_, Option<$t>>(i) {
                Ok(Some(v)) => return Some($fmt(v)),
                Ok(None) => return None,
                Err(_) => return raw_fallback(row, i),
            }
        };
    }
    macro_rules! arr {
        ($t:ty, $fmt:expr) => {
            match row.try_get::<_, Option<Vec<Option<$t>>>>(i) {
                Ok(Some(v)) => return Some(fmt_array(v, $fmt)),
                Ok(None) => return None,
                Err(_) => return raw_fallback(row, i),
            }
        };
    }

    let oid = ty.oid();
    // PostGIS geometry/geography: binary EWKB; readable EWKT decode is a
    // later, on-demand feature.
    if Some(oid) == geom || Some(oid) == geog {
        return raw_fallback(row, i);
    }

    let t = ty.clone();
    if t == Type::BOOL {
        get!(bool, |v: bool| v.to_string());
    } else if t == Type::INT2 {
        get!(i16, |v: i16| v.to_string());
    } else if t == Type::INT4 {
        get!(i32, |v: i32| v.to_string());
    } else if t == Type::INT8 {
        get!(i64, |v: i64| v.to_string());
    } else if t == Type::OID {
        get!(u32, |v: u32| v.to_string());
    } else if t == Type::FLOAT4 {
        get!(f32, |v: f32| v.to_string());
    } else if t == Type::FLOAT8 {
        get!(f64, |v: f64| v.to_string());
    } else if t == Type::NUMERIC {
        get!(Decimal, |v: Decimal| v.to_string());
    } else if t == Type::CHAR {
        get!(i8, |v: i8| v.to_string());
    } else if t == Type::TEXT
        || t == Type::VARCHAR
        || t == Type::NAME
        || t == Type::BPCHAR
        || t == Type::UNKNOWN
    {
        get!(String, |v: String| v);
    } else if t == Type::UUID {
        get!(Uuid, |v: Uuid| v.to_string());
    } else if t == Type::JSON || t == Type::JSONB {
        get!(serde_json::Value, |v: serde_json::Value| v.to_string());
    } else if t == Type::DATE {
        get!(NaiveDate, |v: NaiveDate| v.to_string());
    } else if t == Type::TIME {
        get!(NaiveTime, |v: NaiveTime| v.to_string());
    } else if t == Type::TIMESTAMP {
        get!(NaiveDateTime, |v: NaiveDateTime| v
            .format("%Y-%m-%d %H:%M:%S%.f")
            .to_string());
    } else if t == Type::TIMESTAMPTZ {
        get!(DateTime<Utc>, |v: DateTime<Utc>| v.to_rfc3339());
    } else if t == Type::BYTEA {
        get!(Vec<u8>, |v: Vec<u8>| hex_prefixed(&v));
    } else if t == Type::BOOL_ARRAY {
        arr!(bool, |v: &bool| v.to_string());
    } else if t == Type::INT2_ARRAY {
        arr!(i16, |v: &i16| v.to_string());
    } else if t == Type::INT4_ARRAY {
        arr!(i32, |v: &i32| v.to_string());
    } else if t == Type::INT8_ARRAY {
        arr!(i64, |v: &i64| v.to_string());
    } else if t == Type::FLOAT8_ARRAY {
        arr!(f64, |v: &f64| v.to_string());
    } else if t == Type::TEXT_ARRAY || t == Type::VARCHAR_ARRAY || t == Type::NAME_ARRAY {
        arr!(String, |v: &String| v.clone());
    }

    raw_fallback(row, i)
}

/// Column metadata + the frontend render hint, derived from the column type.
pub fn column_meta(
    name: &str,
    ty: &Type,
    geom: Option<u32>,
    geog: Option<u32>,
) -> ColumnMeta {
    let oid = ty.oid();
    let is_geom = Some(oid) == geom;
    let is_geog = Some(oid) == geog;

    let render_hint = if is_geom || is_geog {
        RenderHint::Geometry { srid: None }
    } else if *ty == Type::JSON || *ty == Type::JSONB {
        RenderHint::Json
    } else if *ty == Type::BYTEA {
        RenderHint::Bytea
    } else {
        RenderHint::Text
    };

    let type_name = if is_geom {
        "geometry".to_string()
    } else if is_geog {
        "geography".to_string()
    } else {
        ty.name().to_string()
    };

    ColumnMeta {
        name: name.to_string(),
        type_name,
        type_oid: oid,
        render_hint,
    }
}
