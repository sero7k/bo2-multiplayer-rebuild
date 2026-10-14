//! Generated: MSVC x64 record layouts of the x64-zt IW7 headers
//! (`clang++ --target=x86_64-pc-windows-msvc -Xclang -fdump-record-layouts`).
//! Offsets in bytes; `SIZE` is `sizeof`. Do not edit by hand.
#![allow(non_snake_case, non_upper_case_globals, dead_code)]

pub mod ADSOverlay {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const shader: usize = 0;
    pub const shaderLowRes: usize = 8;
    pub const shaderEMP: usize = 16;
    pub const shaderEMPLowRes: usize = 24;
    pub const reticle: usize = 32;
    pub const width: usize = 36;
    pub const height: usize = 40;
    pub const widthSplitscreen: usize = 44;
    pub const heightSplitscreen: usize = 48;
}
pub mod ADSRSetting {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const id: usize = 8;
    pub const attackLength: usize = 12;
    pub const attackCurve: usize = 16;
    pub const decayLength: usize = 20;
    pub const decayCurve: usize = 24;
    pub const releaseLength: usize = 28;
    pub const releaseCurve: usize = 32;
    pub const sustainLevel: usize = 36;
    pub const sustainLength: usize = 40;
}
pub mod AddonMapEnts {
    pub const SIZE: usize = 136;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const entityString: usize = 8;
    pub const numEntityChars: usize = 16;
    pub const trigger: usize = 24;
    pub const trigger__count: usize = 24;
    pub const trigger__models: usize = 32;
    pub const trigger__hullCount: usize = 40;
    pub const trigger__hulls: usize = 48;
    pub const trigger__slabCount: usize = 56;
    pub const trigger__slabs: usize = 64;
    pub const trigger__windingCount: usize = 72;
    pub const trigger__windings: usize = 80;
    pub const trigger__windingPointCount: usize = 88;
    pub const trigger__windingPoints: usize = 96;
    pub const info: usize = 104;
    pub const numSubModels: usize = 112;
    pub const cmodels: usize = 120;
    pub const models: usize = 128;
}
pub mod AdsAltSwitch {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const fovInterpTimeSecMainToAlt: usize = 0;
    pub const fovInterpDelaySecMainToAlt: usize = 4;
    pub const dualFovSwitchRateMainToAlt: usize = 8;
    pub const fovInterpTypeMainToAlt: usize = 12;
    pub const fovInterpTimeSecAltToMain: usize = 16;
    pub const fovInterpDelaySecAltToMain: usize = 20;
    pub const dualFovSwitchRateAltToMain: usize = 24;
    pub const fovInterpTypeAltToMain: usize = 28;
}
pub mod AdsrZoneEntry {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const id: usize = 8;
    pub const weaponName: usize = 16;
    pub const adsrIdx: usize = 24;
    pub const weaponIdx: usize = 28;
}
pub mod AlwaysloadedFlags {
    pub const SIZE: usize = 72;
    pub const ALIGN: usize = 8;
    pub const imageCount: usize = 0;
    pub const images: usize = 8;
    pub const imageFlags: usize = 16;
    pub const stringCount: usize = 24;
    pub const strings: usize = 32;
    pub const stringFlags: usize = 40;
    pub const materialCount: usize = 48;
    pub const materials: usize = 56;
    pub const materialFlags: usize = 64;
}
pub mod AlwaysloadedFlagsPerModel {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 8;
    pub const model: usize = 0;
    pub const flags: usize = 8;
    pub const flags__imageCount: usize = 8;
    pub const flags__images: usize = 16;
    pub const flags__imageFlags: usize = 24;
    pub const flags__stringCount: usize = 32;
    pub const flags__strings: usize = 40;
    pub const flags__stringFlags: usize = 48;
    pub const flags__materialCount: usize = 56;
    pub const flags__materials: usize = 64;
    pub const flags__materialFlags: usize = 72;
}
pub mod AmbientDef {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const id: usize = 8;
    pub const ambientEventIndex: usize = 12;
    pub const numEvents: usize = 14;
    pub const timeMin: usize = 16;
    pub const timeMax: usize = 20;
}
pub mod AmbientElement {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const id: usize = 0;
    pub const unusedNameField: usize = 8;
    pub const aliasName: usize = 16;
    pub const rangeMin: usize = 24;
    pub const rangeMax: usize = 28;
    pub const coneMin: usize = 32;
    pub const coneMax: usize = 36;
}
pub mod AmbientEvent {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const id: usize = 0;
    pub const ambientElementIndex: usize = 4;
    pub const weight: usize = 8;
}
pub mod AnimAlias {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const aliasName: usize = 0;
    pub const animCount: usize = 4;
    pub const aliasInfo: usize = 8;
}
pub mod AnimAliasInfo {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const animIndex: usize = 0;
    pub const animWeight: usize = 4;
}
pub mod AnimOverride {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const attachment1: usize = 0;
    pub const attachment2: usize = 4;
    pub const overrides: usize = 8;
    pub const overridesAlt: usize = 16;
}
pub mod AnimationAimSet {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const rootName: usize = 4;
    pub const animCount: usize = 8;
    pub const animName: usize = 16;
    pub const rootIndex: usize = 24;
    pub const animIndices: usize = 32;
    pub const aimNodeIndices: usize = 40;
}
pub mod AnimationClass {
    pub const SIZE: usize = 88;
    pub const ALIGN: usize = 8;
    pub const className: usize = 0;
    pub const stateMachine: usize = 8;
    pub const animCtrl: usize = 16;
    pub const animTree: usize = 20;
    pub const scriptable: usize = 24;
    pub const soundCount: usize = 32;
    pub const effectCount: usize = 34;
    pub const soundNotes: usize = 40;
    pub const soundNames: usize = 48;
    pub const soundOptions: usize = 56;
    pub const effectNotes: usize = 64;
    pub const effectDefs: usize = 72;
    pub const effectTags: usize = 80;
}
pub mod AnimationEntry {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const animName: usize = 0;
    pub const aimSetIndex: usize = 4;
}
pub mod AnimationState {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const notify: usize = 4;
    pub const blendTime: usize = 8;
    pub const blendOutTime: usize = 12;
    pub const flags: usize = 16;
    pub const entryCount: usize = 17;
    pub const aliasCount: usize = 18;
    pub const aimSetIndex: usize = 20;
    pub const animEntries: usize = 24;
    pub const animIndices: usize = 32;
    pub const aliasList: usize = 40;
}
pub mod AnimationStateMachine {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const stateCount: usize = 4;
    pub const aimSetCount: usize = 6;
    pub const states: usize = 8;
    pub const aimSets: usize = 16;
}
pub mod AttADSOverlay {
    pub const SIZE: usize = 64;
    pub const ALIGN: usize = 8;
    pub const overlay: usize = 0;
    pub const overlay__shader: usize = 0;
    pub const overlay__shaderLowRes: usize = 8;
    pub const overlay__shaderEMP: usize = 16;
    pub const overlay__shaderEMPLowRes: usize = 24;
    pub const overlay__reticle: usize = 32;
    pub const overlay__width: usize = 36;
    pub const overlay__height: usize = 40;
    pub const overlay__widthSplitscreen: usize = 44;
    pub const overlay__heightSplitscreen: usize = 48;
    pub const hybridToggle: usize = 56;
    pub const thermalScope: usize = 57;
    pub const thermalToggle: usize = 58;
    pub const outlineEnemies: usize = 59;
}
pub mod AttADSSettings {
    pub const SIZE: usize = 88;
    pub const ALIGN: usize = 4;
    pub const adsSpread: usize = 0;
    pub const adsAimPitch: usize = 4;
    pub const adsTransInTime: usize = 8;
    pub const adsTransOutTime: usize = 12;
    pub const adsReloadTransTime: usize = 16;
    pub const adsCrosshairInFrac: usize = 20;
    pub const adsCrosshairOutFrac: usize = 24;
    pub const adsInCrosshairAlphaStart: usize = 28;
    pub const adsInCrosshairAlphaEnd: usize = 32;
    pub const adsOutCrosshairAlphaStart: usize = 36;
    pub const adsOutCrosshairAlphaEnd: usize = 40;
    pub const adsZoomFov: usize = 44;
    pub const adsZoomInFrac: usize = 48;
    pub const adsZoomOutFrac: usize = 52;
    pub const adsFovLerpTime: usize = 56;
    pub const adsBobFactor: usize = 60;
    pub const adsViewBobMult: usize = 64;
    pub const adsFireRateScale: usize = 68;
    pub const adsDamageRangeScale: usize = 72;
    pub const adsFireAnimFrac: usize = 76;
    pub const fireTimerLerpToAdsScale: usize = 80;
    pub const alwaysFireAtMaxRangeInAds: usize = 84;
    pub const adsAlignEnabled: usize = 85;
    pub const disableTagAlignX: usize = 86;
}
pub mod AttAddOns {
    pub const SIZE: usize = 3;
    pub const ALIGN: usize = 1;
    pub const motionTracker: usize = 0;
    pub const silenced: usize = 1;
    pub const riotShield: usize = 2;
}
pub mod AttAimAssist {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const autoAimRange: usize = 0;
    pub const aimAssistRange: usize = 4;
    pub const aimAssistRangeAds: usize = 8;
    pub const aimAssistPitchSlowdown: usize = 12;
    pub const aimAssistPitchSlowdownAds: usize = 16;
    pub const aimAssistYawSlowdown: usize = 20;
    pub const aimAssistYawSlowdownAds: usize = 24;
    pub const aimAssistLockonStrength: usize = 28;
}
pub mod AttAmmoGeneral {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 4;
    pub const vfxImpactType: usize = 0;
    pub const sfxImpactType: usize = 4;
    pub const meleeImpactType: usize = 8;
    pub const fireType: usize = 12;
    pub const burstCount: usize = 16;
}
pub mod AttAmmoTracer {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const tracerType: usize = 0;
    pub const tracerStyle: usize = 8;
}
pub mod AttAmmunition {
    pub const SIZE: usize = 28;
    pub const ALIGN: usize = 4;
    pub const maxAmmo: usize = 0;
    pub const startAmmo: usize = 4;
    pub const clipSize: usize = 8;
    pub const shotCount: usize = 12;
    pub const reloadAmmoAdd: usize = 16;
    pub const reloadStartAdd: usize = 20;
    pub const requireAmmoUsedPerShot: usize = 24;
}
pub mod AttBurst {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const burstFireCooldown: usize = 0;
    pub const burstFireAuto: usize = 4;
}
pub mod AttCharged {
    pub const SIZE: usize = 144;
    pub const ALIGN: usize = 8;
    pub const chargeType: usize = 0;
    pub const chargeGain: usize = 4;
    pub const chargeCostPerShot: usize = 8;
    pub const chargeLossWhenIdle: usize = 12;
    pub const chargeEmptyCooldown: usize = 16;
    pub const chargeFireAtMaxDamageMultiplier: usize = 20;
    pub const chargeMeterEffectBoneCount: usize = 24;
    pub const chargeMeterEffect: usize = 32;
    pub const chargeMeterEffect__u: usize = 32;
    pub const chargeMeterEffect__u__fx: usize = 32;
    pub const chargeMeterEffect__u__vfx: usize = 32;
    pub const chargeMeterEffect__u__data: usize = 32;
    pub const chargeMeterEffect__type: usize = 40;
    pub const chargeUpSound: usize = 48;
    pub const chargeDownSound: usize = 56;
    pub const chargeDownToUpSound: usize = 64;
    pub const chargeUpToDownSound: usize = 72;
    pub const chargeMaxSound: usize = 80;
    pub const chargeUpSoundPlayer: usize = 88;
    pub const chargeDownSoundPlayer: usize = 96;
    pub const chargeDownToUpSoundPlayer: usize = 104;
    pub const chargeUpToDownSoundPlayer: usize = 112;
    pub const chargeMaxSoundPlayer: usize = 120;
    pub const chargeHudReveal: usize = 128;
    pub const chargeRumble: usize = 136;
}
pub mod AttDamage {
    pub const SIZE: usize = 68;
    pub const ALIGN: usize = 4;
    pub const damage: usize = 0;
    pub const mid1Damage: usize = 4;
    pub const mid2Damage: usize = 8;
    pub const mid3Damage: usize = 12;
    pub const minDamage: usize = 16;
    pub const meleeDamage: usize = 20;
    pub const maxDamageRange: usize = 24;
    pub const mid1DamageRange: usize = 28;
    pub const mid2DamageRange: usize = 32;
    pub const mid3DamageRange: usize = 36;
    pub const minDamageRange: usize = 40;
    pub const playerDamage: usize = 44;
    pub const mid1PlayerDamage: usize = 48;
    pub const mid2PlayerDamage: usize = 52;
    pub const mid3PlayerDamage: usize = 56;
    pub const minPlayerDamage: usize = 60;
    pub const deathAnimDamageType: usize = 64;
}
pub mod AttGeneral {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const boltAction: usize = 0;
    pub const inheritsPerks: usize = 1;
    pub const reticleSpin45: usize = 2;
    pub const adsShouldShowCrosshair: usize = 3;
    pub const enemyCrosshairRange: usize = 4;
    pub const reticleType: usize = 8;
    pub const reticleCenter: usize = 16;
    pub const reticleSide: usize = 24;
    pub const reticleOnePiece: usize = 32;
    pub const reticleCenterSize: usize = 40;
    pub const reticleSideSize: usize = 44;
    pub const szLUICrosshairWidget: usize = 48;
}
pub mod AttGunKick {
    pub const SIZE: usize = 88;
    pub const ALIGN: usize = 4;
    pub const hipGunKickReducedKickBullets: usize = 0;
    pub const hipGunKickReducedKickPercent: usize = 4;
    pub const hipGunKickPitchMin: usize = 8;
    pub const hipGunKickPitchMax: usize = 12;
    pub const hipGunKickYawMin: usize = 16;
    pub const hipGunKickYawMax: usize = 20;
    pub const hipGunKickMagMin: usize = 24;
    pub const hipGunKickAccel: usize = 28;
    pub const hipGunKickSpeedMax: usize = 32;
    pub const hipGunKickSpeedDecay: usize = 36;
    pub const hipGunKickStaticDecay: usize = 40;
    pub const adsGunKickReducedKickBullets: usize = 44;
    pub const adsGunKickReducedKickPercent: usize = 48;
    pub const adsGunKickPitchMin: usize = 52;
    pub const adsGunKickPitchMax: usize = 56;
    pub const adsGunKickYawMin: usize = 60;
    pub const adsGunKickYawMax: usize = 64;
    pub const adsGunKickMagMin: usize = 68;
    pub const adsGunKickAccel: usize = 72;
    pub const adsGunKickSpeedMax: usize = 76;
    pub const adsGunKickSpeedDecay: usize = 80;
    pub const adsGunKickStaticDecay: usize = 84;
}
pub mod AttHipSpread {
    pub const SIZE: usize = 72;
    pub const ALIGN: usize = 4;
    pub const hipSpreadStandMin: usize = 0;
    pub const hipSpreadDuckedMin: usize = 4;
    pub const hipSpreadProneMin: usize = 8;
    pub const hipSpreadSprintMin: usize = 12;
    pub const hipSpreadInAirMin: usize = 16;
    pub const hipSpreadMax: usize = 20;
    pub const hipSpreadDuckedMax: usize = 24;
    pub const hipSpreadProneMax: usize = 28;
    pub const hipSpreadSprintMax: usize = 32;
    pub const hipSpreadInAirMax: usize = 36;
    pub const hipSpreadFireAdd: usize = 40;
    pub const hipSpreadTurnAdd: usize = 44;
    pub const hipSpreadMoveAdd: usize = 48;
    pub const hipSpreadDecayRate: usize = 52;
    pub const hipSpreadDuckedDecay: usize = 56;
    pub const hipSpreadProneDecay: usize = 60;
    pub const hipSpreadSprintDecay: usize = 64;
    pub const hipSpreadInAirDecay: usize = 68;
}
pub mod AttHoldBreath {
    pub const SIZE: usize = 1;
    pub const ALIGN: usize = 1;
    pub const canHoldBreath: usize = 0;
}
pub mod AttIdleSettings {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 4;
    pub const hipIdleAmount: usize = 0;
    pub const hipIdleSpeed: usize = 4;
    pub const idleCrouchFactor: usize = 8;
    pub const idleProneFactor: usize = 12;
    pub const adsIdleLerpStartTime: usize = 16;
    pub const adsIdleLerpTime: usize = 20;
}
pub mod AttLaser {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const laserTypeFriendly: usize = 0;
    pub const laserTypeEnemy: usize = 8;
    pub const forceLaserOn: usize = 16;
}
pub mod AttMovement {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const moveSpeedScale: usize = 0;
    pub const adsMoveSpeedScale: usize = 4;
    pub const fastADS3PAnims: usize = 8;
}
pub mod AttOutline {
    pub const SIZE: usize = 128;
    pub const ALIGN: usize = 4;
    pub const outlineColor: usize = 0;
    pub const outlineAltColor: usize = 16;
    pub const outlineFillColor0: usize = 32;
    pub const outlineFillColor1: usize = 48;
    pub const outlineOccludedOutlineColor: usize = 64;
    pub const outlineOccludedInlineColor: usize = 80;
    pub const outlineOccludedInteriorColor: usize = 96;
    pub const outlineWidth: usize = 112;
    pub const outlineEnemies: usize = 116;
    pub const outlineDepthTest: usize = 117;
    pub const outlineFill: usize = 118;
    pub const outlineScriptControlled: usize = 119;
    pub const outlineVehicles: usize = 120;
    pub const outlineTurrets: usize = 121;
    pub const outlineCharacters: usize = 122;
    pub const outlineAgents: usize = 123;
    pub const outlineScriptMovers: usize = 124;
    pub const outlineLockOn: usize = 125;
}
pub mod AttPenetration {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const penetrateType: usize = 0;
    pub const penetrateMultiplier: usize = 4;
    pub const rifleBullet: usize = 8;
    pub const armorPiercing: usize = 9;
    pub const impaling: usize = 10;
}
pub mod AttProjectile {
    pub const SIZE: usize = 248;
    pub const ALIGN: usize = 8;
    pub const explosionRadius: usize = 0;
    pub const explosionForceRadius: usize = 4;
    pub const explosionInnerDamage: usize = 8;
    pub const explosionOuterDamage: usize = 12;
    pub const damageConeAngle: usize = 16;
    pub const projectileSpeed: usize = 20;
    pub const projectileSpeedUp: usize = 24;
    pub const projectileActivateDist: usize = 28;
    pub const projectileLifetime: usize = 32;
    pub const projectileLifetimeStdDeviation: usize = 36;
    pub const projectileModel: usize = 40;
    pub const projExplosionType: usize = 48;
    pub const projExplosionEffect: usize = 56;
    pub const projExplosionEffect__u: usize = 56;
    pub const projExplosionEffect__u__fx: usize = 56;
    pub const projExplosionEffect__u__vfx: usize = 56;
    pub const projExplosionEffect__u__data: usize = 56;
    pub const projExplosionEffect__type: usize = 64;
    pub const projExplosionEffectForceNormalUp: usize = 72;
    pub const projExplosionSound: usize = 80;
    pub const projDudEffect: usize = 88;
    pub const projDudEffect__u: usize = 88;
    pub const projDudEffect__u__fx: usize = 88;
    pub const projDudEffect__u__vfx: usize = 88;
    pub const projDudEffect__u__data: usize = 88;
    pub const projDudEffect__type: usize = 96;
    pub const projDudSound: usize = 104;
    pub const projImpactExplode: usize = 112;
    pub const disableProjectileCrumpleCheck: usize = 113;
    pub const destabilizationRateTime: usize = 116;
    pub const destabilizationCurvatureMax: usize = 120;
    pub const destabilizeDistance: usize = 124;
    pub const projBodyEffect: usize = 128;
    pub const projBodyEffect__u: usize = 128;
    pub const projBodyEffect__u__fx: usize = 128;
    pub const projBodyEffect__u__vfx: usize = 128;
    pub const projBodyEffect__u__data: usize = 128;
    pub const projBodyEffect__type: usize = 136;
    pub const projTrailEffect: usize = 144;
    pub const projTrailEffect__u: usize = 144;
    pub const projTrailEffect__u__fx: usize = 144;
    pub const projTrailEffect__u__vfx: usize = 144;
    pub const projTrailEffect__u__data: usize = 144;
    pub const projTrailEffect__type: usize = 152;
    pub const projBeaconEffect: usize = 160;
    pub const projBeaconEffect__u: usize = 160;
    pub const projBeaconEffect__u__fx: usize = 160;
    pub const projBeaconEffect__u__vfx: usize = 160;
    pub const projBeaconEffect__u__data: usize = 160;
    pub const projBeaconEffect__type: usize = 168;
    pub const projIgnitionDelay: usize = 176;
    pub const projIgnitionEffect: usize = 184;
    pub const projIgnitionEffect__u: usize = 184;
    pub const projIgnitionEffect__u__fx: usize = 184;
    pub const projIgnitionEffect__u__vfx: usize = 184;
    pub const projIgnitionEffect__u__data: usize = 184;
    pub const projIgnitionEffect__type: usize = 192;
    pub const projIgnitionSound: usize = 200;
    pub const stickiness: usize = 208;
    pub const fuseTime: usize = 212;
    pub const aiFuseTime: usize = 216;
    pub const stickToPlayers: usize = 220;
    pub const stickToVehicles: usize = 221;
    pub const stickToTurrets: usize = 222;
    pub const stickToNonStick: usize = 223;
    pub const timedDetonation: usize = 224;
    pub const isPredictedProjectile: usize = 225;
    pub const useSteppedExplosionDamage: usize = 226;
    pub const reactiveMotionRadiusScale: usize = 228;
    pub const reactiveMotionFrequencyScale: usize = 232;
    pub const reactiveMotionAmplitudeScale: usize = 236;
    pub const reactiveMotionFalloff: usize = 240;
    pub const reactiveMotionLifetime: usize = 244;
}
pub mod AttRegeneration {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 4;
    pub const regenerationEnabled: usize = 0;
    pub const regenerationTimeMs: usize = 4;
    pub const regenerationAddTimeMs: usize = 8;
    pub const regenerationAmount: usize = 12;
    pub const regenerationConsumeStock: usize = 16;
    pub const regenerationDisableWhileFiring: usize = 17;
}
pub mod AttReload {
    pub const SIZE: usize = 3;
    pub const ALIGN: usize = 1;
    pub const noPartialReload: usize = 0;
    pub const segmentedReload: usize = 1;
    pub const reloadDisabled: usize = 2;
}
pub mod AttRumbles {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const fireRumble: usize = 0;
    pub const meleeImpactRumble: usize = 8;
}
pub mod AttScopeDriftSettings {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 4;
    pub const fScopeDriftDelay: usize = 0;
    pub const fScopeDriftLerpInTime: usize = 4;
    pub const fScopeDriftSteadyTime: usize = 8;
    pub const fScopeDriftLerpOutTime: usize = 12;
    pub const fScopeDriftSteadyFactor: usize = 16;
    pub const fScopeDriftUnsteadyFactor: usize = 20;
}
pub mod AttSight {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 4;
    pub const blurSceneAdsInFraction: usize = 0;
    pub const blurSceneAdsOutFraction: usize = 4;
    pub const aimDownSight: usize = 8;
    pub const adsFire: usize = 9;
    pub const rechamberWhileAds: usize = 10;
    pub const noAdsWhenMagEmpty: usize = 11;
    pub const canVariableZoom: usize = 12;
    pub const hideRailWithThisScope: usize = 13;
    pub const forceIronSightDown: usize = 14;
    pub const useScopeDrift: usize = 15;
    pub const useDualFOV: usize = 16;
}
pub mod AttSlideSpread {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 4;
    pub const slideSpreadMin: usize = 0;
    pub const slideSpreadMax: usize = 4;
    pub const slideSpreadDecayRate: usize = 8;
    pub const slideSpreadFireAdd: usize = 12;
    pub const slideSpreadTurnAdd: usize = 16;
}
pub mod AttTargetAssist {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const targetAssistType: usize = 0;
    pub const targetAssistBehavior: usize = 4;
    pub const targetAssistRange: usize = 8;
    pub const targetAssistAngle: usize = 12;
    pub const targetAssistLosOffsetForward: usize = 16;
    pub const targetAssistLosOffsetRight: usize = 20;
    pub const targetAssistLosOffsetUp: usize = 24;
    pub const targetAssistOnlyAvailableInAds: usize = 28;
}
pub mod AttUI {
    pub const SIZE: usize = 64;
    pub const ALIGN: usize = 8;
    pub const hudIcon: usize = 0;
    pub const pickupIcon: usize = 8;
    pub const dangerIcon: usize = 16;
    pub const throwBackIcon: usize = 24;
    pub const hideWarningIcons: usize = 32;
    pub const warningIconsDelay: usize = 36;
    pub const dpadIcon: usize = 40;
    pub const ammoCounterIcon: usize = 48;
    pub const ammoCounterClip: usize = 56;
    pub const grenadeDangerIconDistance: usize = 60;
}
pub mod AttUnknown01 {
    pub const SIZE: usize = 200;
    pub const ALIGN: usize = 8;
    pub const material: usize = 0;
    pub const unk_0: usize = 8;
    pub const unk_1: usize = 12;
    pub const unk_2: usize = 16;
    pub const unk_3: usize = 20;
    pub const unk_4: usize = 24;
    pub const unk_5: usize = 28;
    pub const unk_6: usize = 32;
    pub const unk_7: usize = 36;
    pub const unk_8: usize = 40;
    pub const unk_9: usize = 44;
    pub const unk_10: usize = 48;
    pub const unk_11: usize = 52;
    pub const unk_12: usize = 56;
    pub const unk_13: usize = 60;
    pub const unk_14: usize = 64;
    pub const unk_15: usize = 68;
    pub const unk_16: usize = 72;
    pub const unk_17: usize = 76;
    pub const unk_18: usize = 80;
    pub const unk_19: usize = 84;
    pub const unk_20: usize = 88;
    pub const unk_21: usize = 92;
    pub const unk_22: usize = 96;
    pub const unk_23: usize = 100;
    pub const unk_24: usize = 104;
    pub const unk_25: usize = 108;
    pub const unk_26: usize = 112;
    pub const unk_27: usize = 116;
    pub const unk_28: usize = 120;
    pub const unk_29: usize = 124;
    pub const unk_30: usize = 128;
    pub const unk_31: usize = 132;
    pub const unk_32: usize = 136;
    pub const unk_33: usize = 140;
    pub const unk_34: usize = 144;
    pub const unk_35: usize = 148;
    pub const unk_36: usize = 152;
    pub const unk_37: usize = 156;
    pub const unk_38: usize = 160;
    pub const unk_39: usize = 164;
    pub const unk_40: usize = 168;
    pub const unk_41: usize = 172;
    pub const unk_42: usize = 176;
    pub const unk_43: usize = 180;
    pub const unk_44: usize = 184;
    pub const unk_45: usize = 188;
    pub const unk_46: usize = 192;
    pub const unk_47: usize = 196;
    pub const unk_48: usize = 197;
    pub const pad0: usize = 198;
    pub const pad1: usize = 199;
}
pub mod AttViewKick {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 4;
    pub const hipViewKickPitchMin: usize = 0;
    pub const hipViewKickPitchMax: usize = 4;
    pub const hipViewKickYawMin: usize = 8;
    pub const hipViewKickYawMax: usize = 12;
    pub const hipViewKickMagMin: usize = 16;
    pub const hipViewKickCenterSpeed: usize = 20;
    pub const adsViewKickPitchMin: usize = 24;
    pub const adsViewKickPitchMax: usize = 28;
    pub const adsViewKickYawMin: usize = 32;
    pub const adsViewKickYawMax: usize = 36;
    pub const adsViewKickMagMin: usize = 40;
    pub const adsViewKickCenterSpeed: usize = 44;
    pub const kickAlignedInputScalar: usize = 48;
    pub const kickOpposedInputScalar: usize = 52;
}
pub mod AudioPASpeaker {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 4;
    pub const origin: usize = 0;
    pub const nameHash: usize = 12;
    pub const entChannelHash: usize = 16;
}
pub mod BModelDrawInfo {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 4;
    pub const surfId: usize = 0;
}
pub mod BehaviorTree {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const nodeCount: usize = 8;
    pub const nodes: usize = 16;
}
pub mod BehaviorTreeNode {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const __pad0: usize = 8;
}
pub mod Bounds {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 4;
    pub const midPoint: usize = 0;
    pub const halfSize: usize = 12;
}
pub mod CTAudRvbPanInfo {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 4;
    pub const hasCustomPosition: usize = 0;
    pub const position: usize = 4;
    pub const maxWallDistance: usize = 16;
    pub const minReverbVolume: usize = 20;
}
pub mod CardMemory {
    pub const SIZE: usize = 2;
    pub const ALIGN: usize = 1;
    pub const platform: usize = 0;
}
pub mod ClientEntAnchor {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const name: usize = 0;
    pub const entNum: usize = 4;
}
pub mod ClientEntityLinkToDef {
    pub const SIZE: usize = 28;
    pub const ALIGN: usize = 4;
    pub const anchorIndex: usize = 0;
    pub const originOffset: usize = 4;
    pub const angleOffset: usize = 16;
}
pub mod ClientTriggerBlend {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const numClientTriggerBlendNodes: usize = 0;
    pub const blendNodes: usize = 8;
}
pub mod ClientTriggerBlendNode {
    pub const SIZE: usize = 28;
    pub const ALIGN: usize = 4;
    pub const pointA: usize = 0;
    pub const pointB: usize = 12;
    pub const triggerA: usize = 24;
    pub const triggerB: usize = 26;
}
pub mod ClientTriggers {
    pub const SIZE: usize = 184;
    pub const ALIGN: usize = 8;
    pub const trigger: usize = 0;
    pub const trigger__count: usize = 0;
    pub const trigger__models: usize = 8;
    pub const trigger__hullCount: usize = 16;
    pub const trigger__hulls: usize = 24;
    pub const trigger__slabCount: usize = 32;
    pub const trigger__slabs: usize = 40;
    pub const trigger__windingCount: usize = 48;
    pub const trigger__windings: usize = 56;
    pub const trigger__windingPointCount: usize = 64;
    pub const trigger__windingPoints: usize = 72;
    pub const triggerStringLength: usize = 80;
    pub const triggerString: usize = 88;
    pub const visionSetTriggers: usize = 96;
    pub const triggerType: usize = 104;
    pub const origins: usize = 112;
    pub const scriptDelay: usize = 120;
    pub const audioTriggers: usize = 128;
    pub const blendLookup: usize = 136;
    pub const npcTriggers: usize = 144;
    pub const audioStateIds: usize = 152;
    pub const audioRvbPanInfo: usize = 160;
    pub const transientIndex: usize = 168;
    pub const linkTo: usize = 176;
}
pub mod ClipInfo {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const planeCount: usize = 0;
    pub const planes: usize = 8;
}
pub mod CollisionHeatmapEntry {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const vertexCount: usize = 0;
    pub const vertexDensity: usize = 4;
    pub const minExtent: usize = 8;
    pub const maxExtent: usize = 20;
}
pub mod ComChangeListInfo {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const changeListNumber: usize = 0;
    pub const time: usize = 4;
    pub const userName: usize = 8;
}
pub mod ComPrimaryLight {
    pub const SIZE: usize = 144;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const canUseShadowMap: usize = 1;
    pub const needsDynamicShadows: usize = 2;
    pub const isVolumetric: usize = 3;
    pub const exponent: usize = 4;
    pub const unused: usize = 5;
    pub const transientZoneList: usize = 8;
    pub const entityId: usize = 12;
    pub const uvIntensity: usize = 16;
    pub const irIntensity: usize = 20;
    pub const color: usize = 24;
    pub const dir: usize = 36;
    pub const up: usize = 48;
    pub const origin: usize = 60;
    pub const radius: usize = 72;
    pub const fadeOffsetRt: usize = 76;
    pub const bulbRadius: usize = 84;
    pub const bulbLength: usize = 88;
    pub const cosHalfFovOuter: usize = 100;
    pub const cosHalfFovInner: usize = 104;
    pub const shadowSoftness: usize = 108;
    pub const shadowBias: usize = 112;
    pub const shadowArea: usize = 116;
    pub const distanceFalloff: usize = 120;
    pub const rotationLimit: usize = 124;
    pub const translationLimit: usize = 128;
    pub const defName: usize = 136;
}
pub mod ComPrimaryLightEnv {
    pub const SIZE: usize = 10;
    pub const ALIGN: usize = 2;
    pub const primaryLightIndices: usize = 0;
    pub const numIndices: usize = 8;
}
pub mod ComWorld {
    pub const SIZE: usize = 104;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const isInUse: usize = 8;
    pub const useForwardPlus: usize = 12;
    pub const bakeQuality: usize = 16;
    pub const primaryLightCount: usize = 20;
    pub const primaryLights: usize = 24;
    pub const scriptablePrimaryLightCount: usize = 32;
    pub const firstScriptablePrimaryLight: usize = 36;
    pub const primaryLightEnvCount: usize = 40;
    pub const primaryLightEnvs: usize = 48;
    pub const changeListInfo: usize = 56;
    pub const changeListInfo__changeListNumber: usize = 56;
    pub const changeListInfo__time: usize = 60;
    pub const changeListInfo__userName: usize = 64;
    pub const numUmbraGates: usize = 72;
    pub const umbraGateNames: usize = 80;
    pub const umbraGateInitialStates: usize = 88;
}
pub mod ComputeShader {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const debugName: usize = 8;
    pub const prog: usize = 16;
    pub const prog__cs: usize = 16;
    pub const prog__loadDef: usize = 24;
    pub const prog__loadDef__program: usize = 24;
    pub const prog__loadDef__programSize: usize = 32;
}
pub mod ComputeShaderProgram {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const cs: usize = 0;
    pub const loadDef: usize = 8;
    pub const loadDef__program: usize = 8;
    pub const loadDef__programSize: usize = 16;
}
pub mod DBFile {
    pub const SIZE: usize = 168;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const handle: usize = 64;
    pub const memoryBufferSize: usize = 72;
    pub const memoryBuffer: usize = 80;
    pub const handle_dcache: usize = 88;
    pub const unkCount: usize = 96;
    pub const pad: usize = 100;
    pub const unkData: usize = 104;
    pub const loaded_pak: usize = 112;
    pub const __pad1: usize = 113;
    pub const isSecure: usize = 118;
    pub const __pad2: usize = 119;
    pub const some_check: usize = 120;
    pub const __pad3: usize = 124;
}
pub mod DB_AuthHash {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 1;
    pub const bytes: usize = 0;
}
pub mod DB_AuthHeader {
    pub const SIZE: usize = 16384;
    pub const ALIGN: usize = 4;
    pub const magic: usize = 0;
    pub const reserved: usize = 8;
    pub const subheaderHash: usize = 12;
    pub const subheaderHash__bytes: usize = 12;
    pub const signedSubheaderHash: usize = 44;
    pub const signedSubheaderHash__bytes: usize = 44;
    pub const subheader: usize = 300;
    pub const subheader__fastfileName: usize = 300;
    pub const subheader__reserved: usize = 332;
    pub const subheader__masterBlockHashes: usize = 336;
    pub const padding: usize = 6480;
}
pub mod DB_AuthSignature {
    pub const SIZE: usize = 256;
    pub const ALIGN: usize = 1;
    pub const bytes: usize = 0;
}
pub mod DB_AuthSubHeader {
    pub const SIZE: usize = 6180;
    pub const ALIGN: usize = 4;
    pub const fastfileName: usize = 0;
    pub const reserved: usize = 32;
    pub const masterBlockHashes: usize = 36;
}
pub mod DB_LoadData {
    pub const SIZE: usize = 168;
    pub const ALIGN: usize = 8;
    pub const file: usize = 0;
    pub const unk1: usize = 8;
    pub const fileBuffer: usize = 16;
    pub const readSize: usize = 24;
    pub const completedReadSize: usize = 32;
    pub const offset: usize = 40;
    pub const start_in: usize = 48;
    pub const readError: usize = 56;
    pub const readingResident: usize = 60;
    pub const stream: usize = 64;
    pub const stream__next_in: usize = 64;
    pub const stream__avail_in: usize = 72;
    pub const stream__total_in: usize = 80;
    pub const stream__next_out: usize = 88;
    pub const stream__avail_out: usize = 96;
    pub const stream__total_out: usize = 104;
    pub const stream__compressor: usize = 112;
    pub const read_size_: usize = 120;
    pub const offset_: usize = 128;
    pub const unk4: usize = 136;
    pub const unk5: usize = 140;
    pub const unk6: usize = 144;
    pub const unk7: usize = 148;
    pub const unk8: usize = 152;
    pub const unk9: usize = 156;
    pub const unk10: usize = 160;
    pub const unk11: usize = 164;
}
pub mod DB_MasterBlock {
    pub const SIZE: usize = 16384;
    pub const ALIGN: usize = 1;
    pub const chunkHashes: usize = 0;
}
pub mod DB_ReadStream {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const next_in: usize = 0;
    pub const avail_in: usize = 8;
    pub const total_in: usize = 16;
    pub const next_out: usize = 24;
    pub const avail_out: usize = 32;
    pub const total_out: usize = 40;
    pub const compressor: usize = 48;
}
pub mod DB_Zone {
    pub const SIZE: usize = 1144;
    pub const ALIGN: usize = 1;
    pub const name: usize = 0;
    pub const __pad0: usize = 64;
}
pub mod DDLDef {
    pub const SIZE: usize = 96;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const version: usize = 8;
    pub const checksum: usize = 12;
    pub const flags: usize = 16;
    pub const bitSize: usize = 20;
    pub const byteSize: usize = 24;
    pub const structList: usize = 32;
    pub const structCount: usize = 40;
    pub const enumList: usize = 48;
    pub const enumCount: usize = 56;
    pub const next: usize = 64;
    pub const headerBitSize: usize = 72;
    pub const headerByteSize: usize = 76;
    pub const reserveSize: usize = 80;
    pub const userFlagsSize: usize = 84;
    pub const paddingUsed: usize = 88;
}
pub mod DDLEnum {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const memberCount: usize = 8;
    pub const members: usize = 16;
    pub const hashTable: usize = 24;
    pub const hashTable__list: usize = 24;
    pub const hashTable__count: usize = 32;
    pub const hashTable__max: usize = 36;
}
pub mod DDLFile {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const ddlDef: usize = 8;
}
pub mod DDLHash {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const hash: usize = 0;
    pub const index: usize = 4;
}
pub mod DDLHashTable {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const list: usize = 0;
    pub const count: usize = 8;
    pub const max: usize = 12;
}
pub mod DDLMember {
    pub const SIZE: usize = 72;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const index: usize = 8;
    pub const parent: usize = 16;
    pub const bitSize: usize = 24;
    pub const limitSize: usize = 28;
    pub const offset: usize = 32;
    pub const r#type: usize = 36;
    pub const externalIndex: usize = 40;
    pub const rangeLimit: usize = 44;
    pub const serverDelta: usize = 48;
    pub const clientDelta: usize = 52;
    pub const arraySize: usize = 56;
    pub const enumIndex: usize = 60;
    pub const permission: usize = 64;
}
pub mod DDLStruct {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const bitSize: usize = 8;
    pub const memberCount: usize = 12;
    pub const members: usize = 16;
    pub const hashTableUpper: usize = 24;
    pub const hashTableUpper__list: usize = 24;
    pub const hashTableUpper__count: usize = 32;
    pub const hashTableUpper__max: usize = 36;
    pub const hashTableLower: usize = 40;
    pub const hashTableLower__list: usize = 40;
    pub const hashTableLower__count: usize = 48;
    pub const hashTableLower__max: usize = 52;
}
pub mod DObjAnimMat {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const quat: usize = 0;
    pub const trans: usize = 16;
    pub const transWeight: usize = 28;
}
pub mod DopplerPreset {
    pub const SIZE: usize = 88;
    pub const ALIGN: usize = 4;
    pub const name: usize = 0;
    pub const id: usize = 64;
    pub const speedOfSound: usize = 68;
    pub const playerVelocityScale: usize = 72;
    pub const minPitch: usize = 76;
    pub const maxPitch: usize = 80;
    pub const smoothing: usize = 84;
}
pub mod DpvsPlane {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const coeffs: usize = 0;
}
pub mod DynEntityClient {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const flags: usize = 0;
    pub const __pad0: usize = 2;
    pub const activeModel: usize = 8;
    pub const physicsSystemId: usize = 16;
    pub const physicsSystemDetailId: usize = 20;
    pub const numPhysicsBodies: usize = 24;
    pub const singlePhysicsBody: usize = 28;
    pub const detailBoundBody: usize = 32;
    pub const physicsSetupNext: usize = 36;
    pub const physicsSetupPrev: usize = 38;
    pub const dynEntDefId: usize = 40;
    pub const locked: usize = 44;
    pub const __pad3: usize = 48;
}
pub mod DynEntityDef {
    pub const SIZE: usize = 112;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const unused1: usize = 1;
    pub const initialPose: usize = 4;
    pub const initialPose__quat: usize = 4;
    pub const initialPose__origin: usize = 20;
    pub const pose: usize = 32;
    pub const pose__quat: usize = 32;
    pub const pose__origin: usize = 48;
    pub const unused2: usize = 60;
    pub const baseModel: usize = 64;
    pub const brushModel: usize = 72;
    pub const spawnActive: usize = 74;
    pub const unused3: usize = 75;
    pub const instanceIndex: usize = 76;
    pub const unk4: usize = 78;
    pub const unk5: usize = 80;
    pub const unk6: usize = 82;
    pub const unused4: usize = 84;
    pub const linkTo: usize = 88;
    pub const noPhysics: usize = 96;
    pub const unk1: usize = 97;
    pub const spawnEnabled: usize = 98;
    pub const distantShadows: usize = 99;
    pub const noSpotShadows: usize = 100;
    pub const isTransient: usize = 101;
    pub const transientZoneLoaded: usize = 102;
    pub const unk3: usize = 103;
    pub const transientIndexStored: usize = 104;
    pub const unused6: usize = 105;
}
pub mod DynEntityGlobalId {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const basis: usize = 0;
    pub const id: usize = 4;
}
pub mod DynEntityLinkToDef {
    pub const SIZE: usize = 28;
    pub const ALIGN: usize = 4;
    pub const anchorIndex: usize = 0;
    pub const originOffset: usize = 4;
    pub const angleOffset: usize = 16;
}
pub mod DynEntityPose {
    pub const SIZE: usize = 72;
    pub const ALIGN: usize = 8;
    pub const pose: usize = 0;
    pub const pose__quat: usize = 0;
    pub const pose__origin: usize = 16;
    pub const radius: usize = 28;
    pub const lastGpuLightGridRequest: usize = 32;
    pub const lastGpuLightGridRequest__lgvFrame: usize = 32;
    pub const lastGpuLightGridRequest__lgvHistoryEntry: usize = 36;
    pub const lastGpuLightGridRequest__lgvNumProbes: usize = 40;
    pub const numPoses: usize = 44;
    pub const poses: usize = 48;
    pub const detailBodyToBoneMap: usize = 56;
    pub const detailBodyToBoneMapCached: usize = 64;
    pub const cachedActive: usize = 65;
    pub const __pad2: usize = 66;
}
pub mod DynEntityTransientEntry {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const basis: usize = 0;
    pub const id: usize = 4;
}
pub mod DynEntityTransientGroup {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const bounds: usize = 0;
    pub const bounds__midPoint: usize = 0;
    pub const bounds__halfSize: usize = 12;
    pub const transientZoneIndex: usize = 24;
    pub const transientEntityIndex: usize = 28;
    pub const dynEntCount: usize = 32;
    pub const dynEnts: usize = 40;
}
pub mod DynEntityTransientGroupRuntime {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const transientIndex: usize = 0;
    pub const state: usize = 4;
}
pub mod DynEntityTransientGroupState {
    pub const SIZE: usize = 1;
    pub const ALIGN: usize = 1;
    pub const state: usize = 0;
}
pub mod EquipSoundSetMantleTypes {
    pub const SIZE: usize = 192;
    pub const ALIGN: usize = 8;
    pub const soundSets: usize = 0;
}
pub mod EquipSoundSetMeleeVM {
    pub const SIZE: usize = 896;
    pub const ALIGN: usize = 8;
    pub const soundSets: usize = 0;
}
pub mod EquipSoundSetMoveTypes {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const soundSets: usize = 0;
}
pub mod EquipSoundSetStanceTypes {
    pub const SIZE: usize = 64;
    pub const ALIGN: usize = 8;
    pub const soundSets: usize = 0;
}
pub mod EquipmentChanceMoveTypes {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const chances: usize = 0;
}
pub mod EquipmentChanceRattleTypes {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const chances: usize = 0;
}
pub mod EquipmentClothData {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const foleyName: usize = 0;
    pub const footstepName: usize = 8;
}
pub mod EquipmentSndChance {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const rattleChance: usize = 0;
    pub const accentChance: usize = 4;
    pub const silentChance: usize = 8;
}
pub mod EquipmentSoundSet {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const soundPLR: usize = 0;
    pub const soundNPC: usize = 8;
}
pub mod EquipmentSoundTable {
    pub const SIZE: usize = 112;
    pub const ALIGN: usize = 8;
    pub const szName: usize = 0;
    pub const numClothTypes: usize = 8;
    pub const numWeaponRattleTypes: usize = 12;
    pub const numMoveTypes: usize = 16;
    pub const numStanceTypes: usize = 20;
    pub const clothTypes: usize = 24;
    pub const weaponRattleTypes: usize = 32;
    pub const chancesPLR: usize = 40;
    pub const chancesNPC: usize = 48;
    pub const mvmtClothFootstepCeilingSoundSets: usize = 56;
    pub const mvmtClothFoleySoundSets: usize = 64;
    pub const mvmtRattleSoundSets: usize = 72;
    pub const mvmtAccentSoundSets: usize = 80;
    pub const mvmtAccentSoundSets__soundSets: usize = 80;
    pub const mvmtMantleSoundSets: usize = 88;
    pub const mvmtStanceSoundSets: usize = 96;
    pub const meleeAttackVMSoundSets: usize = 104;
}
pub mod EquipmentWeaponRattleData {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const szName: usize = 0;
    pub const priority: usize = 8;
}
pub mod ExtentBounds {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 4;
    pub const mins: usize = 0;
    pub const maxs: usize = 12;
}
pub mod Face {
    pub const SIZE: usize = 6;
    pub const ALIGN: usize = 2;
    pub const v1: usize = 0;
    pub const v2: usize = 2;
    pub const v3: usize = 4;
}
pub mod FilterDef {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 4;
    pub const id: usize = 0;
    pub const entChannelIdx: usize = 4;
    pub const band: usize = 5;
    pub const r#type: usize = 8;
    pub const freq: usize = 12;
    pub const gain: usize = 16;
    pub const q: usize = 20;
}
pub mod FootstepAnim {
    pub const SIZE: usize = 68;
    pub const ALIGN: usize = 4;
    pub const leftCount: usize = 0;
    pub const step: usize = 4;
    pub const movement: usize = 36;
}
pub mod FootstepAnims {
    pub const SIZE: usize = 612;
    pub const ALIGN: usize = 4;
    pub const footstep: usize = 0;
}
pub mod FootstepTime {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const time: usize = 0;
    pub const isLeft: usize = 4;
}
pub mod ForcedStubAssetReference {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const r#type: usize = 8;
}
pub mod FullOcclusionDef {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const id: usize = 0;
    pub const presetName: usize = 4;
    pub const entChannelIdx: usize = 8;
}
pub mod FxCombinedDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const u: usize = 0;
    pub const u__fx: usize = 0;
    pub const u__vfx: usize = 0;
    pub const u__data: usize = 0;
    pub const r#type: usize = 8;
}
pub mod FxCombinedUnion {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const fx: usize = 0;
    pub const vfx: usize = 0;
    pub const data: usize = 0;
}
pub mod FxEffectDef {
    pub const SIZE: usize = 64;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const flags: usize = 8;
    pub const totalSize: usize = 12;
    pub const msecLoopingLife: usize = 16;
    pub const elemDefCountLooping: usize = 20;
    pub const elemDefCountOneShot: usize = 24;
    pub const elemDefCountEmission: usize = 28;
    pub const elemMaxRadius: usize = 32;
    pub const occlusionQueryDepthBias: usize = 36;
    pub const occlusionQueryFadeIn: usize = 40;
    pub const occlusionQueryFadeOut: usize = 44;
    pub const occlusionQueryScaleRange: usize = 48;
    pub const occlusionQueryScaleRange__base: usize = 48;
    pub const occlusionQueryScaleRange__amplitude: usize = 52;
    pub const elemDefs: usize = 56;
}
pub mod FxEffectDefRef {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const handle: usize = 0;
    pub const name: usize = 0;
}
pub mod FxElemAtlas {
    pub const SIZE: usize = 10;
    pub const ALIGN: usize = 2;
    pub const behavior: usize = 0;
    pub const index: usize = 1;
    pub const fps: usize = 2;
    pub const unk: usize = 3;
    pub const loopCount: usize = 4;
    pub const colIndexBits: usize = 5;
    pub const rowIndexBits: usize = 6;
    pub const entryCount: usize = 8;
}
pub mod FxElemDef {
    pub const SIZE: usize = 304;
    pub const ALIGN: usize = 8;
    pub const flags: usize = 0;
    pub const flags2: usize = 4;
    pub const spawn: usize = 8;
    pub const spawn__looping: usize = 8;
    pub const spawn__looping__intervalMsec: usize = 8;
    pub const spawn__looping__count: usize = 12;
    pub const spawn__oneShot: usize = 8;
    pub const spawn__oneShot__count: usize = 8;
    pub const spawn__oneShot__count__base: usize = 8;
    pub const spawn__oneShot__count__amplitude: usize = 12;
    pub const spawnRange: usize = 16;
    pub const spawnRange__base: usize = 16;
    pub const spawnRange__amplitude: usize = 20;
    pub const fadeInRange: usize = 24;
    pub const fadeInRange__base: usize = 24;
    pub const fadeInRange__amplitude: usize = 28;
    pub const fadeOutRange: usize = 32;
    pub const fadeOutRange__base: usize = 32;
    pub const fadeOutRange__amplitude: usize = 36;
    pub const spawnFrustumCullRadius: usize = 40;
    pub const spawnDelayMsec: usize = 44;
    pub const spawnDelayMsec__base: usize = 44;
    pub const spawnDelayMsec__amplitude: usize = 48;
    pub const lifeSpanMsec: usize = 52;
    pub const lifeSpanMsec__base: usize = 52;
    pub const lifeSpanMsec__amplitude: usize = 56;
    pub const spawnOrigin: usize = 60;
    pub const spawnOffsetRadius: usize = 84;
    pub const spawnOffsetRadius__base: usize = 84;
    pub const spawnOffsetRadius__amplitude: usize = 88;
    pub const spawnOffsetHeight: usize = 92;
    pub const spawnOffsetHeight__base: usize = 92;
    pub const spawnOffsetHeight__amplitude: usize = 96;
    pub const spawnAngles: usize = 100;
    pub const angularVelocity: usize = 124;
    pub const initialRotation: usize = 148;
    pub const initialRotation__base: usize = 148;
    pub const initialRotation__amplitude: usize = 152;
    pub const gravity: usize = 156;
    pub const gravity__base: usize = 156;
    pub const gravity__amplitude: usize = 160;
    pub const reflectionFactor: usize = 164;
    pub const reflectionFactor__base: usize = 164;
    pub const reflectionFactor__amplitude: usize = 168;
    pub const atlas: usize = 172;
    pub const atlas__behavior: usize = 172;
    pub const atlas__index: usize = 173;
    pub const atlas__fps: usize = 174;
    pub const atlas__unk: usize = 175;
    pub const atlas__loopCount: usize = 176;
    pub const atlas__colIndexBits: usize = 177;
    pub const atlas__rowIndexBits: usize = 178;
    pub const atlas__entryCount: usize = 180;
    pub const elemType: usize = 182;
    pub const visualCount: usize = 183;
    pub const velIntervalCount: usize = 184;
    pub const visStateIntervalCount: usize = 185;
    pub const velSamples: usize = 192;
    pub const visSamples: usize = 200;
    pub const visuals: usize = 208;
    pub const visuals__markArray: usize = 208;
    pub const visuals__array: usize = 208;
    pub const visuals__instance: usize = 208;
    pub const visuals__instance__anonymous: usize = 208;
    pub const visuals__instance__material: usize = 208;
    pub const visuals__instance__model: usize = 208;
    pub const visuals__instance__effectDef: usize = 208;
    pub const visuals__instance__effectDef__handle: usize = 208;
    pub const visuals__instance__effectDef__name: usize = 208;
    pub const visuals__instance__soundName: usize = 208;
    pub const visuals__instance__vectorFieldName: usize = 208;
    pub const visuals__instance__lightDef: usize = 208;
    pub const visuals__instance__particleSimAnimation: usize = 208;
    pub const collBounds: usize = 216;
    pub const collBounds__midPoint: usize = 216;
    pub const collBounds__halfSize: usize = 228;
    pub const effectOnImpact: usize = 240;
    pub const effectOnImpact__handle: usize = 240;
    pub const effectOnImpact__name: usize = 240;
    pub const effectOnDeath: usize = 248;
    pub const effectOnDeath__handle: usize = 248;
    pub const effectOnDeath__name: usize = 248;
    pub const effectEmitted: usize = 256;
    pub const effectEmitted__handle: usize = 256;
    pub const effectEmitted__name: usize = 256;
    pub const emitDist: usize = 264;
    pub const emitDist__base: usize = 264;
    pub const emitDist__amplitude: usize = 268;
    pub const emitDistVariance: usize = 272;
    pub const emitDistVariance__base: usize = 272;
    pub const emitDistVariance__amplitude: usize = 276;
    pub const extended: usize = 280;
    pub const extended__trailDef: usize = 280;
    pub const extended__sparkFountainDef: usize = 280;
    pub const extended__spotLightDef: usize = 280;
    pub const extended__flareDef: usize = 280;
    pub const extended__unknownDef: usize = 280;
    pub const sortOrder: usize = 288;
    pub const lightingFrac: usize = 289;
    pub const useItemClip: usize = 290;
    pub const fadeInfo: usize = 291;
    pub const randomSeed: usize = 292;
    pub const litUnlitBlendFactor: usize = 296;
    pub const pad: usize = 300;
}
pub mod FxElemDefVisuals {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const markArray: usize = 0;
    pub const array: usize = 0;
    pub const instance: usize = 0;
    pub const instance__anonymous: usize = 0;
    pub const instance__material: usize = 0;
    pub const instance__model: usize = 0;
    pub const instance__effectDef: usize = 0;
    pub const instance__effectDef__handle: usize = 0;
    pub const instance__effectDef__name: usize = 0;
    pub const instance__soundName: usize = 0;
    pub const instance__vectorFieldName: usize = 0;
    pub const instance__lightDef: usize = 0;
    pub const instance__particleSimAnimation: usize = 0;
}
pub mod FxElemExtendedDefPtr {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const trailDef: usize = 0;
    pub const sparkFountainDef: usize = 0;
    pub const spotLightDef: usize = 0;
    pub const flareDef: usize = 0;
    pub const unknownDef: usize = 0;
}
pub mod FxElemMarkVisuals {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const materials: usize = 0;
}
pub mod FxElemVec3Range {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 4;
    pub const base: usize = 0;
    pub const amplitude: usize = 12;
}
pub mod FxElemVelStateInFrame {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 4;
    pub const velocity: usize = 0;
    pub const velocity__base: usize = 0;
    pub const velocity__amplitude: usize = 12;
    pub const totalDelta: usize = 24;
    pub const totalDelta__base: usize = 24;
    pub const totalDelta__amplitude: usize = 36;
}
pub mod FxElemVelStateSample {
    pub const SIZE: usize = 96;
    pub const ALIGN: usize = 4;
    pub const local: usize = 0;
    pub const local__velocity: usize = 0;
    pub const local__velocity__base: usize = 0;
    pub const local__velocity__amplitude: usize = 12;
    pub const local__totalDelta: usize = 24;
    pub const local__totalDelta__base: usize = 24;
    pub const local__totalDelta__amplitude: usize = 36;
    pub const world: usize = 48;
    pub const world__velocity: usize = 48;
    pub const world__velocity__base: usize = 48;
    pub const world__velocity__amplitude: usize = 60;
    pub const world__totalDelta: usize = 72;
    pub const world__totalDelta__base: usize = 72;
    pub const world__totalDelta__amplitude: usize = 84;
}
pub mod FxElemVisStateSample {
    pub const SIZE: usize = 72;
    pub const ALIGN: usize = 4;
    pub const base: usize = 0;
    pub const base__color: usize = 0;
    pub const base__rotationDelta: usize = 16;
    pub const base__rotationTotal: usize = 20;
    pub const base__size: usize = 24;
    pub const base__scale: usize = 32;
    pub const amplitude: usize = 36;
    pub const amplitude__color: usize = 36;
    pub const amplitude__rotationDelta: usize = 52;
    pub const amplitude__rotationTotal: usize = 56;
    pub const amplitude__size: usize = 60;
    pub const amplitude__scale: usize = 68;
}
pub mod FxElemVisualState {
    pub const SIZE: usize = 36;
    pub const ALIGN: usize = 4;
    pub const color: usize = 0;
    pub const rotationDelta: usize = 16;
    pub const rotationTotal: usize = 20;
    pub const size: usize = 24;
    pub const scale: usize = 32;
}
pub mod FxElemVisuals {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const anonymous: usize = 0;
    pub const material: usize = 0;
    pub const model: usize = 0;
    pub const effectDef: usize = 0;
    pub const effectDef__handle: usize = 0;
    pub const effectDef__name: usize = 0;
    pub const soundName: usize = 0;
    pub const vectorFieldName: usize = 0;
    pub const lightDef: usize = 0;
    pub const particleSimAnimation: usize = 0;
}
pub mod FxFlareDef {
    pub const SIZE: usize = 112;
    pub const ALIGN: usize = 8;
    pub const position: usize = 0;
    pub const angularRotCount: usize = 4;
    pub const flags: usize = 8;
    pub const depthScaleRange: usize = 12;
    pub const depthScaleRange__base: usize = 12;
    pub const depthScaleRange__amplitude: usize = 16;
    pub const depthScaleValue: usize = 20;
    pub const depthScaleValue__base: usize = 20;
    pub const depthScaleValue__amplitude: usize = 24;
    pub const radialRot: usize = 28;
    pub const radialRot__base: usize = 28;
    pub const radialRot__amplitude: usize = 32;
    pub const radialScaleX: usize = 36;
    pub const radialScaleX__base: usize = 36;
    pub const radialScaleX__amplitude: usize = 40;
    pub const radialScaleY: usize = 44;
    pub const radialScaleY__base: usize = 44;
    pub const radialScaleY__amplitude: usize = 48;
    pub const dir: usize = 52;
    pub const intensityXIntervalCount: usize = 64;
    pub const intensityYIntervalCount: usize = 68;
    pub const srcCosIntensityIntervalCount: usize = 72;
    pub const srcCosScaleIntervalCount: usize = 76;
    pub const intensityX: usize = 80;
    pub const intensityY: usize = 88;
    pub const srcCosIntensity: usize = 96;
    pub const srcCosScale: usize = 104;
}
pub mod FxFloatRange {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const base: usize = 0;
    pub const amplitude: usize = 4;
}
pub mod FxGlassCrackHeader {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 2;
    pub const uniqueVertCount: usize = 0;
    pub const beginVertIndex: usize = 2;
    pub const endVertIndex: usize = 3;
}
pub mod FxGlassDef {
    pub const SIZE: usize = 152;
    pub const ALIGN: usize = 8;
    pub const halfThickness: usize = 0;
    pub const texVecs: usize = 4;
    pub const color: usize = 20;
    pub const color__array: usize = 20;
    pub const color__packed: usize = 20;
    pub const material: usize = 24;
    pub const materialShattered: usize = 32;
    pub const physicsAsset: usize = 40;
    pub const pieceBreakEffect: usize = 48;
    pub const pieceBreakEffect__u: usize = 48;
    pub const pieceBreakEffect__u__fx: usize = 48;
    pub const pieceBreakEffect__u__vfx: usize = 48;
    pub const pieceBreakEffect__u__data: usize = 48;
    pub const pieceBreakEffect__type: usize = 56;
    pub const shatterEffect: usize = 64;
    pub const shatterEffect__u: usize = 64;
    pub const shatterEffect__u__fx: usize = 64;
    pub const shatterEffect__u__vfx: usize = 64;
    pub const shatterEffect__u__data: usize = 64;
    pub const shatterEffect__type: usize = 72;
    pub const shatterSmallEffect: usize = 80;
    pub const shatterSmallEffect__u: usize = 80;
    pub const shatterSmallEffect__u__fx: usize = 80;
    pub const shatterSmallEffect__u__vfx: usize = 80;
    pub const shatterSmallEffect__u__data: usize = 80;
    pub const shatterSmallEffect__type: usize = 88;
    pub const crackDecalEffect: usize = 96;
    pub const crackDecalEffect__u: usize = 96;
    pub const crackDecalEffect__u__fx: usize = 96;
    pub const crackDecalEffect__u__vfx: usize = 96;
    pub const crackDecalEffect__u__data: usize = 96;
    pub const crackDecalEffect__type: usize = 104;
    pub const damagedSound: usize = 112;
    pub const destroyedSound: usize = 120;
    pub const destroyedQuietSound: usize = 128;
    pub const highMipRadiusInvSq: usize = 136;
    pub const shatteredHighMipRadiusInvSq: usize = 140;
    pub const numCrackRings: usize = 144;
    pub const isOpaque: usize = 148;
}
pub mod FxGlassGeometryData {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 2;
    pub const vert: usize = 0;
    pub const vert__x: usize = 0;
    pub const vert__y: usize = 2;
    pub const hole: usize = 0;
    pub const hole__uniqueVertCount: usize = 0;
    pub const hole__touchVert: usize = 2;
    pub const hole__pad: usize = 3;
    pub const crack: usize = 0;
    pub const crack__uniqueVertCount: usize = 0;
    pub const crack__beginVertIndex: usize = 2;
    pub const crack__endVertIndex: usize = 3;
    pub const asBytes: usize = 0;
    pub const anonymous: usize = 0;
}
pub mod FxGlassHoleHeader {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 2;
    pub const uniqueVertCount: usize = 0;
    pub const touchVert: usize = 2;
    pub const pad: usize = 3;
}
pub mod FxGlassInitPieceState {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 4;
    pub const frame: usize = 0;
    pub const frame__quat: usize = 0;
    pub const frame__origin: usize = 16;
    pub const radius: usize = 28;
    pub const texCoordOrigin: usize = 32;
    pub const supportMask: usize = 40;
    pub const areaX2: usize = 44;
    pub const lightingIndex: usize = 48;
    pub const defIndex: usize = 50;
    pub const vertCount: usize = 51;
    pub const fanDataCount: usize = 52;
    pub const pad: usize = 53;
}
pub mod FxGlassPieceDynamics {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 4;
    pub const fallTime: usize = 0;
    pub const physicsInstance: usize = 4;
    pub const physicsDetailInstance: usize = 8;
    pub const vel: usize = 12;
    pub const avel: usize = 24;
    pub const pendingCreation: usize = 36;
}
pub mod FxGlassPiecePlace {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const __s0: usize = 0;
    pub const __s0__frame: usize = 0;
    pub const __s0__frame__quat: usize = 0;
    pub const __s0__frame__origin: usize = 16;
    pub const __s0__radius: usize = 28;
    pub const nextFree: usize = 0;
}
pub mod FxGlassPieceState {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const texCoordOrigin: usize = 0;
    pub const supportMask: usize = 8;
    pub const initIndex: usize = 12;
    pub const geoDataStart: usize = 14;
    pub const lightingIndex: usize = 16;
    pub const defIndex: usize = 18;
    pub const pad: usize = 19;
    pub const vertCount: usize = 22;
    pub const holeDataCount: usize = 23;
    pub const crackDataCount: usize = 24;
    pub const fanDataCount: usize = 25;
    pub const flags: usize = 26;
    pub const areaX2: usize = 28;
}
pub mod FxGlassSystem {
    pub const SIZE: usize = 168;
    pub const ALIGN: usize = 8;
    pub const time: usize = 0;
    pub const prevTime: usize = 4;
    pub const defCount: usize = 8;
    pub const pieceLimit: usize = 12;
    pub const pieceWordCount: usize = 16;
    pub const initPieceCount: usize = 20;
    pub const cellCount: usize = 24;
    pub const activePieceCount: usize = 28;
    pub const firstFreePiece: usize = 32;
    pub const geoDataLimit: usize = 36;
    pub const geoDataCount: usize = 40;
    pub const initGeoDataCount: usize = 44;
    pub const defs: usize = 48;
    pub const piecePlaces: usize = 56;
    pub const pieceStates: usize = 64;
    pub const pieceDynamics: usize = 72;
    pub const geoData: usize = 80;
    pub const isInUse: usize = 88;
    pub const cellBits: usize = 96;
    pub const visData: usize = 104;
    pub const linkOrg: usize = 112;
    pub const halfThickness: usize = 120;
    pub const lightingHandles: usize = 128;
    pub const initPieceStates: usize = 136;
    pub const initGeoData: usize = 144;
    pub const needToCompactData: usize = 152;
    pub const initCount: usize = 153;
    pub const effectChanceAccum: usize = 156;
    pub const lastPieceDeletionTime: usize = 160;
}
pub mod FxGlassVertex {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 2;
    pub const x: usize = 0;
    pub const y: usize = 2;
}
pub mod FxImpactEntry {
    pub const SIZE: usize = 1392;
    pub const ALIGN: usize = 8;
    pub const nonflesh: usize = 0;
    pub const flesh: usize = 1024;
}
pub mod FxImpactTable {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const table: usize = 8;
}
pub mod FxIntRange {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const base: usize = 0;
    pub const amplitude: usize = 4;
}
pub mod FxParticleSimAnimation {
    pub const SIZE: usize = 120;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const material: usize = 8;
    pub const header: usize = 16;
    pub const header__playbackRate: usize = 16;
    pub const header__duration: usize = 20;
    pub const header__frameCount: usize = 24;
    pub const header__minX: usize = 28;
    pub const header__minY: usize = 32;
    pub const header__minZ: usize = 36;
    pub const header__boundsXDelta: usize = 40;
    pub const header__boundsYDelta: usize = 44;
    pub const header__boundsZDelta: usize = 48;
    pub const header__maxWidth: usize = 52;
    pub const header__maxHeight: usize = 56;
    pub const header__colorTableSize: usize = 60;
    pub const header__particleDataCount: usize = 64;
    pub const header__maxActiveParticles: usize = 68;
    pub const header__evalVisStatePerParticle: usize = 72;
    pub const header__sortParticlesAtRuntime: usize = 73;
    pub const header__hasOrientation3D: usize = 74;
    pub const header__hasIndex: usize = 75;
    pub const particleData: usize = 80;
    pub const particleDataOrientation3D: usize = 88;
    pub const particleDataIndex: usize = 96;
    pub const frames: usize = 104;
    pub const colorTable: usize = 112;
}
pub mod FxParticleSimAnimationFrame {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const particleDataOffset: usize = 0;
    pub const numActiveParticles: usize = 4;
}
pub mod FxParticleSimAnimationHeader {
    pub const SIZE: usize = 60;
    pub const ALIGN: usize = 4;
    pub const playbackRate: usize = 0;
    pub const duration: usize = 4;
    pub const frameCount: usize = 8;
    pub const minX: usize = 12;
    pub const minY: usize = 16;
    pub const minZ: usize = 20;
    pub const boundsXDelta: usize = 24;
    pub const boundsYDelta: usize = 28;
    pub const boundsZDelta: usize = 32;
    pub const maxWidth: usize = 36;
    pub const maxHeight: usize = 40;
    pub const colorTableSize: usize = 44;
    pub const particleDataCount: usize = 48;
    pub const maxActiveParticles: usize = 52;
    pub const evalVisStatePerParticle: usize = 56;
    pub const sortParticlesAtRuntime: usize = 57;
    pub const hasOrientation3D: usize = 58;
    pub const hasIndex: usize = 59;
}
pub mod FxParticleSimAnimationParticleData {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 2;
    pub const xNormalizedPos: usize = 0;
    pub const yNormalizedPos: usize = 2;
    pub const zNormalizedPos: usize = 4;
    pub const xNormalizedWidth: usize = 6;
    pub const yNormalizedHeight: usize = 8;
    pub const orientation: usize = 10;
    pub const lifetime: usize = 12;
    pub const particleID: usize = 14;
    pub const xNormalizedPosNextFrame: usize = 16;
    pub const yNormalizedPosNextFrame: usize = 18;
    pub const zNormalizedPosNextFrame: usize = 20;
    pub const xNormalizedWidthNextFrame: usize = 22;
    pub const yNormalizedHeightNextFrame: usize = 24;
    pub const orientationDelta: usize = 26;
    pub const colorTableIndex: usize = 28;
    pub const nextColorTableIndex: usize = 30;
}
pub mod FxParticleSimAnimationParticleDataIndex {
    pub const SIZE: usize = 2;
    pub const ALIGN: usize = 1;
    pub const index: usize = 0;
    pub const nextIndex: usize = 1;
}
pub mod FxParticleSimAnimationParticleDataOrientation3D {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 2;
    pub const yOrientation: usize = 0;
    pub const zOrientation: usize = 2;
    pub const yOrientationDelta: usize = 4;
    pub const zOrientationDelta: usize = 6;
}
pub mod FxSparkFountainDef {
    pub const SIZE: usize = 52;
    pub const ALIGN: usize = 4;
    pub const gravity: usize = 0;
    pub const bounceFrac: usize = 4;
    pub const bounceRand: usize = 8;
    pub const sparkSpacing: usize = 12;
    pub const sparkLength: usize = 16;
    pub const sparkCount: usize = 20;
    pub const loopTime: usize = 24;
    pub const velMin: usize = 28;
    pub const velMax: usize = 32;
    pub const velConeFrac: usize = 36;
    pub const restSpeed: usize = 40;
    pub const boostTime: usize = 44;
    pub const boostFactor: usize = 48;
}
pub mod FxSpatialFrame {
    pub const SIZE: usize = 28;
    pub const ALIGN: usize = 4;
    pub const quat: usize = 0;
    pub const origin: usize = 16;
}
pub mod FxSpawnDef {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const looping: usize = 0;
    pub const looping__intervalMsec: usize = 0;
    pub const looping__count: usize = 4;
    pub const oneShot: usize = 0;
    pub const oneShot__count: usize = 0;
    pub const oneShot__count__base: usize = 0;
    pub const oneShot__count__amplitude: usize = 4;
}
pub mod FxSpawnDefLooping {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const intervalMsec: usize = 0;
    pub const count: usize = 4;
}
pub mod FxSpawnDefOneShot {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const count: usize = 0;
    pub const count__base: usize = 0;
    pub const count__amplitude: usize = 4;
}
pub mod FxSpotLightDef {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 4;
    pub const fovInnerFraction: usize = 0;
    pub const startRadius: usize = 4;
    pub const endRadius: usize = 8;
    pub const brightness: usize = 12;
    pub const maxLength: usize = 16;
    pub const exponent: usize = 20;
}
pub mod FxTrailDef {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const scrollTimeMsec: usize = 0;
    pub const repeatDist: usize = 4;
    pub const invSplitDist: usize = 8;
    pub const invSplitArcDist: usize = 12;
    pub const invSplitTime: usize = 16;
    pub const vertCount: usize = 20;
    pub const verts: usize = 24;
    pub const indCount: usize = 32;
    pub const inds: usize = 40;
}
pub mod FxTrailVertex {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 4;
    pub const pos: usize = 0;
    pub const normal: usize = 8;
    pub const texCoord: usize = 16;
}
pub mod FxWorld {
    pub const SIZE: usize = 176;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const glassSys: usize = 8;
    pub const glassSys__time: usize = 8;
    pub const glassSys__prevTime: usize = 12;
    pub const glassSys__defCount: usize = 16;
    pub const glassSys__pieceLimit: usize = 20;
    pub const glassSys__pieceWordCount: usize = 24;
    pub const glassSys__initPieceCount: usize = 28;
    pub const glassSys__cellCount: usize = 32;
    pub const glassSys__activePieceCount: usize = 36;
    pub const glassSys__firstFreePiece: usize = 40;
    pub const glassSys__geoDataLimit: usize = 44;
    pub const glassSys__geoDataCount: usize = 48;
    pub const glassSys__initGeoDataCount: usize = 52;
    pub const glassSys__defs: usize = 56;
    pub const glassSys__piecePlaces: usize = 64;
    pub const glassSys__pieceStates: usize = 72;
    pub const glassSys__pieceDynamics: usize = 80;
    pub const glassSys__geoData: usize = 88;
    pub const glassSys__isInUse: usize = 96;
    pub const glassSys__cellBits: usize = 104;
    pub const glassSys__visData: usize = 112;
    pub const glassSys__linkOrg: usize = 120;
    pub const glassSys__halfThickness: usize = 128;
    pub const glassSys__lightingHandles: usize = 136;
    pub const glassSys__initPieceStates: usize = 144;
    pub const glassSys__initGeoData: usize = 152;
    pub const glassSys__needToCompactData: usize = 160;
    pub const glassSys__initCount: usize = 161;
    pub const glassSys__effectChanceAccum: usize = 164;
    pub const glassSys__lastPieceDeletionTime: usize = 168;
}
pub mod G_GlassData {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const glassPieces: usize = 0;
    pub const pieceCount: usize = 8;
    pub const damageToWeaken: usize = 12;
    pub const damageToDestroy: usize = 14;
    pub const glassNameCount: usize = 16;
    pub const glassNames: usize = 24;
}
pub mod G_GlassName {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const nameStr: usize = 0;
    pub const name: usize = 8;
    pub const pieceCount: usize = 12;
    pub const pieceIndices: usize = 16;
}
pub mod G_GlassPiece {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const damageTaken: usize = 0;
    pub const collapseTime: usize = 2;
    pub const lastStateChangeTime: usize = 4;
    pub const impactDir: usize = 8;
    pub const impactPos: usize = 9;
}
pub mod Gesture {
    pub const SIZE: usize = 168;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const r#type: usize = 8;
    pub const priority: usize = 12;
    pub const looping: usize = 16;
    pub const anims: usize = 24;
    pub const weaponSettings: usize = 32;
    pub const weaponSettings__blendToStates: usize = 32;
    pub const weaponSettings__hideReticle: usize = 34;
    pub const weaponSettings__fireDelay: usize = 36;
    pub const weaponSettings__sprintDelay: usize = 40;
    pub const weaponSettings__useLeftIdleAkimbo: usize = 44;
    pub const weaponSettings__splitAnimsAkimbo: usize = 45;
    pub const weaponSettings__blendToDemeanorLoop: usize = 46;
    pub const weaponSettings__blendOutRaise: usize = 47;
    pub const directionalSettings: usize = 48;
    pub const directionalSettings__maxAngle: usize = 48;
    pub const directionalSettings__lerpAtMaxAngle: usize = 52;
    pub const directionalSettings__widthCushionAngle: usize = 56;
    pub const directionalSettings__lerpAtMinCushionAngle: usize = 60;
    pub const directionalSettings__lerpAtMaxCushionAngle: usize = 64;
    pub const directionalSettings__limitLeft: usize = 68;
    pub const directionalSettings__limitRight: usize = 72;
    pub const directionalSettings__limitUp: usize = 76;
    pub const directionalSettings__limitDown: usize = 80;
    pub const lookAroundSettings: usize = 84;
    pub const lookAroundSettings__yawLerpIn: usize = 84;
    pub const lookAroundSettings__yawLerpOut: usize = 88;
    pub const lookAroundSettings__walkTime: usize = 92;
    pub const lookAroundSettings__walkFootStepAnim: usize = 96;
    pub const lookAroundSettings__walkFootStepAnim__leftCount: usize = 96;
    pub const lookAroundSettings__walkFootStepAnim__step: usize = 100;
    pub const lookAroundSettings__walkFootStepAnim__movement: usize = 132;
}
pub mod GestureDirectionalSettings {
    pub const SIZE: usize = 36;
    pub const ALIGN: usize = 4;
    pub const maxAngle: usize = 0;
    pub const lerpAtMaxAngle: usize = 4;
    pub const widthCushionAngle: usize = 8;
    pub const lerpAtMinCushionAngle: usize = 12;
    pub const lerpAtMaxCushionAngle: usize = 16;
    pub const limitLeft: usize = 20;
    pub const limitRight: usize = 24;
    pub const limitUp: usize = 28;
    pub const limitDown: usize = 32;
}
pub mod GestureLookAroundSettings {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 4;
    pub const yawLerpIn: usize = 0;
    pub const yawLerpOut: usize = 4;
    pub const walkTime: usize = 8;
    pub const walkFootStepAnim: usize = 12;
    pub const walkFootStepAnim__leftCount: usize = 12;
    pub const walkFootStepAnim__step: usize = 16;
    pub const walkFootStepAnim__movement: usize = 48;
}
pub mod GestureWeaponSettings {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const blendToStates: usize = 0;
    pub const hideReticle: usize = 2;
    pub const fireDelay: usize = 4;
    pub const sprintDelay: usize = 8;
    pub const useLeftIdleAkimbo: usize = 12;
    pub const splitAnimsAkimbo: usize = 13;
    pub const blendToDemeanorLoop: usize = 14;
    pub const blendOutRaise: usize = 15;
}
pub mod GfxAabbTree {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const bounds: usize = 0;
    pub const bounds__midPoint: usize = 0;
    pub const bounds__halfSize: usize = 12;
    pub const childCount: usize = 24;
    pub const surfaceCount: usize = 26;
    pub const startSurfIndex: usize = 28;
    pub const smodelIndexCount: usize = 32;
    pub const smodelIndexes: usize = 40;
    pub const childrenOffset: usize = 48;
}
pub mod GfxBrushModel {
    pub const SIZE: usize = 60;
    pub const ALIGN: usize = 4;
    pub const writable: usize = 0;
    pub const writable__bounds: usize = 0;
    pub const writable__bounds__midPoint: usize = 0;
    pub const writable__bounds__halfSize: usize = 12;
    pub const bounds: usize = 24;
    pub const bounds__midPoint: usize = 24;
    pub const bounds__halfSize: usize = 36;
    pub const radius: usize = 48;
    pub const startSurfIndex: usize = 52;
    pub const surfaceCount: usize = 56;
}
pub mod GfxBrushModelWritable {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 4;
    pub const bounds: usize = 0;
    pub const bounds__midPoint: usize = 0;
    pub const bounds__halfSize: usize = 12;
}
pub mod GfxCell {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const bounds: usize = 0;
    pub const bounds__midPoint: usize = 0;
    pub const bounds__halfSize: usize = 12;
    pub const portalCount: usize = 24;
    pub const portals: usize = 32;
}
pub mod GfxCellTransientInfo {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 2;
    pub const aabbTreeIndex: usize = 0;
    pub const transientZone: usize = 2;
}
pub mod GfxCellTree {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const aabbTree: usize = 0;
}
pub mod GfxCellTreeCount {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 4;
    pub const aabbTreeCount: usize = 0;
}
pub mod GfxColor {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 4;
    pub const array: usize = 0;
    pub const packed: usize = 0;
}
pub mod GfxComputeShaderLoadDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const program: usize = 0;
    pub const programSize: usize = 8;
}
pub mod GfxDecalVolumeCollection {
    pub const SIZE: usize = 388;
    pub const ALIGN: usize = 1;
    pub const __pad0: usize = 0;
}
pub mod GfxDomainShaderLoadDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const program: usize = 0;
    pub const programSize: usize = 8;
}
pub mod GfxDrawSurf {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const fields: usize = 0;
    pub const fields__objectId: usize = 0;
    pub const fields__pad0: usize = 2;
    pub const fields__tessellation: usize = 6;
    pub const fields__pad1: usize = 6;
    pub const fields__pad2: usize = 8;
    pub const packed: usize = 0;
    pub const packed__p0: usize = 0;
    pub const packed__p1: usize = 8;
}
pub mod GfxDrawSurfFields {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const objectId: usize = 0;
    pub const pad0: usize = 2;
    pub const tessellation: usize = 6;
    pub const pad1: usize = 6;
    pub const pad2: usize = 8;
}
pub mod GfxFrustumLights {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const indexCount: usize = 0;
    pub const indices: usize = 8;
    pub const vertexCount: usize = 16;
    pub const vertices: usize = 24;
    pub const indexBuffer: usize = 32;
    pub const vertexBuffer: usize = 40;
}
pub mod GfxGpuLightGridProbePosition {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const origin: usize = 0;
}
pub mod GfxGpuLightGridTetrahedron {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const indexFlags: usize = 0;
}
pub mod GfxGpuLightGridTetrahedronNeighbors {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const neighbors: usize = 0;
}
pub mod GfxGpuLightGridTetrahedronVisibility {
    pub const SIZE: usize = 64;
    pub const ALIGN: usize = 4;
    pub const visibility: usize = 0;
}
pub mod GfxGpuLightGridVoxelStartTetrahedron {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 4;
    pub const index: usize = 0;
}
pub mod GfxGpuLightGridZone {
    pub const SIZE: usize = 88;
    pub const ALIGN: usize = 4;
    pub const numProbes: usize = 0;
    pub const firstProbe: usize = 4;
    pub const numTetrahedrons: usize = 8;
    pub const firstTetrahedron: usize = 12;
    pub const firstVoxelTetrahedronIndex: usize = 16;
    pub const numVoxelTetrahedronIndices: usize = 20;
    pub const fallbackProbeData: usize = 24;
    pub const fallbackProbeData__coeffs: usize = 24;
    pub const fallbackProbeData__pad: usize = 82;
}
pub mod GfxHeightfield {
    pub const SIZE: usize = 96;
    pub const ALIGN: usize = 8;
    pub const bounds: usize = 0;
    pub const bounds__midPoint: usize = 0;
    pub const bounds__halfSize: usize = 12;
    pub const lookupMatrix: usize = 24;
    pub const image: usize = 88;
}
pub mod GfxHeroOnlyLight {
    pub const SIZE: usize = 68;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const unused: usize = 1;
    pub const color: usize = 4;
    pub const dir: usize = 16;
    pub const up: usize = 28;
    pub const origin: usize = 40;
    pub const radius: usize = 52;
    pub const cosHalfFovOuter: usize = 56;
    pub const cosHalfFovInner: usize = 60;
    pub const exponent: usize = 64;
}
pub mod GfxHullShaderLoadDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const program: usize = 0;
    pub const programSize: usize = 8;
}
pub mod GfxImage {
    pub const SIZE: usize = 112;
    pub const ALIGN: usize = 8;
    pub const texture: usize = 0;
    pub const texture__linemap: usize = 0;
    pub const texture__map: usize = 0;
    pub const texture__volmap: usize = 0;
    pub const texture__cubemap: usize = 0;
    pub const texture__loadDef: usize = 0;
    pub const texture__shaderView: usize = 8;
    pub const texture__shaderViewAlternate: usize = 16;
    pub const imageFormat: usize = 24;
    pub const flags: usize = 28;
    pub const mapType: usize = 32;
    pub const semantic: usize = 33;
    pub const category: usize = 34;
    pub const picmip: usize = 35;
    pub const picmip__platform: usize = 35;
    pub const cardMemory: usize = 37;
    pub const cardMemory__platform: usize = 37;
    pub const dataLen1: usize = 40;
    pub const dataLen2: usize = 44;
    pub const width: usize = 48;
    pub const height: usize = 50;
    pub const depth: usize = 52;
    pub const numElements: usize = 54;
    pub const levelCount: usize = 56;
    pub const streamed: usize = 57;
    pub const unknown1: usize = 58;
    pub const pixelData: usize = 64;
    pub const streams: usize = 72;
    pub const name: usize = 104;
}
pub mod GfxImageLoadDef {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 4;
    pub const levelCount: usize = 0;
    pub const numElements: usize = 1;
    pub const pad: usize = 2;
    pub const flags: usize = 4;
    pub const format: usize = 8;
    pub const resourceSize: usize = 12;
    pub const data: usize = 16;
}
pub mod GfxImageStreamData {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const width: usize = 0;
    pub const height: usize = 2;
    pub const levelCountAndSize: usize = 4;
    pub const levelCountAndSize__pixelSize: usize = 4;
    pub const levelCountAndSize__levelCount: usize = 7;
}
pub mod GfxImageStreamLevelCountAndSize {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 4;
    pub const pixelSize: usize = 0;
    pub const levelCount: usize = 3;
}
pub mod GfxLightAABB {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const nodeCount: usize = 0;
    pub const lightCount: usize = 2;
    pub const nodeArray: usize = 8;
    pub const lightArray: usize = 16;
}
pub mod GfxLightAABBNode {
    pub const SIZE: usize = 28;
    pub const ALIGN: usize = 4;
    pub const bound: usize = 0;
    pub const bound__midPoint: usize = 0;
    pub const bound__halfSize: usize = 12;
    pub const firstChild: usize = 24;
    pub const childCount: usize = 26;
}
pub mod GfxLightDef {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const iesProfile: usize = 8;
    pub const unk0: usize = 16;
    pub const unk1: usize = 20;
    pub const coordOffset: usize = 24;
    pub const coordScale: usize = 28;
    pub const unk2: usize = 32;
    pub const pad: usize = 36;
}
pub mod GfxLightGrid {
    pub const SIZE: usize = 1752;
    pub const ALIGN: usize = 8;
    pub const unk: usize = 0;
    pub const tableVersion: usize = 36;
    pub const paletteVersion: usize = 40;
    pub const rangeExponent8BitsEncoding: usize = 44;
    pub const rangeExponent12BitsEncoding: usize = 45;
    pub const rangeExponent16BitsEncoding: usize = 46;
    pub const stageCount: usize = 47;
    pub const stageLightingContrastGain: usize = 48;
    pub const paletteEntryCount: usize = 56;
    pub const paletteEntryAddress: usize = 64;
    pub const paletteBitstreamSize: usize = 72;
    pub const paletteBitstream: usize = 80;
    pub const skyLightGridColors: usize = 88;
    pub const skyLightGridColors__rgb: usize = 88;
    pub const defaultLightGridColors: usize = 760;
    pub const defaultLightGridColors__rgb: usize = 760;
    pub const tree: usize = 1432;
    pub const tree__maxDepth: usize = 1432;
    pub const tree__nodeCount: usize = 1436;
    pub const tree__leafCount: usize = 1440;
    pub const tree__coordMinGridSpace: usize = 1444;
    pub const tree__coordMaxGridSpace: usize = 1456;
    pub const tree__coordHalfSizeGridSpace: usize = 1468;
    pub const tree__defaultColorIndexBitCount: usize = 1480;
    pub const tree__defaultLightIndexBitCount: usize = 1484;
    pub const tree__p_nodeTable: usize = 1488;
    pub const tree__leafTableSize: usize = 1496;
    pub const tree__p_leafTable: usize = 1504;
    pub const probeData: usize = 1512;
    pub const probeData__gpuVisibleProbesCount: usize = 1512;
    pub const probeData__gpuVisibleProbePositions: usize = 1520;
    pub const probeData__gpuVisibleProbesData: usize = 1528;
    pub const probeData__gpuVisibleProbesBuffer: usize = 1536;
    pub const probeData__gpuVisibleProbesView: usize = 1544;
    pub const probeData__gpuVisibleProbesRWView: usize = 1552;
    pub const probeData__probeCount: usize = 1560;
    pub const probeData__probes: usize = 1568;
    pub const probeData__probesBuffer: usize = 1576;
    pub const probeData__probesView: usize = 1584;
    pub const probeData__probesRWView: usize = 1592;
    pub const probeData__probePositions: usize = 1600;
    pub const probeData__probePositionsBuffer: usize = 1608;
    pub const probeData__probePositionsView: usize = 1616;
    pub const probeData__zoneCount: usize = 1624;
    pub const probeData__zones: usize = 1632;
    pub const probeData__tetrahedronCount: usize = 1640;
    pub const probeData__tetrahedronCountVisible: usize = 1644;
    pub const probeData__tetrahedrons: usize = 1648;
    pub const probeData__tetrahedronBuffer: usize = 1656;
    pub const probeData__tetrahedronView: usize = 1664;
    pub const probeData__tetrahedronNeighbors: usize = 1672;
    pub const probeData__tetrahedronNeighborsBuffer: usize = 1680;
    pub const probeData__tetrahedronNeighborsView: usize = 1688;
    pub const probeData__tetrahedronVisibility: usize = 1696;
    pub const probeData__tetrahedronVisibilityBuffer: usize = 1704;
    pub const probeData__tetrahedronVisibilityView: usize = 1712;
    pub const probeData__voxelStartTetrahedronCount: usize = 1720;
    pub const probeData__voxelStartTetrahedron: usize = 1728;
    pub const probeData__voxelStartTetrahedronBuffer: usize = 1736;
    pub const probeData__voxelStartTetrahedronView: usize = 1744;
}
pub mod GfxLightGridColorsHDR {
    pub const SIZE: usize = 672;
    pub const ALIGN: usize = 4;
    pub const rgb: usize = 0;
}
pub mod GfxLightGridProbeData {
    pub const SIZE: usize = 240;
    pub const ALIGN: usize = 8;
    pub const gpuVisibleProbesCount: usize = 0;
    pub const gpuVisibleProbePositions: usize = 8;
    pub const gpuVisibleProbesData: usize = 16;
    pub const gpuVisibleProbesBuffer: usize = 24;
    pub const gpuVisibleProbesView: usize = 32;
    pub const gpuVisibleProbesRWView: usize = 40;
    pub const probeCount: usize = 48;
    pub const probes: usize = 56;
    pub const probesBuffer: usize = 64;
    pub const probesView: usize = 72;
    pub const probesRWView: usize = 80;
    pub const probePositions: usize = 88;
    pub const probePositionsBuffer: usize = 96;
    pub const probePositionsView: usize = 104;
    pub const zoneCount: usize = 112;
    pub const zones: usize = 120;
    pub const tetrahedronCount: usize = 128;
    pub const tetrahedronCountVisible: usize = 132;
    pub const tetrahedrons: usize = 136;
    pub const tetrahedronBuffer: usize = 144;
    pub const tetrahedronView: usize = 152;
    pub const tetrahedronNeighbors: usize = 160;
    pub const tetrahedronNeighborsBuffer: usize = 168;
    pub const tetrahedronNeighborsView: usize = 176;
    pub const tetrahedronVisibility: usize = 184;
    pub const tetrahedronVisibilityBuffer: usize = 192;
    pub const tetrahedronVisibilityView: usize = 200;
    pub const voxelStartTetrahedronCount: usize = 208;
    pub const voxelStartTetrahedron: usize = 216;
    pub const voxelStartTetrahedronBuffer: usize = 224;
    pub const voxelStartTetrahedronView: usize = 232;
}
pub mod GfxLightGridTree {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 8;
    pub const maxDepth: usize = 0;
    pub const nodeCount: usize = 4;
    pub const leafCount: usize = 8;
    pub const coordMinGridSpace: usize = 12;
    pub const coordMaxGridSpace: usize = 24;
    pub const coordHalfSizeGridSpace: usize = 36;
    pub const defaultColorIndexBitCount: usize = 48;
    pub const defaultLightIndexBitCount: usize = 52;
    pub const p_nodeTable: usize = 56;
    pub const leafTableSize: usize = 64;
    pub const p_leafTable: usize = 72;
}
pub mod GfxLightMap {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const textures: usize = 8;
}
pub mod GfxLightRegion {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const hullCount: usize = 0;
    pub const hulls: usize = 8;
}
pub mod GfxLightRegionAxis {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 4;
    pub const dir: usize = 0;
    pub const midPoint: usize = 12;
    pub const halfSize: usize = 16;
}
pub mod GfxLightRegionHull {
    pub const SIZE: usize = 88;
    pub const ALIGN: usize = 8;
    pub const kdopMidPoint: usize = 0;
    pub const kdopHalfSize: usize = 36;
    pub const axisCount: usize = 72;
    pub const axis: usize = 80;
}
pub mod GfxLightViewFrustum {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const planeCount: usize = 0;
    pub const planes: usize = 8;
    pub const indexCount: usize = 16;
    pub const indices: usize = 24;
    pub const vertexCount: usize = 32;
    pub const vertices: usize = 40;
}
pub mod GfxOrientedBoundingBox {
    pub const SIZE: usize = 60;
    pub const ALIGN: usize = 4;
    pub const center: usize = 0;
    pub const xAxis: usize = 12;
    pub const yAxis: usize = 24;
    pub const zAxis: usize = 36;
    pub const halfSize: usize = 48;
}
pub mod GfxPackedMotionVertex {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const xyz: usize = 0;
    pub const binormalSignAndHeight: usize = 12;
    pub const pieceIndex: usize = 16;
    pub const pieceIndex__array: usize = 16;
    pub const pieceIndex__packed: usize = 16;
    pub const texCoord: usize = 20;
    pub const texCoord__packed: usize = 20;
    pub const texCoord__array: usize = 20;
    pub const normal: usize = 24;
    pub const normal__packed: usize = 24;
    pub const normal__array: usize = 24;
    pub const tangent: usize = 28;
    pub const tangent__packed: usize = 28;
    pub const tangent__array: usize = 28;
}
pub mod GfxPackedPlacement {
    pub const SIZE: usize = 52;
    pub const ALIGN: usize = 4;
    pub const origin: usize = 0;
    pub const axis: usize = 12;
    pub const scale: usize = 48;
}
pub mod GfxPackedVertex {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const xyz: usize = 0;
    pub const selfVisibility: usize = 12;
    pub const selfVisibility__packed: usize = 12;
    pub const color: usize = 16;
    pub const color__array: usize = 16;
    pub const color__packed: usize = 16;
    pub const texCoord: usize = 20;
    pub const texCoord__packed: usize = 20;
    pub const texCoord__array: usize = 20;
    pub const normal: usize = 24;
    pub const normal__packed: usize = 24;
    pub const normal__array: usize = 24;
    pub const tangent: usize = 28;
    pub const tangent__packed: usize = 28;
    pub const tangent__array: usize = 28;
}
pub mod GfxPixelShaderLoadDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const program: usize = 0;
    pub const programSize: usize = 8;
    pub const microCodeCrc: usize = 12;
}
pub mod GfxPlacement {
    pub const SIZE: usize = 28;
    pub const ALIGN: usize = 4;
    pub const quat: usize = 0;
    pub const origin: usize = 16;
}
pub mod GfxPortal {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 8;
    pub const writable: usize = 0;
    pub const writable__isQueued: usize = 0;
    pub const writable__isAncestor: usize = 1;
    pub const writable__recursionDepth: usize = 2;
    pub const writable__hullPointCount: usize = 3;
    pub const writable__hullPoints: usize = 8;
    pub const writable__queuedParent: usize = 16;
    pub const plane: usize = 24;
    pub const plane__coeffs: usize = 24;
    pub const vertices: usize = 40;
    pub const cellIndex: usize = 48;
    pub const closeDistance: usize = 50;
    pub const vertexCount: usize = 52;
    pub const hullAxis: usize = 56;
}
pub mod GfxPortalWritable {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const isQueued: usize = 0;
    pub const isAncestor: usize = 1;
    pub const recursionDepth: usize = 2;
    pub const hullPointCount: usize = 3;
    pub const hullPoints: usize = 8;
    pub const queuedParent: usize = 16;
}
pub mod GfxRawTexture {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const linemap: usize = 0;
    pub const map: usize = 0;
    pub const volmap: usize = 0;
    pub const cubemap: usize = 0;
    pub const loadDef: usize = 0;
    pub const shaderView: usize = 8;
    pub const shaderViewAlternate: usize = 16;
}
pub mod GfxReflectionProbe {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const livePath: usize = 0;
    pub const origin: usize = 8;
    pub const angles: usize = 20;
    pub const probeInstances: usize = 32;
    pub const probeInstanceCount: usize = 40;
    pub const probeRelightingIndex: usize = 44;
}
pub mod GfxReflectionProbeInstance {
    pub const SIZE: usize = 152;
    pub const ALIGN: usize = 8;
    pub const livePath: usize = 0;
    pub const livePath2: usize = 8;
    pub const probePosition: usize = 16;
    pub const probeImageIndex: usize = 28;
    pub const lodLevel: usize = 30;
    pub const flags: usize = 31;
    pub const probeRotation: usize = 32;
    pub const volumeObb: usize = 48;
    pub const volumeObb__center: usize = 48;
    pub const volumeObb__xAxis: usize = 60;
    pub const volumeObb__yAxis: usize = 72;
    pub const volumeObb__zAxis: usize = 84;
    pub const volumeObb__halfSize: usize = 96;
    pub const priority: usize = 108;
    pub const feather: usize = 112;
    pub const expandProjectionNeg: usize = 124;
    pub const expandProjectionPos: usize = 136;
}
pub mod GfxReflectionProbeObb {
    pub const SIZE: usize = 60;
    pub const ALIGN: usize = 4;
    pub const center: usize = 0;
    pub const xAxis: usize = 12;
    pub const yAxis: usize = 24;
    pub const zAxis: usize = 36;
    pub const halfSize: usize = 48;
}
pub mod GfxReflectionProbeRelightingData {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 4;
    pub const reflectionProbeIndex: usize = 0;
    pub const relightingFlags: usize = 4;
    pub const gBufferAlbedoImageIndex: usize = 8;
    pub const gBufferNormalImageIndex: usize = 12;
    pub const relightingScale: usize = 16;
}
pub mod GfxReflectionProbeSampleData {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const unk: usize = 0;
}
pub mod GfxSHProbeData {
    pub const SIZE: usize = 64;
    pub const ALIGN: usize = 2;
    pub const coeffs: usize = 0;
    pub const pad: usize = 58;
}
pub mod GfxSceneDynBrush {
    pub const SIZE: usize = 6;
    pub const ALIGN: usize = 2;
    pub const info: usize = 0;
    pub const info__surfId: usize = 0;
    pub const dynEntId: usize = 4;
}
pub mod GfxSceneDynModel {
    pub const SIZE: usize = 10;
    pub const ALIGN: usize = 2;
    pub const info: usize = 0;
    pub const info__hasGfxEntIndex: usize = 0;
    pub const info__lod: usize = 0;
    pub const info__materialLod: usize = 0;
    pub const info__surfBufSize: usize = 0;
    pub const info__surfId: usize = 4;
    pub const dynEntId: usize = 8;
}
pub mod GfxShadowGeometry {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const surfaceCount: usize = 0;
    pub const smodelCount: usize = 2;
    pub const sortedSurfIndex: usize = 8;
    pub const smodelIndex: usize = 16;
}
pub mod GfxSky {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const skySurfCount: usize = 0;
    pub const skyStartSurfs: usize = 8;
    pub const skyImage: usize = 16;
    pub const skySamplerState: usize = 24;
}
pub mod GfxStateBits {
    pub const SIZE: usize = 44;
    pub const ALIGN: usize = 4;
    pub const loadBits: usize = 0;
    pub const zone: usize = 24;
    pub const depthStencilState: usize = 26;
    pub const blendState: usize = 40;
    pub const rasterizerState: usize = 41;
}
pub mod GfxStaticModelDrawInst {
    pub const SIZE: usize = 184;
    pub const ALIGN: usize = 8;
    pub const placement: usize = 0;
    pub const placement__origin: usize = 0;
    pub const placement__axis: usize = 12;
    pub const placement__scale: usize = 48;
    pub const model: usize = 56;
    pub const vertexLightingInfo: usize = 64;
    pub const vertexLightingInfo__lightingValues: usize = 64;
    pub const vertexLightingInfo__smodelLightingVb: usize = 72;
    pub const vertexLightingInfo__lightingValuesVb: usize = 80;
    pub const vertexLightingInfo__cache: usize = 88;
    pub const vertexLightingInfo__cache__size: usize = 88;
    pub const vertexLightingInfo__cache__subdivCacheBuffer: usize = 96;
    pub const vertexLightingInfo__cache__subdivCacheView: usize = 104;
    pub const vertexLightingInfo__flags: usize = 112;
    pub const vertexLightingInfo__numLightingValues: usize = 116;
    pub const modelLightmapInfo: usize = 120;
    pub const modelLightmapInfo__offset: usize = 120;
    pub const modelLightmapInfo__scale: usize = 128;
    pub const modelLightmapInfo__lightmapIndex: usize = 136;
    pub const unk0: usize = 140;
    pub const unk1: usize = 142;
    pub const unk2: usize = 144;
    pub const unk3: usize = 146;
    pub const lightingHandle: usize = 148;
    pub const unk5: usize = 150;
    pub const unk6: usize = 152;
    pub const cullDist: usize = 160;
    pub const flags: usize = 162;
    pub const unk9: usize = 164;
    pub const unk10: usize = 166;
    pub const unk11: usize = 168;
    pub const unk13: usize = 170;
    pub const unk14: usize = 171;
    pub const unk15: usize = 172;
    pub const primaryLightEnvIndex: usize = 176;
    pub const reflectionProbeIndex: usize = 178;
    pub const firstMtlSkinIndex: usize = 179;
    pub const sunShadowFlags: usize = 180;
    pub const transientZone: usize = 181;
    pub const unk21: usize = 182;
}
pub mod GfxStaticModelInst {
    pub const SIZE: usize = 36;
    pub const ALIGN: usize = 4;
    pub const bounds: usize = 0;
    pub const bounds__midPoint: usize = 0;
    pub const bounds__halfSize: usize = 12;
    pub const lightingOrigin: usize = 24;
}
pub mod GfxStaticModelLightmapInfo {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 4;
    pub const offset: usize = 0;
    pub const scale: usize = 8;
    pub const lightmapIndex: usize = 16;
}
pub mod GfxStaticModelVertexLighting {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 2;
    pub const visibility: usize = 0;
    pub const ambientColorFloat16: usize = 4;
    pub const highlightColorFloat16: usize = 12;
}
pub mod GfxStaticModelVertexLightingInfo {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const lightingValues: usize = 0;
    pub const smodelLightingVb: usize = 8;
    pub const lightingValuesVb: usize = 16;
    pub const cache: usize = 24;
    pub const cache__size: usize = 24;
    pub const cache__subdivCacheBuffer: usize = 32;
    pub const cache__subdivCacheView: usize = 40;
    pub const flags: usize = 48;
    pub const numLightingValues: usize = 52;
}
pub mod GfxSubdivCache {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const size: usize = 0;
    pub const subdivCacheBuffer: usize = 8;
    pub const subdivCacheView: usize = 16;
}
pub mod GfxSurface {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const tris: usize = 0;
    pub const tris__vertexLayerData: usize = 0;
    pub const tris__firstVertex: usize = 4;
    pub const tris__maxEdgeLength: usize = 8;
    pub const tris__vertexCount: usize = 12;
    pub const tris__triCount: usize = 14;
    pub const tris__baseIndex: usize = 16;
    pub const material: usize = 24;
    pub const lightmapIndex: usize = 32;
    pub const flags: usize = 33;
    pub const sortKey: usize = 34;
    pub const unk1: usize = 36;
    pub const unk2: usize = 37;
    pub const unk3: usize = 38;
    pub const unk4: usize = 39;
    pub const transientZone: usize = 40;
}
pub mod GfxSurfaceBounds {
    pub const SIZE: usize = 36;
    pub const ALIGN: usize = 4;
    pub const bounds: usize = 0;
    pub const bounds__midPoint: usize = 0;
    pub const bounds__halfSize: usize = 12;
    pub const unk: usize = 24;
}
pub mod GfxTexture {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const linemap: usize = 0;
    pub const map: usize = 0;
    pub const volmap: usize = 0;
    pub const cubemap: usize = 0;
    pub const loadDef: usize = 0;
    pub const shaderView: usize = 8;
    pub const shaderViewAlternate: usize = 16;
}
pub mod GfxVertexShaderLoadDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const program: usize = 0;
    pub const programSize: usize = 8;
    pub const microCodeCrc: usize = 12;
}
pub mod GfxVertexUnion0 {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const packedVerts0: usize = 0;
    pub const packedMotionVerts0: usize = 0;
    pub const verts0: usize = 0;
}
pub mod GfxVolumetric {
    pub const SIZE: usize = 240;
    pub const ALIGN: usize = 8;
    pub const livePath: usize = 0;
    pub const flags: usize = 8;
    pub const obb: usize = 12;
    pub const obb__center: usize = 12;
    pub const obb__xAxis: usize = 24;
    pub const obb__yAxis: usize = 36;
    pub const obb__zAxis: usize = 48;
    pub const obb__halfSize: usize = 60;
    pub const density: usize = 72;
    pub const falloff: usize = 76;
    pub const masks: usize = 80;
}
pub mod GfxVolumetricMask {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const axis: usize = 4;
    pub const image: usize = 8;
    pub const scale: usize = 16;
    pub const offset: usize = 24;
    pub const scroll: usize = 32;
}
pub mod GfxVoxelInternalNode {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const firstNodeIndex: usize = 0;
    pub const childNodeMask: usize = 8;
}
pub mod GfxVoxelLeafNode {
    pub const SIZE: usize = 2;
    pub const ALIGN: usize = 2;
    pub const lightListAddress: usize = 0;
}
pub mod GfxVoxelTopDownViewNode {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const firstNodeIndex: usize = 0;
    pub const zMin: usize = 4;
    pub const zMax: usize = 8;
}
pub mod GfxVoxelTree {
    pub const SIZE: usize = 112;
    pub const ALIGN: usize = 8;
    pub const zoneBound: usize = 0;
    pub const zoneBound__midPoint: usize = 0;
    pub const zoneBound__halfSize: usize = 12;
    pub const voxelTopDownViewNodeCount: usize = 24;
    pub const voxelInternalNodeCount: usize = 28;
    pub const voxelLeafNodeCount: usize = 32;
    pub const lightListArraySize: usize = 36;
    pub const voxelTreeHeader: usize = 40;
    pub const voxelTopDownViewNodeArray: usize = 48;
    pub const voxelInternalNodeArray: usize = 56;
    pub const voxelLeafNodeArray: usize = 64;
    pub const lightListArray: usize = 72;
    pub const voxelInternalNodeDynamicLightList: usize = 80;
    pub const __pad0: usize = 88;
}
pub mod GfxVoxelTreeHeader {
    pub const SIZE: usize = 64;
    pub const ALIGN: usize = 4;
    pub const rootNodeDimension: usize = 0;
    pub const nodeCoordBitShift: usize = 16;
    pub const boundMin: usize = 32;
    pub const boundMax: usize = 48;
}
pub mod GfxWorld {
    pub const SIZE: usize = 4520;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const baseName: usize = 8;
    pub const bspVersion: usize = 16;
    pub const planeCount: usize = 20;
    pub const nodeCount: usize = 24;
    pub const surfaceCount: usize = 28;
    pub const skyCount: usize = 32;
    pub const skies: usize = 40;
    pub const lastSunPrimaryLightIndex: usize = 48;
    pub const primaryLightCount: usize = 52;
    pub const movingScriptablePrimaryLightCount: usize = 56;
    pub const sortKeyLitDecal: usize = 60;
    pub const sortKeyEffectDecal: usize = 64;
    pub const sortKeyTopDecal: usize = 68;
    pub const sortKeyEffectAuto: usize = 72;
    pub const sortKeyDistortion: usize = 76;
    pub const sortKeyEffectDistortion: usize = 80;
    pub const sortKey2D: usize = 84;
    pub const sortKeyOpaqueBegin: usize = 88;
    pub const sortKeyOpaqueEnd: usize = 92;
    pub const sortKeyDecalBegin: usize = 96;
    pub const sortKeyDecalEnd: usize = 100;
    pub const sortKeyTransBegin: usize = 104;
    pub const sortKeyTransEnd: usize = 108;
    pub const sortKeyEmissiveBegin: usize = 112;
    pub const sortKeyEmissiveEnd: usize = 116;
    pub const dpvsPlanes: usize = 120;
    pub const dpvsPlanes__cellCount: usize = 120;
    pub const dpvsPlanes__planes: usize = 128;
    pub const dpvsPlanes__nodes: usize = 136;
    pub const dpvsPlanes__sceneEntCellBits: usize = 144;
    pub const cellTransientInfos: usize = 152;
    pub const cells: usize = 160;
    pub const draw: usize = 168;
    pub const draw__reflectionProbeData: usize = 168;
    pub const draw__reflectionProbeData__reflectionProbeCount: usize = 168;
    pub const draw__reflectionProbeData__sharedReflectionProbeCount: usize = 172;
    pub const draw__reflectionProbeData__reflectionProbes: usize = 176;
    pub const draw__reflectionProbeData__reflectionProbeArrayImage: usize = 184;
    pub const draw__reflectionProbeData__probeRelightingCount: usize = 192;
    pub const draw__reflectionProbeData__probeRelightingData: usize = 200;
    pub const draw__reflectionProbeData__reflectionProbeGBufferImageCount: usize = 208;
    pub const draw__reflectionProbeData__reflectionProbeGBufferImages: usize = 216;
    pub const draw__reflectionProbeData__reflectionProbeGBufferTextures: usize = 224;
    pub const draw__reflectionProbeData__reflectionProbeInstanceCount: usize = 232;
    pub const draw__reflectionProbeData__reflectionProbeInstances: usize = 240;
    pub const draw__reflectionProbeData__reflectionProbeLightgridSampleData: usize = 248;
    pub const draw__reflectionProbeData__reflectionProbeLightgridSampleDataBuffer: usize = 256;
    pub const draw__reflectionProbeData__reflectionProbeLightgridSampleDataBufferView: usize = 264;
    pub const draw__reflectionProbeData__reflectionProbeLightgridSampleDataBufferRWView: usize = 272;
    pub const draw__lightmapReindexData: usize = 280;
    pub const draw__lightmapReindexData__imagePixelSize: usize = 280;
    pub const draw__lightmapReindexData__reindexCount: usize = 284;
    pub const draw__lightmapReindexData__reindexElement: usize = 288;
    pub const draw__lightmapReindexData__packedLightmapCount: usize = 296;
    pub const draw__lightmapReindexData__packedLightmap: usize = 304;
    pub const draw__iesLookupTexture: usize = 312;
    pub const draw__decalVolumeCollectionCount: usize = 320;
    pub const draw__decalVolumeCollections: usize = 328;
    pub const draw__lightmapOverridePrimary: usize = 336;
    pub const draw__lightmapOverrideSecondary: usize = 344;
    pub const draw__lightMapCount: usize = 352;
    pub const draw__lightMaps: usize = 360;
    pub const draw__lightmapTextures: usize = 368;
    pub const draw__unused1: usize = 376;
    pub const draw__unused2: usize = 384;
    pub const draw__unused3: usize = 392;
    pub const draw__transientZoneCount: usize = 400;
    pub const draw__transientZones: usize = 408;
    pub const draw__indexCount: usize = 664;
    pub const draw__indices: usize = 672;
    pub const draw__indexBuffer: usize = 680;
    pub const draw__volumetrics: usize = 688;
    pub const draw__volumetrics__volumetricCount: usize = 688;
    pub const draw__volumetrics__volumetrics: usize = 696;
    pub const lightGrid: usize = 704;
    pub const lightGrid__unk: usize = 704;
    pub const lightGrid__tableVersion: usize = 740;
    pub const lightGrid__paletteVersion: usize = 744;
    pub const lightGrid__rangeExponent8BitsEncoding: usize = 748;
    pub const lightGrid__rangeExponent12BitsEncoding: usize = 749;
    pub const lightGrid__rangeExponent16BitsEncoding: usize = 750;
    pub const lightGrid__stageCount: usize = 751;
    pub const lightGrid__stageLightingContrastGain: usize = 752;
    pub const lightGrid__paletteEntryCount: usize = 760;
    pub const lightGrid__paletteEntryAddress: usize = 768;
    pub const lightGrid__paletteBitstreamSize: usize = 776;
    pub const lightGrid__paletteBitstream: usize = 784;
    pub const lightGrid__skyLightGridColors: usize = 792;
    pub const lightGrid__skyLightGridColors__rgb: usize = 792;
    pub const lightGrid__defaultLightGridColors: usize = 1464;
    pub const lightGrid__defaultLightGridColors__rgb: usize = 1464;
    pub const lightGrid__tree: usize = 2136;
    pub const lightGrid__tree__maxDepth: usize = 2136;
    pub const lightGrid__tree__nodeCount: usize = 2140;
    pub const lightGrid__tree__leafCount: usize = 2144;
    pub const lightGrid__tree__coordMinGridSpace: usize = 2148;
    pub const lightGrid__tree__coordMaxGridSpace: usize = 2160;
    pub const lightGrid__tree__coordHalfSizeGridSpace: usize = 2172;
    pub const lightGrid__tree__defaultColorIndexBitCount: usize = 2184;
    pub const lightGrid__tree__defaultLightIndexBitCount: usize = 2188;
    pub const lightGrid__tree__p_nodeTable: usize = 2192;
    pub const lightGrid__tree__leafTableSize: usize = 2200;
    pub const lightGrid__tree__p_leafTable: usize = 2208;
    pub const lightGrid__probeData: usize = 2216;
    pub const lightGrid__probeData__gpuVisibleProbesCount: usize = 2216;
    pub const lightGrid__probeData__gpuVisibleProbePositions: usize = 2224;
    pub const lightGrid__probeData__gpuVisibleProbesData: usize = 2232;
    pub const lightGrid__probeData__gpuVisibleProbesBuffer: usize = 2240;
    pub const lightGrid__probeData__gpuVisibleProbesView: usize = 2248;
    pub const lightGrid__probeData__gpuVisibleProbesRWView: usize = 2256;
    pub const lightGrid__probeData__probeCount: usize = 2264;
    pub const lightGrid__probeData__probes: usize = 2272;
    pub const lightGrid__probeData__probesBuffer: usize = 2280;
    pub const lightGrid__probeData__probesView: usize = 2288;
    pub const lightGrid__probeData__probesRWView: usize = 2296;
    pub const lightGrid__probeData__probePositions: usize = 2304;
    pub const lightGrid__probeData__probePositionsBuffer: usize = 2312;
    pub const lightGrid__probeData__probePositionsView: usize = 2320;
    pub const lightGrid__probeData__zoneCount: usize = 2328;
    pub const lightGrid__probeData__zones: usize = 2336;
    pub const lightGrid__probeData__tetrahedronCount: usize = 2344;
    pub const lightGrid__probeData__tetrahedronCountVisible: usize = 2348;
    pub const lightGrid__probeData__tetrahedrons: usize = 2352;
    pub const lightGrid__probeData__tetrahedronBuffer: usize = 2360;
    pub const lightGrid__probeData__tetrahedronView: usize = 2368;
    pub const lightGrid__probeData__tetrahedronNeighbors: usize = 2376;
    pub const lightGrid__probeData__tetrahedronNeighborsBuffer: usize = 2384;
    pub const lightGrid__probeData__tetrahedronNeighborsView: usize = 2392;
    pub const lightGrid__probeData__tetrahedronVisibility: usize = 2400;
    pub const lightGrid__probeData__tetrahedronVisibilityBuffer: usize = 2408;
    pub const lightGrid__probeData__tetrahedronVisibilityView: usize = 2416;
    pub const lightGrid__probeData__voxelStartTetrahedronCount: usize = 2424;
    pub const lightGrid__probeData__voxelStartTetrahedron: usize = 2432;
    pub const lightGrid__probeData__voxelStartTetrahedronBuffer: usize = 2440;
    pub const lightGrid__probeData__voxelStartTetrahedronView: usize = 2448;
    pub const frustumLights: usize = 2456;
    pub const lightViewFrustums: usize = 2464;
    pub const voxelTreeCount: usize = 2472;
    pub const voxelTree: usize = 2480;
    pub const heightfieldCount: usize = 2488;
    pub const heightfields: usize = 2496;
    pub const lightLists: usize = 2504;
    pub const lightLists__surfaceListOffsetCount: usize = 2504;
    pub const lightLists__surfaceListOffsets: usize = 2512;
    pub const lightLists__smodelListOffsetCount: usize = 2520;
    pub const lightLists__smodelListOffsets: usize = 2528;
    pub const lightLists__listsSize: usize = 2536;
    pub const lightLists__lists: usize = 2544;
    pub const modelCount: usize = 2552;
    pub const models: usize = 2560;
    pub const bounds: usize = 2568;
    pub const bounds__midPoint: usize = 2568;
    pub const bounds__halfSize: usize = 2580;
    pub const checksum: usize = 2592;
    pub const materialMemoryCount: usize = 2596;
    pub const materialMemory: usize = 2600;
    pub const sun: usize = 2608;
    pub const sun__hasValidData: usize = 2608;
    pub const sun__spriteMaterial: usize = 2616;
    pub const sun__flareMaterial: usize = 2624;
    pub const sun__spriteSize: usize = 2632;
    pub const sun__flareMinSize: usize = 2636;
    pub const sun__flareMinDot: usize = 2640;
    pub const sun__flareMaxSize: usize = 2644;
    pub const sun__flareMaxDot: usize = 2648;
    pub const sun__flareMaxAlpha: usize = 2652;
    pub const sun__flareFadeInTime: usize = 2656;
    pub const sun__flareFadeOutTime: usize = 2660;
    pub const sun__blindMinDot: usize = 2664;
    pub const sun__blindMaxDot: usize = 2668;
    pub const sun__blindMaxDarken: usize = 2672;
    pub const sun__blindFadeInTime: usize = 2676;
    pub const sun__blindFadeOutTime: usize = 2680;
    pub const sun__glareMinDot: usize = 2684;
    pub const sun__glareMaxDot: usize = 2688;
    pub const sun__glareMaxLighten: usize = 2692;
    pub const sun__glareFadeInTime: usize = 2696;
    pub const sun__glareFadeOutTime: usize = 2700;
    pub const sun__sunFxPosition: usize = 2704;
    pub const outdoorLookupMatrix: usize = 2720;
    pub const outdoorImage: usize = 2784;
    pub const dustMaterial: usize = 2792;
    pub const materialLod0SizeThreshold: usize = 2800;
    pub const cellCasterBits: usize = 2808;
    pub const cellHasSunLitSurfsBits: usize = 2816;
    pub const sceneDynModel: usize = 2824;
    pub const sceneDynBrush: usize = 2832;
    pub const primaryLightMotionDetectBitsEntries: usize = 2840;
    pub const primaryLightMotionDetectBits: usize = 2848;
    pub const entityMotionBitsEntries: usize = 2856;
    pub const entityMotionBits: usize = 2864;
    pub const staticSpotOmniPrimaryLightCountAligned: usize = 2872;
    pub const numPrimaryLightEntityShadowVisEntries: usize = 2876;
    pub const primaryLightEntityShadowVis: usize = 2880;
    pub const dynEntMotionBitsEntries: usize = 2888;
    pub const dynEntMotionBits: usize = 2896;
    pub const numPrimaryLightDynEntShadowVisEntries: usize = 2912;
    pub const primaryLightDynEntShadowVis: usize = 2920;
    pub const shadowGeomOptimized: usize = 2936;
    pub const lightRegion: usize = 2944;
    pub const lightAABB: usize = 2952;
    pub const lightAABB__nodeCount: usize = 2952;
    pub const lightAABB__lightCount: usize = 2954;
    pub const lightAABB__nodeArray: usize = 2960;
    pub const lightAABB__lightArray: usize = 2968;
    pub const dpvs: usize = 2976;
    pub const dpvs__smodelCount: usize = 2976;
    pub const dpvs__staticSurfaceCount: usize = 2980;
    pub const dpvs__litOpaqueSurfsBegin: usize = 2984;
    pub const dpvs__litOpaqueSurfsEnd: usize = 2988;
    pub const dpvs__litDecalSurfsBegin: usize = 2992;
    pub const dpvs__litDecalSurfsEnd: usize = 2996;
    pub const dpvs__litTransSurfsBegin: usize = 3000;
    pub const dpvs__litTransSurfsEnd: usize = 3004;
    pub const dpvs__emissiveSurfsBegin: usize = 3008;
    pub const dpvs__emissiveSurfsEnd: usize = 3012;
    pub const dpvs__smodelVisDataCount: usize = 3016;
    pub const dpvs__surfaceVisDataCount: usize = 3020;
    pub const dpvs__primaryLightVisDataCount: usize = 3024;
    pub const dpvs__reflectionProbeVisDataCount: usize = 3028;
    pub const dpvs__volumetricVisDataCount: usize = 3032;
    pub const dpvs__decalVisDataCount: usize = 3036;
    pub const dpvs__smodelVisData: usize = 3040;
    pub const dpvs__surfaceVisData: usize = 3280;
    pub const dpvs__lodData: usize = 3520;
    pub const dpvs__primaryLightVisData: usize = 3528;
    pub const dpvs__reflectionProbeVisData: usize = 3536;
    pub const dpvs__volumetricVisData: usize = 3544;
    pub const dpvs__decalVisData: usize = 3552;
    pub const dpvs__tessellationCutoffVisData: usize = 3560;
    pub const dpvs__sortedSurfIndex: usize = 3800;
    pub const dpvs__smodelInsts: usize = 3808;
    pub const dpvs__surfaces: usize = 3816;
    pub const dpvs__surfacesBounds: usize = 3824;
    pub const dpvs__smodelDrawInsts: usize = 3832;
    pub const dpvs__surfaceMaterials: usize = 3840;
    pub const dpvs__surfaceCastsSunShadow: usize = 3848;
    pub const dpvs__sortedSmodelIndices: usize = 3856;
    pub const dpvs__sunShadowOptCount: usize = 3864;
    pub const dpvs__sunSurfVisDataCount: usize = 3868;
    pub const dpvs__surfaceCastsSunShadowOpt: usize = 3872;
    pub const dpvs__constantBuffers: usize = 3880;
    pub const dpvs__usageCount: usize = 3888;
    pub const dpvsDyn: usize = 3896;
    pub const dpvsDyn__dynEntClientWordCount: usize = 3896;
    pub const dpvsDyn__dynEntClientCount: usize = 3904;
    pub const dpvsDyn__dynEntCellBits: usize = 3912;
    pub const dpvsDyn__dynEntVisData: usize = 3928;
    pub const mapVtxChecksum: usize = 4408;
    pub const heroOnlyLightCount: usize = 4412;
    pub const heroOnlyLights: usize = 4416;
    pub const numUmbraGates: usize = 4424;
    pub const umbraGates: usize = 4432;
    pub const umbraTomeSize: usize = 4440;
    pub const umbraTomeData: usize = 4448;
    pub const umbraTomePtr: usize = 4456;
    pub const numUmbraGates2: usize = 4464;
    pub const umbraGates2: usize = 4472;
    pub const umbraTomeSize2: usize = 4480;
    pub const umbraTomeData2: usize = 4488;
    pub const umbraTomePtr2: usize = 4496;
    pub const umbraUnkSize: usize = 4504;
    pub const umbraUnkData: usize = 4512;
}
pub mod GfxWorldDpvsDynamic {
    pub const SIZE: usize = 512;
    pub const ALIGN: usize = 8;
    pub const dynEntClientWordCount: usize = 0;
    pub const dynEntClientCount: usize = 8;
    pub const dynEntCellBits: usize = 16;
    pub const dynEntVisData: usize = 32;
}
pub mod GfxWorldDpvsPlanes {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const cellCount: usize = 0;
    pub const planes: usize = 8;
    pub const nodes: usize = 16;
    pub const sceneEntCellBits: usize = 24;
}
pub mod GfxWorldDpvsStatic {
    pub const SIZE: usize = 920;
    pub const ALIGN: usize = 8;
    pub const smodelCount: usize = 0;
    pub const staticSurfaceCount: usize = 4;
    pub const litOpaqueSurfsBegin: usize = 8;
    pub const litOpaqueSurfsEnd: usize = 12;
    pub const litDecalSurfsBegin: usize = 16;
    pub const litDecalSurfsEnd: usize = 20;
    pub const litTransSurfsBegin: usize = 24;
    pub const litTransSurfsEnd: usize = 28;
    pub const emissiveSurfsBegin: usize = 32;
    pub const emissiveSurfsEnd: usize = 36;
    pub const smodelVisDataCount: usize = 40;
    pub const surfaceVisDataCount: usize = 44;
    pub const primaryLightVisDataCount: usize = 48;
    pub const reflectionProbeVisDataCount: usize = 52;
    pub const volumetricVisDataCount: usize = 56;
    pub const decalVisDataCount: usize = 60;
    pub const smodelVisData: usize = 64;
    pub const surfaceVisData: usize = 304;
    pub const lodData: usize = 544;
    pub const primaryLightVisData: usize = 552;
    pub const reflectionProbeVisData: usize = 560;
    pub const volumetricVisData: usize = 568;
    pub const decalVisData: usize = 576;
    pub const tessellationCutoffVisData: usize = 584;
    pub const sortedSurfIndex: usize = 824;
    pub const smodelInsts: usize = 832;
    pub const surfaces: usize = 840;
    pub const surfacesBounds: usize = 848;
    pub const smodelDrawInsts: usize = 856;
    pub const surfaceMaterials: usize = 864;
    pub const surfaceCastsSunShadow: usize = 872;
    pub const sortedSmodelIndices: usize = 880;
    pub const sunShadowOptCount: usize = 888;
    pub const sunSurfVisDataCount: usize = 892;
    pub const surfaceCastsSunShadowOpt: usize = 896;
    pub const constantBuffers: usize = 904;
    pub const usageCount: usize = 912;
}
pub mod GfxWorldDraw {
    pub const SIZE: usize = 536;
    pub const ALIGN: usize = 8;
    pub const reflectionProbeData: usize = 0;
    pub const reflectionProbeData__reflectionProbeCount: usize = 0;
    pub const reflectionProbeData__sharedReflectionProbeCount: usize = 4;
    pub const reflectionProbeData__reflectionProbes: usize = 8;
    pub const reflectionProbeData__reflectionProbeArrayImage: usize = 16;
    pub const reflectionProbeData__probeRelightingCount: usize = 24;
    pub const reflectionProbeData__probeRelightingData: usize = 32;
    pub const reflectionProbeData__reflectionProbeGBufferImageCount: usize = 40;
    pub const reflectionProbeData__reflectionProbeGBufferImages: usize = 48;
    pub const reflectionProbeData__reflectionProbeGBufferTextures: usize = 56;
    pub const reflectionProbeData__reflectionProbeInstanceCount: usize = 64;
    pub const reflectionProbeData__reflectionProbeInstances: usize = 72;
    pub const reflectionProbeData__reflectionProbeLightgridSampleData: usize = 80;
    pub const reflectionProbeData__reflectionProbeLightgridSampleDataBuffer: usize = 88;
    pub const reflectionProbeData__reflectionProbeLightgridSampleDataBufferView: usize = 96;
    pub const reflectionProbeData__reflectionProbeLightgridSampleDataBufferRWView: usize = 104;
    pub const lightmapReindexData: usize = 112;
    pub const lightmapReindexData__imagePixelSize: usize = 112;
    pub const lightmapReindexData__reindexCount: usize = 116;
    pub const lightmapReindexData__reindexElement: usize = 120;
    pub const lightmapReindexData__packedLightmapCount: usize = 128;
    pub const lightmapReindexData__packedLightmap: usize = 136;
    pub const iesLookupTexture: usize = 144;
    pub const decalVolumeCollectionCount: usize = 152;
    pub const decalVolumeCollections: usize = 160;
    pub const lightmapOverridePrimary: usize = 168;
    pub const lightmapOverrideSecondary: usize = 176;
    pub const lightMapCount: usize = 184;
    pub const lightMaps: usize = 192;
    pub const lightmapTextures: usize = 200;
    pub const unused1: usize = 208;
    pub const unused2: usize = 216;
    pub const unused3: usize = 224;
    pub const transientZoneCount: usize = 232;
    pub const transientZones: usize = 240;
    pub const indexCount: usize = 496;
    pub const indices: usize = 504;
    pub const indexBuffer: usize = 512;
    pub const volumetrics: usize = 520;
    pub const volumetrics__volumetricCount: usize = 520;
    pub const volumetrics__volumetrics: usize = 528;
}
pub mod GfxWorldLightLists {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const surfaceListOffsetCount: usize = 0;
    pub const surfaceListOffsets: usize = 8;
    pub const smodelListOffsetCount: usize = 16;
    pub const smodelListOffsets: usize = 24;
    pub const listsSize: usize = 32;
    pub const lists: usize = 40;
}
pub mod GfxWorldLightmapReindexData {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const imagePixelSize: usize = 0;
    pub const reindexCount: usize = 4;
    pub const reindexElement: usize = 8;
    pub const packedLightmapCount: usize = 16;
    pub const packedLightmap: usize = 24;
}
pub mod GfxWorldLightmapReindexDataElement {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 4;
    pub const index: usize = 0;
    pub const posX: usize = 4;
    pub const posY: usize = 8;
    pub const width: usize = 12;
    pub const height: usize = 16;
}
pub mod GfxWorldPackedLightmap {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const imageWidth: usize = 0;
    pub const imageHeight: usize = 4;
}
pub mod GfxWorldReflectionProbeData {
    pub const SIZE: usize = 112;
    pub const ALIGN: usize = 8;
    pub const reflectionProbeCount: usize = 0;
    pub const sharedReflectionProbeCount: usize = 4;
    pub const reflectionProbes: usize = 8;
    pub const reflectionProbeArrayImage: usize = 16;
    pub const probeRelightingCount: usize = 24;
    pub const probeRelightingData: usize = 32;
    pub const reflectionProbeGBufferImageCount: usize = 40;
    pub const reflectionProbeGBufferImages: usize = 48;
    pub const reflectionProbeGBufferTextures: usize = 56;
    pub const reflectionProbeInstanceCount: usize = 64;
    pub const reflectionProbeInstances: usize = 72;
    pub const reflectionProbeLightgridSampleData: usize = 80;
    pub const reflectionProbeLightgridSampleDataBuffer: usize = 88;
    pub const reflectionProbeLightgridSampleDataBufferView: usize = 96;
    pub const reflectionProbeLightgridSampleDataBufferRWView: usize = 104;
}
pub mod GfxWorldTransientZone {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const transientZoneIndex: usize = 8;
    pub const vertexCount: usize = 12;
    pub const vd: usize = 16;
    pub const vd__vertices: usize = 16;
    pub const vd__worldVb: usize = 24;
    pub const vertexLayerDataSize: usize = 32;
    pub const vld: usize = 40;
    pub const vld__data: usize = 40;
    pub const vld__layerVb: usize = 48;
    pub const cellCount: usize = 56;
    pub const aabbTreeCounts: usize = 64;
    pub const aabbTrees: usize = 72;
}
pub mod GfxWorldVertex {
    pub const SIZE: usize = 44;
    pub const ALIGN: usize = 4;
    pub const xyz: usize = 0;
    pub const binormalSign: usize = 12;
    pub const color: usize = 16;
    pub const color__array: usize = 16;
    pub const color__packed: usize = 16;
    pub const texCoord: usize = 20;
    pub const lmapCoord: usize = 28;
    pub const normal: usize = 36;
    pub const normal__packed: usize = 36;
    pub const normal__array: usize = 36;
    pub const tangent: usize = 40;
    pub const tangent__packed: usize = 40;
    pub const tangent__array: usize = 40;
}
pub mod GfxWorldVertex0Union {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const vertices: usize = 0;
}
pub mod GfxWorldVertexData {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const vertices: usize = 0;
    pub const worldVb: usize = 8;
}
pub mod GfxWorldVertexLayerData {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const data: usize = 0;
    pub const layerVb: usize = 8;
}
pub mod GfxWorldVolumetrics {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const volumetricCount: usize = 0;
    pub const volumetrics: usize = 8;
}
pub mod GfxWrappedBaseBuffer {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const buffer: usize = 0;
}
pub mod GfxZoneTableEntry {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const dataPtr: usize = 0;
    pub const voidPtr: usize = 0;
    pub const buffer: usize = 0;
    pub const depthStencilState: usize = 0;
    pub const blendState: usize = 0;
}
pub mod GlassWorld {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const g_glassData: usize = 8;
}
pub mod GpuLightGridRequestRecord {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const lgvFrame: usize = 0;
    pub const lgvHistoryEntry: usize = 4;
    pub const lgvNumProbes: usize = 8;
}
pub mod GrenadeRotationParams {
    pub const SIZE: usize = 52;
    pub const ALIGN: usize = 4;
    pub const initialPitch: usize = 0;
    pub const initialYaw: usize = 4;
    pub const initialRoll: usize = 8;
    pub const rotationPitchDir: usize = 12;
    pub const rotationPitchMin: usize = 16;
    pub const rotationPitchMax: usize = 20;
    pub const rotationYawDir: usize = 24;
    pub const rotationYawMin: usize = 28;
    pub const rotationYawMax: usize = 32;
    pub const rotationRollDir: usize = 36;
    pub const rotationRollMin: usize = 40;
    pub const rotationRollMax: usize = 44;
    pub const rotate: usize = 48;
}
pub mod LaserDef {
    pub const SIZE: usize = 184;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const laserMaterial: usize = 8;
    pub const laserLightMaterial: usize = 16;
    pub const ownerOnly: usize = 24;
    pub const pad1: usize = 25;
    pub const range: usize = 28;
    pub const radius: usize = 32;
    pub const endOffsetViewmodel: usize = 36;
    pub const flarePct: usize = 40;
    pub const beamEffect: usize = 48;
    pub const beamEffect__u: usize = 48;
    pub const beamEffect__u__fx: usize = 48;
    pub const beamEffect__u__vfx: usize = 48;
    pub const beamEffect__u__data: usize = 48;
    pub const beamEffect__type: usize = 56;
    pub const laserEndEffect: usize = 64;
    pub const laserEndEffect__u: usize = 64;
    pub const laserEndEffect__u__fx: usize = 64;
    pub const laserEndEffect__u__vfx: usize = 64;
    pub const laserEndEffect__u__data: usize = 64;
    pub const laserEndEffect__type: usize = 72;
    pub const color: usize = 80;
    pub const hdrColorScale: usize = 96;
    pub const laserLight: usize = 112;
    pub const pad2: usize = 113;
    pub const laserLightRadius: usize = 116;
    pub const laserLightBeginOffset: usize = 120;
    pub const laserLightEndOffset: usize = 124;
    pub const laserLightBodyTweak: usize = 128;
    pub const laserLightColor: usize = 132;
    pub const laserLightHdrColorScale: usize = 148;
    pub const range_alt: usize = 164;
    pub const radius_alt: usize = 168;
    pub const laserLightRadius_alt: usize = 172;
    pub const flarePct_alt: usize = 176;
}
pub mod LbColumnDef {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const id: usize = 8;
    pub const propertyId: usize = 12;
    pub const hidden: usize = 16;
    pub const statsGroup: usize = 20;
    pub const statName: usize = 24;
    pub const r#type: usize = 32;
    pub const precision: usize = 36;
    pub const agg: usize = 40;
    pub const uiCalColX: usize = 44;
    pub const uiCalColY: usize = 48;
}
pub mod LbVrColumnDef {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const id: usize = 8;
    pub const uiCalColX: usize = 12;
    pub const uiCalColY: usize = 16;
}
pub mod LeaderboardDef {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const id: usize = 8;
    pub const sourceLbId: usize = 12;
    pub const sourceLbWidth: usize = 16;
    pub const columnCount: usize = 20;
    pub const xpColId: usize = 24;
    pub const prestigeColId: usize = 28;
    pub const columns: usize = 32;
    pub const updateType: usize = 40;
    pub const trackTypes: usize = 44;
    pub const rankColIdX: usize = 48;
    pub const rankColIdY: usize = 52;
}
pub mod LocalizeEntry {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const value: usize = 0;
    pub const name: usize = 8;
}
pub mod LuaFile {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const len: usize = 8;
    pub const strippingType: usize = 12;
    pub const buffer: usize = 16;
}
pub mod MapEnts {
    pub const SIZE: usize = 832;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const entityString: usize = 8;
    pub const numEntityChars: usize = 16;
    pub const trigger: usize = 24;
    pub const trigger__count: usize = 24;
    pub const trigger__models: usize = 32;
    pub const trigger__hullCount: usize = 40;
    pub const trigger__hulls: usize = 48;
    pub const trigger__slabCount: usize = 56;
    pub const trigger__slabs: usize = 64;
    pub const trigger__windingCount: usize = 72;
    pub const trigger__windings: usize = 80;
    pub const trigger__windingPointCount: usize = 88;
    pub const trigger__windingPoints: usize = 96;
    pub const clientTrigger: usize = 104;
    pub const clientTrigger__trigger: usize = 104;
    pub const clientTrigger__trigger__count: usize = 104;
    pub const clientTrigger__trigger__models: usize = 112;
    pub const clientTrigger__trigger__hullCount: usize = 120;
    pub const clientTrigger__trigger__hulls: usize = 128;
    pub const clientTrigger__trigger__slabCount: usize = 136;
    pub const clientTrigger__trigger__slabs: usize = 144;
    pub const clientTrigger__trigger__windingCount: usize = 152;
    pub const clientTrigger__trigger__windings: usize = 160;
    pub const clientTrigger__trigger__windingPointCount: usize = 168;
    pub const clientTrigger__trigger__windingPoints: usize = 176;
    pub const clientTrigger__triggerStringLength: usize = 184;
    pub const clientTrigger__triggerString: usize = 192;
    pub const clientTrigger__visionSetTriggers: usize = 200;
    pub const clientTrigger__triggerType: usize = 208;
    pub const clientTrigger__origins: usize = 216;
    pub const clientTrigger__scriptDelay: usize = 224;
    pub const clientTrigger__audioTriggers: usize = 232;
    pub const clientTrigger__blendLookup: usize = 240;
    pub const clientTrigger__npcTriggers: usize = 248;
    pub const clientTrigger__audioStateIds: usize = 256;
    pub const clientTrigger__audioRvbPanInfo: usize = 264;
    pub const clientTrigger__transientIndex: usize = 272;
    pub const clientTrigger__linkTo: usize = 280;
    pub const clientTriggerBlend: usize = 288;
    pub const clientTriggerBlend__numClientTriggerBlendNodes: usize = 288;
    pub const clientTriggerBlend__blendNodes: usize = 296;
    pub const spawnList: usize = 304;
    pub const spawnList__spawnsCount: usize = 304;
    pub const spawnList__spawns: usize = 312;
    pub const splineList: usize = 320;
    pub const splineList__splineCount: usize = 320;
    pub const splineList__splines: usize = 328;
    pub const havokEntsShapeDataSize: usize = 336;
    pub const havokEntsShapeData: usize = 344;
    pub const numSubModels: usize = 352;
    pub const cmodels: usize = 360;
    pub const dynEntCount: usize = 368;
    pub const dynEntCountTotal: usize = 372;
    pub const dynEntDefList: usize = 376;
    pub const dynEntPoseList: usize = 392;
    pub const dynEntClientList: usize = 424;
    pub const dynEntPhysicsSetupHead: usize = 456;
    pub const dynEntPhysicsSetupTail: usize = 464;
    pub const dynEntGlobalIdList: usize = 472;
    pub const __pad1: usize = 488;
    pub const dynEntTransientGroupCount: usize = 496;
    pub const dynEntTransientGroups: usize = 504;
    pub const dynEntTransientGroupRuntime: usize = 512;
    pub const dynEntTransientGroupState: usize = 528;
    pub const unk3Count: usize = 544;
    pub const unk3: usize = 552;
    pub const clientEntAnchorCount: usize = 560;
    pub const clientEntAnchors: usize = 568;
    pub const scriptableMapEnts: usize = 576;
    pub const scriptableMapEnts__totalInstanceCount: usize = 576;
    pub const scriptableMapEnts__runtimeInstanceCount: usize = 580;
    pub const scriptableMapEnts__reservedInstanceCount: usize = 584;
    pub const scriptableMapEnts__pad: usize = 588;
    pub const scriptableMapEnts__instances: usize = 592;
    pub const scriptableMapEnts__runtimeData: usize = 600;
    pub const scriptableMapEnts__runtimeData__partRuntimeCount: usize = 600;
    pub const scriptableMapEnts__runtimeData__partRuntime: usize = 608;
    pub const scriptableMapEnts__runtimeData__partRuntimeLocalClientCount: usize = 616;
    pub const scriptableMapEnts__runtimeData__partRuntimeLocalClient: usize = 624;
    pub const scriptableMapEnts__sunClientDatas: usize = 640;
    pub const scriptableMapEnts__reservedDynents: usize = 744;
    pub const scriptableMapEnts__ffMemCost: usize = 776;
    pub const numMayhemScenes: usize = 784;
    pub const mayhemScenes: usize = 792;
    pub const spawners: usize = 800;
    pub const spawners__spawnerCount: usize = 800;
    pub const spawners__spawnerList: usize = 808;
    pub const audioPASpeakerCount: usize = 816;
    pub const audioPASpeakers: usize = 824;
}
pub mod MapTriggers {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 8;
    pub const count: usize = 0;
    pub const models: usize = 8;
    pub const hullCount: usize = 16;
    pub const hulls: usize = 24;
    pub const slabCount: usize = 32;
    pub const slabs: usize = 40;
    pub const windingCount: usize = 48;
    pub const windings: usize = 56;
    pub const windingPointCount: usize = 64;
    pub const windingPoints: usize = 72;
}
pub mod Material {
    pub const SIZE: usize = 280;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const info: usize = 0;
    pub const info__name: usize = 0;
    pub const info__gameFlags: usize = 8;
    pub const info__unkFlags: usize = 9;
    pub const info__sortKey: usize = 10;
    pub const info__textureAtlasRowCount: usize = 11;
    pub const info__textureAtlasColumnCount: usize = 12;
    pub const info__textureAtlasFrameBlend: usize = 13;
    pub const info__textureAtlasAsArray: usize = 14;
    pub const info__renderFlags: usize = 15;
    pub const info__drawSurf: usize = 16;
    pub const info__drawSurf__fields: usize = 16;
    pub const info__drawSurf__fields__objectId: usize = 16;
    pub const info__drawSurf__fields__pad0: usize = 18;
    pub const info__drawSurf__fields__tessellation: usize = 22;
    pub const info__drawSurf__fields__pad1: usize = 22;
    pub const info__drawSurf__fields__pad2: usize = 24;
    pub const info__drawSurf__packed: usize = 16;
    pub const info__drawSurf__packed__p0: usize = 16;
    pub const info__drawSurf__packed__p1: usize = 24;
    pub const info__surfaceTypeBits: usize = 32;
    pub const info__hashIndex: usize = 40;
    pub const textureCount: usize = 48;
    pub const constantCount: usize = 49;
    pub const stateFlags: usize = 50;
    pub const cameraRegion: usize = 51;
    pub const materialType: usize = 52;
    pub const layerCount: usize = 53;
    pub const assetFlags: usize = 54;
    pub const pad: usize = 55;
    pub const techniqueSet: usize = 56;
    pub const textureTable: usize = 64;
    pub const constantTable: usize = 72;
    pub const stateBitsEntry: usize = 80;
    pub const stateBitsCount: usize = 232;
    pub const stateBitsTable: usize = 240;
    pub const constantBufferIndex: usize = 248;
    pub const constantBufferTable: usize = 256;
    pub const constantBufferCount: usize = 264;
    pub const subMaterials: usize = 272;
}
pub mod MaterialArgumentCodeConst {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 2;
    pub const index: usize = 0;
    pub const firstRow: usize = 2;
    pub const rowCount: usize = 3;
}
pub mod MaterialArgumentDef {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const literalConst: usize = 0;
    pub const codeConst: usize = 0;
    pub const codeConst__index: usize = 0;
    pub const codeConst__firstRow: usize = 2;
    pub const codeConst__rowCount: usize = 3;
    pub const codeSampler: usize = 0;
    pub const nameHash: usize = 0;
}
pub mod MaterialConstantBufferDef {
    pub const SIZE: usize = 128;
    pub const ALIGN: usize = 8;
    pub const vsDataSize: usize = 0;
    pub const hsDataSize: usize = 4;
    pub const dsDataSize: usize = 8;
    pub const psDataSize: usize = 12;
    pub const vsOffsetDataSize: usize = 16;
    pub const hsOffsetDataSize: usize = 20;
    pub const dsOffsetDataSize: usize = 24;
    pub const psOffsetDataSize: usize = 28;
    pub const vsData: usize = 32;
    pub const hsData: usize = 40;
    pub const dsData: usize = 48;
    pub const psData: usize = 56;
    pub const vsOffsetData: usize = 64;
    pub const hsOffsetData: usize = 72;
    pub const dsOffsetData: usize = 80;
    pub const psOffsetData: usize = 88;
    pub const vsConstantBuffer: usize = 96;
    pub const hsConstantBuffer: usize = 104;
    pub const dsConstantBuffer: usize = 112;
    pub const psConstantBuffer: usize = 120;
}
pub mod MaterialConstantDef {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const nameHash: usize = 0;
    pub const name: usize = 4;
    pub const literal: usize = 16;
}
pub mod MaterialDomainShader {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const debugName: usize = 8;
    pub const prog: usize = 16;
    pub const prog__ds: usize = 16;
    pub const prog__loadDef: usize = 24;
    pub const prog__loadDef__program: usize = 24;
    pub const prog__loadDef__programSize: usize = 32;
}
pub mod MaterialDomainShaderProgram {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const ds: usize = 0;
    pub const loadDef: usize = 8;
    pub const loadDef__program: usize = 8;
    pub const loadDef__programSize: usize = 16;
}
pub mod MaterialHullShader {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const debugName: usize = 8;
    pub const prog: usize = 16;
    pub const prog__hs: usize = 16;
    pub const prog__loadDef: usize = 24;
    pub const prog__loadDef__program: usize = 24;
    pub const prog__loadDef__programSize: usize = 32;
}
pub mod MaterialHullShaderProgram {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const hs: usize = 0;
    pub const loadDef: usize = 8;
    pub const loadDef__program: usize = 8;
    pub const loadDef__programSize: usize = 16;
}
pub mod MaterialInfo {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const gameFlags: usize = 8;
    pub const unkFlags: usize = 9;
    pub const sortKey: usize = 10;
    pub const textureAtlasRowCount: usize = 11;
    pub const textureAtlasColumnCount: usize = 12;
    pub const textureAtlasFrameBlend: usize = 13;
    pub const textureAtlasAsArray: usize = 14;
    pub const renderFlags: usize = 15;
    pub const drawSurf: usize = 16;
    pub const drawSurf__fields: usize = 16;
    pub const drawSurf__fields__objectId: usize = 16;
    pub const drawSurf__fields__pad0: usize = 18;
    pub const drawSurf__fields__tessellation: usize = 22;
    pub const drawSurf__fields__pad1: usize = 22;
    pub const drawSurf__fields__pad2: usize = 24;
    pub const drawSurf__packed: usize = 16;
    pub const drawSurf__packed__p0: usize = 16;
    pub const drawSurf__packed__p1: usize = 24;
    pub const surfaceTypeBits: usize = 32;
    pub const hashIndex: usize = 40;
}
pub mod MaterialMemory {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const material: usize = 0;
    pub const memory: usize = 8;
}
pub mod MaterialPass {
    pub const SIZE: usize = 104;
    pub const ALIGN: usize = 8;
    pub const vertexShader: usize = 0;
    pub const vertexDecl: usize = 8;
    pub const hullShader: usize = 16;
    pub const domainShader: usize = 24;
    pub const pixelShader: usize = 32;
    pub const pixelOutputMask: usize = 40;
    pub const perPrimArgCount: usize = 41;
    pub const perObjArgCount: usize = 42;
    pub const stableArgCount: usize = 43;
    pub const perPrimArgSize: usize = 44;
    pub const perObjArgSize: usize = 46;
    pub const stableArgSize: usize = 48;
    pub const zone: usize = 50;
    pub const perPrimConstantBuffer: usize = 52;
    pub const perObjConstantBuffer: usize = 53;
    pub const stableConstantBuffer: usize = 54;
    pub const customBufferFlags: usize = 56;
    pub const customSamplerFlags: usize = 60;
    pub const precompiledIndex: usize = 61;
    pub const stageConfig: usize = 62;
    pub const __pad0: usize = 63;
    pub const unk: usize = 64;
    pub const args: usize = 96;
}
pub mod MaterialPixelShader {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const debugName: usize = 8;
    pub const prog: usize = 16;
    pub const prog__ps: usize = 16;
    pub const prog__loadDef: usize = 24;
    pub const prog__loadDef__program: usize = 24;
    pub const prog__loadDef__programSize: usize = 32;
    pub const prog__loadDef__microCodeCrc: usize = 36;
}
pub mod MaterialPixelShaderProgram {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const ps: usize = 0;
    pub const loadDef: usize = 8;
    pub const loadDef__program: usize = 8;
    pub const loadDef__programSize: usize = 16;
    pub const loadDef__microCodeCrc: usize = 20;
}
pub mod MaterialShaderArgument {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const shader: usize = 1;
    pub const dest: usize = 2;
    pub const u: usize = 8;
    pub const u__literalConst: usize = 8;
    pub const u__codeConst: usize = 8;
    pub const u__codeConst__index: usize = 8;
    pub const u__codeConst__firstRow: usize = 10;
    pub const u__codeConst__rowCount: usize = 11;
    pub const u__codeSampler: usize = 8;
    pub const u__nameHash: usize = 8;
}
pub mod MaterialStreamRouting {
    pub const SIZE: usize = 3;
    pub const ALIGN: usize = 1;
    pub const source: usize = 0;
    pub const dest: usize = 1;
    pub const mask: usize = 2;
}
pub mod MaterialTechnique {
    pub const SIZE: usize = 120;
    pub const ALIGN: usize = 8;
    pub const hdr: usize = 0;
    pub const hdr__name: usize = 0;
    pub const hdr__flags: usize = 8;
    pub const hdr__passCount: usize = 10;
    pub const passArray: usize = 16;
}
pub mod MaterialTechniqueHeader {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const flags: usize = 8;
    pub const passCount: usize = 10;
}
pub mod MaterialTechniqueSet {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const flags: usize = 8;
    pub const worldVertFormat: usize = 10;
    pub const preDisplacementOnlyCount: usize = 11;
    pub const techniqueCount: usize = 12;
    pub const techniqueMaskStartIndex: usize = 14;
    pub const unk: usize = 20;
    pub const techniqueMask: usize = 24;
    pub const techniques: usize = 48;
}
pub mod MaterialTextureDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const nameHash: usize = 0;
    pub const nameStart: usize = 4;
    pub const nameEnd: usize = 5;
    pub const samplerState: usize = 6;
    pub const semantic: usize = 7;
    pub const image: usize = 8;
}
pub mod MaterialVertexDeclaration {
    pub const SIZE: usize = 2280;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const streamCount: usize = 8;
    pub const hasOptionalSource: usize = 9;
    pub const routing: usize = 16;
    pub const routing__data: usize = 16;
    pub const routing__decl: usize = 120;
}
pub mod MaterialVertexShader {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const debugName: usize = 8;
    pub const prog: usize = 16;
    pub const prog__vs: usize = 16;
    pub const prog__loadDef: usize = 24;
    pub const prog__loadDef__program: usize = 24;
    pub const prog__loadDef__programSize: usize = 32;
    pub const prog__loadDef__microCodeCrc: usize = 36;
}
pub mod MaterialVertexShaderProgram {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const vs: usize = 0;
    pub const loadDef: usize = 8;
    pub const loadDef__program: usize = 8;
    pub const loadDef__programSize: usize = 16;
    pub const loadDef__microCodeCrc: usize = 20;
}
pub mod MaterialVertexStreamRouting {
    pub const SIZE: usize = 2264;
    pub const ALIGN: usize = 8;
    pub const data: usize = 0;
    pub const decl: usize = 104;
}
pub mod MayhemAnim {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const frames: usize = 0;
    pub const frames__data: usize = 0;
    pub const frames__splineCompressedFrames: usize = 0;
    pub const frames__uncompressedFrames: usize = 0;
    pub const numBones: usize = 8;
    pub const numFrames: usize = 12;
    pub const framerate: usize = 16;
    pub const frequency: usize = 20;
    pub const notifyCount: usize = 24;
    pub const dataChannelCount: usize = 28;
    pub const isLooping: usize = 32;
    pub const isSplineCompressed: usize = 33;
    pub const quantizeQuats: usize = 34;
    pub const quantizeTrans: usize = 35;
    pub const quantizeChannels: usize = 36;
    pub const notify: usize = 40;
    pub const dataChannels: usize = 48;
    pub const dataChannels__data: usize = 48;
    pub const dataChannels__splineCompressedKeys: usize = 48;
    pub const dataChannels__uncompressedKeys: usize = 48;
}
pub mod MayhemAnimFramesSplineCompressed {
    pub const SIZE: usize = 88;
    pub const ALIGN: usize = 8;
    pub const diskQuat: usize = 0;
    pub const diskPos: usize = 8;
    pub const diskQuatFrames: usize = 16;
    pub const diskPosFrames: usize = 24;
    pub const numDiskQuatFrames: usize = 32;
    pub const numDiskPosFrames: usize = 40;
    pub const transBounds: usize = 48;
    pub const quat: usize = 56;
    pub const quat__buffer: usize = 56;
    pub const pos: usize = 64;
    pub const pos__buffer: usize = 64;
    pub const quatStride: usize = 72;
    pub const posStride: usize = 76;
    pub const totalCompressedQuatFrames: usize = 80;
    pub const totalCompressedPosFrames: usize = 84;
}
pub mod MayhemAnimFramesUncompressed {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const diskQuat: usize = 0;
    pub const diskPos: usize = 8;
    pub const transBounds: usize = 16;
    pub const quat: usize = 24;
    pub const quat__buffer: usize = 24;
    pub const pos: usize = 32;
    pub const pos__buffer: usize = 32;
    pub const quatStride: usize = 40;
    pub const posStride: usize = 44;
}
pub mod MayhemData {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const preModel: usize = 8;
    pub const postModel: usize = 16;
    pub const numModels: usize = 24;
    pub const models: usize = 32;
    pub const numAnims: usize = 40;
    pub const anims: usize = 48;
    pub const numObjects: usize = 56;
    pub const objects: usize = 64;
    pub const length: usize = 72;
}
pub mod MayhemDataChannelsUnion {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const data: usize = 0;
    pub const splineCompressedKeys: usize = 0;
    pub const uncompressedKeys: usize = 0;
}
pub mod MayhemDataKeysSplineCompressed {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const keys: usize = 0;
    pub const numKeys: usize = 8;
    pub const keyFrames: usize = 16;
    pub const totalCompressedKeyFrames: usize = 24;
    pub const numStreams: usize = 28;
    pub const keyStride: usize = 32;
}
pub mod MayhemDataKeysUncompressed {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const keys: usize = 0;
    pub const numStreams: usize = 8;
    pub const keyStride: usize = 12;
}
pub mod MayhemFramesUnion {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const data: usize = 0;
    pub const splineCompressedFrames: usize = 0;
    pub const uncompressedFrames: usize = 0;
}
pub mod MayhemInstance {
    pub const SIZE: usize = 72;
    pub const ALIGN: usize = 8;
    pub const initialPose: usize = 0;
    pub const initialPose__quat: usize = 0;
    pub const initialPose__origin: usize = 16;
    pub const mayhem: usize = 32;
    pub const linkTo: usize = 40;
    pub const scriptName: usize = 48;
    pub const curTime: usize = 52;
    pub const oldTime: usize = 56;
    pub const initialFlags: usize = 60;
    pub const flags: usize = 64;
    pub const mapEntLookup: usize = 68;
}
pub mod MayhemModel {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const model: usize = 0;
}
pub mod MayhemObject {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const modelIndex: usize = 0;
    pub const animIndex: usize = 4;
}
pub mod MayhemTransBounds {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 4;
    pub const mins: usize = 0;
    pub const size: usize = 12;
}
pub mod MixDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const id: usize = 0;
    pub const volModIndex: usize = 4;
    pub const volume: usize = 8;
    pub const fade: usize = 12;
}
pub mod MovementTime {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const time: usize = 0;
    pub const isLeadIn: usize = 4;
}
pub mod NavMeshData {
    pub const SIZE: usize = 144;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const version: usize = 8;
    pub const hasExposureData: usize = 12;
    pub const numNavResources: usize = 16;
    pub const navResources: usize = 24;
    pub const numObstacleBounds: usize = 32;
    pub const obstacleBounds: usize = 40;
    pub const numGlassBounds: usize = 48;
    pub const glassBounds: usize = 56;
    pub const numModifiers: usize = 64;
    pub const modifiers: usize = 72;
    pub const numLinkCreationData: usize = 80;
    pub const linkCreationData: usize = 88;
    pub const numVolumeSeeds: usize = 96;
    pub const volumeSeeds: usize = 104;
    pub const numRawVolumes: usize = 112;
    pub const rawVolumes: usize = 120;
    pub const numRawCustomVolumes: usize = 128;
    pub const rawCustomVolumes: usize = 136;
}
pub mod NetConstStrings {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const stringType: usize = 8;
    pub const sourceType: usize = 12;
    pub const flags: usize = 16;
    pub const entryCount: usize = 20;
    pub const stringList: usize = 24;
}
pub mod NoteTrackToSoundEntry {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const numSoundMappings: usize = 0;
    pub const attachment: usize = 4;
    pub const notetrackSoundMapKeys: usize = 8;
    pub const notetrackSoundMapValues: usize = 16;
}
pub mod NoteTrackToSuitSoundEntry {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const numSoundMappings: usize = 0;
    pub const suitAnimState: usize = 4;
    pub const notetrackSoundMapKeys: usize = 8;
    pub const notetrackSoundMapValues: usize = 16;
}
pub mod OccludeDef {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 4;
    pub const id: usize = 0;
    pub const entChannelIdx: usize = 4;
    pub const cutoffLevel1: usize = 8;
    pub const cutoffLevel2: usize = 12;
    pub const cutoffLevel3: usize = 16;
    pub const cutoffLevel4: usize = 20;
    pub const attnLevel1: usize = 24;
    pub const attnLevel2: usize = 28;
    pub const attnLevel3: usize = 32;
    pub const attnLevel4: usize = 36;
}
pub mod OcclusionShape {
    pub const SIZE: usize = 96;
    pub const ALIGN: usize = 4;
    pub const name: usize = 0;
    pub const id: usize = 64;
    pub const unk: usize = 68;
    pub const innerAngleVolume: usize = 72;
    pub const outerAngleVolume: usize = 76;
    pub const occludedVolume: usize = 80;
    pub const innerAngleLPF: usize = 84;
    pub const outerAngleLPF: usize = 88;
    pub const occludedLPF: usize = 92;
}
pub mod Packed128 {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const p0: usize = 0;
    pub const p1: usize = 8;
}
pub mod PackedSelfVisibility {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 4;
    pub const packed: usize = 0;
}
pub mod PackedTexCoords {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 4;
    pub const packed: usize = 0;
    pub const array: usize = 0;
}
pub mod PackedUnitVec {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 4;
    pub const packed: usize = 0;
    pub const array: usize = 0;
}
pub mod ParticleCurveControlPointDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const time: usize = 0;
    pub const value: usize = 4;
    pub const invTimeDelta: usize = 8;
    pub const pad: usize = 12;
}
pub mod ParticleCurveDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const controlPoints: usize = 0;
    pub const numControlPoints: usize = 8;
    pub const scale: usize = 12;
}
pub mod ParticleEmitterDef {
    pub const SIZE: usize = 144;
    pub const ALIGN: usize = 8;
    pub const stateDefs: usize = 0;
    pub const numStates: usize = 8;
    pub const particleSpawnRate: usize = 12;
    pub const particleSpawnRate__min: usize = 12;
    pub const particleSpawnRate__max: usize = 16;
    pub const particleLife: usize = 20;
    pub const particleLife__min: usize = 20;
    pub const particleLife__max: usize = 24;
    pub const particleDelay: usize = 28;
    pub const particleDelay__min: usize = 28;
    pub const particleDelay__max: usize = 32;
    pub const particleCountMax: usize = 36;
    pub const particleBurstCount: usize = 40;
    pub const particleBurstCount__min: usize = 40;
    pub const particleBurstCount__max: usize = 44;
    pub const emitterLife: usize = 48;
    pub const emitterLife__min: usize = 48;
    pub const emitterLife__max: usize = 52;
    pub const emitterDelay: usize = 56;
    pub const emitterDelay__min: usize = 56;
    pub const emitterDelay__max: usize = 60;
    pub const randomSeed: usize = 64;
    pub const spawnRangeSq: usize = 68;
    pub const spawnRangeSq__min: usize = 68;
    pub const spawnRangeSq__max: usize = 72;
    pub const fadeOutMaxDistance: usize = 76;
    pub const fadeCurveDef: usize = 80;
    pub const fadeCurveDef__controlPoints: usize = 80;
    pub const fadeCurveDef__numControlPoints: usize = 88;
    pub const fadeCurveDef__scale: usize = 92;
    pub const spawnFrustumCullRadius: usize = 96;
    pub const flags: usize = 100;
    pub const gravityOptions: usize = 104;
    pub const groupIDs: usize = 108;
    pub const unk1: usize = 124;
    pub const unk2: usize = 128;
    pub const particleSpawnShapeRange: usize = 132;
    pub const particleSpawnShapeRange__min: usize = 132;
    pub const particleSpawnShapeRange__max: usize = 136;
    pub const pad: usize = 140;
}
pub mod ParticleFloatRange {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const min: usize = 0;
    pub const max: usize = 4;
}
pub mod ParticleIntRange {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const min: usize = 0;
    pub const max: usize = 4;
}
pub mod ParticleLinkedAssetDef {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const material: usize = 0;
    pub const model: usize = 0;
    pub const particleSystem: usize = 0;
    pub const physicsFXData: usize = 0;
    pub const physicsFXData__physicsFXPipeline: usize = 0;
    pub const physicsFXData__physicsFXShape: usize = 8;
    pub const particleSim: usize = 0;
    pub const decal: usize = 0;
    pub const decal__materials: usize = 0;
    pub const sound: usize = 0;
    pub const vectorField: usize = 0;
    pub const lightDef: usize = 0;
    pub const pad: usize = 0;
}
pub mod ParticleLinkedAssetListDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const assetList: usize = 0;
    pub const numAssets: usize = 8;
}
pub mod ParticleMarkVisuals {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const materials: usize = 0;
}
pub mod ParticleModule {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
}
pub mod ParticleModuleAttractor {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_pad: usize = 8;
    pub const m_killWhenNear: usize = 20;
    pub const m_forceMag: usize = 24;
    pub const m_nearDistanceSq: usize = 28;
    pub const m_attractPoint: usize = 32;
    pub const m_attractPoint__v: usize = 32;
}
pub mod ParticleModuleColorGraph {
    pub const SIZE: usize = 144;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const firstCurve: usize = 8;
    pub const m_pad: usize = 9;
    pub const m_modulateColorByAlpha: usize = 12;
    pub const m_curves: usize = 16;
}
pub mod ParticleModuleColorLerp {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_pad: usize = 8;
    pub const m_colorBegin: usize = 16;
    pub const m_colorBegin__v: usize = 16;
    pub const m_colorEnd: usize = 32;
    pub const m_colorEnd__v: usize = 32;
}
pub mod ParticleModuleDef {
    pub const SIZE: usize = 160;
    pub const ALIGN: usize = 8;
    pub const moduleType: usize = 0;
    pub const m_pad: usize = 4;
    pub const moduleData: usize = 16;
    pub const moduleData__moduleBase: usize = 16;
    pub const moduleData__moduleBase__type: usize = 16;
    pub const moduleData__moduleBase__m_flags: usize = 20;
    pub const moduleData__initAtlas: usize = 16;
    pub const moduleData__initAtlas__type: usize = 16;
    pub const moduleData__initAtlas__m_flags: usize = 20;
    pub const moduleData__initAtlas__m_playRate: usize = 24;
    pub const moduleData__initAtlas__m_startFrame: usize = 28;
    pub const moduleData__initAtlas__m_loopCount: usize = 32;
    pub const moduleData__initAttributes: usize = 16;
    pub const moduleData__initAttributes__type: usize = 16;
    pub const moduleData__initAttributes__m_flags: usize = 20;
    pub const moduleData__initAttributes__m_useNonUniformInterpolationForColor: usize = 24;
    pub const moduleData__initAttributes__m_useNonUniformInterpolationForSize: usize = 25;
    pub const moduleData__initAttributes__m_pad: usize = 26;
    pub const moduleData__initAttributes__m_sizeMin: usize = 32;
    pub const moduleData__initAttributes__m_sizeMin__v: usize = 32;
    pub const moduleData__initAttributes__m_sizeMax: usize = 48;
    pub const moduleData__initAttributes__m_sizeMax__v: usize = 48;
    pub const moduleData__initAttributes__m_colorMin: usize = 64;
    pub const moduleData__initAttributes__m_colorMin__v: usize = 64;
    pub const moduleData__initAttributes__m_colorMax: usize = 80;
    pub const moduleData__initAttributes__m_colorMax__v: usize = 80;
    pub const moduleData__initAttributes__m_velocityMin: usize = 96;
    pub const moduleData__initAttributes__m_velocityMin__v: usize = 96;
    pub const moduleData__initAttributes__m_velocityMax: usize = 112;
    pub const moduleData__initAttributes__m_velocityMax__v: usize = 112;
    pub const moduleData__initBeam: usize = 16;
    pub const moduleData__initBeam__type: usize = 16;
    pub const moduleData__initBeam__m_flags: usize = 20;
    pub const moduleData__initBeam__m_beamFlags: usize = 24;
    pub const moduleData__initBeam__m_splitDistance: usize = 28;
    pub const moduleData__initBeam__m_startPos: usize = 32;
    pub const moduleData__initBeam__m_startPos__v: usize = 32;
    pub const moduleData__initBeam__m_endPos: usize = 48;
    pub const moduleData__initBeam__m_endPos__v: usize = 48;
    pub const moduleData__initBeam__m_offset: usize = 64;
    pub const moduleData__initBeam__m_offset__v: usize = 64;
    pub const moduleData__initBeam__m_curvePoint1: usize = 80;
    pub const moduleData__initBeam__m_curvePoint1__v: usize = 80;
    pub const moduleData__initBeam__m_curvePoint2: usize = 96;
    pub const moduleData__initBeam__m_curvePoint2__v: usize = 96;
    pub const moduleData__initBeam__m_linkedAssetList: usize = 112;
    pub const moduleData__initBeam__m_linkedAssetList__assetList: usize = 112;
    pub const moduleData__initBeam__m_linkedAssetList__numAssets: usize = 120;
    pub const moduleData__initBeam__m_scrollRateUV: usize = 128;
    pub const moduleData__initBeam__m_scrollRateSpiralGraph: usize = 132;
    pub const moduleData__initBeam__m_spiralRepeatDistance: usize = 136;
    pub const moduleData__initCameraOffset: usize = 16;
    pub const moduleData__initCameraOffset__type: usize = 16;
    pub const moduleData__initCameraOffset__m_flags: usize = 20;
    pub const moduleData__initCameraOffset__m_pad: usize = 24;
    pub const moduleData__initCloud: usize = 16;
    pub const moduleData__initCloud__type: usize = 16;
    pub const moduleData__initCloud__m_flags: usize = 20;
    pub const moduleData__initCloud__m_pad: usize = 24;
    pub const moduleData__initCloud__curves: usize = 32;
    pub const moduleData__initDecal: usize = 16;
    pub const moduleData__initDecal__type: usize = 16;
    pub const moduleData__initDecal__m_flags: usize = 20;
    pub const moduleData__initDecal__m_fadeInTime: usize = 24;
    pub const moduleData__initDecal__m_fadeOutTime: usize = 26;
    pub const moduleData__initDecal__m_stoppableFadeOutTime: usize = 28;
    pub const moduleData__initDecal__m_lerpWaitTime: usize = 30;
    pub const moduleData__initDecal__m_lerpColor: usize = 32;
    pub const moduleData__initDecal__m_lerpColor__v: usize = 32;
    pub const moduleData__initDecal__m_linkedAssetList: usize = 48;
    pub const moduleData__initDecal__m_linkedAssetList__assetList: usize = 48;
    pub const moduleData__initDecal__m_linkedAssetList__numAssets: usize = 56;
    pub const moduleData__initFlare: usize = 16;
    pub const moduleData__initFlare__type: usize = 16;
    pub const moduleData__initFlare__m_flags: usize = 20;
    pub const moduleData__initFlare__m_pad: usize = 24;
    pub const moduleData__initFlare__m_position: usize = 28;
    pub const moduleData__initFlare__m_direction: usize = 32;
    pub const moduleData__initFlare__m_direction__v: usize = 32;
    pub const moduleData__initFlare__m_angularRotCount: usize = 48;
    pub const moduleData__initFlare__m_flareFlags: usize = 52;
    pub const moduleData__initFlare__m_depthScaleRange: usize = 56;
    pub const moduleData__initFlare__m_depthScaleRange__min: usize = 56;
    pub const moduleData__initFlare__m_depthScaleRange__max: usize = 60;
    pub const moduleData__initFlare__m_depthScaleValue: usize = 64;
    pub const moduleData__initFlare__m_depthScaleValue__min: usize = 64;
    pub const moduleData__initFlare__m_depthScaleValue__max: usize = 68;
    pub const moduleData__initFlare__m_radialRot: usize = 72;
    pub const moduleData__initFlare__m_radialRot__min: usize = 72;
    pub const moduleData__initFlare__m_radialRot__max: usize = 76;
    pub const moduleData__initFlare__m_radialScaleX: usize = 80;
    pub const moduleData__initFlare__m_radialScaleX__min: usize = 80;
    pub const moduleData__initFlare__m_radialScaleX__max: usize = 84;
    pub const moduleData__initFlare__m_radialScaleY: usize = 88;
    pub const moduleData__initFlare__m_radialScaleY__min: usize = 88;
    pub const moduleData__initFlare__m_radialScaleY__max: usize = 92;
    pub const moduleData__initFlare__m_curves: usize = 96;
    pub const moduleData__initGeoTrail: usize = 16;
    pub const moduleData__initGeoTrail__type: usize = 16;
    pub const moduleData__initGeoTrail__m_flags: usize = 20;
    pub const moduleData__initGeoTrail__m_numPointsMax: usize = 24;
    pub const moduleData__initGeoTrail__m_splitDistance: usize = 28;
    pub const moduleData__initGeoTrail__m_splitAngle: usize = 32;
    pub const moduleData__initGeoTrail__m_centerOffset: usize = 36;
    pub const moduleData__initGeoTrail__m_numSheets: usize = 40;
    pub const moduleData__initGeoTrail__m_fadeInDistance: usize = 44;
    pub const moduleData__initGeoTrail__m_fadeOutDistance: usize = 48;
    pub const moduleData__initGeoTrail__m_tileDistance: usize = 52;
    pub const moduleData__initGeoTrail__m_tileOffset: usize = 56;
    pub const moduleData__initGeoTrail__m_tileOffset__min: usize = 56;
    pub const moduleData__initGeoTrail__m_tileOffset__max: usize = 60;
    pub const moduleData__initGeoTrail__m_scrollTime: usize = 64;
    pub const moduleData__initGeoTrail__m_useLocalVelocity: usize = 68;
    pub const moduleData__initGeoTrail__m_useVerticalTexture: usize = 69;
    pub const moduleData__initGeoTrail__m_cameraFacing: usize = 70;
    pub const moduleData__initGeoTrail__m_fixLeadingEdge: usize = 71;
    pub const moduleData__initGeoTrail__m_clampUVs: usize = 72;
    pub const moduleData__initGeoTrail__m_pad: usize = 73;
    pub const moduleData__initGeoTrail__m_pad2: usize = 76;
    pub const moduleData__initLightOmni: usize = 16;
    pub const moduleData__initLightOmni__type: usize = 16;
    pub const moduleData__initLightOmni__m_flags: usize = 20;
    pub const moduleData__initLightOmni__m_linkedAssetList: usize = 24;
    pub const moduleData__initLightOmni__m_linkedAssetList__assetList: usize = 24;
    pub const moduleData__initLightOmni__m_linkedAssetList__numAssets: usize = 32;
    pub const moduleData__initLightOmni__m_tonemappingScaleFactor: usize = 40;
    pub const moduleData__initLightOmni__m_intensityIR: usize = 44;
    pub const moduleData__initLightOmni__m_disableVolumetric: usize = 48;
    pub const moduleData__initLightOmni__m_exponent: usize = 49;
    pub const moduleData__initLightOmni____pad0: usize = 50;
    pub const moduleData__initLightOmni__m_shadowSoftness: usize = 52;
    pub const moduleData__initLightOmni__m_shadowBias: usize = 56;
    pub const moduleData__initLightOmni__m_shadowArea: usize = 60;
    pub const moduleData__initLightSpot: usize = 16;
    pub const moduleData__initLightSpot__type: usize = 16;
    pub const moduleData__initLightSpot__m_flags: usize = 20;
    pub const moduleData__initLightSpot__m_fovOuter: usize = 24;
    pub const moduleData__initLightSpot__m_fovInner: usize = 28;
    pub const moduleData__initLightSpot__m_bulbRadius: usize = 32;
    pub const moduleData__initLightSpot__m_bulbLength: usize = 36;
    pub const moduleData__initLightSpot__m_brightness: usize = 40;
    pub const moduleData__initLightSpot__unk1: usize = 44;
    pub const moduleData__initLightSpot__m_intensityUV: usize = 48;
    pub const moduleData__initLightSpot__m_intensityIR: usize = 52;
    pub const moduleData__initLightSpot__m_shadowNearPlane: usize = 56;
    pub const moduleData__initLightSpot__m_toneMappingScaleFactor: usize = 60;
    pub const moduleData__initLightSpot__m_shadowSoftness: usize = 64;
    pub const moduleData__initLightSpot__m_shadowBias: usize = 68;
    pub const moduleData__initLightSpot__m_shadowArea: usize = 72;
    pub const moduleData__initLightSpot__m_disableVolumetric: usize = 76;
    pub const moduleData__initLightSpot__m_disableShadowMap: usize = 77;
    pub const moduleData__initLightSpot__m_disableDynamicShadows: usize = 78;
    pub const moduleData__initLightSpot__m_exponent: usize = 79;
    pub const moduleData__initLightSpot__m_linkedAssetList: usize = 80;
    pub const moduleData__initLightSpot__m_linkedAssetList__assetList: usize = 80;
    pub const moduleData__initLightSpot__m_linkedAssetList__numAssets: usize = 88;
    pub const moduleData__initMaterial: usize = 16;
    pub const moduleData__initMaterial__type: usize = 16;
    pub const moduleData__initMaterial__m_flags: usize = 20;
    pub const moduleData__initMaterial__m_pad: usize = 24;
    pub const moduleData__initMaterial__m_linkedAssetList: usize = 32;
    pub const moduleData__initMaterial__m_linkedAssetList__assetList: usize = 32;
    pub const moduleData__initMaterial__m_linkedAssetList__numAssets: usize = 40;
    pub const moduleData__initMirrorTexture: usize = 16;
    pub const moduleData__initMirrorTexture__type: usize = 16;
    pub const moduleData__initMirrorTexture__m_flags: usize = 20;
    pub const moduleData__initMirrorTexture__m_mirrorHorizontal: usize = 24;
    pub const moduleData__initMirrorTexture__m_mirrorVertical: usize = 28;
    pub const moduleData__initModel: usize = 16;
    pub const moduleData__initModel__type: usize = 16;
    pub const moduleData__initModel__m_flags: usize = 20;
    pub const moduleData__initModel__m_usePhysics: usize = 24;
    pub const moduleData__initModel__m_motionBlurHQ: usize = 25;
    pub const moduleData__initModel__m_linkedAssetList: usize = 32;
    pub const moduleData__initModel__m_linkedAssetList__assetList: usize = 32;
    pub const moduleData__initModel__m_linkedAssetList__numAssets: usize = 40;
    pub const moduleData__initOcclusionQuery: usize = 16;
    pub const moduleData__initOcclusionQuery__type: usize = 16;
    pub const moduleData__initOcclusionQuery__m_flags: usize = 20;
    pub const moduleData__initOcclusionQuery__m_fadeInTime: usize = 24;
    pub const moduleData__initOcclusionQuery__m_fadeOutTime: usize = 28;
    pub const moduleData__initOcclusionQuery__m_scale: usize = 32;
    pub const moduleData__initOcclusionQuery__m_scale__min: usize = 32;
    pub const moduleData__initOcclusionQuery__m_scale__max: usize = 36;
    pub const moduleData__initOcclusionQuery__m_worldSize: usize = 40;
    pub const moduleData__initOrientedSprite: usize = 16;
    pub const moduleData__initOrientedSprite__type: usize = 16;
    pub const moduleData__initOrientedSprite__m_flags: usize = 20;
    pub const moduleData__initOrientedSprite__m_pad: usize = 24;
    pub const moduleData__initOrientedSprite__m_orientationQuat: usize = 32;
    pub const moduleData__initOrientedSprite__m_orientationQuat__v: usize = 32;
    pub const moduleData__initParticleSim: usize = 16;
    pub const moduleData__initParticleSim__type: usize = 16;
    pub const moduleData__initParticleSim__m_flags: usize = 20;
    pub const moduleData__initParticleSim__m_linkedAssetList: usize = 24;
    pub const moduleData__initParticleSim__m_linkedAssetList__assetList: usize = 24;
    pub const moduleData__initParticleSim__m_linkedAssetList__numAssets: usize = 32;
    pub const moduleData__initParticleSim__m_scaleFactor: usize = 40;
    pub const moduleData__initParticleSim__m_scaleFactor__min: usize = 40;
    pub const moduleData__initParticleSim__m_scaleFactor__max: usize = 44;
    pub const moduleData__initParticleSim__m_holdLastFrame: usize = 48;
    pub const moduleData__initParticleSim__m_pad: usize = 52;
    pub const moduleData__initPlayerFacing: usize = 16;
    pub const moduleData__initPlayerFacing__type: usize = 16;
    pub const moduleData__initPlayerFacing__m_flags: usize = 20;
    pub const moduleData__initPlayerFacing__m_pad: usize = 24;
    pub const moduleData__initRelativeVelocity: usize = 16;
    pub const moduleData__initRelativeVelocity__type: usize = 16;
    pub const moduleData__initRelativeVelocity__m_flags: usize = 20;
    pub const moduleData__initRelativeVelocity__m_velocityType: usize = 24;
    pub const moduleData__initRelativeVelocity__m_useBoltInfo: usize = 28;
    pub const moduleData__initRotation: usize = 16;
    pub const moduleData__initRotation__type: usize = 16;
    pub const moduleData__initRotation__m_flags: usize = 20;
    pub const moduleData__initRotation__m_pad: usize = 24;
    pub const moduleData__initRotation__m_rotationAngle: usize = 32;
    pub const moduleData__initRotation__m_rotationAngle__min: usize = 32;
    pub const moduleData__initRotation__m_rotationAngle__max: usize = 36;
    pub const moduleData__initRotation__m_rotationRate: usize = 40;
    pub const moduleData__initRotation__m_rotationRate__min: usize = 40;
    pub const moduleData__initRotation__m_rotationRate__max: usize = 44;
    pub const moduleData__initRotation3D: usize = 16;
    pub const moduleData__initRotation3D__type: usize = 16;
    pub const moduleData__initRotation3D__m_flags: usize = 20;
    pub const moduleData__initRotation3D__m_pad: usize = 24;
    pub const moduleData__initRotation3D__m_rotationAngleMin: usize = 32;
    pub const moduleData__initRotation3D__m_rotationAngleMin__v: usize = 32;
    pub const moduleData__initRotation3D__m_rotationAngleMax: usize = 48;
    pub const moduleData__initRotation3D__m_rotationAngleMax__v: usize = 48;
    pub const moduleData__initRotation3D__m_rotationRateMin: usize = 64;
    pub const moduleData__initRotation3D__m_rotationRateMin__v: usize = 64;
    pub const moduleData__initRotation3D__m_rotationRateMax: usize = 80;
    pub const moduleData__initRotation3D__m_rotationRateMax__v: usize = 80;
    pub const moduleData__initRunner: usize = 16;
    pub const moduleData__initRunner__type: usize = 16;
    pub const moduleData__initRunner__m_flags: usize = 20;
    pub const moduleData__initRunner__m_pad: usize = 24;
    pub const moduleData__initRunner__m_linkedAssetList: usize = 32;
    pub const moduleData__initRunner__m_linkedAssetList__assetList: usize = 32;
    pub const moduleData__initRunner__m_linkedAssetList__numAssets: usize = 40;
    pub const moduleData__initSound: usize = 16;
    pub const moduleData__initSound__type: usize = 16;
    pub const moduleData__initSound__m_flags: usize = 20;
    pub const moduleData__initSound__m_pad: usize = 24;
    pub const moduleData__initSound__m_linkedAssetList: usize = 32;
    pub const moduleData__initSound__m_linkedAssetList__assetList: usize = 32;
    pub const moduleData__initSound__m_linkedAssetList__numAssets: usize = 40;
    pub const moduleData__initSpawn: usize = 16;
    pub const moduleData__initSpawn__type: usize = 16;
    pub const moduleData__initSpawn__m_flags: usize = 20;
    pub const moduleData__initSpawn__m_pad: usize = 24;
    pub const moduleData__initSpawn__m_curves: usize = 32;
    pub const moduleData__initSpawnShapeBox: usize = 16;
    pub const moduleData__initSpawnShapeBox__type: usize = 16;
    pub const moduleData__initSpawnShapeBox__m_flags: usize = 20;
    pub const moduleData__initSpawnShapeBox__m_axisFlags: usize = 24;
    pub const moduleData__initSpawnShapeBox__m_spawnFlags: usize = 25;
    pub const moduleData__initSpawnShapeBox__m_normalAxis: usize = 26;
    pub const moduleData__initSpawnShapeBox__m_spawnType: usize = 27;
    pub const moduleData__initSpawnShapeBox__m_volumeCubeRoot: usize = 28;
    pub const moduleData__initSpawnShapeBox__m_offset: usize = 32;
    pub const moduleData__initSpawnShapeBox__m_offset__v: usize = 32;
    pub const moduleData__initSpawnShapeBox__m_useBeamInfo: usize = 48;
    pub const moduleData__initSpawnShapeBox__m_pad: usize = 49;
    pub const moduleData__initSpawnShapeBox__m_dimensionsMin: usize = 64;
    pub const moduleData__initSpawnShapeBox__m_dimensionsMin__v: usize = 64;
    pub const moduleData__initSpawnShapeBox__m_dimensionsMax: usize = 80;
    pub const moduleData__initSpawnShapeBox__m_dimensionsMax__v: usize = 80;
    pub const moduleData__initSpawnShapeCylinder: usize = 16;
    pub const moduleData__initSpawnShapeCylinder__type: usize = 16;
    pub const moduleData__initSpawnShapeCylinder__m_flags: usize = 20;
    pub const moduleData__initSpawnShapeCylinder__m_axisFlags: usize = 24;
    pub const moduleData__initSpawnShapeCylinder__m_spawnFlags: usize = 25;
    pub const moduleData__initSpawnShapeCylinder__m_normalAxis: usize = 26;
    pub const moduleData__initSpawnShapeCylinder__m_spawnType: usize = 27;
    pub const moduleData__initSpawnShapeCylinder__m_volumeCubeRoot: usize = 28;
    pub const moduleData__initSpawnShapeCylinder__m_offset: usize = 32;
    pub const moduleData__initSpawnShapeCylinder__m_offset__v: usize = 32;
    pub const moduleData__initSpawnShapeCylinder__m_hasRotation: usize = 48;
    pub const moduleData__initSpawnShapeCylinder__m_rotateCalculatedOffset: usize = 49;
    pub const moduleData__initSpawnShapeCylinder__m_halfHeight: usize = 52;
    pub const moduleData__initSpawnShapeCylinder__m_radius: usize = 56;
    pub const moduleData__initSpawnShapeCylinder__m_radius__min: usize = 56;
    pub const moduleData__initSpawnShapeCylinder__m_radius__max: usize = 60;
    pub const moduleData__initSpawnShapeCylinder__unk: usize = 64;
    pub const moduleData__initSpawnShapeCylinder__unk__v: usize = 64;
    pub const moduleData__initSpawnShapeCylinder__m_directionQuat: usize = 80;
    pub const moduleData__initSpawnShapeCylinder__m_directionQuat__v: usize = 80;
    pub const moduleData__initSpawnShapeEllipsoid: usize = 16;
    pub const moduleData__initSpawnShapeEllipsoid__type: usize = 16;
    pub const moduleData__initSpawnShapeEllipsoid__m_flags: usize = 20;
    pub const moduleData__initSpawnShapeEllipsoid__m_axisFlags: usize = 24;
    pub const moduleData__initSpawnShapeEllipsoid__m_spawnFlags: usize = 25;
    pub const moduleData__initSpawnShapeEllipsoid__m_normalAxis: usize = 26;
    pub const moduleData__initSpawnShapeEllipsoid__m_spawnType: usize = 27;
    pub const moduleData__initSpawnShapeEllipsoid__m_volumeCubeRoot: usize = 28;
    pub const moduleData__initSpawnShapeEllipsoid__m_offset: usize = 32;
    pub const moduleData__initSpawnShapeEllipsoid__m_offset__v: usize = 32;
    pub const moduleData__initSpawnShapeEllipsoid__m_radiusMin: usize = 48;
    pub const moduleData__initSpawnShapeEllipsoid__m_radiusMin__v: usize = 48;
    pub const moduleData__initSpawnShapeEllipsoid__m_radiusMax: usize = 64;
    pub const moduleData__initSpawnShapeEllipsoid__m_radiusMax__v: usize = 64;
    pub const moduleData__initSpawnShapeEllipsoid__m_pad: usize = 80;
    pub const moduleData__initSpawnShapeMesh: usize = 16;
    pub const moduleData__initSpawnShapeMesh__type: usize = 16;
    pub const moduleData__initSpawnShapeMesh__m_flags: usize = 20;
    pub const moduleData__initSpawnShapeMesh__m_axisFlags: usize = 24;
    pub const moduleData__initSpawnShapeMesh__m_spawnFlags: usize = 25;
    pub const moduleData__initSpawnShapeMesh__m_normalAxis: usize = 26;
    pub const moduleData__initSpawnShapeMesh__m_spawnType: usize = 27;
    pub const moduleData__initSpawnShapeMesh__m_volumeCubeRoot: usize = 28;
    pub const moduleData__initSpawnShapeMesh__m_offset: usize = 32;
    pub const moduleData__initSpawnShapeMesh__m_offset__v: usize = 32;
    pub const moduleData__initSpawnShapeMesh__m_numMeshAssets: usize = 48;
    pub const moduleData__initSpawnShapeMesh__m_linkedAssetList: usize = 56;
    pub const moduleData__initSpawnShapeMesh__m_linkedAssetList__assetList: usize = 56;
    pub const moduleData__initSpawnShapeMesh__m_linkedAssetList__numAssets: usize = 64;
    pub const moduleData__initSpawnShapeMesh__m_meshAssetData: usize = 72;
    pub const moduleData__initSpawnShapeMesh__m_pad: usize = 80;
    pub const moduleData__initSpawnShapeSphere: usize = 16;
    pub const moduleData__initSpawnShapeSphere__type: usize = 16;
    pub const moduleData__initSpawnShapeSphere__m_flags: usize = 20;
    pub const moduleData__initSpawnShapeSphere__m_axisFlags: usize = 24;
    pub const moduleData__initSpawnShapeSphere__m_spawnFlags: usize = 25;
    pub const moduleData__initSpawnShapeSphere__m_normalAxis: usize = 26;
    pub const moduleData__initSpawnShapeSphere__m_spawnType: usize = 27;
    pub const moduleData__initSpawnShapeSphere__m_volumeCubeRoot: usize = 28;
    pub const moduleData__initSpawnShapeSphere__m_offset: usize = 32;
    pub const moduleData__initSpawnShapeSphere__m_offset__v: usize = 32;
    pub const moduleData__initSpawnShapeSphere__m_pad: usize = 48;
    pub const moduleData__initSpawnShapeSphere__m_radius: usize = 56;
    pub const moduleData__initSpawnShapeSphere__m_radius__min: usize = 56;
    pub const moduleData__initSpawnShapeSphere__m_radius__max: usize = 60;
    pub const moduleData__initSpawnShapeSphere__m_offset2: usize = 64;
    pub const moduleData__initSpawnShapeSphere__m_offset2__v: usize = 64;
    pub const moduleData__initTail: usize = 16;
    pub const moduleData__initTail__type: usize = 16;
    pub const moduleData__initTail__m_flags: usize = 20;
    pub const moduleData__initTail__m_averagePastVelocities: usize = 24;
    pub const moduleData__initTail__m_maxParentSpeed: usize = 26;
    pub const moduleData__initTail__m_tailLeading: usize = 28;
    pub const moduleData__initTail__m_scaleWithVelocity: usize = 29;
    pub const moduleData__initTail__m_rotateAroundPivot: usize = 30;
    pub const moduleData__initVectorField: usize = 16;
    pub const moduleData__initVectorField__type: usize = 16;
    pub const moduleData__initVectorField__m_flags: usize = 20;
    pub const moduleData__initVectorField__m_pad: usize = 24;
    pub const moduleData__initVectorField__m_linkedAssetList: usize = 32;
    pub const moduleData__initVectorField__m_linkedAssetList__assetList: usize = 32;
    pub const moduleData__initVectorField__m_linkedAssetList__numAssets: usize = 40;
    pub const moduleData__initVolumetric: usize = 16;
    pub const moduleData__initVolumetric__type: usize = 16;
    pub const moduleData__initVolumetric__m_flags: usize = 20;
    pub const moduleData__initVolumetric__m_density: usize = 24;
    pub const moduleData__initVolumetric__m_falloff: usize = 28;
    pub const moduleData__initVolumetric__m_noiseAmplitude: usize = 32;
    pub const moduleData__initVolumetric__m_noiseMode: usize = 36;
    pub const moduleData__initVolumetric__m_pad: usize = 40;
    pub const moduleData__attractor: usize = 16;
    pub const moduleData__attractor__type: usize = 16;
    pub const moduleData__attractor__m_flags: usize = 20;
    pub const moduleData__attractor__m_pad: usize = 24;
    pub const moduleData__attractor__m_killWhenNear: usize = 36;
    pub const moduleData__attractor__m_forceMag: usize = 40;
    pub const moduleData__attractor__m_nearDistanceSq: usize = 44;
    pub const moduleData__attractor__m_attractPoint: usize = 48;
    pub const moduleData__attractor__m_attractPoint__v: usize = 48;
    pub const moduleData__colorGraph: usize = 16;
    pub const moduleData__colorGraph__type: usize = 16;
    pub const moduleData__colorGraph__m_flags: usize = 20;
    pub const moduleData__colorGraph__firstCurve: usize = 24;
    pub const moduleData__colorGraph__m_pad: usize = 25;
    pub const moduleData__colorGraph__m_modulateColorByAlpha: usize = 28;
    pub const moduleData__colorGraph__m_curves: usize = 32;
    pub const moduleData__colorLerp: usize = 16;
    pub const moduleData__colorLerp__type: usize = 16;
    pub const moduleData__colorLerp__m_flags: usize = 20;
    pub const moduleData__colorLerp__m_pad: usize = 24;
    pub const moduleData__colorLerp__m_colorBegin: usize = 32;
    pub const moduleData__colorLerp__m_colorBegin__v: usize = 32;
    pub const moduleData__colorLerp__m_colorEnd: usize = 48;
    pub const moduleData__colorLerp__m_colorEnd__v: usize = 48;
    pub const moduleData__emissionGraph: usize = 16;
    pub const moduleData__emissionGraph__type: usize = 16;
    pub const moduleData__emissionGraph__m_flags: usize = 20;
    pub const moduleData__emissionGraph__m_pad: usize = 24;
    pub const moduleData__emissionGraph__m_curves: usize = 32;
    pub const moduleData__emissiveGraph: usize = 16;
    pub const moduleData__emissiveGraph__type: usize = 16;
    pub const moduleData__emissiveGraph__m_flags: usize = 20;
    pub const moduleData__emissiveGraph__firstCurve: usize = 24;
    pub const moduleData__emissiveGraph__m_pad: usize = 25;
    pub const moduleData__emissiveGraph__m_curves: usize = 32;
    pub const moduleData__force: usize = 16;
    pub const moduleData__force__type: usize = 16;
    pub const moduleData__force__m_flags: usize = 20;
    pub const moduleData__force__m_pad: usize = 24;
    pub const moduleData__force__m_forceMin: usize = 32;
    pub const moduleData__force__m_forceMin__v: usize = 32;
    pub const moduleData__force__m_forceMax: usize = 48;
    pub const moduleData__force__m_forceMax__v: usize = 48;
    pub const moduleData__gravity: usize = 16;
    pub const moduleData__gravity__type: usize = 16;
    pub const moduleData__gravity__m_flags: usize = 20;
    pub const moduleData__gravity__m_gravityPercentage: usize = 24;
    pub const moduleData__gravity__m_gravityPercentage__min: usize = 24;
    pub const moduleData__gravity__m_gravityPercentage__max: usize = 28;
    pub const moduleData__intensityGraph: usize = 16;
    pub const moduleData__intensityGraph__type: usize = 16;
    pub const moduleData__intensityGraph__m_flags: usize = 20;
    pub const moduleData__intensityGraph__firstCurve: usize = 24;
    pub const moduleData__intensityGraph__m_pad: usize = 25;
    pub const moduleData__intensityGraph__m_curves: usize = 32;
    pub const moduleData__parentVelocityGraph: usize = 16;
    pub const moduleData__parentVelocityGraph__type: usize = 16;
    pub const moduleData__parentVelocityGraph__m_flags: usize = 20;
    pub const moduleData__parentVelocityGraph__m_pad: usize = 24;
    pub const moduleData__parentVelocityGraph__m_curves: usize = 32;
    pub const moduleData__physicsLight: usize = 16;
    pub const moduleData__physicsLight__type: usize = 16;
    pub const moduleData__physicsLight__m_flags: usize = 20;
    pub const moduleData__physicsLight__m_ignoreEmitterOrientation: usize = 24;
    pub const moduleData__physicsLight__m_useSurfaceType: usize = 25;
    pub const moduleData__physicsLight__m_pad: usize = 26;
    pub const moduleData__physicsLight__m_linkedAssetList: usize = 32;
    pub const moduleData__physicsLight__m_linkedAssetList__assetList: usize = 32;
    pub const moduleData__physicsLight__m_linkedAssetList__numAssets: usize = 40;
    pub const moduleData__physicsRayCast: usize = 16;
    pub const moduleData__physicsRayCast__type: usize = 16;
    pub const moduleData__physicsRayCast__m_flags: usize = 20;
    pub const moduleData__physicsRayCast__m_bounce: usize = 24;
    pub const moduleData__physicsRayCast__m_bounce__min: usize = 24;
    pub const moduleData__physicsRayCast__m_bounce__max: usize = 28;
    pub const moduleData__physicsRayCast__m_bounds: usize = 32;
    pub const moduleData__physicsRayCast__m_bounds__midPoint: usize = 32;
    pub const moduleData__physicsRayCast__m_bounds__halfSize: usize = 44;
    pub const moduleData__physicsRayCast__m_useItemClip: usize = 56;
    pub const moduleData__physicsRayCast__m_useSurfaceType: usize = 57;
    pub const moduleData__physicsRayCast__m_collideWithWater: usize = 58;
    pub const moduleData__physicsRayCast__m_ignoreContentItem: usize = 59;
    pub const moduleData__physicsRayCast__m_pad: usize = 60;
    pub const moduleData__positionGraph: usize = 16;
    pub const moduleData__positionGraph__type: usize = 16;
    pub const moduleData__positionGraph__m_flags: usize = 20;
    pub const moduleData__positionGraph__m_pad: usize = 24;
    pub const moduleData__positionGraph__m_curves: usize = 32;
    pub const moduleData__rotationGraph: usize = 16;
    pub const moduleData__rotationGraph__type: usize = 16;
    pub const moduleData__rotationGraph__m_flags: usize = 20;
    pub const moduleData__rotationGraph__m_useRotationRate: usize = 24;
    pub const moduleData__rotationGraph__m_pad: usize = 28;
    pub const moduleData__rotationGraph__m_curves: usize = 32;
    pub const moduleData__rotationGraph3D: usize = 16;
    pub const moduleData__rotationGraph3D__type: usize = 16;
    pub const moduleData__rotationGraph3D__m_flags: usize = 20;
    pub const moduleData__rotationGraph3D__m_useRotationRate: usize = 24;
    pub const moduleData__rotationGraph3D__m_pad: usize = 28;
    pub const moduleData__rotationGraph3D__m_curves: usize = 32;
    pub const moduleData__sizeGraph: usize = 16;
    pub const moduleData__sizeGraph__type: usize = 16;
    pub const moduleData__sizeGraph__m_flags: usize = 20;
    pub const moduleData__sizeGraph__firstCurve: usize = 24;
    pub const moduleData__sizeGraph__m_pad: usize = 25;
    pub const moduleData__sizeGraph__m_curves: usize = 32;
    pub const moduleData__sizeGraph__m_sizeBegin: usize = 128;
    pub const moduleData__sizeGraph__m_sizeBegin__v: usize = 128;
    pub const moduleData__sizeGraph__m_sizeEnd: usize = 144;
    pub const moduleData__sizeGraph__m_sizeEnd__v: usize = 144;
    pub const moduleData__sizeLerp: usize = 16;
    pub const moduleData__sizeLerp__type: usize = 16;
    pub const moduleData__sizeLerp__m_flags: usize = 20;
    pub const moduleData__sizeLerp__m_pad: usize = 24;
    pub const moduleData__sizeLerp__m_sizeBegin: usize = 32;
    pub const moduleData__sizeLerp__m_sizeBegin__v: usize = 32;
    pub const moduleData__sizeLerp__m_sizeEnd: usize = 48;
    pub const moduleData__sizeLerp__m_sizeEnd__v: usize = 48;
    pub const moduleData__velocityGraph: usize = 16;
    pub const moduleData__velocityGraph__type: usize = 16;
    pub const moduleData__velocityGraph__m_flags: usize = 20;
    pub const moduleData__velocityGraph__m_pad: usize = 24;
    pub const moduleData__velocityGraph__m_curves: usize = 32;
    pub const moduleData__velocityGraph__m_velocityBegin: usize = 128;
    pub const moduleData__velocityGraph__m_velocityBegin__v: usize = 128;
    pub const moduleData__velocityGraph__m_velocityEnd: usize = 144;
    pub const moduleData__velocityGraph__m_velocityEnd__v: usize = 144;
    pub const moduleData__testAge: usize = 16;
    pub const moduleData__testAge__type: usize = 16;
    pub const moduleData__testAge__m_flags: usize = 20;
    pub const moduleData__testAge__m_moduleIndex: usize = 24;
    pub const moduleData__testAge__m_useOrientationOptions: usize = 26;
    pub const moduleData__testAge__m_useScaleOptions: usize = 27;
    pub const moduleData__testAge__m_useVelocityOptions: usize = 28;
    pub const moduleData__testAge__m_eventHandlerData: usize = 32;
    pub const moduleData__testAge__m_eventHandlerData__m_nextState: usize = 32;
    pub const moduleData__testAge__m_eventHandlerData__m_linkedAssetList: usize = 40;
    pub const moduleData__testAge__m_eventHandlerData__m_linkedAssetList__assetList: usize = 40;
    pub const moduleData__testAge__m_eventHandlerData__m_linkedAssetList__numAssets: usize = 48;
    pub const moduleData__testAge__m_eventHandlerData__m_kill: usize = 56;
    pub const moduleData__testAge__m_eventHandlerData__m_pad: usize = 60;
    pub const moduleData__testAge__m_age: usize = 64;
    pub const moduleData__testBirth: usize = 16;
    pub const moduleData__testBirth__type: usize = 16;
    pub const moduleData__testBirth__m_flags: usize = 20;
    pub const moduleData__testBirth__m_moduleIndex: usize = 24;
    pub const moduleData__testBirth__m_useOrientationOptions: usize = 26;
    pub const moduleData__testBirth__m_useScaleOptions: usize = 27;
    pub const moduleData__testBirth__m_useVelocityOptions: usize = 28;
    pub const moduleData__testBirth__m_eventHandlerData: usize = 32;
    pub const moduleData__testBirth__m_eventHandlerData__m_nextState: usize = 32;
    pub const moduleData__testBirth__m_eventHandlerData__m_linkedAssetList: usize = 40;
    pub const moduleData__testBirth__m_eventHandlerData__m_linkedAssetList__assetList: usize = 40;
    pub const moduleData__testBirth__m_eventHandlerData__m_linkedAssetList__numAssets: usize = 48;
    pub const moduleData__testBirth__m_eventHandlerData__m_kill: usize = 56;
    pub const moduleData__testBirth__m_eventHandlerData__m_pad: usize = 60;
    pub const moduleData__testDeath: usize = 16;
    pub const moduleData__testDeath__type: usize = 16;
    pub const moduleData__testDeath__m_flags: usize = 20;
    pub const moduleData__testDeath__m_moduleIndex: usize = 24;
    pub const moduleData__testDeath__m_useOrientationOptions: usize = 26;
    pub const moduleData__testDeath__m_useScaleOptions: usize = 27;
    pub const moduleData__testDeath__m_useVelocityOptions: usize = 28;
    pub const moduleData__testDeath__m_eventHandlerData: usize = 32;
    pub const moduleData__testDeath__m_eventHandlerData__m_nextState: usize = 32;
    pub const moduleData__testDeath__m_eventHandlerData__m_linkedAssetList: usize = 40;
    pub const moduleData__testDeath__m_eventHandlerData__m_linkedAssetList__assetList: usize = 40;
    pub const moduleData__testDeath__m_eventHandlerData__m_linkedAssetList__numAssets: usize = 48;
    pub const moduleData__testDeath__m_eventHandlerData__m_kill: usize = 56;
    pub const moduleData__testDeath__m_eventHandlerData__m_pad: usize = 60;
    pub const moduleData__testGravity: usize = 16;
    pub const moduleData__testGravity__type: usize = 16;
    pub const moduleData__testGravity__m_flags: usize = 20;
    pub const moduleData__testGravity__m_moduleIndex: usize = 24;
    pub const moduleData__testGravity__m_useOrientationOptions: usize = 26;
    pub const moduleData__testGravity__m_useScaleOptions: usize = 27;
    pub const moduleData__testGravity__m_useVelocityOptions: usize = 28;
    pub const moduleData__testGravity__m_eventHandlerData: usize = 32;
    pub const moduleData__testGravity__m_eventHandlerData__m_nextState: usize = 32;
    pub const moduleData__testGravity__m_eventHandlerData__m_linkedAssetList: usize = 40;
    pub const moduleData__testGravity__m_eventHandlerData__m_linkedAssetList__assetList: usize = 40;
    pub const moduleData__testGravity__m_eventHandlerData__m_linkedAssetList__numAssets: usize = 48;
    pub const moduleData__testGravity__m_eventHandlerData__m_kill: usize = 56;
    pub const moduleData__testGravity__m_eventHandlerData__m_pad: usize = 60;
    pub const moduleData__testImpact: usize = 16;
    pub const moduleData__testImpact__type: usize = 16;
    pub const moduleData__testImpact__m_flags: usize = 20;
    pub const moduleData__testImpact__m_moduleIndex: usize = 24;
    pub const moduleData__testImpact__m_useOrientationOptions: usize = 26;
    pub const moduleData__testImpact__m_useScaleOptions: usize = 27;
    pub const moduleData__testImpact__m_useVelocityOptions: usize = 28;
    pub const moduleData__testImpact__m_eventHandlerData: usize = 32;
    pub const moduleData__testImpact__m_eventHandlerData__m_nextState: usize = 32;
    pub const moduleData__testImpact__m_eventHandlerData__m_linkedAssetList: usize = 40;
    pub const moduleData__testImpact__m_eventHandlerData__m_linkedAssetList__assetList: usize = 40;
    pub const moduleData__testImpact__m_eventHandlerData__m_linkedAssetList__numAssets: usize = 48;
    pub const moduleData__testImpact__m_eventHandlerData__m_kill: usize = 56;
    pub const moduleData__testImpact__m_eventHandlerData__m_pad: usize = 60;
    pub const moduleData__testPos: usize = 16;
    pub const moduleData__testPos__type: usize = 16;
    pub const moduleData__testPos__m_flags: usize = 20;
    pub const moduleData__testPos__m_moduleIndex: usize = 24;
    pub const moduleData__testPos__m_useOrientationOptions: usize = 26;
    pub const moduleData__testPos__m_useScaleOptions: usize = 27;
    pub const moduleData__testPos__m_useVelocityOptions: usize = 28;
    pub const moduleData__testPos__m_eventHandlerData: usize = 32;
    pub const moduleData__testPos__m_eventHandlerData__m_nextState: usize = 32;
    pub const moduleData__testPos__m_eventHandlerData__m_linkedAssetList: usize = 40;
    pub const moduleData__testPos__m_eventHandlerData__m_linkedAssetList__assetList: usize = 40;
    pub const moduleData__testPos__m_eventHandlerData__m_linkedAssetList__numAssets: usize = 48;
    pub const moduleData__testPos__m_eventHandlerData__m_kill: usize = 56;
    pub const moduleData__testPos__m_eventHandlerData__m_pad: usize = 60;
}
pub mod ParticleModuleEmissionGraph {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_pad: usize = 8;
    pub const m_curves: usize = 16;
}
pub mod ParticleModuleEmissiveGraph {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const firstCurve: usize = 8;
    pub const m_pad: usize = 9;
    pub const m_curves: usize = 16;
}
pub mod ParticleModuleForce {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_pad: usize = 8;
    pub const m_forceMin: usize = 16;
    pub const m_forceMin__v: usize = 16;
    pub const m_forceMax: usize = 32;
    pub const m_forceMax__v: usize = 32;
}
pub mod ParticleModuleGravity {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_gravityPercentage: usize = 8;
    pub const m_gravityPercentage__min: usize = 8;
    pub const m_gravityPercentage__max: usize = 12;
}
pub mod ParticleModuleGroupDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const moduleDefs: usize = 0;
    pub const numModules: usize = 8;
    pub const disabled: usize = 12;
}
pub mod ParticleModuleInitAtlas {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_playRate: usize = 8;
    pub const m_startFrame: usize = 12;
    pub const m_loopCount: usize = 16;
}
pub mod ParticleModuleInitAttributes {
    pub const SIZE: usize = 112;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_useNonUniformInterpolationForColor: usize = 8;
    pub const m_useNonUniformInterpolationForSize: usize = 9;
    pub const m_pad: usize = 10;
    pub const m_sizeMin: usize = 16;
    pub const m_sizeMin__v: usize = 16;
    pub const m_sizeMax: usize = 32;
    pub const m_sizeMax__v: usize = 32;
    pub const m_colorMin: usize = 48;
    pub const m_colorMin__v: usize = 48;
    pub const m_colorMax: usize = 64;
    pub const m_colorMax__v: usize = 64;
    pub const m_velocityMin: usize = 80;
    pub const m_velocityMin__v: usize = 80;
    pub const m_velocityMax: usize = 96;
    pub const m_velocityMax__v: usize = 96;
}
pub mod ParticleModuleInitBeam {
    pub const SIZE: usize = 128;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_beamFlags: usize = 8;
    pub const m_splitDistance: usize = 12;
    pub const m_startPos: usize = 16;
    pub const m_startPos__v: usize = 16;
    pub const m_endPos: usize = 32;
    pub const m_endPos__v: usize = 32;
    pub const m_offset: usize = 48;
    pub const m_offset__v: usize = 48;
    pub const m_curvePoint1: usize = 64;
    pub const m_curvePoint1__v: usize = 64;
    pub const m_curvePoint2: usize = 80;
    pub const m_curvePoint2__v: usize = 80;
    pub const m_linkedAssetList: usize = 96;
    pub const m_linkedAssetList__assetList: usize = 96;
    pub const m_linkedAssetList__numAssets: usize = 104;
    pub const m_scrollRateUV: usize = 112;
    pub const m_scrollRateSpiralGraph: usize = 116;
    pub const m_spiralRepeatDistance: usize = 120;
}
pub mod ParticleModuleInitCameraOffset {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_pad: usize = 8;
}
pub mod ParticleModuleInitCloud {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_pad: usize = 8;
    pub const curves: usize = 16;
}
pub mod ParticleModuleInitDecal {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_fadeInTime: usize = 8;
    pub const m_fadeOutTime: usize = 10;
    pub const m_stoppableFadeOutTime: usize = 12;
    pub const m_lerpWaitTime: usize = 14;
    pub const m_lerpColor: usize = 16;
    pub const m_lerpColor__v: usize = 16;
    pub const m_linkedAssetList: usize = 32;
    pub const m_linkedAssetList__assetList: usize = 32;
    pub const m_linkedAssetList__numAssets: usize = 40;
}
pub mod ParticleModuleInitFlare {
    pub const SIZE: usize = 144;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_pad: usize = 8;
    pub const m_position: usize = 12;
    pub const m_direction: usize = 16;
    pub const m_direction__v: usize = 16;
    pub const m_angularRotCount: usize = 32;
    pub const m_flareFlags: usize = 36;
    pub const m_depthScaleRange: usize = 40;
    pub const m_depthScaleRange__min: usize = 40;
    pub const m_depthScaleRange__max: usize = 44;
    pub const m_depthScaleValue: usize = 48;
    pub const m_depthScaleValue__min: usize = 48;
    pub const m_depthScaleValue__max: usize = 52;
    pub const m_radialRot: usize = 56;
    pub const m_radialRot__min: usize = 56;
    pub const m_radialRot__max: usize = 60;
    pub const m_radialScaleX: usize = 64;
    pub const m_radialScaleX__min: usize = 64;
    pub const m_radialScaleX__max: usize = 68;
    pub const m_radialScaleY: usize = 72;
    pub const m_radialScaleY__min: usize = 72;
    pub const m_radialScaleY__max: usize = 76;
    pub const m_curves: usize = 80;
}
pub mod ParticleModuleInitGeoTrail {
    pub const SIZE: usize = 64;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_numPointsMax: usize = 8;
    pub const m_splitDistance: usize = 12;
    pub const m_splitAngle: usize = 16;
    pub const m_centerOffset: usize = 20;
    pub const m_numSheets: usize = 24;
    pub const m_fadeInDistance: usize = 28;
    pub const m_fadeOutDistance: usize = 32;
    pub const m_tileDistance: usize = 36;
    pub const m_tileOffset: usize = 40;
    pub const m_tileOffset__min: usize = 40;
    pub const m_tileOffset__max: usize = 44;
    pub const m_scrollTime: usize = 48;
    pub const m_useLocalVelocity: usize = 52;
    pub const m_useVerticalTexture: usize = 53;
    pub const m_cameraFacing: usize = 54;
    pub const m_fixLeadingEdge: usize = 55;
    pub const m_clampUVs: usize = 56;
    pub const m_pad: usize = 57;
    pub const m_pad2: usize = 60;
}
pub mod ParticleModuleInitLightOmni {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_linkedAssetList: usize = 8;
    pub const m_linkedAssetList__assetList: usize = 8;
    pub const m_linkedAssetList__numAssets: usize = 16;
    pub const m_tonemappingScaleFactor: usize = 24;
    pub const m_intensityIR: usize = 28;
    pub const m_disableVolumetric: usize = 32;
    pub const m_exponent: usize = 33;
    pub const __pad0: usize = 34;
    pub const m_shadowSoftness: usize = 36;
    pub const m_shadowBias: usize = 40;
    pub const m_shadowArea: usize = 44;
}
pub mod ParticleModuleInitLightSpot {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_fovOuter: usize = 8;
    pub const m_fovInner: usize = 12;
    pub const m_bulbRadius: usize = 16;
    pub const m_bulbLength: usize = 20;
    pub const m_brightness: usize = 24;
    pub const unk1: usize = 28;
    pub const m_intensityUV: usize = 32;
    pub const m_intensityIR: usize = 36;
    pub const m_shadowNearPlane: usize = 40;
    pub const m_toneMappingScaleFactor: usize = 44;
    pub const m_shadowSoftness: usize = 48;
    pub const m_shadowBias: usize = 52;
    pub const m_shadowArea: usize = 56;
    pub const m_disableVolumetric: usize = 60;
    pub const m_disableShadowMap: usize = 61;
    pub const m_disableDynamicShadows: usize = 62;
    pub const m_exponent: usize = 63;
    pub const m_linkedAssetList: usize = 64;
    pub const m_linkedAssetList__assetList: usize = 64;
    pub const m_linkedAssetList__numAssets: usize = 72;
}
pub mod ParticleModuleInitMaterial {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_pad: usize = 8;
    pub const m_linkedAssetList: usize = 16;
    pub const m_linkedAssetList__assetList: usize = 16;
    pub const m_linkedAssetList__numAssets: usize = 24;
}
pub mod ParticleModuleInitMirrorTexture {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_mirrorHorizontal: usize = 8;
    pub const m_mirrorVertical: usize = 12;
}
pub mod ParticleModuleInitModel {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_usePhysics: usize = 8;
    pub const m_motionBlurHQ: usize = 9;
    pub const m_linkedAssetList: usize = 16;
    pub const m_linkedAssetList__assetList: usize = 16;
    pub const m_linkedAssetList__numAssets: usize = 24;
}
pub mod ParticleModuleInitOcclusionQuery {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_fadeInTime: usize = 8;
    pub const m_fadeOutTime: usize = 12;
    pub const m_scale: usize = 16;
    pub const m_scale__min: usize = 16;
    pub const m_scale__max: usize = 20;
    pub const m_worldSize: usize = 24;
}
pub mod ParticleModuleInitOrientedSprite {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_pad: usize = 8;
    pub const m_orientationQuat: usize = 16;
    pub const m_orientationQuat__v: usize = 16;
}
pub mod ParticleModuleInitParticleSim {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_linkedAssetList: usize = 8;
    pub const m_linkedAssetList__assetList: usize = 8;
    pub const m_linkedAssetList__numAssets: usize = 16;
    pub const m_scaleFactor: usize = 24;
    pub const m_scaleFactor__min: usize = 24;
    pub const m_scaleFactor__max: usize = 28;
    pub const m_holdLastFrame: usize = 32;
    pub const m_pad: usize = 36;
}
pub mod ParticleModuleInitPlayerFacing {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_pad: usize = 8;
}
pub mod ParticleModuleInitRelativeVelocity {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_velocityType: usize = 8;
    pub const m_useBoltInfo: usize = 12;
}
pub mod ParticleModuleInitRotation {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_pad: usize = 8;
    pub const m_rotationAngle: usize = 16;
    pub const m_rotationAngle__min: usize = 16;
    pub const m_rotationAngle__max: usize = 20;
    pub const m_rotationRate: usize = 24;
    pub const m_rotationRate__min: usize = 24;
    pub const m_rotationRate__max: usize = 28;
}
pub mod ParticleModuleInitRotation3D {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_pad: usize = 8;
    pub const m_rotationAngleMin: usize = 16;
    pub const m_rotationAngleMin__v: usize = 16;
    pub const m_rotationAngleMax: usize = 32;
    pub const m_rotationAngleMax__v: usize = 32;
    pub const m_rotationRateMin: usize = 48;
    pub const m_rotationRateMin__v: usize = 48;
    pub const m_rotationRateMax: usize = 64;
    pub const m_rotationRateMax__v: usize = 64;
}
pub mod ParticleModuleInitRunner {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_pad: usize = 8;
    pub const m_linkedAssetList: usize = 16;
    pub const m_linkedAssetList__assetList: usize = 16;
    pub const m_linkedAssetList__numAssets: usize = 24;
}
pub mod ParticleModuleInitSound {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_pad: usize = 8;
    pub const m_linkedAssetList: usize = 16;
    pub const m_linkedAssetList__assetList: usize = 16;
    pub const m_linkedAssetList__numAssets: usize = 24;
}
pub mod ParticleModuleInitSpawn {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_pad: usize = 8;
    pub const m_curves: usize = 16;
}
pub mod ParticleModuleInitSpawnShape {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_axisFlags: usize = 8;
    pub const m_spawnFlags: usize = 9;
    pub const m_normalAxis: usize = 10;
    pub const m_spawnType: usize = 11;
    pub const m_volumeCubeRoot: usize = 12;
    pub const m_offset: usize = 16;
    pub const m_offset__v: usize = 16;
}
pub mod ParticleModuleInitSpawnShapeBox {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_axisFlags: usize = 8;
    pub const m_spawnFlags: usize = 9;
    pub const m_normalAxis: usize = 10;
    pub const m_spawnType: usize = 11;
    pub const m_volumeCubeRoot: usize = 12;
    pub const m_offset: usize = 16;
    pub const m_offset__v: usize = 16;
    pub const m_useBeamInfo: usize = 32;
    pub const m_pad: usize = 33;
    pub const m_dimensionsMin: usize = 48;
    pub const m_dimensionsMin__v: usize = 48;
    pub const m_dimensionsMax: usize = 64;
    pub const m_dimensionsMax__v: usize = 64;
}
pub mod ParticleModuleInitSpawnShapeCylinder {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_axisFlags: usize = 8;
    pub const m_spawnFlags: usize = 9;
    pub const m_normalAxis: usize = 10;
    pub const m_spawnType: usize = 11;
    pub const m_volumeCubeRoot: usize = 12;
    pub const m_offset: usize = 16;
    pub const m_offset__v: usize = 16;
    pub const m_hasRotation: usize = 32;
    pub const m_rotateCalculatedOffset: usize = 33;
    pub const m_halfHeight: usize = 36;
    pub const m_radius: usize = 40;
    pub const m_radius__min: usize = 40;
    pub const m_radius__max: usize = 44;
    pub const unk: usize = 48;
    pub const unk__v: usize = 48;
    pub const m_directionQuat: usize = 64;
    pub const m_directionQuat__v: usize = 64;
}
pub mod ParticleModuleInitSpawnShapeEllipsoid {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_axisFlags: usize = 8;
    pub const m_spawnFlags: usize = 9;
    pub const m_normalAxis: usize = 10;
    pub const m_spawnType: usize = 11;
    pub const m_volumeCubeRoot: usize = 12;
    pub const m_offset: usize = 16;
    pub const m_offset__v: usize = 16;
    pub const m_radiusMin: usize = 32;
    pub const m_radiusMin__v: usize = 32;
    pub const m_radiusMax: usize = 48;
    pub const m_radiusMax__v: usize = 48;
    pub const m_pad: usize = 64;
}
pub mod ParticleModuleInitSpawnShapeMesh {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_axisFlags: usize = 8;
    pub const m_spawnFlags: usize = 9;
    pub const m_normalAxis: usize = 10;
    pub const m_spawnType: usize = 11;
    pub const m_volumeCubeRoot: usize = 12;
    pub const m_offset: usize = 16;
    pub const m_offset__v: usize = 16;
    pub const m_numMeshAssets: usize = 32;
    pub const m_linkedAssetList: usize = 40;
    pub const m_linkedAssetList__assetList: usize = 40;
    pub const m_linkedAssetList__numAssets: usize = 48;
    pub const m_meshAssetData: usize = 56;
    pub const m_pad: usize = 64;
}
pub mod ParticleModuleInitSpawnShapeSphere {
    pub const SIZE: usize = 64;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_axisFlags: usize = 8;
    pub const m_spawnFlags: usize = 9;
    pub const m_normalAxis: usize = 10;
    pub const m_spawnType: usize = 11;
    pub const m_volumeCubeRoot: usize = 12;
    pub const m_offset: usize = 16;
    pub const m_offset__v: usize = 16;
    pub const m_pad: usize = 32;
    pub const m_radius: usize = 40;
    pub const m_radius__min: usize = 40;
    pub const m_radius__max: usize = 44;
    pub const m_offset2: usize = 48;
    pub const m_offset2__v: usize = 48;
}
pub mod ParticleModuleInitTail {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_averagePastVelocities: usize = 8;
    pub const m_maxParentSpeed: usize = 10;
    pub const m_tailLeading: usize = 12;
    pub const m_scaleWithVelocity: usize = 13;
    pub const m_rotateAroundPivot: usize = 14;
}
pub mod ParticleModuleInitVectorField {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_pad: usize = 8;
    pub const m_linkedAssetList: usize = 16;
    pub const m_linkedAssetList__assetList: usize = 16;
    pub const m_linkedAssetList__numAssets: usize = 24;
}
pub mod ParticleModuleInitVolumetric {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_density: usize = 8;
    pub const m_falloff: usize = 12;
    pub const m_noiseAmplitude: usize = 16;
    pub const m_noiseMode: usize = 20;
    pub const m_pad: usize = 24;
}
pub mod ParticleModuleIntensityGraph {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const firstCurve: usize = 8;
    pub const m_pad: usize = 9;
    pub const m_curves: usize = 16;
}
pub mod ParticleModuleParentVelocityGraph {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_pad: usize = 8;
    pub const m_curves: usize = 16;
}
pub mod ParticleModulePhysicsLight {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_ignoreEmitterOrientation: usize = 8;
    pub const m_useSurfaceType: usize = 9;
    pub const m_pad: usize = 10;
    pub const m_linkedAssetList: usize = 16;
    pub const m_linkedAssetList__assetList: usize = 16;
    pub const m_linkedAssetList__numAssets: usize = 24;
}
pub mod ParticleModulePhysicsRayCast {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_bounce: usize = 8;
    pub const m_bounce__min: usize = 8;
    pub const m_bounce__max: usize = 12;
    pub const m_bounds: usize = 16;
    pub const m_bounds__midPoint: usize = 16;
    pub const m_bounds__halfSize: usize = 28;
    pub const m_useItemClip: usize = 40;
    pub const m_useSurfaceType: usize = 41;
    pub const m_collideWithWater: usize = 42;
    pub const m_ignoreContentItem: usize = 43;
    pub const m_pad: usize = 44;
}
pub mod ParticleModulePositionGraph {
    pub const SIZE: usize = 112;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_pad: usize = 8;
    pub const m_curves: usize = 16;
}
pub mod ParticleModuleRotationGraph {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_useRotationRate: usize = 8;
    pub const m_pad: usize = 12;
    pub const m_curves: usize = 16;
}
pub mod ParticleModuleRotationGraph3D {
    pub const SIZE: usize = 112;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_useRotationRate: usize = 8;
    pub const m_pad: usize = 12;
    pub const m_curves: usize = 16;
}
pub mod ParticleModuleSizeGraph {
    pub const SIZE: usize = 144;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const firstCurve: usize = 8;
    pub const m_pad: usize = 9;
    pub const m_curves: usize = 16;
    pub const m_sizeBegin: usize = 112;
    pub const m_sizeBegin__v: usize = 112;
    pub const m_sizeEnd: usize = 128;
    pub const m_sizeEnd__v: usize = 128;
}
pub mod ParticleModuleSizeLerp {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_pad: usize = 8;
    pub const m_sizeBegin: usize = 16;
    pub const m_sizeBegin__v: usize = 16;
    pub const m_sizeEnd: usize = 32;
    pub const m_sizeEnd__v: usize = 32;
}
pub mod ParticleModuleTest {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_moduleIndex: usize = 8;
    pub const m_useOrientationOptions: usize = 10;
    pub const m_useScaleOptions: usize = 11;
    pub const m_useVelocityOptions: usize = 12;
    pub const m_eventHandlerData: usize = 16;
    pub const m_eventHandlerData__m_nextState: usize = 16;
    pub const m_eventHandlerData__m_linkedAssetList: usize = 24;
    pub const m_eventHandlerData__m_linkedAssetList__assetList: usize = 24;
    pub const m_eventHandlerData__m_linkedAssetList__numAssets: usize = 32;
    pub const m_eventHandlerData__m_kill: usize = 40;
    pub const m_eventHandlerData__m_pad: usize = 44;
}
pub mod ParticleModuleTestAge {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_moduleIndex: usize = 8;
    pub const m_useOrientationOptions: usize = 10;
    pub const m_useScaleOptions: usize = 11;
    pub const m_useVelocityOptions: usize = 12;
    pub const m_eventHandlerData: usize = 16;
    pub const m_eventHandlerData__m_nextState: usize = 16;
    pub const m_eventHandlerData__m_linkedAssetList: usize = 24;
    pub const m_eventHandlerData__m_linkedAssetList__assetList: usize = 24;
    pub const m_eventHandlerData__m_linkedAssetList__numAssets: usize = 32;
    pub const m_eventHandlerData__m_kill: usize = 40;
    pub const m_eventHandlerData__m_pad: usize = 44;
    pub const m_age: usize = 48;
}
pub mod ParticleModuleTestBirth {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_moduleIndex: usize = 8;
    pub const m_useOrientationOptions: usize = 10;
    pub const m_useScaleOptions: usize = 11;
    pub const m_useVelocityOptions: usize = 12;
    pub const m_eventHandlerData: usize = 16;
    pub const m_eventHandlerData__m_nextState: usize = 16;
    pub const m_eventHandlerData__m_linkedAssetList: usize = 24;
    pub const m_eventHandlerData__m_linkedAssetList__assetList: usize = 24;
    pub const m_eventHandlerData__m_linkedAssetList__numAssets: usize = 32;
    pub const m_eventHandlerData__m_kill: usize = 40;
    pub const m_eventHandlerData__m_pad: usize = 44;
}
pub mod ParticleModuleTestDeath {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_moduleIndex: usize = 8;
    pub const m_useOrientationOptions: usize = 10;
    pub const m_useScaleOptions: usize = 11;
    pub const m_useVelocityOptions: usize = 12;
    pub const m_eventHandlerData: usize = 16;
    pub const m_eventHandlerData__m_nextState: usize = 16;
    pub const m_eventHandlerData__m_linkedAssetList: usize = 24;
    pub const m_eventHandlerData__m_linkedAssetList__assetList: usize = 24;
    pub const m_eventHandlerData__m_linkedAssetList__numAssets: usize = 32;
    pub const m_eventHandlerData__m_kill: usize = 40;
    pub const m_eventHandlerData__m_pad: usize = 44;
}
pub mod ParticleModuleTestEventHandlerData {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const m_nextState: usize = 0;
    pub const m_linkedAssetList: usize = 8;
    pub const m_linkedAssetList__assetList: usize = 8;
    pub const m_linkedAssetList__numAssets: usize = 16;
    pub const m_kill: usize = 24;
    pub const m_pad: usize = 28;
}
pub mod ParticleModuleTestGravity {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_moduleIndex: usize = 8;
    pub const m_useOrientationOptions: usize = 10;
    pub const m_useScaleOptions: usize = 11;
    pub const m_useVelocityOptions: usize = 12;
    pub const m_eventHandlerData: usize = 16;
    pub const m_eventHandlerData__m_nextState: usize = 16;
    pub const m_eventHandlerData__m_linkedAssetList: usize = 24;
    pub const m_eventHandlerData__m_linkedAssetList__assetList: usize = 24;
    pub const m_eventHandlerData__m_linkedAssetList__numAssets: usize = 32;
    pub const m_eventHandlerData__m_kill: usize = 40;
    pub const m_eventHandlerData__m_pad: usize = 44;
}
pub mod ParticleModuleTestImpact {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_moduleIndex: usize = 8;
    pub const m_useOrientationOptions: usize = 10;
    pub const m_useScaleOptions: usize = 11;
    pub const m_useVelocityOptions: usize = 12;
    pub const m_eventHandlerData: usize = 16;
    pub const m_eventHandlerData__m_nextState: usize = 16;
    pub const m_eventHandlerData__m_linkedAssetList: usize = 24;
    pub const m_eventHandlerData__m_linkedAssetList__assetList: usize = 24;
    pub const m_eventHandlerData__m_linkedAssetList__numAssets: usize = 32;
    pub const m_eventHandlerData__m_kill: usize = 40;
    pub const m_eventHandlerData__m_pad: usize = 44;
}
pub mod ParticleModuleTestPos {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_moduleIndex: usize = 8;
    pub const m_useOrientationOptions: usize = 10;
    pub const m_useScaleOptions: usize = 11;
    pub const m_useVelocityOptions: usize = 12;
    pub const m_eventHandlerData: usize = 16;
    pub const m_eventHandlerData__m_nextState: usize = 16;
    pub const m_eventHandlerData__m_linkedAssetList: usize = 24;
    pub const m_eventHandlerData__m_linkedAssetList__assetList: usize = 24;
    pub const m_eventHandlerData__m_linkedAssetList__numAssets: usize = 32;
    pub const m_eventHandlerData__m_kill: usize = 40;
    pub const m_eventHandlerData__m_pad: usize = 44;
}
pub mod ParticleModuleTypeDef {
    pub const SIZE: usize = 144;
    pub const ALIGN: usize = 8;
    pub const moduleBase: usize = 0;
    pub const moduleBase__type: usize = 0;
    pub const moduleBase__m_flags: usize = 4;
    pub const initAtlas: usize = 0;
    pub const initAtlas__type: usize = 0;
    pub const initAtlas__m_flags: usize = 4;
    pub const initAtlas__m_playRate: usize = 8;
    pub const initAtlas__m_startFrame: usize = 12;
    pub const initAtlas__m_loopCount: usize = 16;
    pub const initAttributes: usize = 0;
    pub const initAttributes__type: usize = 0;
    pub const initAttributes__m_flags: usize = 4;
    pub const initAttributes__m_useNonUniformInterpolationForColor: usize = 8;
    pub const initAttributes__m_useNonUniformInterpolationForSize: usize = 9;
    pub const initAttributes__m_pad: usize = 10;
    pub const initAttributes__m_sizeMin: usize = 16;
    pub const initAttributes__m_sizeMin__v: usize = 16;
    pub const initAttributes__m_sizeMax: usize = 32;
    pub const initAttributes__m_sizeMax__v: usize = 32;
    pub const initAttributes__m_colorMin: usize = 48;
    pub const initAttributes__m_colorMin__v: usize = 48;
    pub const initAttributes__m_colorMax: usize = 64;
    pub const initAttributes__m_colorMax__v: usize = 64;
    pub const initAttributes__m_velocityMin: usize = 80;
    pub const initAttributes__m_velocityMin__v: usize = 80;
    pub const initAttributes__m_velocityMax: usize = 96;
    pub const initAttributes__m_velocityMax__v: usize = 96;
    pub const initBeam: usize = 0;
    pub const initBeam__type: usize = 0;
    pub const initBeam__m_flags: usize = 4;
    pub const initBeam__m_beamFlags: usize = 8;
    pub const initBeam__m_splitDistance: usize = 12;
    pub const initBeam__m_startPos: usize = 16;
    pub const initBeam__m_startPos__v: usize = 16;
    pub const initBeam__m_endPos: usize = 32;
    pub const initBeam__m_endPos__v: usize = 32;
    pub const initBeam__m_offset: usize = 48;
    pub const initBeam__m_offset__v: usize = 48;
    pub const initBeam__m_curvePoint1: usize = 64;
    pub const initBeam__m_curvePoint1__v: usize = 64;
    pub const initBeam__m_curvePoint2: usize = 80;
    pub const initBeam__m_curvePoint2__v: usize = 80;
    pub const initBeam__m_linkedAssetList: usize = 96;
    pub const initBeam__m_linkedAssetList__assetList: usize = 96;
    pub const initBeam__m_linkedAssetList__numAssets: usize = 104;
    pub const initBeam__m_scrollRateUV: usize = 112;
    pub const initBeam__m_scrollRateSpiralGraph: usize = 116;
    pub const initBeam__m_spiralRepeatDistance: usize = 120;
    pub const initCameraOffset: usize = 0;
    pub const initCameraOffset__type: usize = 0;
    pub const initCameraOffset__m_flags: usize = 4;
    pub const initCameraOffset__m_pad: usize = 8;
    pub const initCloud: usize = 0;
    pub const initCloud__type: usize = 0;
    pub const initCloud__m_flags: usize = 4;
    pub const initCloud__m_pad: usize = 8;
    pub const initCloud__curves: usize = 16;
    pub const initDecal: usize = 0;
    pub const initDecal__type: usize = 0;
    pub const initDecal__m_flags: usize = 4;
    pub const initDecal__m_fadeInTime: usize = 8;
    pub const initDecal__m_fadeOutTime: usize = 10;
    pub const initDecal__m_stoppableFadeOutTime: usize = 12;
    pub const initDecal__m_lerpWaitTime: usize = 14;
    pub const initDecal__m_lerpColor: usize = 16;
    pub const initDecal__m_lerpColor__v: usize = 16;
    pub const initDecal__m_linkedAssetList: usize = 32;
    pub const initDecal__m_linkedAssetList__assetList: usize = 32;
    pub const initDecal__m_linkedAssetList__numAssets: usize = 40;
    pub const initFlare: usize = 0;
    pub const initFlare__type: usize = 0;
    pub const initFlare__m_flags: usize = 4;
    pub const initFlare__m_pad: usize = 8;
    pub const initFlare__m_position: usize = 12;
    pub const initFlare__m_direction: usize = 16;
    pub const initFlare__m_direction__v: usize = 16;
    pub const initFlare__m_angularRotCount: usize = 32;
    pub const initFlare__m_flareFlags: usize = 36;
    pub const initFlare__m_depthScaleRange: usize = 40;
    pub const initFlare__m_depthScaleRange__min: usize = 40;
    pub const initFlare__m_depthScaleRange__max: usize = 44;
    pub const initFlare__m_depthScaleValue: usize = 48;
    pub const initFlare__m_depthScaleValue__min: usize = 48;
    pub const initFlare__m_depthScaleValue__max: usize = 52;
    pub const initFlare__m_radialRot: usize = 56;
    pub const initFlare__m_radialRot__min: usize = 56;
    pub const initFlare__m_radialRot__max: usize = 60;
    pub const initFlare__m_radialScaleX: usize = 64;
    pub const initFlare__m_radialScaleX__min: usize = 64;
    pub const initFlare__m_radialScaleX__max: usize = 68;
    pub const initFlare__m_radialScaleY: usize = 72;
    pub const initFlare__m_radialScaleY__min: usize = 72;
    pub const initFlare__m_radialScaleY__max: usize = 76;
    pub const initFlare__m_curves: usize = 80;
    pub const initGeoTrail: usize = 0;
    pub const initGeoTrail__type: usize = 0;
    pub const initGeoTrail__m_flags: usize = 4;
    pub const initGeoTrail__m_numPointsMax: usize = 8;
    pub const initGeoTrail__m_splitDistance: usize = 12;
    pub const initGeoTrail__m_splitAngle: usize = 16;
    pub const initGeoTrail__m_centerOffset: usize = 20;
    pub const initGeoTrail__m_numSheets: usize = 24;
    pub const initGeoTrail__m_fadeInDistance: usize = 28;
    pub const initGeoTrail__m_fadeOutDistance: usize = 32;
    pub const initGeoTrail__m_tileDistance: usize = 36;
    pub const initGeoTrail__m_tileOffset: usize = 40;
    pub const initGeoTrail__m_tileOffset__min: usize = 40;
    pub const initGeoTrail__m_tileOffset__max: usize = 44;
    pub const initGeoTrail__m_scrollTime: usize = 48;
    pub const initGeoTrail__m_useLocalVelocity: usize = 52;
    pub const initGeoTrail__m_useVerticalTexture: usize = 53;
    pub const initGeoTrail__m_cameraFacing: usize = 54;
    pub const initGeoTrail__m_fixLeadingEdge: usize = 55;
    pub const initGeoTrail__m_clampUVs: usize = 56;
    pub const initGeoTrail__m_pad: usize = 57;
    pub const initGeoTrail__m_pad2: usize = 60;
    pub const initLightOmni: usize = 0;
    pub const initLightOmni__type: usize = 0;
    pub const initLightOmni__m_flags: usize = 4;
    pub const initLightOmni__m_linkedAssetList: usize = 8;
    pub const initLightOmni__m_linkedAssetList__assetList: usize = 8;
    pub const initLightOmni__m_linkedAssetList__numAssets: usize = 16;
    pub const initLightOmni__m_tonemappingScaleFactor: usize = 24;
    pub const initLightOmni__m_intensityIR: usize = 28;
    pub const initLightOmni__m_disableVolumetric: usize = 32;
    pub const initLightOmni__m_exponent: usize = 33;
    pub const initLightOmni____pad0: usize = 34;
    pub const initLightOmni__m_shadowSoftness: usize = 36;
    pub const initLightOmni__m_shadowBias: usize = 40;
    pub const initLightOmni__m_shadowArea: usize = 44;
    pub const initLightSpot: usize = 0;
    pub const initLightSpot__type: usize = 0;
    pub const initLightSpot__m_flags: usize = 4;
    pub const initLightSpot__m_fovOuter: usize = 8;
    pub const initLightSpot__m_fovInner: usize = 12;
    pub const initLightSpot__m_bulbRadius: usize = 16;
    pub const initLightSpot__m_bulbLength: usize = 20;
    pub const initLightSpot__m_brightness: usize = 24;
    pub const initLightSpot__unk1: usize = 28;
    pub const initLightSpot__m_intensityUV: usize = 32;
    pub const initLightSpot__m_intensityIR: usize = 36;
    pub const initLightSpot__m_shadowNearPlane: usize = 40;
    pub const initLightSpot__m_toneMappingScaleFactor: usize = 44;
    pub const initLightSpot__m_shadowSoftness: usize = 48;
    pub const initLightSpot__m_shadowBias: usize = 52;
    pub const initLightSpot__m_shadowArea: usize = 56;
    pub const initLightSpot__m_disableVolumetric: usize = 60;
    pub const initLightSpot__m_disableShadowMap: usize = 61;
    pub const initLightSpot__m_disableDynamicShadows: usize = 62;
    pub const initLightSpot__m_exponent: usize = 63;
    pub const initLightSpot__m_linkedAssetList: usize = 64;
    pub const initLightSpot__m_linkedAssetList__assetList: usize = 64;
    pub const initLightSpot__m_linkedAssetList__numAssets: usize = 72;
    pub const initMaterial: usize = 0;
    pub const initMaterial__type: usize = 0;
    pub const initMaterial__m_flags: usize = 4;
    pub const initMaterial__m_pad: usize = 8;
    pub const initMaterial__m_linkedAssetList: usize = 16;
    pub const initMaterial__m_linkedAssetList__assetList: usize = 16;
    pub const initMaterial__m_linkedAssetList__numAssets: usize = 24;
    pub const initMirrorTexture: usize = 0;
    pub const initMirrorTexture__type: usize = 0;
    pub const initMirrorTexture__m_flags: usize = 4;
    pub const initMirrorTexture__m_mirrorHorizontal: usize = 8;
    pub const initMirrorTexture__m_mirrorVertical: usize = 12;
    pub const initModel: usize = 0;
    pub const initModel__type: usize = 0;
    pub const initModel__m_flags: usize = 4;
    pub const initModel__m_usePhysics: usize = 8;
    pub const initModel__m_motionBlurHQ: usize = 9;
    pub const initModel__m_linkedAssetList: usize = 16;
    pub const initModel__m_linkedAssetList__assetList: usize = 16;
    pub const initModel__m_linkedAssetList__numAssets: usize = 24;
    pub const initOcclusionQuery: usize = 0;
    pub const initOcclusionQuery__type: usize = 0;
    pub const initOcclusionQuery__m_flags: usize = 4;
    pub const initOcclusionQuery__m_fadeInTime: usize = 8;
    pub const initOcclusionQuery__m_fadeOutTime: usize = 12;
    pub const initOcclusionQuery__m_scale: usize = 16;
    pub const initOcclusionQuery__m_scale__min: usize = 16;
    pub const initOcclusionQuery__m_scale__max: usize = 20;
    pub const initOcclusionQuery__m_worldSize: usize = 24;
    pub const initOrientedSprite: usize = 0;
    pub const initOrientedSprite__type: usize = 0;
    pub const initOrientedSprite__m_flags: usize = 4;
    pub const initOrientedSprite__m_pad: usize = 8;
    pub const initOrientedSprite__m_orientationQuat: usize = 16;
    pub const initOrientedSprite__m_orientationQuat__v: usize = 16;
    pub const initParticleSim: usize = 0;
    pub const initParticleSim__type: usize = 0;
    pub const initParticleSim__m_flags: usize = 4;
    pub const initParticleSim__m_linkedAssetList: usize = 8;
    pub const initParticleSim__m_linkedAssetList__assetList: usize = 8;
    pub const initParticleSim__m_linkedAssetList__numAssets: usize = 16;
    pub const initParticleSim__m_scaleFactor: usize = 24;
    pub const initParticleSim__m_scaleFactor__min: usize = 24;
    pub const initParticleSim__m_scaleFactor__max: usize = 28;
    pub const initParticleSim__m_holdLastFrame: usize = 32;
    pub const initParticleSim__m_pad: usize = 36;
    pub const initPlayerFacing: usize = 0;
    pub const initPlayerFacing__type: usize = 0;
    pub const initPlayerFacing__m_flags: usize = 4;
    pub const initPlayerFacing__m_pad: usize = 8;
    pub const initRelativeVelocity: usize = 0;
    pub const initRelativeVelocity__type: usize = 0;
    pub const initRelativeVelocity__m_flags: usize = 4;
    pub const initRelativeVelocity__m_velocityType: usize = 8;
    pub const initRelativeVelocity__m_useBoltInfo: usize = 12;
    pub const initRotation: usize = 0;
    pub const initRotation__type: usize = 0;
    pub const initRotation__m_flags: usize = 4;
    pub const initRotation__m_pad: usize = 8;
    pub const initRotation__m_rotationAngle: usize = 16;
    pub const initRotation__m_rotationAngle__min: usize = 16;
    pub const initRotation__m_rotationAngle__max: usize = 20;
    pub const initRotation__m_rotationRate: usize = 24;
    pub const initRotation__m_rotationRate__min: usize = 24;
    pub const initRotation__m_rotationRate__max: usize = 28;
    pub const initRotation3D: usize = 0;
    pub const initRotation3D__type: usize = 0;
    pub const initRotation3D__m_flags: usize = 4;
    pub const initRotation3D__m_pad: usize = 8;
    pub const initRotation3D__m_rotationAngleMin: usize = 16;
    pub const initRotation3D__m_rotationAngleMin__v: usize = 16;
    pub const initRotation3D__m_rotationAngleMax: usize = 32;
    pub const initRotation3D__m_rotationAngleMax__v: usize = 32;
    pub const initRotation3D__m_rotationRateMin: usize = 48;
    pub const initRotation3D__m_rotationRateMin__v: usize = 48;
    pub const initRotation3D__m_rotationRateMax: usize = 64;
    pub const initRotation3D__m_rotationRateMax__v: usize = 64;
    pub const initRunner: usize = 0;
    pub const initRunner__type: usize = 0;
    pub const initRunner__m_flags: usize = 4;
    pub const initRunner__m_pad: usize = 8;
    pub const initRunner__m_linkedAssetList: usize = 16;
    pub const initRunner__m_linkedAssetList__assetList: usize = 16;
    pub const initRunner__m_linkedAssetList__numAssets: usize = 24;
    pub const initSound: usize = 0;
    pub const initSound__type: usize = 0;
    pub const initSound__m_flags: usize = 4;
    pub const initSound__m_pad: usize = 8;
    pub const initSound__m_linkedAssetList: usize = 16;
    pub const initSound__m_linkedAssetList__assetList: usize = 16;
    pub const initSound__m_linkedAssetList__numAssets: usize = 24;
    pub const initSpawn: usize = 0;
    pub const initSpawn__type: usize = 0;
    pub const initSpawn__m_flags: usize = 4;
    pub const initSpawn__m_pad: usize = 8;
    pub const initSpawn__m_curves: usize = 16;
    pub const initSpawnShapeBox: usize = 0;
    pub const initSpawnShapeBox__type: usize = 0;
    pub const initSpawnShapeBox__m_flags: usize = 4;
    pub const initSpawnShapeBox__m_axisFlags: usize = 8;
    pub const initSpawnShapeBox__m_spawnFlags: usize = 9;
    pub const initSpawnShapeBox__m_normalAxis: usize = 10;
    pub const initSpawnShapeBox__m_spawnType: usize = 11;
    pub const initSpawnShapeBox__m_volumeCubeRoot: usize = 12;
    pub const initSpawnShapeBox__m_offset: usize = 16;
    pub const initSpawnShapeBox__m_offset__v: usize = 16;
    pub const initSpawnShapeBox__m_useBeamInfo: usize = 32;
    pub const initSpawnShapeBox__m_pad: usize = 33;
    pub const initSpawnShapeBox__m_dimensionsMin: usize = 48;
    pub const initSpawnShapeBox__m_dimensionsMin__v: usize = 48;
    pub const initSpawnShapeBox__m_dimensionsMax: usize = 64;
    pub const initSpawnShapeBox__m_dimensionsMax__v: usize = 64;
    pub const initSpawnShapeCylinder: usize = 0;
    pub const initSpawnShapeCylinder__type: usize = 0;
    pub const initSpawnShapeCylinder__m_flags: usize = 4;
    pub const initSpawnShapeCylinder__m_axisFlags: usize = 8;
    pub const initSpawnShapeCylinder__m_spawnFlags: usize = 9;
    pub const initSpawnShapeCylinder__m_normalAxis: usize = 10;
    pub const initSpawnShapeCylinder__m_spawnType: usize = 11;
    pub const initSpawnShapeCylinder__m_volumeCubeRoot: usize = 12;
    pub const initSpawnShapeCylinder__m_offset: usize = 16;
    pub const initSpawnShapeCylinder__m_offset__v: usize = 16;
    pub const initSpawnShapeCylinder__m_hasRotation: usize = 32;
    pub const initSpawnShapeCylinder__m_rotateCalculatedOffset: usize = 33;
    pub const initSpawnShapeCylinder__m_halfHeight: usize = 36;
    pub const initSpawnShapeCylinder__m_radius: usize = 40;
    pub const initSpawnShapeCylinder__m_radius__min: usize = 40;
    pub const initSpawnShapeCylinder__m_radius__max: usize = 44;
    pub const initSpawnShapeCylinder__unk: usize = 48;
    pub const initSpawnShapeCylinder__unk__v: usize = 48;
    pub const initSpawnShapeCylinder__m_directionQuat: usize = 64;
    pub const initSpawnShapeCylinder__m_directionQuat__v: usize = 64;
    pub const initSpawnShapeEllipsoid: usize = 0;
    pub const initSpawnShapeEllipsoid__type: usize = 0;
    pub const initSpawnShapeEllipsoid__m_flags: usize = 4;
    pub const initSpawnShapeEllipsoid__m_axisFlags: usize = 8;
    pub const initSpawnShapeEllipsoid__m_spawnFlags: usize = 9;
    pub const initSpawnShapeEllipsoid__m_normalAxis: usize = 10;
    pub const initSpawnShapeEllipsoid__m_spawnType: usize = 11;
    pub const initSpawnShapeEllipsoid__m_volumeCubeRoot: usize = 12;
    pub const initSpawnShapeEllipsoid__m_offset: usize = 16;
    pub const initSpawnShapeEllipsoid__m_offset__v: usize = 16;
    pub const initSpawnShapeEllipsoid__m_radiusMin: usize = 32;
    pub const initSpawnShapeEllipsoid__m_radiusMin__v: usize = 32;
    pub const initSpawnShapeEllipsoid__m_radiusMax: usize = 48;
    pub const initSpawnShapeEllipsoid__m_radiusMax__v: usize = 48;
    pub const initSpawnShapeEllipsoid__m_pad: usize = 64;
    pub const initSpawnShapeMesh: usize = 0;
    pub const initSpawnShapeMesh__type: usize = 0;
    pub const initSpawnShapeMesh__m_flags: usize = 4;
    pub const initSpawnShapeMesh__m_axisFlags: usize = 8;
    pub const initSpawnShapeMesh__m_spawnFlags: usize = 9;
    pub const initSpawnShapeMesh__m_normalAxis: usize = 10;
    pub const initSpawnShapeMesh__m_spawnType: usize = 11;
    pub const initSpawnShapeMesh__m_volumeCubeRoot: usize = 12;
    pub const initSpawnShapeMesh__m_offset: usize = 16;
    pub const initSpawnShapeMesh__m_offset__v: usize = 16;
    pub const initSpawnShapeMesh__m_numMeshAssets: usize = 32;
    pub const initSpawnShapeMesh__m_linkedAssetList: usize = 40;
    pub const initSpawnShapeMesh__m_linkedAssetList__assetList: usize = 40;
    pub const initSpawnShapeMesh__m_linkedAssetList__numAssets: usize = 48;
    pub const initSpawnShapeMesh__m_meshAssetData: usize = 56;
    pub const initSpawnShapeMesh__m_pad: usize = 64;
    pub const initSpawnShapeSphere: usize = 0;
    pub const initSpawnShapeSphere__type: usize = 0;
    pub const initSpawnShapeSphere__m_flags: usize = 4;
    pub const initSpawnShapeSphere__m_axisFlags: usize = 8;
    pub const initSpawnShapeSphere__m_spawnFlags: usize = 9;
    pub const initSpawnShapeSphere__m_normalAxis: usize = 10;
    pub const initSpawnShapeSphere__m_spawnType: usize = 11;
    pub const initSpawnShapeSphere__m_volumeCubeRoot: usize = 12;
    pub const initSpawnShapeSphere__m_offset: usize = 16;
    pub const initSpawnShapeSphere__m_offset__v: usize = 16;
    pub const initSpawnShapeSphere__m_pad: usize = 32;
    pub const initSpawnShapeSphere__m_radius: usize = 40;
    pub const initSpawnShapeSphere__m_radius__min: usize = 40;
    pub const initSpawnShapeSphere__m_radius__max: usize = 44;
    pub const initSpawnShapeSphere__m_offset2: usize = 48;
    pub const initSpawnShapeSphere__m_offset2__v: usize = 48;
    pub const initTail: usize = 0;
    pub const initTail__type: usize = 0;
    pub const initTail__m_flags: usize = 4;
    pub const initTail__m_averagePastVelocities: usize = 8;
    pub const initTail__m_maxParentSpeed: usize = 10;
    pub const initTail__m_tailLeading: usize = 12;
    pub const initTail__m_scaleWithVelocity: usize = 13;
    pub const initTail__m_rotateAroundPivot: usize = 14;
    pub const initVectorField: usize = 0;
    pub const initVectorField__type: usize = 0;
    pub const initVectorField__m_flags: usize = 4;
    pub const initVectorField__m_pad: usize = 8;
    pub const initVectorField__m_linkedAssetList: usize = 16;
    pub const initVectorField__m_linkedAssetList__assetList: usize = 16;
    pub const initVectorField__m_linkedAssetList__numAssets: usize = 24;
    pub const initVolumetric: usize = 0;
    pub const initVolumetric__type: usize = 0;
    pub const initVolumetric__m_flags: usize = 4;
    pub const initVolumetric__m_density: usize = 8;
    pub const initVolumetric__m_falloff: usize = 12;
    pub const initVolumetric__m_noiseAmplitude: usize = 16;
    pub const initVolumetric__m_noiseMode: usize = 20;
    pub const initVolumetric__m_pad: usize = 24;
    pub const attractor: usize = 0;
    pub const attractor__type: usize = 0;
    pub const attractor__m_flags: usize = 4;
    pub const attractor__m_pad: usize = 8;
    pub const attractor__m_killWhenNear: usize = 20;
    pub const attractor__m_forceMag: usize = 24;
    pub const attractor__m_nearDistanceSq: usize = 28;
    pub const attractor__m_attractPoint: usize = 32;
    pub const attractor__m_attractPoint__v: usize = 32;
    pub const colorGraph: usize = 0;
    pub const colorGraph__type: usize = 0;
    pub const colorGraph__m_flags: usize = 4;
    pub const colorGraph__firstCurve: usize = 8;
    pub const colorGraph__m_pad: usize = 9;
    pub const colorGraph__m_modulateColorByAlpha: usize = 12;
    pub const colorGraph__m_curves: usize = 16;
    pub const colorLerp: usize = 0;
    pub const colorLerp__type: usize = 0;
    pub const colorLerp__m_flags: usize = 4;
    pub const colorLerp__m_pad: usize = 8;
    pub const colorLerp__m_colorBegin: usize = 16;
    pub const colorLerp__m_colorBegin__v: usize = 16;
    pub const colorLerp__m_colorEnd: usize = 32;
    pub const colorLerp__m_colorEnd__v: usize = 32;
    pub const emissionGraph: usize = 0;
    pub const emissionGraph__type: usize = 0;
    pub const emissionGraph__m_flags: usize = 4;
    pub const emissionGraph__m_pad: usize = 8;
    pub const emissionGraph__m_curves: usize = 16;
    pub const emissiveGraph: usize = 0;
    pub const emissiveGraph__type: usize = 0;
    pub const emissiveGraph__m_flags: usize = 4;
    pub const emissiveGraph__firstCurve: usize = 8;
    pub const emissiveGraph__m_pad: usize = 9;
    pub const emissiveGraph__m_curves: usize = 16;
    pub const force: usize = 0;
    pub const force__type: usize = 0;
    pub const force__m_flags: usize = 4;
    pub const force__m_pad: usize = 8;
    pub const force__m_forceMin: usize = 16;
    pub const force__m_forceMin__v: usize = 16;
    pub const force__m_forceMax: usize = 32;
    pub const force__m_forceMax__v: usize = 32;
    pub const gravity: usize = 0;
    pub const gravity__type: usize = 0;
    pub const gravity__m_flags: usize = 4;
    pub const gravity__m_gravityPercentage: usize = 8;
    pub const gravity__m_gravityPercentage__min: usize = 8;
    pub const gravity__m_gravityPercentage__max: usize = 12;
    pub const intensityGraph: usize = 0;
    pub const intensityGraph__type: usize = 0;
    pub const intensityGraph__m_flags: usize = 4;
    pub const intensityGraph__firstCurve: usize = 8;
    pub const intensityGraph__m_pad: usize = 9;
    pub const intensityGraph__m_curves: usize = 16;
    pub const parentVelocityGraph: usize = 0;
    pub const parentVelocityGraph__type: usize = 0;
    pub const parentVelocityGraph__m_flags: usize = 4;
    pub const parentVelocityGraph__m_pad: usize = 8;
    pub const parentVelocityGraph__m_curves: usize = 16;
    pub const physicsLight: usize = 0;
    pub const physicsLight__type: usize = 0;
    pub const physicsLight__m_flags: usize = 4;
    pub const physicsLight__m_ignoreEmitterOrientation: usize = 8;
    pub const physicsLight__m_useSurfaceType: usize = 9;
    pub const physicsLight__m_pad: usize = 10;
    pub const physicsLight__m_linkedAssetList: usize = 16;
    pub const physicsLight__m_linkedAssetList__assetList: usize = 16;
    pub const physicsLight__m_linkedAssetList__numAssets: usize = 24;
    pub const physicsRayCast: usize = 0;
    pub const physicsRayCast__type: usize = 0;
    pub const physicsRayCast__m_flags: usize = 4;
    pub const physicsRayCast__m_bounce: usize = 8;
    pub const physicsRayCast__m_bounce__min: usize = 8;
    pub const physicsRayCast__m_bounce__max: usize = 12;
    pub const physicsRayCast__m_bounds: usize = 16;
    pub const physicsRayCast__m_bounds__midPoint: usize = 16;
    pub const physicsRayCast__m_bounds__halfSize: usize = 28;
    pub const physicsRayCast__m_useItemClip: usize = 40;
    pub const physicsRayCast__m_useSurfaceType: usize = 41;
    pub const physicsRayCast__m_collideWithWater: usize = 42;
    pub const physicsRayCast__m_ignoreContentItem: usize = 43;
    pub const physicsRayCast__m_pad: usize = 44;
    pub const positionGraph: usize = 0;
    pub const positionGraph__type: usize = 0;
    pub const positionGraph__m_flags: usize = 4;
    pub const positionGraph__m_pad: usize = 8;
    pub const positionGraph__m_curves: usize = 16;
    pub const rotationGraph: usize = 0;
    pub const rotationGraph__type: usize = 0;
    pub const rotationGraph__m_flags: usize = 4;
    pub const rotationGraph__m_useRotationRate: usize = 8;
    pub const rotationGraph__m_pad: usize = 12;
    pub const rotationGraph__m_curves: usize = 16;
    pub const rotationGraph3D: usize = 0;
    pub const rotationGraph3D__type: usize = 0;
    pub const rotationGraph3D__m_flags: usize = 4;
    pub const rotationGraph3D__m_useRotationRate: usize = 8;
    pub const rotationGraph3D__m_pad: usize = 12;
    pub const rotationGraph3D__m_curves: usize = 16;
    pub const sizeGraph: usize = 0;
    pub const sizeGraph__type: usize = 0;
    pub const sizeGraph__m_flags: usize = 4;
    pub const sizeGraph__firstCurve: usize = 8;
    pub const sizeGraph__m_pad: usize = 9;
    pub const sizeGraph__m_curves: usize = 16;
    pub const sizeGraph__m_sizeBegin: usize = 112;
    pub const sizeGraph__m_sizeBegin__v: usize = 112;
    pub const sizeGraph__m_sizeEnd: usize = 128;
    pub const sizeGraph__m_sizeEnd__v: usize = 128;
    pub const sizeLerp: usize = 0;
    pub const sizeLerp__type: usize = 0;
    pub const sizeLerp__m_flags: usize = 4;
    pub const sizeLerp__m_pad: usize = 8;
    pub const sizeLerp__m_sizeBegin: usize = 16;
    pub const sizeLerp__m_sizeBegin__v: usize = 16;
    pub const sizeLerp__m_sizeEnd: usize = 32;
    pub const sizeLerp__m_sizeEnd__v: usize = 32;
    pub const velocityGraph: usize = 0;
    pub const velocityGraph__type: usize = 0;
    pub const velocityGraph__m_flags: usize = 4;
    pub const velocityGraph__m_pad: usize = 8;
    pub const velocityGraph__m_curves: usize = 16;
    pub const velocityGraph__m_velocityBegin: usize = 112;
    pub const velocityGraph__m_velocityBegin__v: usize = 112;
    pub const velocityGraph__m_velocityEnd: usize = 128;
    pub const velocityGraph__m_velocityEnd__v: usize = 128;
    pub const testAge: usize = 0;
    pub const testAge__type: usize = 0;
    pub const testAge__m_flags: usize = 4;
    pub const testAge__m_moduleIndex: usize = 8;
    pub const testAge__m_useOrientationOptions: usize = 10;
    pub const testAge__m_useScaleOptions: usize = 11;
    pub const testAge__m_useVelocityOptions: usize = 12;
    pub const testAge__m_eventHandlerData: usize = 16;
    pub const testAge__m_eventHandlerData__m_nextState: usize = 16;
    pub const testAge__m_eventHandlerData__m_linkedAssetList: usize = 24;
    pub const testAge__m_eventHandlerData__m_linkedAssetList__assetList: usize = 24;
    pub const testAge__m_eventHandlerData__m_linkedAssetList__numAssets: usize = 32;
    pub const testAge__m_eventHandlerData__m_kill: usize = 40;
    pub const testAge__m_eventHandlerData__m_pad: usize = 44;
    pub const testAge__m_age: usize = 48;
    pub const testBirth: usize = 0;
    pub const testBirth__type: usize = 0;
    pub const testBirth__m_flags: usize = 4;
    pub const testBirth__m_moduleIndex: usize = 8;
    pub const testBirth__m_useOrientationOptions: usize = 10;
    pub const testBirth__m_useScaleOptions: usize = 11;
    pub const testBirth__m_useVelocityOptions: usize = 12;
    pub const testBirth__m_eventHandlerData: usize = 16;
    pub const testBirth__m_eventHandlerData__m_nextState: usize = 16;
    pub const testBirth__m_eventHandlerData__m_linkedAssetList: usize = 24;
    pub const testBirth__m_eventHandlerData__m_linkedAssetList__assetList: usize = 24;
    pub const testBirth__m_eventHandlerData__m_linkedAssetList__numAssets: usize = 32;
    pub const testBirth__m_eventHandlerData__m_kill: usize = 40;
    pub const testBirth__m_eventHandlerData__m_pad: usize = 44;
    pub const testDeath: usize = 0;
    pub const testDeath__type: usize = 0;
    pub const testDeath__m_flags: usize = 4;
    pub const testDeath__m_moduleIndex: usize = 8;
    pub const testDeath__m_useOrientationOptions: usize = 10;
    pub const testDeath__m_useScaleOptions: usize = 11;
    pub const testDeath__m_useVelocityOptions: usize = 12;
    pub const testDeath__m_eventHandlerData: usize = 16;
    pub const testDeath__m_eventHandlerData__m_nextState: usize = 16;
    pub const testDeath__m_eventHandlerData__m_linkedAssetList: usize = 24;
    pub const testDeath__m_eventHandlerData__m_linkedAssetList__assetList: usize = 24;
    pub const testDeath__m_eventHandlerData__m_linkedAssetList__numAssets: usize = 32;
    pub const testDeath__m_eventHandlerData__m_kill: usize = 40;
    pub const testDeath__m_eventHandlerData__m_pad: usize = 44;
    pub const testGravity: usize = 0;
    pub const testGravity__type: usize = 0;
    pub const testGravity__m_flags: usize = 4;
    pub const testGravity__m_moduleIndex: usize = 8;
    pub const testGravity__m_useOrientationOptions: usize = 10;
    pub const testGravity__m_useScaleOptions: usize = 11;
    pub const testGravity__m_useVelocityOptions: usize = 12;
    pub const testGravity__m_eventHandlerData: usize = 16;
    pub const testGravity__m_eventHandlerData__m_nextState: usize = 16;
    pub const testGravity__m_eventHandlerData__m_linkedAssetList: usize = 24;
    pub const testGravity__m_eventHandlerData__m_linkedAssetList__assetList: usize = 24;
    pub const testGravity__m_eventHandlerData__m_linkedAssetList__numAssets: usize = 32;
    pub const testGravity__m_eventHandlerData__m_kill: usize = 40;
    pub const testGravity__m_eventHandlerData__m_pad: usize = 44;
    pub const testImpact: usize = 0;
    pub const testImpact__type: usize = 0;
    pub const testImpact__m_flags: usize = 4;
    pub const testImpact__m_moduleIndex: usize = 8;
    pub const testImpact__m_useOrientationOptions: usize = 10;
    pub const testImpact__m_useScaleOptions: usize = 11;
    pub const testImpact__m_useVelocityOptions: usize = 12;
    pub const testImpact__m_eventHandlerData: usize = 16;
    pub const testImpact__m_eventHandlerData__m_nextState: usize = 16;
    pub const testImpact__m_eventHandlerData__m_linkedAssetList: usize = 24;
    pub const testImpact__m_eventHandlerData__m_linkedAssetList__assetList: usize = 24;
    pub const testImpact__m_eventHandlerData__m_linkedAssetList__numAssets: usize = 32;
    pub const testImpact__m_eventHandlerData__m_kill: usize = 40;
    pub const testImpact__m_eventHandlerData__m_pad: usize = 44;
    pub const testPos: usize = 0;
    pub const testPos__type: usize = 0;
    pub const testPos__m_flags: usize = 4;
    pub const testPos__m_moduleIndex: usize = 8;
    pub const testPos__m_useOrientationOptions: usize = 10;
    pub const testPos__m_useScaleOptions: usize = 11;
    pub const testPos__m_useVelocityOptions: usize = 12;
    pub const testPos__m_eventHandlerData: usize = 16;
    pub const testPos__m_eventHandlerData__m_nextState: usize = 16;
    pub const testPos__m_eventHandlerData__m_linkedAssetList: usize = 24;
    pub const testPos__m_eventHandlerData__m_linkedAssetList__assetList: usize = 24;
    pub const testPos__m_eventHandlerData__m_linkedAssetList__numAssets: usize = 32;
    pub const testPos__m_eventHandlerData__m_kill: usize = 40;
    pub const testPos__m_eventHandlerData__m_pad: usize = 44;
}
pub mod ParticleModuleVelocityGraph {
    pub const SIZE: usize = 144;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_pad: usize = 8;
    pub const m_curves: usize = 16;
    pub const m_velocityBegin: usize = 112;
    pub const m_velocityBegin__v: usize = 112;
    pub const m_velocityEnd: usize = 128;
    pub const m_velocityEnd__v: usize = 128;
}
pub mod ParticlePhysicsFXData {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const physicsFXPipeline: usize = 0;
    pub const physicsFXShape: usize = 8;
}
pub mod ParticleScriptedInputNode {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const m_type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_emitterIndex: usize = 8;
    pub const m_pad: usize = 12;
}
pub mod ParticleScriptedInputNodeDef {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 4;
    pub const nodeBase: usize = 0;
    pub const nodeBase__m_type: usize = 0;
    pub const nodeBase__m_flags: usize = 4;
    pub const nodeBase__m_emitterIndex: usize = 8;
    pub const nodeBase__m_pad: usize = 12;
    pub const KVP: usize = 0;
    pub const KVP__m_type: usize = 0;
    pub const KVP__m_flags: usize = 4;
    pub const KVP__m_emitterIndex: usize = 8;
    pub const KVP__m_pad: usize = 12;
    pub const KVP__m_inputRecord: usize = 32;
    pub const KVP__m_inputRecord__key: usize = 32;
    pub const KVP__m_inputRecord__dirty: usize = 36;
    pub const KVP__m_inputRecord__value: usize = 37;
    pub const KVP__m_inputRecord__pad: usize = 40;
    pub const emitterDisable: usize = 0;
    pub const emitterDisable__m_type: usize = 0;
    pub const emitterDisable__m_flags: usize = 4;
    pub const emitterDisable__m_emitterIndex: usize = 8;
    pub const emitterDisable__m_pad: usize = 12;
}
pub mod ParticleScriptedInputNodeEmitterDisable {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const m_type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_emitterIndex: usize = 8;
    pub const m_pad: usize = 12;
}
pub mod ParticleScriptedInputNodeKVP {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 4;
    pub const m_type: usize = 0;
    pub const m_flags: usize = 4;
    pub const m_emitterIndex: usize = 8;
    pub const m_pad: usize = 12;
    pub const m_inputRecord: usize = 32;
    pub const m_inputRecord__key: usize = 32;
    pub const m_inputRecord__dirty: usize = 36;
    pub const m_inputRecord__value: usize = 37;
    pub const m_inputRecord__pad: usize = 40;
}
pub mod ParticleScriptedInputRecord {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const key: usize = 0;
    pub const dirty: usize = 4;
    pub const value: usize = 5;
    pub const pad: usize = 8;
}
pub mod ParticleSpawnMeshAssetDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const m_numVertsTotal: usize = 0;
    pub const m_numTrisTotal: usize = 4;
    pub const m_pad: usize = 8;
}
pub mod ParticleStateDef {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const moduleGroupDefs: usize = 0;
    pub const elementType: usize = 8;
    pub const flags: usize = 16;
    pub const pad: usize = 24;
}
pub mod ParticleSystemDef {
    pub const SIZE: usize = 128;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const emitterDefs: usize = 8;
    pub const scriptedInputNodeDefs: usize = 16;
    pub const version: usize = 24;
    pub const numEmitters: usize = 28;
    pub const numScriptedInputNodes: usize = 32;
    pub const flags: usize = 36;
    pub const occlusionOverrideEmitterIndex: usize = 40;
    pub const phaseOptions: usize = 44;
    pub const drawFrustumCullRadius: usize = 48;
    pub const updateFrustumCullRadius: usize = 52;
    pub const sunDistance: usize = 56;
    pub const preRollMSec: usize = 60;
    pub const editorPosition: usize = 64;
    pub const editorPosition__v: usize = 64;
    pub const editorRotation: usize = 80;
    pub const editorRotation__v: usize = 80;
    pub const gameTweakPosition: usize = 96;
    pub const gameTweakPosition__v: usize = 96;
    pub const gameTweakRotation: usize = 112;
    pub const gameTweakRotation__v: usize = 112;
}
pub mod PathData {
    pub const SIZE: usize = 144;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const nodeCount: usize = 8;
    pub const nodes: usize = 16;
    pub const parentIndexResolved: usize = 24;
    pub const version: usize = 26;
    pub const visBytes: usize = 28;
    pub const pathVis: usize = 32;
    pub const nodeTreeCount: usize = 40;
    pub const nodeTree: usize = 48;
    pub const dynamicNodeGroupCount: usize = 56;
    pub const dynamicNodeGroups: usize = 64;
    pub const exposureBytes: usize = 72;
    pub const pathExposure: usize = 80;
    pub const noPeekVisBytes: usize = 88;
    pub const pathNoPeekVis: usize = 96;
    pub const zoneCount: usize = 104;
    pub const zonesBytes: usize = 108;
    pub const pathZones: usize = 112;
    pub const fixedNodeCount: usize = 120;
    pub const maxDynamicSpawnedNodeCount: usize = 124;
    pub const dynStatesBytes: usize = 128;
    pub const pathDynStates: usize = 136;
}
pub mod PathDynamicNodeGroup {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const parentIndex: usize = 0;
    pub const nodeTreeCount: usize = 4;
    pub const nodeTree: usize = 8;
}
pub mod PathNodeOrientationUnion {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const yaw_orient: usize = 0;
    pub const yaw_orient__fLocalAngle: usize = 0;
    pub const yaw_orient__localForward: usize = 4;
    pub const angles: usize = 0;
}
pub mod PathNodeParentUnion {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 4;
    pub const name: usize = 0;
    pub const index: usize = 0;
}
pub mod PhysicsAsset {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const havokDataSize: usize = 8;
    pub const havokData: usize = 16;
    pub const numRigidBodies: usize = 24;
    pub const numConstraints: usize = 28;
    pub const numSFXEventAssets: usize = 32;
    pub const sfxEventAssets: usize = 40;
    pub const numVFXEventAssets: usize = 48;
    pub const vfxEventAssets: usize = 56;
    pub const physicsUsageCounter: usize = 64;
    pub const physicsUsageCounter__serverEnt: usize = 64;
    pub const physicsUsageCounter__clientEnt: usize = 68;
    pub const physicsUsageCounter__dynEnt: usize = 72;
    pub const __pad0: usize = 76;
}
pub mod PhysicsAssetUsageCounter {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const serverEnt: usize = 0;
    pub const clientEnt: usize = 4;
    pub const dynEnt: usize = 8;
}
pub mod PhysicsCapacities {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const maxNumRigidBodiesServer: usize = 0;
    pub const maxNumDetailRigidBodiesServer: usize = 4;
    pub const maxNumConstraintsServer: usize = 8;
    pub const maxNumMotionsServer: usize = 12;
    pub const maxNumRigidBodiesClient: usize = 16;
    pub const maxNumDetailRigidBodiesClient: usize = 20;
    pub const maxNumConstraintsClient: usize = 24;
    pub const maxNumMotionsClient: usize = 28;
}
pub mod PhysicsFXPipeline {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const __pad0: usize = 8;
    pub const unk: usize = 16;
    pub const unk__numUnk: usize = 16;
    pub const unk__unk: usize = 24;
}
pub mod PhysicsFXShape {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const size: usize = 8;
    pub const vecs: usize = 16;
}
pub mod PhysicsLibrary {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const isMaterialList: usize = 8;
    pub const isBodyQualityList: usize = 9;
    pub const isMotionPropertiesList: usize = 10;
    pub const isGlobalTypeCompendium: usize = 11;
    pub const havokDataSize: usize = 12;
    pub const havokData: usize = 16;
}
pub mod PhysicsSFXEventAsset {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const priority: usize = 8;
    pub const rules: usize = 16;
    pub const rules__numrules: usize = 16;
    pub const rules__rules: usize = 24;
}
pub mod PhysicsSFXEventAssetAngleEntry {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const angle: usize = 0;
    pub const hitSoundVolumeModifierMin: usize = 4;
    pub const hitSoundVolumeModifierMax: usize = 8;
    pub const hitSoundPitchModifierMin: usize = 12;
    pub const hitSoundPitchModifierMax: usize = 16;
    pub const rules: usize = 24;
    pub const rules__numrules: usize = 24;
    pub const rules__rules: usize = 32;
}
pub mod PhysicsSFXEventAssetAngleRule {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const numentries: usize = 0;
    pub const entries: usize = 8;
}
pub mod PhysicsSFXEventAssetAtmosphereEntry {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const atmosphere: usize = 0;
    pub const rules: usize = 8;
    pub const rules__numrules: usize = 8;
    pub const rules__rules: usize = 16;
}
pub mod PhysicsSFXEventAssetAtmosphereRule {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const numentries: usize = 0;
    pub const entries: usize = 8;
}
pub mod PhysicsSFXEventAssetCameraDistanceEntry {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const distance: usize = 0;
    pub const rules: usize = 8;
    pub const rules__numrules: usize = 8;
    pub const rules__rules: usize = 16;
}
pub mod PhysicsSFXEventAssetCameraDistanceRule {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const numentries: usize = 0;
    pub const entries: usize = 8;
}
pub mod PhysicsSFXEventAssetFrequencyEntry {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const maxEvents: usize = 0;
    pub const rules: usize = 8;
    pub const rules__numrules: usize = 8;
    pub const rules__rules: usize = 16;
}
pub mod PhysicsSFXEventAssetFrequencyRule {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const numentries: usize = 0;
    pub const entries: usize = 8;
}
pub mod PhysicsSFXEventAssetGravityEntry {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const gravity: usize = 0;
    pub const rules: usize = 8;
    pub const rules__numrules: usize = 8;
    pub const rules__rules: usize = 16;
}
pub mod PhysicsSFXEventAssetGravityRule {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const numentries: usize = 0;
    pub const entries: usize = 8;
}
pub mod PhysicsSFXEventAssetMassEntry {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const massThreshold: usize = 0;
    pub const hitSoundVolumeModifierMin: usize = 4;
    pub const hitSoundVolumeModifierMax: usize = 8;
    pub const hitSoundPitchModifierMin: usize = 12;
    pub const hitSoundPitchModifierMax: usize = 16;
    pub const rules: usize = 24;
    pub const rules__numrules: usize = 24;
    pub const rules__rules: usize = 32;
}
pub mod PhysicsSFXEventAssetMassRule {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const numentries: usize = 0;
    pub const entries: usize = 8;
}
pub mod PhysicsSFXEventAssetMaterialEntry {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const surfaceType: usize = 0;
    pub const rules: usize = 8;
    pub const rules__numrules: usize = 8;
    pub const rules__rules: usize = 16;
}
pub mod PhysicsSFXEventAssetMaterialRule {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const numentries: usize = 0;
    pub const entries: usize = 8;
}
pub mod PhysicsSFXEventAssetRule {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const u: usize = 8;
    pub const u__soundRule: usize = 8;
    pub const u__soundRule__hitSoundAlias: usize = 8;
    pub const u__soundRule__scrapeSoundAlias: usize = 16;
    pub const u__soundRule__scrapeSoundVolumeSpeedThresholdMin: usize = 24;
    pub const u__soundRule__scrapeSoundVolumeSpeedThresholdMax: usize = 28;
    pub const u__soundRule__scrapeSoundVolumeBlendSpeed: usize = 32;
    pub const u__materialRule: usize = 8;
    pub const u__materialRule__numentries: usize = 8;
    pub const u__materialRule__entries: usize = 16;
    pub const u__angleRule: usize = 8;
    pub const u__angleRule__numentries: usize = 8;
    pub const u__angleRule__entries: usize = 16;
    pub const u__speedRule: usize = 8;
    pub const u__speedRule__numentries: usize = 8;
    pub const u__speedRule__entries: usize = 16;
    pub const u__massRule: usize = 8;
    pub const u__massRule__numentries: usize = 8;
    pub const u__massRule__entries: usize = 16;
    pub const u__gravityRule: usize = 8;
    pub const u__gravityRule__numentries: usize = 8;
    pub const u__gravityRule__entries: usize = 16;
    pub const u__atmosphereRule: usize = 8;
    pub const u__atmosphereRule__numentries: usize = 8;
    pub const u__atmosphereRule__entries: usize = 16;
    pub const u__cameraDistanceRule: usize = 8;
    pub const u__cameraDistanceRule__numentries: usize = 8;
    pub const u__cameraDistanceRule__entries: usize = 16;
    pub const u__frequencyRule: usize = 8;
    pub const u__frequencyRule__numentries: usize = 8;
    pub const u__frequencyRule__entries: usize = 16;
}
pub mod PhysicsSFXEventAssetRuleUnion {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const soundRule: usize = 0;
    pub const soundRule__hitSoundAlias: usize = 0;
    pub const soundRule__scrapeSoundAlias: usize = 8;
    pub const soundRule__scrapeSoundVolumeSpeedThresholdMin: usize = 16;
    pub const soundRule__scrapeSoundVolumeSpeedThresholdMax: usize = 20;
    pub const soundRule__scrapeSoundVolumeBlendSpeed: usize = 24;
    pub const materialRule: usize = 0;
    pub const materialRule__numentries: usize = 0;
    pub const materialRule__entries: usize = 8;
    pub const angleRule: usize = 0;
    pub const angleRule__numentries: usize = 0;
    pub const angleRule__entries: usize = 8;
    pub const speedRule: usize = 0;
    pub const speedRule__numentries: usize = 0;
    pub const speedRule__entries: usize = 8;
    pub const massRule: usize = 0;
    pub const massRule__numentries: usize = 0;
    pub const massRule__entries: usize = 8;
    pub const gravityRule: usize = 0;
    pub const gravityRule__numentries: usize = 0;
    pub const gravityRule__entries: usize = 8;
    pub const atmosphereRule: usize = 0;
    pub const atmosphereRule__numentries: usize = 0;
    pub const atmosphereRule__entries: usize = 8;
    pub const cameraDistanceRule: usize = 0;
    pub const cameraDistanceRule__numentries: usize = 0;
    pub const cameraDistanceRule__entries: usize = 8;
    pub const frequencyRule: usize = 0;
    pub const frequencyRule__numentries: usize = 0;
    pub const frequencyRule__entries: usize = 8;
}
pub mod PhysicsSFXEventAssetRules {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const numrules: usize = 0;
    pub const rules: usize = 8;
}
pub mod PhysicsSFXEventAssetSoundRule {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const hitSoundAlias: usize = 0;
    pub const scrapeSoundAlias: usize = 8;
    pub const scrapeSoundVolumeSpeedThresholdMin: usize = 16;
    pub const scrapeSoundVolumeSpeedThresholdMax: usize = 20;
    pub const scrapeSoundVolumeBlendSpeed: usize = 24;
}
pub mod PhysicsSFXEventAssetSpeedEntry {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const speedThreshold: usize = 0;
    pub const hitSoundVolumeModifierMin: usize = 4;
    pub const hitSoundVolumeModifierMax: usize = 8;
    pub const hitSoundPitchModifierMin: usize = 12;
    pub const hitSoundPitchModifierMax: usize = 16;
    pub const rules: usize = 24;
    pub const rules__numrules: usize = 24;
    pub const rules__rules: usize = 32;
}
pub mod PhysicsSFXEventAssetSpeedRule {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const numentries: usize = 0;
    pub const entries: usize = 8;
}
pub mod PhysicsVFXEventAsset {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const priority: usize = 8;
    pub const rules: usize = 16;
    pub const rules__numrules: usize = 16;
    pub const rules__rules: usize = 24;
}
pub mod PhysicsVFXEventAssetAngleEntry {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const angle: usize = 0;
    pub const rules: usize = 8;
    pub const rules__numrules: usize = 8;
    pub const rules__rules: usize = 16;
}
pub mod PhysicsVFXEventAssetAngleRule {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const numentries: usize = 0;
    pub const entries: usize = 8;
}
pub mod PhysicsVFXEventAssetAtmosphereEntry {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const atmosphere: usize = 0;
    pub const rules: usize = 8;
    pub const rules__numrules: usize = 8;
    pub const rules__rules: usize = 16;
}
pub mod PhysicsVFXEventAssetAtmosphereRule {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const numentries: usize = 0;
    pub const entries: usize = 8;
}
pub mod PhysicsVFXEventAssetCameraDistanceEntry {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const distance: usize = 0;
    pub const rules: usize = 8;
    pub const rules__numrules: usize = 8;
    pub const rules__rules: usize = 16;
}
pub mod PhysicsVFXEventAssetCameraDistanceRule {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const numentries: usize = 0;
    pub const entries: usize = 8;
}
pub mod PhysicsVFXEventAssetFrequencyEntry {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const maxEvents: usize = 0;
    pub const rules: usize = 8;
    pub const rules__numrules: usize = 8;
    pub const rules__rules: usize = 16;
}
pub mod PhysicsVFXEventAssetFrequencyRule {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const numentries: usize = 0;
    pub const entries: usize = 8;
}
pub mod PhysicsVFXEventAssetGravityEntry {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const gravity: usize = 0;
    pub const rules: usize = 8;
    pub const rules__numrules: usize = 8;
    pub const rules__rules: usize = 16;
}
pub mod PhysicsVFXEventAssetGravityRule {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const numentries: usize = 0;
    pub const entries: usize = 8;
}
pub mod PhysicsVFXEventAssetMassEntry {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const massThreshold: usize = 0;
    pub const rules: usize = 8;
    pub const rules__numrules: usize = 8;
    pub const rules__rules: usize = 16;
}
pub mod PhysicsVFXEventAssetMassRule {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const numentries: usize = 0;
    pub const entries: usize = 8;
}
pub mod PhysicsVFXEventAssetMaterialEntry {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const surfaceType: usize = 0;
    pub const rules: usize = 8;
    pub const rules__numrules: usize = 8;
    pub const rules__rules: usize = 16;
}
pub mod PhysicsVFXEventAssetMaterialRule {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const numentries: usize = 0;
    pub const entries: usize = 8;
}
pub mod PhysicsVFXEventAssetParticleEffectRule {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const hitParticleEffectAlias: usize = 0;
    pub const scrapeParticleEffectAlias: usize = 8;
}
pub mod PhysicsVFXEventAssetRule {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const u: usize = 8;
    pub const u__particleEffectRule: usize = 8;
    pub const u__particleEffectRule__hitParticleEffectAlias: usize = 8;
    pub const u__particleEffectRule__scrapeParticleEffectAlias: usize = 16;
    pub const u__materialRule: usize = 8;
    pub const u__materialRule__numentries: usize = 8;
    pub const u__materialRule__entries: usize = 16;
    pub const u__angleRule: usize = 8;
    pub const u__angleRule__numentries: usize = 8;
    pub const u__angleRule__entries: usize = 16;
    pub const u__speedRule: usize = 8;
    pub const u__speedRule__numentries: usize = 8;
    pub const u__speedRule__entries: usize = 16;
    pub const u__massRule: usize = 8;
    pub const u__massRule__numentries: usize = 8;
    pub const u__massRule__entries: usize = 16;
    pub const u__gravityRule: usize = 8;
    pub const u__gravityRule__numentries: usize = 8;
    pub const u__gravityRule__entries: usize = 16;
    pub const u__atmosphereRule: usize = 8;
    pub const u__atmosphereRule__numentries: usize = 8;
    pub const u__atmosphereRule__entries: usize = 16;
    pub const u__cameraDistanceRule: usize = 8;
    pub const u__cameraDistanceRule__numentries: usize = 8;
    pub const u__cameraDistanceRule__entries: usize = 16;
    pub const u__frequencyRule: usize = 8;
    pub const u__frequencyRule__numentries: usize = 8;
    pub const u__frequencyRule__entries: usize = 16;
}
pub mod PhysicsVFXEventAssetRuleUnion {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const particleEffectRule: usize = 0;
    pub const particleEffectRule__hitParticleEffectAlias: usize = 0;
    pub const particleEffectRule__scrapeParticleEffectAlias: usize = 8;
    pub const materialRule: usize = 0;
    pub const materialRule__numentries: usize = 0;
    pub const materialRule__entries: usize = 8;
    pub const angleRule: usize = 0;
    pub const angleRule__numentries: usize = 0;
    pub const angleRule__entries: usize = 8;
    pub const speedRule: usize = 0;
    pub const speedRule__numentries: usize = 0;
    pub const speedRule__entries: usize = 8;
    pub const massRule: usize = 0;
    pub const massRule__numentries: usize = 0;
    pub const massRule__entries: usize = 8;
    pub const gravityRule: usize = 0;
    pub const gravityRule__numentries: usize = 0;
    pub const gravityRule__entries: usize = 8;
    pub const atmosphereRule: usize = 0;
    pub const atmosphereRule__numentries: usize = 0;
    pub const atmosphereRule__entries: usize = 8;
    pub const cameraDistanceRule: usize = 0;
    pub const cameraDistanceRule__numentries: usize = 0;
    pub const cameraDistanceRule__entries: usize = 8;
    pub const frequencyRule: usize = 0;
    pub const frequencyRule__numentries: usize = 0;
    pub const frequencyRule__entries: usize = 8;
}
pub mod PhysicsVFXEventAssetRules {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const numrules: usize = 0;
    pub const rules: usize = 8;
}
pub mod PhysicsVFXEventAssetSpeedEntry {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const speedThreshold: usize = 0;
    pub const rules: usize = 8;
    pub const rules__numrules: usize = 8;
    pub const rules__rules: usize = 16;
}
pub mod PhysicsVFXEventAssetSpeedRule {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const numentries: usize = 0;
    pub const entries: usize = 8;
}
pub mod PicMip {
    pub const SIZE: usize = 2;
    pub const ALIGN: usize = 1;
    pub const platform: usize = 0;
}
pub mod PlayerAnimEntry {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 4;
    pub const entryName: usize = 0;
    pub const animName: usize = 4;
    pub const shadowAnimName: usize = 8;
    pub const __pad0: usize = 12;
}
pub mod PlayerAnimScript {
    pub const SIZE: usize = 176;
    pub const ALIGN: usize = 8;
    pub const scriptName: usize = 0;
    pub const animationCount: usize = 8;
    pub const scriptItemCount: usize = 12;
    pub const scriptTransitionCount: usize = 16;
    pub const scriptIdleTurnCount: usize = 20;
    pub const scriptIdleTwitchCount: usize = 24;
    pub const scriptAimSetCount: usize = 28;
    pub const scriptLeanSetCount: usize = 32;
    pub const unkCount: usize = 36;
    pub const torsoAnimCount: usize = 40;
    pub const legsAnimCount: usize = 44;
    pub const animations: usize = 48;
    pub const scriptAnims: usize = 56;
    pub const scriptEvents: usize = 64;
    pub const scriptItems: usize = 72;
    pub const scriptTransitions: usize = 80;
    pub const scriptIdleTurns: usize = 88;
    pub const scriptIdleTwitches: usize = 96;
    pub const scriptAimSets: usize = 104;
    pub const scriptLeanSets: usize = 112;
    pub const unk: usize = 120;
    pub const torsoAnimPackMap: usize = 128;
    pub const torsoAnimUnpackMap: usize = 136;
    pub const legsAnimPackMap: usize = 144;
    pub const legsAnimUnpackMap: usize = 152;
    pub const xAnimCount: usize = 160;
    pub const xAnims: usize = 168;
}
pub mod PlayerAnimScriptAimSet {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 2;
    pub const aim_1: usize = 0;
    pub const aim_2: usize = 2;
    pub const aim_3: usize = 4;
    pub const aim_4: usize = 6;
    pub const aim_6: usize = 8;
    pub const aim_7: usize = 10;
    pub const aim_8: usize = 12;
    pub const aim_9: usize = 14;
}
pub mod PlayerAnimScriptCommand {
    pub const SIZE: usize = 6;
    pub const ALIGN: usize = 2;
    pub const bodyPart: usize = 0;
    pub const animIndex: usize = 2;
    pub const animDuration: usize = 4;
}
pub mod PlayerAnimScriptCondition {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 4;
    pub const index: usize = 0;
    pub const value: usize = 4;
}
pub mod PlayerAnimScriptEntry {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const flags: usize = 0;
    pub const itemCount: usize = 4;
    pub const transitionCount: usize = 8;
    pub const items: usize = 16;
    pub const transitions: usize = 24;
}
pub mod PlayerAnimScriptIdleTurn {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 2;
    pub const turnleft: usize = 0;
    pub const turnright: usize = 2;
}
pub mod PlayerAnimScriptIdleTwitch {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const twitchCount: usize = 0;
    pub const cycleLength: usize = 4;
    pub const twitches: usize = 8;
}
pub mod PlayerAnimScriptItem {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const conditionCount: usize = 0;
    pub const commandCount: usize = 4;
    pub const conditions: usize = 8;
    pub const commands: usize = 16;
}
pub mod PlayerAnimScriptLeanSet {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 2;
    pub const lean_left: usize = 0;
    pub const lean_right: usize = 2;
}
pub mod PlayerAnimScriptTransition {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const triggerType: usize = 0;
    pub const triggerParam: usize = 4;
    pub const triggerParam__moveType: usize = 4;
    pub const targetMoveType: usize = 8;
}
pub mod PlayerAnimTriggerParam {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 4;
    pub const moveType: usize = 0;
}
pub mod PlayerBreathStateDef {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const stateType: usize = 0;
    pub const inhaleAlias: usize = 8;
    pub const exhaleAlias: usize = 16;
    pub const minBreaths: usize = 24;
}
pub mod RawFile {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const compressedLen: usize = 8;
    pub const len: usize = 12;
    pub const buffer: usize = 16;
}
pub mod ReactiveMotionModelPart {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const center: usize = 0;
    pub const stiffness: usize = 12;
}
pub mod ReactiveMotionModelTweaks {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const scale: usize = 0;
}
pub mod ReticleDef {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const widgetName: usize = 8;
    pub const lockReticleToTag: usize = 16;
    pub const primaryAffectedByEMP: usize = 17;
    pub const secondaryAffectedByEMP: usize = 18;
    pub const scaleWithFOV: usize = 19;
    pub const naturalDistance: usize = 20;
    pub const minReticleScale: usize = 24;
    pub const maxReticleScale: usize = 28;
}
pub mod ReverbDef {
    pub const SIZE: usize = 84;
    pub const ALIGN: usize = 4;
    pub const id: usize = 0;
    pub const roomTypeId: usize = 4;
    pub const roomType: usize = 8;
    pub const dryLevel: usize = 72;
    pub const wetLevel: usize = 76;
    pub const fadeTime: usize = 80;
}
pub mod RumbleGraph {
    pub const SIZE: usize = 144;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const knots: usize = 8;
    pub const knotCount: usize = 136;
}
pub mod RumbleInfo {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const duration: usize = 8;
    pub const range: usize = 12;
    pub const highRumbleGraph: usize = 16;
    pub const lowRumbleGraph: usize = 24;
    pub const fadeWithDistance: usize = 32;
    pub const broadcast: usize = 33;
}
pub mod SFXOverride {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const attachment1: usize = 0;
    pub const attachment2: usize = 4;
    pub const overrides: usize = 8;
    pub const overridesAlt: usize = 16;
}
pub mod SHProbeSimplexData0 {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 1;
    pub const a: usize = 0;
    pub const a____pad0: usize = 0;
    pub const b: usize = 24;
    pub const b____pad0: usize = 24;
}
pub mod SHProbeSimplexData1 {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 1;
    pub const __pad0: usize = 0;
}
pub mod SHProbeSimplexData2 {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 1;
    pub const __pad0: usize = 0;
}
pub mod SHProbeSimplexDataUnion {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const data: usize = 0;
    pub const data0: usize = 0;
    pub const data1: usize = 0;
    pub const data2: usize = 0;
}
pub mod ScriptFile {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const compressedLen: usize = 8;
    pub const len: usize = 12;
    pub const bytecodeLen: usize = 16;
    pub const buffer: usize = 24;
    pub const bytecode: usize = 32;
}
pub mod ScriptStringList {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const count: usize = 0;
    pub const strings: usize = 8;
}
pub mod ScriptableDamageTagOverride {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const tag: usize = 0;
    pub const scrTag: usize = 8;
}
pub mod ScriptableDef {
    pub const SIZE: usize = 112;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const flags: usize = 8;
    pub const r#type: usize = 12;
    pub const nextScriptableDef: usize = 16;
    pub const numParts: usize = 24;
    pub const parts: usize = 32;
    pub const maxNumDynEntsRequired: usize = 40;
    pub const partCount: usize = 44;
    pub const serverInstancedPartCount: usize = 48;
    pub const serverControlledPartCount: usize = 52;
    pub const maxNumDynEntPartsBase: usize = 56;
    pub const maxNumDynEntPartsForSpawning: usize = 60;
    pub const eventStreamSizeRequiredServer: usize = 64;
    pub const eventStreamSizeRequiredClient: usize = 66;
    pub const eventStreamSize: usize = 68;
    pub const ffMemCost: usize = 72;
    pub const animationTreeName: usize = 76;
    pub const animationTreeDef: usize = 80;
    pub const numXModels: usize = 96;
    pub const models: usize = 104;
}
pub mod ScriptableEventAddModelDef {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const tagName: usize = 8;
    pub const scrTagName: usize = 16;
    pub const model: usize = 24;
}
pub mod ScriptableEventAnimationDef {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const animation: usize = 8;
    pub const stateful: usize = 16;
    pub const overrideAnimation: usize = 17;
    pub const pauseOnExit: usize = 18;
    pub const clientOnlyInMP: usize = 19;
    pub const startTimeMin: usize = 20;
    pub const startTimeMax: usize = 24;
    pub const startTimeNormalized: usize = 28;
    pub const playbackRateMin: usize = 32;
    pub const playbackRateMax: usize = 36;
    pub const blendTime: usize = 40;
    pub const eventAtEndCount: usize = 44;
    pub const eventsAtEnd: usize = 48;
    pub const eventStreamBufferOffsetClient: usize = 56;
    pub const eventStreamBufferOffsetServer: usize = 58;
    pub const animationIndex: usize = 64;
}
pub mod ScriptableEventAnonymousDef {
    pub const SIZE: usize = 152;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const buffer: usize = 8;
}
pub mod ScriptableEventApplyForceDef {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const forcePos: usize = 8;
    pub const forceVector: usize = 20;
}
pub mod ScriptableEventBaseDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const flags: usize = 8;
}
pub mod ScriptableEventChunkDynentDef {
    pub const SIZE: usize = 88;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const partReference: usize = 8;
    pub const partReference__base: usize = 8;
    pub const partReference__base__name: usize = 8;
    pub const partReference__base__flags: usize = 16;
    pub const partReference__base____pad0: usize = 20;
    pub const partReference__type: usize = 24;
    pub const partReference__u: usize = 32;
    pub const partReference__u____0: usize = 32;
    pub const partReference__u____0__base: usize = 32;
    pub const partReference__u____1: usize = 32;
    pub const partReference__u____1__base: usize = 32;
    pub const partReference__u____2: usize = 32;
    pub const partReference__u____2__base: usize = 32;
    pub const partReference__u____2____pad0: usize = 40;
    pub const partReference__u____3: usize = 32;
    pub const partReference__u____3__base: usize = 32;
    pub const partReference__u____3__count: usize = 40;
    pub const partReference__u____3__val: usize = 48;
    pub const launchLinVel: usize = 56;
    pub const launchAngVel: usize = 68;
    pub const part: usize = 80;
}
pub mod ScriptableEventClientViewSelectorDef {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const event1pCount: usize = 8;
    pub const events1p: usize = 16;
    pub const event3pCount: usize = 24;
    pub const events3p: usize = 32;
    pub const eventStreamBufferOffsetClient: usize = 40;
}
pub mod ScriptableEventCollisionDef {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const collmapName: usize = 8;
    pub const aiObstacle: usize = 16;
    pub const clipmapCModelIndex: usize = 20;
    pub const neverMoves: usize = 24;
    pub const canPush: usize = 25;
    pub const canTouch: usize = 26;
    pub const destroyOnExit: usize = 27;
}
pub mod ScriptableEventCompassIconDef {
    pub const SIZE: usize = 152;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const stateful: usize = 8;
    pub const friendlyCompassIconAlwaysVisible: usize = 9;
    pub const enemyCompassIconAlwaysVisible: usize = 10;
    pub const friendlyArrow: usize = 16;
    pub const friendlyFiring: usize = 24;
    pub const friendlyChatting: usize = 32;
    pub const friendlyYelling: usize = 40;
    pub const partyArrow: usize = 48;
    pub const partyFiring: usize = 56;
    pub const partyChatting: usize = 64;
    pub const partyYelling: usize = 72;
    pub const squadArrow: usize = 80;
    pub const squadFiring: usize = 88;
    pub const enemyCompassIconQuiet: usize = 96;
    pub const enemyCompassIconFiring: usize = 120;
    pub const enemyCompassIconDirectional: usize = 144;
}
pub mod ScriptableEventDef {
    pub const SIZE: usize = 176;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const base__name: usize = 0;
    pub const base__flags: usize = 8;
    pub const r#type: usize = 16;
    pub const data: usize = 24;
    pub const data__anonymous: usize = 24;
    pub const data__anonymous__base: usize = 24;
    pub const data__anonymous__buffer: usize = 32;
    pub const data__stateChange: usize = 24;
    pub const data__stateChange__base: usize = 24;
    pub const data__stateChange__partReference: usize = 32;
    pub const data__stateChange__partReference__base: usize = 32;
    pub const data__stateChange__partReference__base__name: usize = 32;
    pub const data__stateChange__partReference__base__flags: usize = 40;
    pub const data__stateChange__partReference__base____pad0: usize = 44;
    pub const data__stateChange__partReference__type: usize = 48;
    pub const data__stateChange__partReference__u: usize = 56;
    pub const data__stateChange__partReference__u____0: usize = 56;
    pub const data__stateChange__partReference__u____0__base: usize = 56;
    pub const data__stateChange__partReference__u____1: usize = 56;
    pub const data__stateChange__partReference__u____1__base: usize = 56;
    pub const data__stateChange__partReference__u____2: usize = 56;
    pub const data__stateChange__partReference__u____2__base: usize = 56;
    pub const data__stateChange__partReference__u____2____pad0: usize = 64;
    pub const data__stateChange__partReference__u____3: usize = 56;
    pub const data__stateChange__partReference__u____3__base: usize = 56;
    pub const data__stateChange__partReference__u____3__count: usize = 64;
    pub const data__stateChange__partReference__u____3__val: usize = 72;
    pub const data__stateChange__part: usize = 80;
    pub const data__wait: usize = 24;
    pub const data__wait__base: usize = 24;
    pub const data__wait__delayMin: usize = 32;
    pub const data__wait__delayMax: usize = 36;
    pub const data__wait__eventStreamBufferOffsetServer: usize = 40;
    pub const data__wait__eventStreamBufferOffsetClient: usize = 42;
    pub const data__random: usize = 24;
    pub const data__random__base: usize = 24;
    pub const data__random__threshold: usize = 32;
    pub const data__random__eventACount: usize = 36;
    pub const data__random__eventsA: usize = 40;
    pub const data__random__eventBCount: usize = 48;
    pub const data__random__eventsB: usize = 56;
    pub const data__random__eventStreamBufferOffsetServer: usize = 64;
    pub const data__random__eventStreamBufferOffsetClient: usize = 66;
    pub const data__script: usize = 24;
    pub const data__script__base: usize = 24;
    pub const data__script__notification: usize = 32;
    pub const data__script__scrNotification: usize = 40;
    pub const data__script__param: usize = 44;
    pub const data__model: usize = 24;
    pub const data__model__base: usize = 24;
    pub const data__model__model: usize = 32;
    pub const data__model__hudOutlineColor: usize = 40;
    pub const data__model__hudOutlineActive: usize = 41;
    pub const data__model__hudOutlineFill: usize = 42;
    pub const data__model__neverMoves: usize = 43;
    pub const data__model__dynamicSimulation: usize = 44;
    pub const data__model__activatePhysics: usize = 45;
    pub const data__collision: usize = 24;
    pub const data__collision__base: usize = 24;
    pub const data__collision__collmapName: usize = 32;
    pub const data__collision__aiObstacle: usize = 40;
    pub const data__collision__clipmapCModelIndex: usize = 44;
    pub const data__collision__neverMoves: usize = 48;
    pub const data__collision__canPush: usize = 49;
    pub const data__collision__canTouch: usize = 50;
    pub const data__collision__destroyOnExit: usize = 51;
    pub const data__animation: usize = 24;
    pub const data__animation__base: usize = 24;
    pub const data__animation__animation: usize = 32;
    pub const data__animation__stateful: usize = 40;
    pub const data__animation__overrideAnimation: usize = 41;
    pub const data__animation__pauseOnExit: usize = 42;
    pub const data__animation__clientOnlyInMP: usize = 43;
    pub const data__animation__startTimeMin: usize = 44;
    pub const data__animation__startTimeMax: usize = 48;
    pub const data__animation__startTimeNormalized: usize = 52;
    pub const data__animation__playbackRateMin: usize = 56;
    pub const data__animation__playbackRateMax: usize = 60;
    pub const data__animation__blendTime: usize = 64;
    pub const data__animation__eventAtEndCount: usize = 68;
    pub const data__animation__eventsAtEnd: usize = 72;
    pub const data__animation__eventStreamBufferOffsetClient: usize = 80;
    pub const data__animation__eventStreamBufferOffsetServer: usize = 82;
    pub const data__animation__animationIndex: usize = 88;
    pub const data__hideShowBone: usize = 24;
    pub const data__hideShowBone__base: usize = 24;
    pub const data__hideShowBone__tagName: usize = 32;
    pub const data__hideShowBone__scrTagName: usize = 40;
    pub const data__hideShowBone__hide: usize = 44;
    pub const data__hideShowBone__allowMissingTag: usize = 45;
    pub const data__hideShowBone__hideShowChildren: usize = 46;
    pub const data__noteTrack: usize = 24;
    pub const data__noteTrack__base: usize = 24;
    pub const data__noteTrack__noteTrackCount: usize = 32;
    pub const data__noteTrack__noteTracks: usize = 40;
    pub const data__chunkDynent: usize = 24;
    pub const data__chunkDynent__base: usize = 24;
    pub const data__chunkDynent__partReference: usize = 32;
    pub const data__chunkDynent__partReference__base: usize = 32;
    pub const data__chunkDynent__partReference__base__name: usize = 32;
    pub const data__chunkDynent__partReference__base__flags: usize = 40;
    pub const data__chunkDynent__partReference__base____pad0: usize = 44;
    pub const data__chunkDynent__partReference__type: usize = 48;
    pub const data__chunkDynent__partReference__u: usize = 56;
    pub const data__chunkDynent__partReference__u____0: usize = 56;
    pub const data__chunkDynent__partReference__u____0__base: usize = 56;
    pub const data__chunkDynent__partReference__u____1: usize = 56;
    pub const data__chunkDynent__partReference__u____1__base: usize = 56;
    pub const data__chunkDynent__partReference__u____2: usize = 56;
    pub const data__chunkDynent__partReference__u____2__base: usize = 56;
    pub const data__chunkDynent__partReference__u____2____pad0: usize = 64;
    pub const data__chunkDynent__partReference__u____3: usize = 56;
    pub const data__chunkDynent__partReference__u____3__base: usize = 56;
    pub const data__chunkDynent__partReference__u____3__count: usize = 64;
    pub const data__chunkDynent__partReference__u____3__val: usize = 72;
    pub const data__chunkDynent__launchLinVel: usize = 80;
    pub const data__chunkDynent__launchAngVel: usize = 92;
    pub const data__chunkDynent__part: usize = 104;
    pub const data__spawnDynent: usize = 24;
    pub const data__spawnDynent__base: usize = 24;
    pub const data__spawnDynent__stateful: usize = 32;
    pub const data__spawnDynent__model: usize = 40;
    pub const data__spawnDynent__tagName: usize = 48;
    pub const data__spawnDynent__scrTagName: usize = 56;
    pub const data__spawnDynent__allowCauseVelocity: usize = 60;
    pub const data__spawnDynent__allowNearestVehicleVelocity: usize = 61;
    pub const data__spawnDynent__worldSpaceLinVel: usize = 62;
    pub const data__spawnDynent__worldSpaceAngVel: usize = 63;
    pub const data__spawnDynent__randomRangeLinVel: usize = 64;
    pub const data__spawnDynent__randomRangeAngVel: usize = 65;
    pub const data__spawnDynent__launchLinVel: usize = 68;
    pub const data__spawnDynent__launchAngVel: usize = 80;
    pub const data__spawnDynent__eventStreamBufferOffsetClient: usize = 92;
    pub const data__spawnDynent__allowMissingTag: usize = 94;
    pub const data__spawnDynent__useRootOnMissingTag: usize = 95;
    pub const data__particleFX: usize = 24;
    pub const data__particleFX__base: usize = 24;
    pub const data__particleFX__stateful: usize = 32;
    pub const data__particleFX__scrTagNames: usize = 40;
    pub const data__particleFX__scrTagCount: usize = 48;
    pub const data__particleFX__scrEndTagNames: usize = 56;
    pub const data__particleFX__scrEndTagCount: usize = 64;
    pub const data__particleFX__useTagAngles: usize = 68;
    pub const data__particleFX__effectAlias: usize = 72;
    pub const data__particleFX__effectDef: usize = 80;
    pub const data__particleFX__effectDef__u: usize = 80;
    pub const data__particleFX__effectDef__u__fx: usize = 80;
    pub const data__particleFX__effectDef__u__vfx: usize = 80;
    pub const data__particleFX__effectDef__u__data: usize = 80;
    pub const data__particleFX__effectDef__type: usize = 88;
    pub const data__particleFX__loopRate: usize = 96;
    pub const data__particleFX__oneshotLooping: usize = 100;
    pub const data__particleFX__eventStreamBufferOffsetClient: usize = 102;
    pub const data__particleFX__allowMissingTag: usize = 104;
    pub const data__particleFX__useRootOnMissingTag: usize = 105;
    pub const data__particleFX__allowNearestVehicleVelocity: usize = 106;
    pub const data__particleFX__isViewmodel: usize = 107;
    pub const data__particleFX__killOnExit: usize = 108;
    pub const data__particleFX__useDynamicScriptedBeamLength: usize = 109;
    pub const data__particleFX__beamBoneAxis: usize = 112;
    pub const data__particleFX__beamLength: usize = 116;
    pub const data__particleFX____pad0: usize = 120;
    pub const data__sound: usize = 24;
    pub const data__sound__base: usize = 24;
    pub const data__sound__stateful: usize = 32;
    pub const data__sound____pad0: usize = 33;
    pub const data__sound__tagName: usize = 40;
    pub const data__sound__scrTagName: usize = 48;
    pub const data__sound__soundAlias: usize = 56;
    pub const data__sound__soundAliasCache: usize = 64;
    pub const data__sound____pad1: usize = 72;
    pub const data__explosion: usize = 24;
    pub const data__explosion__base: usize = 24;
    pub const data__explosion__weapon: usize = 32;
    pub const data__explosion__tagName: usize = 40;
    pub const data__explosion__scrTagName: usize = 48;
    pub const data__explosion__radius: usize = 52;
    pub const data__explosion__dmgOuter: usize = 56;
    pub const data__explosion__dmgInner: usize = 60;
    pub const data__explosion__stateful: usize = 64;
    pub const data__explosion__allowMissingTag: usize = 65;
    pub const data__explosion__useRootOnMissingTag: usize = 66;
    pub const data__light: usize = 24;
    pub const data__light__base: usize = 24;
    pub const data__light__name: usize = 32;
    pub const data__light__scrName: usize = 40;
    pub const data__light__intensityScaleMin: usize = 44;
    pub const data__light__intensityScaleMax: usize = 48;
    pub const data__light__radiusMin: usize = 52;
    pub const data__light__radiusMax: usize = 56;
    pub const data__light__useColor: usize = 60;
    pub const data__light__colorLinearSrgb: usize = 64;
    pub const data__light__useStateTransitionTime: usize = 80;
    pub const data__light__transitionMin: usize = 84;
    pub const data__light__transitionMax: usize = 88;
    pub const data__light__eventStreamBufferOffsetClient: usize = 92;
    pub const data__sun: usize = 24;
    pub const data__sun__base: usize = 24;
    pub const data__sun__useIntensity: usize = 32;
    pub const data__sun__intensityOverrideMin: usize = 36;
    pub const data__sun__intensityOverrideMax: usize = 40;
    pub const data__sun__useColor: usize = 44;
    pub const data__sun__colorLinearSrgb: usize = 48;
    pub const data__sun__useStateTransitionTime: usize = 64;
    pub const data__sun__transitionMin: usize = 68;
    pub const data__sun__transitionMax: usize = 72;
    pub const data__sun__pitchMin: usize = 76;
    pub const data__sun__pitchMax: usize = 80;
    pub const data__sun__headingMin: usize = 84;
    pub const data__sun__headingMax: usize = 88;
    pub const data__sun__useDirection: usize = 92;
    pub const data__rumble: usize = 24;
    pub const data__rumble__base: usize = 24;
    pub const data__rumble__stateful: usize = 32;
    pub const data__rumble__tagName: usize = 40;
    pub const data__rumble__scrTagName: usize = 48;
    pub const data__rumble__rumble: usize = 56;
    pub const data__rumble__rumbleAsset: usize = 64;
    pub const data__rumble__allowMissingTag: usize = 72;
    pub const data__rumble__useRootOnMissingTag: usize = 73;
    pub const data__screenshake: usize = 24;
    pub const data__screenshake__base: usize = 24;
    pub const data__screenshake__stateful: usize = 32;
    pub const data__screenshake__isEarthquake: usize = 33;
    pub const data__screenshake__isScreenShake: usize = 34;
    pub const data__screenshake__tagName: usize = 40;
    pub const data__screenshake__scrTagName: usize = 48;
    pub const data__screenshake__duration: usize = 52;
    pub const data__screenshake__durationFadeUp: usize = 56;
    pub const data__screenshake__durationFadeDown: usize = 60;
    pub const data__screenshake__radius: usize = 64;
    pub const data__screenshake__scaleEarthquake: usize = 68;
    pub const data__screenshake__scalePitch: usize = 72;
    pub const data__screenshake__scaleYaw: usize = 76;
    pub const data__screenshake__scaleRoll: usize = 80;
    pub const data__screenshake__frequencyPitch: usize = 84;
    pub const data__screenshake__frequencyYaw: usize = 88;
    pub const data__screenshake__frequencyRoll: usize = 92;
    pub const data__screenshake__allowMissingTag: usize = 96;
    pub const data__screenshake__useRootOnMissingTag: usize = 97;
    pub const data__partDamage: usize = 24;
    pub const data__partDamage__base: usize = 24;
    pub const data__partDamage__partReference: usize = 32;
    pub const data__partDamage__partReference__base: usize = 32;
    pub const data__partDamage__partReference__base__name: usize = 32;
    pub const data__partDamage__partReference__base__flags: usize = 40;
    pub const data__partDamage__partReference__base____pad0: usize = 44;
    pub const data__partDamage__partReference__type: usize = 48;
    pub const data__partDamage__partReference__u: usize = 56;
    pub const data__partDamage__partReference__u____0: usize = 56;
    pub const data__partDamage__partReference__u____0__base: usize = 56;
    pub const data__partDamage__partReference__u____1: usize = 56;
    pub const data__partDamage__partReference__u____1__base: usize = 56;
    pub const data__partDamage__partReference__u____2: usize = 56;
    pub const data__partDamage__partReference__u____2__base: usize = 56;
    pub const data__partDamage__partReference__u____2____pad0: usize = 64;
    pub const data__partDamage__partReference__u____3: usize = 56;
    pub const data__partDamage__partReference__u____3__base: usize = 56;
    pub const data__partDamage__partReference__u____3__count: usize = 64;
    pub const data__partDamage__partReference__u____3__val: usize = 72;
    pub const data__partDamage__amount: usize = 80;
    pub const data__partDamage__amountDoT: usize = 84;
    pub const data__partDamage__intervalDoT: usize = 88;
    pub const data__partDamage__eventStreamBufferOffsetServer: usize = 92;
    pub const data__partDamage__eventStreamBufferOffsetClient: usize = 94;
    pub const data__partDamage__part: usize = 96;
    pub const data__setMayhem: usize = 24;
    pub const data__setMayhem__base: usize = 24;
    pub const data__setMayhem__mayhem: usize = 32;
    pub const data__setMayhem__eventStreamBufferOffsetClient: usize = 40;
    pub const data__playMayhem: usize = 24;
    pub const data__playMayhem__base: usize = 24;
    pub const data__playMayhem__action: usize = 32;
    pub const data__playMayhem__eventStreamBufferOffsetClient: usize = 36;
    pub const data__viewmodelShaderParam: usize = 24;
    pub const data__viewmodelShaderParam__base: usize = 24;
    pub const data__viewmodelShaderParam__shaderParam: usize = 32;
    pub const data__viewmodelShaderParam__floatValue: usize = 36;
    pub const data__viewmodelShaderParam__placeHolderValue1: usize = 40;
    pub const data__viewmodelShaderParam__placeHolderValue2: usize = 44;
    pub const data__viewmodelShaderParam__placeHolderValue3: usize = 48;
    pub const data__viewmodelShaderParam__placeHolderValue4: usize = 52;
    pub const data__viewmodelShaderParam__transitionTime: usize = 56;
    pub const data__viewmodelShaderParam__eventStreamBufferOffsetClient: usize = 60;
    pub const data__viewmodelChangeImage: usize = 24;
    pub const data__viewmodelChangeImage__base: usize = 24;
    pub const data__viewmodelChangeImage__imageId: usize = 32;
    pub const data__viewmodelChangeImage__imageValue: usize = 36;
    pub const data__clientViewSelector: usize = 24;
    pub const data__clientViewSelector__base: usize = 24;
    pub const data__clientViewSelector__event1pCount: usize = 32;
    pub const data__clientViewSelector__events1p: usize = 40;
    pub const data__clientViewSelector__event3pCount: usize = 48;
    pub const data__clientViewSelector__events3p: usize = 56;
    pub const data__clientViewSelector__eventStreamBufferOffsetClient: usize = 64;
    pub const data__teamSelector: usize = 24;
    pub const data__teamSelector__base: usize = 24;
    pub const data__teamSelector__teamFilter: usize = 32;
    pub const data__teamSelector__testLocalPlayer: usize = 36;
    pub const data__teamSelector__eventPassCount: usize = 40;
    pub const data__teamSelector__eventsPass: usize = 48;
    pub const data__teamSelector__eventFailCount: usize = 56;
    pub const data__teamSelector__eventsFail: usize = 64;
    pub const data__teamSelector__eventStreamBufferOffsetClient: usize = 72;
    pub const data__teamSelector__eventStreamBufferOffsetServer: usize = 74;
    pub const data__addModel: usize = 24;
    pub const data__addModel__base: usize = 24;
    pub const data__addModel__tagName: usize = 32;
    pub const data__addModel__scrTagName: usize = 40;
    pub const data__addModel__model: usize = 48;
    pub const data__applyForce: usize = 24;
    pub const data__applyForce__base: usize = 24;
    pub const data__applyForce__forcePos: usize = 32;
    pub const data__applyForce__forceVector: usize = 44;
    pub const data__compassIcon: usize = 24;
    pub const data__compassIcon__base: usize = 24;
    pub const data__compassIcon__stateful: usize = 32;
    pub const data__compassIcon__friendlyCompassIconAlwaysVisible: usize = 33;
    pub const data__compassIcon__enemyCompassIconAlwaysVisible: usize = 34;
    pub const data__compassIcon__friendlyArrow: usize = 40;
    pub const data__compassIcon__friendlyFiring: usize = 48;
    pub const data__compassIcon__friendlyChatting: usize = 56;
    pub const data__compassIcon__friendlyYelling: usize = 64;
    pub const data__compassIcon__partyArrow: usize = 72;
    pub const data__compassIcon__partyFiring: usize = 80;
    pub const data__compassIcon__partyChatting: usize = 88;
    pub const data__compassIcon__partyYelling: usize = 96;
    pub const data__compassIcon__squadArrow: usize = 104;
    pub const data__compassIcon__squadFiring: usize = 112;
    pub const data__compassIcon__enemyCompassIconQuiet: usize = 120;
    pub const data__compassIcon__enemyCompassIconFiring: usize = 144;
    pub const data__compassIcon__enemyCompassIconDirectional: usize = 168;
    pub const data__materialOverride: usize = 24;
    pub const data__materialOverride__base: usize = 24;
    pub const data__materialOverride__materialOverrideType: usize = 32;
    pub const data__materialOverride____pad0: usize = 36;
    pub const data__materialOverride__material: usize = 40;
    pub const data__materialOverride__transitionTime: usize = 48;
    pub const data__materialOverride__finalValue: usize = 52;
    pub const data__materialOverride__eventStreamBufferOffsetClient: usize = 54;
    pub const data__materialOverride____pad1: usize = 56;
}
pub mod ScriptableEventDefUnion {
    pub const SIZE: usize = 152;
    pub const ALIGN: usize = 8;
    pub const anonymous: usize = 0;
    pub const anonymous__base: usize = 0;
    pub const anonymous__buffer: usize = 8;
    pub const stateChange: usize = 0;
    pub const stateChange__base: usize = 0;
    pub const stateChange__partReference: usize = 8;
    pub const stateChange__partReference__base: usize = 8;
    pub const stateChange__partReference__base__name: usize = 8;
    pub const stateChange__partReference__base__flags: usize = 16;
    pub const stateChange__partReference__base____pad0: usize = 20;
    pub const stateChange__partReference__type: usize = 24;
    pub const stateChange__partReference__u: usize = 32;
    pub const stateChange__partReference__u____0: usize = 32;
    pub const stateChange__partReference__u____0__base: usize = 32;
    pub const stateChange__partReference__u____1: usize = 32;
    pub const stateChange__partReference__u____1__base: usize = 32;
    pub const stateChange__partReference__u____2: usize = 32;
    pub const stateChange__partReference__u____2__base: usize = 32;
    pub const stateChange__partReference__u____2____pad0: usize = 40;
    pub const stateChange__partReference__u____3: usize = 32;
    pub const stateChange__partReference__u____3__base: usize = 32;
    pub const stateChange__partReference__u____3__count: usize = 40;
    pub const stateChange__partReference__u____3__val: usize = 48;
    pub const stateChange__part: usize = 56;
    pub const wait: usize = 0;
    pub const wait__base: usize = 0;
    pub const wait__delayMin: usize = 8;
    pub const wait__delayMax: usize = 12;
    pub const wait__eventStreamBufferOffsetServer: usize = 16;
    pub const wait__eventStreamBufferOffsetClient: usize = 18;
    pub const random: usize = 0;
    pub const random__base: usize = 0;
    pub const random__threshold: usize = 8;
    pub const random__eventACount: usize = 12;
    pub const random__eventsA: usize = 16;
    pub const random__eventBCount: usize = 24;
    pub const random__eventsB: usize = 32;
    pub const random__eventStreamBufferOffsetServer: usize = 40;
    pub const random__eventStreamBufferOffsetClient: usize = 42;
    pub const script: usize = 0;
    pub const script__base: usize = 0;
    pub const script__notification: usize = 8;
    pub const script__scrNotification: usize = 16;
    pub const script__param: usize = 20;
    pub const model: usize = 0;
    pub const model__base: usize = 0;
    pub const model__model: usize = 8;
    pub const model__hudOutlineColor: usize = 16;
    pub const model__hudOutlineActive: usize = 17;
    pub const model__hudOutlineFill: usize = 18;
    pub const model__neverMoves: usize = 19;
    pub const model__dynamicSimulation: usize = 20;
    pub const model__activatePhysics: usize = 21;
    pub const collision: usize = 0;
    pub const collision__base: usize = 0;
    pub const collision__collmapName: usize = 8;
    pub const collision__aiObstacle: usize = 16;
    pub const collision__clipmapCModelIndex: usize = 20;
    pub const collision__neverMoves: usize = 24;
    pub const collision__canPush: usize = 25;
    pub const collision__canTouch: usize = 26;
    pub const collision__destroyOnExit: usize = 27;
    pub const animation: usize = 0;
    pub const animation__base: usize = 0;
    pub const animation__animation: usize = 8;
    pub const animation__stateful: usize = 16;
    pub const animation__overrideAnimation: usize = 17;
    pub const animation__pauseOnExit: usize = 18;
    pub const animation__clientOnlyInMP: usize = 19;
    pub const animation__startTimeMin: usize = 20;
    pub const animation__startTimeMax: usize = 24;
    pub const animation__startTimeNormalized: usize = 28;
    pub const animation__playbackRateMin: usize = 32;
    pub const animation__playbackRateMax: usize = 36;
    pub const animation__blendTime: usize = 40;
    pub const animation__eventAtEndCount: usize = 44;
    pub const animation__eventsAtEnd: usize = 48;
    pub const animation__eventStreamBufferOffsetClient: usize = 56;
    pub const animation__eventStreamBufferOffsetServer: usize = 58;
    pub const animation__animationIndex: usize = 64;
    pub const hideShowBone: usize = 0;
    pub const hideShowBone__base: usize = 0;
    pub const hideShowBone__tagName: usize = 8;
    pub const hideShowBone__scrTagName: usize = 16;
    pub const hideShowBone__hide: usize = 20;
    pub const hideShowBone__allowMissingTag: usize = 21;
    pub const hideShowBone__hideShowChildren: usize = 22;
    pub const noteTrack: usize = 0;
    pub const noteTrack__base: usize = 0;
    pub const noteTrack__noteTrackCount: usize = 8;
    pub const noteTrack__noteTracks: usize = 16;
    pub const chunkDynent: usize = 0;
    pub const chunkDynent__base: usize = 0;
    pub const chunkDynent__partReference: usize = 8;
    pub const chunkDynent__partReference__base: usize = 8;
    pub const chunkDynent__partReference__base__name: usize = 8;
    pub const chunkDynent__partReference__base__flags: usize = 16;
    pub const chunkDynent__partReference__base____pad0: usize = 20;
    pub const chunkDynent__partReference__type: usize = 24;
    pub const chunkDynent__partReference__u: usize = 32;
    pub const chunkDynent__partReference__u____0: usize = 32;
    pub const chunkDynent__partReference__u____0__base: usize = 32;
    pub const chunkDynent__partReference__u____1: usize = 32;
    pub const chunkDynent__partReference__u____1__base: usize = 32;
    pub const chunkDynent__partReference__u____2: usize = 32;
    pub const chunkDynent__partReference__u____2__base: usize = 32;
    pub const chunkDynent__partReference__u____2____pad0: usize = 40;
    pub const chunkDynent__partReference__u____3: usize = 32;
    pub const chunkDynent__partReference__u____3__base: usize = 32;
    pub const chunkDynent__partReference__u____3__count: usize = 40;
    pub const chunkDynent__partReference__u____3__val: usize = 48;
    pub const chunkDynent__launchLinVel: usize = 56;
    pub const chunkDynent__launchAngVel: usize = 68;
    pub const chunkDynent__part: usize = 80;
    pub const spawnDynent: usize = 0;
    pub const spawnDynent__base: usize = 0;
    pub const spawnDynent__stateful: usize = 8;
    pub const spawnDynent__model: usize = 16;
    pub const spawnDynent__tagName: usize = 24;
    pub const spawnDynent__scrTagName: usize = 32;
    pub const spawnDynent__allowCauseVelocity: usize = 36;
    pub const spawnDynent__allowNearestVehicleVelocity: usize = 37;
    pub const spawnDynent__worldSpaceLinVel: usize = 38;
    pub const spawnDynent__worldSpaceAngVel: usize = 39;
    pub const spawnDynent__randomRangeLinVel: usize = 40;
    pub const spawnDynent__randomRangeAngVel: usize = 41;
    pub const spawnDynent__launchLinVel: usize = 44;
    pub const spawnDynent__launchAngVel: usize = 56;
    pub const spawnDynent__eventStreamBufferOffsetClient: usize = 68;
    pub const spawnDynent__allowMissingTag: usize = 70;
    pub const spawnDynent__useRootOnMissingTag: usize = 71;
    pub const particleFX: usize = 0;
    pub const particleFX__base: usize = 0;
    pub const particleFX__stateful: usize = 8;
    pub const particleFX__scrTagNames: usize = 16;
    pub const particleFX__scrTagCount: usize = 24;
    pub const particleFX__scrEndTagNames: usize = 32;
    pub const particleFX__scrEndTagCount: usize = 40;
    pub const particleFX__useTagAngles: usize = 44;
    pub const particleFX__effectAlias: usize = 48;
    pub const particleFX__effectDef: usize = 56;
    pub const particleFX__effectDef__u: usize = 56;
    pub const particleFX__effectDef__u__fx: usize = 56;
    pub const particleFX__effectDef__u__vfx: usize = 56;
    pub const particleFX__effectDef__u__data: usize = 56;
    pub const particleFX__effectDef__type: usize = 64;
    pub const particleFX__loopRate: usize = 72;
    pub const particleFX__oneshotLooping: usize = 76;
    pub const particleFX__eventStreamBufferOffsetClient: usize = 78;
    pub const particleFX__allowMissingTag: usize = 80;
    pub const particleFX__useRootOnMissingTag: usize = 81;
    pub const particleFX__allowNearestVehicleVelocity: usize = 82;
    pub const particleFX__isViewmodel: usize = 83;
    pub const particleFX__killOnExit: usize = 84;
    pub const particleFX__useDynamicScriptedBeamLength: usize = 85;
    pub const particleFX__beamBoneAxis: usize = 88;
    pub const particleFX__beamLength: usize = 92;
    pub const particleFX____pad0: usize = 96;
    pub const sound: usize = 0;
    pub const sound__base: usize = 0;
    pub const sound__stateful: usize = 8;
    pub const sound____pad0: usize = 9;
    pub const sound__tagName: usize = 16;
    pub const sound__scrTagName: usize = 24;
    pub const sound__soundAlias: usize = 32;
    pub const sound__soundAliasCache: usize = 40;
    pub const sound____pad1: usize = 48;
    pub const explosion: usize = 0;
    pub const explosion__base: usize = 0;
    pub const explosion__weapon: usize = 8;
    pub const explosion__tagName: usize = 16;
    pub const explosion__scrTagName: usize = 24;
    pub const explosion__radius: usize = 28;
    pub const explosion__dmgOuter: usize = 32;
    pub const explosion__dmgInner: usize = 36;
    pub const explosion__stateful: usize = 40;
    pub const explosion__allowMissingTag: usize = 41;
    pub const explosion__useRootOnMissingTag: usize = 42;
    pub const light: usize = 0;
    pub const light__base: usize = 0;
    pub const light__name: usize = 8;
    pub const light__scrName: usize = 16;
    pub const light__intensityScaleMin: usize = 20;
    pub const light__intensityScaleMax: usize = 24;
    pub const light__radiusMin: usize = 28;
    pub const light__radiusMax: usize = 32;
    pub const light__useColor: usize = 36;
    pub const light__colorLinearSrgb: usize = 40;
    pub const light__useStateTransitionTime: usize = 56;
    pub const light__transitionMin: usize = 60;
    pub const light__transitionMax: usize = 64;
    pub const light__eventStreamBufferOffsetClient: usize = 68;
    pub const sun: usize = 0;
    pub const sun__base: usize = 0;
    pub const sun__useIntensity: usize = 8;
    pub const sun__intensityOverrideMin: usize = 12;
    pub const sun__intensityOverrideMax: usize = 16;
    pub const sun__useColor: usize = 20;
    pub const sun__colorLinearSrgb: usize = 24;
    pub const sun__useStateTransitionTime: usize = 40;
    pub const sun__transitionMin: usize = 44;
    pub const sun__transitionMax: usize = 48;
    pub const sun__pitchMin: usize = 52;
    pub const sun__pitchMax: usize = 56;
    pub const sun__headingMin: usize = 60;
    pub const sun__headingMax: usize = 64;
    pub const sun__useDirection: usize = 68;
    pub const rumble: usize = 0;
    pub const rumble__base: usize = 0;
    pub const rumble__stateful: usize = 8;
    pub const rumble__tagName: usize = 16;
    pub const rumble__scrTagName: usize = 24;
    pub const rumble__rumble: usize = 32;
    pub const rumble__rumbleAsset: usize = 40;
    pub const rumble__allowMissingTag: usize = 48;
    pub const rumble__useRootOnMissingTag: usize = 49;
    pub const screenshake: usize = 0;
    pub const screenshake__base: usize = 0;
    pub const screenshake__stateful: usize = 8;
    pub const screenshake__isEarthquake: usize = 9;
    pub const screenshake__isScreenShake: usize = 10;
    pub const screenshake__tagName: usize = 16;
    pub const screenshake__scrTagName: usize = 24;
    pub const screenshake__duration: usize = 28;
    pub const screenshake__durationFadeUp: usize = 32;
    pub const screenshake__durationFadeDown: usize = 36;
    pub const screenshake__radius: usize = 40;
    pub const screenshake__scaleEarthquake: usize = 44;
    pub const screenshake__scalePitch: usize = 48;
    pub const screenshake__scaleYaw: usize = 52;
    pub const screenshake__scaleRoll: usize = 56;
    pub const screenshake__frequencyPitch: usize = 60;
    pub const screenshake__frequencyYaw: usize = 64;
    pub const screenshake__frequencyRoll: usize = 68;
    pub const screenshake__allowMissingTag: usize = 72;
    pub const screenshake__useRootOnMissingTag: usize = 73;
    pub const partDamage: usize = 0;
    pub const partDamage__base: usize = 0;
    pub const partDamage__partReference: usize = 8;
    pub const partDamage__partReference__base: usize = 8;
    pub const partDamage__partReference__base__name: usize = 8;
    pub const partDamage__partReference__base__flags: usize = 16;
    pub const partDamage__partReference__base____pad0: usize = 20;
    pub const partDamage__partReference__type: usize = 24;
    pub const partDamage__partReference__u: usize = 32;
    pub const partDamage__partReference__u____0: usize = 32;
    pub const partDamage__partReference__u____0__base: usize = 32;
    pub const partDamage__partReference__u____1: usize = 32;
    pub const partDamage__partReference__u____1__base: usize = 32;
    pub const partDamage__partReference__u____2: usize = 32;
    pub const partDamage__partReference__u____2__base: usize = 32;
    pub const partDamage__partReference__u____2____pad0: usize = 40;
    pub const partDamage__partReference__u____3: usize = 32;
    pub const partDamage__partReference__u____3__base: usize = 32;
    pub const partDamage__partReference__u____3__count: usize = 40;
    pub const partDamage__partReference__u____3__val: usize = 48;
    pub const partDamage__amount: usize = 56;
    pub const partDamage__amountDoT: usize = 60;
    pub const partDamage__intervalDoT: usize = 64;
    pub const partDamage__eventStreamBufferOffsetServer: usize = 68;
    pub const partDamage__eventStreamBufferOffsetClient: usize = 70;
    pub const partDamage__part: usize = 72;
    pub const setMayhem: usize = 0;
    pub const setMayhem__base: usize = 0;
    pub const setMayhem__mayhem: usize = 8;
    pub const setMayhem__eventStreamBufferOffsetClient: usize = 16;
    pub const playMayhem: usize = 0;
    pub const playMayhem__base: usize = 0;
    pub const playMayhem__action: usize = 8;
    pub const playMayhem__eventStreamBufferOffsetClient: usize = 12;
    pub const viewmodelShaderParam: usize = 0;
    pub const viewmodelShaderParam__base: usize = 0;
    pub const viewmodelShaderParam__shaderParam: usize = 8;
    pub const viewmodelShaderParam__floatValue: usize = 12;
    pub const viewmodelShaderParam__placeHolderValue1: usize = 16;
    pub const viewmodelShaderParam__placeHolderValue2: usize = 20;
    pub const viewmodelShaderParam__placeHolderValue3: usize = 24;
    pub const viewmodelShaderParam__placeHolderValue4: usize = 28;
    pub const viewmodelShaderParam__transitionTime: usize = 32;
    pub const viewmodelShaderParam__eventStreamBufferOffsetClient: usize = 36;
    pub const viewmodelChangeImage: usize = 0;
    pub const viewmodelChangeImage__base: usize = 0;
    pub const viewmodelChangeImage__imageId: usize = 8;
    pub const viewmodelChangeImage__imageValue: usize = 12;
    pub const clientViewSelector: usize = 0;
    pub const clientViewSelector__base: usize = 0;
    pub const clientViewSelector__event1pCount: usize = 8;
    pub const clientViewSelector__events1p: usize = 16;
    pub const clientViewSelector__event3pCount: usize = 24;
    pub const clientViewSelector__events3p: usize = 32;
    pub const clientViewSelector__eventStreamBufferOffsetClient: usize = 40;
    pub const teamSelector: usize = 0;
    pub const teamSelector__base: usize = 0;
    pub const teamSelector__teamFilter: usize = 8;
    pub const teamSelector__testLocalPlayer: usize = 12;
    pub const teamSelector__eventPassCount: usize = 16;
    pub const teamSelector__eventsPass: usize = 24;
    pub const teamSelector__eventFailCount: usize = 32;
    pub const teamSelector__eventsFail: usize = 40;
    pub const teamSelector__eventStreamBufferOffsetClient: usize = 48;
    pub const teamSelector__eventStreamBufferOffsetServer: usize = 50;
    pub const addModel: usize = 0;
    pub const addModel__base: usize = 0;
    pub const addModel__tagName: usize = 8;
    pub const addModel__scrTagName: usize = 16;
    pub const addModel__model: usize = 24;
    pub const applyForce: usize = 0;
    pub const applyForce__base: usize = 0;
    pub const applyForce__forcePos: usize = 8;
    pub const applyForce__forceVector: usize = 20;
    pub const compassIcon: usize = 0;
    pub const compassIcon__base: usize = 0;
    pub const compassIcon__stateful: usize = 8;
    pub const compassIcon__friendlyCompassIconAlwaysVisible: usize = 9;
    pub const compassIcon__enemyCompassIconAlwaysVisible: usize = 10;
    pub const compassIcon__friendlyArrow: usize = 16;
    pub const compassIcon__friendlyFiring: usize = 24;
    pub const compassIcon__friendlyChatting: usize = 32;
    pub const compassIcon__friendlyYelling: usize = 40;
    pub const compassIcon__partyArrow: usize = 48;
    pub const compassIcon__partyFiring: usize = 56;
    pub const compassIcon__partyChatting: usize = 64;
    pub const compassIcon__partyYelling: usize = 72;
    pub const compassIcon__squadArrow: usize = 80;
    pub const compassIcon__squadFiring: usize = 88;
    pub const compassIcon__enemyCompassIconQuiet: usize = 96;
    pub const compassIcon__enemyCompassIconFiring: usize = 120;
    pub const compassIcon__enemyCompassIconDirectional: usize = 144;
    pub const materialOverride: usize = 0;
    pub const materialOverride__base: usize = 0;
    pub const materialOverride__materialOverrideType: usize = 8;
    pub const materialOverride____pad0: usize = 12;
    pub const materialOverride__material: usize = 16;
    pub const materialOverride__transitionTime: usize = 24;
    pub const materialOverride__finalValue: usize = 28;
    pub const materialOverride__eventStreamBufferOffsetClient: usize = 30;
    pub const materialOverride____pad1: usize = 32;
}
pub mod ScriptableEventExplosionDef {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const weapon: usize = 8;
    pub const tagName: usize = 16;
    pub const scrTagName: usize = 24;
    pub const radius: usize = 28;
    pub const dmgOuter: usize = 32;
    pub const dmgInner: usize = 36;
    pub const stateful: usize = 40;
    pub const allowMissingTag: usize = 41;
    pub const useRootOnMissingTag: usize = 42;
}
pub mod ScriptableEventHideShowBoneDef {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const tagName: usize = 8;
    pub const scrTagName: usize = 16;
    pub const hide: usize = 20;
    pub const allowMissingTag: usize = 21;
    pub const hideShowChildren: usize = 22;
}
pub mod ScriptableEventLightDef {
    pub const SIZE: usize = 72;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const name: usize = 8;
    pub const scrName: usize = 16;
    pub const intensityScaleMin: usize = 20;
    pub const intensityScaleMax: usize = 24;
    pub const radiusMin: usize = 28;
    pub const radiusMax: usize = 32;
    pub const useColor: usize = 36;
    pub const colorLinearSrgb: usize = 40;
    pub const useStateTransitionTime: usize = 56;
    pub const transitionMin: usize = 60;
    pub const transitionMax: usize = 64;
    pub const eventStreamBufferOffsetClient: usize = 68;
}
pub mod ScriptableEventMaterialOverrideDef {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const materialOverrideType: usize = 8;
    pub const __pad0: usize = 12;
    pub const material: usize = 16;
    pub const transitionTime: usize = 24;
    pub const finalValue: usize = 28;
    pub const eventStreamBufferOffsetClient: usize = 30;
    pub const __pad1: usize = 32;
}
pub mod ScriptableEventModelDef {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const model: usize = 8;
    pub const hudOutlineColor: usize = 16;
    pub const hudOutlineActive: usize = 17;
    pub const hudOutlineFill: usize = 18;
    pub const neverMoves: usize = 19;
    pub const dynamicSimulation: usize = 20;
    pub const activatePhysics: usize = 21;
}
pub mod ScriptableEventNoteTrackDef {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const noteTrackCount: usize = 8;
    pub const noteTracks: usize = 16;
}
pub mod ScriptableEventPFXDef {
    pub const SIZE: usize = 104;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const stateful: usize = 8;
    pub const scrTagNames: usize = 16;
    pub const scrTagCount: usize = 24;
    pub const scrEndTagNames: usize = 32;
    pub const scrEndTagCount: usize = 40;
    pub const useTagAngles: usize = 44;
    pub const effectAlias: usize = 48;
    pub const effectDef: usize = 56;
    pub const effectDef__u: usize = 56;
    pub const effectDef__u__fx: usize = 56;
    pub const effectDef__u__vfx: usize = 56;
    pub const effectDef__u__data: usize = 56;
    pub const effectDef__type: usize = 64;
    pub const loopRate: usize = 72;
    pub const oneshotLooping: usize = 76;
    pub const eventStreamBufferOffsetClient: usize = 78;
    pub const allowMissingTag: usize = 80;
    pub const useRootOnMissingTag: usize = 81;
    pub const allowNearestVehicleVelocity: usize = 82;
    pub const isViewmodel: usize = 83;
    pub const killOnExit: usize = 84;
    pub const useDynamicScriptedBeamLength: usize = 85;
    pub const beamBoneAxis: usize = 88;
    pub const beamLength: usize = 92;
    pub const __pad0: usize = 96;
}
pub mod ScriptableEventPartDamageDef {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const partReference: usize = 8;
    pub const partReference__base: usize = 8;
    pub const partReference__base__name: usize = 8;
    pub const partReference__base__flags: usize = 16;
    pub const partReference__base____pad0: usize = 20;
    pub const partReference__type: usize = 24;
    pub const partReference__u: usize = 32;
    pub const partReference__u____0: usize = 32;
    pub const partReference__u____0__base: usize = 32;
    pub const partReference__u____1: usize = 32;
    pub const partReference__u____1__base: usize = 32;
    pub const partReference__u____2: usize = 32;
    pub const partReference__u____2__base: usize = 32;
    pub const partReference__u____2____pad0: usize = 40;
    pub const partReference__u____3: usize = 32;
    pub const partReference__u____3__base: usize = 32;
    pub const partReference__u____3__count: usize = 40;
    pub const partReference__u____3__val: usize = 48;
    pub const amount: usize = 56;
    pub const amountDoT: usize = 60;
    pub const intervalDoT: usize = 64;
    pub const eventStreamBufferOffsetServer: usize = 68;
    pub const eventStreamBufferOffsetClient: usize = 70;
    pub const part: usize = 72;
}
pub mod ScriptableEventPlayMayhemDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const action: usize = 8;
    pub const eventStreamBufferOffsetClient: usize = 12;
}
pub mod ScriptableEventRandomDef {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const threshold: usize = 8;
    pub const eventACount: usize = 12;
    pub const eventsA: usize = 16;
    pub const eventBCount: usize = 24;
    pub const eventsB: usize = 32;
    pub const eventStreamBufferOffsetServer: usize = 40;
    pub const eventStreamBufferOffsetClient: usize = 42;
}
pub mod ScriptableEventRumbleDef {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const stateful: usize = 8;
    pub const tagName: usize = 16;
    pub const scrTagName: usize = 24;
    pub const rumble: usize = 32;
    pub const rumbleAsset: usize = 40;
    pub const allowMissingTag: usize = 48;
    pub const useRootOnMissingTag: usize = 49;
}
pub mod ScriptableEventScreenshakeDef {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const stateful: usize = 8;
    pub const isEarthquake: usize = 9;
    pub const isScreenShake: usize = 10;
    pub const tagName: usize = 16;
    pub const scrTagName: usize = 24;
    pub const duration: usize = 28;
    pub const durationFadeUp: usize = 32;
    pub const durationFadeDown: usize = 36;
    pub const radius: usize = 40;
    pub const scaleEarthquake: usize = 44;
    pub const scalePitch: usize = 48;
    pub const scaleYaw: usize = 52;
    pub const scaleRoll: usize = 56;
    pub const frequencyPitch: usize = 60;
    pub const frequencyYaw: usize = 64;
    pub const frequencyRoll: usize = 68;
    pub const allowMissingTag: usize = 72;
    pub const useRootOnMissingTag: usize = 73;
}
pub mod ScriptableEventScriptDef {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const notification: usize = 8;
    pub const scrNotification: usize = 16;
    pub const param: usize = 20;
}
pub mod ScriptableEventSetMayhemDef {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const mayhem: usize = 8;
    pub const eventStreamBufferOffsetClient: usize = 16;
}
pub mod ScriptableEventSoundDef {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const stateful: usize = 8;
    pub const __pad0: usize = 9;
    pub const tagName: usize = 16;
    pub const scrTagName: usize = 24;
    pub const soundAlias: usize = 32;
    pub const soundAliasCache: usize = 40;
    pub const __pad1: usize = 48;
}
pub mod ScriptableEventSpawnDynentDef {
    pub const SIZE: usize = 72;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const stateful: usize = 8;
    pub const model: usize = 16;
    pub const tagName: usize = 24;
    pub const scrTagName: usize = 32;
    pub const allowCauseVelocity: usize = 36;
    pub const allowNearestVehicleVelocity: usize = 37;
    pub const worldSpaceLinVel: usize = 38;
    pub const worldSpaceAngVel: usize = 39;
    pub const randomRangeLinVel: usize = 40;
    pub const randomRangeAngVel: usize = 41;
    pub const launchLinVel: usize = 44;
    pub const launchAngVel: usize = 56;
    pub const eventStreamBufferOffsetClient: usize = 68;
    pub const allowMissingTag: usize = 70;
    pub const useRootOnMissingTag: usize = 71;
}
pub mod ScriptableEventStateChangeDef {
    pub const SIZE: usize = 64;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const partReference: usize = 8;
    pub const partReference__base: usize = 8;
    pub const partReference__base__name: usize = 8;
    pub const partReference__base__flags: usize = 16;
    pub const partReference__base____pad0: usize = 20;
    pub const partReference__type: usize = 24;
    pub const partReference__u: usize = 32;
    pub const partReference__u____0: usize = 32;
    pub const partReference__u____0__base: usize = 32;
    pub const partReference__u____1: usize = 32;
    pub const partReference__u____1__base: usize = 32;
    pub const partReference__u____2: usize = 32;
    pub const partReference__u____2__base: usize = 32;
    pub const partReference__u____2____pad0: usize = 40;
    pub const partReference__u____3: usize = 32;
    pub const partReference__u____3__base: usize = 32;
    pub const partReference__u____3__count: usize = 40;
    pub const partReference__u____3__val: usize = 48;
    pub const part: usize = 56;
}
pub mod ScriptableEventSunDef {
    pub const SIZE: usize = 72;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const useIntensity: usize = 8;
    pub const intensityOverrideMin: usize = 12;
    pub const intensityOverrideMax: usize = 16;
    pub const useColor: usize = 20;
    pub const colorLinearSrgb: usize = 24;
    pub const useStateTransitionTime: usize = 40;
    pub const transitionMin: usize = 44;
    pub const transitionMax: usize = 48;
    pub const pitchMin: usize = 52;
    pub const pitchMax: usize = 56;
    pub const headingMin: usize = 60;
    pub const headingMax: usize = 64;
    pub const useDirection: usize = 68;
}
pub mod ScriptableEventTeamSelectorDef {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const teamFilter: usize = 8;
    pub const testLocalPlayer: usize = 12;
    pub const eventPassCount: usize = 16;
    pub const eventsPass: usize = 24;
    pub const eventFailCount: usize = 32;
    pub const eventsFail: usize = 40;
    pub const eventStreamBufferOffsetClient: usize = 48;
    pub const eventStreamBufferOffsetServer: usize = 50;
}
pub mod ScriptableEventViewmodelChangeImageDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const imageId: usize = 8;
    pub const imageValue: usize = 12;
}
pub mod ScriptableEventViewmodelShaderParamDef {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const shaderParam: usize = 8;
    pub const floatValue: usize = 12;
    pub const placeHolderValue1: usize = 16;
    pub const placeHolderValue2: usize = 20;
    pub const placeHolderValue3: usize = 24;
    pub const placeHolderValue4: usize = 28;
    pub const transitionTime: usize = 32;
    pub const eventStreamBufferOffsetClient: usize = 36;
}
pub mod ScriptableEventWaitDef {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const delayMin: usize = 8;
    pub const delayMax: usize = 12;
    pub const eventStreamBufferOffsetServer: usize = 16;
    pub const eventStreamBufferOffsetClient: usize = 18;
}
pub mod ScriptableInstance {
    pub const SIZE: usize = 464;
    pub const ALIGN: usize = 8;
    pub const contextHeader: usize = 0;
    pub const contextHeader__context: usize = 0;
    pub const contextHeader__context__def: usize = 0;
    pub const contextHeader__context__initialOrigin: usize = 8;
    pub const contextHeader__context__initialAngles: usize = 20;
    pub const contextHeader__context__origin: usize = 32;
    pub const contextHeader__context__angles: usize = 44;
    pub const contextHeader__context__partStateIndex: usize = 56;
    pub const contextHeader__context__entityId: usize = 60;
    pub const contextHeader__context__padding1: usize = 62;
    pub const contextHeader__context__unk04: usize = 64;
    pub const contextHeader__context__needsInitialization: usize = 68;
    pub const contextHeader__context__needsShutdown: usize = 69;
    pub const contextHeader__context__isInitialized: usize = 70;
    pub const contextHeader__context__padding2: usize = 71;
    pub const contextHeader__context__modelData: usize = 72;
    pub const contextHeader__context__modelData__model: usize = 72;
    pub const contextHeader__context__modelData__neverMoves: usize = 80;
    pub const contextHeader__context__modelData__dynamicSimulation: usize = 81;
    pub const contextHeader__context__modelData__padding: usize = 82;
    pub const contextHeader__context__eventStreamBufferSize: usize = 88;
    pub const contextHeader__context__eventStreamBuffer: usize = 96;
    pub const contextHeader__context__reservedStateListHeader: usize = 104;
    pub const contextHeader__unk02: usize = 112;
    pub const contextHeaderLocalClient: usize = 128;
    pub const targetname: usize = 432;
    pub const flags: usize = 436;
    pub const __pad0: usize = 440;
    pub const debugName: usize = 448;
    pub const debugNameScr: usize = 456;
}
pub mod ScriptableInstanceContext {
    pub const SIZE: usize = 112;
    pub const ALIGN: usize = 8;
    pub const def: usize = 0;
    pub const initialOrigin: usize = 8;
    pub const initialAngles: usize = 20;
    pub const origin: usize = 32;
    pub const angles: usize = 44;
    pub const partStateIndex: usize = 56;
    pub const entityId: usize = 60;
    pub const padding1: usize = 62;
    pub const unk04: usize = 64;
    pub const needsInitialization: usize = 68;
    pub const needsShutdown: usize = 69;
    pub const isInitialized: usize = 70;
    pub const padding2: usize = 71;
    pub const modelData: usize = 72;
    pub const modelData__model: usize = 72;
    pub const modelData__neverMoves: usize = 80;
    pub const modelData__dynamicSimulation: usize = 81;
    pub const modelData__padding: usize = 82;
    pub const eventStreamBufferSize: usize = 88;
    pub const eventStreamBuffer: usize = 96;
    pub const reservedStateListHeader: usize = 104;
}
pub mod ScriptableInstanceContextHeader {
    pub const SIZE: usize = 128;
    pub const ALIGN: usize = 8;
    pub const context: usize = 0;
    pub const context__def: usize = 0;
    pub const context__initialOrigin: usize = 8;
    pub const context__initialAngles: usize = 20;
    pub const context__origin: usize = 32;
    pub const context__angles: usize = 44;
    pub const context__partStateIndex: usize = 56;
    pub const context__entityId: usize = 60;
    pub const context__padding1: usize = 62;
    pub const context__unk04: usize = 64;
    pub const context__needsInitialization: usize = 68;
    pub const context__needsShutdown: usize = 69;
    pub const context__isInitialized: usize = 70;
    pub const context__padding2: usize = 71;
    pub const context__modelData: usize = 72;
    pub const context__modelData__model: usize = 72;
    pub const context__modelData__neverMoves: usize = 80;
    pub const context__modelData__dynamicSimulation: usize = 81;
    pub const context__modelData__padding: usize = 82;
    pub const context__eventStreamBufferSize: usize = 88;
    pub const context__eventStreamBuffer: usize = 96;
    pub const context__reservedStateListHeader: usize = 104;
    pub const unk02: usize = 112;
}
pub mod ScriptableInstanceContextHeaderLocalClient {
    pub const SIZE: usize = 152;
    pub const ALIGN: usize = 8;
    pub const context: usize = 0;
    pub const context__def: usize = 0;
    pub const context__initialOrigin: usize = 8;
    pub const context__initialAngles: usize = 20;
    pub const context__origin: usize = 32;
    pub const context__angles: usize = 44;
    pub const context__partStateIndex: usize = 56;
    pub const context__entityId: usize = 60;
    pub const context__padding1: usize = 62;
    pub const context__unk04: usize = 64;
    pub const context__needsInitialization: usize = 68;
    pub const context__needsShutdown: usize = 69;
    pub const context__isInitialized: usize = 70;
    pub const context__padding2: usize = 71;
    pub const context__modelData: usize = 72;
    pub const context__modelData__model: usize = 72;
    pub const context__modelData__neverMoves: usize = 80;
    pub const context__modelData__dynamicSimulation: usize = 81;
    pub const context__modelData__padding: usize = 82;
    pub const context__eventStreamBufferSize: usize = 88;
    pub const context__eventStreamBuffer: usize = 96;
    pub const context__reservedStateListHeader: usize = 104;
    pub const unk02: usize = 112;
}
pub mod ScriptableMapEnts {
    pub const SIZE: usize = 208;
    pub const ALIGN: usize = 8;
    pub const totalInstanceCount: usize = 0;
    pub const runtimeInstanceCount: usize = 4;
    pub const reservedInstanceCount: usize = 8;
    pub const pad: usize = 12;
    pub const instances: usize = 16;
    pub const runtimeData: usize = 24;
    pub const runtimeData__partRuntimeCount: usize = 24;
    pub const runtimeData__partRuntime: usize = 32;
    pub const runtimeData__partRuntimeLocalClientCount: usize = 40;
    pub const runtimeData__partRuntimeLocalClient: usize = 48;
    pub const sunClientDatas: usize = 64;
    pub const reservedDynents: usize = 168;
    pub const ffMemCost: usize = 200;
}
pub mod ScriptableModelData {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const model: usize = 0;
    pub const neverMoves: usize = 8;
    pub const dynamicSimulation: usize = 9;
    pub const padding: usize = 10;
}
pub mod ScriptableNoteTrackDef {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const flags: usize = 0;
    pub const noteTrackName: usize = 8;
    pub const scrNoteTrackName: usize = 16;
    pub const numEvents: usize = 20;
    pub const events: usize = 24;
}
pub mod ScriptablePartDef {
    pub const SIZE: usize = 104;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const scrName: usize = 8;
    pub const flags: usize = 12;
    pub const flatId: usize = 16;
    pub const serverInstanceFlatId: usize = 20;
    pub const serverControlledFlatId: usize = 24;
    pub const tagName: usize = 32;
    pub const scrTagName: usize = 40;
    pub const numStates: usize = 44;
    pub const states: usize = 48;
    pub const parentPart: usize = 56;
    pub const numChildParts: usize = 64;
    pub const childParts: usize = 72;
    pub const numDamageTagOverrides: usize = 80;
    pub const damageTagOverrides: usize = 88;
    pub const eventStreamBufferOffsetServer: usize = 96;
    pub const eventStreamBufferOffsetClient: usize = 98;
    pub const eventStreamSize: usize = 100;
}
pub mod ScriptablePartReference {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const base__name: usize = 0;
    pub const base__flags: usize = 8;
    pub const base____pad0: usize = 12;
    pub const r#type: usize = 16;
    pub const u: usize = 24;
    pub const u____0: usize = 24;
    pub const u____0__base: usize = 24;
    pub const u____1: usize = 24;
    pub const u____1__base: usize = 24;
    pub const u____2: usize = 24;
    pub const u____2__base: usize = 24;
    pub const u____2____pad0: usize = 32;
    pub const u____3: usize = 24;
    pub const u____3__base: usize = 24;
    pub const u____3__count: usize = 32;
    pub const u____3__val: usize = 40;
}
pub mod ScriptablePartRuntime {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 2;
    pub const stateId: usize = 0;
    pub const nextfree: usize = 2;
}
pub mod ScriptablePartRuntimeData {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const partRuntimeCount: usize = 0;
    pub const partRuntime: usize = 8;
    pub const partRuntimeLocalClientCount: usize = 16;
    pub const partRuntimeLocalClient: usize = 24;
}
pub mod ScriptableReservedDynent {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const dynentId: usize = 0;
    pub const next: usize = 4;
}
pub mod ScriptableReservedDynents {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const freeHead: usize = 0;
    pub const numReservedDynents: usize = 4;
    pub const reservedDynents: usize = 8;
}
pub mod ScriptableStateBaseDef {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const flags: usize = 8;
    pub const numEvents: usize = 12;
    pub const events: usize = 16;
}
pub mod ScriptableStateDef {
    pub const SIZE: usize = 88;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const base__name: usize = 0;
    pub const base__flags: usize = 8;
    pub const base__numEvents: usize = 12;
    pub const base__events: usize = 16;
    pub const r#type: usize = 24;
    pub const data: usize = 32;
    pub const data__simple: usize = 32;
    pub const data__simple__base: usize = 32;
    pub const data__health: usize = 32;
    pub const data__health__base: usize = 32;
    pub const data__health__health: usize = 40;
    pub const data__health__validDamageCause: usize = 44;
    pub const data__health__minimumDamage: usize = 48;
    pub const data__health__damagePropagationToParent: usize = 52;
    pub const data__health__damagePropagationToChild: usize = 56;
    pub const data__health__damagePropagationFromParent: usize = 60;
    pub const data__health__damagePropagationFromChild: usize = 64;
    pub const data__health__script_id: usize = 72;
    pub const data__health__scrScript_id: usize = 80;
    pub const data__health__localizeSplashDamage: usize = 84;
    pub const data__scripted: usize = 32;
    pub const data__scripted__base: usize = 32;
    pub const data__scripted__script_id: usize = 40;
    pub const data__scripted__scrScript_id: usize = 48;
}
pub mod ScriptableStateDefUnion {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const simple: usize = 0;
    pub const simple__base: usize = 0;
    pub const health: usize = 0;
    pub const health__base: usize = 0;
    pub const health__health: usize = 8;
    pub const health__validDamageCause: usize = 12;
    pub const health__minimumDamage: usize = 16;
    pub const health__damagePropagationToParent: usize = 20;
    pub const health__damagePropagationToChild: usize = 24;
    pub const health__damagePropagationFromParent: usize = 28;
    pub const health__damagePropagationFromChild: usize = 32;
    pub const health__script_id: usize = 40;
    pub const health__scrScript_id: usize = 48;
    pub const health__localizeSplashDamage: usize = 52;
    pub const scripted: usize = 0;
    pub const scripted__base: usize = 0;
    pub const scripted__script_id: usize = 8;
    pub const scripted__scrScript_id: usize = 16;
}
pub mod ScriptableStateHealthDef {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const health: usize = 8;
    pub const validDamageCause: usize = 12;
    pub const minimumDamage: usize = 16;
    pub const damagePropagationToParent: usize = 20;
    pub const damagePropagationToChild: usize = 24;
    pub const damagePropagationFromParent: usize = 28;
    pub const damagePropagationFromChild: usize = 32;
    pub const script_id: usize = 40;
    pub const scrScript_id: usize = 48;
    pub const localizeSplashDamage: usize = 52;
}
pub mod ScriptableStateScriptedDef {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const script_id: usize = 8;
    pub const scrScript_id: usize = 16;
}
pub mod ScriptableStateSimpleDef {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
}
pub mod Scriptable_EventSun_Data {
    pub const SIZE: usize = 52;
    pub const ALIGN: usize = 4;
    pub const startTime: usize = 0;
    pub const transTime: usize = 4;
    pub const startColorLinearSrgb: usize = 8;
    pub const startIntensity: usize = 20;
    pub const targetIntensity: usize = 24;
    pub const startAngles: usize = 28;
    pub const targetAngles: usize = 40;
}
pub mod SentientHandle {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 2;
    pub const number: usize = 0;
    pub const infoIndex: usize = 2;
}
pub mod SndAlias {
    pub const SIZE: usize = 200;
    pub const ALIGN: usize = 8;
    pub const aliasName: usize = 0;
    pub const subtitle: usize = 8;
    pub const secondaryAliasName: usize = 16;
    pub const stopAliasName: usize = 24;
    pub const assetFileName: usize = 32;
    pub const id: usize = 40;
    pub const secondaryId: usize = 44;
    pub const stopAliasID: usize = 48;
    pub const assetId: usize = 52;
    pub const contextType: usize = 56;
    pub const contextValue: usize = 60;
    pub const duck: usize = 64;
    pub const sequence: usize = 68;
    pub const volMin: usize = 72;
    pub const volMax: usize = 76;
    pub const volModIndex: usize = 80;
    pub const pitchMin: usize = 84;
    pub const pitchMax: usize = 88;
    pub const donutFadeEnd: usize = 92;
    pub const distMin: usize = 96;
    pub const distMax: usize = 100;
    pub const velocityMin: usize = 104;
    pub const flags: usize = 108;
    pub const flags__looping: usize = 108;
    pub const flags__reverb: usize = 108;
    pub const flags__shape: usize = 108;
    pub const flags__type: usize = 108;
    pub const flags__channel: usize = 109;
    pub const flags__unused: usize = 110;
    pub const flags__ignoreDistanceCheck: usize = 110;
    pub const flags__precached: usize = 110;
    pub const flags__MusicContent: usize = 110;
    pub const flags__GPadOutput: usize = 110;
    pub const flags__ForceSubtitle: usize = 111;
    pub const flags__pad: usize = 111;
    pub const masterPriority: usize = 112;
    pub const masterPercentage: usize = 116;
    pub const slavePercentage: usize = 120;
    pub const probability: usize = 124;
    pub const lfePercentage: usize = 128;
    pub const centerPercentage: usize = 132;
    pub const startDelay: usize = 136;
    pub const volumeFalloffCurveIndex: usize = 140;
    pub const lpfCurveIndex: usize = 141;
    pub const hpfCurveIndex: usize = 142;
    pub const reverbSendCurveIndex: usize = 143;
    pub const envelopMin: usize = 144;
    pub const envelopMax: usize = 148;
    pub const envelopPercentage: usize = 152;
    pub const speakerMapIndex: usize = 156;
    pub const pad1: usize = 157;
    pub const reverbMultiplier: usize = 160;
    pub const farReverbMultiplier: usize = 164;
    pub const occlusionShapeIndex: usize = 168;
    pub const dopplerPresetIndex: usize = 169;
    pub const pad2: usize = 170;
    pub const smartPanDistance2d: usize = 172;
    pub const smartPanDistance3d: usize = 176;
    pub const smartPanAttenuation2d: usize = 180;
    pub const stereoSpreadMinDist: usize = 184;
    pub const stereoSpreadMaxDist: usize = 188;
    pub const stereoSpreadMidPoint: usize = 192;
    pub const stereoSpreadMaxAngle: usize = 196;
}
pub mod SndAliasFlags {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 4;
    pub const looping: usize = 0;
    pub const reverb: usize = 0;
    pub const shape: usize = 0;
    pub const r#type: usize = 0;
    pub const channel: usize = 1;
    pub const unused: usize = 2;
    pub const ignoreDistanceCheck: usize = 2;
    pub const precached: usize = 2;
    pub const MusicContent: usize = 2;
    pub const GPadOutput: usize = 2;
    pub const ForceSubtitle: usize = 3;
    pub const pad: usize = 3;
}
pub mod SndAliasList {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const aliasName: usize = 0;
    pub const id: usize = 8;
    pub const head: usize = 16;
    pub const count: usize = 24;
    pub const sequence: usize = 28;
}
pub mod SndAssetBankEntry {
    pub const SIZE: usize = 44;
    pub const ALIGN: usize = 1;
    pub const id: usize = 0;
    pub const size: usize = 4;
    pub const seekTableSize: usize = 8;
    pub const frameCount: usize = 12;
    pub const hybridPcmSize: usize = 16;
    pub const offset: usize = 20;
    pub const frameRate: usize = 28;
    pub const channelCount: usize = 32;
    pub const looping: usize = 33;
    pub const format: usize = 34;
    pub const EnvelopeTime1: usize = 35;
    pub const EnvelopeTime2: usize = 37;
    pub const EnvelopeLoudness0: usize = 39;
    pub const EnvelopeLoudness1: usize = 40;
    pub const EnvelopeLoudness2: usize = 41;
    pub const EnvelopeLoudness3: usize = 42;
    pub const BlocksizeKB: usize = 43;
}
pub mod SndAssetBankHeader {
    pub const SIZE: usize = 688;
    pub const ALIGN: usize = 1;
    pub const magic: usize = 0;
    pub const version: usize = 4;
    pub const entrySize: usize = 8;
    pub const checksumSize: usize = 12;
    pub const dependencySize: usize = 16;
    pub const entryCount: usize = 20;
    pub const dependencyCount: usize = 24;
    pub const buildVersion: usize = 28;
    pub const fileSize: usize = 32;
    pub const entryOffset: usize = 40;
    pub const checksumOffset: usize = 48;
    pub const checksumChecksum: usize = 56;
    pub const dependencies: usize = 72;
    pub const SourceChecksumOffset: usize = 584;
    pub const AssetNameOffset: usize = 592;
    pub const zoneName: usize = 600;
    pub const platform: usize = 664;
    pub const language: usize = 672;
    pub const convertedAssetVersion: usize = 675;
    pub const assetSectionSize: usize = 679;
    pub const padding: usize = 683;
}
pub mod SndBank {
    pub const SIZE: usize = 352;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const zone: usize = 8;
    pub const gameLanguage: usize = 16;
    pub const soundLanguage: usize = 24;
    pub const aliasCount: usize = 32;
    pub const alias: usize = 40;
    pub const aliasIndex: usize = 48;
    pub const sendEffectCount: usize = 56;
    pub const sendEffects: usize = 64;
    pub const soundTable: usize = 72;
    pub const soundTable__id: usize = 72;
    pub const soundTable__adsrSettings: usize = 80;
    pub const soundTable__adsrCount: usize = 88;
    pub const soundTable__ambientDefs: usize = 96;
    pub const soundTable__ambientDefCount: usize = 104;
    pub const soundTable__ambientEvents: usize = 112;
    pub const soundTable__ambientEventCount: usize = 120;
    pub const soundTable__ambientElements: usize = 128;
    pub const soundTable__ambientElementsCount: usize = 136;
    pub const soundTable__filters: usize = 144;
    pub const soundTable__filterCount: usize = 152;
    pub const soundTable__mixes: usize = 160;
    pub const soundTable__mixCount: usize = 168;
    pub const soundTable__npcADSRZones: usize = 176;
    pub const soundTable__npcADSRZoneCount: usize = 184;
    pub const soundTable__occlusionFilters: usize = 192;
    pub const soundTable__occlusionFilterCount: usize = 200;
    pub const soundTable__playerADSRZones: usize = 208;
    pub const soundTable__playerADSRZoneCount: usize = 216;
    pub const soundTable__reverbs: usize = 224;
    pub const soundTable__reverbCount: usize = 232;
    pub const soundTable__timeScaleSettings: usize = 240;
    pub const soundTable__timeScaleSettingCount: usize = 248;
    pub const soundTable__zones: usize = 256;
    pub const soundTable__zoneCount: usize = 264;
    pub const soundTable__unk: usize = 272;
    pub const soundTable__unkCount: usize = 280;
    pub const soundTable__fullOcclusionDefs: usize = 288;
    pub const soundTable__fullOcclusionDefCount: usize = 296;
    pub const soundTable__plrBreathStateDefs: usize = 304;
    pub const soundTable__plrBreathStateDefCount: usize = 312;
    pub const duckCount: usize = 320;
    pub const ducks: usize = 328;
    pub const musicSetCount: usize = 336;
    pub const musicSets: usize = 344;
}
pub mod SndBankResident {
    pub const SIZE: usize = 352;
    pub const ALIGN: usize = 8;
    pub const bank: usize = 0;
    pub const bank__name: usize = 0;
    pub const bank__zone: usize = 8;
    pub const bank__gameLanguage: usize = 16;
    pub const bank__soundLanguage: usize = 24;
    pub const bank__aliasCount: usize = 32;
    pub const bank__alias: usize = 40;
    pub const bank__aliasIndex: usize = 48;
    pub const bank__sendEffectCount: usize = 56;
    pub const bank__sendEffects: usize = 64;
    pub const bank__soundTable: usize = 72;
    pub const bank__soundTable__id: usize = 72;
    pub const bank__soundTable__adsrSettings: usize = 80;
    pub const bank__soundTable__adsrCount: usize = 88;
    pub const bank__soundTable__ambientDefs: usize = 96;
    pub const bank__soundTable__ambientDefCount: usize = 104;
    pub const bank__soundTable__ambientEvents: usize = 112;
    pub const bank__soundTable__ambientEventCount: usize = 120;
    pub const bank__soundTable__ambientElements: usize = 128;
    pub const bank__soundTable__ambientElementsCount: usize = 136;
    pub const bank__soundTable__filters: usize = 144;
    pub const bank__soundTable__filterCount: usize = 152;
    pub const bank__soundTable__mixes: usize = 160;
    pub const bank__soundTable__mixCount: usize = 168;
    pub const bank__soundTable__npcADSRZones: usize = 176;
    pub const bank__soundTable__npcADSRZoneCount: usize = 184;
    pub const bank__soundTable__occlusionFilters: usize = 192;
    pub const bank__soundTable__occlusionFilterCount: usize = 200;
    pub const bank__soundTable__playerADSRZones: usize = 208;
    pub const bank__soundTable__playerADSRZoneCount: usize = 216;
    pub const bank__soundTable__reverbs: usize = 224;
    pub const bank__soundTable__reverbCount: usize = 232;
    pub const bank__soundTable__timeScaleSettings: usize = 240;
    pub const bank__soundTable__timeScaleSettingCount: usize = 248;
    pub const bank__soundTable__zones: usize = 256;
    pub const bank__soundTable__zoneCount: usize = 264;
    pub const bank__soundTable__unk: usize = 272;
    pub const bank__soundTable__unkCount: usize = 280;
    pub const bank__soundTable__fullOcclusionDefs: usize = 288;
    pub const bank__soundTable__fullOcclusionDefCount: usize = 296;
    pub const bank__soundTable__plrBreathStateDefs: usize = 304;
    pub const bank__soundTable__plrBreathStateDefCount: usize = 312;
    pub const bank__duckCount: usize = 320;
    pub const bank__ducks: usize = 328;
    pub const bank__musicSetCount: usize = 336;
    pub const bank__musicSets: usize = 344;
}
pub mod SndBankTransient {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const elementCount: usize = 8;
    pub const elements: usize = 16;
}
pub mod SndContext {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const typeId: usize = 8;
    pub const numValues: usize = 12;
    pub const startIndex: usize = 16;
    pub const allowDefaults: usize = 20;
    pub const includeDefaultsInRandom: usize = 21;
    pub const canBlend: usize = 22;
}
pub mod SndContextValue {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const value: usize = 0;
    pub const valueId: usize = 8;
}
pub mod SndCurve {
    pub const SIZE: usize = 200;
    pub const ALIGN: usize = 4;
    pub const name: usize = 0;
    pub const id: usize = 64;
    pub const numPoints: usize = 68;
    pub const points: usize = 72;
}
pub mod SndDuck {
    pub const SIZE: usize = 144;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const id: usize = 64;
    pub const fadeIn: usize = 68;
    pub const fadeOut: usize = 72;
    pub const startDelay: usize = 76;
    pub const minDistance: usize = 80;
    pub const distance: usize = 84;
    pub const length: usize = 88;
    pub const fadeInCurve: usize = 92;
    pub const fadeOutCurve: usize = 96;
    pub const attenuation: usize = 104;
    pub const lpf: usize = 112;
    pub const updateWhilePaused: usize = 120;
    pub const trackAmplitude: usize = 121;
    pub const disableInSplitscreen: usize = 122;
    pub const unused: usize = 124;
    pub const aliasAttenuation: usize = 132;
    pub const aliasLpf: usize = 136;
    pub const duckAlias: usize = 140;
}
pub mod SndDynamicsParams {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 4;
    pub const enable: usize = 0;
    pub const preGain: usize = 4;
    pub const postGain: usize = 8;
    pub const threshold: usize = 12;
    pub const ratio: usize = 16;
    pub const timeAttack: usize = 20;
    pub const timeRelease: usize = 24;
    pub const spread: usize = 28;
    pub const linear: usize = 32;
    pub const ITU1770: usize = 36;
}
pub mod SndGlobals {
    pub const SIZE: usize = 208;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const globalVolumeModifier: usize = 8;
    pub const binkVolumeModifier: usize = 12;
    pub const volmodinfoCount: usize = 16;
    pub const volmodinfo: usize = 24;
    pub const entchannelCount: usize = 32;
    pub const entchannelInfo: usize = 40;
    pub const dopplerPresetCount: usize = 48;
    pub const dopplerPresets: usize = 56;
    pub const occlusionShapeCount: usize = 64;
    pub const occlusionShapes: usize = 72;
    pub const vfCurveCount: usize = 80;
    pub const vfCurves: usize = 88;
    pub const lpfCurveCount: usize = 96;
    pub const lpfCurves: usize = 104;
    pub const hpfCurveCount: usize = 112;
    pub const hpfCurves: usize = 120;
    pub const rvbCurveCount: usize = 128;
    pub const rvbCurves: usize = 136;
    pub const speakerMapCount: usize = 144;
    pub const speakerMaps: usize = 152;
    pub const contextCount: usize = 160;
    pub const contexts: usize = 168;
    pub const contextValueCount: usize = 176;
    pub const contextValues: usize = 184;
    pub const masterCount: usize = 192;
    pub const masters: usize = 200;
}
pub mod SndIndexEntry {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 2;
    pub const value: usize = 0;
    pub const next: usize = 2;
}
pub mod SndMaster {
    pub const SIZE: usize = 500;
    pub const ALIGN: usize = 4;
    pub const name: usize = 0;
    pub const id: usize = 64;
    pub const dialogEq: usize = 68;
    pub const dialogEq__low: usize = 68;
    pub const dialogEq__low__enabled: usize = 68;
    pub const dialogEq__low__gain: usize = 72;
    pub const dialogEq__low__frequency: usize = 76;
    pub const dialogEq__low__q: usize = 80;
    pub const dialogEq__peak1: usize = 84;
    pub const dialogEq__peak1__enabled: usize = 84;
    pub const dialogEq__peak1__gain: usize = 88;
    pub const dialogEq__peak1__frequency: usize = 92;
    pub const dialogEq__peak1__q: usize = 96;
    pub const dialogEq__peak2: usize = 100;
    pub const dialogEq__peak2__enabled: usize = 100;
    pub const dialogEq__peak2__gain: usize = 104;
    pub const dialogEq__peak2__frequency: usize = 108;
    pub const dialogEq__peak2__q: usize = 112;
    pub const dialogEq__high: usize = 116;
    pub const dialogEq__high__enabled: usize = 116;
    pub const dialogEq__high__gain: usize = 120;
    pub const dialogEq__high__frequency: usize = 124;
    pub const dialogEq__high__q: usize = 128;
    pub const weapExpScenesEq: usize = 132;
    pub const weapExpScenesEq__low: usize = 132;
    pub const weapExpScenesEq__low__enabled: usize = 132;
    pub const weapExpScenesEq__low__gain: usize = 136;
    pub const weapExpScenesEq__low__frequency: usize = 140;
    pub const weapExpScenesEq__low__q: usize = 144;
    pub const weapExpScenesEq__peak1: usize = 148;
    pub const weapExpScenesEq__peak1__enabled: usize = 148;
    pub const weapExpScenesEq__peak1__gain: usize = 152;
    pub const weapExpScenesEq__peak1__frequency: usize = 156;
    pub const weapExpScenesEq__peak1__q: usize = 160;
    pub const weapExpScenesEq__peak2: usize = 164;
    pub const weapExpScenesEq__peak2__enabled: usize = 164;
    pub const weapExpScenesEq__peak2__gain: usize = 168;
    pub const weapExpScenesEq__peak2__frequency: usize = 172;
    pub const weapExpScenesEq__peak2__q: usize = 176;
    pub const weapExpScenesEq__high: usize = 180;
    pub const weapExpScenesEq__high__enabled: usize = 180;
    pub const weapExpScenesEq__high__gain: usize = 184;
    pub const weapExpScenesEq__high__frequency: usize = 188;
    pub const weapExpScenesEq__high__q: usize = 192;
    pub const sfxEq: usize = 196;
    pub const sfxEq__low: usize = 196;
    pub const sfxEq__low__enabled: usize = 196;
    pub const sfxEq__low__gain: usize = 200;
    pub const sfxEq__low__frequency: usize = 204;
    pub const sfxEq__low__q: usize = 208;
    pub const sfxEq__peak1: usize = 212;
    pub const sfxEq__peak1__enabled: usize = 212;
    pub const sfxEq__peak1__gain: usize = 216;
    pub const sfxEq__peak1__frequency: usize = 220;
    pub const sfxEq__peak1__q: usize = 224;
    pub const sfxEq__peak2: usize = 228;
    pub const sfxEq__peak2__enabled: usize = 228;
    pub const sfxEq__peak2__gain: usize = 232;
    pub const sfxEq__peak2__frequency: usize = 236;
    pub const sfxEq__peak2__q: usize = 240;
    pub const sfxEq__high: usize = 244;
    pub const sfxEq__high__enabled: usize = 244;
    pub const sfxEq__high__gain: usize = 248;
    pub const sfxEq__high__frequency: usize = 252;
    pub const sfxEq__high__q: usize = 256;
    pub const mainEq: usize = 260;
    pub const mainEq__low: usize = 260;
    pub const mainEq__low__enabled: usize = 260;
    pub const mainEq__low__gain: usize = 264;
    pub const mainEq__low__frequency: usize = 268;
    pub const mainEq__low__q: usize = 272;
    pub const mainEq__peak1: usize = 276;
    pub const mainEq__peak1__enabled: usize = 276;
    pub const mainEq__peak1__gain: usize = 280;
    pub const mainEq__peak1__frequency: usize = 284;
    pub const mainEq__peak1__q: usize = 288;
    pub const mainEq__peak2: usize = 292;
    pub const mainEq__peak2__enabled: usize = 292;
    pub const mainEq__peak2__gain: usize = 296;
    pub const mainEq__peak2__frequency: usize = 300;
    pub const mainEq__peak2__q: usize = 304;
    pub const mainEq__high: usize = 308;
    pub const mainEq__high__enabled: usize = 308;
    pub const mainEq__high__gain: usize = 312;
    pub const mainEq__high__frequency: usize = 316;
    pub const mainEq__high__q: usize = 320;
    pub const dialogDyn: usize = 324;
    pub const dialogDyn__enable: usize = 324;
    pub const dialogDyn__preGain: usize = 328;
    pub const dialogDyn__postGain: usize = 332;
    pub const dialogDyn__threshold: usize = 336;
    pub const dialogDyn__ratio: usize = 340;
    pub const dialogDyn__timeAttack: usize = 344;
    pub const dialogDyn__timeRelease: usize = 348;
    pub const dialogDyn__spread: usize = 352;
    pub const dialogDyn__linear: usize = 356;
    pub const dialogDyn__ITU1770: usize = 360;
    pub const sfxDyn: usize = 364;
    pub const sfxDyn__enable: usize = 364;
    pub const sfxDyn__preGain: usize = 368;
    pub const sfxDyn__postGain: usize = 372;
    pub const sfxDyn__threshold: usize = 376;
    pub const sfxDyn__ratio: usize = 380;
    pub const sfxDyn__timeAttack: usize = 384;
    pub const sfxDyn__timeRelease: usize = 388;
    pub const sfxDyn__spread: usize = 392;
    pub const sfxDyn__linear: usize = 396;
    pub const sfxDyn__ITU1770: usize = 400;
    pub const mainDyn: usize = 404;
    pub const mainDyn__enable: usize = 404;
    pub const mainDyn__preGain: usize = 408;
    pub const mainDyn__postGain: usize = 412;
    pub const mainDyn__threshold: usize = 416;
    pub const mainDyn__ratio: usize = 420;
    pub const mainDyn__timeAttack: usize = 424;
    pub const mainDyn__timeRelease: usize = 428;
    pub const mainDyn__spread: usize = 432;
    pub const mainDyn__linear: usize = 436;
    pub const mainDyn__ITU1770: usize = 440;
    pub const busDialogGain: usize = 444;
    pub const busWeapExpSceneGain: usize = 448;
    pub const busSfxGain: usize = 452;
    pub const busReverbGain: usize = 456;
    pub const busMusicGain: usize = 460;
    pub const busMovieGain: usize = 464;
    pub const busReferenceGain: usize = 468;
    pub const busDialogEnable: usize = 472;
    pub const busWeapExpSceneEnable: usize = 476;
    pub const busSfxEnable: usize = 480;
    pub const busReverbEnable: usize = 484;
    pub const busMusicEnable: usize = 488;
    pub const busMovieEnable: usize = 492;
    pub const busReferenceEnable: usize = 496;
}
pub mod SndMasterEqParam {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const enabled: usize = 0;
    pub const gain: usize = 4;
    pub const frequency: usize = 8;
    pub const q: usize = 12;
}
pub mod SndMasterEqParamBank {
    pub const SIZE: usize = 64;
    pub const ALIGN: usize = 4;
    pub const low: usize = 0;
    pub const low__enabled: usize = 0;
    pub const low__gain: usize = 4;
    pub const low__frequency: usize = 8;
    pub const low__q: usize = 12;
    pub const peak1: usize = 16;
    pub const peak1__enabled: usize = 16;
    pub const peak1__gain: usize = 20;
    pub const peak1__frequency: usize = 24;
    pub const peak1__q: usize = 28;
    pub const peak2: usize = 32;
    pub const peak2__enabled: usize = 32;
    pub const peak2__gain: usize = 36;
    pub const peak2__frequency: usize = 40;
    pub const peak2__q: usize = 44;
    pub const high: usize = 48;
    pub const high__enabled: usize = 48;
    pub const high__gain: usize = 52;
    pub const high__frequency: usize = 56;
    pub const high__q: usize = 60;
}
pub mod SndMusicAsset {
    pub const SIZE: usize = 140;
    pub const ALIGN: usize = 4;
    pub const alias: usize = 0;
    pub const aliasId: usize = 64;
    pub const looping: usize = 68;
    pub const completeLoop: usize = 72;
    pub const removeAfterPlay: usize = 76;
    pub const firstRandom: usize = 80;
    pub const startSync: usize = 84;
    pub const stopSync: usize = 88;
    pub const completeOnStop: usize = 92;
    pub const loopStartOffset: usize = 96;
    pub const bpm: usize = 100;
    pub const assetType: usize = 104;
    pub const loopNumber: usize = 108;
    pub const order: usize = 112;
    pub const startDelayBeats: usize = 116;
    pub const startFadeBeats: usize = 120;
    pub const stopDelayBeats: usize = 124;
    pub const stopFadeBeats: usize = 128;
    pub const startOffsetFrames: usize = 132;
    pub const meter: usize = 136;
}
pub mod SndMusicSet {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const id: usize = 64;
    pub const stateCount: usize = 68;
    pub const states: usize = 72;
}
pub mod SndMusicState {
    pub const SIZE: usize = 384;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const id: usize = 64;
    pub const intro: usize = 68;
    pub const intro__alias: usize = 68;
    pub const intro__aliasId: usize = 132;
    pub const intro__looping: usize = 136;
    pub const intro__completeLoop: usize = 140;
    pub const intro__removeAfterPlay: usize = 144;
    pub const intro__firstRandom: usize = 148;
    pub const intro__startSync: usize = 152;
    pub const intro__stopSync: usize = 156;
    pub const intro__completeOnStop: usize = 160;
    pub const intro__loopStartOffset: usize = 164;
    pub const intro__bpm: usize = 168;
    pub const intro__assetType: usize = 172;
    pub const intro__loopNumber: usize = 176;
    pub const intro__order: usize = 180;
    pub const intro__startDelayBeats: usize = 184;
    pub const intro__startFadeBeats: usize = 188;
    pub const intro__stopDelayBeats: usize = 192;
    pub const intro__stopFadeBeats: usize = 196;
    pub const intro__startOffsetFrames: usize = 200;
    pub const intro__meter: usize = 204;
    pub const exit: usize = 208;
    pub const exit__alias: usize = 208;
    pub const exit__aliasId: usize = 272;
    pub const exit__looping: usize = 276;
    pub const exit__completeLoop: usize = 280;
    pub const exit__removeAfterPlay: usize = 284;
    pub const exit__firstRandom: usize = 288;
    pub const exit__startSync: usize = 292;
    pub const exit__stopSync: usize = 296;
    pub const exit__completeOnStop: usize = 300;
    pub const exit__loopStartOffset: usize = 304;
    pub const exit__bpm: usize = 308;
    pub const exit__assetType: usize = 312;
    pub const exit__loopNumber: usize = 316;
    pub const exit__order: usize = 320;
    pub const exit__startDelayBeats: usize = 324;
    pub const exit__startFadeBeats: usize = 328;
    pub const exit__stopDelayBeats: usize = 332;
    pub const exit__stopFadeBeats: usize = 336;
    pub const exit__startOffsetFrames: usize = 340;
    pub const exit__meter: usize = 344;
    pub const loopCount: usize = 348;
    pub const loops: usize = 352;
    pub const order: usize = 360;
    pub const interruptPriority: usize = 364;
    pub const persistPriority: usize = 368;
    pub const isRandom: usize = 372;
    pub const isSequential: usize = 373;
    pub const skipPreviousExit: usize = 374;
    pub const wallClockSync: usize = 375;
    pub const refCount: usize = 376;
    pub const status: usize = 380;
}
pub mod SndSendEffectParams {
    pub const SIZE: usize = 136;
    pub const ALIGN: usize = 4;
    pub const name: usize = 0;
    pub const id: usize = 64;
    pub const smoothing: usize = 68;
    pub const earlyTime: usize = 72;
    pub const lateTime: usize = 76;
    pub const earlyGain: usize = 80;
    pub const lateGain: usize = 84;
    pub const lateGainProx: usize = 88;
    pub const returnGain: usize = 92;
    pub const earlyLpf: usize = 96;
    pub const lateLpf: usize = 100;
    pub const inputLpf: usize = 104;
    pub const dampLpf: usize = 108;
    pub const wallReflect: usize = 112;
    pub const dryGain: usize = 116;
    pub const earlySize: usize = 120;
    pub const lateSize: usize = 124;
    pub const diffusion: usize = 128;
    pub const rearLevel: usize = 132;
}
pub mod SoundTable {
    pub const SIZE: usize = 248;
    pub const ALIGN: usize = 8;
    pub const id: usize = 0;
    pub const adsrSettings: usize = 8;
    pub const adsrCount: usize = 16;
    pub const ambientDefs: usize = 24;
    pub const ambientDefCount: usize = 32;
    pub const ambientEvents: usize = 40;
    pub const ambientEventCount: usize = 48;
    pub const ambientElements: usize = 56;
    pub const ambientElementsCount: usize = 64;
    pub const filters: usize = 72;
    pub const filterCount: usize = 80;
    pub const mixes: usize = 88;
    pub const mixCount: usize = 96;
    pub const npcADSRZones: usize = 104;
    pub const npcADSRZoneCount: usize = 112;
    pub const occlusionFilters: usize = 120;
    pub const occlusionFilterCount: usize = 128;
    pub const playerADSRZones: usize = 136;
    pub const playerADSRZoneCount: usize = 144;
    pub const reverbs: usize = 152;
    pub const reverbCount: usize = 160;
    pub const timeScaleSettings: usize = 168;
    pub const timeScaleSettingCount: usize = 176;
    pub const zones: usize = 184;
    pub const zoneCount: usize = 192;
    pub const unk: usize = 200;
    pub const unkCount: usize = 208;
    pub const fullOcclusionDefs: usize = 216;
    pub const fullOcclusionDefCount: usize = 224;
    pub const plrBreathStateDefs: usize = 232;
    pub const plrBreathStateDefCount: usize = 240;
}
pub mod SpaceshipAnimParts {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const anim: usize = 0;
}
pub mod SpaceshipTargetDef {
    pub const SIZE: usize = 136;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const targetTag: usize = 8;
    pub const annotationOffsetX: usize = 12;
    pub const annotationOffsetY: usize = 16;
    pub const annotationOffsetZ: usize = 20;
    pub const disableLeading: usize = 24;
    pub const lock_minDistance: usize = 28;
    pub const lock_maxDistance: usize = 32;
    pub const lock_cutoffDistance: usize = 36;
    pub const lock_minRate: usize = 40;
    pub const lock_maxRate: usize = 44;
    pub const lock_maxZoomFOV: usize = 48;
    pub const lock_followDistance: usize = 52;
    pub const lockCooldown_rate: usize = 56;
    pub const aa_closeDistance: usize = 60;
    pub const aa_farDistance: usize = 64;
    pub const aa_closeMinAngle: usize = 68;
    pub const aa_closeMaxAngle: usize = 72;
    pub const aa_closeBlendMax: usize = 76;
    pub const aa_farMinAngle: usize = 80;
    pub const aa_farMaxAngle: usize = 84;
    pub const aa_farBlendMax: usize = 88;
    pub const aaLocked_closeDistance: usize = 92;
    pub const aaLocked_farDistance: usize = 96;
    pub const aaLocked_closeMinAngle: usize = 100;
    pub const aaLocked_closeMaxAngle: usize = 104;
    pub const aaLocked_closeBlendMax: usize = 108;
    pub const aaLocked_farMinAngle: usize = 112;
    pub const aaLocked_farMaxAngle: usize = 116;
    pub const aaLocked_farBlendMax: usize = 120;
    pub const pilotAssistScale: usize = 124;
    pub const callout_maxDistance: usize = 128;
    pub const callout_maxAngle: usize = 132;
}
pub mod SpawnPointEntityRecord {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 4;
    pub const index: usize = 0;
    pub const name: usize = 4;
    pub const target: usize = 8;
    pub const script_noteworthy: usize = 12;
    pub const origin: usize = 16;
    pub const angles: usize = 28;
}
pub mod SpawnPointRecordList {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const spawnsCount: usize = 0;
    pub const spawns: usize = 8;
}
pub mod Spawner {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const origin: usize = 0;
    pub const angles: usize = 12;
    pub const fields: usize = 24;
    pub const flags: usize = 32;
    pub const count: usize = 36;
    pub const numFields: usize = 40;
    pub const targetname: usize = 44;
    pub const classname: usize = 48;
}
pub mod SpawnerField {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const key: usize = 0;
    pub const keyCanonical: usize = 4;
    pub const value: usize = 8;
    pub const r#type: usize = 12;
}
pub mod SpawnerList {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const spawnerCount: usize = 0;
    pub const spawnerList: usize = 8;
}
pub mod SpeakerMap {
    pub const SIZE: usize = 260;
    pub const ALIGN: usize = 4;
    pub const name: usize = 0;
    pub const id: usize = 64;
    pub const monoToStereo: usize = 68;
    pub const stereoToStereo: usize = 76;
    pub const monoTo51: usize = 92;
    pub const stereoTo51: usize = 116;
    pub const monoTo71: usize = 164;
    pub const stereoTo71: usize = 196;
}
pub mod SplinePointEntityRecord {
    pub const SIZE: usize = 168;
    pub const ALIGN: usize = 4;
    pub const splineId: usize = 0;
    pub const splineNodeId: usize = 4;
    pub const splineNodeLabel: usize = 8;
    pub const targetname: usize = 12;
    pub const target: usize = 16;
    pub const string: usize = 20;
    pub const speed: usize = 24;
    pub const splineNodeTension: usize = 28;
    pub const origin: usize = 32;
    pub const angles: usize = 44;
    pub const throttle: usize = 56;
    pub const corridorDims: usize = 60;
    pub const tangent: usize = 68;
    pub const distToNextNode: usize = 80;
    pub const positionCubic: usize = 84;
    pub const tangentQuadratic: usize = 132;
}
pub mod SplinePointRecordList {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const splinePointCount: usize = 0;
    pub const splineLength: usize = 4;
    pub const splinePoints: usize = 8;
}
pub mod SplineRecordList {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const splineCount: usize = 0;
    pub const splines: usize = 8;
}
pub mod Stage {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const origin: usize = 8;
    pub const triggerIndex: usize = 20;
    pub const sunPrimaryLightIndex: usize = 22;
    pub const entityUID: usize = 24;
    pub const unk: usize = 28;
}
pub mod StaticModelCollisionModelList {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const numModels: usize = 0;
    pub const staticModelIndex: usize = 8;
}
pub mod StreamingInfo {
    pub const SIZE: usize = 192;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const unused: usize = 8;
    pub const flags: usize = 16;
    pub const flags__imageCount: usize = 16;
    pub const flags__images: usize = 24;
    pub const flags__imageFlags: usize = 32;
    pub const flags__stringCount: usize = 40;
    pub const flags__strings: usize = 48;
    pub const flags__stringFlags: usize = 56;
    pub const flags__materialCount: usize = 64;
    pub const flags__materials: usize = 72;
    pub const flags__materialFlags: usize = 80;
    pub const transientInfoFlags: usize = 88;
    pub const modelCount: usize = 92;
    pub const perModelFlags: usize = 96;
    pub const forcedStubAssetCount: usize = 104;
    pub const forcedStubAssets: usize = 112;
    pub const transientFileCount: usize = 120;
    pub const transientFiles: usize = 128;
    pub const transientAssetCount: usize = 136;
    pub const transientAssets: usize = 144;
    pub const stringCount: usize = 152;
    pub const strings: usize = 160;
    pub const transientSets: usize = 168;
    pub const transientSets__count: usize = 168;
    pub const transientSets__names: usize = 176;
    pub const transientSets__masks: usize = 184;
}
pub mod StringTable {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const columnCount: usize = 8;
    pub const rowCount: usize = 12;
    pub const values: usize = 16;
}
pub mod StringTableCell {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const string: usize = 0;
    pub const hash: usize = 8;
}
pub mod StructuredDataDef {
    pub const SIZE: usize = 88;
    pub const ALIGN: usize = 8;
    pub const version: usize = 0;
    pub const formatChecksum: usize = 4;
    pub const enumCount: usize = 8;
    pub const enums: usize = 16;
    pub const structCount: usize = 24;
    pub const structs: usize = 32;
    pub const indexedArrayCount: usize = 40;
    pub const indexedArrays: usize = 48;
    pub const enumedArrayCount: usize = 56;
    pub const enumedArrays: usize = 64;
    pub const rootType: usize = 72;
    pub const rootType__type: usize = 72;
    pub const rootType__u: usize = 76;
    pub const rootType__u__stringDataLength: usize = 76;
    pub const rootType__u__enumIndex: usize = 76;
    pub const rootType__u__structIndex: usize = 76;
    pub const rootType__u__indexedArrayIndex: usize = 76;
    pub const rootType__u__enumedArrayIndex: usize = 76;
    pub const rootType__u__index: usize = 76;
    pub const size: usize = 80;
}
pub mod StructuredDataDefSet {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const defCount: usize = 8;
    pub const defs: usize = 16;
}
pub mod StructuredDataEnum {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const entryCount: usize = 0;
    pub const reservedEntryCount: usize = 4;
    pub const entries: usize = 8;
}
pub mod StructuredDataEnumEntry {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const string: usize = 0;
    pub const index: usize = 4;
}
pub mod StructuredDataEnumedArray {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const enumIndex: usize = 0;
    pub const elementType: usize = 4;
    pub const elementType__type: usize = 4;
    pub const elementType__u: usize = 8;
    pub const elementType__u__stringDataLength: usize = 8;
    pub const elementType__u__enumIndex: usize = 8;
    pub const elementType__u__structIndex: usize = 8;
    pub const elementType__u__indexedArrayIndex: usize = 8;
    pub const elementType__u__enumedArrayIndex: usize = 8;
    pub const elementType__u__index: usize = 8;
    pub const elementSize: usize = 12;
}
pub mod StructuredDataIndexedArray {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const arraySize: usize = 0;
    pub const elementType: usize = 4;
    pub const elementType__type: usize = 4;
    pub const elementType__u: usize = 8;
    pub const elementType__u__stringDataLength: usize = 8;
    pub const elementType__u__enumIndex: usize = 8;
    pub const elementType__u__structIndex: usize = 8;
    pub const elementType__u__indexedArrayIndex: usize = 8;
    pub const elementType__u__enumedArrayIndex: usize = 8;
    pub const elementType__u__index: usize = 8;
    pub const elementSize: usize = 12;
}
pub mod StructuredDataStruct {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const propertyCount: usize = 0;
    pub const properties: usize = 8;
    pub const size: usize = 16;
    pub const bitOffset: usize = 20;
}
pub mod StructuredDataStructProperty {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 4;
    pub const name: usize = 0;
    pub const r#type: usize = 4;
    pub const type__type: usize = 4;
    pub const type__u: usize = 8;
    pub const type__u__stringDataLength: usize = 8;
    pub const type__u__enumIndex: usize = 8;
    pub const type__u__structIndex: usize = 8;
    pub const type__u__indexedArrayIndex: usize = 8;
    pub const type__u__enumedArrayIndex: usize = 8;
    pub const type__u__index: usize = 8;
    pub const offset: usize = 12;
    pub const validation: usize = 16;
}
pub mod StructuredDataType {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const u: usize = 4;
    pub const u__stringDataLength: usize = 4;
    pub const u__enumIndex: usize = 4;
    pub const u__structIndex: usize = 4;
    pub const u__indexedArrayIndex: usize = 4;
    pub const u__enumedArrayIndex: usize = 4;
    pub const u__index: usize = 4;
}
pub mod StructuredDataTypeUnion {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 4;
    pub const stringDataLength: usize = 0;
    pub const enumIndex: usize = 0;
    pub const structIndex: usize = 0;
    pub const indexedArrayIndex: usize = 0;
    pub const enumedArrayIndex: usize = 0;
    pub const index: usize = 0;
}
pub mod SuitAnimEntry {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const anim: usize = 0;
    pub const time: usize = 8;
    pub const animType: usize = 12;
}
pub mod SuitAnimPackage {
    pub const SIZE: usize = 3848;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const animOverrides: usize = 8;
}
pub mod SuitAnimPerWeapClass {
    pub const SIZE: usize = 640;
    pub const ALIGN: usize = 8;
    pub const anims: usize = 0;
    pub const numAnims: usize = 8;
    pub const weapClasses: usize = 12;
    pub const gestures: usize = 16;
    pub const numGestures: usize = 24;
    pub const footstep: usize = 28;
}
pub mod SuitDef {
    pub const SIZE: usize = 704;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const cam_minVelocityForFovIncrease: usize = 8;
    pub const cam_maxVelocityForFovIncrease: usize = 12;
    pub const cam_velocityToDecreaseFov: usize = 16;
    pub const cam_fovIncreaseAtMaxVelocity: usize = 20;
    pub const cam_oneOverFovEaseInTimeMs: usize = 24;
    pub const cam_oneOverFovEaseOutTimeMs: usize = 28;
    pub const enableIKOverride: usize = 32;
    pub const player_globalAccelScale: usize = 36;
    pub const player_sprintSpeedScale: usize = 40;
    pub const player_sprintUnlimited: usize = 44;
    pub const player_viewBobScale: usize = 48;
    pub const jump_slowdownEnable: usize = 52;
    pub const jump_height: usize = 56;
    pub const sprintLeap_height: usize = 60;
    pub const sprintLeap_forwardVelocityScale: usize = 64;
    pub const sprintLeap_minSprintTimeMs: usize = 68;
    pub const doubleJump_accel: usize = 72;
    pub const doubleJump_speed: usize = 76;
    pub const doubleJump_speedNoBoost: usize = 80;
    pub const doubleJump_frictionMin: usize = 84;
    pub const doubleJump_frictionMax: usize = 88;
    pub const doubleJump_initialUpBoostAccel: usize = 92;
    pub const doubleJump_standardUpBoostAccel: usize = 96;
    pub const doubleJump_energyNeededForInitialUpBoost: usize = 100;
    pub const doubleJump_energyNeededForStandardUpBoost: usize = 104;
    pub const doubleJump_maxUpwardsVelocity: usize = 108;
    pub const doubleJump_enableMinZVelocity: usize = 112;
    pub const doubleJump_minZVelocity: usize = 116;
    pub const doubleJump_energyInitialCost: usize = 120;
    pub const doubleJump_boostUpEnergyUseRate: usize = 124;
    pub const doubleJump_energyUsePerButtonPress: usize = 128;
    pub const doubleJump_hoverOnly: usize = 132;
    pub const doubleJump_maxViewPitchDip: usize = 136;
    pub const doubleJump_maxViewBackwardsPitchDip: usize = 140;
    pub const doubleJump_maxViewRoll: usize = 144;
    pub const doubleJump_oneOverEaseInTime: usize = 148;
    pub const doubleJump_oneOverEaseOutTimeMs: usize = 152;
    pub const doubleJump_alwaysDipView: usize = 156;
    pub const doubleJump_sound: usize = 160;
    pub const doubleJump_soundPlayer: usize = 168;
    pub const doubleJump_releaseSound: usize = 176;
    pub const doubleJump_releaseSoundPlayer: usize = 184;
    pub const slide_sprint_penalty_ms: usize = 192;
    pub const slide_allow_firing: usize = 196;
    pub const slide_allow_ads: usize = 197;
    pub const slide_allow_weapon_switch: usize = 198;
    pub const slide_jump_speed_scale: usize = 200;
    pub const slide_energy_cost_ratio: usize = 204;
    pub const slide_energy_unknown01: usize = 208;
    pub const slide_energy_unknown02: usize = 212;
    pub const slide_energy_unknown03: usize = 216;
    pub const slide_max_time_ms: usize = 220;
    pub const slide_max_time_reduced_ms: usize = 224;
    pub const slide_max_time_base_ms: usize = 228;
    pub const wallRun_minZVel: usize = 232;
    pub const wallRun_minTriggerSpeed: usize = 236;
    pub const wallRun_minMaintainSpeed: usize = 240;
    pub const wallRun_delayPeriodMs: usize = 244;
    pub const wallRun_minJumpHeight: usize = 248;
    pub const wallRun_maxTimeMs: usize = 252;
    pub const wallRun_fallStageTimeMs: usize = 256;
    pub const wallRun_maxHeight: usize = 260;
    pub const wallRun_jumpHeight: usize = 264;
    pub const wallRun_jumpVelocity: usize = 268;
    pub const wallRun_frictionScale: usize = 272;
    pub const wallRun_speedScale: usize = 276;
    pub const wallRun_speedScaleADS: usize = 280;
    pub const wallRun_energyInitialCost: usize = 284;
    pub const wallRun_energyChangePerSecond: usize = 288;
    pub const wallRun_energyUnknown01: usize = 292;
    pub const wallRun_energyUnknown02: usize = 296;
    pub const suitAnimType: usize = 300;
    pub const animPackage: usize = 304;
    pub const animPackageL: usize = 312;
    pub const animPackageR: usize = 320;
    pub const animPackageRelaxed: usize = 328;
    pub const animPackageSafe: usize = 336;
    pub const facialAnimType: usize = 344;
    pub const bodyAnimType: usize = 348;
    pub const scriptableDef: usize = 352;
    pub const viewheight_stand: usize = 360;
    pub const viewheight_crouch: usize = 364;
    pub const viewheight_prone: usize = 368;
    pub const viewheight_laststand: usize = 372;
    pub const viewheight_dead: usize = 376;
    pub const viewheight_swim: usize = 380;
    pub const viewheight_slide: usize = 384;
    pub const bounds_radius: usize = 388;
    pub const bounds_height_stand: usize = 392;
    pub const bounds_height_crouch: usize = 396;
    pub const bounds_height_prone: usize = 400;
    pub const bounds_stand: usize = 404;
    pub const bounds_stand__midPoint: usize = 404;
    pub const bounds_stand__halfSize: usize = 416;
    pub const bounds_crouch: usize = 428;
    pub const bounds_crouch__midPoint: usize = 428;
    pub const bounds_crouch__halfSize: usize = 440;
    pub const bounds_prone: usize = 452;
    pub const bounds_prone__midPoint: usize = 452;
    pub const bounds_prone__halfSize: usize = 464;
    pub const radialMotionBlur_interpTimeIn: usize = 476;
    pub const radialMotionBlur_interpTimeOut: usize = 480;
    pub const radialMotionBlur_sprintMinRadius: usize = 484;
    pub const radialMotionBlur_sprintMaxRadius: usize = 488;
    pub const radialMotionBlur_sprintMinStrength: usize = 492;
    pub const radialMotionBlur_sprintMaxStrength: usize = 496;
    pub const radialMotionBlur_slideMinRadius: usize = 500;
    pub const radialMotionBlur_slideMaxRadius: usize = 504;
    pub const radialMotionBlur_slideMinStrength: usize = 508;
    pub const radialMotionBlur_slideMaxStrength: usize = 512;
    pub const radialMotionBlur_doubleJumpMinRadius: usize = 516;
    pub const radialMotionBlur_doubleJumpMaxRadius: usize = 520;
    pub const radialMotionBlur_doubleJumpMinStrength: usize = 524;
    pub const radialMotionBlur_doubleJumpMaxStrength: usize = 528;
    pub const radialMotionBlur_wallRunMinRadius: usize = 532;
    pub const radialMotionBlur_wallRunMaxRadius: usize = 536;
    pub const radialMotionBlur_wallRunMinStrength: usize = 540;
    pub const radialMotionBlur_wallRunMaxStrength: usize = 544;
    pub const radialMotionBlur_groundPoundMinRadius: usize = 548;
    pub const radialMotionBlur_groundPoundMaxRadius: usize = 552;
    pub const radialMotionBlur_groundPoundMinStrength: usize = 556;
    pub const radialMotionBlur_groundPoundMaxStrength: usize = 560;
    pub const radialMotionBlur_rewindMinRadius: usize = 564;
    pub const radialMotionBlur_rewindMaxRadius: usize = 568;
    pub const radialMotionBlur_rewindMinStrength: usize = 572;
    pub const radialMotionBlur_rewindMaxStrength: usize = 576;
    pub const radialMotionBlur_dodgeMinRadius: usize = 580;
    pub const radialMotionBlur_dodgeMaxRadius: usize = 584;
    pub const radialMotionBlur_dodgeMinStrength: usize = 588;
    pub const radialMotionBlur_dodgeMaxStrength: usize = 592;
    pub const groundPound_activationRumble: usize = 600;
    pub const groundPound_activationSound: usize = 608;
    pub const groundPound_activationSoundPlayer: usize = 616;
    pub const groundPound_landingRumble: usize = 624;
    pub const groundPound_landingSound: usize = 632;
    pub const groundPound_landingSoundPlayer: usize = 640;
    pub const landing_rumbleLowHeight: usize = 648;
    pub const landing_rumbleMediumHeight: usize = 656;
    pub const landing_rumbleHighHeight: usize = 664;
    pub const landing_rumbleExtremeHeight: usize = 672;
    pub const landing_speedScale: usize = 680;
    pub const footstep_shakeBroadcastRadiusInches: usize = 684;
    pub const footstep_shakeDurationMs: usize = 688;
    pub const footstep_shakeAmplitude: usize = 692;
    pub const footstep_rumble: usize = 696;
}
pub mod SuitGestureEntry {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const gesture: usize = 0;
    pub const gestureAnimType: usize = 8;
}
pub mod SurfaceFxEntry {
    pub const SIZE: usize = 1024;
    pub const ALIGN: usize = 8;
    pub const surfaceEffect: usize = 0;
}
pub mod SurfaceFxTable {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const table: usize = 8;
}
pub mod TTFDef {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const fileLen: usize = 8;
    pub const file: usize = 16;
    pub const ftFace: usize = 24;
}
pub mod TimescaleEntry {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const id: usize = 0;
    pub const presetName: usize = 4;
    pub const entChannelIdx: usize = 8;
    pub const scale: usize = 12;
}
pub mod TopDownMapData {
    pub const SIZE: usize = 8220;
    pub const ALIGN: usize = 4;
    pub const header: usize = 0;
    pub const header__version: usize = 0;
    pub const header__samplesPerAxis: usize = 4;
    pub const header__bottomLeft: usize = 8;
    pub const header__topRight: usize = 16;
    pub const content: usize = 24;
    pub const content__navMeshVisible: usize = 24;
}
pub mod TopDownMapDataContent {
    pub const SIZE: usize = 8193;
    pub const ALIGN: usize = 1;
    pub const navMeshVisible: usize = 0;
}
pub mod TopDownMapDataHeader {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 4;
    pub const version: usize = 0;
    pub const samplesPerAxis: usize = 4;
    pub const bottomLeft: usize = 8;
    pub const topRight: usize = 16;
}
pub mod TracerDef {
    pub const SIZE: usize = 128;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const material: usize = 8;
    pub const drawInterval: usize = 16;
    pub const fadeOverTime: usize = 20;
    pub const fadeTime: usize = 24;
    pub const speed: usize = 28;
    pub const beamLength: usize = 32;
    pub const beamWidth: usize = 36;
    pub const screwRadius: usize = 40;
    pub const screwDist: usize = 44;
    pub const colors: usize = 48;
}
pub mod TransientAssetReference {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const r#type: usize = 8;
    pub const mpParentIndex: usize = 12;
}
pub mod TransientFileReference {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const r#type: usize = 8;
}
pub mod TransientSets {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const count: usize = 0;
    pub const names: usize = 8;
    pub const masks: usize = 16;
}
pub mod TriggerHull {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const bounds: usize = 0;
    pub const bounds__midPoint: usize = 0;
    pub const bounds__halfSize: usize = 12;
    pub const contents: usize = 24;
    pub const slabCount: usize = 28;
    pub const firstSlab: usize = 30;
}
pub mod TriggerModel {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const contents: usize = 0;
    pub const hullCount: usize = 4;
    pub const firstHull: usize = 6;
    pub const windingCount: usize = 8;
    pub const firstWinding: usize = 10;
    pub const flags: usize = 12;
    pub const physicsAsset: usize = 16;
    pub const physicsShapeOverrideIdx: usize = 24;
}
pub mod TriggerSlab {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 4;
    pub const dir: usize = 0;
    pub const midPoint: usize = 12;
    pub const halfSize: usize = 16;
}
pub mod TriggerWinding {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 4;
    pub const dir: usize = 0;
    pub const pointCount: usize = 12;
    pub const firstPoint: usize = 14;
    pub const flags: usize = 16;
}
pub mod TriggerWindingPoint {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const loc: usize = 0;
}
pub mod UmbraGate {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const objID: usize = 0;
    pub const bounds: usize = 4;
    pub const bounds__midPoint: usize = 4;
    pub const bounds__halfSize: usize = 16;
    pub const closeDistance: usize = 28;
}
pub mod UmbraGate2 {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 1;
    pub const __pad0: usize = 0;
}
pub mod VFXOverride {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const attachment1: usize = 0;
    pub const attachment2: usize = 4;
    pub const overrides: usize = 8;
    pub const overridesAlt: usize = 16;
}
pub mod VectorField {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const subFields: usize = 8;
    pub const worldBounds: usize = 16;
    pub const worldBounds__mins: usize = 16;
    pub const worldBounds__maxs: usize = 28;
    pub const numSubFields: usize = 40;
    pub const pad: usize = 44;
}
pub mod VectorSubField {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 8;
    pub const linearData: usize = 0;
    pub const worldBounds: usize = 8;
    pub const worldBounds__mins: usize = 8;
    pub const worldBounds__maxs: usize = 20;
    pub const ds: usize = 32;
    pub const numEntries: usize = 44;
    pub const flags: usize = 56;
    pub const r#type: usize = 60;
    pub const pad: usize = 64;
}
pub mod VehicleDef {
    pub const SIZE: usize = 2704;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const r#type: usize = 8;
    pub const useHintString: usize = 16;
    pub const health: usize = 24;
    pub const hitClientScriptables: usize = 28;
    pub const texScrollScale: usize = 32;
    pub const topSpeed: usize = 36;
    pub const accel: usize = 40;
    pub const rotRate: usize = 44;
    pub const rotAccel: usize = 48;
    pub const maxBodyPitch: usize = 52;
    pub const maxBodyRoll: usize = 56;
    pub const fakeBodyAccelPitch: usize = 60;
    pub const fakeBodyAccelRoll: usize = 64;
    pub const fakeBodyVelPitch: usize = 68;
    pub const fakeBodyVelRoll: usize = 72;
    pub const fakeBodySideVelPitch: usize = 76;
    pub const fakeBodyPitchStrength: usize = 80;
    pub const fakeBodyRollStrength: usize = 84;
    pub const fakeBodyPitchDampening: usize = 88;
    pub const fakeBodyRollDampening: usize = 92;
    pub const fakeBodyBoatRockingAmplitude: usize = 96;
    pub const fakeBodyBoatRockingPeriod: usize = 100;
    pub const fakeBodyBoatRockingRotationPeriod: usize = 104;
    pub const fakeBodyBoatRockingFadeoutSpeed: usize = 108;
    pub const boatBouncingMinForce: usize = 112;
    pub const boatBouncingMaxForce: usize = 116;
    pub const boatBouncingRate: usize = 120;
    pub const boatBouncingFadeinSpeed: usize = 124;
    pub const boatBouncingFadeoutSteeringAngle: usize = 128;
    pub const collisionDamage: usize = 132;
    pub const collisionSpeed: usize = 136;
    pub const killcamOffset: usize = 140;
    pub const playerProtected: usize = 152;
    pub const bulletDamage: usize = 156;
    pub const armorPiercingDamage: usize = 160;
    pub const grenadeDamage: usize = 164;
    pub const projectileDamage: usize = 168;
    pub const projectileSplashDamage: usize = 172;
    pub const heavyExplosiveDamage: usize = 176;
    pub const vehPhysDef: usize = 180;
    pub const vehPhysDef__physicsEnabled: usize = 180;
    pub const vehPhysDef__unk_1: usize = 184;
    pub const vehPhysDef__unk_2: usize = 188;
    pub const vehPhysDef__unk_3: usize = 192;
    pub const vehPhysDef__unk_4: usize = 196;
    pub const vehPhysDef__unk_5: usize = 200;
    pub const vehPhysDef__unk_6: usize = 204;
    pub const vehPhysDef__unk_7: usize = 208;
    pub const vehPhysDef__unk_8: usize = 212;
    pub const vehPhysDef__unk_9: usize = 216;
    pub const vehPhysDef__unk_10: usize = 220;
    pub const vehPhysDef__unk_11: usize = 224;
    pub const vehPhysDef__unk_12: usize = 228;
    pub const vehPhysDef__unk_13: usize = 232;
    pub const vehPhysDef__unk_14: usize = 236;
    pub const vehPhysDef__unk_15: usize = 240;
    pub const vehPhysDef__unk_16: usize = 244;
    pub const vehPhysDef__unk_17: usize = 248;
    pub const vehPhysDef__unk_18: usize = 252;
    pub const vehPhysDef__unk_19: usize = 256;
    pub const vehPhysDef__unk_20: usize = 260;
    pub const vehPhysDef__unk_21: usize = 264;
    pub const vehPhysDef__unk_22: usize = 268;
    pub const vehPhysDef__unk_23: usize = 272;
    pub const vehPhysDef__unk_24: usize = 276;
    pub const vehPhysDef__unk_25: usize = 280;
    pub const vehPhysDef__unk_26: usize = 284;
    pub const vehPhysDef__unk_27: usize = 288;
    pub const vehPhysDef__unk_28: usize = 292;
    pub const vehPhysDef__unk_29: usize = 296;
    pub const vehPhysDef__unk_30: usize = 300;
    pub const vehPhysDef__unk_31: usize = 304;
    pub const vehPhysDef__unk_32: usize = 308;
    pub const vehPhysDef__unk_33: usize = 312;
    pub const vehPhysDef__unk_34: usize = 316;
    pub const vehPhysDef__unk_35: usize = 320;
    pub const vehPhysDef__unk_36: usize = 324;
    pub const vehPhysDef__unk_37: usize = 328;
    pub const vehPhysDef__unk_38: usize = 332;
    pub const vehPhysDef__unk_39: usize = 336;
    pub const vehPhysDef__unk_40: usize = 340;
    pub const vehPhysDef__unk_41: usize = 344;
    pub const vehPhysDef__unk_42: usize = 348;
    pub const vehPhysDef__unk_43: usize = 352;
    pub const vehPhysDef__unk_44: usize = 356;
    pub const vehPhysDef__unk_45: usize = 360;
    pub const vehPhysDef__unk_46: usize = 364;
    pub const vehPhysDef__unk_47: usize = 368;
    pub const vehPhysDef__unk_48: usize = 372;
    pub const vehPhysDef__unk_49: usize = 376;
    pub const vehPhysDef__unk_50: usize = 380;
    pub const vehPhysDef__unk_51: usize = 384;
    pub const vehPhysDef__unk_52: usize = 388;
    pub const vehPhysDef__unk_53: usize = 392;
    pub const vehPhysDef__unk_54: usize = 396;
    pub const vehPhysDef__unk_55: usize = 400;
    pub const vehPhysDef__unk_56: usize = 404;
    pub const vehPhysDef__unk_57: usize = 408;
    pub const vehPhysDef__unk_58: usize = 412;
    pub const vehPhysDef__unk_59: usize = 416;
    pub const vehPhysDef__unk_60: usize = 420;
    pub const vehPhysDef__unk_61: usize = 424;
    pub const vehPhysDef__unk_62: usize = 428;
    pub const vehPhysDef__unk_63: usize = 432;
    pub const vehPhysDef__unk_64: usize = 436;
    pub const vehPhysDef__unk_65: usize = 440;
    pub const vehPhysDef__unk_66: usize = 444;
    pub const vehPhysDef__unk_67: usize = 448;
    pub const vehPhysDef__unk_68: usize = 452;
    pub const vehPhysDef__unk_69: usize = 456;
    pub const vehPhysDef__unk_70: usize = 460;
    pub const vehPhysDef__unk_71: usize = 464;
    pub const vehPhysDef__unk_72: usize = 468;
    pub const vehPhysDef__unk_73: usize = 472;
    pub const vehPhysDef__unk_74: usize = 476;
    pub const vehPhysDef__unk_75: usize = 480;
    pub const vehPhysDef__unk_76: usize = 484;
    pub const vehPhysDef__unk_77: usize = 488;
    pub const vehPhysDef__unk_78: usize = 492;
    pub const vehPhysDef__unk_79: usize = 496;
    pub const vehPhysDef__unk_80: usize = 500;
    pub const vehPhysDef__unk_81: usize = 504;
    pub const vehPhysDef__unk_82: usize = 508;
    pub const vehPhysDef__unk_83: usize = 512;
    pub const vehPhysDef__unk_84: usize = 516;
    pub const vehPhysDef__unk_85: usize = 520;
    pub const vehPhysDef__unk_86: usize = 524;
    pub const vehPhysDef__unk_87: usize = 528;
    pub const vehPhysDef__unk_88: usize = 532;
    pub const vehPhysDef__unk_89: usize = 536;
    pub const vehPhysDef__unk_90: usize = 540;
    pub const vehPhysDef__unk_91: usize = 544;
    pub const vehPhysDef__unk_92: usize = 548;
    pub const vehPhysDef__unk_93: usize = 552;
    pub const vehPhysDef__unk_94: usize = 556;
    pub const vehPhysDef__unk_95: usize = 560;
    pub const vehPhysDef__unk_96: usize = 564;
    pub const vehPhysDef__unk_97: usize = 568;
    pub const vehPhysDef__unk_98: usize = 572;
    pub const vehPhysDef__unk_99: usize = 576;
    pub const vehPhysDef__unk_100: usize = 580;
    pub const vehPhysDef__unk_101: usize = 584;
    pub const vehPhysDef__unk_102: usize = 588;
    pub const vehPhysDef__unk_103: usize = 592;
    pub const vehPhysDef__unk_104: usize = 596;
    pub const vehPhysDef__unk_105: usize = 600;
    pub const vehPhysDef__unk_106: usize = 604;
    pub const vehPhysDef__unk_107: usize = 608;
    pub const vehPhysDef__unk_108: usize = 612;
    pub const vehPhysDef__unk_109: usize = 616;
    pub const vehPhysDef__unk_110: usize = 620;
    pub const vehPhysDef__unk_111: usize = 624;
    pub const vehHelicopterLockAltitude: usize = 628;
    pub const vehHelicopterOffsetFromMesh: usize = 632;
    pub const vehHelicopterAltitudeOffset: usize = 636;
    pub const vehHelicopterPitchOffset: usize = 640;
    pub const vehHelicopterBoundsRadius: usize = 644;
    pub const vehHelicopterMaxSpeed: usize = 648;
    pub const vehHelicopterMaxSpeedVertical: usize = 652;
    pub const vehHelicopterMaxAccel: usize = 656;
    pub const vehHelicopterMaxAccelVertical: usize = 660;
    pub const vehHelicopterDecelerationFwd: usize = 664;
    pub const vehHelicopterDecelerationSide: usize = 668;
    pub const vehHelicopterDecelerationUp: usize = 672;
    pub const vehHelicopterMaxYawRate: usize = 676;
    pub const vehHelicopterMaxYawAccel: usize = 680;
    pub const vehHelicopterTiltFromVelocity: usize = 684;
    pub const vehHelicopterTiltFromControllerAxes: usize = 688;
    pub const vehHelicopterTiltFromAcceleration: usize = 692;
    pub const vehHelicopterTiltFromDeceleration: usize = 696;
    pub const vehHelicopterTiltFromFwdAndYaw_VelAtMaxTilt: usize = 700;
    pub const vehHelicopterTiltFromFwdAndYaw: usize = 704;
    pub const vehHelicopterTiltMomentum: usize = 708;
    pub const vehHelicopterTiltSpeed: usize = 712;
    pub const vehHelicopterMaxPitch: usize = 716;
    pub const vehHelicopterMaxRoll: usize = 720;
    pub const vehHelicopterHoverSpeedThreshold: usize = 724;
    pub const vehHelicopterJitterJerkyness: usize = 728;
    pub const vehHelicopterLookaheadTime: usize = 732;
    pub const vehHelicopterSoftCollisions: usize = 736;
    pub const vehHelicopterUseGroundFX: usize = 740;
    pub const vehHelicopterGroundFx: usize = 744;
    pub const vehHelicopterGroundFx__u: usize = 744;
    pub const vehHelicopterGroundFx__u__fx: usize = 744;
    pub const vehHelicopterGroundFx__u__vfx: usize = 744;
    pub const vehHelicopterGroundFx__u__data: usize = 744;
    pub const vehHelicopterGroundFx__type: usize = 752;
    pub const vehHelicopterGroundWaterFx: usize = 760;
    pub const vehHelicopterGroundWaterFx__u: usize = 760;
    pub const vehHelicopterGroundWaterFx__u__fx: usize = 760;
    pub const vehHelicopterGroundWaterFx__u__vfx: usize = 760;
    pub const vehHelicopterGroundWaterFx__u__data: usize = 760;
    pub const vehHelicopterGroundWaterFx__type: usize = 768;
    pub const vehHelicopterGroundFxDefaultRepeatRate: usize = 776;
    pub const vehHelicopterGroundFxSlowestRepeatRate: usize = 780;
    pub const vehHelicopterGroundFxFastestRepeatRate: usize = 784;
    pub const vehHelicopterGroundFxMinGroundDist: usize = 788;
    pub const vehHelicopterGroundFxMaxGroundDist: usize = 792;
    pub const ssWeaponSwitchDelaySeconds: usize = 796;
    pub const ssWeaponRaiseDelaySeconds: usize = 800;
    pub const ssFlyMaxSpeedMph: usize = 804;
    pub const ssFlyMaxAccelerationMphps: usize = 816;
    pub const ssFlyMaxCounterAccelerationMphps: usize = 828;
    pub const ssFlyFrictionMphps: usize = 840;
    pub const ssFlyJukeSpeedMph: usize = 852;
    pub const ssHoverMaxSpeedMph: usize = 864;
    pub const ssHoverMaxAccelerationMphps: usize = 876;
    pub const ssHoverMaxCounterAccelerationMphps: usize = 888;
    pub const ssHoverFrictionMphps: usize = 900;
    pub const ssHoverJukeSpeedMph: usize = 912;
    pub const ssFlyJukeRampTime: usize = 924;
    pub const ssHoverJukeRampTime: usize = 928;
    pub const ssBrakingFrictionMphps: usize = 932;
    pub const ssFlyRedirectVelocityScale: usize = 936;
    pub const ssFlySlowRedirectVelocityScale: usize = 940;
    pub const ssHoverRedirectVelocityScale: usize = 944;
    pub const ssFlyMinSpeedRatio: usize = 948;
    pub const ssProjectVelocityDesiresToXY: usize = 952;
    pub const ssEnforceMinGroundClearance: usize = 953;
    pub const ssMinGroundClearanceHeight: usize = 956;
    pub const ssMinGroundClearanceLiftForce: usize = 960;
    pub const ssTerrainModeMovement: usize = 964;
    pub const ssTerrainModeGravity: usize = 968;
    pub const ssTerrainModeFlyTime: usize = 972;
    pub const ssAimOffsetClampDeg: usize = 976;
    pub const ssPitchUpSoftLimitDeg: usize = 984;
    pub const ssPitchDownSoftLimitDeg: usize = 992;
    pub const ssClutchPitchSpeedScale: usize = 1000;
    pub const ssFlyAimSpeedDps: usize = 1004;
    pub const ssFlySlowAimSpeedDps: usize = 1012;
    pub const ssHoverAimSpeedDps: usize = 1020;
    pub const ssFlyAimDriftScale: usize = 1028;
    pub const ssFlySlowAimDriftScale: usize = 1036;
    pub const ssHoverAimDriftScale: usize = 1044;
    pub const ssMinSpringOscillation: usize = 1052;
    pub const ssMaxSpringOscillation: usize = 1056;
    pub const ssMinSpringStrength: usize = 1060;
    pub const ssMaxSpringStrength: usize = 1064;
    pub const ssMinSpringAngSpeed: usize = 1068;
    pub const ssMaxSpringAngSpeed: usize = 1072;
    pub const ssSpringControllerContribution: usize = 1076;
    pub const ssTiltSpringOscillation: usize = 1080;
    pub const ssTiltSpringStrength: usize = 1084;
    pub const ssTiltSpringAngleContribution: usize = 1088;
    pub const ssADSToLockOn: usize = 1092;
    pub const ssLockOnMinAngle: usize = 1096;
    pub const ssLockOnMaxAngle: usize = 1100;
    pub const ssPilotAssistNoLockAimScale: usize = 1104;
    pub const ssPilotAssistNoLockMinAngle: usize = 1108;
    pub const ssPilotAssistNoLockMaxAngle: usize = 1112;
    pub const ssPilotAssistAimScale: usize = 1116;
    pub const ssPilotAssistMinAngle: usize = 1120;
    pub const ssPilotAssistMaxAngle: usize = 1124;
    pub const ssAutoLevelStrengthHover: usize = 1128;
    pub const ssAutoLevelStrengthFly: usize = 1136;
    pub const ssIsRollRightStick: usize = 1144;
    pub const ssRollMaxSpeedDps: usize = 1148;
    pub const ssBoostSpeedScale: usize = 1152;
    pub const ssBoostThrustScale: usize = 1156;
    pub const ssEnableBoostToFly: usize = 1160;
    pub const ssBoostTime: usize = 1164;
    pub const ssBoostRegenTime: usize = 1168;
    pub const ssBoostRegenStallTime: usize = 1172;
    pub const ssBoostMinThresholdRatio: usize = 1176;
    pub const ssCollisionRadiusI: usize = 1180;
    pub const ssCylinderHeightI: usize = 1184;
    pub const ssUseCylinder: usize = 1188;
    pub const ssUsePlayerClip: usize = 1189;
    pub const ssCollisionOffsetI: usize = 1192;
    pub const ssCollisionHardThresholdMph: usize = 1196;
    pub const ssFOVFlyOffsetDeg: usize = 1200;
    pub const ssFOVFlyBoostOffsetDeg: usize = 1204;
    pub const ssFOVHoverOffsetDeg: usize = 1208;
    pub const ssZoomBlendSpeedDps: usize = 1212;
    pub const ssZoomMaxAngle: usize = 1216;
    pub const ssZoomMinDistI: usize = 1220;
    pub const ssZoomIdealDistI: usize = 1224;
    pub const ssZoomMaxDistI: usize = 1228;
    pub const ssShowPilotViewmodel: usize = 1232;
    pub const ssCameraAnimScale: usize = 1236;
    pub const ssFlyTiltShipAngles: usize = 1240;
    pub const ssFlyTiltCameraScale: usize = 1248;
    pub const ssHoverTiltShipAngles: usize = 1252;
    pub const ssHoverTiltCameraScale: usize = 1260;
    pub const ssMotionIdleRateMinSpeed: usize = 1264;
    pub const ssMotionIdleRateMaxSpeed: usize = 1268;
    pub const ssMotionIdleMinRate: usize = 1272;
    pub const ssMotionIdleMaxRate: usize = 1276;
    pub const ssAnimTree: usize = 1280;
    pub const ssAnimParts: usize = 1288;
    pub const ssThrustFxLoop: usize = 1816;
    pub const ssThrustFxLoop__u: usize = 1816;
    pub const ssThrustFxLoop__u__fx: usize = 1816;
    pub const ssThrustFxLoop__u__vfx: usize = 1816;
    pub const ssThrustFxLoop__u__data: usize = 1816;
    pub const ssThrustFxLoop__type: usize = 1824;
    pub const ssJukeFx: usize = 1832;
    pub const ssJukeFx__u: usize = 1832;
    pub const ssJukeFx__u__fx: usize = 1832;
    pub const ssJukeFx__u__vfx: usize = 1832;
    pub const ssJukeFx__u__data: usize = 1832;
    pub const ssJukeFx__type: usize = 1840;
    pub const ssIdleRumble: usize = 1848;
    pub const ssSmallRumble: usize = 1856;
    pub const ssMedRumble: usize = 1864;
    pub const ssLargeRumble: usize = 1872;
    pub const rattleLoop: usize = 1880;
    pub const rattleLoopMinVolume: usize = 1888;
    pub const rattleLoopMaxVolume: usize = 1892;
    pub const airLoop: usize = 1896;
    pub const airLoopMinVolume: usize = 1904;
    pub const airLoopMaxVolume: usize = 1908;
    pub const engineLoop: usize = 1912;
    pub const engineLoopMinVolume: usize = 1920;
    pub const engineLoopMaxVolume: usize = 1924;
    pub const engineLoopMinPitch: usize = 1928;
    pub const engineLoopMaxPitch: usize = 1932;
    pub const hoverLoop: usize = 1936;
    pub const hoverLoopMinVolume: usize = 1944;
    pub const hoverLoopMaxVolume: usize = 1948;
    pub const hoverLoopMinPitch: usize = 1952;
    pub const hoverLoopMaxPitch: usize = 1956;
    pub const boostLoop: usize = 1960;
    pub const boostLoopMaxVolume: usize = 1968;
    pub const boostLoopMaxPitch: usize = 1972;
    pub const ssThrustLoop: usize = 1976;
    pub const ssThrustLoopMaxVolume: usize = 1984;
    pub const ssThrustLoopMaxPitch: usize = 1988;
    pub const boostStart: usize = 1992;
    pub const boostStop: usize = 2000;
    pub const boostDepleted: usize = 2008;
    pub const boostUnavailable: usize = 2016;
    pub const jukeLeft: usize = 2024;
    pub const jukeRight: usize = 2032;
    pub const jukeUpDown: usize = 2040;
    pub const jukeBack: usize = 2048;
    pub const jukeFront: usize = 2056;
    pub const flightOn: usize = 2064;
    pub const flightOff: usize = 2072;
    pub const flightOnTimer: usize = 2080;
    pub const flightOffTimer: usize = 2084;
    pub const hardCollision: usize = 2088;
    pub const softCollision: usize = 2096;
    pub const camLookEnabled: usize = 2104;
    pub const camRelativeControl: usize = 2108;
    pub const camRemoteDrive: usize = 2112;
    pub const camLerp: usize = 2116;
    pub const camHeight: usize = 2120;
    pub const camRadius: usize = 2124;
    pub const camPitchInfluence: usize = 2128;
    pub const camYawInfluence: usize = 2132;
    pub const camRollInfluence: usize = 2136;
    pub const camFovIncrease: usize = 2140;
    pub const camFovOffset: usize = 2144;
    pub const camFovSpeed: usize = 2148;
    pub const camReturnSpeed: usize = 2152;
    pub const camReturnLerp: usize = 2156;
    pub const camVehicleAnglePitchRate: usize = 2160;
    pub const camVehicleAngleYawRate: usize = 2164;
    pub const camVehicleAngleRollRate: usize = 2168;
    pub const vehCam_UseGDT: usize = 2172;
    pub const vehCam_anglesPitch: usize = 2176;
    pub const vehCam_anglesYaw: usize = 2180;
    pub const vehCam_anglesRoll: usize = 2184;
    pub const vehCam_offsetX: usize = 2188;
    pub const vehCam_offsetY: usize = 2192;
    pub const vehCam_offsetZ: usize = 2196;
    pub const vehCam_radius: usize = 2200;
    pub const vehCam_speedInfluence: usize = 2204;
    pub const vehCam_pitchTurnRate: usize = 2208;
    pub const vehCam_pitchClamp: usize = 2212;
    pub const vehCam_yawTurnRate: usize = 2216;
    pub const vehCam_yawClamp: usize = 2220;
    pub const vehCam_zOffsetMode: usize = 2224;
    pub const vehCam_anglesPitch3P: usize = 2228;
    pub const vehCam_anglesYaw3P: usize = 2232;
    pub const vehCam_anglesRoll3P: usize = 2236;
    pub const vehCam_offsetX3P: usize = 2240;
    pub const vehCam_offsetY3P: usize = 2244;
    pub const vehCam_offsetZ3P: usize = 2248;
    pub const vehCam_radius3P: usize = 2252;
    pub const vehCam_speedInfluence3P: usize = 2256;
    pub const vehCam_pitchTurnRate3P: usize = 2260;
    pub const vehCam_pitchClamp3P: usize = 2264;
    pub const vehCam_yawTurnRate3P: usize = 2268;
    pub const vehCam_yawClamp3P: usize = 2272;
    pub const vehCam_zOffsetMode3P: usize = 2276;
    pub const turretWeaponName: usize = 2280;
    pub const turretWeapon: usize = 2288;
    pub const turretHorizSpanLeft: usize = 2296;
    pub const turretHorizSpanRight: usize = 2300;
    pub const turretVertSpanUp: usize = 2304;
    pub const turretVertSpanDown: usize = 2308;
    pub const turretHorizResistLeft: usize = 2312;
    pub const turretHorizResistRight: usize = 2316;
    pub const turretVertResistUp: usize = 2320;
    pub const turretVertResistDown: usize = 2324;
    pub const turretRotRate: usize = 2328;
    pub const turretFireType: usize = 2332;
    pub const turretSpinSnd: usize = 2336;
    pub const turretStopSnd: usize = 2344;
    pub const trophyEnabled: usize = 2352;
    pub const trophyRadius: usize = 2356;
    pub const trophyInactiveRadius: usize = 2360;
    pub const trophyAmmoCount: usize = 2364;
    pub const trophyReloadTime: usize = 2368;
    pub const trophyTags: usize = 2372;
    pub const trophyExplodeFx: usize = 2392;
    pub const trophyExplodeFx__u: usize = 2392;
    pub const trophyExplodeFx__u__fx: usize = 2392;
    pub const trophyExplodeFx__u__vfx: usize = 2392;
    pub const trophyExplodeFx__u__data: usize = 2392;
    pub const trophyExplodeFx__type: usize = 2400;
    pub const trophyFlashFx: usize = 2408;
    pub const trophyFlashFx__u: usize = 2408;
    pub const trophyFlashFx__u__fx: usize = 2408;
    pub const trophyFlashFx__u__vfx: usize = 2408;
    pub const trophyFlashFx__u__data: usize = 2408;
    pub const trophyFlashFx__type: usize = 2416;
    pub const compassFriendlyIcon: usize = 2424;
    pub const compassEnemyIcon: usize = 2432;
    pub const compassFriendlyAltIcon: usize = 2440;
    pub const compassEnemyAltIcon: usize = 2448;
    pub const compassIconWidth: usize = 2456;
    pub const compassIconHeight: usize = 2460;
    pub const idleLowSnd: usize = 2464;
    pub const idleHighSnd: usize = 2472;
    pub const engineLowSnd: usize = 2480;
    pub const engineHighSnd: usize = 2488;
    pub const engineSndSpeed: usize = 2496;
    pub const audioOriginTag: usize = 2500;
    pub const engineStartUpSnd: usize = 2504;
    pub const engineStartUpLength: usize = 2512;
    pub const engineShutdownSnd: usize = 2520;
    pub const engineIdleSnd: usize = 2528;
    pub const engineSustainSnd: usize = 2536;
    pub const engineRampUpSnd: usize = 2544;
    pub const engineRampUpLength: usize = 2552;
    pub const engineRampDownSnd: usize = 2560;
    pub const engineRampDownLength: usize = 2568;
    pub const suspensionSoftSnd: usize = 2576;
    pub const suspensionSoftCompression: usize = 2584;
    pub const suspensionHardSnd: usize = 2592;
    pub const suspensionHardCompression: usize = 2600;
    pub const collisionSnd: usize = 2608;
    pub const collisionBlendSpeed: usize = 2616;
    pub const speedSnd: usize = 2624;
    pub const speedSndBlendSpeed: usize = 2632;
    pub const surfaceSndName: usize = 2640;
    pub const surfaceSndBlendSpeed: usize = 2648;
    pub const slideVolume: usize = 2652;
    pub const slideBlendSpeed: usize = 2656;
    pub const inAirPitch: usize = 2660;
    pub const soundTriggerOverrideZone: usize = 2664;
    pub const soundTriggerOverrideReverb: usize = 2672;
    pub const soundTriggerOverrideMix: usize = 2673;
    pub const soundTriggerOverrideFilter: usize = 2674;
    pub const soundTriggerOverrideOcclusion: usize = 2675;
    pub const soundTriggerOverrideAmbient: usize = 2676;
    pub const soundTriggerOverrideAmbientEvents: usize = 2677;
    pub const soundTriggerOverrideADSR: usize = 2678;
    pub const visionBlendTime: usize = 2680;
    pub const globalVisionSettings: usize = 2688;
    pub const mapVisionSettings: usize = 2696;
}
pub mod VehiclePhysicsDef {
    pub const SIZE: usize = 448;
    pub const ALIGN: usize = 4;
    pub const physicsEnabled: usize = 0;
    pub const unk_1: usize = 4;
    pub const unk_2: usize = 8;
    pub const unk_3: usize = 12;
    pub const unk_4: usize = 16;
    pub const unk_5: usize = 20;
    pub const unk_6: usize = 24;
    pub const unk_7: usize = 28;
    pub const unk_8: usize = 32;
    pub const unk_9: usize = 36;
    pub const unk_10: usize = 40;
    pub const unk_11: usize = 44;
    pub const unk_12: usize = 48;
    pub const unk_13: usize = 52;
    pub const unk_14: usize = 56;
    pub const unk_15: usize = 60;
    pub const unk_16: usize = 64;
    pub const unk_17: usize = 68;
    pub const unk_18: usize = 72;
    pub const unk_19: usize = 76;
    pub const unk_20: usize = 80;
    pub const unk_21: usize = 84;
    pub const unk_22: usize = 88;
    pub const unk_23: usize = 92;
    pub const unk_24: usize = 96;
    pub const unk_25: usize = 100;
    pub const unk_26: usize = 104;
    pub const unk_27: usize = 108;
    pub const unk_28: usize = 112;
    pub const unk_29: usize = 116;
    pub const unk_30: usize = 120;
    pub const unk_31: usize = 124;
    pub const unk_32: usize = 128;
    pub const unk_33: usize = 132;
    pub const unk_34: usize = 136;
    pub const unk_35: usize = 140;
    pub const unk_36: usize = 144;
    pub const unk_37: usize = 148;
    pub const unk_38: usize = 152;
    pub const unk_39: usize = 156;
    pub const unk_40: usize = 160;
    pub const unk_41: usize = 164;
    pub const unk_42: usize = 168;
    pub const unk_43: usize = 172;
    pub const unk_44: usize = 176;
    pub const unk_45: usize = 180;
    pub const unk_46: usize = 184;
    pub const unk_47: usize = 188;
    pub const unk_48: usize = 192;
    pub const unk_49: usize = 196;
    pub const unk_50: usize = 200;
    pub const unk_51: usize = 204;
    pub const unk_52: usize = 208;
    pub const unk_53: usize = 212;
    pub const unk_54: usize = 216;
    pub const unk_55: usize = 220;
    pub const unk_56: usize = 224;
    pub const unk_57: usize = 228;
    pub const unk_58: usize = 232;
    pub const unk_59: usize = 236;
    pub const unk_60: usize = 240;
    pub const unk_61: usize = 244;
    pub const unk_62: usize = 248;
    pub const unk_63: usize = 252;
    pub const unk_64: usize = 256;
    pub const unk_65: usize = 260;
    pub const unk_66: usize = 264;
    pub const unk_67: usize = 268;
    pub const unk_68: usize = 272;
    pub const unk_69: usize = 276;
    pub const unk_70: usize = 280;
    pub const unk_71: usize = 284;
    pub const unk_72: usize = 288;
    pub const unk_73: usize = 292;
    pub const unk_74: usize = 296;
    pub const unk_75: usize = 300;
    pub const unk_76: usize = 304;
    pub const unk_77: usize = 308;
    pub const unk_78: usize = 312;
    pub const unk_79: usize = 316;
    pub const unk_80: usize = 320;
    pub const unk_81: usize = 324;
    pub const unk_82: usize = 328;
    pub const unk_83: usize = 332;
    pub const unk_84: usize = 336;
    pub const unk_85: usize = 340;
    pub const unk_86: usize = 344;
    pub const unk_87: usize = 348;
    pub const unk_88: usize = 352;
    pub const unk_89: usize = 356;
    pub const unk_90: usize = 360;
    pub const unk_91: usize = 364;
    pub const unk_92: usize = 368;
    pub const unk_93: usize = 372;
    pub const unk_94: usize = 376;
    pub const unk_95: usize = 380;
    pub const unk_96: usize = 384;
    pub const unk_97: usize = 388;
    pub const unk_98: usize = 392;
    pub const unk_99: usize = 396;
    pub const unk_100: usize = 400;
    pub const unk_101: usize = 404;
    pub const unk_102: usize = 408;
    pub const unk_103: usize = 412;
    pub const unk_104: usize = 416;
    pub const unk_105: usize = 420;
    pub const unk_106: usize = 424;
    pub const unk_107: usize = 428;
    pub const unk_108: usize = 432;
    pub const unk_109: usize = 436;
    pub const unk_110: usize = 440;
    pub const unk_111: usize = 444;
}
pub mod VirtualLeaderboardDef {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const sourceName: usize = 8;
    pub const id: usize = 16;
    pub const sourceId: usize = 20;
    pub const columns: usize = 24;
    pub const columnCount: usize = 32;
    pub const rankColIdX: usize = 36;
    pub const rankColIdY: usize = 40;
}
pub mod WeaponAnimPackage {
    pub const SIZE: usize = 64;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const anims: usize = 8;
    pub const timers: usize = 16;
    pub const meleeAnimType: usize = 24;
    pub const meleeAnimPrimaryCount: usize = 28;
    pub const meleeAnimAltCount: usize = 32;
    pub const fireAnimTimesMs: usize = 36;
    pub const pad: usize = 52;
    pub const footstep: usize = 56;
}
pub mod WeaponAnimPackageStateTimers {
    pub const SIZE: usize = 206;
    pub const ALIGN: usize = 2;
    pub const iFireDelay: usize = 0;
    pub const iMeleeDelay: usize = 2;
    pub const iDetonateDelay: usize = 4;
    pub const iFirstRaiseTime: usize = 6;
    pub const iAltRaiseTime: usize = 8;
    pub const iAltRaiseADSTime: usize = 10;
    pub const iAltRaiseAkimboTime: usize = 12;
    pub const iRechamberTime: usize = 14;
    pub const rechamberTimeOneHanded: usize = 16;
    pub const iRechamberBoltTime: usize = 18;
    pub const iHoldFireTime: usize = 20;
    pub const iHoldPrimeTime: usize = 22;
    pub const iDetonateTime: usize = 24;
    pub const iMeleeTime: usize = 26;
    pub const meleeChargeTime: usize = 28;
    pub const meleeChargeDelay: usize = 30;
    pub const altMeleeTime: usize = 32;
    pub const altMeleeChargeTime: usize = 34;
    pub const altMeleeChargeDelay: usize = 36;
    pub const iReloadTime: usize = 38;
    pub const iFastReloadTime: usize = 40;
    pub const reloadShowRocketTime: usize = 42;
    pub const iReloadEmptyTime: usize = 44;
    pub const iFastReloadEmptyTime: usize = 46;
    pub const iReloadAddTime: usize = 48;
    pub const iFastReloadAddTime: usize = 50;
    pub const iReloadEmptyAddTime: usize = 52;
    pub const iFastReloadEmptyAddTime: usize = 54;
    pub const iReloadStartTime: usize = 56;
    pub const iAltOverride3PInTime: usize = 58;
    pub const iAltOverride3POutTime: usize = 60;
    pub const iReloadStartAddTime: usize = 62;
    pub const iFastReloadStartAddTime: usize = 64;
    pub const iReloadEndTime: usize = 66;
    pub const iFastReloadEndTime: usize = 68;
    pub const iDropTime: usize = 70;
    pub const iRaiseTime: usize = 72;
    pub const iAltDropTime: usize = 74;
    pub const iAltDropADSTime: usize = 76;
    pub const iAltDropAkimboTime: usize = 78;
    pub const quickDropTime: usize = 80;
    pub const quickRaiseTime: usize = 82;
    pub const iBreachRaiseTime: usize = 84;
    pub const iEmptyRaiseTime: usize = 86;
    pub const iEmptyDropTime: usize = 88;
    pub const sprintInTime: usize = 90;
    pub const sprintLoopTime: usize = 92;
    pub const sprintOutTime: usize = 94;
    pub const walkTime: usize = 96;
    pub const gungHoTime: usize = 98;
    pub const crawlTime: usize = 100;
    pub const stunnedTimeBegin: usize = 102;
    pub const stunnedTimeLoop: usize = 104;
    pub const stunnedTimeEnd: usize = 106;
    pub const nightVisionWearTime: usize = 108;
    pub const nightVisionWearTimeFadeOutEnd: usize = 110;
    pub const nightVisionWearTimePowerUp: usize = 112;
    pub const nightVisionRemoveTime: usize = 114;
    pub const nightVisionRemoveTimePowerDown: usize = 116;
    pub const nightVisionRemoveTimeFadeInStart: usize = 118;
    pub const primeTime: usize = 120;
    pub const bHoldFullPrime: usize = 122;
    pub const bAltOverride3PADSOnly: usize = 123;
    pub const blastFrontTime: usize = 124;
    pub const blastRightTime: usize = 126;
    pub const blastBackTime: usize = 128;
    pub const blastLeftTime: usize = 130;
    pub const slideInTime: usize = 132;
    pub const slideLoopTime: usize = 134;
    pub const slideOutTime: usize = 136;
    pub const dodgeTime: usize = 138;
    pub const wallRunInTime: usize = 140;
    pub const wallRunOutTime: usize = 142;
    pub const leapInTime: usize = 144;
    pub const leapLoopTime: usize = 146;
    pub const leapOutTime: usize = 148;
    pub const leapCancelTime: usize = 150;
    pub const chargeInTime: usize = 152;
    pub const chargeLoopTime: usize = 154;
    pub const chargeOutTime: usize = 156;
    pub const iRegenerationTime: usize = 158;
    pub const iRegenerationAddTime: usize = 160;
    pub const iReloadInterruptTime: usize = 162;
    pub const iReloadEmptyInterruptTime: usize = 164;
    pub const iReloadEndInterruptTime: usize = 166;
    pub const iReloadMultiple1InterruptTime: usize = 168;
    pub const iReloadMultiple2InterruptTime: usize = 170;
    pub const iReloadMultiple3InterruptTime: usize = 172;
    pub const iReloadMultiple4InterruptTime: usize = 174;
    pub const iReloadMultiple5InterruptTime: usize = 176;
    pub const iReloadMultiple6InterruptTime: usize = 178;
    pub const iReloadMultiple7InterruptTime: usize = 180;
    pub const iReloadMultiple8InterruptTime: usize = 182;
    pub const iFastReloadInterruptTime: usize = 184;
    pub const iFastReloadEmptyInterruptTime: usize = 186;
    pub const iFastReloadEndInterruptTime: usize = 188;
    pub const iFastReloadMultiple1InterruptTime: usize = 190;
    pub const iFastReloadMultiple2InterruptTime: usize = 192;
    pub const iFastReloadMultiple3InterruptTime: usize = 194;
    pub const iFastReloadMultiple4InterruptTime: usize = 196;
    pub const iFastReloadMultiple5InterruptTime: usize = 198;
    pub const iFastReloadMultiple6InterruptTime: usize = 200;
    pub const iFastReloadMultiple7InterruptTime: usize = 202;
    pub const iFastReloadMultiple8InterruptTime: usize = 204;
}
pub mod WeaponAttachment {
    pub const SIZE: usize = 536;
    pub const ALIGN: usize = 8;
    pub const szInternalName: usize = 0;
    pub const name: usize = 0;
    pub const szDisplayName: usize = 8;
    pub const internalName: usize = 16;
    pub const papAttachment: usize = 20;
    pub const r#type: usize = 24;
    pub const weaponType: usize = 28;
    pub const weapClass: usize = 32;
    pub const numReticles: usize = 36;
    pub const numAltReticles: usize = 40;
    pub const unk: usize = 44;
    pub const worldModelCamo: usize = 48;
    pub const viewModelCamo: usize = 56;
    pub const worldModelVariations: usize = 64;
    pub const viewModelVariations: usize = 72;
    pub const numModelVariations: usize = 80;
    pub const worldModelCamoMaterials: usize = 88;
    pub const viewModelCamoMaterials: usize = 96;
    pub const numWorldModelCamoMaterials: usize = 104;
    pub const numViewModelCamoMaterials: usize = 108;
    pub const attachPoint: usize = 112;
    pub const reticles: usize = 120;
    pub const altReticles: usize = 128;
    pub const ammogeneral: usize = 136;
    pub const tracer: usize = 144;
    pub const penetration: usize = 152;
    pub const sight: usize = 160;
    pub const holdBreath: usize = 168;
    pub const reload: usize = 176;
    pub const addOns: usize = 184;
    pub const laser: usize = 192;
    pub const general: usize = 200;
    pub const aimAssist: usize = 208;
    pub const targetAssist: usize = 216;
    pub const unkAtt01: usize = 224;
    pub const ammunition: usize = 232;
    pub const damage: usize = 240;
    pub const idleSettings: usize = 248;
    pub const adsSettings: usize = 256;
    pub const adsSettingsMain: usize = 264;
    pub const scopeDriftSettings: usize = 272;
    pub const scopeDriftSettingsMain: usize = 280;
    pub const hipSpread: usize = 288;
    pub const slideSpread: usize = 296;
    pub const gunKick: usize = 304;
    pub const viewKick: usize = 312;
    pub const adsOverlay: usize = 320;
    pub const outline: usize = 328;
    pub const ui: usize = 336;
    pub const rumbles: usize = 344;
    pub const projectile: usize = 352;
    pub const charged: usize = 360;
    pub const adsAltSwitch: usize = 368;
    pub const regeneration: usize = 376;
    pub const movement: usize = 384;
    pub const burst: usize = 392;
    pub const locationDamage: usize = 400;
    pub const ammunitionScale: usize = 408;
    pub const damageScale: usize = 412;
    pub const stateTimersScale: usize = 416;
    pub const reloadTimersScale: usize = 420;
    pub const fireTimeOverride: usize = 424;
    pub const fireTimeAkimboOverride: usize = 428;
    pub const fireTimersScale: usize = 432;
    pub const burstFireCooldownScale: usize = 436;
    pub const idleHipMotionScale: usize = 440;
    pub const idleADSMotionScale: usize = 444;
    pub const idleADSMotionTimeScale: usize = 448;
    pub const adsSettingsScale: usize = 452;
    pub const adsSettingsScaleMain: usize = 456;
    pub const hipSpreadScale: usize = 460;
    pub const slideSpreadScale: usize = 464;
    pub const gunKickScale: usize = 468;
    pub const viewKickScale: usize = 472;
    pub const viewCenterScale: usize = 476;
    pub const adsSwayScale: usize = 480;
    pub const adsSwayScale2: usize = 484;
    pub const adsSwayScale3: usize = 488;
    pub const iAmmoIndex: usize = 492;
    pub const loadIndex: usize = 496;
    pub const useAlternateViewModelLaser: usize = 500;
    pub const hideIronSightsWithThisAttachment: usize = 501;
    pub const shareAmmoWithAlt: usize = 502;
    pub const useLeftTriggerAltFire: usize = 503;
    pub const containsIKTag: usize = 504;
    pub const isAkimboAttachment: usize = 505;
    pub const attachLeftWeaponAkimbo: usize = 506;
    pub const attachRightWeaponAkimbo: usize = 507;
    pub const luiWeaponInfoWidgetUsesScopeStencil: usize = 508;
    pub const szAltModeName: usize = 512;
    pub const szLUIWeaponInfoWidgetName: usize = 520;
    pub const szLUIWeaponInfoWidgetTag: usize = 528;
}
pub mod WeaponCompleteDef {
    pub const SIZE: usize = 312;
    pub const ALIGN: usize = 8;
    pub const szInternalName: usize = 0;
    pub const name: usize = 0;
    pub const weapDef: usize = 8;
    pub const szDisplayName: usize = 16;
    pub const szLootTable: usize = 24;
    pub const hideTags: usize = 32;
    pub const numAttachments: usize = 40;
    pub const numAttachments2: usize = 44;
    pub const numAttachments3: usize = 48;
    pub const numAttachments4: usize = 52;
    pub const numAttachments5: usize = 56;
    pub const numAttachments6: usize = 60;
    pub const numLootVariants: usize = 64;
    pub const pad: usize = 68;
    pub const attachments: usize = 72;
    pub const attachments2: usize = 80;
    pub const attachments3: usize = 88;
    pub const attachments4: usize = 96;
    pub const attachments5: usize = 104;
    pub const attachments6: usize = 112;
    pub const numNotetrackSoundMappings: usize = 120;
    pub const pad2: usize = 124;
    pub const notetrackSoundMapKeys: usize = 128;
    pub const notetrackSoundMapValues: usize = 136;
    pub const numNotetrackOverrides: usize = 144;
    pub const notetrackOverrides: usize = 152;
    pub const numNotetrackSuitEntries: usize = 160;
    pub const notetrackSuitEntries: usize = 168;
    pub const fAdsZoomFov: usize = 176;
    pub const iAdsTransInTime: usize = 180;
    pub const iAdsTransOutTime: usize = 184;
    pub const iClipSize: usize = 188;
    pub const vfxImpactType: usize = 192;
    pub const sfxImpactType: usize = 196;
    pub const meleeImpactType: usize = 200;
    pub const penetrateMultiplier: usize = 204;
    pub const fAdsViewKickCenterSpeed: usize = 208;
    pub const fHipViewKickCenterSpeed: usize = 212;
    pub const szAltWeaponName: usize = 216;
    pub const killIcon: usize = 224;
    pub const dpadIcon: usize = 232;
    pub const ammoDropStockMax: usize = 240;
    pub const adsDofStart: usize = 244;
    pub const adsDofEnd: usize = 248;
    pub const accuracyGraphKnotCount: usize = 252;
    pub const accuracyGraphKnots: usize = 256;
    pub const motionTracker: usize = 272;
    pub const enhanced: usize = 273;
    pub const dpadIconShowsAmmo: usize = 274;
    pub const luiWeaponInfoWidgetUsesScopeStencil: usize = 275;
    pub const szAdsrBaseSetting: usize = 280;
    pub const szLUIWeaponInfoWidgetName: usize = 288;
    pub const szLUIWeaponInfoWidgetTag: usize = 296;
    pub const szLUICrosshairWidget: usize = 304;
}
pub mod WeaponDef {
    pub const SIZE: usize = 5144;
    pub const ALIGN: usize = 8;
    pub const szOverlayName: usize = 0;
    pub const gunXModel: usize = 8;
    pub const gunXModelLeftHand: usize = 16;
    pub const gunXModelRightHand: usize = 24;
    pub const defaultViewModel: usize = 32;
    pub const defaultWorldModelLeftHand: usize = 40;
    pub const defaultWorldModelRightHand: usize = 48;
    pub const worldModel: usize = 56;
    pub const worldXModelLeftHand: usize = 64;
    pub const worldXModelRightHand: usize = 72;
    pub const defaultWorldModel: usize = 80;
    pub const playerShadowModel: usize = 88;
    pub const playerShadowModelLeftHand: usize = 96;
    pub const playerShadowModelRightHand: usize = 104;
    pub const handXModel: usize = 112;
    pub const complexWorldModel: usize = 120;
    pub const complexWorldModelLeftHand: usize = 128;
    pub const complexWorldModelRightHand: usize = 136;
    pub const complexGunXModel: usize = 144;
    pub const complexGunXModelLeftHand: usize = 152;
    pub const complexGunXModelRightHand: usize = 160;
    pub const numComplexModels: usize = 168;
    pub const pad: usize = 172;
    pub const camoWorldModelMaterialOverride: usize = 176;
    pub const camoViewModelMaterialOverride: usize = 304;
    pub const unk_0: usize = 432;
    pub const unk_1: usize = 436;
    pub const unk_2: usize = 440;
    pub const unk_3: usize = 444;
    pub const szXAnims: usize = 448;
    pub const szXAnimsRightHanded: usize = 456;
    pub const szXAnimsLeftHanded: usize = 464;
    pub const unk_4_0: usize = 472;
    pub const unk_4_1: usize = 473;
    pub const pad17: usize = 474;
    pub const iFireTime: usize = 476;
    pub const iFireDelay: usize = 480;
    pub const iFireTimeAkimbo: usize = 484;
    pub const iFireDelayAkimbo: usize = 488;
    pub const pad3: usize = 492;
    pub const szModeName: usize = 496;
    pub const notetrackRumbleMapKeys: usize = 504;
    pub const notetrackRumbleMapValues: usize = 512;
    pub const notetrackFXMapKeys: usize = 520;
    pub const notetrackFXMapValues: usize = 528;
    pub const notetrackFXMapTagValues: usize = 536;
    pub const playerAnimType: usize = 544;
    pub const scriptedAnimEvent: usize = 548;
    pub const scriptedAnimType: usize = 552;
    pub const weapType: usize = 556;
    pub const whizbyType: usize = 560;
    pub const weapClass: usize = 564;
    pub const penetrateType: usize = 568;
    pub const deathAnimDamageType: usize = 572;
    pub const inventoryType: usize = 576;
    pub const fireType: usize = 580;
    pub const burstCount: usize = 584;
    pub const targetAssistType: usize = 588;
    pub const targetAssistBehavior: usize = 592;
    pub const pad28: usize = 596;
    pub const unkMaterial: usize = 600;
    pub const unk_18: usize = 608;
    pub const unk_19: usize = 612;
    pub const unk_20: usize = 616;
    pub const unk_21: usize = 620;
    pub const unk_22: usize = 624;
    pub const unk_23: usize = 628;
    pub const unk_24: usize = 632;
    pub const unk_25: usize = 636;
    pub const unk_26: usize = 640;
    pub const unk_27: usize = 644;
    pub const unk_28: usize = 648;
    pub const unk_29: usize = 652;
    pub const unk_30: usize = 656;
    pub const unk_31: usize = 660;
    pub const unk_32: usize = 664;
    pub const unk_33: usize = 668;
    pub const unk_34: usize = 672;
    pub const unk_35: usize = 676;
    pub const unk_36: usize = 680;
    pub const unk_37: usize = 684;
    pub const unk_38: usize = 688;
    pub const unk_39: usize = 692;
    pub const unk_40: usize = 696;
    pub const unk_41: usize = 700;
    pub const unk_42: usize = 704;
    pub const unk_43: usize = 708;
    pub const unk_44: usize = 712;
    pub const unk_45: usize = 716;
    pub const unk_46: usize = 720;
    pub const unk_47: usize = 724;
    pub const unk_48: usize = 728;
    pub const unk_49: usize = 732;
    pub const unk_50: usize = 736;
    pub const unk_51: usize = 740;
    pub const unk_52: usize = 744;
    pub const unk_53: usize = 748;
    pub const unk_54: usize = 752;
    pub const unk_55: usize = 756;
    pub const unk_56: usize = 760;
    pub const unk_57: usize = 764;
    pub const unk_58: usize = 768;
    pub const unk_59: usize = 772;
    pub const unk_60: usize = 776;
    pub const unk_61: usize = 780;
    pub const unk_62: usize = 784;
    pub const unk_63: usize = 788;
    pub const unk_64: usize = 792;
    pub const unk_65_0: usize = 796;
    pub const unk_65_1: usize = 797;
    pub const unk_65_2: usize = 798;
    pub const unk_65_3: usize = 799;
    pub const targetAssistRange: usize = 800;
    pub const targetAssistAngle: usize = 804;
    pub const targetAssistLosOffsetForward: usize = 808;
    pub const targetAssistLosOffsetRight: usize = 812;
    pub const targetAssistLosOffsetUp: usize = 816;
    pub const targetAssistOnlyAvailableInAds: usize = 820;
    pub const unk_71_1: usize = 821;
    pub const unk_71_2: usize = 822;
    pub const unk_71_3: usize = 823;
    pub const bSegmentedReload: usize = 824;
    pub const unk_72_1: usize = 825;
    pub const unk_72_2: usize = 826;
    pub const unk_72_3: usize = 827;
    pub const burstFireCooldown: usize = 828;
    pub const burstFireAuto: usize = 832;
    pub const pad27: usize = 833;
    pub const offhandClass: usize = 836;
    pub const stance: usize = 840;
    pub const pad18: usize = 844;
    pub const vfxPackage: usize = 848;
    pub const sfxPackage: usize = 856;
    pub const reticleType: usize = 864;
    pub const pad8: usize = 868;
    pub const reticleCenter: usize = 872;
    pub const reticleSide: usize = 880;
    pub const reticleOnePiece: usize = 888;
    pub const iReticleCenterSize: usize = 896;
    pub const iReticleSideSize: usize = 900;
    pub const iReticleMinOfs: usize = 904;
    pub const reticleCenterPulse: usize = 908;
    pub const pad25: usize = 909;
    pub const activeReticleType: usize = 912;
    pub const vStandMove: usize = 916;
    pub const vStandRot: usize = 928;
    pub const strafeMove: usize = 940;
    pub const strafeRot: usize = 952;
    pub const vDuckedOfs: usize = 964;
    pub const vDuckedMove: usize = 976;
    pub const vDuckedRot: usize = 988;
    pub const vProneOfs: usize = 1000;
    pub const vProneMove: usize = 1012;
    pub const vProneRot: usize = 1024;
    pub const fPosMoveRate: usize = 1036;
    pub const fPosProneMoveRate: usize = 1040;
    pub const fStandMoveMinSpeed: usize = 1044;
    pub const fDuckedMoveMinSpeed: usize = 1048;
    pub const fProneMoveMinSpeed: usize = 1052;
    pub const fPosRotRate: usize = 1056;
    pub const fPosProneRotRate: usize = 1060;
    pub const fStandRotMinSpeed: usize = 1064;
    pub const fDuckedRotMinSpeed: usize = 1068;
    pub const fProneRotMinSpeed: usize = 1072;
    pub const pad4: usize = 1076;
    pub const worldClipModel: usize = 1080;
    pub const rocketModel: usize = 1088;
    pub const knifeModel: usize = 1096;
    pub const worldKnifeModel: usize = 1104;
    pub const hudIcon: usize = 1112;
    pub const pickupIcon: usize = 1120;
    pub const dangerIcon: usize = 1128;
    pub const throwBackIcon: usize = 1136;
    pub const hideWarningIcons: usize = 1144;
    pub const pad6: usize = 1145;
    pub const warningIconsDelay: usize = 1148;
    pub const ammoCounterIcon: usize = 1152;
    pub const ammoCounterClip: usize = 1160;
    pub const iStartAmmo: usize = 1164;
    pub const iPerkStartAmmo: usize = 1168;
    pub const iPerkMaxAmmo: usize = 1172;
    pub const szAmmoName: usize = 1176;
    pub const iAmmoIndex: usize = 1184;
    pub const iMaxAmmo: usize = 1188;
    pub const shotCount: usize = 1192;
    pub const szSharedAmmoCapName: usize = 1200;
    pub const iSharedAmmoCapIndex: usize = 1208;
    pub const iSharedAmmoCap: usize = 1212;
    pub const iAmmoUsedPerShot: usize = 1216;
    pub const requireAmmoUsedPerShot: usize = 1220;
    pub const pad24: usize = 1221;
    pub const lowAmmoWarningThreshold: usize = 1224;
    pub const disableNoAmmoWarning: usize = 1228;
    pub const pad19: usize = 1229;
    pub const damage: usize = 1232;
    pub const playerDamage: usize = 1236;
    pub const meleeDamage: usize = 1240;
    pub const iDamageType: usize = 1244;
    pub const autoAimRange: usize = 1248;
    pub const aimAssistRange: usize = 1252;
    pub const aimAssistRangeAds: usize = 1256;
    pub const aimAssistPitchSlowdown: usize = 1260;
    pub const aimAssistPitchSlowdownAds: usize = 1264;
    pub const aimAssistYawSlowdown: usize = 1268;
    pub const aimAssistYawSlowdownAds: usize = 1272;
    pub const aimAssistLockonStrength: usize = 1276;
    pub const aimPadding: usize = 1280;
    pub const enemyCrosshairRange: usize = 1284;
    pub const moveSpeedScale: usize = 1288;
    pub const adsMoveSpeedScale: usize = 1292;
    pub const sprintDurationScale: usize = 1296;
    pub const sprintRestoreDelay: usize = 1300;
    pub const fAdsZoomInFrac: usize = 1304;
    pub const fAdsZoomOutFrac: usize = 1308;
    pub const overlay: usize = 1312;
    pub const overlay__shader: usize = 1312;
    pub const overlay__shaderLowRes: usize = 1320;
    pub const overlay__shaderEMP: usize = 1328;
    pub const overlay__shaderEMPLowRes: usize = 1336;
    pub const overlay__reticle: usize = 1344;
    pub const overlay__width: usize = 1348;
    pub const overlay__height: usize = 1352;
    pub const overlay__widthSplitscreen: usize = 1356;
    pub const overlay__heightSplitscreen: usize = 1360;
    pub const overlayInterface: usize = 1368;
    pub const fAdsBobFactor: usize = 1372;
    pub const fAdsViewBobMult: usize = 1376;
    pub const fHipSpreadStandMin: usize = 1380;
    pub const fHipSpreadDuckedMin: usize = 1384;
    pub const fHipSpreadProneMin: usize = 1388;
    pub const hipSpreadSprintMin: usize = 1392;
    pub const hipSpreadInAirMin: usize = 1396;
    pub const hipSpreadStandMax: usize = 1400;
    pub const hipSpreadDuckedMax: usize = 1404;
    pub const hipSpreadProneMax: usize = 1408;
    pub const hipSpreadSprintMax: usize = 1412;
    pub const hipSpreadInAirMax: usize = 1416;
    pub const fHipSpreadDecayRate: usize = 1420;
    pub const fHipSpreadFireAdd: usize = 1424;
    pub const fHipSpreadTurnAdd: usize = 1428;
    pub const fHipSpreadMoveAdd: usize = 1432;
    pub const fHipSpreadDuckedDecay: usize = 1436;
    pub const fHipSpreadProneDecay: usize = 1440;
    pub const hipSpreadSprintDecay: usize = 1444;
    pub const hipSpreadInAirDecay: usize = 1448;
    pub const fHipReticleSidePos: usize = 1452;
    pub const fAdsIdleAmount: usize = 1456;
    pub const fHipIdleAmount: usize = 1460;
    pub const adsIdleSpeed: usize = 1464;
    pub const hipIdleSpeed: usize = 1468;
    pub const fIdleCrouchFactor: usize = 1472;
    pub const fIdleProneFactor: usize = 1476;
    pub const fGunMaxPitch: usize = 1480;
    pub const fGunMaxYaw: usize = 1484;
    pub const adsIdleLerpStartTime: usize = 1488;
    pub const adsIdleLerpTime: usize = 1492;
    pub const slideSpreadMin: usize = 1496;
    pub const slideSpreadMax: usize = 1500;
    pub const slideSpreadDecayRate: usize = 1504;
    pub const slideSpreadFireAdd: usize = 1508;
    pub const slideSpreadTurnAdd: usize = 1512;
    pub const swayMaxAngleSteadyAim: usize = 1516;
    pub const swayMaxAngle: usize = 1520;
    pub const swayLerpSpeed: usize = 1524;
    pub const swayPitchScale: usize = 1528;
    pub const swayYawScale: usize = 1532;
    pub const swayHorizScale: usize = 1536;
    pub const swayVertScale: usize = 1540;
    pub const swayShellShockScale: usize = 1544;
    pub const adsSwayMaxAngle: usize = 1548;
    pub const adsSwayLerpSpeed: usize = 1552;
    pub const adsSway_UNKNOWN_Scale: usize = 1556;
    pub const adsSwayPitchScale: usize = 1560;
    pub const adsSwayYawScale: usize = 1564;
    pub const adsSwayHorizScale: usize = 1568;
    pub const adsSwayVertScale: usize = 1572;
    pub const adsFireRateScale: usize = 1576;
    pub const adsDamageRangeScale: usize = 1580;
    pub const adsFireAnimFrac: usize = 1584;
    pub const fireTimerLerpToAdsScale: usize = 1588;
    pub const alwaysFireAtMaxRangeInAds: usize = 1592;
    pub const pad23: usize = 1593;
    pub const dualWieldViewModelOffset: usize = 1596;
    pub const fScopeDriftDelay: usize = 1600;
    pub const fScopeDriftLerpInTime: usize = 1604;
    pub const fScopeDriftSteadyTime: usize = 1608;
    pub const fScopeDriftLerpOutTime: usize = 1612;
    pub const fScopeDriftSteadyFactor: usize = 1616;
    pub const fScopeDriftUnsteadyFactor: usize = 1620;
    pub const killIconRatio: usize = 1624;
    pub const iReloadAmmoAdd: usize = 1628;
    pub const iReloadStartAdd: usize = 1632;
    pub const ammoDropStockMin: usize = 1636;
    pub const ammoDropClipPercentMin: usize = 1640;
    pub const ammoDropClipPercentMax: usize = 1644;
    pub const iExplosionRadius: usize = 1648;
    pub const iExplosionRadiusMin: usize = 1652;
    pub const iExplosionForceRadius: usize = 1656;
    pub const iExplosionInnerDamage: usize = 1660;
    pub const iExplosionOuterDamage: usize = 1664;
    pub const damageConeAngle: usize = 1668;
    pub const bulletExplDmgMult: usize = 1672;
    pub const bulletExplRadiusMult: usize = 1676;
    pub const iProjectileSpeed: usize = 1680;
    pub const iProjectileSpeedUp: usize = 1684;
    pub const iProjectileSpeedForward: usize = 1688;
    pub const iProjectileActivateDist: usize = 1692;
    pub const iProjectileDetonationRadius: usize = 1696;
    pub const projLifetime: usize = 1700;
    pub const projLifetimeStdDeviation: usize = 1704;
    pub const timeToAccelerate: usize = 1708;
    pub const projectileCurvature: usize = 1712;
    pub const pad2: usize = 1716;
    pub const projectileModel: usize = 1720;
    pub const projExplosion: usize = 1728;
    pub const projExplosionEffect: usize = 1736;
    pub const projExplosionEffect__u: usize = 1736;
    pub const projExplosionEffect__u__fx: usize = 1736;
    pub const projExplosionEffect__u__vfx: usize = 1736;
    pub const projExplosionEffect__u__data: usize = 1736;
    pub const projExplosionEffect__type: usize = 1744;
    pub const projDudEffect: usize = 1752;
    pub const projDudEffect__u: usize = 1752;
    pub const projDudEffect__u__fx: usize = 1752;
    pub const projDudEffect__u__vfx: usize = 1752;
    pub const projDudEffect__u__data: usize = 1752;
    pub const projDudEffect__type: usize = 1760;
    pub const stickiness: usize = 1768;
    pub const ricochetChance: usize = 1772;
    pub const riotShieldEnableDamage: usize = 1776;
    pub const riotShieldHealth: usize = 1780;
    pub const riotShieldDamageMult: usize = 1784;
    pub const pad7: usize = 1788;
    pub const parallelBounce: usize = 1792;
    pub const perpendicularBounce: usize = 1800;
    pub const projBodyEffect: usize = 1808;
    pub const projBodyEffect__u: usize = 1808;
    pub const projBodyEffect__u__fx: usize = 1808;
    pub const projBodyEffect__u__vfx: usize = 1808;
    pub const projBodyEffect__u__data: usize = 1808;
    pub const projBodyEffect__type: usize = 1816;
    pub const projTrailEffect: usize = 1824;
    pub const projTrailEffect__u: usize = 1824;
    pub const projTrailEffect__u__fx: usize = 1824;
    pub const projTrailEffect__u__vfx: usize = 1824;
    pub const projTrailEffect__u__data: usize = 1824;
    pub const projTrailEffect__type: usize = 1832;
    pub const projBeaconEffect: usize = 1840;
    pub const projBeaconEffect__u: usize = 1840;
    pub const projBeaconEffect__u__fx: usize = 1840;
    pub const projBeaconEffect__u__vfx: usize = 1840;
    pub const projBeaconEffect__u__data: usize = 1840;
    pub const projBeaconEffect__type: usize = 1848;
    pub const vProjectileColor: usize = 1856;
    pub const guidedMissileType: usize = 1868;
    pub const maxSteeringAccel: usize = 1872;
    pub const projIgnitionDelay: usize = 1876;
    pub const projIgnitionEffect: usize = 1880;
    pub const projIgnitionEffect__u: usize = 1880;
    pub const projIgnitionEffect__u__fx: usize = 1880;
    pub const projIgnitionEffect__u__vfx: usize = 1880;
    pub const projIgnitionEffect__u__data: usize = 1880;
    pub const projIgnitionEffect__type: usize = 1888;
    pub const fAdsAimPitch: usize = 1896;
    pub const fAdsCrosshairInFrac: usize = 1900;
    pub const fAdsCrosshairOutFrac: usize = 1904;
    pub const adsInCrosshairAlphaStart: usize = 1908;
    pub const adsInCrosshairAlphaEnd: usize = 1912;
    pub const adsOutCrosshairAlphaStart: usize = 1916;
    pub const adsOutCrosshairAlphaEnd: usize = 1920;
    pub const adsShouldShowCrosshair: usize = 1924;
    pub const adsGunKickReducedKickBullets: usize = 1928;
    pub const adsGunKickReducedKickPercent: usize = 1932;
    pub const fAdsGunKickPitchMin: usize = 1936;
    pub const fAdsGunKickPitchMax: usize = 1940;
    pub const fAdsGunKickYawMin: usize = 1944;
    pub const fAdsGunKickYawMax: usize = 1948;
    pub const fAdsGunKickMagMin: usize = 1952;
    pub const fAdsGunKickAccel: usize = 1956;
    pub const fAdsGunKickSpeedMax: usize = 1960;
    pub const fAdsGunKickSpeedDecay: usize = 1964;
    pub const fAdsGunKickStaticDecay: usize = 1968;
    pub const fAdsViewKickPitchMin: usize = 1972;
    pub const fAdsViewKickPitchMax: usize = 1976;
    pub const fAdsViewKickYawMin: usize = 1980;
    pub const fAdsViewKickYawMax: usize = 1984;
    pub const fAdsViewKickMagMin: usize = 1988;
    pub const fAdsViewScatterMin: usize = 1992;
    pub const fAdsViewScatterMax: usize = 1996;
    pub const fAdsSpread: usize = 2000;
    pub const iVisibilityAxis: usize = 2004;
    pub const fVisibilityUpOffset: usize = 2008;
    pub const hipGunKickReducedKickBullets: usize = 2012;
    pub const hipGunKickReducedKickPercent: usize = 2016;
    pub const fHipGunKickPitchMin: usize = 2020;
    pub const fHipGunKickPitchMax: usize = 2024;
    pub const fHipGunKickYawMin: usize = 2028;
    pub const fHipGunKickYawMax: usize = 2032;
    pub const fHipGunKickMagMin: usize = 2036;
    pub const fHipGunKickAccel: usize = 2040;
    pub const fHipGunKickSpeedMax: usize = 2044;
    pub const fHipGunKickSpeedDecay: usize = 2048;
    pub const fHipGunKickStaticDecay: usize = 2052;
    pub const fHipViewKickPitchMin: usize = 2056;
    pub const fHipViewKickPitchMax: usize = 2060;
    pub const fHipViewKickYawMin: usize = 2064;
    pub const fHipViewKickYawMax: usize = 2068;
    pub const fHipViewKickMagMin: usize = 2072;
    pub const fHipViewScatterMin: usize = 2076;
    pub const fHipViewScatterMax: usize = 2080;
    pub const multipleReloadClipPercentage: usize = 2084;
    pub const kickAlignedInputScalar: usize = 2088;
    pub const kickOpposedInputScalar: usize = 2092;
    pub const fightDist: usize = 2096;
    pub const maxDist: usize = 2100;
    pub const accuracyGraphName: usize = 2104;
    pub const originalAccuracyGraphKnots: usize = 2120;
    pub const originalAccuracyGraphKnotCount: usize = 2136;
    pub const iPositionReloadTransTime: usize = 2140;
    pub const leftArc: usize = 2144;
    pub const rightArc: usize = 2148;
    pub const topArc: usize = 2152;
    pub const bottomArc: usize = 2156;
    pub const softLeftRightArc: usize = 2160;
    pub const pad20: usize = 2161;
    pub const accuracy: usize = 2164;
    pub const aiSpread: usize = 2168;
    pub const playerSpread: usize = 2172;
    pub const minTurnSpeed: usize = 2176;
    pub const maxTurnSpeed: usize = 2184;
    pub const pitchConvergenceTime: usize = 2192;
    pub const yawConvergenceTime: usize = 2196;
    pub const suppressTime: usize = 2200;
    pub const maxRange: usize = 2204;
    pub const fAnimHorRotateInc: usize = 2208;
    pub const fPlayerPositionDist: usize = 2212;
    pub const szUseHintString: usize = 2216;
    pub const dropHintString: usize = 2224;
    pub const iUseHintStringIndex: usize = 2232;
    pub const dropHintStringIndex: usize = 2236;
    pub const horizViewJitter: usize = 2240;
    pub const vertViewJitter: usize = 2244;
    pub const scanSpeed: usize = 2248;
    pub const scanAccel: usize = 2252;
    pub const scanPauseTime: usize = 2256;
    pub const szScript: usize = 2264;
    pub const adsSpeedMs: usize = 2272;
    pub const minDamage: usize = 2280;
    pub const minPlayerDamage: usize = 2284;
    pub const mid1Damage: usize = 2288;
    pub const mid2Damage: usize = 2292;
    pub const mid3Damage: usize = 2296;
    pub const mid1PlayerDamage: usize = 2300;
    pub const mid2PlayerDamage: usize = 2304;
    pub const mid3PlayerDamage: usize = 2308;
    pub const maxDamageRange: usize = 2312;
    pub const mid1DamageRange: usize = 2316;
    pub const mid2DamageRange: usize = 2320;
    pub const mid3DamageRange: usize = 2324;
    pub const minDamageRange: usize = 2328;
    pub const destabilizationRateTime: usize = 2332;
    pub const destabilizationCurvatureMax: usize = 2336;
    pub const destabilizeDistance: usize = 2340;
    pub const robotechMaxPitch: usize = 2344;
    pub const robotechMaxYaw: usize = 2348;
    pub const robotechFrequency: usize = 2352;
    pub const robotechVariancePitch: usize = 2356;
    pub const robotechVarianceYaw: usize = 2360;
    pub const robotechVarianceFrequency: usize = 2364;
    pub const locationDamageMultipliers: usize = 2368;
    pub const hitLocPriorityMap: usize = 2376;
    pub const unittypeMultipliers: usize = 2384;
    pub const pad30: usize = 2420;
    pub const fireRumble: usize = 2424;
    pub const meleeImpactRumble: usize = 2432;
    pub const tracerStyle: usize = 2440;
    pub const tracerType: usize = 2448;
    pub const overchargeTracerType: usize = 2456;
    pub const laserTypeFriendly: usize = 2464;
    pub const laserTypeEnemy: usize = 2472;
    pub const turretADSEnabled: usize = 2480;
    pub const turretADSTime: usize = 2484;
    pub const turretFov: usize = 2488;
    pub const turretFovADS: usize = 2492;
    pub const turretScopeZoomRate: usize = 2496;
    pub const turretScopeZoomMin: usize = 2500;
    pub const turretScopeZoomMax: usize = 2504;
    pub const turretOverheatUpRate: usize = 2508;
    pub const turretOverheatDownRate: usize = 2512;
    pub const turretOverheatPenalty: usize = 2516;
    pub const turretOverheatSound: usize = 2520;
    pub const turretOverheatEffect: usize = 2528;
    pub const turretOverheatEffect__u: usize = 2528;
    pub const turretOverheatEffect__u__fx: usize = 2528;
    pub const turretOverheatEffect__u__vfx: usize = 2528;
    pub const turretOverheatEffect__u__data: usize = 2528;
    pub const turretOverheatEffect__type: usize = 2536;
    pub const turretBarrelSpinRumble: usize = 2544;
    pub const turretBarrelSpinSpeed: usize = 2552;
    pub const turretBarrelSpinUpTime: usize = 2556;
    pub const turretBarrelSpinDownTime: usize = 2560;
    pub const turretBarrelSpinMaxSnd: usize = 2568;
    pub const turretBarrelSpinUpSnd: usize = 2576;
    pub const turretBarrelSpinDownSnd: usize = 2608;
    pub const missileConeSoundAlias: usize = 2640;
    pub const missileConeSoundAliasAtBase: usize = 2648;
    pub const missileConeSoundRadiusAtTop: usize = 2656;
    pub const missileConeSoundRadiusAtBase: usize = 2660;
    pub const missileConeSoundHeight: usize = 2664;
    pub const missileConeSoundOriginOffset: usize = 2668;
    pub const missileConeSoundVolumescaleAtCore: usize = 2672;
    pub const missileConeSoundVolumescaleAtEdge: usize = 2676;
    pub const missileConeSoundVolumescaleCoreSize: usize = 2680;
    pub const missileConeSoundPitchAtTop: usize = 2684;
    pub const missileConeSoundPitchAtBottom: usize = 2688;
    pub const missileConeSoundPitchTopSize: usize = 2692;
    pub const missileConeSoundPitchBottomSize: usize = 2696;
    pub const missileConeSoundCrossfadeTopSize: usize = 2700;
    pub const missileConeSoundCrossfadeBottomSize: usize = 2704;
    pub const knifeAttachTagOverride: usize = 2708;
    pub const knifeAttachTagOverride2: usize = 2712;
    pub const knifeAlwaysAttached: usize = 2716;
    pub const meleeOverrideValues: usize = 2717;
    pub const pad26: usize = 2718;
    pub const aim_automelee_lerp: usize = 2720;
    pub const aim_automelee_region_height: usize = 2724;
    pub const aim_automelee_region_width: usize = 2728;
    pub const aim_automelee_maxPitchMovement: usize = 2732;
    pub const aim_automelee_maxYawMovement: usize = 2736;
    pub const player_meleeHeight: usize = 2740;
    pub const player_meleeWidth: usize = 2744;
    pub const playerMeleeRangeStanding: usize = 2748;
    pub const playerMeleeRangeCrouched: usize = 2752;
    pub const playerMeleeRangeProne: usize = 2756;
    pub const playerMeleeRangeChargeStanding: usize = 2760;
    pub const playerMeleeRangeChargeCrouched: usize = 2764;
    pub const playerMeleeChargeHeightTolerance: usize = 2768;
    pub const shieldAllowFiring: usize = 2772;
    pub const shieldUnkBool: usize = 2773;
    pub const pad13: usize = 2774;
    pub const shieldMaxSpeed: usize = 2776;
    pub const shieldAlwaysDisplay: usize = 2780;
    pub const pad14: usize = 2781;
    pub const shieldDeployGesture: usize = 2784;
    pub const shieldFireWeapGesture: usize = 2792;
    pub const shieldDeployWhileFiring: usize = 2800;
    pub const shieldRetractWhileFiring: usize = 2808;
    pub const shieldBashGesture: usize = 2816;
    pub const shieldMeleeFx: usize = 2824;
    pub const shieldMeleeFx__u: usize = 2824;
    pub const shieldMeleeFx__u__fx: usize = 2824;
    pub const shieldMeleeFx__u__vfx: usize = 2824;
    pub const shieldMeleeFx__u__data: usize = 2824;
    pub const shieldMeleeFx__type: usize = 2832;
    pub const shieldMeleeFxDelay: usize = 2840;
    pub const HitEarthquakeScale: usize = 2844;
    pub const HitEarthquakeDuration: usize = 2848;
    pub const HitEarthquakeRadius: usize = 2852;
    pub const shieldHitRumble: usize = 2856;
    pub const MissEarthquakeScale: usize = 2864;
    pub const MissEarthquakeDuration: usize = 2868;
    pub const MissEarthquakeRadius: usize = 2872;
    pub const shieldMissRumble: usize = 2880;
    pub const shieldDeployButton: usize = 2888;
    pub const shieldUsesEnergy: usize = 2892;
    pub const shieldMaxEnergy: usize = 2896;
    pub const shieldConsumptionRate: usize = 2900;
    pub const shieldMeleeEnergyCost: usize = 2904;
    pub const shieldMeleeHitEnergyCost: usize = 2908;
    pub const reactiveMotionRadiusScale: usize = 2912;
    pub const reactiveMotionFrequencyScale: usize = 2916;
    pub const reactiveMotionAmplitudeScale: usize = 2920;
    pub const reactiveMotionFalloff: usize = 2924;
    pub const reactiveMotionLifetime: usize = 2928;
    pub const hasAnyTransientModels: usize = 2932;
    pub const sharedAmmo: usize = 2933;
    pub const lockonSupported: usize = 2934;
    pub const requireLockonToFire: usize = 2935;
    pub const isAirburstWeapon: usize = 2936;
    pub const bigExplosion: usize = 2937;
    pub const noAdsWhenMagEmpty: usize = 2938;
    pub const avoidDropCleanup: usize = 2939;
    pub const allowGrenadeSwitching: usize = 2940;
    pub const inheritsPerks: usize = 2941;
    pub const crosshairColorChange: usize = 2942;
    pub const bRifleBullet: usize = 2943;
    pub const bEnergyBullet: usize = 2944;
    pub const armorPiercing: usize = 2945;
    pub const impaling: usize = 2946;
    pub const bBoltAction: usize = 2947;
    pub const aimDownSight: usize = 2948;
    pub const canHoldBreath: usize = 2949;
    pub const meleeOnly: usize = 2950;
    pub const supportsAlternateMelee: usize = 2951;
    pub const canVariableZoom: usize = 2952;
    pub const bRechamberWhileAds: usize = 2953;
    pub const bBulletExplosiveDamage: usize = 2954;
    pub const bCookOffHold: usize = 2955;
    pub const reticleSpin45: usize = 2956;
    pub const bClipOnly: usize = 2957;
    pub const bDoesNotConsumeAmmo: usize = 2958;
    pub const bRemoveWeaponOnEmpty: usize = 2959;
    pub const noAmmoPickup: usize = 2960;
    pub const adsFireOnly: usize = 2961;
    pub const cancelAutoHolsterWhenEmpty: usize = 2962;
    pub const disableSwitchToWhenEmpty: usize = 2963;
    pub const suppressAmmoPrimaryDisplay: usize = 2964;
    pub const suppressAmmoReserveDisplay: usize = 2965;
    pub const laserSightDuringNightvision: usize = 2966;
    pub const markableViewmodel: usize = 2967;
    pub const dualWieldType: usize = 2968;
    pub const flipKillIcon: usize = 2972;
    pub const bNoPartialReload: usize = 2973;
    pub const reloadDisabled: usize = 2974;
    pub const blocksProne: usize = 2975;
    pub const silenced: usize = 2976;
    pub const isRollingGrenade: usize = 2977;
    pub const dropGrenadeHeldOnDeath: usize = 2978;
    pub const projExplosionEffectForceNormalUp: usize = 2979;
    pub const projExplosionEffectInheritParentDirection: usize = 2980;
    pub const bProjImpactExplode: usize = 2981;
    pub const disableProjectileCrumpleCheck: usize = 2982;
    pub const bProjTrajectoryEvents: usize = 2983;
    pub const bProjWhizByEnabled: usize = 2984;
    pub const stickToPlayers: usize = 2985;
    pub const stickToVehicles: usize = 2986;
    pub const stickToTurrets: usize = 2987;
    pub const stickToNonStick: usize = 2988;
    pub const projEnableMissileStickiness: usize = 2989;
    pub const thrownSideways: usize = 2990;
    pub const hasDetonator: usize = 2991;
    pub const disableFiring: usize = 2992;
    pub const firesOnWeaponSwitch: usize = 2993;
    pub const disableHolding: usize = 2994;
    pub const timedDetonation: usize = 2995;
    pub const explosiveDamageDelay: usize = 2996;
    pub const fuseTime: usize = 3000;
    pub const aiFuseTime: usize = 3004;
    pub const maxHoldTime: usize = 3008;
    pub const rotationParams: usize = 3012;
    pub const rotationParams__initialPitch: usize = 3012;
    pub const rotationParams__initialYaw: usize = 3016;
    pub const rotationParams__initialRoll: usize = 3020;
    pub const rotationParams__rotationPitchDir: usize = 3024;
    pub const rotationParams__rotationPitchMin: usize = 3028;
    pub const rotationParams__rotationPitchMax: usize = 3032;
    pub const rotationParams__rotationYawDir: usize = 3036;
    pub const rotationParams__rotationYawMin: usize = 3040;
    pub const rotationParams__rotationYawMax: usize = 3044;
    pub const rotationParams__rotationRollDir: usize = 3048;
    pub const rotationParams__rotationRollMin: usize = 3052;
    pub const rotationParams__rotationRollMax: usize = 3056;
    pub const rotationParams__rotate: usize = 3060;
    pub const holdButtonToThrow: usize = 3064;
    pub const infiniteHold: usize = 3065;
    pub const freezeMovementWhenFiring: usize = 3066;
    pub const offhandAllowsSprint: usize = 3067;
    pub const thermalScope: usize = 3068;
    pub const thermalToggle: usize = 3069;
    pub const outlineEnemies: usize = 3070;
    pub const outlineDepthTest: usize = 3071;
    pub const outlineFill: usize = 3072;
    pub const pad10: usize = 3073;
    pub const enemyOutlineR: usize = 3076;
    pub const enemyOutlineG: usize = 3080;
    pub const enemyOutlineB: usize = 3084;
    pub const enemyOutlineA: usize = 3088;
    pub const allyOutlineR: usize = 3092;
    pub const allyOutlineG: usize = 3096;
    pub const allyOutlineB: usize = 3100;
    pub const allyOutlineA: usize = 3104;
    pub const depthScan: usize = 3108;
    pub const pad11: usize = 3109;
    pub const depthScanThickness: usize = 3112;
    pub const depthScanR: usize = 3116;
    pub const depthScanG: usize = 3120;
    pub const depthScanB: usize = 3124;
    pub const depthScanA: usize = 3128;
    pub const depthScanOutlineThickness: usize = 3132;
    pub const depthScanOutlineR: usize = 3136;
    pub const depthScanOutlineG: usize = 3140;
    pub const depthScanOutlineB: usize = 3144;
    pub const depthScanOutlineA: usize = 3148;
    pub const depthScanOverlay: usize = 3152;
    pub const pad12: usize = 3153;
    pub const depthScanOverlayStrength: usize = 3156;
    pub const depthScanOverlayXTiles: usize = 3160;
    pub const depthScanOverlayYTiles: usize = 3164;
    pub const depthScanOverlayXScroll: usize = 3168;
    pub const depthScanOverlayYScroll: usize = 3172;
    pub const blurSceneAdsInFraction: usize = 3176;
    pub const blurSceneAdsOutFraction: usize = 3180;
    pub const altModeSameWeapon: usize = 3184;
    pub const turretBarrelSpinEnabled: usize = 3185;
    pub const missileConeSoundEnabled: usize = 3186;
    pub const missileConeSoundPitchshiftEnabled: usize = 3187;
    pub const missileConeSoundCrossfadeEnabled: usize = 3188;
    pub const offhandHoldIsCancelable: usize = 3189;
    pub const doNotAllowAttachmentsToOverrideSpread: usize = 3190;
    pub const useScopeDrift: usize = 3191;
    pub const alwaysShatterGlassOnImpact: usize = 3192;
    pub const oldWeapon: usize = 3193;
    pub const jumpSpread: usize = 3194;
    pub const noFullViewmodelAnimations: usize = 3195;
    pub const killcamOffset: usize = 3196;
    pub const useDualFOV: usize = 3200;
    pub const disableDrop: usize = 3201;
    pub const preferredDrop: usize = 3202;
    pub const pad21: usize = 3203;
    pub const gestureAnimation: usize = 3208;
    pub const gestureFireStateTime: usize = 3216;
    pub const gesturePullback: usize = 3224;
    pub const minPullbackTime: usize = 3232;
    pub const gestureThrow: usize = 3240;
    pub const gestureFireTime: usize = 3248;
    pub const gestureDetonate: usize = 3256;
    pub const gestureDetonationTime: usize = 3264;
    pub const unk_433: usize = 3268;
    pub const unknownString1: usize = 3272;
    pub const unk_434: usize = 3280;
    pub const pad22: usize = 3284;
    pub const unknownString2: usize = 3288;
    pub const gesturesDisablePrimary: usize = 3296;
    pub const pad15: usize = 3297;
    pub const cameraFireEffect: usize = 3304;
    pub const cameraFireEffect__u: usize = 3304;
    pub const cameraFireEffect__u__fx: usize = 3304;
    pub const cameraFireEffect__u__vfx: usize = 3304;
    pub const cameraFireEffect__u__data: usize = 3304;
    pub const cameraFireEffect__type: usize = 3312;
    pub const cameraFireEffectDurationSec: usize = 3320;
    pub const changedFireTime: usize = 3324;
    pub const changedFireTimeAkimbo: usize = 3328;
    pub const changedFireTimeNumBullets: usize = 3332;
    pub const chargeType: usize = 3336;
    pub const chargeGain: usize = 3340;
    pub const chargeCostPerShot: usize = 3344;
    pub const chargeLossWhenIdle: usize = 3348;
    pub const chargeEmptyCooldown: usize = 3352;
    pub const chargeFireAtMaxDamageMultiplier: usize = 3356;
    pub const chargeMeterEffectBoneCount: usize = 3360;
    pub const chargeMeterEffect: usize = 3368;
    pub const chargeMeterEffect__u: usize = 3368;
    pub const chargeMeterEffect__u__fx: usize = 3368;
    pub const chargeMeterEffect__u__vfx: usize = 3368;
    pub const chargeMeterEffect__u__data: usize = 3368;
    pub const chargeMeterEffect__type: usize = 3376;
    pub const chargeUpSound: usize = 3384;
    pub const chargeDownSound: usize = 3392;
    pub const chargeUpSoundPlayer: usize = 3400;
    pub const chargeDownSoundPlayer: usize = 3408;
    pub const chargeDownToUpSound: usize = 3416;
    pub const chargeDownToUpSoundPlayer: usize = 3424;
    pub const chargeUpToDownSound: usize = 3432;
    pub const chargeUpToDownSoundPlayer: usize = 3440;
    pub const chargeMaxSound: usize = 3448;
    pub const chargeMaxSoundPlayer: usize = 3456;
    pub const chargeHudReveal: usize = 3464;
    pub const chargeRumble: usize = 3472;
    pub const stowTag: usize = 3480;
    pub const stowOffsetModel: usize = 3488;
    pub const slot: usize = 3496;
    pub const maxTargetAngle: usize = 3500;
    pub const spaceshipSecondaryWeapon: usize = 3504;
    pub const impulseFieldRadius: usize = 3508;
    pub const impulseFieldInitialSpeed: usize = 3512;
    pub const impulseFieldMaxSpeed: usize = 3516;
    pub const impulseFieldAcceleration: usize = 3520;
    pub const impulseFieldInAirImpulseMultiplier: usize = 3524;
    pub const impulseFieldInAirImpulseMultiplierInterpTime: usize = 3528;
    pub const impulseFieldSlideMultiplier: usize = 3532;
    pub const impulseFieldSlideMultiplierInterpTime: usize = 3536;
    pub const impulseFieldIsPush: usize = 3540;
    pub const impulseFieldAffectsFriendlies: usize = 3541;
    pub const impulseFieldAffectsSelf: usize = 3542;
    pub const impulseFieldAffectsProne: usize = 3543;
    pub const regenerationEnabled: usize = 3544;
    pub const regenerationTimeMs: usize = 3548;
    pub const regenerationAddTimeMs: usize = 3552;
    pub const regenerationAmount: usize = 3556;
    pub const regenerationConsumeStock: usize = 3560;
    pub const regenerationDisableWhileFiring: usize = 3561;
    pub const deployRequireOnWalkableSurface: usize = 3562;
    pub const deployRequireOnNavmesh: usize = 3563;
    pub const deployRequireSkyAbove: usize = 3564;
    pub const deployRequireNoOverhang: usize = 3565;
    pub const deployAlwaysUpright: usize = 3566;
    pub const deployEdgeSnap: usize = 3567;
    pub const deployCylinderRadius: usize = 3568;
    pub const deployCylinderHeight: usize = 3572;
    pub const deployMaxDistance: usize = 3576;
    pub const pad16: usize = 3580;
    pub const deployEffect: usize = 3584;
    pub const deployEffect__u: usize = 3584;
    pub const deployEffect__u__fx: usize = 3584;
    pub const deployEffect__u__vfx: usize = 3584;
    pub const deployEffect__u__data: usize = 3584;
    pub const deployEffect__type: usize = 3592;
    pub const deployValidGroupId: usize = 3600;
    pub const deployIndoorGroupId: usize = 3604;
    pub const deployInvalidGroupId: usize = 3608;
    pub const deployOutOfRangeGroupId: usize = 3612;
    pub const animOverrides: usize = 3616;
    pub const sfxOverrides: usize = 4120;
    pub const vfxOverrides: usize = 4624;
    pub const reactiveEmitterDelay: usize = 5128;
    pub const grenadeDangerIconDistance: usize = 5132;
    pub const unk_466_0: usize = 5136;
    pub const pad29: usize = 5137;
}
pub mod WeaponSFXPackage {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const sounds: usize = 8;
    pub const rattleSoundType: usize = 16;
    pub const szAdsrBaseSetting: usize = 24;
    pub const szWeapSndReflectionClass: usize = 32;
    pub const weapSndFireVolumeShot1: usize = 40;
    pub const weapSndFireVolumeShot2: usize = 44;
    pub const weapSndFireVolumeShot3: usize = 48;
    pub const weapSndProneFireLFEVolume: usize = 52;
}
pub mod WeaponSFXPackageSounds {
    pub const SIZE: usize = 496;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const pickupSound: usize = 8;
    pub const pickupSoundPlayer: usize = 16;
    pub const ammoPickupSound: usize = 24;
    pub const ammoPickupSoundPlayer: usize = 32;
    pub const projectileSound: usize = 40;
    pub const pullbackSound: usize = 48;
    pub const pullbackSoundPlayer: usize = 56;
    pub const pullbackSoundQuick: usize = 64;
    pub const pullbackSoundQuickPlayer: usize = 72;
    pub const fireSound: usize = 80;
    pub const fireSoundPlayer: usize = 88;
    pub const fireSoundPlayerAkimbo: usize = 96;
    pub const fireSoundPlayerAkimboRight: usize = 104;
    pub const fireStartSound: usize = 112;
    pub const fireStartSoundPlayer: usize = 120;
    pub const fireLoopSound: usize = 128;
    pub const fireLoopSoundPlayer: usize = 136;
    pub const fireStopSound: usize = 144;
    pub const fireStopSoundPlayer: usize = 152;
    pub const fireLastSound: usize = 160;
    pub const fireLastSoundPlayer: usize = 168;
    pub const fireLastSoundPlayerAkimbo: usize = 176;
    pub const fireLastSoundPlayerAkimboRight: usize = 184;
    pub const regenerateSound: usize = 192;
    pub const regenerateSoundPlayer: usize = 200;
    pub const emptyFireSound: usize = 208;
    pub const emptyFireSoundPlayer: usize = 216;
    pub const emptyRegenFireSound: usize = 224;
    pub const emptyRegenFireSoundPlayer: usize = 232;
    pub const rechamberSound: usize = 240;
    pub const rechamberSoundPlayer: usize = 248;
    pub const reloadSound: usize = 256;
    pub const reloadSoundPlayer: usize = 264;
    pub const reloadEmptySound: usize = 272;
    pub const reloadEmptySoundPlayer: usize = 280;
    pub const reloadStartSound: usize = 288;
    pub const reloadStartSoundPlayer: usize = 296;
    pub const reloadEndSound: usize = 304;
    pub const reloadEndSoundPlayer: usize = 312;
    pub const detonateSound: usize = 320;
    pub const detonateSoundPlayer: usize = 328;
    pub const nightVisionWearSound: usize = 336;
    pub const nightVisionWearSoundPlayer: usize = 344;
    pub const nightVisionRemoveSound: usize = 352;
    pub const nightVisionRemoveSoundPlayer: usize = 360;
    pub const altSwitchSound: usize = 368;
    pub const altSwitchSoundPlayer: usize = 376;
    pub const raiseSound: usize = 384;
    pub const raiseSoundPlayer: usize = 392;
    pub const firstRaiseSound: usize = 400;
    pub const firstRaiseSoundPlayer: usize = 408;
    pub const putawaySound: usize = 416;
    pub const putawaySoundPlayer: usize = 424;
    pub const enterAdsSoundPlayer: usize = 432;
    pub const leaveAdsSoundPlayer: usize = 440;
    pub const bulletRicochetImpact: usize = 448;
    pub const projExplosionSound: usize = 456;
    pub const projDudSound: usize = 464;
    pub const projIgnitionSound: usize = 472;
    pub const bounceSound: usize = 480;
    pub const rollingSound: usize = 488;
}
pub mod WeaponVFXPackage {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const vfx: usize = 8;
}
pub mod WeaponVFXPackageDefs {
    pub const SIZE: usize = 176;
    pub const ALIGN: usize = 8;
    pub const viewFlashEffect: usize = 0;
    pub const viewFlashEffect__u: usize = 0;
    pub const viewFlashEffect__u__fx: usize = 0;
    pub const viewFlashEffect__u__vfx: usize = 0;
    pub const viewFlashEffect__u__data: usize = 0;
    pub const viewFlashEffect__type: usize = 8;
    pub const worldFlashEffect: usize = 16;
    pub const worldFlashEffect__u: usize = 16;
    pub const worldFlashEffect__u__fx: usize = 16;
    pub const worldFlashEffect__u__vfx: usize = 16;
    pub const worldFlashEffect__u__data: usize = 16;
    pub const worldFlashEffect__type: usize = 24;
    pub const viewFlashADSEffect: usize = 32;
    pub const viewFlashADSEffect__u: usize = 32;
    pub const viewFlashADSEffect__u__fx: usize = 32;
    pub const viewFlashADSEffect__u__vfx: usize = 32;
    pub const viewFlashADSEffect__u__data: usize = 32;
    pub const viewFlashADSEffect__type: usize = 40;
    pub const viewFlashOverchargeEffect: usize = 48;
    pub const viewFlashOverchargeEffect__u: usize = 48;
    pub const viewFlashOverchargeEffect__u__fx: usize = 48;
    pub const viewFlashOverchargeEffect__u__vfx: usize = 48;
    pub const viewFlashOverchargeEffect__u__data: usize = 48;
    pub const viewFlashOverchargeEffect__type: usize = 56;
    pub const viewFlashOverchargeADSEffect: usize = 64;
    pub const viewFlashOverchargeADSEffect__u: usize = 64;
    pub const viewFlashOverchargeADSEffect__u__fx: usize = 64;
    pub const viewFlashOverchargeADSEffect__u__vfx: usize = 64;
    pub const viewFlashOverchargeADSEffect__u__data: usize = 64;
    pub const viewFlashOverchargeADSEffect__type: usize = 72;
    pub const worldFlashOverchargeEffect: usize = 80;
    pub const worldFlashOverchargeEffect__u: usize = 80;
    pub const worldFlashOverchargeEffect__u__fx: usize = 80;
    pub const worldFlashOverchargeEffect__u__vfx: usize = 80;
    pub const worldFlashOverchargeEffect__u__data: usize = 80;
    pub const worldFlashOverchargeEffect__type: usize = 88;
    pub const viewShellEjectEffect: usize = 96;
    pub const viewShellEjectEffect__u: usize = 96;
    pub const viewShellEjectEffect__u__fx: usize = 96;
    pub const viewShellEjectEffect__u__vfx: usize = 96;
    pub const viewShellEjectEffect__u__data: usize = 96;
    pub const viewShellEjectEffect__type: usize = 104;
    pub const worldShellEjectEffect: usize = 112;
    pub const worldShellEjectEffect__u: usize = 112;
    pub const worldShellEjectEffect__u__fx: usize = 112;
    pub const worldShellEjectEffect__u__vfx: usize = 112;
    pub const worldShellEjectEffect__u__data: usize = 112;
    pub const worldShellEjectEffect__type: usize = 120;
    pub const viewLastShotEjectEffect: usize = 128;
    pub const viewLastShotEjectEffect__u: usize = 128;
    pub const viewLastShotEjectEffect__u__fx: usize = 128;
    pub const viewLastShotEjectEffect__u__vfx: usize = 128;
    pub const viewLastShotEjectEffect__u__data: usize = 128;
    pub const viewLastShotEjectEffect__type: usize = 136;
    pub const worldLastShotEjectEffect: usize = 144;
    pub const worldLastShotEjectEffect__u: usize = 144;
    pub const worldLastShotEjectEffect__u__fx: usize = 144;
    pub const worldLastShotEjectEffect__u__vfx: usize = 144;
    pub const worldLastShotEjectEffect__u__data: usize = 144;
    pub const worldLastShotEjectEffect__type: usize = 152;
    pub const viewMagEjectEffect: usize = 160;
    pub const viewMagEjectEffect__u: usize = 160;
    pub const viewMagEjectEffect__u__fx: usize = 160;
    pub const viewMagEjectEffect__u__vfx: usize = 160;
    pub const viewMagEjectEffect__u__data: usize = 160;
    pub const viewMagEjectEffect__type: usize = 168;
}
pub mod XAnimAimConstraint {
    pub const SIZE: usize = 64;
    pub const ALIGN: usize = 4;
    pub const aimTargetLocalOffset: usize = 0;
    pub const localAimVector: usize = 12;
    pub const localUpVector: usize = 24;
    pub const axisMask: usize = 36;
    pub const upVector: usize = 48;
    pub const upVectorType: usize = 60;
}
pub mod XAnimConstraint {
    pub const SIZE: usize = 88;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const localTargetBoneIndex: usize = 1;
    pub const sourceBoneNames: usize = 4;
    pub const u: usize = 12;
    pub const u__positionConstraint: usize = 12;
    pub const u__positionConstraint__blendFraction: usize = 12;
    pub const u__positionConstraint__minLocalTranslationLimit: usize = 16;
    pub const u__positionConstraint__maxLocalTranslationLimit: usize = 28;
    pub const u__positionConstraint__offset: usize = 40;
    pub const u__positionConstraint__sourceBoneOffsets: usize = 52;
    pub const u__positionConstraint__axisMask: usize = 76;
    pub const u__rotationConstraint: usize = 12;
    pub const u__rotationConstraint__rotationOffsetQuat: usize = 12;
    pub const u__rotationConstraint__axisMask: usize = 28;
    pub const u__rotationConstraint__blendFraction: usize = 40;
    pub const u__aimConstraint: usize = 12;
    pub const u__aimConstraint__aimTargetLocalOffset: usize = 12;
    pub const u__aimConstraint__localAimVector: usize = 24;
    pub const u__aimConstraint__localUpVector: usize = 36;
    pub const u__aimConstraint__axisMask: usize = 48;
    pub const u__aimConstraint__upVector: usize = 60;
    pub const u__aimConstraint__upVectorType: usize = 72;
    pub const u__distanceDrivenOffsetConstraint: usize = 12;
    pub const u__distanceDrivenOffsetConstraint__minLocalOffset: usize = 12;
    pub const u__distanceDrivenOffsetConstraint__maxLocalOffset: usize = 28;
    pub const u__distanceDrivenOffsetConstraint__sourceBoneOffsets: usize = 44;
    pub const u__distanceDrivenOffsetConstraint__distanceScale: usize = 68;
    pub const u__distanceDrivenOffsetConstraint__distanceBias: usize = 72;
    pub const u__distanceDrivenOffsetConstraint__curveType: usize = 76;
}
pub mod XAnimConstraintUnion {
    pub const SIZE: usize = 76;
    pub const ALIGN: usize = 4;
    pub const positionConstraint: usize = 0;
    pub const positionConstraint__blendFraction: usize = 0;
    pub const positionConstraint__minLocalTranslationLimit: usize = 4;
    pub const positionConstraint__maxLocalTranslationLimit: usize = 16;
    pub const positionConstraint__offset: usize = 28;
    pub const positionConstraint__sourceBoneOffsets: usize = 40;
    pub const positionConstraint__axisMask: usize = 64;
    pub const rotationConstraint: usize = 0;
    pub const rotationConstraint__rotationOffsetQuat: usize = 0;
    pub const rotationConstraint__axisMask: usize = 16;
    pub const rotationConstraint__blendFraction: usize = 28;
    pub const aimConstraint: usize = 0;
    pub const aimConstraint__aimTargetLocalOffset: usize = 0;
    pub const aimConstraint__localAimVector: usize = 12;
    pub const aimConstraint__localUpVector: usize = 24;
    pub const aimConstraint__axisMask: usize = 36;
    pub const aimConstraint__upVector: usize = 48;
    pub const aimConstraint__upVectorType: usize = 60;
    pub const distanceDrivenOffsetConstraint: usize = 0;
    pub const distanceDrivenOffsetConstraint__minLocalOffset: usize = 0;
    pub const distanceDrivenOffsetConstraint__maxLocalOffset: usize = 16;
    pub const distanceDrivenOffsetConstraint__sourceBoneOffsets: usize = 32;
    pub const distanceDrivenOffsetConstraint__distanceScale: usize = 56;
    pub const distanceDrivenOffsetConstraint__distanceBias: usize = 60;
    pub const distanceDrivenOffsetConstraint__curveType: usize = 64;
}
pub mod XAnimDeltaPart {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const trans: usize = 0;
    pub const quat2: usize = 8;
    pub const quat: usize = 16;
}
pub mod XAnimDeltaPartQuat {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const size: usize = 0;
    pub const u: usize = 8;
    pub const u__frames: usize = 8;
    pub const u__frames__frames: usize = 8;
    pub const u__frames__indices: usize = 16;
    pub const u__frames__indices___1: usize = 16;
    pub const u__frames__indices___2: usize = 16;
    pub const u__frame0: usize = 8;
}
pub mod XAnimDeltaPartQuat2 {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const size: usize = 0;
    pub const u: usize = 8;
    pub const u__frames: usize = 8;
    pub const u__frames__frames: usize = 8;
    pub const u__frames__indices: usize = 16;
    pub const u__frames__indices___1: usize = 16;
    pub const u__frames__indices___2: usize = 16;
    pub const u__frame0: usize = 8;
}
pub mod XAnimDeltaPartQuatData {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const frames: usize = 0;
    pub const frames__frames: usize = 0;
    pub const frames__indices: usize = 8;
    pub const frames__indices___1: usize = 8;
    pub const frames__indices___2: usize = 8;
    pub const frame0: usize = 0;
}
pub mod XAnimDeltaPartQuatData2 {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const frames: usize = 0;
    pub const frames__frames: usize = 0;
    pub const frames__indices: usize = 8;
    pub const frames__indices___1: usize = 8;
    pub const frames__indices___2: usize = 8;
    pub const frame0: usize = 0;
}
pub mod XAnimDeltaPartQuatDataFrames {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const frames: usize = 0;
    pub const indices: usize = 8;
    pub const indices___1: usize = 8;
    pub const indices___2: usize = 8;
}
pub mod XAnimDeltaPartQuatDataFrames2 {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const frames: usize = 0;
    pub const indices: usize = 8;
    pub const indices___1: usize = 8;
    pub const indices___2: usize = 8;
}
pub mod XAnimDistanceDrivenOffsetConstraint {
    pub const SIZE: usize = 68;
    pub const ALIGN: usize = 4;
    pub const minLocalOffset: usize = 0;
    pub const maxLocalOffset: usize = 16;
    pub const sourceBoneOffsets: usize = 32;
    pub const distanceScale: usize = 56;
    pub const distanceBias: usize = 60;
    pub const curveType: usize = 64;
}
pub mod XAnimDynamicFrames {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const _1: usize = 0;
    pub const _2: usize = 0;
}
pub mod XAnimDynamicIndices {
    pub const SIZE: usize = 2;
    pub const ALIGN: usize = 2;
    pub const _1: usize = 0;
    pub const _2: usize = 0;
}
pub mod XAnimIndices {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const _1: usize = 0;
    pub const _2: usize = 0;
    pub const data: usize = 0;
}
pub mod XAnimNotifyInfo {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const name: usize = 0;
    pub const time: usize = 4;
}
pub mod XAnimPartTrans {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const size: usize = 0;
    pub const smallTrans: usize = 2;
    pub const u: usize = 8;
    pub const u__frames: usize = 8;
    pub const u__frames__mins: usize = 8;
    pub const u__frames__size: usize = 20;
    pub const u__frames__frames: usize = 32;
    pub const u__frames__frames___1: usize = 32;
    pub const u__frames__frames___2: usize = 32;
    pub const u__frames__indices: usize = 40;
    pub const u__frames__indices___1: usize = 40;
    pub const u__frames__indices___2: usize = 40;
    pub const u__frame0: usize = 8;
}
pub mod XAnimPartTransData {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const frames: usize = 0;
    pub const frames__mins: usize = 0;
    pub const frames__size: usize = 12;
    pub const frames__frames: usize = 24;
    pub const frames__frames___1: usize = 24;
    pub const frames__frames___2: usize = 24;
    pub const frames__indices: usize = 32;
    pub const frames__indices___1: usize = 32;
    pub const frames__indices___2: usize = 32;
    pub const frame0: usize = 0;
}
pub mod XAnimPartTransFrames {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const mins: usize = 0;
    pub const size: usize = 12;
    pub const frames: usize = 24;
    pub const frames___1: usize = 24;
    pub const frames___2: usize = 24;
    pub const indices: usize = 32;
    pub const indices___1: usize = 32;
    pub const indices___2: usize = 32;
}
pub mod XAnimParts {
    pub const SIZE: usize = 136;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const names: usize = 8;
    pub const dataByte: usize = 16;
    pub const dataShort: usize = 24;
    pub const dataInt: usize = 32;
    pub const randomDataShort: usize = 40;
    pub const randomDataByte: usize = 48;
    pub const randomDataInt: usize = 56;
    pub const indices: usize = 64;
    pub const indices___1: usize = 64;
    pub const indices___2: usize = 64;
    pub const indices__data: usize = 64;
    pub const notify: usize = 72;
    pub const deltaPart: usize = 80;
    pub const randomDataShortCount: usize = 88;
    pub const randomDataByteCount: usize = 92;
    pub const indexCount: usize = 96;
    pub const framerate: usize = 100;
    pub const frequency: usize = 104;
    pub const dataByteCount: usize = 108;
    pub const dataShortCount: usize = 110;
    pub const dataIntCount: usize = 112;
    pub const randomDataIntCount: usize = 114;
    pub const numframes: usize = 116;
    pub const flags: usize = 118;
    pub const boneCount: usize = 119;
    pub const notifyCount: usize = 129;
    pub const assetType: usize = 130;
    pub const unk1: usize = 131;
    pub const unk2: usize = 132;
}
pub mod XAnimPositionConstraint {
    pub const SIZE: usize = 76;
    pub const ALIGN: usize = 4;
    pub const blendFraction: usize = 0;
    pub const minLocalTranslationLimit: usize = 4;
    pub const maxLocalTranslationLimit: usize = 16;
    pub const offset: usize = 28;
    pub const sourceBoneOffsets: usize = 40;
    pub const axisMask: usize = 64;
}
pub mod XAnimProceduralBones {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const constraints: usize = 8;
    pub const numConstraints: usize = 16;
    pub const numTargetBones: usize = 20;
    pub const targetBoneNames: usize = 24;
    pub const unk01_count: usize = 32;
    pub const unk01: usize = 40;
    pub const unk02_count: usize = 48;
    pub const unk02: usize = 56;
    pub const unk_04: usize = 64;
    pub const unk03_count: usize = 68;
    pub const unk03: usize = 72;
}
pub mod XAnimRotationConstraint {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const rotationOffsetQuat: usize = 0;
    pub const axisMask: usize = 16;
    pub const blendFraction: usize = 28;
}
pub mod XAsset {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const header: usize = 8;
    pub const header__data: usize = 8;
    pub const header__physicsLibrary: usize = 8;
    pub const header__physicsSFXEventAsset: usize = 8;
    pub const header__physicsVFXEventAsset: usize = 8;
    pub const header__physicsAsset: usize = 8;
    pub const header__physicsFXPipeline: usize = 8;
    pub const header__physicsFXShape: usize = 8;
    pub const header__parts: usize = 8;
    pub const header__modelSurfs: usize = 8;
    pub const header__model: usize = 8;
    pub const header__mayhem: usize = 8;
    pub const header__material: usize = 8;
    pub const header__computeShader: usize = 8;
    pub const header__vertexShader: usize = 8;
    pub const header__hullShader: usize = 8;
    pub const header__domainShader: usize = 8;
    pub const header__pixelShader: usize = 8;
    pub const header__vertexDecl: usize = 8;
    pub const header__techniqueSet: usize = 8;
    pub const header__image: usize = 8;
    pub const header__soundGlobals: usize = 8;
    pub const header__soundBank: usize = 8;
    pub const header__soundBankTransient: usize = 8;
    pub const header__clipMap: usize = 8;
    pub const header__comWorld: usize = 8;
    pub const header__glassWorld: usize = 8;
    pub const header__pathData: usize = 8;
    pub const header__navMeshData: usize = 8;
    pub const header__mapEnts: usize = 8;
    pub const header__fxWorld: usize = 8;
    pub const header__gfxWorld: usize = 8;
    pub const header__gfxWorldTransientZone: usize = 8;
    pub const header__lightDef: usize = 8;
    pub const header__animClass: usize = 8;
    pub const header__playerAnim: usize = 8;
    pub const header__gesture: usize = 8;
    pub const header__localize: usize = 8;
    pub const header__attachment: usize = 8;
    pub const header__weapon: usize = 8;
    pub const header__vfx: usize = 8;
    pub const header__fx: usize = 8;
    pub const header__impactFx: usize = 8;
    pub const header__surfaceFx: usize = 8;
    pub const header__rawfile: usize = 8;
    pub const header__scriptfile: usize = 8;
    pub const header__stringTable: usize = 8;
    pub const header__leaderboardDef: usize = 8;
    pub const header__virtualLeaderboardDef: usize = 8;
    pub const header__structuredDataDefSet: usize = 8;
    pub const header__ddlFile: usize = 8;
    pub const header__tracerDef: usize = 8;
    pub const header__vehDef: usize = 8;
    pub const header__addonMapEnts: usize = 8;
    pub const header__netConstStrings: usize = 8;
    pub const header__luaFile: usize = 8;
    pub const header__scriptable: usize = 8;
    pub const header__equipSndTable: usize = 8;
    pub const header__vectorField: usize = 8;
    pub const header__particleSimAnimation: usize = 8;
    pub const header__streamingInfo: usize = 8;
    pub const header__laserDef: usize = 8;
    pub const header__ttfDef: usize = 8;
    pub const header__suitDef: usize = 8;
    pub const header__suitAnimPackage: usize = 8;
    pub const header__spaceshipTargetDef: usize = 8;
    pub const header__rumble: usize = 8;
    pub const header__rumbleGraph: usize = 8;
    pub const header__weaponAnimPackage: usize = 8;
    pub const header__weaponSFXPackage: usize = 8;
    pub const header__weaponVFXPackage: usize = 8;
    pub const header__behaviorTree: usize = 8;
    pub const header__proceduralBones: usize = 8;
    pub const header__reticleDef: usize = 8;
    pub const header__lightMap: usize = 8;
}
pub mod XAssetEntry {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const header: usize = 0;
    pub const header__data: usize = 0;
    pub const header__physicsLibrary: usize = 0;
    pub const header__physicsSFXEventAsset: usize = 0;
    pub const header__physicsVFXEventAsset: usize = 0;
    pub const header__physicsAsset: usize = 0;
    pub const header__physicsFXPipeline: usize = 0;
    pub const header__physicsFXShape: usize = 0;
    pub const header__parts: usize = 0;
    pub const header__modelSurfs: usize = 0;
    pub const header__model: usize = 0;
    pub const header__mayhem: usize = 0;
    pub const header__material: usize = 0;
    pub const header__computeShader: usize = 0;
    pub const header__vertexShader: usize = 0;
    pub const header__hullShader: usize = 0;
    pub const header__domainShader: usize = 0;
    pub const header__pixelShader: usize = 0;
    pub const header__vertexDecl: usize = 0;
    pub const header__techniqueSet: usize = 0;
    pub const header__image: usize = 0;
    pub const header__soundGlobals: usize = 0;
    pub const header__soundBank: usize = 0;
    pub const header__soundBankTransient: usize = 0;
    pub const header__clipMap: usize = 0;
    pub const header__comWorld: usize = 0;
    pub const header__glassWorld: usize = 0;
    pub const header__pathData: usize = 0;
    pub const header__navMeshData: usize = 0;
    pub const header__mapEnts: usize = 0;
    pub const header__fxWorld: usize = 0;
    pub const header__gfxWorld: usize = 0;
    pub const header__gfxWorldTransientZone: usize = 0;
    pub const header__lightDef: usize = 0;
    pub const header__animClass: usize = 0;
    pub const header__playerAnim: usize = 0;
    pub const header__gesture: usize = 0;
    pub const header__localize: usize = 0;
    pub const header__attachment: usize = 0;
    pub const header__weapon: usize = 0;
    pub const header__vfx: usize = 0;
    pub const header__fx: usize = 0;
    pub const header__impactFx: usize = 0;
    pub const header__surfaceFx: usize = 0;
    pub const header__rawfile: usize = 0;
    pub const header__scriptfile: usize = 0;
    pub const header__stringTable: usize = 0;
    pub const header__leaderboardDef: usize = 0;
    pub const header__virtualLeaderboardDef: usize = 0;
    pub const header__structuredDataDefSet: usize = 0;
    pub const header__ddlFile: usize = 0;
    pub const header__tracerDef: usize = 0;
    pub const header__vehDef: usize = 0;
    pub const header__addonMapEnts: usize = 0;
    pub const header__netConstStrings: usize = 0;
    pub const header__luaFile: usize = 0;
    pub const header__scriptable: usize = 0;
    pub const header__equipSndTable: usize = 0;
    pub const header__vectorField: usize = 0;
    pub const header__particleSimAnimation: usize = 0;
    pub const header__streamingInfo: usize = 0;
    pub const header__laserDef: usize = 0;
    pub const header__ttfDef: usize = 0;
    pub const header__suitDef: usize = 0;
    pub const header__suitAnimPackage: usize = 0;
    pub const header__spaceshipTargetDef: usize = 0;
    pub const header__rumble: usize = 0;
    pub const header__rumbleGraph: usize = 0;
    pub const header__weaponAnimPackage: usize = 0;
    pub const header__weaponSFXPackage: usize = 0;
    pub const header__weaponVFXPackage: usize = 0;
    pub const header__behaviorTree: usize = 0;
    pub const header__proceduralBones: usize = 0;
    pub const header__reticleDef: usize = 0;
    pub const header__lightMap: usize = 0;
    pub const nextHash: usize = 8;
    pub const nextOverride: usize = 12;
    pub const inuseMask: usize = 16;
    pub const r#type: usize = 17;
    pub const zoneIndex: usize = 18;
}
pub mod XAssetHeader {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const data: usize = 0;
    pub const physicsLibrary: usize = 0;
    pub const physicsSFXEventAsset: usize = 0;
    pub const physicsVFXEventAsset: usize = 0;
    pub const physicsAsset: usize = 0;
    pub const physicsFXPipeline: usize = 0;
    pub const physicsFXShape: usize = 0;
    pub const parts: usize = 0;
    pub const modelSurfs: usize = 0;
    pub const model: usize = 0;
    pub const mayhem: usize = 0;
    pub const material: usize = 0;
    pub const computeShader: usize = 0;
    pub const vertexShader: usize = 0;
    pub const hullShader: usize = 0;
    pub const domainShader: usize = 0;
    pub const pixelShader: usize = 0;
    pub const vertexDecl: usize = 0;
    pub const techniqueSet: usize = 0;
    pub const image: usize = 0;
    pub const soundGlobals: usize = 0;
    pub const soundBank: usize = 0;
    pub const soundBankTransient: usize = 0;
    pub const clipMap: usize = 0;
    pub const comWorld: usize = 0;
    pub const glassWorld: usize = 0;
    pub const pathData: usize = 0;
    pub const navMeshData: usize = 0;
    pub const mapEnts: usize = 0;
    pub const fxWorld: usize = 0;
    pub const gfxWorld: usize = 0;
    pub const gfxWorldTransientZone: usize = 0;
    pub const lightDef: usize = 0;
    pub const animClass: usize = 0;
    pub const playerAnim: usize = 0;
    pub const gesture: usize = 0;
    pub const localize: usize = 0;
    pub const attachment: usize = 0;
    pub const weapon: usize = 0;
    pub const vfx: usize = 0;
    pub const fx: usize = 0;
    pub const impactFx: usize = 0;
    pub const surfaceFx: usize = 0;
    pub const rawfile: usize = 0;
    pub const scriptfile: usize = 0;
    pub const stringTable: usize = 0;
    pub const leaderboardDef: usize = 0;
    pub const virtualLeaderboardDef: usize = 0;
    pub const structuredDataDefSet: usize = 0;
    pub const ddlFile: usize = 0;
    pub const tracerDef: usize = 0;
    pub const vehDef: usize = 0;
    pub const addonMapEnts: usize = 0;
    pub const netConstStrings: usize = 0;
    pub const luaFile: usize = 0;
    pub const scriptable: usize = 0;
    pub const equipSndTable: usize = 0;
    pub const vectorField: usize = 0;
    pub const particleSimAnimation: usize = 0;
    pub const streamingInfo: usize = 0;
    pub const laserDef: usize = 0;
    pub const ttfDef: usize = 0;
    pub const suitDef: usize = 0;
    pub const suitAnimPackage: usize = 0;
    pub const spaceshipTargetDef: usize = 0;
    pub const rumble: usize = 0;
    pub const rumbleGraph: usize = 0;
    pub const weaponAnimPackage: usize = 0;
    pub const weaponSFXPackage: usize = 0;
    pub const weaponVFXPackage: usize = 0;
    pub const behaviorTree: usize = 0;
    pub const proceduralBones: usize = 0;
    pub const reticleDef: usize = 0;
    pub const lightMap: usize = 0;
}
pub mod XAssetList {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const stringList: usize = 0;
    pub const stringList__count: usize = 0;
    pub const stringList__strings: usize = 8;
    pub const assetCount: usize = 16;
    pub const assets: usize = 24;
    pub const globals: usize = 32;
}
pub mod XBlock {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const alloc: usize = 0;
    pub const size: usize = 8;
}
pub mod XBlockCompressionBlockHeader {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 1;
    pub const compressedSize: usize = 0;
    pub const uncompressedSize: usize = 4;
}
pub mod XBlockCompressionBlockSizeAndCompressionType {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 1;
    pub const blockSize: usize = 0;
    pub const compressionType: usize = 3;
}
pub mod XBlockCompressionDataHeader {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 1;
    pub const uncompressedSize: usize = 0;
    pub const blockSizeAndType: usize = 8;
    pub const blockSizeAndType__blockSize: usize = 8;
    pub const blockSizeAndType__compressionType: usize = 11;
}
pub mod XBoneInfo {
    pub const SIZE: usize = 28;
    pub const ALIGN: usize = 4;
    pub const bounds: usize = 0;
    pub const bounds__midPoint: usize = 0;
    pub const bounds__halfSize: usize = 12;
    pub const bounds__radiusSquared: usize = 24;
    pub const bounds__radiusSquaredAsInt: usize = 24;
}
pub mod XFileCompressorHeader {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 1;
    pub const compressor: usize = 0;
    pub const magic: usize = 1;
}
pub mod XFileHeader {
    pub const SIZE: usize = 232;
    pub const ALIGN: usize = 8;
    pub const magic: usize = 0;
    pub const version: usize = 8;
    pub const unused: usize = 12;
    pub const has_no_image_fastfile: usize = 13;
    pub const has_no_shared_fastfile: usize = 14;
    pub const unk1: usize = 15;
    pub const fileTimeHigh: usize = 16;
    pub const fileTimeLow: usize = 20;
    pub const stream_data: usize = 24;
    pub const stream_data__size: usize = 24;
    pub const stream_data__unk1: usize = 32;
    pub const stream_data__unk2: usize = 40;
    pub const stream_data__block_size: usize = 48;
    pub const stream_data__unk_arr: usize = 128;
    pub const shared_ff_hash: usize = 192;
    pub const shared_ff_count: usize = 196;
    pub const image_ff_hash: usize = 200;
    pub const image_ff_count: usize = 204;
    pub const fileLen: usize = 208;
    pub const fileLenUnk1: usize = 216;
    pub const fileLenUnk2: usize = 224;
}
pub mod XFileReadData {
    pub const SIZE: usize = 224;
    pub const ALIGN: usize = 8;
    pub const header_parsed: usize = 0;
    pub const has_no_image_fastfile: usize = 1;
    pub const has_no_shared_fastfile: usize = 2;
    pub const unk1: usize = 3;
    pub const __pad0: usize = 4;
    pub const stream_data: usize = 8;
    pub const stream_data__size: usize = 8;
    pub const stream_data__unk1: usize = 16;
    pub const stream_data__unk2: usize = 24;
    pub const stream_data__block_size: usize = 32;
    pub const stream_data__unk_arr: usize = 112;
    pub const version: usize = 176;
    pub const shared_ff_hash: usize = 180;
    pub const shared_ff_count: usize = 184;
    pub const fileLenUnk1: usize = 192;
    pub const image_ff_hash: usize = 200;
    pub const image_ff_count: usize = 204;
    pub const fileLen: usize = 208;
    pub const fileLenUnk2: usize = 216;
}
pub mod XFileStreamData {
    pub const SIZE: usize = 168;
    pub const ALIGN: usize = 8;
    pub const size: usize = 0;
    pub const unk1: usize = 8;
    pub const unk2: usize = 16;
    pub const block_size: usize = 24;
    pub const unk_arr: usize = 104;
}
pub mod XGfxGlobals {
    pub const SIZE: usize = 104;
    pub const ALIGN: usize = 8;
    pub const depthStencilStateCount: usize = 0;
    pub const blendStateCount: usize = 4;
    pub const depthStencilStateBits: usize = 8;
    pub const blendStateBits: usize = 16;
    pub const depthStencilStates: usize = 24;
    pub const blendStates: usize = 32;
    pub const perPrimConstantBufferCount: usize = 40;
    pub const perObjConstantBufferCount: usize = 44;
    pub const stableConstantBufferCount: usize = 48;
    pub const perPrimConstantBufferSizes: usize = 56;
    pub const perObjConstantBufferSizes: usize = 64;
    pub const stableConstantBufferSizes: usize = 72;
    pub const perPrimConstantBuffers: usize = 80;
    pub const perObjConstantBuffers: usize = 88;
    pub const stableConstantBuffers: usize = 96;
}
pub mod XGlobals {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const gfxGlobals: usize = 0;
}
pub mod XModel {
    pub const SIZE: usize = 736;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const unused01: usize = 8;
    pub const hasLods: usize = 9;
    pub const maxLoadedLod: usize = 10;
    pub const numLods: usize = 11;
    pub const collLod: usize = 12;
    pub const shadowCutoffLod: usize = 13;
    pub const characterCollBoundsType: usize = 14;
    pub const flags: usize = 16;
    pub const numBones: usize = 20;
    pub const numRootBones: usize = 21;
    pub const numReactiveMotionParts: usize = 22;
    pub const numClientBones: usize = 24;
    pub const numsurfs: usize = 26;
    pub const scale: usize = 28;
    pub const numCollSurfs: usize = 32;
    pub const contents: usize = 36;
    pub const radius: usize = 40;
    pub const bounds: usize = 44;
    pub const bounds__midPoint: usize = 44;
    pub const bounds__halfSize: usize = 56;
    pub const memUsage: usize = 68;
    pub const physicsLODDataSize: usize = 72;
    pub const physicsUsageCounter: usize = 76;
    pub const physicsUsageCounter__serverEnt: usize = 76;
    pub const physicsUsageCounter__clientEnt: usize = 80;
    pub const physicsUsageCounter__dynEnt: usize = 84;
    pub const scriptableMoverDef: usize = 88;
    pub const proceduralBones: usize = 96;
    pub const aimAssistBones: usize = 104;
    pub const numAimAssistBones: usize = 112;
    pub const edgeLength: usize = 116;
    pub const noScalePartBits: usize = 120;
    pub const boneNames: usize = 152;
    pub const parentList: usize = 160;
    pub const tagAngles: usize = 168;
    pub const tagPositions: usize = 176;
    pub const partClassification: usize = 184;
    pub const baseMat: usize = 192;
    pub const reactiveMotionParts: usize = 200;
    pub const reactiveMotionTweaks: usize = 208;
    pub const materialHandles: usize = 216;
    pub const lodInfo: usize = 224;
    pub const collSurfs: usize = 608;
    pub const boneInfo: usize = 616;
    pub const invHighMipRadius: usize = 624;
    pub const physicsAsset: usize = 632;
    pub const physicsFXShape: usize = 640;
    pub const physicsLODData: usize = 648;
    pub const physicsLODDataNameCount: usize = 656;
    pub const physicsLODDataNames: usize = 664;
    pub const impactType: usize = 672;
    pub const unk_03: usize = 673;
    pub const unknown03Count: usize = 688;
    pub const unknown03: usize = 696;
    pub const unknownIndex: usize = 704;
    pub const unknownVec3Count: usize = 705;
    pub const unk_04: usize = 706;
    pub const unknownVec3: usize = 712;
    pub const unknownIndex2: usize = 720;
    pub const unknown04Count: usize = 721;
    pub const unk_05: usize = 722;
    pub const unknown04: usize = 728;
}
pub mod XModelAngle {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 2;
    pub const x: usize = 0;
    pub const y: usize = 2;
    pub const z: usize = 4;
    pub const base: usize = 6;
}
pub mod XModelCollSurf_s {
    pub const SIZE: usize = 36;
    pub const ALIGN: usize = 4;
    pub const bounds: usize = 0;
    pub const bounds__midPoint: usize = 0;
    pub const bounds__halfSize: usize = 12;
    pub const boneIdx: usize = 24;
    pub const contents: usize = 28;
    pub const surfFlags: usize = 32;
}
pub mod XModelDrawInfo {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const hasGfxEntIndex: usize = 0;
    pub const lod: usize = 0;
    pub const materialLod: usize = 0;
    pub const surfBufSize: usize = 0;
    pub const surfId: usize = 4;
}
pub mod XModelLodInfo {
    pub const SIZE: usize = 64;
    pub const ALIGN: usize = 8;
    pub const dist: usize = 0;
    pub const numsurfs: usize = 4;
    pub const surfIndex: usize = 6;
    pub const modelSurfs: usize = 8;
    pub const partBits: usize = 16;
    pub const surfs: usize = 48;
    pub const subdivLodValidMask: usize = 56;
    pub const flags: usize = 60;
}
pub mod XModelPhysicsUsageCounter {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const serverEnt: usize = 0;
    pub const clientEnt: usize = 4;
    pub const dynEnt: usize = 8;
}
pub mod XModelSurfs {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const surfs: usize = 8;
    pub const numsurfs: usize = 16;
    pub const partBits: usize = 20;
}
pub mod XModelTagPos {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const x: usize = 0;
    pub const y: usize = 4;
    pub const z: usize = 8;
}
pub mod XPakHeader {
    pub const SIZE: usize = 336;
    pub const ALIGN: usize = 4;
    pub const magic: usize = 0;
    pub const version: usize = 8;
    pub const unknown: usize = 12;
    pub const iv: usize = 32;
    pub const hash: usize = 48;
    pub const hash__bytes: usize = 48;
    pub const signedhash: usize = 80;
    pub const signedhash__bytes: usize = 80;
}
pub mod XRigidVertList {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const boneOffsetIndex: usize = 0;
    pub const vertCount: usize = 2;
    pub const triOffset: usize = 4;
    pub const triCount: usize = 6;
    pub const collisionTree: usize = 8;
}
pub mod XStreamFile {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const offset: usize = 0;
    pub const offsetEnd: usize = 8;
    pub const fileIndex: usize = 16;
    pub const isLocalized: usize = 18;
}
pub mod XSubdivRigidVertList {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const firstFace: usize = 0;
    pub const faceCount: usize = 4;
    pub const firstRegularPatch: usize = 8;
    pub const regularPatchCount: usize = 12;
}
pub mod XSurface {
    pub const SIZE: usize = 256;
    pub const ALIGN: usize = 8;
    pub const flags: usize = 0;
    pub const vertCount: usize = 2;
    pub const triCount: usize = 4;
    pub const rigidVertListCount: usize = 6;
    pub const subdivLevelCount: usize = 7;
    pub const blendVertCounts: usize = 8;
    pub const blendVertSize: usize = 24;
    pub const __pad0: usize = 28;
    pub const verts0: usize = 32;
    pub const verts0__packedVerts0: usize = 32;
    pub const verts0__packedMotionVerts0: usize = 32;
    pub const verts0__verts0: usize = 32;
    pub const triIndices: usize = 40;
    pub const vb0: usize = 48;
    pub const vb0View: usize = 56;
    pub const indexBuffer: usize = 64;
    pub const rigidVertLists: usize = 72;
    pub const blendVerts: usize = 80;
    pub const blendVertsBuffer: usize = 88;
    pub const blendVertsView: usize = 96;
    pub const lmapUnwrap: usize = 104;
    pub const vblmapBuffer: usize = 112;
    pub const vblmapView: usize = 120;
    pub const subdiv: usize = 128;
    pub const tensionData: usize = 136;
    pub const tensionAccumTable: usize = 144;
    pub const tensionAccumTableBuffer: usize = 152;
    pub const tensionAccumTableView: usize = 160;
    pub const tensionDataBuffer: usize = 168;
    pub const tensionDataView: usize = 176;
    pub const indexBufferView: usize = 184;
    pub const unk: usize = 192;
    pub const pad: usize = 196;
    pub const shProbeSimplexVertData: usize = 200;
    pub const shProbeSimplexVertData__data: usize = 200;
    pub const shProbeSimplexVertData__data0: usize = 200;
    pub const shProbeSimplexVertData__data1: usize = 200;
    pub const shProbeSimplexVertData__data2: usize = 200;
    pub const shProbeSimplexVertBuffer: usize = 208;
    pub const shProbeSimplexVertBufferView: usize = 216;
    pub const partBits: usize = 224;
}
pub mod XSurfaceCollisionAabb {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 2;
    pub const mins: usize = 0;
    pub const maxs: usize = 6;
}
pub mod XSurfaceCollisionLeaf {
    pub const SIZE: usize = 2;
    pub const ALIGN: usize = 2;
    pub const triangleBeginIndex: usize = 0;
}
pub mod XSurfaceCollisionNode {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 2;
    pub const aabb: usize = 0;
    pub const aabb__mins: usize = 0;
    pub const aabb__maxs: usize = 6;
    pub const childBeginIndex: usize = 12;
    pub const childCount: usize = 14;
}
pub mod XSurfaceCollisionTree {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const trans: usize = 0;
    pub const scale: usize = 12;
    pub const nodeCount: usize = 24;
    pub const nodes: usize = 32;
    pub const leafCount: usize = 40;
    pub const leafs: usize = 48;
}
pub mod XSurfaceSubdivInfo {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const levels: usize = 0;
    pub const flags: usize = 8;
    pub const __pad0: usize = 12;
    pub const cache: usize = 32;
    pub const cache__size: usize = 32;
    pub const cache__subdivCacheBuffer: usize = 40;
    pub const cache__subdivCacheView: usize = 48;
}
pub mod XSurfaceSubdivLevel {
    pub const SIZE: usize = 232;
    pub const ALIGN: usize = 8;
    pub const rigidVertLists: usize = 0;
    pub const faceCount: usize = 8;
    pub const regularPatchCount: usize = 12;
    pub const regularPatchOffset: usize = 16;
    pub const facePointCount: usize = 20;
    pub const facePointValence4Count: usize = 24;
    pub const facePointBufferSize: usize = 28;
    pub const edgePointCount: usize = 32;
    pub const edgePointSmoothEnd: usize = 36;
    pub const edgePointUVBorderEnd: usize = 40;
    pub const vertexPointCount: usize = 44;
    pub const vertexPointValence4Count: usize = 48;
    pub const vertexPointBufferSize: usize = 52;
    pub const normalCount: usize = 56;
    pub const normalBufferSize: usize = 60;
    pub const transitionPointCount: usize = 64;
    pub const vertCount: usize = 68;
    pub const vertOffset: usize = 72;
    pub const faceIndices: usize = 80;
    pub const regularPatchIndices: usize = 88;
    pub const regularPatchFlags: usize = 96;
    pub const facePoints: usize = 104;
    pub const edgePoints: usize = 112;
    pub const vertexPoints: usize = 120;
    pub const normals: usize = 128;
    pub const transitionPoints: usize = 136;
    pub const regularPatchCones: usize = 144;
    pub const regularPatchIndexBuffer: usize = 152;
    pub const faceIndexBuffer: usize = 160;
    pub const regularPatchIndexBufferView: usize = 168;
    pub const regularPatchFlagsView: usize = 176;
    pub const facePointsView: usize = 184;
    pub const edgePointsView: usize = 192;
    pub const vertexPointsView: usize = 200;
    pub const normalsView: usize = 208;
    pub const transitionPointsView: usize = 216;
    pub const regularPatchConesView: usize = 224;
}
pub mod XZoneInfo {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const allocFlags: usize = 8;
    pub const freeFlags: usize = 12;
}
pub mod XZoneMemory {
    pub const SIZE: usize = 312;
    pub const ALIGN: usize = 8;
    pub const blocks: usize = 0;
    pub const callbackPos: usize = 160;
    pub const __pad0: usize = 168;
    pub const unk1: usize = 264;
    pub const shared_ff_data: usize = 272;
    pub const shared_ff_count: usize = 280;
    pub const shared_ff_count2: usize = 284;
    pub const unknown: usize = 288;
    pub const streamed_images: usize = 296;
    pub const streamed_image_count: usize = 304;
    pub const streamed_image_index: usize = 308;
}
pub mod ZoneDef {
    pub const SIZE: usize = 184;
    pub const ALIGN: usize = 8;
    pub const id: usize = 0;
    pub const stateId: usize = 4;
    pub const reverbIndex: usize = 8;
    pub const startOcclusionIndex: usize = 10;
    pub const numOcclusion: usize = 12;
    pub const startFilterIndex: usize = 14;
    pub const numFilter: usize = 16;
    pub const startMixIndex: usize = 18;
    pub const numMix: usize = 20;
    pub const ambientStream: usize = 24;
    pub const startAdsrZoneIndex: usize = 32;
    pub const numAdsrZone: usize = 34;
    pub const startNPCAdsrZoneIndex: usize = 36;
    pub const numNPCAdsrZone: usize = 38;
    pub const ambientDefIndex: usize = 40;
    pub const unkIndex: usize = 42;
    pub const startFullOccIndex: usize = 44;
    pub const numDisableFullOcc: usize = 46;
    pub const startPlayerBreathStateIndex: usize = 48;
    pub const numPlayerBreathStates: usize = 50;
    pub const weapReflId: usize = 52;
    pub const zoneName: usize = 56;
    pub const stateName: usize = 64;
    pub const ambientName: usize = 72;
    pub const mixName: usize = 80;
    pub const reverbName: usize = 88;
    pub const filterName: usize = 96;
    pub const occlusionName: usize = 104;
    pub const plrAdsrName: usize = 112;
    pub const npcAdsrName: usize = 120;
    pub const weapReflName: usize = 128;
    pub const fullOccName: usize = 136;
    pub const playerBreathStateName: usize = 144;
    pub const contextType1: usize = 152;
    pub const contextValue1: usize = 156;
    pub const contextType2: usize = 160;
    pub const contextValue2: usize = 164;
    pub const contextType3: usize = 168;
    pub const contextValue3: usize = 172;
    pub const contextType4: usize = 176;
    pub const contextValue4: usize = 180;
}
pub mod cStaticModel_s {
    pub const SIZE: usize = 64;
    pub const ALIGN: usize = 8;
    pub const xmodel: usize = 0;
    pub const origin: usize = 8;
    pub const invScaledAxis: usize = 20;
    pub const unk1: usize = 56;
    pub const unk2: usize = 57;
    pub const hasTransientModel: usize = 58;
    pub const hasTransientPhysicsAsset: usize = 59;
    pub const pad: usize = 60;
}
pub mod clipMap_t {
    pub const SIZE: usize = 384;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const isInUse: usize = 8;
    pub const info: usize = 16;
    pub const info__planeCount: usize = 16;
    pub const info__planes: usize = 24;
    pub const pInfo: usize = 32;
    pub const numStaticModels: usize = 40;
    pub const staticModelList: usize = 48;
    pub const staticModelCollisionModelList: usize = 56;
    pub const staticModelCollisionModelList__numModels: usize = 56;
    pub const staticModelCollisionModelList__staticModelIndex: usize = 64;
    pub const numStaticModelCollisionModelLists: usize = 72;
    pub const staticModelCollisionModelLists: usize = 80;
    pub const mapEnts: usize = 88;
    pub const stages: usize = 96;
    pub const stageCount: usize = 104;
    pub const stageTrigger: usize = 112;
    pub const stageTrigger__count: usize = 112;
    pub const stageTrigger__models: usize = 120;
    pub const stageTrigger__hullCount: usize = 128;
    pub const stageTrigger__hulls: usize = 136;
    pub const stageTrigger__slabCount: usize = 144;
    pub const stageTrigger__slabs: usize = 152;
    pub const stageTrigger__windingCount: usize = 160;
    pub const stageTrigger__windings: usize = 168;
    pub const stageTrigger__windingPointCount: usize = 176;
    pub const stageTrigger__windingPoints: usize = 184;
    pub const broadphaseMin: usize = 192;
    pub const broadphaseMax: usize = 204;
    pub const physicsCapacities: usize = 216;
    pub const physicsCapacities__maxNumRigidBodiesServer: usize = 216;
    pub const physicsCapacities__maxNumDetailRigidBodiesServer: usize = 220;
    pub const physicsCapacities__maxNumConstraintsServer: usize = 224;
    pub const physicsCapacities__maxNumMotionsServer: usize = 228;
    pub const physicsCapacities__maxNumRigidBodiesClient: usize = 232;
    pub const physicsCapacities__maxNumDetailRigidBodiesClient: usize = 236;
    pub const physicsCapacities__maxNumConstraintsClient: usize = 240;
    pub const physicsCapacities__maxNumMotionsClient: usize = 244;
    pub const havokWorldShapeDataSize: usize = 248;
    pub const havokWorldShapeData: usize = 256;
    pub const numCollisionHeatmapEntries: usize = 264;
    pub const collisionHeatmap: usize = 272;
    pub const topDownMapData: usize = 280;
    pub const checksum: usize = 288;
    pub const padding: usize = 292;
}
pub mod cmodel_t {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const bounds: usize = 0;
    pub const bounds__midPoint: usize = 0;
    pub const bounds__halfSize: usize = 12;
    pub const radius: usize = 24;
    pub const info: usize = 32;
    pub const physicsAsset: usize = 40;
    pub const physicsShapeOverrideIdx: usize = 48;
    pub const navObstacleIdx: usize = 50;
    pub const edgeFirstIndex: usize = 52;
}
pub mod cplane_s {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 4;
    pub const normal: usize = 0;
    pub const dist: usize = 12;
    pub const r#type: usize = 16;
    pub const pad: usize = 17;
}
pub mod db_internal_state {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 4;
    pub const dummy: usize = 0;
}
pub mod db_z_stream_s {
    pub const SIZE: usize = 88;
    pub const ALIGN: usize = 8;
    pub const next_in: usize = 0;
    pub const avail_in: usize = 8;
    pub const total_in: usize = 12;
    pub const next_out: usize = 16;
    pub const avail_out: usize = 24;
    pub const total_out: usize = 28;
    pub const msg: usize = 32;
    pub const state: usize = 40;
    pub const zalloc: usize = 48;
    pub const zfree: usize = 56;
    pub const opaque: usize = 64;
    pub const data_type: usize = 72;
    pub const adler: usize = 76;
    pub const reserved: usize = 80;
}
pub mod float4 {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const v: usize = 0;
}
pub mod nav_boundary_plane_s {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const m_Normal: usize = 0;
    pub const m_DistFromCenter: usize = 12;
}
pub mod nav_glass_bounds_s {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const m_Boundaries: usize = 0;
    pub const m_NumBoundaries: usize = 8;
    pub const m_Pos: usize = 12;
    pub const m_GlassPieceIndex: usize = 24;
}
pub mod nav_link_creation_data_s {
    pub const SIZE: usize = 72;
    pub const ALIGN: usize = 4;
    pub const m_Start: usize = 0;
    pub const m_Start__m_Pt1: usize = 0;
    pub const m_Start__m_Pt2: usize = 12;
    pub const m_End: usize = 24;
    pub const m_End__m_Pt1: usize = 24;
    pub const m_End__m_Pt2: usize = 36;
    pub const m_Animscript: usize = 48;
    pub const m_Target: usize = 52;
    pub const m_Parent: usize = 56;
    pub const m_UsageFlags: usize = 60;
    pub const m_PenaltyMult: usize = 64;
    pub const m_bBidirectional: usize = 68;
}
pub mod nav_link_point_s {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 4;
    pub const m_Pt1: usize = 0;
    pub const m_Pt2: usize = 12;
}
pub mod nav_modifier_s {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const m_Boundaries: usize = 0;
    pub const m_NumBoundaries: usize = 8;
    pub const m_Pos: usize = 12;
    pub const m_TargetName: usize = 24;
    pub const m_UserFlags: usize = 28;
    pub const m_LayerFlags: usize = 32;
    pub const m_Weight: usize = 36;
    pub const m_bActiveOnLoad: usize = 40;
}
pub mod nav_obstacle_bounds_s {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const m_Hulls: usize = 0;
    pub const m_NumHulls: usize = 8;
}
pub mod nav_obstacle_hull_s {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const m_Boundaries: usize = 0;
    pub const m_NumBoundaries: usize = 8;
    pub const m_BoundsX: usize = 12;
    pub const m_BoundsY: usize = 16;
    pub const m_BoundsZ: usize = 20;
    pub const m_Offset: usize = 24;
    pub const m_bUseBounds: usize = 36;
}
pub mod nav_raw_custom_volume_s {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const m_Tris: usize = 0;
    pub const m_NumTris: usize = 8;
}
pub mod nav_raw_custom_volume_tri_s {
    pub const SIZE: usize = 36;
    pub const ALIGN: usize = 4;
    pub const m_Verts: usize = 0;
}
pub mod nav_raw_volume_s {
    pub const SIZE: usize = 36;
    pub const ALIGN: usize = 4;
    pub const m_MidPoint: usize = 0;
    pub const m_HalfSize: usize = 12;
    pub const m_MinVoxelSize: usize = 24;
    pub const m_ClearanceXY: usize = 28;
    pub const m_ClearanceZ: usize = 32;
}
pub mod nav_resource_s {
    pub const SIZE: usize = 104;
    pub const ALIGN: usize = 8;
    pub const targetName: usize = 0;
    pub const modelIdx: usize = 4;
    pub const targetEntNum: usize = 8;
    pub const pSpace: usize = 16;
    pub const graphSize: usize = 24;
    pub const localOffsetPos: usize = 28;
    pub const localOffsetRot: usize = 40;
    pub const offsetLayer: usize = 56;
    pub const pPrev: usize = 64;
    pub const pNext: usize = 72;
    pub const layerFlags: usize = 80;
    pub const bDockable: usize = 84;
    pub const bIsVolume: usize = 85;
    pub const pGraphBuffer: usize = 88;
    pub const pWorkingGraph: usize = 96;
}
pub mod pathlink_s {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const fDist: usize = 0;
    pub const nodeNum: usize = 4;
    pub const disconnectCount: usize = 6;
    pub const negotiationLink: usize = 7;
    pub const flags: usize = 8;
    pub const ubBadPlaceCount: usize = 9;
}
pub mod pathnode_constant_t {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const spawnflags: usize = 4;
    pub const targetname: usize = 8;
    pub const script_linkName: usize = 12;
    pub const script_noteworthy: usize = 16;
    pub const target: usize = 20;
    pub const animscript: usize = 24;
    pub const animscriptfunc: usize = 28;
    pub const vLocalOrigin: usize = 32;
    pub const orientation: usize = 44;
    pub const orientation__yaw_orient: usize = 44;
    pub const orientation__yaw_orient__fLocalAngle: usize = 44;
    pub const orientation__yaw_orient__localForward: usize = 48;
    pub const orientation__angles: usize = 44;
    pub const parent: usize = 56;
    pub const parent__name: usize = 56;
    pub const parent__index: usize = 56;
    pub const ___u11: usize = 60;
    pub const ___u11__minUseDistSq: usize = 60;
    pub const ___u11__error: usize = 60;
    pub const wOverlapNode: usize = 64;
    pub const totalLinkCount: usize = 68;
    pub const Links: usize = 72;
}
pub mod pathnode_dynamic_t {
    pub const SIZE: usize = 44;
    pub const ALIGN: usize = 4;
    pub const pOwner: usize = 0;
    pub const pOwner__number: usize = 0;
    pub const pOwner__infoIndex: usize = 2;
    pub const iFreeTime: usize = 4;
    pub const iValidTime: usize = 8;
    pub const wLinkCount: usize = 20;
    pub const wOverlapCount: usize = 22;
    pub const turretEntNumber: usize = 24;
    pub const userCount: usize = 26;
    pub const hasBadPlaceLink: usize = 27;
    pub const spreadUsedTime: usize = 28;
    pub const flags: usize = 36;
    pub const dangerousCount: usize = 38;
    pub const recentUseProxTime: usize = 40;
}
pub mod pathnode_t {
    pub const SIZE: usize = 176;
    pub const ALIGN: usize = 8;
    pub const constant: usize = 0;
    pub const constant__type: usize = 0;
    pub const constant__spawnflags: usize = 4;
    pub const constant__targetname: usize = 8;
    pub const constant__script_linkName: usize = 12;
    pub const constant__script_noteworthy: usize = 16;
    pub const constant__target: usize = 20;
    pub const constant__animscript: usize = 24;
    pub const constant__animscriptfunc: usize = 28;
    pub const constant__vLocalOrigin: usize = 32;
    pub const constant__orientation: usize = 44;
    pub const constant__orientation__yaw_orient: usize = 44;
    pub const constant__orientation__yaw_orient__fLocalAngle: usize = 44;
    pub const constant__orientation__yaw_orient__localForward: usize = 48;
    pub const constant__orientation__angles: usize = 44;
    pub const constant__parent: usize = 56;
    pub const constant__parent__name: usize = 56;
    pub const constant__parent__index: usize = 56;
    pub const constant_____u11: usize = 60;
    pub const constant_____u11__minUseDistSq: usize = 60;
    pub const constant_____u11__error: usize = 60;
    pub const constant__wOverlapNode: usize = 64;
    pub const constant__totalLinkCount: usize = 68;
    pub const constant__Links: usize = 72;
    pub const dynamic: usize = 80;
    pub const dynamic__pOwner: usize = 80;
    pub const dynamic__pOwner__number: usize = 80;
    pub const dynamic__pOwner__infoIndex: usize = 82;
    pub const dynamic__iFreeTime: usize = 84;
    pub const dynamic__iValidTime: usize = 88;
    pub const dynamic__wLinkCount: usize = 100;
    pub const dynamic__wOverlapCount: usize = 102;
    pub const dynamic__turretEntNumber: usize = 104;
    pub const dynamic__userCount: usize = 106;
    pub const dynamic__hasBadPlaceLink: usize = 107;
    pub const dynamic__spreadUsedTime: usize = 108;
    pub const dynamic__flags: usize = 116;
    pub const dynamic__dangerousCount: usize = 118;
    pub const dynamic__recentUseProxTime: usize = 120;
    pub const transient: usize = 128;
    pub const transient__iSearchFrame: usize = 128;
    pub const transient__pNextOpen: usize = 136;
    pub const transient__pPrevOpen: usize = 144;
    pub const transient__pParent: usize = 152;
    pub const transient__fCost: usize = 160;
    pub const transient__fHeuristic: usize = 164;
    pub const transient_____u6: usize = 168;
    pub const transient_____u6__nodeCost: usize = 168;
    pub const transient_____u6__linkIndex: usize = 168;
}
pub mod pathnode_transient_t {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const iSearchFrame: usize = 0;
    pub const pNextOpen: usize = 8;
    pub const pPrevOpen: usize = 16;
    pub const pParent: usize = 24;
    pub const fCost: usize = 32;
    pub const fHeuristic: usize = 36;
    pub const ___u6: usize = 40;
    pub const ___u6__nodeCost: usize = 40;
    pub const ___u6__linkIndex: usize = 40;
}
pub mod pathnode_tree_info_t {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const child: usize = 0;
    pub const s: usize = 0;
    pub const s__nodeCount: usize = 0;
    pub const s__nodes: usize = 8;
}
pub mod pathnode_tree_nodes_t {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const nodeCount: usize = 0;
    pub const nodes: usize = 8;
}
pub mod pathnode_tree_t {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const axis: usize = 0;
    pub const dist: usize = 4;
    pub const u: usize = 8;
    pub const u__child: usize = 8;
    pub const u__s: usize = 8;
    pub const u__s__nodeCount: usize = 8;
    pub const u__s__nodes: usize = 16;
}
pub mod pathnode_yaworient_t {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const fLocalAngle: usize = 0;
    pub const localForward: usize = 4;
}
pub mod scr_anim_t {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const ___u0: usize = 0;
    pub const ___u0____s0: usize = 0;
    pub const ___u0____s0__index: usize = 0;
    pub const ___u0____s0__tree: usize = 2;
    pub const ___u0__linkPointer: usize = 0;
}
pub mod snd_entchannel_info_t {
    pub const SIZE: usize = 104;
    pub const ALIGN: usize = 4;
    pub const name: usize = 0;
    pub const id: usize = 64;
    pub const priority: usize = 68;
    pub const spatialType: usize = 72;
    pub const isRestricted: usize = 76;
    pub const isPausable: usize = 77;
    pub const maxVoices: usize = 80;
    pub const limitBehavior: usize = 84;
    pub const centerSpeakerPan: usize = 88;
    pub const mpFarReverbBehavior: usize = 92;
    pub const spFarReverbBehavior: usize = 96;
    pub const cpFarReverbBehavior: usize = 100;
}
pub mod snd_volmod_info_t {
    pub const SIZE: usize = 72;
    pub const ALIGN: usize = 4;
    pub const name: usize = 0;
    pub const id: usize = 64;
    pub const value: usize = 68;
}
pub mod srfTriangles_t {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 4;
    pub const vertexLayerData: usize = 0;
    pub const firstVertex: usize = 4;
    pub const maxEdgeLength: usize = 8;
    pub const vertexCount: usize = 12;
    pub const triCount: usize = 14;
    pub const baseIndex: usize = 16;
}
pub mod sunflare_t {
    pub const SIZE: usize = 112;
    pub const ALIGN: usize = 8;
    pub const hasValidData: usize = 0;
    pub const spriteMaterial: usize = 8;
    pub const flareMaterial: usize = 16;
    pub const spriteSize: usize = 24;
    pub const flareMinSize: usize = 28;
    pub const flareMinDot: usize = 32;
    pub const flareMaxSize: usize = 36;
    pub const flareMaxDot: usize = 40;
    pub const flareMaxAlpha: usize = 44;
    pub const flareFadeInTime: usize = 48;
    pub const flareFadeOutTime: usize = 52;
    pub const blindMinDot: usize = 56;
    pub const blindMaxDot: usize = 60;
    pub const blindMaxDarken: usize = 64;
    pub const blindFadeInTime: usize = 68;
    pub const blindFadeOutTime: usize = 72;
    pub const glareMinDot: usize = 76;
    pub const glareMaxDot: usize = 80;
    pub const glareMaxLighten: usize = 84;
    pub const glareFadeInTime: usize = 88;
    pub const glareFadeOutTime: usize = 92;
    pub const sunFxPosition: usize = 96;
}
pub mod unk_1453E1068 {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const numUnk: usize = 0;
    pub const unk: usize = 8;
}
pub mod unk_1453E1188 {
    pub const SIZE: usize = 68;
    pub const ALIGN: usize = 1;
    pub const __pad0: usize = 0;
}
pub mod unk_1453E14D8 {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 4;
    pub const bounds: usize = 0;
    pub const bounds__midPoint: usize = 0;
    pub const bounds__halfSize: usize = 12;
}
pub mod unk_1453E1B90 {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const flags: usize = 8;
    pub const __pad0: usize = 12;
}
pub mod unk_1453E1BC0 {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
}
pub mod unk_1453E1C00 {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
}
pub mod unk_1453E1C20 {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const __pad0: usize = 8;
}
pub mod unk_1453E1C70 {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const base: usize = 0;
    pub const count: usize = 8;
    pub const val: usize = 16;
}
pub mod unk_1453E1C80 {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const __0: usize = 0;
    pub const __0__base: usize = 0;
    pub const __1: usize = 0;
    pub const __1__base: usize = 0;
    pub const __2: usize = 0;
    pub const __2__base: usize = 0;
    pub const __2____pad0: usize = 8;
    pub const __3: usize = 0;
    pub const __3__base: usize = 0;
    pub const __3__count: usize = 8;
    pub const __3__val: usize = 16;
}
pub mod unk_1453E1E30 {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const unk01: usize = 0;
    pub const __pad0: usize = 4;
}
pub mod unk_1453E1E68 {
    pub const SIZE: usize = 128;
    pub const ALIGN: usize = 4;
    pub const unk01: usize = 0;
    pub const __pad0: usize = 4;
    pub const unk02: usize = 96;
    pub const __pad1: usize = 100;
    pub const unk03: usize = 104;
    pub const __pad2: usize = 108;
}
pub mod unk_1453E1EA8 {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 1;
    pub const __pad0: usize = 0;
}
pub mod unk_1453E2FD8 {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const id: usize = 0;
    pub const name: usize = 8;
    pub const unk3: usize = 16;
    pub const unk4: usize = 20;
    pub const unk5: usize = 24;
    pub const unk6: usize = 28;
    pub const unk7: usize = 32;
    pub const unk8: usize = 36;
    pub const unk9: usize = 40;
    pub const unk10: usize = 44;
    pub const unk11: usize = 48;
    pub const unk12: usize = 52;
}
pub mod unk_1453E3788 {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 1;
    pub const __pad0: usize = 0;
}
