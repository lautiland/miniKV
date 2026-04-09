// Entrada: "SET key value"
// Salida: Command { type: Set, key: "key", value: Some("value") }
// Entrada: "GET key"
// Salida: Command { type: Get, key: "key", value: None }

#[must_use]
pub fn ok() -> String {
    "OK".to_string()
}

#[must_use]
pub fn ok_value(value: &str) -> String {
    format!("OK \"{value}\"")
}

#[must_use]
pub fn error(reason: &str) -> String {
    format!("ERROR \"{reason}\"")
}

#[must_use]
pub fn number(n: usize) -> String {
    n.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ok() {
        assert_eq!(ok(), "OK");
    }
    #[test]
    fn test_ok_value() {
        assert_eq!(ok_value("hello"), "OK \"hello\"");
    }
    #[test]
    fn test_ok_value_with_spaces() {
        assert_eq!(ok_value("hello world"), "OK \"hello world\"");
    }
    #[test]
    fn test_error() {
        assert_eq!(error("NOT FOUND"), "ERROR \"NOT FOUND\"");
    }
    #[test]
    fn test_number() {
        assert_eq!(number(42), "42");
        assert_eq!(number(0), "0");
    }
}
