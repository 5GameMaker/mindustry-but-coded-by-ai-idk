// Mirrors upstream core/src/mindustry/logic. Implemented incrementally from D:\MDT\mindustry-upstream-v157.4.

/// Mirrors upstream `mindustry.logic.LAccess`.
///
/// The declaration order is observable by logic scripts and generated
/// processors, so keep it identical to Java `values()`.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LAccess {
    TotalItems,
    FirstItem,
    TotalLiquids,
    TotalPower,
    ItemCapacity,
    LiquidCapacity,
    PowerCapacity,
    PowerNetStored,
    PowerNetCapacity,
    PowerNetIn,
    PowerNetOut,
    Ammo,
    AmmoCapacity,
    CurrentAmmoType,
    MemoryCapacity,
    Health,
    MaxHealth,
    Heat,
    Shield,
    Armor,
    Efficiency,
    Progress,
    Timescale,
    Rotation,
    X,
    Y,
    VelocityX,
    VelocityY,
    ShootX,
    ShootY,
    CameraX,
    CameraY,
    CameraWidth,
    CameraHeight,
    DisplayWidth,
    DisplayHeight,
    BufferSize,
    Operations,
    Size,
    Solid,
    Dead,
    Range,
    Shooting,
    Boosting,
    MineX,
    MineY,
    Mining,
    BuildX,
    BuildY,
    PingX,
    PingY,
    PingText,
    Building,
    Breaking,
    Speed,
    Team,
    Type,
    Flag,
    Controlled,
    Controller,
    Name,
    PayloadCount,
    PayloadType,
    TotalPayload,
    PayloadCapacity,
    MaxUnits,
    Id,
    SelectedBlock,
    SelectedRotation,
    BulletLifetime,
    BulletTime,
    Enabled,
    Shoot,
    Shootp,
    Config,
    Color,
}

impl LAccess {
    pub const ALL: [LAccess; 76] = [
        LAccess::TotalItems,
        LAccess::FirstItem,
        LAccess::TotalLiquids,
        LAccess::TotalPower,
        LAccess::ItemCapacity,
        LAccess::LiquidCapacity,
        LAccess::PowerCapacity,
        LAccess::PowerNetStored,
        LAccess::PowerNetCapacity,
        LAccess::PowerNetIn,
        LAccess::PowerNetOut,
        LAccess::Ammo,
        LAccess::AmmoCapacity,
        LAccess::CurrentAmmoType,
        LAccess::MemoryCapacity,
        LAccess::Health,
        LAccess::MaxHealth,
        LAccess::Heat,
        LAccess::Shield,
        LAccess::Armor,
        LAccess::Efficiency,
        LAccess::Progress,
        LAccess::Timescale,
        LAccess::Rotation,
        LAccess::X,
        LAccess::Y,
        LAccess::VelocityX,
        LAccess::VelocityY,
        LAccess::ShootX,
        LAccess::ShootY,
        LAccess::CameraX,
        LAccess::CameraY,
        LAccess::CameraWidth,
        LAccess::CameraHeight,
        LAccess::DisplayWidth,
        LAccess::DisplayHeight,
        LAccess::BufferSize,
        LAccess::Operations,
        LAccess::Size,
        LAccess::Solid,
        LAccess::Dead,
        LAccess::Range,
        LAccess::Shooting,
        LAccess::Boosting,
        LAccess::MineX,
        LAccess::MineY,
        LAccess::Mining,
        LAccess::BuildX,
        LAccess::BuildY,
        LAccess::PingX,
        LAccess::PingY,
        LAccess::PingText,
        LAccess::Building,
        LAccess::Breaking,
        LAccess::Speed,
        LAccess::Team,
        LAccess::Type,
        LAccess::Flag,
        LAccess::Controlled,
        LAccess::Controller,
        LAccess::Name,
        LAccess::PayloadCount,
        LAccess::PayloadType,
        LAccess::TotalPayload,
        LAccess::PayloadCapacity,
        LAccess::MaxUnits,
        LAccess::Id,
        LAccess::SelectedBlock,
        LAccess::SelectedRotation,
        LAccess::BulletLifetime,
        LAccess::BulletTime,
        LAccess::Enabled,
        LAccess::Shoot,
        LAccess::Shootp,
        LAccess::Config,
        LAccess::Color,
    ];

