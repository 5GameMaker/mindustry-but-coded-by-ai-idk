use crate::mindustry::{ctype::ContentType, world::meta::BlockFlag};

use super::{
    condition_op_test_vars, exec_apply_status_runtime, exec_client_data_runtime,
    exec_control_runtime, exec_cutscene_runtime, exec_draw_flush_runtime, exec_effect_runtime,
    exec_explosion_runtime, exec_fetch_runtime, exec_flush_message_runtime, exec_get_block_runtime,
    exec_get_flag_runtime, exec_locale_print_runtime, exec_make_marker_runtime,
    exec_play_sound_runtime, exec_print_flush_runtime, exec_query_runtime, exec_radar_runtime,
    exec_read_runtime, exec_sense_runtime, exec_set_block_runtime, exec_set_flag_runtime,
    exec_set_marker_runtime, exec_set_prop_runtime, exec_set_rate_runtime, exec_set_rule_runtime,
    exec_spawn_bullet_runtime, exec_spawn_unit_runtime, exec_spawn_wave_runtime, exec_sync_runtime,
    exec_unit_bind_runtime, exec_unit_control_runtime, exec_unit_locate_runtime,
    exec_unit_radar_runtime, exec_weather_sense_runtime, exec_weather_set_runtime,
    exec_write_runtime, first_logic_placeholder, logic_color_channel_to_byte,
    logic_var_strict_equal, lookup_logic_content_name, print_logic_value, rgba_to_double_bits,
    unpack_double_color, ConditionOp, CutsceneAction, FetchType, GraphicsType, LAccess, LLocate,
    LMarkerControl, LUnitControl, LVar, LogicDisplayCommand, LogicExecutor, LogicOp, LogicRule,
    MessageType, QueryShape, QueryType, RadarSort, RadarTarget, TileLayer,
};

