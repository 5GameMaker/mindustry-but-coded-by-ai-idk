use crate::mindustry::{entities::comp::DecalColor, graphics::Layer, vars::TILE_SIZE};

pub const SHAKE_FALLOFF: f32 = 10000.0;
pub const DEFAULT_EFFECT_LIFETIME: f32 = 50.0;
pub const DEFAULT_EFFECT_CLIP: f32 = 50.0;
pub const DEFAULT_EFFECT_LAYER: f32 = 110.0;
/// Upstream `Fx.unitAssemble` id in `mindustry.content.Fx` for v158.1.
pub const FX_UNIT_ASSEMBLE_ID: i32 = 35;
/// Upstream `Fx.smokeAoeCloud` id in `mindustry.content.Fx` for v158.1.
pub const FX_SMOKE_AOE_CLOUD_ID: i32 = 55;
/// Upstream `Fx.healWaveDynamic` id in `mindustry.content.Fx` for v158.1.
pub const FX_HEAL_WAVE_DYNAMIC_ID: i32 = 70;
/// Upstream `Fx.healWave` id in `mindustry.content.Fx` for v158.1.
pub const FX_HEAL_WAVE_ID: i32 = 71;
/// Upstream `Fx.heal` id in `mindustry.content.Fx` for v158.1.
pub const FX_HEAL_ID: i32 = 72;
/// Upstream `Fx.dynamicWave` id in `mindustry.content.Fx` for v158.1.
pub const FX_DYNAMIC_WAVE_ID: i32 = 73;
/// Upstream `Fx.shieldWave` id in `mindustry.content.Fx` for v158.1.
pub const FX_SHIELD_WAVE_ID: i32 = 74;
/// Upstream `Fx.shieldApply` id in `mindustry.content.Fx` for v158.1.
pub const FX_SHIELD_APPLY_ID: i32 = 75;
/// Upstream `Fx.disperseTrail` id in `mindustry.content.Fx` for v158.1.
pub const FX_DISPERSE_TRAIL_ID: i32 = 76;
/// Upstream `Fx.hitBulletBig` id in `mindustry.content.Fx` for v158.1.
pub const FX_HIT_BULLET_BIG_ID: i32 = 82;
/// Upstream `Fx.hitFlameSmall` id in `mindustry.content.Fx` for v158.1.
pub const FX_HIT_FLAME_SMALL_ID: i32 = 83;
/// Upstream `Fx.hitFlamePlasma` id in `mindustry.content.Fx` for v158.1.
pub const FX_HIT_FLAME_PLASMA_ID: i32 = 84;
/// Upstream `Fx.hitLaserBlast` id in `mindustry.content.Fx` for v158.1.
pub const FX_HIT_LASER_BLAST_ID: i32 = 86;
/// Upstream `Fx.hitEmpSpark` id in `mindustry.content.Fx` for v158.1.
pub const FX_HIT_EMP_SPARK_ID: i32 = 87;
/// Upstream `Fx.hitLancer` id in `mindustry.content.Fx` for v158.1.
pub const FX_HIT_LANCER_ID: i32 = 88;
/// Upstream `Fx.hitLancerLow` id in `mindustry.content.Fx` for v158.1.
pub const FX_HIT_LANCER_LOW_ID: i32 = 89;
/// Upstream `Fx.hitBeam` id in `mindustry.content.Fx` for v158.1.
pub const FX_HIT_BEAM_ID: i32 = 90;
/// Upstream `Fx.hitFlameBeam` id in `mindustry.content.Fx` for v158.1.
pub const FX_HIT_FLAME_BEAM_ID: i32 = 91;
/// Upstream `Fx.hitMeltdown` id in `mindustry.content.Fx` for v158.1.
pub const FX_HIT_MELTDOWN_ID: i32 = 92;
/// Upstream `Fx.hitMeltHeal` id in `mindustry.content.Fx` for v158.1.
pub const FX_HIT_MELT_HEAL_ID: i32 = 93;
/// Upstream `Fx.hitLaser` id in `mindustry.content.Fx` for v158.1.
pub const FX_HIT_LASER_ID: i32 = 98;
/// Upstream `Fx.despawn` id in `mindustry.content.Fx` for v158.1.
pub const FX_DESPAWN_ID: i32 = 100;
/// Upstream `Fx.smoke` id in `mindustry.content.Fx` for v158.1.
pub const FX_SMOKE_ID: i32 = 28;
/// Upstream `Fx.fallSmoke` id in `mindustry.content.Fx` for v158.1.
pub const FX_FALL_SMOKE_ID: i32 = 29;
/// Upstream `Fx.rocketSmoke` id in `mindustry.content.Fx` for v158.1.
pub const FX_ROCKET_SMOKE_ID: i32 = 31;
/// Upstream `Fx.rocketSmokeLarge` id in `mindustry.content.Fx` for v158.1.
pub const FX_ROCKET_SMOKE_LARGE_ID: i32 = 32;
/// Upstream `Fx.magmasmoke` id in `mindustry.content.Fx` for v158.1.
pub const FX_MAGMA_SMOKE_ID: i32 = 33;
/// Upstream `Fx.breakProp` id in `mindustry.content.Fx` for v158.1.
pub const FX_BREAK_PROP_ID: i32 = 37;
/// Upstream `Fx.unitDrop` id in `mindustry.content.Fx` for v158.1.
pub const FX_UNIT_DROP_ID: i32 = 38;
/// Upstream `Fx.unitLand` id in `mindustry.content.Fx` for v158.1.
pub const FX_UNIT_LAND_ID: i32 = 39;
/// Upstream `Fx.unitDust` id in `mindustry.content.Fx` for v158.1.
pub const FX_UNIT_DUST_ID: i32 = 40;
/// Upstream `Fx.unitLandSmall` id in `mindustry.content.Fx` for v158.1.
pub const FX_UNIT_LAND_SMALL_ID: i32 = 41;
/// Upstream `Fx.crawlDust` id in `mindustry.content.Fx` for v158.1.
pub const FX_CRAWL_DUST_ID: i32 = 43;
/// Upstream `Fx.hitLiquid` id in `mindustry.content.Fx` for v158.1.
pub const FX_HIT_LIQUID_ID: i32 = 85;
/// Upstream `Fx.artilleryTrail` id in `mindustry.content.Fx` for v158.1.
pub const FX_ARTILLERY_TRAIL_ID: i32 = 108;
/// Upstream `Fx.incendTrail` id in `mindustry.content.Fx` for v158.1.
pub const FX_INCEND_TRAIL_ID: i32 = 109;
/// Upstream `Fx.missileTrail` id in `mindustry.content.Fx` for v158.1.
pub const FX_MISSILE_TRAIL_ID: i32 = 110;
/// Upstream `Fx.missileTrailShort` id in `mindustry.content.Fx` for v158.1.
pub const FX_MISSILE_TRAIL_SHORT_ID: i32 = 111;
/// Upstream `Fx.colorTrail` id in `mindustry.content.Fx` for v158.1.
pub const FX_COLOR_TRAIL_ID: i32 = 113;
/// Upstream `Fx.absorb` id in `mindustry.content.Fx` for v158.1.
pub const FX_ABSORB_ID: i32 = 114;
/// Upstream `Fx.burning` id in `mindustry.content.Fx` for v158.1.
pub const FX_BURNING_ID: i32 = 117;
/// Upstream `Fx.fire` id in `mindustry.content.Fx` for v158.1.
pub const FX_FIRE_ID: i32 = 119;
/// Upstream `Fx.fireHit` id in `mindustry.content.Fx` for v158.1.
pub const FX_FIRE_HIT_ID: i32 = 120;
/// Upstream `Fx.fireSmoke` id in `mindustry.content.Fx` for v158.1.
pub const FX_FIRE_SMOKE_ID: i32 = 121;
/// Upstream `Fx.neoplasmHeal` id in `mindustry.content.Fx` for v158.1.
pub const FX_NEOPLASM_HEAL_ID: i32 = 122;
/// Upstream `Fx.steam` id in `mindustry.content.Fx` for v158.1.
pub const FX_STEAM_ID: i32 = 123;
/// Upstream `Fx.vapor` id in `mindustry.content.Fx` for v158.1.
pub const FX_VAPOR_ID: i32 = 128;
/// Upstream `Fx.corrosionVapor` id in `mindustry.content.Fx` for v158.1.
pub const FX_CORROSION_VAPOR_ID: i32 = 127;
/// Upstream `Fx.vaporSmall` id in `mindustry.content.Fx` for v158.1.
pub const FX_VAPOR_SMALL_ID: i32 = 129;
/// Upstream `Fx.fireballsmoke` id in `mindustry.content.Fx` for v158.1.
pub const FX_FIREBALL_SMOKE_ID: i32 = 130;
/// Upstream `Fx.ballfire` id in `mindustry.content.Fx` for v158.1.
pub const FX_BALLFIRE_ID: i32 = 131;
/// Upstream `Fx.freezing` id in `mindustry.content.Fx` for v158.1.
pub const FX_FREEZING_ID: i32 = 132;
/// Upstream `Fx.melting` id in `mindustry.content.Fx` for v158.1.
pub const FX_MELTING_ID: i32 = 133;
/// Upstream `Fx.wet` id in `mindustry.content.Fx` for v158.1.
pub const FX_WET_ID: i32 = 134;
/// Upstream `Fx.muddy` id in `mindustry.content.Fx` for v158.1.
pub const FX_MUDDY_ID: i32 = 135;
/// Upstream `Fx.sapped` id in `mindustry.content.Fx` for v158.1.
pub const FX_SAPPED_ID: i32 = 136;
/// Upstream `Fx.electrified` id in `mindustry.content.Fx` for v158.1.
pub const FX_ELECTRIFIED_ID: i32 = 137;
/// Upstream `Fx.sporeSlowed` id in `mindustry.content.Fx` for v158.1.
pub const FX_SPORE_SLOWED_ID: i32 = 138;
/// Upstream `Fx.oily` id in `mindustry.content.Fx` for v158.1.
pub const FX_OILY_ID: i32 = 139;
/// Upstream `Fx.overdriven` id in `mindustry.content.Fx` for v158.1.
pub const FX_OVERDRIVEN_ID: i32 = 140;
/// Upstream `Fx.overclocked` id in `mindustry.content.Fx` for v158.1.
pub const FX_OVERCLOCKED_ID: i32 = 141;
/// Upstream `Fx.shockwave` id in `mindustry.content.Fx` for v158.1.
pub const FX_SHOCKWAVE_ID: i32 = 143;
/// Upstream `Fx.shockwaveSmaller` id in `mindustry.content.Fx` for v158.1.
pub const FX_SHOCKWAVE_SMALLER_ID: i32 = 144;
/// Upstream `Fx.bigShockwave` id in `mindustry.content.Fx` for v158.1.
pub const FX_BIG_SHOCKWAVE_ID: i32 = 145;
/// Upstream `Fx.spawnShockwave` id in `mindustry.content.Fx` for v158.1.
pub const FX_SPAWN_SHOCKWAVE_ID: i32 = 146;
/// Upstream `Fx.podLandShockwave` id in `mindustry.content.Fx` for v158.1.
pub const FX_POD_LAND_SHOCKWAVE_ID: i32 = 147;
/// Upstream `Fx.blockExplosionSmoke` id in `mindustry.content.Fx` for v158.1.
pub const FX_BLOCK_EXPLOSION_SMOKE_ID: i32 = 152;
/// Upstream `Fx.steamCoolSmoke` id in `mindustry.content.Fx` for v158.1.
pub const FX_STEAM_COOL_SMOKE_ID: i32 = 153;
/// Upstream `Fx.smokePuff` id in `mindustry.content.Fx` for v158.1.
pub const FX_SMOKE_PUFF_ID: i32 = 154;
/// Upstream `Fx.shootSmallSmoke` id in `mindustry.content.Fx` for v158.1.
pub const FX_SHOOT_SMALL_SMOKE_ID: i32 = 159;
/// Upstream `Fx.shootBigSmoke` id in `mindustry.content.Fx` for v158.1.
pub const FX_SHOOT_BIG_SMOKE_ID: i32 = 166;
/// Upstream `Fx.shootBigSmoke2` id in `mindustry.content.Fx` for v158.1.
pub const FX_SHOOT_BIG_SMOKE2_ID: i32 = 167;
/// Upstream `Fx.shootSmokeDisperse` id in `mindustry.content.Fx` for v158.1.
pub const FX_SHOOT_SMOKE_DISPERSE_ID: i32 = 168;
/// Upstream `Fx.smokeCloud` id in `mindustry.content.Fx` for v158.1.
pub const FX_SMOKE_CLOUD_ID: i32 = 222;
/// Upstream `Fx.blastsmoke` id in `mindustry.content.Fx` for v158.1.
pub const FX_BLAST_SMOKE_ID: i32 = 226;
/// Upstream `Fx.ripple` id in `mindustry.content.Fx` for v158.1.
pub const FX_RIPPLE_ID: i32 = 244;
/// Upstream `Fx.bubble` id in `mindustry.content.Fx` for v158.1.
pub const FX_BUBBLE_ID: i32 = 245;
/// Upstream `Fx.launchAccelerator` id in `mindustry.content.Fx` for v158.1.
pub const FX_LAUNCH_ACCELERATOR_ID: i32 = 246;
/// Upstream `Fx.launch` id in `mindustry.content.Fx` for v158.1.
pub const FX_LAUNCH_ID: i32 = 247;
/// Upstream `Fx.healWaveMend` id in `mindustry.content.Fx` for v158.1.
pub const FX_HEAL_WAVE_MEND_ID: i32 = 249;
/// Upstream `Fx.overdriveWave` id in `mindustry.content.Fx` for v158.1.
pub const FX_OVERDRIVE_WAVE_ID: i32 = 250;
/// Upstream `Fx.healBlock` id in `mindustry.content.Fx` for v158.1.
pub const FX_HEAL_BLOCK_ID: i32 = 251;
/// Upstream `Fx.rotateBlock` id in `mindustry.content.Fx` for v158.1.
pub const FX_ROTATE_BLOCK_ID: i32 = 253;
/// Upstream `Fx.lightBlock` id in `mindustry.content.Fx` for v158.1.
pub const FX_LIGHT_BLOCK_ID: i32 = 254;
/// Upstream `Fx.overdriveBlockFull` id in `mindustry.content.Fx` for v158.1.
pub const FX_OVERDRIVE_BLOCK_FULL_ID: i32 = 255;

pub fn standard_effect_id(name: &str) -> Option<i32> {
    match name {
        "smoke" => Some(FX_SMOKE_ID),
        "fallSmoke" => Some(FX_FALL_SMOKE_ID),
        "rocketSmoke" => Some(FX_ROCKET_SMOKE_ID),
        "rocketSmokeLarge" => Some(FX_ROCKET_SMOKE_LARGE_ID),
        "magmasmoke" => Some(FX_MAGMA_SMOKE_ID),
        "breakProp" => Some(FX_BREAK_PROP_ID),
        "unitDrop" => Some(FX_UNIT_DROP_ID),
        "unitLand" => Some(FX_UNIT_LAND_ID),
        "unitDust" => Some(FX_UNIT_DUST_ID),
        "unitLandSmall" => Some(FX_UNIT_LAND_SMALL_ID),
        "crawlDust" => Some(FX_CRAWL_DUST_ID),
        "smokeAoeCloud" => Some(FX_SMOKE_AOE_CLOUD_ID),
        "healWaveDynamic" => Some(FX_HEAL_WAVE_DYNAMIC_ID),
        "healWave" => Some(FX_HEAL_WAVE_ID),
        "heal" => Some(FX_HEAL_ID),
        "dynamicWave" => Some(FX_DYNAMIC_WAVE_ID),
        "shieldWave" => Some(FX_SHIELD_WAVE_ID),
        "shieldApply" => Some(FX_SHIELD_APPLY_ID),
        "disperseTrail" => Some(FX_DISPERSE_TRAIL_ID),
        "hitBulletBig" => Some(FX_HIT_BULLET_BIG_ID),
        "hitFlameSmall" => Some(FX_HIT_FLAME_SMALL_ID),
        "hitFlamePlasma" => Some(FX_HIT_FLAME_PLASMA_ID),
        "hitLaserBlast" => Some(FX_HIT_LASER_BLAST_ID),
        "hitEmpSpark" => Some(FX_HIT_EMP_SPARK_ID),
        "hitLancer" => Some(FX_HIT_LANCER_ID),
        "hitLancerLow" => Some(FX_HIT_LANCER_LOW_ID),
        "hitBeam" => Some(FX_HIT_BEAM_ID),
        "hitFlameBeam" => Some(FX_HIT_FLAME_BEAM_ID),
        "hitMeltdown" => Some(FX_HIT_MELTDOWN_ID),
        "hitMeltHeal" => Some(FX_HIT_MELT_HEAL_ID),
        "hitLaser" => Some(FX_HIT_LASER_ID),
        "despawn" => Some(FX_DESPAWN_ID),
        "hitLiquid" => Some(FX_HIT_LIQUID_ID),
        "artilleryTrail" => Some(FX_ARTILLERY_TRAIL_ID),
        "incendTrail" => Some(FX_INCEND_TRAIL_ID),
        "unitAssemble" => Some(FX_UNIT_ASSEMBLE_ID),
        "missileTrail" => Some(FX_MISSILE_TRAIL_ID),
        "missileTrailShort" => Some(FX_MISSILE_TRAIL_SHORT_ID),
        "colorTrail" => Some(FX_COLOR_TRAIL_ID),
        "absorb" => Some(FX_ABSORB_ID),
        "burning" => Some(FX_BURNING_ID),
        "fire" => Some(FX_FIRE_ID),
        "fireHit" => Some(FX_FIRE_HIT_ID),
        "fireSmoke" => Some(FX_FIRE_SMOKE_ID),
        "neoplasmHeal" => Some(FX_NEOPLASM_HEAL_ID),
        "steam" => Some(FX_STEAM_ID),
        "corrosionVapor" => Some(FX_CORROSION_VAPOR_ID),
        "vapor" => Some(FX_VAPOR_ID),
        "vaporSmall" => Some(FX_VAPOR_SMALL_ID),
        "fireballsmoke" => Some(FX_FIREBALL_SMOKE_ID),
        "ballfire" => Some(FX_BALLFIRE_ID),
        "freezing" => Some(FX_FREEZING_ID),
        "melting" => Some(FX_MELTING_ID),
        "wet" => Some(FX_WET_ID),
        "muddy" => Some(FX_MUDDY_ID),
        "sapped" => Some(FX_SAPPED_ID),
        "electrified" => Some(FX_ELECTRIFIED_ID),
        "sporeSlowed" => Some(FX_SPORE_SLOWED_ID),
        "oily" => Some(FX_OILY_ID),
        "overdriven" => Some(FX_OVERDRIVEN_ID),
        "overclocked" => Some(FX_OVERCLOCKED_ID),
        "shockwave" => Some(FX_SHOCKWAVE_ID),
        "shockwaveSmaller" => Some(FX_SHOCKWAVE_SMALLER_ID),
        "bigShockwave" => Some(FX_BIG_SHOCKWAVE_ID),
        "spawnShockwave" => Some(FX_SPAWN_SHOCKWAVE_ID),
        "podLandShockwave" => Some(FX_POD_LAND_SHOCKWAVE_ID),
        "blockExplosionSmoke" => Some(FX_BLOCK_EXPLOSION_SMOKE_ID),
        "steamCoolSmoke" => Some(FX_STEAM_COOL_SMOKE_ID),
        "smokePuff" => Some(FX_SMOKE_PUFF_ID),
        "shootSmallSmoke" => Some(FX_SHOOT_SMALL_SMOKE_ID),
        "shootBigSmoke" => Some(FX_SHOOT_BIG_SMOKE_ID),
        "shootBigSmoke2" => Some(FX_SHOOT_BIG_SMOKE2_ID),
        "shootSmokeDisperse" => Some(FX_SHOOT_SMOKE_DISPERSE_ID),
        "smokeCloud" => Some(FX_SMOKE_CLOUD_ID),
        "blastsmoke" => Some(FX_BLAST_SMOKE_ID),
        "ripple" => Some(FX_RIPPLE_ID),
        "bubble" => Some(FX_BUBBLE_ID),
        "launchAccelerator" => Some(FX_LAUNCH_ACCELERATOR_ID),
        "launch" => Some(FX_LAUNCH_ID),
        "healWaveMend" => Some(FX_HEAL_WAVE_MEND_ID),
        "overdriveWave" => Some(FX_OVERDRIVE_WAVE_ID),
        "healBlock" => Some(FX_HEAL_BLOCK_ID),
        "rotateBlock" => Some(FX_ROTATE_BLOCK_ID),
        "lightBlock" => Some(FX_LIGHT_BLOCK_ID),
        "overdriveBlockFull" => Some(FX_OVERDRIVE_BLOCK_FULL_ID),
        _ => None,
    }
}

pub fn standard_effect(effect_id: i32) -> Option<Effect> {
    let effect = match effect_id {
        FX_SMOKE_ID => Effect::with_lifetime(FX_SMOKE_ID, 100.0, DEFAULT_EFFECT_CLIP),
        FX_FALL_SMOKE_ID => Effect::with_lifetime(FX_FALL_SMOKE_ID, 110.0, DEFAULT_EFFECT_CLIP),
        FX_ROCKET_SMOKE_ID => Effect::with_lifetime(FX_ROCKET_SMOKE_ID, 120.0, DEFAULT_EFFECT_CLIP),
        FX_ROCKET_SMOKE_LARGE_ID => {
            Effect::with_lifetime(FX_ROCKET_SMOKE_LARGE_ID, 220.0, DEFAULT_EFFECT_CLIP)
        }
        FX_MAGMA_SMOKE_ID => Effect::with_lifetime(FX_MAGMA_SMOKE_ID, 110.0, DEFAULT_EFFECT_CLIP),
        FX_BREAK_PROP_ID => {
            Effect::with_lifetime(FX_BREAK_PROP_ID, 23.0, DEFAULT_EFFECT_CLIP).layer(Layer::DEBRIS)
        }
        FX_UNIT_DROP_ID => {
            Effect::with_lifetime(FX_UNIT_DROP_ID, 30.0, DEFAULT_EFFECT_CLIP).layer(Layer::DEBRIS)
        }
        FX_UNIT_LAND_ID => {
            Effect::with_lifetime(FX_UNIT_LAND_ID, 30.0, DEFAULT_EFFECT_CLIP).layer(Layer::DEBRIS)
        }
        FX_UNIT_DUST_ID => {
            Effect::with_lifetime(FX_UNIT_DUST_ID, 30.0, DEFAULT_EFFECT_CLIP).layer(Layer::DEBRIS)
        }
        FX_UNIT_LAND_SMALL_ID => {
            Effect::with_lifetime(FX_UNIT_LAND_SMALL_ID, 30.0, DEFAULT_EFFECT_CLIP)
                .layer(Layer::DEBRIS)
        }
        FX_CRAWL_DUST_ID => {
            Effect::with_lifetime(FX_CRAWL_DUST_ID, 35.0, DEFAULT_EFFECT_CLIP).layer(Layer::DEBRIS)
        }
        FX_SMOKE_AOE_CLOUD_ID => Effect::with_lifetime(FX_SMOKE_AOE_CLOUD_ID, 180.0, 250.0),
        FX_HEAL_WAVE_DYNAMIC_ID => {
            Effect::with_lifetime(FX_HEAL_WAVE_DYNAMIC_ID, 22.0, DEFAULT_EFFECT_CLIP)
        }
        FX_HEAL_WAVE_ID => Effect::with_lifetime(FX_HEAL_WAVE_ID, 22.0, DEFAULT_EFFECT_CLIP),
        FX_HEAL_ID => Effect::with_lifetime(FX_HEAL_ID, 11.0, DEFAULT_EFFECT_CLIP),
        FX_DYNAMIC_WAVE_ID => Effect::with_lifetime(FX_DYNAMIC_WAVE_ID, 22.0, DEFAULT_EFFECT_CLIP),
        FX_SHIELD_WAVE_ID => Effect::with_lifetime(FX_SHIELD_WAVE_ID, 22.0, DEFAULT_EFFECT_CLIP),
        FX_SHIELD_APPLY_ID => Effect::with_lifetime(FX_SHIELD_APPLY_ID, 11.0, DEFAULT_EFFECT_CLIP),
        FX_DISPERSE_TRAIL_ID => {
            Effect::with_lifetime(FX_DISPERSE_TRAIL_ID, 13.0, DEFAULT_EFFECT_CLIP)
        }
        FX_HIT_BULLET_BIG_ID => {
            Effect::with_lifetime(FX_HIT_BULLET_BIG_ID, 13.0, DEFAULT_EFFECT_CLIP)
        }
        FX_HIT_FLAME_SMALL_ID => {
            Effect::with_lifetime(FX_HIT_FLAME_SMALL_ID, 14.0, DEFAULT_EFFECT_CLIP)
        }
        FX_HIT_FLAME_PLASMA_ID => {
            Effect::with_lifetime(FX_HIT_FLAME_PLASMA_ID, 14.0, DEFAULT_EFFECT_CLIP)
        }
        FX_HIT_LASER_BLAST_ID => {
            Effect::with_lifetime(FX_HIT_LASER_BLAST_ID, 12.0, DEFAULT_EFFECT_CLIP)
        }
        FX_HIT_EMP_SPARK_ID => {
            Effect::with_lifetime(FX_HIT_EMP_SPARK_ID, 40.0, DEFAULT_EFFECT_CLIP)
        }
        FX_HIT_LANCER_ID => Effect::with_lifetime(FX_HIT_LANCER_ID, 12.0, DEFAULT_EFFECT_CLIP),
        FX_HIT_LANCER_LOW_ID => {
            Effect::with_lifetime(FX_HIT_LANCER_LOW_ID, 12.0, DEFAULT_EFFECT_CLIP)
        }
        FX_HIT_BEAM_ID => Effect::with_lifetime(FX_HIT_BEAM_ID, 12.0, DEFAULT_EFFECT_CLIP),
        FX_HIT_FLAME_BEAM_ID => {
            Effect::with_lifetime(FX_HIT_FLAME_BEAM_ID, 19.0, DEFAULT_EFFECT_CLIP)
        }
        FX_HIT_MELTDOWN_ID => Effect::with_lifetime(FX_HIT_MELTDOWN_ID, 12.0, DEFAULT_EFFECT_CLIP),
        FX_HIT_MELT_HEAL_ID => {
            Effect::with_lifetime(FX_HIT_MELT_HEAL_ID, 12.0, DEFAULT_EFFECT_CLIP)
        }
        FX_HIT_LASER_ID => Effect::with_lifetime(FX_HIT_LASER_ID, 8.0, DEFAULT_EFFECT_CLIP),
        FX_DESPAWN_ID => Effect::with_lifetime(FX_DESPAWN_ID, 12.0, DEFAULT_EFFECT_CLIP),
        FX_HIT_LIQUID_ID => Effect::with_lifetime(FX_HIT_LIQUID_ID, 16.0, DEFAULT_EFFECT_CLIP),
        FX_ARTILLERY_TRAIL_ID => {
            Effect::with_lifetime(FX_ARTILLERY_TRAIL_ID, 50.0, DEFAULT_EFFECT_CLIP)
                .layer(Layer::BULLET - 0.01)
        }
        FX_INCEND_TRAIL_ID => Effect::with_lifetime(FX_INCEND_TRAIL_ID, 50.0, DEFAULT_EFFECT_CLIP),
        FX_UNIT_ASSEMBLE_ID => {
            Effect::with_lifetime(FX_UNIT_ASSEMBLE_ID, 70.0, DEFAULT_EFFECT_CLIP)
                .layer(Layer::FLYING_UNIT + 5.0)
        }
        FX_MISSILE_TRAIL_ID => {
            Effect::with_lifetime(FX_MISSILE_TRAIL_ID, 50.0, DEFAULT_EFFECT_CLIP)
                .layer(Layer::BULLET - 0.001)
        }
        FX_MISSILE_TRAIL_SHORT_ID => {
            Effect::with_lifetime(FX_MISSILE_TRAIL_SHORT_ID, 22.0, DEFAULT_EFFECT_CLIP)
                .layer(Layer::BULLET - 0.001)
        }
        FX_COLOR_TRAIL_ID => Effect::with_lifetime(FX_COLOR_TRAIL_ID, 50.0, DEFAULT_EFFECT_CLIP),
        FX_ABSORB_ID => Effect::with_lifetime(FX_ABSORB_ID, 12.0, DEFAULT_EFFECT_CLIP),
        FX_BURNING_ID => Effect::with_lifetime(FX_BURNING_ID, 35.0, DEFAULT_EFFECT_CLIP),
        FX_FIRE_ID => Effect::with_lifetime(FX_FIRE_ID, 50.0, DEFAULT_EFFECT_CLIP),
        FX_FIRE_HIT_ID => Effect::with_lifetime(FX_FIRE_HIT_ID, 35.0, DEFAULT_EFFECT_CLIP),
        FX_FIRE_SMOKE_ID => Effect::with_lifetime(FX_FIRE_SMOKE_ID, 35.0, DEFAULT_EFFECT_CLIP),
        FX_NEOPLASM_HEAL_ID => {
            Effect::with_lifetime(FX_NEOPLASM_HEAL_ID, 120.0, DEFAULT_EFFECT_CLIP)
                .follow_parent(true)
                .rot_with_parent(true)
                .layer(Layer::BULLET - 2.0)
        }
        FX_STEAM_ID => Effect::with_lifetime(FX_STEAM_ID, 35.0, DEFAULT_EFFECT_CLIP),
        FX_CORROSION_VAPOR_ID => {
            Effect::with_lifetime(FX_CORROSION_VAPOR_ID, 50.0, DEFAULT_EFFECT_CLIP)
        }
        FX_VAPOR_ID => Effect::with_lifetime(FX_VAPOR_ID, 110.0, DEFAULT_EFFECT_CLIP),
        FX_VAPOR_SMALL_ID => Effect::with_lifetime(FX_VAPOR_SMALL_ID, 50.0, DEFAULT_EFFECT_CLIP),
        FX_FIREBALL_SMOKE_ID => {
            Effect::with_lifetime(FX_FIREBALL_SMOKE_ID, 25.0, DEFAULT_EFFECT_CLIP)
        }
        FX_BALLFIRE_ID => Effect::with_lifetime(FX_BALLFIRE_ID, 25.0, DEFAULT_EFFECT_CLIP),
        FX_FREEZING_ID => Effect::with_lifetime(FX_FREEZING_ID, 40.0, DEFAULT_EFFECT_CLIP),
        FX_MELTING_ID => Effect::with_lifetime(FX_MELTING_ID, 40.0, DEFAULT_EFFECT_CLIP),
        FX_WET_ID => Effect::with_lifetime(FX_WET_ID, 80.0, DEFAULT_EFFECT_CLIP),
        FX_MUDDY_ID => Effect::with_lifetime(FX_MUDDY_ID, 80.0, DEFAULT_EFFECT_CLIP),
        FX_SAPPED_ID => Effect::with_lifetime(FX_SAPPED_ID, 40.0, DEFAULT_EFFECT_CLIP),
        FX_ELECTRIFIED_ID => Effect::with_lifetime(FX_ELECTRIFIED_ID, 40.0, DEFAULT_EFFECT_CLIP),
        FX_SPORE_SLOWED_ID => Effect::with_lifetime(FX_SPORE_SLOWED_ID, 40.0, DEFAULT_EFFECT_CLIP),
        FX_OILY_ID => Effect::with_lifetime(FX_OILY_ID, 42.0, DEFAULT_EFFECT_CLIP),
        FX_OVERDRIVEN_ID => Effect::with_lifetime(FX_OVERDRIVEN_ID, 20.0, DEFAULT_EFFECT_CLIP),
        FX_OVERCLOCKED_ID => Effect::with_lifetime(FX_OVERCLOCKED_ID, 50.0, DEFAULT_EFFECT_CLIP),
        FX_SHOCKWAVE_ID => Effect::with_lifetime(FX_SHOCKWAVE_ID, 10.0, 80.0),
        FX_SHOCKWAVE_SMALLER_ID => Effect::with_lifetime(FX_SHOCKWAVE_SMALLER_ID, 9.0, 80.0),
        FX_BIG_SHOCKWAVE_ID => Effect::with_lifetime(FX_BIG_SHOCKWAVE_ID, 10.0, 80.0),
        FX_SPAWN_SHOCKWAVE_ID => Effect::with_lifetime(FX_SPAWN_SHOCKWAVE_ID, 20.0, 400.0),
        FX_POD_LAND_SHOCKWAVE_ID => Effect::with_lifetime(FX_POD_LAND_SHOCKWAVE_ID, 12.0, 80.0),
        FX_BLOCK_EXPLOSION_SMOKE_ID => {
            Effect::with_lifetime(FX_BLOCK_EXPLOSION_SMOKE_ID, 30.0, DEFAULT_EFFECT_CLIP)
        }
        FX_STEAM_COOL_SMOKE_ID => {
            Effect::with_lifetime(FX_STEAM_COOL_SMOKE_ID, 35.0, DEFAULT_EFFECT_CLIP)
        }
        FX_SMOKE_PUFF_ID => Effect::with_lifetime(FX_SMOKE_PUFF_ID, 30.0, DEFAULT_EFFECT_CLIP),
        FX_SHOOT_SMALL_SMOKE_ID => {
            Effect::with_lifetime(FX_SHOOT_SMALL_SMOKE_ID, 20.0, DEFAULT_EFFECT_CLIP)
        }
        FX_SHOOT_BIG_SMOKE_ID => {
            Effect::with_lifetime(FX_SHOOT_BIG_SMOKE_ID, 17.0, DEFAULT_EFFECT_CLIP)
        }
        FX_SHOOT_BIG_SMOKE2_ID => {
            Effect::with_lifetime(FX_SHOOT_BIG_SMOKE2_ID, 18.0, DEFAULT_EFFECT_CLIP)
        }
        FX_SHOOT_SMOKE_DISPERSE_ID => {
            Effect::with_lifetime(FX_SHOOT_SMOKE_DISPERSE_ID, 25.0, DEFAULT_EFFECT_CLIP)
        }
        FX_SMOKE_CLOUD_ID => Effect::with_lifetime(FX_SMOKE_CLOUD_ID, 70.0, DEFAULT_EFFECT_CLIP),
        FX_BLAST_SMOKE_ID => Effect::with_lifetime(FX_BLAST_SMOKE_ID, 26.0, DEFAULT_EFFECT_CLIP),
        FX_RIPPLE_ID => {
            Effect::with_lifetime(FX_RIPPLE_ID, 30.0, DEFAULT_EFFECT_CLIP).layer(Layer::DEBRIS)
        }
        FX_BUBBLE_ID => Effect::with_lifetime(FX_BUBBLE_ID, 20.0, DEFAULT_EFFECT_CLIP),
        FX_LAUNCH_ACCELERATOR_ID => {
            Effect::with_lifetime(FX_LAUNCH_ACCELERATOR_ID, 22.0, DEFAULT_EFFECT_CLIP)
        }
        FX_LAUNCH_ID => Effect::with_lifetime(FX_LAUNCH_ID, 28.0, DEFAULT_EFFECT_CLIP),
        FX_HEAL_WAVE_MEND_ID => {
            Effect::with_lifetime(FX_HEAL_WAVE_MEND_ID, 40.0, DEFAULT_EFFECT_CLIP)
        }
        FX_OVERDRIVE_WAVE_ID => {
            Effect::with_lifetime(FX_OVERDRIVE_WAVE_ID, 50.0, DEFAULT_EFFECT_CLIP)
        }
        FX_HEAL_BLOCK_ID => Effect::with_lifetime(FX_HEAL_BLOCK_ID, 20.0, DEFAULT_EFFECT_CLIP),
        FX_ROTATE_BLOCK_ID => Effect::with_lifetime(FX_ROTATE_BLOCK_ID, 30.0, DEFAULT_EFFECT_CLIP),
        FX_LIGHT_BLOCK_ID => Effect::with_lifetime(FX_LIGHT_BLOCK_ID, 60.0, DEFAULT_EFFECT_CLIP),
        FX_OVERDRIVE_BLOCK_FULL_ID => {
            Effect::with_lifetime(FX_OVERDRIVE_BLOCK_FULL_ID, 60.0, DEFAULT_EFFECT_CLIP)
        }
        _ => return None,
    };
    Some(effect)
}

