//! Identifier / literal quoting for the (unavoidable) DDL we build by hand.
//! Roles, schema ops and grants cannot be parameterized, so every
//! user-supplied name flows through here.

/// Double-quote an identifier, escaping embedded quotes.
pub fn qident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

/// Single-quote a string literal, escaping embedded quotes.
pub fn qliteral(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

/// Reject anything that isn't a plain unqualified identifier. Used as a
/// belt-and-braces guard before quoting role/schema names.
pub fn ensure_simple_ident(s: &str) -> Result<(), crate::error::AppError> {
    if s.is_empty() || s.len() > 63 {
        return Err(crate::error::AppError::msg("invalid identifier length"));
    }
    if s.chars().any(|c| c == '\0' || c == '"') {
        return Err(crate::error::AppError::msg(
            "identifier contains illegal characters",
        ));
    }
    Ok(())
}
