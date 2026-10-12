//! Generated: MSVC x64 record layouts of the x64-zt IW6 headers
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
pub mod AddonMapEnts {
    pub const SIZE: usize = 104;
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
    pub const info: usize = 72;
    pub const numSubModels: usize = 80;
    pub const cmodels: usize = 88;
    pub const models: usize = 96;
}
pub mod AnimOverrideEntry {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const attachment1: usize = 0;
    pub const attachment2: usize = 2;
    pub const overrideAnim: usize = 8;
    pub const altmodeAnim: usize = 16;
    pub const animTreeType: usize = 24;
    pub const animTime: usize = 28;
    pub const altTime: usize = 32;
}
pub mod AnimationAimSet {
    pub const SIZE: usize = 112;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const rootName: usize = 4;
    pub const animName: usize = 8;
    pub const rootIndex: usize = 40;
    pub const animIndices: usize = 48;
}
pub mod AnimationClass {
    pub const SIZE: usize = 88;
    pub const ALIGN: usize = 8;
    pub const className: usize = 0;
    pub const name: usize = 0;
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
    pub const alias: usize = 0;
    pub const animName: usize = 4;
}
pub mod AnimationState {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const notify: usize = 4;
    pub const blendTime: usize = 8;
    pub const flags: usize = 12;
    pub const entryCount: usize = 13;
    pub const aimSet: usize = 16;
    pub const animEntries: usize = 24;
    pub const animIndices: usize = 32;
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
    pub const SIZE: usize = 64;
    pub const ALIGN: usize = 4;
    pub const adsSpread: usize = 0;
    pub const adsAimPitch: usize = 4;
    pub const adsTransInTime: usize = 8;
    pub const adsTransOutTime: usize = 12;
    pub const adsReloadTransTime: usize = 16;
    pub const adsCrosshairInFrac: usize = 20;
    pub const adsCrosshairOutFrac: usize = 24;
    pub const adsZoomFov: usize = 28;
    pub const adsZoomInFrac: usize = 32;
    pub const adsZoomOutFrac: usize = 36;
    pub const adsFovLerpTime: usize = 40;
    pub const adsBobFactor: usize = 44;
    pub const adsViewBobMult: usize = 48;
    pub const adsFireRateScale: usize = 52;
    pub const adsDamageRangeScale: usize = 56;
    pub const adsFireAnimFrac: usize = 60;
}
pub mod AttAddOns {
    pub const SIZE: usize = 3;
    pub const ALIGN: usize = 1;
    pub const motionTracker: usize = 0;
    pub const silenced: usize = 1;
    pub const riotShield: usize = 2;
}
pub mod AttAimAssist {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const autoAimRange: usize = 0;
    pub const aimAssistRange: usize = 4;
    pub const aimAssistRangeAds: usize = 8;
}
pub mod AttAmmoGeneral {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const penetrateType: usize = 0;
    pub const penetrateMultiplier: usize = 4;
    pub const impactType: usize = 8;
    pub const fireType: usize = 12;
    pub const tracerType: usize = 16;
    pub const rifleBullet: usize = 24;
    pub const armorPiercing: usize = 25;
}
pub mod AttAmmunition {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 4;
    pub const maxAmmo: usize = 0;
    pub const startAmmo: usize = 4;
    pub const clipSize: usize = 8;
    pub const shotCount: usize = 12;
    pub const reloadAmmoAdd: usize = 16;
    pub const reloadStartAdd: usize = 20;
}
pub mod AttDamage {
    pub const SIZE: usize = 28;
    pub const ALIGN: usize = 4;
    pub const damage: usize = 0;
    pub const minDamage: usize = 4;
    pub const meleeDamage: usize = 8;
    pub const maxDamageRange: usize = 12;
    pub const minDamageRange: usize = 16;
    pub const playerDamage: usize = 20;
    pub const minPlayerDamage: usize = 24;
}
pub mod AttGeneral {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const boltAction: usize = 0;
    pub const inheritsPerks: usize = 1;
    pub const reticleSpin45: usize = 2;
    pub const enemyCrosshairRange: usize = 4;
    pub const reticleCenter: usize = 8;
    pub const reticleSide: usize = 16;
    pub const reticleCenterSize: usize = 24;
    pub const reticleSideSize: usize = 28;
    pub const moveSpeedScale: usize = 32;
    pub const adsMoveSpeedScale: usize = 36;
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
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 4;
    pub const hipSpreadStandMin: usize = 0;
    pub const hipSpreadDuckedMin: usize = 4;
    pub const hipSpreadProneMin: usize = 8;
    pub const hipSpreadMax: usize = 12;
    pub const hipSpreadDuckedMax: usize = 16;
    pub const hipSpreadProneMax: usize = 20;
    pub const hipSpreadFireAdd: usize = 24;
    pub const hipSpreadTurnAdd: usize = 28;
    pub const hipSpreadMoveAdd: usize = 32;
    pub const hipSpreadDecayRate: usize = 36;
    pub const hipSpreadDuckedDecay: usize = 40;
    pub const hipSpreadProneDecay: usize = 44;
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
pub mod AttLocationDamage {
    pub const SIZE: usize = 76;
    pub const ALIGN: usize = 4;
    pub const locNone: usize = 0;
    pub const locHelmet: usize = 4;
    pub const locHead: usize = 8;
    pub const locNeck: usize = 12;
    pub const locTorsoUpper: usize = 16;
    pub const locTorsoLower: usize = 20;
    pub const locRightArmUpper: usize = 24;
    pub const locRightArmLower: usize = 28;
    pub const locRightHand: usize = 32;
    pub const locLeftArmUpper: usize = 36;
    pub const locLeftArmLower: usize = 40;
    pub const locLeftHand: usize = 44;
    pub const locRightLegUpper: usize = 48;
    pub const locRightLegLower: usize = 52;
    pub const locRightFoot: usize = 56;
    pub const locLeftLegUpper: usize = 60;
    pub const locLeftLegLower: usize = 64;
    pub const locLeftFoot: usize = 68;
    pub const locGun: usize = 72;
}
pub mod AttProjectile {
    pub const SIZE: usize = 144;
    pub const ALIGN: usize = 8;
    pub const explosionRadius: usize = 0;
    pub const explosionInnerDamage: usize = 4;
    pub const explosionOuterDamage: usize = 8;
    pub const damageConeAngle: usize = 12;
    pub const projectileSpeed: usize = 16;
    pub const projectileSpeedUp: usize = 20;
    pub const projectileActivateDist: usize = 24;
    pub const projectileLifetime: usize = 28;
    pub const projectileName: usize = 32;
    pub const projectileModel: usize = 40;
    pub const projExplosionType: usize = 48;
    pub const projExplosionEffect: usize = 56;
    pub const projExplosionEffectForceNormalUp: usize = 64;
    pub const projExplosionSound: usize = 72;
    pub const projDudEffect: usize = 80;
    pub const projDudSound: usize = 88;
    pub const projImpactExplode: usize = 96;
    pub const destabilizationRateTime: usize = 100;
    pub const destabilizationCurvatureMax: usize = 104;
    pub const destabilizeDistance: usize = 108;
    pub const projTrailEffect: usize = 112;
    pub const projIgnitionDelay: usize = 120;
    pub const projIgnitionEffect: usize = 128;
    pub const projIgnitionSound: usize = 136;
}
pub mod AttReload {
    pub const SIZE: usize = 2;
    pub const ALIGN: usize = 1;
    pub const noPartialReload: usize = 0;
    pub const segmentedReload: usize = 1;
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
    pub const SIZE: usize = 9;
    pub const ALIGN: usize = 1;
    pub const aimDownSight: usize = 0;
    pub const adsFire: usize = 1;
    pub const rechamberWhileAds: usize = 2;
    pub const noAdsWhenMagEmpty: usize = 3;
    pub const canHoldBreath: usize = 4;
    pub const canVariableZoom: usize = 5;
    pub const hideRailWithThisScope: usize = 6;
    pub const useScopeDrift: usize = 7;
    pub const useDualFOV: usize = 8;
}
pub mod AttUI {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const dpadIcon: usize = 0;
    pub const ammoCounterIcon: usize = 8;
    pub const dpadIconRatio: usize = 16;
    pub const ammoCounterIconRatio: usize = 20;
    pub const ammoCounterClip: usize = 24;
}
pub mod AttViewKick {
    pub const SIZE: usize = 48;
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
}
pub mod BModelDrawInfo {
    pub const SIZE: usize = 2;
    pub const ALIGN: usize = 2;
    pub const surfId: usize = 0;
}
pub mod BlendVertsUnknown {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 2;
    pub const b: usize = 0;
    pub const blendVertCountIndex: usize = 30;
}
pub mod Bounds {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 4;
    pub const midPoint: usize = 0;
    pub const halfSize: usize = 12;
}
pub mod BrushWrapper {
    pub const SIZE: usize = 88;
    pub const ALIGN: usize = 8;
    pub const bounds: usize = 0;
    pub const bounds__midPoint: usize = 0;
    pub const bounds__halfSize: usize = 12;
    pub const brush: usize = 24;
    pub const brush__numsides: usize = 24;
    pub const brush__glassPieceIndex: usize = 26;
    pub const brush__sides: usize = 32;
    pub const brush__baseAdjacentSide: usize = 40;
    pub const brush__axialMaterialNum: usize = 48;
    pub const brush__firstAdjacentSideOffsets: usize = 60;
    pub const brush__edgeCount: usize = 66;
    pub const totalEdgeCount: usize = 72;
    pub const planes: usize = 80;
}
pub mod ChannelMap {
    pub const SIZE: usize = 100;
    pub const ALIGN: usize = 4;
    pub const speakerCount: usize = 0;
    pub const speakers: usize = 4;
}
pub mod ClientTriggerAabbNode {
    pub const SIZE: usize = 28;
    pub const ALIGN: usize = 4;
    pub const bounds: usize = 0;
    pub const bounds__midPoint: usize = 0;
    pub const bounds__halfSize: usize = 12;
    pub const firstChild: usize = 24;
    pub const childCount: usize = 26;
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
    pub const SIZE: usize = 136;
    pub const ALIGN: usize = 8;
    pub const trigger: usize = 0;
    pub const trigger__count: usize = 0;
    pub const trigger__models: usize = 8;
    pub const trigger__hullCount: usize = 16;
    pub const trigger__hulls: usize = 24;
    pub const trigger__slabCount: usize = 32;
    pub const trigger__slabs: usize = 40;
    pub const numClientTriggerNodes: usize = 48;
    pub const clientTriggerAabbTree: usize = 56;
    pub const triggerStringLength: usize = 64;
    pub const triggerString: usize = 72;
    pub const visionSetTriggers: usize = 80;
    pub const triggerType: usize = 88;
    pub const origins: usize = 96;
    pub const scriptDelay: usize = 104;
    pub const audioTriggers: usize = 112;
    pub const blendLookup: usize = 120;
    pub const npcTriggers: usize = 128;
}
pub mod ClipInfo {
    pub const SIZE: usize = 128;
    pub const ALIGN: usize = 8;
    pub const planeCount: usize = 0;
    pub const planes: usize = 8;
    pub const numMaterials: usize = 16;
    pub const materials: usize = 24;
    pub const numBrushSides: usize = 32;
    pub const brushsides: usize = 40;
    pub const numBrushEdges: usize = 48;
    pub const brushEdges: usize = 56;
    pub const leafbrushNodesCount: usize = 64;
    pub const leafbrushNodes: usize = 72;
    pub const numLeafBrushes: usize = 80;
    pub const leafbrushes: usize = 88;
    pub const numBrushes: usize = 96;
    pub const brushes: usize = 104;
    pub const brushBounds: usize = 112;
    pub const brushContents: usize = 120;
}
pub mod ClipMaterial {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const surfaceFlags: usize = 8;
    pub const contents: usize = 12;
}
pub mod CollisionAabbTree {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const midPoint: usize = 0;
    pub const materialIndex: usize = 12;
    pub const childCount: usize = 14;
    pub const halfSize: usize = 16;
    pub const u: usize = 28;
    pub const u__firstChildIndex: usize = 28;
    pub const u__partitionIndex: usize = 28;
}
pub mod CollisionAabbTreeIndex {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 4;
    pub const firstChildIndex: usize = 0;
    pub const partitionIndex: usize = 0;
}
pub mod CollisionBorder {
    pub const SIZE: usize = 28;
    pub const ALIGN: usize = 4;
    pub const distEq: usize = 0;
    pub const zBase: usize = 12;
    pub const zSlope: usize = 16;
    pub const start: usize = 20;
    pub const length: usize = 24;
}
pub mod CollisionPartition {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const triCount: usize = 0;
    pub const borderCount: usize = 1;
    pub const firstVertSegment: usize = 2;
    pub const firstTri: usize = 4;
    pub const borders: usize = 8;
}
pub mod Colorization {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const powerAndDesaturation: usize = 0;
    pub const scale: usize = 16;
    pub const bias: usize = 28;
    pub const name: usize = 40;
}
pub mod ColorizationSet {
    pub const SIZE: usize = 104;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const lowAndHighEndPoints: usize = 8;
    pub const midEndPoints: usize = 24;
    pub const alternateDepthScaleAndBias: usize = 40;
    pub const colorization: usize = 48;
    pub const colorizationAlternate: usize = 56;
    pub const colorizationLow: usize = 64;
    pub const colorizationMid: usize = 72;
    pub const colorizationHigh: usize = 80;
    pub const alternateMaskMaterial: usize = 88;
    pub const flags: usize = 96;
}
pub mod ComPrimaryLight {
    pub const SIZE: usize = 120;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const canUseShadowMap: usize = 1;
    pub const exponent: usize = 2;
    pub const unused: usize = 3;
    pub const color: usize = 4;
    pub const dir: usize = 16;
    pub const up: usize = 28;
    pub const origin: usize = 40;
    pub const radius: usize = 52;
    pub const cosHalfFovOuter: usize = 56;
    pub const cosHalfFovInner: usize = 60;
    pub const cosHalfFovExpanded: usize = 64;
    pub const rotationLimit: usize = 68;
    pub const translationLimit: usize = 72;
    pub const cucRotationOffsetRad: usize = 76;
    pub const cucRotationSpeedRad: usize = 80;
    pub const cucScrollVector: usize = 84;
    pub const cucScaleVector: usize = 92;
    pub const cucTransVector: usize = 100;
    pub const defName: usize = 112;
}
pub mod ComPrimaryLightEnv {
    pub const SIZE: usize = 10;
    pub const ALIGN: usize = 2;
    pub const primaryLightIndices: usize = 0;
    pub const numIndices: usize = 8;
}
pub mod ComWorld {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const isInUse: usize = 8;
    pub const primaryLightCount: usize = 12;
    pub const primaryLights: usize = 16;
    pub const primaryLightEnvCount: usize = 24;
    pub const primaryLightEnvs: usize = 32;
}
pub mod ComputeShader {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const prog: usize = 8;
    pub const prog__cs: usize = 8;
    pub const prog__loadDef: usize = 16;
    pub const prog__loadDef__program: usize = 16;
    pub const prog__loadDef__programSize: usize = 24;
}
pub mod ComputeShaderProgram {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const cs: usize = 0;
    pub const loadDef: usize = 8;
    pub const loadDef__program: usize = 8;
    pub const loadDef__programSize: usize = 16;
}
pub mod ConditionalScript {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const eventHandlerSet: usize = 0;
    pub const eventExpression: usize = 8;
}
pub mod DBFile {
    pub const SIZE: usize = 88;
    pub const ALIGN: usize = 8;
    pub const handle: usize = 0;
    pub const memoryBufferSize: usize = 8;
    pub const memoryBuffer: usize = 16;
    pub const name: usize = 24;
}
pub mod DObjAnimMat {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const quat: usize = 0;
    pub const trans: usize = 16;
    pub const transWeight: usize = 28;
}
pub mod DopplerPreset {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const p_name: usize = 0;
    pub const name: usize = 0;
    pub const speedOfSound: usize = 8;
    pub const playerVelocityScale: usize = 12;
    pub const minPitch: usize = 16;
    pub const maxPitch: usize = 20;
    pub const smoothing: usize = 24;
}
pub mod DpvsPlane {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const coeffs: usize = 0;
}
pub mod DynEntityClient {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const physObjId: usize = 0;
    pub const flags: usize = 8;
    pub const lightingHandle: usize = 10;
    pub const health: usize = 12;
    pub const hinge: usize = 16;
    pub const activeModel: usize = 24;
    pub const contents: usize = 32;
}
pub mod DynEntityColl {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 4;
    pub const sector: usize = 0;
    pub const nextEntInSector: usize = 2;
    pub const linkMins: usize = 4;
    pub const linkMaxs: usize = 12;
}
pub mod DynEntityDef {
    pub const SIZE: usize = 120;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const pose: usize = 4;
    pub const pose__quat: usize = 4;
    pub const pose__origin: usize = 20;
    pub const baseModel: usize = 32;
    pub const brushModel: usize = 40;
    pub const physicsBrushModel: usize = 42;
    pub const scriptableIndex: usize = 44;
    pub const health: usize = 46;
    pub const destroyFx: usize = 48;
    pub const physPreset: usize = 56;
    pub const hinge: usize = 64;
    pub const linkTo: usize = 72;
    pub const mass: usize = 80;
    pub const mass__centerOfMass: usize = 80;
    pub const mass__momentsOfInertia: usize = 92;
    pub const mass__productsOfInertia: usize = 104;
    pub const contents: usize = 116;
}
pub mod DynEntityHingeDef {
    pub const SIZE: usize = 44;
    pub const ALIGN: usize = 4;
    pub const axisOrigin: usize = 0;
    pub const axisDir: usize = 12;
    pub const isLimited: usize = 24;
    pub const angleMin: usize = 28;
    pub const angleMax: usize = 32;
    pub const momentOfInertia: usize = 36;
    pub const friction: usize = 40;
}
pub mod DynEntityLinkToDef {
    pub const SIZE: usize = 28;
    pub const ALIGN: usize = 4;
    pub const anchorIndex: usize = 0;
    pub const originOffset: usize = 4;
    pub const angleOffset: usize = 16;
}
pub mod DynEntityPose {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const pose: usize = 0;
    pub const pose__quat: usize = 0;
    pub const pose__origin: usize = 16;
    pub const radius: usize = 28;
}
pub mod EventData {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const unconditionalScript: usize = 0;
    pub const conditionalScript: usize = 0;
    pub const elseScript: usize = 0;
    pub const setLocalVarData: usize = 0;
}
pub mod ExpressionPersistentState {
    pub const SIZE: usize = 84;
    pub const ALIGN: usize = 4;
    pub const flags: usize = 0;
    pub const playerDataKey: usize = 4;
    pub const lastExecuteTime: usize = 20;
    pub const lastResult: usize = 36;
}
pub mod ExpressionString {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const string: usize = 0;
}
pub mod ExpressionSupportingData {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const uifunctions: usize = 0;
    pub const uifunctions__totalFunctions: usize = 0;
    pub const uifunctions__functions: usize = 8;
    pub const staticDvarList: usize = 16;
    pub const staticDvarList__numStaticDvars: usize = 16;
    pub const staticDvarList__staticDvars: usize = 24;
    pub const uiStrings: usize = 32;
    pub const uiStrings__totalStrings: usize = 32;
    pub const uiStrings__strings: usize = 40;
}
pub mod ExtentBounds {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 4;
    pub const mins: usize = 0;
    pub const maxs: usize = 12;
}
pub mod FXOverrideEntry {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const attachment1: usize = 0;
    pub const attachment2: usize = 2;
    pub const overrideFX: usize = 8;
    pub const altmodeFX: usize = 16;
    pub const fxType: usize = 24;
}
pub mod Face {
    pub const SIZE: usize = 6;
    pub const ALIGN: usize = 2;
    pub const v1: usize = 0;
    pub const v2: usize = 2;
    pub const v3: usize = 4;
}
pub mod Font_s {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const fontName: usize = 0;
    pub const name: usize = 0;
    pub const pixelHeight: usize = 8;
    pub const glyphCount: usize = 12;
    pub const material: usize = 16;
    pub const glowMaterial: usize = 24;
    pub const glyphs: usize = 32;
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
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 2;
    pub const behavior: usize = 0;
    pub const index: usize = 1;
    pub const fps: usize = 2;
    pub const loopCount: usize = 3;
    pub const colIndexBits: usize = 4;
    pub const rowIndexBits: usize = 5;
    pub const entryCount: usize = 6;
}
pub mod FxElemDef {
    pub const SIZE: usize = 312;
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
    pub const atlas__loopCount: usize = 175;
    pub const atlas__colIndexBits: usize = 176;
    pub const atlas__rowIndexBits: usize = 177;
    pub const atlas__entryCount: usize = 178;
    pub const elemType: usize = 180;
    pub const elemLitType: usize = 181;
    pub const visualCount: usize = 182;
    pub const velIntervalCount: usize = 183;
    pub const visStateIntervalCount: usize = 184;
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
    pub const litMaxColorChangePerSec: usize = 296;
    pub const unlitHDRScalar: usize = 300;
    pub const litHDRScalar: usize = 304;
    pub const litUnlitBlendFactor: usize = 308;
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
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 4;
    pub const base: usize = 0;
    pub const base__color: usize = 0;
    pub const base__unlitHDRScale: usize = 16;
    pub const base__rotationDelta: usize = 20;
    pub const base__rotationTotal: usize = 24;
    pub const base__size: usize = 28;
    pub const base__scale: usize = 36;
    pub const amplitude: usize = 40;
    pub const amplitude__color: usize = 40;
    pub const amplitude__unlitHDRScale: usize = 56;
    pub const amplitude__rotationDelta: usize = 60;
    pub const amplitude__rotationTotal: usize = 64;
    pub const amplitude__size: usize = 68;
    pub const amplitude__scale: usize = 76;
}
pub mod FxElemVisualState {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 4;
    pub const color: usize = 0;
    pub const unlitHDRScale: usize = 16;
    pub const rotationDelta: usize = 20;
    pub const rotationTotal: usize = 24;
    pub const size: usize = 28;
    pub const scale: usize = 36;
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
    pub const SIZE: usize = 120;
    pub const ALIGN: usize = 8;
    pub const halfThickness: usize = 0;
    pub const texVecs: usize = 4;
    pub const color: usize = 20;
    pub const color__array: usize = 20;
    pub const color__packed: usize = 20;
    pub const material: usize = 24;
    pub const materialShattered: usize = 32;
    pub const physPreset: usize = 40;
    pub const pieceBreakEffect: usize = 48;
    pub const shatterEffect: usize = 56;
    pub const shatterSmallEffect: usize = 64;
    pub const crackDecalEffect: usize = 72;
    pub const damagedSound: usize = 80;
    pub const destroyedSound: usize = 88;
    pub const destroyedQuietSound: usize = 96;
    pub const invHighMipRadius: usize = 104;
    pub const shatteredInvHighMipRadius: usize = 108;
    pub const numCrackRings: usize = 112;
    pub const isOpaque: usize = 116;
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
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const fallTime: usize = 0;
    pub const physObjId: usize = 8;
    pub const physJointId: usize = 16;
    pub const vel: usize = 24;
    pub const avel: usize = 36;
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
pub mod FxIntRange {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const base: usize = 0;
    pub const amplitude: usize = 4;
}
pub mod FxParticleSimAnimation {
    pub const SIZE: usize = 96;
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
    pub const header__evalVisStatePerParticle: usize = 68;
    pub const header__sortParticlesAtRuntime: usize = 69;
    pub const particleData: usize = 72;
    pub const frames: usize = 80;
    pub const colorTable: usize = 88;
}
pub mod FxParticleSimAnimationFrame {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const particleDataOffset: usize = 0;
    pub const numActiveParticles: usize = 4;
}
pub mod FxParticleSimAnimationHeader {
    pub const SIZE: usize = 56;
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
    pub const evalVisStatePerParticle: usize = 52;
    pub const sortParticlesAtRuntime: usize = 53;
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
    pub const SIZE: usize = 144;
    pub const ALIGN: usize = 8;
    pub const glassPieces: usize = 0;
    pub const pieceCount: usize = 8;
    pub const damageToWeaken: usize = 12;
    pub const damageToDestroy: usize = 14;
    pub const glassNameCount: usize = 16;
    pub const glassNames: usize = 24;
    pub const pad: usize = 32;
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
    pub const SIZE: usize = 72;
    pub const ALIGN: usize = 8;
    pub const bounds: usize = 0;
    pub const bounds__midPoint: usize = 0;
    pub const bounds__halfSize: usize = 12;
    pub const portalCount: usize = 24;
    pub const portals: usize = 32;
    pub const reflectionProbeCount: usize = 40;
    pub const reflectionProbes: usize = 48;
    pub const reflectionProbeReferenceCount: usize = 56;
    pub const reflectionProbeReferences: usize = 64;
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
pub mod GfxColorFloat {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const array: usize = 0;
}
pub mod GfxColorization {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 4;
    pub const powerAndDesaturation: usize = 0;
    pub const scale: usize = 16;
    pub const bias: usize = 28;
}
pub mod GfxComputeShaderLoadDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const program: usize = 0;
    pub const programSize: usize = 8;
}
pub mod GfxDomainShaderLoadDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const program: usize = 0;
    pub const programSize: usize = 8;
    pub const loadForRenderer: usize = 12;
}
pub mod GfxDrawSurf {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const fields: usize = 0;
    pub const fields__objectId: usize = 0;
    pub const fields__reflectionProbeIndex: usize = 2;
    pub const fields__hasGfxEntIndex: usize = 3;
    pub const fields__customIndex: usize = 3;
    pub const fields__materialSortedIndex: usize = 3;
    pub const fields__tessellation: usize = 5;
    pub const fields__prepass: usize = 5;
    pub const fields__useHeroLighting: usize = 6;
    pub const fields__sceneLightEnvIndex: usize = 8;
    pub const fields__viewModelRender: usize = 10;
    pub const fields__surfType: usize = 10;
    pub const fields__primarySortKey: usize = 10;
    pub const fields__unused: usize = 11;
    pub const packed: usize = 0;
    pub const packed__p0: usize = 0;
    pub const packed__p1: usize = 8;
}
pub mod GfxDrawSurfFields {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const objectId: usize = 0;
    pub const reflectionProbeIndex: usize = 2;
    pub const hasGfxEntIndex: usize = 3;
    pub const customIndex: usize = 3;
    pub const materialSortedIndex: usize = 3;
    pub const tessellation: usize = 5;
    pub const prepass: usize = 5;
    pub const useHeroLighting: usize = 6;
    pub const sceneLightEnvIndex: usize = 8;
    pub const viewModelRender: usize = 10;
    pub const surfType: usize = 10;
    pub const primarySortKey: usize = 10;
    pub const unused: usize = 11;
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
    pub const loadForRenderer: usize = 12;
}
pub mod GfxImage {
    pub const SIZE: usize = 104;
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
    pub const mapType: usize = 28;
    pub const semantic: usize = 29;
    pub const category: usize = 30;
    pub const flags: usize = 31;
    pub const picmip: usize = 32;
    pub const picmip__platform: usize = 32;
    pub const __pad0: usize = 34;
    pub const dataLen1: usize = 36;
    pub const dataLen2: usize = 40;
    pub const width: usize = 44;
    pub const height: usize = 46;
    pub const depth: usize = 48;
    pub const numElements: usize = 50;
    pub const levelCount: usize = 52;
    pub const streamed: usize = 53;
    pub const __pad1: usize = 54;
    pub const pixelData: usize = 56;
    pub const streams: usize = 64;
    pub const name: usize = 96;
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
pub mod GfxLightDef {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const attenuation: usize = 8;
    pub const attenuation__image: usize = 8;
    pub const attenuation__samplerState: usize = 16;
    pub const cucoloris: usize = 24;
    pub const cucoloris__image: usize = 24;
    pub const cucoloris__samplerState: usize = 32;
    pub const lmapLookupStart: usize = 40;
}
pub mod GfxLightGrid {
    pub const SIZE: usize = 1568;
    pub const ALIGN: usize = 8;
    pub const hasLightRegions: usize = 0;
    pub const useSkyForLowZ: usize = 1;
    pub const lastSunPrimaryLightIndex: usize = 4;
    pub const mins: usize = 8;
    pub const maxs: usize = 14;
    pub const rowAxis: usize = 20;
    pub const colAxis: usize = 24;
    pub const rowDataStart: usize = 32;
    pub const rawRowDataSize: usize = 40;
    pub const rawRowData: usize = 48;
    pub const entryCount: usize = 56;
    pub const entries: usize = 64;
    pub const colorCount: usize = 72;
    pub const colors: usize = 80;
    pub const missingGridColorIndex: usize = 88;
    pub const tableVersion: usize = 92;
    pub const paletteVersion: usize = 96;
    pub const rangeExponent8BitsEncoding: usize = 100;
    pub const rangeExponent12BitsEncoding: usize = 101;
    pub const rangeExponent16BitsEncoding: usize = 102;
    pub const stageCount: usize = 103;
    pub const stageLightingContrastGain: usize = 104;
    pub const paletteEntryCount: usize = 112;
    pub const paletteEntryAddress: usize = 120;
    pub const paletteBitstreamSize: usize = 128;
    pub const paletteBitstream: usize = 136;
    pub const skyLightGridColors: usize = 144;
    pub const skyLightGridColors__rgb: usize = 144;
    pub const defaultLightGridColors: usize = 816;
    pub const defaultLightGridColors__rgb: usize = 816;
    pub const tree: usize = 1488;
    pub const tree__maxDepth: usize = 1488;
    pub const tree__nodeCount: usize = 1492;
    pub const tree__leafCount: usize = 1496;
    pub const tree__coordMinGridSpace: usize = 1500;
    pub const tree__coordMaxGridSpace: usize = 1512;
    pub const tree__coordHalfSizeGridSpace: usize = 1524;
    pub const tree__defaultColorIndexBitCount: usize = 1536;
    pub const tree__defaultLightIndexBitCount: usize = 1540;
    pub const tree__p_nodeTable: usize = 1544;
    pub const tree__leafTableSize: usize = 1552;
    pub const tree__p_leafTable: usize = 1560;
}
pub mod GfxLightGridColors {
    pub const SIZE: usize = 168;
    pub const ALIGN: usize = 1;
    pub const rgb: usize = 0;
}
pub mod GfxLightGridColorsHDR {
    pub const SIZE: usize = 672;
    pub const ALIGN: usize = 4;
    pub const rgb: usize = 0;
}
pub mod GfxLightGridEntry {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const colorsIndex: usize = 0;
    pub const primaryLightEnvIndex: usize = 4;
    pub const unused: usize = 6;
    pub const needsTrace: usize = 7;
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
pub mod GfxLightImage {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const image: usize = 0;
    pub const samplerState: usize = 8;
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
pub mod GfxLightmapArray {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const primary: usize = 0;
    pub const secondary: usize = 8;
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
    pub const normal: usize = 24;
    pub const normal__packed: usize = 24;
    pub const tangent: usize = 28;
    pub const tangent__packed: usize = 28;
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
    pub const binormalSign: usize = 12;
    pub const color: usize = 16;
    pub const color__array: usize = 16;
    pub const color__packed: usize = 16;
    pub const texCoord: usize = 20;
    pub const texCoord__packed: usize = 20;
    pub const normal: usize = 24;
    pub const normal__packed: usize = 24;
    pub const tangent: usize = 28;
    pub const tangent__packed: usize = 28;
}
pub mod GfxPixelShaderLoadDef {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const program: usize = 0;
    pub const programSize: usize = 8;
    pub const loadForRenderer: usize = 12;
    pub const microCodeCrc: usize = 16;
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
pub mod GfxQuantizedMotionVertex {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 4;
    pub const xyz: usize = 0;
    pub const binormalSignAndHeight: usize = 6;
    pub const normal: usize = 8;
    pub const normal__packed: usize = 8;
    pub const tangent: usize = 12;
    pub const tangent__packed: usize = 12;
    pub const texCoord: usize = 16;
    pub const texCoord__packed: usize = 16;
    pub const pieceIndex: usize = 20;
    pub const pieceIndex__array: usize = 20;
    pub const pieceIndex__packed: usize = 20;
}
pub mod GfxQuantizedNoColorMotionVertex {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 4;
    pub const xyz: usize = 0;
    pub const binormalSignAndHeight: usize = 6;
    pub const normal: usize = 8;
    pub const normal__packed: usize = 8;
    pub const tangent: usize = 12;
    pub const tangent__packed: usize = 12;
    pub const texCoord: usize = 16;
    pub const texCoord__packed: usize = 16;
    pub const pieceIndex: usize = 20;
    pub const pieceIndex__array: usize = 20;
    pub const pieceIndex__packed: usize = 20;
}
pub mod GfxQuantizedNoColorVertex {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 4;
    pub const xyz: usize = 0;
    pub const binormalSign: usize = 6;
    pub const normal: usize = 8;
    pub const normal__packed: usize = 8;
    pub const tangent: usize = 12;
    pub const tangent__packed: usize = 12;
    pub const texCoord: usize = 16;
    pub const texCoord__packed: usize = 16;
}
pub mod GfxQuantizedVertex {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 4;
    pub const xyz: usize = 0;
    pub const binormalSign: usize = 6;
    pub const normal: usize = 8;
    pub const normal__packed: usize = 8;
    pub const tangent: usize = 12;
    pub const tangent__packed: usize = 12;
    pub const texCoord: usize = 16;
    pub const texCoord__packed: usize = 16;
    pub const color: usize = 20;
    pub const color__array: usize = 20;
    pub const color__packed: usize = 20;
}
pub mod GfxReflectionProbe {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const origin: usize = 0;
    pub const probeVolumes: usize = 16;
    pub const probeVolumeCount: usize = 24;
}
pub mod GfxReflectionProbeReference {
    pub const SIZE: usize = 1;
    pub const ALIGN: usize = 1;
    pub const index: usize = 0;
}
pub mod GfxReflectionProbeReferenceOrigin {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const origin: usize = 0;
}
pub mod GfxReflectionProbeVolumeData {
    pub const SIZE: usize = 96;
    pub const ALIGN: usize = 4;
    pub const volumePlanes: usize = 0;
}
pub mod GfxSceneDynBrush {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 2;
    pub const info: usize = 0;
    pub const info__surfId: usize = 0;
    pub const dynEntId: usize = 2;
}
pub mod GfxSceneDynModel {
    pub const SIZE: usize = 6;
    pub const ALIGN: usize = 2;
    pub const info: usize = 0;
    pub const info__hasGfxEntIndex: usize = 0;
    pub const info__lod: usize = 1;
    pub const info__surfId: usize = 2;
    pub const dynEntId: usize = 4;
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
    pub const SIZE: usize = 28;
    pub const ALIGN: usize = 4;
    pub const loadBits: usize = 0;
    pub const zone: usize = 12;
    pub const depthStencilState: usize = 13;
    pub const blendState: usize = 24;
    pub const rasterizerState: usize = 25;
}
pub mod GfxStateBitsRaw {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const flags: usize = 0;
    pub const flags__loadbit0: usize = 0;
    pub const flags__loadbit1: usize = 4;
    pub const flags__loadbit2: usize = 8;
    pub const loadBits: usize = 0;
}
pub mod GfxStatebitsFlags {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const loadbit0: usize = 0;
    pub const loadbit1: usize = 4;
    pub const loadbit2: usize = 8;
}
pub mod GfxStaticModelDrawInst {
    pub const SIZE: usize = 176;
    pub const ALIGN: usize = 8;
    pub const placement: usize = 0;
    pub const placement__origin: usize = 0;
    pub const placement__axis: usize = 12;
    pub const placement__scale: usize = 48;
    pub const model: usize = 56;
    pub const groundLighting: usize = 64;
    pub const vertexLightingInfo: usize = 80;
    pub const vertexLightingInfo__lightingValues: usize = 80;
    pub const vertexLightingInfo__lightingValuesVb: usize = 88;
    pub const vertexLightingInfo__lightingValuesSb: usize = 96;
    pub const vertexLightingInfo__cache: usize = 104;
    pub const vertexLightingInfo__cache__size: usize = 104;
    pub const vertexLightingInfo__cache__subdivCacheBuffer: usize = 112;
    pub const vertexLightingInfo__cache__subdivCacheView: usize = 120;
    pub const vertexLightingInfo__flags: usize = 128;
    pub const vertexLightingInfo__numLightingValues: usize = 132;
    pub const modelLightmapInfo: usize = 136;
    pub const modelLightmapInfo__offset: usize = 136;
    pub const modelLightmapInfo__scale: usize = 144;
    pub const modelLightmapInfo__lightmapIndex: usize = 152;
    pub const lightingHandle: usize = 156;
    pub const cullDist: usize = 158;
    pub const flags: usize = 160;
    pub const staticModelId: usize = 162;
    pub const primaryLightEnvIndex: usize = 164;
    pub const reflectionProbeIndex: usize = 166;
    pub const firstMtlSkinIndex: usize = 167;
    pub const sunShadowFlags: usize = 168;
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
    pub const lightingValuesVb: usize = 8;
    pub const lightingValuesSb: usize = 16;
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
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const tris: usize = 0;
    pub const tris__vertexLayerData: usize = 0;
    pub const tris__firstVertex: usize = 4;
    pub const tris__maxEdgeLength: usize = 8;
    pub const tris__vertexCount: usize = 12;
    pub const tris__triCount: usize = 14;
    pub const tris__baseIndex: usize = 16;
    pub const material: usize = 24;
    pub const laf: usize = 32;
    pub const laf__fields: usize = 32;
    pub const laf__fields__lightmapIndex: usize = 32;
    pub const laf__fields__reflectionProbeIndex: usize = 33;
    pub const laf__fields__primaryLightEnvIndex: usize = 34;
    pub const laf__fields__flags: usize = 36;
    pub const laf__fields__unused: usize = 37;
    pub const laf__packed: usize = 32;
}
pub mod GfxSurfaceBounds {
    pub const SIZE: usize = 36;
    pub const ALIGN: usize = 4;
    pub const bounds: usize = 0;
    pub const bounds__midPoint: usize = 0;
    pub const bounds__halfSize: usize = 12;
    pub const flags: usize = 24;
    pub const mipRadius: usize = 26;
    pub const invHighMipRadius: usize = 28;
}
pub mod GfxSurfaceLightingAndFlags {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const fields: usize = 0;
    pub const fields__lightmapIndex: usize = 0;
    pub const fields__reflectionProbeIndex: usize = 1;
    pub const fields__primaryLightEnvIndex: usize = 2;
    pub const fields__flags: usize = 4;
    pub const fields__unused: usize = 5;
    pub const packed: usize = 0;
}
pub mod GfxSurfaceLightingAndFlagsFields {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 2;
    pub const lightmapIndex: usize = 0;
    pub const reflectionProbeIndex: usize = 1;
    pub const primaryLightEnvIndex: usize = 2;
    pub const flags: usize = 4;
    pub const unused: usize = 5;
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
pub mod GfxToneMapping {
    pub const SIZE: usize = 104;
    pub const ALIGN: usize = 4;
    pub const toeStrength: usize = 0;
    pub const toeAngleNumerator: usize = 12;
    pub const toeAngleDenominator: usize = 24;
    pub const linearStrength: usize = 36;
    pub const linearAngle: usize = 48;
    pub const shoulderStrength: usize = 60;
    pub const whitePoint: usize = 72;
    pub const exposure: usize = 84;
    pub const adaptationRates: usize = 96;
}
pub mod GfxVertexShaderLoadDef {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const program: usize = 0;
    pub const programSize: usize = 8;
    pub const loadForRenderer: usize = 12;
    pub const microCodeCrc: usize = 16;
}
pub mod GfxVertexUnion0 {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const quantizedNoColorVerts0: usize = 0;
    pub const quantizedNoColorMotionVerts0: usize = 0;
    pub const quantizedVerts0: usize = 0;
    pub const quantizedMotionVerts0: usize = 0;
    pub const packedVerts0: usize = 0;
    pub const packedMotionVerts0: usize = 0;
    pub const verts0: usize = 0;
}
pub mod GfxWorld {
    pub const SIZE: usize = 2608;
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
    pub const primaryLightEnvCount: usize = 56;
    pub const sortKeyLitDecal: usize = 60;
    pub const sortKeyEffectDecal: usize = 64;
    pub const sortKeyTopDecal: usize = 68;
    pub const sortKeyEffectAuto: usize = 72;
    pub const sortKeyDistortion: usize = 76;
    pub const dpvsPlanes: usize = 80;
    pub const dpvsPlanes__cellCount: usize = 80;
    pub const dpvsPlanes__planes: usize = 88;
    pub const dpvsPlanes__nodes: usize = 96;
    pub const dpvsPlanes__sceneEntCellBits: usize = 104;
    pub const aabbTreeCounts: usize = 112;
    pub const aabbTrees: usize = 120;
    pub const cells: usize = 128;
    pub const draw: usize = 136;
    pub const draw__reflectionProbeCount: usize = 136;
    pub const draw__reflectionProbes: usize = 144;
    pub const draw__reflectionProbeOrigins: usize = 152;
    pub const draw__reflectionProbeTextures: usize = 160;
    pub const draw__reflectionProbeReferenceCount: usize = 168;
    pub const draw__reflectionProbeReferenceOrigins: usize = 176;
    pub const draw__reflectionProbeReferences: usize = 184;
    pub const draw__lightmapCount: usize = 192;
    pub const draw__lightmaps: usize = 200;
    pub const draw__lightmapPrimaryTextures: usize = 208;
    pub const draw__lightmapSecondaryTextures: usize = 216;
    pub const draw__lightmapOverridePrimary: usize = 224;
    pub const draw__lightmapOverrideSecondary: usize = 232;
    pub const draw__trisType: usize = 240;
    pub const draw__vertexCount: usize = 244;
    pub const draw__vd: usize = 248;
    pub const draw__vd__vertices: usize = 248;
    pub const draw__vd__worldVb: usize = 256;
    pub const draw__vertexLayerDataSize: usize = 264;
    pub const draw__vld: usize = 272;
    pub const draw__vld__data: usize = 272;
    pub const draw__vld__layerVb: usize = 280;
    pub const draw__indexCount: usize = 288;
    pub const draw__indices: usize = 296;
    pub const draw__indexBuffer: usize = 304;
    pub const lightGrid: usize = 312;
    pub const lightGrid__hasLightRegions: usize = 312;
    pub const lightGrid__useSkyForLowZ: usize = 313;
    pub const lightGrid__lastSunPrimaryLightIndex: usize = 316;
    pub const lightGrid__mins: usize = 320;
    pub const lightGrid__maxs: usize = 326;
    pub const lightGrid__rowAxis: usize = 332;
    pub const lightGrid__colAxis: usize = 336;
    pub const lightGrid__rowDataStart: usize = 344;
    pub const lightGrid__rawRowDataSize: usize = 352;
    pub const lightGrid__rawRowData: usize = 360;
    pub const lightGrid__entryCount: usize = 368;
    pub const lightGrid__entries: usize = 376;
    pub const lightGrid__colorCount: usize = 384;
    pub const lightGrid__colors: usize = 392;
    pub const lightGrid__missingGridColorIndex: usize = 400;
    pub const lightGrid__tableVersion: usize = 404;
    pub const lightGrid__paletteVersion: usize = 408;
    pub const lightGrid__rangeExponent8BitsEncoding: usize = 412;
    pub const lightGrid__rangeExponent12BitsEncoding: usize = 413;
    pub const lightGrid__rangeExponent16BitsEncoding: usize = 414;
    pub const lightGrid__stageCount: usize = 415;
    pub const lightGrid__stageLightingContrastGain: usize = 416;
    pub const lightGrid__paletteEntryCount: usize = 424;
    pub const lightGrid__paletteEntryAddress: usize = 432;
    pub const lightGrid__paletteBitstreamSize: usize = 440;
    pub const lightGrid__paletteBitstream: usize = 448;
    pub const lightGrid__skyLightGridColors: usize = 456;
    pub const lightGrid__skyLightGridColors__rgb: usize = 456;
    pub const lightGrid__defaultLightGridColors: usize = 1128;
    pub const lightGrid__defaultLightGridColors__rgb: usize = 1128;
    pub const lightGrid__tree: usize = 1800;
    pub const lightGrid__tree__maxDepth: usize = 1800;
    pub const lightGrid__tree__nodeCount: usize = 1804;
    pub const lightGrid__tree__leafCount: usize = 1808;
    pub const lightGrid__tree__coordMinGridSpace: usize = 1812;
    pub const lightGrid__tree__coordMaxGridSpace: usize = 1824;
    pub const lightGrid__tree__coordHalfSizeGridSpace: usize = 1836;
    pub const lightGrid__tree__defaultColorIndexBitCount: usize = 1848;
    pub const lightGrid__tree__defaultLightIndexBitCount: usize = 1852;
    pub const lightGrid__tree__p_nodeTable: usize = 1856;
    pub const lightGrid__tree__leafTableSize: usize = 1864;
    pub const lightGrid__tree__p_leafTable: usize = 1872;
    pub const modelCount: usize = 1880;
    pub const models: usize = 1888;
    pub const bounds: usize = 1896;
    pub const bounds__midPoint: usize = 1896;
    pub const bounds__halfSize: usize = 1908;
    pub const checksum: usize = 1920;
    pub const materialMemoryCount: usize = 1924;
    pub const materialMemory: usize = 1928;
    pub const sun: usize = 1936;
    pub const sun__hasValidData: usize = 1936;
    pub const sun__spriteMaterial: usize = 1944;
    pub const sun__flareMaterial: usize = 1952;
    pub const sun__spriteSize: usize = 1960;
    pub const sun__flareMinSize: usize = 1964;
    pub const sun__flareMinDot: usize = 1968;
    pub const sun__flareMaxSize: usize = 1972;
    pub const sun__flareMaxDot: usize = 1976;
    pub const sun__flareMaxAlpha: usize = 1980;
    pub const sun__flareFadeInTime: usize = 1984;
    pub const sun__flareFadeOutTime: usize = 1988;
    pub const sun__blindMinDot: usize = 1992;
    pub const sun__blindMaxDot: usize = 1996;
    pub const sun__blindMaxDarken: usize = 2000;
    pub const sun__blindFadeInTime: usize = 2004;
    pub const sun__blindFadeOutTime: usize = 2008;
    pub const sun__glareMinDot: usize = 2012;
    pub const sun__glareMaxDot: usize = 2016;
    pub const sun__glareMaxLighten: usize = 2020;
    pub const sun__glareFadeInTime: usize = 2024;
    pub const sun__glareFadeOutTime: usize = 2028;
    pub const sun__sunFxPosition: usize = 2032;
    pub const outdoorLookupMatrix: usize = 2048;
    pub const outdoorImage: usize = 2112;
    pub const cellCasterBits: usize = 2120;
    pub const cellHasSunLitSurfsBits: usize = 2128;
    pub const sceneDynModel: usize = 2136;
    pub const sceneDynBrush: usize = 2144;
    pub const primaryLightEntityShadowVis: usize = 2152;
    pub const primaryLightDynEntShadowVis: usize = 2160;
    pub const nonSunPrimaryLightForModelDynEnt: usize = 2176;
    pub const shadowGeom: usize = 2184;
    pub const shadowGeomOptimized: usize = 2192;
    pub const lightRegion: usize = 2200;
    pub const dpvs: usize = 2208;
    pub const dpvs__smodelCount: usize = 2208;
    pub const dpvs__staticSurfaceCount: usize = 2212;
    pub const dpvs__litOpaqueSurfsBegin: usize = 2216;
    pub const dpvs__litOpaqueSurfsEnd: usize = 2220;
    pub const dpvs__litDecalSurfsBegin: usize = 2224;
    pub const dpvs__litDecalSurfsEnd: usize = 2228;
    pub const dpvs__litTransSurfsBegin: usize = 2232;
    pub const dpvs__litTransSurfsEnd: usize = 2236;
    pub const dpvs__shadowCasterSurfsBegin: usize = 2240;
    pub const dpvs__shadowCasterSurfsEnd: usize = 2244;
    pub const dpvs__emissiveSurfsBegin: usize = 2248;
    pub const dpvs__emissiveSurfsEnd: usize = 2252;
    pub const dpvs__smodelVisDataCount: usize = 2256;
    pub const dpvs__surfaceVisDataCount: usize = 2260;
    pub const dpvs__smodelVisData: usize = 2264;
    pub const dpvs__surfaceVisData: usize = 2288;
    pub const dpvs__unknownData01: usize = 2312;
    pub const dpvs__unknownData02: usize = 2336;
    pub const dpvs__lodData: usize = 2360;
    pub const dpvs__tessellationCutoffVisData: usize = 2368;
    pub const dpvs__sortedSurfIndex: usize = 2392;
    pub const dpvs__smodelInsts: usize = 2400;
    pub const dpvs__surfaces: usize = 2408;
    pub const dpvs__surfacesBounds: usize = 2416;
    pub const dpvs__smodelDrawInsts: usize = 2424;
    pub const dpvs__surfaceMaterials: usize = 2432;
    pub const dpvs__surfaceCastsSunShadow: usize = 2440;
    pub const dpvs__sunShadowOptCount: usize = 2448;
    pub const dpvs__sunSurfVisDataCount: usize = 2452;
    pub const dpvs__surfaceCastsSunShadowOpt: usize = 2456;
    pub const dpvs__constantBuffersLit: usize = 2464;
    pub const dpvs__constantBuffersAmbient: usize = 2472;
    pub const dpvs__usageCount: usize = 2480;
    pub const dpvsDyn: usize = 2488;
    pub const dpvsDyn__dynEntClientWordCount: usize = 2488;
    pub const dpvsDyn__dynEntClientCount: usize = 2496;
    pub const dpvsDyn__dynEntCellBits: usize = 2504;
    pub const dpvsDyn__dynEntVisData: usize = 2520;
    pub const mapVtxChecksum: usize = 2568;
    pub const heroOnlyLightCount: usize = 2572;
    pub const heroOnlyLights: usize = 2576;
    pub const fogTypesAllowed: usize = 2584;
    pub const umbraTomeSize: usize = 2588;
    pub const umbraTomeData: usize = 2592;
    pub const umbraTomePtr: usize = 2600;
}
pub mod GfxWorldDpvsDynamic {
    pub const SIZE: usize = 80;
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
    pub const SIZE: usize = 280;
    pub const ALIGN: usize = 8;
    pub const smodelCount: usize = 0;
    pub const staticSurfaceCount: usize = 4;
    pub const litOpaqueSurfsBegin: usize = 8;
    pub const litOpaqueSurfsEnd: usize = 12;
    pub const litDecalSurfsBegin: usize = 16;
    pub const litDecalSurfsEnd: usize = 20;
    pub const litTransSurfsBegin: usize = 24;
    pub const litTransSurfsEnd: usize = 28;
    pub const shadowCasterSurfsBegin: usize = 32;
    pub const shadowCasterSurfsEnd: usize = 36;
    pub const emissiveSurfsBegin: usize = 40;
    pub const emissiveSurfsEnd: usize = 44;
    pub const smodelVisDataCount: usize = 48;
    pub const surfaceVisDataCount: usize = 52;
    pub const smodelVisData: usize = 56;
    pub const surfaceVisData: usize = 80;
    pub const unknownData01: usize = 104;
    pub const unknownData02: usize = 128;
    pub const lodData: usize = 152;
    pub const tessellationCutoffVisData: usize = 160;
    pub const sortedSurfIndex: usize = 184;
    pub const smodelInsts: usize = 192;
    pub const surfaces: usize = 200;
    pub const surfacesBounds: usize = 208;
    pub const smodelDrawInsts: usize = 216;
    pub const surfaceMaterials: usize = 224;
    pub const surfaceCastsSunShadow: usize = 232;
    pub const sunShadowOptCount: usize = 240;
    pub const sunSurfVisDataCount: usize = 244;
    pub const surfaceCastsSunShadowOpt: usize = 248;
    pub const constantBuffersLit: usize = 256;
    pub const constantBuffersAmbient: usize = 264;
    pub const usageCount: usize = 272;
}
pub mod GfxWorldDraw {
    pub const SIZE: usize = 176;
    pub const ALIGN: usize = 8;
    pub const reflectionProbeCount: usize = 0;
    pub const reflectionProbes: usize = 8;
    pub const reflectionProbeOrigins: usize = 16;
    pub const reflectionProbeTextures: usize = 24;
    pub const reflectionProbeReferenceCount: usize = 32;
    pub const reflectionProbeReferenceOrigins: usize = 40;
    pub const reflectionProbeReferences: usize = 48;
    pub const lightmapCount: usize = 56;
    pub const lightmaps: usize = 64;
    pub const lightmapPrimaryTextures: usize = 72;
    pub const lightmapSecondaryTextures: usize = 80;
    pub const lightmapOverridePrimary: usize = 88;
    pub const lightmapOverrideSecondary: usize = 96;
    pub const trisType: usize = 104;
    pub const vertexCount: usize = 108;
    pub const vd: usize = 112;
    pub const vd__vertices: usize = 112;
    pub const vd__worldVb: usize = 120;
    pub const vertexLayerDataSize: usize = 128;
    pub const vld: usize = 136;
    pub const vld__data: usize = 136;
    pub const vld__layerVb: usize = 144;
    pub const indexCount: usize = 152;
    pub const indices: usize = 160;
    pub const indexBuffer: usize = 168;
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
    pub const tangent: usize = 40;
    pub const tangent__packed: usize = 40;
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
pub mod Glyph {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 4;
    pub const letter: usize = 0;
    pub const x0: usize = 2;
    pub const y0: usize = 3;
    pub const dx: usize = 4;
    pub const pixelWidth: usize = 5;
    pub const pixelHeight: usize = 6;
    pub const s0: usize = 8;
    pub const t0: usize = 12;
    pub const s1: usize = 16;
    pub const t1: usize = 20;
}
pub mod Hinge {
    pub const SIZE: usize = 72;
    pub const ALIGN: usize = 8;
    pub const angle: usize = 0;
    pub const quat: usize = 4;
    pub const angularVel: usize = 20;
    pub const torqueAccum: usize = 24;
    pub const active: usize = 28;
    pub const autoDisableTimeLeft: usize = 32;
    pub const def: usize = 40;
    pub const physPreset: usize = 48;
    pub const centerOfMassRelToAxisOriginAtAngleZero: usize = 56;
}
pub mod ItemFloatExpression {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const target: usize = 0;
    pub const expression: usize = 8;
}
pub mod ItemKeyHandler {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const key: usize = 0;
    pub const action: usize = 8;
    pub const next: usize = 16;
}
pub mod LoadedSound {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const sound: usize = 8;
    pub const sound__data: usize = 8;
    pub const sound__seekTable: usize = 16;
    pub const sound__format: usize = 24;
    pub const sound__format__sampleRate: usize = 24;
    pub const sound__format__dataByteCount: usize = 28;
    pub const sound__format__numSamples: usize = 32;
    pub const sound__format__avgBytesPerSec: usize = 36;
    pub const sound__format__channels: usize = 40;
    pub const sound__format__numBits: usize = 42;
    pub const sound__format__blockAlign: usize = 44;
    pub const sound__format__format: usize = 46;
    pub const sound__loadedSize: usize = 48;
    pub const sound__seekTableSize: usize = 52;
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
    pub const SIZE: usize = 256;
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
    pub const clientTrigger: usize = 72;
    pub const clientTrigger__trigger: usize = 72;
    pub const clientTrigger__trigger__count: usize = 72;
    pub const clientTrigger__trigger__models: usize = 80;
    pub const clientTrigger__trigger__hullCount: usize = 88;
    pub const clientTrigger__trigger__hulls: usize = 96;
    pub const clientTrigger__trigger__slabCount: usize = 104;
    pub const clientTrigger__trigger__slabs: usize = 112;
    pub const clientTrigger__numClientTriggerNodes: usize = 120;
    pub const clientTrigger__clientTriggerAabbTree: usize = 128;
    pub const clientTrigger__triggerStringLength: usize = 136;
    pub const clientTrigger__triggerString: usize = 144;
    pub const clientTrigger__visionSetTriggers: usize = 152;
    pub const clientTrigger__triggerType: usize = 160;
    pub const clientTrigger__origins: usize = 168;
    pub const clientTrigger__scriptDelay: usize = 176;
    pub const clientTrigger__audioTriggers: usize = 184;
    pub const clientTrigger__blendLookup: usize = 192;
    pub const clientTrigger__npcTriggers: usize = 200;
    pub const clientTriggerBlend: usize = 208;
    pub const clientTriggerBlend__numClientTriggerBlendNodes: usize = 208;
    pub const clientTriggerBlend__blendNodes: usize = 216;
    pub const spawnList: usize = 224;
    pub const spawnList__spawnsCount: usize = 224;
    pub const spawnList__spawns: usize = 232;
    pub const splineList: usize = 240;
    pub const splineList__splineCount: usize = 240;
    pub const splineList__splines: usize = 248;
}
pub mod MapTriggers {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const count: usize = 0;
    pub const models: usize = 8;
    pub const hullCount: usize = 16;
    pub const hulls: usize = 24;
    pub const slabCount: usize = 32;
    pub const slabs: usize = 40;
}
pub mod Material {
    pub const SIZE: usize = 936;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const info: usize = 0;
    pub const info__name: usize = 0;
    pub const info__gameFlags: usize = 8;
    pub const info__sortKey: usize = 9;
    pub const info__textureAtlasRowCount: usize = 10;
    pub const info__textureAtlasColumnCount: usize = 11;
    pub const info__textureAtlasFrameBlend: usize = 12;
    pub const info__textureAtlasAsArray: usize = 13;
    pub const info__renderFlags: usize = 14;
    pub const info__drawSurf: usize = 16;
    pub const info__drawSurf__fields: usize = 16;
    pub const info__drawSurf__fields__objectId: usize = 16;
    pub const info__drawSurf__fields__reflectionProbeIndex: usize = 18;
    pub const info__drawSurf__fields__hasGfxEntIndex: usize = 19;
    pub const info__drawSurf__fields__customIndex: usize = 19;
    pub const info__drawSurf__fields__materialSortedIndex: usize = 19;
    pub const info__drawSurf__fields__tessellation: usize = 21;
    pub const info__drawSurf__fields__prepass: usize = 21;
    pub const info__drawSurf__fields__useHeroLighting: usize = 22;
    pub const info__drawSurf__fields__sceneLightEnvIndex: usize = 24;
    pub const info__drawSurf__fields__viewModelRender: usize = 26;
    pub const info__drawSurf__fields__surfType: usize = 26;
    pub const info__drawSurf__fields__primarySortKey: usize = 26;
    pub const info__drawSurf__fields__unused: usize = 27;
    pub const info__drawSurf__packed: usize = 16;
    pub const info__drawSurf__packed__p0: usize = 16;
    pub const info__drawSurf__packed__p1: usize = 24;
    pub const info__surfaceTypeBits: usize = 32;
    pub const info__hashIndex: usize = 36;
    pub const stateBitsEntry: usize = 40;
    pub const textureCount: usize = 452;
    pub const constantCount: usize = 453;
    pub const stateBitsCount: usize = 454;
    pub const stateFlags: usize = 455;
    pub const cameraRegion: usize = 456;
    pub const materialType: usize = 457;
    pub const layerCount: usize = 458;
    pub const assetFlags: usize = 459;
    pub const __pad2: usize = 460;
    pub const techniqueSet: usize = 464;
    pub const textureTable: usize = 472;
    pub const constantTable: usize = 480;
    pub const stateBitsTable: usize = 488;
    pub const constantBufferIndex: usize = 496;
    pub const __pad3: usize = 908;
    pub const constantBufferTable: usize = 912;
    pub const constantBufferCount: usize = 920;
    pub const __pad4: usize = 921;
    pub const subMaterials: usize = 928;
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
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const prog: usize = 8;
    pub const prog__ds: usize = 8;
    pub const prog__loadDef: usize = 16;
    pub const prog__loadDef__program: usize = 16;
    pub const prog__loadDef__programSize: usize = 24;
    pub const prog__loadDef__loadForRenderer: usize = 28;
}
pub mod MaterialDomainShaderProgram {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const ds: usize = 0;
    pub const loadDef: usize = 8;
    pub const loadDef__program: usize = 8;
    pub const loadDef__programSize: usize = 16;
    pub const loadDef__loadForRenderer: usize = 20;
}
pub mod MaterialHullShader {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const prog: usize = 8;
    pub const prog__hs: usize = 8;
    pub const prog__loadDef: usize = 16;
    pub const prog__loadDef__program: usize = 16;
    pub const prog__loadDef__programSize: usize = 24;
    pub const prog__loadDef__loadForRenderer: usize = 28;
}
pub mod MaterialHullShaderProgram {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const hs: usize = 0;
    pub const loadDef: usize = 8;
    pub const loadDef__program: usize = 8;
    pub const loadDef__programSize: usize = 16;
    pub const loadDef__loadForRenderer: usize = 20;
}
pub mod MaterialInfo {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const gameFlags: usize = 8;
    pub const sortKey: usize = 9;
    pub const textureAtlasRowCount: usize = 10;
    pub const textureAtlasColumnCount: usize = 11;
    pub const textureAtlasFrameBlend: usize = 12;
    pub const textureAtlasAsArray: usize = 13;
    pub const renderFlags: usize = 14;
    pub const drawSurf: usize = 16;
    pub const drawSurf__fields: usize = 16;
    pub const drawSurf__fields__objectId: usize = 16;
    pub const drawSurf__fields__reflectionProbeIndex: usize = 18;
    pub const drawSurf__fields__hasGfxEntIndex: usize = 19;
    pub const drawSurf__fields__customIndex: usize = 19;
    pub const drawSurf__fields__materialSortedIndex: usize = 19;
    pub const drawSurf__fields__tessellation: usize = 21;
    pub const drawSurf__fields__prepass: usize = 21;
    pub const drawSurf__fields__useHeroLighting: usize = 22;
    pub const drawSurf__fields__sceneLightEnvIndex: usize = 24;
    pub const drawSurf__fields__viewModelRender: usize = 26;
    pub const drawSurf__fields__surfType: usize = 26;
    pub const drawSurf__fields__primarySortKey: usize = 26;
    pub const drawSurf__fields__unused: usize = 27;
    pub const drawSurf__packed: usize = 16;
    pub const drawSurf__packed__p0: usize = 16;
    pub const drawSurf__packed__p1: usize = 24;
    pub const surfaceTypeBits: usize = 32;
    pub const hashIndex: usize = 36;
}
pub mod MaterialMemory {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const material: usize = 0;
    pub const memory: usize = 8;
}
pub mod MaterialPass {
    pub const SIZE: usize = 72;
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
    pub const perPrimConstantBuffer: usize = 51;
    pub const perObjConstantBuffer: usize = 52;
    pub const stableConstantBuffer: usize = 53;
    pub const customBufferFlags: usize = 56;
    pub const customSamplerFlags: usize = 60;
    pub const precompiledIndex: usize = 61;
    pub const stageConfig: usize = 62;
    pub const args: usize = 64;
}
pub mod MaterialPixelShader {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const prog: usize = 8;
    pub const prog__ps: usize = 8;
    pub const prog__loadDef: usize = 16;
    pub const prog__loadDef__program: usize = 16;
    pub const prog__loadDef__programSize: usize = 24;
    pub const prog__loadDef__loadForRenderer: usize = 28;
    pub const prog__loadDef__microCodeCrc: usize = 32;
}
pub mod MaterialPixelShaderProgram {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const ps: usize = 0;
    pub const loadDef: usize = 8;
    pub const loadDef__program: usize = 8;
    pub const loadDef__programSize: usize = 16;
    pub const loadDef__loadForRenderer: usize = 20;
    pub const loadDef__microCodeCrc: usize = 24;
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
    pub const SIZE: usize = 88;
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
    pub const SIZE: usize = 3312;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const flags: usize = 8;
    pub const worldVertFormat: usize = 10;
    pub const preDisplacementOnlyCount: usize = 11;
    pub const techniques: usize = 16;
}
pub mod MaterialTextureDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const nameHash: usize = 0;
    pub const nameStart: usize = 4;
    pub const nameEnd: usize = 5;
    pub const samplerState: usize = 6;
    pub const semantic: usize = 7;
    pub const u: usize = 8;
    pub const u__image: usize = 8;
    pub const u__water: usize = 8;
}
pub mod MaterialTextureDefInfo {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const image: usize = 0;
    pub const water: usize = 0;
}
pub mod MaterialVertexDeclaration {
    pub const SIZE: usize = 2104;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const streamCount: usize = 8;
    pub const hasOptionalSource: usize = 9;
    pub const routing: usize = 16;
    pub const routing__data: usize = 16;
    pub const routing__decl: usize = 112;
}
pub mod MaterialVertexShader {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const prog: usize = 8;
    pub const prog__vs: usize = 8;
    pub const prog__loadDef: usize = 16;
    pub const prog__loadDef__program: usize = 16;
    pub const prog__loadDef__programSize: usize = 24;
    pub const prog__loadDef__loadForRenderer: usize = 28;
    pub const prog__loadDef__microCodeCrc: usize = 32;
}
pub mod MaterialVertexShaderProgram {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const vs: usize = 0;
    pub const loadDef: usize = 8;
    pub const loadDef__program: usize = 8;
    pub const loadDef__programSize: usize = 16;
    pub const loadDef__loadForRenderer: usize = 20;
    pub const loadDef__microCodeCrc: usize = 24;
}
pub mod MaterialVertexStreamRouting {
    pub const SIZE: usize = 2088;
    pub const ALIGN: usize = 8;
    pub const data: usize = 0;
    pub const decl: usize = 96;
}
pub mod MenuEventHandler {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const eventData: usize = 0;
    pub const eventData__unconditionalScript: usize = 0;
    pub const eventData__conditionalScript: usize = 0;
    pub const eventData__elseScript: usize = 0;
    pub const eventData__setLocalVarData: usize = 0;
    pub const eventType: usize = 8;
}
pub mod MenuEventHandlerSet {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const eventHandlerCount: usize = 0;
    pub const eventHandlers: usize = 8;
}
pub mod MenuList {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const menuCount: usize = 8;
    pub const menus: usize = 16;
}
pub mod NetConstStrings {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const stringType: usize = 8;
    pub const sourceType: usize = 12;
    pub const entryCount: usize = 16;
    pub const stringList: usize = 24;
}
pub mod NoteTrackToSoundEntry {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const attachment: usize = 0;
    pub const notetrackSoundMapKeys: usize = 8;
    pub const notetrackSoundMapValues: usize = 16;
}
pub mod OcclusionShape {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const isDefault: usize = 0;
    pub const name: usize = 8;
    pub const shapeKind: usize = 16;
    pub const innerAngle: usize = 20;
    pub const outerAngle: usize = 24;
    pub const occludedVolume: usize = 28;
}
pub mod Operand {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 1;
    pub const internals: usize = 0;
    pub const internals__intVal: usize = 0;
    pub const internals__floatVal: usize = 0;
    pub const internals__stringVal: usize = 0;
    pub const internals__stringVal__string: usize = 0;
    pub const internals__function: usize = 0;
    pub const dataType: usize = 8;
}
pub mod Packed128 {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const p0: usize = 0;
    pub const p1: usize = 8;
}
pub mod PackedTexCoords {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 4;
    pub const packed: usize = 0;
}
pub mod PackedUnitVec {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 4;
    pub const packed: usize = 0;
}
pub mod PathData {
    pub const SIZE: usize = 136;
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
    pub const dynStatesBytes: usize = 120;
    pub const pathDynStates: usize = 128;
}
pub mod PathDynamicNodeGroup {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const parentIndex: usize = 0;
    pub const nodeTreeCount: usize = 4;
    pub const nodeTree: usize = 8;
}
pub mod PathNodeParentUnion {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 4;
    pub const name: usize = 0;
    pub const index: usize = 0;
}
pub mod PhysCollmap {
    pub const SIZE: usize = 88;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const count: usize = 8;
    pub const geoms: usize = 16;
    pub const mass: usize = 24;
    pub const mass__centerOfMass: usize = 24;
    pub const mass__momentsOfInertia: usize = 36;
    pub const mass__productsOfInertia: usize = 48;
    pub const bounds: usize = 60;
    pub const bounds__midPoint: usize = 60;
    pub const bounds__halfSize: usize = 72;
}
pub mod PhysGeomInfo {
    pub const SIZE: usize = 72;
    pub const ALIGN: usize = 8;
    pub const brushWrapper: usize = 0;
    pub const r#type: usize = 8;
    pub const orientation: usize = 12;
    pub const bounds: usize = 48;
    pub const bounds__midPoint: usize = 48;
    pub const bounds__halfSize: usize = 60;
}
pub mod PhysMass {
    pub const SIZE: usize = 36;
    pub const ALIGN: usize = 4;
    pub const centerOfMass: usize = 0;
    pub const momentsOfInertia: usize = 12;
    pub const productsOfInertia: usize = 24;
}
pub mod PhysPreset {
    pub const SIZE: usize = 88;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const r#type: usize = 8;
    pub const mass: usize = 12;
    pub const bounce: usize = 16;
    pub const friction: usize = 20;
    pub const bulletForceScale: usize = 24;
    pub const explosiveForceScale: usize = 28;
    pub const sndAliasPrefix: usize = 32;
    pub const piecesSpreadFraction: usize = 40;
    pub const piecesUpwardVelocity: usize = 44;
    pub const minMomentum: usize = 48;
    pub const maxMomentum: usize = 52;
    pub const minVolume: usize = 56;
    pub const maxVolume: usize = 60;
    pub const minPitch: usize = 64;
    pub const maxPitch: usize = 68;
    pub const volumeType: usize = 72;
    pub const pitchType: usize = 76;
    pub const tempDefaultToCylinder: usize = 80;
    pub const perSurfaceSndAlias: usize = 81;
}
pub mod Picmip {
    pub const SIZE: usize = 2;
    pub const ALIGN: usize = 1;
    pub const platform: usize = 0;
}
pub mod PrimedSound {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const filename: usize = 0;
    pub const filename__isLocalized: usize = 0;
    pub const filename__fileIndex: usize = 2;
    pub const filename__info: usize = 8;
    pub const filename__info__raw: usize = 8;
    pub const filename__info__raw__dir: usize = 8;
    pub const filename__info__raw__name: usize = 16;
    pub const filename__info__packed: usize = 8;
    pub const filename__info__packed__offset: usize = 8;
    pub const filename__info__packed__length: usize = 16;
    pub const loadedPart: usize = 24;
    pub const dataOffset: usize = 32;
    pub const totalSize: usize = 36;
    pub const primedCrc: usize = 40;
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
pub mod ReloadStateTimerEntry {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const attachment: usize = 0;
    pub const reloadAddTime: usize = 4;
    pub const reloadEmptyAddTime: usize = 8;
    pub const reloadStartAddTime: usize = 12;
}
pub mod RiffAudioInfo {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 4;
    pub const sampleRate: usize = 0;
    pub const dataByteCount: usize = 4;
    pub const numSamples: usize = 8;
    pub const avgBytesPerSec: usize = 12;
    pub const channels: usize = 16;
    pub const numBits: usize = 18;
    pub const blockAlign: usize = 20;
    pub const format: usize = 22;
}
pub mod SModelAabbNode {
    pub const SIZE: usize = 28;
    pub const ALIGN: usize = 4;
    pub const bounds: usize = 0;
    pub const bounds__midPoint: usize = 0;
    pub const bounds__halfSize: usize = 12;
    pub const firstChild: usize = 24;
    pub const childCount: usize = 26;
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
pub mod ScriptableAnimationEntry {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const animName: usize = 0;
    pub const runtimeBuf: usize = 8;
}
pub mod ScriptableDef {
    pub const SIZE: usize = 64;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const baseModel: usize = 8;
    pub const baseCollisionBrush: usize = 16;
    pub const destroyedCollisionBrush: usize = 24;
    pub const parts: usize = 32;
    pub const notetracks: usize = 40;
    pub const r#type: usize = 48;
    pub const flags: usize = 52;
    pub const partCount: usize = 53;
    pub const serverInstancePartCount: usize = 54;
    pub const serverControlledPartCount: usize = 55;
    pub const notetrackCount: usize = 56;
    pub const eventStreamSize: usize = 57;
    pub const eventConstantsSize: usize = 58;
}
pub mod ScriptableEventAnimationDef {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const animName: usize = 0;
    pub const r#override: usize = 8;
    pub const stateful: usize = 9;
    pub const animEntryIndex: usize = 10;
    pub const animPlaybackStreamIndex: usize = 11;
    pub const timeOffsetMin: usize = 12;
    pub const timeOffsetMax: usize = 14;
    pub const playbackRateMin: usize = 16;
    pub const playbackRateMax: usize = 18;
    pub const blendTime: usize = 20;
}
pub mod ScriptableEventDataUnion {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const stateChange: usize = 0;
    pub const stateChange__targetIndex: usize = 0;
    pub const stateChange__delayStreamIndex: usize = 1;
    pub const stateChange__delayMin: usize = 2;
    pub const stateChange__delayMax: usize = 4;
    pub const setModel: usize = 0;
    pub const setModel__model: usize = 0;
    pub const playFx: usize = 0;
    pub const playFx__handle: usize = 0;
    pub const playFx__tagName: usize = 8;
    pub const playFx__loopTime: usize = 12;
    pub const playFx__loopTimeStreamIndex: usize = 14;
    pub const playFx__tagUseAngles: usize = 15;
    pub const playSound: usize = 0;
    pub const playSound__alias: usize = 0;
    pub const playSound__looping: usize = 8;
    pub const playAnim: usize = 0;
    pub const playAnim__animName: usize = 0;
    pub const playAnim__override: usize = 8;
    pub const playAnim__stateful: usize = 9;
    pub const playAnim__animEntryIndex: usize = 10;
    pub const playAnim__animPlaybackStreamIndex: usize = 11;
    pub const playAnim__timeOffsetMin: usize = 12;
    pub const playAnim__timeOffsetMax: usize = 14;
    pub const playAnim__playbackRateMin: usize = 16;
    pub const playAnim__playbackRateMax: usize = 18;
    pub const playAnim__blendTime: usize = 20;
    pub const doExplosion: usize = 0;
    pub const doExplosion__forceMin: usize = 0;
    pub const doExplosion__forceMax: usize = 2;
    pub const doExplosion__radius: usize = 4;
    pub const doExplosion__damageMin: usize = 6;
    pub const doExplosion__damageMax: usize = 8;
    pub const doExplosion__aiAvoid: usize = 10;
    pub const healthDrain: usize = 0;
    pub const healthDrain__amount: usize = 0;
    pub const healthDrain__interval: usize = 2;
    pub const healthDrain__badPlaceRadius: usize = 4;
    pub const healthDrain__streamIndex: usize = 6;
    pub const physicsLaunch: usize = 0;
    pub const physicsLaunch__model: usize = 0;
    pub const physicsLaunch__launchDirX: usize = 8;
    pub const physicsLaunch__launchDirY: usize = 9;
    pub const physicsLaunch__launchDirZ: usize = 10;
    pub const physicsLaunch__explForceScale: usize = 12;
    pub const physicsLaunch__bulletForceScale: usize = 14;
    pub const lightSettings: usize = 0;
    pub const lightSettings__color: usize = 0;
    pub const lightSettings__lightIndexConstIndex: usize = 4;
    pub const lightSettings__transStateStreamIndex: usize = 5;
    pub const lightSettings__useColor: usize = 6;
    pub const lightSettings__useStateTransitionTime: usize = 7;
    pub const lightSettings__intensityScaleMin: usize = 8;
    pub const lightSettings__intensityScaleMax: usize = 10;
    pub const lightSettings__radiusScaleMin: usize = 12;
    pub const lightSettings__radiusScaleMax: usize = 14;
    pub const lightSettings__noteworthy: usize = 16;
    pub const lightSettings__transitionTimeMin: usize = 24;
    pub const lightSettings__transitionTimeMax: usize = 26;
    pub const sunlightSettings: usize = 0;
    pub const sunlightSettings__color: usize = 0;
    pub const sunlightSettings__transStateStreamIndex: usize = 4;
    pub const sunlightSettings__flags: usize = 5;
    pub const sunlightSettings__intensityScaleMin: usize = 6;
    pub const sunlightSettings__intensityScaleMax: usize = 8;
    pub const sunlightSettings__pitchMin: usize = 10;
    pub const sunlightSettings__pitchMax: usize = 12;
    pub const sunlightSettings__headingMin: usize = 14;
    pub const sunlightSettings__headingMax: usize = 16;
    pub const sunlightSettings__transitionTimeMin: usize = 18;
    pub const sunlightSettings__transitionTimeMax: usize = 20;
    pub const shake: usize = 0;
    pub const shake__rumbleName: usize = 0;
    pub const shake__duration: usize = 8;
    pub const shake__durationFadeUp: usize = 10;
    pub const shake__durationFadeDown: usize = 12;
    pub const shake__radius: usize = 14;
    pub const shake__exponent: usize = 16;
    pub const shake__scaleEarthquake: usize = 18;
    pub const shake__scalePitch: usize = 20;
    pub const shake__scaleYaw: usize = 21;
    pub const shake__scaleRoll: usize = 22;
    pub const shake__frequencyPitch: usize = 23;
    pub const shake__frequencyYaw: usize = 24;
    pub const shake__frequencyRoll: usize = 25;
    pub const shake__flags: usize = 26;
}
pub mod ScriptableEventDef {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const data: usize = 8;
    pub const data__stateChange: usize = 8;
    pub const data__stateChange__targetIndex: usize = 8;
    pub const data__stateChange__delayStreamIndex: usize = 9;
    pub const data__stateChange__delayMin: usize = 10;
    pub const data__stateChange__delayMax: usize = 12;
    pub const data__setModel: usize = 8;
    pub const data__setModel__model: usize = 8;
    pub const data__playFx: usize = 8;
    pub const data__playFx__handle: usize = 8;
    pub const data__playFx__tagName: usize = 16;
    pub const data__playFx__loopTime: usize = 20;
    pub const data__playFx__loopTimeStreamIndex: usize = 22;
    pub const data__playFx__tagUseAngles: usize = 23;
    pub const data__playSound: usize = 8;
    pub const data__playSound__alias: usize = 8;
    pub const data__playSound__looping: usize = 16;
    pub const data__playAnim: usize = 8;
    pub const data__playAnim__animName: usize = 8;
    pub const data__playAnim__override: usize = 16;
    pub const data__playAnim__stateful: usize = 17;
    pub const data__playAnim__animEntryIndex: usize = 18;
    pub const data__playAnim__animPlaybackStreamIndex: usize = 19;
    pub const data__playAnim__timeOffsetMin: usize = 20;
    pub const data__playAnim__timeOffsetMax: usize = 22;
    pub const data__playAnim__playbackRateMin: usize = 24;
    pub const data__playAnim__playbackRateMax: usize = 26;
    pub const data__playAnim__blendTime: usize = 28;
    pub const data__doExplosion: usize = 8;
    pub const data__doExplosion__forceMin: usize = 8;
    pub const data__doExplosion__forceMax: usize = 10;
    pub const data__doExplosion__radius: usize = 12;
    pub const data__doExplosion__damageMin: usize = 14;
    pub const data__doExplosion__damageMax: usize = 16;
    pub const data__doExplosion__aiAvoid: usize = 18;
    pub const data__healthDrain: usize = 8;
    pub const data__healthDrain__amount: usize = 8;
    pub const data__healthDrain__interval: usize = 10;
    pub const data__healthDrain__badPlaceRadius: usize = 12;
    pub const data__healthDrain__streamIndex: usize = 14;
    pub const data__physicsLaunch: usize = 8;
    pub const data__physicsLaunch__model: usize = 8;
    pub const data__physicsLaunch__launchDirX: usize = 16;
    pub const data__physicsLaunch__launchDirY: usize = 17;
    pub const data__physicsLaunch__launchDirZ: usize = 18;
    pub const data__physicsLaunch__explForceScale: usize = 20;
    pub const data__physicsLaunch__bulletForceScale: usize = 22;
    pub const data__lightSettings: usize = 8;
    pub const data__lightSettings__color: usize = 8;
    pub const data__lightSettings__lightIndexConstIndex: usize = 12;
    pub const data__lightSettings__transStateStreamIndex: usize = 13;
    pub const data__lightSettings__useColor: usize = 14;
    pub const data__lightSettings__useStateTransitionTime: usize = 15;
    pub const data__lightSettings__intensityScaleMin: usize = 16;
    pub const data__lightSettings__intensityScaleMax: usize = 18;
    pub const data__lightSettings__radiusScaleMin: usize = 20;
    pub const data__lightSettings__radiusScaleMax: usize = 22;
    pub const data__lightSettings__noteworthy: usize = 24;
    pub const data__lightSettings__transitionTimeMin: usize = 32;
    pub const data__lightSettings__transitionTimeMax: usize = 34;
    pub const data__sunlightSettings: usize = 8;
    pub const data__sunlightSettings__color: usize = 8;
    pub const data__sunlightSettings__transStateStreamIndex: usize = 12;
    pub const data__sunlightSettings__flags: usize = 13;
    pub const data__sunlightSettings__intensityScaleMin: usize = 14;
    pub const data__sunlightSettings__intensityScaleMax: usize = 16;
    pub const data__sunlightSettings__pitchMin: usize = 18;
    pub const data__sunlightSettings__pitchMax: usize = 20;
    pub const data__sunlightSettings__headingMin: usize = 22;
    pub const data__sunlightSettings__headingMax: usize = 24;
    pub const data__sunlightSettings__transitionTimeMin: usize = 26;
    pub const data__sunlightSettings__transitionTimeMax: usize = 28;
    pub const data__shake: usize = 8;
    pub const data__shake__rumbleName: usize = 8;
    pub const data__shake__duration: usize = 16;
    pub const data__shake__durationFadeUp: usize = 18;
    pub const data__shake__durationFadeDown: usize = 20;
    pub const data__shake__radius: usize = 22;
    pub const data__shake__exponent: usize = 24;
    pub const data__shake__scaleEarthquake: usize = 26;
    pub const data__shake__scalePitch: usize = 28;
    pub const data__shake__scaleYaw: usize = 29;
    pub const data__shake__scaleRoll: usize = 30;
    pub const data__shake__frequencyPitch: usize = 31;
    pub const data__shake__frequencyYaw: usize = 32;
    pub const data__shake__frequencyRoll: usize = 33;
    pub const data__shake__flags: usize = 34;
}
pub mod ScriptableEventExplodeDef {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 2;
    pub const forceMin: usize = 0;
    pub const forceMax: usize = 2;
    pub const radius: usize = 4;
    pub const damageMin: usize = 6;
    pub const damageMax: usize = 8;
    pub const aiAvoid: usize = 10;
}
pub mod ScriptableEventFxDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const handle: usize = 0;
    pub const tagName: usize = 8;
    pub const loopTime: usize = 12;
    pub const loopTimeStreamIndex: usize = 14;
    pub const tagUseAngles: usize = 15;
}
pub mod ScriptableEventHealthDef {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 2;
    pub const amount: usize = 0;
    pub const interval: usize = 2;
    pub const badPlaceRadius: usize = 4;
    pub const streamIndex: usize = 6;
}
pub mod ScriptableEventLightSettingsDef {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const color: usize = 0;
    pub const lightIndexConstIndex: usize = 4;
    pub const transStateStreamIndex: usize = 5;
    pub const useColor: usize = 6;
    pub const useStateTransitionTime: usize = 7;
    pub const intensityScaleMin: usize = 8;
    pub const intensityScaleMax: usize = 10;
    pub const radiusScaleMin: usize = 12;
    pub const radiusScaleMax: usize = 14;
    pub const noteworthy: usize = 16;
    pub const transitionTimeMin: usize = 24;
    pub const transitionTimeMax: usize = 26;
}
pub mod ScriptableEventModelDef {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const model: usize = 0;
}
pub mod ScriptableEventPhysicsDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const model: usize = 0;
    pub const launchDirX: usize = 8;
    pub const launchDirY: usize = 9;
    pub const launchDirZ: usize = 10;
    pub const explForceScale: usize = 12;
    pub const bulletForceScale: usize = 14;
}
pub mod ScriptableEventShakeDef {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const rumbleName: usize = 0;
    pub const duration: usize = 8;
    pub const durationFadeUp: usize = 10;
    pub const durationFadeDown: usize = 12;
    pub const radius: usize = 14;
    pub const exponent: usize = 16;
    pub const scaleEarthquake: usize = 18;
    pub const scalePitch: usize = 20;
    pub const scaleYaw: usize = 21;
    pub const scaleRoll: usize = 22;
    pub const frequencyPitch: usize = 23;
    pub const frequencyYaw: usize = 24;
    pub const frequencyRoll: usize = 25;
    pub const flags: usize = 26;
}
pub mod ScriptableEventSoundDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const alias: usize = 0;
    pub const looping: usize = 8;
}
pub mod ScriptableEventStateChangeDef {
    pub const SIZE: usize = 6;
    pub const ALIGN: usize = 2;
    pub const targetIndex: usize = 0;
    pub const delayStreamIndex: usize = 1;
    pub const delayMin: usize = 2;
    pub const delayMax: usize = 4;
}
pub mod ScriptableEventSunlightSettingsDef {
    pub const SIZE: usize = 22;
    pub const ALIGN: usize = 2;
    pub const color: usize = 0;
    pub const transStateStreamIndex: usize = 4;
    pub const flags: usize = 5;
    pub const intensityScaleMin: usize = 6;
    pub const intensityScaleMax: usize = 8;
    pub const pitchMin: usize = 10;
    pub const pitchMax: usize = 12;
    pub const headingMin: usize = 14;
    pub const headingMax: usize = 16;
    pub const transitionTimeMin: usize = 18;
    pub const transitionTimeMax: usize = 20;
}
pub mod ScriptableInstance {
    pub const SIZE: usize = 120;
    pub const ALIGN: usize = 8;
    pub const def: usize = 0;
    pub const eventConstantsBuf: usize = 8;
    pub const origin: usize = 16;
    pub const angles: usize = 28;
    pub const targetname: usize = 40;
    pub const preBrushModel: usize = 44;
    pub const postBrushModel: usize = 46;
    pub const flags: usize = 48;
    pub const currentModel: usize = 56;
    pub const partStates: usize = 64;
    pub const eventStreamBuf: usize = 72;
    pub const currentPartBits: usize = 80;
    pub const damageOwnerEntHandle: usize = 112;
    pub const updateNextInstance: usize = 116;
    pub const linkedObject: usize = 118;
}
pub mod ScriptableInstancePartState {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 2;
    pub const curHealth: usize = 0;
    pub const lastExecutedStateIndex: usize = 2;
    pub const stateIndex: usize = 3;
}
pub mod ScriptableMapEnts {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const instanceStateSize: usize = 0;
    pub const instanceCount: usize = 4;
    pub const reservedInstanceCount: usize = 8;
    pub const instances: usize = 16;
    pub const animEntryCount: usize = 24;
    pub const animEntries: usize = 32;
    pub const replicatedInstanceCount: usize = 40;
}
pub mod ScriptableNotetrackDataUnion {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const playFx: usize = 0;
    pub const playFx__handle: usize = 0;
    pub const playFx__tagName: usize = 8;
    pub const playFx__useAngles: usize = 12;
    pub const playSound: usize = 0;
    pub const playSound__alias: usize = 0;
}
pub mod ScriptableNotetrackDef {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const r#type: usize = 4;
    pub const data: usize = 8;
    pub const data__playFx: usize = 8;
    pub const data__playFx__handle: usize = 8;
    pub const data__playFx__tagName: usize = 16;
    pub const data__playFx__useAngles: usize = 20;
    pub const data__playSound: usize = 8;
    pub const data__playSound__alias: usize = 8;
}
pub mod ScriptableNotetrackFxDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const handle: usize = 0;
    pub const tagName: usize = 8;
    pub const useAngles: usize = 12;
}
pub mod ScriptableNotetrackSoundDef {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const alias: usize = 0;
}
pub mod ScriptablePartDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const states: usize = 0;
    pub const name: usize = 8;
    pub const stateCount: usize = 12;
    pub const flags: usize = 13;
    pub const eventStreamTimeRemainIndex: usize = 14;
    pub const eventStreamNextChangeTimeIndex: usize = 15;
}
pub mod ScriptableStateDef {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const tagName: usize = 4;
    pub const onEnterEvents: usize = 8;
    pub const onEnterEventCount: usize = 16;
    pub const damageFlags: usize = 17;
    pub const damageParentTransferRate: usize = 18;
    pub const damageParentReceiveRate: usize = 19;
    pub const maxHealth: usize = 20;
}
pub mod SentientHandle {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 2;
    pub const number: usize = 0;
    pub const infoIndex: usize = 2;
}
pub mod SetLocalVarData {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const localVarName: usize = 0;
    pub const expression: usize = 8;
}
pub mod SndCurve {
    pub const SIZE: usize = 152;
    pub const ALIGN: usize = 8;
    pub const isDefault: usize = 0;
    pub const filename: usize = 8;
    pub const name: usize = 8;
    pub const knotCount: usize = 16;
    pub const knots: usize = 20;
}
pub mod SoundAliasFlags {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 4;
    pub const packed: usize = 0;
    pub const packed__looping: usize = 0;
    pub const packed__isMaster: usize = 0;
    pub const packed__isSlave: usize = 0;
    pub const packed__fullDryLevel: usize = 0;
    pub const packed__noWetLevel: usize = 0;
    pub const packed__randomLooping: usize = 0;
    pub const packed__spatialize: usize = 0;
    pub const packed__type: usize = 0;
    pub const packed__channel: usize = 1;
    pub const intValue: usize = 0;
}
pub mod SoundFile {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const exists: usize = 1;
    pub const u: usize = 8;
    pub const u__loadSnd: usize = 8;
    pub const u__streamSnd: usize = 8;
    pub const u__streamSnd__filename: usize = 8;
    pub const u__streamSnd__filename__isLocalized: usize = 8;
    pub const u__streamSnd__filename__fileIndex: usize = 10;
    pub const u__streamSnd__filename__info: usize = 16;
    pub const u__streamSnd__filename__info__raw: usize = 16;
    pub const u__streamSnd__filename__info__raw__dir: usize = 16;
    pub const u__streamSnd__filename__info__raw__name: usize = 24;
    pub const u__streamSnd__filename__info__packed: usize = 16;
    pub const u__streamSnd__filename__info__packed__offset: usize = 16;
    pub const u__streamSnd__filename__info__packed__length: usize = 24;
    pub const u__streamSnd__totalMsec: usize = 32;
    pub const u__primedSnd: usize = 8;
    pub const u__primedSnd__filename: usize = 8;
    pub const u__primedSnd__filename__isLocalized: usize = 8;
    pub const u__primedSnd__filename__fileIndex: usize = 10;
    pub const u__primedSnd__filename__info: usize = 16;
    pub const u__primedSnd__filename__info__raw: usize = 16;
    pub const u__primedSnd__filename__info__raw__dir: usize = 16;
    pub const u__primedSnd__filename__info__raw__name: usize = 24;
    pub const u__primedSnd__filename__info__packed: usize = 16;
    pub const u__primedSnd__filename__info__packed__offset: usize = 16;
    pub const u__primedSnd__filename__info__packed__length: usize = 24;
    pub const u__primedSnd__loadedPart: usize = 32;
    pub const u__primedSnd__dataOffset: usize = 40;
    pub const u__primedSnd__totalSize: usize = 44;
    pub const u__primedSnd__primedCrc: usize = 48;
}
pub mod SoundFileRef {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const loadSnd: usize = 0;
    pub const streamSnd: usize = 0;
    pub const streamSnd__filename: usize = 0;
    pub const streamSnd__filename__isLocalized: usize = 0;
    pub const streamSnd__filename__fileIndex: usize = 2;
    pub const streamSnd__filename__info: usize = 8;
    pub const streamSnd__filename__info__raw: usize = 8;
    pub const streamSnd__filename__info__raw__dir: usize = 8;
    pub const streamSnd__filename__info__raw__name: usize = 16;
    pub const streamSnd__filename__info__packed: usize = 8;
    pub const streamSnd__filename__info__packed__offset: usize = 8;
    pub const streamSnd__filename__info__packed__length: usize = 16;
    pub const streamSnd__totalMsec: usize = 24;
    pub const primedSnd: usize = 0;
    pub const primedSnd__filename: usize = 0;
    pub const primedSnd__filename__isLocalized: usize = 0;
    pub const primedSnd__filename__fileIndex: usize = 2;
    pub const primedSnd__filename__info: usize = 8;
    pub const primedSnd__filename__info__raw: usize = 8;
    pub const primedSnd__filename__info__raw__dir: usize = 8;
    pub const primedSnd__filename__info__raw__name: usize = 16;
    pub const primedSnd__filename__info__packed: usize = 8;
    pub const primedSnd__filename__info__packed__offset: usize = 8;
    pub const primedSnd__filename__info__packed__length: usize = 16;
    pub const primedSnd__loadedPart: usize = 24;
    pub const primedSnd__dataOffset: usize = 32;
    pub const primedSnd__totalSize: usize = 36;
    pub const primedSnd__primedCrc: usize = 40;
}
pub mod SoundOverrideEntry {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const attachment1: usize = 0;
    pub const attachment2: usize = 2;
    pub const overrideSound: usize = 8;
    pub const altmodeSound: usize = 16;
    pub const soundType: usize = 24;
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
pub mod SpeakerLevels {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const speaker: usize = 0;
    pub const numLevels: usize = 4;
    pub const levels: usize = 8;
}
pub mod SpeakerMap {
    pub const SIZE: usize = 416;
    pub const ALIGN: usize = 8;
    pub const isDefault: usize = 0;
    pub const name: usize = 8;
    pub const channelMaps: usize = 16;
}
pub mod SplinePointEntityRecord {
    pub const SIZE: usize = 72;
    pub const ALIGN: usize = 8;
    pub const splineId: usize = 0;
    pub const splineNodeId: usize = 4;
    pub const splineNodeLabel: usize = 8;
    pub const splineNodeTension: usize = 16;
    pub const origin: usize = 20;
    pub const corridorDims: usize = 32;
    pub const tangent: usize = 40;
    pub const distToNextNode: usize = 52;
    pub const positionCubic: usize = 56;
    pub const tangentQuadratic: usize = 64;
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
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const origin: usize = 8;
    pub const triggerIndex: usize = 20;
    pub const sunPrimaryLightIndex: usize = 22;
}
pub mod StateTimers {
    pub const SIZE: usize = 204;
    pub const ALIGN: usize = 4;
    pub const iFireDelay: usize = 0;
    pub const iMeleeDelay: usize = 4;
    pub const meleeChargeDelay: usize = 8;
    pub const iDetonateDelay: usize = 12;
    pub const iRechamberTime: usize = 16;
    pub const rechamberTimeOneHanded: usize = 20;
    pub const iRechamberBoltTime: usize = 24;
    pub const iHoldFireTime: usize = 28;
    pub const iHoldPrimeTime: usize = 32;
    pub const iDetonateTime: usize = 36;
    pub const iMeleeTime: usize = 40;
    pub const meleeChargeTime: usize = 44;
    pub const iReloadTime: usize = 48;
    pub const reloadShowRocketTime: usize = 52;
    pub const iReloadEmptyTime: usize = 56;
    pub const iReloadAddTime: usize = 60;
    pub const iReloadEmptyAddTime: usize = 64;
    pub const iReloadStartTime: usize = 68;
    pub const iReloadStartAddTime: usize = 72;
    pub const iReloadEndTime: usize = 76;
    pub const iDropTime: usize = 80;
    pub const iRaiseTime: usize = 84;
    pub const iAltDropTime: usize = 88;
    pub const quickDropTime: usize = 92;
    pub const quickRaiseTime: usize = 96;
    pub const iBreachRaiseTime: usize = 100;
    pub const iEmptyRaiseTime: usize = 104;
    pub const iEmptyDropTime: usize = 108;
    pub const sprintInTime: usize = 112;
    pub const sprintLoopTime: usize = 116;
    pub const sprintOutTime: usize = 120;
    pub const stunnedTimeBegin: usize = 124;
    pub const stunnedTimeLoop: usize = 128;
    pub const stunnedTimeEnd: usize = 132;
    pub const nightVisionWearTime: usize = 136;
    pub const nightVisionWearTimeFadeOutEnd: usize = 140;
    pub const nightVisionWearTimePowerUp: usize = 144;
    pub const nightVisionRemoveTime: usize = 148;
    pub const nightVisionRemoveTimePowerDown: usize = 152;
    pub const nightVisionRemoveTimeFadeInStart: usize = 156;
    pub const fuseTime: usize = 160;
    pub const aiFuseTime: usize = 164;
    pub const primeTime: usize = 168;
    pub const bHoldFullPrime: usize = 172;
    pub const blastFrontTime: usize = 176;
    pub const blastRightTime: usize = 180;
    pub const blastBackTime: usize = 184;
    pub const blastLeftTime: usize = 188;
    pub const slideInTime: usize = 192;
    pub const slideLoopTime: usize = 196;
    pub const slideOutTime: usize = 200;
}
pub mod Statement_s {
    pub const SIZE: usize = 112;
    pub const ALIGN: usize = 8;
    pub const numEntries: usize = 0;
    pub const entries: usize = 8;
    pub const supportingData: usize = 16;
    pub const persistentState: usize = 24;
    pub const persistentState__flags: usize = 24;
    pub const persistentState__playerDataKey: usize = 28;
    pub const persistentState__lastExecuteTime: usize = 44;
    pub const persistentState__lastResult: usize = 60;
}
pub mod StaticDvar {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const dvar: usize = 0;
    pub const dvarName: usize = 8;
}
pub mod StaticDvarList {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const numStaticDvars: usize = 0;
    pub const staticDvars: usize = 8;
}
pub mod StreamFileInfo {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const raw: usize = 0;
    pub const raw__dir: usize = 0;
    pub const raw__name: usize = 8;
    pub const packed: usize = 0;
    pub const packed__offset: usize = 0;
    pub const packed__length: usize = 8;
}
pub mod StreamFileName {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const isLocalized: usize = 0;
    pub const fileIndex: usize = 2;
    pub const info: usize = 8;
    pub const info__raw: usize = 8;
    pub const info__raw__dir: usize = 8;
    pub const info__raw__name: usize = 16;
    pub const info__packed: usize = 8;
    pub const info__packed__offset: usize = 8;
    pub const info__packed__length: usize = 16;
}
pub mod StreamFileNamePacked {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const offset: usize = 0;
    pub const length: usize = 8;
}
pub mod StreamFileNameRaw {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const dir: usize = 0;
    pub const name: usize = 8;
}
pub mod StreamedSound {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const filename: usize = 0;
    pub const filename__isLocalized: usize = 0;
    pub const filename__fileIndex: usize = 2;
    pub const filename__info: usize = 8;
    pub const filename__info__raw: usize = 8;
    pub const filename__info__raw__dir: usize = 8;
    pub const filename__info__raw__name: usize = 16;
    pub const filename__info__packed: usize = 8;
    pub const filename__info__packed__offset: usize = 8;
    pub const filename__info__packed__length: usize = 16;
    pub const totalMsec: usize = 24;
}
pub mod StringList {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const totalStrings: usize = 0;
    pub const strings: usize = 8;
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
pub mod ToneMapping {
    pub const SIZE: usize = 112;
    pub const ALIGN: usize = 8;
    pub const toeStrength: usize = 0;
    pub const toeAngleNumerator: usize = 12;
    pub const toeAngleDenominator: usize = 24;
    pub const linearStrength: usize = 36;
    pub const linearAngle: usize = 48;
    pub const shoulderStrength: usize = 60;
    pub const whitePoint: usize = 72;
    pub const exposure: usize = 84;
    pub const adaptationRates: usize = 96;
    pub const name: usize = 104;
}
pub mod TracerDef {
    pub const SIZE: usize = 120;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const material: usize = 8;
    pub const drawInterval: usize = 16;
    pub const speed: usize = 20;
    pub const beamLength: usize = 24;
    pub const beamWidth: usize = 28;
    pub const screwRadius: usize = 32;
    pub const screwDist: usize = 36;
    pub const colors: usize = 40;
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
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const contents: usize = 0;
    pub const hullCount: usize = 4;
    pub const firstHull: usize = 6;
}
pub mod TriggerSlab {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 4;
    pub const dir: usize = 0;
    pub const midPoint: usize = 12;
    pub const halfSize: usize = 16;
}
pub mod UIFunctionList {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const totalFunctions: usize = 0;
    pub const functions: usize = 8;
}
pub mod VectorField {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const linearData: usize = 8;
    pub const worldBounds: usize = 16;
    pub const worldBounds__mins: usize = 16;
    pub const worldBounds__maxs: usize = 28;
    pub const ds: usize = 40;
    pub const numEntries: usize = 52;
    pub const flags: usize = 64;
    pub const r#type: usize = 68;
    pub const pad: usize = 72;
}
pub mod VehicleDef {
    pub const SIZE: usize = 1552;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const r#type: usize = 8;
    pub const useHintString: usize = 16;
    pub const health: usize = 24;
    pub const quadBarrel: usize = 28;
    pub const hitClientScriptables: usize = 32;
    pub const texScrollScale: usize = 36;
    pub const topSpeed: usize = 40;
    pub const accel: usize = 44;
    pub const rotRate: usize = 48;
    pub const rotAccel: usize = 52;
    pub const maxBodyPitch: usize = 56;
    pub const maxBodyRoll: usize = 60;
    pub const fakeBodyAccelPitch: usize = 64;
    pub const fakeBodyAccelRoll: usize = 68;
    pub const fakeBodyVelPitch: usize = 72;
    pub const fakeBodyVelRoll: usize = 76;
    pub const fakeBodySideVelPitch: usize = 80;
    pub const fakeBodyPitchStrength: usize = 84;
    pub const fakeBodyRollStrength: usize = 88;
    pub const fakeBodyPitchDampening: usize = 92;
    pub const fakeBodyRollDampening: usize = 96;
    pub const fakeBodyBoatRockingAmplitude: usize = 100;
    pub const fakeBodyBoatRockingPeriod: usize = 104;
    pub const fakeBodyBoatRockingRotationPeriod: usize = 108;
    pub const fakeBodyBoatRockingFadeoutSpeed: usize = 112;
    pub const boatBouncingMinForce: usize = 116;
    pub const boatBouncingMaxForce: usize = 120;
    pub const boatBouncingRate: usize = 124;
    pub const boatBouncingFadeinSpeed: usize = 128;
    pub const boatBouncingFadeoutSteeringAngle: usize = 132;
    pub const collisionDamage: usize = 136;
    pub const collisionSpeed: usize = 140;
    pub const killcamOffset: usize = 144;
    pub const playerProtected: usize = 156;
    pub const bulletDamage: usize = 160;
    pub const armorPiercingDamage: usize = 164;
    pub const grenadeDamage: usize = 168;
    pub const projectileDamage: usize = 172;
    pub const projectileSplashDamage: usize = 176;
    pub const heavyExplosiveDamage: usize = 180;
    pub const vehPhysDef: usize = 184;
    pub const vehPhysDef__physicsEnabled: usize = 184;
    pub const vehPhysDef__physPresetName: usize = 192;
    pub const vehPhysDef__physPreset: usize = 200;
    pub const vehPhysDef__accelGraphName: usize = 208;
    pub const vehPhysDef__steeringAxle: usize = 216;
    pub const vehPhysDef__powerAxle: usize = 220;
    pub const vehPhysDef__brakingAxle: usize = 224;
    pub const vehPhysDef__topSpeed: usize = 228;
    pub const vehPhysDef__reverseSpeed: usize = 232;
    pub const vehPhysDef__maxVelocity: usize = 236;
    pub const vehPhysDef__maxPitch: usize = 240;
    pub const vehPhysDef__maxRoll: usize = 244;
    pub const vehPhysDef__suspensionTravelFront: usize = 248;
    pub const vehPhysDef__suspensionTravelRear: usize = 252;
    pub const vehPhysDef__suspensionStrengthFront: usize = 256;
    pub const vehPhysDef__suspensionDampingFront: usize = 260;
    pub const vehPhysDef__suspensionStrengthRear: usize = 264;
    pub const vehPhysDef__suspensionDampingRear: usize = 268;
    pub const vehPhysDef__frictionBraking: usize = 272;
    pub const vehPhysDef__frictionCoasting: usize = 276;
    pub const vehPhysDef__frictionTopSpeed: usize = 280;
    pub const vehPhysDef__frictionSide: usize = 284;
    pub const vehPhysDef__frictionSideRear: usize = 288;
    pub const vehPhysDef__velocityDependentSlip: usize = 292;
    pub const vehPhysDef__rollStability: usize = 296;
    pub const vehPhysDef__rollResistance: usize = 300;
    pub const vehPhysDef__pitchResistance: usize = 304;
    pub const vehPhysDef__yawResistance: usize = 308;
    pub const vehPhysDef__uprightStrengthPitch: usize = 312;
    pub const vehPhysDef__uprightStrengthRoll: usize = 316;
    pub const vehPhysDef__targetAirPitch: usize = 320;
    pub const vehPhysDef__airYawTorque: usize = 324;
    pub const vehPhysDef__airPitchTorque: usize = 328;
    pub const vehPhysDef__minimumMomentumForCollision: usize = 332;
    pub const vehPhysDef__collisionLaunchForceScale: usize = 336;
    pub const vehPhysDef__wreckedMassScale: usize = 340;
    pub const vehPhysDef__wreckedBodyFriction: usize = 344;
    pub const vehPhysDef__minimumJoltForNotify: usize = 348;
    pub const vehPhysDef__slipThresholdFront: usize = 352;
    pub const vehPhysDef__slipThresholdRear: usize = 356;
    pub const vehPhysDef__slipFricScaleFront: usize = 360;
    pub const vehPhysDef__slipFricScaleRear: usize = 364;
    pub const vehPhysDef__slipFricRateFront: usize = 368;
    pub const vehPhysDef__slipFricRateRear: usize = 372;
    pub const vehPhysDef__slipYawTorque: usize = 376;
    pub const boostDuration: usize = 384;
    pub const boostRechargeTime: usize = 388;
    pub const boostAcceleration: usize = 392;
    pub const suspensionTravel: usize = 396;
    pub const maxSteeringAngle: usize = 400;
    pub const steeringLerp: usize = 404;
    pub const minSteeringScale: usize = 408;
    pub const minSteeringSpeed: usize = 412;
    pub const disableWheelsTurning: usize = 416;
    pub const vehHelicopterIsASplinePlane: usize = 420;
    pub const vehHelicopterLockAltitude: usize = 424;
    pub const vehHelicopterOffsetFromMesh: usize = 428;
    pub const vehHelicopterAltitudeOffset: usize = 432;
    pub const vehHelicopterPitchOffset: usize = 436;
    pub const vehHelicopterBoundsRadius: usize = 440;
    pub const vehHelicopterMaxSpeed: usize = 444;
    pub const vehHelicopterMaxSpeedVertical: usize = 448;
    pub const vehHelicopterMaxAccel: usize = 452;
    pub const vehHelicopterMaxAccelVertical: usize = 456;
    pub const vehHelicopterDecelerationFwd: usize = 460;
    pub const vehHelicopterDecelerationSide: usize = 464;
    pub const vehHelicopterDecelerationUp: usize = 468;
    pub const vehHelicopterMaxYawRate: usize = 472;
    pub const vehHelicopterMaxYawAccel: usize = 476;
    pub const vehHelicopterTiltFromVelocity: usize = 480;
    pub const vehHelicopterTiltFromControllerAxes: usize = 484;
    pub const vehHelicopterTiltFromAcceleration: usize = 488;
    pub const vehHelicopterTiltFromDeceleration: usize = 492;
    pub const vehHelicopterTiltFromFwdAndYaw_VelAtMaxTilt: usize = 496;
    pub const vehHelicopterTiltFromFwdAndYaw: usize = 500;
    pub const vehHelicopterTiltMomentum: usize = 504;
    pub const vehHelicopterTiltSpeed: usize = 508;
    pub const vehHelicopterMaxPitch: usize = 512;
    pub const vehHelicopterMaxRoll: usize = 516;
    pub const vehHelicopterHoverSpeedThreshold: usize = 520;
    pub const vehHelicopterJitterJerkyness: usize = 524;
    pub const vehHelicopterLookaheadTime: usize = 528;
    pub const vehHelicopterSoftCollisions: usize = 532;
    pub const vehHelicopterUseGroundFX: usize = 536;
    pub const vehHelicopterGroundFx: usize = 544;
    pub const vehHelicopterGroundWaterFx: usize = 552;
    pub const vehHelicopterGroundFxDefaultRepeatRate: usize = 560;
    pub const vehHelicopterGroundFxSlowestRepeatRate: usize = 564;
    pub const vehHelicopterGroundFxFastestRepeatRate: usize = 568;
    pub const vehHelicopterGroundFxMinGroundDist: usize = 572;
    pub const vehHelicopterGroundFxMaxGroundDist: usize = 576;
    pub const vehSplinePlaneCorridorMaxXVel: usize = 580;
    pub const vehSplinePlaneCorridorMaxZVel: usize = 584;
    pub const vehSplinePlaneCorridorMaxXAccel: usize = 588;
    pub const vehSplinePlaneCorridorMaxZAccel: usize = 592;
    pub const vehSplinePlaneTangentLookAtRate: usize = 596;
    pub const vehSplinePlaneMaxPitchSpeed: usize = 600;
    pub const vehSplinePlaneMaxYawSpeed: usize = 604;
    pub const vehSplinePlaneMaxRollSpeed: usize = 608;
    pub const vehSplinePlanePitchSpeedRate: usize = 612;
    pub const vehSplinePlaneYawSpeedRate: usize = 616;
    pub const vehSplinePlaneRollSpeedRateUp: usize = 620;
    pub const vehSplinePlaneRollSpeedRateDown: usize = 624;
    pub const vehSplinePlaneMaxPitchAccel: usize = 628;
    pub const vehSplinePlaneMaxYawAccel: usize = 632;
    pub const vehSplinePlaneMaxRollAccel: usize = 636;
    pub const vehSplinePlaneYawToRollFactor: usize = 640;
    pub const vehSplinePlaneRollToYawFactor: usize = 644;
    pub const vehSplinePlaneRollToYawFactorBlend: usize = 648;
    pub const vehSplinePlaneMaxPitch: usize = 652;
    pub const vehSplinePlaneMaxRoll: usize = 656;
    pub const vehSplinePlaneMaxPitchFromRoll: usize = 660;
    pub const vehSplinePlaneMaxRollYawOffset: usize = 664;
    pub const vehSplinePlaneRollYawOffsetThreshold: usize = 668;
    pub const vehSplinePlaneMaxTiltRoll: usize = 672;
    pub const vehSplinePlaneMaxTiltPitch: usize = 676;
    pub const vehSplinePlaneTiltRollRate: usize = 680;
    pub const vehSplinePlaneTiltPitchRate: usize = 684;
    pub const camLookEnabled: usize = 688;
    pub const camRelativeControl: usize = 692;
    pub const camRemoteDrive: usize = 696;
    pub const camLerp: usize = 700;
    pub const camHeight: usize = 704;
    pub const camRadius: usize = 708;
    pub const camPitchInfluence: usize = 712;
    pub const camYawInfluence: usize = 716;
    pub const camRollInfluence: usize = 720;
    pub const camFovIncrease: usize = 724;
    pub const camFovOffset: usize = 728;
    pub const camFovSpeed: usize = 732;
    pub const camReturnSpeed: usize = 736;
    pub const camReturnLerp: usize = 740;
    pub const camVehicleAnglePitchRate: usize = 744;
    pub const camVehicleAngleYawRate: usize = 748;
    pub const camVehicleAngleRollRate: usize = 752;
    pub const vehCam_UseGDT: usize = 756;
    pub const vehCam_anglesPitch: usize = 760;
    pub const vehCam_anglesYaw: usize = 764;
    pub const vehCam_anglesRoll: usize = 768;
    pub const vehCam_offsetX: usize = 772;
    pub const vehCam_offsetY: usize = 776;
    pub const vehCam_offsetZ: usize = 780;
    pub const vehCam_radius: usize = 784;
    pub const vehCam_speedInfluence: usize = 788;
    pub const vehCam_pitchTurnRate: usize = 792;
    pub const vehCam_pitchClamp: usize = 796;
    pub const vehCam_yawTurnRate: usize = 800;
    pub const vehCam_yawClamp: usize = 804;
    pub const vehCam_zOffsetMode: usize = 808;
    pub const vehCam_anglesPitch3P: usize = 812;
    pub const vehCam_anglesYaw3P: usize = 816;
    pub const vehCam_anglesRoll3P: usize = 820;
    pub const vehCam_offsetX3P: usize = 824;
    pub const vehCam_offsetY3P: usize = 828;
    pub const vehCam_offsetZ3P: usize = 832;
    pub const vehCam_radius3P: usize = 836;
    pub const vehCam_speedInfluence3P: usize = 840;
    pub const vehCam_pitchTurnRate3P: usize = 844;
    pub const vehCam_pitchClamp3P: usize = 848;
    pub const vehCam_yawTurnRate3P: usize = 852;
    pub const vehCam_yawClamp3P: usize = 856;
    pub const vehCam_zOffsetMode3P: usize = 860;
    pub const turretWeaponName: usize = 864;
    pub const turretWeapon: usize = 872;
    pub const turretHorizSpanLeft: usize = 880;
    pub const turretHorizSpanRight: usize = 884;
    pub const turretVertSpanUp: usize = 888;
    pub const turretVertSpanDown: usize = 892;
    pub const turretHorizResistLeft: usize = 896;
    pub const turretHorizResistRight: usize = 900;
    pub const turretVertResistUp: usize = 904;
    pub const turretVertResistDown: usize = 908;
    pub const turretRotRate: usize = 912;
    pub const turretFireType: usize = 916;
    pub const turretSpinSnd: usize = 920;
    pub const turretStopSnd: usize = 928;
    pub const trophyEnabled: usize = 936;
    pub const trophyRadius: usize = 940;
    pub const trophyInactiveRadius: usize = 944;
    pub const trophyAmmoCount: usize = 948;
    pub const trophyReloadTime: usize = 952;
    pub const trophyTags: usize = 956;
    pub const trophyExplodeFx: usize = 976;
    pub const trophyFlashFx: usize = 984;
    pub const compassFriendlyIcon: usize = 992;
    pub const compassEnemyIcon: usize = 1000;
    pub const compassFriendlyAltIcon: usize = 1008;
    pub const compassEnemyAltIcon: usize = 1016;
    pub const compassIconWidth: usize = 1024;
    pub const compassIconHeight: usize = 1028;
    pub const idleLowSnd: usize = 1032;
    pub const idleHighSnd: usize = 1040;
    pub const engineLowSnd: usize = 1048;
    pub const engineHighSnd: usize = 1056;
    pub const engineSndSpeed: usize = 1064;
    pub const audioOriginTag: usize = 1068;
    pub const idleLowSndAlt: usize = 1072;
    pub const idleHighSndAlt: usize = 1080;
    pub const engineLowSndAlt: usize = 1088;
    pub const engineHighSndAlt: usize = 1096;
    pub const engineSndSpeedAlt: usize = 1104;
    pub const audioOriginTagAlt: usize = 1108;
    pub const turretSpinSndAlt: usize = 1112;
    pub const turretStopSndAlt: usize = 1120;
    pub const engineStartUpSnd: usize = 1128;
    pub const engineStartUpLength: usize = 1136;
    pub const engineShutdownSnd: usize = 1144;
    pub const engineIdleSnd: usize = 1152;
    pub const engineSustainSnd: usize = 1160;
    pub const engineRampUpSnd: usize = 1168;
    pub const engineRampUpLength: usize = 1176;
    pub const engineRampDownSnd: usize = 1184;
    pub const engineRampDownLength: usize = 1192;
    pub const suspensionSoftSnd: usize = 1200;
    pub const suspensionSoftCompression: usize = 1208;
    pub const suspensionHardSnd: usize = 1216;
    pub const suspensionHardCompression: usize = 1224;
    pub const collisionSnd: usize = 1232;
    pub const collisionBlendSpeed: usize = 1240;
    pub const speedSnd: usize = 1248;
    pub const speedSndBlendSpeed: usize = 1256;
    pub const surfaceSndPrefix: usize = 1264;
    pub const surfaceSnds: usize = 1272;
    pub const surfaceSndBlendSpeed: usize = 1520;
    pub const slideVolume: usize = 1524;
    pub const slideBlendSpeed: usize = 1528;
    pub const inAirPitch: usize = 1532;
    pub const soundTriggerOverrideZone: usize = 1536;
    pub const soundTriggerOverrideReverb: usize = 1544;
    pub const soundTriggerOverrideMix: usize = 1545;
    pub const soundTriggerOverrideFilter: usize = 1546;
    pub const soundTriggerOverrideOcclusion: usize = 1547;
    pub const soundTriggerOverrideAmbient: usize = 1548;
    pub const soundTriggerOverrideAmbientEvents: usize = 1549;
    pub const soundTriggerOverrideADSR: usize = 1550;
}
pub mod VehiclePhysDef {
    pub const SIZE: usize = 200;
    pub const ALIGN: usize = 8;
    pub const physicsEnabled: usize = 0;
    pub const physPresetName: usize = 8;
    pub const physPreset: usize = 16;
    pub const accelGraphName: usize = 24;
    pub const steeringAxle: usize = 32;
    pub const powerAxle: usize = 36;
    pub const brakingAxle: usize = 40;
    pub const topSpeed: usize = 44;
    pub const reverseSpeed: usize = 48;
    pub const maxVelocity: usize = 52;
    pub const maxPitch: usize = 56;
    pub const maxRoll: usize = 60;
    pub const suspensionTravelFront: usize = 64;
    pub const suspensionTravelRear: usize = 68;
    pub const suspensionStrengthFront: usize = 72;
    pub const suspensionDampingFront: usize = 76;
    pub const suspensionStrengthRear: usize = 80;
    pub const suspensionDampingRear: usize = 84;
    pub const frictionBraking: usize = 88;
    pub const frictionCoasting: usize = 92;
    pub const frictionTopSpeed: usize = 96;
    pub const frictionSide: usize = 100;
    pub const frictionSideRear: usize = 104;
    pub const velocityDependentSlip: usize = 108;
    pub const rollStability: usize = 112;
    pub const rollResistance: usize = 116;
    pub const pitchResistance: usize = 120;
    pub const yawResistance: usize = 124;
    pub const uprightStrengthPitch: usize = 128;
    pub const uprightStrengthRoll: usize = 132;
    pub const targetAirPitch: usize = 136;
    pub const airYawTorque: usize = 140;
    pub const airPitchTorque: usize = 144;
    pub const minimumMomentumForCollision: usize = 148;
    pub const collisionLaunchForceScale: usize = 152;
    pub const wreckedMassScale: usize = 156;
    pub const wreckedBodyFriction: usize = 160;
    pub const minimumJoltForNotify: usize = 164;
    pub const slipThresholdFront: usize = 168;
    pub const slipThresholdRear: usize = 172;
    pub const slipFricScaleFront: usize = 176;
    pub const slipFricScaleRear: usize = 180;
    pub const slipFricRateFront: usize = 184;
    pub const slipFricRateRear: usize = 188;
    pub const slipYawTorque: usize = 192;
}
pub mod WaterWritable {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 4;
    pub const floatTime: usize = 0;
}
pub mod WeaponAttachment {
    pub const SIZE: usize = 280;
    pub const ALIGN: usize = 8;
    pub const szInternalName: usize = 0;
    pub const name: usize = 0;
    pub const szDisplayName: usize = 8;
    pub const r#type: usize = 16;
    pub const weaponType: usize = 20;
    pub const weapClass: usize = 24;
    pub const worldModels: usize = 32;
    pub const viewModels: usize = 40;
    pub const reticleViewModels: usize = 48;
    pub const ammogeneral: usize = 56;
    pub const sight: usize = 64;
    pub const reload: usize = 72;
    pub const addOns: usize = 80;
    pub const general: usize = 88;
    pub const aimAssist: usize = 96;
    pub const ammunition: usize = 104;
    pub const damage: usize = 112;
    pub const locationDamage: usize = 120;
    pub const idleSettings: usize = 128;
    pub const adsSettings: usize = 136;
    pub const adsSettingsMain: usize = 144;
    pub const scopeDriftSettings: usize = 152;
    pub const scopeDriftSettingsMain: usize = 160;
    pub const hipSpread: usize = 168;
    pub const gunKick: usize = 176;
    pub const viewKick: usize = 184;
    pub const adsOverlay: usize = 192;
    pub const ui: usize = 200;
    pub const rumbles: usize = 208;
    pub const projectile: usize = 216;
    pub const ammunitionScale: usize = 224;
    pub const damageScale: usize = 228;
    pub const damageScaleMin: usize = 232;
    pub const stateTimersScale: usize = 236;
    pub const fireTimersScale: usize = 240;
    pub const idleSettingsScale: usize = 244;
    pub const adsSettingsScale: usize = 248;
    pub const adsSettingsScaleMain: usize = 252;
    pub const hipSpreadScale: usize = 256;
    pub const gunKickScale: usize = 260;
    pub const viewKickScale: usize = 264;
    pub const viewCenterScale: usize = 268;
    pub const loadIndex: usize = 272;
    pub const hideIronSightsWithThisAttachment: usize = 276;
    pub const shareAmmoWithAlt: usize = 277;
}
pub mod WeaponCompleteDef {
    pub const SIZE: usize = 296;
    pub const ALIGN: usize = 8;
    pub const szInternalName: usize = 0;
    pub const name: usize = 0;
    pub const weapDef: usize = 8;
    pub const szDisplayName: usize = 16;
    pub const hideTags: usize = 24;
    pub const scopes: usize = 32;
    pub const underBarrels: usize = 40;
    pub const others: usize = 48;
    pub const szXAnims: usize = 56;
    pub const numAnimOverrides: usize = 64;
    pub const animOverrides: usize = 72;
    pub const numSoundOverrides: usize = 80;
    pub const soundOverrides: usize = 88;
    pub const numFXOverrides: usize = 96;
    pub const fxOverrides: usize = 104;
    pub const numReloadStateTimerOverrides: usize = 112;
    pub const reloadOverrides: usize = 120;
    pub const numNotetrackOverrides: usize = 128;
    pub const notetrackOverrides: usize = 136;
    pub const fAdsZoomFov: usize = 144;
    pub const iAdsTransInTime: usize = 148;
    pub const iAdsTransOutTime: usize = 152;
    pub const iClipSize: usize = 156;
    pub const impactType: usize = 160;
    pub const iFireTime: usize = 164;
    pub const iFireTimeAkimbo: usize = 168;
    pub const dpadIconRatio: usize = 172;
    pub const penetrateMultiplier: usize = 176;
    pub const fAdsViewKickCenterSpeed: usize = 180;
    pub const fHipViewKickCenterSpeed: usize = 184;
    pub const szAltWeaponName: usize = 192;
    pub const altWeapon: usize = 200;
    pub const iAltRaiseTime: usize = 204;
    pub const iAltRaiseTimeAkimbo: usize = 208;
    pub const killIcon: usize = 216;
    pub const dpadIcon: usize = 224;
    pub const fireAnimLength: usize = 232;
    pub const fireAnimLengthAkimbo: usize = 236;
    pub const iFirstRaiseTime: usize = 240;
    pub const iFirstRaiseTimeAkimbo: usize = 244;
    pub const ammoDropStockMax: usize = 248;
    pub const adsDofStart: usize = 252;
    pub const adsDofEnd: usize = 256;
    pub const accuracyGraphKnotCount: usize = 260;
    pub const accuracyGraphKnots: usize = 264;
    pub const motionTracker: usize = 280;
    pub const enhanced: usize = 281;
    pub const dpadIconShowsAmmo: usize = 282;
    pub const szAdsrBaseSetting: usize = 288;
}
pub mod WeaponDef {
    pub const SIZE: usize = 2712;
    pub const ALIGN: usize = 8;
    pub const szOverlayName: usize = 0;
    pub const gunXModel: usize = 8;
    pub const handXModel: usize = 16;
    pub const camoWorldModel: usize = 24;
    pub const camoViewModel: usize = 32;
    pub const camoWorldModelMaterialOverride: usize = 40;
    pub const camoViewModelMaterialOverride: usize = 48;
    pub const camoMaterialTarget: usize = 56;
    pub const szXAnimsRightHanded: usize = 64;
    pub const szXAnimsLeftHanded: usize = 72;
    pub const szModeName: usize = 80;
    pub const notetrackSoundMapKeys: usize = 88;
    pub const notetrackSoundMapValues: usize = 96;
    pub const notetrackRumbleMapKeys: usize = 104;
    pub const notetrackRumbleMapValues: usize = 112;
    pub const notetrackFXMapKeys: usize = 120;
    pub const notetrackFXMapValues: usize = 128;
    pub const notetrackFXMapTagValues: usize = 136;
    pub const playerAnimType: usize = 144;
    pub const weapType: usize = 148;
    pub const weapClass: usize = 152;
    pub const penetrateType: usize = 156;
    pub const inventoryType: usize = 160;
    pub const fireType: usize = 164;
    pub const burstFireCooldown: usize = 168;
    pub const offhandClass: usize = 172;
    pub const stance: usize = 176;
    pub const rattleSoundType: usize = 180;
    pub const viewFlashEffect: usize = 184;
    pub const worldFlashEffect: usize = 192;
    pub const viewFlashADSEffect: usize = 200;
    pub const pickupSound: usize = 208;
    pub const pickupSoundPlayer: usize = 216;
    pub const ammoPickupSound: usize = 224;
    pub const ammoPickupSoundPlayer: usize = 232;
    pub const projectileSound: usize = 240;
    pub const pullbackSound: usize = 248;
    pub const pullbackSoundPlayer: usize = 256;
    pub const fireSound: usize = 264;
    pub const fireSoundPlayer: usize = 272;
    pub const fireSoundPlayerAkimbo: usize = 280;
    pub const fireLoopSound: usize = 288;
    pub const fireLoopSoundPlayer: usize = 296;
    pub const fireStopSound: usize = 304;
    pub const fireStopSoundPlayer: usize = 312;
    pub const fireLastSound: usize = 320;
    pub const fireLastSoundPlayer: usize = 328;
    pub const emptyFireSound: usize = 336;
    pub const emptyFireSoundPlayer: usize = 344;
    pub const meleeSwipeSound: usize = 352;
    pub const meleeSwipeSoundPlayer: usize = 360;
    pub const meleeHitSound: usize = 368;
    pub const meleeMissSound: usize = 376;
    pub const rechamberSound: usize = 384;
    pub const rechamberSoundPlayer: usize = 392;
    pub const reloadSound: usize = 400;
    pub const reloadSoundPlayer: usize = 408;
    pub const reloadEmptySound: usize = 416;
    pub const reloadEmptySoundPlayer: usize = 424;
    pub const reloadStartSound: usize = 432;
    pub const reloadStartSoundPlayer: usize = 440;
    pub const reloadEndSound: usize = 448;
    pub const reloadEndSoundPlayer: usize = 456;
    pub const detonateSound: usize = 464;
    pub const detonateSoundPlayer: usize = 472;
    pub const nightVisionWearSound: usize = 480;
    pub const nightVisionWearSoundPlayer: usize = 488;
    pub const nightVisionRemoveSound: usize = 496;
    pub const nightVisionRemoveSoundPlayer: usize = 504;
    pub const altSwitchSound: usize = 512;
    pub const altSwitchSoundPlayer: usize = 520;
    pub const raiseSound: usize = 528;
    pub const raiseSoundPlayer: usize = 536;
    pub const firstRaiseSound: usize = 544;
    pub const firstRaiseSoundPlayer: usize = 552;
    pub const putawaySound: usize = 560;
    pub const putawaySoundPlayer: usize = 568;
    pub const scanSound: usize = 576;
    pub const changeVariableZoomSound: usize = 584;
    pub const bounceSound: usize = 592;
    pub const rollingSound: usize = 600;
    pub const viewShellEjectEffect: usize = 608;
    pub const worldShellEjectEffect: usize = 616;
    pub const viewLastShotEjectEffect: usize = 624;
    pub const worldLastShotEjectEffect: usize = 632;
    pub const viewMagEjectEffect: usize = 640;
    pub const reticleCenter: usize = 648;
    pub const reticleSide: usize = 656;
    pub const iReticleCenterSize: usize = 664;
    pub const iReticleSideSize: usize = 668;
    pub const iReticleMinOfs: usize = 672;
    pub const activeReticleType: usize = 676;
    pub const vStandMove: usize = 680;
    pub const vStandRot: usize = 692;
    pub const strafeMove: usize = 704;
    pub const strafeRot: usize = 716;
    pub const vDuckedOfs: usize = 728;
    pub const vDuckedMove: usize = 740;
    pub const vDuckedRot: usize = 752;
    pub const vProneOfs: usize = 764;
    pub const vProneMove: usize = 776;
    pub const vProneRot: usize = 788;
    pub const fPosMoveRate: usize = 800;
    pub const fPosProneMoveRate: usize = 804;
    pub const fStandMoveMinSpeed: usize = 808;
    pub const fDuckedMoveMinSpeed: usize = 812;
    pub const fProneMoveMinSpeed: usize = 816;
    pub const fPosRotRate: usize = 820;
    pub const fPosProneRotRate: usize = 824;
    pub const fStandRotMinSpeed: usize = 828;
    pub const fDuckedRotMinSpeed: usize = 832;
    pub const fProneRotMinSpeed: usize = 836;
    pub const worldModel: usize = 840;
    pub const worldClipModel: usize = 848;
    pub const rocketModel: usize = 856;
    pub const knifeModel: usize = 864;
    pub const worldKnifeModel: usize = 872;
    pub const hudIcon: usize = 880;
    pub const hudIconRatio: usize = 888;
    pub const pickupIcon: usize = 896;
    pub const pickupIconRatio: usize = 904;
    pub const ammoCounterIcon: usize = 912;
    pub const ammoCounterIconRatio: usize = 920;
    pub const ammoCounterClip: usize = 924;
    pub const iStartAmmo: usize = 928;
    pub const szAmmoName: usize = 936;
    pub const iAmmoIndex: usize = 944;
    pub const szClipName: usize = 952;
    pub const iClipIndex: usize = 960;
    pub const iMaxAmmo: usize = 964;
    pub const shotCount: usize = 968;
    pub const szSharedAmmoCapName: usize = 976;
    pub const iSharedAmmoCapIndex: usize = 984;
    pub const iSharedAmmoCap: usize = 988;
    pub const damage: usize = 992;
    pub const playerDamage: usize = 996;
    pub const iMeleeDamage: usize = 1000;
    pub const iDamageType: usize = 1004;
    pub const stateTimers: usize = 1008;
    pub const stateTimers__iFireDelay: usize = 1008;
    pub const stateTimers__iMeleeDelay: usize = 1012;
    pub const stateTimers__meleeChargeDelay: usize = 1016;
    pub const stateTimers__iDetonateDelay: usize = 1020;
    pub const stateTimers__iRechamberTime: usize = 1024;
    pub const stateTimers__rechamberTimeOneHanded: usize = 1028;
    pub const stateTimers__iRechamberBoltTime: usize = 1032;
    pub const stateTimers__iHoldFireTime: usize = 1036;
    pub const stateTimers__iHoldPrimeTime: usize = 1040;
    pub const stateTimers__iDetonateTime: usize = 1044;
    pub const stateTimers__iMeleeTime: usize = 1048;
    pub const stateTimers__meleeChargeTime: usize = 1052;
    pub const stateTimers__iReloadTime: usize = 1056;
    pub const stateTimers__reloadShowRocketTime: usize = 1060;
    pub const stateTimers__iReloadEmptyTime: usize = 1064;
    pub const stateTimers__iReloadAddTime: usize = 1068;
    pub const stateTimers__iReloadEmptyAddTime: usize = 1072;
    pub const stateTimers__iReloadStartTime: usize = 1076;
    pub const stateTimers__iReloadStartAddTime: usize = 1080;
    pub const stateTimers__iReloadEndTime: usize = 1084;
    pub const stateTimers__iDropTime: usize = 1088;
    pub const stateTimers__iRaiseTime: usize = 1092;
    pub const stateTimers__iAltDropTime: usize = 1096;
    pub const stateTimers__quickDropTime: usize = 1100;
    pub const stateTimers__quickRaiseTime: usize = 1104;
    pub const stateTimers__iBreachRaiseTime: usize = 1108;
    pub const stateTimers__iEmptyRaiseTime: usize = 1112;
    pub const stateTimers__iEmptyDropTime: usize = 1116;
    pub const stateTimers__sprintInTime: usize = 1120;
    pub const stateTimers__sprintLoopTime: usize = 1124;
    pub const stateTimers__sprintOutTime: usize = 1128;
    pub const stateTimers__stunnedTimeBegin: usize = 1132;
    pub const stateTimers__stunnedTimeLoop: usize = 1136;
    pub const stateTimers__stunnedTimeEnd: usize = 1140;
    pub const stateTimers__nightVisionWearTime: usize = 1144;
    pub const stateTimers__nightVisionWearTimeFadeOutEnd: usize = 1148;
    pub const stateTimers__nightVisionWearTimePowerUp: usize = 1152;
    pub const stateTimers__nightVisionRemoveTime: usize = 1156;
    pub const stateTimers__nightVisionRemoveTimePowerDown: usize = 1160;
    pub const stateTimers__nightVisionRemoveTimeFadeInStart: usize = 1164;
    pub const stateTimers__fuseTime: usize = 1168;
    pub const stateTimers__aiFuseTime: usize = 1172;
    pub const stateTimers__primeTime: usize = 1176;
    pub const stateTimers__bHoldFullPrime: usize = 1180;
    pub const stateTimers__blastFrontTime: usize = 1184;
    pub const stateTimers__blastRightTime: usize = 1188;
    pub const stateTimers__blastBackTime: usize = 1192;
    pub const stateTimers__blastLeftTime: usize = 1196;
    pub const stateTimers__slideInTime: usize = 1200;
    pub const stateTimers__slideLoopTime: usize = 1204;
    pub const stateTimers__slideOutTime: usize = 1208;
    pub const akimboStateTimers: usize = 1212;
    pub const akimboStateTimers__iFireDelay: usize = 1212;
    pub const akimboStateTimers__iMeleeDelay: usize = 1216;
    pub const akimboStateTimers__meleeChargeDelay: usize = 1220;
    pub const akimboStateTimers__iDetonateDelay: usize = 1224;
    pub const akimboStateTimers__iRechamberTime: usize = 1228;
    pub const akimboStateTimers__rechamberTimeOneHanded: usize = 1232;
    pub const akimboStateTimers__iRechamberBoltTime: usize = 1236;
    pub const akimboStateTimers__iHoldFireTime: usize = 1240;
    pub const akimboStateTimers__iHoldPrimeTime: usize = 1244;
    pub const akimboStateTimers__iDetonateTime: usize = 1248;
    pub const akimboStateTimers__iMeleeTime: usize = 1252;
    pub const akimboStateTimers__meleeChargeTime: usize = 1256;
    pub const akimboStateTimers__iReloadTime: usize = 1260;
    pub const akimboStateTimers__reloadShowRocketTime: usize = 1264;
    pub const akimboStateTimers__iReloadEmptyTime: usize = 1268;
    pub const akimboStateTimers__iReloadAddTime: usize = 1272;
    pub const akimboStateTimers__iReloadEmptyAddTime: usize = 1276;
    pub const akimboStateTimers__iReloadStartTime: usize = 1280;
    pub const akimboStateTimers__iReloadStartAddTime: usize = 1284;
    pub const akimboStateTimers__iReloadEndTime: usize = 1288;
    pub const akimboStateTimers__iDropTime: usize = 1292;
    pub const akimboStateTimers__iRaiseTime: usize = 1296;
    pub const akimboStateTimers__iAltDropTime: usize = 1300;
    pub const akimboStateTimers__quickDropTime: usize = 1304;
    pub const akimboStateTimers__quickRaiseTime: usize = 1308;
    pub const akimboStateTimers__iBreachRaiseTime: usize = 1312;
    pub const akimboStateTimers__iEmptyRaiseTime: usize = 1316;
    pub const akimboStateTimers__iEmptyDropTime: usize = 1320;
    pub const akimboStateTimers__sprintInTime: usize = 1324;
    pub const akimboStateTimers__sprintLoopTime: usize = 1328;
    pub const akimboStateTimers__sprintOutTime: usize = 1332;
    pub const akimboStateTimers__stunnedTimeBegin: usize = 1336;
    pub const akimboStateTimers__stunnedTimeLoop: usize = 1340;
    pub const akimboStateTimers__stunnedTimeEnd: usize = 1344;
    pub const akimboStateTimers__nightVisionWearTime: usize = 1348;
    pub const akimboStateTimers__nightVisionWearTimeFadeOutEnd: usize = 1352;
    pub const akimboStateTimers__nightVisionWearTimePowerUp: usize = 1356;
    pub const akimboStateTimers__nightVisionRemoveTime: usize = 1360;
    pub const akimboStateTimers__nightVisionRemoveTimePowerDown: usize = 1364;
    pub const akimboStateTimers__nightVisionRemoveTimeFadeInStart: usize = 1368;
    pub const akimboStateTimers__fuseTime: usize = 1372;
    pub const akimboStateTimers__aiFuseTime: usize = 1376;
    pub const akimboStateTimers__primeTime: usize = 1380;
    pub const akimboStateTimers__bHoldFullPrime: usize = 1384;
    pub const akimboStateTimers__blastFrontTime: usize = 1388;
    pub const akimboStateTimers__blastRightTime: usize = 1392;
    pub const akimboStateTimers__blastBackTime: usize = 1396;
    pub const akimboStateTimers__blastLeftTime: usize = 1400;
    pub const akimboStateTimers__slideInTime: usize = 1404;
    pub const akimboStateTimers__slideLoopTime: usize = 1408;
    pub const akimboStateTimers__slideOutTime: usize = 1412;
    pub const autoAimRange: usize = 1416;
    pub const aimAssistRange: usize = 1420;
    pub const aimAssistRangeAds: usize = 1424;
    pub const aimPadding: usize = 1428;
    pub const enemyCrosshairRange: usize = 1432;
    pub const moveSpeedScale: usize = 1436;
    pub const adsMoveSpeedScale: usize = 1440;
    pub const sprintDurationScale: usize = 1444;
    pub const fAdsZoomInFrac: usize = 1448;
    pub const fAdsZoomOutFrac: usize = 1452;
    pub const overlay: usize = 1456;
    pub const overlay__shader: usize = 1456;
    pub const overlay__shaderLowRes: usize = 1464;
    pub const overlay__shaderEMP: usize = 1472;
    pub const overlay__shaderEMPLowRes: usize = 1480;
    pub const overlay__reticle: usize = 1488;
    pub const overlay__width: usize = 1492;
    pub const overlay__height: usize = 1496;
    pub const overlay__widthSplitscreen: usize = 1500;
    pub const overlay__heightSplitscreen: usize = 1504;
    pub const overlayInterface: usize = 1512;
    pub const fAdsBobFactor: usize = 1516;
    pub const fAdsViewBobMult: usize = 1520;
    pub const fHipSpreadStandMin: usize = 1524;
    pub const fHipSpreadDuckedMin: usize = 1528;
    pub const fHipSpreadProneMin: usize = 1532;
    pub const hipSpreadStandMax: usize = 1536;
    pub const hipSpreadDuckedMax: usize = 1540;
    pub const hipSpreadProneMax: usize = 1544;
    pub const fHipSpreadDecayRate: usize = 1548;
    pub const fHipSpreadFireAdd: usize = 1552;
    pub const fHipSpreadTurnAdd: usize = 1556;
    pub const fHipSpreadMoveAdd: usize = 1560;
    pub const fHipSpreadDuckedDecay: usize = 1564;
    pub const fHipSpreadProneDecay: usize = 1568;
    pub const fHipReticleSidePos: usize = 1572;
    pub const fAdsIdleAmount: usize = 1576;
    pub const fHipIdleAmount: usize = 1580;
    pub const adsIdleSpeed: usize = 1584;
    pub const hipIdleSpeed: usize = 1588;
    pub const fIdleCrouchFactor: usize = 1592;
    pub const fIdleProneFactor: usize = 1596;
    pub const fGunMaxPitch: usize = 1600;
    pub const fGunMaxYaw: usize = 1604;
    pub const adsIdleLerpStartTime: usize = 1608;
    pub const adsIdleLerpTime: usize = 1612;
    pub const swayMaxAngleSteadyAim: usize = 1616;
    pub const swayMaxAngle: usize = 1620;
    pub const swayLerpSpeed: usize = 1624;
    pub const swayPitchScale: usize = 1628;
    pub const swayYawScale: usize = 1632;
    pub const swayHorizScale: usize = 1636;
    pub const swayVertScale: usize = 1640;
    pub const swayShellShockScale: usize = 1644;
    pub const adsSwayMaxAngle: usize = 1648;
    pub const adsSwayLerpSpeed: usize = 1652;
    pub const adsSwayPitchScale: usize = 1656;
    pub const adsSwayYawScale: usize = 1660;
    pub const adsSwayHorizScale: usize = 1664;
    pub const adsSwayVertScale: usize = 1668;
    pub const adsFireRateScale: usize = 1672;
    pub const adsDamageRangeScale: usize = 1676;
    pub const adsFireAnimFrac: usize = 1680;
    pub const physCollmap: usize = 1688;
    pub const dualWieldViewModelOffset: usize = 1696;
    pub const fScopeDriftDelay: usize = 1700;
    pub const fScopeDriftLerpInTime: usize = 1704;
    pub const fScopeDriftSteadyTime: usize = 1708;
    pub const fScopeDriftLerpOutTime: usize = 1712;
    pub const fScopeDriftSteadyFactor: usize = 1716;
    pub const fScopeDriftUnsteadyFactor: usize = 1720;
    pub const killIconRatio: usize = 1724;
    pub const iReloadAmmoAdd: usize = 1728;
    pub const iReloadStartAdd: usize = 1732;
    pub const ammoDropStockMin: usize = 1736;
    pub const ammoDropClipPercentMin: usize = 1740;
    pub const ammoDropClipPercentMax: usize = 1744;
    pub const iExplosionRadius: usize = 1748;
    pub const iExplosionRadiusMin: usize = 1752;
    pub const iExplosionInnerDamage: usize = 1756;
    pub const iExplosionOuterDamage: usize = 1760;
    pub const damageConeAngle: usize = 1764;
    pub const bulletExplDmgMult: usize = 1768;
    pub const bulletExplRadiusMult: usize = 1772;
    pub const iProjectileSpeed: usize = 1776;
    pub const iProjectileSpeedUp: usize = 1780;
    pub const iProjectileSpeedForward: usize = 1784;
    pub const iProjectileActivateDist: usize = 1788;
    pub const projLifetime: usize = 1792;
    pub const timeToAccelerate: usize = 1796;
    pub const projectileCurvature: usize = 1800;
    pub const projectileName: usize = 1808;
    pub const projectileModel: usize = 1816;
    pub const projExplosion: usize = 1824;
    pub const projExplosionEffect: usize = 1832;
    pub const projDudEffect: usize = 1840;
    pub const projExplosionSound: usize = 1848;
    pub const projDudSound: usize = 1856;
    pub const stickiness: usize = 1864;
    pub const lowAmmoWarningThreshold: usize = 1868;
    pub const ricochetChance: usize = 1872;
    pub const riotShieldEnableDamage: usize = 1876;
    pub const riotShieldHealth: usize = 1880;
    pub const riotShieldDamageMult: usize = 1884;
    pub const parallelBounce: usize = 1888;
    pub const perpendicularBounce: usize = 1896;
    pub const projTrailEffect: usize = 1904;
    pub const projBeaconEffect: usize = 1912;
    pub const vProjectileColor: usize = 1920;
    pub const guidedMissileType: usize = 1932;
    pub const maxSteeringAccel: usize = 1936;
    pub const projIgnitionDelay: usize = 1940;
    pub const projIgnitionEffect: usize = 1944;
    pub const projIgnitionSound: usize = 1952;
    pub const fAdsAimPitch: usize = 1960;
    pub const fAdsCrosshairInFrac: usize = 1964;
    pub const fAdsCrosshairOutFrac: usize = 1968;
    pub const adsShouldShowCrosshair: usize = 1972;
    pub const adsGunKickReducedKickBullets: usize = 1976;
    pub const adsGunKickReducedKickPercent: usize = 1980;
    pub const fAdsGunKickPitchMin: usize = 1984;
    pub const fAdsGunKickPitchMax: usize = 1988;
    pub const fAdsGunKickYawMin: usize = 1992;
    pub const fAdsGunKickYawMax: usize = 1996;
    pub const fAdsGunKickMagMin: usize = 2000;
    pub const fAdsGunKickAccel: usize = 2004;
    pub const fAdsGunKickSpeedMax: usize = 2008;
    pub const fAdsGunKickSpeedDecay: usize = 2012;
    pub const fAdsGunKickStaticDecay: usize = 2016;
    pub const fAdsViewKickPitchMin: usize = 2020;
    pub const fAdsViewKickPitchMax: usize = 2024;
    pub const fAdsViewKickYawMin: usize = 2028;
    pub const fAdsViewKickYawMax: usize = 2032;
    pub const fAdsViewKickMagMin: usize = 2036;
    pub const fAdsViewScatterMin: usize = 2040;
    pub const fAdsViewScatterMax: usize = 2044;
    pub const fAdsSpread: usize = 2048;
    pub const hipGunKickReducedKickBullets: usize = 2052;
    pub const hipGunKickReducedKickPercent: usize = 2056;
    pub const fHipGunKickPitchMin: usize = 2060;
    pub const fHipGunKickPitchMax: usize = 2064;
    pub const fHipGunKickYawMin: usize = 2068;
    pub const fHipGunKickYawMax: usize = 2072;
    pub const fHipGunKickMagMin: usize = 2076;
    pub const fHipGunKickAccel: usize = 2080;
    pub const fHipGunKickSpeedMax: usize = 2084;
    pub const fHipGunKickSpeedDecay: usize = 2088;
    pub const fHipGunKickStaticDecay: usize = 2092;
    pub const fHipViewKickPitchMin: usize = 2096;
    pub const fHipViewKickPitchMax: usize = 2100;
    pub const fHipViewKickYawMin: usize = 2104;
    pub const fHipViewKickYawMax: usize = 2108;
    pub const fHipViewKickMagMin: usize = 2112;
    pub const fHipViewScatterMin: usize = 2116;
    pub const fHipViewScatterMax: usize = 2120;
    pub const fightDist: usize = 2124;
    pub const maxDist: usize = 2128;
    pub const accuracyGraphName: usize = 2136;
    pub const originalAccuracyGraphKnots: usize = 2152;
    pub const originalAccuracyGraphKnotCount: usize = 2168;
    pub const iPositionReloadTransTime: usize = 2172;
    pub const leftArc: usize = 2176;
    pub const rightArc: usize = 2180;
    pub const topArc: usize = 2184;
    pub const bottomArc: usize = 2188;
    pub const accuracy: usize = 2192;
    pub const aiSpread: usize = 2196;
    pub const playerSpread: usize = 2200;
    pub const minTurnSpeed: usize = 2204;
    pub const maxTurnSpeed: usize = 2212;
    pub const pitchConvergenceTime: usize = 2220;
    pub const yawConvergenceTime: usize = 2224;
    pub const suppressTime: usize = 2228;
    pub const maxRange: usize = 2232;
    pub const fAnimHorRotateInc: usize = 2236;
    pub const fPlayerPositionDist: usize = 2240;
    pub const szUseHintString: usize = 2248;
    pub const dropHintString: usize = 2256;
    pub const iUseHintStringIndex: usize = 2264;
    pub const dropHintStringIndex: usize = 2268;
    pub const horizViewJitter: usize = 2272;
    pub const vertViewJitter: usize = 2276;
    pub const scanSpeed: usize = 2280;
    pub const scanAccel: usize = 2284;
    pub const scanPauseTime: usize = 2288;
    pub const szScript: usize = 2296;
    pub const fOOPosAnimLength: usize = 2304;
    pub const minDamage: usize = 2312;
    pub const minPlayerDamage: usize = 2316;
    pub const fMaxDamageRange: usize = 2320;
    pub const fMinDamageRange: usize = 2324;
    pub const destabilizationRateTime: usize = 2328;
    pub const destabilizationCurvatureMax: usize = 2332;
    pub const destabilizeDistance: usize = 2336;
    pub const locationDamageMultipliers: usize = 2344;
    pub const fireRumble: usize = 2352;
    pub const meleeImpactRumble: usize = 2360;
    pub const tracerType: usize = 2368;
    pub const turretADSEnabled: usize = 2376;
    pub const turretADSTime: usize = 2380;
    pub const turretFov: usize = 2384;
    pub const turretFovADS: usize = 2388;
    pub const turretScopeZoomRate: usize = 2392;
    pub const turretScopeZoomMin: usize = 2396;
    pub const turretScopeZoomMax: usize = 2400;
    pub const turretOverheatUpRate: usize = 2404;
    pub const turretOverheatDownRate: usize = 2408;
    pub const turretOverheatPenalty: usize = 2412;
    pub const turretOverheatSound: usize = 2416;
    pub const turretOverheatEffect: usize = 2424;
    pub const turretBarrelSpinRumble: usize = 2432;
    pub const turretBarrelSpinSpeed: usize = 2440;
    pub const turretBarrelSpinUpTime: usize = 2444;
    pub const turretBarrelSpinDownTime: usize = 2448;
    pub const turretBarrelSpinMaxSnd: usize = 2456;
    pub const turretBarrelSpinUpSnd: usize = 2464;
    pub const turretBarrelSpinDownSnd: usize = 2496;
    pub const missileConeSoundAlias: usize = 2528;
    pub const missileConeSoundAliasAtBase: usize = 2536;
    pub const missileConeSoundRadiusAtTop: usize = 2544;
    pub const missileConeSoundRadiusAtBase: usize = 2548;
    pub const missileConeSoundHeight: usize = 2552;
    pub const missileConeSoundOriginOffset: usize = 2556;
    pub const missileConeSoundVolumescaleAtCore: usize = 2560;
    pub const missileConeSoundVolumescaleAtEdge: usize = 2564;
    pub const missileConeSoundVolumescaleCoreSize: usize = 2568;
    pub const missileConeSoundPitchAtTop: usize = 2572;
    pub const missileConeSoundPitchAtBottom: usize = 2576;
    pub const missileConeSoundPitchTopSize: usize = 2580;
    pub const missileConeSoundPitchBottomSize: usize = 2584;
    pub const missileConeSoundCrossfadeTopSize: usize = 2588;
    pub const missileConeSoundCrossfadeBottomSize: usize = 2592;
    pub const knifeAttachTagOverride: usize = 2596;
    pub const knifeAlwaysAttached: usize = 2600;
    pub const meleeOverrideValues: usize = 2601;
    pub const aim_automelee_lerp: usize = 2604;
    pub const aim_automelee_range: usize = 2608;
    pub const aim_automelee_region_height: usize = 2612;
    pub const aim_automelee_region_width: usize = 2616;
    pub const player_meleeHeight: usize = 2620;
    pub const player_meleeRange: usize = 2624;
    pub const player_meleeWidth: usize = 2628;
    pub const sharedAmmo: usize = 2632;
    pub const lockonSupported: usize = 2633;
    pub const requireLockonToFire: usize = 2634;
    pub const isAirburstWeapon: usize = 2635;
    pub const bigExplosion: usize = 2636;
    pub const noAdsWhenMagEmpty: usize = 2637;
    pub const avoidDropCleanup: usize = 2638;
    pub const inheritsPerks: usize = 2639;
    pub const crosshairColorChange: usize = 2640;
    pub const bRifleBullet: usize = 2641;
    pub const armorPiercing: usize = 2642;
    pub const bBoltAction: usize = 2643;
    pub const aimDownSight: usize = 2644;
    pub const canHoldBreath: usize = 2645;
    pub const meleeOnly: usize = 2646;
    pub const canVariableZoom: usize = 2647;
    pub const bRechamberWhileAds: usize = 2648;
    pub const bBulletExplosiveDamage: usize = 2649;
    pub const bCookOffHold: usize = 2650;
    pub const reticleSpin45: usize = 2651;
    pub const bClipOnly: usize = 2652;
    pub const noAmmoPickup: usize = 2653;
    pub const adsFireOnly: usize = 2654;
    pub const cancelAutoHolsterWhenEmpty: usize = 2655;
    pub const disableSwitchToWhenEmpty: usize = 2656;
    pub const suppressAmmoReserveDisplay: usize = 2657;
    pub const laserSightDuringNightvision: usize = 2658;
    pub const markableViewmodel: usize = 2659;
    pub const noDualWield: usize = 2660;
    pub const flipKillIcon: usize = 2661;
    pub const bNoPartialReload: usize = 2662;
    pub const bSegmentedReload: usize = 2663;
    pub const bMultipleReload: usize = 2664;
    pub const blocksProne: usize = 2665;
    pub const silenced: usize = 2666;
    pub const isRollingGrenade: usize = 2667;
    pub const projExplosionEffectForceNormalUp: usize = 2668;
    pub const projExplosionEffectInheritParentDirection: usize = 2669;
    pub const bProjImpactExplode: usize = 2670;
    pub const bProjTrajectoryEvents: usize = 2671;
    pub const bProjWhizByEnabled: usize = 2672;
    pub const stickToPlayers: usize = 2673;
    pub const stickToVehicles: usize = 2674;
    pub const stickToTurrets: usize = 2675;
    pub const thrownSideways: usize = 2676;
    pub const hasDetonator: usize = 2677;
    pub const disableFiring: usize = 2678;
    pub const timedDetonation: usize = 2679;
    pub const rotate: usize = 2680;
    pub const holdButtonToThrow: usize = 2681;
    pub const freezeMovementWhenFiring: usize = 2682;
    pub const thermalScope: usize = 2683;
    pub const thermalToggle: usize = 2684;
    pub const outlineEnemies: usize = 2685;
    pub const altModeSameWeapon: usize = 2686;
    pub const turretBarrelSpinEnabled: usize = 2687;
    pub const missileConeSoundEnabled: usize = 2688;
    pub const missileConeSoundPitchshiftEnabled: usize = 2689;
    pub const missileConeSoundCrossfadeEnabled: usize = 2690;
    pub const offhandHoldIsCancelable: usize = 2691;
    pub const doNotAllowAttachmentsToOverrideSpread: usize = 2692;
    pub const useFastReloadAnims: usize = 2693;
    pub const useScopeDrift: usize = 2694;
    pub const alwaysShatterGlassOnImpact: usize = 2695;
    pub const oldWeapon: usize = 2696;
    pub const stowTag: usize = 2700;
    pub const stowOffsetModel: usize = 2704;
}
pub mod XA2Sound {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const data: usize = 0;
    pub const seekTable: usize = 8;
    pub const format: usize = 16;
    pub const format__sampleRate: usize = 16;
    pub const format__dataByteCount: usize = 20;
    pub const format__numSamples: usize = 24;
    pub const format__avgBytesPerSec: usize = 28;
    pub const format__channels: usize = 32;
    pub const format__numBits: usize = 34;
    pub const format__blockAlign: usize = 36;
    pub const format__format: usize = 38;
    pub const loadedSize: usize = 40;
    pub const seekTableSize: usize = 44;
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
    pub const dataByteCount: usize = 8;
    pub const dataShortCount: usize = 10;
    pub const dataIntCount: usize = 12;
    pub const randomDataByteCount: usize = 14;
    pub const randomDataIntCount: usize = 16;
    pub const numframes: usize = 18;
    pub const flags: usize = 20;
    pub const boneCount: usize = 21;
    pub const notifyCount: usize = 31;
    pub const assetType: usize = 32;
    pub const ikType: usize = 33;
    pub const randomDataShortCount: usize = 36;
    pub const indexCount: usize = 40;
    pub const framerate: usize = 44;
    pub const frequency: usize = 48;
    pub const names: usize = 56;
    pub const dataByte: usize = 64;
    pub const dataShort: usize = 72;
    pub const dataInt: usize = 80;
    pub const randomDataShort: usize = 88;
    pub const randomDataByte: usize = 96;
    pub const randomDataInt: usize = 104;
    pub const indices: usize = 112;
    pub const indices___1: usize = 112;
    pub const indices___2: usize = 112;
    pub const indices__data: usize = 112;
    pub const notify: usize = 120;
    pub const deltaPart: usize = 128;
}
pub mod XAsset {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const header: usize = 8;
    pub const header__data: usize = 8;
    pub const header__physPreset: usize = 8;
    pub const header__physCollmap: usize = 8;
    pub const header__parts: usize = 8;
    pub const header__modelSurfs: usize = 8;
    pub const header__model: usize = 8;
    pub const header__material: usize = 8;
    pub const header__computeShader: usize = 8;
    pub const header__vertexShader: usize = 8;
    pub const header__hullShader: usize = 8;
    pub const header__domainShader: usize = 8;
    pub const header__pixelShader: usize = 8;
    pub const header__vertexDecl: usize = 8;
    pub const header__techniqueSet: usize = 8;
    pub const header__image: usize = 8;
    pub const header__sound: usize = 8;
    pub const header__sndCurve: usize = 8;
    pub const header__lpfCurve: usize = 8;
    pub const header__reverbCurve: usize = 8;
    pub const header__loadSnd: usize = 8;
    pub const header__clipMap: usize = 8;
    pub const header__comWorld: usize = 8;
    pub const header__glassWorld: usize = 8;
    pub const header__pathData: usize = 8;
    pub const header__mapEnts: usize = 8;
    pub const header__fxWorld: usize = 8;
    pub const header__gfxWorld: usize = 8;
    pub const header__lightDef: usize = 8;
    pub const header__font: usize = 8;
    pub const header__menuList: usize = 8;
    pub const header__menu: usize = 8;
    pub const header__animClass: usize = 8;
    pub const header__localize: usize = 8;
    pub const header__attachment: usize = 8;
    pub const header__weapon: usize = 8;
    pub const header__fx: usize = 8;
    pub const header__rawfile: usize = 8;
    pub const header__scriptfile: usize = 8;
    pub const header__stringTable: usize = 8;
    pub const header__structuredDataDefSet: usize = 8;
    pub const header__tracerDef: usize = 8;
    pub const header__vehDef: usize = 8;
    pub const header__addonMapEnts: usize = 8;
    pub const header__netConstStrings: usize = 8;
    pub const header__luaFile: usize = 8;
    pub const header__scriptable: usize = 8;
    pub const header__colorization: usize = 8;
    pub const header__colorizationSet: usize = 8;
    pub const header__toneMapping: usize = 8;
    pub const header__vectorField: usize = 8;
    pub const header__dopplerPreset: usize = 8;
    pub const header__particleSimAnimation: usize = 8;
}
pub mod XAssetEntry {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const asset: usize = 0;
    pub const asset__type: usize = 0;
    pub const asset__header: usize = 8;
    pub const asset__header__data: usize = 8;
    pub const asset__header__physPreset: usize = 8;
    pub const asset__header__physCollmap: usize = 8;
    pub const asset__header__parts: usize = 8;
    pub const asset__header__modelSurfs: usize = 8;
    pub const asset__header__model: usize = 8;
    pub const asset__header__material: usize = 8;
    pub const asset__header__computeShader: usize = 8;
    pub const asset__header__vertexShader: usize = 8;
    pub const asset__header__hullShader: usize = 8;
    pub const asset__header__domainShader: usize = 8;
    pub const asset__header__pixelShader: usize = 8;
    pub const asset__header__vertexDecl: usize = 8;
    pub const asset__header__techniqueSet: usize = 8;
    pub const asset__header__image: usize = 8;
    pub const asset__header__sound: usize = 8;
    pub const asset__header__sndCurve: usize = 8;
    pub const asset__header__lpfCurve: usize = 8;
    pub const asset__header__reverbCurve: usize = 8;
    pub const asset__header__loadSnd: usize = 8;
    pub const asset__header__clipMap: usize = 8;
    pub const asset__header__comWorld: usize = 8;
    pub const asset__header__glassWorld: usize = 8;
    pub const asset__header__pathData: usize = 8;
    pub const asset__header__mapEnts: usize = 8;
    pub const asset__header__fxWorld: usize = 8;
    pub const asset__header__gfxWorld: usize = 8;
    pub const asset__header__lightDef: usize = 8;
    pub const asset__header__font: usize = 8;
    pub const asset__header__menuList: usize = 8;
    pub const asset__header__menu: usize = 8;
    pub const asset__header__animClass: usize = 8;
    pub const asset__header__localize: usize = 8;
    pub const asset__header__attachment: usize = 8;
    pub const asset__header__weapon: usize = 8;
    pub const asset__header__fx: usize = 8;
    pub const asset__header__rawfile: usize = 8;
    pub const asset__header__scriptfile: usize = 8;
    pub const asset__header__stringTable: usize = 8;
    pub const asset__header__structuredDataDefSet: usize = 8;
    pub const asset__header__tracerDef: usize = 8;
    pub const asset__header__vehDef: usize = 8;
    pub const asset__header__addonMapEnts: usize = 8;
    pub const asset__header__netConstStrings: usize = 8;
    pub const asset__header__luaFile: usize = 8;
    pub const asset__header__scriptable: usize = 8;
    pub const asset__header__colorization: usize = 8;
    pub const asset__header__colorizationSet: usize = 8;
    pub const asset__header__toneMapping: usize = 8;
    pub const asset__header__vectorField: usize = 8;
    pub const asset__header__dopplerPreset: usize = 8;
    pub const asset__header__particleSimAnimation: usize = 8;
    pub const zoneIndex: usize = 16;
    pub const inuseMask: usize = 17;
    pub const nextHash: usize = 20;
    pub const nextOverride: usize = 24;
    pub const nextPoolEntry: usize = 28;
}
pub mod XAssetHeader {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const data: usize = 0;
    pub const physPreset: usize = 0;
    pub const physCollmap: usize = 0;
    pub const parts: usize = 0;
    pub const modelSurfs: usize = 0;
    pub const model: usize = 0;
    pub const material: usize = 0;
    pub const computeShader: usize = 0;
    pub const vertexShader: usize = 0;
    pub const hullShader: usize = 0;
    pub const domainShader: usize = 0;
    pub const pixelShader: usize = 0;
    pub const vertexDecl: usize = 0;
    pub const techniqueSet: usize = 0;
    pub const image: usize = 0;
    pub const sound: usize = 0;
    pub const sndCurve: usize = 0;
    pub const lpfCurve: usize = 0;
    pub const reverbCurve: usize = 0;
    pub const loadSnd: usize = 0;
    pub const clipMap: usize = 0;
    pub const comWorld: usize = 0;
    pub const glassWorld: usize = 0;
    pub const pathData: usize = 0;
    pub const mapEnts: usize = 0;
    pub const fxWorld: usize = 0;
    pub const gfxWorld: usize = 0;
    pub const lightDef: usize = 0;
    pub const font: usize = 0;
    pub const menuList: usize = 0;
    pub const menu: usize = 0;
    pub const animClass: usize = 0;
    pub const localize: usize = 0;
    pub const attachment: usize = 0;
    pub const weapon: usize = 0;
    pub const fx: usize = 0;
    pub const rawfile: usize = 0;
    pub const scriptfile: usize = 0;
    pub const stringTable: usize = 0;
    pub const structuredDataDefSet: usize = 0;
    pub const tracerDef: usize = 0;
    pub const vehDef: usize = 0;
    pub const addonMapEnts: usize = 0;
    pub const netConstStrings: usize = 0;
    pub const luaFile: usize = 0;
    pub const scriptable: usize = 0;
    pub const colorization: usize = 0;
    pub const colorizationSet: usize = 0;
    pub const toneMapping: usize = 0;
    pub const vectorField: usize = 0;
    pub const dopplerPreset: usize = 0;
    pub const particleSimAnimation: usize = 0;
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
    pub const data: usize = 0;
    pub const size: usize = 8;
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
pub mod XFile {
    pub const SIZE: usize = 72;
    pub const ALIGN: usize = 8;
    pub const size: usize = 0;
    pub const externalSize: usize = 8;
    pub const blockSize: usize = 16;
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
    pub const SIZE: usize = 600;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const numBones: usize = 8;
    pub const numRootBones: usize = 9;
    pub const numsurfs: usize = 10;
    pub const numReactiveMotionParts: usize = 11;
    pub const scale: usize = 12;
    pub const __pad0: usize = 16;
    pub const noScalePartBits: usize = 20;
    pub const boneNames: usize = 48;
    pub const parentList: usize = 56;
    pub const tagAngles: usize = 64;
    pub const tagPositions: usize = 72;
    pub const partClassification: usize = 80;
    pub const baseMat: usize = 88;
    pub const reactiveMotionParts: usize = 96;
    pub const materialHandles: usize = 104;
    pub const lodInfo: usize = 112;
    pub const maxLoadedLod: usize = 496;
    pub const numLods: usize = 497;
    pub const collLod: usize = 498;
    pub const flags: usize = 499;
    pub const numCollSurfs: usize = 500;
    pub const collSurfs: usize = 504;
    pub const contents: usize = 512;
    pub const boneInfo: usize = 520;
    pub const radius: usize = 528;
    pub const bounds: usize = 532;
    pub const bounds__midPoint: usize = 532;
    pub const bounds__halfSize: usize = 544;
    pub const invHighMipRadius: usize = 560;
    pub const memUsage: usize = 568;
    pub const physPreset: usize = 576;
    pub const physCollmap: usize = 584;
    pub const quantization: usize = 592;
    pub const subdivRadius: usize = 596;
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
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 2;
    pub const hasGfxEntIndex: usize = 0;
    pub const lod: usize = 1;
    pub const surfId: usize = 2;
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
    pub const pad: usize = 61;
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
pub mod XRigidVertList {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const boneOffset: usize = 0;
    pub const vertCount: usize = 2;
    pub const triOffset: usize = 4;
    pub const triCount: usize = 6;
    pub const collisionTree: usize = 8;
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
    pub const SIZE: usize = 232;
    pub const ALIGN: usize = 8;
    pub const flags: usize = 0;
    pub const vertCount: usize = 2;
    pub const triCount: usize = 4;
    pub const rigidVertListCount: usize = 6;
    pub const subdivLevelCount: usize = 7;
    pub const blendVertCounts: usize = 8;
    pub const verts0: usize = 24;
    pub const verts0__quantizedNoColorVerts0: usize = 24;
    pub const verts0__quantizedNoColorMotionVerts0: usize = 24;
    pub const verts0__quantizedVerts0: usize = 24;
    pub const verts0__quantizedMotionVerts0: usize = 24;
    pub const verts0__packedVerts0: usize = 24;
    pub const verts0__packedMotionVerts0: usize = 24;
    pub const verts0__verts0: usize = 24;
    pub const triIndices: usize = 32;
    pub const vb0: usize = 40;
    pub const vb0View: usize = 48;
    pub const indexBuffer: usize = 56;
    pub const rigidVertLists: usize = 64;
    pub const blendVerts: usize = 72;
    pub const blendVertsTable: usize = 80;
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
    pub const vertexLightingIndex: usize = 192;
    pub const partBits: usize = 196;
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
    pub const totalVertCount: usize = 12;
    pub const totalRegularPatchCount: usize = 16;
    pub const unk1: usize = 20;
    pub const unk2: usize = 24;
    pub const unk3: usize = 28;
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
pub mod XZoneInfoInternal {
    pub const SIZE: usize = 72;
    pub const ALIGN: usize = 4;
    pub const name: usize = 0;
    pub const flags: usize = 64;
    pub const isBaseMap: usize = 68;
}
pub mod XZoneMemory_ {
    pub const SIZE: usize = 112;
    pub const ALIGN: usize = 8;
    pub const blocks: usize = 0;
}
pub mod cLeafBrushNodeChildren_t {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const dist: usize = 0;
    pub const range: usize = 4;
    pub const childOffset: usize = 8;
}
pub mod cLeafBrushNodeData_t {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const leaf: usize = 0;
    pub const leaf__brushes: usize = 0;
    pub const children: usize = 0;
    pub const children__dist: usize = 0;
    pub const children__range: usize = 4;
    pub const children__childOffset: usize = 8;
}
pub mod cLeafBrushNodeLeaf_t {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const brushes: usize = 0;
}
pub mod cLeafBrushNode_s {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const axis: usize = 0;
    pub const leafBrushCount: usize = 2;
    pub const contents: usize = 4;
    pub const data: usize = 8;
    pub const data__leaf: usize = 8;
    pub const data__leaf__brushes: usize = 8;
    pub const data__children: usize = 8;
    pub const data__children__dist: usize = 8;
    pub const data__children__range: usize = 12;
    pub const data__children__childOffset: usize = 16;
}
pub mod cLeaf_t {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 4;
    pub const firstCollAabbIndex: usize = 0;
    pub const collAabbCount: usize = 2;
    pub const brushContents: usize = 4;
    pub const terrainContents: usize = 8;
    pub const bounds: usize = 12;
    pub const bounds__midPoint: usize = 12;
    pub const bounds__halfSize: usize = 24;
    pub const leafBrushNode: usize = 36;
}
pub mod cNode_t {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const plane: usize = 0;
    pub const children: usize = 8;
}
pub mod cStaticModel_s {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 8;
    pub const xmodel: usize = 0;
    pub const origin: usize = 8;
    pub const invScaledAxis: usize = 20;
    pub const absBounds: usize = 56;
    pub const absBounds__midPoint: usize = 56;
    pub const absBounds__halfSize: usize = 68;
}
pub mod cbrush_t {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const numsides: usize = 0;
    pub const glassPieceIndex: usize = 2;
    pub const sides: usize = 8;
    pub const baseAdjacentSide: usize = 16;
    pub const axialMaterialNum: usize = 24;
    pub const firstAdjacentSideOffsets: usize = 36;
    pub const edgeCount: usize = 42;
}
pub mod cbrushside_t {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const plane: usize = 0;
    pub const materialNum: usize = 8;
    pub const firstAdjacentSideOffset: usize = 10;
    pub const edgeCount: usize = 11;
}
pub mod cmodel_t {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 8;
    pub const bounds: usize = 0;
    pub const bounds__midPoint: usize = 0;
    pub const bounds__halfSize: usize = 12;
    pub const radius: usize = 24;
    pub const info: usize = 32;
    pub const leaf: usize = 40;
    pub const leaf__firstCollAabbIndex: usize = 40;
    pub const leaf__collAabbCount: usize = 42;
    pub const leaf__brushContents: usize = 44;
    pub const leaf__terrainContents: usize = 48;
    pub const leaf__bounds: usize = 52;
    pub const leaf__bounds__midPoint: usize = 52;
    pub const leaf__bounds__halfSize: usize = 64;
    pub const leaf__leafBrushNode: usize = 76;
}
pub mod columnInfo_s {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 4;
    pub const xpos: usize = 0;
    pub const ypos: usize = 4;
    pub const width: usize = 8;
    pub const height: usize = 12;
    pub const maxChars: usize = 16;
    pub const alignment: usize = 20;
}
pub mod cplane_s {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 4;
    pub const normal: usize = 0;
    pub const dist: usize = 12;
    pub const r#type: usize = 16;
}
pub mod editFieldDef_s {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const minVal: usize = 0;
    pub const maxVal: usize = 4;
    pub const stepVal: usize = 8;
    pub const range: usize = 12;
    pub const maxChars: usize = 16;
    pub const maxCharsGotoNext: usize = 20;
    pub const maxPaintChars: usize = 24;
    pub const paintOffset: usize = 28;
}
pub mod entryInternalData {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const op: usize = 0;
    pub const operand: usize = 0;
    pub const operand__internals: usize = 0;
    pub const operand__internals__intVal: usize = 0;
    pub const operand__internals__floatVal: usize = 0;
    pub const operand__internals__stringVal: usize = 0;
    pub const operand__internals__stringVal__string: usize = 0;
    pub const operand__internals__function: usize = 0;
    pub const operand__dataType: usize = 8;
}
pub mod expressionEntry {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const data: usize = 0;
    pub const data__op: usize = 0;
    pub const data__operand: usize = 0;
    pub const data__operand__internals: usize = 0;
    pub const data__operand__internals__intVal: usize = 0;
    pub const data__operand__internals__floatVal: usize = 0;
    pub const data__operand__internals__stringVal: usize = 0;
    pub const data__operand__internals__stringVal__string: usize = 0;
    pub const data__operand__internals__function: usize = 0;
    pub const data__operand__dataType: usize = 8;
    pub const r#type: usize = 12;
}
pub mod itemDefData_t {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const listBox: usize = 0;
    pub const editField: usize = 0;
    pub const multi: usize = 0;
    pub const enumDvarName: usize = 0;
    pub const ticker: usize = 0;
    pub const scroll: usize = 0;
    pub const data: usize = 0;
}
pub mod itemDef_t {
    pub const SIZE: usize = 504;
    pub const ALIGN: usize = 8;
    pub const window: usize = 0;
    pub const window__name: usize = 0;
    pub const window__rect: usize = 8;
    pub const window__rect__x: usize = 8;
    pub const window__rect__y: usize = 12;
    pub const window__rect__w: usize = 16;
    pub const window__rect__h: usize = 20;
    pub const window__rect__horzAlign: usize = 24;
    pub const window__rect__vertAlign: usize = 25;
    pub const window__rectClient: usize = 28;
    pub const window__rectClient__x: usize = 28;
    pub const window__rectClient__y: usize = 32;
    pub const window__rectClient__w: usize = 36;
    pub const window__rectClient__h: usize = 40;
    pub const window__rectClient__horzAlign: usize = 44;
    pub const window__rectClient__vertAlign: usize = 45;
    pub const window__group: usize = 48;
    pub const window__style: usize = 56;
    pub const window__border: usize = 60;
    pub const window__ownerDraw: usize = 64;
    pub const window__ownerDrawFlags: usize = 68;
    pub const window__borderSize: usize = 72;
    pub const window__staticFlags: usize = 76;
    pub const window__dynamicFlags: usize = 80;
    pub const window__nextTime: usize = 84;
    pub const window__foreColor: usize = 88;
    pub const window__backColor: usize = 104;
    pub const window__borderColor: usize = 120;
    pub const window__outlineColor: usize = 136;
    pub const window__disableColor: usize = 152;
    pub const window__background: usize = 168;
    pub const textRect: usize = 176;
    pub const r#type: usize = 196;
    pub const dataType: usize = 200;
    pub const alignment: usize = 204;
    pub const fontEnum: usize = 208;
    pub const textAlignMode: usize = 212;
    pub const textalignx: usize = 216;
    pub const textaligny: usize = 220;
    pub const textscale: usize = 224;
    pub const textStyle: usize = 228;
    pub const gameMsgWindowIndex: usize = 232;
    pub const gameMsgWindowMode: usize = 236;
    pub const text: usize = 240;
    pub const itemFlags: usize = 248;
    pub const parent: usize = 256;
    pub const mouseEnterText: usize = 264;
    pub const mouseExitText: usize = 272;
    pub const mouseEnter: usize = 280;
    pub const mouseExit: usize = 288;
    pub const action: usize = 296;
    pub const accept: usize = 304;
    pub const onFocus: usize = 312;
    pub const hasFocus: usize = 320;
    pub const leaveFocus: usize = 328;
    pub const dvar: usize = 336;
    pub const dvarTest: usize = 344;
    pub const onKey: usize = 352;
    pub const enableDvar: usize = 360;
    pub const localVar: usize = 368;
    pub const dvarFlags: usize = 376;
    pub const focusSound: usize = 384;
    pub const special: usize = 392;
    pub const cursorPos: usize = 396;
    pub const typeData: usize = 400;
    pub const typeData__listBox: usize = 400;
    pub const typeData__editField: usize = 400;
    pub const typeData__multi: usize = 400;
    pub const typeData__enumDvarName: usize = 400;
    pub const typeData__ticker: usize = 400;
    pub const typeData__scroll: usize = 400;
    pub const typeData__data: usize = 400;
    pub const floatExpressionCount: usize = 408;
    pub const floatExpressions: usize = 416;
    pub const visibleExp: usize = 424;
    pub const disabledExp: usize = 432;
    pub const textExp: usize = 440;
    pub const materialExp: usize = 448;
    pub const glowColor: usize = 456;
    pub const decayActive: usize = 472;
    pub const fxBirthTime: usize = 476;
    pub const fxLetterTime: usize = 480;
    pub const fxDecayStartTime: usize = 484;
    pub const fxDecayDuration: usize = 488;
    pub const lastSoundPlayedTime: usize = 492;
    pub const textAlignYExp: usize = 496;
}
pub mod listBoxDef_s {
    pub const SIZE: usize = 472;
    pub const ALIGN: usize = 8;
    pub const startPos: usize = 0;
    pub const endPos: usize = 4;
    pub const drawPadding: usize = 8;
    pub const elementWidth: usize = 12;
    pub const elementHeight: usize = 16;
    pub const elementStyle: usize = 20;
    pub const numColumns: usize = 24;
    pub const columnInfo: usize = 28;
    pub const onDoubleClick: usize = 416;
    pub const notselectable: usize = 424;
    pub const noScrollBars: usize = 428;
    pub const usePaging: usize = 432;
    pub const selectBorder: usize = 436;
    pub const selectIcon: usize = 456;
    pub const elementHeightExp: usize = 464;
}
pub mod menuData_t {
    pub const SIZE: usize = 288;
    pub const ALIGN: usize = 8;
    pub const fullScreen: usize = 0;
    pub const fadeCycle: usize = 4;
    pub const fadeClamp: usize = 8;
    pub const fadeAmount: usize = 12;
    pub const fadeInAmount: usize = 16;
    pub const blurRadius: usize = 20;
    pub const onOpen: usize = 24;
    pub const onCloseRequest: usize = 32;
    pub const onClose: usize = 40;
    pub const onESC: usize = 48;
    pub const onFocusDueToClose: usize = 56;
    pub const onKey: usize = 64;
    pub const visibleExp: usize = 72;
    pub const allowedBinding: usize = 80;
    pub const soundName: usize = 88;
    pub const focusColor: usize = 96;
    pub const rectXExp: usize = 112;
    pub const rectYExp: usize = 120;
    pub const rectWExp: usize = 128;
    pub const rectHExp: usize = 136;
    pub const openSoundExp: usize = 144;
    pub const closeSoundExp: usize = 152;
    pub const soundLoopExp: usize = 160;
    pub const cursorItem: usize = 168;
    pub const scaleTransition: usize = 172;
    pub const alphaTransition: usize = 196;
    pub const xTransition: usize = 220;
    pub const yTransition: usize = 244;
    pub const expressionData: usize = 272;
    pub const priority: usize = 280;
}
pub mod menuDef_t {
    pub const SIZE: usize = 200;
    pub const ALIGN: usize = 8;
    pub const data: usize = 0;
    pub const window: usize = 8;
    pub const window__name: usize = 8;
    pub const window__rect: usize = 16;
    pub const window__rect__x: usize = 16;
    pub const window__rect__y: usize = 20;
    pub const window__rect__w: usize = 24;
    pub const window__rect__h: usize = 28;
    pub const window__rect__horzAlign: usize = 32;
    pub const window__rect__vertAlign: usize = 33;
    pub const window__rectClient: usize = 36;
    pub const window__rectClient__x: usize = 36;
    pub const window__rectClient__y: usize = 40;
    pub const window__rectClient__w: usize = 44;
    pub const window__rectClient__h: usize = 48;
    pub const window__rectClient__horzAlign: usize = 52;
    pub const window__rectClient__vertAlign: usize = 53;
    pub const window__group: usize = 56;
    pub const window__style: usize = 64;
    pub const window__border: usize = 68;
    pub const window__ownerDraw: usize = 72;
    pub const window__ownerDrawFlags: usize = 76;
    pub const window__borderSize: usize = 80;
    pub const window__staticFlags: usize = 84;
    pub const window__dynamicFlags: usize = 88;
    pub const window__nextTime: usize = 92;
    pub const window__foreColor: usize = 96;
    pub const window__backColor: usize = 112;
    pub const window__borderColor: usize = 128;
    pub const window__outlineColor: usize = 144;
    pub const window__disableColor: usize = 160;
    pub const window__background: usize = 176;
    pub const itemCount: usize = 184;
    pub const items: usize = 192;
}
pub mod menuTransition {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 4;
    pub const transitionType: usize = 0;
    pub const startTime: usize = 4;
    pub const startVal: usize = 8;
    pub const endVal: usize = 12;
    pub const time: usize = 16;
    pub const endTriggerType: usize = 20;
}
pub mod multiDef_s {
    pub const SIZE: usize = 648;
    pub const ALIGN: usize = 8;
    pub const dvarList: usize = 0;
    pub const dvarStr: usize = 256;
    pub const dvarValue: usize = 512;
    pub const count: usize = 640;
    pub const strDef: usize = 644;
}
pub mod newsTickerDef_s {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const feedId: usize = 0;
    pub const speed: usize = 4;
    pub const spacing: usize = 8;
}
pub mod operandInternalDataUnion {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const intVal: usize = 0;
    pub const floatVal: usize = 0;
    pub const stringVal: usize = 0;
    pub const stringVal__string: usize = 0;
    pub const function: usize = 0;
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
    pub const ___u9: usize = 44;
    pub const ___u9__yaw_orient: usize = 44;
    pub const ___u9__yaw_orient__fLocalAngle: usize = 44;
    pub const ___u9__yaw_orient__localForward: usize = 48;
    pub const ___u9__angles: usize = 44;
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
    pub const constant_____u9: usize = 44;
    pub const constant_____u9__yaw_orient: usize = 44;
    pub const constant_____u9__yaw_orient__fLocalAngle: usize = 44;
    pub const constant_____u9__yaw_orient__localForward: usize = 48;
    pub const constant_____u9__angles: usize = 44;
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
pub mod rectDef_s {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 4;
    pub const x: usize = 0;
    pub const y: usize = 4;
    pub const w: usize = 8;
    pub const h: usize = 12;
    pub const horzAlign: usize = 16;
    pub const vertAlign: usize = 17;
}
pub mod snd_alias_list_t {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const aliasName: usize = 0;
    pub const name: usize = 0;
    pub const head: usize = 8;
    pub const count: usize = 16;
}
pub mod snd_alias_t {
    pub const SIZE: usize = 200;
    pub const ALIGN: usize = 8;
    pub const aliasName: usize = 0;
    pub const name: usize = 0;
    pub const subtitle: usize = 8;
    pub const secondaryAliasName: usize = 16;
    pub const chainAliasName: usize = 24;
    pub const mixerGroup: usize = 32;
    pub const soundFile: usize = 40;
    pub const sequence: usize = 48;
    pub const volMin: usize = 52;
    pub const volMax: usize = 56;
    pub const volModIndex: usize = 60;
    pub const pitchMin: usize = 64;
    pub const pitchMax: usize = 68;
    pub const distMin: usize = 72;
    pub const distMax: usize = 76;
    pub const velocityMin: usize = 80;
    pub const flags: usize = 84;
    pub const masterPriority: usize = 88;
    pub const masterPercentage: usize = 92;
    pub const slavePercentage: usize = 96;
    pub const probability: usize = 100;
    pub const lfePercentage: usize = 104;
    pub const centerPercentage: usize = 108;
    pub const startDelay: usize = 112;
    pub const volumeFalloffCurve: usize = 120;
    pub const lpfCurve: usize = 128;
    pub const reverbSendCurve: usize = 136;
    pub const envelopMin: usize = 144;
    pub const envelopMax: usize = 148;
    pub const envelopPercentage: usize = 152;
    pub const speakerMap: usize = 160;
    pub const wetMixOverride: usize = 168;
    pub const occlusionShape: usize = 176;
    pub const allowDoppler: usize = 184;
    pub const dopplerPreset: usize = 192;
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
pub mod textScrollDef_s {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 4;
    pub const startTime: usize = 0;
}
pub mod water_t {
    pub const SIZE: usize = 88;
    pub const ALIGN: usize = 8;
    pub const writable: usize = 0;
    pub const writable__floatTime: usize = 0;
    pub const H0X: usize = 8;
    pub const H0Y: usize = 16;
    pub const wTerm: usize = 24;
    pub const M: usize = 32;
    pub const N: usize = 36;
    pub const Lx: usize = 40;
    pub const Lz: usize = 44;
    pub const gravity: usize = 48;
    pub const windvel: usize = 52;
    pub const winddir: usize = 56;
    pub const amplitude: usize = 64;
    pub const image: usize = 72;
    pub const stagingImage: usize = 80;
}
pub mod windowDef_t {
    pub const SIZE: usize = 176;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const rect: usize = 8;
    pub const rect__x: usize = 8;
    pub const rect__y: usize = 12;
    pub const rect__w: usize = 16;
    pub const rect__h: usize = 20;
    pub const rect__horzAlign: usize = 24;
    pub const rect__vertAlign: usize = 25;
    pub const rectClient: usize = 28;
    pub const rectClient__x: usize = 28;
    pub const rectClient__y: usize = 32;
    pub const rectClient__w: usize = 36;
    pub const rectClient__h: usize = 40;
    pub const rectClient__horzAlign: usize = 44;
    pub const rectClient__vertAlign: usize = 45;
    pub const group: usize = 48;
    pub const style: usize = 56;
    pub const border: usize = 60;
    pub const ownerDraw: usize = 64;
    pub const ownerDrawFlags: usize = 68;
    pub const borderSize: usize = 72;
    pub const staticFlags: usize = 76;
    pub const dynamicFlags: usize = 80;
    pub const nextTime: usize = 84;
    pub const foreColor: usize = 88;
    pub const backColor: usize = 104;
    pub const borderColor: usize = 120;
    pub const outlineColor: usize = 136;
    pub const disableColor: usize = 152;
    pub const background: usize = 168;
}
