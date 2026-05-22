// Mirrors upstream core/src/mindustry/logic. Implemented incrementally from D:\MDT\mindustry-upstream-v157.4.

pub mod condition_op;
pub mod controllable;
pub mod cutscene_action;
pub mod fetch_type;
pub mod global_vars;
pub mod graphics_type;
pub mod l_access;
pub mod l_category;
pub mod l_locate;
pub mod l_marker_control;
pub mod l_readable;
pub mod l_unit_control;
pub mod l_var;
pub mod l_writable;
pub mod logic_assembler;
pub mod logic_building_object;
pub mod logic_canvas;
pub mod logic_controllable_object;
pub mod logic_display_command;
pub mod logic_events;
pub mod logic_executor;
pub mod logic_fx;
pub mod logic_instruction;
pub mod logic_marker_object;
pub mod logic_memory_object;
pub mod logic_object_helpers;
pub mod logic_op;
pub mod logic_parser;
pub mod logic_query_view;
pub mod logic_radar_source;
pub mod logic_rule;
pub mod logic_rules_state;
pub mod logic_runtime_object;
pub mod logic_sanitize;
pub mod logic_sense_object;
pub mod logic_statement;
pub mod logic_unit_object;
pub mod logic_value_helpers;
pub mod logic_world_object;
pub mod message_type;
pub mod query_shape;
pub mod query_type;
pub mod radar_sort;
pub mod radar_target;
pub mod ranged;
pub mod senseable;
pub mod settable;
pub mod tile_layer;

pub use condition_op::{ConditionOp, ConditionValue};
pub use controllable::Controllable;
pub use cutscene_action::CutsceneAction;
pub use fetch_type::FetchType;
pub use global_vars::{
    logic_access_from_object_name, logic_content_name_from_object_name, logic_global_value,
    logic_known_global_content_name, lookup_logic_content_name, GlobalVarEntry, GlobalVarSnapshot,
    LOGIC_CTRL_COMMAND, LOGIC_CTRL_PLAYER, LOGIC_CTRL_PROCESSOR, LOOKABLE_CONTENT,
    LOOKABLE_CONTENT_TYPES, WRITABLE_LOOKABLE_CONTENT,
};
pub use graphics_type::GraphicsType;
pub use l_access::LAccess;
pub use l_category::LCategory;
pub use l_locate::LLocate;
pub use l_marker_control::LMarkerControl;
pub use l_readable::{LReadable, LReadable as LogicReadable};
pub use l_unit_control::LUnitControl;
pub use l_var::{LVar, LVarValue};
pub use l_writable::{LWritable, LWritable as LogicWritable};
pub use logic_assembler::{
    double_bits_to_rgba, logic_color_channel_to_byte, named_logic_color_rgba, parse_logic_color,
    parse_logic_double, parse_logic_long, parse_named_logic_color, rgba_to_double_bits,
    rgba_u32_to_double_bits, unpack_double_color, LogicAssembler, LogicValue,
};
pub use logic_building_object::LogicBuildingObject;
pub use logic_canvas::{
    assign_logic_jump_heights, logic_canvas_use_rows, normalize_logic_jump_range,
    representative_logic_jumps, LogicAlign, LogicJumpPlacement, LogicJumpRange,
};
pub use logic_controllable_object::{LogicControlCall, LogicControllableObject};
pub use logic_display_command::LogicDisplayCommand;
pub use logic_events::{
    LogicBulletEvent, LogicClientDataEvent, LogicCutsceneState, LogicEffectEvent,
    LogicExplosionEvent, LogicMessageEvent, LogicMessageState, LogicSoundEvent, LogicSpawnEvent,
    LogicSyncEvent, LogicWeatherEvent, LogicWeatherState,
};
pub use logic_executor::LogicExecutor;
pub use logic_fx::{
    get_logic_effect, logic_effect_names, LogicEffectEntry, LogicEffectRegistry, LogicEffectSpec,
    LOGIC_EFFECTS,
};
pub use logic_instruction::LogicInstruction;
pub use logic_marker_object::{LogicMarkerControlEvent, LogicMarkerEvent, LogicMarkerObject};
pub use logic_memory_object::LogicMemoryObject;
pub use logic_object_helpers::{
    logic_conv, logic_object_name, logic_team_from_name, logic_team_from_var, logic_unconv,
    logic_unwrap_object_name,
};
pub use logic_op::LogicOp;
pub use logic_parser::{
    check_logic_tokens, parse_logic_statements, LogicParseError, LogicParserOutput,
    LogicStatementKind, LOGIC_PARSER_MAX_JUMPS, LOGIC_PARSER_MAX_TOKENS,
};
pub use logic_query_view::{LogicLocateResult, RadarUnitView};
pub use logic_radar_source::LogicRadarSource;
pub use logic_rule::LogicRule;
pub use logic_rules_state::{LogicRulesState, LogicTeamRules};
pub use logic_runtime_object::LogicRuntimeObject;
pub use logic_sanitize::sanitize_logic_value;
pub use logic_sense_object::LogicSenseObject;
pub use logic_statement::{assemble_logic_source, LogicStatement};
pub use logic_unit_object::LogicUnitObject;
pub use logic_value_helpers::{
    condition_op_test_vars, first_logic_placeholder, logic_utf16_char_code_at, logic_utf16_len,
    logic_var_strict_equal, lvar_value, print_logic_value, read_logic_sequence, read_logic_text,
    set_lvar_value,
};
pub use logic_world_object::{LogicTileObject, LogicWorldObject};
pub use message_type::MessageType;
pub use query_shape::QueryShape;
pub use query_type::QueryType;
pub use radar_sort::RadarSort;
pub use radar_target::RadarTarget;
pub use ranged::Ranged;
pub use senseable::Senseable;
pub use settable::Settable;
pub use tile_layer::TileLayer;

use crate::mindustry::{ctype::ContentType, world::meta::BlockFlag};

use std::collections::BTreeMap;

pub const LOGIC_CANVAS_INVALID_JUMP: i32 = i32::MAX;
pub const LOGIC_TILE_SIZE: f32 = 8.0;
pub const LOGIC_BUILDING_RANGE: f32 = 220.0;
pub const LOGIC_WEATHER_FADE_TIME: f32 = 60.0 * 4.0;
pub const LOGIC_DEFAULT_MAX_IPT: i32 = 1000;
pub const LOGIC_SYNC_INTERVAL_MILLIS: i64 = 1000 / 20;
pub const LOGIC_MAX_MARKERS: usize = 20_000;

pub fn exec_read_runtime(exec: &LogicExecutor, target: &LVar, position: &LVar, output: &mut LVar) {
    if let Some(name) = target.obj() {
        if let Some(object) = exec.objects.get(name) {
            if object.read_runtime(exec.privileged, exec.team, position, output) {
                return;
            }
        }
    }

    output.set_obj(None);
}

pub fn exec_write_runtime(exec: &mut LogicExecutor, target: &LVar, position: &LVar, value: &LVar) {
    let privileged = exec.privileged;
    let team = exec.team;
    if let Some(name) = target.obj() {
        if let Some(object) = exec.objects.get_mut(name) {
            object.write_runtime(privileged, team, position, value);
        }
    }
}

pub fn exec_sense_runtime(exec: &LogicExecutor, from: &LVar, type_: &LVar, to: &mut LVar) {
    let target_name = from.obj();
    let sense_obj = type_.obj();

    if target_name.is_none() && sense_obj == Some("@dead") {
        to.set_num(1.0);
        return;
    }

    if let Some(name) = target_name {
        if let Some(object) = exec.objects.get(name) {
            if let Some(access) = sense_obj.and_then(logic_access_from_object_name) {
                if let Some(value) = object.sense_access(access) {
                    set_lvar_value(to, &value);
                } else {
                    to.set_obj(None);
                }
                return;
            }

            if let Some(content_name) = sense_obj.and_then(logic_content_name_from_object_name) {
                if let Some(value) = object.sense_content(content_name) {
                    to.set_num(value);
                    return;
                }
            }
        }
    }

    to.set_obj(None);
}

pub fn exec_control_runtime(
    exec: &mut LogicExecutor,
    type_: LAccess,
    target: &LVar,
    p1: &LVar,
    p2: &LVar,
    p3: &LVar,
    p4: &LVar,
) {
    let privileged = exec.privileged;
    let Some(name) = target.obj() else {
        return;
    };
    let Some(LogicRuntimeObject::Controllable(controllable)) = exec.objects.get_mut(name) else {
        return;
    };
    if !controllable.controllable_by(privileged) {
        return;
    }

    if type_ == LAccess::Enabled {
        if p1.bool() {
            controllable.no_sleep_calls += 1;
        } else {
            controllable.disabled_by_processor = true;
        }
        controllable.enabled = p1.bool();
    }

    if type_.is_obj() && p1.is_obj {
        controllable.calls.push(LogicControlCall::Object {
            access: type_,
            p1: p1.objval.clone(),
            p2: p2.num(),
            p3: p3.num(),
            p4: p4.num(),
        });
    } else {
        controllable.calls.push(LogicControlCall::Numeric {
            access: type_,
            p1: p1.num(),
            p2: p2.num(),
            p3: p3.num(),
            p4: p4.num(),
        });
    }
}

pub fn exec_radar_runtime(
    exec: &LogicExecutor,
    target1: RadarTarget,
    target2: RadarTarget,
    target3: RadarTarget,
    sort: RadarSort,
    radar: &LVar,
    sort_order: &LVar,
) -> Option<String> {
    let source_name = radar.obj()?;
    let LogicRuntimeObject::RadarSource(source) = exec.objects.get(source_name)? else {
        return None;
    };
    if !source.usable_by(exec.privileged, exec.team) {
        return None;
    }
    find_radar_target(
        source,
        target1,
        target2,
        target3,
        sort,
        sort_order,
        exec.radar_units
            .iter()
            .map(|(name, unit)| (name.clone(), *unit)),
        None,
    )
}

pub fn exec_unit_bind_runtime(exec: &mut LogicExecutor, type_: &LVar) {
    if !exec.privileged && !exec.logic_unit_control {
        return;
    }

    let Some(requested) = type_.obj() else {
        exec.bound_unit = None;
        return;
    };

    if let Some(LogicRuntimeObject::Unit(unit)) = exec.objects.get(requested) {
        exec.bound_unit = unit
            .controllable_by(exec.privileged, exec.team)
            .then(|| requested.to_string());
        return;
    }

    let type_name = logic_unwrap_object_name(requested);
    let mut candidates: Vec<String> = exec
        .objects
        .iter()
        .filter_map(|(name, object)| match object {
            LogicRuntimeObject::Unit(unit)
                if unit.type_name == type_name
                    && unit.controllable_by(exec.privileged, exec.team) =>
            {
                Some(name.clone())
            }
            _ => None,
        })
        .collect();

    candidates.sort();
    if candidates.is_empty() {
        exec.bound_unit = None;
        return;
    }

    let bind_index = exec.unit_binds.entry(type_name.to_string()).or_default();
    *bind_index %= candidates.len();
    exec.bound_unit = Some(candidates[*bind_index].clone());
    *bind_index += 1;
}

pub fn exec_unit_control_runtime(
    exec: &mut LogicExecutor,
    type_: LUnitControl,
    p1: &LVar,
    p2: &LVar,
    p3: &mut LVar,
    p4: &mut LVar,
    p5: &mut LVar,
) {
    if !exec.privileged && !exec.logic_unit_control {
        return;
    }

    let Some(bound_name) = exec.bound_unit.clone() else {
        return;
    };
    let Some(LogicRuntimeObject::Unit(unit)) = exec.objects.get_mut(&bound_name) else {
        exec.bound_unit = None;
        return;
    };
    if !unit.controllable_by(exec.privileged, exec.team) {
        return;
    }

    unit.control_timer_refreshed = true;
    let x1 = logic_unconv(p1.numf());
    let y1 = logic_unconv(p2.numf());
    let d1 = logic_unconv(p3.numf());

    match type_ {
        LUnitControl::Idle | LUnitControl::AutoPathfind => {
            unit.control = Some(type_);
        }
        LUnitControl::Move
        | LUnitControl::Stop
        | LUnitControl::Approach
        | LUnitControl::Pathfind => {
            unit.control = Some(type_);
            unit.move_x = x1;
            unit.move_y = y1;
            if type_ == LUnitControl::Approach {
                unit.move_rad = d1;
            }
            if type_ == LUnitControl::Stop {
                unit.clear_unit_action();
            }
        }
        LUnitControl::Unbind => {
            unit.controller_reset = true;
            exec.bound_unit = None;
        }
        LUnitControl::Within => {
            let dx = unit.x - x1;
            let dy = unit.y - y1;
            p4.set_num((dx * dx + dy * dy <= d1 * d1) as u8 as f64);
        }
        LUnitControl::Target => {
            unit.target_x = x1;
            unit.target_y = y1;
            unit.aim_control = Some(type_);
            unit.main_target = None;
            unit.shoot = p3.bool();
        }
        LUnitControl::Targetp => {
            unit.aim_control = Some(type_);
            unit.main_target = p1.obj().map(str::to_string);
            unit.shoot = p2.bool();
        }
        LUnitControl::Boost => {
            unit.boost = p1.bool();
        }
        LUnitControl::Flag => {
            unit.flag = p1.num();
        }
        LUnitControl::Mine => {
            unit.mine_x = Some(x1);
            unit.mine_y = Some(y1);
        }
        LUnitControl::GetBlock => {
            let dx = unit.x - x1;
            let dy = unit.y - y1;
            let range = unit.range.max(LOGIC_TILE_SIZE);
            if dx * dx + dy * dy > range * range {
                p3.set_obj(None);
                p4.set_obj(None);
                p5.set_obj(None);
            }
        }
        _ => {}
    }
}

pub fn exec_unit_radar_runtime(
    exec: &LogicExecutor,
    target1: RadarTarget,
    target2: RadarTarget,
    target3: RadarTarget,
    sort: RadarSort,
    sort_order: &LVar,
) -> Option<String> {
    if !exec.privileged && !exec.logic_unit_control {
        return None;
    }

    let source_name = exec.bound_unit.as_deref()?;
    let LogicRuntimeObject::Unit(source_unit) = exec.objects.get(source_name)? else {
        return None;
    };
    if !source_unit.controllable_by(exec.privileged, exec.team) {
        return None;
    }

    let source = source_unit.radar_source();
    find_radar_target(
        &source,
        target1,
        target2,
        target3,
        sort,
        sort_order,
        radar_units_with_runtime_units(exec),
        Some(source_name),
    )
}

pub fn exec_unit_locate_runtime(
    exec: &mut LogicExecutor,
    locate: LLocate,
    flag: BlockFlag,
    enemy: &LVar,
    ore: &LVar,
    out_x: &mut LVar,
    out_y: &mut LVar,
    out_found: &mut LVar,
    out_build: &mut LVar,
) {
    if !exec.privileged && !exec.logic_unit_control {
        return;
    }

    let Some(unit) = exec.bound_unit.as_deref().and_then(|name| {
        let LogicRuntimeObject::Unit(unit) = exec.objects.get(name)? else {
            return None;
        };
        unit.controllable_by(exec.privileged, exec.team)
            .then_some(unit)
    }) else {
        out_found.set_bool(false);
        return;
    };

    let result = match locate {
        LLocate::Ore => find_closest_ore(exec, unit, ore.obj()),
        LLocate::Building => find_closest_flagged_building(exec, unit, flag, enemy.bool()),
        LLocate::Spawn => find_closest_spawn(exec, unit),
        LLocate::Damaged => find_closest_damaged_building(exec, unit),
    };

    if let Some(result) = result {
        out_x.set_num(logic_conv(result.x) as f64);
        out_y.set_num(logic_conv(result.y) as f64);
        out_found.set_bool(true);
        out_build.set_obj(result.building);
    } else {
        out_found.set_bool(false);
        out_build.set_obj(None);
    }
}

pub fn exec_query_runtime(
    exec: &mut LogicExecutor,
    shape: QueryShape,
    type_: QueryType,
    team: &LVar,
    x: &LVar,
    y: &LVar,
    w: &LVar,
    h: &LVar,
) {
    if type_ == QueryType::Bullet {
        return;
    }

    let Some(query_result_name) = exec.query_result.clone() else {
        return;
    };

    let team_filter = logic_team_from_var(team);
    let mut x = logic_unconv(x.numf());
    let mut y = logic_unconv(y.numf());
    let mut w = logic_unconv(w.numf());
    let mut h = logic_unconv(h.numf());
    let mut radius = w;
    let circle_x = x;
    let circle_y = y;
    let circle = shape == QueryShape::Circle;
    if circle {
        x -= radius;
        y -= radius;
        w = radius * 2.0;
        h = radius * 2.0;
    } else {
        radius = 0.0;
    }

    let mut results = Vec::new();
    match type_ {
        QueryType::Unit => {
            for (name, object) in &exec.objects {
                let LogicRuntimeObject::Unit(unit) = object else {
                    continue;
                };
                if !unit.valid || team_filter.is_some_and(|team| unit.team != team) {
                    continue;
                }
                if !logic_rect_contains(unit.x, unit.y, x, y, w, h) {
                    continue;
                }
                if circle && !logic_circle_contains(unit.x, unit.y, circle_x, circle_y, radius, 0.0)
                {
                    continue;
                }
                results.push(name.clone());
            }
        }
        QueryType::Building => {
            for (name, object) in &exec.objects {
                let LogicRuntimeObject::Building(building) = object else {
                    continue;
                };
                if !building.valid || team_filter.is_some_and(|team| building.team != team) {
                    continue;
                }
                if !logic_rect_contains(building.x, building.y, x, y, w, h) {
                    continue;
                }
                if circle
                    && !logic_circle_contains(
                        building.x,
                        building.y,
                        circle_x,
                        circle_y,
                        radius,
                        building.hit_size / 2.0,
                    )
                {
                    continue;
                }
                results.push(name.clone());
            }
        }
        QueryType::Bullet => {}
    }

    exec.objects
        .insert(query_result_name, LogicRuntimeObject::QueryResult(results));
}

pub fn exec_get_block_runtime(
    exec: &LogicExecutor,
    layer: TileLayer,
    result: &mut LVar,
    x: &LVar,
    y: &LVar,
) {
    let x = x.numf().round() as i32;
    let y = y.numf().round() as i32;
    result.set_obj(exec.world.tile(x, y).and_then(|tile| tile.get_layer(layer)));
}

pub fn exec_set_block_runtime(
    exec: &mut LogicExecutor,
    layer: TileLayer,
    block: &LVar,
    x: &LVar,
    y: &LVar,
    team: &LVar,
    rotation: &LVar,
) {
    if layer == TileLayer::Building {
        return;
    }

    let x = x.numi();
    let y = y.numi();
    let Some(block_name) = block.obj().map(logic_object_name) else {
        return;
    };
    let Some(tile) = exec.world.tile_mut(x, y) else {
        return;
    };

    let team = logic_team_from_var(team).unwrap_or(RadarTarget::DERELICT_TEAM);
    tile.set_layer(layer, Some(block_name), team, rotation.numi());
}

pub fn exec_fetch_runtime(
    exec: &LogicExecutor,
    type_: FetchType,
    result: &mut LVar,
    team: &LVar,
    index: &LVar,
    extra: &LVar,
) {
    let Some(team) = logic_team_from_var(team) else {
        return;
    };
    let index = index.numi();

    match type_ {
        FetchType::Unit | FetchType::Player => {
            let units = fetch_units(exec, team, extra.obj());
            if matches!(type_, FetchType::Player) {
                let players: Vec<_> = units
                    .into_iter()
                    .filter(|name| {
                        matches!(exec.objects.get(name), Some(LogicRuntimeObject::Unit(unit)) if unit.is_player)
                    })
                    .collect();
                result.set_obj(logic_index_name(&players, index));
            } else {
                result.set_obj(logic_index_name(&units, index));
            }
        }
        FetchType::Core | FetchType::Build => {
            let builds = fetch_buildings(exec, team, extra.obj(), matches!(type_, FetchType::Core));
            result.set_obj(logic_index_name(&builds, index));
        }
        FetchType::UnitCount => {
            result.set_num(fetch_units(exec, team, extra.obj()).len() as f64);
        }
        FetchType::PlayerCount => {
            result.set_num(
                fetch_units(exec, team, None)
                    .into_iter()
                    .filter(|name| {
                        matches!(exec.objects.get(name), Some(LogicRuntimeObject::Unit(unit)) if unit.is_player)
                    })
                    .count() as f64,
            );
        }
        FetchType::CoreCount => {
            result.set_num(fetch_buildings(exec, team, None, true).len() as f64);
        }
        FetchType::BuildCount => {
            result.set_num(fetch_buildings(exec, team, extra.obj(), false).len() as f64);
        }
    }
}

pub fn exec_locale_print_runtime(exec: &mut LogicExecutor, value: &LVar) {
    if exec.text_buffer.len() >= LogicExecutor::MAX_TEXT_BUFFER || !value.is_obj {
        return;
    }

    let key = print_logic_value(value);
    let localized = if exec.mobile {
        exec.map_locales
            .get(&format!("{key}.mobile"))
            .or_else(|| exec.map_locales.get(&key))
    } else {
        exec.map_locales.get(&key)
    };

    if let Some(localized) = localized.cloned() {
        exec.push_text_bounded(&localized);
    }
}

pub fn exec_draw_flush_runtime(exec: &mut LogicExecutor, target: &LVar) {
    let commands = std::mem::take(&mut exec.graphics_buffer);
    let Some(target_name) = target.obj() else {
        return;
    };
    let Some(LogicRuntimeObject::Building(building)) = exec.objects.get_mut(target_name) else {
        return;
    };
    if building.valid && (building.team == exec.team || exec.privileged) {
        building.display_commands = commands;
    }
}

pub fn exec_print_flush_runtime(exec: &mut LogicExecutor, target: &LVar) {
    let text = std::mem::take(&mut exec.text_buffer);
    let Some(target_name) = target.obj() else {
        return;
    };
    let Some(LogicRuntimeObject::Building(building)) = exec.objects.get_mut(target_name) else {
        return;
    };
    if building.valid
        && (exec.privileged || (building.team == exec.team && !building.block_privileged))
    {
        building.message = text.chars().take(LogicExecutor::MAX_TEXT_BUFFER).collect();
    }
}

pub fn exec_set_rate_runtime(exec: &mut LogicExecutor, amount: &LVar) {
    exec.ipt = amount.numi().clamp(1, exec.max_ipt.max(1));
}

pub fn exec_sync_runtime(exec: &mut LogicExecutor, variable: &mut LVar) {
    if variable.constant
        || exec.current_time_millis.saturating_sub(variable.sync_time) <= LOGIC_SYNC_INTERVAL_MILLIS
    {
        return;
    }

    variable.sync_time = exec.current_time_millis;
    exec.sync_events.push(LogicSyncEvent {
        variable_id: variable.id,
        value: variable.value(),
    });
}

