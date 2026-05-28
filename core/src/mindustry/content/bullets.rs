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

    let mut arkyid_sapper = sap_bullet(40.0);
    arkyid_sapper.sap_strength = 0.85;
    arkyid_sapper.length = 55.0;
    arkyid_sapper.shoot_effect = "shootSmall".into();
    arkyid_sapper.hit_color = "bf92f9".into();
    arkyid_sapper.color = "bf92f9".into();
    arkyid_sapper.width = 0.55;
    arkyid_sapper.lifetime = 30.0;
    arkyid_sapper.knockback = -1.0;

    let mut arkyid_artillery_sap = artillery_bullet(2.0, 12.0, "shell");
    arkyid_artillery_sap.hit_effect = "sapExplosion".into();
    arkyid_artillery_sap.despawn_sound = "explosionArtilleryShock".into();
    arkyid_artillery_sap.knockback = 0.8;
    arkyid_artillery_sap.lifetime = 70.0;
    arkyid_artillery_sap.width = 19.0;
    arkyid_artillery_sap.height = 19.0;
    arkyid_artillery_sap.collides_tiles = true;
    arkyid_artillery_sap.ammo_multiplier = 4.0;
    arkyid_artillery_sap.splash_damage_radius = 70.0;
    arkyid_artillery_sap.splash_damage = 65.0;
    arkyid_artillery_sap.back_color = "sapBulletBack".into();
    arkyid_artillery_sap.front_color = "sapBullet".into();
    arkyid_artillery_sap.lightning_color = "sapBullet".into();
    arkyid_artillery_sap.lightning = 3;
    arkyid_artillery_sap.lightning_length = 10;
    arkyid_artillery_sap.smoke_effect = "shootBigSmoke2".into();
    arkyid_artillery_sap.shake = 5.0;
    arkyid_artillery_sap.status = "sapped".into();
    arkyid_artillery_sap.status_duration = 60.0 * 10.0;

    let mut toxopid_shrapnel = shrapnel_bullet(110.0);
    toxopid_shrapnel.length = 90.0;
    toxopid_shrapnel.width = 25.0;
    toxopid_shrapnel.serration_len_scl = 7.0;
    toxopid_shrapnel.serration_space_offset = 60.0;
    toxopid_shrapnel.serration_fade_offset = 0.0;
    toxopid_shrapnel.serrations = 10;
    toxopid_shrapnel.serration_width = 6.0;
    toxopid_shrapnel.from_color = "sapBullet".into();
    toxopid_shrapnel.to_color = "sapBulletBack".into();
    toxopid_shrapnel.shoot_effect = "sparkShoot".into();
    toxopid_shrapnel.smoke_effect = "sparkShoot".into();

    let mut toxopid_frag_sap = artillery_bullet(2.3, 30.0, "shell");
    toxopid_frag_sap.despawn_sound = "explosionArtilleryShock".into();
    toxopid_frag_sap.hit_effect = "sapExplosion".into();
    toxopid_frag_sap.knockback = 0.8;
    toxopid_frag_sap.lifetime = 90.0;
    toxopid_frag_sap.width = 20.0;
    toxopid_frag_sap.height = 20.0;
    toxopid_frag_sap.collides_tiles = false;
    toxopid_frag_sap.splash_damage_radius = 70.0;
    toxopid_frag_sap.splash_damage = 40.0;
    toxopid_frag_sap.back_color = "sapBulletBack".into();
    toxopid_frag_sap.front_color = "sapBullet".into();
    toxopid_frag_sap.lightning_color = "sapBullet".into();
    toxopid_frag_sap.lightning = 2;
    toxopid_frag_sap.lightning_length = 5;
    toxopid_frag_sap.smoke_effect = "shootBigSmoke2".into();
    toxopid_frag_sap.hit_shake = 5.0;
    toxopid_frag_sap.light_radius = 30.0;
    toxopid_frag_sap.light_color = "sap".into();
    toxopid_frag_sap.light_opacity = 0.5;
    toxopid_frag_sap.status = "sapped".into();
    toxopid_frag_sap.status_duration = 60.0 * 10.0;

    let mut toxopid_cannon = artillery_bullet(3.0, 50.0, "shell");
    toxopid_cannon.despawn_sound = "explosionArtilleryShockBig".into();
    toxopid_cannon.hit_effect = "sapExplosion".into();
    toxopid_cannon.knockback = 0.8;
    toxopid_cannon.lifetime = 80.0;
    toxopid_cannon.width = 25.0;
    toxopid_cannon.height = 25.0;
    toxopid_cannon.collides_tiles = true;
    toxopid_cannon.collides = true;
    toxopid_cannon.ammo_multiplier = 4.0;
    toxopid_cannon.splash_damage_radius = 80.0;
    toxopid_cannon.splash_damage = 75.0;
    toxopid_cannon.back_color = "sapBulletBack".into();
    toxopid_cannon.front_color = "sapBullet".into();
    toxopid_cannon.lightning_color = "sapBullet".into();
    toxopid_cannon.lightning = 5;
    toxopid_cannon.lightning_length = 20;
    toxopid_cannon.smoke_effect = "shootBigSmoke2".into();
    toxopid_cannon.hit_shake = 10.0;
    toxopid_cannon.light_radius = 40.0;
    toxopid_cannon.light_color = "sap".into();
    toxopid_cannon.light_opacity = 0.6;
    toxopid_cannon.status = "sapped".into();
    toxopid_cannon.status_duration = 60.0 * 10.0;
    toxopid_cannon.frag_life_min = 0.3;
    toxopid_cannon.frag_bullets = 9;
    toxopid_cannon.frag_bullet = Some(Box::new(toxopid_frag_sap));

    let mut flare_basic = basic_bullet(2.5, 9.0);
    flare_basic.inaccuracy = 4.0;
    flare_basic.width = 7.0;
    flare_basic.height = 9.0;
    flare_basic.lifetime = 32.0;
    flare_basic.shoot_effect = "shootSmall".into();
    flare_basic.smoke_effect = "shootSmallSmoke".into();
    flare_basic.ammo_multiplier = 2.0;

    let mut horizon_bomb = bomb_bullet(27.0, 25.0);
    horizon_bomb.width = 10.0;
    horizon_bomb.height = 14.0;
    horizon_bomb.hit_effect = "flakExplosion".into();
    horizon_bomb.shoot_effect = "none".into();
    horizon_bomb.smoke_effect = "none".into();
    horizon_bomb.status = "blasted".into();
    horizon_bomb.status_duration = 60.0;
    horizon_bomb.damage = horizon_bomb.splash_damage * 0.5;

    let mut zenith_missile = missile_bullet(3.0, 14.0);
    zenith_missile.width = 8.0;
    zenith_missile.height = 8.0;
    zenith_missile.shrink_y = 0.0;
    zenith_missile.drag = -0.003;
    zenith_missile.homing_range = 60.0;
    zenith_missile.scale_keep_velocity = true;
    zenith_missile.splash_damage_radius = 25.0;
    zenith_missile.splash_damage = 15.0;
    zenith_missile.lifetime = 50.0;
    zenith_missile.trail_color = "unitBack".into();
    zenith_missile.back_color = "unitBack".into();
    zenith_missile.front_color = "unitFront".into();
    zenith_missile.hit_effect = "blastExplosion".into();
    zenith_missile.despawn_effect = "blastExplosion".into();
    zenith_missile.weave_scale = 6.0;
    zenith_missile.weave_mag = 1.0;

    let mut antumbra_missile = missile_bullet(2.7, 18.0);
    antumbra_missile.width = 8.0;
    antumbra_missile.height = 8.0;
    antumbra_missile.shrink_y = 0.0;
    antumbra_missile.drag = -0.01;
    antumbra_missile.splash_damage_radius = 20.0;
    antumbra_missile.splash_damage = 37.0;
    antumbra_missile.ammo_multiplier = 4.0;
    antumbra_missile.lifetime = 50.0;
    antumbra_missile.hit_effect = "blastExplosion".into();
    antumbra_missile.despawn_effect = "blastExplosion".into();
    antumbra_missile.status = "blasted".into();
    antumbra_missile.status_duration = 60.0;

    let mut antumbra_large_bullet = basic_bullet(7.0, 55.0);
    antumbra_large_bullet.width = 12.0;
    antumbra_large_bullet.height = 18.0;
    antumbra_large_bullet.lifetime = 25.0;
    antumbra_large_bullet.shoot_effect = "shootBig".into();

    let mut eclipse_flak = flak_bullet(4.0, 15.0);
    eclipse_flak.shoot_effect = "shootBig".into();
    eclipse_flak.ammo_multiplier = 4.0;
    eclipse_flak.splash_damage = 65.0;
    eclipse_flak.splash_damage_radius = 25.0;
    eclipse_flak.collides_ground = true;
    eclipse_flak.lifetime = 47.0;
    eclipse_flak.status = "blasted".into();
    eclipse_flak.status_duration = 60.0;

    let mut eclipse_laser = laser_bullet(115.0);
    eclipse_laser.side_angle = 20.0;
    eclipse_laser.side_width = 1.5;
    eclipse_laser.side_length = 80.0;
    eclipse_laser.width = 25.0;
    eclipse_laser.length = 230.0;
    eclipse_laser.shoot_effect = "shockwave".into();
    eclipse_laser.colors = vec!["ec7458aa".into(), "ff9c5a".into(), "white".into()];

    let mut poly_missile = missile_bullet(4.0, 12.0);
    poly_missile.homing_power = 0.08;
    poly_missile.weave_mag = 4.0;
    poly_missile.weave_scale = 4.0;
    poly_missile.lifetime = 50.0;
    poly_missile.scale_keep_velocity = true;
    poly_missile.shoot_effect = "shootHeal".into();
    poly_missile.smoke_effect = "hitLaser".into();
    poly_missile.hit_effect = "hitLaser".into();
    poly_missile.despawn_effect = "hitLaser".into();
    poly_missile.front_color = "white".into();
    poly_missile.hit_sound = "none".into();
    poly_missile.heal_percent = 5.5;
    poly_missile.collides_team = true;
    poly_missile.reflectable = false;
    poly_missile.back_color = "heal".into();
    poly_missile.trail_color = "heal".into();

    let mut mega_heal_bolt_large = laser_bolt_bullet(5.2, 10.0);
    mega_heal_bolt_large.lifetime = 35.0;
    mega_heal_bolt_large.heal_percent = 5.5;
    mega_heal_bolt_large.collides_team = true;
    mega_heal_bolt_large.back_color = "heal".into();
    mega_heal_bolt_large.front_color = "white".into();

    let mut mega_heal_bolt_small = laser_bolt_bullet(5.2, 8.0);
    mega_heal_bolt_small.lifetime = 35.0;
    mega_heal_bolt_small.heal_percent = 3.0;
    mega_heal_bolt_small.collides_team = true;
    mega_heal_bolt_small.back_color = "heal".into();
    mega_heal_bolt_small.front_color = "white".into();

    let mut quad_bomb = BulletSpec::new(BulletKind::Basic, 0.0, 0.0);
    quad_bomb.sprite = "large-bomb".into();
    quad_bomb.width = 120.0 / 4.0;
    quad_bomb.height = 120.0 / 4.0;
    quad_bomb.max_range = 30.0;
    quad_bomb.ignore_rotation = true;
    quad_bomb.back_color = "heal".into();
    quad_bomb.front_color = "white".into();
    quad_bomb.mix_color_to = "white".into();
    quad_bomb.hit_sound = "explosionQuad".into();
    quad_bomb.hit_sound_volume = 0.9;
    quad_bomb.shoot_cone = 180.0;
    quad_bomb.eject_effect = "none".into();
    quad_bomb.hit_shake = 4.0;
    quad_bomb.collides_air = false;
    quad_bomb.lifetime = 70.0;
    quad_bomb.despawn_effect = "greenBomb".into();
    quad_bomb.hit_effect = "massiveExplosion".into();
    quad_bomb.keep_velocity = false;
    quad_bomb.spin = 2.0;
    quad_bomb.shrink_x = 0.7;
    quad_bomb.shrink_y = 0.7;
    quad_bomb.collides = false;
    quad_bomb.heal_percent = 15.0;
    quad_bomb.splash_damage = 220.0;
    quad_bomb.splash_damage_radius = 80.0;
    quad_bomb.damage = quad_bomb.splash_damage * 0.7;

    let mut risso_basic = basic_bullet(2.5, 9.0);
    risso_basic.width = 7.0;
    risso_basic.height = 9.0;
    risso_basic.lifetime = 60.0;
    risso_basic.ammo_multiplier = 2.0;

    let mut risso_missile = missile_bullet(2.7, 12.0);
    risso_missile.keep_velocity = true;
    risso_missile.width = 8.0;
    risso_missile.height = 8.0;
    risso_missile.shrink_y = 0.0;
    risso_missile.drag = -0.003;
    risso_missile.homing_range = 60.0;
    risso_missile.splash_damage_radius = 25.0;
    risso_missile.splash_damage = 10.0;
    risso_missile.lifetime = 65.0;
    risso_missile.trail_color = "gray".into();
    risso_missile.back_color = "bulletYellowBack".into();
    risso_missile.front_color = "bulletYellow".into();
    risso_missile.hit_effect = "blastExplosion".into();
    risso_missile.despawn_effect = "blastExplosion".into();
    risso_missile.weave_scale = 8.0;
    risso_missile.weave_mag = 2.0;

    let mut minke_flak = flak_bullet(4.2, 3.0);
    minke_flak.lifetime = 52.5;
    minke_flak.ammo_multiplier = 4.0;
    minke_flak.shoot_effect = "shootSmall".into();
    minke_flak.width = 6.0;
    minke_flak.height = 8.0;
    minke_flak.hit_effect = "flakExplosion".into();
    minke_flak.splash_damage = 27.0 * 1.5;
    minke_flak.splash_damage_radius = 15.0;

    let mut minke_artillery = artillery_bullet(3.0, 20.0, "shell");
    minke_artillery.hit_effect = "flakExplosion".into();
    minke_artillery.knockback = 0.8;
    minke_artillery.lifetime = 73.5;
    minke_artillery.width = 11.0;
    minke_artillery.height = 11.0;
    minke_artillery.collides_tiles = false;
    minke_artillery.splash_damage_radius = 30.0 * 0.75;
    minke_artillery.splash_damage = 40.0;

    let mut bryde_artillery = artillery_bullet(3.2, 15.0, "shell");
    bryde_artillery.trail_mult = 0.8;
    bryde_artillery.hit_effect = "massiveExplosion".into();
    bryde_artillery.knockback = 1.5;
    bryde_artillery.lifetime = 84.0;
    bryde_artillery.height = 15.5;
    bryde_artillery.width = 15.0;
    bryde_artillery.collides_tiles = false;
    bryde_artillery.splash_damage_radius = 40.0;
    bryde_artillery.splash_damage = 70.0;
    bryde_artillery.back_color = "missileYellowBack".into();
    bryde_artillery.front_color = "missileYellow".into();
    bryde_artillery.trail_effect = "artilleryTrail".into();
    bryde_artillery.trail_size = 6.0;
    bryde_artillery.hit_shake = 4.0;
    bryde_artillery.shoot_effect = "shootBig2".into();
    bryde_artillery.status = "blasted".into();
    bryde_artillery.status_duration = 60.0;

    let mut bryde_missile = missile_bullet(2.7, 12.0);
    bryde_missile.width = 8.0;
    bryde_missile.height = 8.0;
    bryde_missile.shrink_y = 0.0;
    bryde_missile.drag = -0.003;
    bryde_missile.homing_range = 60.0;
    bryde_missile.keep_velocity = false;
    bryde_missile.splash_damage_radius = 25.0;
    bryde_missile.splash_damage = 10.0;
    bryde_missile.lifetime = 70.0;
    bryde_missile.trail_color = "gray".into();
    bryde_missile.back_color = "bulletYellowBack".into();
    bryde_missile.front_color = "bulletYellow".into();
    bryde_missile.hit_effect = "blastExplosion".into();
    bryde_missile.despawn_effect = "blastExplosion".into();
    bryde_missile.weave_scale = 8.0;
    bryde_missile.weave_mag = 1.0;

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
        make_bullet(&mut next_id, "arkyid_sapper", arkyid_sapper),
        make_bullet(&mut next_id, "arkyid_artillery_sap", arkyid_artillery_sap),
        make_bullet(&mut next_id, "toxopid_shrapnel", toxopid_shrapnel),
        make_bullet(&mut next_id, "toxopid_cannon", toxopid_cannon),
        make_bullet(&mut next_id, "flare_basic", flare_basic),
        make_bullet(&mut next_id, "horizon_bomb", horizon_bomb),
        make_bullet(&mut next_id, "zenith_missile", zenith_missile),
        make_bullet(&mut next_id, "antumbra_missile", antumbra_missile),
        make_bullet(&mut next_id, "antumbra_large_bullet", antumbra_large_bullet),
        make_bullet(&mut next_id, "eclipse_flak", eclipse_flak),
        make_bullet(&mut next_id, "eclipse_laser", eclipse_laser),
        make_bullet(&mut next_id, "poly_missile", poly_missile),
        make_bullet(&mut next_id, "mega_heal_bolt_large", mega_heal_bolt_large),
        make_bullet(&mut next_id, "mega_heal_bolt_small", mega_heal_bolt_small),
        make_bullet(&mut next_id, "quad_bomb", quad_bomb),
        make_bullet(&mut next_id, "risso_basic", risso_basic),
        make_bullet(&mut next_id, "risso_missile", risso_missile),
        make_bullet(&mut next_id, "minke_flak", minke_flak),
        make_bullet(&mut next_id, "minke_artillery", minke_artillery),
        make_bullet(&mut next_id, "bryde_artillery", bryde_artillery),
        make_bullet(&mut next_id, "bryde_missile", bryde_missile),
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

