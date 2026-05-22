use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicStatement {
    Invalid,
    Read {
        output: String,
        target: String,
        address: String,
    },
    Write {
        input: String,
        target: String,
        address: String,
    },
    Draw {
        type_: GraphicsType,
        x: String,
        y: String,
        p1: String,
        p2: String,
        p3: String,
        p4: String,
    },
    Print {
        value: String,
    },
    PrintChar {
        value: String,
    },
    Format {
        value: String,
    },
    LocalePrint {
        value: String,
    },
    DrawFlush {
        target: String,
    },
    PrintFlush {
        target: String,
    },
    GetLink {
        output: String,
        address: String,
    },
    SetRate {
        amount: String,
    },
    Sync {
        variable: String,
    },
    Set {
        to: String,
        from: String,
    },
    Operation {
        op: LogicOp,
        dest: String,
        a: String,
        b: String,
    },
    Select {
        result: String,
        op: ConditionOp,
        comp0: String,
        comp1: String,
        a: String,
        b: String,
    },
    Wait {
        value: String,
    },
    Stop,
    End,
    PackColor {
        result: String,
        r: String,
        g: String,
        b: String,
        a: String,
    },
    UnpackColor {
        r: String,
        g: String,
        b: String,
        a: String,
        value: String,
    },
    Lookup {
        type_: ContentType,
        result: String,
        id: String,
    },
    Jump {
        dest_index: i32,
        op: ConditionOp,
        value: String,
        compare: String,
    },
    Control {
        type_: LAccess,
        target: String,
        p1: String,
        p2: String,
        p3: String,
        p4: String,
    },
    Radar {
        target1: RadarTarget,
        target2: RadarTarget,
        target3: RadarTarget,
        sort: RadarSort,
        radar: String,
        sort_order: String,
        output: String,
    },
    Sensor {
        to: String,
        from: String,
        type_: String,
    },
    UnitBind {
        type_: String,
    },
    UnitControl {
        type_: LUnitControl,
        p1: String,
        p2: String,
        p3: String,
        p4: String,
        p5: String,
    },
    UnitRadar {
        target1: RadarTarget,
        target2: RadarTarget,
        target3: RadarTarget,
        sort: RadarSort,
        radar: String,
        sort_order: String,
        output: String,
    },
    UnitLocate {
        locate: LLocate,
        flag: BlockFlag,
        enemy: String,
        ore: String,
        out_x: String,
        out_y: String,
        out_found: String,
        out_build: String,
    },
    Query {
        shape: QueryShape,
        type_: QueryType,
        team: String,
        x: String,
        y: String,
        w: String,
        h: String,
    },
    GetBlock {
        layer: TileLayer,
        result: String,
        x: String,
        y: String,
    },
    SetBlock {
        layer: TileLayer,
        block: String,
        x: String,
        y: String,
        team: String,
        rotation: String,
    },
    SpawnUnit {
        type_: String,
        x: String,
        y: String,
        rotation: String,
        team: String,
        result: String,
    },
    ApplyStatus {
        clear: bool,
        effect: String,
        unit: String,
        duration: String,
    },
    SpawnWave {
        x: String,
        y: String,
        natural: String,
    },
    SpawnBullet {
        result: String,
        from: String,
        index: String,
        x: String,
        y: String,
        rotation: String,
        team: String,
        owner: String,
        damage: String,
        velocity_scl: String,
        life_scl: String,
        aim_x: String,
        aim_y: String,
    },
    WeatherSense {
        to: String,
        weather: String,
    },
    WeatherSet {
        weather: String,
        state: String,
    },
    Effect {
        type_: String,
        x: String,
        y: String,
        sizerot: String,
        color: String,
        data: String,
    },
    Explosion {
        team: String,
        x: String,
        y: String,
        radius: String,
        damage: String,
        air: String,
        ground: String,
        pierce: String,
        effect: String,
    },
    SetRule {
        rule: LogicRule,
        value: String,
        p1: String,
        p2: String,
        p3: String,
        p4: String,
    },
    Fetch {
        type_: FetchType,
        result: String,
        team: String,
        index: String,
        extra: String,
    },
    GetFlag {
        result: String,
        flag: String,
    },
    SetFlag {
        flag: String,
        value: String,
    },
    SetProp {
        type_: String,
        of: String,
        value: String,
    },
    FlushMessage {
        type_: MessageType,
        duration: String,
        out_success: String,
    },
    Cutscene {
        action: CutsceneAction,
        p1: String,
        p2: String,
        p3: String,
        p4: String,
    },
    ClientData {
        channel: String,
        value: String,
        reliable: String,
    },
    PlaySound {
        positional: bool,
        id: String,
        volume: String,
        pitch: String,
        pan: String,
        x: String,
        y: String,
        limit: String,
    },
    SetMarker {
        type_: LMarkerControl,
        id: String,
        p1: String,
        p2: String,
        p3: String,
    },
    MakeMarker {
        type_: String,
        id: String,
        x: String,
        y: String,
        replace: String,
    },
}

impl LogicStatement {
    pub fn invalid() -> Self {
        Self::Invalid
    }

    pub fn read() -> Self {
        Self::Read {
            output: "result".into(),
            target: "cell1".into(),
            address: "0".into(),
        }
    }

    pub fn write() -> Self {
        Self::Write {
            input: "result".into(),
            target: "cell1".into(),
            address: "0".into(),
        }
    }

    pub fn draw() -> Self {
        Self::Draw {
            type_: GraphicsType::Clear,
            x: "0".into(),
            y: "0".into(),
            p1: "0".into(),
            p2: "0".into(),
            p3: "0".into(),
            p4: "0".into(),
        }
    }

    pub fn print() -> Self {
        Self::Print {
            value: "\"frog\"".into(),
        }
    }

    pub fn print_char() -> Self {
        Self::PrintChar { value: "65".into() }
    }

    pub fn format() -> Self {
        Self::Format {
            value: "\"frog\"".into(),
        }
    }

    pub fn locale_print() -> Self {
        Self::LocalePrint {
            value: "\"name\"".into(),
        }
    }

    pub fn draw_flush() -> Self {
        Self::DrawFlush {
            target: "display1".into(),
        }
    }

    pub fn print_flush() -> Self {
        Self::PrintFlush {
            target: "message1".into(),
        }
    }

    pub fn get_link() -> Self {
        Self::GetLink {
            output: "result".into(),
            address: "0".into(),
        }
    }

    pub fn set_rate() -> Self {
        Self::SetRate {
            amount: "10".into(),
        }
    }

    pub fn sync() -> Self {
        Self::Sync {
            variable: "var".into(),
        }
    }

    pub fn set() -> Self {
        Self::Set {
            to: "result".into(),
            from: "0".into(),
        }
    }

    pub fn operation() -> Self {
        Self::Operation {
            op: LogicOp::Add,
            dest: "result".into(),
            a: "a".into(),
            b: "b".into(),
        }
    }

    pub fn select() -> Self {
        Self::Select {
            result: "result".into(),
            op: ConditionOp::NotEqual,
            comp0: "x".into(),
            comp1: "false".into(),
            a: "a".into(),
            b: "b".into(),
        }
    }

    pub fn wait() -> Self {
        Self::Wait {
            value: "0.5".into(),
        }
    }

    pub fn stop() -> Self {
        Self::Stop
    }

    pub fn end() -> Self {
        Self::End
    }

    pub fn pack_color() -> Self {
        Self::PackColor {
            result: "result".into(),
            r: "1".into(),
            g: "0".into(),
            b: "0".into(),
            a: "1".into(),
        }
    }

    pub fn unpack_color() -> Self {
        Self::UnpackColor {
            r: "r".into(),
            g: "g".into(),
            b: "b".into(),
            a: "a".into(),
            value: "color".into(),
        }
    }

    pub fn lookup() -> Self {
        Self::Lookup {
            type_: ContentType::Item,
            result: "result".into(),
            id: "0".into(),
        }
    }

    pub fn jump() -> Self {
        Self::Jump {
            dest_index: 0,
            op: ConditionOp::NotEqual,
            value: "x".into(),
            compare: "false".into(),
        }
    }