#[allow(clippy::too_many_arguments)]
pub fn exec_spawn_bullet_runtime(
    exec: &mut LogicExecutor,
    result: &mut LVar,
    from: &LVar,
    weapon: &LVar,
    x: &LVar,
    y: &LVar,
    rotation: &LVar,
    team: &LVar,
    owner: &LVar,
    damage: &LVar,
    velocity_scl: &LVar,
    life_scl: &LVar,
    aim_x: &LVar,
    aim_y: &LVar,
) {
    let Some(from_name) = from.obj().map(str::to_string) else {
        return;
    };
    let owner_name = owner.obj().map(str::to_string);
    let team = logic_team_from_var(team)
        .or_else(|| {
            owner_name
                .as_deref()
                .and_then(|name| exec.objects.get(name))
                .and_then(|object| match object {
                    LogicRuntimeObject::Unit(unit) => Some(unit.team),
                    LogicRuntimeObject::Building(building) => Some(building.team),
                    LogicRuntimeObject::Bullet(bullet) => Some(bullet.team),
                    _ => None,
                })
        })
        .unwrap_or(RadarTarget::DERELICT_TEAM);

    let bullet_name = format!("bullet-{}", exec.bullet_events.len());
    let event = LogicBulletEvent {
        bullet_name: bullet_name.clone(),
        from_name,
        weapon: weapon.value(),
        team,
        x: logic_unconv(x.numf()),
        y: logic_unconv(y.numf()),
        rotation: rotation.numf(),
        owner: owner_name,
        damage: damage.numf(),
        velocity_scl: velocity_scl.numf(),
        life_scl: life_scl.numf(),
        aim_x: logic_unconv(aim_x.numf()),
        aim_y: logic_unconv(aim_y.numf()),
    };
    exec.register_object(
        bullet_name.clone(),
        LogicRuntimeObject::Bullet(event.clone()),
    );
    exec.bullet_events.push(event);
    result.set_obj(Some(bullet_name));
}

pub fn exec_weather_sense_runtime(exec: &LogicExecutor, to: &mut LVar, weather: &LVar) {
    let active = weather
        .obj()
        .and_then(|name| exec.weather_states.get(&logic_object_name(name)))
        .is_some_and(|state| state.active);
    to.set_bool(active);
}

pub fn exec_weather_set_runtime(exec: &mut LogicExecutor, weather: &LVar, state: &LVar) {
    let Some(weather_name) = weather.obj().map(logic_object_name) else {
        return;
    };
    let active = state.bool();
    let weather_state = exec.weather_states.entry(weather_name.clone()).or_default();
    if active {
        weather_state.active = true;
        weather_state.life = LOGIC_WEATHER_FADE_TIME;
    } else if weather_state.active && weather_state.life > LOGIC_WEATHER_FADE_TIME {
        weather_state.life = LOGIC_WEATHER_FADE_TIME;
    }
    exec.weather_events.push(LogicWeatherEvent {
        weather_name,
        active,
        life: weather_state.life,
    });
}

pub fn exec_set_prop_runtime(exec: &mut LogicExecutor, type_: &LVar, of: &LVar, value: &LVar) {
    let Some(target_name) = of.obj() else {
        return;
    };
    let Some(key) = type_.obj() else {
        return;
    };
    let value = lvar_value(value);

    let Some(object) = exec.objects.get_mut(target_name) else {
        return;
    };
    if let Some(access) = logic_access_from_object_name(key) {
        match object {
            LogicRuntimeObject::Unit(unit) => unit.set_prop(access, value),
            LogicRuntimeObject::Building(building) => building.set_prop(access, value),
            _ => {}
        }
    } else {
        let content_name = logic_object_name(key);
        let amount = match value {
            LVarValue::Number(value) => value,
            LVarValue::Object(Some(_)) => 1.0,
            LVarValue::Object(None) => 0.0,
        };
        match object {
            LogicRuntimeObject::Unit(unit) => unit.set_content_prop(content_name, amount),
            LogicRuntimeObject::Building(building) => {
                building.set_content_prop(content_name, amount)
            }
            _ => {}
        }
    }
}

pub fn exec_client_data_runtime(
    exec: &mut LogicExecutor,
    channel: &LVar,
    value: &LVar,
    reliable: &LVar,
) {
    if let Some(channel) = channel.obj() {
        exec.client_data_events.push(LogicClientDataEvent {
            channel: channel.to_string(),
            value: value.value(),
            reliable: reliable.bool(),
        });
    }
}

#[allow(clippy::too_many_arguments)]
pub fn exec_play_sound_runtime(
    exec: &mut LogicExecutor,
    positional: bool,
    id: &LVar,
    volume: &LVar,
    pitch: &LVar,
    pan: &LVar,
    x: &LVar,
    y: &LVar,
    limit: &LVar,
) {
    exec.sound_events.push(LogicSoundEvent {
        positional,
        sound_id: id.numi(),
        sound_name: id.obj().map(str::to_string),
        volume: volume.numf().min(2.0),
        pitch: pitch.numf(),
        pan: pan.numf(),
        x: positional.then(|| logic_unconv(x.numf())),
        y: positional.then(|| logic_unconv(y.numf())),
        limit: limit.bool(),
    });
}

pub fn logic_marker_type_known(type_name: &str) -> bool {
    matches!(
        type_name,
        "ShapeText"
            | "shapeText"
            | "Point"
            | "point"
            | "Shape"
            | "shape"
            | "Text"
            | "text"
            | "Line"
            | "line"
            | "Texture"
            | "texture"
            | "Quad"
            | "quad"
            | "Minimap"
            | "minimap"
    )
}

pub fn exec_make_marker_runtime(
    exec: &mut LogicExecutor,
    type_name: &str,
    id: &LVar,
    x: &LVar,
    y: &LVar,
    replace: &LVar,
) {
    if !logic_marker_type_known(type_name) || exec.markers.len() >= LOGIC_MAX_MARKERS {
        return;
    }

    let id = id.numi();
    let replaced = exec.markers.contains_key(&id);
    if replace.bool() || !replaced {
        let marker =
            LogicMarkerObject::new(type_name, logic_unconv(x.numf()), logic_unconv(y.numf()));
        exec.markers.insert(id, marker);
        exec.marker_events.push(LogicMarkerEvent::Created {
            id,
            type_name: type_name.to_string(),
            x: logic_unconv(x.numf()),
            y: logic_unconv(y.numf()),
            replaced,
        });
    }
}

pub fn exec_set_marker_runtime(
    exec: &mut LogicExecutor,
    type_: LMarkerControl,
    id: &LVar,
    p1: &LVar,
    p2: &LVar,
    p3: &LVar,
) {
    let id = id.numi();
    if type_ == LMarkerControl::Remove {
        exec.markers.remove(&id);
        exec.marker_events.push(LogicMarkerEvent::Removed { id });
        return;
    }

    let Some(marker) = exec.markers.get_mut(&id) else {
        return;
    };
    if type_ == LMarkerControl::FlushText {
        let text = std::mem::take(&mut exec.text_buffer);
        let fetch = p1.bool();
        marker.text = text.clone();
        marker.text_fetch = fetch;
        exec.marker_events
            .push(LogicMarkerEvent::Text { id, text, fetch });
    } else if type_ == LMarkerControl::Texture {
        let texture = if p1.bool() {
            LVarValue::Object(Some(std::mem::take(&mut exec.text_buffer)))
        } else {
            p2.value()
        };
        marker.texture = texture.clone();
        exec.marker_events
            .push(LogicMarkerEvent::Texture { id, texture });
    } else {
        let event = LogicMarkerControlEvent {
            id,
            control: type_,
            p1: p1.num_or_nan(),
            p2: p2.num_or_nan(),
            p3: p3.num_or_nan(),
        };
        marker.control(event.clone());
        exec.marker_events.push(LogicMarkerEvent::Controlled(event));
    }
}

pub fn exec_get_flag_runtime(exec: &LogicExecutor, result: &mut LVar, flag: &LVar) {
    if let Some(flag) = flag.obj() {
        result.set_bool(exec.objective_flags.contains(flag));
    } else {
        result.set_obj(None);
    }
}

pub fn exec_set_flag_runtime(exec: &mut LogicExecutor, flag: &LVar, value: &LVar) {
    let Some(flag) = flag.obj() else {
        return;
    };
    if value.bool() {
        exec.objective_flags.insert(flag.to_string());
    } else {
        exec.objective_flags.remove(flag);
    }
}

pub fn exec_spawn_unit_runtime(
    exec: &mut LogicExecutor,
    type_: &LVar,
    x: &LVar,
    y: &LVar,
    rotation: &LVar,
    team: &LVar,
    result: &mut LVar,
) {
    if exec.is_client {
        return;
    }

    let Some(team) = logic_team_from_var(team) else {
        return;
    };
    let Some(type_name) = type_.obj().map(logic_unwrap_object_name) else {
        return;
    };

    let unit_name = format!("spawned-{}-{}", type_name, exec.spawn_events.len());
    let x = logic_unconv(x.numf());
    let y = logic_unconv(y.numf());
    let rotation = rotation.numf();
    exec.register_object(
        unit_name.clone(),
        LogicRuntimeObject::Unit(LogicUnitObject::new(type_name, team, x, y)),
    );
    exec.spawn_events.push(LogicSpawnEvent {
        unit_name: unit_name.clone(),
        type_name: type_name.to_string(),
        team,
        x,
        y,
        rotation,
    });
    result.set_obj(Some(unit_name));
}

pub fn exec_apply_status_runtime(
    exec: &mut LogicExecutor,
    clear: bool,
    effect: &str,
    unit: &LVar,
    duration: &LVar,
) {
    if exec.is_client {
        return;
    }

    let Some(unit_name) = unit.obj() else {
        return;
    };
    let Some(LogicRuntimeObject::Unit(unit)) = exec.objects.get_mut(unit_name) else {
        return;
    };

    if clear {
        unit.statuses.remove(effect);
    } else {
        unit.statuses
            .insert(effect.to_string(), duration.numf() * 60.0);
    }
}

pub fn exec_spawn_wave_runtime(exec: &mut LogicExecutor, x: &LVar, y: &LVar, natural: &LVar) {
    if exec.is_client {
        return;
    }
    exec.spawn_wave_events.push((
        logic_unconv(x.numf()),
        logic_unconv(y.numf()),
        natural.bool(),
    ));
}

pub fn exec_effect_runtime(
    exec: &mut LogicExecutor,
    type_name: &str,
    x: &LVar,
    y: &LVar,
    rotation: &LVar,
    color: &LVar,
    data: &LVar,
) {
    let Some(effect) = get_logic_effect(type_name) else {
        return;
    };
    let rotation = if effect.rotate {
        rotation.numf()
    } else {
        rotation.numf().min(1000.0)
    };
    exec.effect_events.push(LogicEffectEvent {
        type_name: type_name.to_string(),
        effect_name: effect.effect.to_string(),
        x: logic_unconv(x.numf()),
        y: logic_unconv(y.numf()),
        rotation,
        color: color.num(),
        data: data.obj().map(str::to_string),
    });
}

#[allow(clippy::too_many_arguments)]
pub fn exec_explosion_runtime(
    exec: &mut LogicExecutor,
    team: &LVar,
    x: &LVar,
    y: &LVar,
    radius: &LVar,
    damage: &LVar,
    air: &LVar,
    ground: &LVar,
    pierce: &LVar,
    effect: &LVar,
) {
    if exec.is_client {
        return;
    }
    exec.explosion_events.push(LogicExplosionEvent {
        team: logic_team_from_var(team),
        x: logic_unconv(x.numf()),
        y: logic_unconv(y.numf()),
        radius: logic_unconv(radius.numf().min(100.0)),
        damage: damage.numf(),
        air: air.bool(),
        ground: ground.bool(),
        pierce: pierce.bool(),
        effect: effect.bool(),
    });
}

pub fn exec_set_rule_runtime(
    exec: &mut LogicExecutor,
    rule: LogicRule,
    value: &LVar,
    p1: &LVar,
    p2: &LVar,
    p3: &LVar,
    p4: &LVar,
) {
    match rule {
        LogicRule::WaveTimer => exec.rules.wave_timer = value.bool(),
        LogicRule::Wave => exec.rules.wave = value.numi().max(1),
        LogicRule::CurrentWaveTime => exec.rules.wave_time = (value.numf() * 60.0).max(0.0),
        LogicRule::Waves => exec.rules.waves = value.bool(),
        LogicRule::WaveSending => exec.rules.wave_sending = value.bool(),
        LogicRule::AttackMode => exec.rules.attack_mode = value.bool(),
        LogicRule::WaveSpacing => exec.rules.wave_spacing = value.numf() * 60.0,
        LogicRule::EnemyCoreBuildRadius => {
            exec.rules.enemy_core_build_radius = value.numf() * LOGIC_TILE_SIZE
        }
        LogicRule::DropZoneRadius => exec.rules.drop_zone_radius = value.numf() * LOGIC_TILE_SIZE,
        LogicRule::UnitCap => exec.rules.unit_cap = value.numi().max(0),
        LogicRule::Lighting => exec.rules.lighting = value.bool(),
        LogicRule::CanGameOver => exec.rules.can_game_over = value.bool(),
        LogicRule::PauseDisabled => exec.rules.pause_disabled = value.bool(),
        LogicRule::MapArea => {
            exec.rules.map_area = Some((p1.numi(), p2.numi(), p3.numi(), p4.numi()));
        }
        LogicRule::AmbientLight => exec.rules.ambient_light = value.num(),
        LogicRule::SolarMultiplier => exec.rules.solar_multiplier = value.numf().max(0.0),
        LogicRule::DragMultiplier => exec.rules.drag_multiplier = value.numf().max(0.0),
        LogicRule::Ban => {
            if let Some(content) = value.obj().map(logic_object_name) {
                if exec
                    .objects
                    .contains_key(logic_unwrap_object_name(&content))
                {
                    exec.rules.banned_units.insert(content);
                } else {
                    exec.rules.banned_blocks.insert(content);
                }
            }
        }
        LogicRule::Unban => {
            if let Some(content) = value.obj().map(logic_object_name) {
                exec.rules.banned_blocks.remove(&content);
                exec.rules.banned_units.remove(&content);
            }
        }
        LogicRule::BuildSpeed
        | LogicRule::UnitHealth
        | LogicRule::UnitBuildSpeed
        | LogicRule::UnitMineSpeed
        | LogicRule::UnitCost
        | LogicRule::UnitDamage
        | LogicRule::BlockHealth
        | LogicRule::BlockDamage
        | LogicRule::RtsMinWeight
        | LogicRule::RtsMinSquad => {
            let Some(team) = logic_team_from_var(p1) else {
                return;
            };
            let team_rules = exec.rules.team_rules.entry(team).or_default();
            let num = value.numf();
            match rule {
                LogicRule::BuildSpeed => team_rules.build_speed_multiplier = num.clamp(0.001, 50.0),
                LogicRule::UnitHealth => team_rules.unit_health_multiplier = num.max(0.001),
                LogicRule::UnitBuildSpeed => {
                    team_rules.unit_build_speed_multiplier = num.clamp(0.0, 50.0)
                }
                LogicRule::UnitMineSpeed => team_rules.unit_mine_speed_multiplier = num.max(0.0),
                LogicRule::UnitCost => team_rules.unit_cost_multiplier = num.max(0.0),
                LogicRule::UnitDamage => team_rules.unit_damage_multiplier = num.max(0.0),
                LogicRule::BlockHealth => team_rules.block_health_multiplier = num.max(0.001),
                LogicRule::BlockDamage => team_rules.block_damage_multiplier = num.max(0.0),
                LogicRule::RtsMinWeight => team_rules.rts_min_weight = num,
                LogicRule::RtsMinSquad => team_rules.rts_min_squad = num as i32,
                _ => {}
            }
        }
    }
}

pub fn exec_flush_message_runtime(
    exec: &mut LogicExecutor,
    type_: MessageType,
    duration: &LVar,
    out_success: &mut LVar,
) {
    out_success.set_num(1.0);
    if exec.headless && type_ != MessageType::Mission {
        exec.text_buffer.clear();
        return;
    }

    let blocked = match type_ {
        MessageType::Announce => exec.message_state.announcement_active,
        MessageType::Notify => exec.message_state.toast_active,
        MessageType::Toast => exec.message_state.announcement_active,
        MessageType::Mission => false,
    };
    if blocked {
        if out_success.name == "@wait" {
            exec.counter.numval -= 1.0;
            exec.yield_ = true;
        } else {
            out_success.set_num(0.0);
        }
        return;
    }

    let text = exec.text_buffer.clone();
    if type_ == MessageType::Mission {
        exec.rules.mission = text.clone();
    } else {
        exec.message_events.push(LogicMessageEvent {
            type_,
            text,
            duration: duration.numf(),
        });
    }
    exec.text_buffer.clear();
}

pub fn exec_cutscene_runtime(
    exec: &mut LogicExecutor,
    action: CutsceneAction,
    p1: &LVar,
    p2: &LVar,
    p3: &LVar,
    _p4: &LVar,
) {
    if exec.headless {
        return;
    }

    match action {
        CutsceneAction::Pan => {
            exec.cutscene.active = true;
            exec.cutscene.pan_x = logic_unconv(p1.numf());
            exec.cutscene.pan_y = logic_unconv(p2.numf());
            exec.cutscene.speed = p3.numf();
        }
        CutsceneAction::Zoom => {
            exec.cutscene.active = true;
            exec.cutscene.zoom = p1.numf().clamp(0.0, 1.0);
        }
        CutsceneAction::Stop => {
            exec.cutscene.active = false;
        }
    }
}

fn radar_units_with_runtime_units(exec: &LogicExecutor) -> Vec<(String, RadarUnitView)> {
    let mut units: BTreeMap<String, RadarUnitView> = exec
        .radar_units
        .iter()
        .map(|(name, unit)| (name.clone(), *unit))
        .collect();
    for (name, object) in &exec.objects {
        if let LogicRuntimeObject::Unit(unit) = object {
            units.insert(name.clone(), unit.radar_view());
        }
    }
    units.into_iter().collect()
}

fn find_radar_target<I>(
    source: &LogicRadarSource,
    target1: RadarTarget,
    target2: RadarTarget,
    target3: RadarTarget,
    sort: RadarSort,
    sort_order: &LVar,
    units: I,
    exclude_name: Option<&str>,
) -> Option<String>
where
    I: IntoIterator<Item = (String, RadarUnitView)>,
{
    let sort_dir = if sort_order.bool() { 1.0 } else { -1.0 };
    let range_sq = source.range * source.range;
    let mut best: Option<(String, f32)> = None;

    for (name, unit) in units {
        if exclude_name == Some(name.as_str()) || !unit.targetable {
            continue;
        }
        let dx = source.x - unit.x;
        let dy = source.y - unit.y;
        if dx * dx + dy * dy > range_sq {
            continue;
        }
        if !target1.matches(source.team, &unit)
            || !target2.matches(source.team, &unit)
            || !target3.matches(source.team, &unit)
        {
            continue;
        }

        let value = sort.score(source.x, source.y, &unit) * sort_dir;
        if best
            .as_ref()
            .is_none_or(|(_, best_value)| value > *best_value)
        {
            best = Some((name, value));
        }
    }

    best.map(|(name, _)| name)
}

fn find_closest_ore(
    exec: &LogicExecutor,
    unit: &LogicUnitObject,
    ore: Option<&str>,
) -> Option<LogicLocateResult> {
    let ore = ore.map(logic_object_name)?;
    exec.world
        .tiles
        .iter()
        .filter(|(_, tile)| tile.ore.as_deref() == Some(ore.as_str()))
        .map(|((x, y), _)| LogicLocateResult {
            x: logic_tile_world(*x),
            y: logic_tile_world(*y),
            building: None,
        })
        .min_by(|a, b| {
            logic_distance_sq(unit.x, unit.y, a.x, a.y)
                .total_cmp(&logic_distance_sq(unit.x, unit.y, b.x, b.y))
        })
}

fn find_closest_flagged_building(
    exec: &LogicExecutor,
    unit: &LogicUnitObject,
    flag: BlockFlag,
    enemy: bool,
) -> Option<LogicLocateResult> {
    exec.objects
        .iter()
        .filter_map(|(name, object)| {
            let LogicRuntimeObject::Building(building) = object else {
                return None;
            };
            if !building.valid || !building.has_flag(flag) {
                return None;
            }
            if enemy == (building.team == unit.team || building.team == RadarTarget::DERELICT_TEAM)
            {
                return None;
            }
            Some((name, building))
        })
        .map(|(name, building)| LogicLocateResult {
            x: building.x,
            y: building.y,
            building: locate_building_visible(unit, name, building),
        })
        .min_by(|a, b| {
            logic_distance_sq(unit.x, unit.y, a.x, a.y)
                .total_cmp(&logic_distance_sq(unit.x, unit.y, b.x, b.y))
        })
}

fn find_closest_spawn(exec: &LogicExecutor, unit: &LogicUnitObject) -> Option<LogicLocateResult> {
    exec.world
        .spawns
        .iter()
        .map(|(x, y)| LogicLocateResult {
            x: *x,
            y: *y,
            building: None,
        })
        .min_by(|a, b| {
            logic_distance_sq(unit.x, unit.y, a.x, a.y)
                .total_cmp(&logic_distance_sq(unit.x, unit.y, b.x, b.y))
        })
}

fn find_closest_damaged_building(
    exec: &LogicExecutor,
    unit: &LogicUnitObject,
) -> Option<LogicLocateResult> {
    exec.objects
        .iter()
        .filter_map(|(name, object)| {
            let LogicRuntimeObject::Building(building) = object else {
                return None;
            };
            (building.valid && building.damaged && building.team == unit.team)
                .then_some((name, building))
        })
        .map(|(name, building)| LogicLocateResult {
            x: building.x,
            y: building.y,
            building: locate_building_visible(unit, name, building),
        })
        .min_by(|a, b| {
            logic_distance_sq(unit.x, unit.y, a.x, a.y)
                .total_cmp(&logic_distance_sq(unit.x, unit.y, b.x, b.y))
        })
}

fn locate_building_visible(
    unit: &LogicUnitObject,
    name: &str,
    building: &LogicBuildingObject,
) -> Option<String> {
    let range = unit.range.max(LOGIC_BUILDING_RANGE);
    (building.team == unit.team
        || logic_distance_sq(unit.x, unit.y, building.x, building.y) <= range * range)
        .then(|| name.to_string())
}

fn fetch_units(exec: &LogicExecutor, team: u8, type_name: Option<&str>) -> Vec<String> {
    let type_name = type_name.map(logic_unwrap_object_name);
    exec.objects
        .iter()
        .filter_map(|(name, object)| {
            let LogicRuntimeObject::Unit(unit) = object else {
                return None;
            };
            if unit.team != team || type_name.is_some_and(|type_name| unit.type_name != type_name) {
                return None;
            }
            Some(name.clone())
        })
        .collect()
}

fn fetch_buildings(
    exec: &LogicExecutor,
    team: u8,
    block_name: Option<&str>,
    core_only: bool,
) -> Vec<String> {
    let block_name = block_name.map(logic_unwrap_object_name);
    exec.objects
        .iter()
        .filter_map(|(name, object)| {
            let LogicRuntimeObject::Building(building) = object else {
                return None;
            };
            if building.team != team
                || block_name.is_some_and(|block_name| building.block_name != block_name)
                || (core_only && !building.has_flag(BlockFlag::Core))
            {
                return None;
            }
            Some(name.clone())
        })
        .collect()
}

