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

    let mut reign_frag = basic_bullet(9.0, 20.0);
    reign_frag.width = 10.0;
    reign_frag.height = 10.0;
    reign_frag.pierce = true;
    reign_frag.pierce_building = true;
    reign_frag.pierce_cap = 3;
    reign_frag.lifetime = 20.0;
    reign_frag.hit_effect = "flakExplosion".into();
    reign_frag.splash_damage = 15.0;
    reign_frag.splash_damage_radius = 10.0;

    let mut reign_shell = basic_bullet(13.0, 80.0);
    reign_shell.pierce = true;
    reign_shell.pierce_cap = 10;
    reign_shell.width = 14.0;
    reign_shell.height = 33.0;
    reign_shell.lifetime = 15.0;
    reign_shell.shoot_effect = "shootBig".into();
    reign_shell.frag_velocity_min = 0.4;
    reign_shell.hit_effect = "blastExplosion".into();
    reign_shell.splash_damage = 18.0;
    reign_shell.splash_damage_radius = 13.0;
    reign_shell.frag_bullets = 3;
    reign_shell.frag_life_min = 0.0;
    reign_shell.frag_random_spread = 30.0;
    reign_shell.despawn_sound = "explosion".into();
    reign_shell.frag_bullet = Some(Box::new(reign_frag));

    let mut scepter_small_bullet = basic_bullet(12.0, 20.0);
    scepter_small_bullet.width = 4.5;
    scepter_small_bullet.height = 35.0;
    scepter_small_bullet.lifetime = (26.0 * 8.0) / 12.0;
    scepter_small_bullet.shrink_x = 0.6;
    scepter_small_bullet.shrink_y = 0.0;
    scepter_small_bullet.shrink_interp = "slope".into();
    scepter_small_bullet.trail_chance = 10.0 / 60.0;
    scepter_small_bullet.trail_color = "bulletYellowBack".into();
    scepter_small_bullet.trail_effect = "bulletSparkSmokeTrailSmall".into();
    scepter_small_bullet.trail_spread = 12.0;
    scepter_small_bullet.shoot_effect = "shootScepterSecondary".into();
    scepter_small_bullet.hit_effect = "hitScepterSecondary".into();

    let mut scepter_interval = lightning_bullet();
    scepter_interval.damage = 5.0;
    scepter_interval.lightning_length = 3;
    scepter_interval.lightning_length_rand = 4;
    scepter_interval.lightning_color = "surge".into();
    scepter_interval.hit_effect = "hitLancerLow".into();

    let mut scepter_bullet = basic_bullet(8.0, 70.0);
    scepter_bullet.width = 11.0;
    scepter_bullet.height = 20.0;
    scepter_bullet.lifetime = 27.0;
    scepter_bullet.shrink_x = 0.4;
    scepter_bullet.shrink_y = 0.0;
    scepter_bullet.shoot_effect = "shootBig".into();
    scepter_bullet.hit_effect = "blastExplosion".into();
    scepter_bullet.trail_param = 0.5;
    scepter_bullet.lightning = 2;
    scepter_bullet.lightning_length = 6;
    scepter_bullet.lightning_color = "surge".into();
    scepter_bullet.lightning_damage = 20.0;
    scepter_bullet.despawn_sound = "shockBullet".into();
    scepter_bullet.bullet_interval = 4.0;
    scepter_bullet.interval_bullet = Some(Box::new(scepter_interval));

    let mut vela_continuous_laser = continuous_laser_bullet(35.0);
    vela_continuous_laser.length = 180.0;
    vela_continuous_laser.hit_effect = "hitMeltHeal".into();
    vela_continuous_laser.draw_size = 420.0;
    vela_continuous_laser.lifetime = 160.0;
    vela_continuous_laser.shake = 1.0;
    vela_continuous_laser.despawn_effect = "smokeCloud".into();
    vela_continuous_laser.smoke_effect = "none".into();
    vela_continuous_laser.charge_effect = "greenLaserChargeSmall".into();
    vela_continuous_laser.incend_chance = 0.1;
    vela_continuous_laser.incend_spread = 5.0;
    vela_continuous_laser.incend_amount = 1;
    vela_continuous_laser.heal_percent = 1.0;
    vela_continuous_laser.collides_team = true;
    vela_continuous_laser.colors = vec![
        "heal@0.2".into(),
        "heal@0.5".into(),
        "heal*1.2".into(),
        "white".into(),
    ];

    let mut vela_repair_range = BulletSpec::new(BulletKind::Generic, 0.0, 0.0);
    vela_repair_range.max_range = 120.0;

    let mut corvus_laser = laser_bullet(560.0);
    corvus_laser.length = 460.0;
    corvus_laser.width = 75.0;
    corvus_laser.lifetime = 65.0;
    corvus_laser.lightning_spacing = 35.0;
    corvus_laser.lightning_length = 5;
    corvus_laser.lightning_delay = 1.1;
    corvus_laser.lightning_length_rand = 15;
    corvus_laser.lightning_damage = 50.0;
    corvus_laser.lightning_angle_rand = 40.0;
    corvus_laser.hit_large = true;
    corvus_laser.light_color = "heal".into();
    corvus_laser.lightning_color = "heal".into();
    corvus_laser.charge_effect = "greenLaserCharge".into();
    corvus_laser.heal_percent = 25.0;
    corvus_laser.collides_team = true;
    corvus_laser.side_angle = 15.0;
    corvus_laser.side_width = 0.0;
    corvus_laser.side_length = 0.0;
    corvus_laser.colors = vec!["heal@0.4".into(), "heal".into(), "white".into()];

    let mut crawler_explosion = BulletSpec::new(BulletKind::Generic, 0.0, 0.0);
    crawler_explosion.collides_tiles = false;
    crawler_explosion.collides = false;
    crawler_explosion.range_override = 25.0;
    crawler_explosion.hit_effect = "pulverize".into();
    crawler_explosion.splash_damage_radius = 44.0;
    crawler_explosion.instant_disappear = true;
    crawler_explosion.splash_damage = 80.0;
    crawler_explosion.building_damage_multiplier = 0.68;
    crawler_explosion.kill_shooter = true;
    crawler_explosion.hittable = false;
    crawler_explosion.collides_air = true;

    let mut atrax_slag = liquid_bullet("slag");
    atrax_slag.damage = 13.0;
    atrax_slag.speed = 2.5;
    atrax_slag.drag = 0.009;
    atrax_slag.shoot_effect = "shootSmall".into();
    atrax_slag.lifetime = 57.0;
    atrax_slag.collides_air = false;

    let mut spiroct_sap = sap_bullet(23.0);
    spiroct_sap.sap_strength = 0.5;
    spiroct_sap.length = 75.0;
    spiroct_sap.shoot_effect = "shootSmall".into();
    spiroct_sap.hit_color = "bf92f9".into();
    spiroct_sap.color = "bf92f9".into();
    spiroct_sap.width = 0.54;
    spiroct_sap.lifetime = 35.0;
    spiroct_sap.knockback = -1.24;

    let mut spiroct_mount_sap = sap_bullet(18.0);
    spiroct_mount_sap.sap_strength = 0.8;
    spiroct_mount_sap.length = 40.0;
    spiroct_mount_sap.shoot_effect = "shootSmall".into();
    spiroct_mount_sap.hit_color = "bf92f9".into();
    spiroct_mount_sap.color = "bf92f9".into();
    spiroct_mount_sap.width = 0.4;
    spiroct_mount_sap.lifetime = 25.0;
    spiroct_mount_sap.knockback = -0.65;

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
        make_bullet(&mut next_id, "reign_shell", reign_shell),
        make_bullet(&mut next_id, "scepter_small_bullet", scepter_small_bullet),
        make_bullet(&mut next_id, "scepter_bullet", scepter_bullet),
        make_bullet(&mut next_id, "vela_continuous_laser", vela_continuous_laser),
        make_bullet(&mut next_id, "vela_repair_range", vela_repair_range),
        make_bullet(&mut next_id, "corvus_laser", corvus_laser),
        make_bullet(&mut next_id, "crawler_explosion", crawler_explosion),
        make_bullet(&mut next_id, "atrax_slag", atrax_slag),
        make_bullet(&mut next_id, "spiroct_sap", spiroct_sap),
        make_bullet(&mut next_id, "spiroct_mount_sap", spiroct_mount_sap),
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

