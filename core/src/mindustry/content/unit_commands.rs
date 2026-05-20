use crate::mindustry::ai::unit_command::UnitCommand;

pub fn load() -> Vec<UnitCommand> {
    let mut next_id = 0;
    let mut make = |name: &str, icon: &str, keybind: &str, controller: Option<&str>| {
        let command = UnitCommand::new(next_id, name, icon, Some(keybind), controller);
        next_id += 1;
        command
    };

    let mut move_command = make("move", "right", "unitCommandMove", None);
    move_command.draw_target = true;
    move_command.reset_target = false;

    let repair = make(
        "repair",
        "modeSurvival",
        "unitCommandRepair",
        Some("RepairAI"),
    );
    let rebuild = make("rebuild", "hammer", "unitCommandRebuild", Some("BuilderAI"));
    let assist = make(
        "assist",
        "players",
        "unitCommandAssist",
        Some("BuilderAI:assist"),
    );

    let mut mine = make("mine", "production", "unitCommandMine", Some("MinerAI"));
    mine.refresh_on_select = true;

    let mut enter_payload = make("enterPayload", "downOpen", "unitCommandEnterPayload", None);
    enter_payload.switch_to_move = false;
    enter_payload.draw_target = true;
    enter_payload.reset_target = false;
    enter_payload.snap_to_building = true;

    let mut load_units = make("loadUnits", "upload", "unitCommandLoadUnits", None);
    load_units.switch_to_move = false;
    load_units.draw_target = true;
    load_units.reset_target = false;

    let mut load_blocks = make("loadBlocks", "up", "unitCommandLoadBlocks", None);
    load_blocks.switch_to_move = false;
    load_blocks.draw_target = true;
    load_blocks.reset_target = false;
    load_blocks.exact_arrival = true;

    let mut unload_payload = make(
        "unloadPayload",
        "download",
        "unitCommandUnloadPayload",
        None,
    );
    unload_payload.switch_to_move = false;
    unload_payload.draw_target = true;
    unload_payload.reset_target = false;

    let mut loop_payload = make("loopPayload", "resize", "unitCommandLoopPayload", None);
    loop_payload.switch_to_move = false;
    loop_payload.draw_target = true;
    loop_payload.reset_target = false;

    vec![
        move_command,
        repair,
        rebuild,
        assist,
        mine,
        enter_payload,
        load_units,
        load_blocks,
        unload_payload,
        loop_payload,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::ctype::{Content, ContentType};

    #[test]
    fn vanilla_unit_commands_keep_upstream_load_all_order_and_ids() {
        let commands = load();
        let names: Vec<_> = commands.iter().map(UnitCommand::name).collect();

        assert_eq!(
            names,
            vec![
                "move",
                "repair",
                "rebuild",
                "assist",
                "mine",
                "enterPayload",
                "loadUnits",
                "loadBlocks",
                "unloadPayload",
                "loopPayload",
            ]
        );

        for (idx, command) in commands.iter().enumerate() {
            assert_eq!(command.id(), idx as i16);
            assert_eq!(command.content_type(), ContentType::UnitCommand);
        }
    }

    #[test]
    fn vanilla_unit_command_fields_match_upstream_subset() {
        let commands = load();
        let command = |name: &str| {
            commands
                .iter()
                .find(|command| command.name() == name)
                .unwrap()
        };

        let move_command = command("move");
        assert_eq!(move_command.icon, "right");
        assert_eq!(move_command.keybind.as_deref(), Some("unitCommandMove"));
        assert!(move_command.draw_target);
        assert!(!move_command.reset_target);

        let mine = command("mine");
        assert_eq!(mine.controller.as_deref(), Some("MinerAI"));
        assert!(mine.refresh_on_select);

        let enter_payload = command("enterPayload");
        assert!(!enter_payload.switch_to_move);
        assert!(enter_payload.snap_to_building);

        let load_blocks = command("loadBlocks");
        assert!(load_blocks.exact_arrival);
    }
}