    pub fn control() -> Self {
        Self::Control {
            type_: LAccess::Enabled,
            target: "block1".into(),
            p1: "0".into(),
            p2: "0".into(),
            p3: "0".into(),
            p4: "0".into(),
        }
    }

    pub fn radar() -> Self {
        Self::Radar {
            target1: RadarTarget::Enemy,
            target2: RadarTarget::Any,
            target3: RadarTarget::Any,
            sort: RadarSort::Distance,
            radar: "turret1".into(),
            sort_order: "1".into(),
            output: "result".into(),
        }
    }

    pub fn sensor() -> Self {
        Self::Sensor {
            to: "result".into(),
            from: "block1".into(),
            type_: "@copper".into(),
        }
    }

    pub fn unit_bind() -> Self {
        Self::UnitBind {
            type_: "@poly".into(),
        }
    }

    pub fn unit_control() -> Self {
        Self::UnitControl {
            type_: LUnitControl::Move,
            p1: "0".into(),
            p2: "0".into(),
            p3: "0".into(),
            p4: "0".into(),
            p5: "0".into(),
        }
    }

    pub fn unit_radar() -> Self {
        Self::UnitRadar {
            target1: RadarTarget::Enemy,
            target2: RadarTarget::Any,
            target3: RadarTarget::Any,
            sort: RadarSort::Distance,
            radar: "0".into(),
            sort_order: "1".into(),
            output: "result".into(),
        }
    }

    pub fn unit_locate() -> Self {
        Self::UnitLocate {
            locate: LLocate::Building,
            flag: BlockFlag::Core,
            enemy: "true".into(),
            ore: "@copper".into(),
            out_x: "outx".into(),
            out_y: "outy".into(),
            out_found: "found".into(),
            out_build: "building".into(),
        }
    }

    pub fn query() -> Self {
        Self::Query {
            shape: QueryShape::Circle,
            type_: QueryType::Unit,
            team: "null".into(),
            x: "0".into(),
            y: "0".into(),
            w: "10".into(),
            h: "10".into(),
        }
    }

    pub fn get_block() -> Self {
        Self::GetBlock {
            layer: TileLayer::Block,
            result: "result".into(),
            x: "0".into(),
            y: "0".into(),
        }
    }

    pub fn set_block() -> Self {
        Self::SetBlock {
            layer: TileLayer::Block,
            block: "@air".into(),
            x: "0".into(),
            y: "0".into(),
            team: "@derelict".into(),
            rotation: "0".into(),
        }
    }

    pub fn spawn_unit() -> Self {
        Self::SpawnUnit {
            type_: "@dagger".into(),
            x: "10".into(),
            y: "10".into(),
            rotation: "90".into(),
            team: "@sharded".into(),
            result: "result".into(),
        }
    }

    pub fn apply_status() -> Self {
        Self::ApplyStatus {
            clear: false,
            effect: "wet".into(),
            unit: "unit".into(),
            duration: "10".into(),
        }
    }

    pub fn spawn_wave() -> Self {
        Self::SpawnWave {
            x: "10".into(),
            y: "10".into(),
            natural: "false".into(),
        }
    }

    pub fn spawn_bullet() -> Self {
        Self::SpawnBullet {
            result: "result".into(),
            from: "@dagger".into(),
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
            aim_y: "-1".into(),
        }
    }

    pub fn weather_sense() -> Self {
        Self::WeatherSense {
            to: "result".into(),
            weather: "@rain".into(),
        }
    }

    pub fn weather_set() -> Self {
        Self::WeatherSet {
            weather: "@rain".into(),
            state: "true".into(),
        }
    }

    pub fn effect() -> Self {
        Self::Effect {
            type_: "warn".into(),
            x: "0".into(),
            y: "0".into(),
            sizerot: "2".into(),
            color: "%ffaaff".into(),
            data: "".into(),
        }
    }

    pub fn explosion() -> Self {
        Self::Explosion {
            team: "@crux".into(),
            x: "0".into(),
            y: "0".into(),
            radius: "5".into(),
            damage: "50".into(),
            air: "true".into(),
            ground: "true".into(),
            pierce: "false".into(),
            effect: "true".into(),
        }
    }

    pub fn set_rule() -> Self {
        Self::SetRule {
            rule: LogicRule::WaveSpacing,
            value: "10".into(),
            p1: "0".into(),
            p2: "0".into(),
            p3: "100".into(),
            p4: "100".into(),
        }
    }

    pub fn fetch() -> Self {
        Self::Fetch {
            type_: FetchType::Unit,
            result: "result".into(),
            team: "@sharded".into(),
            index: "0".into(),
            extra: "@conveyor".into(),
        }
    }

    pub fn get_flag() -> Self {
        Self::GetFlag {
            result: "result".into(),
            flag: "\"flag\"".into(),
        }
    }

    pub fn set_flag() -> Self {
        Self::SetFlag {
            flag: "\"flag\"".into(),
            value: "true".into(),
        }
    }

    pub fn set_prop() -> Self {
        Self::SetProp {
            type_: "@copper".into(),
            of: "block1".into(),
            value: "0".into(),
        }
    }

    pub fn flush_message() -> Self {
        Self::FlushMessage {
            type_: MessageType::Announce,
            duration: "3".into(),
            out_success: "@wait".into(),
        }
    }

    pub fn cutscene() -> Self {
        Self::Cutscene {
            action: CutsceneAction::Pan,
            p1: "100".into(),
            p2: "100".into(),
            p3: "0.06".into(),
            p4: "0".into(),
        }
    }

    pub fn client_data() -> Self {
        Self::ClientData {
            channel: "\"frog\"".into(),
            value: "\"bar\"".into(),
            reliable: "0".into(),
        }
    }

    pub fn play_sound() -> Self {
        Self::PlaySound {
            positional: false,
            id: "@sfx-shoot".into(),
            volume: "1".into(),
            pitch: "1".into(),
            pan: "0".into(),
            x: "@thisx".into(),
            y: "@thisy".into(),
            limit: "true".into(),
        }
    }

    pub fn set_marker() -> Self {
        Self::SetMarker {
            type_: LMarkerControl::Pos,
            id: "0".into(),
            p1: "0".into(),
            p2: "0".into(),
            p3: "0".into(),
        }
    }

    pub fn make_marker() -> Self {
        Self::MakeMarker {
            type_: "shape".into(),
            id: "0".into(),
            x: "0".into(),
            y: "0".into(),
            replace: "true".into(),
        }
    }

