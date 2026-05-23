//! Side-effect-free mirror of upstream `mindustry.input.Binding`.
//!
//! Java registers all bindings through the global `KeyBind` registry, carrying
//! forward the last explicit category when later entries omit one.  This Rust
//! layer keeps that registry as plain data so desktop/mobile input ports can
//! share the same default binding table.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    A,
    D,
    S,
    W,
    MouseBack,
    MouseForward,
    ShiftLeft,
    V,
    ControlLeft,
    MouseLeft,
    MouseRight,
    LeftBracket,
    RightBracket,
    Q,
    E,
    Scroll,
    R,
    MouseMiddle,
    P,
    B,
    F,
    Z,
    X,
    T,
    G,
    H,
    Unset,
    AltLeft,
    Comma,
    Period,
    Left,
    Right,
    Up,
    Down,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Num0,
    Back,
    Escape,
    F11,
    Space,
    M,
    J,
    N,
    F1,
    C,
    F12,
    F5,
    F6,
    Tab,
    Enter,
    F8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyBindingInput {
    Key(KeyCode),
    AxisPair { min: KeyCode, max: KeyCode },
    AxisSingle(KeyCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyBindingSpec {
    pub name: &'static str,
    pub input: KeyBindingInput,
    pub category: &'static str,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Binding;

impl Binding {
    pub fn init(android: bool) -> Vec<KeyBindingSpec> {
        Self::defaults(android)
    }

    pub fn defaults(android: bool) -> Vec<KeyBindingSpec> {
        let mut bindings = Vec::new();
        let mut category = "general";

        push_with_category(
            &mut bindings,
            &mut category,
            "move_x",
            KeyBindingInput::AxisPair {
                min: KeyCode::A,
                max: KeyCode::D,
            },
            "general",
        );
        push(
            &mut bindings,
            &mut category,
            "move_y",
            KeyBindingInput::AxisPair {
                min: KeyCode::S,
                max: KeyCode::W,
            },
        );
        push(
            &mut bindings,
            &mut category,
            "mouse_move",
            KeyBindingInput::Key(KeyCode::MouseBack),
        );
        push(
            &mut bindings,
            &mut category,
            "pan",
            KeyBindingInput::Key(KeyCode::MouseForward),
        );
        push(
            &mut bindings,
            &mut category,
            "boost",
            KeyBindingInput::Key(KeyCode::ShiftLeft),
        );
        push(
            &mut bindings,
            &mut category,
            "respawn",
            KeyBindingInput::Key(KeyCode::V),
        );
        push(
            &mut bindings,
            &mut category,
            "control",
            KeyBindingInput::Key(KeyCode::ControlLeft),
        );
        push(
            &mut bindings,
            &mut category,
            "select",
            KeyBindingInput::Key(KeyCode::MouseLeft),
        );
        push(
            &mut bindings,
            &mut category,
            "deselect",
            KeyBindingInput::Key(KeyCode::MouseRight),
        );
        push(
            &mut bindings,
            &mut category,
            "break_block",
            KeyBindingInput::Key(KeyCode::MouseRight),
        );
        push(
            &mut bindings,
            &mut category,
            "pickupCargo",
            KeyBindingInput::Key(KeyCode::LeftBracket),
        );
        push(
            &mut bindings,
            &mut category,
            "dropCargo",
            KeyBindingInput::Key(KeyCode::RightBracket),
        );
        push(
            &mut bindings,
            &mut category,
            "clear_building",
            KeyBindingInput::Key(KeyCode::Q),
        );
        push(
            &mut bindings,
            &mut category,
            "pause_building",
            KeyBindingInput::Key(KeyCode::E),
        );
        push(
            &mut bindings,
            &mut category,
            "rotate",
            KeyBindingInput::AxisSingle(KeyCode::Scroll),
        );
        push(
            &mut bindings,
            &mut category,
            "rotateplaced",
            KeyBindingInput::Key(KeyCode::R),
        );
        push(
            &mut bindings,
            &mut category,
            "diagonal_placement",
            KeyBindingInput::Key(KeyCode::ControlLeft),
        );
        push(
            &mut bindings,
            &mut category,
            "pick",
            KeyBindingInput::Key(KeyCode::MouseMiddle),
        );
        push(
            &mut bindings,
            &mut category,
            "ping",
            KeyBindingInput::Key(KeyCode::P),
        );
        push(
            &mut bindings,
            &mut category,
            "rebuild_select",
            KeyBindingInput::Key(KeyCode::B),
        );
        push(
            &mut bindings,
            &mut category,
            "schematic_select",
            KeyBindingInput::Key(KeyCode::F),
        );
        push(
            &mut bindings,
            &mut category,
            "schematic_flip_x",
            KeyBindingInput::Key(KeyCode::Z),
        );
        push(
            &mut bindings,
            &mut category,
            "schematic_flip_y",
            KeyBindingInput::Key(KeyCode::X),
        );
        push(
            &mut bindings,
            &mut category,
            "schematic_menu",
            KeyBindingInput::Key(KeyCode::T),
        );

        push_with_category(
            &mut bindings,
            &mut category,
            "command_mode",
            KeyBindingInput::Key(KeyCode::ShiftLeft),
            "command",
        );
        push(
            &mut bindings,
            &mut category,
            "command_queue",
            KeyBindingInput::Key(KeyCode::MouseMiddle),
        );
        push(
            &mut bindings,
            &mut category,
            "create_control_group",
            KeyBindingInput::Key(KeyCode::ControlLeft),
        );
        push(
            &mut bindings,
            &mut category,
            "select_all_units",
            KeyBindingInput::Key(KeyCode::G),
        );
        push(
            &mut bindings,
            &mut category,
            "select_all_unit_factories",
            KeyBindingInput::Key(KeyCode::H),
        );
        push(
            &mut bindings,
            &mut category,
            "select_all_unit_transport",
            KeyBindingInput::Key(KeyCode::Unset),
        );
        push(
            &mut bindings,
            &mut category,
            "select_across_screen",
            KeyBindingInput::Key(KeyCode::AltLeft),
        );
        push(
            &mut bindings,
            &mut category,
            "cancel_orders",
            KeyBindingInput::Key(KeyCode::Unset),
        );
        push(
            &mut bindings,
            &mut category,
            "unit_stance_shoot",
            KeyBindingInput::Key(KeyCode::Unset),
        );
        push(
            &mut bindings,
            &mut category,
            "unit_stance_hold_fire",
            KeyBindingInput::Key(KeyCode::Unset),
        );
        push(
            &mut bindings,
            &mut category,
            "unit_stance_pursue_target",
            KeyBindingInput::Key(KeyCode::Unset),
        );
        push(
            &mut bindings,
            &mut category,
            "unit_stance_patrol",
            KeyBindingInput::Key(KeyCode::Unset),
        );
        push(
            &mut bindings,
            &mut category,
            "unit_stance_ram",
            KeyBindingInput::Key(KeyCode::Unset),
        );
        push(
            &mut bindings,
            &mut category,
            "unit_stance_boost",
            KeyBindingInput::Key(KeyCode::Unset),
        );
        push(
            &mut bindings,
            &mut category,
            "unit_stance_hold_position",
            KeyBindingInput::Key(KeyCode::Unset),
        );
        push(
            &mut bindings,
            &mut category,
            "unit_command_move",
            KeyBindingInput::Key(KeyCode::Unset),
        );
        push(
            &mut bindings,
            &mut category,
            "unit_command_repair",
            KeyBindingInput::Key(KeyCode::Unset),
        );
        push(
            &mut bindings,
            &mut category,
            "unit_command_rebuild",
            KeyBindingInput::Key(KeyCode::Unset),
        );
        push(
            &mut bindings,
            &mut category,
            "unit_command_assist",
            KeyBindingInput::Key(KeyCode::Unset),
        );
        push(
            &mut bindings,
            &mut category,
            "unit_command_mine",
            KeyBindingInput::Key(KeyCode::Unset),
        );
        push(
            &mut bindings,
            &mut category,
            "unit_command_boost",
            KeyBindingInput::Key(KeyCode::Unset),
        );
        push(
            &mut bindings,
            &mut category,
            "unit_command_enter_payload",
            KeyBindingInput::Key(KeyCode::Unset),
        );
        push(
            &mut bindings,
            &mut category,
            "unit_command_load_units",
            KeyBindingInput::Key(KeyCode::Unset),
        );
        push(
            &mut bindings,
            &mut category,
            "unit_command_load_blocks",
            KeyBindingInput::Key(KeyCode::Unset),
        );
        push(
            &mut bindings,
            &mut category,
            "unit_command_unload_payload",
            KeyBindingInput::Key(KeyCode::Unset),
        );
        push(
            &mut bindings,
            &mut category,
            "unit_command_loop_payload",
            KeyBindingInput::Key(KeyCode::Unset),
        );

        push_with_category(
            &mut bindings,
            &mut category,
            "category_prev",
            KeyBindingInput::Key(KeyCode::Comma),
            "blocks",
        );
        push(
            &mut bindings,
            &mut category,
            "category_next",
            KeyBindingInput::Key(KeyCode::Period),
        );
        push(
            &mut bindings,
            &mut category,
            "block_select_left",
            KeyBindingInput::Key(KeyCode::Left),
        );
        push(
            &mut bindings,
            &mut category,
            "block_select_right",
            KeyBindingInput::Key(KeyCode::Right),
        );
        push(
            &mut bindings,
            &mut category,
            "block_select_up",
            KeyBindingInput::Key(KeyCode::Up),
        );
        push(
            &mut bindings,
            &mut category,
            "block_select_down",
            KeyBindingInput::Key(KeyCode::Down),
        );
        push(
            &mut bindings,
            &mut category,
            "block_select_01",
            KeyBindingInput::Key(KeyCode::Num1),
        );
        push(
            &mut bindings,
            &mut category,
            "block_select_02",
            KeyBindingInput::Key(KeyCode::Num2),
        );
        push(
            &mut bindings,
            &mut category,
            "block_select_03",
            KeyBindingInput::Key(KeyCode::Num3),
        );
        push(
            &mut bindings,
            &mut category,
            "block_select_04",
            KeyBindingInput::Key(KeyCode::Num4),
        );
        push(
            &mut bindings,
            &mut category,
            "block_select_05",
            KeyBindingInput::Key(KeyCode::Num5),
        );
        push(
            &mut bindings,
            &mut category,
            "block_select_06",
            KeyBindingInput::Key(KeyCode::Num6),
        );
        push(
            &mut bindings,
            &mut category,
            "block_select_07",
            KeyBindingInput::Key(KeyCode::Num7),
        );
        push(
            &mut bindings,
            &mut category,
            "block_select_08",
            KeyBindingInput::Key(KeyCode::Num8),
        );
        push(
            &mut bindings,
            &mut category,
            "block_select_09",
            KeyBindingInput::Key(KeyCode::Num9),
        );
        push(
            &mut bindings,
            &mut category,
            "block_select_10",
            KeyBindingInput::Key(KeyCode::Num0),
        );

        push_with_category(
            &mut bindings,
            &mut category,
            "zoom",
            KeyBindingInput::AxisSingle(KeyCode::Scroll),
            "view",
        );
        push(
            &mut bindings,
            &mut category,
            "detach_camera",
            KeyBindingInput::Key(KeyCode::Unset),
        );
        push(
            &mut bindings,
            &mut category,
            "menu",
            KeyBindingInput::Key(if android {
                KeyCode::Back
            } else {
                KeyCode::Escape
            }),
        );
        push(
            &mut bindings,
            &mut category,
            "fullscreen",
            KeyBindingInput::Key(KeyCode::F11),
        );
        push(
            &mut bindings,
            &mut category,
            "pause",
            KeyBindingInput::Key(KeyCode::Space),
        );
        push(
            &mut bindings,
            &mut category,
            "skip_wave",
            KeyBindingInput::Key(KeyCode::Unset),
        );
        push(
            &mut bindings,
            &mut category,
            "minimap",
            KeyBindingInput::Key(KeyCode::M),
        );
        push(
            &mut bindings,
            &mut category,
            "research",
            KeyBindingInput::Key(KeyCode::J),
        );
        push(
            &mut bindings,
            &mut category,
            "planet_map",
            KeyBindingInput::Key(KeyCode::N),
        );
        push(
            &mut bindings,
            &mut category,
            "block_info",
            KeyBindingInput::Key(KeyCode::F1),
        );
        push(
            &mut bindings,
            &mut category,
            "toggle_menus",
            KeyBindingInput::Key(KeyCode::C),
        );
        push(
            &mut bindings,
            &mut category,
            "screenshot",
            KeyBindingInput::Key(KeyCode::F12),
        );
        push(
            &mut bindings,
            &mut category,
            "toggle_power_lines",
            KeyBindingInput::Key(KeyCode::F5),
        );
        push(
            &mut bindings,
            &mut category,
            "toggle_block_status",
            KeyBindingInput::Key(KeyCode::F6),
        );

        push_with_category(
            &mut bindings,
            &mut category,
            "player_list",
            KeyBindingInput::Key(KeyCode::Tab),
            "multiplayer",
        );
        push(
            &mut bindings,
            &mut category,
            "chat",
            KeyBindingInput::Key(KeyCode::Enter),
        );
        push(
            &mut bindings,
            &mut category,
            "chat_history_prev",
            KeyBindingInput::Key(KeyCode::Up),
        );
        push(
            &mut bindings,
            &mut category,
            "chat_history_next",
            KeyBindingInput::Key(KeyCode::Down),
        );
        push(
            &mut bindings,
            &mut category,
            "chat_scroll",
            KeyBindingInput::AxisSingle(KeyCode::Scroll),
        );
        push(
            &mut bindings,
            &mut category,
            "chat_mode",
            KeyBindingInput::Key(KeyCode::Tab),
        );
        push(
            &mut bindings,
            &mut category,
            "console",
            KeyBindingInput::Key(KeyCode::F8),
        );
        push(
            &mut bindings,
            &mut category,
            "debug_hitboxes",
            KeyBindingInput::Key(KeyCode::Unset),
        );

        bindings
    }

    pub fn find(name: &str, android: bool) -> Option<KeyBindingSpec> {
        Self::defaults(android)
            .into_iter()
            .find(|binding| binding.name == name)
    }
}

fn push(
    bindings: &mut Vec<KeyBindingSpec>,
    category: &mut &'static str,
    name: &'static str,
    input: KeyBindingInput,
) {
    bindings.push(KeyBindingSpec {
        name,
        input,
        category: *category,
    });
}

fn push_with_category(
    bindings: &mut Vec<KeyBindingSpec>,
    current_category: &mut &'static str,
    name: &'static str,
    input: KeyBindingInput,
    category: &'static str,
) {
    *current_category = category;
    push(bindings, current_category, name, input);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_keep_java_category_sections() {
        let bindings = Binding::defaults(false);

        assert_eq!(Binding::find("move_x", false).unwrap().category, "general");
        assert_eq!(Binding::find("move_y", false).unwrap().category, "general");
        assert_eq!(
            Binding::find("command_mode", false).unwrap().category,
            "command"
        );
        assert_eq!(
            Binding::find("unit_command_move", false).unwrap().category,
            "command"
        );
        assert_eq!(
            Binding::find("category_prev", false).unwrap().category,
            "blocks"
        );
        assert_eq!(Binding::find("zoom", false).unwrap().category, "view");
        assert_eq!(
            Binding::find("player_list", false).unwrap().category,
            "multiplayer"
        );
        assert_eq!(
            Binding::find("debug_hitboxes", false).unwrap().category,
            "multiplayer"
        );
        assert_eq!(bindings.len(), 88);
    }

    #[test]
    fn menu_binding_switches_between_escape_and_android_back() {
        assert_eq!(
            Binding::find("menu", false).unwrap().input,
            KeyBindingInput::Key(KeyCode::Escape)
        );
        assert_eq!(
            Binding::find("menu", true).unwrap().input,
            KeyBindingInput::Key(KeyCode::Back)
        );
    }

    #[test]
    fn axis_bindings_match_java_defaults() {
        assert_eq!(
            Binding::find("move_x", false).unwrap().input,
            KeyBindingInput::AxisPair {
                min: KeyCode::A,
                max: KeyCode::D,
            }
        );
        assert_eq!(
            Binding::find("rotate", false).unwrap().input,
            KeyBindingInput::AxisSingle(KeyCode::Scroll)
        );
        assert_eq!(
            Binding::find("zoom", false).unwrap().input,
            KeyBindingInput::AxisSingle(KeyCode::Scroll)
        );
        assert_eq!(
            Binding::find("chat_scroll", false).unwrap().input,
            KeyBindingInput::AxisSingle(KeyCode::Scroll)
        );
    }

    #[test]
    fn representative_shortcuts_match_upstream_names() {
        assert_eq!(
            Binding::find("schematic_flip_x", false).unwrap().input,
            KeyBindingInput::Key(KeyCode::Z)
        );
        assert_eq!(
            Binding::find("toggle_power_lines", false).unwrap().input,
            KeyBindingInput::Key(KeyCode::F5)
        );
        assert_eq!(
            Binding::find("unit_command_loop_payload", false)
                .unwrap()
                .input,
            KeyBindingInput::Key(KeyCode::Unset)
        );
        assert!(Binding::find("missing", false).is_none());
    }
}
