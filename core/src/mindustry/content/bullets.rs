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

    let mut beta_laser_bolt = laser_bolt_bullet(3.0, 11.0);
    beta_laser_bolt.scale_keep_velocity = true;
    beta_laser_bolt.width = 1.5;
    beta_laser_bolt.height = 4.5;
    beta_laser_bolt.hit_effect = "hitBulletColor".into();
    beta_laser_bolt.despawn_effect = "hitBulletColor".into();
    beta_laser_bolt.trail_width = 1.2;
    beta_laser_bolt.trail_length = 3;
    beta_laser_bolt.shoot_effect = "shootSmallColor".into();
    beta_laser_bolt.smoke_effect = "hitLaserColor".into();
    beta_laser_bolt.back_color = "yellowBoltFront".into();
    beta_laser_bolt.trail_color = "yellowBoltFront".into();
    beta_laser_bolt.hit_color = "yellowBoltFront".into();
    beta_laser_bolt.front_color = "white".into();
    beta_laser_bolt.light_color = "yellowBoltFront".into();
    beta_laser_bolt.lifetime = 60.0;
    beta_laser_bolt.building_damage_multiplier = 0.01;
    beta_laser_bolt.homing_power = 0.03;

    let mut nova_heal_bolt = laser_bolt_bullet(5.2, 13.0);
    nova_heal_bolt.lifetime = 30.0;
    nova_heal_bolt.heal_percent = 5.0;
    nova_heal_bolt.collides_team = true;
    nova_heal_bolt.back_color = "heal".into();
    nova_heal_bolt.front_color = "white".into();

    let mut fortress_artillery = artillery_bullet(2.0, 20.0, "shell");
    fortress_artillery.hit_effect = "blastExplosion".into();
    fortress_artillery.knockback = 0.8;
    fortress_artillery.lifetime = 120.0 - (35.0 - 8.0) / 2.0;
    fortress_artillery.max_range = 240.0;
    fortress_artillery.width = 14.0;
    fortress_artillery.height = 14.0;
    fortress_artillery.collides = true;
    fortress_artillery.collides_tiles = true;
    fortress_artillery.splash_damage_radius = 35.0;
    fortress_artillery.splash_damage = 80.0;
    fortress_artillery.back_color = "bulletYellowBack".into();
    fortress_artillery.front_color = "bulletYellow".into();

    let mut pulsar_heal_lightning_type = BulletSpec::new(BulletKind::Generic, 0.0001, 0.0);
    pulsar_heal_lightning_type.lifetime = 10.0;
    pulsar_heal_lightning_type.hit_effect = "hitLancer".into();
    pulsar_heal_lightning_type.despawn_effect = "none".into();
    pulsar_heal_lightning_type.status = "shocked".into();
    pulsar_heal_lightning_type.status_duration = 10.0;
    pulsar_heal_lightning_type.hittable = false;
    pulsar_heal_lightning_type.heal_percent = 1.6;
    pulsar_heal_lightning_type.collides_team = true;

    let mut pulsar_heal_lightning = lightning_bullet();
    pulsar_heal_lightning.lightning_color = "heal".into();
    pulsar_heal_lightning.hit_color = "heal".into();
    pulsar_heal_lightning.damage = 15.0;
    pulsar_heal_lightning.lightning_length = 8;
    pulsar_heal_lightning.lightning_length_rand = 7;
    pulsar_heal_lightning.shoot_effect = "shootHeal".into();
    pulsar_heal_lightning.heal_percent = 2.0;
    pulsar_heal_lightning.lightning_type = Some(Box::new(pulsar_heal_lightning_type));

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
        make_bullet(&mut next_id, "beta_laser_bolt", beta_laser_bolt),
        make_bullet(&mut next_id, "nova_heal_bolt", nova_heal_bolt),
        make_bullet(&mut next_id, "fortress_artillery", fortress_artillery),
        make_bullet(&mut next_id, "pulsar_heal_lightning", pulsar_heal_lightning),
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