pub fn standard_effect_by_name(name: &str) -> Option<Effect> {
    standard_effect_id(name).and_then(standard_effect)
}

pub fn standard_effect_render_lifetime(effect_id: Option<u16>, rotation: f32, current: f32) -> f32 {
    match effect_id.map(i32::from) {
        // Java `Fx.ripple` sets `e.lifetime = 30f * e.rotation` inside the
        // renderer, so it must be applied during `EffectStateComp::draw_with`
        // rather than at static metadata lookup time.
        Some(FX_RIPPLE_ID) => 30.0 * rotation,
        _ => current,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandardEffectDrawKind {
    FilledCircle,
    StrokedCircle,
    SeededCircleParticles,
    SeededStrokedCircleParticles,
    SeededLineParticles,
    SeededRadialLineParticles,
    FilledSquare,
    StrokedSquare,
    SeededSquareParticles,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StandardEffectParticleSpec {
    pub seed: i32,
    pub count: u16,
    pub progress: Option<f32>,
    pub angle: Option<f32>,
    pub angle_range: f32,
    pub length: f32,
    pub fin: f32,
    pub fout: f32,
    pub fslope: f32,
    pub radius_base: f32,
    pub radius_fin_scale: f32,
    pub radius_fout_scale: f32,
    pub radius_fslope_scale: f32,
    pub secondary_vector_scale: f32,
    pub secondary_radius_base: f32,
    pub secondary_radius_fin_scale: f32,
    pub secondary_radius_fout_scale: f32,
    pub secondary_radius_fslope_scale: f32,
    pub alpha_midpoint: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StandardEffectParticleVector {
    pub x: f32,
    pub y: f32,
    pub fin: f32,
    pub fout: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StandardEffectCirclePrimitive {
    pub center: (f32, f32),
    pub radius: f32,
    pub alpha: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StandardEffectCircleRenderPrimitive {
    pub kind: StandardEffectDrawKind,
    pub center: (f32, f32),
    pub radius: f32,
    pub stroke: f32,
    pub alpha: f32,
    pub color: Option<DecalColor>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StandardEffectSquareRenderPrimitive {
    pub center: (f32, f32),
    pub radius: f32,
    pub stroke: f32,
    pub rotation: f32,
    pub alpha: f32,
    pub color: Option<DecalColor>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StandardEffectLineRenderPrimitive {
    pub start: (f32, f32),
    pub angle: f32,
    pub length: f32,
    pub stroke: f32,
    pub alpha: f32,
    pub color: Option<DecalColor>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StandardEffectLightRenderPrimitive {
    pub center: (f32, f32),
    pub radius: f32,
    pub color: &'static str,
    pub color_rgba: Option<DecalColor>,
    pub opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StandardEffectDrawPlan {
    pub effect_id: i32,
    pub layer: f32,
    pub kind: StandardEffectDrawKind,
    pub center: (f32, f32),
    pub color_from: Option<&'static str>,
    pub color_mid: Option<&'static str>,
    pub color_to: Option<&'static str>,
    pub color_mix: f32,
    pub input_color: Option<DecalColor>,
    pub color_mul: f32,
    pub alpha: f32,
    pub radius: f32,
    pub stroke: f32,
    pub particles: Option<StandardEffectParticleSpec>,
    pub light_color: Option<&'static str>,
    pub light_radius: f32,
    pub light_opacity: f32,
}

impl StandardEffectDrawPlan {
    pub fn seeded_particle_vectors(&self) -> Vec<StandardEffectParticleVector> {
        self.particles
            .map(|particles| particles.seeded_vectors())
            .unwrap_or_default()
    }

    pub fn expand_seeded_particle_circles(
        &self,
        vectors: &[StandardEffectParticleVector],
    ) -> Vec<StandardEffectCirclePrimitive> {
        let Some(particles) = self.particles else {
            return Vec::new();
        };

        let has_secondary_circle = particles.secondary_vector_scale != 0.0
            || particles.secondary_radius_base != 0.0
            || particles.secondary_radius_fin_scale != 0.0
            || particles.secondary_radius_fout_scale != 0.0
            || particles.secondary_radius_fslope_scale != 0.0;
        let mut circles = Vec::with_capacity(
            particles.count as usize
                * if has_secondary_circle {
                    2_usize
                } else {
                    1_usize
                },
        );

        for vector in vectors.iter().take(particles.count as usize) {
            let (fin, fout, fslope) = if particles.progress.is_some() {
                (
                    vector.fin.clamp(0.0, 1.0),
                    vector.fout.clamp(0.0, 1.0),
                    effect_fslope_from_fin(vector.fin),
                )
            } else {
                (particles.fin, particles.fout, particles.fslope)
            };
            let radius = particles.radius_base
                + particles.radius_fin_scale * fin
                + particles.radius_fout_scale * fout
                + particles.radius_fslope_scale * fslope;
            let alpha = if particles.alpha_midpoint {
                self.alpha * effect_fslope_from_fin(fin)
            } else {
                self.alpha
            };
            circles.push(StandardEffectCirclePrimitive {
                center: (self.center.0 + vector.x, self.center.1 + vector.y),
                radius,
                alpha,
            });

            if has_secondary_circle {
                let radius = particles.secondary_radius_base
                    + particles.secondary_radius_fin_scale * fin
                    + particles.secondary_radius_fout_scale * fout
                    + particles.secondary_radius_fslope_scale * fslope;
                circles.push(StandardEffectCirclePrimitive {
                    center: (
                        self.center.0 + vector.x * particles.secondary_vector_scale,
                        self.center.1 + vector.y * particles.secondary_vector_scale,
                    ),
                    radius,
                    alpha,
                });
            }
        }

        circles
    }

    pub fn expand_seeded_particle_circles_from_seed(&self) -> Vec<StandardEffectCirclePrimitive> {
        let vectors = self.seeded_particle_vectors();
        self.expand_seeded_particle_circles(&vectors)
    }

    pub fn circle_render_primitives_from_seed(&self) -> Vec<StandardEffectCircleRenderPrimitive> {
        let color = self.resolved_draw_color();
        match self.kind {
            StandardEffectDrawKind::FilledCircle | StandardEffectDrawKind::StrokedCircle => {
                vec![StandardEffectCircleRenderPrimitive {
                    kind: self.kind,
                    center: self.center,
                    radius: self.radius,
                    stroke: self.stroke,
                    alpha: self.alpha,
                    color,
                }]
            }
            StandardEffectDrawKind::SeededCircleParticles => self
                .expand_seeded_particle_circles_from_seed()
                .into_iter()
                .map(|circle| StandardEffectCircleRenderPrimitive {
                    kind: StandardEffectDrawKind::FilledCircle,
                    center: circle.center,
                    radius: circle.radius,
                    stroke: 0.0,
                    alpha: circle.alpha,
                    color,
                })
                .collect(),
            StandardEffectDrawKind::SeededStrokedCircleParticles => self
                .expand_seeded_particle_circles_from_seed()
                .into_iter()
                .map(|circle| StandardEffectCircleRenderPrimitive {
                    kind: StandardEffectDrawKind::StrokedCircle,
                    center: circle.center,
                    radius: circle.radius,
                    stroke: self.stroke,
                    alpha: circle.alpha,
                    color,
                })
                .collect(),
            StandardEffectDrawKind::SeededLineParticles
            | StandardEffectDrawKind::SeededRadialLineParticles => Vec::new(),
            StandardEffectDrawKind::FilledSquare
            | StandardEffectDrawKind::StrokedSquare
            | StandardEffectDrawKind::SeededSquareParticles => Vec::new(),
        }
    }

    pub fn square_render_primitives_from_seed(&self) -> Vec<StandardEffectSquareRenderPrimitive> {
        let color = self.resolved_draw_color();
        match self.kind {
            StandardEffectDrawKind::FilledSquare => {
                vec![StandardEffectSquareRenderPrimitive {
                    center: self.center,
                    radius: self.radius,
                    stroke: 0.0,
                    rotation: self.stroke,
                    alpha: self.alpha,
                    color,
                }]
            }
            StandardEffectDrawKind::StrokedSquare => {
                vec![StandardEffectSquareRenderPrimitive {
                    center: self.center,
                    radius: self.radius,
                    stroke: self.stroke,
                    rotation: 0.0,
                    alpha: self.alpha,
                    color,
                }]
            }
            StandardEffectDrawKind::SeededSquareParticles => self
                .expand_seeded_particle_circles_from_seed()
                .into_iter()
                .map(|square| StandardEffectSquareRenderPrimitive {
                    center: square.center,
                    radius: square.radius,
                    stroke: 0.0,
                    rotation: self.stroke,
                    alpha: square.alpha,
                    color,
                })
                .collect(),
            StandardEffectDrawKind::FilledCircle
            | StandardEffectDrawKind::StrokedCircle
            | StandardEffectDrawKind::SeededCircleParticles
            | StandardEffectDrawKind::SeededStrokedCircleParticles
            | StandardEffectDrawKind::SeededLineParticles
            | StandardEffectDrawKind::SeededRadialLineParticles => Vec::new(),
        }
    }

    pub fn line_render_primitives_from_seed(&self) -> Vec<StandardEffectLineRenderPrimitive> {
        let Some(particles) = self.particles else {
            return Vec::new();
        };
        let color = self.resolved_draw_color();
        match self.kind {
            StandardEffectDrawKind::SeededLineParticles => {
                let mut rand = ArcRand::with_seed(particles.seed as i64);
                let mut lines = Vec::with_capacity(particles.count as usize);
                for _ in 0..particles.count {
                    let angle = particles.angle.unwrap_or(0.0) + rand.range(particles.angle_range);
                    let length = rand.random(particles.length);
                    let (x, y) = trns(angle, length);
                    lines.push(StandardEffectLineRenderPrimitive {
                        start: (self.center.0 + x, self.center.1 + y),
                        angle,
                        length: self.radius + particles.fout * rand.random_between(2.0, 7.0),
                        stroke: self.stroke,
                        alpha: self.alpha,
                        color,
                    });
                }
                lines
            }
            StandardEffectDrawKind::SeededRadialLineParticles => self
                .seeded_particle_vectors()
                .into_iter()
                .map(|vector| StandardEffectLineRenderPrimitive {
                    start: (self.center.0 + vector.x, self.center.1 + vector.y),
                    angle: vector.y.atan2(vector.x).to_degrees(),
                    length: self.radius
                        + particles.radius_base
                        + particles.radius_fin_scale * vector.fin
                        + particles.radius_fout_scale * vector.fout
                        + particles.radius_fslope_scale * particles.fslope,
                    stroke: self.stroke,
                    alpha: self.alpha,
                    color,
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    pub fn light_render_primitives(&self) -> Vec<StandardEffectLightRenderPrimitive> {
        self.light_color
            .filter(|_| self.light_radius > 0.0 && self.light_opacity > 0.0)
            .map(|color| {
                vec![StandardEffectLightRenderPrimitive {
                    center: self.center,
                    radius: self.light_radius,
                    color,
                    color_rgba: standard_effect_color_symbol(color),
                    opacity: self.light_opacity,
                }]
            })
            .unwrap_or_default()
    }

    pub fn resolved_draw_color(&self) -> Option<DecalColor> {
        let mut color = match (
            self.input_color,
            self.color_from,
            self.color_mid,
            self.color_to,
        ) {
            (Some(color), _, _, _) => color,
            (None, Some(from), Some(mid), Some(to)) => lerp_color_three(
                standard_effect_color_symbol(from)?,
                standard_effect_color_symbol(mid)?,
                standard_effect_color_symbol(to)?,
                self.color_mix,
            ),
            (None, Some(from), None, Some(to)) => lerp_color(
                standard_effect_color_symbol(from)?,
                standard_effect_color_symbol(to)?,
                self.color_mix,
            ),
            (None, Some(from), None, None) => standard_effect_color_symbol(from)?,
            _ => return None,
        };

        color.r *= self.color_mul;
        color.g *= self.color_mul;
        color.b *= self.color_mul;
        color.a *= self.alpha;
        Some(color)
    }
}

impl StandardEffectParticleSpec {
    pub fn seeded_vectors(&self) -> Vec<StandardEffectParticleVector> {
        let mut rand = ArcRand::with_seed(self.seed as i64);
        let count = self.count as usize;
        let mut vectors = Vec::with_capacity(count);

        for _ in 0..count {
            let (x, y, fin, fout) = if let Some(progress) = self.progress {
                let local = rand.next_float();
                let angle = rand.random(360.0);
                let length = self.length * local * progress;
                let (x, y) = trns(angle, length);
                (x, y, progress * local, (1.0 - progress) * local)
            } else {
                let angle = self
                    .angle
                    .map(|angle| angle + rand.range(self.angle_range))
                    .unwrap_or_else(|| rand.random(360.0));
                let length = rand.random(self.length);
                let (x, y) = trns(angle, length);
                (x, y, self.fin, self.fout)
            };

            vectors.push(StandardEffectParticleVector { x, y, fin, fout });
        }

        vectors
    }
}

pub fn standard_effect_draw_plan(
    effect_id: Option<u16>,
    state_id: i32,
    x: f32,
    y: f32,
    rotation: f32,
    time: f32,
    lifetime: f32,
    color: DecalColor,
) -> Option<StandardEffectDrawPlan> {
    let effect_id = effect_id.map(i32::from)?;
    let effect = standard_effect(effect_id)?;
    let lifetime = standard_effect_render_lifetime(Some(effect_id as u16), rotation, lifetime);
    let fin = if lifetime.abs() <= f32::EPSILON {
        1.0
    } else {
        (time / lifetime).clamp(0.0, 1.0)
    };
    let fout = 1.0 - fin;
    let finpow = effect_finpow_from_fin(fin);
    let fslope = effect_fslope_from_fin(fin);
    let rocket_smoke_alpha = (fout * 1.6 - rotation.powi(3) * 1.2).clamp(0.0, 1.0);

    let plan = match effect_id {
        FX_SMOKE_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::FilledCircle,
            center: (x, y),
            color_from: Some("Color.gray"),
            color_mid: None,
            color_to: Some("Pal.darkishGray"),
            color_mix: fin,
            input_color: None,
            color_mul: 1.0,
            alpha: 1.0,
            radius: (7.0 - 7.0 * fin) / 2.0,
            stroke: 0.0,
            particles: None,
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_FALL_SMOKE_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::FilledCircle,
            center: (x, y),
            color_from: Some("Color.gray"),
            color_mid: None,
            color_to: Some("Color.darkGray"),
            color_mix: rotation,
            input_color: None,
            color_mul: 1.0,
            alpha: 1.0,
            radius: fout * 3.5,
            stroke: 0.0,
            particles: None,
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_ROCKET_SMOKE_ID | FX_ROCKET_SMOKE_LARGE_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::FilledCircle,
            center: (x, y),
            color_from: Some("Color.gray"),
            color_mid: None,
            color_to: None,
            color_mix: 0.0,
            input_color: None,
            color_mul: 1.0,
            alpha: rocket_smoke_alpha,
            radius: if effect_id == FX_ROCKET_SMOKE_LARGE_ID {
                1.0 + 6.0 * rotation * 1.3 - fin * 2.0
            } else {
                1.0 + 6.0 * rotation - fin * 2.0
            },
            stroke: 0.0,
            particles: None,
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_MAGMA_SMOKE_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::FilledCircle,
            center: (x, y),
            color_from: Some("Color.gray"),
            color_mid: None,
            color_to: None,
            color_mix: 0.0,
            input_color: None,
            color_mul: 1.0,
            alpha: 1.0,
            radius: fslope * 6.0,
            stroke: 0.0,
            particles: None,
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_BREAK_PROP_ID
        | FX_UNIT_DROP_ID
        | FX_UNIT_LAND_ID
        | FX_UNIT_DUST_ID
        | FX_UNIT_LAND_SMALL_ID
        | FX_CRAWL_DUST_ID => {
            let break_scl = rotation.max(1.0);
            let unit_land_small_count = (6.0 * rotation).max(0.0) as u16;
            let (
                color_from,
                input_color,
                color_mul,
                count,
                angle,
                angle_range,
                length,
                radius_base,
                radius_fout_scale,
                radius_fslope_scale,
            ) = match effect_id {
                FX_BREAK_PROP_ID => (
                    None,
                    Some(color),
                    1.1,
                    6,
                    None,
                    0.0,
                    19.0 * finpow * break_scl,
                    0.3,
                    3.5 * break_scl,
                    0.0,
                ),
                FX_UNIT_DROP_ID => (
                    Some("Pal.lightishGray"),
                    None,
                    1.0,
                    9,
                    None,
                    0.0,
                    3.0 + 20.0 * finpow,
                    0.4,
                    4.0,
                    0.0,
                ),
                FX_UNIT_LAND_ID => (
                    None,
                    Some(color),
                    1.1,
                    6,
                    None,
                    0.0,
                    17.0 * finpow,
                    0.3,
                    4.0,
                    0.0,
                ),
                FX_UNIT_DUST_ID => (
                    None,
                    Some(color),
                    1.3,
                    3,
                    Some(rotation),
                    30.0,
                    8.0 * finpow,
                    0.3,
                    3.0,
                    0.0,
                ),
                FX_UNIT_LAND_SMALL_ID => (
                    None,
                    Some(color),
                    1.1,
                    unit_land_small_count,
                    None,
                    0.0,
                    12.0 * finpow * rotation,
                    0.1,
                    3.0,
                    0.0,
                ),
                FX_CRAWL_DUST_ID => (
                    None,
                    Some(color),
                    1.6,
                    2,
                    None,
                    0.0,
                    10.0 * finpow,
                    0.3,
                    0.0,
                    4.0,
                ),
                _ => unreachable!(),
            };

            StandardEffectDrawPlan {
                effect_id,
                layer: effect.layer,
                kind: StandardEffectDrawKind::SeededCircleParticles,
                center: (x, y),
                color_from,
                color_mid: None,
                color_to: None,
                color_mix: 0.0,
                input_color,
                color_mul,
                alpha: 1.0,
                radius: 0.0,
                stroke: 0.0,
                particles: Some(StandardEffectParticleSpec {
                    seed: state_id,
                    count,
                    progress: None,
                    angle,
                    angle_range,
                    length,
                    fin,
                    fout,
                    fslope,
                    radius_base,
                    radius_fin_scale: 0.0,
                    radius_fout_scale,
                    radius_fslope_scale,
                    secondary_vector_scale: 0.0,
                    secondary_radius_base: 0.0,
                    secondary_radius_fin_scale: 0.0,
                    secondary_radius_fout_scale: 0.0,
                    secondary_radius_fslope_scale: 0.0,
                    alpha_midpoint: false,
                }),
                light_color: None,
                light_radius: 0.0,
                light_opacity: 0.0,
            }
        }
        FX_SMOKE_AOE_CLOUD_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::SeededCircleParticles,
            center: (x, y),
            color_from: None,
            color_mid: None,
            color_to: None,
            color_mix: 0.0,
            input_color: Some(color),
            color_mul: 1.0,
            alpha: 0.65,
            radius: 0.0,
            stroke: 0.0,
            particles: Some(StandardEffectParticleSpec {
                seed: state_id,
                count: 80,
                progress: None,
                angle: None,
                angle_range: 0.0,
                length: 90.0,
                fin,
                fout,
                fslope,
                radius_base: 6.0 * (fin / 0.1).clamp(0.0, 1.0) * (fout / 0.1).clamp(0.0, 1.0),
                radius_fin_scale: 0.0,
                radius_fout_scale: 0.0,
                radius_fslope_scale: 0.0,
                secondary_vector_scale: 0.0,
                secondary_radius_base: 0.0,
                secondary_radius_fin_scale: 0.0,
                secondary_radius_fout_scale: 0.0,
                secondary_radius_fslope_scale: 0.0,
                alpha_midpoint: false,
            }),
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_HEAL_WAVE_DYNAMIC_ID
        | FX_HEAL_WAVE_ID
        | FX_HEAL_ID
        | FX_DYNAMIC_WAVE_ID
        | FX_SHIELD_WAVE_ID
        | FX_SHIELD_APPLY_ID => {
            let (color_from, input_color, alpha, radius, stroke) = match effect_id {
                FX_HEAL_WAVE_DYNAMIC_ID => (
                    Some("Pal.heal"),
                    None,
                    1.0,
                    4.0 + finpow * rotation,
                    fout * 2.0,
                ),
                FX_HEAL_WAVE_ID => (Some("Pal.heal"), None, 1.0, 4.0 + finpow * 60.0, fout * 2.0),
                FX_HEAL_ID => (Some("Pal.heal"), None, 1.0, 2.0 + finpow * 7.0, fout * 2.0),
                FX_DYNAMIC_WAVE_ID => (None, Some(color), 0.7, 4.0 + finpow * rotation, fout * 2.0),
                FX_SHIELD_WAVE_ID => (None, Some(color), 0.7, 4.0 + finpow * 60.0, fout * 2.0),
                FX_SHIELD_APPLY_ID => (None, Some(color), 0.7, 2.0 + finpow * 7.0, fout * 2.0),
                _ => unreachable!(),
            };

            StandardEffectDrawPlan {
                effect_id,
                layer: effect.layer,
                kind: StandardEffectDrawKind::StrokedCircle,
                center: (x, y),
                color_from,
                color_mid: None,
                color_to: None,
                color_mix: 0.0,
                input_color,
                color_mul: 1.0,
                alpha,
                radius,
                stroke,
                particles: None,
                light_color: None,
                light_radius: 0.0,
                light_opacity: 0.0,
            }
        }
        FX_DISPERSE_TRAIL_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::SeededLineParticles,
            center: (x, y),
            color_from: None,
            color_mid: None,
            color_to: None,
            color_mix: 0.0,
            input_color: Some(lerp_color(DecalColor::WHITE, color, fin)),
            color_mul: 1.0,
            alpha: 1.0,
            radius: 1.5,
            stroke: 0.6 + fout * 1.7,
            particles: Some(StandardEffectParticleSpec {
                seed: state_id,
                count: 2,
                progress: None,
                angle: Some(rotation + 180.0),
                angle_range: 15.0,
                length: fin * 27.0,
                fin,
                fout,
                fslope,
                radius_base: 0.0,
                radius_fin_scale: 0.0,
                radius_fout_scale: 0.0,
                radius_fslope_scale: 0.0,
                secondary_vector_scale: 0.0,
                secondary_radius_base: 0.0,
                secondary_radius_fin_scale: 0.0,
                secondary_radius_fout_scale: 0.0,
                secondary_radius_fslope_scale: 0.0,
                alpha_midpoint: false,
            }),
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_HIT_BULLET_BIG_ID | FX_HIT_FLAME_SMALL_ID | FX_HIT_FLAME_PLASMA_ID => {
            let (color_from, color_to, count, length, stroke, line_base, line_fout_scale) =
                match effect_id {
                    FX_HIT_BULLET_BIG_ID => (
                        "Color.white",
                        "Pal.lightOrange",
                        8,
                        finpow * 30.0,
                        0.5 + fout * 1.5,
                        1.5,
                        4.0,
                    ),
                    FX_HIT_FLAME_SMALL_ID => (
                        "Pal.lightFlame",
                        "Pal.darkFlame",
                        2,
                        1.0 + fin * 15.0,
                        0.5 + fout,
                        1.0,
                        3.0,
                    ),
                    FX_HIT_FLAME_PLASMA_ID => (
                        "Color.white",
                        "Pal.heal",
                        2,
                        1.0 + fin * 15.0,
                        0.5 + fout,
                        1.0,
                        3.0,
                    ),
                    _ => unreachable!(),
                };

            StandardEffectDrawPlan {
                effect_id,
                layer: effect.layer,
                kind: StandardEffectDrawKind::SeededRadialLineParticles,
                center: (x, y),
                color_from: Some(color_from),
                color_mid: None,
                color_to: Some(color_to),
                color_mix: fin,
                input_color: None,
                color_mul: 1.0,
                alpha: 1.0,
                radius: line_base,
                stroke,
                particles: Some(StandardEffectParticleSpec {
                    seed: state_id,
                    count,
                    progress: None,
                    angle: Some(rotation),
                    angle_range: 50.0,
                    length,
                    fin,
                    fout,
                    fslope,
                    radius_base: 0.0,
                    radius_fin_scale: 0.0,
                    radius_fout_scale: line_fout_scale,
                    radius_fslope_scale: 0.0,
                    secondary_vector_scale: 0.0,
                    secondary_radius_base: 0.0,
                    secondary_radius_fin_scale: 0.0,
                    secondary_radius_fout_scale: 0.0,
                    secondary_radius_fslope_scale: 0.0,
                    alpha_midpoint: false,
                }),
                light_color: None,
                light_radius: 0.0,
                light_opacity: 0.0,
            }
        }
        FX_HIT_LASER_BLAST_ID
        | FX_HIT_EMP_SPARK_ID
        | FX_HIT_LANCER_ID
        | FX_HIT_LANCER_LOW_ID
        | FX_HIT_BEAM_ID
        | FX_HIT_MELTDOWN_ID
        | FX_HIT_MELT_HEAL_ID => {
            let (
                color_from,
                input_color,
                count,
                length,
                angle,
                angle_range,
                stroke,
                line_fout_scale,
            ) = match effect_id {
                FX_HIT_LASER_BLAST_ID => (
                    None,
                    Some(color),
                    8,
                    finpow * 17.0,
                    None,
                    0.0,
                    fout * 1.5,
                    4.0,
                ),
                FX_HIT_EMP_SPARK_ID => (
                    Some("Pal.heal"),
                    None,
                    18,
                    finpow * 27.0,
                    Some(rotation),
                    360.0,
                    fout * 1.6,
                    6.0,
                ),
                FX_HIT_LANCER_ID => (
                    Some("Color.white"),
                    None,
                    8,
                    finpow * 17.0,
                    None,
                    0.0,
                    fout * 1.5,
                    4.0,
                ),
                FX_HIT_LANCER_LOW_ID => (
                    Some("Color.white"),
                    None,
                    4,
                    finpow * 17.0,
                    None,
                    0.0,
                    fout * 1.5,
                    4.0,
                ),
                FX_HIT_BEAM_ID => (
                    None,
                    Some(color),
                    6,
                    finpow * 18.0,
                    None,
                    0.0,
                    fout * 2.0,
                    4.0,
                ),
                FX_HIT_MELTDOWN_ID => (
                    Some("Pal.meltdownHit"),
                    None,
                    6,
                    finpow * 18.0,
                    None,
                    0.0,
                    fout * 2.0,
                    4.0,
                ),
                FX_HIT_MELT_HEAL_ID => (
                    Some("Pal.heal"),
                    None,
                    6,
                    finpow * 18.0,
                    None,
                    0.0,
                    fout * 2.0,
                    4.0,
                ),
                _ => unreachable!(),
            };

            StandardEffectDrawPlan {
                effect_id,
                layer: effect.layer,
                kind: StandardEffectDrawKind::SeededRadialLineParticles,
                center: (x, y),
                color_from,
                color_mid: None,
                color_to: None,
                color_mix: 0.0,
                input_color,
                color_mul: 1.0,
                alpha: 1.0,
                radius: 1.0,
                stroke,
                particles: Some(StandardEffectParticleSpec {
                    seed: state_id,
                    count,
                    progress: None,
                    angle,
                    angle_range,
                    length,
                    fin,
                    fout,
                    fslope,
                    radius_base: 0.0,
                    radius_fin_scale: 0.0,
                    radius_fout_scale: line_fout_scale,
                    radius_fslope_scale: 0.0,
                    secondary_vector_scale: 0.0,
                    secondary_radius_base: 0.0,
                    secondary_radius_fin_scale: 0.0,
                    secondary_radius_fout_scale: 0.0,
                    secondary_radius_fslope_scale: 0.0,
                    alpha_midpoint: false,
                }),
                light_color: None,
                light_radius: 0.0,
                light_opacity: 0.0,
            }
        }
        FX_HIT_FLAME_BEAM_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::SeededCircleParticles,
            center: (x, y),
            color_from: None,
            color_mid: None,
            color_to: None,
            color_mix: 0.0,
            input_color: Some(color),
            color_mul: 1.0,
            alpha: 1.0,
            radius: 0.0,
            stroke: 0.0,
            particles: Some(StandardEffectParticleSpec {
                seed: state_id,
                count: 7,
                progress: None,
                angle: None,
                angle_range: 0.0,
                length: finpow * 11.0,
                fin,
                fout,
                fslope,
                radius_base: 0.5,
                radius_fin_scale: 0.0,
                radius_fout_scale: 2.0,
                radius_fslope_scale: 0.0,
                secondary_vector_scale: 0.0,
                secondary_radius_base: 0.0,
                secondary_radius_fin_scale: 0.0,
                secondary_radius_fout_scale: 0.0,
                secondary_radius_fslope_scale: 0.0,
                alpha_midpoint: false,
            }),
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_HIT_LASER_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::StrokedCircle,
            center: (x, y),
            color_from: Some("Color.white"),
            color_mid: None,
            color_to: Some("Pal.heal"),
            color_mix: fin,
            input_color: None,
            color_mul: 1.0,
            alpha: 1.0,
            radius: fin * 5.0,
            stroke: 0.5 + fout,
            particles: None,
            light_color: Some("Pal.heal"),
            light_radius: 23.0,
            light_opacity: fout * 0.7,
        },
        FX_DESPAWN_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::SeededRadialLineParticles,
            center: (x, y),
            color_from: Some("Pal.lighterOrange"),
            color_mid: None,
            color_to: Some("Color.gray"),
            color_mix: fin,
            input_color: None,
            color_mul: 1.0,
            alpha: 1.0,
            radius: 1.0,
            stroke: fout,
            particles: Some(StandardEffectParticleSpec {
                seed: state_id,
                count: 7,
                progress: None,
                angle: Some(rotation),
                angle_range: 40.0,
                length: fin * 7.0,
                fin,
                fout,
                fslope,
                radius_base: 0.0,
                radius_fin_scale: 0.0,
                radius_fout_scale: 2.0,
                radius_fslope_scale: 0.0,
                secondary_vector_scale: 0.0,
                secondary_radius_base: 0.0,
                secondary_radius_fin_scale: 0.0,
                secondary_radius_fout_scale: 0.0,
                secondary_radius_fslope_scale: 0.0,
                alpha_midpoint: false,
            }),
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_ARTILLERY_TRAIL_ID
        | FX_MISSILE_TRAIL_ID
        | FX_MISSILE_TRAIL_SHORT_ID
        | FX_COLOR_TRAIL_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::FilledCircle,
            center: (x, y),
            color_from: None,
            color_mid: None,
            color_to: None,
            color_mix: 0.0,
            input_color: Some(color),
            color_mul: 1.0,
            alpha: 1.0,
            radius: rotation * fout,
            stroke: 0.0,
            particles: None,
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_INCEND_TRAIL_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::FilledCircle,
            center: (x, y),
            color_from: Some("Pal.lightOrange"),
            color_mid: None,
            color_to: None,
            color_mix: 0.0,
            input_color: None,
            color_mul: 1.0,
            alpha: 1.0,
            radius: rotation * fout,
            stroke: 0.0,
            particles: None,
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_ABSORB_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::StrokedCircle,
            center: (x, y),
            color_from: Some("Pal.accent"),
            color_mid: None,
            color_to: None,
            color_mix: 0.0,
            input_color: None,
            color_mul: 1.0,
            alpha: 1.0,
            radius: 5.0 * fout,
            stroke: 2.0 * fout,
            particles: None,
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_HIT_LIQUID_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::SeededCircleParticles,
            center: (x, y),
            color_from: None,
            color_mid: None,
            color_to: None,
            color_mix: 0.0,
            input_color: Some(color),
            color_mul: 1.0,
            alpha: 1.0,
            radius: 0.0,
            stroke: 0.0,
            particles: Some(StandardEffectParticleSpec {
                seed: state_id,
                count: 5,
                progress: None,
                angle: Some(rotation),
                angle_range: 60.0,
                length: 1.0 + fin * 15.0,
                fin,
                fout,
                fslope,
                radius_base: 0.0,
                radius_fin_scale: 0.0,
                radius_fout_scale: 2.0,
                radius_fslope_scale: 0.0,
                secondary_vector_scale: 0.0,
                secondary_radius_base: 0.0,
                secondary_radius_fin_scale: 0.0,
                secondary_radius_fout_scale: 0.0,
                secondary_radius_fslope_scale: 0.0,
                alpha_midpoint: false,
            }),
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_BURNING_ID | FX_FIRE_HIT_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::SeededCircleParticles,
            center: (x, y),
            color_from: Some("Pal.lightFlame"),
            color_mid: None,
            color_to: Some("Pal.darkFlame"),
            color_mix: fin,
            input_color: None,
            color_mul: 1.0,
            alpha: 1.0,
            radius: 0.0,
            stroke: 0.0,
            particles: Some(StandardEffectParticleSpec {
                seed: state_id,
                count: 3,
                progress: None,
                angle: None,
                angle_range: 0.0,
                length: if effect_id == FX_FIRE_HIT_ID {
                    2.0 + fin * 10.0
                } else {
                    2.0 + fin * 7.0
                },
                fin,
                fout,
                fslope,
                radius_base: if effect_id == FX_FIRE_HIT_ID {
                    0.2
                } else {
                    0.1
                },
                radius_fin_scale: 0.0,
                radius_fout_scale: if effect_id == FX_FIRE_HIT_ID {
                    1.6
                } else {
                    1.4
                },
                radius_fslope_scale: 0.0,
                secondary_vector_scale: 0.0,
                secondary_radius_base: 0.0,
                secondary_radius_fin_scale: 0.0,
                secondary_radius_fout_scale: 0.0,
                secondary_radius_fslope_scale: 0.0,
                alpha_midpoint: false,
            }),
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_FIRE_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::SeededCircleParticles,
            center: (x, y),
            color_from: Some("Pal.lightFlame"),
            color_mid: None,
            color_to: Some("Pal.darkFlame"),
            color_mix: fin,
            input_color: None,
            color_mul: 1.0,
            alpha: 1.0,
            radius: 0.0,
            stroke: 0.0,
            particles: Some(StandardEffectParticleSpec {
                seed: state_id,
                count: 2,
                progress: None,
                angle: None,
                angle_range: 0.0,
                length: 2.0 + fin * 9.0,
                fin,
                fout,
                fslope,
                radius_base: 0.2,
                radius_fin_scale: 0.0,
                radius_fout_scale: 0.0,
                radius_fslope_scale: 1.5,
                secondary_vector_scale: 0.0,
                secondary_radius_base: 0.0,
                secondary_radius_fin_scale: 0.0,
                secondary_radius_fout_scale: 0.0,
                secondary_radius_fslope_scale: 0.0,
                alpha_midpoint: false,
            }),
            light_color: Some("Pal.lightFlame"),
            light_radius: 20.0 * fslope,
            light_opacity: 0.5,
        },
        FX_FIRE_SMOKE_ID | FX_STEAM_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::SeededCircleParticles,
            center: (x, y),
            color_from: Some(if effect_id == FX_STEAM_ID {
                "Color.lightGray"
            } else {
                "Color.gray"
            }),
            color_mid: None,
            color_to: None,
            color_mix: 0.0,
            input_color: None,
            color_mul: 1.0,
            alpha: 1.0,
            radius: 0.0,
            stroke: 0.0,
            particles: Some(StandardEffectParticleSpec {
                seed: state_id,
                count: if effect_id == FX_STEAM_ID { 2 } else { 1 },
                progress: None,
                angle: None,
                angle_range: 0.0,
                length: 2.0 + fin * 7.0,
                fin,
                fout,
                fslope,
                radius_base: 0.2,
                radius_fin_scale: 0.0,
                radius_fout_scale: 0.0,
                radius_fslope_scale: 1.5,
                secondary_vector_scale: 0.0,
                secondary_radius_base: 0.0,
                secondary_radius_fin_scale: 0.0,
                secondary_radius_fout_scale: 0.0,
                secondary_radius_fslope_scale: 0.0,
                alpha_midpoint: false,
            }),
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_CORROSION_VAPOR_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::SeededCircleParticles,
            center: (x, y),
            color_from: None,
            color_mid: None,
            color_to: None,
            color_mix: 0.0,
            input_color: Some(color),
            color_mul: 1.0,
            alpha: interp_pow2_out(fslope) * 0.5,
            radius: 0.0,
            stroke: 0.0,
            particles: Some(StandardEffectParticleSpec {
                seed: state_id,
                count: 2,
                progress: None,
                angle: None,
                angle_range: 0.0,
                length: 8.0 + finpow * 3.0,
                fin,
                fout,
                fslope,
                radius_base: 3.0,
                radius_fin_scale: 0.0,
                radius_fout_scale: 0.0,
                radius_fslope_scale: 0.0,
                secondary_vector_scale: 0.0,
                secondary_radius_base: 0.0,
                secondary_radius_fin_scale: 0.0,
                secondary_radius_fout_scale: 0.0,
                secondary_radius_fslope_scale: 0.0,
                alpha_midpoint: false,
            }),
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_VAPOR_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::SeededCircleParticles,
            center: (x, y),
            color_from: None,
            color_mid: None,
            color_to: None,
            color_mix: 0.0,
            input_color: Some(color),
            color_mul: 1.0,
            alpha: fout,
            radius: 0.0,
            stroke: 0.0,
            particles: Some(StandardEffectParticleSpec {
                seed: state_id,
                count: 3,
                progress: None,
                angle: None,
                angle_range: 0.0,
                length: 2.0 + finpow * 11.0,
                fin,
                fout,
                fslope,
                radius_base: 0.6,
                radius_fin_scale: 5.0,
                radius_fout_scale: 0.0,
                radius_fslope_scale: 0.0,
                secondary_vector_scale: 0.0,
                secondary_radius_base: 0.0,
                secondary_radius_fin_scale: 0.0,
                secondary_radius_fout_scale: 0.0,
                secondary_radius_fslope_scale: 0.0,
                alpha_midpoint: false,
            }),
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_VAPOR_SMALL_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::SeededCircleParticles,
            center: (x, y),
            color_from: None,
            color_mid: None,
            color_to: None,
            color_mix: 0.0,
            input_color: Some(color),
            color_mul: 1.0,
            alpha: fout,
            radius: 0.0,
            stroke: 0.0,
            particles: Some(StandardEffectParticleSpec {
                seed: state_id,
                count: 4,
                progress: None,
                angle: None,
                angle_range: 0.0,
                length: 2.0 + finpow * 5.0,
                fin,
                fout,
                fslope,
                radius_base: 1.0,
                radius_fin_scale: 4.0,
                radius_fout_scale: 0.0,
                radius_fslope_scale: 0.0,
                secondary_vector_scale: 0.0,
                secondary_radius_base: 0.0,
                secondary_radius_fin_scale: 0.0,
                secondary_radius_fout_scale: 0.0,
                secondary_radius_fslope_scale: 0.0,
                alpha_midpoint: false,
            }),
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_FIREBALL_SMOKE_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::SeededCircleParticles,
            center: (x, y),
            color_from: Some("Color.gray"),
            color_mid: None,
            color_to: None,
            color_mix: 0.0,
            input_color: None,
            color_mul: 1.0,
            alpha: 1.0,
            radius: 0.0,
            stroke: 0.0,
            particles: Some(StandardEffectParticleSpec {
                seed: state_id,
                count: 1,
                progress: None,
                angle: None,
                angle_range: 0.0,
                length: 2.0 + fin * 7.0,
                fin,
                fout,
                fslope,
                radius_base: 0.2,
                radius_fin_scale: 0.0,
                radius_fout_scale: 1.5,
                radius_fslope_scale: 0.0,
                secondary_vector_scale: 0.0,
                secondary_radius_base: 0.0,
                secondary_radius_fin_scale: 0.0,
                secondary_radius_fout_scale: 0.0,
                secondary_radius_fslope_scale: 0.0,
                alpha_midpoint: false,
            }),
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_BALLFIRE_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::SeededCircleParticles,
            center: (x, y),
            color_from: Some("Pal.lightFlame"),
            color_mid: None,
            color_to: Some("Pal.darkFlame"),
            color_mix: fin,
            input_color: None,
            color_mul: 1.0,
            alpha: 1.0,
            radius: 0.0,
            stroke: 0.0,
            particles: Some(StandardEffectParticleSpec {
                seed: state_id,
                count: 2,
                progress: None,
                angle: None,
                angle_range: 0.0,
                length: 2.0 + fin * 7.0,
                fin,
                fout,
                fslope,
                radius_base: 0.2,
                radius_fin_scale: 0.0,
                radius_fout_scale: 1.5,
                radius_fslope_scale: 0.0,
                secondary_vector_scale: 0.0,
                secondary_radius_base: 0.0,
                secondary_radius_fin_scale: 0.0,
                secondary_radius_fout_scale: 0.0,
                secondary_radius_fslope_scale: 0.0,
                alpha_midpoint: false,
            }),
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_MELTING_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::SeededCircleParticles,
            center: (x, y),
            color_from: Some("Liquids.slag.color"),
            color_mid: None,
            color_to: Some("Color.white"),
            color_mix: fout / 5.0 + mathf_random_seed_range(state_id as i64, 0.12),
            input_color: None,
            color_mul: 1.0,
            alpha: 1.0,
            radius: 0.0,
            stroke: 0.0,
            particles: Some(StandardEffectParticleSpec {
                seed: state_id,
                count: 2,
                progress: None,
                angle: None,
                angle_range: 0.0,
                length: 1.0 + fin * 3.0,
                fin,
                fout,
                fslope,
                radius_base: 0.2,
                radius_fin_scale: 0.0,
                radius_fout_scale: 1.2,
                radius_fslope_scale: 0.0,
                secondary_vector_scale: 0.0,
                secondary_radius_base: 0.0,
                secondary_radius_fin_scale: 0.0,
                secondary_radius_fout_scale: 0.0,
                secondary_radius_fslope_scale: 0.0,
                alpha_midpoint: false,
            }),
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_FREEZING_ID | FX_OILY_ID => {
            let (color_from, radius_fout_scale) = if effect_id == FX_FREEZING_ID {
                ("Liquids.cryofluid.color", 1.2)
            } else {
                ("Liquids.oil.color", 1.0)
            };

            StandardEffectDrawPlan {
                effect_id,
                layer: effect.layer,
                kind: StandardEffectDrawKind::SeededCircleParticles,
                center: (x, y),
                color_from: Some(color_from),
                color_mid: None,
                color_to: None,
                color_mix: 0.0,
                input_color: None,
                color_mul: 1.0,
                alpha: 1.0,
                radius: 0.0,
                stroke: 0.0,
                particles: Some(StandardEffectParticleSpec {
                    seed: state_id,
                    count: 2,
                    progress: None,
                    angle: None,
                    angle_range: 0.0,
                    length: 1.0 + fin * 2.0,
                    fin,
                    fout,
                    fslope,
                    radius_base: 0.0,
                    radius_fin_scale: 0.0,
                    radius_fout_scale,
                    radius_fslope_scale: 0.0,
                    secondary_vector_scale: 0.0,
                    secondary_radius_base: 0.0,
                    secondary_radius_fin_scale: 0.0,
                    secondary_radius_fout_scale: 0.0,
                    secondary_radius_fslope_scale: 0.0,
                    alpha_midpoint: false,
                }),
                light_color: None,
                light_radius: 0.0,
                light_opacity: 0.0,
            }
        }
        FX_WET_ID | FX_MUDDY_ID | FX_SPORE_SLOWED_ID => {
            let (color_from, alpha, radius) = match effect_id {
                FX_WET_ID => ("Liquids.water.color", (fin * 2.0).clamp(0.0, 1.0), fout),
                FX_MUDDY_ID => ("Pal.muddy", (fin * 2.0).clamp(0.0, 1.0), fout),
                FX_SPORE_SLOWED_ID => ("Pal.spore", 1.0, fslope * 1.1),
                _ => unreachable!(),
            };

            StandardEffectDrawPlan {
                effect_id,
                layer: effect.layer,
                kind: StandardEffectDrawKind::FilledCircle,
                center: (x, y),
                color_from: Some(color_from),
                color_mid: None,
                color_to: None,
                color_mix: 0.0,
                input_color: None,
                color_mul: 1.0,
                alpha,
                radius,
                stroke: 0.0,
                particles: None,
                light_color: None,
                light_radius: 0.0,
                light_opacity: 0.0,
            }
        }
        FX_SAPPED_ID | FX_ELECTRIFIED_ID | FX_OVERDRIVEN_ID => {
            let (
                color_from,
                input_color,
                radius_base,
                radius_fout_scale,
                radius_fslope_scale,
                rotation,
            ) = match effect_id {
                FX_SAPPED_ID => (Some("Pal.sap"), None, 0.0, 0.0, 1.1, 45.0),
                FX_ELECTRIFIED_ID => (Some("Pal.heal"), None, 0.0, 0.0, 1.1, 45.0),
                FX_OVERDRIVEN_ID => (None, Some(color), 0.5, 2.3, 0.0, 0.0),
                _ => unreachable!(),
            };

            StandardEffectDrawPlan {
                effect_id,
                layer: effect.layer,
                kind: StandardEffectDrawKind::SeededSquareParticles,
                center: (x, y),
                color_from,
                color_mid: None,
                color_to: None,
                color_mix: 0.0,
                input_color,
                color_mul: 1.0,
                alpha: 1.0,
                radius: 0.0,
                stroke: rotation,
                particles: Some(StandardEffectParticleSpec {
                    seed: state_id,
                    count: 2,
                    progress: None,
                    angle: None,
                    angle_range: 0.0,
                    length: 1.0 + fin * 2.0,
                    fin,
                    fout,
                    fslope,
                    radius_base,
                    radius_fin_scale: 0.0,
                    radius_fout_scale,
                    radius_fslope_scale,
                    secondary_vector_scale: 0.0,
                    secondary_radius_base: 0.0,
                    secondary_radius_fin_scale: 0.0,
                    secondary_radius_fout_scale: 0.0,
                    secondary_radius_fslope_scale: 0.0,
                    alpha_midpoint: false,
                }),
                light_color: None,
                light_radius: 0.0,
                light_opacity: 0.0,
            }
        }
        FX_OVERCLOCKED_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::FilledSquare,
            center: (x, y),
            color_from: None,
            color_mid: None,
            color_to: None,
            color_mix: 0.0,
            input_color: Some(color),
            color_mul: 1.0,
            alpha: 1.0,
            radius: fslope * 2.0,
            stroke: 45.0,
            particles: None,
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_SHOCKWAVE_ID
        | FX_SHOCKWAVE_SMALLER_ID
        | FX_BIG_SHOCKWAVE_ID
        | FX_SPAWN_SHOCKWAVE_ID
        | FX_POD_LAND_SHOCKWAVE_ID => {
            let (color_from, color_to, color_mix, radius, stroke) = match effect_id {
                FX_SHOCKWAVE_ID => (
                    "Color.white",
                    Some("Color.lightGray"),
                    fin,
                    fin * 28.0,
                    fout * 2.0 + 0.2,
                ),
                FX_SHOCKWAVE_SMALLER_ID => (
                    "Color.white",
                    Some("Color.lightGray"),
                    fin,
                    fin * 22.0,
                    fout * 2.0 + 0.2,
                ),
                FX_BIG_SHOCKWAVE_ID => (
                    "Color.white",
                    Some("Color.lightGray"),
                    fin,
                    fin * 50.0,
                    fout * 3.0,
                ),
                FX_SPAWN_SHOCKWAVE_ID => (
                    "Color.white",
                    Some("Color.lightGray"),
                    fin,
                    fin * (rotation + 50.0),
                    fout * 3.0 + 0.5,
                ),
                FX_POD_LAND_SHOCKWAVE_ID => ("Pal.accent", None, 0.0, fin * 26.0, fout * 2.0 + 0.2),
                _ => unreachable!(),
            };

            StandardEffectDrawPlan {
                effect_id,
                layer: effect.layer,
                kind: StandardEffectDrawKind::StrokedCircle,
                center: (x, y),
                color_from: Some(color_from),
                color_mid: None,
                color_to,
                color_mix,
                input_color: None,
                color_mul: 1.0,
                alpha: 1.0,
                radius,
                stroke,
                particles: None,
                light_color: None,
                light_radius: 0.0,
                light_opacity: 0.0,
            }
        }
        FX_BLOCK_EXPLOSION_SMOKE_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::SeededCircleParticles,
            center: (x, y),
            color_from: Some("Color.gray"),
            color_mid: None,
            color_to: None,
            color_mix: 0.0,
            input_color: None,
            color_mul: 1.0,
            alpha: 1.0,
            radius: 0.0,
            stroke: 0.0,
            particles: Some(StandardEffectParticleSpec {
                seed: state_id,
                count: 6,
                progress: None,
                angle: None,
                angle_range: 0.0,
                length: 4.0 + 30.0 * finpow,
                fin,
                fout,
                fslope,
                radius_base: 0.0,
                radius_fin_scale: 0.0,
                radius_fout_scale: 3.0,
                radius_fslope_scale: 0.0,
                secondary_vector_scale: 0.5,
                secondary_radius_base: 0.0,
                secondary_radius_fin_scale: 0.0,
                secondary_radius_fout_scale: 1.0,
                secondary_radius_fslope_scale: 0.0,
                alpha_midpoint: false,
            }),
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_STEAM_COOL_SMOKE_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::SeededCircleParticles,
            center: (x, y),
            color_from: Some("Pal.water"),
            color_mid: None,
            color_to: Some("Color.lightGray"),
            color_mix: interp_pow2_out(fin),
            input_color: None,
            color_mul: 1.0,
            alpha: interp_pow3_out(fout),
            radius: 0.0,
            stroke: 0.0,
            particles: Some(StandardEffectParticleSpec {
                seed: state_id,
                count: 4,
                progress: None,
                angle: Some(rotation),
                angle_range: 30.0,
                length: finpow * 7.0,
                fin,
                fout,
                fslope,
                radius_base: fout.max((fin * 8.0).min(1.0)) * 2.8,
                radius_fin_scale: 0.0,
                radius_fout_scale: 0.0,
                radius_fslope_scale: 0.0,
                secondary_vector_scale: 0.0,
                secondary_radius_base: 0.0,
                secondary_radius_fin_scale: 0.0,
                secondary_radius_fout_scale: 0.0,
                secondary_radius_fslope_scale: 0.0,
                alpha_midpoint: false,
            }),
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_SMOKE_PUFF_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::SeededCircleParticles,
            center: (x, y),
            color_from: None,
            color_mid: None,
            color_to: None,
            color_mix: 0.0,
            input_color: Some(color),
            color_mul: 1.0,
            alpha: 1.0,
            radius: 0.0,
            stroke: 0.0,
            particles: Some(StandardEffectParticleSpec {
                seed: state_id,
                count: 6,
                progress: None,
                angle: None,
                angle_range: 0.0,
                length: 4.0 + 30.0 * finpow,
                fin,
                fout,
                fslope,
                radius_base: 0.0,
                radius_fin_scale: 0.0,
                radius_fout_scale: 3.0,
                radius_fslope_scale: 0.0,
                secondary_vector_scale: 0.5,
                secondary_radius_base: 0.0,
                secondary_radius_fin_scale: 0.0,
                secondary_radius_fout_scale: 1.0,
                secondary_radius_fslope_scale: 0.0,
                alpha_midpoint: false,
            }),
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_SHOOT_SMALL_SMOKE_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::SeededCircleParticles,
            center: (x, y),
            color_from: Some("Pal.lighterOrange"),
            color_mid: Some("Color.lightGray"),
            color_to: Some("Color.gray"),
            color_mix: fin,
            input_color: None,
            color_mul: 1.0,
            alpha: 1.0,
            radius: 0.0,
            stroke: 0.0,
            particles: Some(StandardEffectParticleSpec {
                seed: state_id,
                count: 5,
                progress: None,
                angle: Some(rotation),
                angle_range: 20.0,
                length: finpow * 6.0,
                fin,
                fout,
                fslope,
                radius_base: 0.0,
                radius_fin_scale: 0.0,
                radius_fout_scale: 1.5,
                radius_fslope_scale: 0.0,
                secondary_vector_scale: 0.0,
                secondary_radius_base: 0.0,
                secondary_radius_fin_scale: 0.0,
                secondary_radius_fout_scale: 0.0,
                secondary_radius_fslope_scale: 0.0,
                alpha_midpoint: false,
            }),
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_SHOOT_BIG_SMOKE_ID | FX_SHOOT_BIG_SMOKE2_ID | FX_SHOOT_SMOKE_DISPERSE_ID => {
            let (
                color_from,
                color_mid,
                count,
                length_scale,
                angle_range,
                radius_base,
                radius_fout_scale,
            ) = match effect_id {
                FX_SHOOT_BIG_SMOKE_ID => (
                    "Pal.lighterOrange",
                    "Color.lightGray",
                    8,
                    19.0,
                    10.0,
                    0.2,
                    2.0,
                ),
                FX_SHOOT_BIG_SMOKE2_ID => (
                    "Pal.lightOrange",
                    "Color.lightGray",
                    9,
                    23.0,
                    20.0,
                    0.2,
                    2.4,
                ),
                FX_SHOOT_SMOKE_DISPERSE_ID => {
                    ("Pal.lightOrange", "Color.white", 9, 29.0, 18.0, 0.1, 2.2)
                }
                _ => unreachable!(),
            };

            StandardEffectDrawPlan {
                effect_id,
                layer: effect.layer,
                kind: StandardEffectDrawKind::SeededCircleParticles,
                center: (x, y),
                color_from: Some(color_from),
                color_mid: Some(color_mid),
                color_to: Some("Color.gray"),
                color_mix: fin,
                input_color: None,
                color_mul: 1.0,
                alpha: 1.0,
                radius: 0.0,
                stroke: 0.0,
                particles: Some(StandardEffectParticleSpec {
                    seed: state_id,
                    count,
                    progress: None,
                    angle: Some(rotation),
                    angle_range,
                    length: finpow * length_scale,
                    fin,
                    fout,
                    fslope,
                    radius_base,
                    radius_fin_scale: 0.0,
                    radius_fout_scale,
                    radius_fslope_scale: 0.0,
                    secondary_vector_scale: 0.0,
                    secondary_radius_base: 0.0,
                    secondary_radius_fin_scale: 0.0,
                    secondary_radius_fout_scale: 0.0,
                    secondary_radius_fslope_scale: 0.0,
                    alpha_midpoint: false,
                }),
                light_color: None,
                light_radius: 0.0,
                light_opacity: 0.0,
            }
        }
        FX_SMOKE_CLOUD_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::SeededCircleParticles,
            center: (x, y),
            color_from: Some("Color.gray"),
            color_mid: None,
            color_to: None,
            color_mix: 0.0,
            input_color: None,
            color_mul: 1.0,
            alpha: 1.0,
            radius: 0.0,
            stroke: 0.0,
            particles: Some(StandardEffectParticleSpec {
                seed: state_id,
                count: 30,
                progress: Some(fin),
                angle: None,
                angle_range: 0.0,
                length: 30.0,
                fin,
                fout,
                fslope,
                radius_base: 0.5,
                radius_fin_scale: 0.0,
                radius_fout_scale: 4.0,
                radius_fslope_scale: 0.0,
                secondary_vector_scale: 0.0,
                secondary_radius_base: 0.0,
                secondary_radius_fin_scale: 0.0,
                secondary_radius_fout_scale: 0.0,
                secondary_radius_fslope_scale: 0.0,
                alpha_midpoint: true,
            }),
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_BLAST_SMOKE_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::SeededCircleParticles,
            center: (x, y),
            color_from: Some("Color.lightGray"),
            color_mid: None,
            color_to: Some("Color.darkGray"),
            color_mix: fin,
            input_color: None,
            color_mul: 1.0,
            alpha: 1.0,
            radius: 0.0,
            stroke: 0.0,
            particles: Some(StandardEffectParticleSpec {
                seed: state_id,
                count: 12,
                progress: None,
                angle: None,
                angle_range: 0.0,
                length: 1.0 + fin * 23.0,
                fin,
                fout,
                fslope,
                radius_base: 1.0,
                radius_fin_scale: 0.0,
                radius_fout_scale: 3.0,
                radius_fslope_scale: 0.0,
                secondary_vector_scale: 0.0,
                secondary_radius_base: 0.0,
                secondary_radius_fin_scale: 0.0,
                secondary_radius_fout_scale: 0.0,
                secondary_radius_fslope_scale: 0.0,
                alpha_midpoint: false,
            }),
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_LAUNCH_ACCELERATOR_ID | FX_LAUNCH_ID | FX_HEAL_WAVE_MEND_ID | FX_OVERDRIVE_WAVE_ID => {
            let (color_from, input_color, radius, stroke) = match effect_id {
                FX_LAUNCH_ACCELERATOR_ID => {
                    (Some("Pal.accent"), None, 4.0 + finpow * 160.0, fout * 2.0)
                }
                FX_LAUNCH_ID => (Some("Pal.command"), None, 4.0 + finpow * 120.0, fout * 2.0),
                FX_HEAL_WAVE_MEND_ID => (None, Some(color), finpow * rotation, fout * 2.0),
                FX_OVERDRIVE_WAVE_ID => (None, Some(color), finpow * rotation, fout),
                _ => unreachable!(),
            };

            StandardEffectDrawPlan {
                effect_id,
                layer: effect.layer,
                kind: StandardEffectDrawKind::StrokedCircle,
                center: (x, y),
                color_from,
                color_mid: None,
                color_to: None,
                color_mix: 0.0,
                input_color,
                color_mul: 1.0,
                alpha: 1.0,
                radius,
                stroke,
                particles: None,
                light_color: None,
                light_radius: 0.0,
                light_opacity: 0.0,
            }
        }
        FX_HEAL_BLOCK_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::StrokedSquare,
            center: (x, y),
            color_from: Some("Pal.heal"),
            color_mid: None,
            color_to: None,
            color_mix: 0.0,
            input_color: None,
            color_mul: 1.0,
            alpha: 1.0,
            radius: fin * rotation * TILE_SIZE as f32 / 2.0,
            stroke: 2.0 * fout + 0.5,
            particles: None,
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_ROTATE_BLOCK_ID | FX_LIGHT_BLOCK_ID | FX_OVERDRIVE_BLOCK_FULL_ID => {
            let (color_from, input_color, alpha, radius) = match effect_id {
                FX_ROTATE_BLOCK_ID => (
                    Some("Pal.accent"),
                    None,
                    fout,
                    rotation * TILE_SIZE as f32 / 2.0,
                ),
                FX_LIGHT_BLOCK_ID => (None, Some(color), fout, rotation * TILE_SIZE as f32 / 2.0),
                FX_OVERDRIVE_BLOCK_FULL_ID => {
                    (None, Some(color), fslope * 0.4, rotation * TILE_SIZE as f32)
                }
                _ => unreachable!(),
            };

            StandardEffectDrawPlan {
                effect_id,
                layer: effect.layer,
                kind: StandardEffectDrawKind::FilledSquare,
                center: (x, y),
                color_from,
                color_mid: None,
                color_to: None,
                color_mix: 0.0,
                input_color,
                color_mul: 1.0,
                alpha,
                radius,
                stroke: 0.0,
                particles: None,
                light_color: None,
                light_radius: 0.0,
                light_opacity: 0.0,
            }
        }
        FX_RIPPLE_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::StrokedCircle,
            center: (x, y),
            color_from: None,
            color_mid: None,
            color_to: None,
            color_mix: 0.0,
            input_color: Some(color),
            color_mul: 1.5,
            alpha: 1.0,
            radius: (2.0 + fin * 4.0) * rotation,
            stroke: fout * 1.4,
            particles: None,
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        FX_BUBBLE_ID => StandardEffectDrawPlan {
            effect_id,
            layer: effect.layer,
            kind: StandardEffectDrawKind::SeededStrokedCircleParticles,
            center: (x, y),
            color_from: None,
            color_mid: None,
            color_to: None,
            color_mix: 0.0,
            input_color: Some(shift_color_value(color, 0.1)),
            color_mul: 1.0,
            alpha: 1.0,
            radius: 0.0,
            stroke: fout + 0.2,
            particles: Some(StandardEffectParticleSpec {
                seed: state_id,
                count: 2,
                progress: None,
                angle: None,
                angle_range: 0.0,
                length: rotation * 0.9,
                fin,
                fout,
                fslope,
                radius_base: 1.0,
                radius_fin_scale: 3.0,
                radius_fout_scale: 0.0,
                radius_fslope_scale: 0.0,
                secondary_vector_scale: 0.0,
                secondary_radius_base: 0.0,
                secondary_radius_fin_scale: 0.0,
                secondary_radius_fout_scale: 0.0,
                secondary_radius_fslope_scale: 0.0,
                alpha_midpoint: false,
            }),
            light_color: None,
            light_radius: 0.0,
            light_opacity: 0.0,
        },
        _ => return None,
    };
    Some(plan)
}

fn effect_fslope_from_fin(fin: f32) -> f32 {
    (1.0 - (fin.clamp(0.0, 1.0) - 0.5).abs() * 2.0).clamp(0.0, 1.0)
}

fn effect_finpow_from_fin(fin: f32) -> f32 {
    interp_pow3_out(fin)
}

fn interp_pow2_out(value: f32) -> f32 {
    1.0 - (1.0 - value.clamp(0.0, 1.0)).powi(2)
}

fn interp_pow3_out(value: f32) -> f32 {
    1.0 - (1.0 - value.clamp(0.0, 1.0)).powi(3)
}

fn mathf_random_seed_range(seed: i64, range: f32) -> f32 {
    ArcRand::with_seed(seed.wrapping_mul(99_999)).range(range)
}

fn shift_color_value(color: DecalColor, amount: f32) -> DecalColor {
    let max = color.r.max(color.g).max(color.b);
    let min = color.r.min(color.g).min(color.b);
    let delta = max - min;
    let hue = if delta.abs() <= f32::EPSILON {
        0.0
    } else if (max - color.r).abs() <= f32::EPSILON {
        60.0 * ((color.g - color.b) / delta).rem_euclid(6.0)
    } else if (max - color.g).abs() <= f32::EPSILON {
        60.0 * ((color.b - color.r) / delta + 2.0)
    } else {
        60.0 * ((color.r - color.g) / delta + 4.0)
    };
    let saturation = if max.abs() <= f32::EPSILON {
        0.0
    } else {
        delta / max
    };

    color_from_hsv(hue, saturation, max + amount, color.a)
}

fn color_from_hsv(hue: f32, saturation: f32, value: f32, alpha: f32) -> DecalColor {
    let hue = (hue / 60.0 + 6.0) % 6.0;
    let sector = hue as i32;
    let fraction = hue - sector as f32;
    let p = value * (1.0 - saturation);
    let q = value * (1.0 - saturation * fraction);
    let t = value * (1.0 - saturation * (1.0 - fraction));

    let (r, g, b) = match sector {
        0 => (value, t, p),
        1 => (q, value, p),
        2 => (p, value, t),
        3 => (p, q, value),
        4 => (t, p, value),
        _ => (value, p, q),
    };

    DecalColor { r, g, b, a: alpha }
}

pub fn standard_effect_color_symbol(name: &str) -> Option<DecalColor> {
    match name {
        "Color.white" => Some(DecalColor::WHITE),
        "Color.gray" => Some(DecalColor::from_rgba(0x7f7f7fff)),
        "Color.lightGray" => Some(DecalColor::from_rgba(0xbfbfbfff)),
        "Color.darkGray" => Some(DecalColor::from_rgba(0x3f3f3fff)),
        "Liquids.water.color" => Some(DecalColor::from_rgba(0x596ab8ff)),
        "Liquids.cryofluid.color" => Some(DecalColor::from_rgba(0x6ecdecff)),
        "Liquids.slag.color" => Some(DecalColor::from_rgba(0xffa166ff)),
        "Liquids.oil.color" => Some(DecalColor::from_rgba(0x313131ff)),
        "Pal.water" => Some(DecalColor::from_rgba(0x596ab8ff)),
        "Pal.accent" => Some(DecalColor::from_rgba(0xffd37fff)),
        "Pal.command" => Some(DecalColor::from_rgba(0xeab678ff)),
        "Pal.heal" => Some(DecalColor::from_rgba(0x98ffa9ff)),
        "Pal.sap" => Some(DecalColor::from_rgba(0x665c9fff)),
        "Pal.darkishGray" => Some(DecalColor {
            r: 0.3,
            g: 0.3,
            b: 0.3,
            a: 1.0,
        }),
        "Pal.muddy" => Some(DecalColor::from_rgba(0x432722ff)),
        "Pal.spore" => Some(DecalColor::from_rgba(0x7457ceff)),
        "Pal.lightishGray" => Some(DecalColor::from_rgba(0xa2a2a2ff)),
        "Pal.lighterOrange" => Some(DecalColor::from_rgba(0xf6e096ff)),
        "Pal.lightOrange" => Some(DecalColor::from_rgba(0xf68021ff)),
        "Pal.lightFlame" => Some(DecalColor::from_rgba(0xffdd55ff)),
        "Pal.darkFlame" => Some(DecalColor::from_rgba(0xdb401cff)),
        "Pal.meltdownHit" => Some(DecalColor::from_rgba(0xffb98bff)),
        _ => None,
    }
}

fn lerp_color(from: DecalColor, to: DecalColor, mix: f32) -> DecalColor {
    let mix = mix.clamp(0.0, 1.0);
    DecalColor {
        r: lerp(from.r, to.r, mix),
        g: lerp(from.g, to.g, mix),
        b: lerp(from.b, to.b, mix),
        a: lerp(from.a, to.a, mix),
    }
}

fn lerp_color_three(from: DecalColor, mid: DecalColor, to: DecalColor, mix: f32) -> DecalColor {
    let mix = mix.clamp(0.0, 1.0);
    if mix < 0.5 {
        lerp_color(from, mid, mix * 2.0)
    } else {
        lerp_color(mid, to, (mix - 0.5) * 2.0)
    }
}

fn trns(angle_degrees: f32, length: f32) -> (f32, f32) {
    let radians = angle_degrees * 0.017453292;
    (mathf_cos(radians) * length, mathf_sin(radians) * length)
}

fn mathf_sin(radians: f32) -> f32 {
    mathf_sin_table(((radians * 2607.5945) as i32 & 16383) as usize)
}

fn mathf_cos(radians: f32) -> f32 {
    mathf_sin_table((((radians + 1.5707964) * 2607.5945) as i32 & 16383) as usize)
}

fn mathf_sin_table(index: usize) -> f32 {
    match index & 16383 {
        0 | 8192 => 0.0,
        4096 => 1.0,
        12288 => -1.0,
        index => {
            let angle = (((index as f32 + 0.5) / 16384.0) * 6.2831855) as f64;
            angle.sin() as f32
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ArcRand {
    seed0: u64,
    seed1: u64,
}

impl ArcRand {
    fn with_seed(seed: i64) -> Self {
        let mut rand = Self { seed0: 0, seed1: 0 };
        rand.set_seed(seed);
        rand
    }

    fn set_seed(&mut self, seed: i64) {
        let seed = if seed == 0 {
            0x8000_0000_0000_0000
        } else {
            seed as u64
        };
        let hashed = murmur_hash3(seed);
        self.seed0 = hashed;
        self.seed1 = murmur_hash3(hashed);
    }

    fn next_long(&mut self) -> u64 {
        let mut seed0 = self.seed0;
        let seed1 = self.seed1;
        self.seed0 = seed1;
        seed0 ^= seed0 << 23;
        self.seed1 = seed0 ^ seed1 ^ (seed0 >> 17) ^ (seed1 >> 26);
        self.seed1.wrapping_add(seed1)
    }

    fn next_float(&mut self) -> f32 {
        ((self.next_long() >> 40) as f64 * (1.0 / (1u64 << 24) as f64)) as f32
    }

    fn random(&mut self, range: f32) -> f32 {
        self.next_float() * range
    }

    fn random_between(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_float() * (max - min)
    }

    fn range(&mut self, range: f32) -> f32 {
        self.next_float() * range * 2.0 - range
    }
}

fn murmur_hash3(mut value: u64) -> u64 {
    value ^= value >> 33;
    value = value.wrapping_mul(0xff51_afd7_ed55_8ccd);
    value ^= value >> 33;
    value = value.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    value ^= value >> 33;
    value
}

#[derive(Debug, Clone, PartialEq)]
pub struct Effect {
    pub id: i32,
    pub initialized: bool,
    pub lifetime: f32,
    pub clip: f32,
    pub start_delay: f32,
    pub base_rotation: f32,
    pub follow_parent: bool,
    pub rot_with_parent: bool,
    pub layer: f32,
    pub layer_duration: f32,
}

impl Effect {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            initialized: false,
            lifetime: DEFAULT_EFFECT_LIFETIME,
            clip: 0.0,
            start_delay: 0.0,
            base_rotation: 0.0,
            follow_parent: true,
            rot_with_parent: false,
            layer: DEFAULT_EFFECT_LAYER,
            layer_duration: 0.0,
        }
    }

    pub fn with_lifetime(id: i32, lifetime: f32, clip: f32) -> Self {
        Self {
            lifetime,
            clip,
            ..Self::new(id)
        }
    }

    pub fn start_delay(mut self, delay: f32) -> Self {
        self.start_delay = delay;
        self
    }

    pub fn follow_parent(mut self, follow: bool) -> Self {
        self.follow_parent = follow;
        self
    }

    pub fn rot_with_parent(mut self, follow: bool) -> Self {
        self.rot_with_parent = follow;
        self
    }

    pub fn layer(mut self, layer: f32) -> Self {
        self.layer = layer;
        self
    }

    pub fn layer_duration(mut self, layer: f32, duration: f32) -> Self {
        self.layer = layer;
        self.layer_duration = duration;
        self
    }

    pub fn base_rotation(mut self, rotation: f32) -> Self {
        self.base_rotation = rotation;
        self
    }

    pub fn should_create(&self, context: EffectCreateContext) -> bool {
        !context.headless && !context.is_none_effect && context.enable_effects
    }

    pub fn create_plan(
        &mut self,
        x: f32,
        y: f32,
        rotation: f32,
        color: DecalColor,
        data: Option<String>,
        parent: Option<EffectParent>,
        context: EffectCreateContext,
    ) -> Option<EffectCreatePlan> {
        if !self.should_create(context) || !context.camera_overlaps {
            return None;
        }

        let initialized_now = !self.initialized;
        self.initialized = true;

        let parent_id = parent
            .filter(|_| self.follow_parent)
            .map(|parent| parent.id);

        Some(EffectCreatePlan {
            delay: self.start_delay.max(0.0),
            initialized_now,
            spawn: EffectSpawnPlan {
                effect_id: self.id,
                x,
                y,
                rotation: self.base_rotation + rotation,
                color,
                data,
                lifetime: self.lifetime,
                clip: self.clip,
                layer: self.layer,
                layer_duration: self.layer_duration,
                parent_id,
                rot_with_parent: self.rot_with_parent && parent_id.is_some(),
            },
        })
    }

    pub fn render_with<F>(
        &self,
        input: EffectRenderParams,
        mut renderer: F,
    ) -> (EffectContainer, f32)
    where
        F: FnMut(&mut EffectContainer),
    {
        let mut container = EffectContainer::from_params(input);
        renderer(&mut container);
        let lifetime = container.lifetime;
        (container, lifetime)
    }
}

impl Default for Effect {
    fn default() -> Self {
        Self::new(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectParent {
    pub id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffectCreateContext {
    pub headless: bool,
    pub enable_effects: bool,
    pub is_none_effect: bool,
    pub camera_overlaps: bool,
}

impl Default for EffectCreateContext {
    fn default() -> Self {
        Self {
            headless: false,
            enable_effects: true,
            is_none_effect: false,
            camera_overlaps: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectCreatePlan {
    pub delay: f32,
    pub initialized_now: bool,
    pub spawn: EffectSpawnPlan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectSpawnPlan {
    pub effect_id: i32,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub color: DecalColor,
    pub data: Option<String>,
    pub lifetime: f32,
    pub clip: f32,
    pub layer: f32,
    pub layer_duration: f32,
    pub parent_id: Option<i32>,
    pub rot_with_parent: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectRenderParams {
    pub id: i32,
    pub color: DecalColor,
    pub time: f32,
    pub lifetime: f32,
    pub rotation: f32,
    pub x: f32,
    pub y: f32,
    pub data: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectContainer {
    pub x: f32,
    pub y: f32,
    pub time: f32,
    pub lifetime: f32,
    pub rotation: f32,
    pub color: DecalColor,
    pub id: i32,
    pub data: Option<String>,
}

impl EffectContainer {
    pub fn from_params(params: EffectRenderParams) -> Self {
        Self {
            x: params.x,
            y: params.y,
            color: params.color,
            time: params.time,
            lifetime: params.lifetime,
            id: params.id,
            rotation: params.rotation,
            data: params.data,
        }
    }

    pub fn fin(&self) -> f32 {
        self.time / self.lifetime
    }

    pub fn fout(&self) -> f32 {
        1.0 - self.fin()
    }

    pub fn finpow(&self) -> f32 {
        effect_finpow_from_fin(self.fin())
    }

    pub fn fslope(&self) -> f32 {
        effect_fslope_from_fin(self.fin())
    }

    pub fn scaled(&self, lifetime: f32) -> Option<Self> {
        (self.time <= lifetime).then(|| Self {
            lifetime,
            ..self.clone()
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectRegistry {
    effects: Vec<Effect>,
}

impl EffectRegistry {
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.effects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }

    pub fn create(&mut self, lifetime: f32, clip: f32) -> i32 {
        let id = self.effects.len() as i32;
        self.effects.push(Effect::with_lifetime(id, lifetime, clip));
        id
    }

    pub fn push(&mut self, mut effect: Effect) -> i32 {
        let id = self.effects.len() as i32;
        effect.id = id;
        self.effects.push(effect);
        id
    }

    pub fn get(&self, id: i32) -> Option<&Effect> {
        (id >= 0).then(|| self.effects.get(id as usize)).flatten()
    }

    pub fn get_mut(&mut self, id: i32) -> Option<&mut Effect> {
        if id >= 0 {
            self.effects.get_mut(id as usize)
        } else {
            None
        }
    }
}

impl Default for EffectRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MultiEffect {
    pub base: Effect,
    pub effects: Vec<Effect>,
}

impl Default for MultiEffect {
    fn default() -> Self {
        Self {
            base: Effect::default(),
            effects: Vec::new(),
        }
    }
}

impl MultiEffect {
    pub fn with_effects(effects: Vec<Effect>) -> Self {
        Self {
            effects,
            ..Default::default()
        }
    }

    pub fn create_plans(
        &mut self,
        x: f32,
        y: f32,
        rotation: f32,
        color: DecalColor,
        data: Option<String>,
        context: EffectCreateContext,
    ) -> Vec<EffectCreatePlan> {
        if !self.base.should_create(context) {
            return Vec::new();
        }

        self.effects
            .iter_mut()
            .filter_map(|effect| {
                effect.create_plan(x, y, rotation, color, data.clone(), None, context)
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeqRenderPlan {
    pub child_index: usize,
    pub params: EffectRenderParams,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeqEffect {
    pub base: Effect,
    pub effects: Vec<Effect>,
}

impl Default for SeqEffect {
    fn default() -> Self {
        let mut base = Effect::default();
        base.clip = 100.0;
        Self {
            base,
            effects: Vec::new(),
        }
    }
}

impl SeqEffect {
    pub fn with_effects(effects: Vec<Effect>) -> Self {
        Self {
            effects,
            ..Default::default()
        }
    }

    pub fn init_defaults(&mut self) {
        self.base.lifetime = 0.0;
        for effect in &self.effects {
            self.base.clip = self.base.clip.max(effect.clip);
            self.base.lifetime += effect.lifetime;
        }
    }

    pub fn render_plan(&mut self, input: EffectRenderParams) -> Option<SeqRenderPlan> {
        let mut sum = 0.0;
        for (index, effect) in self.effects.iter().enumerate() {
            if input.time <= effect.lifetime + sum {
                self.base.clip = self.base.clip.max(effect.clip);
                return Some(SeqRenderPlan {
                    child_index: index,
                    params: EffectRenderParams {
                        id: input.id + index as i32,
                        color: input.color,
                        time: input.time - sum,
                        lifetime: effect.lifetime,
                        rotation: input.rotation,
                        x: input.x,
                        y: input.y,
                        data: input.data,
                    },
                });
            }
            sum += effect.lifetime;
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WrapEffect {
    pub base: Effect,
    pub effect: Effect,
    pub color: DecalColor,
    pub rotation: f32,
}

impl Default for WrapEffect {
    fn default() -> Self {
        Self {
            base: Effect::default(),
            effect: Effect::default(),
            color: DecalColor::WHITE,
            rotation: 0.0,
        }
    }
}

impl WrapEffect {
    pub fn new(effect: Effect, color: DecalColor, rotation: f32) -> Self {
        Self {
            effect,
            color,
            rotation,
            ..Default::default()
        }
    }

    pub fn init_defaults(&mut self) {
        self.base.clip = self.effect.clip;
        self.base.lifetime = self.effect.lifetime;
    }

    pub fn create_plan(
        &mut self,
        x: f32,
        y: f32,
        data: Option<String>,
        context: EffectCreateContext,
    ) -> Option<EffectCreatePlan> {
        self.effect
            .create_plan(x, y, self.rotation, self.color, data, None, context)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RadialEffect {
    pub base: Effect,
    pub effect: Effect,
    pub rotation_spacing: f32,
    pub rotation_offset: f32,
    pub effect_rotation_offset: f32,
    pub length_offset: f32,
    pub amount: i32,
}

impl Default for RadialEffect {
    fn default() -> Self {
        let mut base = Effect::default();
        base.clip = 100.0;
        Self {
            base,
            effect: Effect::default(),
            rotation_spacing: 90.0,
            rotation_offset: 0.0,
            effect_rotation_offset: 0.0,
            length_offset: 0.0,
            amount: 4,
        }
    }
}

impl RadialEffect {
    pub fn new(
        effect: Effect,
        amount: i32,
        spacing: f32,
        length_offset: f32,
        effect_rotation_offset: f32,
    ) -> Self {
        Self {
            effect,
            amount,
            rotation_spacing: spacing,
            length_offset,
            effect_rotation_offset,
            ..Default::default()
        }
    }

    pub fn create_plans(
        &mut self,
        x: f32,
        y: f32,
        rotation: f32,
        color: DecalColor,
        data: Option<String>,
        context: EffectCreateContext,
    ) -> Vec<EffectCreatePlan> {
        if !self.base.should_create(context) {
            return Vec::new();
        }

        let mut out = Vec::with_capacity(self.amount.max(0) as usize);
        let mut current_rotation = rotation + self.rotation_offset;
        for _ in 0..self.amount.max(0) {
            if let Some(plan) = self.effect.create_plan(
                x + trnsx(current_rotation, self.length_offset),
                y + trnsy(current_rotation, self.length_offset),
                current_rotation + self.effect_rotation_offset,
                color,
                data.clone(),
                None,
                context,
            ) {
                out.push(plan);
            }
            current_rotation += self.rotation_spacing;
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoundPlaybackPlan {
    pub sound: String,
    pub x: f32,
    pub y: f32,
    pub delay: f32,
    pub pitch: f32,
    pub volume: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoundEffectCreatePlan {
    pub sound: SoundPlaybackPlan,
    pub effect: Option<EffectCreatePlan>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoundEffect {
    pub base: Effect,
    pub sound: String,
    pub min_pitch: f32,
    pub max_pitch: f32,
    pub min_volume: f32,
    pub max_volume: f32,
    pub effect: Effect,
}

impl Default for SoundEffect {
    fn default() -> Self {
        let mut base = Effect::default();
        base.start_delay = -1.0;
        Self {
            base,
            sound: "none".into(),
            min_pitch: 0.8,
            max_pitch: 1.2,
            min_volume: 1.0,
            max_volume: 1.0,
            effect: Effect::default(),
        }
    }
}

impl SoundEffect {
    pub fn new(sound: impl Into<String>, effect: Effect) -> Self {
        Self {
            sound: sound.into(),
            effect,
            ..Default::default()
        }
    }

    pub fn init_defaults(&mut self) {
        if self.base.start_delay < 0.0 {
            self.base.start_delay = self.effect.start_delay;
        }
    }

    pub fn create_plan(
        &mut self,
        x: f32,
        y: f32,
        rotation: f32,
        color: DecalColor,
        data: Option<String>,
        pitch_random: f32,
        volume_random: f32,
        context: EffectCreateContext,
    ) -> Option<SoundEffectCreatePlan> {
        if !self.base.should_create(context) {
            return None;
        }

        let pitch = lerp(self.min_pitch, self.max_pitch, pitch_random.clamp(0.0, 1.0));
        let volume = lerp(
            self.min_volume,
            self.max_volume,
            volume_random.clamp(0.0, 1.0),
        );
        Some(SoundEffectCreatePlan {
            sound: SoundPlaybackPlan {
                sound: self.sound.clone(),
                x,
                y,
                delay: self.base.start_delay.max(0.0),
                pitch,
                volume,
            },
            effect: self
                .effect
                .create_plan(x, y, rotation, color, data, None, context),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectInterp {
    Linear,
    Reverse,
}

impl EffectInterp {
    pub fn apply(self, from: f32, to: f32, t: f32) -> f32 {
        let t = match self {
            EffectInterp::Linear => t,
            EffectInterp::Reverse => 1.0 - t,
        }
        .clamp(0.0, 1.0);
        lerp(from, to, t)
    }

    pub fn scalar(self, t: f32) -> f32 {
        self.apply(0.0, 1.0, t)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WaveDrawPlan {
    pub center: (f32, f32),
    pub color_from: String,
    pub color_to: String,
    pub color_mix: f32,
    pub stroke: f32,
    pub radius: f32,
    pub sides: i32,
    pub rotation: f32,
    pub light_radius: f32,
    pub light_color: String,
    pub light_opacity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WaveEffect {
    pub base: Effect,
    pub color_from: String,
    pub color_to: String,
    pub light_color: Option<String>,
    pub size_from: f32,
    pub size_to: f32,
    pub light_scl: f32,
    pub light_opacity: f32,
    pub sides: i32,
    pub rotation: f32,
    pub stroke_from: f32,
    pub stroke_to: f32,
    pub interp: EffectInterp,
    pub light_interp: EffectInterp,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl Default for WaveEffect {
    fn default() -> Self {
        Self {
            base: Effect::default(),
            color_from: "white".into(),
            color_to: "white".into(),
            light_color: None,
            size_from: 0.0,
            size_to: 100.0,
            light_scl: 3.0,
            light_opacity: 0.8,
            sides: -1,
            rotation: 0.0,
            stroke_from: 2.0,
            stroke_to: 0.0,
            interp: EffectInterp::Linear,
            light_interp: EffectInterp::Reverse,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }
}

impl WaveEffect {
    pub fn init_defaults(&mut self) {
        self.base.clip = self
            .base
            .clip
            .max(self.size_from.max(self.size_to) + self.stroke_from.max(self.stroke_to));
    }

    pub fn draw_plan(&self, params: &EffectRenderParams) -> WaveDrawPlan {
        let fin = params.time / params.lifetime;
        let color_mix = self.interp.scalar(fin);
        let offset = rotate_offset(params.rotation, self.offset_x, self.offset_y);
        let center = (params.x + offset.0, params.y + offset.1);
        let radius = self.interp.apply(self.size_from, self.size_to, fin);
        WaveDrawPlan {
            center,
            color_from: self.color_from.clone(),
            color_to: self.color_to.clone(),
            color_mix,
            stroke: self.interp.apply(self.stroke_from, self.stroke_to, fin),
            radius,
            sides: self.sides,
            rotation: self.rotation + params.rotation,
            light_radius: radius * self.light_scl,
            light_color: self
                .light_color
                .clone()
                .unwrap_or_else(|| self.color_to.clone()),
            light_opacity: self.light_opacity * self.light_interp.scalar(fin),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExplosionWavePlan {
    pub stroke: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExplosionSmokePlan {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExplosionSparkPlan {
    pub x: f32,
    pub y: f32,
    pub stroke: f32,
    pub angle: f32,
    pub length: f32,
    pub light_radius: f32,
    pub light_opacity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExplosionDrawPlan {
    pub wave_color: String,
    pub smoke_color: String,
    pub spark_color: String,
    pub wave: Option<ExplosionWavePlan>,
    pub smoke_vector_radius: f32,
    pub spark_vector_radius: f32,
    pub smokes: Vec<ExplosionSmokePlan>,
    pub sparks: Vec<ExplosionSparkPlan>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExplosionEffect {
    pub base: Effect,
    pub wave_color: String,
    pub smoke_color: String,
    pub spark_color: String,
    pub wave_life: f32,
    pub wave_stroke: f32,
    pub wave_rad: f32,
    pub wave_rad_base: f32,
    pub spark_stroke: f32,
    pub spark_rad: f32,
    pub spark_len: f32,
    pub smoke_size: f32,
    pub smoke_size_base: f32,
    pub smoke_rad: f32,
    pub smokes: i32,
    pub sparks: i32,
}

impl Default for ExplosionEffect {
    fn default() -> Self {
        let mut base = Effect::default();
        base.clip = 100.0;
        base.lifetime = 22.0;
        Self {
            base,
            wave_color: "missileYellow".into(),
            smoke_color: "gray".into(),
            spark_color: "missileYellowBack".into(),
            wave_life: 6.0,
            wave_stroke: 3.0,
            wave_rad: 15.0,
            wave_rad_base: 2.0,
            spark_stroke: 1.0,
            spark_rad: 23.0,
            spark_len: 3.0,
            smoke_size: 4.0,
            smoke_size_base: 0.5,
            smoke_rad: 23.0,
            smokes: 5,
            sparks: 4,
        }
    }
}

impl ExplosionEffect {
    pub fn draw_plan(
        &self,
        container: &EffectContainer,
        smoke_vectors: &[(f32, f32)],
        spark_vectors: &[(f32, f32)],
    ) -> ExplosionDrawPlan {
        let wave = container
            .scaled(self.wave_life)
            .map(|inner| ExplosionWavePlan {
                stroke: self.wave_stroke * inner.fout(),
                radius: self.wave_rad_base + inner.fin() * self.wave_rad,
            });
        let smoke_radius = container.fout() * self.smoke_size + self.smoke_size_base;
        let smokes = if self.smoke_size > 0.0 {
            smoke_vectors
                .iter()
                .take(self.smokes.max(0) as usize)
                .map(|(x, y)| ExplosionSmokePlan {
                    x: container.x + x,
                    y: container.y + y,
                    radius: smoke_radius,
                })
                .collect()
        } else {
            Vec::new()
        };

        let spark_stroke = container.fout() * self.spark_stroke;
        let spark_len = 1.0 + container.fout() * self.spark_len;
        let sparks = spark_vectors
            .iter()
            .take(self.sparks.max(0) as usize)
            .map(|(x, y)| ExplosionSparkPlan {
                x: container.x + x,
                y: container.y + y,
                stroke: spark_stroke,
                angle: (*y).atan2(*x).to_degrees(),
                length: spark_len,
                light_radius: container.fout() * self.spark_len * 4.0,
                light_opacity: 0.7,
            })
            .collect();

        ExplosionDrawPlan {
            wave_color: self.wave_color.clone(),
            smoke_color: self.smoke_color.clone(),
            spark_color: self.spark_color.clone(),
            wave,
            smoke_vector_radius: 2.0 + self.smoke_rad * container.finpow(),
            spark_vector_radius: 1.0 + self.spark_rad * container.finpow(),
            smokes,
            sparks,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParticleVectorInput {
    pub angle_offset: f32,
    pub length_factor: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParticleDrawKind {
    Sprite {
        region: String,
        width: f32,
        height: f32,
        rotation: f32,
    },
    Line {
        stroke: f32,
        length: f32,
        angle: f32,
        cap: bool,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleDrawItem {
    pub x: f32,
    pub y: f32,
    pub kind: ParticleDrawKind,
    pub light_radius: f32,
    pub light_color: String,
    pub light_opacity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleDrawPlan {
    pub color_from: String,
    pub color_to: String,
    pub color_mix: f32,
    pub origin: (f32, f32),
    pub requested_length: f32,
    pub particles: Vec<ParticleDrawItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParticleEffect {
    pub base: Effect,
    pub color_from: String,
    pub color_to: String,
    pub particles: i32,
    pub rand_length: bool,
    pub casing_flip: bool,
    pub cone: f32,
    pub length: f32,
    pub base_length: f32,
    pub interp: EffectInterp,
    pub size_interp: Option<EffectInterp>,
    pub offset_x: f32,
    pub offset_y: f32,
    pub light_scl: f32,
    pub light_opacity: f32,
    pub light_color: Option<String>,
    pub spin: f32,
    pub size_from: f32,
    pub size_to: f32,
    pub size_change_start: f32,
    pub use_rotation: bool,
    pub offset: f32,
    pub region: String,
    pub line: bool,
    pub stroke_from: f32,
    pub stroke_to: f32,
    pub len_from: f32,
    pub len_to: f32,
    pub cap: bool,
}

impl Default for ParticleEffect {
    fn default() -> Self {
        Self {
            base: Effect::default(),
            color_from: "white".into(),
            color_to: "white".into(),
            particles: 6,
            rand_length: true,
            casing_flip: false,
            cone: 180.0,
            length: 20.0,
            base_length: 0.0,
            interp: EffectInterp::Linear,
            size_interp: None,
            offset_x: 0.0,
            offset_y: 0.0,
            light_scl: 2.0,
            light_opacity: 0.6,
            light_color: None,
            spin: 0.0,
            size_from: 2.0,
            size_to: 0.0,
            size_change_start: 0.0,
            use_rotation: true,
            offset: 0.0,
            region: "circle".into(),
            line: false,
            stroke_from: 2.0,
            stroke_to: 0.0,
            len_from: 4.0,
            len_to: 2.0,
            cap: true,
        }
    }
}

impl ParticleEffect {
    pub fn init_defaults(&mut self) {
        self.base.clip = self
            .base
            .clip
            .max(self.length + self.size_from.max(self.size_to));
        self.size_change_start = self.size_change_start.clamp(0.0, self.base.lifetime);
        if self.size_interp.is_none() {
            self.size_interp = Some(self.interp);
        }
    }

    pub fn draw_plan(
        &self,
        params: &EffectRenderParams,
        vectors: &[ParticleVectorInput],
        texture_ratio: f32,
    ) -> ParticleDrawPlan {
        let real_rotation = if self.use_rotation {
            if self.casing_flip {
                params.rotation.abs()
            } else {
                params.rotation
            }
        } else {
            self.base.base_rotation
        };
        let flip = if self.casing_flip {
            -signum_nonzero(params.rotation)
        } else {
            1.0
        };
        let raw_fin = params.time / params.lifetime;
        let fin = self.interp.scalar(raw_fin);
        let size_interp = self.size_interp.unwrap_or(self.interp);
        let size_curve = curve(raw_fin, self.size_change_start / params.lifetime, 1.0);
        let rad = size_interp.apply(self.size_from, self.size_to, size_curve) * 2.0;
        let offset = rotate_offset(real_rotation, self.offset_x * flip, self.offset_y);
        let origin = (params.x + offset.0, params.y + offset.1);
        let requested_length = self.length * fin + self.base_length;
        let light_color = self
            .light_color
            .clone()
            .unwrap_or_else(|| self.color_to.clone());

        let particles = vectors
            .iter()
            .take(self.particles.max(0) as usize)
            .map(|vector| {
                let len = if self.rand_length {
                    requested_length * vector.length_factor.clamp(0.0, 1.0)
                } else {
                    requested_length
                };
                let angle = real_rotation + vector.angle_offset.clamp(-self.cone, self.cone);
                let local = (trnsx(angle, len), trnsy(angle, len));
                let x = origin.0 + local.0;
                let y = origin.1 + local.1;
                if self.line {
                    let stroke = size_interp.apply(self.stroke_from, self.stroke_to, raw_fin);
                    let length = size_interp.apply(self.len_from, self.len_to, raw_fin);
                    ParticleDrawItem {
                        x,
                        y,
                        kind: ParticleDrawKind::Line {
                            stroke,
                            length,
                            angle: local.1.atan2(local.0).to_degrees(),
                            cap: self.cap,
                        },
                        light_radius: length * self.light_scl,
                        light_color: light_color.clone(),
                        light_opacity: self.light_opacity,
                    }
                } else {
                    ParticleDrawItem {
                        x,
                        y,
                        kind: ParticleDrawKind::Sprite {
                            region: self.region.clone(),
                            width: rad,
                            height: rad / texture_ratio.max(f32::EPSILON),
                            rotation: real_rotation + self.offset + params.time * self.spin,
                        },
                        light_radius: rad * self.light_scl,
                        light_color: light_color.clone(),
                        light_opacity: self.light_opacity,
                    }
                }
            })
            .collect();

        ParticleDrawPlan {
            color_from: self.color_from.clone(),
            color_to: self.color_to.clone(),
            color_mix: fin,
            origin,
            requested_length,
            particles,
        }
    }
}

fn trnsx(angle: f32, len: f32) -> f32 {
    angle.to_radians().cos() * len
}

fn trnsy(angle: f32, len: f32) -> f32 {
    angle.to_radians().sin() * len
}

fn rotate_offset(angle: f32, x: f32, y: f32) -> (f32, f32) {
    let rad = angle.to_radians();
    (x * rad.cos() - y * rad.sin(), x * rad.sin() + y * rad.cos())
}

fn lerp(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}

fn curve(value: f32, start: f32, end: f32) -> f32 {
    if (end - start).abs() <= f32::EPSILON {
        return if value >= end { 1.0 } else { 0.0 };
    }
    ((value - start) / (end - start)).clamp(0.0, 1.0)
}

fn signum_nonzero(value: f32) -> f32 {
    if value < 0.0 {
        -1.0
    } else {
        1.0
    }
}

pub fn shake_intensity(intensity: f32, camera_x: f32, camera_y: f32, x: f32, y: f32) -> f32 {
    let dx = x - camera_x;
    let dy = y - camera_y;
    let mut distance = (dx * dx + dy * dy).sqrt();
    if distance < 1.0 {
        distance = 1.0;
    }

    (1.0 / (distance * distance / SHAKE_FALLOFF)).clamp(0.0, 1.0) * intensity
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_effect_ids_include_puddle_ripple_dependencies() {
        assert_eq!(standard_effect_id("smoke"), Some(FX_SMOKE_ID));
        assert_eq!(standard_effect_id("fallSmoke"), Some(FX_FALL_SMOKE_ID));
        assert_eq!(standard_effect_id("rocketSmoke"), Some(FX_ROCKET_SMOKE_ID));
        assert_eq!(
            standard_effect_id("rocketSmokeLarge"),
            Some(FX_ROCKET_SMOKE_LARGE_ID)
        );
        assert_eq!(standard_effect_id("magmasmoke"), Some(FX_MAGMA_SMOKE_ID));
        assert_eq!(standard_effect_id("breakProp"), Some(FX_BREAK_PROP_ID));
        assert_eq!(standard_effect_id("unitDrop"), Some(FX_UNIT_DROP_ID));
        assert_eq!(standard_effect_id("unitLand"), Some(FX_UNIT_LAND_ID));
        assert_eq!(standard_effect_id("unitDust"), Some(FX_UNIT_DUST_ID));
        assert_eq!(
            standard_effect_id("unitLandSmall"),
            Some(FX_UNIT_LAND_SMALL_ID)
        );
        assert_eq!(standard_effect_id("crawlDust"), Some(FX_CRAWL_DUST_ID));
        assert_eq!(
            standard_effect_id("smokeAoeCloud"),
            Some(FX_SMOKE_AOE_CLOUD_ID)
        );
        assert_eq!(
            standard_effect_id("healWaveDynamic"),
            Some(FX_HEAL_WAVE_DYNAMIC_ID)
        );
        assert_eq!(standard_effect_id("healWave"), Some(FX_HEAL_WAVE_ID));
        assert_eq!(standard_effect_id("heal"), Some(FX_HEAL_ID));
        assert_eq!(standard_effect_id("dynamicWave"), Some(FX_DYNAMIC_WAVE_ID));
        assert_eq!(standard_effect_id("shieldWave"), Some(FX_SHIELD_WAVE_ID));
        assert_eq!(standard_effect_id("shieldApply"), Some(FX_SHIELD_APPLY_ID));
        assert_eq!(
            standard_effect_id("disperseTrail"),
            Some(FX_DISPERSE_TRAIL_ID)
        );
        assert_eq!(
            standard_effect_id("hitBulletBig"),
            Some(FX_HIT_BULLET_BIG_ID)
        );
        assert_eq!(
            standard_effect_id("hitFlameSmall"),
            Some(FX_HIT_FLAME_SMALL_ID)
        );
        assert_eq!(
            standard_effect_id("hitFlamePlasma"),
            Some(FX_HIT_FLAME_PLASMA_ID)
        );
        assert_eq!(
            standard_effect_id("hitLaserBlast"),
            Some(FX_HIT_LASER_BLAST_ID)
        );
        assert_eq!(standard_effect_id("hitEmpSpark"), Some(FX_HIT_EMP_SPARK_ID));
        assert_eq!(standard_effect_id("hitLancer"), Some(FX_HIT_LANCER_ID));
        assert_eq!(
            standard_effect_id("hitLancerLow"),
            Some(FX_HIT_LANCER_LOW_ID)
        );
        assert_eq!(standard_effect_id("hitBeam"), Some(FX_HIT_BEAM_ID));
        assert_eq!(
            standard_effect_id("hitFlameBeam"),
            Some(FX_HIT_FLAME_BEAM_ID)
        );
        assert_eq!(standard_effect_id("hitMeltdown"), Some(FX_HIT_MELTDOWN_ID));
        assert_eq!(standard_effect_id("hitMeltHeal"), Some(FX_HIT_MELT_HEAL_ID));
        assert_eq!(standard_effect_id("hitLaser"), Some(FX_HIT_LASER_ID));
        assert_eq!(standard_effect_id("despawn"), Some(FX_DESPAWN_ID));
        assert_eq!(standard_effect_id("hitLiquid"), Some(FX_HIT_LIQUID_ID));
        assert_eq!(
            standard_effect_id("artilleryTrail"),
            Some(FX_ARTILLERY_TRAIL_ID)
        );
        assert_eq!(standard_effect_id("incendTrail"), Some(FX_INCEND_TRAIL_ID));
        assert_eq!(
            standard_effect_id("unitAssemble"),
            Some(FX_UNIT_ASSEMBLE_ID)
        );
        assert_eq!(
            standard_effect_id("missileTrail"),
            Some(FX_MISSILE_TRAIL_ID)
        );
        assert_eq!(
            standard_effect_id("missileTrailShort"),
            Some(FX_MISSILE_TRAIL_SHORT_ID)
        );
        assert_eq!(standard_effect_id("colorTrail"), Some(FX_COLOR_TRAIL_ID));
        assert_eq!(standard_effect_id("absorb"), Some(FX_ABSORB_ID));
        assert_eq!(standard_effect_id("burning"), Some(FX_BURNING_ID));
        assert_eq!(standard_effect_id("fire"), Some(FX_FIRE_ID));
        assert_eq!(standard_effect_id("fireHit"), Some(FX_FIRE_HIT_ID));
        assert_eq!(standard_effect_id("fireSmoke"), Some(FX_FIRE_SMOKE_ID));
        assert_eq!(
            standard_effect_id("neoplasmHeal"),
            Some(FX_NEOPLASM_HEAL_ID)
        );
        assert_eq!(standard_effect_id("steam"), Some(FX_STEAM_ID));
        assert_eq!(
            standard_effect_id("corrosionVapor"),
            Some(FX_CORROSION_VAPOR_ID)
        );
        assert_eq!(standard_effect_id("vapor"), Some(FX_VAPOR_ID));
        assert_eq!(standard_effect_id("vaporSmall"), Some(FX_VAPOR_SMALL_ID));
        assert_eq!(
            standard_effect_id("fireballsmoke"),
            Some(FX_FIREBALL_SMOKE_ID)
        );
        assert_eq!(standard_effect_id("ballfire"), Some(FX_BALLFIRE_ID));
        assert_eq!(standard_effect_id("freezing"), Some(FX_FREEZING_ID));
        assert_eq!(standard_effect_id("melting"), Some(FX_MELTING_ID));
        assert_eq!(standard_effect_id("wet"), Some(FX_WET_ID));
        assert_eq!(standard_effect_id("muddy"), Some(FX_MUDDY_ID));
        assert_eq!(standard_effect_id("sapped"), Some(FX_SAPPED_ID));
        assert_eq!(standard_effect_id("electrified"), Some(FX_ELECTRIFIED_ID));
        assert_eq!(standard_effect_id("sporeSlowed"), Some(FX_SPORE_SLOWED_ID));
        assert_eq!(standard_effect_id("oily"), Some(FX_OILY_ID));
        assert_eq!(standard_effect_id("overdriven"), Some(FX_OVERDRIVEN_ID));
        assert_eq!(standard_effect_id("overclocked"), Some(FX_OVERCLOCKED_ID));
        assert_eq!(standard_effect_id("shockwave"), Some(FX_SHOCKWAVE_ID));
        assert_eq!(
            standard_effect_id("shockwaveSmaller"),
            Some(FX_SHOCKWAVE_SMALLER_ID)
        );
        assert_eq!(
            standard_effect_id("bigShockwave"),
            Some(FX_BIG_SHOCKWAVE_ID)
        );
        assert_eq!(
            standard_effect_id("spawnShockwave"),
            Some(FX_SPAWN_SHOCKWAVE_ID)
        );
        assert_eq!(
            standard_effect_id("podLandShockwave"),
            Some(FX_POD_LAND_SHOCKWAVE_ID)
        );
        assert_eq!(
            standard_effect_id("blockExplosionSmoke"),
            Some(FX_BLOCK_EXPLOSION_SMOKE_ID)
        );
        assert_eq!(
            standard_effect_id("steamCoolSmoke"),
            Some(FX_STEAM_COOL_SMOKE_ID)
        );
        assert_eq!(standard_effect_id("smokePuff"), Some(FX_SMOKE_PUFF_ID));
        assert_eq!(
            standard_effect_id("shootSmallSmoke"),
            Some(FX_SHOOT_SMALL_SMOKE_ID)
        );
        assert_eq!(
            standard_effect_id("shootBigSmoke"),
            Some(FX_SHOOT_BIG_SMOKE_ID)
        );
        assert_eq!(
            standard_effect_id("shootBigSmoke2"),
            Some(FX_SHOOT_BIG_SMOKE2_ID)
        );
        assert_eq!(
            standard_effect_id("shootSmokeDisperse"),
            Some(FX_SHOOT_SMOKE_DISPERSE_ID)
        );
        assert_eq!(standard_effect_id("smokeCloud"), Some(FX_SMOKE_CLOUD_ID));
        assert_eq!(standard_effect_id("blastsmoke"), Some(FX_BLAST_SMOKE_ID));
        assert_eq!(standard_effect_id("ripple"), Some(FX_RIPPLE_ID));
        assert_eq!(standard_effect_id("bubble"), Some(FX_BUBBLE_ID));
        assert_eq!(
            standard_effect_id("launchAccelerator"),
            Some(FX_LAUNCH_ACCELERATOR_ID)
        );
        assert_eq!(standard_effect_id("launch"), Some(FX_LAUNCH_ID));
        assert_eq!(
            standard_effect_id("healWaveMend"),
            Some(FX_HEAL_WAVE_MEND_ID)
        );
        assert_eq!(
            standard_effect_id("overdriveWave"),
            Some(FX_OVERDRIVE_WAVE_ID)
        );
        assert_eq!(standard_effect_id("healBlock"), Some(FX_HEAL_BLOCK_ID));
        assert_eq!(standard_effect_id("rotateBlock"), Some(FX_ROTATE_BLOCK_ID));
        assert_eq!(standard_effect_id("lightBlock"), Some(FX_LIGHT_BLOCK_ID));
        assert_eq!(
            standard_effect_id("overdriveBlockFull"),
            Some(FX_OVERDRIVE_BLOCK_FULL_ID)
        );
        assert_eq!(standard_effect_id("none"), None);
    }

    #[test]
    fn mathf_random_seed_range_matches_arc_seeded_range() {
        assert!((mathf_random_seed_range(133, 0.12) + 0.085_423_604).abs() < 0.000_001);
        assert!((mathf_random_seed_range(42, 0.12) + 0.019_490_603).abs() < 0.000_001);
    }

    #[test]
    fn standard_effect_lookup_matches_java_fx_lifetime_clip_and_layers() {
        let smoke = standard_effect_by_name("smoke").unwrap();
        assert_eq!(smoke.id, FX_SMOKE_ID);
        assert_eq!(smoke.lifetime, 100.0);
        assert_eq!(smoke.clip, 50.0);
        assert!(smoke.follow_parent);
        assert!(!smoke.rot_with_parent);

        assert_eq!(standard_effect(FX_FALL_SMOKE_ID).unwrap().lifetime, 110.0);
        assert_eq!(standard_effect(FX_ROCKET_SMOKE_ID).unwrap().lifetime, 120.0);
        assert_eq!(
            standard_effect(FX_ROCKET_SMOKE_LARGE_ID).unwrap().lifetime,
            220.0
        );
        assert_eq!(standard_effect(FX_MAGMA_SMOKE_ID).unwrap().lifetime, 110.0);
        let break_prop = standard_effect(FX_BREAK_PROP_ID).unwrap();
        assert_eq!(break_prop.lifetime, 23.0);
        assert_eq!(break_prop.layer, Layer::DEBRIS);
        assert_eq!(standard_effect(FX_UNIT_DROP_ID).unwrap().lifetime, 30.0);
        assert_eq!(standard_effect(FX_UNIT_LAND_ID).unwrap().lifetime, 30.0);
        assert_eq!(standard_effect(FX_UNIT_DUST_ID).unwrap().lifetime, 30.0);
        assert_eq!(
            standard_effect(FX_UNIT_LAND_SMALL_ID).unwrap().lifetime,
            30.0
        );
        assert_eq!(standard_effect(FX_CRAWL_DUST_ID).unwrap().lifetime, 35.0);
        assert_eq!(
            standard_effect(FX_UNIT_DROP_ID).unwrap().layer,
            Layer::DEBRIS
        );
        assert_eq!(
            standard_effect(FX_CRAWL_DUST_ID).unwrap().layer,
            Layer::DEBRIS
        );
        let smoke_aoe = standard_effect(FX_SMOKE_AOE_CLOUD_ID).unwrap();
        assert_eq!(smoke_aoe.lifetime, 180.0);
        assert_eq!(smoke_aoe.clip, 250.0);
        assert_eq!(
            standard_effect(FX_HEAL_WAVE_DYNAMIC_ID).unwrap().lifetime,
            22.0
        );
        assert_eq!(standard_effect(FX_HEAL_WAVE_ID).unwrap().lifetime, 22.0);
        assert_eq!(standard_effect(FX_HEAL_ID).unwrap().lifetime, 11.0);
        assert_eq!(standard_effect(FX_DYNAMIC_WAVE_ID).unwrap().lifetime, 22.0);
        assert_eq!(standard_effect(FX_SHIELD_WAVE_ID).unwrap().lifetime, 22.0);
        assert_eq!(standard_effect(FX_SHIELD_APPLY_ID).unwrap().lifetime, 11.0);
        assert_eq!(
            standard_effect(FX_DISPERSE_TRAIL_ID).unwrap().lifetime,
            13.0
        );
        assert_eq!(
            standard_effect(FX_HIT_BULLET_BIG_ID).unwrap().lifetime,
            13.0
        );
        assert_eq!(
            standard_effect(FX_HIT_FLAME_SMALL_ID).unwrap().lifetime,
            14.0
        );
        assert_eq!(
            standard_effect(FX_HIT_FLAME_PLASMA_ID).unwrap().lifetime,
            14.0
        );
        assert_eq!(
            standard_effect(FX_HIT_LASER_BLAST_ID).unwrap().lifetime,
            12.0
        );
        assert_eq!(standard_effect(FX_HIT_EMP_SPARK_ID).unwrap().lifetime, 40.0);
        assert_eq!(standard_effect(FX_HIT_LANCER_ID).unwrap().lifetime, 12.0);
        assert_eq!(
            standard_effect(FX_HIT_LANCER_LOW_ID).unwrap().lifetime,
            12.0
        );
        assert_eq!(standard_effect(FX_HIT_BEAM_ID).unwrap().lifetime, 12.0);
        assert_eq!(
            standard_effect(FX_HIT_FLAME_BEAM_ID).unwrap().lifetime,
            19.0
        );
        assert_eq!(standard_effect(FX_HIT_MELTDOWN_ID).unwrap().lifetime, 12.0);
        assert_eq!(standard_effect(FX_HIT_MELT_HEAL_ID).unwrap().lifetime, 12.0);
        assert_eq!(standard_effect(FX_HIT_LASER_ID).unwrap().lifetime, 8.0);
        assert_eq!(standard_effect(FX_DESPAWN_ID).unwrap().lifetime, 12.0);
        assert_eq!(standard_effect(FX_HIT_LIQUID_ID).unwrap().lifetime, 16.0);
        let artillery_trail = standard_effect(FX_ARTILLERY_TRAIL_ID).unwrap();
        assert_eq!(artillery_trail.lifetime, 50.0);
        assert_eq!(artillery_trail.layer, Layer::BULLET - 0.01);
        assert_eq!(standard_effect(FX_INCEND_TRAIL_ID).unwrap().lifetime, 50.0);
        assert_eq!(standard_effect(FX_BURNING_ID).unwrap().lifetime, 35.0);
        assert_eq!(standard_effect(FX_FIRE_HIT_ID).unwrap().lifetime, 35.0);
        assert_eq!(
            standard_effect(FX_STEAM_COOL_SMOKE_ID).unwrap().lifetime,
            35.0
        );
        assert_eq!(standard_effect(FX_COLOR_TRAIL_ID).unwrap().lifetime, 50.0);
        assert_eq!(standard_effect(FX_ABSORB_ID).unwrap().lifetime, 12.0);
        assert_eq!(
            standard_effect(FX_CORROSION_VAPOR_ID).unwrap().lifetime,
            50.0
        );
        assert_eq!(standard_effect(FX_VAPOR_SMALL_ID).unwrap().lifetime, 50.0);
        assert_eq!(
            standard_effect(FX_BLOCK_EXPLOSION_SMOKE_ID)
                .unwrap()
                .lifetime,
            30.0
        );
        assert_eq!(standard_effect(FX_BALLFIRE_ID).unwrap().lifetime, 25.0);
        assert_eq!(standard_effect(FX_FREEZING_ID).unwrap().lifetime, 40.0);
        assert_eq!(standard_effect(FX_MELTING_ID).unwrap().lifetime, 40.0);
        assert_eq!(standard_effect(FX_WET_ID).unwrap().lifetime, 80.0);
        assert_eq!(standard_effect(FX_MUDDY_ID).unwrap().lifetime, 80.0);
        assert_eq!(standard_effect(FX_SAPPED_ID).unwrap().lifetime, 40.0);
        assert_eq!(standard_effect(FX_ELECTRIFIED_ID).unwrap().lifetime, 40.0);
        assert_eq!(standard_effect(FX_SPORE_SLOWED_ID).unwrap().lifetime, 40.0);
        assert_eq!(standard_effect(FX_OILY_ID).unwrap().lifetime, 42.0);
        assert_eq!(standard_effect(FX_OVERDRIVEN_ID).unwrap().lifetime, 20.0);
        assert_eq!(standard_effect(FX_OVERCLOCKED_ID).unwrap().lifetime, 50.0);
        assert_eq!(standard_effect(FX_SHOCKWAVE_ID).unwrap().lifetime, 10.0);
        assert_eq!(standard_effect(FX_SHOCKWAVE_ID).unwrap().clip, 80.0);
        assert_eq!(
            standard_effect(FX_SHOCKWAVE_SMALLER_ID).unwrap().lifetime,
            9.0
        );
        assert_eq!(standard_effect(FX_BIG_SHOCKWAVE_ID).unwrap().lifetime, 10.0);
        assert_eq!(
            standard_effect(FX_SPAWN_SHOCKWAVE_ID).unwrap().lifetime,
            20.0
        );
        assert_eq!(standard_effect(FX_SPAWN_SHOCKWAVE_ID).unwrap().clip, 400.0);
        assert_eq!(
            standard_effect(FX_POD_LAND_SHOCKWAVE_ID).unwrap().lifetime,
            12.0
        );
        assert_eq!(standard_effect(FX_SMOKE_PUFF_ID).unwrap().lifetime, 30.0);
        assert_eq!(
            standard_effect(FX_SHOOT_SMALL_SMOKE_ID).unwrap().lifetime,
            20.0
        );
        assert_eq!(
            standard_effect(FX_SHOOT_BIG_SMOKE_ID).unwrap().lifetime,
            17.0
        );
        assert_eq!(
            standard_effect(FX_SHOOT_BIG_SMOKE2_ID).unwrap().lifetime,
            18.0
        );
        assert_eq!(
            standard_effect(FX_SHOOT_SMOKE_DISPERSE_ID)
                .unwrap()
                .lifetime,
            25.0
        );
        assert_eq!(standard_effect(FX_BLAST_SMOKE_ID).unwrap().lifetime, 26.0);

        let assemble = standard_effect(FX_UNIT_ASSEMBLE_ID).unwrap();
        assert_eq!(assemble.lifetime, 70.0);
        assert_eq!(assemble.clip, 50.0);
        assert_eq!(
            assemble.layer,
            crate::mindustry::graphics::Layer::FLYING_UNIT + 5.0
        );

        let trail = standard_effect(FX_MISSILE_TRAIL_SHORT_ID).unwrap();
        assert_eq!(trail.lifetime, 22.0);
        assert_eq!(
            trail.layer,
            crate::mindustry::graphics::Layer::BULLET - 0.001
        );

        let heal = standard_effect_by_name("neoplasmHeal").unwrap();
        assert_eq!(heal.lifetime, 120.0);
        assert!(heal.follow_parent);
        assert!(heal.rot_with_parent);
        assert_eq!(heal.layer, crate::mindustry::graphics::Layer::BULLET - 2.0);

        let ripple = standard_effect(FX_RIPPLE_ID).unwrap();
        assert_eq!(ripple.lifetime, 30.0);
        assert_eq!(ripple.layer, crate::mindustry::graphics::Layer::DEBRIS);
        assert_eq!(standard_effect(FX_BUBBLE_ID).unwrap().lifetime, 20.0);
        assert_eq!(
            standard_effect(FX_LAUNCH_ACCELERATOR_ID).unwrap().lifetime,
            22.0
        );
        assert_eq!(standard_effect(FX_LAUNCH_ID).unwrap().lifetime, 28.0);
        assert_eq!(
            standard_effect(FX_HEAL_WAVE_MEND_ID).unwrap().lifetime,
            40.0
        );
        assert_eq!(
            standard_effect(FX_OVERDRIVE_WAVE_ID).unwrap().lifetime,
            50.0
        );
        assert_eq!(standard_effect(FX_HEAL_BLOCK_ID).unwrap().lifetime, 20.0);
        assert_eq!(standard_effect(FX_ROTATE_BLOCK_ID).unwrap().lifetime, 30.0);
        assert_eq!(standard_effect(FX_LIGHT_BLOCK_ID).unwrap().lifetime, 60.0);
        assert_eq!(
            standard_effect(FX_OVERDRIVE_BLOCK_FULL_ID)
                .unwrap()
                .lifetime,
            60.0
        );
        assert!(standard_effect_by_name("none").is_none());
        assert!(standard_effect(-1).is_none());
    }

    #[test]
    fn standard_effect_render_lifetime_applies_ripple_dynamic_rotation_rule() {
        assert_eq!(
            standard_effect_render_lifetime(Some(FX_RIPPLE_ID as u16), 2.5, 30.0),
            75.0
        );
        assert_eq!(
            standard_effect_render_lifetime(Some(FX_SMOKE_ID as u16), 2.5, 100.0),
            100.0
        );
        assert_eq!(standard_effect_render_lifetime(None, 2.5, 10.0), 10.0);
    }

    #[test]
    fn standard_effect_draw_plan_covers_smoke_trails_and_ripple() {
        let smoke = standard_effect_draw_plan(
            Some(FX_SMOKE_ID as u16),
            7,
            10.0,
            20.0,
            0.0,
            50.0,
            100.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(smoke.kind, StandardEffectDrawKind::FilledCircle);
        assert_eq!(smoke.center, (10.0, 20.0));
        assert_eq!(smoke.color_from, Some("Color.gray"));
        assert_eq!(smoke.color_to, Some("Pal.darkishGray"));
        assert_eq!(smoke.color_mix, 0.5);
        assert_eq!(smoke.radius, 1.75);

        let trail = standard_effect_draw_plan(
            Some(FX_MISSILE_TRAIL_SHORT_ID as u16),
            8,
            1.0,
            2.0,
            4.0,
            11.0,
            22.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(trail.kind, StandardEffectDrawKind::FilledCircle);
        assert_eq!(trail.input_color, Some(DecalColor::WHITE));
        assert_eq!(trail.radius, 2.0);
        assert_eq!(trail.layer, Layer::BULLET - 0.001);

        let artillery_trail = standard_effect_draw_plan(
            Some(FX_ARTILLERY_TRAIL_ID as u16),
            108,
            1.0,
            2.0,
            4.0,
            25.0,
            50.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(artillery_trail.kind, StandardEffectDrawKind::FilledCircle);
        assert_eq!(artillery_trail.input_color, Some(DecalColor::WHITE));
        assert_eq!(artillery_trail.radius, 2.0);
        assert_eq!(artillery_trail.layer, Layer::BULLET - 0.01);

        let incend_trail = standard_effect_draw_plan(
            Some(FX_INCEND_TRAIL_ID as u16),
            109,
            1.0,
            2.0,
            4.0,
            25.0,
            50.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(incend_trail.kind, StandardEffectDrawKind::FilledCircle);
        assert_eq!(incend_trail.color_from, Some("Pal.lightOrange"));
        assert_eq!(incend_trail.radius, 2.0);

        let trail_color = DecalColor::from_rgba(0xabcdefcc);
        let color_trail = standard_effect_draw_plan(
            Some(FX_COLOR_TRAIL_ID as u16),
            113,
            1.0,
            2.0,
            6.0,
            25.0,
            50.0,
            trail_color,
        )
        .unwrap();
        assert_eq!(color_trail.kind, StandardEffectDrawKind::FilledCircle);
        assert_eq!(color_trail.input_color, Some(trail_color));
        assert_eq!(color_trail.radius, 3.0);

        let absorb = standard_effect_draw_plan(
            Some(FX_ABSORB_ID as u16),
            114,
            1.0,
            2.0,
            0.0,
            6.0,
            12.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(absorb.kind, StandardEffectDrawKind::StrokedCircle);
        assert_eq!(absorb.color_from, Some("Pal.accent"));
        assert_eq!(absorb.radius, 2.5);
        assert_eq!(absorb.stroke, 1.0);

        let heal_wave_dynamic = standard_effect_draw_plan(
            Some(FX_HEAL_WAVE_DYNAMIC_ID as u16),
            70,
            1.0,
            2.0,
            40.0,
            11.0,
            22.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(
            heal_wave_dynamic.kind,
            StandardEffectDrawKind::StrokedCircle
        );
        assert_eq!(heal_wave_dynamic.color_from, Some("Pal.heal"));
        assert_eq!(heal_wave_dynamic.radius, 39.0);
        assert_eq!(heal_wave_dynamic.stroke, 1.0);

        let heal_wave = standard_effect_draw_plan(
            Some(FX_HEAL_WAVE_ID as u16),
            71,
            1.0,
            2.0,
            0.0,
            11.0,
            22.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(heal_wave.radius, 56.5);
        assert_eq!(heal_wave.stroke, 1.0);

        let heal = standard_effect_draw_plan(
            Some(FX_HEAL_ID as u16),
            72,
            1.0,
            2.0,
            0.0,
            5.5,
            11.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(heal.radius, 8.125);
        assert_eq!(heal.stroke, 1.0);

        let shield_color = DecalColor::from_rgba(0xabcdefcc);
        let dynamic_wave = standard_effect_draw_plan(
            Some(FX_DYNAMIC_WAVE_ID as u16),
            73,
            1.0,
            2.0,
            40.0,
            11.0,
            22.0,
            shield_color,
        )
        .unwrap();
        assert_eq!(dynamic_wave.input_color, Some(shield_color));
        assert_eq!(dynamic_wave.alpha, 0.7);
        assert_eq!(dynamic_wave.radius, 39.0);
        assert_eq!(dynamic_wave.stroke, 1.0);

        let shield_wave = standard_effect_draw_plan(
            Some(FX_SHIELD_WAVE_ID as u16),
            74,
            1.0,
            2.0,
            0.0,
            11.0,
            22.0,
            shield_color,
        )
        .unwrap();
        assert_eq!(shield_wave.input_color, Some(shield_color));
        assert_eq!(shield_wave.alpha, 0.7);
        assert_eq!(shield_wave.radius, 56.5);

        let shield_apply = standard_effect_draw_plan(
            Some(FX_SHIELD_APPLY_ID as u16),
            75,
            1.0,
            2.0,
            0.0,
            5.5,
            11.0,
            shield_color,
        )
        .unwrap();
        assert_eq!(shield_apply.input_color, Some(shield_color));
        assert_eq!(shield_apply.alpha, 0.7);
        assert_eq!(shield_apply.radius, 8.125);

        let disperse_color = DecalColor::from_rgba(0x204080ff);
        let disperse = standard_effect_draw_plan(
            Some(FX_DISPERSE_TRAIL_ID as u16),
            76,
            1.0,
            2.0,
            30.0,
            6.5,
            13.0,
            disperse_color,
        )
        .unwrap();
        assert_eq!(disperse.kind, StandardEffectDrawKind::SeededLineParticles);
        assert_eq!(disperse.stroke, 1.45);
        assert_eq!(disperse.radius, 1.5);
        let disperse_particles = disperse.particles.unwrap();
        assert_eq!(disperse_particles.count, 2);
        assert_eq!(disperse_particles.angle, Some(210.0));
        assert_eq!(disperse_particles.angle_range, 15.0);
        assert_eq!(disperse_particles.length, 13.5);
        let disperse_lines = disperse.line_render_primitives_from_seed();
        assert_eq!(disperse_lines.len(), 2);
        assert_eq!(disperse_lines[0].stroke, 1.45);
        assert!((disperse_lines[0].angle - 202.726_27).abs() < 0.0001);
        assert!((disperse_lines[0].start.0 + 6.679_361).abs() < 0.0001);
        assert!((disperse_lines[0].start.1 + 1.217_186).abs() < 0.0001);
        assert!((disperse_lines[0].length - 4.280_647).abs() < 0.0001);
        assert!((disperse_lines[1].angle - 211.697_9).abs() < 0.0001);
        assert!((disperse_lines[1].start.0 + 0.282_225).abs() < 0.0001);
        assert!((disperse_lines[1].start.1 - 1.208_219).abs() < 0.0001);
        assert!((disperse_lines[1].length - 4.923_983).abs() < 0.0001);

        let hit_bullet_big = standard_effect_draw_plan(
            Some(FX_HIT_BULLET_BIG_ID as u16),
            82,
            1.0,
            2.0,
            30.0,
            6.5,
            13.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(
            hit_bullet_big.kind,
            StandardEffectDrawKind::SeededRadialLineParticles
        );
        assert_eq!(hit_bullet_big.color_from, Some("Color.white"));
        assert_eq!(hit_bullet_big.color_to, Some("Pal.lightOrange"));
        assert_eq!(hit_bullet_big.color_mix, 0.5);
        assert_eq!(hit_bullet_big.stroke, 1.25);
        assert_eq!(hit_bullet_big.radius, 1.5);
        let hit_bullet_big_particles = hit_bullet_big.particles.unwrap();
        assert_eq!(hit_bullet_big_particles.count, 8);
        assert_eq!(hit_bullet_big_particles.angle, Some(30.0));
        assert_eq!(hit_bullet_big_particles.angle_range, 50.0);
        assert_eq!(hit_bullet_big_particles.length, 26.25);
        assert_eq!(hit_bullet_big_particles.radius_fout_scale, 4.0);
        let hit_bullet_big_lines = hit_bullet_big.line_render_primitives_from_seed();
        assert_eq!(hit_bullet_big_lines.len(), 8);
        assert_eq!(hit_bullet_big_lines[0].stroke, 1.25);
        assert!((hit_bullet_big_lines[0].length - 3.5).abs() < 0.0001);

        let hit_flame_small = standard_effect_draw_plan(
            Some(FX_HIT_FLAME_SMALL_ID as u16),
            83,
            1.0,
            2.0,
            45.0,
            7.0,
            14.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(
            hit_flame_small.kind,
            StandardEffectDrawKind::SeededRadialLineParticles
        );
        assert_eq!(hit_flame_small.color_from, Some("Pal.lightFlame"));
        assert_eq!(hit_flame_small.color_to, Some("Pal.darkFlame"));
        assert_eq!(hit_flame_small.stroke, 1.0);
        let hit_flame_small_particles = hit_flame_small.particles.unwrap();
        assert_eq!(hit_flame_small_particles.count, 2);
        assert_eq!(hit_flame_small_particles.length, 8.5);
        assert_eq!(hit_flame_small_particles.angle, Some(45.0));
        assert_eq!(hit_flame_small_particles.angle_range, 50.0);
        assert_eq!(hit_flame_small_particles.radius_fout_scale, 3.0);
        let hit_flame_small_lines = hit_flame_small.line_render_primitives_from_seed();
        assert_eq!(hit_flame_small_lines.len(), 2);
        assert!((hit_flame_small_lines[0].length - 2.5).abs() < 0.0001);

        let hit_flame_plasma = standard_effect_draw_plan(
            Some(FX_HIT_FLAME_PLASMA_ID as u16),
            84,
            1.0,
            2.0,
            45.0,
            7.0,
            14.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(hit_flame_plasma.color_from, Some("Color.white"));
        assert_eq!(hit_flame_plasma.color_to, Some("Pal.heal"));
        assert_eq!(
            hit_flame_plasma.kind,
            StandardEffectDrawKind::SeededRadialLineParticles
        );
        assert_eq!(hit_flame_plasma.line_render_primitives_from_seed().len(), 2);

        let hit_liquid = standard_effect_draw_plan(
            Some(FX_HIT_LIQUID_ID as u16),
            85,
            1.0,
            2.0,
            30.0,
            8.0,
            16.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(
            hit_liquid.kind,
            StandardEffectDrawKind::SeededCircleParticles
        );
        assert_eq!(hit_liquid.input_color, Some(DecalColor::WHITE));
        let hit_liquid_particles = hit_liquid.particles.unwrap();
        assert_eq!(hit_liquid_particles.count, 5);
        assert_eq!(hit_liquid_particles.length, 8.5);
        assert_eq!(hit_liquid_particles.angle, Some(30.0));
        assert_eq!(hit_liquid_particles.angle_range, 60.0);
        assert_eq!(hit_liquid_particles.radius_fout_scale, 2.0);
        assert_eq!(hit_liquid.circle_render_primitives_from_seed().len(), 5);

        let shockwave = standard_effect_draw_plan(
            Some(FX_SHOCKWAVE_ID as u16),
            143,
            3.0,
            4.0,
            0.0,
            5.0,
            10.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(shockwave.kind, StandardEffectDrawKind::StrokedCircle);
        assert_eq!(shockwave.color_from, Some("Color.white"));
        assert_eq!(shockwave.color_to, Some("Color.lightGray"));
        assert_eq!(shockwave.color_mix, 0.5);
        assert_eq!(shockwave.radius, 14.0);
        assert_eq!(shockwave.stroke, 1.2);

        let shockwave_smaller = standard_effect_draw_plan(
            Some(FX_SHOCKWAVE_SMALLER_ID as u16),
            144,
            3.0,
            4.0,
            0.0,
            4.5,
            9.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(shockwave_smaller.radius, 11.0);
        assert_eq!(shockwave_smaller.stroke, 1.2);

        let big_shockwave = standard_effect_draw_plan(
            Some(FX_BIG_SHOCKWAVE_ID as u16),
            145,
            3.0,
            4.0,
            0.0,
            5.0,
            10.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(big_shockwave.radius, 25.0);
        assert_eq!(big_shockwave.stroke, 1.5);

        let spawn_shockwave = standard_effect_draw_plan(
            Some(FX_SPAWN_SHOCKWAVE_ID as u16),
            146,
            3.0,
            4.0,
            30.0,
            10.0,
            20.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(spawn_shockwave.radius, 40.0);
        assert_eq!(spawn_shockwave.stroke, 2.0);

        let pod_land_shockwave = standard_effect_draw_plan(
            Some(FX_POD_LAND_SHOCKWAVE_ID as u16),
            147,
            3.0,
            4.0,
            0.0,
            6.0,
            12.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(pod_land_shockwave.color_from, Some("Pal.accent"));
        assert_eq!(pod_land_shockwave.color_to, None);
        assert_eq!(
            pod_land_shockwave.resolved_draw_color(),
            standard_effect_color_symbol("Pal.accent")
        );
        assert_eq!(pod_land_shockwave.radius, 13.0);
        assert_eq!(pod_land_shockwave.stroke, 1.2);

        let ripple = standard_effect_draw_plan(
            Some(FX_RIPPLE_ID as u16),
            9,
            3.0,
            4.0,
            2.0,
            15.0,
            30.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(ripple.kind, StandardEffectDrawKind::StrokedCircle);
        assert_eq!(ripple.color_mul, 1.5);
        assert!((ripple.radius - 6.0).abs() < 0.0001);
        assert!((ripple.stroke - 1.05).abs() < 0.0001);

        let bubble = standard_effect_draw_plan(
            Some(FX_BUBBLE_ID as u16),
            245,
            3.0,
            4.0,
            10.0,
            10.0,
            20.0,
            DecalColor::from_rgba(0x000000ff),
        )
        .unwrap();
        assert_eq!(
            bubble.kind,
            StandardEffectDrawKind::SeededStrokedCircleParticles
        );
        assert_eq!(bubble.stroke, 0.7);
        let bubble_color = bubble.resolved_draw_color().unwrap();
        assert!((bubble_color.r - 0.1).abs() < 0.0001);
        assert!((bubble_color.g - 0.1).abs() < 0.0001);
        assert!((bubble_color.b - 0.1).abs() < 0.0001);
        let bubble_particles = bubble.particles.unwrap();
        assert_eq!(bubble_particles.count, 2);
        assert_eq!(bubble_particles.length, 9.0);
        assert_eq!(bubble_particles.radius_base, 1.0);
        assert_eq!(bubble_particles.radius_fin_scale, 3.0);
        let bubble_primitives = bubble.circle_render_primitives_from_seed();
        assert_eq!(bubble_primitives.len(), 2);
        assert_eq!(
            bubble_primitives[0].kind,
            StandardEffectDrawKind::StrokedCircle
        );
        assert_eq!(bubble_primitives[0].stroke, 0.7);
        assert_eq!(bubble_primitives[0].radius, 2.5);

        let launch_accelerator = standard_effect_draw_plan(
            Some(FX_LAUNCH_ACCELERATOR_ID as u16),
            246,
            3.0,
            4.0,
            0.0,
            11.0,
            22.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(
            launch_accelerator.kind,
            StandardEffectDrawKind::StrokedCircle
        );
        assert_eq!(launch_accelerator.color_from, Some("Pal.accent"));
        assert_eq!(launch_accelerator.radius, 144.0);
        assert_eq!(launch_accelerator.stroke, 1.0);

        let launch = standard_effect_draw_plan(
            Some(FX_LAUNCH_ID as u16),
            247,
            3.0,
            4.0,
            0.0,
            14.0,
            28.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(launch.color_from, Some("Pal.command"));
        assert_eq!(
            launch.resolved_draw_color(),
            standard_effect_color_symbol("Pal.command")
        );
        assert_eq!(launch.radius, 109.0);
        assert_eq!(launch.stroke, 1.0);

        let input_color = DecalColor::from_rgba(0x123456ff);
        let heal_wave_mend = standard_effect_draw_plan(
            Some(FX_HEAL_WAVE_MEND_ID as u16),
            249,
            3.0,
            4.0,
            32.0,
            20.0,
            40.0,
            input_color,
        )
        .unwrap();
        assert_eq!(heal_wave_mend.input_color, Some(input_color));
        assert_eq!(heal_wave_mend.radius, 28.0);
        assert_eq!(heal_wave_mend.stroke, 1.0);

        let overdrive_wave = standard_effect_draw_plan(
            Some(FX_OVERDRIVE_WAVE_ID as u16),
            250,
            3.0,
            4.0,
            32.0,
            25.0,
            50.0,
            input_color,
        )
        .unwrap();
        assert_eq!(overdrive_wave.input_color, Some(input_color));
        assert_eq!(overdrive_wave.radius, 28.0);
        assert_eq!(overdrive_wave.stroke, 0.5);

        let heal_block = standard_effect_draw_plan(
            Some(FX_HEAL_BLOCK_ID as u16),
            251,
            3.0,
            4.0,
            3.0,
            10.0,
            20.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(heal_block.kind, StandardEffectDrawKind::StrokedSquare);
        assert_eq!(heal_block.color_from, Some("Pal.heal"));
        assert_eq!(heal_block.radius, 6.0);
        assert_eq!(heal_block.stroke, 1.5);
        let heal_block_square = heal_block.square_render_primitives_from_seed();
        assert_eq!(heal_block_square.len(), 1);
        assert_eq!(heal_block_square[0].stroke, 1.5);

        let rotate_block = standard_effect_draw_plan(
            Some(FX_ROTATE_BLOCK_ID as u16),
            253,
            3.0,
            4.0,
            2.0,
            15.0,
            30.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(rotate_block.kind, StandardEffectDrawKind::FilledSquare);
        assert_eq!(rotate_block.color_from, Some("Pal.accent"));
        assert_eq!(rotate_block.alpha, 0.5);
        assert_eq!(rotate_block.radius, 8.0);

        let light_block = standard_effect_draw_plan(
            Some(FX_LIGHT_BLOCK_ID as u16),
            254,
            3.0,
            4.0,
            2.0,
            30.0,
            60.0,
            input_color,
        )
        .unwrap();
        assert_eq!(light_block.input_color, Some(input_color));
        assert_eq!(light_block.alpha, 0.5);
        assert_eq!(light_block.radius, 8.0);

        let overdrive_block = standard_effect_draw_plan(
            Some(FX_OVERDRIVE_BLOCK_FULL_ID as u16),
            255,
            3.0,
            4.0,
            2.0,
            30.0,
            60.0,
            input_color,
        )
        .unwrap();
        assert_eq!(overdrive_block.input_color, Some(input_color));
        assert_eq!(overdrive_block.alpha, 0.4);
        assert_eq!(overdrive_block.radius, 16.0);

        assert!(
            standard_effect_draw_plan(None, 0, 0.0, 0.0, 0.0, 0.0, 1.0, DecalColor::WHITE)
                .is_none()
        );
    }

    #[test]
    fn standard_effect_draw_plan_covers_hit_radial_line_batch() {
        let input_color = DecalColor::from_rgba(0xabcdefcc);
        let hit_laser_blast = standard_effect_draw_plan(
            Some(FX_HIT_LASER_BLAST_ID as u16),
            86,
            3.0,
            4.0,
            30.0,
            6.0,
            12.0,
            input_color,
        )
        .unwrap();
        assert_eq!(
            hit_laser_blast.kind,
            StandardEffectDrawKind::SeededRadialLineParticles
        );
        assert_eq!(hit_laser_blast.input_color, Some(input_color));
        assert_eq!(hit_laser_blast.color_from, None);
        assert_eq!(hit_laser_blast.stroke, 0.75);
        let hit_laser_blast_particles = hit_laser_blast.particles.unwrap();
        assert_eq!(hit_laser_blast_particles.count, 8);
        assert_eq!(hit_laser_blast_particles.angle, None);
        assert_eq!(hit_laser_blast_particles.angle_range, 0.0);
        assert_eq!(hit_laser_blast_particles.length, 14.875);
        assert_eq!(hit_laser_blast_particles.radius_fout_scale, 4.0);
        let hit_laser_blast_lines = hit_laser_blast.line_render_primitives_from_seed();
        assert_eq!(hit_laser_blast_lines.len(), 8);
        assert!((hit_laser_blast_lines[0].length - 3.0).abs() < 0.0001);

        let hit_emp_spark = standard_effect_draw_plan(
            Some(FX_HIT_EMP_SPARK_ID as u16),
            87,
            3.0,
            4.0,
            30.0,
            20.0,
            40.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(hit_emp_spark.color_from, Some("Pal.heal"));
        assert_eq!(hit_emp_spark.stroke, 0.8);
        let hit_emp_spark_particles = hit_emp_spark.particles.unwrap();
        assert_eq!(hit_emp_spark_particles.count, 18);
        assert_eq!(hit_emp_spark_particles.angle, Some(30.0));
        assert_eq!(hit_emp_spark_particles.angle_range, 360.0);
        assert_eq!(hit_emp_spark_particles.length, 23.625);
        assert_eq!(hit_emp_spark_particles.radius_fout_scale, 6.0);
        assert_eq!(hit_emp_spark.line_render_primitives_from_seed().len(), 18);

        let hit_lancer_low = standard_effect_draw_plan(
            Some(FX_HIT_LANCER_LOW_ID as u16),
            89,
            3.0,
            4.0,
            30.0,
            6.0,
            12.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(hit_lancer_low.color_from, Some("Color.white"));
        assert_eq!(hit_lancer_low.particles.unwrap().count, 4);
        assert_eq!(hit_lancer_low.line_render_primitives_from_seed().len(), 4);

        let hit_beam = standard_effect_draw_plan(
            Some(FX_HIT_BEAM_ID as u16),
            90,
            3.0,
            4.0,
            30.0,
            6.0,
            12.0,
            input_color,
        )
        .unwrap();
        assert_eq!(hit_beam.input_color, Some(input_color));
        assert_eq!(hit_beam.stroke, 1.0);
        assert_eq!(hit_beam.particles.unwrap().length, 15.75);
        assert_eq!(hit_beam.line_render_primitives_from_seed().len(), 6);

        let hit_flame_beam = standard_effect_draw_plan(
            Some(FX_HIT_FLAME_BEAM_ID as u16),
            91,
            3.0,
            4.0,
            30.0,
            9.5,
            19.0,
            input_color,
        )
        .unwrap();
        assert_eq!(
            hit_flame_beam.kind,
            StandardEffectDrawKind::SeededCircleParticles
        );
        assert_eq!(hit_flame_beam.input_color, Some(input_color));
        let hit_flame_beam_particles = hit_flame_beam.particles.unwrap();
        assert_eq!(hit_flame_beam_particles.count, 7);
        assert_eq!(hit_flame_beam_particles.length, 9.625);
        assert_eq!(hit_flame_beam_particles.radius_base, 0.5);
        assert_eq!(hit_flame_beam_particles.radius_fout_scale, 2.0);
        let hit_flame_beam_circles = hit_flame_beam.circle_render_primitives_from_seed();
        assert_eq!(hit_flame_beam_circles.len(), 7);
        assert!((hit_flame_beam_circles[0].radius - 1.5).abs() < 0.0001);

        let hit_meltdown = standard_effect_draw_plan(
            Some(FX_HIT_MELTDOWN_ID as u16),
            92,
            3.0,
            4.0,
            30.0,
            6.0,
            12.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(hit_meltdown.color_from, Some("Pal.meltdownHit"));
        assert_eq!(
            hit_meltdown.resolved_draw_color(),
            standard_effect_color_symbol("Pal.meltdownHit")
        );

        let hit_melt_heal = standard_effect_draw_plan(
            Some(FX_HIT_MELT_HEAL_ID as u16),
            93,
            3.0,
            4.0,
            30.0,
            6.0,
            12.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(hit_melt_heal.color_from, Some("Pal.heal"));
        assert_eq!(hit_melt_heal.line_render_primitives_from_seed().len(), 6);

        let hit_laser = standard_effect_draw_plan(
            Some(FX_HIT_LASER_ID as u16),
            98,
            3.0,
            4.0,
            30.0,
            4.0,
            8.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(hit_laser.kind, StandardEffectDrawKind::StrokedCircle);
        assert_eq!(hit_laser.color_from, Some("Color.white"));
        assert_eq!(hit_laser.color_to, Some("Pal.heal"));
        assert_eq!(hit_laser.color_mix, 0.5);
        assert_eq!(hit_laser.radius, 2.5);
        assert_eq!(hit_laser.stroke, 1.0);
        assert_eq!(hit_laser.light_color, Some("Pal.heal"));
        assert_eq!(hit_laser.light_radius, 23.0);
        assert_eq!(hit_laser.light_opacity, 0.35);

        let despawn = standard_effect_draw_plan(
            Some(FX_DESPAWN_ID as u16),
            100,
            3.0,
            4.0,
            30.0,
            6.0,
            12.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(
            despawn.kind,
            StandardEffectDrawKind::SeededRadialLineParticles
        );
        assert_eq!(despawn.color_from, Some("Pal.lighterOrange"));
        assert_eq!(despawn.color_to, Some("Color.gray"));
        assert_eq!(despawn.stroke, 0.5);
        let despawn_particles = despawn.particles.unwrap();
        assert_eq!(despawn_particles.count, 7);
        assert_eq!(despawn_particles.angle, Some(30.0));
        assert_eq!(despawn_particles.angle_range, 40.0);
        assert_eq!(despawn_particles.length, 3.5);
        assert_eq!(despawn_particles.radius_fout_scale, 2.0);
        let despawn_lines = despawn.line_render_primitives_from_seed();
        assert_eq!(despawn_lines.len(), 7);
        assert!((despawn_lines[0].length - 2.0).abs() < 0.0001);
    }

    #[test]
    fn standard_effect_draw_plan_covers_fire_smoke_steam_vapor_cloud_particles() {
        let fire = standard_effect_draw_plan(
            Some(FX_FIRE_ID as u16),
            42,
            10.0,
            20.0,
            0.0,
            25.0,
            50.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(fire.kind, StandardEffectDrawKind::SeededCircleParticles);
        assert_eq!(fire.color_from, Some("Pal.lightFlame"));
        assert_eq!(fire.color_to, Some("Pal.darkFlame"));
        assert_eq!(fire.color_mix, 0.5);
        assert_eq!(fire.light_color, Some("Pal.lightFlame"));
        assert_eq!(fire.light_radius, 20.0);
        assert_eq!(fire.light_opacity, 0.5);
        let fire_particles = fire.particles.unwrap();
        assert_eq!(fire_particles.seed, 42);
        assert_eq!(fire_particles.count, 2);
        assert_eq!(fire_particles.length, 6.5);
        assert_eq!(fire_particles.radius_base, 0.2);
        assert_eq!(fire_particles.radius_fslope_scale, 1.5);

        let fire_smoke = standard_effect_draw_plan(
            Some(FX_FIRE_SMOKE_ID as u16),
            43,
            0.0,
            0.0,
            0.0,
            17.5,
            35.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(fire_smoke.color_from, Some("Color.gray"));
        let fire_smoke_particles = fire_smoke.particles.unwrap();
        assert_eq!(fire_smoke_particles.count, 1);
        assert_eq!(fire_smoke_particles.length, 5.5);
        assert_eq!(fire_smoke_particles.radius_fslope_scale, 1.5);

        let steam = standard_effect_draw_plan(
            Some(FX_STEAM_ID as u16),
            44,
            0.0,
            0.0,
            0.0,
            17.5,
            35.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(steam.color_from, Some("Color.lightGray"));
        assert_eq!(steam.particles.unwrap().count, 2);

        let corrosion = standard_effect_draw_plan(
            Some(FX_CORROSION_VAPOR_ID as u16),
            127,
            0.0,
            0.0,
            0.0,
            25.0,
            50.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(corrosion.input_color, Some(DecalColor::WHITE));
        assert_eq!(corrosion.alpha, 0.5);
        let corrosion_particles = corrosion.particles.unwrap();
        assert_eq!(corrosion_particles.count, 2);
        assert_eq!(corrosion_particles.length, 10.625);
        assert_eq!(corrosion_particles.radius_base, 3.0);

        let vapor = standard_effect_draw_plan(
            Some(FX_VAPOR_ID as u16),
            45,
            0.0,
            0.0,
            0.0,
            55.0,
            110.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(vapor.input_color, Some(DecalColor::WHITE));
        assert_eq!(vapor.alpha, 0.5);
        let vapor_particles = vapor.particles.unwrap();
        assert_eq!(vapor_particles.count, 3);
        assert_eq!(vapor_particles.length, 11.625);
        assert_eq!(vapor_particles.radius_base, 0.6);
        assert_eq!(vapor_particles.radius_fin_scale, 5.0);

        let vapor_small = standard_effect_draw_plan(
            Some(FX_VAPOR_SMALL_ID as u16),
            129,
            0.0,
            0.0,
            0.0,
            25.0,
            50.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(vapor_small.input_color, Some(DecalColor::WHITE));
        assert_eq!(vapor_small.alpha, 0.5);
        let vapor_small_particles = vapor_small.particles.unwrap();
        assert_eq!(vapor_small_particles.count, 4);
        assert_eq!(vapor_small_particles.length, 6.375);
        assert_eq!(vapor_small_particles.radius_base, 1.0);
        assert_eq!(vapor_small_particles.radius_fin_scale, 4.0);

        let fireball = standard_effect_draw_plan(
            Some(FX_FIREBALL_SMOKE_ID as u16),
            46,
            0.0,
            0.0,
            0.0,
            12.5,
            25.0,
            DecalColor::WHITE,
        )
        .unwrap();
        let fireball_particles = fireball.particles.unwrap();
        assert_eq!(fireball.color_from, Some("Color.gray"));
        assert_eq!(fireball_particles.count, 1);
        assert_eq!(fireball_particles.length, 5.5);
        assert_eq!(fireball_particles.radius_base, 0.2);
        assert_eq!(fireball_particles.radius_fout_scale, 1.5);

        let ballfire = standard_effect_draw_plan(
            Some(FX_BALLFIRE_ID as u16),
            131,
            0.0,
            0.0,
            0.0,
            12.5,
            25.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(ballfire.color_from, Some("Pal.lightFlame"));
        assert_eq!(ballfire.color_to, Some("Pal.darkFlame"));
        assert_eq!(ballfire.color_mix, 0.5);
        let ballfire_particles = ballfire.particles.unwrap();
        assert_eq!(ballfire_particles.count, 2);
        assert_eq!(ballfire_particles.length, 5.5);
        assert_eq!(ballfire_particles.radius_base, 0.2);
        assert_eq!(ballfire_particles.radius_fout_scale, 1.5);

        let melting = standard_effect_draw_plan(
            Some(FX_MELTING_ID as u16),
            133,
            0.0,
            0.0,
            0.0,
            20.0,
            40.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(melting.color_from, Some("Liquids.slag.color"));
        assert_eq!(melting.color_to, Some("Color.white"));
        assert!((melting.color_mix - 0.014_576_398).abs() < 0.000_001);
        assert_eq!(
            standard_effect_color_symbol("Liquids.slag.color"),
            Some(DecalColor::from_rgba(0xffa166ff))
        );
        let melting_particles = melting.particles.unwrap();
        assert_eq!(melting_particles.count, 2);
        assert_eq!(melting_particles.length, 2.5);
        assert_eq!(melting_particles.radius_base, 0.2);
        assert_eq!(melting_particles.radius_fout_scale, 1.2);
        assert_eq!(melting.circle_render_primitives_from_seed().len(), 2);

        let freezing = standard_effect_draw_plan(
            Some(FX_FREEZING_ID as u16),
            132,
            0.0,
            0.0,
            0.0,
            20.0,
            40.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(freezing.color_from, Some("Liquids.cryofluid.color"));
        let freezing_particles = freezing.particles.unwrap();
        assert_eq!(freezing_particles.count, 2);
        assert_eq!(freezing_particles.length, 2.0);
        assert_eq!(freezing_particles.radius_fout_scale, 1.2);

        let oily = standard_effect_draw_plan(
            Some(FX_OILY_ID as u16),
            139,
            0.0,
            0.0,
            0.0,
            21.0,
            42.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(oily.color_from, Some("Liquids.oil.color"));
        let oily_particles = oily.particles.unwrap();
        assert_eq!(oily_particles.count, 2);
        assert_eq!(oily_particles.length, 2.0);
        assert_eq!(oily_particles.radius_fout_scale, 1.0);

        let sapped = standard_effect_draw_plan(
            Some(FX_SAPPED_ID as u16),
            136,
            0.0,
            0.0,
            0.0,
            20.0,
            40.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(sapped.kind, StandardEffectDrawKind::SeededSquareParticles);
        assert_eq!(sapped.color_from, Some("Pal.sap"));
        assert_eq!(sapped.stroke, 45.0);
        let sapped_particles = sapped.particles.unwrap();
        assert_eq!(sapped_particles.count, 2);
        assert_eq!(sapped_particles.length, 2.0);
        assert_eq!(sapped_particles.radius_fslope_scale, 1.1);
        assert_eq!(sapped.square_render_primitives_from_seed().len(), 2);

        let electrified = standard_effect_draw_plan(
            Some(FX_ELECTRIFIED_ID as u16),
            137,
            0.0,
            0.0,
            0.0,
            20.0,
            40.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(electrified.color_from, Some("Pal.heal"));
        assert_eq!(electrified.square_render_primitives_from_seed().len(), 2);

        let overdriven_color = DecalColor::from_rgba(0xfedcbaff);
        let overdriven = standard_effect_draw_plan(
            Some(FX_OVERDRIVEN_ID as u16),
            140,
            0.0,
            0.0,
            0.0,
            10.0,
            20.0,
            overdriven_color,
        )
        .unwrap();
        assert_eq!(
            overdriven.kind,
            StandardEffectDrawKind::SeededSquareParticles
        );
        assert_eq!(overdriven.input_color, Some(overdriven_color));
        let overdriven_particles = overdriven.particles.unwrap();
        assert_eq!(overdriven_particles.radius_base, 0.5);
        assert_eq!(overdriven_particles.radius_fout_scale, 2.3);

        let overclocked = standard_effect_draw_plan(
            Some(FX_OVERCLOCKED_ID as u16),
            141,
            0.0,
            0.0,
            0.0,
            25.0,
            50.0,
            overdriven_color,
        )
        .unwrap();
        assert_eq!(overclocked.kind, StandardEffectDrawKind::FilledSquare);
        assert_eq!(overclocked.input_color, Some(overdriven_color));
        assert_eq!(overclocked.radius, 2.0);
        assert_eq!(overclocked.stroke, 45.0);
        let overclocked_squares = overclocked.square_render_primitives_from_seed();
        assert_eq!(overclocked_squares.len(), 1);
        assert_eq!(overclocked_squares[0].rotation, 45.0);

        let wet = standard_effect_draw_plan(
            Some(FX_WET_ID as u16),
            134,
            1.0,
            2.0,
            0.0,
            40.0,
            80.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(wet.kind, StandardEffectDrawKind::FilledCircle);
        assert_eq!(wet.color_from, Some("Liquids.water.color"));
        assert_eq!(wet.alpha, 1.0);
        assert_eq!(wet.radius, 0.5);

        let muddy = standard_effect_draw_plan(
            Some(FX_MUDDY_ID as u16),
            135,
            1.0,
            2.0,
            0.0,
            40.0,
            80.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(muddy.color_from, Some("Pal.muddy"));
        assert_eq!(muddy.alpha, 1.0);
        assert_eq!(muddy.radius, 0.5);

        let spore_slowed = standard_effect_draw_plan(
            Some(FX_SPORE_SLOWED_ID as u16),
            138,
            1.0,
            2.0,
            0.0,
            20.0,
            40.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(spore_slowed.color_from, Some("Pal.spore"));
        assert_eq!(spore_slowed.radius, 1.1);

        let block_explosion_smoke = standard_effect_draw_plan(
            Some(FX_BLOCK_EXPLOSION_SMOKE_ID as u16),
            152,
            1.0,
            2.0,
            0.0,
            15.0,
            30.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(block_explosion_smoke.color_from, Some("Color.gray"));
        let block_explosion_smoke_particles = block_explosion_smoke.particles.unwrap();
        assert_eq!(block_explosion_smoke_particles.count, 6);
        assert_eq!(block_explosion_smoke_particles.length, 30.25);
        assert_eq!(block_explosion_smoke_particles.radius_fout_scale, 3.0);
        assert_eq!(block_explosion_smoke_particles.secondary_vector_scale, 0.5);
        assert_eq!(
            block_explosion_smoke_particles.secondary_radius_fout_scale,
            1.0
        );
        assert_eq!(
            block_explosion_smoke
                .circle_render_primitives_from_seed()
                .len(),
            12
        );

        let steam_cool = standard_effect_draw_plan(
            Some(FX_STEAM_COOL_SMOKE_ID as u16),
            153,
            0.0,
            0.0,
            45.0,
            17.5,
            35.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(steam_cool.color_from, Some("Pal.water"));
        assert_eq!(steam_cool.color_to, Some("Color.lightGray"));
        assert_eq!(steam_cool.color_mix, 0.75);
        assert_eq!(steam_cool.alpha, 0.875);
        let steam_cool_particles = steam_cool.particles.unwrap();
        assert_eq!(steam_cool_particles.count, 4);
        assert_eq!(steam_cool_particles.length, 6.125);
        assert_eq!(steam_cool_particles.angle, Some(45.0));
        assert_eq!(steam_cool_particles.angle_range, 30.0);
        assert!((steam_cool_particles.radius_base - 2.8).abs() < 0.0001);
        assert_eq!(steam_cool.circle_render_primitives_from_seed().len(), 4);

        let smoke_puff = standard_effect_draw_plan(
            Some(FX_SMOKE_PUFF_ID as u16),
            154,
            1.0,
            2.0,
            0.0,
            15.0,
            30.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(smoke_puff.input_color, Some(DecalColor::WHITE));
        let smoke_puff_particles = smoke_puff.particles.unwrap();
        assert_eq!(smoke_puff_particles.count, 6);
        assert_eq!(smoke_puff_particles.length, 30.25);
        assert_eq!(smoke_puff_particles.radius_fout_scale, 3.0);
        assert_eq!(smoke_puff_particles.secondary_vector_scale, 0.5);
        assert_eq!(smoke_puff_particles.secondary_radius_fout_scale, 1.0);

        let shoot_small_smoke = standard_effect_draw_plan(
            Some(FX_SHOOT_SMALL_SMOKE_ID as u16),
            159,
            1.0,
            2.0,
            45.0,
            10.0,
            20.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(shoot_small_smoke.color_from, Some("Pal.lighterOrange"));
        assert_eq!(shoot_small_smoke.color_mid, Some("Color.lightGray"));
        assert_eq!(shoot_small_smoke.color_to, Some("Color.gray"));
        assert_eq!(
            shoot_small_smoke.resolved_draw_color(),
            standard_effect_color_symbol("Color.lightGray")
        );
        let shoot_small_smoke_particles = shoot_small_smoke.particles.unwrap();
        assert_eq!(shoot_small_smoke_particles.count, 5);
        assert_eq!(shoot_small_smoke_particles.length, 5.25);
        assert_eq!(shoot_small_smoke_particles.angle, Some(45.0));
        assert_eq!(shoot_small_smoke_particles.angle_range, 20.0);
        assert_eq!(shoot_small_smoke_particles.radius_fout_scale, 1.5);

        let shoot_big_smoke = standard_effect_draw_plan(
            Some(FX_SHOOT_BIG_SMOKE_ID as u16),
            166,
            1.0,
            2.0,
            45.0,
            8.5,
            17.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(shoot_big_smoke.color_from, Some("Pal.lighterOrange"));
        assert_eq!(shoot_big_smoke.color_mid, Some("Color.lightGray"));
        let shoot_big_smoke_particles = shoot_big_smoke.particles.unwrap();
        assert_eq!(shoot_big_smoke_particles.count, 8);
        assert_eq!(shoot_big_smoke_particles.length, 16.625);
        assert_eq!(shoot_big_smoke_particles.angle_range, 10.0);
        assert_eq!(shoot_big_smoke_particles.radius_base, 0.2);
        assert_eq!(shoot_big_smoke_particles.radius_fout_scale, 2.0);

        let shoot_big_smoke2 = standard_effect_draw_plan(
            Some(FX_SHOOT_BIG_SMOKE2_ID as u16),
            167,
            1.0,
            2.0,
            45.0,
            9.0,
            18.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(shoot_big_smoke2.color_from, Some("Pal.lightOrange"));
        assert_eq!(shoot_big_smoke2.color_mid, Some("Color.lightGray"));
        let shoot_big_smoke2_particles = shoot_big_smoke2.particles.unwrap();
        assert_eq!(shoot_big_smoke2_particles.count, 9);
        assert_eq!(shoot_big_smoke2_particles.length, 20.125);
        assert_eq!(shoot_big_smoke2_particles.angle_range, 20.0);
        assert_eq!(shoot_big_smoke2_particles.radius_base, 0.2);
        assert_eq!(shoot_big_smoke2_particles.radius_fout_scale, 2.4);

        let shoot_smoke_disperse = standard_effect_draw_plan(
            Some(FX_SHOOT_SMOKE_DISPERSE_ID as u16),
            168,
            1.0,
            2.0,
            45.0,
            12.5,
            25.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(shoot_smoke_disperse.color_from, Some("Pal.lightOrange"));
        assert_eq!(shoot_smoke_disperse.color_mid, Some("Color.white"));
        let shoot_smoke_disperse_particles = shoot_smoke_disperse.particles.unwrap();
        assert_eq!(shoot_smoke_disperse_particles.count, 9);
        assert_eq!(shoot_smoke_disperse_particles.length, 25.375);
        assert_eq!(shoot_smoke_disperse_particles.angle_range, 18.0);
        assert_eq!(shoot_smoke_disperse_particles.radius_base, 0.1);
        assert_eq!(shoot_smoke_disperse_particles.radius_fout_scale, 2.2);

        let cloud = standard_effect_draw_plan(
            Some(FX_SMOKE_CLOUD_ID as u16),
            47,
            0.0,
            0.0,
            0.0,
            35.0,
            70.0,
            DecalColor::WHITE,
        )
        .unwrap();
        let cloud_particles = cloud.particles.unwrap();
        assert_eq!(cloud.color_from, Some("Color.gray"));
        assert_eq!(cloud_particles.count, 30);
        assert_eq!(cloud_particles.progress, Some(0.5));
        assert_eq!(cloud_particles.length, 30.0);
        assert_eq!(cloud_particles.radius_base, 0.5);
        assert_eq!(cloud_particles.radius_fout_scale, 4.0);
        assert!(cloud_particles.alpha_midpoint);
    }

    #[test]
    fn standard_effect_draw_plan_covers_simple_smoke_and_fire_variants() {
        let fall = standard_effect_draw_plan(
            Some(FX_FALL_SMOKE_ID as u16),
            50,
            10.0,
            20.0,
            0.25,
            55.0,
            110.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(fall.kind, StandardEffectDrawKind::FilledCircle);
        assert_eq!(fall.color_from, Some("Color.gray"));
        assert_eq!(fall.color_to, Some("Color.darkGray"));
        assert_eq!(fall.color_mix, 0.25);
        assert_eq!(fall.radius, 1.75);

        let rocket = standard_effect_draw_plan(
            Some(FX_ROCKET_SMOKE_ID as u16),
            51,
            0.0,
            0.0,
            0.5,
            60.0,
            120.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(rocket.kind, StandardEffectDrawKind::FilledCircle);
        assert!((rocket.alpha - 0.65).abs() < 0.0001);
        assert_eq!(rocket.radius, 3.0);

        let rocket_large = standard_effect_draw_plan(
            Some(FX_ROCKET_SMOKE_LARGE_ID as u16),
            52,
            0.0,
            0.0,
            0.5,
            110.0,
            220.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(rocket_large.kind, StandardEffectDrawKind::FilledCircle);
        assert!((rocket_large.radius - 3.9).abs() < 0.0001);

        let magma = standard_effect_draw_plan(
            Some(FX_MAGMA_SMOKE_ID as u16),
            53,
            0.0,
            0.0,
            0.0,
            55.0,
            110.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(magma.kind, StandardEffectDrawKind::FilledCircle);
        assert_eq!(magma.radius, 6.0);

        let break_prop = standard_effect_draw_plan(
            Some(FX_BREAK_PROP_ID as u16),
            37,
            0.0,
            0.0,
            2.0,
            11.5,
            23.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(break_prop.layer, Layer::DEBRIS);
        assert_eq!(break_prop.input_color, Some(DecalColor::WHITE));
        assert_eq!(break_prop.color_mul, 1.1);
        let break_prop_particles = break_prop.particles.unwrap();
        assert_eq!(break_prop_particles.count, 6);
        assert_eq!(break_prop_particles.length, 33.25);
        assert_eq!(break_prop_particles.radius_base, 0.3);
        assert_eq!(break_prop_particles.radius_fout_scale, 7.0);

        let unit_drop = standard_effect_draw_plan(
            Some(FX_UNIT_DROP_ID as u16),
            38,
            0.0,
            0.0,
            0.0,
            15.0,
            30.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(unit_drop.color_from, Some("Pal.lightishGray"));
        let unit_drop_particles = unit_drop.particles.unwrap();
        assert_eq!(unit_drop_particles.count, 9);
        assert_eq!(unit_drop_particles.length, 20.5);
        assert_eq!(unit_drop_particles.radius_base, 0.4);
        assert_eq!(unit_drop_particles.radius_fout_scale, 4.0);

        let unit_land = standard_effect_draw_plan(
            Some(FX_UNIT_LAND_ID as u16),
            39,
            0.0,
            0.0,
            0.0,
            15.0,
            30.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(unit_land.color_mul, 1.1);
        let unit_land_particles = unit_land.particles.unwrap();
        assert_eq!(unit_land_particles.count, 6);
        assert_eq!(unit_land_particles.length, 14.875);
        assert_eq!(unit_land_particles.radius_base, 0.3);
        assert_eq!(unit_land_particles.radius_fout_scale, 4.0);

        let unit_dust = standard_effect_draw_plan(
            Some(FX_UNIT_DUST_ID as u16),
            40,
            0.0,
            0.0,
            45.0,
            15.0,
            30.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(unit_dust.color_mul, 1.3);
        let unit_dust_particles = unit_dust.particles.unwrap();
        assert_eq!(unit_dust_particles.count, 3);
        assert_eq!(unit_dust_particles.length, 7.0);
        assert_eq!(unit_dust_particles.angle, Some(45.0));
        assert_eq!(unit_dust_particles.angle_range, 30.0);
        assert_eq!(unit_dust_particles.radius_fout_scale, 3.0);

        let unit_land_small = standard_effect_draw_plan(
            Some(FX_UNIT_LAND_SMALL_ID as u16),
            41,
            0.0,
            0.0,
            2.0,
            15.0,
            30.0,
            DecalColor::WHITE,
        )
        .unwrap();
        let unit_land_small_particles = unit_land_small.particles.unwrap();
        assert_eq!(unit_land_small_particles.count, 12);
        assert_eq!(unit_land_small_particles.length, 21.0);
        assert_eq!(unit_land_small_particles.radius_base, 0.1);
        assert_eq!(unit_land_small_particles.radius_fout_scale, 3.0);

        let crawl_dust = standard_effect_draw_plan(
            Some(FX_CRAWL_DUST_ID as u16),
            43,
            0.0,
            0.0,
            0.0,
            17.5,
            35.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(crawl_dust.color_mul, 1.6);
        let crawl_dust_particles = crawl_dust.particles.unwrap();
        assert_eq!(crawl_dust_particles.count, 2);
        assert_eq!(crawl_dust_particles.length, 8.75);
        assert_eq!(crawl_dust_particles.radius_base, 0.3);
        assert_eq!(crawl_dust_particles.radius_fslope_scale, 4.0);

        let smoke_aoe = standard_effect_draw_plan(
            Some(FX_SMOKE_AOE_CLOUD_ID as u16),
            57,
            3.0,
            4.0,
            0.0,
            90.0,
            180.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert_eq!(
            smoke_aoe.kind,
            StandardEffectDrawKind::SeededCircleParticles
        );
        assert_eq!(smoke_aoe.input_color, Some(DecalColor::WHITE));
        assert_eq!(smoke_aoe.alpha, 0.65);
        let smoke_aoe_particles = smoke_aoe.particles.unwrap();
        assert_eq!(smoke_aoe_particles.count, 80);
        assert_eq!(smoke_aoe_particles.length, 90.0);
        assert_eq!(smoke_aoe_particles.radius_base, 6.0);
        assert_eq!(smoke_aoe.circle_render_primitives_from_seed().len(), 80);

        let burning = standard_effect_draw_plan(
            Some(FX_BURNING_ID as u16),
            54,
            0.0,
            0.0,
            0.0,
            17.5,
            35.0,
            DecalColor::WHITE,
        )
        .unwrap();
        let burning_particles = burning.particles.unwrap();
        assert_eq!(burning.kind, StandardEffectDrawKind::SeededCircleParticles);
        assert_eq!(burning_particles.count, 3);
        assert_eq!(burning_particles.length, 5.5);
        assert_eq!(burning_particles.radius_base, 0.1);
        assert_eq!(burning_particles.radius_fout_scale, 1.4);

        let fire_hit = standard_effect_draw_plan(
            Some(FX_FIRE_HIT_ID as u16),
            55,
            0.0,
            0.0,
            0.0,
            17.5,
            35.0,
            DecalColor::WHITE,
        )
        .unwrap();
        let fire_hit_particles = fire_hit.particles.unwrap();
        assert_eq!(fire_hit_particles.count, 3);
        assert_eq!(fire_hit_particles.length, 7.0);
        assert_eq!(fire_hit_particles.radius_base, 0.2);
        assert_eq!(fire_hit_particles.radius_fout_scale, 1.6);

        let blast = standard_effect_draw_plan(
            Some(FX_BLAST_SMOKE_ID as u16),
            56,
            0.0,
            0.0,
            0.0,
            13.0,
            26.0,
            DecalColor::WHITE,
        )
        .unwrap();
        let blast_particles = blast.particles.unwrap();
        assert_eq!(blast_particles.count, 12);
        assert_eq!(blast_particles.length, 12.5);
        assert_eq!(blast_particles.radius_base, 1.0);
        assert_eq!(blast_particles.radius_fout_scale, 3.0);
        let blast_primitives = blast.circle_render_primitives_from_seed();
        assert_eq!(blast_primitives.len(), 12);
        assert_eq!(blast_primitives[0].radius, 2.5);
    }

    #[test]
    fn standard_effect_particle_plan_expands_to_circle_primitives() {
        let fire = standard_effect_draw_plan(
            Some(FX_FIRE_ID as u16),
            42,
            10.0,
            20.0,
            0.0,
            25.0,
            50.0,
            DecalColor::WHITE,
        )
        .unwrap();
        let fire_circles = fire.expand_seeded_particle_circles(&[
            StandardEffectParticleVector {
                x: 1.0,
                y: 2.0,
                fin: 0.0,
                fout: 1.0,
            },
            StandardEffectParticleVector {
                x: -3.0,
                y: 4.0,
                fin: 1.0,
                fout: 0.0,
            },
        ]);
        assert_eq!(fire_circles.len(), 2);
        assert_eq!(fire_circles[0].center, (11.0, 22.0));
        assert_eq!(fire_circles[0].radius, 1.7);
        assert_eq!(fire_circles[0].alpha, 1.0);
        assert_eq!(fire_circles[1].center, (7.0, 24.0));
        assert_eq!(fire_circles[1].radius, 1.7);

        let seeded_fire_vectors = fire.seeded_particle_vectors();
        assert_eq!(seeded_fire_vectors.len(), 2);
        assert!(
            (seeded_fire_vectors[0].x - 2.0617113).abs() < 0.00001,
            "{seeded_fire_vectors:?}"
        );
        assert!((seeded_fire_vectors[0].y - 5.4725294).abs() < 0.00001);
        assert!((seeded_fire_vectors[1].x - 0.56237954).abs() < 0.00001);
        assert!((seeded_fire_vectors[1].y - 0.88172233).abs() < 0.00001);
        let seeded_fire_circles = fire.expand_seeded_particle_circles_from_seed();
        assert_eq!(seeded_fire_circles.len(), 2);
        assert!((seeded_fire_circles[0].center.0 - 12.061711).abs() < 0.00001);
        assert!((seeded_fire_circles[0].center.1 - 25.472529).abs() < 0.00001);
        assert_eq!(seeded_fire_circles[0].radius, 1.7);

        let cloud = standard_effect_draw_plan(
            Some(FX_SMOKE_CLOUD_ID as u16),
            47,
            0.0,
            0.0,
            0.0,
            35.0,
            70.0,
            DecalColor::WHITE,
        )
        .unwrap();
        let cloud_circles = cloud.expand_seeded_particle_circles(&[
            StandardEffectParticleVector {
                x: 5.0,
                y: 6.0,
                fin: 0.5,
                fout: 0.5,
            },
            StandardEffectParticleVector {
                x: 7.0,
                y: 8.0,
                fin: 1.0,
                fout: 0.0,
            },
        ]);
        assert_eq!(cloud_circles.len(), 2);
        assert_eq!(cloud_circles[0].center, (5.0, 6.0));
        assert_eq!(cloud_circles[0].radius, 2.5);
        assert_eq!(cloud_circles[0].alpha, 1.0);
        assert_eq!(cloud_circles[1].center, (7.0, 8.0));
        assert_eq!(cloud_circles[1].radius, 0.5);
        assert_eq!(cloud_circles[1].alpha, 0.0);

        let seeded_cloud_vectors = cloud.seeded_particle_vectors();
        assert_eq!(seeded_cloud_vectors.len(), 30);
        assert!((seeded_cloud_vectors[0].x + 1.9581623).abs() < 0.00001);
        assert!((seeded_cloud_vectors[0].y + 0.15539533).abs() < 0.00001);
        assert!((seeded_cloud_vectors[0].fin - 0.06547728).abs() < 0.00001);
        assert!((seeded_cloud_vectors[0].fout - 0.06547728).abs() < 0.00001);
        assert!((seeded_cloud_vectors[1].x - 0.50366443).abs() < 0.00001);
        assert!((seeded_cloud_vectors[1].y - 2.8928096).abs() < 0.00001);
        assert!((seeded_cloud_vectors[1].fin - 0.09787762).abs() < 0.00001);
        assert!((seeded_cloud_vectors[1].fout - 0.09787762).abs() < 0.00001);

        let smoke_puff = standard_effect_draw_plan(
            Some(FX_SMOKE_PUFF_ID as u16),
            154,
            3.0,
            4.0,
            0.0,
            15.0,
            30.0,
            DecalColor::WHITE,
        )
        .unwrap();
        let smoke_puff_circles = smoke_puff.expand_seeded_particle_circles(&[
            StandardEffectParticleVector {
                x: 4.0,
                y: 6.0,
                fin: 0.5,
                fout: 0.5,
            },
            StandardEffectParticleVector {
                x: -2.0,
                y: 8.0,
                fin: 0.5,
                fout: 0.5,
            },
        ]);
        assert_eq!(smoke_puff_circles.len(), 4);
        assert_eq!(smoke_puff_circles[0].center, (7.0, 10.0));
        assert_eq!(smoke_puff_circles[0].radius, 1.5);
        assert_eq!(smoke_puff_circles[1].center, (5.0, 7.0));
        assert_eq!(smoke_puff_circles[1].radius, 0.5);
        assert_eq!(smoke_puff_circles[2].center, (1.0, 12.0));
        assert_eq!(smoke_puff_circles[3].center, (2.0, 8.0));
        assert_eq!(smoke_puff.circle_render_primitives_from_seed().len(), 12);

        let shoot_small_smoke = standard_effect_draw_plan(
            Some(FX_SHOOT_SMALL_SMOKE_ID as u16),
            159,
            0.0,
            0.0,
            45.0,
            10.0,
            20.0,
            DecalColor::WHITE,
        )
        .unwrap();
        let shoot_small_smoke_vectors = shoot_small_smoke.seeded_particle_vectors();
        assert_eq!(shoot_small_smoke_vectors.len(), 5);
        assert!((shoot_small_smoke_vectors[0].x - 0.34184948).abs() < 0.00001);
        assert!((shoot_small_smoke_vectors[0].y - 0.61245298).abs() < 0.00001);
        assert!((shoot_small_smoke_vectors[1].x - 1.5068227).abs() < 0.00001);
        assert!((shoot_small_smoke_vectors[1].y - 1.0755522).abs() < 0.00001);
        let shoot_small_smoke_circles =
            shoot_small_smoke.expand_seeded_particle_circles(&shoot_small_smoke_vectors);
        assert_eq!(shoot_small_smoke_circles.len(), 5);
        assert!((shoot_small_smoke_circles[0].center.0 - 0.34184948).abs() < 0.00001);
        assert!((shoot_small_smoke_circles[0].center.1 - 0.61245298).abs() < 0.00001);
        assert_eq!(shoot_small_smoke_circles[0].radius, 0.75);

        let ripple = standard_effect_draw_plan(
            Some(FX_RIPPLE_ID as u16),
            9,
            0.0,
            0.0,
            1.0,
            0.0,
            30.0,
            DecalColor::WHITE,
        )
        .unwrap();
        assert!(ripple
            .expand_seeded_particle_circles(&[StandardEffectParticleVector {
                x: 1.0,
                y: 1.0,
                fin: 0.5,
                fout: 0.5,
            }])
            .is_empty());
    }

    #[test]
    fn standard_effect_plan_resolves_circle_render_primitives_from_seed() {
        let smoke = standard_effect_draw_plan(
            Some(FX_SMOKE_ID as u16),
            7,
            10.0,
            20.0,
            0.0,
            50.0,
            100.0,
            DecalColor::WHITE,
        )
        .unwrap();
        let smoke_primitives = smoke.circle_render_primitives_from_seed();
        assert_eq!(smoke_primitives.len(), 1);
        assert_eq!(
            smoke_primitives[0].kind,
            StandardEffectDrawKind::FilledCircle
        );
        assert_eq!(smoke_primitives[0].center, (10.0, 20.0));
        assert_eq!(smoke_primitives[0].radius, 1.75);
        assert_eq!(smoke_primitives[0].stroke, 0.0);
        assert_eq!(smoke_primitives[0].alpha, 1.0);
        let smoke_color = smoke_primitives[0].color.unwrap();
        assert!((smoke_color.r - 0.3990196).abs() < 0.0001);
        assert!((smoke_color.g - 0.3990196).abs() < 0.0001);
        assert!((smoke_color.b - 0.3990196).abs() < 0.0001);
        assert_eq!(smoke_color.a, 1.0);

        let ripple = standard_effect_draw_plan(
            Some(FX_RIPPLE_ID as u16),
            9,
            3.0,
            4.0,
            2.0,
            15.0,
            30.0,
            DecalColor::WHITE,
        )
        .unwrap();
        let ripple_primitives = ripple.circle_render_primitives_from_seed();
        assert_eq!(ripple_primitives.len(), 1);
        assert_eq!(
            ripple_primitives[0].kind,
            StandardEffectDrawKind::StrokedCircle
        );
        assert!((ripple_primitives[0].radius - 6.0).abs() < 0.0001);
        assert!((ripple_primitives[0].stroke - 1.05).abs() < 0.0001);

        let fire = standard_effect_draw_plan(
            Some(FX_FIRE_ID as u16),
            42,
            10.0,
            20.0,
            0.0,
            25.0,
            50.0,
            DecalColor::WHITE,
        )
        .unwrap();
        let fire_primitives = fire.circle_render_primitives_from_seed();
        assert_eq!(fire_primitives.len(), 2);
        assert_eq!(
            fire_primitives[0].kind,
            StandardEffectDrawKind::FilledCircle
        );
        assert!((fire_primitives[0].center.0 - 12.061711).abs() < 0.00001);
        assert!((fire_primitives[0].center.1 - 25.472529).abs() < 0.00001);
        assert_eq!(fire_primitives[0].radius, 1.7);
        assert_eq!(fire_primitives[0].stroke, 0.0);
        assert_eq!(fire_primitives[0].alpha, 1.0);
        let fire_color = fire_primitives[0].color.unwrap();
        assert!((fire_color.r - (1.0 + 0xdb as f32 / 255.0) / 2.0).abs() < 0.0001);
        assert!((fire_color.g - (0xdd as f32 / 255.0 + 0x40 as f32 / 255.0) / 2.0).abs() < 0.0001);
        assert!((fire_color.b - (0x55 as f32 / 255.0 + 0x1c as f32 / 255.0) / 2.0).abs() < 0.0001);

        assert_eq!(
            fire.light_render_primitives(),
            vec![StandardEffectLightRenderPrimitive {
                center: (10.0, 20.0),
                radius: 20.0,
                color: "Pal.lightFlame",
                color_rgba: Some(DecalColor::from_rgba(0xffdd55ff)),
                opacity: 0.5,
            }]
        );
        assert!(smoke.light_render_primitives().is_empty());
    }

    #[test]
    fn effect_defaults_and_builder_methods_match_java_shape() {
        let effect = Effect::with_lifetime(3, 20.0, 40.0)
            .start_delay(5.0)
            .follow_parent(false)
            .rot_with_parent(true)
            .layer_duration(7.0, 9.0)
            .base_rotation(15.0);

        assert_eq!(effect.id, 3);
        assert_eq!(effect.lifetime, 20.0);
        assert_eq!(effect.clip, 40.0);
        assert_eq!(effect.start_delay, 5.0);
        assert!(!effect.follow_parent);
        assert!(effect.rot_with_parent);
        assert_eq!(effect.layer, 7.0);
        assert_eq!(effect.layer_duration, 9.0);
        assert_eq!(effect.base_rotation, 15.0);
    }

    #[test]
    fn create_plan_checks_headless_none_effects_camera_and_initializes_once() {
        let mut effect = Effect::with_lifetime(1, 30.0, 50.0).start_delay(2.0);

        assert!(effect
            .create_plan(
                1.0,
                2.0,
                3.0,
                DecalColor::WHITE,
                None,
                None,
                EffectCreateContext {
                    headless: true,
                    ..EffectCreateContext::default()
                },
            )
            .is_none());

        let plan = effect
            .create_plan(
                1.0,
                2.0,
                3.0,
                DecalColor::WHITE,
                Some("payload".into()),
                None,
                EffectCreateContext::default(),
            )
            .unwrap();

        assert!(plan.initialized_now);
        assert_eq!(plan.delay, 2.0);
        assert_eq!(plan.spawn.effect_id, 1);
        assert_eq!(plan.spawn.lifetime, 30.0);
        assert_eq!(plan.spawn.clip, 50.0);
        assert_eq!(plan.spawn.data.as_deref(), Some("payload"));

        let second = effect
            .create_plan(
                0.0,
                0.0,
                0.0,
                DecalColor::WHITE,
                None,
                None,
                EffectCreateContext::default(),
            )
            .unwrap();
        assert!(!second.initialized_now);

        assert!(effect
            .create_plan(
                0.0,
                0.0,
                0.0,
                DecalColor::WHITE,
                None,
                None,
                EffectCreateContext {
                    camera_overlaps: false,
                    ..EffectCreateContext::default()
                },
            )
            .is_none());
    }

    #[test]
    fn create_plan_applies_base_rotation_and_parent_flags() {
        let mut effect = Effect::with_lifetime(2, 10.0, 20.0)
            .base_rotation(30.0)
            .rot_with_parent(true);

        let plan = effect
            .create_plan(
                5.0,
                6.0,
                15.0,
                DecalColor::WHITE,
                None,
                Some(EffectParent { id: 99 }),
                EffectCreateContext::default(),
            )
            .unwrap();

        assert_eq!(plan.spawn.rotation, 45.0);
        assert_eq!(plan.spawn.parent_id, Some(99));
        assert!(plan.spawn.rot_with_parent);

        effect.follow_parent = false;
        let no_parent = effect
            .create_plan(
                5.0,
                6.0,
                15.0,
                DecalColor::WHITE,
                None,
                Some(EffectParent { id: 99 }),
                EffectCreateContext::default(),
            )
            .unwrap();
        assert_eq!(no_parent.spawn.parent_id, None);
        assert!(!no_parent.spawn.rot_with_parent);
    }

    #[test]
    fn effect_container_fin_scaled_and_render_lifetime_are_data_only() {
        let effect = Effect::with_lifetime(3, 10.0, 20.0);
        let params = EffectRenderParams {
            id: 7,
            color: DecalColor::WHITE,
            time: 5.0,
            lifetime: 10.0,
            rotation: 90.0,
            x: 1.0,
            y: 2.0,
            data: Some("data".into()),
        };

        let (container, lifetime) = effect.render_with(params, |container| {
            assert_eq!(container.fin(), 0.5);
            assert_eq!(container.fout(), 0.5);
            assert_eq!(container.finpow(), 0.875);
            assert_eq!(container.fslope(), 1.0);
            container.lifetime = 12.0;
        });

        assert_eq!(lifetime, 12.0);
        assert_eq!(container.scaled(6.0).unwrap().lifetime, 6.0);
        assert!(container.scaled(4.0).is_none());
    }

    #[test]
    fn effect_registry_assigns_ids_and_get_handles_invalid_ids() {
        let mut registry = EffectRegistry::new();

        assert_eq!(registry.create(10.0, 20.0), 0);
        assert_eq!(registry.push(Effect::with_lifetime(99, 30.0, 40.0)), 1);
        assert_eq!(registry.len(), 2);
        assert_eq!(registry.get(0).unwrap().id, 0);
        assert_eq!(registry.get(1).unwrap().id, 1);
        assert!(registry.get(-1).is_none());
        assert!(registry.get(99).is_none());
    }

    #[test]
    fn multi_effect_creates_all_child_effects_without_rendering_itself() {
        let child_a = Effect::with_lifetime(1, 10.0, 20.0).start_delay(2.0);
        let child_b = Effect::with_lifetime(2, 30.0, 40.0).base_rotation(5.0);
        let mut multi = MultiEffect::with_effects(vec![child_a, child_b]);

        let plans = multi.create_plans(
            7.0,
            8.0,
            9.0,
            DecalColor::WHITE,
            Some("payload".into()),
            EffectCreateContext::default(),
        );

        assert_eq!(plans.len(), 2);
        assert_eq!(plans[0].delay, 2.0);
        assert_eq!(plans[0].spawn.effect_id, 1);
        assert_eq!(plans[0].spawn.x, 7.0);
        assert_eq!(plans[0].spawn.y, 8.0);
        assert_eq!(plans[0].spawn.rotation, 9.0);
        assert_eq!(plans[0].spawn.data.as_deref(), Some("payload"));
        assert_eq!(plans[1].spawn.effect_id, 2);
        assert_eq!(plans[1].spawn.rotation, 14.0);
        assert!(plans[0].initialized_now);
        assert!(plans[1].initialized_now);

        let blocked = multi.create_plans(
            0.0,
            0.0,
            0.0,
            DecalColor::WHITE,
            None,
            EffectCreateContext {
                enable_effects: false,
                ..EffectCreateContext::default()
            },
        );
        assert!(blocked.is_empty());
    }

    #[test]
    fn seq_effect_sums_lifetime_clip_and_selects_child_by_time() {
        let child_a = Effect::with_lifetime(1, 10.0, 20.0);
        let child_b = Effect::with_lifetime(2, 30.0, 140.0);
        let mut seq = SeqEffect::with_effects(vec![child_a, child_b]);
        assert_eq!(seq.base.clip, 100.0);

        seq.init_defaults();
        assert_eq!(seq.base.lifetime, 40.0);
        assert_eq!(seq.base.clip, 140.0);

        let first = seq
            .render_plan(EffectRenderParams {
                id: 5,
                color: DecalColor::WHITE,
                time: 8.0,
                lifetime: 40.0,
                rotation: 45.0,
                x: 1.0,
                y: 2.0,
                data: Some("seq".into()),
            })
            .expect("first child should render");
        assert_eq!(first.child_index, 0);
        assert_eq!(first.params.id, 5);
        assert_eq!(first.params.time, 8.0);
        assert_eq!(first.params.lifetime, 10.0);
        assert_eq!(first.params.data.as_deref(), Some("seq"));

        let second = seq
            .render_plan(EffectRenderParams {
                id: 5,
                color: DecalColor::WHITE,
                time: 12.0,
                lifetime: 40.0,
                rotation: 45.0,
                x: 1.0,
                y: 2.0,
                data: None,
            })
            .expect("second child should render");
        assert_eq!(second.child_index, 1);
        assert_eq!(second.params.id, 6);
        assert_eq!(second.params.time, 2.0);
        assert_eq!(second.params.lifetime, 30.0);

        assert!(seq
            .render_plan(EffectRenderParams {
                id: 5,
                color: DecalColor::WHITE,
                time: 45.0,
                lifetime: 40.0,
                rotation: 0.0,
                x: 0.0,
                y: 0.0,
                data: None,
            })
            .is_none());
    }

    #[test]
    fn wrap_effect_syncs_child_lifetime_and_forwards_fixed_color_rotation() {
        let child = Effect::with_lifetime(4, 33.0, 77.0).base_rotation(5.0);
        let color = DecalColor {
            r: 0.2,
            g: 0.4,
            b: 0.6,
            a: 0.8,
        };
        let mut wrap = WrapEffect::new(child, color, 90.0);

        wrap.init_defaults();
        assert_eq!(wrap.base.lifetime, 33.0);
        assert_eq!(wrap.base.clip, 77.0);

        let plan = wrap
            .create_plan(
                3.0,
                4.0,
                Some("wrapped".into()),
                EffectCreateContext::default(),
            )
            .expect("wrapped child should create");
        assert_eq!(plan.spawn.effect_id, 4);
        assert_eq!(plan.spawn.x, 3.0);
        assert_eq!(plan.spawn.y, 4.0);
        assert_eq!(plan.spawn.rotation, 95.0);
        assert_eq!(plan.spawn.color, color);
        assert_eq!(plan.spawn.data.as_deref(), Some("wrapped"));
    }

    #[test]
    fn radial_effect_repeats_child_create_at_angle_intervals() {
        let child = Effect::with_lifetime(9, 10.0, 20.0);
        let mut radial = RadialEffect::new(child, 4, 90.0, 10.0, 5.0);
        radial.rotation_offset = 0.0;

        let plans = radial.create_plans(
            1.0,
            2.0,
            0.0,
            DecalColor::WHITE,
            Some("radial".into()),
            EffectCreateContext::default(),
        );

        assert_eq!(plans.len(), 4);
        assert_eq!(plans[0].spawn.effect_id, 9);
        assert!((plans[0].spawn.x - 11.0).abs() < 0.0001);
        assert!((plans[0].spawn.y - 2.0).abs() < 0.0001);
        assert_eq!(plans[0].spawn.rotation, 5.0);
        assert!((plans[1].spawn.x - 1.0).abs() < 0.0001);
        assert!((plans[1].spawn.y - 12.0).abs() < 0.0001);
        assert_eq!(plans[1].spawn.rotation, 95.0);
        assert!((plans[2].spawn.x + 9.0).abs() < 0.0001);
        assert!((plans[2].spawn.y - 2.0).abs() < 0.0001);
        assert_eq!(plans[2].spawn.rotation, 185.0);
        assert!((plans[3].spawn.x - 1.0).abs() < 0.0001);
        assert!((plans[3].spawn.y + 8.0).abs() < 0.0001);
        assert_eq!(plans[3].spawn.rotation, 275.0);
        assert_eq!(plans[3].spawn.data.as_deref(), Some("radial"));

        radial.amount = 0;
        assert!(radial
            .create_plans(
                0.0,
                0.0,
                0.0,
                DecalColor::WHITE,
                None,
                EffectCreateContext::default(),
            )
            .is_empty());
    }

    #[test]
    fn sound_effect_inherits_delay_and_records_sound_plus_child_effect() {
        let child = Effect::with_lifetime(12, 20.0, 30.0).start_delay(3.0);
        let mut sound = SoundEffect::new("boom", child);
        sound.min_pitch = 0.5;
        sound.max_pitch = 1.5;
        sound.min_volume = 0.25;
        sound.max_volume = 0.75;

        assert_eq!(sound.base.start_delay, -1.0);
        sound.init_defaults();
        assert_eq!(sound.base.start_delay, 3.0);

        let plan = sound
            .create_plan(
                4.0,
                5.0,
                6.0,
                DecalColor::WHITE,
                Some("sound".into()),
                0.25,
                0.5,
                EffectCreateContext::default(),
            )
            .expect("sound effect should create");
        assert_eq!(plan.sound.sound, "boom");
        assert_eq!(plan.sound.x, 4.0);
        assert_eq!(plan.sound.y, 5.0);
        assert_eq!(plan.sound.delay, 3.0);
        assert_eq!(plan.sound.pitch, 0.75);
        assert_eq!(plan.sound.volume, 0.5);

        let child = plan.effect.expect("child effect should also create");
        assert_eq!(child.spawn.effect_id, 12);
        assert_eq!(child.spawn.rotation, 6.0);
        assert_eq!(child.spawn.data.as_deref(), Some("sound"));

        assert!(sound
            .create_plan(
                0.0,
                0.0,
                0.0,
                DecalColor::WHITE,
                None,
                0.0,
                0.0,
                EffectCreateContext {
                    headless: true,
                    ..EffectCreateContext::default()
                },
            )
            .is_none());
    }

    #[test]
    fn wave_effect_init_and_draw_plan_follow_java_radius_stroke_light_math() {
        let mut wave = WaveEffect::default();
        wave.color_from = "from".into();
        wave.color_to = "to".into();
        wave.light_color = Some("light".into());
        wave.sides = 6;
        wave.rotation = 15.0;
        wave.offset_x = 10.0;
        wave.offset_y = 0.0;

        wave.init_defaults();
        assert_eq!(wave.base.clip, 102.0);

        let draw = wave.draw_plan(&EffectRenderParams {
            id: 1,
            color: DecalColor::WHITE,
            time: 25.0,
            lifetime: 100.0,
            rotation: 90.0,
            x: 1.0,
            y: 2.0,
            data: None,
        });
        assert!((draw.center.0 - 1.0).abs() < 0.0001);
        assert!((draw.center.1 - 12.0).abs() < 0.0001);
        assert_eq!(draw.color_from, "from");
        assert_eq!(draw.color_to, "to");
        assert_eq!(draw.color_mix, 0.25);
        assert_eq!(draw.stroke, 1.5);
        assert_eq!(draw.radius, 25.0);
        assert_eq!(draw.sides, 6);
        assert_eq!(draw.rotation, 105.0);
        assert_eq!(draw.light_radius, 75.0);
        assert_eq!(draw.light_color, "light");
        assert_eq!(draw.light_opacity, 0.6);
    }

    #[test]
    fn explosion_effect_draw_plan_covers_wave_smoke_and_sparks() {
        let explosion = ExplosionEffect::default();
        assert_eq!(explosion.base.clip, 100.0);
        assert_eq!(explosion.base.lifetime, 22.0);
        assert_eq!(explosion.wave_color, "missileYellow");
        assert_eq!(explosion.smoke_color, "gray");
        assert_eq!(explosion.spark_color, "missileYellowBack");
        assert_eq!(explosion.wave_life, 6.0);
        assert_eq!(explosion.smokes, 5);
        assert_eq!(explosion.sparks, 4);

        let container = EffectContainer {
            x: 10.0,
            y: 20.0,
            time: 0.0,
            lifetime: 22.0,
            rotation: 0.0,
            color: DecalColor::WHITE,
            id: 7,
            data: None,
        };
        let plan = explosion.draw_plan(&container, &[(1.0, 0.0), (0.0, 2.0)], &[(3.0, 4.0)]);
        assert_eq!(
            plan.wave,
            Some(ExplosionWavePlan {
                stroke: 3.0,
                radius: 2.0,
            })
        );
        assert_eq!(plan.smoke_vector_radius, 2.0);
        assert_eq!(plan.spark_vector_radius, 1.0);
        assert_eq!(
            plan.smokes,
            vec![
                ExplosionSmokePlan {
                    x: 11.0,
                    y: 20.0,
                    radius: 4.5,
                },
                ExplosionSmokePlan {
                    x: 10.0,
                    y: 22.0,
                    radius: 4.5,
                },
            ]
        );
        assert_eq!(plan.sparks.len(), 1);
        assert_eq!(plan.sparks[0].x, 13.0);
        assert_eq!(plan.sparks[0].y, 24.0);
        assert_eq!(plan.sparks[0].stroke, 1.0);
        assert!((plan.sparks[0].angle - 53.130104).abs() < 0.0001);
        assert_eq!(plan.sparks[0].length, 4.0);
        assert_eq!(plan.sparks[0].light_radius, 12.0);
        assert_eq!(plan.sparks[0].light_opacity, 0.7);
    }

    #[test]
    fn particle_effect_init_and_draw_plan_cover_sprite_and_line_modes() {
        let mut particle = ParticleEffect::default();
        assert_eq!(particle.color_from, "white");
        assert_eq!(particle.color_to, "white");
        assert_eq!(particle.particles, 6);
        assert!(particle.rand_length);
        assert_eq!(particle.cone, 180.0);
        assert_eq!(particle.length, 20.0);
        assert_eq!(particle.light_scl, 2.0);
        assert_eq!(particle.size_from, 2.0);
        assert_eq!(particle.size_to, 0.0);
        assert_eq!(particle.region, "circle");
        assert!(!particle.line);
        particle.init_defaults();
        assert_eq!(particle.base.clip, 22.0);
        assert_eq!(particle.size_interp, Some(EffectInterp::Linear));

        let params = EffectRenderParams {
            id: 1,
            color: DecalColor::WHITE,
            time: 25.0,
            lifetime: 50.0,
            rotation: 30.0,
            x: 0.0,
            y: 0.0,
            data: None,
        };
        let sprite = particle.draw_plan(
            &params,
            &[ParticleVectorInput {
                angle_offset: 0.0,
                length_factor: 1.0,
            }],
            2.0,
        );
        assert_eq!(sprite.color_mix, 0.5);
        assert_eq!(sprite.requested_length, 10.0);
        assert!((sprite.particles[0].x - 8.660254).abs() < 0.0001);
        assert!((sprite.particles[0].y - 5.0).abs() < 0.0001);
        assert_eq!(
            sprite.particles[0].kind,
            ParticleDrawKind::Sprite {
                region: "circle".into(),
                width: 2.0,
                height: 1.0,
                rotation: 30.0,
            }
        );
        assert_eq!(sprite.particles[0].light_radius, 4.0);
        assert_eq!(sprite.particles[0].light_opacity, 0.6);

        particle.line = true;
        particle.rand_length = false;
        particle.use_rotation = false;
        particle.base.base_rotation = 90.0;
        let line = particle.draw_plan(
            &params,
            &[ParticleVectorInput {
                angle_offset: 0.0,
                length_factor: 0.25,
            }],
            1.0,
        );
        assert!((line.particles[0].x).abs() < 0.0001);
        assert!((line.particles[0].y - 10.0).abs() < 0.0001);
        assert_eq!(
            line.particles[0].kind,
            ParticleDrawKind::Line {
                stroke: 1.0,
                length: 3.0,
                angle: 90.0,
                cap: true,
            }
        );
        assert_eq!(line.particles[0].light_radius, 6.0);
    }

    #[test]
    fn shake_intensity_falls_off_with_camera_distance() {
        assert_eq!(shake_intensity(5.0, 0.0, 0.0, 0.0, 0.0), 5.0);
        assert_eq!(shake_intensity(5.0, 0.0, 0.0, 100.0, 0.0), 5.0);
        assert_eq!(shake_intensity(8.0, 0.0, 0.0, 200.0, 0.0), 2.0);
    }
}