#[derive(Debug, Clone, PartialEq)]
pub enum LogicInstruction {
    Draw {
        type_: GraphicsType,
        x: LVar,
        y: LVar,
        p1: LVar,
        p2: LVar,
        p3: LVar,
        p4: LVar,
    },
    Control {
        type_: LAccess,
        target: LVar,
        p1: LVar,
        p2: LVar,
        p3: LVar,
        p4: LVar,
    },
    Radar {
        target1: RadarTarget,
        target2: RadarTarget,
        target3: RadarTarget,
        sort: RadarSort,
        radar: LVar,
        sort_order: LVar,
        output: LVar,
        last_target: Option<String>,
    },
    UnitBind {
        type_: LVar,
    },
    UnitControl {
        type_: LUnitControl,
        p1: LVar,
        p2: LVar,
        p3: LVar,
        p4: LVar,
        p5: LVar,
    },
    UnitRadar {
        target1: RadarTarget,
        target2: RadarTarget,
        target3: RadarTarget,
        sort: RadarSort,
        sort_order: LVar,
        output: LVar,
        last_target: Option<String>,
    },
    UnitLocate {
        locate: LLocate,
        flag: BlockFlag,
        enemy: LVar,
        ore: LVar,
        out_x: LVar,
        out_y: LVar,
        out_found: LVar,
        out_build: LVar,
    },
    Query {
        shape: QueryShape,
        type_: QueryType,
        team: LVar,
        x: LVar,
        y: LVar,
        w: LVar,
        h: LVar,
    },
    GetBlock {
        layer: TileLayer,
        result: LVar,
        x: LVar,
        y: LVar,
    },
    SetBlock {
        layer: TileLayer,
        block: LVar,
        x: LVar,
        y: LVar,
        team: LVar,
        rotation: LVar,
    },
    Fetch {
        type_: FetchType,
        result: LVar,
        team: LVar,
        index: LVar,
        extra: LVar,
    },
    GetFlag {
        result: LVar,
        flag: LVar,
    },
    SetFlag {
        flag: LVar,
        value: LVar,
    },
    SpawnUnit {
        type_: LVar,
        x: LVar,
        y: LVar,
        rotation: LVar,
        team: LVar,
        result: LVar,
    },
    ApplyStatus {
        clear: bool,
        effect: String,
        unit: LVar,
        duration: LVar,
    },
    SpawnWave {
        x: LVar,
        y: LVar,
        natural: LVar,
    },
    Effect {
        type_name: String,
        x: LVar,
        y: LVar,
        rotation: LVar,
        color: LVar,
        data: LVar,
    },
    Explosion {
        team: LVar,
        x: LVar,
        y: LVar,
        radius: LVar,
        damage: LVar,
        air: LVar,
        ground: LVar,
        pierce: LVar,
        effect: LVar,
    },
    SetRule {
        rule: LogicRule,
        value: LVar,
        p1: LVar,
        p2: LVar,
        p3: LVar,
        p4: LVar,
    },
    FlushMessage {
        type_: MessageType,
        duration: LVar,
        out_success: LVar,
    },
    Cutscene {
        action: CutsceneAction,
        p1: LVar,
        p2: LVar,
        p3: LVar,
        p4: LVar,
    },
    LocalePrint {
        value: LVar,
    },
    DrawFlush {
        target: LVar,
    },
    PrintFlush {
        target: LVar,
    },
    SetRate {
        amount: LVar,
    },
    Sync {
        variable: LVar,
    },
    SpawnBullet {
        result: LVar,
        from: LVar,
        weapon: LVar,
        x: LVar,
        y: LVar,
        rotation: LVar,
        team: LVar,
        owner: LVar,
        damage: LVar,
        velocity_scl: LVar,
        life_scl: LVar,
        aim_x: LVar,
        aim_y: LVar,
    },
    WeatherSense {
        to: LVar,
        weather: LVar,
    },
    WeatherSet {
        weather: LVar,
        state: LVar,
    },
    SetProp {
        type_: LVar,
        of: LVar,
        value: LVar,
    },
    ClientData {
        channel: LVar,
        value: LVar,
        reliable: LVar,
    },
    PlaySound {
        positional: bool,
        id: LVar,
        volume: LVar,
        pitch: LVar,
        pan: LVar,
        x: LVar,
        y: LVar,
        limit: LVar,
    },
    SetMarker {
        type_: LMarkerControl,
        id: LVar,
        p1: LVar,
        p2: LVar,
        p3: LVar,
    },
    MakeMarker {
        type_name: String,
        id: LVar,
        x: LVar,
        y: LVar,
        replace: LVar,
    },
    Set {
        from: LVar,
        to: LVar,
    },
    Op {
        op: LogicOp,
        a: LVar,
        b: LVar,
        dest: LVar,
    },
    Select {
        op: ConditionOp,
        result: LVar,
        comp0: LVar,
        comp1: LVar,
        a: LVar,
        b: LVar,
    },
    End,
    Noop,
    Print {
        value: LVar,
    },
    PrintChar {
        value: LVar,
    },
    Format {
        value: LVar,
    },
    Jump {
        op: ConditionOp,
        value: LVar,
        compare: LVar,
        address: i32,
    },
    Wait {
        value: LVar,
        cur_time: f32,
    },
    Stop,
    GetLink {
        output: LVar,
        index: LVar,
    },
    Read {
        target: LVar,
        position: LVar,
        output: LVar,
    },
    Write {
        target: LVar,
        position: LVar,
        value: LVar,
    },
    Sense {
        from: LVar,
        to: LVar,
        type_: LVar,
    },
    Lookup {
        dest: LVar,
        from: LVar,
        type_: ContentType,
    },
    PackColor {
        result: LVar,
        r: LVar,
        g: LVar,
        b: LVar,
        a: LVar,
    },
    UnpackColor {
        r: LVar,
        g: LVar,
        b: LVar,
        a: LVar,
        value: LVar,
    },
}

macro_rules! visit_lvars {
    ($visitor:expr $(, $var:expr)* $(,)?) => {
        {
            $(
                $visitor($var);
            )*
        }
    };
}