fn laser_bolt_bullet(speed: f32, damage: f32) -> BulletSpec {
    let mut bullet = BulletSpec::new(BulletKind::LaserBolt, speed, damage);
    bullet.width = 2.0;
    bullet.height = 7.0;
    bullet.shrink_y = 0.5;
    bullet.sprite = "bullet".into();
    bullet.back_color = "bulletYellowBack".into();
    bullet.front_color = "bulletYellow".into();
    bullet.smoke_effect = "hitLaser".into();
    bullet.hit_effect = "hitLaser".into();
    bullet.despawn_effect = "hitLaser".into();
    bullet.hittable = false;
    bullet.reflectable = false;
    bullet.light_color = "heal".into();
    bullet.light_opacity = 0.6;
    bullet
}

fn artillery_bullet(speed: f32, damage: f32, sprite: &str) -> BulletSpec {
    let mut bullet = BulletSpec::new(BulletKind::Artillery, speed, damage);
    bullet.sprite = sprite.into();
    bullet.collides_tiles = false;
    bullet.collides = false;
    bullet.collides_air = false;
    bullet.scale_life = true;
    bullet.hit_shake = 1.0;
    bullet.hit_sound = "explosionArtillery".into();
    bullet.hit_effect = "flakExplosion".into();
    bullet.shoot_effect = "shootBig".into();
    bullet.trail_effect = "artilleryTrail".into();
    bullet.shrink_x = 0.15;
    bullet.shrink_y = 0.5;
    bullet.back_color = "bulletYellowBack".into();
    bullet.front_color = "bulletYellow".into();
    bullet
}

