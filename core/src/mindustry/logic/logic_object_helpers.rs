use super::{LVar, LOGIC_TILE_SIZE};

pub fn logic_unwrap_object_name(name: &str) -> &str {
    name.strip_prefix('@').unwrap_or(name)
}

pub fn logic_object_name(name: &str) -> String {
    if name.starts_with('@') {
        name.to_string()
    } else {
        format!("@{name}")
    }
}

pub fn logic_unconv(coord: f32) -> f32 {
    coord * LOGIC_TILE_SIZE
}

pub fn logic_conv(coord: f32) -> f32 {
    coord / LOGIC_TILE_SIZE
}

pub fn logic_team_from_var(var: &LVar) -> Option<u8> {
    if var.is_obj {
        var.obj().and_then(logic_team_from_name)
    } else {
        let value = var.numi();
        (0..=255).contains(&value).then_some(value as u8)
    }
}

pub fn logic_team_from_name(name: &str) -> Option<u8> {
    let name = logic_unwrap_object_name(name);
    match name {
        "derelict" => Some(0),
        "sharded" => Some(1),
        "crux" => Some(2),
        "malis" => Some(3),
        "green" => Some(4),
        "blue" => Some(5),
        "neoplastic" => Some(6),
        _ => name.parse::<u8>().ok(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn num_var(value: f64) -> LVar {
        let mut var = LVar::new("team");
        var.set_num(value);
        var
    }

    fn obj_var(value: &str) -> LVar {
        let mut var = LVar::new("team");
        var.set_obj(Some(value.into()));
        var
    }

    #[test]
    fn object_coordinate_and_team_helpers_match_java_logic_names() {
        assert_eq!(logic_unwrap_object_name("@copper"), "copper");
        assert_eq!(logic_unwrap_object_name("lead"), "lead");
        assert_eq!(logic_object_name("lead"), "@lead");
        assert_eq!(logic_object_name("@lead"), "@lead");

        assert_eq!(logic_unconv(2.5), 20.0);
        assert_eq!(logic_conv(20.0), 2.5);

        assert_eq!(logic_team_from_name("@derelict"), Some(0));
        assert_eq!(logic_team_from_name("sharded"), Some(1));
        assert_eq!(logic_team_from_name("neoplastic"), Some(6));
        assert_eq!(logic_team_from_name("255"), Some(255));
        assert_eq!(logic_team_from_name("unknown"), None);

        assert_eq!(logic_team_from_var(&num_var(2.0)), Some(2));
        assert_eq!(logic_team_from_var(&num_var(-1.0)), None);
        assert_eq!(logic_team_from_var(&num_var(256.0)), None);
        assert_eq!(logic_team_from_var(&obj_var("@crux")), Some(2));
    }
}