fn shrapnel_bullet(damage: f32) -> BulletSpec {
    let mut bullet = BulletSpec::new(BulletKind::Shrapnel, 0.0, damage);
    bullet.hit_effect = "hitLancer".into();
    bullet.shoot_effect = "lightningShoot".into();
    bullet.smoke_effect = "lightningShoot".into();
    bullet.lifetime = 10.0;
    bullet.despawn_effect = "none".into();
    bullet.keep_velocity = false;
    bullet.collides = false;
    bullet.pierce = true;
    bullet.hittable = false;
    bullet.absorbable = false;
    bullet.light_opacity = 0.6;
    bullet.length = 100.0;
    bullet.width = 20.0;
    bullet.from_color = "white".into();
    bullet.to_color = "lancerLaser".into();
    bullet
}

fn bomb_bullet(splash_damage: f32, splash_damage_radius: f32) -> BulletSpec {
    let mut bullet = BulletSpec::new(BulletKind::Bomb, 0.7, 0.0);
    bullet.sprite = "shell".into();
    bullet.splash_damage_radius = splash_damage_radius;
    bullet.splash_damage = splash_damage;
    bullet.collides_tiles = false;
    bullet.collides = false;
    bullet.shrink_y = 0.7;
    bullet.lifetime = 30.0;
    bullet.drag = 0.05;
    bullet.keep_velocity = false;
    bullet.collides_air = false;
    bullet.hit_sound = "explosion".into();
    bullet.width = 5.0;
    bullet.height = 7.0;
    bullet.back_color = "bulletYellowBack".into();
    bullet.front_color = "bulletYellow".into();
    bullet
}

