use crate::mindustry::{ai::unit_stance::UnitStance, r#type::Item};

pub fn load(items: &[Item]) -> Vec<UnitStance> {
    let mut next_id = 0;
    let mut make = |name: &str, icon: &str, keybind: Option<&str>, toggle: bool| {
        let stance = UnitStance::new(next_id, name, icon, keybind, toggle);
        next_id += 1;
        stance
    };

    let stop = make("stop", "cancel", Some("cancelOrders"), false);
    let hold_fire = make("holdfire", "none", Some("unitStanceHoldFire"), true);
    let pursue_target = make(
        "pursuetarget",
        "right",
        Some("unitStancePursueTarget"),
        true,
    );

    let mut patrol = make("patrol", "refresh", Some("unitStancePatrol"), true);
    patrol.incompatible_commands = vec!["repair".into(), "assist".into(), "rebuild".into()];
    patrol.incompatible_command_bits = command_bits(&[1, 3, 2]);

    let ram = make("ram", "rightOpen", Some("unitStanceRam"), true);

    let mut boost = make("boost", "up", Some("unitStanceBoost"), true);
    boost.incompatible_commands = vec![
        "rebuild".into(),
        "repair".into(),
        "assist".into(),
        "enterPayload".into(),
    ];
    boost.incompatible_command_bits = command_bits(&[2, 1, 3, 5]);

    let hold_position = make(
        "holdposition",
        "effect",
        Some("unitStanceHoldPosition"),
        true,
    );
    let mine_auto = make("mineauto", "settings", None, false);

    let mut stances = vec![
        stop,
        hold_fire,
        pursue_target,
        patrol,
        ram,
        boost,
        hold_position,
        mine_auto,
    ];

    for item in items {
        let mut stance = UnitStance::item(next_id, item.name());
        stance.incompatible_stance_bits = 1 << 7;
        next_id += 1;
        stances.push(stance);
    }

    stances
}

const fn command_bits(ids: &[u32]) -> u64 {
    let mut bits = 0;
    let mut i = 0;
    while i < ids.len() {
        bits |= 1 << ids[i];
        i += 1;
    }
    bits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::{
        ai::unit_stance::UnitStanceKind,
        content::items,
        ctype::{Content, ContentType},
    };

    #[test]
    fn vanilla_unit_stances_keep_upstream_static_then_item_order_and_ids() {
        let vanilla_items = items::load();
        let stances = load(&vanilla_items);
        let names: Vec<_> = stances.iter().map(UnitStance::name).collect();

        assert_eq!(
            &names[..10],
            &[
                "stop",
                "holdfire",
                "pursuetarget",
                "patrol",
                "ram",
                "boost",
                "holdposition",
                "mineauto",
                "item-scrap",
                "item-copper",
            ]
        );
        assert_eq!(stances.len(), 8 + vanilla_items.len());

        for (idx, stance) in stances.iter().enumerate() {
            assert_eq!(stance.id(), idx as i16);
            assert_eq!(stance.content_type(), ContentType::UnitStance);
        }
    }

    #[test]
    fn vanilla_unit_stance_fields_match_upstream_subset() {
        let vanilla_items = items::load();
        let stances = load(&vanilla_items);
        let stance = |name: &str| {
            stances
                .iter()
                .find(|stance| stance.name() == name)
                .unwrap_or_else(|| panic!("missing stance: {name}"))
        };

        let stop = stance("stop");
        assert_eq!(stop.icon, "cancel");
        assert_eq!(stop.keybind.as_deref(), Some("cancelOrders"));
        assert!(!stop.toggle);

        let patrol = stance("patrol");
        assert_eq!(
            patrol.incompatible_commands,
            vec!["repair", "assist", "rebuild"]
        );
        assert!(!patrol.is_compatible_with_command_id(1));
        assert!(!patrol.is_compatible_with_command_id(2));
        assert!(!patrol.is_compatible_with_command_id(3));

        let boost = stance("boost");
        assert!(!boost.is_compatible_with_command_id(5));

        let item_scrap = stance("item-scrap");
        assert_eq!(
            item_scrap.kind,
            UnitStanceKind::Item {
                item: "scrap".into()
            }
        );
        assert_eq!(item_scrap.incompatible_stance_bits, 1 << 7);
    }
}