impl LogicInstruction {
    fn for_each_var_mut(&mut self, visitor: &mut impl FnMut(&mut LVar)) {
        match self {
            LogicInstruction::Draw {
                x,
                y,
                p1,
                p2,
                p3,
                p4,
                ..
            } => visit_lvars!(visitor, x, y, p1, p2, p3, p4),
            LogicInstruction::Control {
                target,
                p1,
                p2,
                p3,
                p4,
                ..
            } => visit_lvars!(visitor, target, p1, p2, p3, p4),
            LogicInstruction::Radar {
                radar,
                sort_order,
                output,
                ..
            } => visit_lvars!(visitor, radar, sort_order, output),
            LogicInstruction::UnitBind { type_ } => visit_lvars!(visitor, type_),
            LogicInstruction::UnitControl {
                p1, p2, p3, p4, p5, ..
            } => {
                visit_lvars!(visitor, p1, p2, p3, p4, p5)
            }
            LogicInstruction::UnitRadar {
                sort_order, output, ..
            } => visit_lvars!(visitor, sort_order, output),
            LogicInstruction::UnitLocate {
                enemy,
                ore,
                out_x,
                out_y,
                out_found,
                out_build,
                ..
            } => visit_lvars!(visitor, enemy, ore, out_x, out_y, out_found, out_build),
            LogicInstruction::Query {
                team, x, y, w, h, ..
            } => {
                visit_lvars!(visitor, team, x, y, w, h)
            }
            LogicInstruction::GetBlock { result, x, y, .. } => {
                visit_lvars!(visitor, result, x, y)
            }
            LogicInstruction::SetBlock {
                block,
                x,
                y,
                team,
                rotation,
                ..
            } => visit_lvars!(visitor, block, x, y, team, rotation),
            LogicInstruction::Fetch {
                result,
                team,
                index,
                extra,
                ..
            } => visit_lvars!(visitor, result, team, index, extra),
            LogicInstruction::GetFlag { result, flag } => visit_lvars!(visitor, result, flag),
            LogicInstruction::SetFlag { flag, value } => visit_lvars!(visitor, flag, value),
            LogicInstruction::SpawnUnit {
                type_,
                x,
                y,
                rotation,
                team,
                result,
            } => visit_lvars!(visitor, type_, x, y, rotation, team, result),
            LogicInstruction::ApplyStatus { unit, duration, .. } => {
                visit_lvars!(visitor, unit, duration)
            }
            LogicInstruction::SpawnWave { x, y, natural } => {
                visit_lvars!(visitor, x, y, natural)
            }
            LogicInstruction::Effect {
                x,
                y,
                rotation,
                color,
                data,
                ..
            } => visit_lvars!(visitor, x, y, rotation, color, data),
            LogicInstruction::Explosion {
                team,
                x,
                y,
                radius,
                damage,
                air,
                ground,
                pierce,
                effect,
            } => visit_lvars!(visitor, team, x, y, radius, damage, air, ground, pierce, effect),
            LogicInstruction::SetRule {
                value,
                p1,
                p2,
                p3,
                p4,
                ..
            } => visit_lvars!(visitor, value, p1, p2, p3, p4),
            LogicInstruction::FlushMessage {
                duration,
                out_success,
                ..
            } => visit_lvars!(visitor, duration, out_success),
            LogicInstruction::Cutscene { p1, p2, p3, p4, .. } => {
                visit_lvars!(visitor, p1, p2, p3, p4)
            }
            LogicInstruction::LocalePrint { value }
            | LogicInstruction::SetRate { amount: value }
            | LogicInstruction::Sync { variable: value }
            | LogicInstruction::Print { value }
            | LogicInstruction::PrintChar { value }
            | LogicInstruction::Format { value }
            | LogicInstruction::Wait { value, .. } => visit_lvars!(visitor, value),
            LogicInstruction::DrawFlush { target } | LogicInstruction::PrintFlush { target } => {
                visit_lvars!(visitor, target)
            }
            LogicInstruction::SpawnBullet {
                result,
                from,
                weapon,
                x,
                y,
                rotation,
                team,
                owner,
                damage,
                velocity_scl,
                life_scl,
                aim_x,
                aim_y,
            } => visit_lvars!(
                visitor,
                result,
                from,
                weapon,
                x,
                y,
                rotation,
                team,
                owner,
                damage,
                velocity_scl,
                life_scl,
                aim_x,
                aim_y
            ),
            LogicInstruction::WeatherSense { to, weather } => visit_lvars!(visitor, to, weather),
            LogicInstruction::WeatherSet { weather, state } => {
                visit_lvars!(visitor, weather, state)
            }
            LogicInstruction::SetProp { type_, of, value } => {
                visit_lvars!(visitor, type_, of, value)
            }
            LogicInstruction::ClientData {
                channel,
                value,
                reliable,
            } => visit_lvars!(visitor, channel, value, reliable),
            LogicInstruction::PlaySound {
                id,
                volume,
                pitch,
                pan,
                x,
                y,
                limit,
                ..
            } => visit_lvars!(visitor, id, volume, pitch, pan, x, y, limit),
            LogicInstruction::SetMarker { id, p1, p2, p3, .. } => {
                visit_lvars!(visitor, id, p1, p2, p3)
            }
            LogicInstruction::MakeMarker {
                id, x, y, replace, ..
            } => visit_lvars!(visitor, id, x, y, replace),
            LogicInstruction::Set { from, to } => visit_lvars!(visitor, from, to),
            LogicInstruction::Op { a, b, dest, .. } => visit_lvars!(visitor, a, b, dest),
            LogicInstruction::Select {
                result,
                comp0,
                comp1,
                a,
                b,
                ..
            } => visit_lvars!(visitor, result, comp0, comp1, a, b),
            LogicInstruction::Jump { value, compare, .. } => {
                visit_lvars!(visitor, value, compare)
            }
            LogicInstruction::GetLink { output, index } => visit_lvars!(visitor, output, index),
            LogicInstruction::Read {
                target,
                position,
                output,
            } => visit_lvars!(visitor, target, position, output),
            LogicInstruction::Write {
                target,
                position,
                value,
            } => visit_lvars!(visitor, target, position, value),
            LogicInstruction::Sense { from, to, type_ } => visit_lvars!(visitor, from, to, type_),
            LogicInstruction::Lookup { dest, from, .. } => visit_lvars!(visitor, dest, from),
            LogicInstruction::PackColor { result, r, g, b, a } => {
                visit_lvars!(visitor, result, r, g, b, a)
            }
            LogicInstruction::UnpackColor { r, g, b, a, value } => {
                visit_lvars!(visitor, r, g, b, a, value)
            }
            LogicInstruction::End | LogicInstruction::Noop | LogicInstruction::Stop => {}
        }
    }

