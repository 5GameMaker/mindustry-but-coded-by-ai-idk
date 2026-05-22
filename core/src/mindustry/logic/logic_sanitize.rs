//! Mirrors upstream `mindustry.logic.LStatement.sanitize`.

pub fn sanitize_logic_value(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    } else if value.chars().count() == 1 {
        let c = value.chars().next().unwrap();
        if c == '"' || c == ';' || c == ' ' {
            return "invalid".to_string();
        }
    } else {
        let mut res = String::with_capacity(value.len());
        if value.starts_with('"') && value.ends_with('"') {
            res.push('"');
            for c in value[1..value.len() - 1].chars() {
                res.push(if c == '"' { '\'' } else { c });
            }
            res.push('"');
        } else {
            for c in value.chars() {
                res.push(match c {
                    ';' => 's',
                    '"' => '\'',
                    ' ' => '_',
                    _ => c,
                });
            }
        }
        return res;
    }

    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::sanitize_logic_value;

    #[test]
    fn statement_sanitize_matches_java_helper() {
        assert_eq!(sanitize_logic_value(""), "");
        assert_eq!(sanitize_logic_value("\""), "invalid");
        assert_eq!(sanitize_logic_value(";"), "invalid");
        assert_eq!(sanitize_logic_value(" "), "invalid");
        assert_eq!(sanitize_logic_value("a b;c\"d"), "a_bsc'd");
        assert_eq!(sanitize_logic_value("\"a\"b;c\""), "\"a'b;c\"");
        assert_eq!(sanitize_logic_value("alpha"), "alpha");
    }
}
