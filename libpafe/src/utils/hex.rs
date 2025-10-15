//! Hexadecimal helpers used for debugging and display purposes.
//!
//! These helpers are intentionally small and avoid external dependencies; they
//! support both compact (no-separator) and spaced output, and provide a simple
//! parser that accepts optional whitespace.

/// Convert a byte slice to a lowercase hex string without separators.
///
/// Example: `&[0xde, 0xad]` -> `"dead"`
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        // write! never fails writing to a String
        let _ = write!(&mut s, "{:02x}", b);
    }
    s
}

/// Convert a byte slice to a lowercase hex string with a single space between
/// each byte.
///
/// Example: `&[0xde, 0xad]` -> `"de ad"`
pub fn bytes_to_hex_spaced(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 3);
    for (i, b) in bytes.iter().enumerate() {
        if i != 0 {
            s.push(' ');
        }
        use std::fmt::Write;
        let _ = write!(&mut s, "{:02x}", b);
    }
    s
}

/// Parse a hex string into bytes.
///
/// Accepts strings with or without ASCII whitespace. Returns an error message
/// string on parse failure.
pub fn parse_hex(s: &str) -> Result<Vec<u8>, String> {
    // Remove ASCII whitespace
    let mut cleaned = String::with_capacity(s.len());
    for c in s.chars() {
        if !c.is_whitespace() {
            cleaned.push(c);
        }
    }

    if cleaned.len() % 2 != 0 {
        return Err("hex string has odd length".to_string());
    }

    let mut out = Vec::with_capacity(cleaned.len() / 2);
    let mut i = 0usize;
    while i < cleaned.len() {
        let hi = cleaned.as_bytes()[i] as char;
        let lo = cleaned.as_bytes()[i + 1] as char;
        let pair = format!("{}{}", hi, lo);
        let byte = u8::from_str_radix(&pair, 16)
            .map_err(|e| format!("invalid hex pair '{}': {}", pair, e))?;
        out.push(byte);
        i += 2;
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_to_hex_basic() {
        assert_eq!(bytes_to_hex(&[0xde, 0xad, 0xbe, 0xef]), "deadbeef");
    }

    #[test]
    fn bytes_to_hex_spaced_basic() {
        assert_eq!(bytes_to_hex_spaced(&[0xde, 0xab]), "de ab");
    }

    #[test]
    fn parse_hex_basic() {
        assert_eq!(parse_hex("deadbeef").unwrap(), vec![0xde, 0xad, 0xbe, 0xef]);
        assert_eq!(
            parse_hex("de ad be ef").unwrap(),
            vec![0xde, 0xad, 0xbe, 0xef]
        );
    }

    #[test]
    fn parse_hex_err_cases() {
        assert!(parse_hex("abc").is_err());
        assert!(parse_hex("zz").is_err());
    }
}