    fn for_each_var(&self, visitor: &mut impl FnMut(&LVar)) {
        match self {
            LogicInstruction::Draw {
                x,
                y,
                p1,
                p2,
                p3,
                p4,
                ..
            } => visit_lvars!(visitor, x, y, p1, p2, p3, p4),
            LogicInstruction::Control {
                target,
                p1,
                p2,
                p3,
                p4,
                ..
            } => visit_lvars!(visitor, target, p1, p2, p3, p4),
            LogicInstruction::Radar {
                radar,
                sort_order,
                output,
                ..
            } => visit_lvars!(visitor, radar, sort_order, output),
            LogicInstruction::UnitBind { type_ } => visit_lvars!(visitor, type_),
            LogicInstruction::UnitControl {
                p1, p2, p3, p4, p5, ..
            } => {
                visit_lvars!(visitor, p1, p2, p3, p4, p5)
            }
            LogicInstruction::UnitRadar {
                sort_order, output, ..
            } => visit_lvars!(visitor, sort_order, output),
            LogicInstruction::UnitLocate {
                enemy,
                ore,
                out_x,
                out_y,
                out_found,
                out_build,
                ..
            } => visit_lvars!(visitor, enemy, ore, out_x, out_y, out_found, out_build),
            LogicInstruction::Query {
                team, x, y, w, h, ..
            } => {
                visit_lvars!(visitor, team, x, y, w, h)
            }
            LogicInstruction::GetBlock { result, x, y, .. } => {
                visit_lvars!(visitor, result, x, y)
            }
            LogicInstruction::SetBlock {
                block,
                x,
                y,
                team,
                rotation,
                ..
            } => visit_lvars!(visitor, block, x, y, team, rotation),
            LogicInstruction::Fetch {
                result,
                team,
                index,
                extra,
                ..
            } => visit_lvars!(visitor, result, team, index, extra),
            LogicInstruction::GetFlag { result, flag } => visit_lvars!(visitor, result, flag),
            LogicInstruction::SetFlag { flag, value } => visit_lvars!(visitor, flag, value),
            LogicInstruction::SpawnUnit {
                type_,
                x,
                y,
                rotation,
                team,
                result,
            } => visit_lvars!(visitor, type_, x, y, rotation, team, result),
            LogicInstruction::ApplyStatus { unit, duration, .. } => {
                visit_lvars!(visitor, unit, duration)
            }
            LogicInstruction::SpawnWave { x, y, natural } => {
                visit_lvars!(visitor, x, y, natural)
            }
            LogicInstruction::Effect {
                x,
                y,
                rotation,
                color,
                data,
                ..
            } => visit_lvars!(visitor, x, y, rotation, color, data),
            LogicInstruction::Explosion {
                team,
                x,
                y,
                radius,
                damage,
                air,
                ground,
                pierce,
                effect,
            } => visit_lvars!(visitor, team, x, y, radius, damage, air, ground, pierce, effect),
            LogicInstruction::SetRule {
                value,
                p1,
                p2,
                p3,
                p4,
                ..
            } => visit_lvars!(visitor, value, p1, p2, p3, p4),
            LogicInstruction::FlushMessage {
                duration,
                out_success,
                ..
            } => visit_lvars!(visitor, duration, out_success),
            LogicInstruction::Cutscene { p1, p2, p3, p4, .. } => {
                visit_lvars!(visitor, p1, p2, p3, p4)
            }
            LogicInstruction::LocalePrint { value }
            | LogicInstruction::SetRate { amount: value }
            | LogicInstruction::Sync { variable: value }
            | LogicInstruction::Print { value }
            | LogicInstruction::PrintChar { value }
            | LogicInstruction::Format { value }
            | LogicInstruction::Wait { value, .. } => visit_lvars!(visitor, value),
            LogicInstruction::DrawFlush { target } | LogicInstruction::PrintFlush { target } => {
                visit_lvars!(visitor, target)
            }
            LogicInstruction::SpawnBullet {
                result,
                from,
                weapon,
                x,
                y,
                rotation,
                team,
                owner,
                damage,
                velocity_scl,
                life_scl,
                aim_x,
                aim_y,
            } => visit_lvars!(
                visitor,
                result,
                from,
                weapon,
                x,
                y,
                rotation,
                team,
                owner,
                damage,
                velocity_scl,
                life_scl,
                aim_x,
                aim_y
            ),
            LogicInstruction::WeatherSense { to, weather } => visit_lvars!(visitor, to, weather),
            LogicInstruction::WeatherSet { weather, state } => {
                visit_lvars!(visitor, weather, state)
            }
            LogicInstruction::SetProp { type_, of, value } => {
                visit_lvars!(visitor, type_, of, value)
            }
            LogicInstruction::ClientData {
                channel,
                value,
                reliable,
            } => visit_lvars!(visitor, channel, value, reliable),
            LogicInstruction::PlaySound {
                id,
                volume,
                pitch,
                pan,
                x,
                y,
                limit,
                ..
            } => visit_lvars!(visitor, id, volume, pitch, pan, x, y, limit),
            LogicInstruction::SetMarker { id, p1, p2, p3, .. } => {
                visit_lvars!(visitor, id, p1, p2, p3)
            }
            LogicInstruction::MakeMarker {
                id, x, y, replace, ..
            } => visit_lvars!(visitor, id, x, y, replace),
            LogicInstruction::Set { from, to } => visit_lvars!(visitor, from, to),
            LogicInstruction::Op { a, b, dest, .. } => visit_lvars!(visitor, a, b, dest),
            LogicInstruction::Select {
                result,
                comp0,
                comp1,
                a,
                b,
                ..
            } => visit_lvars!(visitor, result, comp0, comp1, a, b),
            LogicInstruction::Jump { value, compare, .. } => {
                visit_lvars!(visitor, value, compare)
            }
            LogicInstruction::GetLink { output, index } => visit_lvars!(visitor, output, index),
            LogicInstruction::Read {
                target,
                position,
                output,
            } => visit_lvars!(visitor, target, position, output),
            LogicInstruction::Write {
                target,
                position,
                value,
            } => visit_lvars!(visitor, target, position, value),
            LogicInstruction::Sense { from, to, type_ } => visit_lvars!(visitor, from, to, type_),
            LogicInstruction::Lookup { dest, from, .. } => visit_lvars!(visitor, dest, from),
            LogicInstruction::PackColor { result, r, g, b, a } => {
                visit_lvars!(visitor, result, r, g, b, a)
            }
            LogicInstruction::UnpackColor { r, g, b, a, value } => {
                visit_lvars!(visitor, r, g, b, a, value)
            }
            LogicInstruction::End | LogicInstruction::Noop | LogicInstruction::Stop => {}
        }
    }

