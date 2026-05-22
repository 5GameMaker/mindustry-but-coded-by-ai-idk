//! Mirrors upstream `mindustry.logic.LVar`.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum LVarValue {
    Number(f64),
    Object(Option<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LVar {
    pub name: String,
    pub id: i32,
    pub is_obj: bool,
    pub constant: bool,
    pub objval: Option<String>,
    pub numval: f64,
    pub sync_time: i64,
}

impl LVar {
    pub fn new(name: impl Into<String>) -> Self {
        Self::with_id(name, -1)
    }

    pub fn with_id(name: impl Into<String>, id: i32) -> Self {
        Self::with_id_constant(name, id, false)
    }

    pub fn with_id_constant(name: impl Into<String>, id: i32, constant: bool) -> Self {
        Self {
            name: name.into(),
            id,
            is_obj: false,
            constant,
            objval: None,
            numval: 0.0,
            sync_time: 0,
        }
    }

    pub const fn invalid(value: f64) -> bool {
        value.is_nan() || value.is_infinite()
    }

    pub fn obj(&self) -> Option<&str> {
        self.is_obj.then_some(self.objval.as_deref()).flatten()
    }

    pub fn bool(&self) -> bool {
        if self.is_obj {
            self.objval.is_some()
        } else {
            self.numval.abs() >= 0.00001
        }
    }

    pub fn num(&self) -> f64 {
        if self.is_obj {
            self.objval.is_some() as u8 as f64
        } else if Self::invalid(self.numval) {
            0.0
        } else {
            self.numval
        }
    }

    pub fn num_or_nan(&self) -> f64 {
        if self.is_obj {
            if self.objval.is_some() {
                1.0
            } else {
                f64::NAN
            }
        } else if Self::invalid(self.numval) {
            0.0
        } else {
            self.numval
        }
    }

    pub fn numf(&self) -> f32 {
        self.num() as f32
    }

    pub fn numf_or_nan(&self) -> f32 {
        self.num_or_nan() as f32
    }

    pub fn numi(&self) -> i32 {
        self.num() as i32
    }

    pub fn set_bool(&mut self, value: bool) {
        self.set_num(if value { 1.0 } else { 0.0 });
    }

    pub fn set_num(&mut self, value: f64) {
        if self.constant {
            return;
        }
        if Self::invalid(value) {
            self.objval = None;
            self.is_obj = true;
        } else {
            self.numval = value;
            self.objval = None;
            self.is_obj = false;
        }
    }

    pub fn set_obj(&mut self, value: Option<String>) {
        if self.constant {
            return;
        }
        self.objval = value;
        self.is_obj = true;
    }

    pub fn set_const_obj(&mut self, value: Option<String>) {
        self.objval = value;
        self.is_obj = true;
    }

    pub fn set_from(&mut self, other: &LVar) {
        self.is_obj = other.is_obj;
        if self.is_obj {
            self.objval = other.objval.clone();
        } else {
            self.numval = if Self::invalid(other.numval) {
                0.0
            } else {
                other.numval
            };
        }
    }

    pub fn value(&self) -> LVarValue {
        if self.is_obj {
            LVarValue::Object(self.objval.clone())
        } else {
            LVarValue::Number(self.numval)
        }
    }
}

impl fmt::Display for LVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_obj {
            match &self.objval {
                Some(value) => write!(f, "{}: {}", self.name, value)?,
                None => write!(f, "{}: null", self.name)?,
            }
        } else {
            write!(f, "{}: {}", self.name, self.numval)?;
        }
        if self.constant {
            f.write_str(" [const]")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{LVar, LVarValue};

    #[test]
    fn lvar_matches_java_numeric_object_and_constant_semantics() {
        assert!(LVar::invalid(f64::NAN));
        assert!(LVar::invalid(f64::INFINITY));
        assert!(!LVar::invalid(42.0));

        let mut var = LVar::with_id("value", 7);
        assert_eq!(var.id, 7);
        assert_eq!(var.num(), 0.0);
        assert!(!var.bool());
        assert_eq!(var.numi(), 0);

        var.set_num(0.000001);
        assert_eq!(var.num(), 0.000001);
        assert!(!var.bool());
        var.set_num(0.00001);
        assert!(var.bool());

        var.set_num(f64::NAN);
        assert!(var.is_obj);
        assert_eq!(var.obj(), None);
        assert_eq!(var.num(), 0.0);
        assert!(var.num_or_nan().is_nan());
        assert_eq!(var.numf(), 0.0);
        assert!(var.numf_or_nan().is_nan());

        var.set_obj(Some("core".into()));
        assert_eq!(var.obj(), Some("core"));
        assert_eq!(var.num(), 1.0);
        assert!(var.bool());
        assert_eq!(var.value(), LVarValue::Object(Some("core".into())));

        let mut constant = LVar::with_id_constant("const", 1, true);
        constant.set_num(5.0);
        assert_eq!(constant.num(), 0.0);
        constant.set_obj(Some("ignored".into()));
        assert_eq!(constant.obj(), None);
        constant.set_const_obj(Some("locked".into()));
        assert_eq!(constant.obj(), Some("locked"));
        assert_eq!(constant.to_string(), "const: locked [const]");

        let mut copy = LVar::new("copy");
        copy.set_from(&var);
        assert_eq!(copy.value(), LVarValue::Object(Some("core".into())));

        let mut numeric_source = LVar::new("numeric");
        numeric_source.numval = f64::INFINITY;
        copy.set_from(&numeric_source);
        assert_eq!(copy.value(), LVarValue::Number(0.0));
    }
}
