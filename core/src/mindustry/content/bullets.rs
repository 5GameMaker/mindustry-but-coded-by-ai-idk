use crate::mindustry::{
    content::blocks::{BulletKind, BulletSpec},
    ctype::{Content, ContentBase, ContentId, ContentType},
};

#[derive(Debug, Clone, PartialEq)]
pub struct BulletContent {
    pub base: ContentBase,
    pub name: String,
    pub spec: BulletSpec,
}

impl BulletContent {
    pub fn new(id: ContentId, name: impl Into<String>, spec: BulletSpec) -> Self {
        Self {
            base: ContentBase::new(id, ContentType::Bullet),
            name: name.into(),
            spec,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Content for BulletContent {
    fn id(&self) -> ContentId {
        self.base.id
    }

    fn content_type(&self) -> ContentType {
        ContentType::Bullet
    }
}

pub fn load() -> Vec<BulletContent> {
    let mut next_id = 0;

    let mut placeholder = basic_bullet(2.5, 9.0);
    placeholder.sprite = "ohno".into();
    placeholder.width = 7.0;
    placeholder.height = 9.0;
    placeholder.lifetime = 60.0;
    placeholder.ammo_multiplier = 2.0;

    let mut dagger_basic = basic_bullet(2.5, 9.0);
    dagger_basic.width = 7.0;
    dagger_basic.height = 9.0;
    dagger_basic.lifetime = 60.0;
    dagger_basic.ammo_multiplier = 2.0;

    let mut mace_flame = BulletSpec::new(BulletKind::Generic, 4.2, 37.0 * 2.0);
    mace_flame.ammo_multiplier = 3.0;
    mace_flame.hit_size = 7.0;
    mace_flame.lifetime = 13.0;
    mace_flame.pierce = true;
    mace_flame.pierce_building = true;
    mace_flame.pierce_cap = 2;
    mace_flame.status_duration = 60.0 * 5.0;
    mace_flame.shoot_effect = "shootSmallFlame".into();
    mace_flame.hit_effect = "hitFlameSmall".into();
    mace_flame.despawn_effect = "none".into();
    mace_flame.status = "burning".into();
    mace_flame.keep_velocity = false;
    mace_flame.hittable = false;

    let mut quasar_beam = laser_bullet(45.0);
    quasar_beam.recoil = 0.0;
    quasar_beam.side_angle = 45.0;
    quasar_beam.side_width = 1.0;
    quasar_beam.side_length = 70.0;
    quasar_beam.heal_percent = 10.0;
    quasar_beam.collides_team = true;
    quasar_beam.length = 150.0;
    quasar_beam.colors = vec!["heal@0.4".into(), "heal".into(), "white".into()];

    let mut damage_lightning = BulletSpec::new(BulletKind::Generic, 0.0001, 0.0);
    damage_lightning.lifetime = 10.0;
    damage_lightning.hit_effect = "hitLancer".into();
    damage_lightning.despawn_effect = "none".into();
    damage_lightning.status = "shocked".into();
    damage_lightning.status_duration = 10.0;
    damage_lightning.hittable = false;
    damage_lightning.light_color = "ffffffff".into();

    let mut damage_lightning_ground = damage_lightning.clone();
    damage_lightning_ground.collides_air = false;

    let mut damage_lightning_air = damage_lightning.clone();
    damage_lightning_air.collides_ground = false;
    damage_lightning_air.collides_tiles = false;

    let mut fireball = BulletSpec::new(BulletKind::Generic, 1.0, 4.0);
    fireball.pierce = true;
    fireball.collides_tiles = false;
    fireball.collides = false;
    fireball.drag = 0.03;
    fireball.hit_effect = "none".into();
    fireball.despawn_effect = "none".into();
    fireball.trail_effect = "fireballsmoke".into();
    fireball.hittable = false;

    let mut space_liquid = BulletSpec::new(BulletKind::Liquid, 3.5, 0.0);
    space_liquid.collides = false;
    space_liquid.lifetime = 90.0;
    space_liquid.despawn_effect = "none".into();
    space_liquid.hit_effect = "none".into();
    space_liquid.smoke_effect = "none".into();
    space_liquid.shoot_effect = "none".into();
    space_liquid.drag = 0.01;
    space_liquid.hittable = false;
    space_liquid.orb_size = 5.5;
    space_liquid.knockback = 0.7;

    vec![
        make_bullet(&mut next_id, "placeholder", placeholder),
        make_bullet(&mut next_id, "dagger_basic", dagger_basic),
        make_bullet(&mut next_id, "mace_flame", mace_flame),
        make_bullet(&mut next_id, "quasar_beam", quasar_beam),
        make_bullet(&mut next_id, "damageLightning", damage_lightning),
        make_bullet(
            &mut next_id,
            "damageLightningGround",
            damage_lightning_ground,
        ),
        make_bullet(&mut next_id, "damageLightningAir", damage_lightning_air),
        make_bullet(&mut next_id, "fireball", fireball),
        make_bullet(&mut next_id, "spaceLiquid", space_liquid),
    ]
}

fn make_bullet(next_id: &mut ContentId, name: &'static str, spec: BulletSpec) -> BulletContent {
    let bullet = BulletContent::new(*next_id, name, spec);
    *next_id += 1;
    bullet
}

fn basic_bullet(speed: f32, damage: f32) -> BulletSpec {
    let mut bullet = BulletSpec::new(BulletKind::Basic, speed, damage);
    bullet.width = 5.0;
    bullet.height = 7.0;
    bullet.shrink_y = 0.5;
    bullet.sprite = "bullet".into();
    bullet.back_color = "bulletYellowBack".into();
    bullet.front_color = "bulletYellow".into();
    bullet.hit_effect = "hitBulletSmall".into();
    bullet.despawn_effect = "hitBulletSmall".into();
    bullet
}

fn laser_bullet(damage: f32) -> BulletSpec {
    let mut bullet = BulletSpec::new(BulletKind::Laser, 0.0, damage);
    bullet.colors = vec![
        "lancerLaser@0.4".into(),
        "lancerLaser".into(),
        "white".into(),
    ];
    bullet.hit_effect = "hitLaserBlast".into();
    bullet.hit_color = "white".into();
    bullet.despawn_effect = "none".into();
    bullet.shoot_effect = "hitLancer".into();
    bullet.smoke_effect = "none".into();
    bullet.hit_size = 4.0;
    bullet.lifetime = 16.0;
    bullet.impact = true;
    bullet.keep_velocity = false;
    bullet.collides = false;
    bullet.pierce = true;
    bullet.hittable = false;
    bullet.absorbable = false;
    bullet.length = 160.0;
    bullet.width = 15.0;
    bullet.side_length = 29.0;
    bullet.side_width = 0.7;
    bullet.side_angle = 90.0;
    bullet
}

#[cfg(test)]
mod tests {
    use super::*;

    fn by_name<'a>(bullets: &'a [BulletContent], name: &str) -> &'a BulletContent {
        bullets
            .iter()
            .find(|bullet| bullet.name() == name)
            .unwrap_or_else(|| panic!("missing bullet {name}"))
    }

    #[test]
    fn vanilla_internal_bullet_order_matches_upstream_load_order() {
        let bullets = load();
        let names: Vec<_> = bullets.iter().map(|bullet| bullet.name()).collect();
        assert_eq!(
            names,
            vec![
                "placeholder",
                "dagger_basic",
                "mace_flame",
                "quasar_beam",
                "damageLightning",
                "damageLightningGround",
                "damageLightningAir",
                "fireball",
                "spaceLiquid",
            ]
        );

        for (index, bullet) in bullets.iter().enumerate() {
            assert_eq!(bullet.id(), index as ContentId);
            assert_eq!(bullet.content_type(), ContentType::Bullet);
        }
    }

    #[test]
    fn placeholder_bullet_keeps_basic_constructor_and_overrides() {
        let bullets = load();
        let placeholder = &by_name(&bullets, "placeholder").spec;

        assert_eq!(placeholder.kind, BulletKind::Basic);
        assert_eq!(placeholder.speed, 2.5);
        assert_eq!(placeholder.damage, 9.0);
        assert_eq!(placeholder.sprite, "ohno");
        assert_eq!(placeholder.width, 7.0);
        assert_eq!(placeholder.height, 9.0);
        assert_eq!(placeholder.lifetime, 60.0);
        assert_eq!(placeholder.ammo_multiplier, 2.0);
        assert_eq!(placeholder.hit_effect, "hitBulletSmall");
        assert_eq!(placeholder.despawn_effect, "hitBulletSmall");
    }

    #[test]
    fn dagger_basic_bullet_matches_java_basic_profile() {
        let bullets = load();
        let bullet = &by_name(&bullets, "dagger_basic").spec;

        assert_eq!(bullet.kind, BulletKind::Basic);
        assert_eq!(bullet.speed, 2.5);
        assert_eq!(bullet.damage, 9.0);
        assert_eq!(bullet.sprite, "bullet");
        assert_eq!(bullet.width, 7.0);
        assert_eq!(bullet.height, 9.0);
        assert_eq!(bullet.lifetime, 60.0);
        assert_eq!(bullet.ammo_multiplier, 2.0);
        assert_eq!(bullet.shrink_y, 0.5);
        assert_eq!(bullet.back_color, "bulletYellowBack");
        assert_eq!(bullet.front_color, "bulletYellow");
        assert_eq!(bullet.hit_effect, "hitBulletSmall");
        assert_eq!(bullet.despawn_effect, "hitBulletSmall");
    }

    #[test]
    fn mace_flame_bullet_matches_java_flamethrower_profile() {
        let bullets = load();
        let bullet = &by_name(&bullets, "mace_flame").spec;

        assert_eq!(bullet.kind, BulletKind::Generic);
        assert_eq!(bullet.speed, 4.2);
        assert_eq!(bullet.damage, 74.0);
        assert_eq!(bullet.ammo_multiplier, 3.0);
        assert_eq!(bullet.hit_size, 7.0);
        assert_eq!(bullet.lifetime, 13.0);
        assert!(bullet.pierce);
        assert!(bullet.pierce_building);
        assert_eq!(bullet.pierce_cap, 2);
        assert_eq!(bullet.status_duration, 300.0);
        assert_eq!(bullet.shoot_effect, "shootSmallFlame");
        assert_eq!(bullet.hit_effect, "hitFlameSmall");
        assert_eq!(bullet.despawn_effect, "none");
        assert_eq!(bullet.status, "burning");
        assert!(!bullet.keep_velocity);
        assert!(!bullet.hittable);
    }

    #[test]
    fn quasar_beam_bullet_matches_java_laser_profile() {
        let bullets = load();
        let bullet = &by_name(&bullets, "quasar_beam").spec;

        assert_eq!(bullet.kind, BulletKind::Laser);
        assert_eq!(bullet.speed, 0.0);
        assert_eq!(bullet.damage, 45.0);
        assert_eq!(bullet.recoil, 0.0);
        assert_eq!(bullet.side_angle, 45.0);
        assert_eq!(bullet.side_width, 1.0);
        assert_eq!(bullet.side_length, 70.0);
        assert_eq!(bullet.heal_percent, 10.0);
        assert!(bullet.collides_team);
        assert_eq!(bullet.length, 150.0);
        assert_eq!(
            bullet.colors,
            vec![
                "heal@0.4".to_string(),
                "heal".to_string(),
                "white".to_string(),
            ]
        );
        assert_eq!(bullet.hit_effect, "hitLaserBlast");
        assert_eq!(bullet.hit_color, "white");
        assert_eq!(bullet.despawn_effect, "none");
        assert_eq!(bullet.shoot_effect, "hitLancer");
        assert_eq!(bullet.smoke_effect, "none");
        assert_eq!(bullet.lifetime, 16.0);
        assert!(bullet.impact);
        assert!(!bullet.keep_velocity);
        assert!(!bullet.collides);
        assert!(bullet.pierce);
        assert!(!bullet.hittable);
        assert!(!bullet.absorbable);
    }

    #[test]
    fn lightning_bullets_keep_copy_overrides() {
        let bullets = load();
        let base = &by_name(&bullets, "damageLightning").spec;

        assert_eq!(base.kind, BulletKind::Generic);
        assert_eq!(base.speed, 0.0001);
        assert_eq!(base.damage, 0.0);
        assert_eq!(base.lifetime, 10.0);
        assert_eq!(base.hit_effect, "hitLancer");
        assert_eq!(base.despawn_effect, "none");
        assert_eq!(base.status, "shocked");
        assert_eq!(base.status_duration, 10.0);
        assert!(!base.hittable);
        assert_eq!(base.light_color, "ffffffff");
        assert!(base.collides_air);
        assert!(base.collides_ground);
        assert!(base.collides_tiles);

        let ground = &by_name(&bullets, "damageLightningGround").spec;
        assert!(!ground.collides_air);
        assert!(ground.collides_ground);
        assert!(ground.collides_tiles);
        assert_eq!(ground.status, base.status);
        assert_eq!(ground.hit_effect, base.hit_effect);

        let air = &by_name(&bullets, "damageLightningAir").spec;
        assert!(air.collides_air);
        assert!(!air.collides_ground);
        assert!(!air.collides_tiles);
        assert_eq!(air.status, base.status);
        assert_eq!(air.hit_effect, base.hit_effect);
    }

    #[test]
    fn fireball_and_space_liquid_match_special_type_defaults() {
        let bullets = load();

        let fireball = &by_name(&bullets, "fireball").spec;
        assert_eq!(fireball.kind, BulletKind::Generic);
        assert_eq!(fireball.speed, 1.0);
        assert_eq!(fireball.damage, 4.0);
        assert!(fireball.pierce);
        assert!(!fireball.collides_tiles);
        assert!(!fireball.collides);
        assert_eq!(fireball.drag, 0.03);
        assert_eq!(fireball.hit_effect, "none");
        assert_eq!(fireball.despawn_effect, "none");
        assert_eq!(fireball.trail_effect, "fireballsmoke");
        assert!(!fireball.hittable);

        let space_liquid = &by_name(&bullets, "spaceLiquid").spec;
        assert_eq!(space_liquid.kind, BulletKind::Liquid);
        assert_eq!(space_liquid.speed, 3.5);
        assert_eq!(space_liquid.damage, 0.0);
        assert!(!space_liquid.collides);
        assert_eq!(space_liquid.lifetime, 90.0);
        assert_eq!(space_liquid.despawn_effect, "none");
        assert_eq!(space_liquid.hit_effect, "none");
        assert_eq!(space_liquid.smoke_effect, "none");
        assert_eq!(space_liquid.shoot_effect, "none");
        assert_eq!(space_liquid.drag, 0.01);
        assert!(!space_liquid.hittable);
        assert_eq!(space_liquid.orb_size, 5.5);
        assert_eq!(space_liquid.knockback, 0.7);
    }
}
