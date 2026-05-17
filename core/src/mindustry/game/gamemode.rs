#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gamemode {
    Survival,
    Sandbox,
    Attack,
    Pvp,
    Editor,
}

impl Gamemode {
    pub const ALL: [Gamemode; 5] = [
        Gamemode::Survival,
        Gamemode::Sandbox,
        Gamemode::Attack,
        Gamemode::Pvp,
        Gamemode::Editor,
    ];

    pub const fn hidden(self) -> bool {
        matches!(self, Gamemode::Editor)
    }

    pub const fn wire_name(self) -> &'static str {
        match self {
            Gamemode::Survival => "survival",
            Gamemode::Sandbox => "sandbox",
            Gamemode::Attack => "attack",
            Gamemode::Pvp => "pvp",
            Gamemode::Editor => "editor",
        }
    }
}
