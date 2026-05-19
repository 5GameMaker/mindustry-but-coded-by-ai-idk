#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatUnit {
    Blocks,
    BlocksSquared,
    TilesSecond,
    PowerSecond,
    LiquidSecond,
    ItemsSecond,
    LiquidUnits,
    PowerUnits,
    PowerEquilibrium,
    HeatUnits,
    Degrees,
    Seconds,
    Minutes,
    Shots,
    PerSecond,
    PerMinute,
    PerShot,
    PerLeg,
    PerSide,
    TimesSpeed,
    Multiplier,
    Percent,
    ShieldHealth,
    None,
    Items,
}

impl StatUnit {
    pub const ALL: [StatUnit; 25] = [
        StatUnit::Blocks,
        StatUnit::BlocksSquared,
        StatUnit::TilesSecond,
        StatUnit::PowerSecond,
        StatUnit::LiquidSecond,
        StatUnit::ItemsSecond,
        StatUnit::LiquidUnits,
        StatUnit::PowerUnits,
        StatUnit::PowerEquilibrium,
        StatUnit::HeatUnits,
        StatUnit::Degrees,
        StatUnit::Seconds,
        StatUnit::Minutes,
        StatUnit::Shots,
        StatUnit::PerSecond,
        StatUnit::PerMinute,
        StatUnit::PerShot,
        StatUnit::PerLeg,
        StatUnit::PerSide,
        StatUnit::TimesSpeed,
        StatUnit::Multiplier,
        StatUnit::Percent,
        StatUnit::ShieldHealth,
        StatUnit::None,
        StatUnit::Items,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            StatUnit::Blocks => "blocks",
            StatUnit::BlocksSquared => "blocksSquared",
            StatUnit::TilesSecond => "tilesSecond",
            StatUnit::PowerSecond => "powerSecond",
            StatUnit::LiquidSecond => "liquidSecond",
            StatUnit::ItemsSecond => "itemsSecond",
            StatUnit::LiquidUnits => "liquidUnits",
            StatUnit::PowerUnits => "powerUnits",
            StatUnit::PowerEquilibrium => "powerEquilibrium",
            StatUnit::HeatUnits => "heatUnits",
            StatUnit::Degrees => "degrees",
            StatUnit::Seconds => "seconds",
            StatUnit::Minutes => "minutes",
            StatUnit::Shots => "shots",
            StatUnit::PerSecond => "perSecond",
            StatUnit::PerMinute => "perMinute",
            StatUnit::PerShot => "perShot",
            StatUnit::PerLeg => "perLeg",
            StatUnit::PerSide => "perSide",
            StatUnit::TimesSpeed => "timesSpeed",
            StatUnit::Multiplier => "multiplier",
            StatUnit::Percent => "percent",
            StatUnit::ShieldHealth => "shieldHealth",
            StatUnit::None => "none",
            StatUnit::Items => "items",
        }
    }

    pub const fn space(self) -> bool {
        !matches!(
            self,
            StatUnit::PerSecond
                | StatUnit::PerMinute
                | StatUnit::PerShot
                | StatUnit::TimesSpeed
                | StatUnit::Multiplier
                | StatUnit::Percent
        )
    }

    pub const fn icon(self) -> Option<&'static str> {
        match self {
            StatUnit::PowerSecond | StatUnit::PowerUnits => Some("[accent]\u{f8c9}[]"),
            StatUnit::LiquidSecond | StatUnit::LiquidUnits => Some("[sky]\u{e80e}[]"),
            StatUnit::HeatUnits => Some("[red]\u{e810}[]"),
            _ => None,
        }
    }

    pub fn bundle_key(self) -> Option<String> {
        if self == StatUnit::None {
            None
        } else {
            Some(format!("unit.{}", self.name().to_ascii_lowercase()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stat_units_keep_java_names_spacing_and_icons() {
        assert_eq!(StatUnit::ALL.len(), 25);
        assert_eq!(StatUnit::Blocks.name(), "blocks");
        assert_eq!(StatUnit::PowerEquilibrium.name(), "powerEquilibrium");
        assert!(StatUnit::Blocks.space());
        assert!(!StatUnit::PerSecond.space());
        assert!(!StatUnit::Percent.space());
        assert!(StatUnit::PowerSecond.icon().unwrap().contains("[accent]"));
        assert!(StatUnit::LiquidUnits.icon().unwrap().contains("[sky]"));
        assert!(StatUnit::HeatUnits.icon().unwrap().contains("[red]"));
        assert_eq!(StatUnit::Seconds.bundle_key().unwrap(), "unit.seconds");
        assert_eq!(StatUnit::None.bundle_key(), None);
    }
}
