//! Mirrors upstream `mindustry.logic.Settable`.

use super::LAccess;

pub trait Settable {
    type Content;
    type Object;

    fn set_prop(&mut self, prop: LAccess, value: f64);
    fn set_prop_object(&mut self, prop: LAccess, value: Self::Object);
    fn set_content_prop(&mut self, content: Self::Content, value: f64);
}

#[cfg(test)]
mod tests {
    use super::Settable;
    use crate::mindustry::logic::LAccess;

    #[derive(Default)]
    struct SettableState {
        numeric: Option<(LAccess, f64)>,
        object: Option<(LAccess, String)>,
        content: Option<(String, f64)>,
    }

    impl Settable for SettableState {
        type Content = String;
        type Object = String;

        fn set_prop(&mut self, prop: LAccess, value: f64) {
            self.numeric = Some((prop, value));
        }

        fn set_prop_object(&mut self, prop: LAccess, value: Self::Object) {
            self.object = Some((prop, value));
        }

        fn set_content_prop(&mut self, content: Self::Content, value: f64) {
            self.content = Some((content, value));
        }
    }

    #[test]
    fn settable_trait_exposes_numeric_object_and_content_contract() {
        let mut state = SettableState::default();
        state.set_prop(LAccess::Enabled, 1.0);
        state.set_prop_object(LAccess::Config, "logic".into());
        state.set_content_prop("copper".into(), 4.0);

        assert_eq!(state.numeric, Some((LAccess::Enabled, 1.0)));
        assert_eq!(state.object, Some((LAccess::Config, "logic".into())));
        assert_eq!(state.content, Some(("copper".into(), 4.0)));
    }
}
