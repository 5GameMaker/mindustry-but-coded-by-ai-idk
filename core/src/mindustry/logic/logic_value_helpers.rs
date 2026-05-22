use super::{ConditionOp, ConditionValue, LVar, LVarValue};

pub fn logic_var_strict_equal(a: &LVar, b: &LVar) -> bool {
    a.is_obj == b.is_obj
        && if a.is_obj {
            a.objval == b.objval
        } else {
            a.numval == b.numval
        }
}

pub fn condition_op_test_vars(op: ConditionOp, a: &LVar, b: &LVar) -> bool {
    if a.is_obj {
        if b.is_obj {
            let left = a.objval.as_deref().unwrap_or("");
            let right = b.objval.as_deref().unwrap_or("");
            op.test_values(ConditionValue::Object(left), ConditionValue::Object(right))
        } else {
            op.test_values(
                ConditionValue::Object(a.objval.as_deref().unwrap_or("")),
                ConditionValue::Number(b.num()),
            )
        }
    } else if b.is_obj {
        op.test_values(
            ConditionValue::Number(a.num()),
            ConditionValue::Object(b.objval.as_deref().unwrap_or("")),
        )
    } else {
        op.test_values(
            ConditionValue::Number(a.num()),
            ConditionValue::Number(b.num()),
        )
    }
}

pub fn print_logic_value(value: &LVar) -> String {
    if value.is_obj {
        value.objval.clone().unwrap_or_else(|| "null".into())
    } else if (value.numval - value.numval.round()).abs() < 0.00001 {
        (value.numval.round() as i64).to_string()
    } else {
        value.numval.to_string()
    }
}

pub fn first_logic_placeholder(buffer: &str) -> Option<(usize, u8)> {
    let bytes = buffer.as_bytes();
    let mut best: Option<(usize, u8)> = None;
    for index in 0..bytes.len().saturating_sub(2) {
        if bytes[index] == b'{' && bytes[index + 2] == b'}' {
            let digit = bytes[index + 1];
            if digit.is_ascii_digit() {
                let number = digit - b'0';
                if best.is_none_or(|(_, best_number)| number < best_number) {
                    best = Some((index, number));
                }
            }
        }
    }
    best
}

pub fn logic_utf16_len(value: &str) -> usize {
    value.encode_utf16().count()
}

pub fn logic_utf16_char_code_at(value: &str, index: i32) -> Option<u16> {
    if index < 0 {
        return None;
    }
    value.encode_utf16().nth(index as usize)
}

pub fn set_lvar_value(target: &mut LVar, value: &LVarValue) {
    match value {
        LVarValue::Number(value) => target.set_num(*value),
        LVarValue::Object(value) => target.set_obj(value.clone()),
    }
}

pub fn lvar_value(value: &LVar) -> LVarValue {
    value.value()
}

pub fn read_logic_text(value: &str, position: &LVar, output: &mut LVar) {
    if let Some(code) = logic_utf16_char_code_at(value, position.numi()) {
        output.set_num(code as f64);
    } else {
        output.set_num(f64::NAN);
    }
}

pub fn read_logic_sequence(values: &[LVarValue], position: &LVar, output: &mut LVar) {
    let address = position.numi();
    if address < 0 || address as usize >= values.len() {
        output.set_obj(None);
    } else {
        set_lvar_value(output, &values[address as usize]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn num_var(name: &str, value: f64) -> LVar {
        let mut var = LVar::new(name);
        var.set_num(value);
        var
    }

    fn obj_var(name: &str, value: Option<&str>) -> LVar {
        let mut var = LVar::new(name);
        var.set_obj(value.map(str::to_string));
        var
    }

    #[test]
    fn logic_value_helpers_follow_java_lvar_and_text_semantics() {
        let mut number = num_var("number", 42.0);
        let same_number = num_var("same", 42.0);
        let object = obj_var("object", Some("copper"));
        let null_object = obj_var("null", None);

        assert!(logic_var_strict_equal(&number, &same_number));
        assert!(!logic_var_strict_equal(&number, &object));
        assert_eq!(print_logic_value(&number), "42");
        number.set_num(1.5);
        assert_eq!(print_logic_value(&number), "1.5");
        assert_eq!(print_logic_value(&object), "copper");
        assert_eq!(print_logic_value(&null_object), "null");

        assert!(condition_op_test_vars(
            ConditionOp::Equal,
            &object,
            &obj_var("other", Some("copper"))
        ));
        assert!(!condition_op_test_vars(
            ConditionOp::StrictEqual,
            &object,
            &number
        ));

        assert_eq!(first_logic_placeholder("a {2} b {0} c {1}"), Some((8, 0)));
        assert_eq!(logic_utf16_len("a😀"), 3);
        assert_eq!(logic_utf16_char_code_at("A", 0), Some(65));
        assert_eq!(logic_utf16_char_code_at("A", -1), None);

        let mut output = LVar::new("out");
        read_logic_text("AZ", &num_var("pos", 1.0), &mut output);
        assert_eq!(output.num(), 90.0);
        read_logic_text("AZ", &num_var("pos", 2.0), &mut output);
        assert!(output.num_or_nan().is_nan());

        read_logic_sequence(
            &[
                LVarValue::Number(7.0),
                LVarValue::Object(Some("lead".into())),
            ],
            &num_var("pos", 1.0),
            &mut output,
        );
        assert_eq!(lvar_value(&output), LVarValue::Object(Some("lead".into())));
        read_logic_sequence(&[], &num_var("pos", 0.0), &mut output);
        assert_eq!(lvar_value(&output), LVarValue::Object(None));
    }
}
