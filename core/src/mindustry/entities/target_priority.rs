/// Higher priority blocks are targeted over lower-priority blocks regardless of
/// distance. Mirrors `mindustry.entities.TargetPriority`.
pub const WALL: f32 = -3.0;
pub const UNDER: f32 = -2.0;
pub const TRANSPORT: f32 = -1.0;
pub const BASE: f32 = 0.0;
pub const TURRET: f32 = 1.0;
pub const CORE: f32 = 2.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_priority_constants_match_java_ordering() {
        assert!(WALL < UNDER);
        assert!(UNDER < TRANSPORT);
        assert!(TRANSPORT < BASE);
        assert!(BASE < TURRET);
        assert!(TURRET < CORE);
        assert_eq!(
            [WALL, UNDER, TRANSPORT, BASE, TURRET, CORE],
            [-3.0, -2.0, -1.0, 0.0, 1.0, 2.0]
        );
    }
}