    pub fn opcode(&self) -> &'static str {
        match self {
            LogicStatement::Invalid => "noop",
            LogicStatement::Read { .. } => "read",
            LogicStatement::Write { .. } => "write",
            LogicStatement::Draw { .. } => "draw",
            LogicStatement::Print { .. } => "print",
            LogicStatement::PrintChar { .. } => "printchar",
            LogicStatement::Format { .. } => "format",
            LogicStatement::LocalePrint { .. } => "localeprint",
            LogicStatement::DrawFlush { .. } => "drawflush",
            LogicStatement::PrintFlush { .. } => "printflush",
            LogicStatement::GetLink { .. } => "getlink",
            LogicStatement::SetRate { .. } => "setrate",
            LogicStatement::Sync { .. } => "sync",
            LogicStatement::Set { .. } => "set",
            LogicStatement::Operation { .. } => "op",
            LogicStatement::Select { .. } => "select",
            LogicStatement::Wait { .. } => "wait",
            LogicStatement::Stop => "stop",
            LogicStatement::End => "end",
            LogicStatement::PackColor { .. } => "packcolor",
            LogicStatement::UnpackColor { .. } => "unpackcolor",
            LogicStatement::Lookup { .. } => "lookup",
            LogicStatement::Jump { .. } => "jump",
            LogicStatement::Control { .. } => "control",
            LogicStatement::Radar { .. } => "radar",
            LogicStatement::Sensor { .. } => "sensor",
            LogicStatement::UnitBind { .. } => "ubind",
            LogicStatement::UnitControl { .. } => "ucontrol",
            LogicStatement::UnitRadar { .. } => "uradar",
            LogicStatement::UnitLocate { .. } => "ulocate",
            LogicStatement::Query { .. } => "query",
            LogicStatement::GetBlock { .. } => "getblock",
            LogicStatement::SetBlock { .. } => "setblock",
            LogicStatement::SpawnUnit { .. } => "spawn",
            LogicStatement::ApplyStatus { .. } => "status",
            LogicStatement::SpawnWave { .. } => "spawnwave",
            LogicStatement::SpawnBullet { .. } => "bullet",
            LogicStatement::WeatherSense { .. } => "weathersense",
            LogicStatement::WeatherSet { .. } => "weatherset",
            LogicStatement::Effect { .. } => "effect",
            LogicStatement::Explosion { .. } => "explosion",
            LogicStatement::SetRule { .. } => "setrule",
            LogicStatement::Fetch { .. } => "fetch",
            LogicStatement::GetFlag { .. } => "getflag",
            LogicStatement::SetFlag { .. } => "setflag",
            LogicStatement::SetProp { .. } => "setprop",
            LogicStatement::FlushMessage { .. } => "message",
            LogicStatement::Cutscene { .. } => "cutscene",
            LogicStatement::ClientData { .. } => "clientdata",
            LogicStatement::PlaySound { .. } => "playsound",
            LogicStatement::SetMarker { .. } => "setmarker",
            LogicStatement::MakeMarker { .. } => "makemarker",
        }
    }

    pub fn category(&self) -> &'static LCategory {
        match self {
            LogicStatement::Invalid => LCategory::by_name("unknown").unwrap(),
            LogicStatement::Read { .. }
            | LogicStatement::Write { .. }
            | LogicStatement::Draw { .. }
            | LogicStatement::Print { .. }
            | LogicStatement::PrintChar { .. }
            | LogicStatement::Format { .. } => LCategory::by_name("io").unwrap(),
            LogicStatement::DrawFlush { .. }
            | LogicStatement::PrintFlush { .. }
            | LogicStatement::GetLink { .. } => LCategory::by_name("block").unwrap(),
            LogicStatement::SetRate { .. }
            | LogicStatement::Sync { .. }
            | LogicStatement::LocalePrint { .. }
            | LogicStatement::Query { .. }
            | LogicStatement::GetBlock { .. }
            | LogicStatement::SetBlock { .. }
            | LogicStatement::SpawnUnit { .. }
            | LogicStatement::ApplyStatus { .. }
            | LogicStatement::SpawnWave { .. }
            | LogicStatement::SpawnBullet { .. }
            | LogicStatement::WeatherSense { .. }
            | LogicStatement::WeatherSet { .. }
            | LogicStatement::Effect { .. }
            | LogicStatement::Explosion { .. }
            | LogicStatement::SetRule { .. }
            | LogicStatement::Fetch { .. }
            | LogicStatement::GetFlag { .. }
            | LogicStatement::SetFlag { .. }
            | LogicStatement::SetProp { .. }
            | LogicStatement::FlushMessage { .. }
            | LogicStatement::Cutscene { .. }
            | LogicStatement::ClientData { .. }
            | LogicStatement::PlaySound { .. }
            | LogicStatement::SetMarker { .. }
            | LogicStatement::MakeMarker { .. } => LCategory::by_name("world").unwrap(),
            LogicStatement::Set { .. }
            | LogicStatement::Operation { .. }
            | LogicStatement::Select { .. }
            | LogicStatement::Lookup { .. }
            | LogicStatement::PackColor { .. }
            | LogicStatement::UnpackColor { .. } => LCategory::by_name("operation").unwrap(),
            LogicStatement::Wait { .. }
            | LogicStatement::Stop
            | LogicStatement::End
            | LogicStatement::Jump { .. } => LCategory::by_name("control").unwrap(),
            LogicStatement::Control { .. }
            | LogicStatement::Radar { .. }
            | LogicStatement::Sensor { .. } => LCategory::by_name("block").unwrap(),
            LogicStatement::UnitBind { .. }
            | LogicStatement::UnitControl { .. }
            | LogicStatement::UnitRadar { .. }
            | LogicStatement::UnitLocate { .. } => LCategory::by_name("unit").unwrap(),
        }
    }

    pub fn privileged(&self) -> bool {
        matches!(
            self,
            LogicStatement::SetRate { .. }
                | LogicStatement::Sync { .. }
                | LogicStatement::LocalePrint { .. }
                | LogicStatement::Query { .. }
                | LogicStatement::GetBlock { .. }
                | LogicStatement::SetBlock { .. }
                | LogicStatement::SpawnUnit { .. }
                | LogicStatement::ApplyStatus { .. }
                | LogicStatement::SpawnWave { .. }
                | LogicStatement::SpawnBullet { .. }
                | LogicStatement::WeatherSense { .. }
                | LogicStatement::WeatherSet { .. }
                | LogicStatement::Effect { .. }
                | LogicStatement::Explosion { .. }
                | LogicStatement::SetRule { .. }
                | LogicStatement::Fetch { .. }
                | LogicStatement::GetFlag { .. }
                | LogicStatement::SetFlag { .. }
                | LogicStatement::SetProp { .. }
                | LogicStatement::FlushMessage { .. }
                | LogicStatement::Cutscene { .. }
                | LogicStatement::ClientData { .. }
                | LogicStatement::PlaySound { .. }
                | LogicStatement::SetMarker { .. }
                | LogicStatement::MakeMarker { .. }
        )
    }

    pub fn tokens(&self) -> Vec<String> {
        match self {
            LogicStatement::Invalid => vec!["noop".into()],
            LogicStatement::Read {
                output,
                target,
                address,
            } => vec![
                "read".into(),
                output.clone(),
                target.clone(),
                address.clone(),
            ],
            LogicStatement::Write {
                input,
                target,
                address,
            } => vec![
                "write".into(),
                input.clone(),
                target.clone(),
                address.clone(),
            ],
            LogicStatement::Draw {
                type_,
                x,
                y,
                p1,
                p2,
                p3,
                p4,
            } => vec![
                "draw".into(),
                type_.wire_name().into(),
                x.clone(),
                y.clone(),
                p1.clone(),
                p2.clone(),
                p3.clone(),
                p4.clone(),
            ],
            LogicStatement::Print { value } => vec!["print".into(), value.clone()],
            LogicStatement::PrintChar { value } => vec!["printchar".into(), value.clone()],
            LogicStatement::Format { value } => vec!["format".into(), value.clone()],
            LogicStatement::LocalePrint { value } => vec!["localeprint".into(), value.clone()],
            LogicStatement::DrawFlush { target } => vec!["drawflush".into(), target.clone()],
            LogicStatement::PrintFlush { target } => vec!["printflush".into(), target.clone()],
            LogicStatement::GetLink { output, address } => {
                vec!["getlink".into(), output.clone(), address.clone()]
            }
            LogicStatement::SetRate { amount } => vec!["setrate".into(), amount.clone()],
            LogicStatement::Sync { variable } => vec!["sync".into(), variable.clone()],
            LogicStatement::Set { to, from } => vec!["set".into(), to.clone(), from.clone()],
            LogicStatement::Operation { op, dest, a, b } => vec![
                "op".into(),
                op.java_name().into(),
                dest.clone(),
                a.clone(),
                b.clone(),
            ],
            LogicStatement::Select {
                result,
                op,
                comp0,
                comp1,
                a,
                b,
            } => vec![
                "select".into(),
                result.clone(),
                op.java_name().into(),
                comp0.clone(),
                comp1.clone(),
                a.clone(),
                b.clone(),
            ],
            LogicStatement::Wait { value } => vec!["wait".into(), value.clone()],
            LogicStatement::Stop => vec!["stop".into()],
            LogicStatement::End => vec!["end".into()],
            LogicStatement::PackColor { result, r, g, b, a } => vec![
                "packcolor".into(),
                result.clone(),
                r.clone(),
                g.clone(),
                b.clone(),
                a.clone(),
            ],
            LogicStatement::UnpackColor { r, g, b, a, value } => vec![
                "unpackcolor".into(),
                r.clone(),
                g.clone(),
                b.clone(),
                a.clone(),
                value.clone(),
            ],
            LogicStatement::Lookup { type_, result, id } => vec![
                "lookup".into(),
                type_.wire_name().into(),
                result.clone(),
                id.clone(),
            ],
            LogicStatement::Jump {
                dest_index,
                op,
                value,
                compare,
            } => vec![
                "jump".into(),
                dest_index.to_string(),
                op.java_name().into(),
                value.clone(),
                compare.clone(),
            ],
            LogicStatement::Control {
                type_,
                target,
                p1,
                p2,
                p3,
                p4,
            } => vec![
                "control".into(),
                type_.wire_name().into(),
                target.clone(),
                p1.clone(),
                p2.clone(),
                p3.clone(),
                p4.clone(),
            ],
            LogicStatement::Radar {
                target1,
                target2,
                target3,
                sort,
                radar,
                sort_order,
                output,
            } => vec![
                "radar".into(),
                target1.wire_name().into(),
                target2.wire_name().into(),
                target3.wire_name().into(),
                sort.wire_name().into(),
                radar.clone(),
                sort_order.clone(),
                output.clone(),
            ],
            LogicStatement::Sensor { to, from, type_ } => {
                vec!["sensor".into(), to.clone(), from.clone(), type_.clone()]
            }
            LogicStatement::UnitBind { type_ } => vec!["ubind".into(), type_.clone()],
            LogicStatement::UnitControl {
                type_,
                p1,
                p2,
                p3,
                p4,
                p5,
            } => vec![
                "ucontrol".into(),
                type_.wire_name().into(),
                p1.clone(),
                p2.clone(),
                p3.clone(),
                p4.clone(),
                p5.clone(),
            ],
            LogicStatement::UnitRadar {
                target1,
                target2,
                target3,
                sort,
                radar,
                sort_order,
                output,
            } => vec![
                "uradar".into(),
                target1.wire_name().into(),
                target2.wire_name().into(),
                target3.wire_name().into(),
                sort.wire_name().into(),
                radar.clone(),
                sort_order.clone(),
                output.clone(),
            ],
            LogicStatement::UnitLocate {
                locate,
                flag,
                enemy,
                ore,
                out_x,
                out_y,
                out_found,
                out_build,
            } => vec![
                "ulocate".into(),
                locate.wire_name().into(),
                flag.wire_name().into(),
                enemy.clone(),
                ore.clone(),
                out_x.clone(),
                out_y.clone(),
                out_found.clone(),
                out_build.clone(),
            ],
            LogicStatement::Query {
                shape,
                type_,
                team,
                x,
                y,
                w,
                h,
            } => vec![
                "query".into(),
                shape.wire_name().into(),
                type_.wire_name().into(),
                team.clone(),
                x.clone(),
                y.clone(),
                w.clone(),
                h.clone(),
            ],
            LogicStatement::GetBlock {
                layer,
                result,
                x,
                y,
            } => vec![
                "getblock".into(),
                layer.wire_name().into(),
                result.clone(),
                x.clone(),
                y.clone(),
            ],
            LogicStatement::SetBlock {
                layer,
                block,
                x,
                y,
                team,
                rotation,
            } => vec![
                "setblock".into(),
                layer.wire_name().into(),
                block.clone(),
                x.clone(),
                y.clone(),
                team.clone(),
                rotation.clone(),
            ],
            LogicStatement::SpawnUnit {
                type_,
                x,
                y,
                rotation,
                team,
                result,
            } => vec![
                "spawn".into(),
                type_.clone(),
                x.clone(),
                y.clone(),
                rotation.clone(),
                team.clone(),
                result.clone(),
            ],
            LogicStatement::ApplyStatus {
                clear,
                effect,
                unit,
                duration,
            } => vec![
                "status".into(),
                clear.to_string(),
                effect.clone(),
                unit.clone(),
                duration.clone(),
            ],
            LogicStatement::SpawnWave { x, y, natural } => {
                vec!["spawnwave".into(), x.clone(), y.clone(), natural.clone()]
            }
            LogicStatement::SpawnBullet {
                result,
                from,
                index,
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
            } => vec![
                "bullet".into(),
                result.clone(),
                from.clone(),
                index.clone(),
                x.clone(),
                y.clone(),
                rotation.clone(),
                team.clone(),
                owner.clone(),
                damage.clone(),
                velocity_scl.clone(),
                life_scl.clone(),
                aim_x.clone(),
                aim_y.clone(),
            ],
            LogicStatement::WeatherSense { to, weather } => {
                vec!["weathersense".into(), to.clone(), weather.clone()]
            }
            LogicStatement::WeatherSet { weather, state } => {
                vec!["weatherset".into(), weather.clone(), state.clone()]
            }
            LogicStatement::Effect {
                type_,
                x,
                y,
                sizerot,
                color,
                data,
            } => vec![
                "effect".into(),
                type_.clone(),
                x.clone(),
                y.clone(),
                sizerot.clone(),
                color.clone(),
                data.clone(),
            ],
            LogicStatement::Explosion {
                team,
                x,
                y,
                radius,
                damage,
                air,
                ground,
                pierce,
                effect,
            } => vec![
                "explosion".into(),
                team.clone(),
                x.clone(),
                y.clone(),
                radius.clone(),
                damage.clone(),
                air.clone(),
                ground.clone(),
                pierce.clone(),
                effect.clone(),
            ],
            LogicStatement::SetRule {
                rule,
                value,
                p1,
                p2,
                p3,
                p4,
            } => vec![
                "setrule".into(),
                rule.wire_name().into(),
                value.clone(),
                p1.clone(),
                p2.clone(),
                p3.clone(),
                p4.clone(),
            ],
            LogicStatement::Fetch {
                type_,
                result,
                team,
                index,
                extra,
            } => vec![
                "fetch".into(),
                type_.wire_name().into(),
                result.clone(),
                team.clone(),
                index.clone(),
                extra.clone(),
            ],
            LogicStatement::GetFlag { result, flag } => {
                vec!["getflag".into(), result.clone(), flag.clone()]
            }
            LogicStatement::SetFlag { flag, value } => {
                vec!["setflag".into(), flag.clone(), value.clone()]
            }
            LogicStatement::SetProp { type_, of, value } => {
                vec!["setprop".into(), type_.clone(), of.clone(), value.clone()]
            }
            LogicStatement::FlushMessage {
                type_,
                duration,
                out_success,
            } => vec![
                "message".into(),
                type_.wire_name().into(),
                duration.clone(),
                out_success.clone(),
            ],
            LogicStatement::Cutscene {
                action,
                p1,
                p2,
                p3,
                p4,
            } => vec![
                "cutscene".into(),
                action.wire_name().into(),
                p1.clone(),
                p2.clone(),
                p3.clone(),
                p4.clone(),
            ],
            LogicStatement::ClientData {
                channel,
                value,
                reliable,
            } => vec![
                "clientdata".into(),
                channel.clone(),
                value.clone(),
                reliable.clone(),
            ],
            LogicStatement::PlaySound {
                positional,
                id,
                volume,
                pitch,
                pan,
                x,
                y,
                limit,
            } => vec![
                "playsound".into(),
                positional.to_string(),
                id.clone(),
                volume.clone(),
                pitch.clone(),
                pan.clone(),
                x.clone(),
                y.clone(),
                limit.clone(),
            ],
            LogicStatement::SetMarker {
                type_,
                id,
                p1,
                p2,
                p3,
            } => vec![
                "setmarker".into(),
                type_.wire_name().into(),
                id.clone(),
                p1.clone(),
                p2.clone(),
                p3.clone(),
            ],
            LogicStatement::MakeMarker {
                type_,
                id,
                x,
                y,
                replace,
            } => vec![
                "makemarker".into(),
                type_.clone(),
                id.clone(),
                x.clone(),
                y.clone(),
                replace.clone(),
            ],
        }
    }

    pub fn write_line(&self) -> String {
        self.tokens().join(" ")
    }

    pub fn read_tokens(tokens: &[String]) -> Option<Self> {
        let opcode = tokens.first()?.as_str();
        Some(match opcode {
            "noop" => Self::Invalid,
            "read" => {
                let mut statement = Self::read();
                if let LogicStatement::Read {
                    output,
                    target,
                    address,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *output = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *target = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *address = tokens[3].clone();
                    }
                }
                statement
            }
            "write" => {
                let mut statement = Self::write();
                if let LogicStatement::Write {
                    input,
                    target,
                    address,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *input = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *target = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *address = tokens[3].clone();
                    }
                }
                statement
            }
            "draw" => {
                let mut statement = Self::draw();
                if let LogicStatement::Draw {
                    type_,
                    x,
                    y,
                    p1,
                    p2,
                    p3,
                    p4,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *type_ = GraphicsType::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *x = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *y = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *p1 = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *p2 = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *p3 = tokens[6].clone();
                    }
                    if tokens.len() > 7 {
                        *p4 = tokens[7].clone();
                    }

                    if *type_ == GraphicsType::Color && p2 == "0" {
                        *p2 = "255".into();
                    }

                    if *type_ == GraphicsType::Print && LogicAlign::by_name(p1).is_some() {
                        p1.insert(0, '@');
                    }
                }
                statement
            }
            "print" => Self::Print {
                value: tokens.get(1).cloned().unwrap_or_else(|| "\"frog\"".into()),
            },
            "printchar" => Self::PrintChar {
                value: tokens.get(1).cloned().unwrap_or_else(|| "65".into()),
            },
            "format" => Self::Format {
                value: tokens.get(1).cloned().unwrap_or_else(|| "\"frog\"".into()),
            },
            "localeprint" => Self::LocalePrint {
                value: tokens.get(1).cloned().unwrap_or_else(|| "\"name\"".into()),
            },
            "drawflush" => Self::DrawFlush {
                target: tokens.get(1).cloned().unwrap_or_else(|| "display1".into()),
            },
            "printflush" => Self::PrintFlush {
                target: tokens.get(1).cloned().unwrap_or_else(|| "message1".into()),
            },
            "getlink" => {
                let mut statement = Self::get_link();
                if let LogicStatement::GetLink { output, address } = &mut statement {
                    if tokens.len() > 1 {
                        *output = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *address = tokens[2].clone();
                    }
                }
                statement
            }
            "setrate" => Self::SetRate {
                amount: tokens.get(1).cloned().unwrap_or_else(|| "10".into()),
            },
            "sync" => Self::Sync {
                variable: tokens.get(1).cloned().unwrap_or_else(|| "var".into()),
            },
            "set" => {
                let mut statement = Self::set();
                if let LogicStatement::Set { to, from } = &mut statement {
                    if tokens.len() > 1 {
                        *to = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *from = tokens[2].clone();
                    }
                }
                statement
            }
            "op" => {
                let mut statement = Self::operation();
                if let LogicStatement::Operation { op, dest, a, b } = &mut statement {
                    if tokens.len() > 1 {
                        *op = LogicOp::by_java_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *dest = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *a = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *b = tokens[4].clone();
                    }
                }
                statement
            }
            "select" => {
                let mut statement = Self::select();
                if let LogicStatement::Select {
                    result,
                    op,
                    comp0,
                    comp1,
                    a,
                    b,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *result = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *op = ConditionOp::by_java_name(&tokens[2])?;
                    }
                    if tokens.len() > 3 {
                        *comp0 = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *comp1 = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *a = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *b = tokens[6].clone();
                    }
                }
                statement
            }
            "wait" => Self::Wait {
                value: tokens.get(1).cloned().unwrap_or_else(|| "0.5".into()),
            },
            "stop" => Self::Stop,
            "end" => Self::End,
            "packcolor" => {
                let mut statement = Self::pack_color();
                if let LogicStatement::PackColor { result, r, g, b, a } = &mut statement {
                    if tokens.len() > 1 {
                        *result = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *r = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *g = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *b = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *a = tokens[5].clone();
                    }
                }
                statement
            }
            "unpackcolor" => {
                let mut statement = Self::unpack_color();
                if let LogicStatement::UnpackColor { r, g, b, a, value } = &mut statement {
                    if tokens.len() > 1 {
                        *r = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *g = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *b = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *a = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *value = tokens[5].clone();
                    }
                }
                statement
            }
            "lookup" => {
                let mut statement = Self::lookup();
                if let LogicStatement::Lookup { type_, result, id } = &mut statement {
                    if tokens.len() > 1 {
                        let value = ContentType::from_wire_name(&tokens[1])?;
                        if !LOOKABLE_CONTENT_TYPES.contains(&value) {
                            return None;
                        }
                        *type_ = value;
                    }
                    if tokens.len() > 2 {
                        *result = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *id = tokens[3].clone();
                    }
                }
                statement
            }
            "jump" => {
                let mut statement = Self::jump();
                if let LogicStatement::Jump {
                    dest_index,
                    op,
                    value,
                    compare,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *dest_index = tokens[1].parse().ok()?;
                    }
                    if tokens.len() > 2 {
                        *op = ConditionOp::by_java_name(&tokens[2])?;
                    }
                    if tokens.len() > 3 {
                        *value = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *compare = tokens[4].clone();
                    }
                }
                statement
            }
            "control" => {
                let mut statement = Self::control();
                if let LogicStatement::Control {
                    type_,
                    target,
                    p1,
                    p2,
                    p3,
                    p4,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        let value = LAccess::by_wire_name(&tokens[1])?;
                        if !value.is_control() {
                            return None;
                        }
                        *type_ = value;
                    }
                    if tokens.len() > 2 {
                        *target = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *p1 = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *p2 = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *p3 = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *p4 = tokens[6].clone();
                    }
                }
                statement
            }
            "radar" => {
                let mut statement = Self::radar();
                if let LogicStatement::Radar {
                    target1,
                    target2,
                    target3,
                    sort,
                    radar,
                    sort_order,
                    output,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *target1 = RadarTarget::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *target2 = RadarTarget::by_wire_name(&tokens[2])?;
                    }
                    if tokens.len() > 3 {
                        *target3 = RadarTarget::by_wire_name(&tokens[3])?;
                    }
                    if tokens.len() > 4 {
                        *sort = RadarSort::by_wire_name(&tokens[4])?;
                    }
                    if tokens.len() > 5 {
                        *radar = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *sort_order = tokens[6].clone();
                    }
                    if tokens.len() > 7 {
                        *output = tokens[7].clone();
                    }
                }
                statement
            }
            "sensor" => {
                let mut statement = Self::sensor();
                if let LogicStatement::Sensor { to, from, type_ } = &mut statement {
                    if tokens.len() > 1 {
                        *to = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *from = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *type_ = tokens[3].clone();
                    }
                }
                statement
            }
            "ubind" => Self::UnitBind {
                type_: tokens.get(1).cloned().unwrap_or_else(|| "@poly".into()),
            },
            "ucontrol" => {
                let mut statement = Self::unit_control();
                if let LogicStatement::UnitControl {
                    type_,
                    p1,
                    p2,
                    p3,
                    p4,
                    p5,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *type_ = LUnitControl::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *p1 = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *p2 = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *p3 = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *p4 = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *p5 = tokens[6].clone();
                    }
                }
                statement
            }
            "uradar" => {
                let mut statement = Self::unit_radar();
                if let LogicStatement::UnitRadar {
                    target1,
                    target2,
                    target3,
                    sort,
                    radar,
                    sort_order,
                    output,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *target1 = RadarTarget::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *target2 = RadarTarget::by_wire_name(&tokens[2])?;
                    }
                    if tokens.len() > 3 {
                        *target3 = RadarTarget::by_wire_name(&tokens[3])?;
                    }
                    if tokens.len() > 4 {
                        *sort = RadarSort::by_wire_name(&tokens[4])?;
                    }
                    if tokens.len() > 5 {
                        *radar = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *sort_order = tokens[6].clone();
                    }
                    if tokens.len() > 7 {
                        *output = tokens[7].clone();
                    }
                }
                statement
            }
            "ulocate" => {
                let mut statement = Self::unit_locate();
                if let LogicStatement::UnitLocate {
                    locate,
                    flag,
                    enemy,
                    ore,
                    out_x,
                    out_y,
                    out_found,
                    out_build,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *locate = LLocate::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        let value = BlockFlag::by_wire_name(&tokens[2])?;
                        if !value.is_logic() {
                            return None;
                        }
                        *flag = value;
                    }
                    if tokens.len() > 3 {
                        *enemy = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *ore = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *out_x = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *out_y = tokens[6].clone();
                    }
                    if tokens.len() > 7 {
                        *out_found = tokens[7].clone();
                    }
                    if tokens.len() > 8 {
                        *out_build = tokens[8].clone();
                    }
                }
                statement
            }
            "query" => {
                let mut statement = Self::query();
                if let LogicStatement::Query {
                    shape,
                    type_,
                    team,
                    x,
                    y,
                    w,
                    h,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *shape = QueryShape::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *type_ = QueryType::by_wire_name(&tokens[2])?;
                    }
                    if tokens.len() > 3 {
                        *team = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *x = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *y = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *w = tokens[6].clone();
                    }
                    if tokens.len() > 7 {
                        *h = tokens[7].clone();
                    }
                }
                statement
            }
            "getblock" => {
                let mut statement = Self::get_block();
                if let LogicStatement::GetBlock {
                    layer,
                    result,
                    x,
                    y,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *layer = TileLayer::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *result = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *x = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *y = tokens[4].clone();
                    }
                }
                statement
            }
            "setblock" => {
                let mut statement = Self::set_block();
                if let LogicStatement::SetBlock {
                    layer,
                    block,
                    x,
                    y,
                    team,
                    rotation,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *layer = TileLayer::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *block = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *x = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *y = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *team = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *rotation = tokens[6].clone();
                    }
                }
                statement
            }
            "spawn" => {
                let mut statement = Self::spawn_unit();
                if let LogicStatement::SpawnUnit {
                    type_,
                    x,
                    y,
                    rotation,
                    team,
                    result,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *type_ = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *x = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *y = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *rotation = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *team = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *result = tokens[6].clone();
                    }
                }
                statement
            }
            "status" => {
                let mut statement = Self::apply_status();
                if let LogicStatement::ApplyStatus {
                    clear,
                    effect,
                    unit,
                    duration,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *clear = java_boolean_value_of(&tokens[1]);
                    }
                    if tokens.len() > 2 {
                        *effect = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *unit = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *duration = tokens[4].clone();
                    }
                }
                statement
            }
            "spawnwave" => {
                let mut statement = Self::spawn_wave();
                if let LogicStatement::SpawnWave { x, y, natural } = &mut statement {
                    if tokens.len() > 1 {
                        *x = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *y = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *natural = tokens[3].clone();
                    }
                }
                statement
            }
            "bullet" => {
                let mut statement = Self::spawn_bullet();
                if let LogicStatement::SpawnBullet {
                    result,
                    from,
                    index,
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
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *result = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *from = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *index = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *x = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *y = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *rotation = tokens[6].clone();
                    }
                    if tokens.len() > 7 {
                        *team = tokens[7].clone();
                    }
                    if tokens.len() > 8 {
                        *owner = tokens[8].clone();
                    }
                    if tokens.len() > 9 {
                        *damage = tokens[9].clone();
                    }
                    if tokens.len() > 10 {
                        *velocity_scl = tokens[10].clone();
                    }
                    if tokens.len() > 11 {
                        *life_scl = tokens[11].clone();
                    }
                    if tokens.len() > 12 {
                        *aim_x = tokens[12].clone();
                    }
                    if tokens.len() > 13 {
                        *aim_y = tokens[13].clone();
                    }
                }
                statement
            }
            "weathersense" => {
                let mut statement = Self::weather_sense();
                if let LogicStatement::WeatherSense { to, weather } = &mut statement {
                    if tokens.len() > 1 {
                        *to = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *weather = tokens[2].clone();
                    }
                }
                statement
            }
            "weatherset" => {
                let mut statement = Self::weather_set();
                if let LogicStatement::WeatherSet { weather, state } = &mut statement {
                    if tokens.len() > 1 {
                        *weather = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *state = tokens[2].clone();
                    }
                }
                statement
            }
            "effect" => {
                let mut statement = Self::effect();
                if let LogicStatement::Effect {
                    type_,
                    x,
                    y,
                    sizerot,
                    color,
                    data,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *type_ = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *x = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *y = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *sizerot = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *color = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *data = tokens[6].clone();
                    }
                }
                statement
            }
            "explosion" => {
                let mut statement = Self::explosion();
                if let LogicStatement::Explosion {
                    team,
                    x,
                    y,
                    radius,
                    damage,
                    air,
                    ground,
                    pierce,
                    effect,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *team = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *x = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *y = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *radius = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *damage = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *air = tokens[6].clone();
                    }
                    if tokens.len() > 7 {
                        *ground = tokens[7].clone();
                    }
                    if tokens.len() > 8 {
                        *pierce = tokens[8].clone();
                    }
                    if tokens.len() > 9 {
                        *effect = tokens[9].clone();
                    }
                }
                statement
            }
            "setrule" => {
                let mut statement = Self::set_rule();
                if let LogicStatement::SetRule {
                    rule,
                    value,
                    p1,
                    p2,
                    p3,
                    p4,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *rule = LogicRule::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *value = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *p1 = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *p2 = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *p3 = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *p4 = tokens[6].clone();
                    }
                }
                statement
            }
            "fetch" => {
                let mut statement = Self::fetch();
                if let LogicStatement::Fetch {
                    type_,
                    result,
                    team,
                    index,
                    extra,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *type_ = FetchType::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *result = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *team = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *index = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *extra = tokens[5].clone();
                    }
                }
                statement
            }
            "getflag" => {
                let mut statement = Self::get_flag();
                if let LogicStatement::GetFlag { result, flag } = &mut statement {
                    if tokens.len() > 1 {
                        *result = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *flag = tokens[2].clone();
                    }
                }
                statement
            }
            "setflag" => {
                let mut statement = Self::set_flag();
                if let LogicStatement::SetFlag { flag, value } = &mut statement {
                    if tokens.len() > 1 {
                        *flag = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *value = tokens[2].clone();
                    }
                }
                statement
            }
            "setprop" => {
                let mut statement = Self::set_prop();
                if let LogicStatement::SetProp { type_, of, value } = &mut statement {
                    if tokens.len() > 1 {
                        *type_ = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *of = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *value = tokens[3].clone();
                    }
                }
                statement
            }
            "message" => {
                let mut statement = Self::flush_message();
                if let LogicStatement::FlushMessage {
                    type_,
                    duration,
                    out_success,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *type_ = MessageType::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *duration = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *out_success = tokens[3].clone();
                    }
                }
                statement
            }
            "cutscene" => {
                let mut statement = Self::cutscene();
                if let LogicStatement::Cutscene {
                    action,
                    p1,
                    p2,
                    p3,
                    p4,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *action = CutsceneAction::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *p1 = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *p2 = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *p3 = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *p4 = tokens[5].clone();
                    }
                }
                statement
            }
            "clientdata" => {
                let mut statement = Self::client_data();
                if let LogicStatement::ClientData {
                    channel,
                    value,
                    reliable,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *channel = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *value = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *reliable = tokens[3].clone();
                    }
                }
                statement
            }
            "playsound" => {
                let mut statement = Self::play_sound();
                if let LogicStatement::PlaySound {
                    positional,
                    id,
                    volume,
                    pitch,
                    pan,
                    x,
                    y,
                    limit,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *positional = java_boolean_value_of(&tokens[1]);
                    }
                    if tokens.len() > 2 {
                        *id = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *volume = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *pitch = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *pan = tokens[5].clone();
                    }
                    if tokens.len() > 6 {
                        *x = tokens[6].clone();
                    }
                    if tokens.len() > 7 {
                        *y = tokens[7].clone();
                    }
                    if tokens.len() > 8 {
                        *limit = tokens[8].clone();
                    }
                }
                statement
            }
            "setmarker" => {
                let mut statement = Self::set_marker();
                if let LogicStatement::SetMarker {
                    type_,
                    id,
                    p1,
                    p2,
                    p3,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *type_ = LMarkerControl::by_wire_name(&tokens[1])?;
                    }
                    if tokens.len() > 2 {
                        *id = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *p1 = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *p2 = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *p3 = tokens[5].clone();
                    }
                }
                statement
            }
            "makemarker" => {
                let mut statement = Self::make_marker();
                if let LogicStatement::MakeMarker {
                    type_,
                    id,
                    x,
                    y,
                    replace,
                } = &mut statement
                {
                    if tokens.len() > 1 {
                        *type_ = tokens[1].clone();
                    }
                    if tokens.len() > 2 {
                        *id = tokens[2].clone();
                    }
                    if tokens.len() > 3 {
                        *x = tokens[3].clone();
                    }
                    if tokens.len() > 4 {
                        *y = tokens[4].clone();
                    }
                    if tokens.len() > 5 {
                        *replace = tokens[5].clone();
                    }
                }
                statement
            }
            _ => return None,
        })
    }

    pub fn to_instruction(&self, assembler: &mut LogicAssembler) -> LogicInstruction {
        match self {
            LogicStatement::Invalid => LogicInstruction::Noop,
            LogicStatement::Read {
                output,
                target,
                address,
            } => LogicInstruction::Read {
                target: assembler.instruction_var(target),
                position: assembler.instruction_var(address),
                output: assembler.instruction_var(output),
            },
            LogicStatement::Write {
                input,
                target,
                address,
            } => LogicInstruction::Write {
                target: assembler.instruction_var(target),
                position: assembler.instruction_var(address),
                value: assembler.instruction_var(input),
            },
            LogicStatement::Draw {
                type_,
                x,
                y,
                p1,
                p2,
                p3,
                p4,
            } => LogicInstruction::Draw {
                type_: *type_,
                x: assembler.instruction_var(x),
                y: assembler.instruction_var(y),
                p1: assembler.instruction_var(p1),
                p2: assembler.instruction_var(p2),
                p3: assembler.instruction_var(p3),
                p4: assembler.instruction_var(p4),
            },
            LogicStatement::Print { value } => LogicInstruction::Print {
                value: assembler.instruction_var(value),
            },
            LogicStatement::PrintChar { value } => LogicInstruction::PrintChar {
                value: assembler.instruction_var(value),
            },
            LogicStatement::Format { value } => LogicInstruction::Format {
                value: assembler.instruction_var(value),
            },
            LogicStatement::LocalePrint { value } => LogicInstruction::LocalePrint {
                value: assembler.instruction_var(value),
            },
            LogicStatement::DrawFlush { target } => LogicInstruction::DrawFlush {
                target: assembler.instruction_var(target),
            },
            LogicStatement::PrintFlush { target } => LogicInstruction::PrintFlush {
                target: assembler.instruction_var(target),
            },
            LogicStatement::GetLink { output, address } => LogicInstruction::GetLink {
                output: assembler.instruction_var(output),
                index: assembler.instruction_var(address),
            },
            LogicStatement::SetRate { amount } => LogicInstruction::SetRate {
                amount: assembler.instruction_var(amount),
            },
            LogicStatement::Sync { variable } => LogicInstruction::Sync {
                variable: assembler.instruction_var(variable),
            },
            LogicStatement::Set { to, from } => LogicInstruction::Set {
                from: assembler.instruction_var(from),
                to: assembler.instruction_var(to),
            },
            LogicStatement::Operation { op, dest, a, b } => LogicInstruction::Op {
                op: *op,
                a: assembler.instruction_var(a),
                b: assembler.instruction_var(b),
                dest: assembler.instruction_var(dest),
            },
            LogicStatement::Select {
                result,
                op,
                comp0,
                comp1,
                a,
                b,
            } => LogicInstruction::Select {
                op: *op,
                result: assembler.instruction_var(result),
                comp0: assembler.instruction_var(comp0),
                comp1: assembler.instruction_var(comp1),
                a: assembler.instruction_var(a),
                b: assembler.instruction_var(b),
            },
            LogicStatement::Wait { value } => LogicInstruction::Wait {
                value: assembler.instruction_var(value),
                cur_time: 0.0,
            },
            LogicStatement::Stop => LogicInstruction::Stop,
            LogicStatement::End => LogicInstruction::End,
            LogicStatement::PackColor { result, r, g, b, a } => LogicInstruction::PackColor {
                result: assembler.instruction_var(result),
                r: assembler.instruction_var(r),
                g: assembler.instruction_var(g),
                b: assembler.instruction_var(b),
                a: assembler.instruction_var(a),
            },
            LogicStatement::UnpackColor { r, g, b, a, value } => LogicInstruction::UnpackColor {
                r: assembler.instruction_var(r),
                g: assembler.instruction_var(g),
                b: assembler.instruction_var(b),
                a: assembler.instruction_var(a),
                value: assembler.instruction_var(value),
            },
            LogicStatement::Lookup { type_, result, id } => LogicInstruction::Lookup {
                dest: assembler.instruction_var(result),
                from: assembler.instruction_var(id),
                type_: *type_,
            },
            LogicStatement::Jump {
                dest_index,
                op,
                value,
                compare,
            } => LogicInstruction::Jump {
                op: *op,
                value: assembler.instruction_var(value),
                compare: assembler.instruction_var(compare),
                address: *dest_index,
            },
            LogicStatement::Control {
                type_,
                target,
                p1,
                p2,
                p3,
                p4,
            } => LogicInstruction::Control {
                type_: *type_,
                target: assembler.instruction_var(target),
                p1: assembler.instruction_var(p1),
                p2: assembler.instruction_var(p2),
                p3: assembler.instruction_var(p3),
                p4: assembler.instruction_var(p4),
            },
            LogicStatement::Radar {
                target1,
                target2,
                target3,
                sort,
                radar,
                sort_order,
                output,
            } => LogicInstruction::Radar {
                target1: *target1,
                target2: *target2,
                target3: *target3,
                sort: *sort,
                radar: assembler.instruction_var(radar),
                sort_order: assembler.instruction_var(sort_order),
                output: assembler.instruction_var(output),
                last_target: None,
            },
            LogicStatement::Sensor { to, from, type_ } => LogicInstruction::Sense {
                from: assembler.instruction_var(from),
                to: assembler.instruction_var(to),
                type_: assembler.instruction_var(type_),
            },
            LogicStatement::UnitBind { type_ } => LogicInstruction::UnitBind {
                type_: assembler.instruction_var(type_),
            },
            LogicStatement::UnitControl {
                type_,
                p1,
                p2,
                p3,
                p4,
                p5,
            } => LogicInstruction::UnitControl {
                type_: *type_,
                p1: assembler.instruction_var(p1),
                p2: assembler.instruction_var(p2),
                p3: assembler.instruction_var(p3),
                p4: assembler.instruction_var(p4),
                p5: assembler.instruction_var(p5),
            },
            LogicStatement::UnitRadar {
                target1,
                target2,
                target3,
                sort,
                sort_order,
                output,
                ..
            } => LogicInstruction::UnitRadar {
                target1: *target1,
                target2: *target2,
                target3: *target3,
                sort: *sort,
                sort_order: assembler.instruction_var(sort_order),
                output: assembler.instruction_var(output),
                last_target: None,
            },
            LogicStatement::UnitLocate {
                locate,
                flag,
                enemy,
                ore,
                out_x,
                out_y,
                out_found,
                out_build,
            } => LogicInstruction::UnitLocate {
                locate: *locate,
                flag: *flag,
                enemy: assembler.instruction_var(enemy),
                ore: assembler.instruction_var(ore),
                out_x: assembler.instruction_var(out_x),
                out_y: assembler.instruction_var(out_y),
                out_found: assembler.instruction_var(out_found),
                out_build: assembler.instruction_var(out_build),
            },
            LogicStatement::Query {
                shape,
                type_,
                team,
                x,
                y,
                w,
                h,
            } => LogicInstruction::Query {
                shape: *shape,
                type_: *type_,
                team: assembler.instruction_var(team),
                x: assembler.instruction_var(x),
                y: assembler.instruction_var(y),
                w: assembler.instruction_var(w),
                h: assembler.instruction_var(h),
            },
            LogicStatement::GetBlock {
                layer,
                result,
                x,
                y,
            } => LogicInstruction::GetBlock {
                layer: *layer,
                result: assembler.instruction_var(result),
                x: assembler.instruction_var(x),
                y: assembler.instruction_var(y),
            },
            LogicStatement::SetBlock {
                layer,
                block,
                x,
                y,
                team,
                rotation,
            } => LogicInstruction::SetBlock {
                layer: *layer,
                block: assembler.instruction_var(block),
                x: assembler.instruction_var(x),
                y: assembler.instruction_var(y),
                team: assembler.instruction_var(team),
                rotation: assembler.instruction_var(rotation),
            },
            LogicStatement::SpawnUnit {
                type_,
                x,
                y,
                rotation,
                team,
                result,
            } => LogicInstruction::SpawnUnit {
                type_: assembler.instruction_var(type_),
                x: assembler.instruction_var(x),
                y: assembler.instruction_var(y),
                rotation: assembler.instruction_var(rotation),
                team: assembler.instruction_var(team),
                result: assembler.instruction_var(result),
            },
            LogicStatement::ApplyStatus {
                clear,
                effect,
                unit,
                duration,
            } => LogicInstruction::ApplyStatus {
                clear: *clear,
                effect: effect.clone(),
                unit: assembler.instruction_var(unit),
                duration: assembler.instruction_var(duration),
            },
            LogicStatement::SpawnWave { x, y, natural } => LogicInstruction::SpawnWave {
                x: assembler.instruction_var(x),
                y: assembler.instruction_var(y),
                natural: assembler.instruction_var(natural),
            },
            LogicStatement::SpawnBullet {
                result,
                from,
                index,
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
            } => LogicInstruction::SpawnBullet {
                result: assembler.instruction_var(result),
                from: assembler.instruction_var(from),
                weapon: assembler.instruction_var(index),
                x: assembler.instruction_var(x),
                y: assembler.instruction_var(y),
                rotation: assembler.instruction_var(rotation),
                team: assembler.instruction_var(team),
                owner: assembler.instruction_var(owner),
                damage: assembler.instruction_var(damage),
                velocity_scl: assembler.instruction_var(velocity_scl),
                life_scl: assembler.instruction_var(life_scl),
                aim_x: assembler.instruction_var(aim_x),
                aim_y: assembler.instruction_var(aim_y),
            },
            LogicStatement::WeatherSense { to, weather } => LogicInstruction::WeatherSense {
                to: assembler.instruction_var(to),
                weather: assembler.instruction_var(weather),
            },
            LogicStatement::WeatherSet { weather, state } => LogicInstruction::WeatherSet {
                weather: assembler.instruction_var(weather),
                state: assembler.instruction_var(state),
            },
            LogicStatement::Effect {
                type_,
                x,
                y,
                sizerot,
                color,
                data,
            } => LogicInstruction::Effect {
                type_name: type_.clone(),
                x: assembler.instruction_var(x),
                y: assembler.instruction_var(y),
                rotation: assembler.instruction_var(sizerot),
                color: assembler.instruction_var(color),
                data: assembler.instruction_var(data),
            },
            LogicStatement::Explosion {
                team,
                x,
                y,
                radius,
                damage,
                air,
                ground,
                pierce,
                effect,
            } => LogicInstruction::Explosion {
                team: assembler.instruction_var(team),
                x: assembler.instruction_var(x),
                y: assembler.instruction_var(y),
                radius: assembler.instruction_var(radius),
                damage: assembler.instruction_var(damage),
                air: assembler.instruction_var(air),
                ground: assembler.instruction_var(ground),
                pierce: assembler.instruction_var(pierce),
                effect: assembler.instruction_var(effect),
            },
            LogicStatement::SetRule {
                rule,
                value,
                p1,
                p2,
                p3,
                p4,
            } => LogicInstruction::SetRule {
                rule: *rule,
                value: assembler.instruction_var(value),
                p1: assembler.instruction_var(p1),
                p2: assembler.instruction_var(p2),
                p3: assembler.instruction_var(p3),
                p4: assembler.instruction_var(p4),
            },
            LogicStatement::Fetch {
                type_,
                result,
                team,
                index,
                extra,
            } => LogicInstruction::Fetch {
                type_: *type_,
                result: assembler.instruction_var(result),
                team: assembler.instruction_var(team),
                index: assembler.instruction_var(index),
                extra: assembler.instruction_var(extra),
            },
            LogicStatement::GetFlag { result, flag } => LogicInstruction::GetFlag {
                result: assembler.instruction_var(result),
                flag: assembler.instruction_var(flag),
            },
            LogicStatement::SetFlag { flag, value } => LogicInstruction::SetFlag {
                flag: assembler.instruction_var(flag),
                value: assembler.instruction_var(value),
            },
            LogicStatement::SetProp { type_, of, value } => LogicInstruction::SetProp {
                type_: assembler.instruction_var(type_),
                of: assembler.instruction_var(of),
                value: assembler.instruction_var(value),
            },
            LogicStatement::FlushMessage {
                type_,
                duration,
                out_success,
            } => LogicInstruction::FlushMessage {
                type_: *type_,
                duration: assembler.instruction_var(duration),
                out_success: assembler.instruction_var(out_success),
            },
            LogicStatement::Cutscene {
                action,
                p1,
                p2,
                p3,
                p4,
            } => LogicInstruction::Cutscene {
                action: *action,
                p1: assembler.instruction_var(p1),
                p2: assembler.instruction_var(p2),
                p3: assembler.instruction_var(p3),
                p4: assembler.instruction_var(p4),
            },
            LogicStatement::ClientData {
                channel,
                value,
                reliable,
            } => LogicInstruction::ClientData {
                channel: assembler.instruction_var(channel),
                value: assembler.instruction_var(value),
                reliable: assembler.instruction_var(reliable),
            },
            LogicStatement::PlaySound {
                positional,
                id,
                volume,
                pitch,
                pan,
                x,
                y,
                limit,
            } => LogicInstruction::PlaySound {
                positional: *positional,
                id: assembler.instruction_var(id),
                volume: assembler.instruction_var(volume),
                pitch: assembler.instruction_var(pitch),
                pan: assembler.instruction_var(pan),
                x: assembler.instruction_var(x),
                y: assembler.instruction_var(y),
                limit: assembler.instruction_var(limit),
            },
            LogicStatement::SetMarker {
                type_,
                id,
                p1,
                p2,
                p3,
            } => LogicInstruction::SetMarker {
                type_: *type_,
                id: assembler.instruction_var(id),
                p1: assembler.instruction_var(p1),
                p2: assembler.instruction_var(p2),
                p3: assembler.instruction_var(p3),
            },
            LogicStatement::MakeMarker {
                type_,
                id,
                x,
                y,
                replace,
            } => LogicInstruction::MakeMarker {
                type_name: type_.clone(),
                id: assembler.instruction_var(id),
                x: assembler.instruction_var(x),
                y: assembler.instruction_var(y),
                replace: assembler.instruction_var(replace),
            },
        }
    }
}