fn lightning_bullet() -> BulletSpec {
    let mut bullet = BulletSpec::new(BulletKind::Lightning, 0.0, 1.0);
    bullet.lifetime = 1.0;
    bullet.despawn_effect = "none".into();
    bullet.hit_effect = "hitLancer".into();
    bullet.keep_velocity = false;
    bullet.hittable = false;
    bullet.status = "shocked".into();
    bullet.lightning_length = 25;
    bullet.lightning_length_rand = 0;
    bullet.lightning_color = "lancerLaser".into();
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
                "beta_laser_bolt",
                "nova_heal_bolt",
                "fortress_artillery",
                "pulsar_heal_lightning",
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
    fn beta_laser_bolt_matches_java_laser_bolt_profile() {
        let bullets = load();
        let bullet = &by_name(&bullets, "beta_laser_bolt").spec;

        assert_eq!(bullet.kind, BulletKind::LaserBolt);
        assert_eq!(bullet.speed, 3.0);
        assert_eq!(bullet.damage, 11.0);
        assert!(bullet.scale_keep_velocity);
        assert_eq!(bullet.width, 1.5);
        assert_eq!(bullet.height, 4.5);
        assert_eq!(bullet.hit_effect, "hitBulletColor");
        assert_eq!(bullet.despawn_effect, "hitBulletColor");
        assert_eq!(bullet.trail_width, 1.2);
        assert_eq!(bullet.trail_length, 3);
        assert_eq!(bullet.shoot_effect, "shootSmallColor");
        assert_eq!(bullet.smoke_effect, "hitLaserColor");
        assert_eq!(bullet.back_color, "yellowBoltFront");
        assert_eq!(bullet.trail_color, "yellowBoltFront");
        assert_eq!(bullet.hit_color, "yellowBoltFront");
        assert_eq!(bullet.front_color, "white");
        assert_eq!(bullet.light_color, "yellowBoltFront");
        assert_eq!(bullet.lifetime, 60.0);
        assert_eq!(bullet.building_damage_multiplier, 0.01);
        assert_eq!(bullet.homing_power, 0.03);
        assert!(!bullet.hittable);
        assert!(!bullet.reflectable);
        assert_eq!(bullet.light_opacity, 0.6);
    }

    #[test]
    fn nova_heal_bolt_matches_java_laser_bolt_profile() {
        let bullets = load();
        let bullet = &by_name(&bullets, "nova_heal_bolt").spec;

        assert_eq!(bullet.kind, BulletKind::LaserBolt);
        assert_eq!(bullet.speed, 5.2);
        assert_eq!(bullet.damage, 13.0);
        assert_eq!(bullet.lifetime, 30.0);
        assert_eq!(bullet.heal_percent, 5.0);
        assert!(bullet.collides_team);
        assert_eq!(bullet.back_color, "heal");
        assert_eq!(bullet.front_color, "white");
        assert_eq!(bullet.light_color, "heal");
        assert!(!bullet.hittable);
        assert!(!bullet.reflectable);
        assert_eq!(bullet.light_opacity, 0.6);
    }

    #[test]
    fn fortress_artillery_matches_java_artillery_profile() {
        let bullets = load();
        let bullet = &by_name(&bullets, "fortress_artillery").spec;

        assert_eq!(bullet.kind, BulletKind::Artillery);
        assert_eq!(bullet.speed, 2.0);
        assert_eq!(bullet.damage, 20.0);
        assert_eq!(bullet.sprite, "shell");
        assert_eq!(bullet.hit_effect, "blastExplosion");
        assert_eq!(bullet.knockback, 0.8);
        assert_eq!(bullet.lifetime, 106.5);
        assert_eq!(bullet.max_range, 240.0);
        assert_eq!(bullet.width, 14.0);
        assert_eq!(bullet.height, 14.0);
        assert!(bullet.collides);
        assert!(bullet.collides_tiles);
        assert_eq!(bullet.splash_damage_radius, 35.0);
        assert_eq!(bullet.splash_damage, 80.0);
        assert_eq!(bullet.back_color, "bulletYellowBack");
        assert_eq!(bullet.front_color, "bulletYellow");
        assert!(!bullet.collides_air);
        assert!(bullet.scale_life);
        assert_eq!(bullet.hit_shake, 1.0);
        assert_eq!(bullet.hit_sound, "explosionArtillery");
        assert_eq!(bullet.shoot_effect, "shootBig");
        assert_eq!(bullet.trail_effect, "artilleryTrail");
        assert_eq!(bullet.shrink_x, 0.15);
        assert_eq!(bullet.shrink_y, 0.5);
    }

    #[test]
    fn pulsar_heal_lightning_matches_java_lightning_profile() {
        let bullets = load();
        let bullet = &by_name(&bullets, "pulsar_heal_lightning").spec;

        assert_eq!(bullet.kind, BulletKind::Lightning);
        assert_eq!(bullet.speed, 0.0);
        assert_eq!(bullet.damage, 15.0);
        assert_eq!(bullet.lifetime, 1.0);
        assert_eq!(bullet.lightning_color, "heal");
        assert_eq!(bullet.hit_color, "heal");
        assert_eq!(bullet.lightning_length, 8);
        assert_eq!(bullet.lightning_length_rand, 7);
        assert_eq!(bullet.shoot_effect, "shootHeal");
        assert_eq!(bullet.heal_percent, 2.0);
        assert_eq!(bullet.despawn_effect, "none");
        assert_eq!(bullet.hit_effect, "hitLancer");
        assert!(!bullet.keep_velocity);
        assert!(!bullet.hittable);
        assert_eq!(bullet.status, "shocked");

        let nested = bullet
            .lightning_type
            .as_deref()
            .expect("pulsar lightning should have nested lightningType");
        assert_eq!(nested.kind, BulletKind::Generic);
        assert_eq!(nested.speed, 0.0001);
        assert_eq!(nested.damage, 0.0);
        assert_eq!(nested.lifetime, 10.0);
        assert_eq!(nested.hit_effect, "hitLancer");
        assert_eq!(nested.despawn_effect, "none");
        assert_eq!(nested.status, "shocked");
        assert_eq!(nested.status_duration, 10.0);
        assert!(!nested.hittable);
        assert_eq!(nested.heal_percent, 1.6);
        assert!(nested.collides_team);
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