fn logic_index_name(values: &[String], index: i32) -> Option<String> {
    (index >= 0)
        .then(|| values.get(index as usize).cloned())
        .flatten()
}

fn logic_rect_contains(px: f32, py: f32, x: f32, y: f32, w: f32, h: f32) -> bool {
    px >= x && py >= y && px <= x + w && py <= y + h
}

fn logic_circle_contains(px: f32, py: f32, x: f32, y: f32, radius: f32, extra: f32) -> bool {
    logic_distance_sq(px, py, x, y) <= (radius + extra) * (radius + extra)
}

fn logic_distance_sq(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x1 - x2;
    let dy = y1 - y2;
    dx * dx + dy * dy
}

fn logic_tile_world(coord: i32) -> f32 {
    coord as f32 * LOGIC_TILE_SIZE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_logic_statements_keep_java_generated_field_order_defaults_and_privilege() {
        assert_eq!(LogicStatement::invalid().write_line(), "noop");
        assert_eq!(LogicStatement::read().write_line(), "read result cell1 0");
        assert_eq!(LogicStatement::write().write_line(), "write result cell1 0");
        assert_eq!(
            LogicStatement::draw().write_line(),
            "draw clear 0 0 0 0 0 0"
        );
        assert_eq!(LogicStatement::print().write_line(), "print \"frog\"");
        assert_eq!(LogicStatement::print_char().write_line(), "printchar 65");
        assert_eq!(LogicStatement::format().write_line(), "format \"frog\"");
        assert_eq!(
            LogicStatement::locale_print().write_line(),
            "localeprint \"name\""
        );
        assert_eq!(
            LogicStatement::draw_flush().write_line(),
            "drawflush display1"
        );
        assert_eq!(
            LogicStatement::print_flush().write_line(),
            "printflush message1"
        );
        assert_eq!(LogicStatement::get_link().write_line(), "getlink result 0");
        assert_eq!(LogicStatement::set_rate().write_line(), "setrate 10");
        assert_eq!(LogicStatement::sync().write_line(), "sync var");
        assert_eq!(LogicStatement::set().write_line(), "set result 0");
        assert_eq!(
            LogicStatement::operation().write_line(),
            "op add result a b"
        );
        assert_eq!(
            LogicStatement::select().write_line(),
            "select result notEqual x false a b"
        );
        assert_eq!(LogicStatement::wait().write_line(), "wait 0.5");
        assert_eq!(LogicStatement::stop().write_line(), "stop");
        assert_eq!(LogicStatement::end().write_line(), "end");
        assert_eq!(
            LogicStatement::pack_color().write_line(),
            "packcolor result 1 0 0 1"
        );
        assert_eq!(
            LogicStatement::unpack_color().write_line(),
            "unpackcolor r g b a color"
        );
        assert_eq!(
            LogicStatement::lookup().write_line(),
            "lookup item result 0"
        );
        assert_eq!(
            LogicStatement::jump().write_line(),
            "jump 0 notEqual x false"
        );
        assert_eq!(
            LogicStatement::control().write_line(),
            "control enabled block1 0 0 0 0"
        );
        assert_eq!(
            LogicStatement::radar().write_line(),
            "radar enemy any any distance turret1 1 result"
        );
        assert_eq!(
            LogicStatement::sensor().write_line(),
            "sensor result block1 @copper"
        );
        assert_eq!(LogicStatement::unit_bind().write_line(), "ubind @poly");
        assert_eq!(
            LogicStatement::unit_control().write_line(),
            "ucontrol move 0 0 0 0 0"
        );
        assert_eq!(
            LogicStatement::unit_radar().write_line(),
            "uradar enemy any any distance 0 1 result"
        );
        assert_eq!(
            LogicStatement::unit_locate().write_line(),
            "ulocate building core true @copper outx outy found building"
        );
        assert_eq!(
            LogicStatement::query().write_line(),
            "query circle unit null 0 0 10 10"
        );
        assert_eq!(
            LogicStatement::get_block().write_line(),
            "getblock block result 0 0"
        );
        assert_eq!(
            LogicStatement::set_block().write_line(),
            "setblock block @air 0 0 @derelict 0"
        );
        assert_eq!(
            LogicStatement::spawn_unit().write_line(),
            "spawn @dagger 10 10 90 @sharded result"
        );
        assert_eq!(
            LogicStatement::apply_status().write_line(),
            "status false wet unit 10"
        );
        assert_eq!(
            LogicStatement::spawn_wave().write_line(),
            "spawnwave 10 10 false"
        );
        assert_eq!(
            LogicStatement::spawn_bullet().write_line(),
            "bullet result @dagger 0 x y angle null null -1 1 1 -1 -1"
        );
        assert_eq!(
            LogicStatement::weather_sense().write_line(),
            "weathersense result @rain"
        );
        assert_eq!(
            LogicStatement::weather_set().write_line(),
            "weatherset @rain true"
        );
        assert_eq!(
            LogicStatement::effect().write_line(),
            "effect warn 0 0 2 %ffaaff "
        );
        assert_eq!(
            LogicStatement::explosion().write_line(),
            "explosion @crux 0 0 5 50 true true false true"
        );
        assert_eq!(
            LogicStatement::set_rule().write_line(),
            "setrule waveSpacing 10 0 0 100 100"
        );
        assert_eq!(
            LogicStatement::fetch().write_line(),
            "fetch unit result @sharded 0 @conveyor"
        );
        assert_eq!(
            LogicStatement::get_flag().write_line(),
            "getflag result \"flag\""
        );
        assert_eq!(
            LogicStatement::set_flag().write_line(),
            "setflag \"flag\" true"
        );
        assert_eq!(
            LogicStatement::set_prop().write_line(),
            "setprop @copper block1 0"
        );
        assert_eq!(
            LogicStatement::flush_message().write_line(),
            "message announce 3 @wait"
        );
        assert_eq!(
            LogicStatement::cutscene().write_line(),
            "cutscene pan 100 100 0.06 0"
        );
        assert_eq!(
            LogicStatement::client_data().write_line(),
            "clientdata \"frog\" \"bar\" 0"
        );
        assert_eq!(
            LogicStatement::play_sound().write_line(),
            "playsound false @sfx-shoot 1 1 0 @thisx @thisy true"
        );
        assert_eq!(
            LogicStatement::set_marker().write_line(),
            "setmarker pos 0 0 0 0"
        );
        assert_eq!(
            LogicStatement::make_marker().write_line(),
            "makemarker shape 0 0 0 true"
        );

        assert_eq!(LogicStatement::read().category().name, "io");
        assert_eq!(LogicStatement::draw().category().name, "io");
        assert_eq!(LogicStatement::print_char().category().name, "io");
        assert_eq!(LogicStatement::draw_flush().category().name, "block");
        assert_eq!(LogicStatement::set_rate().category().name, "world");
        assert_eq!(LogicStatement::operation().category().name, "operation");
        assert_eq!(LogicStatement::select().category().name, "operation");
        assert_eq!(LogicStatement::lookup().category().name, "operation");
        assert_eq!(LogicStatement::wait().category().name, "control");
        assert_eq!(LogicStatement::jump().category().name, "control");
        assert_eq!(LogicStatement::control().category().name, "block");
        assert_eq!(LogicStatement::radar().category().name, "block");
        assert_eq!(LogicStatement::sensor().category().name, "block");
        assert_eq!(LogicStatement::unit_bind().category().name, "unit");
        assert_eq!(LogicStatement::unit_control().category().name, "unit");
        assert_eq!(LogicStatement::unit_radar().category().name, "unit");
        assert_eq!(LogicStatement::unit_locate().category().name, "unit");
        assert_eq!(LogicStatement::query().category().name, "world");
        assert_eq!(LogicStatement::get_block().category().name, "world");
        assert_eq!(LogicStatement::set_block().category().name, "world");
        assert_eq!(LogicStatement::spawn_unit().category().name, "world");
        assert_eq!(LogicStatement::apply_status().category().name, "world");
        assert_eq!(LogicStatement::spawn_wave().category().name, "world");
        assert_eq!(LogicStatement::spawn_bullet().category().name, "world");
        assert_eq!(LogicStatement::weather_sense().category().name, "world");
        assert_eq!(LogicStatement::weather_set().category().name, "world");
        assert_eq!(LogicStatement::effect().category().name, "world");
        assert_eq!(LogicStatement::explosion().category().name, "world");
        assert_eq!(LogicStatement::set_rule().category().name, "world");
        assert_eq!(LogicStatement::fetch().category().name, "world");
        assert_eq!(LogicStatement::get_flag().category().name, "world");
        assert_eq!(LogicStatement::set_flag().category().name, "world");
        assert_eq!(LogicStatement::set_prop().category().name, "world");
        assert_eq!(LogicStatement::flush_message().category().name, "world");
        assert_eq!(LogicStatement::cutscene().category().name, "world");
        assert_eq!(LogicStatement::client_data().category().name, "world");
        assert_eq!(LogicStatement::play_sound().category().name, "world");
        assert_eq!(LogicStatement::set_marker().category().name, "world");
        assert_eq!(LogicStatement::make_marker().category().name, "world");
        assert!(!LogicStatement::read().privileged());
        assert!(!LogicStatement::draw().privileged());
        assert!(!LogicStatement::print_char().privileged());
        assert!(!LogicStatement::operation().privileged());
        assert!(!LogicStatement::select().privileged());
        assert!(!LogicStatement::stop().privileged());
        assert!(!LogicStatement::lookup().privileged());
        assert!(!LogicStatement::jump().privileged());
        assert!(!LogicStatement::sensor().privileged());
        assert!(!LogicStatement::unit_bind().privileged());
        assert!(!LogicStatement::unit_control().privileged());
        assert!(!LogicStatement::unit_radar().privileged());
        assert!(!LogicStatement::unit_locate().privileged());
        assert!(LogicStatement::query().privileged());
        assert!(LogicStatement::set_rate().privileged());
        assert!(LogicStatement::sync().privileged());
        assert!(LogicStatement::locale_print().privileged());
        assert!(LogicStatement::get_block().privileged());
        assert!(LogicStatement::set_block().privileged());
        assert!(LogicStatement::spawn_unit().privileged());
        assert!(LogicStatement::apply_status().privileged());
        assert!(LogicStatement::spawn_wave().privileged());
        assert!(LogicStatement::spawn_bullet().privileged());
        assert!(LogicStatement::weather_sense().privileged());
        assert!(LogicStatement::weather_set().privileged());
        assert!(LogicStatement::effect().privileged());
        assert!(LogicStatement::explosion().privileged());
        assert!(LogicStatement::set_rule().privileged());
        assert!(LogicStatement::fetch().privileged());
        assert!(LogicStatement::get_flag().privileged());
        assert!(LogicStatement::set_flag().privileged());
        assert!(LogicStatement::set_prop().privileged());
        assert!(LogicStatement::flush_message().privileged());
        assert!(LogicStatement::cutscene().privileged());
        assert!(LogicStatement::client_data().privileged());
        assert!(LogicStatement::play_sound().privileged());
        assert!(LogicStatement::set_marker().privileged());
        assert!(LogicStatement::make_marker().privileged());
    }

    #[test]
    fn simple_logic_statements_parse_tokens_like_generated_logic_io() {
        assert_eq!(
            LogicStatement::read_tokens(&["read", "out", "cell2", "7"].map(String::from)),
            Some(LogicStatement::Read {
                output: "out".into(),
                target: "cell2".into(),
                address: "7".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["read", "out"].map(String::from)),
            Some(LogicStatement::Read {
                output: "out".into(),
                target: "cell1".into(),
                address: "0".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["write", "value", "cell3", "2"].map(String::from))
                .unwrap()
                .write_line(),
            "write value cell3 2"
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["draw", "line", "1", "2", "3", "4", "5", "6"].map(String::from)
            ),
            Some(LogicStatement::Draw {
                type_: GraphicsType::Line,
                x: "1".into(),
                y: "2".into(),
                p1: "3".into(),
                p2: "4".into(),
                p3: "5".into(),
                p4: "6".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["draw", "color", "1", "2", "3", "0"].map(String::from))
                .unwrap()
                .write_line(),
            "draw color 1 2 3 255 0 0"
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["draw", "print", "1", "2", "bottomLeft"].map(String::from)
            )
            .unwrap()
            .write_line(),
            "draw print 1 2 @bottomLeft 0 0 0"
        );
        assert_eq!(
            LogicStatement::read_tokens(&["draw", "missing"].map(String::from)),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(&["print", "\"hello world\""].map(String::from))
                .unwrap()
                .tokens(),
            vec!["print".to_string(), "\"hello world\"".to_string()]
        );
        assert_eq!(
            LogicStatement::read_tokens(&["printchar", "97"].map(String::from)),
            Some(LogicStatement::PrintChar { value: "97".into() })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["format", "\"x=%d\""].map(String::from)),
            Some(LogicStatement::Format {
                value: "\"x=%d\"".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["localeprint", "\"unit.name\""].map(String::from)),
            Some(LogicStatement::LocalePrint {
                value: "\"unit.name\"".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["drawflush", "display2"].map(String::from)),
            Some(LogicStatement::DrawFlush {
                target: "display2".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["printflush", "message2"].map(String::from)),
            Some(LogicStatement::PrintFlush {
                target: "message2".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["getlink", "linked", "3"].map(String::from)),
            Some(LogicStatement::GetLink {
                output: "linked".into(),
                address: "3".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["setrate", "20"].map(String::from)),
            Some(LogicStatement::SetRate {
                amount: "20".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["sync", "flag"].map(String::from)),
            Some(LogicStatement::Sync {
                variable: "flag".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["set", "x"].map(String::from)),
            Some(LogicStatement::Set {
                to: "x".into(),
                from: "0".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["op", "lessThanEq", "out", "a", "b"].map(String::from)),
            Some(LogicStatement::Operation {
                op: LogicOp::LessThanEq,
                dest: "out".into(),
                a: "a".into(),
                b: "b".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["op", "angleDiff", "result", "x"].map(String::from)),
            Some(LogicStatement::Operation {
                op: LogicOp::AngleDiff,
                dest: "result".into(),
                a: "x".into(),
                b: "b".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["op", "missing", "out", "a", "b"].map(String::from)),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &[
                    "select",
                    "out",
                    "greaterThanEq",
                    "hp",
                    "10",
                    "alive",
                    "dead"
                ]
                .map(String::from)
            ),
            Some(LogicStatement::Select {
                result: "out".into(),
                op: ConditionOp::GreaterThanEq,
                comp0: "hp".into(),
                comp1: "10".into(),
                a: "alive".into(),
                b: "dead".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["select", "out"].map(String::from)),
            Some(LogicStatement::Select {
                result: "out".into(),
                op: ConditionOp::NotEqual,
                comp0: "x".into(),
                comp1: "false".into(),
                a: "a".into(),
                b: "b".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["select", "out", "missing"].map(String::from)),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(&["wait", "2"].map(String::from)),
            Some(LogicStatement::Wait { value: "2".into() })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["stop".into()]),
            Some(LogicStatement::Stop)
        );
        assert_eq!(
            LogicStatement::read_tokens(&["end".into()]),
            Some(LogicStatement::End)
        );
        assert_eq!(
            LogicStatement::read_tokens(&["packcolor", "c", "0.1", "0.2"].map(String::from)),
            Some(LogicStatement::PackColor {
                result: "c".into(),
                r: "0.1".into(),
                g: "0.2".into(),
                b: "0".into(),
                a: "1".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["unpackcolor", "red", "green", "blue", "alpha", "packed"].map(String::from)
            ),
            Some(LogicStatement::UnpackColor {
                r: "red".into(),
                g: "green".into(),
                b: "blue".into(),
                a: "alpha".into(),
                value: "packed".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["lookup", "block", "out", "7"].map(String::from)),
            Some(LogicStatement::Lookup {
                type_: ContentType::Block,
                result: "out".into(),
                id: "7".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["lookup", "bullet", "out", "7"].map(String::from)),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["jump", "-1", "always", "ignored", "alsoIgnored"].map(String::from)
            ),
            Some(LogicStatement::Jump {
                dest_index: -1,
                op: ConditionOp::Always,
                value: "ignored".into(),
                compare: "alsoIgnored".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["control", "shootp", "turret", "@unit", "true"].map(String::from)
            ),
            Some(LogicStatement::Control {
                type_: LAccess::Shootp,
                target: "turret".into(),
                p1: "@unit".into(),
                p2: "true".into(),
                p3: "0".into(),
                p4: "0".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["control", "health", "block1", "1"].map(String::from)),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &[
                    "radar",
                    "enemy",
                    "flying",
                    "boss",
                    "maxHealth",
                    "radar1",
                    "-1",
                    "target"
                ]
                .map(String::from)
            ),
            Some(LogicStatement::Radar {
                target1: RadarTarget::Enemy,
                target2: RadarTarget::Flying,
                target3: RadarTarget::Boss,
                sort: RadarSort::MaxHealth,
                radar: "radar1".into(),
                sort_order: "-1".into(),
                output: "target".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["sensor", "out", "turret", "@health"].map(String::from)),
            Some(LogicStatement::Sensor {
                to: "out".into(),
                from: "turret".into(),
                type_: "@health".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["ubind", "@dagger"].map(String::from)),
            Some(LogicStatement::UnitBind {
                type_: "@dagger".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["ucontrol", "build", "1", "2", "@copper", "0", "config"].map(String::from)
            ),
            Some(LogicStatement::UnitControl {
                type_: LUnitControl::Build,
                p1: "1".into(),
                p2: "2".into(),
                p3: "@copper".into(),
                p4: "0".into(),
                p5: "config".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &[
                    "uradar",
                    "enemy",
                    "flying",
                    "boss",
                    "maxHealth",
                    "0",
                    "-1",
                    "out"
                ]
                .map(String::from)
            ),
            Some(LogicStatement::UnitRadar {
                target1: RadarTarget::Enemy,
                target2: RadarTarget::Flying,
                target3: RadarTarget::Boss,
                sort: RadarSort::MaxHealth,
                radar: "0".into(),
                sort_order: "-1".into(),
                output: "out".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["ulocate", "ore", "core", "false", "@thorium", "x", "y", "found", "build"]
                    .map(String::from)
            ),
            Some(LogicStatement::UnitLocate {
                locate: LLocate::Ore,
                flag: BlockFlag::Core,
                enemy: "false".into(),
                ore: "@thorium".into(),
                out_x: "x".into(),
                out_y: "y".into(),
                out_found: "found".into(),
                out_build: "build".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &[
                    "ulocate",
                    "building",
                    "launchPad",
                    "true",
                    "@copper",
                    "x",
                    "y",
                    "found",
                    "build"
                ]
                .map(String::from)
            ),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["query", "rect", "building", "@sharded", "1", "2", "3", "4"].map(String::from)
            ),
            Some(LogicStatement::Query {
                shape: QueryShape::Rect,
                type_: QueryType::Building,
                team: "@sharded".into(),
                x: "1".into(),
                y: "2".into(),
                w: "3".into(),
                h: "4".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["query", "circle", "bullet"].map(String::from)),
            Some(LogicStatement::Query {
                shape: QueryShape::Circle,
                type_: QueryType::Bullet,
                team: "null".into(),
                x: "0".into(),
                y: "0".into(),
                w: "10".into(),
                h: "10".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["query", "missing"].map(String::from)),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(&["getblock", "ore", "oreOut", "4", "5"].map(String::from)),
            Some(LogicStatement::GetBlock {
                layer: TileLayer::Ore,
                result: "oreOut".into(),
                x: "4".into(),
                y: "5".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["getblock", "floor"].map(String::from)),
            Some(LogicStatement::GetBlock {
                layer: TileLayer::Floor,
                result: "result".into(),
                x: "0".into(),
                y: "0".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["getblock", "missing"].map(String::from)),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["setblock", "floor", "@sand", "1", "2", "@blue", "3"].map(String::from)
            ),
            Some(LogicStatement::SetBlock {
                layer: TileLayer::Floor,
                block: "@sand".into(),
                x: "1".into(),
                y: "2".into(),
                team: "@blue".into(),
                rotation: "3".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["setblock", "ore", "@thorium"].map(String::from)),
            Some(LogicStatement::SetBlock {
                layer: TileLayer::Ore,
                block: "@thorium".into(),
                x: "0".into(),
                y: "0".into(),
                team: "@derelict".into(),
                rotation: "0".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["spawn", "@flare", "7", "8", "45", "@crux", "spawned"].map(String::from)
            ),
            Some(LogicStatement::SpawnUnit {
                type_: "@flare".into(),
                x: "7".into(),
                y: "8".into(),
                rotation: "45".into(),
                team: "@crux".into(),
                result: "spawned".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["spawn", "@mono"].map(String::from)),
            Some(LogicStatement::SpawnUnit {
                type_: "@mono".into(),
                x: "10".into(),
                y: "10".into(),
                rotation: "90".into(),
                team: "@sharded".into(),
                result: "result".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["status", "true", "burning", "@unit", "3"].map(String::from)
            ),
            Some(LogicStatement::ApplyStatus {
                clear: true,
                effect: "burning".into(),
                unit: "@unit".into(),
                duration: "3".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["status", "1"].map(String::from)),
            Some(LogicStatement::ApplyStatus {
                clear: false,
                effect: "wet".into(),
                unit: "unit".into(),
                duration: "10".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["status", "TRUE"].map(String::from))
                .unwrap()
                .write_line(),
            "status true wet unit 10"
        );
        assert_eq!(
            LogicStatement::read_tokens(&["spawnwave", "20", "30", "true"].map(String::from)),
            Some(LogicStatement::SpawnWave {
                x: "20".into(),
                y: "30".into(),
                natural: "true".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["spawnwave"].map(String::from)),
            Some(LogicStatement::SpawnWave {
                x: "10".into(),
                y: "10".into(),
                natural: "false".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &[
                    "bullet", "out", "@duo", "2", "10", "20", "90", "@sharded", "@unit", "50",
                    "1.5", "0.5", "30", "40"
                ]
                .map(String::from)
            ),
            Some(LogicStatement::SpawnBullet {
                result: "out".into(),
                from: "@duo".into(),
                index: "2".into(),
                x: "10".into(),
                y: "20".into(),
                rotation: "90".into(),
                team: "@sharded".into(),
                owner: "@unit".into(),
                damage: "50".into(),
                velocity_scl: "1.5".into(),
                life_scl: "0.5".into(),
                aim_x: "30".into(),
                aim_y: "40".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["bullet", "out", "@foreshadow"].map(String::from)),
            Some(LogicStatement::SpawnBullet {
                result: "out".into(),
                from: "@foreshadow".into(),
                index: "0".into(),
                x: "x".into(),
                y: "y".into(),
                rotation: "angle".into(),
                team: "null".into(),
                owner: "null".into(),
                damage: "-1".into(),
                velocity_scl: "1".into(),
                life_scl: "1".into(),
                aim_x: "-1".into(),
                aim_y: "-1".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["weathersense", "out", "@sandstorm"].map(String::from)),
            Some(LogicStatement::WeatherSense {
                to: "out".into(),
                weather: "@sandstorm".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["weatherset", "@rain", "false"].map(String::from)),
            Some(LogicStatement::WeatherSet {
                weather: "@rain".into(),
                state: "false".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["effect", "spark", "1", "2", "3", "%ffffff", "payload"].map(String::from)
            ),
            Some(LogicStatement::Effect {
                type_: "spark".into(),
                x: "1".into(),
                y: "2".into(),
                sizerot: "3".into(),
                color: "%ffffff".into(),
                data: "payload".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["effect", "smoke"].map(String::from))
                .unwrap()
                .write_line(),
            "effect smoke 0 0 2 %ffaaff "
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &[
                    "explosion",
                    "@crux",
                    "5",
                    "6",
                    "7",
                    "8",
                    "false",
                    "true",
                    "true",
                    "false"
                ]
                .map(String::from)
            ),
            Some(LogicStatement::Explosion {
                team: "@crux".into(),
                x: "5".into(),
                y: "6".into(),
                radius: "7".into(),
                damage: "8".into(),
                air: "false".into(),
                ground: "true".into(),
                pierce: "true".into(),
                effect: "false".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["setrule", "mapArea", "1", "2", "3", "4", "5"].map(String::from)
            ),
            Some(LogicStatement::SetRule {
                rule: LogicRule::MapArea,
                value: "1".into(),
                p1: "2".into(),
                p2: "3".into(),
                p3: "4".into(),
                p4: "5".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["setrule", "unitCost"].map(String::from)),
            Some(LogicStatement::SetRule {
                rule: LogicRule::UnitCost,
                value: "10".into(),
                p1: "0".into(),
                p2: "0".into(),
                p3: "100".into(),
                p4: "100".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["setrule", "missing"].map(String::from)),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["fetch", "build", "out", "@crux", "2", "@duo"].map(String::from)
            ),
            Some(LogicStatement::Fetch {
                type_: FetchType::Build,
                result: "out".into(),
                team: "@crux".into(),
                index: "2".into(),
                extra: "@duo".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["fetch", "unitCount", "count"].map(String::from)),
            Some(LogicStatement::Fetch {
                type_: FetchType::UnitCount,
                result: "count".into(),
                team: "@sharded".into(),
                index: "0".into(),
                extra: "@conveyor".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["fetch", "missing"].map(String::from)),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(&["getflag", "out", "\"waves\""].map(String::from)),
            Some(LogicStatement::GetFlag {
                result: "out".into(),
                flag: "\"waves\"".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["setflag", "\"done\"", "false"].map(String::from)),
            Some(LogicStatement::SetFlag {
                flag: "\"done\"".into(),
                value: "false".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["setprop", "@health", "@unit", "100"].map(String::from)),
            Some(LogicStatement::SetProp {
                type_: "@health".into(),
                of: "@unit".into(),
                value: "100".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["setprop", "@x"].map(String::from)),
            Some(LogicStatement::SetProp {
                type_: "@x".into(),
                of: "block1".into(),
                value: "0".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["message", "toast", "5", "ok"].map(String::from)),
            Some(LogicStatement::FlushMessage {
                type_: MessageType::Toast,
                duration: "5".into(),
                out_success: "ok".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["message", "mission"].map(String::from)),
            Some(LogicStatement::FlushMessage {
                type_: MessageType::Mission,
                duration: "3".into(),
                out_success: "@wait".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["message", "missing"].map(String::from)),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["cutscene", "zoom", "2.5", "ignoredY", "0.1", "extra"].map(String::from)
            ),
            Some(LogicStatement::Cutscene {
                action: CutsceneAction::Zoom,
                p1: "2.5".into(),
                p2: "ignoredY".into(),
                p3: "0.1".into(),
                p4: "extra".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["cutscene", "missing"].map(String::from)),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["clientdata", "\"chan\"", "\"payload\"", "1"].map(String::from)
            ),
            Some(LogicStatement::ClientData {
                channel: "\"chan\"".into(),
                value: "\"payload\"".into(),
                reliable: "1".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["clientdata", "\"chan\""].map(String::from)),
            Some(LogicStatement::ClientData {
                channel: "\"chan\"".into(),
                value: "\"bar\"".into(),
                reliable: "0".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &[
                    "playsound",
                    "true",
                    "@sfx-explosion",
                    "0.5",
                    "2",
                    "-1",
                    "10",
                    "20",
                    "false"
                ]
                .map(String::from)
            ),
            Some(LogicStatement::PlaySound {
                positional: true,
                id: "@sfx-explosion".into(),
                volume: "0.5".into(),
                pitch: "2".into(),
                pan: "-1".into(),
                x: "10".into(),
                y: "20".into(),
                limit: "false".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["playsound", "1", "@sfx-pew"].map(String::from)),
            Some(LogicStatement::PlaySound {
                positional: false,
                id: "@sfx-pew".into(),
                volume: "1".into(),
                pitch: "1".into(),
                pan: "0".into(),
                x: "@thisx".into(),
                y: "@thisy".into(),
                limit: "true".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["setmarker", "color", "7", "%ff00aa", "0", "0"].map(String::from)
            ),
            Some(LogicStatement::SetMarker {
                type_: LMarkerControl::Color,
                id: "7".into(),
                p1: "%ff00aa".into(),
                p2: "0".into(),
                p3: "0".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["setmarker", "shape"].map(String::from)),
            Some(LogicStatement::SetMarker {
                type_: LMarkerControl::Shape,
                id: "0".into(),
                p1: "0".into(),
                p2: "0".into(),
                p3: "0".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["setmarker", "missing"].map(String::from)),
            None
        );
        assert_eq!(
            LogicStatement::read_tokens(
                &["makemarker", "text", "3", "10", "20", "false"].map(String::from)
            ),
            Some(LogicStatement::MakeMarker {
                type_: "text".into(),
                id: "3".into(),
                x: "10".into(),
                y: "20".into(),
                replace: "false".into()
            })
        );
        assert_eq!(
            LogicStatement::read_tokens(&["makemarker", "line"].map(String::from)),
            Some(LogicStatement::MakeMarker {
                type_: "line".into(),
                id: "0".into(),
                x: "0".into(),
                y: "0".into(),
                replace: "true".into()
            })
        );
        assert_eq!(LogicStatement::read_tokens(&["missing".into()]), None);
    }

    #[test]
    fn statement_sanitize_and_align_tables_match_java_helpers() {
        assert_eq!(sanitize_logic_value(""), "");
        assert_eq!(sanitize_logic_value("\""), "invalid");
        assert_eq!(sanitize_logic_value(";"), "invalid");
        assert_eq!(sanitize_logic_value(" "), "invalid");
        assert_eq!(sanitize_logic_value("a b;c\"d"), "a_bsc'd");
        assert_eq!(sanitize_logic_value("\"a\"b;c\""), "\"a'b;c\"");

        assert_eq!(
            LogicAlign::ALL
                .iter()
                .map(|align| align.wire_name())
                .collect::<Vec<_>>(),
            vec![
                "topLeft",
                "top",
                "topRight",
                "left",
                "center",
                "right",
                "bottomLeft",
                "bottom",
                "bottomRight"
            ]
        );
        assert_eq!(LogicAlign::Center.java_bits(), 1);
        assert_eq!(LogicAlign::Top.java_bits(), 2);
        assert_eq!(LogicAlign::Bottom.java_bits(), 4);
        assert_eq!(LogicAlign::Left.java_bits(), 8);
        assert_eq!(LogicAlign::Right.java_bits(), 16);
        assert_eq!(LogicAlign::TopLeft.java_bits(), 10);
        assert_eq!(LogicAlign::BottomRight.java_bits(), 20);
        assert_eq!(LogicAlign::by_name("topRight"), Some(LogicAlign::TopRight));
        assert_eq!(LogicAlign::by_java_bits(12), Some(LogicAlign::BottomLeft));
        assert!(LogicAlign::Top.is_center_horizontal());
        assert!(!LogicAlign::TopLeft.is_center_horizontal());
        assert!(LogicAlign::Left.is_center_vertical());
        assert!(!LogicAlign::BottomLeft.is_center_vertical());
        assert_eq!(
            LMarkerControl::by_wire_name("drawLayer"),
            Some(LMarkerControl::DrawLayer)
        );
        assert_eq!(LMarkerControl::by_wire_name("missing"), None);
    }

    #[test]
    fn logic_fx_registry_matches_java_order_flags_and_extension_semantics() {
        assert_eq!(LOGIC_EFFECTS.len(), 33);
        assert_eq!(
            logic_effect_names(),
            vec![
                "warn",
                "cross",
                "blockFall",
                "placeBlock",
                "placeBlockSpark",
                "breakBlock",
                "spawn",
                "trail",
                "breakProp",
                "smokeCloud",
                "vapor",
                "hit",
                "hitSquare",
                "shootSmall",
                "shootBig",
                "smokeSmall",
                "smokeBig",
                "smokeColor",
                "smokeSquare",
                "smokeSquareBig",
                "spark",
                "sparkBig",
                "sparkShoot",
                "sparkShootBig",
                "drill",
                "drillBig",
                "lightBlock",
                "explosion",
                "smokePuff",
                "sparkExplosion",
                "crossExplosion",
                "wave",
                "bubble"
            ]
        );

        let block_fall = get_logic_effect("blockFall").unwrap();
        assert_eq!(block_fall.effect, "blockCrash");
        assert_eq!(block_fall.data, Some("Block"));
        assert_eq!(block_fall.bounds, 100.0);
        assert!(!block_fall.size);
        assert!(!block_fall.rotate);
        assert!(!block_fall.color);

        let trail = get_logic_effect("trail").unwrap();
        assert_eq!(trail.effect, "colorTrail");
        assert!(trail.size);
        assert!(trail.color);
        assert!(!trail.rotate);

        let shoot_big = get_logic_effect("shootBig").unwrap();
        assert_eq!(shoot_big.effect, "shootTitan");
        assert!(shoot_big.rotate);
        assert!(shoot_big.color);
        assert!(!shoot_big.size);

        let wave = get_logic_effect("wave").unwrap();
        assert_eq!(wave.effect, "dynamicWave");
        assert!(wave.size);
        assert!(wave.color);

        assert_eq!(get_logic_effect("missing"), None);

        let mut registry = LogicEffectRegistry::new();
        assert_eq!(registry.all().first(), Some(&"warn"));
        assert_eq!(registry.all().last(), Some(&"bubble"));
        registry.add(
            "custom",
            LogicEffectEntry::new("ignored", "customFx")
                .size()
                .rotate()
                .color()
                .bounds(42.0),
        );
        let custom = registry.get("custom").unwrap();
        assert_eq!(custom.name, "custom");
        assert_eq!(custom.effect, "customFx");
        assert!(custom.size && custom.rotate && custom.color);
        assert_eq!(custom.bounds, 42.0);
        assert_eq!(registry.all().last(), Some(&"custom"));

        registry.add("warn", LogicEffectEntry::new("ignored", "replacement"));
        assert_eq!(registry.get("warn").unwrap().name, "warn");
        assert_eq!(registry.get("warn").unwrap().effect, "replacement");
        assert_eq!(registry.all().first(), Some(&"warn"));
    }

    #[test]
    fn logic_canvas_rows_and_jump_normalization_match_lcanvas_helpers() {
        assert!(logic_canvas_use_rows(1079.9, 1.0));
        assert!(!logic_canvas_use_rows(1080.0, 1.0));
        assert!(logic_canvas_use_rows(2159.0, 2.0));
        assert!(!logic_canvas_use_rows(2160.0, 2.0));

        assert_eq!(
            normalize_logic_jump_range(3, Some(8)),
            Some(LogicJumpRange {
                begin: 3,
                end: 8,
                flipped: false
            })
        );
        assert_eq!(
            normalize_logic_jump_range(8, Some(3)),
            Some(LogicJumpRange {
                begin: 3,
                end: 8,
                flipped: true
            })
        );
        assert_eq!(
            normalize_logic_jump_range(5, Some(5)),
            Some(LogicJumpRange {
                begin: 5,
                end: 5,
                flipped: true
            })
        );
        assert_eq!(normalize_logic_jump_range(1, None), None);
        assert_eq!(
            LogicJumpRange::invalid(),
            LogicJumpRange {
                begin: LOGIC_CANVAS_INVALID_JUMP,
                end: LOGIC_CANVAS_INVALID_JUMP,
                flipped: false
            }
        );
    }

    #[test]
    fn logic_canvas_jump_height_assignment_matches_lcanvas_dedup_and_layering() {
        let single = vec![normalize_logic_jump_range(0, Some(3))];
        assert_eq!(
            assign_logic_jump_heights(&single),
            vec![Some(LogicJumpPlacement {
                begin: 0,
                end: 3,
                flipped: false,
                pred_height: 0
            })]
        );

        let disjoint = vec![
            normalize_logic_jump_range(0, Some(2)),
            normalize_logic_jump_range(3, Some(5)),
        ];
        assert_eq!(
            assign_logic_jump_heights(&disjoint)
                .into_iter()
                .map(|placement| placement.unwrap().pred_height)
                .collect::<Vec<_>>(),
            vec![0, 0]
        );

        let nested = vec![
            normalize_logic_jump_range(0, Some(5)),
            normalize_logic_jump_range(1, Some(4)),
        ];
        let nested_heights = assign_logic_jump_heights(&nested)
            .into_iter()
            .map(|placement| placement.unwrap().pred_height)
            .collect::<Vec<_>>();
        assert_eq!(nested_heights, vec![1, 0]);

        // Java reprBefore de-duplicates forward curves by end index, keeping the
        // representative with the smallest begin. Both curves then copy that
        // representative height back.
        let same_forward_end = vec![
            normalize_logic_jump_range(2, Some(6)),
            normalize_logic_jump_range(1, Some(6)),
        ];
        assert_eq!(representative_logic_jumps(&same_forward_end), vec![1]);
        assert_eq!(
            assign_logic_jump_heights(&same_forward_end)
                .into_iter()
                .map(|placement| placement.unwrap().pred_height)
                .collect::<Vec<_>>(),
            vec![0, 0]
        );

        // Java reprAfter de-duplicates backward curves by begin index, keeping the
        // representative with the largest end.
        let same_backward_begin = vec![
            normalize_logic_jump_range(6, Some(2)),
            normalize_logic_jump_range(7, Some(2)),
        ];
        assert_eq!(representative_logic_jumps(&same_backward_begin), vec![1]);
        assert_eq!(
            assign_logic_jump_heights(&same_backward_begin)
                .into_iter()
                .map(|placement| placement.unwrap().pred_height)
                .collect::<Vec<_>>(),
            vec![0, 0]
        );

        let mixed = vec![
            normalize_logic_jump_range(0, Some(4)),
            normalize_logic_jump_range(4, Some(0)),
            None,
            normalize_logic_jump_range(1, Some(3)),
        ];
        let placements = assign_logic_jump_heights(&mixed);
        assert_eq!(placements[2], None);
        assert_eq!(placements[0].unwrap().pred_height, 2);
        assert_eq!(placements[1].unwrap().pred_height, 1);
        assert_eq!(placements[3].unwrap().pred_height, 0);
        assert_eq!(placements[1].unwrap().flipped, true);
    }

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

        var.set_bool(false);
        assert!(!var.is_obj);
        assert_eq!(var.numval, 0.0);
        assert_eq!(var.value(), LVarValue::Number(0.0));

        let mut constant = LVar::with_id_constant("const", 1, true);
        constant.set_num(9.0);
        assert_eq!(constant.numval, 0.0);
        constant.set_obj(Some("ignored".into()));
        assert_eq!(constant.obj(), None);
        constant.set_const_obj(Some("forced".into()));
        assert_eq!(constant.obj(), Some("forced"));
        assert_eq!(constant.to_string(), "const: forced [const]");

        let mut source = LVar::new("source");
        source.set_num(f64::INFINITY);
        let mut target = LVar::new("@counter");
        target.numval = 99.0;
        target.set_from(&source);
        assert!(target.is_obj);
        assert_eq!(target.obj(), None);
        assert_eq!(target.numval, 99.0);

        source.set_obj(None);
        target.numval = 77.0;
        target.set_from(&source);
        assert!(target.is_obj);
        assert_eq!(target.obj(), None);
        assert_eq!(target.numval, 77.0);
        assert_eq!(target.to_string(), "@counter: null");
    }

    #[test]
    fn l_access_matches_java_order_params_and_derived_sets() {
        assert_eq!(LAccess::ALL.len(), 76);
        assert_eq!(LAccess::TotalItems.ordinal(), 0);
        assert_eq!(LAccess::TotalPower.ordinal(), 3);
        assert_eq!(LAccess::Health.ordinal(), 15);
        assert_eq!(LAccess::BulletTime.ordinal(), 70);
        assert_eq!(LAccess::Enabled.ordinal(), 71);
        assert_eq!(LAccess::Color.ordinal(), 75);
        assert_eq!(LAccess::from_ordinal(72), Some(LAccess::Shoot));
        assert_eq!(LAccess::from_ordinal(76), None);
        assert_eq!(LAccess::by_wire_name("enabled"), Some(LAccess::Enabled));
        assert_eq!(
            LAccess::by_wire_name("currentAmmoType"),
            Some(LAccess::CurrentAmmoType)
        );
        assert_eq!(LAccess::by_wire_name("missing"), None);

        assert_eq!(
            &LAccess::ALL[LAccess::ALL.len() - 5..],
            [
                LAccess::Enabled,
                LAccess::Shoot,
                LAccess::Shootp,
                LAccess::Config,
                LAccess::Color
            ]
        );
        assert_eq!(LAccess::VelocityX.wire_name(), "velocityX");
        assert_eq!(LAccess::CurrentAmmoType.wire_name(), "currentAmmoType");
        assert_eq!(LAccess::Shoot.params(), ["x", "y", "shoot"]);
        assert_eq!(LAccess::Shootp.params(), ["unit", "shoot"]);
        assert!(LAccess::Shootp.is_obj());
        assert!(LAccess::Config.is_obj());
        assert!(!LAccess::Shoot.is_obj());

        assert_eq!(
            LAccess::SETTABLE,
            [
                LAccess::X,
                LAccess::Y,
                LAccess::VelocityX,
                LAccess::VelocityY,
                LAccess::Rotation,
                LAccess::Speed,
                LAccess::Armor,
                LAccess::Health,
                LAccess::Shield,
                LAccess::Team,
                LAccess::Flag,
                LAccess::TotalPower,
                LAccess::PayloadType,
                LAccess::BulletTime,
                LAccess::BulletLifetime
            ]
        );

        let expected_senseable: Vec<_> = LAccess::ALL
            .iter()
            .copied()
            .filter(|access| access.params().len() <= 1)
            .collect();
        assert_eq!(LAccess::SENSEABLE.as_slice(), expected_senseable.as_slice());
        assert_eq!(
            LAccess::CONTROLS,
            [
                LAccess::Enabled,
                LAccess::Shoot,
                LAccess::Shootp,
                LAccess::Config,
                LAccess::Color
            ]
        );
        assert!(LAccess::Enabled.is_senseable());
        assert!(!LAccess::Shoot.is_senseable());
        assert!(LAccess::Color.is_control());
        assert!(!LAccess::Health.is_control());
    }

    #[test]
    fn logic_ops_match_java_order_symbols_flags_and_core_math() {
        assert_eq!(LogicOp::ALL.len(), 45);
        assert_eq!(LogicOp::Add.ordinal(), 0);
        assert_eq!(LogicOp::StrictEqual.ordinal(), 15);
        assert_eq!(LogicOp::Not.ordinal(), 22);
        assert_eq!(LogicOp::Atan.ordinal(), 44);
        assert_eq!(LogicOp::from_ordinal(44), Some(LogicOp::Atan));
        assert_eq!(LogicOp::from_ordinal(45), None);
        assert_eq!(LogicOp::Add.java_name(), "add");
        assert_eq!(LogicOp::NotEqual.java_name(), "notEqual");
        assert_eq!(LogicOp::AngleDiff.java_name(), "angleDiff");
        assert_eq!(LogicOp::by_java_name("lessThan"), Some(LogicOp::LessThan));
        assert_eq!(LogicOp::by_java_name("angleDiff"), Some(LogicOp::AngleDiff));
        assert_eq!(LogicOp::by_java_name("+"), None);

        let symbols: Vec<_> = LogicOp::ALL.iter().map(|op| op.symbol()).collect();
        assert_eq!(
            symbols,
            vec![
                "+",
                "-",
                "*",
                "/",
                "//",
                "%",
                "%%",
                "^",
                "==",
                "not",
                "and",
                "<",
                "<=",
                ">",
                ">=",
                "===",
                "<<",
                ">>",
                ">>>",
                "or",
                "b-and",
                "xor",
                "flip",
                "max",
                "min",
                "angle",
                "anglediff",
                "len",
                "noise",
                "abs",
                "sign",
                "log",
                "logn",
                "log10",
                "floor",
                "ceil",
                "round",
                "sqrt",
                "rand",
                "sin",
                "cos",
                "tan",
                "asin",
                "acos",
                "atan"
            ]
        );

        assert!(LogicOp::Not.unary());
        assert!(LogicOp::Sin.unary());
        assert!(!LogicOp::Add.unary());
        assert!(LogicOp::Max.func());
        assert!(LogicOp::Angle.func());
        assert!(!LogicOp::Logn.func());

        assert_eq!(LogicOp::Add.eval_binary(2.0, 3.0), Some(5.0));
        assert_eq!(LogicOp::Idiv.eval_binary(7.0, 2.0), Some(3.0));
        assert_eq!(LogicOp::Emod.eval_binary(-1.0, 5.0), Some(4.0));
        assert_eq!(LogicOp::Equal.eval_binary(1.0, 1.0 + 0.0000005), Some(1.0));
        assert_eq!(LogicOp::Land.eval_binary(1.0, 0.0), Some(0.0));
        assert_eq!(LogicOp::Shl.eval_binary(3.0, 2.0), Some(12.0));
        assert_eq!(LogicOp::And.eval_binary(6.0, 3.0), Some(2.0));
        assert_eq!(LogicOp::Not.eval_unary(0.0), Some(-1.0));
        assert_eq!(LogicOp::Abs.eval_unary(-3.5), Some(3.5));
        assert!((LogicOp::Angle.eval_binary(0.0, 1.0).unwrap() - 90.0).abs() < 0.000001);
        assert!((LogicOp::AngleDiff.eval_binary(350.0, 10.0).unwrap() - 20.0).abs() < 0.000001);
        assert!((LogicOp::Len.eval_binary(3.0, 4.0).unwrap() - 5.0).abs() < 0.000001);
        assert!((LogicOp::Sin.eval_unary(90.0).unwrap() - 1.0).abs() < 0.000001);
        assert_eq!(LogicOp::Noise.eval_binary(1.0, 2.0), None);
        assert_eq!(LogicOp::Rand.eval_unary(10.0), None);
        assert_eq!(LogicOp::Add.to_string(), "+");
    }

    #[test]
    fn basic_logic_executor_instructions_follow_java_l_executor_semantics() {
        let mut set = LogicInstruction::Set {
            from: {
                let mut from = LVar::new("from");
                from.set_num(7.0);
                from
            },
            to: LVar::new("to"),
        };
        set.run(&mut LogicExecutor::new());
        match set {
            LogicInstruction::Set { to, .. } => assert_eq!(to.value(), LVarValue::Number(7.0)),
            _ => unreachable!(),
        }

        let mut op = LogicInstruction::Op {
            op: LogicOp::Add,
            a: {
                let mut value = LVar::new("a");
                value.set_num(2.0);
                value
            },
            b: {
                let mut value = LVar::new("b");
                value.set_num(3.0);
                value
            },
            dest: LVar::new("dest"),
        };
        op.run(&mut LogicExecutor::new());
        match op {
            LogicInstruction::Op { dest, .. } => assert_eq!(dest.value(), LVarValue::Number(5.0)),
            _ => unreachable!(),
        }

        let mut select = LogicInstruction::Select {
            op: ConditionOp::GreaterThan,
            result: LVar::new("result"),
            comp0: {
                let mut value = LVar::new("hp");
                value.set_num(11.0);
                value
            },
            comp1: {
                let mut value = LVar::new("limit");
                value.set_num(10.0);
                value
            },
            a: {
                let mut value = LVar::new("alive");
                value.set_obj(Some("alive".into()));
                value
            },
            b: {
                let mut value = LVar::new("dead");
                value.set_obj(Some("dead".into()));
                value
            },
        };
        select.run(&mut LogicExecutor::new());
        match select {
            LogicInstruction::Select { result, .. } => {
                assert_eq!(result.value(), LVarValue::Object(Some("alive".into())));
            }
            _ => unreachable!(),
        }

        assert!(logic_var_strict_equal(
            &{
                let mut value = LVar::new("a");
                value.set_obj(Some("same".into()));
                value
            },
            &{
                let mut value = LVar::new("b");
                value.set_obj(Some("same".into()));
                value
            }
        ));
        assert!(!condition_op_test_vars(
            ConditionOp::Equal,
            &{
                let mut value = LVar::new("a");
                value.set_obj(Some("1".into()));
                value
            },
            &{
                let mut value = LVar::new("b");
                value.set_num(1.0);
                value
            }
        ));
    }

    #[test]
    fn text_and_flow_logic_executor_instructions_follow_java_l_executor_semantics() {
        let mut exec = LogicExecutor::new();
        LogicInstruction::Print {
            value: {
                let mut value = LVar::new("n");
                value.set_num(4.0);
                value
            },
        }
        .run(&mut exec);
        LogicInstruction::Print {
            value: {
                let mut value = LVar::new("s");
                value.set_obj(Some(" frogs".into()));
                value
            },
        }
        .run(&mut exec);
        LogicInstruction::PrintChar {
            value: {
                let mut value = LVar::new("bang");
                value.set_num(33.0);
                value
            },
        }
        .run(&mut exec);
        assert_eq!(exec.text_buffer, "4 frogs!");

        exec.text_buffer = "{1} before {0}".into();
        LogicInstruction::Format {
            value: {
                let mut value = LVar::new("value");
                value.set_obj(Some("first".into()));
                value
            },
        }
        .run(&mut exec);
        assert_eq!(exec.text_buffer, "{1} before first");

        assert_eq!(
            print_logic_value(&{
                let mut value = LVar::new("fraction");
                value.set_num(1.25);
                value
            }),
            "1.25"
        );
        assert_eq!(first_logic_placeholder("x {3} {1} {2}"), Some((6, 1)));

        let mut jump_exec = LogicExecutor::new();
        jump_exec.counter.set_num(1.0);
        LogicInstruction::Jump {
            op: ConditionOp::Always,
            value: LVar::new("a"),
            compare: LVar::new("b"),
            address: 5,
        }
        .run(&mut jump_exec);
        assert_eq!(jump_exec.counter.numval, 5.0);

        let mut stop_exec = LogicExecutor::new();
        stop_exec.counter.set_num(3.0);
        LogicInstruction::Stop.run(&mut stop_exec);
        assert_eq!(stop_exec.counter.numval, 2.0);
        assert!(stop_exec.yield_);

        let mut wait = LogicInstruction::Wait {
            value: {
                let mut value = LVar::new("seconds");
                value.set_num(0.5);
                value
            },
            cur_time: 0.0,
        };
        let mut wait_exec = LogicExecutor::new();
        wait_exec.counter.set_num(2.0);
        wait.run(&mut wait_exec);
        assert_eq!(wait_exec.counter.numval, 1.0);
        assert!(wait_exec.yield_);
        match wait {
            LogicInstruction::Wait { cur_time, .. } => {
                assert!((cur_time - 1.0 / 60.0).abs() < 0.000001)
            }
            _ => unreachable!(),
        }

        let mut end_exec = LogicExecutor {
            instructions: vec![LogicInstruction::Noop, LogicInstruction::Noop],
            ..LogicExecutor::new()
        };
        LogicInstruction::End.run(&mut end_exec);
        assert_eq!(end_exec.counter.numval, 2.0);
    }

    #[test]
    fn display_command_packing_and_draw_instruction_follow_java_logic_display_bits() {
        let packed = LogicDisplayCommand::get(4, 1, 2, 3, 4, 5, 6);
        assert_eq!(
            LogicDisplayCommand::unpack(packed),
            LogicDisplayCommand {
                type_: 4,
                x: 1,
                y: 2,
                p1: 3,
                p2: 4,
                p3: 5,
                p4: 6
            }
        );
        assert_eq!(LogicDisplayCommand::pack(1025), 1);
        assert_eq!(LogicDisplayCommand::pack_sign(-12), 0b1000001100);
        assert_eq!(LogicDisplayCommand::unpack_sign(0b1000001100), -12);

        let mut exec = LogicExecutor::new();
        let mut draw = LogicInstruction::Draw {
            type_: GraphicsType::Line,
            x: {
                let mut value = LVar::new("x");
                value.set_num(-1.0);
                value
            },
            y: {
                let mut value = LVar::new("y");
                value.set_num(2.0);
                value
            },
            p1: {
                let mut value = LVar::new("p1");
                value.set_num(3.0);
                value
            },
            p2: {
                let mut value = LVar::new("p2");
                value.set_num(4.0);
                value
            },
            p3: {
                let mut value = LVar::new("p3");
                value.set_num(5.0);
                value
            },
            p4: {
                let mut value = LVar::new("p4");
                value.set_num(-6.0);
                value
            },
        };
        draw.run(&mut exec);
        assert_eq!(exec.graphics_buffer.len(), 1);
        let command = LogicDisplayCommand::unpack(exec.graphics_buffer[0]);
        assert_eq!(command.type_, GraphicsType::Line.ordinal());
        assert_eq!(LogicDisplayCommand::unpack_sign(command.x), -1);
        assert_eq!(LogicDisplayCommand::unpack_sign(command.y), 2);
        assert_eq!(LogicDisplayCommand::unpack_sign(command.p1), 3);
        assert_eq!(LogicDisplayCommand::unpack_sign(command.p4), -6);

        let mut color_exec = LogicExecutor::new();
        LogicInstruction::Draw {
            type_: GraphicsType::Col,
            x: {
                let mut value = LVar::new("packed");
                value.set_num(rgba_to_double_bits(0xff, 0x00, 0xaa, 0x80));
                value
            },
            y: LVar::new("y"),
            p1: LVar::new("p1"),
            p2: LVar::new("p2"),
            p3: LVar::new("p3"),
            p4: LVar::new("p4"),
        }
        .run(&mut color_exec);
        let color = LogicDisplayCommand::unpack(color_exec.graphics_buffer[0]);
        assert_eq!(color.type_, GraphicsType::Color.ordinal());
        assert_eq!((color.x, color.y, color.p1, color.p2), (255, 0, 170, 128));
    }

    #[test]
    fn color_and_lookup_executor_instructions_follow_java_l_executor_semantics() {
        let mut pack = LogicInstruction::PackColor {
            result: LVar::new("result"),
            r: {
                let mut value = LVar::new("r");
                value.set_num(1.0);
                value
            },
            g: {
                let mut value = LVar::new("g");
                value.set_num(0.0);
                value
            },
            b: {
                let mut value = LVar::new("b");
                value.set_num(0.5);
                value
            },
            a: {
                let mut value = LVar::new("a");
                value.set_num(2.0);
                value
            },
        };
        pack.run(&mut LogicExecutor::new());
        let packed = match pack {
            LogicInstruction::PackColor { result, .. } => {
                assert_eq!(double_bits_to_rgba(result.numval), 0xff007fff);
                result.numval
            }
            _ => unreachable!(),
        };

        let mut unpack = LogicInstruction::UnpackColor {
            r: LVar::new("r"),
            g: LVar::new("g"),
            b: LVar::new("b"),
            a: LVar::new("a"),
            value: {
                let mut value = LVar::new("value");
                value.set_num(packed);
                value
            },
        };
        unpack.run(&mut LogicExecutor::new());
        match unpack {
            LogicInstruction::UnpackColor { r, g, b, a, .. } => {
                assert_eq!(r.value(), LVarValue::Number(1.0));
                assert_eq!(g.value(), LVarValue::Number(0.0));
                assert!((b.numval - 127.0 / 255.0).abs() < 0.000001);
                assert_eq!(a.value(), LVarValue::Number(1.0));
            }
            _ => unreachable!(),
        }

        assert_eq!(
            lookup_logic_content_name(ContentType::Item, 0),
            Some("copper")
        );
        assert_eq!(
            lookup_logic_content_name(ContentType::Liquid, 0),
            Some("water")
        );
        assert_eq!(
            lookup_logic_content_name(ContentType::Status, 0),
            Some("none")
        );
        assert_eq!(lookup_logic_content_name(ContentType::Item, -1), None);

        let mut lookup = LogicInstruction::Lookup {
            dest: LVar::new("dest"),
            from: {
                let mut value = LVar::new("from");
                value.set_num(0.0);
                value
            },
            type_: ContentType::Item,
        };
        lookup.run(&mut LogicExecutor::new());
        match lookup {
            LogicInstruction::Lookup { dest, .. } => {
                assert_eq!(dest.value(), LVarValue::Object(Some("copper".into())));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn linked_read_write_and_sense_executor_instructions_follow_java_l_executor_semantics() {
        let mut exec = LogicExecutor::new();
        exec.team = 1;
        exec.links = vec!["cell1".into(), "message1".into()];
        exec.register_object(
            "cell1",
            LogicRuntimeObject::Memory(LogicMemoryObject::new(4, 1)),
        );
        exec.register_object(
            "enemy-cell",
            LogicRuntimeObject::Memory(LogicMemoryObject::new(2, 2)),
        );
        exec.register_object("message1", LogicRuntimeObject::Text("ab💥".into()));
        exec.register_object(
            "seq1",
            LogicRuntimeObject::Sequence(vec![
                LVarValue::Object(Some("copper".into())),
                LVarValue::Number(7.0),
            ]),
        );
        let mut sensor = LogicSenseObject::default();
        sensor.numeric_senses.insert(LAccess::Health, 12.5);
        sensor
            .object_senses
            .insert(LAccess::CurrentAmmoType, Some("copper".into()));
        sensor.content_senses.insert("copper".into(), 42.0);
        exec.register_object("turret", LogicRuntimeObject::Senseable(sensor));

        let mut getlink = LogicInstruction::GetLink {
            output: LVar::new("out"),
            index: {
                let mut value = LVar::new("index");
                value.set_num(1.0);
                value
            },
        };
        getlink.run(&mut exec);
        match getlink {
            LogicInstruction::GetLink { output, .. } => {
                assert_eq!(output.value(), LVarValue::Object(Some("message1".into())));
            }
            _ => unreachable!(),
        }

        let mut missing_link = LogicInstruction::GetLink {
            output: {
                let mut value = LVar::new("out");
                value.set_obj(Some("old".into()));
                value
            },
            index: {
                let mut value = LVar::new("index");
                value.set_num(-1.0);
                value
            },
        };
        missing_link.run(&mut exec);
        match missing_link {
            LogicInstruction::GetLink { output, .. } => {
                assert_eq!(output.value(), LVarValue::Object(None));
            }
            _ => unreachable!(),
        }

        let mut write = LogicInstruction::Write {
            target: {
                let mut value = LVar::new("target");
                value.set_obj(Some("cell1".into()));
                value
            },
            position: {
                let mut value = LVar::new("position");
                value.set_num(2.0);
                value
            },
            value: {
                let mut value = LVar::new("value");
                value.set_num(9.0);
                value
            },
        };
        write.run(&mut exec);
        assert_eq!(
            match exec.objects.get("cell1").unwrap() {
                LogicRuntimeObject::Memory(memory) => memory.memory[2],
                _ => unreachable!(),
            },
            9.0
        );

        let mut read_memory = LogicInstruction::Read {
            target: {
                let mut value = LVar::new("target");
                value.set_obj(Some("cell1".into()));
                value
            },
            position: {
                let mut value = LVar::new("position");
                value.set_num(2.0);
                value
            },
            output: LVar::new("output"),
        };
        read_memory.run(&mut exec);
        match read_memory {
            LogicInstruction::Read { output, .. } => {
                assert_eq!(output.value(), LVarValue::Number(9.0));
            }
            _ => unreachable!(),
        }

        let mut read_oob = LogicInstruction::Read {
            target: {
                let mut value = LVar::new("target");
                value.set_obj(Some("cell1".into()));
                value
            },
            position: {
                let mut value = LVar::new("position");
                value.set_num(99.0);
                value
            },
            output: LVar::new("output"),
        };
        read_oob.run(&mut exec);
        match read_oob {
            LogicInstruction::Read { output, .. } => {
                assert_eq!(output.value(), LVarValue::Object(None));
            }
            _ => unreachable!(),
        }

        let mut read_denied = LogicInstruction::Read {
            target: {
                let mut value = LVar::new("target");
                value.set_obj(Some("enemy-cell".into()));
                value
            },
            position: {
                let mut value = LVar::new("position");
                value.set_num(0.0);
                value
            },
            output: {
                let mut value = LVar::new("output");
                value.set_num(1.0);
                value
            },
        };
        read_denied.run(&mut exec);
        match read_denied {
            LogicInstruction::Read { output, .. } => {
                assert_eq!(output.value(), LVarValue::Object(None));
            }
            _ => unreachable!(),
        }

        let mut read_text = LogicInstruction::Read {
            target: {
                let mut value = LVar::new("target");
                value.set_obj(Some("message1".into()));
                value
            },
            position: {
                let mut value = LVar::new("position");
                value.set_num(1.0);
                value
            },
            output: LVar::new("output"),
        };
        read_text.run(&mut exec);
        match read_text {
            LogicInstruction::Read { output, .. } => {
                assert_eq!(output.value(), LVarValue::Number('b' as u32 as f64));
            }
            _ => unreachable!(),
        }

        let mut read_text_oob = LogicInstruction::Read {
            target: {
                let mut value = LVar::new("target");
                value.set_obj(Some("message1".into()));
                value
            },
            position: {
                let mut value = LVar::new("position");
                value.set_num(4.0);
                value
            },
            output: LVar::new("output"),
        };
        read_text_oob.run(&mut exec);
        match read_text_oob {
            LogicInstruction::Read { output, .. } => {
                assert_eq!(output.value(), LVarValue::Object(None));
            }
            _ => unreachable!(),
        }

        let mut read_seq = LogicInstruction::Read {
            target: {
                let mut value = LVar::new("target");
                value.set_obj(Some("seq1".into()));
                value
            },
            position: {
                let mut value = LVar::new("position");
                value.set_num(0.0);
                value
            },
            output: LVar::new("output"),
        };
        read_seq.run(&mut exec);
        match read_seq {
            LogicInstruction::Read { output, .. } => {
                assert_eq!(output.value(), LVarValue::Object(Some("copper".into())));
            }
            _ => unreachable!(),
        }

        let mut sense_dead = LogicInstruction::Sense {
            from: {
                let mut value = LVar::new("from");
                value.set_obj(None);
                value
            },
            to: LVar::new("to"),
            type_: {
                let mut value = LVar::new("type");
                value.set_obj(Some("@dead".into()));
                value
            },
        };
        sense_dead.run(&mut exec);
        match sense_dead {
            LogicInstruction::Sense { to, .. } => {
                assert_eq!(to.value(), LVarValue::Number(1.0));
            }
            _ => unreachable!(),
        }

        let mut sense_text_size = LogicInstruction::Sense {
            from: {
                let mut value = LVar::new("from");
                value.set_obj(Some("message1".into()));
                value
            },
            to: LVar::new("to"),
            type_: {
                let mut value = LVar::new("type");
                value.set_obj(Some("@size".into()));
                value
            },
        };
        sense_text_size.run(&mut exec);
        match sense_text_size {
            LogicInstruction::Sense { to, .. } => {
                assert_eq!(to.value(), LVarValue::Number(4.0));
            }
            _ => unreachable!(),
        }

        let mut sense_memory_capacity = LogicInstruction::Sense {
            from: {
                let mut value = LVar::new("from");
                value.set_obj(Some("cell1".into()));
                value
            },
            to: LVar::new("to"),
            type_: {
                let mut value = LVar::new("type");
                value.set_obj(Some("@memoryCapacity".into()));
                value
            },
        };
        sense_memory_capacity.run(&mut exec);
        match sense_memory_capacity {
            LogicInstruction::Sense { to, .. } => {
                assert_eq!(to.value(), LVarValue::Number(4.0));
            }
            _ => unreachable!(),
        }

        let mut sense_numeric = LogicInstruction::Sense {
            from: {
                let mut value = LVar::new("from");
                value.set_obj(Some("turret".into()));
                value
            },
            to: LVar::new("to"),
            type_: {
                let mut value = LVar::new("type");
                value.set_obj(Some("@health".into()));
                value
            },
        };
        sense_numeric.run(&mut exec);
        match sense_numeric {
            LogicInstruction::Sense { to, .. } => {
                assert_eq!(to.value(), LVarValue::Number(12.5));
            }
            _ => unreachable!(),
        }

        let mut sense_object = LogicInstruction::Sense {
            from: {
                let mut value = LVar::new("from");
                value.set_obj(Some("turret".into()));
                value
            },
            to: LVar::new("to"),
            type_: {
                let mut value = LVar::new("type");
                value.set_obj(Some("@currentAmmoType".into()));
                value
            },
        };
        sense_object.run(&mut exec);
        match sense_object {
            LogicInstruction::Sense { to, .. } => {
                assert_eq!(to.value(), LVarValue::Object(Some("copper".into())));
            }
            _ => unreachable!(),
        }

        let mut sense_content = LogicInstruction::Sense {
            from: {
                let mut value = LVar::new("from");
                value.set_obj(Some("turret".into()));
                value
            },
            to: LVar::new("to"),
            type_: {
                let mut value = LVar::new("type");
                value.set_obj(Some("@copper".into()));
                value
            },
        };
        sense_content.run(&mut exec);
        match sense_content {
            LogicInstruction::Sense { to, .. } => {
                assert_eq!(to.value(), LVarValue::Number(42.0));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn control_and_radar_executor_instructions_follow_java_l_executor_semantics() {
        let mut exec = LogicExecutor::new();
        exec.team = 1;
        exec.register_object(
            "switch1",
            LogicRuntimeObject::Controllable(LogicControllableObject::new(1)),
        );
        exec.register_object(
            "turret1",
            LogicRuntimeObject::RadarSource(LogicRadarSource::new(0.0, 0.0, 1, 10.0)),
        );

        let mut enable = LogicInstruction::Control {
            type_: LAccess::Enabled,
            target: {
                let mut value = LVar::new("target");
                value.set_obj(Some("switch1".into()));
                value
            },
            p1: {
                let mut value = LVar::new("p1");
                value.set_num(1.0);
                value
            },
            p2: LVar::new("p2"),
            p3: LVar::new("p3"),
            p4: LVar::new("p4"),
        };
        enable.run(&mut exec);
        match exec.objects.get("switch1").unwrap() {
            LogicRuntimeObject::Controllable(control) => {
                assert!(control.enabled);
                assert_eq!(control.no_sleep_calls, 1);
                assert_eq!(
                    control.calls,
                    vec![LogicControlCall::Numeric {
                        access: LAccess::Enabled,
                        p1: 1.0,
                        p2: 0.0,
                        p3: 0.0,
                        p4: 0.0
                    }]
                );
            }
            _ => unreachable!(),
        }

        LogicInstruction::Control {
            type_: LAccess::Config,
            target: {
                let mut value = LVar::new("target");
                value.set_obj(Some("switch1".into()));
                value
            },
            p1: {
                let mut value = LVar::new("p1");
                value.set_obj(Some("copper".into()));
                value
            },
            p2: {
                let mut value = LVar::new("p2");
                value.set_num(2.0);
                value
            },
            p3: LVar::new("p3"),
            p4: LVar::new("p4"),
        }
        .run(&mut exec);
        match exec.objects.get("switch1").unwrap() {
            LogicRuntimeObject::Controllable(control) => {
                assert_eq!(
                    control.calls.last(),
                    Some(&LogicControlCall::Object {
                        access: LAccess::Config,
                        p1: Some("copper".into()),
                        p2: 2.0,
                        p3: 0.0,
                        p4: 0.0
                    })
                );
            }
            _ => unreachable!(),
        }

        if let Some(LogicRuntimeObject::Controllable(control)) = exec.objects.get_mut("switch1") {
            control.valid_link = false;
        }
        LogicInstruction::Control {
            type_: LAccess::Enabled,
            target: {
                let mut value = LVar::new("target");
                value.set_obj(Some("switch1".into()));
                value
            },
            p1: {
                let mut value = LVar::new("p1");
                value.set_num(0.0);
                value
            },
            p2: LVar::new("p2"),
            p3: LVar::new("p3"),
            p4: LVar::new("p4"),
        }
        .run(&mut exec);
        match exec.objects.get("switch1").unwrap() {
            LogicRuntimeObject::Controllable(control) => {
                assert!(control.enabled);
                assert!(!control.disabled_by_processor);
                assert_eq!(control.calls.len(), 2);
            }
            _ => unreachable!(),
        }

        let mut close_enemy = RadarUnitView::new(3.0, 0.0, 2);
        close_enemy.health = 10.0;
        close_enemy.is_grounded = true;
        let mut far_enemy = RadarUnitView::new(8.0, 0.0, 2);
        far_enemy.health = 100.0;
        far_enemy.is_grounded = true;
        let mut ally = RadarUnitView::new(1.0, 0.0, 1);
        ally.health = 1000.0;
        ally.is_grounded = true;
        let mut out_of_range = RadarUnitView::new(20.0, 0.0, 2);
        out_of_range.health = 5000.0;
        out_of_range.is_grounded = true;
        exec.register_radar_unit("close", close_enemy);
        exec.register_radar_unit("far", far_enemy);
        exec.register_radar_unit("ally", ally);
        exec.register_radar_unit("outside", out_of_range);

        let mut radar_nearest = LogicInstruction::Radar {
            target1: RadarTarget::Enemy,
            target2: RadarTarget::Ground,
            target3: RadarTarget::Any,
            sort: RadarSort::Distance,
            radar: {
                let mut value = LVar::new("radar");
                value.set_obj(Some("turret1".into()));
                value
            },
            sort_order: {
                let mut value = LVar::new("sort");
                value.set_num(1.0);
                value
            },
            output: LVar::new("output"),
            last_target: None,
        };
        radar_nearest.run(&mut exec);
        match radar_nearest {
            LogicInstruction::Radar {
                output,
                last_target,
                ..
            } => {
                assert_eq!(output.value(), LVarValue::Object(Some("close".into())));
                assert_eq!(last_target, Some("close".into()));
            }
            _ => unreachable!(),
        }

        let mut radar_farthest = LogicInstruction::Radar {
            target1: RadarTarget::Enemy,
            target2: RadarTarget::Any,
            target3: RadarTarget::Any,
            sort: RadarSort::Distance,
            radar: {
                let mut value = LVar::new("radar");
                value.set_obj(Some("turret1".into()));
                value
            },
            sort_order: {
                let mut value = LVar::new("sort");
                value.set_num(0.0);
                value
            },
            output: LVar::new("output"),
            last_target: None,
        };
        radar_farthest.run(&mut exec);
        match radar_farthest {
            LogicInstruction::Radar { output, .. } => {
                assert_eq!(output.value(), LVarValue::Object(Some("far".into())));
            }
            _ => unreachable!(),
        }

        let mut radar_health = LogicInstruction::Radar {
            target1: RadarTarget::Enemy,
            target2: RadarTarget::Any,
            target3: RadarTarget::Any,
            sort: RadarSort::Health,
            radar: {
                let mut value = LVar::new("radar");
                value.set_obj(Some("turret1".into()));
                value
            },
            sort_order: {
                let mut value = LVar::new("sort");
                value.set_num(1.0);
                value
            },
            output: LVar::new("output"),
            last_target: None,
        };
        radar_health.run(&mut exec);
        match radar_health {
            LogicInstruction::Radar { output, .. } => {
                assert_eq!(output.value(), LVarValue::Object(Some("far".into())));
            }
            _ => unreachable!(),
        }

        if let Some(LogicRuntimeObject::RadarSource(source)) = exec.objects.get_mut("turret1") {
            source.team = 2;
        }
        let mut denied_radar = LogicInstruction::Radar {
            target1: RadarTarget::Enemy,
            target2: RadarTarget::Any,
            target3: RadarTarget::Any,
            sort: RadarSort::Health,
            radar: {
                let mut value = LVar::new("radar");
                value.set_obj(Some("turret1".into()));
                value
            },
            sort_order: {
                let mut value = LVar::new("sort");
                value.set_num(1.0);
                value
            },
            output: {
                let mut value = LVar::new("output");
                value.set_obj(Some("old".into()));
                value
            },
            last_target: Some("old".into()),
        };
        denied_radar.run(&mut exec);
        match denied_radar {
            LogicInstruction::Radar {
                output,
                last_target,
                ..
            } => {
                assert_eq!(output.value(), LVarValue::Object(None));
                assert_eq!(last_target, None);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn unit_bind_control_and_radar_executor_instructions_follow_java_l_executor_semantics() {
        let mut exec = LogicExecutor::new();
        exec.team = 1;

        let mut unit_a = LogicUnitObject::new("dagger", 1, 8.0, 8.0);
        unit_a.range = 96.0;
        unit_a.health = 50.0;
        unit_a.is_grounded = true;
        let mut unit_b = LogicUnitObject::new("dagger", 1, 16.0, 8.0);
        unit_b.range = 96.0;
        unit_b.health = 60.0;
        unit_b.is_grounded = true;
        let mut enemy_close = LogicUnitObject::new("crawler", 2, 24.0, 8.0);
        enemy_close.health = 25.0;
        enemy_close.max_health = 30.0;
        enemy_close.is_grounded = true;
        let mut enemy_far = LogicUnitObject::new("crawler", 2, 64.0, 8.0);
        enemy_far.health = 5.0;
        enemy_far.max_health = 30.0;
        enemy_far.is_grounded = true;
        let enemy_unreachable = LogicUnitObject::new("crawler", 2, 200.0, 8.0);
        let mut enemy_unclickable = LogicUnitObject::new("crawler", 2, 24.0, 16.0);
        enemy_unclickable.targetable = false;
        let mut ally_other_type = LogicUnitObject::new("flare", 1, 32.0, 8.0);
        ally_other_type.logic_controllable = false;
        exec.register_object("unit-a", LogicRuntimeObject::Unit(unit_a));
        exec.register_object("unit-b", LogicRuntimeObject::Unit(unit_b));
        exec.register_object("enemy-close", LogicRuntimeObject::Unit(enemy_close));
        exec.register_object("enemy-far", LogicRuntimeObject::Unit(enemy_far));
        exec.register_object(
            "enemy-unreachable",
            LogicRuntimeObject::Unit(enemy_unreachable),
        );
        exec.register_object(
            "enemy-unclickable",
            LogicRuntimeObject::Unit(enemy_unclickable),
        );
        exec.register_object("ally-other-type", LogicRuntimeObject::Unit(ally_other_type));

        let mut bind_dagger = LogicInstruction::UnitBind {
            type_: {
                let mut value = LVar::new("type");
                value.set_obj(Some("@dagger".into()));
                value
            },
        };
        bind_dagger.run(&mut exec);
        assert_eq!(exec.bound_unit, Some("unit-a".into()));
        bind_dagger.run(&mut exec);
        assert_eq!(exec.bound_unit, Some("unit-b".into()));
        bind_dagger.run(&mut exec);
        assert_eq!(exec.bound_unit, Some("unit-a".into()));

        let mut move_control = LogicInstruction::UnitControl {
            type_: LUnitControl::Move,
            p1: {
                let mut value = LVar::new("x");
                value.set_num(3.0);
                value
            },
            p2: {
                let mut value = LVar::new("y");
                value.set_num(4.0);
                value
            },
            p3: LVar::new("radius"),
            p4: LVar::new("result"),
            p5: LVar::new("unused"),
        };
        move_control.run(&mut exec);
        match exec.objects.get("unit-a").unwrap() {
            LogicRuntimeObject::Unit(unit) => {
                assert_eq!(unit.control, Some(LUnitControl::Move));
                assert_eq!(unit.move_x, 24.0);
                assert_eq!(unit.move_y, 32.0);
                assert!(unit.control_timer_refreshed);
            }
            _ => unreachable!(),
        }

        LogicInstruction::UnitControl {
            type_: LUnitControl::Approach,
            p1: {
                let mut value = LVar::new("x");
                value.set_num(5.0);
                value
            },
            p2: {
                let mut value = LVar::new("y");
                value.set_num(6.0);
                value
            },
            p3: {
                let mut value = LVar::new("radius");
                value.set_num(2.0);
                value
            },
            p4: LVar::new("result"),
            p5: LVar::new("unused"),
        }
        .run(&mut exec);
        match exec.objects.get("unit-a").unwrap() {
            LogicRuntimeObject::Unit(unit) => {
                assert_eq!(unit.control, Some(LUnitControl::Approach));
                assert_eq!(unit.move_x, 40.0);
                assert_eq!(unit.move_y, 48.0);
                assert_eq!(unit.move_rad, 16.0);
            }
            _ => unreachable!(),
        }

        LogicInstruction::UnitControl {
            type_: LUnitControl::Mine,
            p1: {
                let mut value = LVar::new("x");
                value.set_num(7.0);
                value
            },
            p2: {
                let mut value = LVar::new("y");
                value.set_num(8.0);
                value
            },
            p3: LVar::new("radius"),
            p4: LVar::new("result"),
            p5: LVar::new("unused"),
        }
        .run(&mut exec);
        LogicInstruction::UnitControl {
            type_: LUnitControl::Stop,
            p1: LVar::new("x"),
            p2: LVar::new("y"),
            p3: LVar::new("radius"),
            p4: LVar::new("result"),
            p5: LVar::new("unused"),
        }
        .run(&mut exec);
        match exec.objects.get("unit-a").unwrap() {
            LogicRuntimeObject::Unit(unit) => {
                assert_eq!(unit.control, Some(LUnitControl::Stop));
                assert!(unit.mine_x.is_none());
                assert!(unit.mine_y.is_none());
                assert!(unit.mine_cleared);
                assert!(unit.building_cleared);
            }
            _ => unreachable!(),
        }

        let mut within = LogicInstruction::UnitControl {
            type_: LUnitControl::Within,
            p1: {
                let mut value = LVar::new("x");
                value.set_num(1.0);
                value
            },
            p2: {
                let mut value = LVar::new("y");
                value.set_num(1.0);
                value
            },
            p3: {
                let mut value = LVar::new("radius");
                value.set_num(1.0);
                value
            },
            p4: LVar::new("result"),
            p5: LVar::new("unused"),
        };
        within.run(&mut exec);
        match within {
            LogicInstruction::UnitControl { p4, .. } => {
                assert_eq!(p4.value(), LVarValue::Number(1.0));
            }
            _ => unreachable!(),
        }

        LogicInstruction::UnitControl {
            type_: LUnitControl::Target,
            p1: {
                let mut value = LVar::new("x");
                value.set_num(10.0);
                value
            },
            p2: {
                let mut value = LVar::new("y");
                value.set_num(11.0);
                value
            },
            p3: {
                let mut value = LVar::new("shoot");
                value.set_num(1.0);
                value
            },
            p4: LVar::new("result"),
            p5: LVar::new("unused"),
        }
        .run(&mut exec);
        match exec.objects.get("unit-a").unwrap() {
            LogicRuntimeObject::Unit(unit) => {
                assert_eq!(unit.aim_control, Some(LUnitControl::Target));
                assert_eq!(unit.target_x, 80.0);
                assert_eq!(unit.target_y, 88.0);
                assert_eq!(unit.main_target, None);
                assert!(unit.shoot);
            }
            _ => unreachable!(),
        }

        LogicInstruction::UnitControl {
            type_: LUnitControl::Targetp,
            p1: {
                let mut value = LVar::new("target");
                value.set_obj(Some("enemy-close".into()));
                value
            },
            p2: {
                let mut value = LVar::new("shoot");
                value.set_num(0.0);
                value
            },
            p3: LVar::new("unused"),
            p4: LVar::new("result"),
            p5: LVar::new("unused"),
        }
        .run(&mut exec);
        match exec.objects.get("unit-a").unwrap() {
            LogicRuntimeObject::Unit(unit) => {
                assert_eq!(unit.aim_control, Some(LUnitControl::Targetp));
                assert_eq!(unit.main_target, Some("enemy-close".into()));
                assert!(!unit.shoot);
            }
            _ => unreachable!(),
        }

        LogicInstruction::UnitControl {
            type_: LUnitControl::Boost,
            p1: {
                let mut value = LVar::new("boost");
                value.set_num(1.0);
                value
            },
            p2: LVar::new("unused"),
            p3: LVar::new("unused"),
            p4: LVar::new("result"),
            p5: LVar::new("unused"),
        }
        .run(&mut exec);
        LogicInstruction::UnitControl {
            type_: LUnitControl::Flag,
            p1: {
                let mut value = LVar::new("flag");
                value.set_num(42.0);
                value
            },
            p2: LVar::new("unused"),
            p3: LVar::new("unused"),
            p4: LVar::new("result"),
            p5: LVar::new("unused"),
        }
        .run(&mut exec);
        match exec.objects.get("unit-a").unwrap() {
            LogicRuntimeObject::Unit(unit) => {
                assert!(unit.boost);
                assert_eq!(unit.flag, 42.0);
            }
            _ => unreachable!(),
        }

        let mut unit_radar_nearest = LogicInstruction::UnitRadar {
            target1: RadarTarget::Enemy,
            target2: RadarTarget::Ground,
            target3: RadarTarget::Any,
            sort: RadarSort::Distance,
            sort_order: {
                let mut value = LVar::new("sort");
                value.set_num(1.0);
                value
            },
            output: LVar::new("output"),
            last_target: None,
        };
        unit_radar_nearest.run(&mut exec);
        match unit_radar_nearest {
            LogicInstruction::UnitRadar {
                output,
                last_target,
                ..
            } => {
                assert_eq!(
                    output.value(),
                    LVarValue::Object(Some("enemy-close".into()))
                );
                assert_eq!(last_target, Some("enemy-close".into()));
            }
            _ => unreachable!(),
        }

        let mut unit_radar_lowest_health = LogicInstruction::UnitRadar {
            target1: RadarTarget::Enemy,
            target2: RadarTarget::Any,
            target3: RadarTarget::Any,
            sort: RadarSort::Health,
            sort_order: {
                let mut value = LVar::new("sort");
                value.set_num(0.0);
                value
            },
            output: LVar::new("output"),
            last_target: None,
        };
        unit_radar_lowest_health.run(&mut exec);
        match unit_radar_lowest_health {
            LogicInstruction::UnitRadar { output, .. } => {
                assert_eq!(output.value(), LVarValue::Object(Some("enemy-far".into())));
            }
            _ => unreachable!(),
        }

        LogicInstruction::UnitControl {
            type_: LUnitControl::Unbind,
            p1: LVar::new("unused"),
            p2: LVar::new("unused"),
            p3: LVar::new("unused"),
            p4: LVar::new("result"),
            p5: LVar::new("unused"),
        }
        .run(&mut exec);
        assert_eq!(exec.bound_unit, None);
        match exec.objects.get("unit-a").unwrap() {
            LogicRuntimeObject::Unit(unit) => assert!(unit.controller_reset),
            _ => unreachable!(),
        }

        let mut unit_radar_unbound = LogicInstruction::UnitRadar {
            target1: RadarTarget::Enemy,
            target2: RadarTarget::Any,
            target3: RadarTarget::Any,
            sort: RadarSort::Distance,
            sort_order: {
                let mut value = LVar::new("sort");
                value.set_num(1.0);
                value
            },
            output: {
                let mut value = LVar::new("output");
                value.set_obj(Some("old".into()));
                value
            },
            last_target: Some("old".into()),
        };
        unit_radar_unbound.run(&mut exec);
        match unit_radar_unbound {
            LogicInstruction::UnitRadar {
                output,
                last_target,
                ..
            } => {
                assert_eq!(output.value(), LVarValue::Object(None));
                assert_eq!(last_target, None);
            }
            _ => unreachable!(),
        }

        LogicInstruction::UnitBind {
            type_: {
                let mut value = LVar::new("type");
                value.set_obj(Some("@flare".into()));
                value
            },
        }
        .run(&mut exec);
        assert_eq!(exec.bound_unit, None);

        LogicInstruction::UnitBind {
            type_: {
                let mut value = LVar::new("type");
                value.set_obj(Some("unit-b".into()));
                value
            },
        }
        .run(&mut exec);
        assert_eq!(exec.bound_unit, Some("unit-b".into()));

        exec.logic_unit_control = false;
        LogicInstruction::UnitControl {
            type_: LUnitControl::Unbind,
            p1: LVar::new("unused"),
            p2: LVar::new("unused"),
            p3: LVar::new("unused"),
            p4: LVar::new("result"),
            p5: LVar::new("unused"),
        }
        .run(&mut exec);
        assert_eq!(exec.bound_unit, Some("unit-b".into()));
    }

    #[test]
    fn world_query_fetch_locate_and_flag_executor_instructions_follow_java_l_executor_semantics() {
        let mut exec = LogicExecutor::new();
        exec.team = 1;
        exec.world.set_tile(
            2,
            3,
            LogicTileObject {
                floor: Some("@stone".into()),
                ore: Some("@copper".into()),
                block: Some("@conveyor".into()),
                building: Some("core-a".into()),
                team: 1,
                rotation: 1,
            },
        );
        exec.world.set_tile(
            8,
            8,
            LogicTileObject {
                ore: Some("@thorium".into()),
                ..LogicTileObject::default()
            },
        );
        exec.world.spawns.push((80.0, 16.0));

        let mut unit = LogicUnitObject::new("dagger", 1, 16.0, 24.0);
        unit.range = 96.0;
        exec.register_object("unit-a", LogicRuntimeObject::Unit(unit));
        let mut player = LogicUnitObject::new("flare", 1, 40.0, 24.0);
        player.is_player = true;
        exec.register_object("player-a", LogicRuntimeObject::Unit(player));
        exec.register_object(
            "enemy-unit",
            LogicRuntimeObject::Unit(LogicUnitObject::new("crawler", 2, 200.0, 200.0)),
        );

        let mut core = LogicBuildingObject::new("core-shard", 1, 16.0, 24.0);
        core.flags.insert(BlockFlag::Core);
        exec.register_object("core-a", LogicRuntimeObject::Building(core));
        let mut enemy_core = LogicBuildingObject::new("core-shard", 2, 56.0, 24.0);
        enemy_core.flags.insert(BlockFlag::Core);
        exec.register_object("enemy-core", LogicRuntimeObject::Building(enemy_core));
        let mut damaged = LogicBuildingObject::new("duo", 1, 24.0, 24.0);
        damaged.flags.insert(BlockFlag::Turret);
        damaged.damaged = true;
        exec.register_object("damaged-turret", LogicRuntimeObject::Building(damaged));

        let mut get_block = LogicInstruction::GetBlock {
            layer: TileLayer::Block,
            result: LVar::new("result"),
            x: {
                let mut value = LVar::new("x");
                value.set_num(2.2);
                value
            },
            y: {
                let mut value = LVar::new("y");
                value.set_num(2.8);
                value
            },
        };
        get_block.run(&mut exec);
        match get_block {
            LogicInstruction::GetBlock { result, .. } => {
                assert_eq!(result.value(), LVarValue::Object(Some("@conveyor".into())));
            }
            _ => unreachable!(),
        }

        LogicInstruction::SetBlock {
            layer: TileLayer::Block,
            block: {
                let mut value = LVar::new("block");
                value.set_obj(Some("@duo".into()));
                value
            },
            x: {
                let mut value = LVar::new("x");
                value.set_num(2.0);
                value
            },
            y: {
                let mut value = LVar::new("y");
                value.set_num(3.0);
                value
            },
            team: {
                let mut value = LVar::new("team");
                value.set_obj(Some("@crux".into()));
                value
            },
            rotation: {
                let mut value = LVar::new("rotation");
                value.set_num(9.0);
                value
            },
        }
        .run(&mut exec);
        let tile = exec.world.tile(2, 3).unwrap();
        assert_eq!(tile.block, Some("@duo".into()));
        assert_eq!(tile.team, 2);
        assert_eq!(tile.rotation, 3);

        LogicInstruction::SetBlock {
            layer: TileLayer::Ore,
            block: {
                let mut value = LVar::new("block");
                value.set_obj(Some("@scrap".into()));
                value
            },
            x: {
                let mut value = LVar::new("x");
                value.set_num(2.0);
                value
            },
            y: {
                let mut value = LVar::new("y");
                value.set_num(3.0);
                value
            },
            team: {
                let mut value = LVar::new("team");
                value.set_obj(None);
                value
            },
            rotation: LVar::new("rotation"),
        }
        .run(&mut exec);
        assert_eq!(exec.world.tile(2, 3).unwrap().ore, Some("@scrap".into()));

        LogicInstruction::Query {
            shape: QueryShape::Circle,
            type_: QueryType::Unit,
            team: {
                let mut value = LVar::new("team");
                value.set_obj(Some("@sharded".into()));
                value
            },
            x: {
                let mut value = LVar::new("x");
                value.set_num(2.0);
                value
            },
            y: {
                let mut value = LVar::new("y");
                value.set_num(3.0);
                value
            },
            w: {
                let mut value = LVar::new("radius");
                value.set_num(4.0);
                value
            },
            h: LVar::new("unused"),
        }
        .run(&mut exec);
        match exec.objects.get("@query").unwrap() {
            LogicRuntimeObject::QueryResult(values) => {
                assert_eq!(values, &vec!["player-a".to_string(), "unit-a".to_string()]);
            }
            _ => unreachable!(),
        }

        LogicInstruction::Query {
            shape: QueryShape::Rect,
            type_: QueryType::Building,
            team: {
                let mut value = LVar::new("team");
                value.set_obj(Some("@sharded".into()));
                value
            },
            x: {
                let mut value = LVar::new("x");
                value.set_num(1.0);
                value
            },
            y: {
                let mut value = LVar::new("y");
                value.set_num(2.0);
                value
            },
            w: {
                let mut value = LVar::new("w");
                value.set_num(4.0);
                value
            },
            h: {
                let mut value = LVar::new("h");
                value.set_num(2.0);
                value
            },
        }
        .run(&mut exec);
        match exec.objects.get("@query").unwrap() {
            LogicRuntimeObject::QueryResult(values) => {
                assert_eq!(
                    values,
                    &vec!["core-a".to_string(), "damaged-turret".to_string()]
                );
            }
            _ => unreachable!(),
        }

        let mut fetch_unit_count = LogicInstruction::Fetch {
            type_: FetchType::UnitCount,
            result: LVar::new("result"),
            team: {
                let mut value = LVar::new("team");
                value.set_obj(Some("@sharded".into()));
                value
            },
            index: LVar::new("index"),
            extra: {
                let mut value = LVar::new("extra");
                value.set_obj(Some("@dagger".into()));
                value
            },
        };
        fetch_unit_count.run(&mut exec);
        match fetch_unit_count {
            LogicInstruction::Fetch { result, .. } => {
                assert_eq!(result.value(), LVarValue::Number(1.0));
            }
            _ => unreachable!(),
        }

        let mut fetch_player = LogicInstruction::Fetch {
            type_: FetchType::Player,
            result: LVar::new("result"),
            team: {
                let mut value = LVar::new("team");
                value.set_num(1.0);
                value
            },
            index: {
                let mut value = LVar::new("index");
                value.set_num(0.0);
                value
            },
            extra: LVar::new("extra"),
        };
        fetch_player.run(&mut exec);
        match fetch_player {
            LogicInstruction::Fetch { result, .. } => {
                assert_eq!(result.value(), LVarValue::Object(Some("player-a".into())));
            }
            _ => unreachable!(),
        }

        let mut fetch_core = LogicInstruction::Fetch {
            type_: FetchType::Core,
            result: LVar::new("result"),
            team: {
                let mut value = LVar::new("team");
                value.set_obj(Some("@sharded".into()));
                value
            },
            index: {
                let mut value = LVar::new("index");
                value.set_num(0.0);
                value
            },
            extra: LVar::new("extra"),
        };
        fetch_core.run(&mut exec);
        match fetch_core {
            LogicInstruction::Fetch { result, .. } => {
                assert_eq!(result.value(), LVarValue::Object(Some("core-a".into())));
            }
            _ => unreachable!(),
        }

        LogicInstruction::UnitBind {
            type_: {
                let mut value = LVar::new("type");
                value.set_obj(Some("unit-a".into()));
                value
            },
        }
        .run(&mut exec);

        let mut locate_enemy_core = LogicInstruction::UnitLocate {
            locate: LLocate::Building,
            flag: BlockFlag::Core,
            enemy: {
                let mut value = LVar::new("enemy");
                value.set_num(1.0);
                value
            },
            ore: LVar::new("ore"),
            out_x: LVar::new("x"),
            out_y: LVar::new("y"),
            out_found: LVar::new("found"),
            out_build: LVar::new("build"),
        };
        locate_enemy_core.run(&mut exec);
        match locate_enemy_core {
            LogicInstruction::UnitLocate {
                out_x,
                out_y,
                out_found,
                out_build,
                ..
            } => {
                assert_eq!(out_x.value(), LVarValue::Number(7.0));
                assert_eq!(out_y.value(), LVarValue::Number(3.0));
                assert_eq!(out_found.value(), LVarValue::Number(1.0));
                assert_eq!(
                    out_build.value(),
                    LVarValue::Object(Some("enemy-core".into()))
                );
            }
            _ => unreachable!(),
        }

        let mut locate_ore = LogicInstruction::UnitLocate {
            locate: LLocate::Ore,
            flag: BlockFlag::Core,
            enemy: LVar::new("enemy"),
            ore: {
                let mut value = LVar::new("ore");
                value.set_obj(Some("@thorium".into()));
                value
            },
            out_x: LVar::new("x"),
            out_y: LVar::new("y"),
            out_found: LVar::new("found"),
            out_build: LVar::new("build"),
        };
        locate_ore.run(&mut exec);
        match locate_ore {
            LogicInstruction::UnitLocate {
                out_x,
                out_y,
                out_found,
                out_build,
                ..
            } => {
                assert_eq!(out_x.value(), LVarValue::Number(8.0));
                assert_eq!(out_y.value(), LVarValue::Number(8.0));
                assert_eq!(out_found.value(), LVarValue::Number(1.0));
                assert_eq!(out_build.value(), LVarValue::Object(None));
            }
            _ => unreachable!(),
        }

        let mut locate_damaged = LogicInstruction::UnitLocate {
            locate: LLocate::Damaged,
            flag: BlockFlag::Core,
            enemy: LVar::new("enemy"),
            ore: LVar::new("ore"),
            out_x: LVar::new("x"),
            out_y: LVar::new("y"),
            out_found: LVar::new("found"),
            out_build: LVar::new("build"),
        };
        locate_damaged.run(&mut exec);
        match locate_damaged {
            LogicInstruction::UnitLocate {
                out_found,
                out_build,
                ..
            } => {
                assert_eq!(out_found.value(), LVarValue::Number(1.0));
                assert_eq!(
                    out_build.value(),
                    LVarValue::Object(Some("damaged-turret".into()))
                );
            }
            _ => unreachable!(),
        }

        LogicInstruction::SetFlag {
            flag: {
                let mut value = LVar::new("flag");
                value.set_obj(Some("sector-clear".into()));
                value
            },
            value: {
                let mut value = LVar::new("value");
                value.set_num(1.0);
                value
            },
        }
        .run(&mut exec);
        let mut get_flag = LogicInstruction::GetFlag {
            result: LVar::new("result"),
            flag: {
                let mut value = LVar::new("flag");
                value.set_obj(Some("sector-clear".into()));
                value
            },
        };
        get_flag.run(&mut exec);
        match get_flag {
            LogicInstruction::GetFlag { result, .. } => {
                assert_eq!(result.value(), LVarValue::Number(1.0));
            }
            _ => unreachable!(),
        }

        let mut missing_flag = LogicInstruction::GetFlag {
            result: LVar::new("result"),
            flag: LVar::new("flag"),
        };
        missing_flag.run(&mut exec);
        match missing_flag {
            LogicInstruction::GetFlag { result, .. } => {
                assert_eq!(result.value(), LVarValue::Object(None));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn privileged_world_effect_message_and_rule_executor_instructions_follow_java_l_executor_semantics(
    ) {
        let mut exec = LogicExecutor::new();

        let mut spawn = LogicInstruction::SpawnUnit {
            type_: {
                let mut value = LVar::new("type");
                value.set_obj(Some("@dagger".into()));
                value
            },
            x: {
                let mut value = LVar::new("x");
                value.set_num(2.0);
                value
            },
            y: {
                let mut value = LVar::new("y");
                value.set_num(3.0);
                value
            },
            rotation: {
                let mut value = LVar::new("rotation");
                value.set_num(90.0);
                value
            },
            team: {
                let mut value = LVar::new("team");
                value.set_obj(Some("@sharded".into()));
                value
            },
            result: LVar::new("result"),
        };
        spawn.run(&mut exec);
        match spawn {
            LogicInstruction::SpawnUnit { result, .. } => {
                assert_eq!(
                    result.value(),
                    LVarValue::Object(Some("spawned-dagger-0".into()))
                );
            }
            _ => unreachable!(),
        }
        assert_eq!(
            exec.spawn_events,
            vec![LogicSpawnEvent {
                unit_name: "spawned-dagger-0".into(),
                type_name: "dagger".into(),
                team: 1,
                x: 16.0,
                y: 24.0,
                rotation: 90.0,
            }]
        );

        LogicInstruction::ApplyStatus {
            clear: false,
            effect: "burning".into(),
            unit: {
                let mut value = LVar::new("unit");
                value.set_obj(Some("spawned-dagger-0".into()));
                value
            },
            duration: {
                let mut value = LVar::new("duration");
                value.set_num(2.5);
                value
            },
        }
        .run(&mut exec);
        match exec.objects.get("spawned-dagger-0").unwrap() {
            LogicRuntimeObject::Unit(unit) => {
                assert_eq!(unit.statuses.get("burning"), Some(&150.0));
            }
            _ => unreachable!(),
        }
        LogicInstruction::ApplyStatus {
            clear: true,
            effect: "burning".into(),
            unit: {
                let mut value = LVar::new("unit");
                value.set_obj(Some("spawned-dagger-0".into()));
                value
            },
            duration: LVar::new("duration"),
        }
        .run(&mut exec);
        match exec.objects.get("spawned-dagger-0").unwrap() {
            LogicRuntimeObject::Unit(unit) => assert!(!unit.statuses.contains_key("burning")),
            _ => unreachable!(),
        }

        LogicInstruction::Effect {
            type_name: "warn".into(),
            x: {
                let mut value = LVar::new("x");
                value.set_num(1.0);
                value
            },
            y: {
                let mut value = LVar::new("y");
                value.set_num(2.0);
                value
            },
            rotation: {
                let mut value = LVar::new("size");
                value.set_num(5000.0);
                value
            },
            color: {
                let mut value = LVar::new("color");
                value.set_num(rgba_to_double_bits(0xff, 0x00, 0xaa, 0xff));
                value
            },
            data: {
                let mut value = LVar::new("data");
                value.set_obj(Some("@duo".into()));
                value
            },
        }
        .run(&mut exec);
        assert_eq!(
            exec.effect_events.last(),
            Some(&LogicEffectEvent {
                type_name: "warn".into(),
                effect_name: "unitCapKill".into(),
                x: 8.0,
                y: 16.0,
                rotation: 1000.0,
                color: rgba_to_double_bits(0xff, 0x00, 0xaa, 0xff),
                data: Some("@duo".into()),
            })
        );

        LogicInstruction::Explosion {
            team: {
                let mut value = LVar::new("team");
                value.set_obj(Some("@crux".into()));
                value
            },
            x: {
                let mut value = LVar::new("x");
                value.set_num(4.0);
                value
            },
            y: {
                let mut value = LVar::new("y");
                value.set_num(5.0);
                value
            },
            radius: {
                let mut value = LVar::new("radius");
                value.set_num(200.0);
                value
            },
            damage: {
                let mut value = LVar::new("damage");
                value.set_num(50.0);
                value
            },
            air: {
                let mut value = LVar::new("air");
                value.set_num(1.0);
                value
            },
            ground: {
                let mut value = LVar::new("ground");
                value.set_num(0.0);
                value
            },
            pierce: {
                let mut value = LVar::new("pierce");
                value.set_num(1.0);
                value
            },
            effect: {
                let mut value = LVar::new("effect");
                value.set_num(1.0);
                value
            },
        }
        .run(&mut exec);
        assert_eq!(
            exec.explosion_events.last(),
            Some(&LogicExplosionEvent {
                team: Some(2),
                x: 32.0,
                y: 40.0,
                radius: 800.0,
                damage: 50.0,
                air: true,
                ground: false,
                pierce: true,
                effect: true,
            })
        );

        LogicInstruction::SetRule {
            rule: LogicRule::Wave,
            value: {
                let mut value = LVar::new("value");
                value.set_num(-5.0);
                value
            },
            p1: LVar::new("p1"),
            p2: LVar::new("p2"),
            p3: LVar::new("p3"),
            p4: LVar::new("p4"),
        }
        .run(&mut exec);
        assert_eq!(exec.rules.wave, 1);
        LogicInstruction::SetRule {
            rule: LogicRule::WaveSpacing,
            value: {
                let mut value = LVar::new("value");
                value.set_num(10.0);
                value
            },
            p1: LVar::new("p1"),
            p2: LVar::new("p2"),
            p3: LVar::new("p3"),
            p4: LVar::new("p4"),
        }
        .run(&mut exec);
        assert_eq!(exec.rules.wave_spacing, 600.0);
        LogicInstruction::SetRule {
            rule: LogicRule::MapArea,
            value: LVar::new("value"),
            p1: {
                let mut value = LVar::new("x");
                value.set_num(1.0);
                value
            },
            p2: {
                let mut value = LVar::new("y");
                value.set_num(2.0);
                value
            },
            p3: {
                let mut value = LVar::new("w");
                value.set_num(30.0);
                value
            },
            p4: {
                let mut value = LVar::new("h");
                value.set_num(40.0);
                value
            },
        }
        .run(&mut exec);
        assert_eq!(exec.rules.map_area, Some((1, 2, 30, 40)));
        LogicInstruction::SetRule {
            rule: LogicRule::BuildSpeed,
            value: {
                let mut value = LVar::new("value");
                value.set_num(100.0);
                value
            },
            p1: {
                let mut value = LVar::new("team");
                value.set_obj(Some("@sharded".into()));
                value
            },
            p2: LVar::new("p2"),
            p3: LVar::new("p3"),
            p4: LVar::new("p4"),
        }
        .run(&mut exec);
        assert_eq!(
            exec.rules
                .team_rules
                .get(&1)
                .unwrap()
                .build_speed_multiplier,
            50.0
        );

        LogicInstruction::SpawnWave {
            x: {
                let mut value = LVar::new("x");
                value.set_num(7.0);
                value
            },
            y: {
                let mut value = LVar::new("y");
                value.set_num(8.0);
                value
            },
            natural: {
                let mut value = LVar::new("natural");
                value.set_num(1.0);
                value
            },
        }
        .run(&mut exec);
        assert_eq!(exec.spawn_wave_events, vec![(56.0, 64.0, true)]);

        LogicInstruction::Cutscene {
            action: CutsceneAction::Pan,
            p1: {
                let mut value = LVar::new("x");
                value.set_num(9.0);
                value
            },
            p2: {
                let mut value = LVar::new("y");
                value.set_num(10.0);
                value
            },
            p3: {
                let mut value = LVar::new("speed");
                value.set_num(0.06);
                value
            },
            p4: LVar::new("p4"),
        }
        .run(&mut exec);
        assert!(exec.cutscene.active);
        assert_eq!((exec.cutscene.pan_x, exec.cutscene.pan_y), (72.0, 80.0));
        LogicInstruction::Cutscene {
            action: CutsceneAction::Zoom,
            p1: {
                let mut value = LVar::new("zoom");
                value.set_num(2.0);
                value
            },
            p2: LVar::new("p2"),
            p3: LVar::new("p3"),
            p4: LVar::new("p4"),
        }
        .run(&mut exec);
        assert_eq!(exec.cutscene.zoom, 1.0);
        LogicInstruction::Cutscene {
            action: CutsceneAction::Stop,
            p1: LVar::new("p1"),
            p2: LVar::new("p2"),
            p3: LVar::new("p3"),
            p4: LVar::new("p4"),
        }
        .run(&mut exec);
        assert!(!exec.cutscene.active);

        exec.text_buffer = "hello".into();
        let mut message = LogicInstruction::FlushMessage {
            type_: MessageType::Announce,
            duration: {
                let mut value = LVar::new("duration");
                value.set_num(3.0);
                value
            },
            out_success: LVar::new("ok"),
        };
        message.run(&mut exec);
        match message {
            LogicInstruction::FlushMessage { out_success, .. } => {
                assert_eq!(out_success.value(), LVarValue::Number(1.0));
            }
            _ => unreachable!(),
        }
        assert_eq!(
            exec.message_events,
            vec![LogicMessageEvent {
                type_: MessageType::Announce,
                text: "hello".into(),
                duration: 3.0,
            }]
        );
        assert!(exec.text_buffer.is_empty());

        exec.message_state.announcement_active = true;
        exec.text_buffer = "blocked".into();
        let mut blocked_message = LogicInstruction::FlushMessage {
            type_: MessageType::Announce,
            duration: LVar::new("duration"),
            out_success: LVar::new("ok"),
        };
        blocked_message.run(&mut exec);
        match blocked_message {
            LogicInstruction::FlushMessage { out_success, .. } => {
                assert_eq!(out_success.value(), LVarValue::Number(0.0));
            }
            _ => unreachable!(),
        }
        assert_eq!(exec.text_buffer, "blocked");

        let mut wait_blocked_message = LogicInstruction::FlushMessage {
            type_: MessageType::Announce,
            duration: LVar::new("duration"),
            out_success: LVar::new("@wait"),
        };
        exec.counter.set_num(5.0);
        wait_blocked_message.run(&mut exec);
        assert_eq!(exec.counter.numval, 4.0);
        assert!(exec.yield_);

        exec.message_state.announcement_active = false;
        exec.headless = true;
        exec.text_buffer = "mission".into();
        LogicInstruction::FlushMessage {
            type_: MessageType::Mission,
            duration: LVar::new("duration"),
            out_success: LVar::new("ok"),
        }
        .run(&mut exec);
        assert_eq!(exec.rules.mission, "mission");
        assert!(exec.text_buffer.is_empty());
    }

    #[test]
    fn remaining_runtime_logic_instructions_record_java_side_effects() {
        let mut exec = LogicExecutor::new();
        exec.team = 1;
        exec.max_ipt = 120;
        exec.map_locales
            .insert("name".into(), "Desktop Name".into());
        exec.map_locales
            .insert("name.mobile".into(), "Mobile Name".into());

        LogicInstruction::LocalePrint {
            value: {
                let mut value = LVar::new("key");
                value.set_obj(Some("name".into()));
                value
            },
        }
        .run(&mut exec);
        assert_eq!(exec.text_buffer, "Desktop Name");
        exec.mobile = true;
        LogicInstruction::LocalePrint {
            value: {
                let mut value = LVar::new("key");
                value.set_obj(Some("name".into()));
                value
            },
        }
        .run(&mut exec);
        assert_eq!(exec.text_buffer, "Desktop NameMobile Name");

        exec.graphics_buffer = vec![LogicDisplayCommand::get(1, 2, 3, 4, 5, 6, 7)];
        exec.register_object(
            "display1",
            LogicRuntimeObject::Building(LogicBuildingObject::new("logic-display", 1, 0.0, 0.0)),
        );
        LogicInstruction::DrawFlush {
            target: {
                let mut value = LVar::new("target");
                value.set_obj(Some("display1".into()));
                value
            },
        }
        .run(&mut exec);
        assert!(exec.graphics_buffer.is_empty());
        match exec.objects.get("display1").unwrap() {
            LogicRuntimeObject::Building(building) => {
                assert_eq!(building.display_commands.len(), 1);
            }
            _ => unreachable!(),
        }

        exec.text_buffer = "screen text".into();
        exec.register_object(
            "message1",
            LogicRuntimeObject::Building(LogicBuildingObject::new("message", 1, 0.0, 0.0)),
        );
        LogicInstruction::PrintFlush {
            target: {
                let mut value = LVar::new("target");
                value.set_obj(Some("message1".into()));
                value
            },
        }
        .run(&mut exec);
        assert!(exec.text_buffer.is_empty());
        match exec.objects.get("message1").unwrap() {
            LogicRuntimeObject::Building(building) => {
                assert_eq!(building.message, "screen text");
            }
            _ => unreachable!(),
        }

        exec.text_buffer = "denied".into();
        let mut blocked = LogicBuildingObject::new("message", 2, 0.0, 0.0);
        blocked.block_privileged = true;
        exec.register_object("blocked-message", LogicRuntimeObject::Building(blocked));
        LogicInstruction::PrintFlush {
            target: {
                let mut value = LVar::new("target");
                value.set_obj(Some("blocked-message".into()));
                value
            },
        }
        .run(&mut exec);
        assert!(exec.text_buffer.is_empty());
        match exec.objects.get("blocked-message").unwrap() {
            LogicRuntimeObject::Building(building) => assert!(building.message.is_empty()),
            _ => unreachable!(),
        }

        LogicInstruction::SetRate {
            amount: {
                let mut value = LVar::new("amount");
                value.set_num(999.0);
                value
            },
        }
        .run(&mut exec);
        assert_eq!(exec.ipt, 120);
        LogicInstruction::SetRate {
            amount: {
                let mut value = LVar::new("amount");
                value.set_num(-1.0);
                value
            },
        }
        .run(&mut exec);
        assert_eq!(exec.ipt, 1);

        let mut sync = LogicInstruction::Sync {
            variable: {
                let mut value = LVar::with_id("shared", 7);
                value.set_obj(Some("payload".into()));
                value.sync_time = 0;
                value
            },
        };
        exec.current_time_millis = LOGIC_SYNC_INTERVAL_MILLIS + 1;
        sync.run(&mut exec);
        assert_eq!(
            exec.sync_events,
            vec![LogicSyncEvent {
                variable_id: 7,
                value: LVarValue::Object(Some("payload".into())),
            }]
        );
        sync.run(&mut exec);
        assert_eq!(exec.sync_events.len(), 1);

        LogicInstruction::WeatherSet {
            weather: {
                let mut value = LVar::new("weather");
                value.set_obj(Some("@rain".into()));
                value
            },
            state: {
                let mut value = LVar::new("state");
                value.set_num(1.0);
                value
            },
        }
        .run(&mut exec);
        let mut weather_sense = LogicInstruction::WeatherSense {
            to: LVar::new("to"),
            weather: {
                let mut value = LVar::new("weather");
                value.set_obj(Some("@rain".into()));
                value
            },
        };
        weather_sense.run(&mut exec);
        match weather_sense {
            LogicInstruction::WeatherSense { to, .. } => {
                assert_eq!(to.value(), LVarValue::Number(1.0));
            }
            _ => unreachable!(),
        }
        assert_eq!(
            exec.weather_events,
            vec![LogicWeatherEvent {
                weather_name: "@rain".into(),
                active: true,
                life: LOGIC_WEATHER_FADE_TIME,
            }]
        );

        exec.register_object(
            "owner",
            LogicRuntimeObject::Unit(LogicUnitObject::new("dagger", 2, 0.0, 0.0)),
        );
        let mut bullet = LogicInstruction::SpawnBullet {
            result: LVar::new("result"),
            from: {
                let mut value = LVar::new("from");
                value.set_obj(Some("@dagger".into()));
                value
            },
            weapon: {
                let mut value = LVar::new("weapon");
                value.set_num(0.0);
                value
            },
            x: {
                let mut value = LVar::new("x");
                value.set_num(2.0);
                value
            },
            y: {
                let mut value = LVar::new("y");
                value.set_num(3.0);
                value
            },
            rotation: {
                let mut value = LVar::new("rot");
                value.set_num(45.0);
                value
            },
            team: {
                let mut value = LVar::new("team");
                value.set_obj(None);
                value
            },
            owner: {
                let mut value = LVar::new("owner");
                value.set_obj(Some("owner".into()));
                value
            },
            damage: {
                let mut value = LVar::new("damage");
                value.set_num(12.0);
                value
            },
            velocity_scl: {
                let mut value = LVar::new("velocity");
                value.set_num(1.5);
                value
            },
            life_scl: {
                let mut value = LVar::new("life");
                value.set_num(0.5);
                value
            },
            aim_x: {
                let mut value = LVar::new("aimx");
                value.set_num(4.0);
                value
            },
            aim_y: {
                let mut value = LVar::new("aimy");
                value.set_num(5.0);
                value
            },
        };
        bullet.run(&mut exec);
        match bullet {
            LogicInstruction::SpawnBullet { result, .. } => {
                assert_eq!(result.value(), LVarValue::Object(Some("bullet-0".into())));
            }
            _ => unreachable!(),
        }
        assert_eq!(exec.bullet_events.len(), 1);
        assert_eq!(exec.bullet_events[0].team, 2);
        assert_eq!(
            (exec.bullet_events[0].x, exec.bullet_events[0].y),
            (16.0, 24.0)
        );
        assert!(matches!(
            exec.objects.get("bullet-0"),
            Some(LogicRuntimeObject::Bullet(_))
        ));

        LogicInstruction::ClientData {
            channel: {
                let mut value = LVar::new("channel");
                value.set_obj(Some("frog".into()));
                value
            },
            value: {
                let mut value = LVar::new("value");
                value.set_num(9.0);
                value
            },
            reliable: {
                let mut value = LVar::new("reliable");
                value.set_num(1.0);
                value
            },
        }
        .run(&mut exec);
        assert_eq!(
            exec.client_data_events,
            vec![LogicClientDataEvent {
                channel: "frog".into(),
                value: LVarValue::Number(9.0),
                reliable: true,
            }]
        );
    }

    #[test]
    fn set_prop_runtime_updates_unit_and_building_state_like_settable_subset() {
        let mut exec = LogicExecutor::new();
        let mut unit = LogicUnitObject::new("dagger", 1, 0.0, 0.0);
        unit.max_health = 100.0;
        exec.register_object("unit", LogicRuntimeObject::Unit(unit));
        exec.register_object(
            "build",
            LogicRuntimeObject::Building(LogicBuildingObject::new("core-shard", 1, 0.0, 0.0)),
        );

        LogicInstruction::SetProp {
            type_: {
                let mut value = LVar::new("type");
                value.set_obj(Some("@health".into()));
                value
            },
            of: {
                let mut value = LVar::new("of");
                value.set_obj(Some("unit".into()));
                value
            },
            value: {
                let mut value = LVar::new("value");
                value.set_num(250.0);
                value
            },
        }
        .run(&mut exec);
        match exec.objects.get("unit").unwrap() {
            LogicRuntimeObject::Unit(unit) => assert_eq!(unit.health, 100.0),
            _ => unreachable!(),
        }

        LogicInstruction::SetProp {
            type_: {
                let mut value = LVar::new("type");
                value.set_obj(Some("@team".into()));
                value
            },
            of: {
                let mut value = LVar::new("of");
                value.set_obj(Some("build".into()));
                value
            },
            value: {
                let mut value = LVar::new("value");
                value.set_obj(Some("@crux".into()));
                value
            },
        }
        .run(&mut exec);
        match exec.objects.get("build").unwrap() {
            LogicRuntimeObject::Building(building) => assert_eq!(building.team, 2),
            _ => unreachable!(),
        }

        LogicInstruction::SetProp {
            type_: {
                let mut value = LVar::new("type");
                value.set_obj(Some("@copper".into()));
                value
            },
            of: {
                let mut value = LVar::new("of");
                value.set_obj(Some("build".into()));
                value
            },
            value: {
                let mut value = LVar::new("value");
                value.set_num(30.0);
                value
            },
        }
        .run(&mut exec);
        match exec.objects.get("build").unwrap() {
            LogicRuntimeObject::Building(building) => {
                assert_eq!(building.content_props.get("@copper"), Some(&30.0));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn sound_and_marker_runtime_instructions_record_world_side_effects() {
        let mut exec = LogicExecutor::new();
        LogicInstruction::PlaySound {
            positional: false,
            id: {
                let mut value = LVar::new("sound");
                value.set_num(3.0);
                value
            },
            volume: {
                let mut value = LVar::new("volume");
                value.set_num(5.0);
                value
            },
            pitch: {
                let mut value = LVar::new("pitch");
                value.set_num(0.75);
                value
            },
            pan: {
                let mut value = LVar::new("pan");
                value.set_num(-0.25);
                value
            },
            x: LVar::new("x"),
            y: LVar::new("y"),
            limit: {
                let mut value = LVar::new("limit");
                value.set_num(1.0);
                value
            },
        }
        .run(&mut exec);
        assert_eq!(
            exec.sound_events,
            vec![LogicSoundEvent {
                positional: false,
                sound_id: 3,
                sound_name: None,
                volume: 2.0,
                pitch: 0.75,
                pan: -0.25,
                x: None,
                y: None,
                limit: true,
            }]
        );

        LogicInstruction::PlaySound {
            positional: true,
            id: {
                let mut value = LVar::new("sound");
                value.set_obj(Some("@sfx-explosion".into()));
                value
            },
            volume: {
                let mut value = LVar::new("volume");
                value.set_num(0.5);
                value
            },
            pitch: {
                let mut value = LVar::new("pitch");
                value.set_num(2.0);
                value
            },
            pan: LVar::new("pan"),
            x: {
                let mut value = LVar::new("x");
                value.set_num(2.0);
                value
            },
            y: {
                let mut value = LVar::new("y");
                value.set_num(3.0);
                value
            },
            limit: LVar::new("limit"),
        }
        .run(&mut exec);
        assert_eq!(
            exec.sound_events[1].sound_name,
            Some("@sfx-explosion".into())
        );
        assert_eq!(
            (exec.sound_events[1].x, exec.sound_events[1].y),
            (Some(16.0), Some(24.0))
        );

        LogicInstruction::MakeMarker {
            type_name: "shape".into(),
            id: {
                let mut value = LVar::new("id");
                value.set_num(7.0);
                value
            },
            x: {
                let mut value = LVar::new("x");
                value.set_num(2.0);
                value
            },
            y: {
                let mut value = LVar::new("y");
                value.set_num(3.0);
                value
            },
            replace: {
                let mut value = LVar::new("replace");
                value.set_num(1.0);
                value
            },
        }
        .run(&mut exec);
        assert_eq!((exec.markers[&7].x, exec.markers[&7].y), (16.0, 24.0));
        assert_eq!(
            exec.marker_events[0],
            LogicMarkerEvent::Created {
                id: 7,
                type_name: "shape".into(),
                x: 16.0,
                y: 24.0,
                replaced: false,
            }
        );

        LogicInstruction::MakeMarker {
            type_name: "missing".into(),
            id: {
                let mut value = LVar::new("id");
                value.set_num(8.0);
                value
            },
            x: LVar::new("x"),
            y: LVar::new("y"),
            replace: {
                let mut value = LVar::new("replace");
                value.set_num(1.0);
                value
            },
        }
        .run(&mut exec);
        assert!(!exec.markers.contains_key(&8));

        LogicInstruction::SetMarker {
            type_: LMarkerControl::Pos,
            id: {
                let mut value = LVar::new("id");
                value.set_num(7.0);
                value
            },
            p1: {
                let mut value = LVar::new("p1");
                value.set_num(5.0);
                value
            },
            p2: {
                let mut value = LVar::new("p2");
                value.set_num(6.0);
                value
            },
            p3: LVar::new("p3"),
        }
        .run(&mut exec);
        assert_eq!((exec.markers[&7].x, exec.markers[&7].y), (40.0, 48.0));

        exec.text_buffer = "marker text".into();
        LogicInstruction::SetMarker {
            type_: LMarkerControl::FlushText,
            id: {
                let mut value = LVar::new("id");
                value.set_num(7.0);
                value
            },
            p1: {
                let mut value = LVar::new("fetch");
                value.set_num(1.0);
                value
            },
            p2: LVar::new("p2"),
            p3: LVar::new("p3"),
        }
        .run(&mut exec);
        assert!(exec.text_buffer.is_empty());
        assert_eq!(exec.markers[&7].text, "marker text");
        assert!(exec.markers[&7].text_fetch);

        exec.text_buffer = "texture-name".into();
        LogicInstruction::SetMarker {
            type_: LMarkerControl::Texture,
            id: {
                let mut value = LVar::new("id");
                value.set_num(7.0);
                value
            },
            p1: {
                let mut value = LVar::new("flush");
                value.set_num(1.0);
                value
            },
            p2: LVar::new("p2"),
            p3: LVar::new("p3"),
        }
        .run(&mut exec);
        assert_eq!(
            exec.markers[&7].texture,
            LVarValue::Object(Some("texture-name".into()))
        );
        assert!(exec.text_buffer.is_empty());

        LogicInstruction::SetMarker {
            type_: LMarkerControl::Remove,
            id: {
                let mut value = LVar::new("id");
                value.set_num(7.0);
                value
            },
            p1: LVar::new("p1"),
            p2: LVar::new("p2"),
            p3: LVar::new("p3"),
        }
        .run(&mut exec);
        assert!(!exec.markers.contains_key(&7));
        assert!(matches!(
            exec.marker_events.last(),
            Some(LogicMarkerEvent::Removed { id: 7 })
        ));
    }

    #[test]
    fn logic_statements_convert_to_runtime_instructions_with_assembler_vars() {
        let mut assembler = LogicAssembler::new();
        let mut exec = LogicExecutor::new();
        exec.max_ipt = 60;
        exec.map_locales.insert("title".into(), "Localized".into());

        LogicStatement::LocalePrint {
            value: "\"title\"".into(),
        }
        .to_instruction(&mut assembler)
        .run(&mut exec);
        assert_eq!(exec.text_buffer, "Localized");

        LogicStatement::SetRate {
            amount: "120".into(),
        }
        .to_instruction(&mut assembler)
        .run(&mut exec);
        assert_eq!(exec.ipt, 60);

        LogicStatement::ClientData {
            channel: "\"chan\"".into(),
            value: "\"payload\"".into(),
            reliable: "1".into(),
        }
        .to_instruction(&mut assembler)
        .run(&mut exec);
        assert_eq!(exec.client_data_events[0].channel, "chan");
        assert!(exec.client_data_events[0].reliable);

        LogicStatement::PlaySound {
            positional: true,
            id: "3".into(),
            volume: "0.5".into(),
            pitch: "2".into(),
            pan: "0".into(),
            x: "2".into(),
            y: "3".into(),
            limit: "0".into(),
        }
        .to_instruction(&mut assembler)
        .run(&mut exec);
        assert_eq!(exec.sound_events[0].sound_id, 3);
        assert_eq!(
            (exec.sound_events[0].x, exec.sound_events[0].y),
            (Some(16.0), Some(24.0))
        );

        LogicStatement::MakeMarker {
            type_: "shape".into(),
            id: "5".into(),
            x: "2".into(),
            y: "3".into(),
            replace: "1".into(),
        }
        .to_instruction(&mut assembler)
        .run(&mut exec);
        assert!(exec.markers.contains_key(&5));

        exec.text_buffer = "from statement".into();
        LogicStatement::SetMarker {
            type_: LMarkerControl::FlushText,
            id: "5".into(),
            p1: "1".into(),
            p2: "0".into(),
            p3: "0".into(),
        }
        .to_instruction(&mut assembler)
        .run(&mut exec);
        assert_eq!(exec.markers[&5].text, "from statement");
    }

    #[test]
    fn assembler_resolves_java_global_constants_for_runtime_scripts() {
        let mut assembler = LogicAssembler::new();

        assert_eq!(assembler.instruction_var("false").num(), 0.0);
        assert_eq!(assembler.instruction_var("true").num(), 1.0);
        assert_eq!(
            assembler.instruction_var("null").value(),
            LVarValue::Object(None)
        );
        assert_eq!(assembler.instruction_var("@ctrlProcessor").numi(), 1);
        assert_eq!(assembler.instruction_var("@health").obj(), Some("@health"));
        assert_eq!(
            assembler.instruction_var("@sharded").obj(),
            Some("@sharded")
        );
        assert_eq!(assembler.instruction_var("@dagger").obj(), Some("@dagger"));
        assert_eq!(
            assembler.instruction_var("@sandstorm").obj(),
            Some("@sandstorm")
        );
        assert!(assembler.instruction_var("@pi").num() > 3.14);

        let unknown = assembler.instruction_var("@notAJavaGlobal");
        assert_eq!(unknown.value(), LVarValue::Object(None));
        assert!(!unknown.constant);
    }

    #[test]
    fn executor_from_source_shares_vars_and_resolves_jump_labels() {
        let mut exec = LogicExecutor::from_source("set a 1\nop add b a 2\nprint b", false).unwrap();

        assert_eq!(exec.instructions.len(), 3);
        assert_eq!(exec.run_steps(10), 3);
        assert_eq!(exec.text_buffer, "3");
        assert_eq!(exec.var_by_name("a").unwrap().num(), 1.0);
        assert_eq!(exec.var_by_name("b").unwrap().num(), 3.0);

        let mut loop_exec = LogicExecutor::from_source(
            "set a 0\nloop:\nop add a a 1\njump loop lessThan a 3\nprint a",
            false,
        )
        .unwrap();
        assert_eq!(loop_exec.run_steps(20), 8);
        assert_eq!(loop_exec.text_buffer, "3");
        assert_eq!(loop_exec.var_by_name("a").unwrap().num(), 3.0);

        let missing = LogicExecutor::from_source("jump nowhere always x false", false).unwrap_err();
        assert!(missing.message.contains("Unknown jump location 'nowhere'"));
    }

    #[test]
    fn condition_ops_match_java_order_symbols_and_tests() {
        assert_eq!(ConditionOp::ALL.len(), 8);
        assert_eq!(ConditionOp::Equal.ordinal(), 0);
        assert_eq!(ConditionOp::StrictEqual.ordinal(), 6);
        assert_eq!(ConditionOp::Always.ordinal(), 7);
        assert_eq!(ConditionOp::from_ordinal(7), Some(ConditionOp::Always));
        assert_eq!(ConditionOp::from_ordinal(8), None);
        assert_eq!(ConditionOp::NotEqual.java_name(), "notEqual");
        assert_eq!(
            ConditionOp::by_java_name("lessThanEq"),
            Some(ConditionOp::LessThanEq)
        );
        assert_eq!(ConditionOp::by_java_name("<="), None);

        let symbols: Vec<_> = ConditionOp::ALL.iter().map(|op| op.symbol()).collect();
        assert_eq!(
            symbols,
            vec!["==", "not", "<", "<=", ">", ">=", "===", "always"]
        );

        assert!(ConditionOp::Equal.test_numbers(1.0, 1.0 + 0.0000005));
        assert!(!ConditionOp::NotEqual.test_numbers(1.0, 1.0 + 0.0000005));
        assert!(ConditionOp::LessThan.test_numbers(1.0, 2.0));
        assert!(ConditionOp::GreaterThanEq.test_numbers(2.0, 2.0));
        assert!(ConditionOp::Always.test_numbers(f64::NAN, f64::NAN));

        assert!(ConditionOp::StrictEqual
            .test_values(ConditionValue::Number(2.0), ConditionValue::Number(2.0)));
        assert!(!ConditionOp::StrictEqual
            .test_values(ConditionValue::Number(2.0), ConditionValue::Object("2")));
        assert!(ConditionOp::Equal.test_values(
            ConditionValue::Object("core"),
            ConditionValue::Object("core")
        ));
        assert!(ConditionOp::NotEqual.test_values(
            ConditionValue::Object("core"),
            ConditionValue::Object("vault")
        ));
        assert_eq!(ConditionOp::Always.to_string(), "always");
    }

    #[test]
    fn radar_sort_and_target_match_java_order_and_predicates() {
        assert_eq!(RadarSort::ALL.len(), 5);
        assert_eq!(
            RadarSort::ALL,
            [
                RadarSort::Distance,
                RadarSort::Health,
                RadarSort::Shield,
                RadarSort::Armor,
                RadarSort::MaxHealth
            ]
        );
        assert_eq!(
            RadarSort::ALL
                .iter()
                .map(|sort| sort.wire_name())
                .collect::<Vec<_>>(),
            vec!["distance", "health", "shield", "armor", "maxHealth"]
        );
        assert_eq!(RadarSort::MaxHealth.ordinal(), 4);
        assert_eq!(RadarSort::from_ordinal(5), None);
        assert_eq!(
            RadarSort::by_wire_name("maxHealth"),
            Some(RadarSort::MaxHealth)
        );
        assert_eq!(RadarSort::by_wire_name("missing"), None);

        let mut unit = RadarUnitView::new(3.0, 4.0, 2);
        unit.health = 10.0;
        unit.shield = 5.0;
        unit.armor = 2.5;
        unit.max_health = 30.0;
        assert_eq!(RadarSort::Distance.score(0.0, 0.0, &unit), -25.0);
        assert_eq!(RadarSort::Health.score(0.0, 0.0, &unit), 10.0);
        assert_eq!(RadarSort::Shield.score(0.0, 0.0, &unit), 5.0);
        assert_eq!(RadarSort::Armor.score(0.0, 0.0, &unit), 2.5);
        assert_eq!(RadarSort::MaxHealth.score(0.0, 0.0, &unit), 30.0);

        assert_eq!(RadarTarget::ALL.len(), 8);
        assert_eq!(
            RadarTarget::ALL,
            [
                RadarTarget::Any,
                RadarTarget::Enemy,
                RadarTarget::Ally,
                RadarTarget::Player,
                RadarTarget::Attacker,
                RadarTarget::Flying,
                RadarTarget::Boss,
                RadarTarget::Ground
            ]
        );
        assert_eq!(
            RadarTarget::ALL
                .iter()
                .map(|target| target.wire_name())
                .collect::<Vec<_>>(),
            vec!["any", "enemy", "ally", "player", "attacker", "flying", "boss", "ground"]
        );
        assert_eq!(
            RadarTarget::by_wire_name("attacker"),
            Some(RadarTarget::Attacker)
        );
        assert_eq!(RadarTarget::by_wire_name("missing"), None);
        assert_eq!(RadarTarget::Ground.ordinal(), 7);
        assert_eq!(RadarTarget::from_ordinal(8), None);

        assert!(RadarTarget::Any.matches(1, &unit));
        assert!(RadarTarget::Enemy.matches(1, &unit));
        assert!(!RadarTarget::Enemy.matches(1, &RadarUnitView::new(0.0, 0.0, 0)));
        assert!(RadarTarget::Ally.matches(2, &unit));
        assert!(!RadarTarget::Ally.matches(1, &unit));

        unit.is_player = true;
        unit.can_shoot = true;
        unit.is_flying = true;
        unit.is_boss = true;
        unit.is_grounded = false;
        assert!(RadarTarget::Player.matches(1, &unit));
        assert!(RadarTarget::Attacker.matches(1, &unit));
        assert!(RadarTarget::Flying.matches(1, &unit));
        assert!(RadarTarget::Boss.matches(1, &unit));
        assert!(!RadarTarget::Ground.matches(1, &unit));
    }

    #[test]
    fn pure_logic_enums_match_java_order_and_sets() {
        assert_eq!(LogicRule::ALL.len(), 29);
        assert_eq!(LogicRule::CurrentWaveTime.ordinal(), 0);
        assert_eq!(LogicRule::PauseDisabled.ordinal(), 18);
        assert_eq!(
            &LogicRule::ALL[LogicRule::ALL.len() - 2..],
            [LogicRule::RtsMinWeight, LogicRule::RtsMinSquad]
        );
        assert_eq!(LogicRule::RtsMinWeight.wire_name(), "rtsMinWeight");
        assert_eq!(LogicRule::from_ordinal(29), None);

        assert_eq!(
            FetchType::ALL,
            [
                FetchType::Unit,
                FetchType::UnitCount,
                FetchType::Player,
                FetchType::PlayerCount,
                FetchType::Core,
                FetchType::CoreCount,
                FetchType::Build,
                FetchType::BuildCount
            ]
        );
        assert_eq!(
            FetchType::ALL
                .iter()
                .map(|value| value.wire_name())
                .collect::<Vec<_>>(),
            vec![
                "unit",
                "unitCount",
                "player",
                "playerCount",
                "core",
                "coreCount",
                "build",
                "buildCount"
            ]
        );
        assert_eq!(FetchType::BuildCount.ordinal(), 7);
        assert_eq!(FetchType::from_ordinal(8), None);

        assert_eq!(
            QueryType::ALL,
            [QueryType::Unit, QueryType::Building, QueryType::Bullet]
        );
        assert_eq!(QueryType::QUERYABLE, [QueryType::Unit, QueryType::Building]);
        assert_eq!(QueryType::Bullet.wire_name(), "bullet");
        assert_eq!(QueryType::from_ordinal(3), None);

        assert_eq!(GraphicsType::ALL.len(), 16);
        assert_eq!(GraphicsType::Clear.ordinal(), 0);
        assert_eq!(GraphicsType::LineRect.ordinal(), 6);
        assert_eq!(GraphicsType::Reset.ordinal(), 15);
        assert_eq!(GraphicsType::from_ordinal(15), Some(GraphicsType::Reset));
        assert_eq!(GraphicsType::from_ordinal(16), None);
        assert_eq!(GraphicsType::LineRect.wire_name(), "lineRect");
        assert_eq!(
            GraphicsType::by_wire_name("triangle"),
            Some(GraphicsType::Triangle)
        );
        assert_eq!(GraphicsType::by_wire_name("missing"), None);

        assert_eq!(QueryShape::ALL, [QueryShape::Circle, QueryShape::Rect]);
        assert_eq!(
            QueryShape::ALL
                .iter()
                .map(|value| value.wire_name())
                .collect::<Vec<_>>(),
            vec!["circle", "rect"]
        );
        assert_eq!(QueryShape::from_ordinal(2), None);

        assert_eq!(
            MessageType::ALL,
            [
                MessageType::Notify,
                MessageType::Announce,
                MessageType::Toast,
                MessageType::Mission
            ]
        );
        assert_eq!(MessageType::Mission.ordinal(), 3);
        assert_eq!(MessageType::Toast.wire_name(), "toast");

        assert_eq!(
            CutsceneAction::ALL,
            [
                CutsceneAction::Pan,
                CutsceneAction::Zoom,
                CutsceneAction::Stop
            ]
        );
        assert_eq!(CutsceneAction::Stop.wire_name(), "stop");
        assert_eq!(CutsceneAction::from_ordinal(3), None);

        assert_eq!(
            TileLayer::ALL,
            [
                TileLayer::Floor,
                TileLayer::Ore,
                TileLayer::Block,
                TileLayer::Building
            ]
        );
        assert_eq!(
            TileLayer::SETTABLE,
            [TileLayer::Floor, TileLayer::Ore, TileLayer::Block]
        );
        assert!(TileLayer::Floor.is_settable());
        assert!(TileLayer::Block.is_settable());
        assert!(!TileLayer::Building.is_settable());
        assert_eq!(TileLayer::Building.wire_name(), "building");
    }

    #[test]
    fn unit_control_locate_and_categories_match_java_small_logic_files() {
        assert_eq!(LUnitControl::ALL.len(), 21);
        assert_eq!(LUnitControl::Idle.ordinal(), 0);
        assert_eq!(LUnitControl::AutoPathfind.ordinal(), 5);
        assert_eq!(LUnitControl::Unbind.ordinal(), 20);
        assert_eq!(LUnitControl::from_ordinal(21), None);
        assert_eq!(
            LUnitControl::ALL
                .iter()
                .map(|value| value.wire_name())
                .collect::<Vec<_>>(),
            vec![
                "idle",
                "stop",
                "move",
                "approach",
                "pathfind",
                "autoPathfind",
                "boost",
                "target",
                "targetp",
                "itemDrop",
                "itemTake",
                "payDrop",
                "payTake",
                "payEnter",
                "mine",
                "flag",
                "build",
                "deconstruct",
                "getBlock",
                "within",
                "unbind"
            ]
        );
        assert_eq!(LUnitControl::Move.params(), ["x", "y"]);
        assert_eq!(LUnitControl::Targetp.params(), ["unit", "shoot"]);
        assert_eq!(
            LUnitControl::Build.params(),
            ["x", "y", "block", "rotation", "config"]
        );
        assert_eq!(
            LUnitControl::GetBlock.params(),
            ["x", "y", "type", "building", "floor"]
        );
        assert_eq!(
            LUnitControl::Within.params(),
            ["x", "y", "radius", "result"]
        );
        assert_eq!(LUnitControl::PayDrop.params(), [] as [&str; 0]);

        assert_eq!(
            LLocate::ALL,
            [
                LLocate::Ore,
                LLocate::Building,
                LLocate::Spawn,
                LLocate::Damaged
            ]
        );
        assert_eq!(
            LLocate::ALL
                .iter()
                .map(|value| value.wire_name())
                .collect::<Vec<_>>(),
            vec!["ore", "building", "spawn", "damaged"]
        );
        assert_eq!(LLocate::Damaged.ordinal(), 3);
        assert_eq!(LLocate::from_ordinal(4), None);

        assert_eq!(LCategory::ALL.len(), 7);
        assert_eq!(
            LCategory::ALL
                .iter()
                .map(|category| (category.id, category.name, category.icon))
                .collect::<Vec<_>>(),
            vec![
                (0, "unknown", None),
                (1, "io", Some("logicSmall")),
                (2, "block", Some("effectSmall")),
                (3, "operation", Some("settingsSmall")),
                (4, "control", Some("rotateSmall")),
                (5, "unit", Some("unitsSmall")),
                (6, "world", Some("terrainSmall"))
            ]
        );
        assert_eq!(LCategory::ALL[0].color_rgba, 0x4c4c4cff);
        assert_eq!(LCategory::ALL[1].color_rgba, 0xa08a8aff);
        assert_eq!(LCategory::ALL[2].color_rgba, 0xd4816bff);
        assert_eq!(LCategory::ALL[3].color_rgba, 0x877badff);
        assert_eq!(LCategory::ALL[4].color_rgba, 0x6bb2b2ff);
        assert_eq!(LCategory::ALL[5].color_rgba, 0xc7b59dff);
        assert_eq!(LCategory::ALL[6].color_rgba, 0x6b84d4ff);

        let unit = LCategory::by_name("unit").unwrap();
        assert_eq!(unit.id, 5);
        assert_eq!(unit.localized_key(), "lcategory.unit");
        assert_eq!(unit.description_key(), "lcategory.unit.description");
        assert_eq!(LCategory::by_name("missing"), None);
    }

    #[test]
    fn marker_controls_match_java_order_and_params() {
        assert_eq!(LMarkerControl::ALL.len(), 25);
        assert_eq!(LMarkerControl::Remove.ordinal(), 0);
        assert_eq!(LMarkerControl::Shape.ordinal(), 12);
        assert_eq!(LMarkerControl::Colori.ordinal(), 24);
        assert_eq!(
            LMarkerControl::from_ordinal(20).unwrap(),
            LMarkerControl::Texture
        );
        assert_eq!(LMarkerControl::from_ordinal(25), None);
        assert_eq!(LMarkerControl::EndPos.wire_name(), "endPos");
        assert_eq!(LMarkerControl::Shape.params(), ["sides", "fill", "outline"]);
        assert_eq!(LMarkerControl::Texture.params(), ["printFlush", "name"]);
    }
}
