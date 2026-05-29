use crate::mindustry::ctype::{ContentId, ContentType, UnlockableContentBase};

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub base: UnlockableContentBase,
    pub color_rgba: u32,
    pub explosiveness: f32,
    pub flammability: f32,
    pub radioactivity: f32,
    pub charge: f32,
    pub hardness: i32,
    pub cost: f32,
    pub health_scaling: f32,
    pub low_priority: bool,
    pub frames: i32,
    pub transition_frames: i32,
    pub frame_time: f32,
    pub buildable: bool,
    pub hidden: bool,
}

impl Item {
    pub fn new(id: ContentId, name: impl Into<String>) -> Self {
        Self {
            base: UnlockableContentBase::new(id, ContentType::Item, name),
            color_rgba: 0x000000ff,
            explosiveness: 0.0,
            flammability: 0.0,
            radioactivity: 0.0,
            charge: 0.0,
            hardness: 0,
            cost: 1.0,
            health_scaling: 0.0,
            low_priority: false,
            frames: 0,
            transition_frames: 0,
            frame_time: 5.0,
            buildable: true,
            hidden: false,
        }
    }

    pub fn name(&self) -> &str {
        &self.base.mappable.name
    }

    pub fn localized_name(&self) -> &str {
        self.base
            .localized_name
            .as_deref()
            .unwrap_or_else(|| self.name())
    }

    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    pub fn is_on_planet(&self, base_on_planet: bool) -> bool {
        base_on_planet && !self.hidden
    }

    pub fn always_unlocked(&mut self) -> &mut Self {
        self.base.always_unlocked = true;
        self.base.unlocked = true;
        self
    }

    pub fn logic_id(&self) -> i32 {
        self.base.mappable.base.id as i32
    }

    pub fn sense_color(&self) -> f64 {
        crate::mindustry::logic::rgba_u32_to_double_bits(self.color_rgba)
    }

    pub fn sense_id(&self) -> f64 {
        self.logic_id() as f64
    }

    pub fn sense_name(&self) -> &str {
        self.name()
    }

    pub fn animation_region_names(&self) -> Vec<String> {
        if self.frames <= 0 {
            return Vec::new();
        }

        let mut out = Vec::with_capacity((self.frames * (self.transition_frames + 1)) as usize);
        if self.transition_frames <= 0 {
            for frame in 1..=self.frames {
                out.push(format!("{}{frame}", self.name()));
            }
        } else {
            for frame in 0..self.frames {
                out.push(format!("{}{}", self.name(), frame + 1));
                for transition in 1..=self.transition_frames {
                    let index = frame * (self.transition_frames + 1) + transition;
                    out.push(format!("{}-t{index}", self.name()));
                }
            }
        }
        out
    }
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.localized_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mindustry::logic::double_bits_to_rgba;

    #[test]
    fn item_defaults_match_java_constructor_shape() {
        let item = Item::new(7, "copper");
        assert_eq!(item.name(), "copper");
        assert_eq!(item.localized_name(), "copper");
        assert_eq!(item.color_rgba, 0x000000ff);
        assert_eq!(item.explosiveness, 0.0);
        assert_eq!(item.flammability, 0.0);
        assert_eq!(item.radioactivity, 0.0);
        assert_eq!(item.charge, 0.0);
        assert_eq!(item.hardness, 0);
        assert_eq!(item.cost, 1.0);
        assert_eq!(item.health_scaling, 0.0);
        assert!(!item.low_priority);
        assert_eq!(item.frames, 0);
        assert_eq!(item.transition_frames, 0);
        assert_eq!(item.frame_time, 5.0);
        assert!(item.buildable);
        assert!(!item.hidden);
        assert_eq!(item.base.mappable.base.content_type, ContentType::Item);
    }

    #[test]
    fn item_hidden_unlock_display_and_sense_helpers_follow_java_contract() {
        let mut item = Item::new(3, "thorium");
        item.base.localized_name = Some("Thorium".into());
        item.color_rgba = 0xf9a3c7ff;

        assert_eq!(item.to_string(), "Thorium");
        assert_eq!(item.sense_name(), "thorium");
        assert_eq!(item.sense_id(), 3.0);
        assert_eq!(double_bits_to_rgba(item.sense_color()), 0xf9a3c7ff);

        assert!(item.is_on_planet(true));
        item.hidden = true;
        assert!(item.is_hidden());
        assert!(!item.is_on_planet(true));
        assert!(!item.is_on_planet(false));

        assert!(!item.base.unlocked());
        item.always_unlocked();
        assert!(item.base.always_unlocked);
        assert!(item.base.unlocked());
    }

    #[test]
    fn item_animation_region_names_match_java_load_icon_order() {
        let mut item = Item::new(0, "phase-fabric");
        assert!(item.animation_region_names().is_empty());

        item.frames = 3;
        assert_eq!(
            item.animation_region_names(),
            vec!["phase-fabric1", "phase-fabric2", "phase-fabric3"]
        );

        item.transition_frames = 2;
        assert_eq!(
            item.animation_region_names(),
            vec![
                "phase-fabric1",
                "phase-fabric-t1",
                "phase-fabric-t2",
                "phase-fabric2",
                "phase-fabric-t4",
                "phase-fabric-t5",
                "phase-fabric3",
                "phase-fabric-t7",
                "phase-fabric-t8",
            ]
        );
    }
}
