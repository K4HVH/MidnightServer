use super::*;

#[test]
fn parse_plain() {
    assert!(matches!(LogStyle::from_str("plain"), LogStyle::Plain));
}

#[test]
fn parse_compact() {
    assert!(matches!(LogStyle::from_str("compact"), LogStyle::Compact));
}

#[test]
fn parse_pretty() {
    assert!(matches!(LogStyle::from_str("pretty"), LogStyle::Pretty));
}

#[test]
fn parse_json() {
    assert!(matches!(LogStyle::from_str("json"), LogStyle::Json));
}

#[test]
fn parse_case_insensitive() {
    assert!(matches!(LogStyle::from_str("JSON"), LogStyle::Json));
    assert!(matches!(LogStyle::from_str("Plain"), LogStyle::Plain));
    assert!(matches!(LogStyle::from_str("COMPACT"), LogStyle::Compact));
    assert!(matches!(LogStyle::from_str("PrEtTy"), LogStyle::Pretty));
}

#[test]
fn parse_auto_debug_defaults_to_pretty() {
    let style = LogStyle::from_str("auto");
    if cfg!(debug_assertions) {
        assert!(matches!(style, LogStyle::Pretty));
    } else {
        assert!(matches!(style, LogStyle::Plain));
    }
}

#[test]
fn parse_unknown_uses_build_default() {
    let style = LogStyle::from_str("nonsense");
    if cfg!(debug_assertions) {
        assert!(matches!(style, LogStyle::Pretty));
    } else {
        assert!(matches!(style, LogStyle::Plain));
    }
}

#[test]
fn parse_empty_string_uses_build_default() {
    let style = LogStyle::from_str("");
    if cfg!(debug_assertions) {
        assert!(matches!(style, LogStyle::Pretty));
    } else {
        assert!(matches!(style, LogStyle::Plain));
    }
}
