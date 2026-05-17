#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trigger {
    NewGame,
    SaveLoad,
    ClientCreate,
    WorldLoad,
    WorldDrawBegin,
    WorldDrawEnd,
    PostDraw,
    UiDrawBegin,
    UiDrawEnd,
    Update,
}