    pub const SENSEABLE: [LAccess; 74] = [
        LAccess::TotalItems,
        LAccess::FirstItem,
        LAccess::TotalLiquids,
        LAccess::TotalPower,
        LAccess::ItemCapacity,
        LAccess::LiquidCapacity,
        LAccess::PowerCapacity,
        LAccess::PowerNetStored,
        LAccess::PowerNetCapacity,
        LAccess::PowerNetIn,
        LAccess::PowerNetOut,
        LAccess::Ammo,
        LAccess::AmmoCapacity,
        LAccess::CurrentAmmoType,
        LAccess::MemoryCapacity,
        LAccess::Health,
        LAccess::MaxHealth,
        LAccess::Heat,
        LAccess::Shield,
        LAccess::Armor,
        LAccess::Efficiency,
        LAccess::Progress,
        LAccess::Timescale,
        LAccess::Rotation,
        LAccess::X,
        LAccess::Y,
        LAccess::VelocityX,
        LAccess::VelocityY,
        LAccess::ShootX,
        LAccess::ShootY,
        LAccess::CameraX,
        LAccess::CameraY,
        LAccess::CameraWidth,
        LAccess::CameraHeight,
        LAccess::DisplayWidth,
        LAccess::DisplayHeight,
        LAccess::BufferSize,
        LAccess::Operations,
        LAccess::Size,
        LAccess::Solid,
        LAccess::Dead,
        LAccess::Range,
        LAccess::Shooting,
        LAccess::Boosting,
        LAccess::MineX,
        LAccess::MineY,
        LAccess::Mining,
        LAccess::BuildX,
        LAccess::BuildY,
        LAccess::PingX,
        LAccess::PingY,
        LAccess::PingText,
        LAccess::Building,
        LAccess::Breaking,
        LAccess::Speed,
        LAccess::Team,
        LAccess::Type,
        LAccess::Flag,
        LAccess::Controlled,
        LAccess::Controller,
        LAccess::Name,
        LAccess::PayloadCount,
        LAccess::PayloadType,
        LAccess::TotalPayload,
        LAccess::PayloadCapacity,
        LAccess::MaxUnits,
        LAccess::Id,
        LAccess::SelectedBlock,
        LAccess::SelectedRotation,
        LAccess::BulletLifetime,
        LAccess::BulletTime,
        LAccess::Enabled,
        LAccess::Config,
        LAccess::Color,
    ];

    pub const CONTROLS: [LAccess; 5] = [
        LAccess::Enabled,
        LAccess::Shoot,
        LAccess::Shootp,
        LAccess::Config,
        LAccess::Color,
    ];

    pub const SETTABLE: [LAccess; 15] = [
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
        LAccess::BulletLifetime,
    ];

    pub const WIRE_NAMES: [&'static str; 76] = [
        "totalItems",
        "firstItem",
        "totalLiquids",
        "totalPower",
        "itemCapacity",
        "liquidCapacity",
        "powerCapacity",
        "powerNetStored",
        "powerNetCapacity",
        "powerNetIn",
        "powerNetOut",
        "ammo",
        "ammoCapacity",
        "currentAmmoType",
        "memoryCapacity",
        "health",
        "maxHealth",
        "heat",
        "shield",
        "armor",
        "efficiency",
        "progress",
        "timescale",
        "rotation",
        "x",
        "y",
        "velocityX",
        "velocityY",
        "shootX",
        "shootY",
        "cameraX",
        "cameraY",
        "cameraWidth",
        "cameraHeight",
        "displayWidth",
        "displayHeight",
        "bufferSize",
        "operations",
        "size",
        "solid",
        "dead",
        "range",
        "shooting",
        "boosting",
        "mineX",
        "mineY",
        "mining",
        "buildX",
        "buildY",
        "pingX",
        "pingY",
        "pingText",
        "building",
        "breaking",
        "speed",
        "team",
        "type",
        "flag",
        "controlled",
        "controller",
        "name",
        "payloadCount",
        "payloadType",
        "totalPayload",
        "payloadCapacity",
        "maxUnits",
        "id",
        "selectedBlock",
        "selectedRotation",
        "bulletLifetime",
        "bulletTime",
        "enabled",
        "shoot",
        "shootp",
        "config",
        "color",
    ];

    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub fn wire_name(self) -> &'static str {
        Self::WIRE_NAMES[self.ordinal() as usize]
    }

    pub const fn params(self) -> &'static [&'static str] {
        match self {
            LAccess::Enabled => &["to"],
            LAccess::Shoot => &["x", "y", "shoot"],
            LAccess::Shootp => &["unit", "shoot"],
            LAccess::Config => &["to"],
            LAccess::Color => &["to"],
            _ => &[],
        }
    }

    pub const fn is_obj(self) -> bool {
        matches!(self, LAccess::Shootp | LAccess::Config)
    }

    pub const fn is_senseable(self) -> bool {
        self.params().len() <= 1
    }

    pub const fn is_control(self) -> bool {
        !self.params().is_empty()
    }
}

/// Mirrors upstream `mindustry.logic.LMarkerControl`.
///
/// The declaration order is network-visible: Java `TypeIO.writeMarkerControl`
/// writes the enum ordinal as a single byte and `readMarkerControl` indexes
/// `LMarkerControl.all` with an unsigned byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LMarkerControl {
    Remove,
    World,
    Minimap,
    Autoscale,
    Pos,
    EndPos,
    DrawLayer,
    Color,
    Radius,
    Stroke,
    Outline,
    Rotation,
    Shape,
    Arc,
    FlushText,
    FontSize,
    TextHeight,
    TextAlign,
    LineAlign,
    LabelFlags,
    Texture,
    TextureSize,
    Posi,
    Uvi,
    Colori,
}