fn missile_bullet(speed: f32, damage: f32) -> BulletSpec {
    let mut bullet = BulletSpec::new(BulletKind::Missile, speed, damage);
    bullet.sprite = "missile".into();
    bullet.back_color = "missileYellowBack".into();
    bullet.front_color = "missileYellow".into();
    bullet.homing_power = 0.08;
    bullet.shrink_y = 0.0;
    bullet.width = 8.0;
    bullet.height = 8.0;
    bullet.hit_sound = "explosion".into();
    bullet.trail_chance = 0.2;
    bullet.lifetime = 52.0;
    bullet
}

fn flak_bullet(speed: f32, damage: f32) -> BulletSpec {
    let mut bullet = BulletSpec::new(BulletKind::Flak, speed, damage);
    bullet.splash_damage = 15.0;
    bullet.splash_damage_radius = 34.0;
    bullet.hit_effect = "flakExplosionBig".into();
    bullet.width = 8.0;
    bullet.height = 10.0;
    bullet.collides_ground = false;
    bullet.explode_range = 30.0;
    bullet.explode_delay = 5.0;
    bullet.flak_delay = 0.0;
    bullet.flak_interval = 6.0;
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
                "arkyid_sapper",
                "arkyid_artillery_sap",
                "toxopid_shrapnel",
                "toxopid_cannon",
                "flare_basic",
                "horizon_bomb",
                "zenith_missile",
                "antumbra_missile",
                "antumbra_large_bullet",
                "eclipse_flak",
                "eclipse_laser",
                "poly_missile",
                "mega_heal_bolt_large",
                "mega_heal_bolt_small",
                "quad_bomb",
                "risso_basic",
                "risso_missile",
                "minke_flak",
                "minke_artillery",
                "bryde_artillery",
                "bryde_missile",
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
    fn arkyid_sapper_and_artillery_match_java_profiles() {
        let bullets = load();
        let sapper = &by_name(&bullets, "arkyid_sapper").spec;
        let artillery = &by_name(&bullets, "arkyid_artillery_sap").spec;

        assert_eq!(sapper.kind, BulletKind::Sap);
        assert_eq!(sapper.sap_strength, 0.85);
        assert_eq!(sapper.length, 55.0);
        assert_eq!(sapper.damage, 40.0);
        assert_eq!(sapper.shoot_effect, "shootSmall");
        assert_eq!(sapper.hit_color, "bf92f9");
        assert_eq!(sapper.color, "bf92f9");
        assert_eq!(sapper.width, 0.55);
        assert_eq!(sapper.lifetime, 30.0);
        assert_eq!(sapper.knockback, -1.0);
        assert_eq!(sapper.status, "sapped");
        assert!(!sapper.collides);
        assert!(sapper.pierce);

        assert_eq!(artillery.kind, BulletKind::Artillery);
        assert_eq!(artillery.speed, 2.0);
        assert_eq!(artillery.damage, 12.0);
        assert_eq!(artillery.hit_effect, "sapExplosion");
        assert_eq!(artillery.despawn_sound, "explosionArtilleryShock");
        assert_eq!(artillery.knockback, 0.8);
        assert_eq!(artillery.lifetime, 70.0);
        assert_eq!(artillery.width, 19.0);
        assert_eq!(artillery.height, 19.0);
        assert!(artillery.collides_tiles);
        assert!(!artillery.collides);
        assert_eq!(artillery.ammo_multiplier, 4.0);
        assert_eq!(artillery.splash_damage_radius, 70.0);
        assert_eq!(artillery.splash_damage, 65.0);
        assert_eq!(artillery.back_color, "sapBulletBack");
        assert_eq!(artillery.front_color, "sapBullet");
        assert_eq!(artillery.lightning_color, "sapBullet");
        assert_eq!(artillery.lightning, 3);
        assert_eq!(artillery.lightning_length, 10);
        assert_eq!(artillery.smoke_effect, "shootBigSmoke2");
        assert_eq!(artillery.shake, 5.0);
        assert_eq!(artillery.status, "sapped");
        assert_eq!(artillery.status_duration, 600.0);
    }

    #[test]
    fn toxopid_shrapnel_and_cannon_match_java_profiles() {
        let bullets = load();
        let shrapnel = &by_name(&bullets, "toxopid_shrapnel").spec;
        let cannon = &by_name(&bullets, "toxopid_cannon").spec;

        assert_eq!(shrapnel.kind, BulletKind::Shrapnel);
        assert_eq!(shrapnel.length, 90.0);
        assert_eq!(shrapnel.damage, 110.0);
        assert_eq!(shrapnel.width, 25.0);
        assert_eq!(shrapnel.serration_len_scl, 7.0);
        assert_eq!(shrapnel.serration_space_offset, 60.0);
        assert_eq!(shrapnel.serration_fade_offset, 0.0);
        assert_eq!(shrapnel.serrations, 10);
        assert_eq!(shrapnel.serration_width, 6.0);
        assert_eq!(shrapnel.from_color, "sapBullet");
        assert_eq!(shrapnel.to_color, "sapBulletBack");
        assert_eq!(shrapnel.shoot_effect, "sparkShoot");
        assert_eq!(shrapnel.smoke_effect, "sparkShoot");
        assert!(!shrapnel.collides);
        assert!(shrapnel.pierce);
        assert!(!shrapnel.hittable);
        assert!(!shrapnel.absorbable);

        assert_eq!(cannon.kind, BulletKind::Artillery);
        assert_eq!(cannon.speed, 3.0);
        assert_eq!(cannon.damage, 50.0);
        assert_eq!(cannon.despawn_sound, "explosionArtilleryShockBig");
        assert_eq!(cannon.hit_effect, "sapExplosion");
        assert_eq!(cannon.knockback, 0.8);
        assert_eq!(cannon.lifetime, 80.0);
        assert_eq!(cannon.width, 25.0);
        assert_eq!(cannon.height, 25.0);
        assert!(cannon.collides_tiles);
        assert!(cannon.collides);
        assert_eq!(cannon.ammo_multiplier, 4.0);
        assert_eq!(cannon.splash_damage_radius, 80.0);
        assert_eq!(cannon.splash_damage, 75.0);
        assert_eq!(cannon.back_color, "sapBulletBack");
        assert_eq!(cannon.front_color, "sapBullet");
        assert_eq!(cannon.lightning_color, "sapBullet");
        assert_eq!(cannon.lightning, 5);
        assert_eq!(cannon.lightning_length, 20);
        assert_eq!(cannon.smoke_effect, "shootBigSmoke2");
        assert_eq!(cannon.hit_shake, 10.0);
        assert_eq!(cannon.light_radius, 40.0);
        assert_eq!(cannon.light_color, "sap");
        assert_eq!(cannon.light_opacity, 0.6);
        assert_eq!(cannon.status, "sapped");
        assert_eq!(cannon.status_duration, 600.0);
        assert_eq!(cannon.frag_life_min, 0.3);
        assert_eq!(cannon.frag_bullets, 9);

        let frag = cannon
            .frag_bullet
            .as_ref()
            .expect("toxopid cannon should carry frag bullet");
        assert_eq!(frag.kind, BulletKind::Artillery);
        assert_eq!(frag.speed, 2.3);
        assert_eq!(frag.damage, 30.0);
        assert_eq!(frag.despawn_sound, "explosionArtilleryShock");
        assert_eq!(frag.hit_effect, "sapExplosion");
        assert_eq!(frag.lifetime, 90.0);
        assert_eq!(frag.width, 20.0);
        assert_eq!(frag.height, 20.0);
        assert!(!frag.collides_tiles);
        assert_eq!(frag.splash_damage_radius, 70.0);
        assert_eq!(frag.splash_damage, 40.0);
        assert_eq!(frag.lightning, 2);
        assert_eq!(frag.lightning_length, 5);
        assert_eq!(frag.hit_shake, 5.0);
        assert_eq!(frag.light_radius, 30.0);
        assert_eq!(frag.light_color, "sap");
        assert_eq!(frag.light_opacity, 0.5);
        assert_eq!(frag.status, "sapped");
        assert_eq!(frag.status_duration, 600.0);
    }

    #[test]
    fn flare_basic_and_horizon_bomb_match_java_profiles() {
        let bullets = load();
        let flare = &by_name(&bullets, "flare_basic").spec;
        let bomb = &by_name(&bullets, "horizon_bomb").spec;

        assert_eq!(flare.kind, BulletKind::Basic);
        assert_eq!(flare.speed, 2.5);
        assert_eq!(flare.damage, 9.0);
        assert_eq!(flare.inaccuracy, 4.0);
        assert_eq!(flare.width, 7.0);
        assert_eq!(flare.height, 9.0);
        assert_eq!(flare.lifetime, 32.0);
        assert_eq!(flare.shoot_effect, "shootSmall");
        assert_eq!(flare.smoke_effect, "shootSmallSmoke");
        assert_eq!(flare.ammo_multiplier, 2.0);

        assert_eq!(bomb.kind, BulletKind::Bomb);
        assert_eq!(bomb.speed, 0.7);
        assert_eq!(bomb.splash_damage, 27.0);
        assert_eq!(bomb.splash_damage_radius, 25.0);
        assert_eq!(bomb.width, 10.0);
        assert_eq!(bomb.height, 14.0);
        assert_eq!(bomb.hit_effect, "flakExplosion");
        assert_eq!(bomb.shoot_effect, "none");
        assert_eq!(bomb.smoke_effect, "none");
        assert_eq!(bomb.status, "blasted");
        assert_eq!(bomb.status_duration, 60.0);
        assert_eq!(bomb.damage, 13.5);
        assert!(!bomb.collides_tiles);
        assert!(!bomb.collides);
        assert!(!bomb.collides_air);
        assert_eq!(bomb.drag, 0.05);
        assert!(!bomb.keep_velocity);
        assert_eq!(bomb.hit_sound, "explosion");
    }

    #[test]
    fn zenith_missile_matches_java_profile() {
        let bullets = load();
        let bullet = &by_name(&bullets, "zenith_missile").spec;

        assert_eq!(bullet.kind, BulletKind::Missile);
        assert_eq!(bullet.speed, 3.0);
        assert_eq!(bullet.damage, 14.0);
        assert_eq!(bullet.sprite, "missile");
        assert_eq!(bullet.width, 8.0);
        assert_eq!(bullet.height, 8.0);
        assert_eq!(bullet.shrink_y, 0.0);
        assert_eq!(bullet.drag, -0.003);
        assert_eq!(bullet.homing_power, 0.08);
        assert_eq!(bullet.homing_range, 60.0);
        assert!(bullet.scale_keep_velocity);
        assert_eq!(bullet.splash_damage_radius, 25.0);
        assert_eq!(bullet.splash_damage, 15.0);
        assert_eq!(bullet.lifetime, 50.0);
        assert_eq!(bullet.trail_color, "unitBack");
        assert_eq!(bullet.back_color, "unitBack");
        assert_eq!(bullet.front_color, "unitFront");
        assert_eq!(bullet.hit_effect, "blastExplosion");
        assert_eq!(bullet.despawn_effect, "blastExplosion");
        assert_eq!(bullet.weave_scale, 6.0);
        assert_eq!(bullet.weave_mag, 1.0);
        assert_eq!(bullet.hit_sound, "explosion");
        assert_eq!(bullet.trail_chance, 0.2);
    }

    #[test]
    fn antumbra_bullets_match_java_profiles() {
        let bullets = load();
        let missile = &by_name(&bullets, "antumbra_missile").spec;

        assert_eq!(missile.kind, BulletKind::Missile);
        assert_eq!(missile.speed, 2.7);
        assert_eq!(missile.damage, 18.0);
        assert_eq!(missile.width, 8.0);
        assert_eq!(missile.height, 8.0);
        assert_eq!(missile.shrink_y, 0.0);
        assert_eq!(missile.drag, -0.01);
        assert_eq!(missile.splash_damage_radius, 20.0);
        assert_eq!(missile.splash_damage, 37.0);
        assert_eq!(missile.ammo_multiplier, 4.0);
        assert_eq!(missile.lifetime, 50.0);
        assert_eq!(missile.hit_effect, "blastExplosion");
        assert_eq!(missile.despawn_effect, "blastExplosion");
        assert_eq!(missile.status, "blasted");
        assert_eq!(missile.status_duration, 60.0);
        assert_eq!(missile.hit_sound, "explosion");
        assert_eq!(missile.trail_chance, 0.2);

        let large = &by_name(&bullets, "antumbra_large_bullet").spec;
        assert_eq!(large.kind, BulletKind::Basic);
        assert_eq!(large.speed, 7.0);
        assert_eq!(large.damage, 55.0);
        assert_eq!(large.width, 12.0);
        assert_eq!(large.height, 18.0);
        assert_eq!(large.lifetime, 25.0);
        assert_eq!(large.shoot_effect, "shootBig");
        assert_eq!(large.sprite, "bullet");
    }

    #[test]
    fn eclipse_bullets_match_java_profiles() {
        let bullets = load();
        let flak = &by_name(&bullets, "eclipse_flak").spec;

        assert_eq!(flak.kind, BulletKind::Flak);
        assert_eq!(flak.speed, 4.0);
        assert_eq!(flak.damage, 15.0);
        assert_eq!(flak.shoot_effect, "shootBig");
        assert_eq!(flak.ammo_multiplier, 4.0);
        assert_eq!(flak.splash_damage, 65.0);
        assert_eq!(flak.splash_damage_radius, 25.0);
        assert!(flak.collides_ground);
        assert_eq!(flak.lifetime, 47.0);
        assert_eq!(flak.status, "blasted");
        assert_eq!(flak.status_duration, 60.0);
        assert_eq!(flak.hit_effect, "flakExplosionBig");
        assert_eq!(flak.explode_range, 30.0);
        assert_eq!(flak.flak_interval, 6.0);

        let laser = &by_name(&bullets, "eclipse_laser").spec;
        assert_eq!(laser.kind, BulletKind::Laser);
        assert_eq!(laser.damage, 115.0);
        assert_eq!(laser.side_angle, 20.0);
        assert_eq!(laser.side_width, 1.5);
        assert_eq!(laser.side_length, 80.0);
        assert_eq!(laser.width, 25.0);
        assert_eq!(laser.length, 230.0);
        assert_eq!(laser.shoot_effect, "shockwave");
        assert_eq!(
            laser.colors,
            vec![
                "ec7458aa".to_string(),
                "ff9c5a".to_string(),
                "white".to_string()
            ]
        );
        assert!(laser.impact);
        assert!(laser.pierce);
    }

    #[test]
    fn poly_missile_matches_java_heal_profile() {
        let bullets = load();
        let missile = &by_name(&bullets, "poly_missile").spec;

        assert_eq!(missile.kind, BulletKind::Missile);
        assert_eq!(missile.speed, 4.0);
        assert_eq!(missile.damage, 12.0);
        assert_eq!(missile.homing_power, 0.08);
        assert_eq!(missile.weave_mag, 4.0);
        assert_eq!(missile.weave_scale, 4.0);
        assert_eq!(missile.lifetime, 50.0);
        assert!(missile.scale_keep_velocity);
        assert_eq!(missile.shoot_effect, "shootHeal");
        assert_eq!(missile.smoke_effect, "hitLaser");
        assert_eq!(missile.hit_effect, "hitLaser");
        assert_eq!(missile.despawn_effect, "hitLaser");
        assert_eq!(missile.front_color, "white");
        assert_eq!(missile.hit_sound, "none");
        assert_eq!(missile.heal_percent, 5.5);
        assert!(missile.collides_team);
        assert!(!missile.reflectable);
        assert_eq!(missile.back_color, "heal");
        assert_eq!(missile.trail_color, "heal");
    }

    #[test]
    fn mega_heal_bolts_match_java_profiles() {
        let bullets = load();
        let large = &by_name(&bullets, "mega_heal_bolt_large").spec;

        assert_eq!(large.kind, BulletKind::LaserBolt);
        assert_eq!(large.speed, 5.2);
        assert_eq!(large.damage, 10.0);
        assert_eq!(large.lifetime, 35.0);
        assert_eq!(large.heal_percent, 5.5);
        assert!(large.collides_team);
        assert_eq!(large.back_color, "heal");
        assert_eq!(large.front_color, "white");
        assert_eq!(large.light_color, "heal");
        assert!(!large.hittable);
        assert!(!large.reflectable);

        let small = &by_name(&bullets, "mega_heal_bolt_small").spec;
        assert_eq!(small.kind, BulletKind::LaserBolt);
        assert_eq!(small.speed, 5.2);
        assert_eq!(small.damage, 8.0);
        assert_eq!(small.lifetime, 35.0);
        assert_eq!(small.heal_percent, 3.0);
        assert!(small.collides_team);
        assert_eq!(small.back_color, "heal");
        assert_eq!(small.front_color, "white");
        assert_eq!(small.light_color, "heal");
        assert!(!small.hittable);
        assert!(!small.reflectable);
    }

    #[test]
    fn quad_bomb_matches_java_profile() {
        let bullets = load();
        let bomb = &by_name(&bullets, "quad_bomb").spec;

        assert_eq!(bomb.kind, BulletKind::Basic);
        assert_eq!(bomb.sprite, "large-bomb");
        assert_eq!(bomb.width, 30.0);
        assert_eq!(bomb.height, 30.0);
        assert_eq!(bomb.max_range, 30.0);
        assert!(bomb.ignore_rotation);
        assert_eq!(bomb.back_color, "heal");
        assert_eq!(bomb.front_color, "white");
        assert_eq!(bomb.mix_color_to, "white");
        assert_eq!(bomb.hit_sound, "explosionQuad");
        assert_eq!(bomb.hit_sound_volume, 0.9);
        assert_eq!(bomb.shoot_cone, 180.0);
        assert_eq!(bomb.eject_effect, "none");
        assert_eq!(bomb.hit_shake, 4.0);
        assert!(!bomb.collides_air);
        assert_eq!(bomb.lifetime, 70.0);
        assert_eq!(bomb.despawn_effect, "greenBomb");
        assert_eq!(bomb.hit_effect, "massiveExplosion");
        assert!(!bomb.keep_velocity);
        assert_eq!(bomb.spin, 2.0);
        assert_eq!(bomb.shrink_x, 0.7);
        assert_eq!(bomb.shrink_y, 0.7);
        assert_eq!(bomb.speed, 0.0);
        assert!(!bomb.collides);
        assert_eq!(bomb.heal_percent, 15.0);
        assert_eq!(bomb.splash_damage, 220.0);
        assert_eq!(bomb.splash_damage_radius, 80.0);
        assert_eq!(bomb.damage, 154.0);
    }

    #[test]
    fn risso_bullets_match_java_profiles() {
        let bullets = load();
        let basic = &by_name(&bullets, "risso_basic").spec;

        assert_eq!(basic.kind, BulletKind::Basic);
        assert_eq!(basic.speed, 2.5);
        assert_eq!(basic.damage, 9.0);
        assert_eq!(basic.width, 7.0);
        assert_eq!(basic.height, 9.0);
        assert_eq!(basic.lifetime, 60.0);
        assert_eq!(basic.ammo_multiplier, 2.0);

        let missile = &by_name(&bullets, "risso_missile").spec;
        assert_eq!(missile.kind, BulletKind::Missile);
        assert_eq!(missile.speed, 2.7);
        assert_eq!(missile.damage, 12.0);
        assert!(missile.keep_velocity);
        assert_eq!(missile.width, 8.0);
        assert_eq!(missile.height, 8.0);
        assert_eq!(missile.shrink_y, 0.0);
        assert_eq!(missile.drag, -0.003);
        assert_eq!(missile.homing_range, 60.0);
        assert_eq!(missile.splash_damage_radius, 25.0);
        assert_eq!(missile.splash_damage, 10.0);
        assert_eq!(missile.lifetime, 65.0);
        assert_eq!(missile.trail_color, "gray");
        assert_eq!(missile.back_color, "bulletYellowBack");
        assert_eq!(missile.front_color, "bulletYellow");
        assert_eq!(missile.hit_effect, "blastExplosion");
        assert_eq!(missile.despawn_effect, "blastExplosion");
        assert_eq!(missile.weave_scale, 8.0);
        assert_eq!(missile.weave_mag, 2.0);
    }

    #[test]
    fn minke_bullets_match_java_profiles() {
        let bullets = load();
        let flak = &by_name(&bullets, "minke_flak").spec;

        assert_eq!(flak.kind, BulletKind::Flak);
        assert_eq!(flak.speed, 4.2);
        assert_eq!(flak.damage, 3.0);
        assert_eq!(flak.lifetime, 52.5);
        assert_eq!(flak.ammo_multiplier, 4.0);
        assert_eq!(flak.shoot_effect, "shootSmall");
        assert_eq!(flak.width, 6.0);
        assert_eq!(flak.height, 8.0);
        assert_eq!(flak.hit_effect, "flakExplosion");
        assert_eq!(flak.splash_damage, 40.5);
        assert_eq!(flak.splash_damage_radius, 15.0);
        assert!(!flak.collides_ground);

        let artillery = &by_name(&bullets, "minke_artillery").spec;
        assert_eq!(artillery.kind, BulletKind::Artillery);
        assert_eq!(artillery.speed, 3.0);
        assert_eq!(artillery.damage, 20.0);
        assert_eq!(artillery.sprite, "shell");
        assert_eq!(artillery.hit_effect, "flakExplosion");
        assert_eq!(artillery.knockback, 0.8);
        assert_eq!(artillery.lifetime, 73.5);
        assert_eq!(artillery.width, 11.0);
        assert_eq!(artillery.height, 11.0);
        assert!(!artillery.collides_tiles);
        assert_eq!(artillery.splash_damage_radius, 22.5);
        assert_eq!(artillery.splash_damage, 40.0);
    }

    #[test]
    fn bryde_bullets_match_java_profiles() {
        let bullets = load();
        let artillery = &by_name(&bullets, "bryde_artillery").spec;

        assert_eq!(artillery.kind, BulletKind::Artillery);
        assert_eq!(artillery.speed, 3.2);
        assert_eq!(artillery.damage, 15.0);
        assert_eq!(artillery.trail_mult, 0.8);
        assert_eq!(artillery.hit_effect, "massiveExplosion");
        assert_eq!(artillery.knockback, 1.5);
        assert_eq!(artillery.lifetime, 84.0);
        assert_eq!(artillery.height, 15.5);
        assert_eq!(artillery.width, 15.0);
        assert!(!artillery.collides_tiles);
        assert_eq!(artillery.splash_damage_radius, 40.0);
        assert_eq!(artillery.splash_damage, 70.0);
        assert_eq!(artillery.back_color, "missileYellowBack");
        assert_eq!(artillery.front_color, "missileYellow");
        assert_eq!(artillery.trail_effect, "artilleryTrail");
        assert_eq!(artillery.trail_size, 6.0);
        assert_eq!(artillery.hit_shake, 4.0);
        assert_eq!(artillery.shoot_effect, "shootBig2");
        assert_eq!(artillery.status, "blasted");
        assert_eq!(artillery.status_duration, 60.0);

        let missile = &by_name(&bullets, "bryde_missile").spec;
        assert_eq!(missile.kind, BulletKind::Missile);
        assert_eq!(missile.speed, 2.7);
        assert_eq!(missile.damage, 12.0);
        assert_eq!(missile.width, 8.0);
        assert_eq!(missile.height, 8.0);
        assert_eq!(missile.shrink_y, 0.0);
        assert_eq!(missile.drag, -0.003);
        assert_eq!(missile.homing_range, 60.0);
        assert!(!missile.keep_velocity);
        assert_eq!(missile.splash_damage_radius, 25.0);
        assert_eq!(missile.splash_damage, 10.0);
        assert_eq!(missile.lifetime, 70.0);
        assert_eq!(missile.trail_color, "gray");
        assert_eq!(missile.back_color, "bulletYellowBack");
        assert_eq!(missile.front_color, "bulletYellow");
        assert_eq!(missile.hit_effect, "blastExplosion");
        assert_eq!(missile.despawn_effect, "blastExplosion");
        assert_eq!(missile.weave_scale, 8.0);
        assert_eq!(missile.weave_mag, 1.0);
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