fn liquid_bullet(liquid: &str) -> BulletSpec {
    let mut bullet = BulletSpec::new(BulletKind::Liquid, 3.5, 0.0);
    bullet.liquid = liquid.into();
    bullet.ammo_multiplier = 1.0;
    bullet.lifetime = 34.0;
    bullet.status_duration = 60.0 * 2.0;
    bullet.despawn_effect = "none".into();
    bullet.hit_effect = "hitLiquid".into();
    bullet.smoke_effect = "none".into();
    bullet.shoot_effect = "none".into();
    bullet.drag = 0.001;
    bullet.knockback = 0.55;
    bullet.display_ammo_multiplier = false;
    bullet.puddle_size = 6.0;
    bullet.orb_size = 3.0;
    bullet.boil_time = 5.0;

    if liquid == "slag" {
        bullet.status = "melting".into();
        bullet.hit_color = "ffa166ff".into();
        bullet.light_color = "f0511d66".into();
        bullet.light_opacity = 0.4;
    }

    bullet
}

fn sap_bullet(damage: f32) -> BulletSpec {
    let mut bullet = BulletSpec::new(BulletKind::Sap, 0.0, damage);
    bullet.despawn_effect = "none".into();
    bullet.pierce = true;
    bullet.collides = false;
    bullet.hit_size = 0.0;
    bullet.hittable = false;
    bullet.hit_effect = "hitLiquid".into();
    bullet.status = "sapped".into();
    bullet.light_color = "sap".into();
    bullet.light_opacity = 0.6;
    bullet.status_duration = 60.0 * 3.0;
    bullet.impact = true;
    bullet.sap_strength = 0.5;
    bullet.length = 100.0;
    bullet.width = 0.4;
    bullet.sprite = "laser".into();
    bullet
}

