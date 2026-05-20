#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockGroup {
    None,
    Walls,
    Projectors,
    Turrets,
    Transportation,
    Power,
    Liquids,
    Drills,
    Units,
    Logic,
    Payloads,
    Heat,
}

impl BlockGroup {
    pub const ALL: [BlockGroup; 12] = [
        BlockGroup::None,
        BlockGroup::Walls,
        BlockGroup::Projectors,
        BlockGroup::Turrets,
        BlockGroup::Transportation,
        BlockGroup::Power,
        BlockGroup::Liquids,
        BlockGroup::Drills,
        BlockGroup::Units,
        BlockGroup::Logic,
        BlockGroup::Payloads,
        BlockGroup::Heat,
    ];

    pub fn ordinal(self) -> u8 {
        Self::ALL
            .iter()
            .position(|value| *value == self)
            .expect("BlockGroup::ALL must contain every variant") as u8
    }

    pub const fn any_replace(self) -> bool {
        matches!(
            self,
            BlockGroup::Walls
                | BlockGroup::Projectors
                | BlockGroup::Turrets
                | BlockGroup::Transportation
                | BlockGroup::Liquids
                | BlockGroup::Logic
                | BlockGroup::Payloads
                | BlockGroup::Heat
        )
    }
}

#[cfg(test)]
mod tests {
    use super::BlockGroup;

    #[test]
    fn block_group_order_and_any_replace_match_java_enum() {
        assert_eq!(BlockGroup::ALL.len(), 12);
        assert_eq!(BlockGroup::None.ordinal(), 0);
        assert_eq!(BlockGroup::Walls.ordinal(), 1);
        assert_eq!(BlockGroup::Heat.ordinal(), 11);

        let replaceable: Vec<BlockGroup> = BlockGroup::ALL
            .iter()
            .copied()
            .filter(|group| group.any_replace())
            .collect();
        assert_eq!(
            replaceable,
            vec![
                BlockGroup::Walls,
                BlockGroup::Projectors,
                BlockGroup::Turrets,
                BlockGroup::Transportation,
                BlockGroup::Liquids,
                BlockGroup::Logic,
                BlockGroup::Payloads,
                BlockGroup::Heat,
            ]
        );

        assert!(!BlockGroup::Power.any_replace());
        assert!(!BlockGroup::Drills.any_replace());
        assert!(!BlockGroup::Units.any_replace());
    }
}
