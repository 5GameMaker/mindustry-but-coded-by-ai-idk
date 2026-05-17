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
