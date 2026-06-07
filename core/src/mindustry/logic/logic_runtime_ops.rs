use crate::mindustry::ui::format_icon_tokens_like_java;
use crate::mindustry::world::meta::BlockFlag;

use std::collections::BTreeMap;

use super::*;

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

    let mut text = format_icon_tokens_like_java(&exec.text_buffer);
    if let Some(key) = text.strip_prefix('@') {
        if let Some(localized) = exec.map_locales.get(key).cloned() {
            text = localized;
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flush_message_formats_icons_and_preserves_non_tokens_like_java() {
        let mut exec = LogicExecutor::new();
        exec.map_locales
            .insert("mission".into(), "mission :play:".into());
        exec.text_buffer = "hello :play: keep :missing: and 127.0.0.1:6567".into();

        let mut out_success = LVar::new("ok");
        exec_flush_message_runtime(
            &mut exec,
            MessageType::Announce,
            &LVar::new("duration"),
            &mut out_success,
        );

        assert_eq!(out_success.value(), LVarValue::Number(1.0));
        assert_eq!(
            exec.message_events,
            vec![LogicMessageEvent {
                type_: MessageType::Announce,
                text: format_icon_tokens_like_java(
                    "hello :play: keep :missing: and 127.0.0.1:6567"
                ),
                duration: 0.0,
            }]
        );
        assert!(exec.text_buffer.is_empty());

        exec.text_buffer = "@mission".into();
        exec_flush_message_runtime(
            &mut exec,
            MessageType::Mission,
            &LVar::new("duration"),
            &mut out_success,
        );
        assert_eq!(exec.rules.mission, "mission :play:");
    }
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