    pub(super) fn load_shared_vars(&mut self, exec: &LogicExecutor) {
        self.for_each_var_mut(&mut |var| {
            if var.constant {
                return;
            }
            if let Some(shared) = exec.var_by_name(&var.name) {
                *var = shared.clone();
            }
        });
    }

    pub(super) fn load_shared_vars_from(&mut self, vars: &[LVar], counter: &LVar) {
        self.for_each_var_mut(&mut |var| {
            if var.constant {
                return;
            }
            if var.name == "@counter" {
                *var = counter.clone();
            } else if let Some(shared) = vars.iter().find(|shared| shared.name == var.name) {
                *var = shared.clone();
            }
        });
    }

    pub(super) fn store_shared_vars(&self, exec: &mut LogicExecutor) {
        self.for_each_var(&mut |var| {
            if !var.constant {
                exec.upsert_runtime_var(var);
            }
        });
    }

    pub fn run(&mut self, exec: &mut LogicExecutor) {
        match self {
            LogicInstruction::Draw {
                type_,
                x,
                y,
                p1,
                p2,
                p3,
                p4,
            } => {
                if exec.headless || exec.graphics_buffer.len() >= LogicExecutor::MAX_GRAPHICS_BUFFER
                {
                    return;
                }

                if *type_ == GraphicsType::Print {
                    exec.text_buffer.clear();
                    return;
                }

                if let Some(command) =
                    LogicDisplayCommand::from_draw_instruction(*type_, x, y, p1, p2, p3, p4)
                {
                    exec.graphics_buffer.push(command);
                }
            }
            LogicInstruction::Control {
                type_,
                target,
                p1,
                p2,
                p3,
                p4,
            } => {
                exec_control_runtime(exec, *type_, target, p1, p2, p3, p4);
            }
            LogicInstruction::Radar {
                target1,
                target2,
                target3,
                sort,
                radar,
                sort_order,
                output,
                last_target,
            } => {
                let targeted = exec_radar_runtime(
                    exec, *target1, *target2, *target3, *sort, radar, sort_order,
                );
                *last_target = targeted.clone();
                output.set_obj(targeted);
            }
            LogicInstruction::UnitBind { type_ } => {
                exec_unit_bind_runtime(exec, type_);
            }
            LogicInstruction::UnitControl {
                type_,
                p1,
                p2,
                p3,
                p4,
                p5,
            } => {
                exec_unit_control_runtime(exec, *type_, p1, p2, p3, p4, p5);
            }
            LogicInstruction::UnitRadar {
                target1,
                target2,
                target3,
                sort,
                sort_order,
                output,
                last_target,
            } => {
                let targeted =
                    exec_unit_radar_runtime(exec, *target1, *target2, *target3, *sort, sort_order);
                *last_target = targeted.clone();
                output.set_obj(targeted);
            }
            LogicInstruction::UnitLocate {
                locate,
                flag,
                enemy,
                ore,
                out_x,
                out_y,
                out_found,
                out_build,
            } => {
                exec_unit_locate_runtime(
                    exec, *locate, *flag, enemy, ore, out_x, out_y, out_found, out_build,
                );
            }
            LogicInstruction::Query {
                shape,
                type_,
                team,
                x,
                y,
                w,
                h,
            } => {
                exec_query_runtime(exec, *shape, *type_, team, x, y, w, h);
            }
            LogicInstruction::GetBlock {
                layer,
                result,
                x,
                y,
            } => {
                exec_get_block_runtime(exec, *layer, result, x, y);
            }
            LogicInstruction::SetBlock {
                layer,
                block,
                x,
                y,
                team,
                rotation,
            } => {
                exec_set_block_runtime(exec, *layer, block, x, y, team, rotation);
            }
            LogicInstruction::Fetch {
                type_,
                result,
                team,
                index,
                extra,
            } => {
                exec_fetch_runtime(exec, *type_, result, team, index, extra);
            }
            LogicInstruction::GetFlag { result, flag } => {
                exec_get_flag_runtime(exec, result, flag);
            }
            LogicInstruction::SetFlag { flag, value } => {
                exec_set_flag_runtime(exec, flag, value);
            }
            LogicInstruction::SpawnUnit {
                type_,
                x,
                y,
                rotation,
                team,
                result,
            } => {
                exec_spawn_unit_runtime(exec, type_, x, y, rotation, team, result);
            }
            LogicInstruction::ApplyStatus {
                clear,
                effect,
                unit,
                duration,
            } => {
                exec_apply_status_runtime(exec, *clear, effect, unit, duration);
            }
            LogicInstruction::SpawnWave { x, y, natural } => {
                exec_spawn_wave_runtime(exec, x, y, natural);
            }
            LogicInstruction::Effect {
                type_name,
                x,
                y,
                rotation,
                color,
                data,
            } => {
                exec_effect_runtime(exec, type_name, x, y, rotation, color, data);
            }
            LogicInstruction::Explosion {
                team,
                x,
                y,
                radius,
                damage,
                air,
                ground,
                pierce,
                effect,
            } => {
                exec_explosion_runtime(
                    exec, team, x, y, radius, damage, air, ground, pierce, effect,
                );
            }
            LogicInstruction::SetRule {
                rule,
                value,
                p1,
                p2,
                p3,
                p4,
            } => {
                exec_set_rule_runtime(exec, *rule, value, p1, p2, p3, p4);
            }
            LogicInstruction::FlushMessage {
                type_,
                duration,
                out_success,
            } => {
                exec_flush_message_runtime(exec, *type_, duration, out_success);
            }
            LogicInstruction::Cutscene {
                action,
                p1,
                p2,
                p3,
                p4,
            } => {
                exec_cutscene_runtime(exec, *action, p1, p2, p3, p4);
            }
            LogicInstruction::LocalePrint { value } => {
                exec_locale_print_runtime(exec, value);
            }
            LogicInstruction::DrawFlush { target } => {
                exec_draw_flush_runtime(exec, target);
            }
            LogicInstruction::PrintFlush { target } => {
                exec_print_flush_runtime(exec, target);
            }
            LogicInstruction::SetRate { amount } => {
                exec_set_rate_runtime(exec, amount);
            }
            LogicInstruction::Sync { variable } => {
                exec_sync_runtime(exec, variable);
            }
            LogicInstruction::SpawnBullet {
                result,
                from,
                weapon,
                x,
                y,
                rotation,
                team,
                owner,
                damage,
                velocity_scl,
                life_scl,
                aim_x,
                aim_y,
            } => {
                exec_spawn_bullet_runtime(
                    exec,
                    result,
                    from,
                    weapon,
                    x,
                    y,
                    rotation,
                    team,
                    owner,
                    damage,
                    velocity_scl,
                    life_scl,
                    aim_x,
                    aim_y,
                );
            }
            LogicInstruction::WeatherSense { to, weather } => {
                exec_weather_sense_runtime(exec, to, weather);
            }
            LogicInstruction::WeatherSet { weather, state } => {
                exec_weather_set_runtime(exec, weather, state);
            }
            LogicInstruction::SetProp { type_, of, value } => {
                exec_set_prop_runtime(exec, type_, of, value);
            }
            LogicInstruction::ClientData {
                channel,
                value,
                reliable,
            } => {
                exec_client_data_runtime(exec, channel, value, reliable);
            }
            LogicInstruction::PlaySound {
                positional,
                id,
                volume,
                pitch,
                pan,
                x,
                y,
                limit,
            } => {
                exec_play_sound_runtime(exec, *positional, id, volume, pitch, pan, x, y, limit);
            }
            LogicInstruction::SetMarker {
                type_,
                id,
                p1,
                p2,
                p3,
            } => {
                exec_set_marker_runtime(exec, *type_, id, p1, p2, p3);
            }
            LogicInstruction::MakeMarker {
                type_name,
                id,
                x,
                y,
                replace,
            } => {
                exec_make_marker_runtime(exec, type_name, id, x, y, replace);
            }
            LogicInstruction::Set { from, to } => {
                if !to.constant {
                    to.set_from(from);
                }
            }
            LogicInstruction::Op { op, a, b, dest } => {
                if dest.constant {
                    return;
                }

                if *op == LogicOp::StrictEqual {
                    dest.set_num(logic_var_strict_equal(a, b) as u8 as f64);
                } else if op.unary() {
                    if let Some(value) = op.eval_unary(a.num()) {
                        dest.set_num(value);
                    }
                } else if let Some(value) = op.eval_binary(a.num(), b.num()) {
                    dest.set_num(value);
                }
            }
            LogicInstruction::Select {
                op,
                result,
                comp0,
                comp1,
                a,
                b,
            } => {
                if result.constant {
                    return;
                }

                if condition_op_test_vars(*op, comp0, comp1) {
                    result.set_from(a);
                } else {
                    result.set_from(b);
                }
            }
            LogicInstruction::End => {
                exec.counter.numval = exec.instructions.len() as f64;
            }
            LogicInstruction::Noop => {}
            LogicInstruction::Print { value } => {
                let text = print_logic_value(value);
                exec.push_text_bounded(&text);
            }
            LogicInstruction::PrintChar { value } => {
                if exec.text_buffer.len() >= LogicExecutor::MAX_TEXT_BUFFER {
                    return;
                }

                if value.is_obj {
                    return;
                }

                let code = value.numval.floor() as u32;
                if let Some(ch) = char::from_u32(code) {
                    exec.push_text_bounded(&ch.to_string());
                }
            }
            LogicInstruction::Format { value } => {
                if exec.text_buffer.len() >= LogicExecutor::MAX_TEXT_BUFFER {
                    return;
                }

                if let Some((index, _number)) = first_logic_placeholder(&exec.text_buffer) {
                    let text = print_logic_value(value);
                    exec.text_buffer.replace_range(index..index + 3, &text);
                }
            }
            LogicInstruction::Jump {
                op,
                value,
                compare,
                address,
            } => {
                if *address != -1 && condition_op_test_vars(*op, value, compare) {
                    exec.counter.numval = *address as f64;
                }
            }
            LogicInstruction::Wait { value, cur_time } => {
                let seconds = value.num();
                if seconds <= 0.0 {
                    exec.yield_ = true;
                    *cur_time = 0.0;
                } else if *cur_time as f64 >= seconds {
                    *cur_time = 0.0;
                } else {
                    exec.counter.numval -= 1.0;
                    exec.yield_ = true;
                    *cur_time += 1.0 / 60.0;
                }
            }
            LogicInstruction::Stop => {
                exec.counter.numval -= 1.0;
                exec.yield_ = true;
            }
            LogicInstruction::GetLink { output, index } => {
                let address = index.numi();
                output.set_obj(
                    (address >= 0)
                        .then(|| exec.links.get(address as usize).cloned())
                        .flatten(),
                );
            }
            LogicInstruction::Read {
                target,
                position,
                output,
            } => {
                exec_read_runtime(exec, target, position, output);
            }
            LogicInstruction::Write {
                target,
                position,
                value,
            } => {
                exec_write_runtime(exec, target, position, value);
            }
            LogicInstruction::Sense { from, to, type_ } => {
                exec_sense_runtime(exec, from, type_, to);
            }
            LogicInstruction::Lookup { dest, from, type_ } => {
                let value = lookup_logic_content_name(*type_, from.numi());
                dest.set_obj(value.map(str::to_string));
            }
            LogicInstruction::PackColor { result, r, g, b, a } => {
                result.set_num(rgba_to_double_bits(
                    logic_color_channel_to_byte(r.num()),
                    logic_color_channel_to_byte(g.num()),
                    logic_color_channel_to_byte(b.num()),
                    logic_color_channel_to_byte(a.num()),
                ));
            }
            LogicInstruction::UnpackColor { r, g, b, a, value } => {
                let (rv, gv, bv, av) = unpack_double_color(value.num());
                r.set_num(rv);
                g.set_num(gv);
                b.set_num(bv);
                a.set_num(av);
            }
        }
    }
}