impl LMarkerControl {
    pub const ALL: [LMarkerControl; 25] = [
        LMarkerControl::Remove,
        LMarkerControl::World,
        LMarkerControl::Minimap,
        LMarkerControl::Autoscale,
        LMarkerControl::Pos,
        LMarkerControl::EndPos,
        LMarkerControl::DrawLayer,
        LMarkerControl::Color,
        LMarkerControl::Radius,
        LMarkerControl::Stroke,
        LMarkerControl::Outline,
        LMarkerControl::Rotation,
        LMarkerControl::Shape,
        LMarkerControl::Arc,
        LMarkerControl::FlushText,
        LMarkerControl::FontSize,
        LMarkerControl::TextHeight,
        LMarkerControl::TextAlign,
        LMarkerControl::LineAlign,
        LMarkerControl::LabelFlags,
        LMarkerControl::Texture,
        LMarkerControl::TextureSize,
        LMarkerControl::Posi,
        LMarkerControl::Uvi,
        LMarkerControl::Colori,
    ];

    pub const fn ordinal(self) -> u8 {
        match self {
            LMarkerControl::Remove => 0,
            LMarkerControl::World => 1,
            LMarkerControl::Minimap => 2,
            LMarkerControl::Autoscale => 3,
            LMarkerControl::Pos => 4,
            LMarkerControl::EndPos => 5,
            LMarkerControl::DrawLayer => 6,
            LMarkerControl::Color => 7,
            LMarkerControl::Radius => 8,
            LMarkerControl::Stroke => 9,
            LMarkerControl::Outline => 10,
            LMarkerControl::Rotation => 11,
            LMarkerControl::Shape => 12,
            LMarkerControl::Arc => 13,
            LMarkerControl::FlushText => 14,
            LMarkerControl::FontSize => 15,
            LMarkerControl::TextHeight => 16,
            LMarkerControl::TextAlign => 17,
            LMarkerControl::LineAlign => 18,
            LMarkerControl::LabelFlags => 19,
            LMarkerControl::Texture => 20,
            LMarkerControl::TextureSize => 21,
            LMarkerControl::Posi => 22,
            LMarkerControl::Uvi => 23,
            LMarkerControl::Colori => 24,
        }
    }

    pub fn from_ordinal(ordinal: u8) -> Option<Self> {
        Self::ALL.get(ordinal as usize).copied()
    }

    pub const fn wire_name(self) -> &'static str {
        match self {
            LMarkerControl::Remove => "remove",
            LMarkerControl::World => "world",
            LMarkerControl::Minimap => "minimap",
            LMarkerControl::Autoscale => "autoscale",
            LMarkerControl::Pos => "pos",
            LMarkerControl::EndPos => "endPos",
            LMarkerControl::DrawLayer => "drawLayer",
            LMarkerControl::Color => "color",
            LMarkerControl::Radius => "radius",
            LMarkerControl::Stroke => "stroke",
            LMarkerControl::Outline => "outline",
            LMarkerControl::Rotation => "rotation",
            LMarkerControl::Shape => "shape",
            LMarkerControl::Arc => "arc",
            LMarkerControl::FlushText => "flushText",
            LMarkerControl::FontSize => "fontSize",
            LMarkerControl::TextHeight => "textHeight",
            LMarkerControl::TextAlign => "textAlign",
            LMarkerControl::LineAlign => "lineAlign",
            LMarkerControl::LabelFlags => "labelFlags",
            LMarkerControl::Texture => "texture",
            LMarkerControl::TextureSize => "textureSize",
            LMarkerControl::Posi => "posi",
            LMarkerControl::Uvi => "uvi",
            LMarkerControl::Colori => "colori",
        }
    }

    pub const fn params(self) -> &'static [&'static str] {
        match self {
            LMarkerControl::Remove => &[],
            LMarkerControl::World => &["true/false"],
            LMarkerControl::Minimap => &["true/false"],
            LMarkerControl::Autoscale => &["true/false"],
            LMarkerControl::Pos => &["x", "y"],
            LMarkerControl::EndPos => &["x", "y"],
            LMarkerControl::DrawLayer => &["layer"],
            LMarkerControl::Color => &["color"],
            LMarkerControl::Radius => &["radius"],
            LMarkerControl::Stroke => &["stroke"],
            LMarkerControl::Outline => &["outline"],
            LMarkerControl::Rotation => &["rotation"],
            LMarkerControl::Shape => &["sides", "fill", "outline"],
            LMarkerControl::Arc => &["start", "end"],
            LMarkerControl::FlushText => &["fetch"],
            LMarkerControl::FontSize => &["size"],
            LMarkerControl::TextHeight => &["height"],
            LMarkerControl::TextAlign => &["align"],
            LMarkerControl::LineAlign => &["align"],
            LMarkerControl::LabelFlags => &["background", "outline"],
            LMarkerControl::Texture => &["printFlush", "name"],
            LMarkerControl::TextureSize => &["width", "height"],
            LMarkerControl::Posi => &["index", "x", "y"],
            LMarkerControl::Uvi => &["index", "x", "y"],
            LMarkerControl::Colori => &["index", "color"],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