fn continuous_laser_bullet(damage: f32) -> BulletSpec {
    let mut bullet = BulletSpec::new(BulletKind::ContinuousLaser, 0.0, damage);
    bullet.length = 220.0;
    bullet.shake = 1.0;
    bullet.damage_interval = 5.0;
    bullet.hit_large = true;
    bullet.continuous = true;
    bullet.timescale_damage = false;
    bullet.remove_after_pierce = false;
    bullet.pierce_cap = -1;
    bullet.speed = 0.0;
    bullet.despawn_effect = "none".into();
    bullet.shoot_effect = "none".into();
    bullet.lifetime = 16.0;
    bullet.impact = true;
    bullet.keep_velocity = false;
    bullet.collides = false;
    bullet.pierce = true;
    bullet.hittable = false;
    bullet.absorbable = false;
    bullet.hit_effect = "hitBeam".into();
    bullet.hit_size = 4.0;
    bullet.draw_size = 420.0;
    bullet.hit_color = "ff9c5a".into();
    bullet.incend_amount = 1;
    bullet.incend_spread = 5.0;
    bullet.incend_chance = 0.4;
    bullet.light_color = "orange".into();
    bullet.light_opacity = 0.7;
    bullet.width = 9.0;
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
                "reign_shell",
                "scepter_small_bullet",
                "scepter_bullet",
                "vela_continuous_laser",
                "vela_repair_range",
                "corvus_laser",
                "crawler_explosion",
                "atrax_slag",
                "spiroct_sap",
                "spiroct_mount_sap",
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
    fn reign_shell_matches_java_basic_frag_profile() {
        let bullets = load();
        let bullet = &by_name(&bullets, "reign_shell").spec;

        assert_eq!(bullet.kind, BulletKind::Basic);
        assert_eq!(bullet.speed, 13.0);
        assert_eq!(bullet.damage, 80.0);
        assert!(bullet.pierce);
        assert_eq!(bullet.pierce_cap, 10);
        assert_eq!(bullet.width, 14.0);
        assert_eq!(bullet.height, 33.0);
        assert_eq!(bullet.lifetime, 15.0);
        assert_eq!(bullet.shoot_effect, "shootBig");
        assert_eq!(bullet.frag_velocity_min, 0.4);
        assert_eq!(bullet.hit_effect, "blastExplosion");
        assert_eq!(bullet.splash_damage, 18.0);
        assert_eq!(bullet.splash_damage_radius, 13.0);
        assert_eq!(bullet.frag_bullets, 3);
        assert_eq!(bullet.frag_life_min, 0.0);
        assert_eq!(bullet.frag_random_spread, 30.0);
        assert_eq!(bullet.despawn_sound, "explosion");

        let frag = bullet
            .frag_bullet
            .as_deref()
            .expect("reign shell should have fragBullet");
        assert_eq!(frag.kind, BulletKind::Basic);
        assert_eq!(frag.speed, 9.0);
        assert_eq!(frag.damage, 20.0);
        assert_eq!(frag.width, 10.0);
        assert_eq!(frag.height, 10.0);
        assert!(frag.pierce);
        assert!(frag.pierce_building);
        assert_eq!(frag.pierce_cap, 3);
        assert_eq!(frag.lifetime, 20.0);
        assert_eq!(frag.hit_effect, "flakExplosion");
        assert_eq!(frag.splash_damage, 15.0);
        assert_eq!(frag.splash_damage_radius, 10.0);
    }

    #[test]
    fn scepter_small_bullet_matches_java_shared_mount_profile() {
        let bullets = load();
        let bullet = &by_name(&bullets, "scepter_small_bullet").spec;

        assert_eq!(bullet.kind, BulletKind::Basic);
        assert_eq!(bullet.speed, 12.0);
        assert_eq!(bullet.damage, 20.0);
        assert_eq!(bullet.width, 4.5);
        assert_eq!(bullet.height, 35.0);
        assert!((bullet.lifetime - ((26.0 * 8.0) / 12.0)).abs() < 0.0001);
        assert_eq!(bullet.shrink_x, 0.6);
        assert_eq!(bullet.shrink_y, 0.0);
        assert_eq!(bullet.shrink_interp, "slope");
        assert!((bullet.trail_chance - (10.0 / 60.0)).abs() < 0.0001);
        assert_eq!(bullet.trail_color, "bulletYellowBack");
        assert_eq!(bullet.trail_effect, "bulletSparkSmokeTrailSmall");
        assert_eq!(bullet.trail_spread, 12.0);
        assert_eq!(bullet.shoot_effect, "shootScepterSecondary");
        assert_eq!(bullet.hit_effect, "hitScepterSecondary");
    }

    #[test]
    fn scepter_bullet_matches_java_interval_lightning_profile() {
        let bullets = load();
        let bullet = &by_name(&bullets, "scepter_bullet").spec;

        assert_eq!(bullet.kind, BulletKind::Basic);
        assert_eq!(bullet.speed, 8.0);
        assert_eq!(bullet.damage, 70.0);
        assert_eq!(bullet.width, 11.0);
        assert_eq!(bullet.height, 20.0);
        assert_eq!(bullet.lifetime, 27.0);
        assert_eq!(bullet.shrink_x, 0.4);
        assert_eq!(bullet.shrink_y, 0.0);
        assert_eq!(bullet.shoot_effect, "shootBig");
        assert_eq!(bullet.hit_effect, "blastExplosion");
        assert_eq!(bullet.trail_param, 0.5);
        assert_eq!(bullet.lightning, 2);
        assert_eq!(bullet.lightning_length, 6);
        assert_eq!(bullet.lightning_color, "surge");
        assert_eq!(bullet.lightning_damage, 20.0);
        assert_eq!(bullet.despawn_sound, "shockBullet");
        assert_eq!(bullet.bullet_interval, 4.0);

        let interval = bullet
            .interval_bullet
            .as_deref()
            .expect("scepter bullet should have intervalBullet");
        assert_eq!(interval.kind, BulletKind::Lightning);
        assert_eq!(interval.damage, 5.0);
        assert_eq!(interval.lightning_length, 3);
        assert_eq!(interval.lightning_length_rand, 4);
        assert_eq!(interval.lightning_color, "surge");
        assert_eq!(interval.hit_effect, "hitLancerLow");
    }

    #[test]
    fn vela_continuous_laser_matches_java_heal_beam_profile() {
        let bullets = load();
        let bullet = &by_name(&bullets, "vela_continuous_laser").spec;

        assert_eq!(bullet.kind, BulletKind::ContinuousLaser);
        assert_eq!(bullet.speed, 0.0);
        assert_eq!(bullet.damage, 35.0);
        assert_eq!(bullet.length, 180.0);
        assert_eq!(bullet.hit_effect, "hitMeltHeal");
        assert_eq!(bullet.draw_size, 420.0);
        assert_eq!(bullet.lifetime, 160.0);
        assert_eq!(bullet.shake, 1.0);
        assert_eq!(bullet.despawn_effect, "smokeCloud");
        assert_eq!(bullet.smoke_effect, "none");
        assert_eq!(bullet.charge_effect, "greenLaserChargeSmall");
        assert_eq!(bullet.incend_chance, 0.1);
        assert_eq!(bullet.incend_spread, 5.0);
        assert_eq!(bullet.incend_amount, 1);
        assert_eq!(bullet.heal_percent, 1.0);
        assert!(bullet.collides_team);
        assert_eq!(
            bullet.colors,
            vec![
                "heal@0.2".to_string(),
                "heal@0.5".to_string(),
                "heal*1.2".to_string(),
                "white".to_string(),
            ]
        );
        assert!(bullet.continuous);
        assert!(!bullet.keep_velocity);
        assert!(!bullet.collides);
        assert!(bullet.pierce);
    }

    #[test]
    fn vela_repair_range_records_repair_beam_max_range() {
        let bullets = load();
        let bullet = &by_name(&bullets, "vela_repair_range").spec;

        assert_eq!(bullet.kind, BulletKind::Generic);
        assert_eq!(bullet.speed, 0.0);
        assert_eq!(bullet.damage, 0.0);
        assert_eq!(bullet.max_range, 120.0);
    }

    #[test]
    fn corvus_laser_matches_java_laser_profile() {
        let bullets = load();
        let bullet = &by_name(&bullets, "corvus_laser").spec;

        assert_eq!(bullet.kind, BulletKind::Laser);
        assert_eq!(bullet.damage, 560.0);
        assert_eq!(bullet.length, 460.0);
        assert_eq!(bullet.width, 75.0);
        assert_eq!(bullet.lifetime, 65.0);
        assert_eq!(bullet.lightning_spacing, 35.0);
        assert_eq!(bullet.lightning_length, 5);
        assert_eq!(bullet.lightning_delay, 1.1);
        assert_eq!(bullet.lightning_length_rand, 15);
        assert_eq!(bullet.lightning_damage, 50.0);
        assert_eq!(bullet.lightning_angle_rand, 40.0);
        assert!(bullet.hit_large);
        assert_eq!(bullet.light_color, "heal");
        assert_eq!(bullet.lightning_color, "heal");
        assert_eq!(bullet.charge_effect, "greenLaserCharge");
        assert_eq!(bullet.heal_percent, 25.0);
        assert!(bullet.collides_team);
        assert_eq!(bullet.side_angle, 15.0);
        assert_eq!(bullet.side_width, 0.0);
        assert_eq!(bullet.side_length, 0.0);
        assert_eq!(
            bullet.colors,
            vec![
                "heal@0.4".to_string(),
                "heal".to_string(),
                "white".to_string(),
            ]
        );
    }

    #[test]
    fn crawler_explosion_matches_java_death_bullet_profile() {
        let bullets = load();
        let bullet = &by_name(&bullets, "crawler_explosion").spec;

        assert_eq!(bullet.kind, BulletKind::Generic);
        assert_eq!(bullet.speed, 0.0);
        assert_eq!(bullet.damage, 0.0);
        assert!(!bullet.collides_tiles);
        assert!(!bullet.collides);
        assert_eq!(bullet.range_override, 25.0);
        assert_eq!(bullet.hit_effect, "pulverize");
        assert_eq!(bullet.splash_damage_radius, 44.0);
        assert!(bullet.instant_disappear);
        assert_eq!(bullet.splash_damage, 80.0);
        assert_eq!(bullet.building_damage_multiplier, 0.68);
        assert!(bullet.kill_shooter);
        assert!(!bullet.hittable);
        assert!(bullet.collides_air);
    }

    #[test]
    fn atrax_slag_matches_java_liquid_bullet_profile() {
        let bullets = load();
        let bullet = &by_name(&bullets, "atrax_slag").spec;

        assert_eq!(bullet.kind, BulletKind::Liquid);
        assert_eq!(bullet.liquid, "slag");
        assert_eq!(bullet.speed, 2.5);
        assert_eq!(bullet.damage, 13.0);
        assert_eq!(bullet.drag, 0.009);
        assert_eq!(bullet.shoot_effect, "shootSmall");
        assert_eq!(bullet.lifetime, 57.0);
        assert!(!bullet.collides_air);
        assert_eq!(bullet.ammo_multiplier, 1.0);
        assert_eq!(bullet.status_duration, 120.0);
        assert_eq!(bullet.despawn_effect, "none");
        assert_eq!(bullet.hit_effect, "hitLiquid");
        assert_eq!(bullet.smoke_effect, "none");
        assert_eq!(bullet.knockback, 0.55);
        assert!(!bullet.display_ammo_multiplier);
        assert_eq!(bullet.puddle_size, 6.0);
        assert_eq!(bullet.orb_size, 3.0);
        assert_eq!(bullet.boil_time, 5.0);
        assert_eq!(bullet.status, "melting");
        assert_eq!(bullet.hit_color, "ffa166ff");
        assert_eq!(bullet.light_color, "f0511d66");
        assert_eq!(bullet.light_opacity, 0.4);
    }

    #[test]
    fn spiroct_sap_bullets_match_java_profiles() {
        let bullets = load();
        let primary = &by_name(&bullets, "spiroct_sap").spec;
        let mount = &by_name(&bullets, "spiroct_mount_sap").spec;

        for bullet in [primary, mount] {
            assert_eq!(bullet.kind, BulletKind::Sap);
            assert_eq!(bullet.speed, 0.0);
            assert_eq!(bullet.shoot_effect, "shootSmall");
            assert_eq!(bullet.hit_color, "bf92f9");
            assert_eq!(bullet.color, "bf92f9");
            assert_eq!(bullet.despawn_effect, "none");
            assert!(bullet.pierce);
            assert!(!bullet.collides);
            assert_eq!(bullet.hit_size, 0.0);
            assert!(!bullet.hittable);
            assert_eq!(bullet.hit_effect, "hitLiquid");
            assert_eq!(bullet.status, "sapped");
            assert_eq!(bullet.light_color, "sap");
            assert_eq!(bullet.light_opacity, 0.6);
            assert_eq!(bullet.status_duration, 180.0);
            assert!(bullet.impact);
            assert_eq!(bullet.sprite, "laser");
        }

        assert_eq!(primary.sap_strength, 0.5);
        assert_eq!(primary.length, 75.0);
        assert_eq!(primary.damage, 23.0);
        assert_eq!(primary.width, 0.54);
        assert_eq!(primary.lifetime, 35.0);
        assert_eq!(primary.knockback, -1.24);

        assert_eq!(mount.sap_strength, 0.8);
        assert_eq!(mount.length, 40.0);
        assert_eq!(mount.damage, 18.0);
        assert_eq!(mount.width, 0.4);
        assert_eq!(mount.lifetime, 25.0);
        assert_eq!(mount.knockback, -0.65);
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