pub fn assemble_logic_source(
    source: &str,
    privileged: bool,
) -> Result<(LogicAssembler, Vec<LogicInstruction>), LogicParseError> {
    let parsed = parse_logic_statements(source)?;
    let mut assembler = LogicAssembler::new();
    assembler.privileged = privileged;
    let mut instructions = Vec::new();

    for statement in parsed.statements {
        let LogicStatementKind::Instruction {
            mut tokens,
            line,
            jump_label,
        } = statement
        else {
            continue;
        };

        if let Some(label) = jump_label {
            let Some(address) = parsed.jump_locations.get(&label) else {
                return Err(LogicParseError::new(format!(
                    "Unknown jump location '{}' on line {}.",
                    label,
                    line + 1
                )));
            };

            if tokens.len() > 1 {
                tokens[1] = address.to_string();
            }
        }

        let Some(statement) = LogicStatement::read_tokens(&tokens) else {
            return Err(LogicParseError::new(format!(
                "Unknown instruction '{}' on line {}.",
                tokens.first().map(String::as_str).unwrap_or(""),
                line + 1
            )));
        };

        instructions.push(statement.to_instruction(&mut assembler));
    }

    Ok((assembler, instructions))
}

fn java_boolean_value_of(value: &str) -> bool {
    value.eq_ignore_ascii_case("true")
}
