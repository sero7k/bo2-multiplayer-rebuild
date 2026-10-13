//! Generated: MSVC x64 record layouts of the x64-zt S1 headers
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
    pub const info: usize = 72;
    pub const numSubModels: usize = 80;
    pub const cmodels: usize = 88;
    pub const models: usize = 96;
    pub const physModels: usize = 104;
    pub const polytope: usize = 112;
    pub const meshData: usize = 120;
    pub const polytopeCount: usize = 128;
    pub const meshDataCount: usize = 132;
}
pub mod AnimOverrideEntry {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const animHand: usize = 0;
    pub const attachment1: usize = 1;
    pub const attachment2: usize = 2;
    pub const animTreeType: usize = 3;
    pub const overrideAnim: usize = 8;
    pub const altmodeAnim: usize = 16;
    pub const animTime: usize = 24;
    pub const altTime: usize = 28;
}
pub mod AttChargeInfo {
    pub const SIZE: usize = 28;
    pub const ALIGN: usize = 4;
    pub const minChargeTime: usize = 0;
    pub const overChargeTime: usize = 4;
    pub const timePerChargeShot: usize = 8;
    pub const maxChargeShots: usize = 12;
    pub const minChargeAngle: usize = 16;
    pub const maxChargeAngle: usize = 20;
    pub const autoFireOnMaxCharge: usize = 24;
}
pub mod AttHybridSettings {
    pub const SIZE: usize = 72;
    pub const ALIGN: usize = 4;
    pub const adsSpread: usize = 0;
    pub const adsAimPitch: usize = 4;
    pub const adsTransInTime: usize = 8;
    pub const adsTransInFromSprintTime: usize = 12;
    pub const adsTransOutTime: usize = 16;
    pub const adsReloadTransTime: usize = 20;
    pub const adsCrosshairInFrac: usize = 24;
    pub const adsCrosshairOutFrac: usize = 28;
    pub const adsZoomFov: usize = 32;
    pub const adsZoomInFrac: usize = 36;
    pub const adsZoomOutFrac: usize = 40;
    pub const adsFovLerpInTime: usize = 44;
    pub const adsFovLerpOutTime: usize = 48;
    pub const adsBobFactor: usize = 52;
    pub const adsViewBobMult: usize = 56;
    pub const adsViewErrorMin: usize = 60;
    pub const adsViewErrorMax: usize = 64;
    pub const adsFireAnimFrac: usize = 68;
}
pub mod BModelDrawInfo {
    pub const SIZE: usize = 2;
    pub const ALIGN: usize = 2;
    pub const surfId: usize = 0;
}
pub mod BlendShape {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const vertCount: usize = 0;
    pub const verts: usize = 8;
    pub const blendShapeVertsBuffer: usize = 16;
    pub const blendShapeVertsView: usize = 24;
}
pub mod BlendShapeVert {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const positionOffset: usize = 0;
    pub const vertIndex: usize = 12;
    pub const normalOffset: usize = 16;
    pub const normalOffset__packed: usize = 16;
    pub const tangentOffset: usize = 20;
    pub const tangentOffset__packed: usize = 20;
    pub const tensionS: usize = 24;
    pub const tensionT: usize = 28;
}
pub mod BlendShapeWeightMap {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const weightIndex: usize = 0;
    pub const targetIndex: usize = 2;
    pub const fullWeight: usize = 4;
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
pub mod BrushesCollisionData {
    pub const SIZE: usize = 64;
    pub const ALIGN: usize = 8;
    pub const numBrushSides: usize = 0;
    pub const brushSides: usize = 8;
    pub const numBrushEdges: usize = 16;
    pub const brushEdges: usize = 24;
    pub const numBrushes: usize = 32;
    pub const brushes: usize = 40;
    pub const brushBounds: usize = 48;
    pub const brushContents: usize = 56;
}
pub mod BrushesCollisionTree {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const leafbrushNodesCount: usize = 0;
    pub const leafbrushNodes: usize = 8;
    pub const numLeafBrushes: usize = 16;
    pub const leafbrushes: usize = 24;
}
pub mod CardMemory {
    pub const SIZE: usize = 2;
    pub const ALIGN: usize = 1;
    pub const platform: usize = 0;
}
pub mod ChannelMap {
    pub const SIZE: usize = 76;
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
    pub const SIZE: usize = 168;
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
    pub const lightSetTriggers: usize = 88;
    pub const clutTriggers: usize = 96;
    pub const triggerType: usize = 104;
    pub const origins: usize = 112;
    pub const scriptDelay: usize = 120;
    pub const audioTriggers: usize = 128;
    pub const blendLookup: usize = 136;
    pub const npcTriggers: usize = 144;
    pub const contextTriggers: usize = 152;
    pub const waterTriggers: usize = 160;
}
pub mod ClipInfo {
    pub const SIZE: usize = 248;
    pub const ALIGN: usize = 8;
    pub const planeCount: usize = 0;
    pub const planes: usize = 8;
    pub const numMaterials: usize = 16;
    pub const materials: usize = 24;
    pub const bCollisionTree: usize = 32;
    pub const bCollisionTree__leafbrushNodesCount: usize = 32;
    pub const bCollisionTree__leafbrushNodes: usize = 40;
    pub const bCollisionTree__numLeafBrushes: usize = 48;
    pub const bCollisionTree__leafbrushes: usize = 56;
    pub const pCollisionTree: usize = 64;
    pub const pCollisionTree__aabbTreeCount: usize = 64;
    pub const pCollisionTree__aabbTrees: usize = 72;
    pub const sCollisionTree: usize = 80;
    pub const sCollisionTree__numStaticModels: usize = 80;
    pub const sCollisionTree__smodelNodeCount: usize = 82;
    pub const sCollisionTree__smodelNodes: usize = 88;
    pub const bCollisionData: usize = 96;
    pub const bCollisionData__numBrushSides: usize = 96;
    pub const bCollisionData__brushSides: usize = 104;
    pub const bCollisionData__numBrushEdges: usize = 112;
    pub const bCollisionData__brushEdges: usize = 120;
    pub const bCollisionData__numBrushes: usize = 128;
    pub const bCollisionData__brushes: usize = 136;
    pub const bCollisionData__brushBounds: usize = 144;
    pub const bCollisionData__brushContents: usize = 152;
    pub const pCollisionData: usize = 160;
    pub const pCollisionData__vertCount: usize = 160;
    pub const pCollisionData__verts: usize = 168;
    pub const pCollisionData__triCount: usize = 176;
    pub const pCollisionData__triIndices: usize = 184;
    pub const pCollisionData__triEdgeIsWalkable: usize = 192;
    pub const pCollisionData__borderCount: usize = 200;
    pub const pCollisionData__borders: usize = 208;
    pub const pCollisionData__partitionCount: usize = 216;
    pub const pCollisionData__partitions: usize = 224;
    pub const sCollisionData: usize = 232;
    pub const sCollisionData__numStaticModels: usize = 232;
    pub const sCollisionData__staticModelList: usize = 240;
}
pub mod ClipMaterial {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const surfaceFlags: usize = 8;
    pub const contents: usize = 12;
}
pub mod Clut {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const width: usize = 0;
    pub const height: usize = 4;
    pub const depth: usize = 8;
    pub const pixels: usize = 16;
    pub const name: usize = 24;
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
pub mod ComPrimaryLight {
    pub const SIZE: usize = 144;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const canUseShadowMap: usize = 1;
    pub const physicallyBased: usize = 2;
    pub const exponent: usize = 3;
    pub const lightingState: usize = 4;
    pub const color: usize = 8;
    pub const dir: usize = 20;
    pub const up: usize = 32;
    pub const origin: usize = 44;
    pub const fadeOffset: usize = 56;
    pub const bulbRadius: usize = 64;
    pub const bulbLength: usize = 68;
    pub const radius: usize = 80;
    pub const cosHalfFovOuter: usize = 84;
    pub const cosHalfFovInner: usize = 88;
    pub const cosHalfFovExpanded: usize = 92;
    pub const rotationLimit: usize = 96;
    pub const translationLimit: usize = 100;
    pub const cucRotationOffsetRad: usize = 104;
    pub const cucRotationSpeedRad: usize = 108;
    pub const cucScrollVector: usize = 112;
    pub const cucScaleVector: usize = 120;
    pub const cucTransVector: usize = 128;
    pub const defName: usize = 136;
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
    pub const prog__loadDef____pad: usize = 28;
}
pub mod ComputeShaderProgram {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const cs: usize = 0;
    pub const loadDef: usize = 8;
    pub const loadDef__program: usize = 8;
    pub const loadDef__programSize: usize = 16;
    pub const loadDef____pad: usize = 20;
}
pub mod ConditionalScript {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const eventHandlerSet: usize = 0;
    pub const eventExpression: usize = 8;
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
    pub const SIZE: usize = 136;
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
    pub const sound: usize = 56;
    pub const physPreset: usize = 64;
    pub const hinge: usize = 72;
    pub const linkTo: usize = 80;
    pub const mass: usize = 88;
    pub const mass__centerOfMass: usize = 88;
    pub const mass__momentsOfInertia: usize = 100;
    pub const mass__productsOfInertia: usize = 112;
    pub const contents: usize = 124;
    pub const unk: usize = 128;
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
    pub const SIZE: usize = 36;
    pub const ALIGN: usize = 4;
    pub const pose: usize = 0;
    pub const pose__quat: usize = 0;
    pub const pose__origin: usize = 16;
    pub const radius: usize = 28;
    pub const __pad0: usize = 32;
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
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const attachment1: usize = 0;
    pub const attachment2: usize = 1;
    pub const fxType: usize = 2;
    pub const overrideFX: usize = 8;
    pub const altmodeFX: usize = 16;
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
    pub const SIZE: usize = 320;
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
    pub const extended__unknownDef: usize = 280;
    pub const extended__trailDef: usize = 280;
    pub const extended__sparkFountainDef: usize = 280;
    pub const extended__spotLightDef: usize = 280;
    pub const extended__omniLightDef: usize = 280;
    pub const extended__flareDef: usize = 280;
    pub const sortOrder: usize = 288;
    pub const lightingFrac: usize = 289;
    pub const useItemClip: usize = 290;
    pub const fadeInfo: usize = 291;
    pub const fadeOutInfo: usize = 292;
    pub const randomSeed: usize = 296;
    pub const emissiveScaleScale: usize = 300;
    pub const hdrLightingFrac: usize = 304;
    pub const shadowDensityScale: usize = 308;
    pub const scatterRatio: usize = 312;
    pub const volumetricTrailFadeStart: usize = 316;
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
    pub const unknownDef: usize = 0;
    pub const trailDef: usize = 0;
    pub const sparkFountainDef: usize = 0;
    pub const spotLightDef: usize = 0;
    pub const omniLightDef: usize = 0;
    pub const flareDef: usize = 0;
}
pub mod FxElemMarkVisuals {
    pub const SIZE: usize = 16;
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
    pub const SIZE: usize = 112;
    pub const ALIGN: usize = 4;
    pub const base: usize = 0;
    pub const base__color: usize = 0;
    pub const base__emissiveScale: usize = 16;
    pub const base__rotationDelta: usize = 28;
    pub const base__rotationTotal: usize = 32;
    pub const base__size: usize = 36;
    pub const base__scale: usize = 44;
    pub const base__pivot: usize = 48;
    pub const amplitude: usize = 56;
    pub const amplitude__color: usize = 56;
    pub const amplitude__emissiveScale: usize = 72;
    pub const amplitude__rotationDelta: usize = 84;
    pub const amplitude__rotationTotal: usize = 88;
    pub const amplitude__size: usize = 92;
    pub const amplitude__scale: usize = 100;
    pub const amplitude__pivot: usize = 104;
}
pub mod FxElemVisualState {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 4;
    pub const color: usize = 0;
    pub const emissiveScale: usize = 16;
    pub const rotationDelta: usize = 28;
    pub const rotationTotal: usize = 32;
    pub const size: usize = 36;
    pub const scale: usize = 44;
    pub const pivot: usize = 48;
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
    pub const cellCount: usize = 20;
    pub const activePieceCount: usize = 24;
    pub const firstFreePiece: usize = 28;
    pub const geoDataLimit: usize = 32;
    pub const geoDataCount: usize = 36;
    pub const initGeoDataCount: usize = 40;
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
    pub const initGeoData: usize = 136;
    pub const needToCompactData: usize = 144;
    pub const initCount: usize = 145;
    pub const effectChanceAccum: usize = 148;
    pub const lastPieceDeletionTime: usize = 152;
    pub const initPieceCount: usize = 156;
    pub const initPieceStates: usize = 160;
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
pub mod FxOmniLightDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const bulbRadius: usize = 0;
    pub const bulbLength: usize = 4;
    pub const fadeOffsetRt: usize = 8;
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
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 4;
    pub const halfFovOuter: usize = 0;
    pub const halfFovInner: usize = 4;
    pub const radius: usize = 8;
    pub const brightness: usize = 12;
    pub const maxLength: usize = 16;
    pub const exponent: usize = 20;
    pub const nearClip: usize = 24;
    pub const bulbRadius: usize = 28;
    pub const bulbLength: usize = 32;
    pub const fadeOffsetRt: usize = 36;
    pub const unk1: usize = 44;
    pub const opl: usize = 45;
    pub const unk2: usize = 46;
    pub const unused: usize = 47;
}
pub mod FxTrailDef {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const scrollTimeMsec: usize = 0;
    pub const repeatDist: usize = 4;
    pub const invSplitDist: usize = 8;
    pub const invSplitArcDist: usize = 12;
    pub const invSplitTime: usize = 16;
    pub const headFadingFactor: usize = 20;
    pub const tailFadingFactor: usize = 24;
    pub const vertCount: usize = 28;
    pub const verts: usize = 32;
    pub const indCount: usize = 40;
    pub const inds: usize = 48;
}
pub mod FxTrailVertex {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const pos: usize = 0;
    pub const normal: usize = 8;
    pub const texCoord: usize = 16;
    pub const radialNormal: usize = 24;
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
    pub const glassSys__cellCount: usize = 28;
    pub const glassSys__activePieceCount: usize = 32;
    pub const glassSys__firstFreePiece: usize = 36;
    pub const glassSys__geoDataLimit: usize = 40;
    pub const glassSys__geoDataCount: usize = 44;
    pub const glassSys__initGeoDataCount: usize = 48;
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
    pub const glassSys__initGeoData: usize = 144;
    pub const glassSys__needToCompactData: usize = 152;
    pub const glassSys__initCount: usize = 153;
    pub const glassSys__effectChanceAccum: usize = 156;
    pub const glassSys__lastPieceDeletionTime: usize = 160;
    pub const glassSys__initPieceCount: usize = 164;
    pub const glassSys__initPieceStates: usize = 168;
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
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const bounds: usize = 0;
    pub const bounds__midPoint: usize = 0;
    pub const bounds__halfSize: usize = 12;
    pub const childrenOffset: usize = 24;
    pub const childCount: usize = 28;
    pub const smodelIndexCount: usize = 30;
    pub const smodelIndexes: usize = 32;
    pub const startSurfIndex: usize = 40;
    pub const surfaceCount: usize = 44;
    pub const pad: usize = 46;
}
pub mod GfxBrushModel {
    pub const SIZE: usize = 96;
    pub const ALIGN: usize = 4;
    pub const writable: usize = 0;
    pub const writable__bounds: usize = 0;
    pub const writable__bounds__midPoint: usize = 0;
    pub const writable__bounds__halfSize: usize = 12;
    pub const writable__origin: usize = 24;
    pub const writable__quat: usize = 36;
    pub const writable__mdaoVolumeProcessed: usize = 52;
    pub const bounds: usize = 56;
    pub const bounds__midPoint: usize = 56;
    pub const bounds__halfSize: usize = 68;
    pub const radius: usize = 80;
    pub const startSurfIndex: usize = 84;
    pub const surfaceCount: usize = 88;
    pub const mdaoVolumeIndex: usize = 92;
}
pub mod GfxBrushModelWritable {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 4;
    pub const bounds: usize = 0;
    pub const bounds__midPoint: usize = 0;
    pub const bounds__halfSize: usize = 12;
    pub const origin: usize = 24;
    pub const quat: usize = 36;
    pub const mdaoVolumeProcessed: usize = 52;
}
pub mod GfxBuildInfo {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const bspCommandline: usize = 0;
    pub const lightCommandline: usize = 8;
    pub const bspTimestamp: usize = 16;
    pub const lightTimestamp: usize = 24;
}
pub mod GfxCell {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const bounds: usize = 0;
    pub const bounds__midPoint: usize = 0;
    pub const bounds__halfSize: usize = 12;
    pub const portalCount: usize = 24;
    pub const reflectionProbeCount: usize = 26;
    pub const reflectionProbeReferenceCount: usize = 27;
    pub const portals: usize = 32;
    pub const reflectionProbes: usize = 40;
    pub const reflectionProbeReferences: usize = 48;
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
pub mod GfxColorHdr {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const array: usize = 0;
    pub const packed: usize = 0;
}
pub mod GfxComputeShaderLoadDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const program: usize = 0;
    pub const programSize: usize = 8;
    pub const __pad: usize = 12;
}
pub mod GfxDepthAndSurf {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const depthSort: usize = 0;
    pub const surfIndex: usize = 4;
}
pub mod GfxDomainShaderLoadDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const program: usize = 0;
    pub const programSize: usize = 8;
    pub const __pad: usize = 12;
}
pub mod GfxDrawSurf {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const fields: usize = 0;
    pub const fields__objectId: usize = 0;
    pub const fields__pad0: usize = 2;
    pub const fields__reflectionProbeIndex: usize = 4;
    pub const fields__hasGfxEntIndex: usize = 5;
    pub const fields__customIndex: usize = 5;
    pub const fields__materialSortedIndex: usize = 6;
    pub const fields__tessellation: usize = 8;
    pub const fields__prepass: usize = 8;
    pub const fields__pad1: usize = 8;
    pub const fields__useHeroLighting: usize = 9;
    pub const fields__sceneLightEnvIndex: usize = 9;
    pub const fields__viewModelRender: usize = 11;
    pub const fields__surfType: usize = 11;
    pub const fields__primarySortKey: usize = 11;
    pub const fields__unused: usize = 12;
    pub const packed: usize = 0;
    pub const packed__p0: usize = 0;
    pub const packed__p1: usize = 8;
}
pub mod GfxDrawSurfFields {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const objectId: usize = 0;
    pub const pad0: usize = 2;
    pub const reflectionProbeIndex: usize = 4;
    pub const hasGfxEntIndex: usize = 5;
    pub const customIndex: usize = 5;
    pub const materialSortedIndex: usize = 6;
    pub const tessellation: usize = 8;
    pub const prepass: usize = 8;
    pub const pad1: usize = 8;
    pub const useHeroLighting: usize = 9;
    pub const sceneLightEnvIndex: usize = 9;
    pub const viewModelRender: usize = 11;
    pub const surfType: usize = 11;
    pub const primarySortKey: usize = 11;
    pub const unused: usize = 12;
}
pub mod GfxHeroOnlyLight {
    pub const SIZE: usize = 68;
    pub const ALIGN: usize = 4;
    pub const r#type: usize = 0;
    pub const needsDynamicShadows: usize = 1;
    pub const unused: usize = 2;
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
    pub const __pad: usize = 12;
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
    pub const cardMemory: usize = 34;
    pub const cardMemory__platform: usize = 34;
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
    pub const SIZE: usize = 1080;
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
    pub const cachedSkyLightGridEntry: usize = 88;
    pub const cachedSkyLightGridEntry__colorsIndex: usize = 88;
    pub const cachedSkyLightGridEntry__primaryLightEnvIndex: usize = 92;
    pub const cachedSkyLightGridEntry__unused: usize = 94;
    pub const cachedSkyLightGridEntry__needsTrace: usize = 95;
    pub const cachedSkyLightEntryAverageColor: usize = 96;
    pub const missingGridColorIndex: usize = 108;
    pub const tableVersion: usize = 112;
    pub const paletteVersion: usize = 116;
    pub const rangeExponent8BitsEncoding: usize = 120;
    pub const rangeExponent12BitsEncoding: usize = 121;
    pub const rangeExponent16BitsEncoding: usize = 122;
    pub const stageCount: usize = 123;
    pub const stageLightingContrastGain: usize = 128;
    pub const paletteEntryCount: usize = 136;
    pub const paletteEntryAddress: usize = 144;
    pub const paletteBitstreamSize: usize = 152;
    pub const paletteBitstream: usize = 160;
    pub const skyLightGridColors: usize = 168;
    pub const skyLightGridColors__rgb: usize = 168;
    pub const defaultLightGridColors: usize = 504;
    pub const defaultLightGridColors__rgb: usize = 504;
    pub const tree: usize = 840;
}
pub mod GfxLightGridColors {
    pub const SIZE: usize = 336;
    pub const ALIGN: usize = 2;
    pub const rgb: usize = 0;
}
pub mod GfxLightGridColorsHDR {
    pub const SIZE: usize = 336;
    pub const ALIGN: usize = 2;
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
    pub const lightingState: usize = 0;
    pub const index: usize = 0;
    pub const maxDepth: usize = 1;
    pub const pad: usize = 2;
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
pub mod GfxLightmapParameters {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 4;
    pub const lightmapWidthPrimary: usize = 0;
    pub const lightmapHeightPrimary: usize = 4;
    pub const lightmapWidthSecondary: usize = 8;
    pub const lightmapHeightSecondary: usize = 12;
    pub const lightmapModelUnitsPerTexel: usize = 16;
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
pub mod GfxPortalGroup {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const targetName: usize = 0;
    pub const gfxPortalArray: usize = 8;
    pub const targetNameIndex: usize = 16;
    pub const numPortals: usize = 20;
}
pub mod GfxPortalGroupInfo {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 2;
    pub const cellIndex: usize = 0;
    pub const portalIndex: usize = 2;
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
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const linemap: usize = 0;
    pub const map: usize = 0;
    pub const volmap: usize = 0;
    pub const cubemap: usize = 0;
    pub const loadDef: usize = 0;
    pub const shaderView: usize = 8;
    pub const shaderViewAlternate: usize = 16;
    pub const __pad0: usize = 24;
}
pub mod GfxReflectionProbe {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const origin: usize = 0;
    pub const probeVolumes: usize = 16;
    pub const probeVolumeCount: usize = 24;
    pub const lightingState: usize = 28;
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
pub mod GfxReflectionProbeVolume {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const probeVolumePlanes: usize = 0;
    pub const probeVolumePlaneCount: usize = 8;
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
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const skySurfCount: usize = 0;
    pub const skyStartSurfs: usize = 8;
    pub const skyImage: usize = 16;
    pub const skySamplerState: usize = 24;
    pub const bounds: usize = 28;
    pub const bounds__midPoint: usize = 28;
    pub const bounds__halfSize: usize = 40;
}
pub mod GfxStateBits {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 4;
    pub const loadBits: usize = 0;
    pub const zone: usize = 24;
    pub const depthStencilState: usize = 26;
    pub const blendState: usize = 36;
    pub const rasterizerState: usize = 37;
}
pub mod GfxStaticModelAmbientLightingInfo {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const groundLighting: usize = 0;
    pub const groundLighting__array: usize = 0;
    pub const groundLighting__packed: usize = 0;
    pub const colorIndex: usize = 8;
    pub const primaryLightWeight: usize = 12;
}
pub mod GfxStaticModelDrawInst {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 8;
    pub const placement: usize = 0;
    pub const placement__origin: usize = 0;
    pub const placement__axis: usize = 12;
    pub const placement__scale: usize = 48;
    pub const model: usize = 56;
    pub const cullDist: usize = 64;
    pub const flags: usize = 66;
    pub const lightingHandle: usize = 68;
    pub const staticModelId: usize = 70;
    pub const primaryLightEnvIndex: usize = 72;
    pub const reactiveMotionCullDist: usize = 74;
    pub const reactiveMotionLOD: usize = 76;
    pub const reflectionProbeIndex: usize = 77;
    pub const firstMtlSkinIndex: usize = 78;
    pub const sunShadowFlags: usize = 79;
}
pub mod GfxStaticModelInst {
    pub const SIZE: usize = 36;
    pub const ALIGN: usize = 4;
    pub const mins: usize = 0;
    pub const maxs: usize = 12;
    pub const lightingOrigin: usize = 24;
}
pub mod GfxStaticModelLighting {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const ambientLightingInfo: usize = 0;
    pub const ambientLightingInfo__groundLighting: usize = 0;
    pub const ambientLightingInfo__groundLighting__array: usize = 0;
    pub const ambientLightingInfo__groundLighting__packed: usize = 0;
    pub const ambientLightingInfo__colorIndex: usize = 8;
    pub const ambientLightingInfo__primaryLightWeight: usize = 12;
    pub const vertexLightingInfo: usize = 0;
    pub const vertexLightingInfo__lightingValues: usize = 0;
    pub const vertexLightingInfo__lightingValuesVb: usize = 8;
    pub const vertexLightingInfo__numLightingValues: usize = 16;
    pub const modelLightmapInfo: usize = 0;
    pub const modelLightmapInfo__offset: usize = 0;
    pub const modelLightmapInfo__scale: usize = 8;
    pub const modelLightmapInfo__lightmapIndex: usize = 16;
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
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const lightingValues: usize = 0;
    pub const lightingValuesVb: usize = 8;
    pub const numLightingValues: usize = 16;
}
pub mod GfxSubdivCache {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const size: usize = 0;
    pub const subdivCacheBuffer: usize = 8;
    pub const subdivCacheView: usize = 16;
}
pub mod GfxSubdivVertexLightingInfo {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const vertexLightingIndex: usize = 0;
    pub const vb: usize = 8;
    pub const cache: usize = 16;
    pub const cache__size: usize = 16;
    pub const cache__subdivCacheBuffer: usize = 24;
    pub const cache__subdivCacheView: usize = 32;
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
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const bounds: usize = 0;
    pub const bounds__midPoint: usize = 0;
    pub const bounds__halfSize: usize = 12;
    pub const mipRadius: usize = 24;
    pub const invHighMipRadius: usize = 26;
    pub const flags: usize = 31;
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
pub mod GfxWorld {
    pub const SIZE: usize = 2800;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const baseName: usize = 8;
    pub const bspVersion: usize = 16;
    pub const planeCount: usize = 20;
    pub const nodeCount: usize = 24;
    pub const surfaceCount: usize = 28;
    pub const skyCount: usize = 32;
    pub const skies: usize = 40;
    pub const portalGroupCount: usize = 48;
    pub const lastSunPrimaryLightIndex: usize = 52;
    pub const primaryLightCount: usize = 56;
    pub const primaryLightEnvCount: usize = 60;
    pub const sortKeyLitDecal: usize = 64;
    pub const sortKeyEffectDecal: usize = 68;
    pub const sortKeyTopDecal: usize = 72;
    pub const sortKeyEffectAuto: usize = 76;
    pub const sortKeyDistortion: usize = 80;
    pub const sortKeyHair: usize = 84;
    pub const sortKeyEffectBlend: usize = 88;
    pub const __pad0: usize = 92;
    pub const dpvsPlanes: usize = 96;
    pub const dpvsPlanes__cellCount: usize = 96;
    pub const dpvsPlanes__planes: usize = 104;
    pub const dpvsPlanes__nodes: usize = 112;
    pub const dpvsPlanes__sceneEntCellBits: usize = 120;
    pub const aabbTreeCounts: usize = 128;
    pub const aabbTrees: usize = 136;
    pub const cells: usize = 144;
    pub const portalGroup: usize = 152;
    pub const portalDistanceAnchorCount: usize = 160;
    pub const __pad1: usize = 164;
    pub const portalDistanceAnchorsAndCloseDistSquared: usize = 168;
    pub const draw: usize = 176;
    pub const draw__reflectionProbeCount: usize = 176;
    pub const draw__reflectionProbes: usize = 184;
    pub const draw__reflectionProbeOrigins: usize = 192;
    pub const draw__reflectionProbeTextures: usize = 200;
    pub const draw__reflectionProbeReferenceCount: usize = 208;
    pub const draw__reflectionProbeReferenceOrigins: usize = 216;
    pub const draw__reflectionProbeReferences: usize = 224;
    pub const draw__lightmapCount: usize = 232;
    pub const draw__lightmaps: usize = 240;
    pub const draw__lightmapPrimaryTextures: usize = 248;
    pub const draw__lightmapSecondaryTextures: usize = 256;
    pub const draw__lightmapOverridePrimary: usize = 264;
    pub const draw__lightmapOverrideSecondary: usize = 272;
    pub const draw__lightmapParameters: usize = 280;
    pub const draw__lightmapParameters__lightmapWidthPrimary: usize = 280;
    pub const draw__lightmapParameters__lightmapHeightPrimary: usize = 284;
    pub const draw__lightmapParameters__lightmapWidthSecondary: usize = 288;
    pub const draw__lightmapParameters__lightmapHeightSecondary: usize = 292;
    pub const draw__lightmapParameters__lightmapModelUnitsPerTexel: usize = 296;
    pub const draw__trisType: usize = 300;
    pub const draw__vertexCount: usize = 304;
    pub const draw__vd: usize = 312;
    pub const draw__vd__vertices: usize = 312;
    pub const draw__vd__worldVb: usize = 320;
    pub const draw__vertexLayerDataSize: usize = 328;
    pub const draw__vld: usize = 336;
    pub const draw__vld__data: usize = 336;
    pub const draw__vld__layerVb: usize = 344;
    pub const draw__indexCount: usize = 352;
    pub const draw__indices: usize = 360;
    pub const draw__indexBuffer: usize = 368;
    pub const lightGrid: usize = 376;
    pub const lightGrid__hasLightRegions: usize = 376;
    pub const lightGrid__useSkyForLowZ: usize = 377;
    pub const lightGrid__lastSunPrimaryLightIndex: usize = 380;
    pub const lightGrid__mins: usize = 384;
    pub const lightGrid__maxs: usize = 390;
    pub const lightGrid__rowAxis: usize = 396;
    pub const lightGrid__colAxis: usize = 400;
    pub const lightGrid__rowDataStart: usize = 408;
    pub const lightGrid__rawRowDataSize: usize = 416;
    pub const lightGrid__rawRowData: usize = 424;
    pub const lightGrid__entryCount: usize = 432;
    pub const lightGrid__entries: usize = 440;
    pub const lightGrid__colorCount: usize = 448;
    pub const lightGrid__colors: usize = 456;
    pub const lightGrid__cachedSkyLightGridEntry: usize = 464;
    pub const lightGrid__cachedSkyLightGridEntry__colorsIndex: usize = 464;
    pub const lightGrid__cachedSkyLightGridEntry__primaryLightEnvIndex: usize = 468;
    pub const lightGrid__cachedSkyLightGridEntry__unused: usize = 470;
    pub const lightGrid__cachedSkyLightGridEntry__needsTrace: usize = 471;
    pub const lightGrid__cachedSkyLightEntryAverageColor: usize = 472;
    pub const lightGrid__missingGridColorIndex: usize = 484;
    pub const lightGrid__tableVersion: usize = 488;
    pub const lightGrid__paletteVersion: usize = 492;
    pub const lightGrid__rangeExponent8BitsEncoding: usize = 496;
    pub const lightGrid__rangeExponent12BitsEncoding: usize = 497;
    pub const lightGrid__rangeExponent16BitsEncoding: usize = 498;
    pub const lightGrid__stageCount: usize = 499;
    pub const lightGrid__stageLightingContrastGain: usize = 504;
    pub const lightGrid__paletteEntryCount: usize = 512;
    pub const lightGrid__paletteEntryAddress: usize = 520;
    pub const lightGrid__paletteBitstreamSize: usize = 528;
    pub const lightGrid__paletteBitstream: usize = 536;
    pub const lightGrid__skyLightGridColors: usize = 544;
    pub const lightGrid__skyLightGridColors__rgb: usize = 544;
    pub const lightGrid__defaultLightGridColors: usize = 880;
    pub const lightGrid__defaultLightGridColors__rgb: usize = 880;
    pub const lightGrid__tree: usize = 1216;
    pub const modelCount: usize = 1456;
    pub const models: usize = 1464;
    pub const bounds: usize = 1472;
    pub const bounds__midPoint: usize = 1472;
    pub const bounds__halfSize: usize = 1484;
    pub const shadowBounds: usize = 1496;
    pub const shadowBounds__midPoint: usize = 1496;
    pub const shadowBounds__halfSize: usize = 1508;
    pub const checksum: usize = 1520;
    pub const materialMemoryCount: usize = 1524;
    pub const materialMemory: usize = 1528;
    pub const sun: usize = 1536;
    pub const sun__hasValidData: usize = 1536;
    pub const sun__spriteMaterial: usize = 1544;
    pub const sun__flareMaterial: usize = 1552;
    pub const sun__spriteSize: usize = 1560;
    pub const sun__flareMinSize: usize = 1564;
    pub const sun__flareMinDot: usize = 1568;
    pub const sun__flareMaxSize: usize = 1572;
    pub const sun__flareMaxDot: usize = 1576;
    pub const sun__flareMaxAlpha: usize = 1580;
    pub const sun__flareFadeInTime: usize = 1584;
    pub const sun__flareFadeOutTime: usize = 1588;
    pub const sun__blindMinDot: usize = 1592;
    pub const sun__blindMaxDot: usize = 1596;
    pub const sun__blindMaxDarken: usize = 1600;
    pub const sun__blindFadeInTime: usize = 1604;
    pub const sun__blindFadeOutTime: usize = 1608;
    pub const sun__glareMinDot: usize = 1612;
    pub const sun__glareMaxDot: usize = 1616;
    pub const sun__glareMaxLighten: usize = 1620;
    pub const sun__glareFadeInTime: usize = 1624;
    pub const sun__glareFadeOutTime: usize = 1628;
    pub const sun__sunFxPosition: usize = 1632;
    pub const outdoorLookupMatrix: usize = 1648;
    pub const outdoorImage: usize = 1712;
    pub const cellCasterBits: usize = 1720;
    pub const cellHasSunLitSurfsBits: usize = 1728;
    pub const sceneDynModel: usize = 1736;
    pub const sceneDynBrush: usize = 1744;
    pub const primaryLightEntityShadowVis: usize = 1752;
    pub const primaryLightDynEntShadowVis: usize = 1760;
    pub const nonSunPrimaryLightForModelDynEnt: usize = 1776;
    pub const shadowGeom: usize = 1784;
    pub const shadowGeomOptimized: usize = 1792;
    pub const lightRegion: usize = 1800;
    pub const dpvs: usize = 1808;
    pub const dpvs__smodelCount: usize = 1808;
    pub const dpvs__subdivVertexLightingInfoCount: usize = 1812;
    pub const dpvs__staticSurfaceCount: usize = 1816;
    pub const dpvs__litOpaqueSurfsBegin: usize = 1820;
    pub const dpvs__litOpaqueSurfsEnd: usize = 1824;
    pub const dpvs__unkSurfsBegin: usize = 1828;
    pub const dpvs__unkSurfsEnd: usize = 1832;
    pub const dpvs__litDecalSurfsBegin: usize = 1836;
    pub const dpvs__litDecalSurfsEnd: usize = 1840;
    pub const dpvs__litTransSurfsBegin: usize = 1844;
    pub const dpvs__litTransSurfsEnd: usize = 1848;
    pub const dpvs__shadowCasterSurfsBegin: usize = 1852;
    pub const dpvs__shadowCasterSurfsEnd: usize = 1856;
    pub const dpvs__emissiveSurfsBegin: usize = 1860;
    pub const dpvs__emissiveSurfsEnd: usize = 1864;
    pub const dpvs__smodelVisDataCount: usize = 1868;
    pub const dpvs__surfaceVisDataCount: usize = 1872;
    pub const dpvs____pad0: usize = 1876;
    pub const dpvs__smodelVisData: usize = 1880;
    pub const dpvs__smodelUnknownVisData: usize = 1912;
    pub const dpvs__surfaceVisData: usize = 2128;
    pub const dpvs__surfaceUnknownVisData: usize = 2160;
    pub const dpvs__smodelUmbraVisData: usize = 2376;
    pub const dpvs__surfaceUmbraVisData: usize = 2408;
    pub const dpvs__lodData: usize = 2440;
    pub const dpvs__sortedSurfIndex: usize = 2448;
    pub const dpvs__smodelInsts: usize = 2456;
    pub const dpvs__surfaces: usize = 2464;
    pub const dpvs__surfacesBounds: usize = 2472;
    pub const dpvs__smodelDrawInsts: usize = 2480;
    pub const dpvs__unknownSModelVisData1: usize = 2488;
    pub const dpvs__unknownSModelVisData2: usize = 2496;
    pub const dpvs__smodelLightingInsts: usize = 2504;
    pub const dpvs__subdivVertexLighting: usize = 2512;
    pub const dpvs__surfaceMaterials: usize = 2520;
    pub const dpvs__surfaceCastsSunShadow: usize = 2528;
    pub const dpvs__sunShadowOptCount: usize = 2536;
    pub const dpvs__sunSurfVisDataCount: usize = 2540;
    pub const dpvs__surfaceCastsSunShadowOpt: usize = 2544;
    pub const dpvs__surfaceDeptAndSurf: usize = 2552;
    pub const dpvs__constantBuffersLit: usize = 2560;
    pub const dpvs__constantBuffersAmbient: usize = 2568;
    pub const dpvs__usageCount: usize = 2576;
    pub const dpvs____pad1: usize = 2580;
    pub const dpvsDyn: usize = 2584;
    pub const dpvsDyn__dynEntClientWordCount: usize = 2584;
    pub const dpvsDyn__dynEntClientCount: usize = 2592;
    pub const dpvsDyn__dynEntCellBits: usize = 2600;
    pub const dpvsDyn__dynEntVisData: usize = 2616;
    pub const mapVtxChecksum: usize = 2680;
    pub const heroOnlyLightCount: usize = 2684;
    pub const heroOnlyLights: usize = 2688;
    pub const fogTypesAllowed: usize = 2696;
    pub const umbraTomeSize: usize = 2700;
    pub const umbraTomeData: usize = 2704;
    pub const umbraTomePtr: usize = 2712;
    pub const mdaoVolumesCount: usize = 2720;
    pub const mdaoVolumes: usize = 2728;
    pub const useLightGridDefaultModelLightingLookup: usize = 2736;
    pub const useLightGridDefaultFXLightingLookup: usize = 2737;
    pub const lightGridDefaultModelLightingLookup: usize = 2740;
    pub const lightGridDefaultFXLightingLookup: usize = 2752;
    pub const buildInfo: usize = 2768;
    pub const buildInfo__bspCommandline: usize = 2768;
    pub const buildInfo__lightCommandline: usize = 2776;
    pub const buildInfo__bspTimestamp: usize = 2784;
    pub const buildInfo__lightTimestamp: usize = 2792;
}
pub mod GfxWorldDpvsDynamic {
    pub const SIZE: usize = 96;
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
    pub const SIZE: usize = 776;
    pub const ALIGN: usize = 8;
    pub const smodelCount: usize = 0;
    pub const subdivVertexLightingInfoCount: usize = 4;
    pub const staticSurfaceCount: usize = 8;
    pub const litOpaqueSurfsBegin: usize = 12;
    pub const litOpaqueSurfsEnd: usize = 16;
    pub const unkSurfsBegin: usize = 20;
    pub const unkSurfsEnd: usize = 24;
    pub const litDecalSurfsBegin: usize = 28;
    pub const litDecalSurfsEnd: usize = 32;
    pub const litTransSurfsBegin: usize = 36;
    pub const litTransSurfsEnd: usize = 40;
    pub const shadowCasterSurfsBegin: usize = 44;
    pub const shadowCasterSurfsEnd: usize = 48;
    pub const emissiveSurfsBegin: usize = 52;
    pub const emissiveSurfsEnd: usize = 56;
    pub const smodelVisDataCount: usize = 60;
    pub const surfaceVisDataCount: usize = 64;
    pub const __pad0: usize = 68;
    pub const smodelVisData: usize = 72;
    pub const smodelUnknownVisData: usize = 104;
    pub const surfaceVisData: usize = 320;
    pub const surfaceUnknownVisData: usize = 352;
    pub const smodelUmbraVisData: usize = 568;
    pub const surfaceUmbraVisData: usize = 600;
    pub const lodData: usize = 632;
    pub const sortedSurfIndex: usize = 640;
    pub const smodelInsts: usize = 648;
    pub const surfaces: usize = 656;
    pub const surfacesBounds: usize = 664;
    pub const smodelDrawInsts: usize = 672;
    pub const unknownSModelVisData1: usize = 680;
    pub const unknownSModelVisData2: usize = 688;
    pub const smodelLightingInsts: usize = 696;
    pub const subdivVertexLighting: usize = 704;
    pub const surfaceMaterials: usize = 712;
    pub const surfaceCastsSunShadow: usize = 720;
    pub const sunShadowOptCount: usize = 728;
    pub const sunSurfVisDataCount: usize = 732;
    pub const surfaceCastsSunShadowOpt: usize = 736;
    pub const surfaceDeptAndSurf: usize = 744;
    pub const constantBuffersLit: usize = 752;
    pub const constantBuffersAmbient: usize = 760;
    pub const usageCount: usize = 768;
    pub const __pad1: usize = 772;
}
pub mod GfxWorldDraw {
    pub const SIZE: usize = 200;
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
    pub const lightmapParameters: usize = 104;
    pub const lightmapParameters__lightmapWidthPrimary: usize = 104;
    pub const lightmapParameters__lightmapHeightPrimary: usize = 108;
    pub const lightmapParameters__lightmapWidthSecondary: usize = 112;
    pub const lightmapParameters__lightmapHeightSecondary: usize = 116;
    pub const lightmapParameters__lightmapModelUnitsPerTexel: usize = 120;
    pub const trisType: usize = 124;
    pub const vertexCount: usize = 128;
    pub const vd: usize = 136;
    pub const vd__vertices: usize = 136;
    pub const vd__worldVb: usize = 144;
    pub const vertexLayerDataSize: usize = 152;
    pub const vld: usize = 160;
    pub const vld__data: usize = 160;
    pub const vld__layerVb: usize = 168;
    pub const indexCount: usize = 176;
    pub const indices: usize = 184;
    pub const indexBuffer: usize = 192;
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
pub mod LaserDef {
    pub const SIZE: usize = 152;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const laserMaterial: usize = 8;
    pub const laserLightMaterial: usize = 16;
    pub const laserEndEffect: usize = 24;
    pub const friendlyTeamLaser: usize = 32;
    pub const laserTag: usize = 40;
    pub const color: usize = 44;
    pub const hdrColorScale: usize = 60;
    pub const laserLightColor: usize = 76;
    pub const laserLightHdrColorScale: usize = 92;
    pub const range: usize = 108;
    pub const radius: usize = 112;
    pub const endOffset: usize = 116;
    pub const flarePct: usize = 120;
    pub const texCoordOffset: usize = 124;
    pub const laserLightRadius: usize = 128;
    pub const laserLightBeginOffset: usize = 132;
    pub const laserLightEndOffset: usize = 136;
    pub const laserLightBodyTweak: usize = 140;
    pub const ownerOnly: usize = 144;
    pub const nightvisionOnly: usize = 145;
    pub const useHalfCylinderGeometry: usize = 146;
    pub const laserLight: usize = 147;
    pub const laserLightNvgOnly: usize = 148;
    pub const laserSightLaser: usize = 149;
}
pub mod LoadedSound {
    pub const SIZE: usize = 64;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const filename: usize = 8;
    pub const filename__isLocalized: usize = 8;
    pub const filename__isStreamed: usize = 9;
    pub const filename__fileIndex: usize = 10;
    pub const filename__info: usize = 16;
    pub const filename__info__raw: usize = 16;
    pub const filename__info__raw__dir: usize = 16;
    pub const filename__info__raw__name: usize = 24;
    pub const filename__info__packed: usize = 16;
    pub const filename__info__packed__offset: usize = 16;
    pub const filename__info__packed__length: usize = 24;
    pub const info: usize = 32;
    pub const info__data: usize = 32;
    pub const info__sampleRate: usize = 40;
    pub const info__dataByteCount: usize = 44;
    pub const info__numSamples: usize = 48;
    pub const info__channels: usize = 52;
    pub const info__numBits: usize = 53;
    pub const info__blockAlign: usize = 54;
    pub const info__format: usize = 56;
    pub const info__loadedSize: usize = 60;
}
pub mod LoadedSoundInfo {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const data: usize = 0;
    pub const sampleRate: usize = 8;
    pub const dataByteCount: usize = 12;
    pub const numSamples: usize = 16;
    pub const channels: usize = 20;
    pub const numBits: usize = 21;
    pub const blockAlign: usize = 22;
    pub const format: usize = 24;
    pub const loadedSize: usize = 28;
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
    pub const SIZE: usize = 288;
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
    pub const clientTrigger__lightSetTriggers: usize = 160;
    pub const clientTrigger__clutTriggers: usize = 168;
    pub const clientTrigger__triggerType: usize = 176;
    pub const clientTrigger__origins: usize = 184;
    pub const clientTrigger__scriptDelay: usize = 192;
    pub const clientTrigger__audioTriggers: usize = 200;
    pub const clientTrigger__blendLookup: usize = 208;
    pub const clientTrigger__npcTriggers: usize = 216;
    pub const clientTrigger__contextTriggers: usize = 224;
    pub const clientTrigger__waterTriggers: usize = 232;
    pub const clientTriggerBlend: usize = 240;
    pub const clientTriggerBlend__numClientTriggerBlendNodes: usize = 240;
    pub const clientTriggerBlend__blendNodes: usize = 248;
    pub const spawnList: usize = 256;
    pub const spawnList__spawnsCount: usize = 256;
    pub const spawnList__spawns: usize = 264;
    pub const splineList: usize = 272;
    pub const splineList__splineCount: usize = 272;
    pub const splineList__splines: usize = 280;
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
    pub const SIZE: usize = 480;
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
    pub const info__drawSurf__fields__pad0: usize = 18;
    pub const info__drawSurf__fields__reflectionProbeIndex: usize = 20;
    pub const info__drawSurf__fields__hasGfxEntIndex: usize = 21;
    pub const info__drawSurf__fields__customIndex: usize = 21;
    pub const info__drawSurf__fields__materialSortedIndex: usize = 22;
    pub const info__drawSurf__fields__tessellation: usize = 24;
    pub const info__drawSurf__fields__prepass: usize = 24;
    pub const info__drawSurf__fields__pad1: usize = 24;
    pub const info__drawSurf__fields__useHeroLighting: usize = 25;
    pub const info__drawSurf__fields__sceneLightEnvIndex: usize = 25;
    pub const info__drawSurf__fields__viewModelRender: usize = 27;
    pub const info__drawSurf__fields__surfType: usize = 27;
    pub const info__drawSurf__fields__primarySortKey: usize = 27;
    pub const info__drawSurf__fields__unused: usize = 28;
    pub const info__drawSurf__packed: usize = 16;
    pub const info__drawSurf__packed__p0: usize = 16;
    pub const info__drawSurf__packed__p1: usize = 24;
    pub const info__surfaceTypeBits: usize = 32;
    pub const info__hashIndex: usize = 40;
    pub const stateBitsEntry: usize = 48;
    pub const textureCount: usize = 228;
    pub const constantCount: usize = 229;
    pub const stateBitsCount: usize = 230;
    pub const stateFlags: usize = 231;
    pub const cameraRegion: usize = 232;
    pub const materialType: usize = 233;
    pub const layerCount: usize = 234;
    pub const assetFlags: usize = 235;
    pub const techniqueSet: usize = 240;
    pub const textureTable: usize = 248;
    pub const constantTable: usize = 256;
    pub const stateBitsTable: usize = 264;
    pub const constantBufferIndex: usize = 272;
    pub const constantBufferTable: usize = 456;
    pub const constantBufferCount: usize = 464;
    pub const subMaterials: usize = 472;
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
    pub const prog__loadDef____pad: usize = 28;
}
pub mod MaterialDomainShaderProgram {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const ds: usize = 0;
    pub const loadDef: usize = 8;
    pub const loadDef__program: usize = 8;
    pub const loadDef__programSize: usize = 16;
    pub const loadDef____pad: usize = 20;
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
    pub const prog__loadDef____pad: usize = 28;
}
pub mod MaterialHullShaderProgram {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const hs: usize = 0;
    pub const loadDef: usize = 8;
    pub const loadDef__program: usize = 8;
    pub const loadDef__programSize: usize = 16;
    pub const loadDef____pad: usize = 20;
}
pub mod MaterialInfo {
    pub const SIZE: usize = 48;
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
    pub const drawSurf__fields__pad0: usize = 18;
    pub const drawSurf__fields__reflectionProbeIndex: usize = 20;
    pub const drawSurf__fields__hasGfxEntIndex: usize = 21;
    pub const drawSurf__fields__customIndex: usize = 21;
    pub const drawSurf__fields__materialSortedIndex: usize = 22;
    pub const drawSurf__fields__tessellation: usize = 24;
    pub const drawSurf__fields__prepass: usize = 24;
    pub const drawSurf__fields__pad1: usize = 24;
    pub const drawSurf__fields__useHeroLighting: usize = 25;
    pub const drawSurf__fields__sceneLightEnvIndex: usize = 25;
    pub const drawSurf__fields__viewModelRender: usize = 27;
    pub const drawSurf__fields__surfType: usize = 27;
    pub const drawSurf__fields__primarySortKey: usize = 27;
    pub const drawSurf__fields__unused: usize = 28;
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
    pub const perPrimConstantBuffer: usize = 52;
    pub const perObjConstantBuffer: usize = 53;
    pub const stableConstantBuffer: usize = 54;
    pub const customBufferFlags: usize = 56;
    pub const customSamplerFlags: usize = 60;
    pub const precompiledIndex: usize = 61;
    pub const stageConfig: usize = 62;
    pub const args: usize = 64;
}
pub mod MaterialPixelShader {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const prog: usize = 8;
    pub const prog__ps: usize = 8;
    pub const prog__loadDef: usize = 16;
    pub const prog__loadDef__program: usize = 16;
    pub const prog__loadDef__programSize: usize = 24;
    pub const prog__loadDef__microCodeCrc: usize = 28;
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
    pub const SIZE: usize = 1456;
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
    pub const SIZE: usize = 352;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const streamCount: usize = 8;
    pub const hasOptionalSource: usize = 9;
    pub const routing: usize = 16;
    pub const routing__data: usize = 16;
    pub const routing__decl: usize = 112;
}
pub mod MaterialVertexShader {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const prog: usize = 8;
    pub const prog__vs: usize = 8;
    pub const prog__loadDef: usize = 16;
    pub const prog__loadDef__program: usize = 16;
    pub const prog__loadDef__programSize: usize = 24;
    pub const prog__loadDef__microCodeCrc: usize = 28;
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
    pub const SIZE: usize = 336;
    pub const ALIGN: usize = 8;
    pub const data: usize = 0;
    pub const decl: usize = 96;
}
pub mod MdaoVolume {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const bounds: usize = 0;
    pub const bounds__mins: usize = 0;
    pub const bounds__maxs: usize = 12;
    pub const cellCount: usize = 24;
    pub const parentBoneIndex: usize = 30;
    pub const volumeData: usize = 32;
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
pub mod PatchesCollisionData {
    pub const SIZE: usize = 72;
    pub const ALIGN: usize = 8;
    pub const vertCount: usize = 0;
    pub const verts: usize = 8;
    pub const triCount: usize = 16;
    pub const triIndices: usize = 24;
    pub const triEdgeIsWalkable: usize = 32;
    pub const borderCount: usize = 40;
    pub const borders: usize = 48;
    pub const partitionCount: usize = 56;
    pub const partitions: usize = 64;
}
pub mod PatchesCollisionTree {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const aabbTreeCount: usize = 0;
    pub const aabbTrees: usize = 8;
}
pub mod PathData {
    pub const SIZE: usize = 408;
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
    pub const visBulletBlockerBytes: usize = 104;
    pub const pathVisBulletBlockers: usize = 112;
    pub const zoneCount: usize = 120;
    pub const zonesBytes: usize = 124;
    pub const pathZones: usize = 128;
    pub const dynStatesBytes: usize = 136;
    pub const pathDynStates: usize = 144;
    pub const pathAbilities: usize = 152;
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
pub mod PhysBrushModel {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const packed: usize = 0;
    pub const packed__p: usize = 0;
    pub const fields: usize = 0;
    pub const fields__polytopeIndex: usize = 0;
    pub const fields__unk: usize = 2;
    pub const fields__worldIndex: usize = 4;
    pub const fields__meshIndex: usize = 6;
}
pub mod PhysBrushModelFields {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 2;
    pub const polytopeIndex: usize = 0;
    pub const unk: usize = 2;
    pub const worldIndex: usize = 4;
    pub const meshIndex: usize = 6;
}
pub mod PhysBrushModelPacked {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const p: usize = 0;
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
pub mod PhysConstraint {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const __pad0: usize = 8;
}
pub mod PhysGeomInfo {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const data: usize = 0;
}
pub mod PhysMass {
    pub const SIZE: usize = 36;
    pub const ALIGN: usize = 4;
    pub const centerOfMass: usize = 0;
    pub const momentsOfInertia: usize = 12;
    pub const productsOfInertia: usize = 24;
}
pub mod PhysPreset {
    pub const SIZE: usize = 96;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const r#type: usize = 8;
    pub const mass: usize = 12;
    pub const bounce: usize = 16;
    pub const friction: usize = 20;
    pub const bulletForceScale: usize = 24;
    pub const explosiveForceScale: usize = 28;
    pub const explosiveSpinScale: usize = 32;
    pub const unk2: usize = 36;
    pub const sndAliasPrefix: usize = 40;
    pub const piecesSpreadFraction: usize = 48;
    pub const piecesUpwardVelocity: usize = 52;
    pub const minMomentum: usize = 56;
    pub const maxMomentum: usize = 60;
    pub const minVolume: usize = 64;
    pub const maxVolume: usize = 68;
    pub const minPitch: usize = 72;
    pub const maxPitch: usize = 76;
    pub const volumeType: usize = 80;
    pub const pitchType: usize = 84;
    pub const tempDefaultToCylinder: usize = 88;
    pub const perSurfaceSndAlias: usize = 89;
}
pub mod PhysWaterPreset {
    pub const SIZE: usize = 128;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const __pad0: usize = 8;
    pub const fx0: usize = 72;
    pub const fx1: usize = 80;
    pub const fx2: usize = 88;
    pub const fx3: usize = 96;
    pub const fx4: usize = 104;
    pub const fx5: usize = 112;
    pub const fx6: usize = 120;
}
pub mod PhysWaterVolumeDef {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const physWaterPreset: usize = 0;
    pub const __pad0: usize = 8;
    pub const string: usize = 20;
    pub const brushModelIndex: usize = 24;
    pub const __pad1: usize = 26;
}
pub mod PhysWorld {
    pub const SIZE: usize = 56;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const brushModels: usize = 8;
    pub const polytopeDatas: usize = 16;
    pub const meshDatas: usize = 24;
    pub const waterVolumeDefs: usize = 32;
    pub const brushModelCount: usize = 40;
    pub const polytopeCount: usize = 44;
    pub const meshDataCount: usize = 48;
    pub const waterVolumeDefCount: usize = 52;
}
pub mod PicMip {
    pub const SIZE: usize = 2;
    pub const ALIGN: usize = 1;
    pub const platform: usize = 0;
}
pub mod PrimedSound {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const loadedPart: usize = 0;
    pub const streamedPart: usize = 8;
    pub const streamedPart__isLocalized: usize = 8;
    pub const streamedPart__isStreamed: usize = 9;
    pub const streamedPart__fileIndex: usize = 10;
    pub const streamedPart__info: usize = 16;
    pub const streamedPart__info__raw: usize = 16;
    pub const streamedPart__info__raw__dir: usize = 16;
    pub const streamedPart__info__raw__name: usize = 24;
    pub const streamedPart__info__packed: usize = 16;
    pub const streamedPart__info__packed__offset: usize = 16;
    pub const streamedPart__info__packed__length: usize = 24;
    pub const dataOffset: usize = 32;
    pub const totalSize: usize = 36;
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
pub mod ReloadStateTimerEntry {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const attachment: usize = 0;
    pub const reloadAddTime: usize = 4;
    pub const reloadEmptyAddTime: usize = 8;
    pub const reloadStartAddTime: usize = 12;
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
pub mod SModelsCollisionData {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const numStaticModels: usize = 0;
    pub const staticModelList: usize = 8;
}
pub mod SModelsCollisionTree {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const numStaticModels: usize = 0;
    pub const smodelNodeCount: usize = 2;
    pub const smodelNodes: usize = 8;
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
    pub const setModel: usize = 0;
    pub const setModel__model: usize = 0;
    pub const playFx: usize = 0;
    pub const playFx__handle: usize = 0;
    pub const playFx__tagName: usize = 8;
    pub const playFx__loopTime: usize = 12;
    pub const playFx__loopTimeStreamIndex: usize = 14;
    pub const playFx__tagUseAngles: usize = 15;
    pub const stopFx: usize = 0;
    pub const stopFx__handle: usize = 0;
    pub const stopFx__tagName: usize = 8;
    pub const stopFx__loopTime: usize = 12;
    pub const stopFx__loopTimeStreamIndex: usize = 14;
    pub const stopFx__tagUseAngles: usize = 15;
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
    pub const lightSettings__transitionTimeMin: usize = 16;
    pub const lightSettings__transitionTimeMax: usize = 18;
    pub const lightSettings__noteworthy: usize = 24;
    pub const sunlightSettings: usize = 0;
    pub const sunlightSettings__color: usize = 0;
    pub const sunlightSettings__transStateStreamIndex: usize = 4;
    pub const sunlightSettings__flags: usize = 5;
    pub const sunlightSettings__intensityScaleMin: usize = 8;
    pub const sunlightSettings__intensityScaleMax: usize = 12;
    pub const sunlightSettings__pitchMin: usize = 16;
    pub const sunlightSettings__pitchMax: usize = 18;
    pub const sunlightSettings__headingMin: usize = 20;
    pub const sunlightSettings__headingMax: usize = 22;
    pub const sunlightSettings__transitionTimeMin: usize = 24;
    pub const sunlightSettings__transitionTimeMax: usize = 26;
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
    pub const translate: usize = 0;
    pub const translate__translationMinX: usize = 0;
    pub const translate__translationMinY: usize = 2;
    pub const translate__translationMinZ: usize = 4;
    pub const translate__translationMaxX: usize = 6;
    pub const translate__translationMaxY: usize = 8;
    pub const translate__translationMaxZ: usize = 10;
    pub const translate__transitionTimeMin: usize = 12;
    pub const translate__transitionTimeMax: usize = 14;
    pub const translate__constIndex: usize = 16;
    pub const translate__transStateStreamIndex: usize = 17;
    pub const translate__flags: usize = 18;
    pub const translate__noteworthy: usize = 24;
    pub const rotate: usize = 0;
    pub const rotate__rotationMinPitch: usize = 0;
    pub const rotate__rotationMinYaw: usize = 2;
    pub const rotate__rotationMinRoll: usize = 4;
    pub const rotate__rotationMaxPitch: usize = 6;
    pub const rotate__rotationMaxYaw: usize = 8;
    pub const rotate__rotationMaxRoll: usize = 10;
    pub const rotate__transitionTimeMin: usize = 12;
    pub const rotate__transitionTimeMax: usize = 14;
    pub const rotate__constIndex: usize = 16;
    pub const rotate__transStateStreamIndex: usize = 17;
    pub const rotate__flags: usize = 18;
    pub const rotate__noteworthy: usize = 24;
    pub const stateChange: usize = 0;
    pub const stateChange__targetIndex: usize = 0;
    pub const stateChange__delayStreamIndex: usize = 1;
    pub const stateChange__delayMin: usize = 2;
    pub const stateChange__delayMax: usize = 4;
}
pub mod ScriptableEventDef {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const data: usize = 8;
    pub const data__setModel: usize = 8;
    pub const data__setModel__model: usize = 8;
    pub const data__playFx: usize = 8;
    pub const data__playFx__handle: usize = 8;
    pub const data__playFx__tagName: usize = 16;
    pub const data__playFx__loopTime: usize = 20;
    pub const data__playFx__loopTimeStreamIndex: usize = 22;
    pub const data__playFx__tagUseAngles: usize = 23;
    pub const data__stopFx: usize = 8;
    pub const data__stopFx__handle: usize = 8;
    pub const data__stopFx__tagName: usize = 16;
    pub const data__stopFx__loopTime: usize = 20;
    pub const data__stopFx__loopTimeStreamIndex: usize = 22;
    pub const data__stopFx__tagUseAngles: usize = 23;
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
    pub const data__lightSettings__transitionTimeMin: usize = 24;
    pub const data__lightSettings__transitionTimeMax: usize = 26;
    pub const data__lightSettings__noteworthy: usize = 32;
    pub const data__sunlightSettings: usize = 8;
    pub const data__sunlightSettings__color: usize = 8;
    pub const data__sunlightSettings__transStateStreamIndex: usize = 12;
    pub const data__sunlightSettings__flags: usize = 13;
    pub const data__sunlightSettings__intensityScaleMin: usize = 16;
    pub const data__sunlightSettings__intensityScaleMax: usize = 20;
    pub const data__sunlightSettings__pitchMin: usize = 24;
    pub const data__sunlightSettings__pitchMax: usize = 26;
    pub const data__sunlightSettings__headingMin: usize = 28;
    pub const data__sunlightSettings__headingMax: usize = 30;
    pub const data__sunlightSettings__transitionTimeMin: usize = 32;
    pub const data__sunlightSettings__transitionTimeMax: usize = 34;
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
    pub const data__translate: usize = 8;
    pub const data__translate__translationMinX: usize = 8;
    pub const data__translate__translationMinY: usize = 10;
    pub const data__translate__translationMinZ: usize = 12;
    pub const data__translate__translationMaxX: usize = 14;
    pub const data__translate__translationMaxY: usize = 16;
    pub const data__translate__translationMaxZ: usize = 18;
    pub const data__translate__transitionTimeMin: usize = 20;
    pub const data__translate__transitionTimeMax: usize = 22;
    pub const data__translate__constIndex: usize = 24;
    pub const data__translate__transStateStreamIndex: usize = 25;
    pub const data__translate__flags: usize = 26;
    pub const data__translate__noteworthy: usize = 32;
    pub const data__rotate: usize = 8;
    pub const data__rotate__rotationMinPitch: usize = 8;
    pub const data__rotate__rotationMinYaw: usize = 10;
    pub const data__rotate__rotationMinRoll: usize = 12;
    pub const data__rotate__rotationMaxPitch: usize = 14;
    pub const data__rotate__rotationMaxYaw: usize = 16;
    pub const data__rotate__rotationMaxRoll: usize = 18;
    pub const data__rotate__transitionTimeMin: usize = 20;
    pub const data__rotate__transitionTimeMax: usize = 22;
    pub const data__rotate__constIndex: usize = 24;
    pub const data__rotate__transStateStreamIndex: usize = 25;
    pub const data__rotate__flags: usize = 26;
    pub const data__rotate__noteworthy: usize = 32;
    pub const data__stateChange: usize = 8;
    pub const data__stateChange__targetIndex: usize = 8;
    pub const data__stateChange__delayStreamIndex: usize = 9;
    pub const data__stateChange__delayMin: usize = 10;
    pub const data__stateChange__delayMax: usize = 12;
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
    pub const transitionTimeMin: usize = 16;
    pub const transitionTimeMax: usize = 18;
    pub const noteworthy: usize = 24;
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
pub mod ScriptableEventRotateDef {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const rotationMinPitch: usize = 0;
    pub const rotationMinYaw: usize = 2;
    pub const rotationMinRoll: usize = 4;
    pub const rotationMaxPitch: usize = 6;
    pub const rotationMaxYaw: usize = 8;
    pub const rotationMaxRoll: usize = 10;
    pub const transitionTimeMin: usize = 12;
    pub const transitionTimeMax: usize = 14;
    pub const constIndex: usize = 16;
    pub const transStateStreamIndex: usize = 17;
    pub const flags: usize = 18;
    pub const noteworthy: usize = 24;
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
pub mod ScriptableEventStopFxDef {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const handle: usize = 0;
    pub const tagName: usize = 8;
    pub const loopTime: usize = 12;
    pub const loopTimeStreamIndex: usize = 14;
    pub const tagUseAngles: usize = 15;
}
pub mod ScriptableEventSunlightSettingsDef {
    pub const SIZE: usize = 28;
    pub const ALIGN: usize = 4;
    pub const color: usize = 0;
    pub const transStateStreamIndex: usize = 4;
    pub const flags: usize = 5;
    pub const intensityScaleMin: usize = 8;
    pub const intensityScaleMax: usize = 12;
    pub const pitchMin: usize = 16;
    pub const pitchMax: usize = 18;
    pub const headingMin: usize = 20;
    pub const headingMax: usize = 22;
    pub const transitionTimeMin: usize = 24;
    pub const transitionTimeMax: usize = 26;
}
pub mod ScriptableEventTranslateDef {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const translationMinX: usize = 0;
    pub const translationMinY: usize = 2;
    pub const translationMinZ: usize = 4;
    pub const translationMaxX: usize = 6;
    pub const translationMaxY: usize = 8;
    pub const translationMaxZ: usize = 10;
    pub const transitionTimeMin: usize = 12;
    pub const transitionTimeMax: usize = 14;
    pub const constIndex: usize = 16;
    pub const transStateStreamIndex: usize = 17;
    pub const flags: usize = 18;
    pub const noteworthy: usize = 24;
}
pub mod ScriptableInstance {
    pub const SIZE: usize = 152;
    pub const ALIGN: usize = 8;
    pub const def: usize = 0;
    pub const eventConstantsBuf: usize = 8;
    pub const targetData: usize = 16;
    pub const origin: usize = 24;
    pub const angles: usize = 36;
    pub const startOrigin: usize = 48;
    pub const startAngles: usize = 60;
    pub const targetname: usize = 72;
    pub const preBrushModel: usize = 76;
    pub const postBrushModel: usize = 78;
    pub const flags: usize = 80;
    pub const targetCount: usize = 81;
    pub const currentModel: usize = 88;
    pub const partStates: usize = 96;
    pub const eventStreamBuf: usize = 104;
    pub const currentPartBits: usize = 112;
    pub const damageOwnerEntHandle: usize = 144;
    pub const updateNextInstance: usize = 148;
    pub const linkedObject: usize = 150;
}
pub mod ScriptableInstancePartState {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 2;
    pub const curHealth: usize = 0;
    pub const lastExecutedStateIndex: usize = 2;
    pub const stateIndex: usize = 3;
}
pub mod ScriptableInstanceTargetData {
    pub const SIZE: usize = 68;
    pub const ALIGN: usize = 4;
    pub const lightIndex: usize = 0;
    pub const lightAxis: usize = 4;
    pub const lightColorStorage: usize = 52;
    pub const lightRadiusStorage: usize = 64;
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
pub mod SkeletonScript {
    pub const SIZE: usize = 64;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const ikData: usize = 8;
    pub const ikData__numSolvers: usize = 8;
    pub const ikData__numFloatParams: usize = 9;
    pub const ikData__numIntParams: usize = 10;
    pub const ikData__numStringParams: usize = 11;
    pub const ikData__solverIDs: usize = 16;
    pub const ikData__floatParams: usize = 24;
    pub const ikData__intParams: usize = 32;
    pub const ikData__stringParams: usize = 40;
    pub const codeSize: usize = 48;
    pub const code: usize = 56;
}
pub mod SkeletonScriptCode {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 4;
    pub const instruction: usize = 0;
    pub const variableIndex: usize = 0;
    pub const immediateValue: usize = 0;
}
pub mod SndContext {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const priority: usize = 8;
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
    pub const packed__spatializedRangeCheck: usize = 0;
    pub const packed__spatializedIs3D: usize = 0;
    pub const packed__unk9: usize = 1;
    pub const packed__inheritPitch: usize = 1;
    pub const packed__inheritVolume: usize = 1;
    pub const packed__useContextList: usize = 1;
    pub const packed__useNoPanning2D: usize = 1;
    pub const packed__useOldPanning: usize = 1;
    pub const packed__useNoPanning3D: usize = 1;
    pub const packed__type: usize = 2;
    pub const packed__unused: usize = 2;
    pub const intValue: usize = 0;
}
pub mod SoundFile {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const exists: usize = 1;
    pub const u: usize = 8;
    pub const u__loadSnd: usize = 8;
    pub const u__streamSnd: usize = 8;
    pub const u__streamSnd__filename: usize = 8;
    pub const u__streamSnd__filename__isLocalized: usize = 8;
    pub const u__streamSnd__filename__isStreamed: usize = 9;
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
    pub const u__primedSnd__loadedPart: usize = 8;
    pub const u__primedSnd__streamedPart: usize = 16;
    pub const u__primedSnd__streamedPart__isLocalized: usize = 16;
    pub const u__primedSnd__streamedPart__isStreamed: usize = 17;
    pub const u__primedSnd__streamedPart__fileIndex: usize = 18;
    pub const u__primedSnd__streamedPart__info: usize = 24;
    pub const u__primedSnd__streamedPart__info__raw: usize = 24;
    pub const u__primedSnd__streamedPart__info__raw__dir: usize = 24;
    pub const u__primedSnd__streamedPart__info__raw__name: usize = 32;
    pub const u__primedSnd__streamedPart__info__packed: usize = 24;
    pub const u__primedSnd__streamedPart__info__packed__offset: usize = 24;
    pub const u__primedSnd__streamedPart__info__packed__length: usize = 32;
    pub const u__primedSnd__dataOffset: usize = 40;
    pub const u__primedSnd__totalSize: usize = 44;
}
pub mod SoundFileRef {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const loadSnd: usize = 0;
    pub const streamSnd: usize = 0;
    pub const streamSnd__filename: usize = 0;
    pub const streamSnd__filename__isLocalized: usize = 0;
    pub const streamSnd__filename__isStreamed: usize = 1;
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
    pub const primedSnd__loadedPart: usize = 0;
    pub const primedSnd__streamedPart: usize = 8;
    pub const primedSnd__streamedPart__isLocalized: usize = 8;
    pub const primedSnd__streamedPart__isStreamed: usize = 9;
    pub const primedSnd__streamedPart__fileIndex: usize = 10;
    pub const primedSnd__streamedPart__info: usize = 16;
    pub const primedSnd__streamedPart__info__raw: usize = 16;
    pub const primedSnd__streamedPart__info__raw__dir: usize = 16;
    pub const primedSnd__streamedPart__info__raw__name: usize = 24;
    pub const primedSnd__streamedPart__info__packed: usize = 16;
    pub const primedSnd__streamedPart__info__packed__offset: usize = 16;
    pub const primedSnd__streamedPart__info__packed__length: usize = 24;
    pub const primedSnd__dataOffset: usize = 32;
    pub const primedSnd__totalSize: usize = 36;
}
pub mod SoundOverrideEntry {
    pub const SIZE: usize = 24;
    pub const ALIGN: usize = 8;
    pub const attachment1: usize = 0;
    pub const attachment2: usize = 1;
    pub const soundType: usize = 2;
    pub const overrideSound: usize = 8;
    pub const altmodeSound: usize = 16;
}
pub mod SpawnPointEntityRecord {
    pub const SIZE: usize = 44;
    pub const ALIGN: usize = 4;
    pub const index: usize = 0;
    pub const name: usize = 4;
    pub const target: usize = 8;
    pub const script_noteworthy: usize = 12;
    pub const targetname: usize = 16;
    pub const origin: usize = 20;
    pub const angles: usize = 32;
}
pub mod SpawnPointRecordList {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const spawnsCount: usize = 0;
    pub const spawns: usize = 8;
}
pub mod SpeakerLevels {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const speaker: usize = 0;
    pub const numLevels: usize = 1;
    pub const levels: usize = 4;
}
pub mod SpeakerMap {
    pub const SIZE: usize = 328;
    pub const ALIGN: usize = 8;
    pub const isDefault: usize = 0;
    pub const name: usize = 8;
    pub const orientation: usize = 16;
    pub const channelMaps: usize = 20;
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
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const origin: usize = 8;
    pub const triggerIndex: usize = 20;
    pub const sunPrimaryLightIndex: usize = 22;
    pub const radiometricScale: usize = 24;
}
pub mod StateTimers {
    pub const SIZE: usize = 300;
    pub const ALIGN: usize = 4;
    pub const fireDelay: usize = 0;
    pub const meleeDelay: usize = 4;
    pub const meleeChargeDelay: usize = 8;
    pub const detonateDelay: usize = 12;
    pub const fireTime: usize = 16;
    pub const rechamberTime: usize = 20;
    pub const rechamberTimeOneHanded: usize = 24;
    pub const rechamberBoltTime: usize = 28;
    pub const holdFireTime: usize = 32;
    pub const grenadePrimeReadyToThrowTime: usize = 36;
    pub const detonateTime: usize = 40;
    pub const meleeTime: usize = 44;
    pub const meleeChargeTime: usize = 48;
    pub const reloadTime: usize = 52;
    pub const reloadShowRocketTime: usize = 56;
    pub const reloadEmptyTime: usize = 60;
    pub const reloadAddTime: usize = 64;
    pub const reloadEmptyAddTime: usize = 68;
    pub const reloadStartTime: usize = 72;
    pub const reloadStartAddTime: usize = 76;
    pub const reloadEndTime: usize = 80;
    pub const reloadTimeDualWield: usize = 84;
    pub const reloadAddTimeDualWield: usize = 88;
    pub const reloadEmptyDualMag: usize = 92;
    pub const reloadEmptyAddTimeDualMag: usize = 96;
    pub const speedReloadTime: usize = 100;
    pub const speedReloadAddTime: usize = 104;
    pub const dropTime: usize = 108;
    pub const raiseTime: usize = 112;
    pub const altDropTime: usize = 116;
    pub const altRaiseTime: usize = 120;
    pub const quickDropTime: usize = 124;
    pub const quickRaiseTime: usize = 128;
    pub const firstRaiseTime: usize = 132;
    pub const breachRaiseTime: usize = 136;
    pub const emptyRaiseTime: usize = 140;
    pub const emptyDropTime: usize = 144;
    pub const sprintInTime: usize = 148;
    pub const sprintLoopTime: usize = 152;
    pub const sprintOutTime: usize = 156;
    pub const stunnedTimeBegin: usize = 160;
    pub const stunnedTimeLoop: usize = 164;
    pub const stunnedTimeEnd: usize = 168;
    pub const nightVisionWearTime: usize = 172;
    pub const nightVisionWearTimeFadeOutEnd: usize = 176;
    pub const nightVisionWearTimePowerUp: usize = 180;
    pub const nightVisionRemoveTime: usize = 184;
    pub const nightVisionRemoveTimePowerDown: usize = 188;
    pub const nightVisionRemoveTimeFadeInStart: usize = 192;
    pub const aiFuseTime: usize = 196;
    pub const fuseTime: usize = 200;
    pub const missileTime: usize = 204;
    pub const primeTime: usize = 208;
    pub const bHoldFullPrime: usize = 212;
    pub const blastFrontTime: usize = 216;
    pub const blastRightTime: usize = 220;
    pub const blastBackTime: usize = 224;
    pub const blastLeftTime: usize = 228;
    pub const slideInTime: usize = 232;
    pub const slideLoopTime: usize = 236;
    pub const slideOutTime: usize = 240;
    pub const highJumpInTime: usize = 244;
    pub const highJumpDropInTime: usize = 248;
    pub const highJumpDropLoopTime: usize = 252;
    pub const highJumpDropLandTime: usize = 256;
    pub const dodgeTime: usize = 260;
    pub const landDipTime: usize = 264;
    pub const hybridSightInTime: usize = 268;
    pub const hybridSightOutTime: usize = 272;
    pub const offhandSwitchTime: usize = 276;
    pub const heatCooldownInTime: usize = 280;
    pub const heatCooldownOutTime: usize = 284;
    pub const heatCooldownOutReadyTime: usize = 288;
    pub const overheatOutTime: usize = 292;
    pub const overheatOutReadyTime: usize = 296;
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
    pub const isStreamed: usize = 1;
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
    pub const filename__isStreamed: usize = 1;
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
pub mod TracerDef {
    pub const SIZE: usize = 128;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const material: usize = 8;
    pub const effectDef: usize = 16;
    pub const drawInterval: usize = 24;
    pub const speed: usize = 28;
    pub const beamLength: usize = 32;
    pub const beamWidth: usize = 36;
    pub const screwRadius: usize = 40;
    pub const screwDist: usize = 44;
    pub const colors: usize = 48;
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
pub mod TurretHydraulicSettings {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const minVelocity: usize = 0;
    pub const verticalSound: usize = 8;
    pub const verticalStopSound: usize = 16;
    pub const horizontalSound: usize = 24;
    pub const horizontalStopSound: usize = 32;
}
pub mod UIFunctionList {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const totalFunctions: usize = 0;
    pub const functions: usize = 8;
}
pub mod UnknownXSurface0 {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const xyz: usize = 0;
    pub const normal: usize = 12;
    pub const normal__packed: usize = 12;
}
pub mod VehicleDef {
    pub const SIZE: usize = 2256;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const internalName: usize = 0;
    pub const r#type: usize = 8;
    pub const useHintString: usize = 16;
    pub const health: usize = 24;
    pub const quadBarrel: usize = 28;
    pub const hitClientScriptables: usize = 32;
    pub const multipleLinkedGroundEntities: usize = 36;
    pub const hideVehicleForDriver: usize = 40;
    pub const texScrollScale: usize = 44;
    pub const topSpeed: usize = 48;
    pub const accel: usize = 52;
    pub const rotRate: usize = 56;
    pub const rotAccel: usize = 60;
    pub const maxBodyPitch: usize = 64;
    pub const maxBodyRoll: usize = 68;
    pub const legIK: usize = 72;
    pub const fakeBodyAccelPitch: usize = 76;
    pub const fakeBodyAccelRoll: usize = 80;
    pub const fakeBodyVelPitch: usize = 84;
    pub const fakeBodyVelRoll: usize = 88;
    pub const fakeBodySideVelPitch: usize = 92;
    pub const fakeBodyPitchStrength: usize = 96;
    pub const fakeBodyRollStrength: usize = 100;
    pub const fakeBodyPitchDampening: usize = 104;
    pub const fakeBodyRollDampening: usize = 108;
    pub const fakeBodyBoatRockingAmplitude: usize = 112;
    pub const fakeBodyBoatRockingPeriod: usize = 116;
    pub const fakeBodyBoatRockingRotationPeriod: usize = 120;
    pub const fakeBodyBoatRockingFadeoutSpeed: usize = 124;
    pub const boatBouncingMinForce: usize = 128;
    pub const boatBouncingMaxForce: usize = 132;
    pub const boatBouncingRate: usize = 136;
    pub const boatBouncingFadeinSpeed: usize = 140;
    pub const boatBouncingFadeoutSteeringAngle: usize = 144;
    pub const collisionDamage: usize = 148;
    pub const collisionSpeed: usize = 152;
    pub const killcamOffset: usize = 156;
    pub const playerProtected: usize = 168;
    pub const bulletDamage: usize = 172;
    pub const armorPiercingDamage: usize = 176;
    pub const grenadeDamage: usize = 180;
    pub const projectileDamage: usize = 184;
    pub const projectileSplashDamage: usize = 188;
    pub const heavyExplosiveDamage: usize = 192;
    pub const pad1: usize = 196;
    pub const vehPhysDef: usize = 200;
    pub const vehPhysDef__physicsEnabled: usize = 200;
    pub const vehPhysDef__physPresetName: usize = 208;
    pub const vehPhysDef__physPreset: usize = 216;
    pub const vehPhysDef__physMassName: usize = 224;
    pub const vehPhysDef__physMass: usize = 232;
    pub const vehPhysDef__accelGraphName: usize = 240;
    pub const vehPhysDef__steeringAxle: usize = 248;
    pub const vehPhysDef__powerAxle: usize = 252;
    pub const vehPhysDef__brakingAxle: usize = 256;
    pub const vehPhysDef__topSpeed: usize = 260;
    pub const vehPhysDef__topSpeedTurbo: usize = 264;
    pub const vehPhysDef__reverseSpeed: usize = 268;
    pub const vehPhysDef__maxVelocity: usize = 272;
    pub const vehPhysDef__maxPitch: usize = 276;
    pub const vehPhysDef__maxRoll: usize = 280;
    pub const vehPhysDef__wheelRadius: usize = 284;
    pub const vehPhysDef__suspensionTravelFront: usize = 288;
    pub const vehPhysDef__suspensionTravelRear: usize = 292;
    pub const vehPhysDef__suspensionStrengthFront: usize = 296;
    pub const vehPhysDef__suspensionDampingFront: usize = 300;
    pub const vehPhysDef__suspensionStrengthRear: usize = 304;
    pub const vehPhysDef__suspensionDampingRear: usize = 308;
    pub const vehPhysDef__frictionBraking: usize = 312;
    pub const vehPhysDef__frictionCoasting: usize = 316;
    pub const vehPhysDef__frictionTopSpeed: usize = 320;
    pub const vehPhysDef__frictionSide: usize = 324;
    pub const vehPhysDef__frictionSideRear: usize = 328;
    pub const vehPhysDef__handBrakeLongitudinalSteerableFrictionScale: usize = 332;
    pub const vehPhysDef__handBrakeLateralSteerableFrictionScale: usize = 336;
    pub const vehPhysDef__handBrakeLongitudinalNonsteerableFrictionScale: usize = 340;
    pub const vehPhysDef__handBrakeLateralNonsteerableFrictionScale: usize = 344;
    pub const vehPhysDef__handBrakingStrength: usize = 348;
    pub const vehPhysDef__handBrakeExtraYawTorque: usize = 352;
    pub const vehPhysDef__speedAtMaxHandBrakeExtraYawTorque: usize = 356;
    pub const vehPhysDef__driveForceFalloffFraction: usize = 360;
    pub const vehPhysDef__velocityDependentSlip: usize = 364;
    pub const vehPhysDef__rollStability: usize = 368;
    pub const vehPhysDef__rollResistance: usize = 372;
    pub const vehPhysDef__pitchResistance: usize = 376;
    pub const vehPhysDef__yawResistance: usize = 380;
    pub const vehPhysDef__uprightStrengthPitch: usize = 384;
    pub const vehPhysDef__uprightStrengthRoll: usize = 388;
    pub const vehPhysDef__targetAirPitch: usize = 392;
    pub const vehPhysDef__airYawTorque: usize = 396;
    pub const vehPhysDef__airPitchTorque: usize = 400;
    pub const vehPhysDef__minimumMomentumForCollision: usize = 404;
    pub const vehPhysDef__collisionLaunchForceScale: usize = 408;
    pub const vehPhysDef__wreckedMassScale: usize = 412;
    pub const vehPhysDef__wreckedBodyFriction: usize = 416;
    pub const vehPhysDef__minimumJoltForNotify: usize = 420;
    pub const vehPhysDef__slipThresholdFront: usize = 424;
    pub const vehPhysDef__slipThresholdRear: usize = 428;
    pub const vehPhysDef__slipFricScaleFront: usize = 432;
    pub const vehPhysDef__slipFricScaleRear: usize = 436;
    pub const vehPhysDef__slipFricRateFront: usize = 440;
    pub const vehPhysDef__slipFricRateRear: usize = 444;
    pub const vehPhysDef__slipYawTorque: usize = 448;
    pub const vehPhysDef__cruiseControlProportionalGain: usize = 452;
    pub const vehPhysDef__cruiseControlIntegralGain: usize = 456;
    pub const boostDuration: usize = 464;
    pub const boostRechargeTime: usize = 468;
    pub const boostAcceleration: usize = 472;
    pub const boostTopSpeed: usize = 476;
    pub const suspensionTravel: usize = 480;
    pub const maxSteeringAngle: usize = 484;
    pub const steeringLerp: usize = 488;
    pub const steeringLerpCentering: usize = 492;
    pub const minSteeringScale: usize = 496;
    pub const minSteeringSpeed: usize = 500;
    pub const disableWheelsTurning: usize = 504;
    pub const pad2: usize = 508;
    pub const treadDefaultFx: usize = 512;
    pub const handBrakeDefaultFx: usize = 520;
    pub const handBrakeLeftFx: usize = 528;
    pub const handBrakeRightFx: usize = 536;
    pub const boostFx: usize = 544;
    pub const treadFxSlowestRepeatRate: usize = 552;
    pub const treadFxFastestRepeatRate: usize = 556;
    pub const treadFxMinSpeed: usize = 560;
    pub const treadFxMaxSpeed: usize = 564;
    pub const vehHelicopterIsASplinePlane: usize = 568;
    pub const vehHelicopterOrbitsAroundPoint: usize = 572;
    pub const vehHelicopterLockAltitude: usize = 576;
    pub const vehHelicopterOffsetFromMesh: usize = 580;
    pub const vehHelicopterAltitudeOffset: usize = 584;
    pub const vehHelicopterPitchOffset: usize = 588;
    pub const vehHelicopterBoundsRadius: usize = 592;
    pub const vehHelicopterBoundsOffsetZ: usize = 596;
    pub const vehHelicopterMaxSpeed: usize = 600;
    pub const vehHelicopterMaxSpeedVertical: usize = 604;
    pub const vehHelicopterMaxAccel: usize = 608;
    pub const vehHelicopterMaxAccelVertical: usize = 612;
    pub const vehHelicopterDecelerationFwd: usize = 616;
    pub const vehHelicopterDecelerationSide: usize = 620;
    pub const vehHelicopterDecelerationUp: usize = 624;
    pub const vehHelicopterMaxYawRate: usize = 628;
    pub const vehHelicopterMaxYawAccel: usize = 632;
    pub const vehHelicopterTiltFromVelocity: usize = 636;
    pub const vehHelicopterTiltFromControllerAxes: usize = 640;
    pub const vehHelicopterTiltFromAcceleration: usize = 644;
    pub const vehHelicopterTiltFromDeceleration: usize = 648;
    pub const vehHelicopterTiltFromFwdAndYaw_VelAtMaxTilt: usize = 652;
    pub const vehHelicopterTiltFromFwdAndYaw: usize = 656;
    pub const vehHelicopterTiltMomentum: usize = 660;
    pub const vehHelicopterTiltSpeed: usize = 664;
    pub const vehHelicopterMaxPitch: usize = 668;
    pub const vehHelicopterMaxRoll: usize = 672;
    pub const vehHelicopterHoverSpeedThreshold: usize = 676;
    pub const vehHelicopterJitterJerkyness: usize = 680;
    pub const vehHelicopterUseHoverWobble: usize = 684;
    pub const vehHelicopterHoverWobblePhase: usize = 688;
    pub const vehHelicopterHoverWobbleAmplitude: usize = 692;
    pub const vehHelicopterUseBob: usize = 696;
    pub const vehHelicopterBobPhase: usize = 700;
    pub const vehHelicopterBobAmplitude: usize = 704;
    pub const vehHelicopterLookaheadTime: usize = 708;
    pub const vehHelicopterSoftCollisions: usize = 712;
    pub const vehHelicopterUseGroundFX: usize = 716;
    pub const vehHelicopterGroundFx: usize = 720;
    pub const vehHelicopterGroundWaterFx: usize = 728;
    pub const vehHelicopterGroundFxDefaultRepeatRate: usize = 736;
    pub const vehHelicopterGroundFxSlowestRepeatRate: usize = 740;
    pub const vehHelicopterGroundFxFastestRepeatRate: usize = 744;
    pub const vehHelicopterGroundFxMinGroundDist: usize = 748;
    pub const vehHelicopterGroundFxMaxGroundDist: usize = 752;
    pub const vehSplinePlaneCorridorMaxXVel: usize = 756;
    pub const vehSplinePlaneCorridorMaxZVel: usize = 760;
    pub const vehSplinePlaneCorridorMaxXAccel: usize = 764;
    pub const vehSplinePlaneCorridorMaxZAccel: usize = 768;
    pub const vehSplinePlaneTangentLookAtRate: usize = 772;
    pub const vehSplinePlaneMaxPitchSpeed: usize = 776;
    pub const vehSplinePlaneMaxYawSpeed: usize = 780;
    pub const vehSplinePlaneMaxRollSpeed: usize = 784;
    pub const vehSplinePlanePitchSpeedRate: usize = 788;
    pub const vehSplinePlaneYawSpeedRate: usize = 792;
    pub const vehSplinePlaneRollSpeedRateUp: usize = 796;
    pub const vehSplinePlaneRollSpeedRateDown: usize = 800;
    pub const vehSplinePlaneMaxPitchAccel: usize = 804;
    pub const vehSplinePlaneMaxYawAccel: usize = 808;
    pub const vehSplinePlaneMaxRollAccel: usize = 812;
    pub const vehSplinePlaneYawToRollFactor: usize = 816;
    pub const vehSplinePlaneRollToYawFactor: usize = 820;
    pub const vehSplinePlaneRollToYawFactorBlend: usize = 824;
    pub const vehSplinePlaneMaxPitch: usize = 828;
    pub const vehSplinePlaneMaxRoll: usize = 832;
    pub const vehSplinePlaneMaxPitchFromRoll: usize = 836;
    pub const vehSplinePlaneMaxRollYawOffset: usize = 840;
    pub const vehSplinePlaneRollYawOffsetThreshold: usize = 844;
    pub const vehSplinePlaneMaxTiltRoll: usize = 848;
    pub const vehSplinePlaneMaxTiltPitch: usize = 852;
    pub const vehSplinePlaneTiltRollRate: usize = 856;
    pub const vehSplinePlaneTiltPitchRate: usize = 860;
    pub const vehJetbikeThrottleForce: usize = 864;
    pub const vehJetbikeStrafeForce: usize = 868;
    pub const vehJetbikeYawTorque: usize = 872;
    pub const vehJetbikePitchTorque: usize = 876;
    pub const vehJetbikeYawDamping: usize = 880;
    pub const vehJetbikePitchDamping: usize = 884;
    pub const vehJetbikeRollDamping: usize = 888;
    pub const vehJetbikeRepulsorMaxForceFraction: usize = 892;
    pub const vehJetbikeRepulsorMinForceFraction: usize = 896;
    pub const vehJetbikeRepulsorCompressionDampingConstant: usize = 900;
    pub const vehJetbikeRepulsorReboundDampingConstant: usize = 904;
    pub const vehJetbikeRepulsorTorqueScale: usize = 908;
    pub const vehJetbikeRepulsorCrossCoupling: usize = 912;
    pub const vehJetbikeAntislipConstant: usize = 916;
    pub const vehJetbikeAntislipMaxForce: usize = 920;
    pub const vehJetbikeControlForceLocalOffsetX: usize = 924;
    pub const vehJetbikeControlForceLocalOffsetZ: usize = 928;
    pub const vehJetbikeControlTorqueLocalOffsetX: usize = 932;
    pub const vehJetbikeControlTorqueLocalOffsetZ: usize = 936;
    pub const vehJetbikeMaxControlForce: usize = 940;
    pub const vehJetbikeMinContactForFullControl: usize = 944;
    pub const vehJetbikeThrustScaleWithNoContact: usize = 948;
    pub const vehJetbikeTorqueScaleWithNoContact: usize = 952;
    pub const vehJetbikeUprightingTorque: usize = 956;
    pub const vehJetbikeUprightingTorqueWithNoContact: usize = 960;
    pub const vehJetbikeWeathervaneTorque: usize = 964;
    pub const vehJetbikeWeathervaneTorqueWithNoContact: usize = 968;
    pub const vehJetbikeAiSteeringConstant: usize = 972;
    pub const vehJetbikeAiStationarySteeringScale: usize = 976;
    pub const vehJetbikeAiThrottleConstant: usize = 980;
    pub const vehHovertankAutoYawForce: usize = 984;
    pub const vehHovertankAutoBrakeForce: usize = 988;
    pub const vehHovertankRandomHoverForceMagMin: usize = 992;
    pub const vehHovertankRandomHoverForceMagMax: usize = 996;
    pub const vehHovertankRandomHoverForceStartTimerMin: usize = 1000;
    pub const vehHovertankRandomHoverForceStartTimerMax: usize = 1004;
    pub const vehHovertankRandomHoverForceDurationMin: usize = 1008;
    pub const vehHovertankRandomHoverForceDurationMax: usize = 1012;
    pub const vehDiveboatInitialDiveForceFactor: usize = 1016;
    pub const vehDiveboatContinuingDiveForceFactor: usize = 1020;
    pub const vehDiveboatMaxDiveTime: usize = 1024;
    pub const vehDiveboatDiveResetTime: usize = 1028;
    pub const vehDiveboatSubmergedDragFactor: usize = 1032;
    pub const vehDiveboatRollFactor: usize = 1036;
    pub const vehDiveboatBuoyancyOffset: usize = 1040;
    pub const pad3: usize = 1044;
    pub const vehDiveboatSteeringGraphName: usize = 1048;
    pub const steeringGraphIndex: usize = 1056;
    pub const vehOrbiterMinYaw: usize = 1060;
    pub const vehOrbiterMaxYaw: usize = 1064;
    pub const vehOrbiterMinZ: usize = 1068;
    pub const vehOrbiterMaxZ: usize = 1072;
    pub const vehOrbiterAngularAcceleration: usize = 1076;
    pub const vehOrbiterAngularMaxVelocity: usize = 1080;
    pub const vehOrbiterAngularDeceleration: usize = 1084;
    pub const vehOrbiterAngularADSDeceleration: usize = 1088;
    pub const vehOrbiterAngularBraking: usize = 1092;
    pub const vehOrbiterAngularADSBraking: usize = 1096;
    pub const vehOrbiterAngularLookAheadTime: usize = 1100;
    pub const vehOrbiterVerticalAcceleration: usize = 1104;
    pub const vehOrbiterVerticalMaxVelocity: usize = 1108;
    pub const vehOrbiterVerticalDeceleration: usize = 1112;
    pub const vehOrbiterVerticalADSDeceleration: usize = 1116;
    pub const vehOrbiterVerticalBraking: usize = 1120;
    pub const vehOrbiterVerticalADSBraking: usize = 1124;
    pub const vehOrbiterVerticalLookAheadTime: usize = 1128;
    pub const vehOrbiterADSVelocityMult: usize = 1132;
    pub const vehOrbiterTiltRollMax: usize = 1136;
    pub const vehOrbiterTiltRollRate: usize = 1140;
    pub const vehOrbiterTiltPitchMax: usize = 1144;
    pub const vehOrbiterTiltPitchRate: usize = 1148;
    pub const camLookEnabled: usize = 1152;
    pub const camRelativeControl: usize = 1156;
    pub const camRemoteDrive: usize = 1160;
    pub const camLerp: usize = 1164;
    pub const camHeight: usize = 1168;
    pub const camRadius: usize = 1172;
    pub const camPitchInfluence: usize = 1176;
    pub const camYawInfluence: usize = 1180;
    pub const camRollInfluence: usize = 1184;
    pub const camFovIncrease: usize = 1188;
    pub const camFovOffset: usize = 1192;
    pub const camFovSpeed: usize = 1196;
    pub const camReturnSpeed: usize = 1200;
    pub const camReturnLerp: usize = 1204;
    pub const camVehicleAnglePitchRate: usize = 1208;
    pub const camVehicleAngleYawRate: usize = 1212;
    pub const camVehicleAngleRollRate: usize = 1216;
    pub const camShakeMinSpeed: usize = 1220;
    pub const camShakeMaxSpeed: usize = 1224;
    pub const camShakeMinFreq: usize = 1228;
    pub const camShakeMaxFreq: usize = 1232;
    pub const camShakeMaxAmplitudePitch: usize = 1236;
    pub const camShakeMaxAmplitudeYaw: usize = 1240;
    pub const camShakeMaxAmplitudeRoll: usize = 1244;
    pub const camShakeMaxAmplitudeX: usize = 1248;
    pub const camShakeMaxAmplitudeY: usize = 1252;
    pub const camShakeMaxAmplitudeZ: usize = 1256;
    pub const camShakeMinAmplitudeScale: usize = 1260;
    pub const camShakeTurretInherit: usize = 1264;
    pub const vehCam_UseGDT: usize = 1268;
    pub const vehCam_anglesPitch: usize = 1272;
    pub const vehCam_anglesYaw: usize = 1276;
    pub const vehCam_anglesRoll: usize = 1280;
    pub const vehCam_offsetX: usize = 1284;
    pub const vehCam_offsetY: usize = 1288;
    pub const vehCam_offsetZ: usize = 1292;
    pub const vehCam_radius: usize = 1296;
    pub const vehCam_speedInfluence: usize = 1300;
    pub const vehCam_pitchTurnRate: usize = 1304;
    pub const vehCam_pitchClamp: usize = 1308;
    pub const vehCam_yawTurnRate: usize = 1312;
    pub const vehCam_yawClamp: usize = 1316;
    pub const vehCam_zOffsetMode: usize = 1320;
    pub const vehCam_anglesPitch3P: usize = 1324;
    pub const vehCam_anglesYaw3P: usize = 1328;
    pub const vehCam_anglesRoll3P: usize = 1332;
    pub const vehCam_offsetX3P: usize = 1336;
    pub const vehCam_offsetY3P: usize = 1340;
    pub const vehCam_offsetZ3P: usize = 1344;
    pub const vehCam_radius3P: usize = 1348;
    pub const vehCam_speedInfluence3P: usize = 1352;
    pub const vehCam_pitchTurnRate3P: usize = 1356;
    pub const vehCam_pitchClamp3P: usize = 1360;
    pub const vehCam_yawTurnRate3P: usize = 1364;
    pub const vehCam_yawTurnRate3PHandbrakeInc: usize = 1368;
    pub const vehCam_yawClamp3P: usize = 1372;
    pub const vehCam_zOffsetMode3P: usize = 1376;
    pub const pad4: usize = 1380;
    pub const turretWeaponName: usize = 1384;
    pub const turretWeapon: usize = 1392;
    pub const turretHorizSpanLeft: usize = 1400;
    pub const turretHorizSpanRight: usize = 1404;
    pub const turretVertSpanUp: usize = 1408;
    pub const turretVertSpanDown: usize = 1412;
    pub const turretHorizResistLeft: usize = 1416;
    pub const turretHorizResistRight: usize = 1420;
    pub const turretVertResistUp: usize = 1424;
    pub const turretVertResistDown: usize = 1428;
    pub const turretRotRate: usize = 1432;
    pub const turretFireType: usize = 1436;
    pub const turretSpinSnd: usize = 1440;
    pub const turretStopSnd: usize = 1448;
    pub const trophyEnabled: usize = 1456;
    pub const trophyRadius: usize = 1460;
    pub const trophyInactiveRadius: usize = 1464;
    pub const trophyAmmoCount: usize = 1468;
    pub const trophyReloadTime: usize = 1472;
    pub const trophyTags: usize = 1476;
    pub const trophyExplodeFx: usize = 1496;
    pub const trophyFlashFx: usize = 1504;
    pub const compassFriendlyIcon: usize = 1512;
    pub const compassEnemyIcon: usize = 1520;
    pub const compassFriendlyAltIcon: usize = 1528;
    pub const compassEnemyAltIcon: usize = 1536;
    pub const compassIconWidth: usize = 1544;
    pub const compassIconHeight: usize = 1548;
    pub const idleLowSnd: usize = 1552;
    pub const idleHighSnd: usize = 1560;
    pub const engineLowSnd: usize = 1568;
    pub const engineHighSnd: usize = 1576;
    pub const boostSnd: usize = 1584;
    pub const engineSndSpeed: usize = 1592;
    pub const audioOriginTag: usize = 1596;
    pub const idleLowSndAlt: usize = 1600;
    pub const idleHighSndAlt: usize = 1608;
    pub const engineLowSndAlt: usize = 1616;
    pub const engineHighSndAlt: usize = 1624;
    pub const engineSndSpeedAlt: usize = 1632;
    pub const audioOriginTagAlt: usize = 1636;
    pub const turretSpinSndAlt: usize = 1640;
    pub const turretStopSndAlt: usize = 1648;
    pub const engineStartUpSnd: usize = 1656;
    pub const engineStartUpLength: usize = 1664;
    pub const engineShutdownSnd: usize = 1672;
    pub const engineIdleSnd: usize = 1680;
    pub const engineSustainSnd: usize = 1688;
    pub const engineRampUpSnd: usize = 1696;
    pub const engineRampUpLength: usize = 1704;
    pub const engineRampDownSnd: usize = 1712;
    pub const engineRampDownLength: usize = 1720;
    pub const suspensionSoftSnd: usize = 1728;
    pub const suspensionSoftCompression: usize = 1736;
    pub const suspensionHardSnd: usize = 1744;
    pub const suspensionHardCompression: usize = 1752;
    pub const collisionSnd: usize = 1760;
    pub const collisionBlendSpeed: usize = 1768;
    pub const speedSnd: usize = 1776;
    pub const speedSndBlendSpeed: usize = 1784;
    pub const surfaceSndPrefix: usize = 1792;
    pub const surfaceSnds: usize = 1800;
    pub const surfaceSndBlendSpeed: usize = 2224;
    pub const slideVolume: usize = 2228;
    pub const slideBlendSpeed: usize = 2232;
    pub const inAirPitch: usize = 2236;
    pub const soundTriggerOverrideZone: usize = 2240;
    pub const soundTriggerOverrideReverb: usize = 2248;
    pub const soundTriggerOverrideMix: usize = 2249;
    pub const soundTriggerOverrideFilter: usize = 2250;
    pub const soundTriggerOverrideOcclusion: usize = 2251;
    pub const soundTriggerOverrideAmbient: usize = 2252;
    pub const soundTriggerOverrideAmbientEvents: usize = 2253;
}
pub mod VehiclePhysDef {
    pub const SIZE: usize = 264;
    pub const ALIGN: usize = 8;
    pub const physicsEnabled: usize = 0;
    pub const physPresetName: usize = 8;
    pub const physPreset: usize = 16;
    pub const physMassName: usize = 24;
    pub const physMass: usize = 32;
    pub const accelGraphName: usize = 40;
    pub const steeringAxle: usize = 48;
    pub const powerAxle: usize = 52;
    pub const brakingAxle: usize = 56;
    pub const topSpeed: usize = 60;
    pub const topSpeedTurbo: usize = 64;
    pub const reverseSpeed: usize = 68;
    pub const maxVelocity: usize = 72;
    pub const maxPitch: usize = 76;
    pub const maxRoll: usize = 80;
    pub const wheelRadius: usize = 84;
    pub const suspensionTravelFront: usize = 88;
    pub const suspensionTravelRear: usize = 92;
    pub const suspensionStrengthFront: usize = 96;
    pub const suspensionDampingFront: usize = 100;
    pub const suspensionStrengthRear: usize = 104;
    pub const suspensionDampingRear: usize = 108;
    pub const frictionBraking: usize = 112;
    pub const frictionCoasting: usize = 116;
    pub const frictionTopSpeed: usize = 120;
    pub const frictionSide: usize = 124;
    pub const frictionSideRear: usize = 128;
    pub const handBrakeLongitudinalSteerableFrictionScale: usize = 132;
    pub const handBrakeLateralSteerableFrictionScale: usize = 136;
    pub const handBrakeLongitudinalNonsteerableFrictionScale: usize = 140;
    pub const handBrakeLateralNonsteerableFrictionScale: usize = 144;
    pub const handBrakingStrength: usize = 148;
    pub const handBrakeExtraYawTorque: usize = 152;
    pub const speedAtMaxHandBrakeExtraYawTorque: usize = 156;
    pub const driveForceFalloffFraction: usize = 160;
    pub const velocityDependentSlip: usize = 164;
    pub const rollStability: usize = 168;
    pub const rollResistance: usize = 172;
    pub const pitchResistance: usize = 176;
    pub const yawResistance: usize = 180;
    pub const uprightStrengthPitch: usize = 184;
    pub const uprightStrengthRoll: usize = 188;
    pub const targetAirPitch: usize = 192;
    pub const airYawTorque: usize = 196;
    pub const airPitchTorque: usize = 200;
    pub const minimumMomentumForCollision: usize = 204;
    pub const collisionLaunchForceScale: usize = 208;
    pub const wreckedMassScale: usize = 212;
    pub const wreckedBodyFriction: usize = 216;
    pub const minimumJoltForNotify: usize = 220;
    pub const slipThresholdFront: usize = 224;
    pub const slipThresholdRear: usize = 228;
    pub const slipFricScaleFront: usize = 232;
    pub const slipFricScaleRear: usize = 236;
    pub const slipFricRateFront: usize = 240;
    pub const slipFricRateRear: usize = 244;
    pub const slipYawTorque: usize = 248;
    pub const cruiseControlProportionalGain: usize = 252;
    pub const cruiseControlIntegralGain: usize = 256;
}
pub mod WAField {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const index: usize = 0;
    pub const r#type: usize = 1;
    pub const code: usize = 2;
    pub const parm: usize = 8;
    pub const parm__p_char: usize = 8;
    pub const parm__p_bool: usize = 8;
    pub const parm__p_int: usize = 8;
    pub const parm__p_float: usize = 8;
    pub const parm__string: usize = 8;
}
pub mod WAFieldParm {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 8;
    pub const p_char: usize = 0;
    pub const p_bool: usize = 0;
    pub const p_int: usize = 0;
    pub const p_float: usize = 0;
    pub const string: usize = 0;
}
pub mod WaterWritable {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 4;
    pub const floatTime: usize = 0;
}
pub mod WeaponAttachment {
    pub const SIZE: usize = 120;
    pub const ALIGN: usize = 8;
    pub const szInternalName: usize = 0;
    pub const name: usize = 0;
    pub const szDisplayName: usize = 8;
    pub const r#type: usize = 16;
    pub const weaponType: usize = 20;
    pub const weapClass: usize = 24;
    pub const greebleType: usize = 28;
    pub const worldModels: usize = 32;
    pub const viewModels: usize = 40;
    pub const reticleViewModels: usize = 48;
    pub const chargeInfo: usize = 56;
    pub const hybridSettings: usize = 64;
    pub const fieldOffsets: usize = 72;
    pub const fields: usize = 80;
    pub const numFields: usize = 88;
    pub const loadIndex: usize = 92;
    pub const adsSettingsMode: usize = 96;
    pub const adsSceneBlurStrength: usize = 100;
    pub const knifeAttachTagOverride: usize = 104;
    pub const hideIronSightsWithThisAttachment: usize = 108;
    pub const showMasterRail: usize = 109;
    pub const showSideRail: usize = 110;
    pub const shareAmmoWithAlt: usize = 111;
    pub const knifeAlwaysAttached: usize = 112;
    pub const useDualFOV: usize = 113;
    pub const riotShield: usize = 114;
    pub const adsSceneBlur: usize = 115;
    pub const automaticAttachment: usize = 116;
}
pub mod WeaponDef {
    pub const SIZE: usize = 3560;
    pub const ALIGN: usize = 8;
    pub const szInternalName: usize = 0;
    pub const name: usize = 0;
    pub const szDisplayName: usize = 8;
    pub const szAltWeaponName: usize = 16;
    pub const gunModel: usize = 24;
    pub const handModel: usize = 32;
    pub const persistentArmXModel: usize = 40;
    pub const lobWorldModelName: usize = 48;
    pub const camoWorldModelMaterialOverride: usize = 56;
    pub const camoViewModelMaterialOverride: usize = 64;
    pub const szXAnimsRightHanded: usize = 72;
    pub const szXAnimsLeftHanded: usize = 80;
    pub const hideTags: usize = 88;
    pub const attachments: usize = 96;
    pub const szXAnims: usize = 104;
    pub const animOverrides: usize = 112;
    pub const soundOverrides: usize = 120;
    pub const fxOverrides: usize = 128;
    pub const reloadOverrides: usize = 136;
    pub const notetrackOverrides: usize = 144;
    pub const notetrackSoundMapKeys: usize = 152;
    pub const notetrackSoundMapValues: usize = 160;
    pub const notetrackRumbleMapKeys: usize = 168;
    pub const notetrackRumbleMapValues: usize = 176;
    pub const notetrackFXMapKeys: usize = 184;
    pub const notetrackFXMapValues: usize = 192;
    pub const notetrackFXMapTagValues: usize = 200;
    pub const szAdsrBaseSetting: usize = 208;
    pub const viewFlashEffect: usize = 216;
    pub const viewBodyFlashEffect: usize = 224;
    pub const worldFlashEffect: usize = 232;
    pub const viewFlashADSEffect: usize = 240;
    pub const viewBodyFlashADSEffect: usize = 248;
    pub const signatureViewFlashEffect: usize = 256;
    pub const signatureViewBodyFlashEffect: usize = 264;
    pub const signatureWorldFlashEffect: usize = 272;
    pub const signatureViewFlashADSEffect: usize = 280;
    pub const signatureViewBodyFlashADSEffect: usize = 288;
    pub const meleeHitEffect: usize = 296;
    pub const meleeMissEffect: usize = 304;
    pub const pickupSound: usize = 312;
    pub const pickupSoundPlayer: usize = 320;
    pub const ammoPickupSound: usize = 328;
    pub const ammoPickupSoundPlayer: usize = 336;
    pub const projectileSound: usize = 344;
    pub const pullbackSound: usize = 352;
    pub const pullbackSoundPlayer: usize = 360;
    pub const throwbackSound: usize = 368;
    pub const throwbackSoundPlayer: usize = 376;
    pub const fireSound: usize = 384;
    pub const fireSoundPlayer: usize = 392;
    pub const fireSoundPlayerAkimbo: usize = 400;
    pub const fireMedSound: usize = 408;
    pub const fireMedSoundPlayer: usize = 416;
    pub const fireHighSound: usize = 424;
    pub const fireHighSoundPlayer: usize = 432;
    pub const fireLoopSound: usize = 440;
    pub const fireLoopSoundPlayer: usize = 448;
    pub const fireMedLoopSound: usize = 456;
    pub const fireMedLoopSoundPlayer: usize = 464;
    pub const fireHighLoopSound: usize = 472;
    pub const fireHighLoopSoundPlayer: usize = 480;
    pub const fireLoopEndPointSound: usize = 488;
    pub const fireLoopEndPointSoundPlayer: usize = 496;
    pub const fireStopSound: usize = 504;
    pub const fireStopSoundPlayer: usize = 512;
    pub const fireMedStopSound: usize = 520;
    pub const fireMedStopSoundPlayer: usize = 528;
    pub const fireHighStopSound: usize = 536;
    pub const fireHighStopSoundPlayer: usize = 544;
    pub const fireLastSound: usize = 552;
    pub const fireLastSoundPlayer: usize = 560;
    pub const fireFirstSound: usize = 568;
    pub const fireFirstSoundPlayer: usize = 576;
    pub const fireCustomSound: usize = 584;
    pub const fireCustomSoundPlayer: usize = 592;
    pub const emptyFireSound: usize = 600;
    pub const emptyFireSoundPlayer: usize = 608;
    pub const adsRequiredFireSoundPlayer: usize = 616;
    pub const meleeSwipeSound: usize = 624;
    pub const meleeSwipeSoundPlayer: usize = 632;
    pub const meleeHitSound: usize = 640;
    pub const meleeMissSound: usize = 648;
    pub const meleeHitSoundPlayer: usize = 656;
    pub const meleeMissSoundPlayer: usize = 664;
    pub const rechamberSound: usize = 672;
    pub const rechamberSoundPlayer: usize = 680;
    pub const reloadSound: usize = 688;
    pub const reloadSoundPlayer: usize = 696;
    pub const reloadEmptySound: usize = 704;
    pub const reloadEmptySoundPlayer: usize = 712;
    pub const reloadStartSound: usize = 720;
    pub const reloadStartSoundPlayer: usize = 728;
    pub const reloadEndSound: usize = 736;
    pub const reloadEndSoundPlayer: usize = 744;
    pub const detonateSound: usize = 752;
    pub const detonateSoundPlayer: usize = 760;
    pub const nightVisionWearSound: usize = 768;
    pub const nightVisionWearSoundPlayer: usize = 776;
    pub const nightVisionRemoveSound: usize = 784;
    pub const nightVisionRemoveSoundPlayer: usize = 792;
    pub const altSwitchSound: usize = 800;
    pub const altSwitchSoundPlayer: usize = 808;
    pub const raiseSound: usize = 816;
    pub const raiseSoundPlayer: usize = 824;
    pub const firstRaiseSound: usize = 832;
    pub const firstRaiseSoundPlayer: usize = 840;
    pub const putawaySound: usize = 848;
    pub const putawaySoundPlayer: usize = 856;
    pub const scanSound: usize = 864;
    pub const changeVariableZoomSound: usize = 872;
    pub const adsUpSound: usize = 880;
    pub const adsDownSound: usize = 888;
    pub const adsCrosshairEnemySound: usize = 896;
    pub const bounceSound: usize = 904;
    pub const rollingSound: usize = 912;
    pub const viewShellEjectEffect: usize = 920;
    pub const worldShellEjectEffect: usize = 928;
    pub const viewLastShotEjectEffect: usize = 936;
    pub const worldLastShotEjectEffect: usize = 944;
    pub const viewMagEjectEffect: usize = 952;
    pub const reticleCenter: usize = 960;
    pub const reticleSide: usize = 968;
    pub const worldModel: usize = 976;
    pub const worldClipModel: usize = 984;
    pub const rocketModel: usize = 992;
    pub const knifeModel: usize = 1000;
    pub const worldKnifeModel: usize = 1008;
    pub const hudIcon: usize = 1016;
    pub const pickupIcon: usize = 1024;
    pub const minimapIconFriendly: usize = 1032;
    pub const minimapIconEnemy: usize = 1040;
    pub const minimapIconNeutral: usize = 1048;
    pub const ammoCounterIcon: usize = 1056;
    pub const szAmmoName: usize = 1064;
    pub const szClipName: usize = 1072;
    pub const szSharedAmmoCapName: usize = 1080;
    pub const physCollmap: usize = 1088;
    pub const physPreset: usize = 1096;
    pub const szUseHintString: usize = 1104;
    pub const dropHintString: usize = 1112;
    pub const locationDamageMultipliers: usize = 1120;
    pub const fireRumble: usize = 1128;
    pub const fireMedRumble: usize = 1136;
    pub const fireHighRumble: usize = 1144;
    pub const meleeImpactRumble: usize = 1152;
    pub const tracerType: usize = 1160;
    pub const signatureTracerType: usize = 1168;
    pub const laserType: usize = 1176;
    pub const turretOverheatSound: usize = 1184;
    pub const turretOverheatEffect: usize = 1192;
    pub const turretBarrelSpinRumble: usize = 1200;
    pub const turretBarrelSpinMaxSnd: usize = 1208;
    pub const turretBarrelSpinUpSnd: usize = 1216;
    pub const turretBarrelSpinDownSnd: usize = 1248;
    pub const missileConeSoundAlias: usize = 1280;
    pub const missileConeSoundAliasAtBase: usize = 1288;
    pub const stowOffsetModel: usize = 1296;
    pub const turretHydraulicSettings: usize = 1304;
    pub const altWeapon: usize = 1312;
    pub const numAttachments: usize = 1316;
    pub const numAnimOverrides: usize = 1317;
    pub const numSoundOverrides: usize = 1318;
    pub const numFXOverrides: usize = 1319;
    pub const numReloadStateTimerOverrides: usize = 1320;
    pub const numNotetrackOverrides: usize = 1321;
    pub const playerAnimType: usize = 1322;
    pub const __pad000: usize = 1323;
    pub const weapType: usize = 1324;
    pub const weapClass: usize = 1328;
    pub const penetrateType: usize = 1332;
    pub const penetrateDepth: usize = 1336;
    pub const impactType: usize = 1340;
    pub const inventoryType: usize = 1344;
    pub const fireType: usize = 1348;
    pub const fireBarrels: usize = 1352;
    pub const adsFireMode: usize = 1356;
    pub const burstFireCooldown: usize = 1360;
    pub const greebleType: usize = 1364;
    pub const autoReloadType: usize = 1368;
    pub const autoHolsterType: usize = 1372;
    pub const offhandClass: usize = 1376;
    pub const stance: usize = 1380;
    pub const reticleCenterSize: usize = 1384;
    pub const reticleSideSize: usize = 1388;
    pub const reticleMinOfs: usize = 1392;
    pub const activeReticleType: usize = 1396;
    pub const standMove: usize = 1400;
    pub const standRot: usize = 1412;
    pub const strafeMove: usize = 1424;
    pub const strafeRot: usize = 1436;
    pub const duckedOfs: usize = 1448;
    pub const duckedMove: usize = 1460;
    pub const duckedRot: usize = 1472;
    pub const proneOfs: usize = 1484;
    pub const proneMove: usize = 1496;
    pub const proneRot: usize = 1508;
    pub const posMoveRate: usize = 1520;
    pub const posProneMoveRate: usize = 1524;
    pub const standMoveMinSpeed: usize = 1528;
    pub const duckedMoveMinSpeed: usize = 1532;
    pub const proneMoveMinSpeed: usize = 1536;
    pub const posRotRate: usize = 1540;
    pub const posProneRotRate: usize = 1544;
    pub const hudIconRatio: usize = 1548;
    pub const pickupIconRatio: usize = 1552;
    pub const ammoCounterIconRatio: usize = 1556;
    pub const ammoCounterClip: usize = 1560;
    pub const startAmmo: usize = 1564;
    pub const iAmmoIndex: usize = 1568;
    pub const iAmmoIndex__ammoIndex: usize = 1568;
    pub const iAmmoIndex__isAlternate: usize = 1572;
    pub const iClipIndex: usize = 1576;
    pub const iClipIndex__clipIndex: usize = 1576;
    pub const iClipIndex__isAlternate: usize = 1580;
    pub const maxAmmo: usize = 1584;
    pub const minAmmoReq: usize = 1588;
    pub const clipSize: usize = 1592;
    pub const shotCount: usize = 1596;
    pub const sharedAmmoCapIndex: usize = 1600;
    pub const sharedAmmoCap: usize = 1604;
    pub const damage: usize = 1608;
    pub const playerDamage: usize = 1612;
    pub const meleeDamage: usize = 1616;
    pub const damageType: usize = 1620;
    pub const stateTimers: usize = 1624;
    pub const stateTimers__fireDelay: usize = 1624;
    pub const stateTimers__meleeDelay: usize = 1628;
    pub const stateTimers__meleeChargeDelay: usize = 1632;
    pub const stateTimers__detonateDelay: usize = 1636;
    pub const stateTimers__fireTime: usize = 1640;
    pub const stateTimers__rechamberTime: usize = 1644;
    pub const stateTimers__rechamberTimeOneHanded: usize = 1648;
    pub const stateTimers__rechamberBoltTime: usize = 1652;
    pub const stateTimers__holdFireTime: usize = 1656;
    pub const stateTimers__grenadePrimeReadyToThrowTime: usize = 1660;
    pub const stateTimers__detonateTime: usize = 1664;
    pub const stateTimers__meleeTime: usize = 1668;
    pub const stateTimers__meleeChargeTime: usize = 1672;
    pub const stateTimers__reloadTime: usize = 1676;
    pub const stateTimers__reloadShowRocketTime: usize = 1680;
    pub const stateTimers__reloadEmptyTime: usize = 1684;
    pub const stateTimers__reloadAddTime: usize = 1688;
    pub const stateTimers__reloadEmptyAddTime: usize = 1692;
    pub const stateTimers__reloadStartTime: usize = 1696;
    pub const stateTimers__reloadStartAddTime: usize = 1700;
    pub const stateTimers__reloadEndTime: usize = 1704;
    pub const stateTimers__reloadTimeDualWield: usize = 1708;
    pub const stateTimers__reloadAddTimeDualWield: usize = 1712;
    pub const stateTimers__reloadEmptyDualMag: usize = 1716;
    pub const stateTimers__reloadEmptyAddTimeDualMag: usize = 1720;
    pub const stateTimers__speedReloadTime: usize = 1724;
    pub const stateTimers__speedReloadAddTime: usize = 1728;
    pub const stateTimers__dropTime: usize = 1732;
    pub const stateTimers__raiseTime: usize = 1736;
    pub const stateTimers__altDropTime: usize = 1740;
    pub const stateTimers__altRaiseTime: usize = 1744;
    pub const stateTimers__quickDropTime: usize = 1748;
    pub const stateTimers__quickRaiseTime: usize = 1752;
    pub const stateTimers__firstRaiseTime: usize = 1756;
    pub const stateTimers__breachRaiseTime: usize = 1760;
    pub const stateTimers__emptyRaiseTime: usize = 1764;
    pub const stateTimers__emptyDropTime: usize = 1768;
    pub const stateTimers__sprintInTime: usize = 1772;
    pub const stateTimers__sprintLoopTime: usize = 1776;
    pub const stateTimers__sprintOutTime: usize = 1780;
    pub const stateTimers__stunnedTimeBegin: usize = 1784;
    pub const stateTimers__stunnedTimeLoop: usize = 1788;
    pub const stateTimers__stunnedTimeEnd: usize = 1792;
    pub const stateTimers__nightVisionWearTime: usize = 1796;
    pub const stateTimers__nightVisionWearTimeFadeOutEnd: usize = 1800;
    pub const stateTimers__nightVisionWearTimePowerUp: usize = 1804;
    pub const stateTimers__nightVisionRemoveTime: usize = 1808;
    pub const stateTimers__nightVisionRemoveTimePowerDown: usize = 1812;
    pub const stateTimers__nightVisionRemoveTimeFadeInStart: usize = 1816;
    pub const stateTimers__aiFuseTime: usize = 1820;
    pub const stateTimers__fuseTime: usize = 1824;
    pub const stateTimers__missileTime: usize = 1828;
    pub const stateTimers__primeTime: usize = 1832;
    pub const stateTimers__bHoldFullPrime: usize = 1836;
    pub const stateTimers__blastFrontTime: usize = 1840;
    pub const stateTimers__blastRightTime: usize = 1844;
    pub const stateTimers__blastBackTime: usize = 1848;
    pub const stateTimers__blastLeftTime: usize = 1852;
    pub const stateTimers__slideInTime: usize = 1856;
    pub const stateTimers__slideLoopTime: usize = 1860;
    pub const stateTimers__slideOutTime: usize = 1864;
    pub const stateTimers__highJumpInTime: usize = 1868;
    pub const stateTimers__highJumpDropInTime: usize = 1872;
    pub const stateTimers__highJumpDropLoopTime: usize = 1876;
    pub const stateTimers__highJumpDropLandTime: usize = 1880;
    pub const stateTimers__dodgeTime: usize = 1884;
    pub const stateTimers__landDipTime: usize = 1888;
    pub const stateTimers__hybridSightInTime: usize = 1892;
    pub const stateTimers__hybridSightOutTime: usize = 1896;
    pub const stateTimers__offhandSwitchTime: usize = 1900;
    pub const stateTimers__heatCooldownInTime: usize = 1904;
    pub const stateTimers__heatCooldownOutTime: usize = 1908;
    pub const stateTimers__heatCooldownOutReadyTime: usize = 1912;
    pub const stateTimers__overheatOutTime: usize = 1916;
    pub const stateTimers__overheatOutReadyTime: usize = 1920;
    pub const akimboStateTimers: usize = 1924;
    pub const akimboStateTimers__fireDelay: usize = 1924;
    pub const akimboStateTimers__meleeDelay: usize = 1928;
    pub const akimboStateTimers__meleeChargeDelay: usize = 1932;
    pub const akimboStateTimers__detonateDelay: usize = 1936;
    pub const akimboStateTimers__fireTime: usize = 1940;
    pub const akimboStateTimers__rechamberTime: usize = 1944;
    pub const akimboStateTimers__rechamberTimeOneHanded: usize = 1948;
    pub const akimboStateTimers__rechamberBoltTime: usize = 1952;
    pub const akimboStateTimers__holdFireTime: usize = 1956;
    pub const akimboStateTimers__grenadePrimeReadyToThrowTime: usize = 1960;
    pub const akimboStateTimers__detonateTime: usize = 1964;
    pub const akimboStateTimers__meleeTime: usize = 1968;
    pub const akimboStateTimers__meleeChargeTime: usize = 1972;
    pub const akimboStateTimers__reloadTime: usize = 1976;
    pub const akimboStateTimers__reloadShowRocketTime: usize = 1980;
    pub const akimboStateTimers__reloadEmptyTime: usize = 1984;
    pub const akimboStateTimers__reloadAddTime: usize = 1988;
    pub const akimboStateTimers__reloadEmptyAddTime: usize = 1992;
    pub const akimboStateTimers__reloadStartTime: usize = 1996;
    pub const akimboStateTimers__reloadStartAddTime: usize = 2000;
    pub const akimboStateTimers__reloadEndTime: usize = 2004;
    pub const akimboStateTimers__reloadTimeDualWield: usize = 2008;
    pub const akimboStateTimers__reloadAddTimeDualWield: usize = 2012;
    pub const akimboStateTimers__reloadEmptyDualMag: usize = 2016;
    pub const akimboStateTimers__reloadEmptyAddTimeDualMag: usize = 2020;
    pub const akimboStateTimers__speedReloadTime: usize = 2024;
    pub const akimboStateTimers__speedReloadAddTime: usize = 2028;
    pub const akimboStateTimers__dropTime: usize = 2032;
    pub const akimboStateTimers__raiseTime: usize = 2036;
    pub const akimboStateTimers__altDropTime: usize = 2040;
    pub const akimboStateTimers__altRaiseTime: usize = 2044;
    pub const akimboStateTimers__quickDropTime: usize = 2048;
    pub const akimboStateTimers__quickRaiseTime: usize = 2052;
    pub const akimboStateTimers__firstRaiseTime: usize = 2056;
    pub const akimboStateTimers__breachRaiseTime: usize = 2060;
    pub const akimboStateTimers__emptyRaiseTime: usize = 2064;
    pub const akimboStateTimers__emptyDropTime: usize = 2068;
    pub const akimboStateTimers__sprintInTime: usize = 2072;
    pub const akimboStateTimers__sprintLoopTime: usize = 2076;
    pub const akimboStateTimers__sprintOutTime: usize = 2080;
    pub const akimboStateTimers__stunnedTimeBegin: usize = 2084;
    pub const akimboStateTimers__stunnedTimeLoop: usize = 2088;
    pub const akimboStateTimers__stunnedTimeEnd: usize = 2092;
    pub const akimboStateTimers__nightVisionWearTime: usize = 2096;
    pub const akimboStateTimers__nightVisionWearTimeFadeOutEnd: usize = 2100;
    pub const akimboStateTimers__nightVisionWearTimePowerUp: usize = 2104;
    pub const akimboStateTimers__nightVisionRemoveTime: usize = 2108;
    pub const akimboStateTimers__nightVisionRemoveTimePowerDown: usize = 2112;
    pub const akimboStateTimers__nightVisionRemoveTimeFadeInStart: usize = 2116;
    pub const akimboStateTimers__aiFuseTime: usize = 2120;
    pub const akimboStateTimers__fuseTime: usize = 2124;
    pub const akimboStateTimers__missileTime: usize = 2128;
    pub const akimboStateTimers__primeTime: usize = 2132;
    pub const akimboStateTimers__bHoldFullPrime: usize = 2136;
    pub const akimboStateTimers__blastFrontTime: usize = 2140;
    pub const akimboStateTimers__blastRightTime: usize = 2144;
    pub const akimboStateTimers__blastBackTime: usize = 2148;
    pub const akimboStateTimers__blastLeftTime: usize = 2152;
    pub const akimboStateTimers__slideInTime: usize = 2156;
    pub const akimboStateTimers__slideLoopTime: usize = 2160;
    pub const akimboStateTimers__slideOutTime: usize = 2164;
    pub const akimboStateTimers__highJumpInTime: usize = 2168;
    pub const akimboStateTimers__highJumpDropInTime: usize = 2172;
    pub const akimboStateTimers__highJumpDropLoopTime: usize = 2176;
    pub const akimboStateTimers__highJumpDropLandTime: usize = 2180;
    pub const akimboStateTimers__dodgeTime: usize = 2184;
    pub const akimboStateTimers__landDipTime: usize = 2188;
    pub const akimboStateTimers__hybridSightInTime: usize = 2192;
    pub const akimboStateTimers__hybridSightOutTime: usize = 2196;
    pub const akimboStateTimers__offhandSwitchTime: usize = 2200;
    pub const akimboStateTimers__heatCooldownInTime: usize = 2204;
    pub const akimboStateTimers__heatCooldownOutTime: usize = 2208;
    pub const akimboStateTimers__heatCooldownOutReadyTime: usize = 2212;
    pub const akimboStateTimers__overheatOutTime: usize = 2216;
    pub const akimboStateTimers__overheatOutReadyTime: usize = 2220;
    pub const autoAimRange: usize = 2224;
    pub const aimAssistRange: usize = 2228;
    pub const aimAssistRangeAds: usize = 2232;
    pub const aimPadding: usize = 2236;
    pub const enemyCrosshairRange: usize = 2240;
    pub const moveSpeedScale: usize = 2244;
    pub const adsMoveSpeedScale: usize = 2248;
    pub const sprintDurationScale: usize = 2252;
    pub const adsZoomFov: usize = 2256;
    pub const adsZoomInFrac: usize = 2260;
    pub const adsZoomOutFrac: usize = 2264;
    pub const adsSceneBlur: usize = 2268;
    pub const overlay: usize = 2272;
    pub const overlay__shader: usize = 2272;
    pub const overlay__shaderLowRes: usize = 2280;
    pub const overlay__shaderEMP: usize = 2288;
    pub const overlay__shaderEMPLowRes: usize = 2296;
    pub const overlay__reticle: usize = 2304;
    pub const overlay__width: usize = 2308;
    pub const overlay__height: usize = 2312;
    pub const overlay__widthSplitscreen: usize = 2316;
    pub const overlay__heightSplitscreen: usize = 2320;
    pub const overlayInterface: usize = 2328;
    pub const adsBobFactor: usize = 2332;
    pub const adsViewBobMult: usize = 2336;
    pub const hipSpreadStandMin: usize = 2340;
    pub const hipSpreadDuckedMin: usize = 2344;
    pub const hipSpreadProneMin: usize = 2348;
    pub const hipSpreadStandMax: usize = 2352;
    pub const hipSpreadSprintMax: usize = 2356;
    pub const hipSpreadSlideMax: usize = 2360;
    pub const hipSpreadDuckedMax: usize = 2364;
    pub const hipSpreadProneMax: usize = 2368;
    pub const hipSpreadDecayRate: usize = 2372;
    pub const hipSpreadFireAdd: usize = 2376;
    pub const hipSpreadTurnAdd: usize = 2380;
    pub const hipSpreadMoveAdd: usize = 2384;
    pub const hipSpreadDuckedDecay: usize = 2388;
    pub const hipSpreadProneDecay: usize = 2392;
    pub const hipReticleSidePos: usize = 2396;
    pub const adsIdleAmount: usize = 2400;
    pub const hipIdleAmount: usize = 2404;
    pub const adsIdleSpeed: usize = 2408;
    pub const hipIdleSpeed: usize = 2412;
    pub const idleCrouchFactor: usize = 2416;
    pub const idleProneFactor: usize = 2420;
    pub const gunMaxPitch: usize = 2424;
    pub const gunMaxYaw: usize = 2428;
    pub const adsIdleLerpStartTime: usize = 2432;
    pub const adsIdleLerpTime: usize = 2436;
    pub const adsTransInTime: usize = 2440;
    pub const adsTransInFromSprintTime: usize = 2444;
    pub const adsTransOutTime: usize = 2448;
    pub const swayMaxAngleSteadyAim: usize = 2452;
    pub const swayMaxAngle: usize = 2456;
    pub const swayLerpSpeed: usize = 2460;
    pub const swayPitchScale: usize = 2464;
    pub const swayYawScale: usize = 2468;
    pub const swayHorizScale: usize = 2472;
    pub const swayVertScale: usize = 2476;
    pub const swayShellShockScale: usize = 2480;
    pub const adsSwayMaxAngle: usize = 2484;
    pub const adsSwayLerpSpeed: usize = 2488;
    pub const adsSwayPitchScale: usize = 2492;
    pub const adsSwayYawScale: usize = 2496;
    pub const adsSwayHorizScale: usize = 2500;
    pub const adsSwayVertScale: usize = 2504;
    pub const adsViewErrorMin: usize = 2508;
    pub const adsViewErrorMax: usize = 2512;
    pub const adsFireAnimFrac: usize = 2516;
    pub const dualWieldViewModelOffset: usize = 2520;
    pub const scopeDriftDelay: usize = 2524;
    pub const scopeDriftLerpInTime: usize = 2528;
    pub const scopeDriftSteadyTime: usize = 2532;
    pub const scopeDriftLerpOutTime: usize = 2536;
    pub const scopeDriftSteadyFactor: usize = 2540;
    pub const scopeDriftUnsteadyFactor: usize = 2544;
    pub const weaponBobVerticalFactor: usize = 2548;
    pub const weaponBobHorizontalFactor: usize = 2552;
    pub const viewBobVerticalFactor: usize = 2556;
    pub const viewBobHorizontalFactor: usize = 2560;
    pub const stationaryZoomFov: usize = 2564;
    pub const stationaryZoomDelayTime: usize = 2568;
    pub const stationaryZoomLerpInTime: usize = 2572;
    pub const stationaryZoomLerpOutTime: usize = 2576;
    pub const adsDofStart: usize = 2580;
    pub const adsDofEnd: usize = 2584;
    pub const killIcon: usize = 2592;
    pub const dpadIcon: usize = 2600;
    pub const hudProximityWarningIcon: usize = 2608;
    pub const killIconRatio: usize = 2616;
    pub const dpadIconRatio: usize = 2620;
    pub const fireAnimLength: usize = 2624;
    pub const fireAnimLengthAkimbo: usize = 2628;
    pub const reloadAmmoAdd: usize = 2632;
    pub const reloadStartAdd: usize = 2636;
    pub const ammoDropStockMin: usize = 2640;
    pub const ammoDropStockMax: usize = 2644;
    pub const ammoDropClipPercentMin: usize = 2648;
    pub const ammoDropClipPercentMax: usize = 2652;
    pub const explosionRadius: usize = 2656;
    pub const explosionRadiusMin: usize = 2660;
    pub const explosionInnerDamage: usize = 2664;
    pub const explosionOuterDamage: usize = 2668;
    pub const damageConeAngle: usize = 2672;
    pub const bulletExplDmgMult: usize = 2676;
    pub const bulletExplRadiusMult: usize = 2680;
    pub const projectileSpeed: usize = 2684;
    pub const projectileSpeedUp: usize = 2688;
    pub const projectileSpeedForward: usize = 2692;
    pub const projectileActivateDist: usize = 2696;
    pub const projLifetime: usize = 2700;
    pub const timeToAccelerate: usize = 2704;
    pub const projectileCurvature: usize = 2708;
    pub const projectileName: usize = 2712;
    pub const projectileModel: usize = 2720;
    pub const projExplosionEffect: usize = 2728;
    pub const projDudEffect: usize = 2736;
    pub const projExplosionSound: usize = 2744;
    pub const projDudSound: usize = 2752;
    pub const projExplosion: usize = 2760;
    pub const stickiness: usize = 2764;
    pub const lowAmmoWarningThreshold: usize = 2768;
    pub const ricochetChance: usize = 2772;
    pub const riotShieldHealth: usize = 2776;
    pub const riotShieldDamageMult: usize = 2780;
    pub const parallelBounce: usize = 2784;
    pub const perpendicularBounce: usize = 2792;
    pub const projTrailEffect: usize = 2800;
    pub const projBeaconEffect: usize = 2808;
    pub const projectileColor: usize = 2816;
    pub const guidedMissileType: usize = 2828;
    pub const maxSteeringAccel: usize = 2832;
    pub const projIgnitionDelay: usize = 2836;
    pub const projIgnitionEffect: usize = 2840;
    pub const projIgnitionSound: usize = 2848;
    pub const adsAimPitch: usize = 2856;
    pub const adsCrosshairInFrac: usize = 2860;
    pub const adsCrosshairOutFrac: usize = 2864;
    pub const adsGunKickReducedKickBullets: usize = 2868;
    pub const adsGunKickReducedKickPercent: usize = 2872;
    pub const adsGunKickPitchMin: usize = 2876;
    pub const adsGunKickPitchMax: usize = 2880;
    pub const adsGunKickYawMin: usize = 2884;
    pub const adsGunKickYawMax: usize = 2888;
    pub const adsGunKickMagMin: usize = 2892;
    pub const adsGunKickAccel: usize = 2896;
    pub const adsGunKickSpeedMax: usize = 2900;
    pub const adsGunKickSpeedDecay: usize = 2904;
    pub const adsGunKickStaticDecay: usize = 2908;
    pub const adsViewKickPitchMin: usize = 2912;
    pub const adsViewKickPitchMax: usize = 2916;
    pub const adsViewKickYawMin: usize = 2920;
    pub const adsViewKickYawMax: usize = 2924;
    pub const adsViewKickMagMin: usize = 2928;
    pub const adsViewKickCenterSpeed: usize = 2932;
    pub const adsViewScatterMin: usize = 2936;
    pub const adsViewScatterMax: usize = 2940;
    pub const adsSpread: usize = 2944;
    pub const hipGunKickReducedKickBullets: usize = 2948;
    pub const hipGunKickReducedKickPercent: usize = 2952;
    pub const hipGunKickPitchMin: usize = 2956;
    pub const hipGunKickPitchMax: usize = 2960;
    pub const hipGunKickYawMin: usize = 2964;
    pub const hipGunKickYawMax: usize = 2968;
    pub const hipGunKickMagMin: usize = 2972;
    pub const hipGunKickAccel: usize = 2976;
    pub const hipGunKickSpeedMax: usize = 2980;
    pub const hipGunKickSpeedDecay: usize = 2984;
    pub const hipGunKickStaticDecay: usize = 2988;
    pub const hipViewKickPitchMin: usize = 2992;
    pub const hipViewKickPitchMax: usize = 2996;
    pub const hipViewKickYawMin: usize = 3000;
    pub const hipViewKickYawMax: usize = 3004;
    pub const hipViewKickMagMin: usize = 3008;
    pub const hipViewKickCenterSpeed: usize = 3012;
    pub const hipViewScatterMin: usize = 3016;
    pub const hipViewScatterMax: usize = 3020;
    pub const viewKickScale: usize = 3024;
    pub const positionReloadTransTime: usize = 3028;
    pub const fightDist: usize = 3032;
    pub const maxDist: usize = 3036;
    pub const accuracyGraphName: usize = 3040;
    pub const accuracyGraphKnots: usize = 3056;
    pub const originalAccuracyGraphKnots: usize = 3072;
    pub const accuracyGraphKnotCount: usize = 3088;
    pub const leftArc: usize = 3092;
    pub const rightArc: usize = 3096;
    pub const topArc: usize = 3100;
    pub const bottomArc: usize = 3104;
    pub const accuracy: usize = 3108;
    pub const aiSpread: usize = 3112;
    pub const playerSpread: usize = 3116;
    pub const minTurnSpeed: usize = 3120;
    pub const maxTurnSpeed: usize = 3128;
    pub const pitchConvergenceTime: usize = 3136;
    pub const yawConvergenceTime: usize = 3140;
    pub const suppressTime: usize = 3144;
    pub const maxRange: usize = 3148;
    pub const animHorRotateInc: usize = 3152;
    pub const playerPositionDist: usize = 3156;
    pub const useHintStringIndex: usize = 3160;
    pub const dropHintStringIndex: usize = 3164;
    pub const horizViewJitter: usize = 3168;
    pub const vertViewJitter: usize = 3172;
    pub const scanSpeed: usize = 3176;
    pub const scanAccel: usize = 3180;
    pub const scanPauseTime: usize = 3184;
    pub const szScript: usize = 3192;
    pub const minDamage: usize = 3200;
    pub const midDamage: usize = 3204;
    pub const minPlayerDamage: usize = 3208;
    pub const midPlayerDamage: usize = 3212;
    pub const maxDamageRange: usize = 3216;
    pub const minDamageRange: usize = 3220;
    pub const signatureAmmoInClip: usize = 3224;
    pub const signatureDamage: usize = 3228;
    pub const signatureMidDamage: usize = 3232;
    pub const signatureMinDamage: usize = 3236;
    pub const signatureMaxDamageRange: usize = 3240;
    pub const signatureMinDamageRange: usize = 3244;
    pub const destabilizationRateTime: usize = 3248;
    pub const destabilizationCurvatureMax: usize = 3252;
    pub const destabilizeDistance: usize = 3256;
    pub const turretADSTime: usize = 3260;
    pub const turretFov: usize = 3264;
    pub const turretFovADS: usize = 3268;
    pub const turretScopeZoomRate: usize = 3272;
    pub const turretScopeZoomMin: usize = 3276;
    pub const turretScopeZoomMax: usize = 3280;
    pub const overheatUpRate: usize = 3284;
    pub const overheatDownRate: usize = 3288;
    pub const overheatCooldownRate: usize = 3292;
    pub const overheatPenalty: usize = 3296;
    pub const turretBarrelSpinSpeed: usize = 3300;
    pub const turretBarrelSpinUpTime: usize = 3304;
    pub const turretBarrelSpinDownTime: usize = 3308;
    pub const missileConeSoundRadiusAtTop: usize = 3312;
    pub const missileConeSoundRadiusAtBase: usize = 3316;
    pub const missileConeSoundHeight: usize = 3320;
    pub const missileConeSoundOriginOffset: usize = 3324;
    pub const missileConeSoundVolumescaleAtCore: usize = 3328;
    pub const missileConeSoundVolumescaleAtEdge: usize = 3332;
    pub const missileConeSoundVolumescaleCoreSize: usize = 3336;
    pub const missileConeSoundPitchAtTop: usize = 3340;
    pub const missileConeSoundPitchAtBottom: usize = 3344;
    pub const missileConeSoundPitchTopSize: usize = 3348;
    pub const missileConeSoundPitchBottomSize: usize = 3352;
    pub const missileConeSoundCrossfadeTopSize: usize = 3356;
    pub const missileConeSoundCrossfadeBottomSize: usize = 3360;
    pub const knifeAttachTagOverride: usize = 3364;
    pub const aim_automelee_lerp: usize = 3368;
    pub const aim_automelee_range: usize = 3372;
    pub const aim_automelee_region_height: usize = 3376;
    pub const aim_automelee_region_width: usize = 3380;
    pub const player_meleeHeight: usize = 3384;
    pub const player_meleeRange: usize = 3388;
    pub const player_meleeWidth: usize = 3392;
    pub const changedFireTime: usize = 3396;
    pub const changedFireTimeNumBullets: usize = 3400;
    pub const fireTimeInterpolationType: usize = 3404;
    pub const generateAmmo: usize = 3408;
    pub const ammoPerShot: usize = 3412;
    pub const explodeCount: usize = 3416;
    pub const batteryDischargeRate: usize = 3420;
    pub const extendedBattery: usize = 3424;
    pub const stowTag: usize = 3428;
    pub const rattleSoundType: usize = 3432;
    pub const adsShouldShowCrosshair: usize = 3433;
    pub const adsCrosshairShouldScale: usize = 3434;
    pub const turretADSEnabled: usize = 3435;
    pub const knifeAlwaysAttached: usize = 3436;
    pub const meleeOverrideValues: usize = 3437;
    pub const riotShieldEnableDamage: usize = 3438;
    pub const allowPrimaryWeaponPickup: usize = 3439;
    pub const sharedAmmo: usize = 3440;
    pub const lockonSupported: usize = 3441;
    pub const requireLockonToFire: usize = 3442;
    pub const isAirburstWeapon: usize = 3443;
    pub const bigExplosion: usize = 3444;
    pub const noAdsWhenMagEmpty: usize = 3445;
    pub const avoidDropCleanup: usize = 3446;
    pub const inheritsPerks: usize = 3447;
    pub const crosshairColorChange: usize = 3448;
    pub const rifleBullet: usize = 3449;
    pub const armorPiercing: usize = 3450;
    pub const boltAction: usize = 3451;
    pub const aimDownSight: usize = 3452;
    pub const canHoldBreath: usize = 3453;
    pub const meleeOnly: usize = 3454;
    pub const altMelee: usize = 3455;
    pub const canVariableZoom: usize = 3456;
    pub const rechamberWhileAds: usize = 3457;
    pub const bulletExplosiveDamage: usize = 3458;
    pub const cookOffHold: usize = 3459;
    pub const useBattery: usize = 3460;
    pub const reticleSpin45: usize = 3461;
    pub const clipOnly: usize = 3462;
    pub const noAmmoPickup: usize = 3463;
    pub const disableSwitchToWhenEmpty: usize = 3464;
    pub const suppressAmmoReserveDisplay: usize = 3465;
    pub const motionTracker: usize = 3466;
    pub const markableViewmodel: usize = 3467;
    pub const noDualWield: usize = 3468;
    pub const flipKillIcon: usize = 3469;
    pub const dpadIconShowsAmmo: usize = 3470;
    pub const noPartialReload: usize = 3471;
    pub const segmentedReload: usize = 3472;
    pub const multipleReload: usize = 3473;
    pub const blocksProne: usize = 3474;
    pub const silenced: usize = 3475;
    pub const isRollingGrenade: usize = 3476;
    pub const projExplosionEffectForceNormalUp: usize = 3477;
    pub const projExplosionEffectInheritParentDirection: usize = 3478;
    pub const projImpactExplode: usize = 3479;
    pub const projTrajectoryEvents: usize = 3480;
    pub const projWhizByEnabled: usize = 3481;
    pub const stickToPlayers: usize = 3482;
    pub const stickToVehicles: usize = 3483;
    pub const stickToTurrets: usize = 3484;
    pub const thrownSideways: usize = 3485;
    pub const detonatesOnEmptyThrow: usize = 3486;
    pub const detonatesOnDoubleTap: usize = 3487;
    pub const disableFiring: usize = 3488;
    pub const timedDetonation: usize = 3489;
    pub const noCrumpleMissile: usize = 3490;
    pub const fuseLitAfterImpact: usize = 3491;
    pub const rotate: usize = 3492;
    pub const holdButtonToThrow: usize = 3493;
    pub const freezeMovementWhenFiring: usize = 3494;
    pub const thermalScope: usize = 3495;
    pub const thermalToggle: usize = 3496;
    pub const outlineEnemies: usize = 3497;
    pub const altModeSameWeapon: usize = 3498;
    pub const turretBarrelSpinEnabled: usize = 3499;
    pub const missileConeSoundEnabled: usize = 3500;
    pub const missileConeSoundPitchshiftEnabled: usize = 3501;
    pub const missileConeSoundCrossfadeEnabled: usize = 3502;
    pub const offhandHoldIsCancelable: usize = 3503;
    pub const doNotAllowAttachmentsToOverrideSpread: usize = 3504;
    pub const useFastReloadAnims: usize = 3505;
    pub const useDualMagReloadAnims: usize = 3506;
    pub const reloadStopsAlt: usize = 3507;
    pub const useScopeDrift: usize = 3508;
    pub const alwaysShatterGlassOnImpact: usize = 3509;
    pub const oldWeapon: usize = 3510;
    pub const raiseToHold: usize = 3511;
    pub const notifyOnPlayerImpact: usize = 3512;
    pub const decreasingKick: usize = 3513;
    pub const counterSilencer: usize = 3514;
    pub const projSuppressedByEMP: usize = 3515;
    pub const projDisabledByEMP: usize = 3516;
    pub const autosimDisableVariableRate: usize = 3517;
    pub const projPlayTrailEffectForOwnerOnly: usize = 3518;
    pub const projPlayBeaconEffectForOwnerOnly: usize = 3519;
    pub const projKillTrailEffectOnDeath: usize = 3520;
    pub const projKillBeaconEffectOnDeath: usize = 3521;
    pub const reticleDetonateHide: usize = 3522;
    pub const cloaked: usize = 3523;
    pub const adsHideWeapon: usize = 3524;
    pub const hasTransientModels: usize = 3525;
    pub const signatureAmmoAlternate: usize = 3526;
    pub const useScriptCallbackForHit: usize = 3527;
    pub const adsDofPhysicalFstop: usize = 3528;
    pub const adsDofPhysicalFocusDistance: usize = 3532;
    pub const autosimSpeedScale: usize = 3536;
    pub const reactiveMotionRadiusScale: usize = 3540;
    pub const reactiveMotionFrequencyScale: usize = 3544;
    pub const reactiveMotionAmplitudeScale: usize = 3548;
    pub const reactiveMotionFalloff: usize = 3552;
    pub const reactiveMotionLifetime: usize = 3556;
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
    pub const SIZE: usize = 216;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const dataByteCount: usize = 8;
    pub const dataShortCount: usize = 10;
    pub const dataIntCount: usize = 12;
    pub const numframes: usize = 14;
    pub const flags: usize = 16;
    pub const boneCount: usize = 17;
    pub const notifyCount: usize = 29;
    pub const assetType: usize = 30;
    pub const ikType: usize = 31;
    pub const randomDataByteCount: usize = 32;
    pub const randomDataShortCount: usize = 36;
    pub const randomDataIntCount: usize = 40;
    pub const indexCount: usize = 44;
    pub const framerate: usize = 48;
    pub const frequency: usize = 52;
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
    pub const secondaryName: usize = 136;
    pub const alternateAnimWeight: usize = 144;
    pub const numAlternatives: usize = 145;
    pub const blendShapeCount: usize = 146;
    pub const pad: usize = 148;
    pub const blendShapeWeightNames: usize = 152;
    pub const blendShapeCoefficientMagnitudes: usize = 160;
    pub const numberOfBlendShapeKeys: usize = 168;
    pub const blendShapeKeys: usize = 176;
    pub const compressedBlendShapeCoefficients: usize = 184;
    pub const blendShapeWeights: usize = 192;
    pub const alternativeAnim: usize = 200;
    pub const svAmimData: usize = 208;
}
pub mod XAnimScriptedViewmodelAnimData {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const blendIn: usize = 0;
    pub const blendOut: usize = 4;
}
pub mod XAsset {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const r#type: usize = 0;
    pub const header: usize = 8;
    pub const header__data: usize = 8;
    pub const header__physPreset: usize = 8;
    pub const header__physCollmap: usize = 8;
    pub const header__physWaterPreset: usize = 8;
    pub const header__physWorld: usize = 8;
    pub const header__physConstraint: usize = 8;
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
    pub const header__sndContext: usize = 8;
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
    pub const header__doppler: usize = 8;
    pub const header__particleSimAnimation: usize = 8;
    pub const header__laser: usize = 8;
    pub const header__skeletonScript: usize = 8;
    pub const header__clut: usize = 8;
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
    pub const asset__header__physWaterPreset: usize = 8;
    pub const asset__header__physWorld: usize = 8;
    pub const asset__header__physConstraint: usize = 8;
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
    pub const asset__header__sndContext: usize = 8;
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
    pub const asset__header__doppler: usize = 8;
    pub const asset__header__particleSimAnimation: usize = 8;
    pub const asset__header__laser: usize = 8;
    pub const asset__header__skeletonScript: usize = 8;
    pub const asset__header__clut: usize = 8;
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
    pub const physWaterPreset: usize = 0;
    pub const physWorld: usize = 0;
    pub const physConstraint: usize = 0;
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
    pub const sndContext: usize = 0;
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
    pub const doppler: usize = 0;
    pub const particleSimAnimation: usize = 0;
    pub const laser: usize = 0;
    pub const skeletonScript: usize = 0;
    pub const clut: usize = 0;
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
pub mod XBoneInfo {
    pub const SIZE: usize = 28;
    pub const ALIGN: usize = 4;
    pub const bounds: usize = 0;
    pub const bounds__midPoint: usize = 0;
    pub const bounds__halfSize: usize = 12;
    pub const bounds__radiusSquared: usize = 24;
    pub const bounds__radiusSquaredAsInt: usize = 24;
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
    pub const SIZE: usize = 680;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const numBones: usize = 8;
    pub const numRootBones: usize = 9;
    pub const numsurfs: usize = 10;
    pub const numReactiveMotionParts: usize = 11;
    pub const lodRampType: usize = 12;
    pub const numBonePhysics: usize = 13;
    pub const __pad0: usize = 14;
    pub const scale: usize = 16;
    pub const noScalePartBits: usize = 20;
    pub const boneNames: usize = 56;
    pub const parentList: usize = 64;
    pub const tagAngles: usize = 72;
    pub const tagPositions: usize = 80;
    pub const partClassification: usize = 88;
    pub const baseMat: usize = 96;
    pub const reactiveMotionParts: usize = 104;
    pub const reactiveMotionTweaks: usize = 112;
    pub const materialHandles: usize = 120;
    pub const lodInfo: usize = 128;
    pub const numLods: usize = 512;
    pub const collLod: usize = 513;
    pub const numCompositeModels: usize = 514;
    pub const reactiveMotionLOD: usize = 515;
    pub const flags: usize = 516;
    pub const numCollSurfs: usize = 518;
    pub const collSurfs: usize = 520;
    pub const contents: usize = 528;
    pub const boneInfo: usize = 536;
    pub const radius: usize = 544;
    pub const bounds: usize = 548;
    pub const bounds__midPoint: usize = 548;
    pub const bounds__halfSize: usize = 560;
    pub const invHighMipRadius: usize = 576;
    pub const memUsage: usize = 584;
    pub const bad: usize = 588;
    pub const pad: usize = 589;
    pub const blendShapeCount: usize = 590;
    pub const numberOfBlendShapeWeights: usize = 592;
    pub const numberOfBlendShapeWeightMaps: usize = 594;
    pub const __pad2: usize = 596;
    pub const blendShapeWeightNames: usize = 600;
    pub const blendShapeWeightMaps: usize = 608;
    pub const physPreset: usize = 616;
    pub const physCollmap: usize = 624;
    pub const mdaoVolumeCount: usize = 632;
    pub const u3: usize = 634;
    pub const quantization: usize = 636;
    pub const mdaoVolumes: usize = 640;
    pub const u4: usize = 648;
    pub const u5: usize = 652;
    pub const skeletonScript: usize = 656;
    pub const compositeModels: usize = 664;
    pub const bonePhysics: usize = 672;
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
pub mod XModelIKData {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 8;
    pub const numSolvers: usize = 0;
    pub const numFloatParams: usize = 1;
    pub const numIntParams: usize = 2;
    pub const numStringParams: usize = 3;
    pub const solverIDs: usize = 8;
    pub const floatParams: usize = 16;
    pub const intParams: usize = 24;
    pub const stringParams: usize = 32;
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
    pub const unk: usize = 56;
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
pub mod XPhysBoneInfo {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const physPreset: usize = 0;
    pub const physContraint: usize = 8;
    pub const physCollmap: usize = 16;
    pub const __pad0: usize = 24;
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
    pub const SIZE: usize = 264;
    pub const ALIGN: usize = 8;
    pub const flags: usize = 0;
    pub const vertCount: usize = 2;
    pub const triCount: usize = 4;
    pub const rigidVertListCount: usize = 6;
    pub const subdivLevelCount: usize = 7;
    pub const blendVertCounts: usize = 8;
    pub const verts0: usize = 24;
    pub const verts0__packedVerts0: usize = 24;
    pub const verts0__packedMotionVerts0: usize = 24;
    pub const verts0__verts0: usize = 24;
    pub const triIndices: usize = 32;
    pub const triIndices2: usize = 40;
    pub const vb0: usize = 48;
    pub const vb0View: usize = 56;
    pub const indexBuffer: usize = 64;
    pub const rigidVertLists: usize = 72;
    pub const unknown0: usize = 80;
    pub const blendVerts: usize = 88;
    pub const blendVertsTable: usize = 96;
    pub const blendVertsBuffer: usize = 104;
    pub const blendVertsView: usize = 112;
    pub const lmapUnwrap: usize = 120;
    pub const vblmapBuffer: usize = 128;
    pub const vblmapView: usize = 136;
    pub const subdiv: usize = 144;
    pub const tensionData: usize = 152;
    pub const tensionAccumTable: usize = 160;
    pub const tensionAccumTableBuffer: usize = 168;
    pub const tensionAccumTableView: usize = 176;
    pub const tensionDataBuffer: usize = 184;
    pub const tensionDataView: usize = 192;
    pub const indexBufferView: usize = 200;
    pub const blendShapes: usize = 208;
    pub const blendShapesCount: usize = 216;
    pub const __pad0: usize = 220;
    pub const vertexLightingIndex: usize = 224;
    pub const partBits: usize = 228;
    pub const __pad1: usize = 260;
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
    pub const __pad0: usize = 8;
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
pub mod XZone {
    pub const SIZE: usize = 496;
    pub const ALIGN: usize = 1;
    pub const __pad0: usize = 0;
    pub const name: usize = 24;
    pub const __pad1: usize = 88;
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
pub mod ammoindex_t {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const ammoIndex: usize = 0;
    pub const isAlternate: usize = 4;
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
    pub const SIZE: usize = 44;
    pub const ALIGN: usize = 4;
    pub const firstCollAabbIndex: usize = 0;
    pub const collAabbCount: usize = 4;
    pub const brushContents: usize = 8;
    pub const terrainContents: usize = 12;
    pub const bounds: usize = 16;
    pub const bounds__midPoint: usize = 16;
    pub const bounds__halfSize: usize = 28;
    pub const leafBrushNode: usize = 40;
}
pub mod cNode_t {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 8;
    pub const plane: usize = 0;
    pub const children: usize = 8;
}
pub mod cStaticModel_s {
    pub const SIZE: usize = 88;
    pub const ALIGN: usize = 8;
    pub const xmodel: usize = 0;
    pub const origin: usize = 8;
    pub const invScaledAxis: usize = 20;
    pub const absBounds: usize = 56;
    pub const absBounds__midPoint: usize = 56;
    pub const absBounds__halfSize: usize = 68;
    pub const lightingHandle: usize = 80;
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
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const planeIndex: usize = 0;
    pub const materialNum: usize = 4;
    pub const firstAdjacentSideOffset: usize = 6;
    pub const edgeCount: usize = 7;
}
pub mod clipMap_t {
    pub const SIZE: usize = 640;
    pub const ALIGN: usize = 8;
    pub const name: usize = 0;
    pub const isInUse: usize = 8;
    pub const info: usize = 16;
    pub const info__planeCount: usize = 16;
    pub const info__planes: usize = 24;
    pub const info__numMaterials: usize = 32;
    pub const info__materials: usize = 40;
    pub const info__bCollisionTree: usize = 48;
    pub const info__bCollisionTree__leafbrushNodesCount: usize = 48;
    pub const info__bCollisionTree__leafbrushNodes: usize = 56;
    pub const info__bCollisionTree__numLeafBrushes: usize = 64;
    pub const info__bCollisionTree__leafbrushes: usize = 72;
    pub const info__pCollisionTree: usize = 80;
    pub const info__pCollisionTree__aabbTreeCount: usize = 80;
    pub const info__pCollisionTree__aabbTrees: usize = 88;
    pub const info__sCollisionTree: usize = 96;
    pub const info__sCollisionTree__numStaticModels: usize = 96;
    pub const info__sCollisionTree__smodelNodeCount: usize = 98;
    pub const info__sCollisionTree__smodelNodes: usize = 104;
    pub const info__bCollisionData: usize = 112;
    pub const info__bCollisionData__numBrushSides: usize = 112;
    pub const info__bCollisionData__brushSides: usize = 120;
    pub const info__bCollisionData__numBrushEdges: usize = 128;
    pub const info__bCollisionData__brushEdges: usize = 136;
    pub const info__bCollisionData__numBrushes: usize = 144;
    pub const info__bCollisionData__brushes: usize = 152;
    pub const info__bCollisionData__brushBounds: usize = 160;
    pub const info__bCollisionData__brushContents: usize = 168;
    pub const info__pCollisionData: usize = 176;
    pub const info__pCollisionData__vertCount: usize = 176;
    pub const info__pCollisionData__verts: usize = 184;
    pub const info__pCollisionData__triCount: usize = 192;
    pub const info__pCollisionData__triIndices: usize = 200;
    pub const info__pCollisionData__triEdgeIsWalkable: usize = 208;
    pub const info__pCollisionData__borderCount: usize = 216;
    pub const info__pCollisionData__borders: usize = 224;
    pub const info__pCollisionData__partitionCount: usize = 232;
    pub const info__pCollisionData__partitions: usize = 240;
    pub const info__sCollisionData: usize = 248;
    pub const info__sCollisionData__numStaticModels: usize = 248;
    pub const info__sCollisionData__staticModelList: usize = 256;
    pub const pInfo: usize = 264;
    pub const numNodes: usize = 272;
    pub const nodes: usize = 280;
    pub const numLeafs: usize = 288;
    pub const leafs: usize = 296;
    pub const numSubModels: usize = 304;
    pub const cmodels: usize = 312;
    pub const mapEnts: usize = 320;
    pub const stages: usize = 328;
    pub const stageCount: usize = 336;
    pub const stageTrigger: usize = 344;
    pub const stageTrigger__count: usize = 344;
    pub const stageTrigger__models: usize = 352;
    pub const stageTrigger__hullCount: usize = 360;
    pub const stageTrigger__hulls: usize = 368;
    pub const stageTrigger__slabCount: usize = 376;
    pub const stageTrigger__slabs: usize = 384;
    pub const dynEntCount: usize = 392;
    pub const dynEntDefList: usize = 400;
    pub const dynEntPoseList: usize = 416;
    pub const dynEntClientList: usize = 432;
    pub const dynEntCollList: usize = 448;
    pub const dynEntAnchorCount: usize = 464;
    pub const dynEntAnchorNames: usize = 472;
    pub const scriptableMapEnts: usize = 480;
    pub const scriptableMapEnts__instanceStateSize: usize = 480;
    pub const scriptableMapEnts__instanceCount: usize = 484;
    pub const scriptableMapEnts__reservedInstanceCount: usize = 488;
    pub const scriptableMapEnts__instances: usize = 496;
    pub const scriptableMapEnts__animEntryCount: usize = 504;
    pub const scriptableMapEnts__animEntries: usize = 512;
    pub const scriptableMapEnts__replicatedInstanceCount: usize = 520;
    pub const grapple: usize = 528;
    pub const grapple__magnetTree: usize = 528;
    pub const grapple__magnetTree__treeCount: usize = 528;
    pub const grapple__magnetTree__tree: usize = 536;
    pub const grapple__magnetTree__numObjects: usize = 544;
    pub const grapple__magnetTree__objects: usize = 552;
    pub const grapple__magnets: usize = 560;
    pub const grapple__magnetCount: usize = 568;
    pub const checksum: usize = 576;
    pub const __pad0: usize = 580;
}
pub mod clipindex_t {
    pub const SIZE: usize = 8;
    pub const ALIGN: usize = 4;
    pub const clipIndex: usize = 0;
    pub const isAlternate: usize = 4;
}
pub mod cmodel_t {
    pub const SIZE: usize = 88;
    pub const ALIGN: usize = 8;
    pub const bounds: usize = 0;
    pub const bounds__midPoint: usize = 0;
    pub const bounds__halfSize: usize = 12;
    pub const radius: usize = 24;
    pub const info: usize = 32;
    pub const leaf: usize = 40;
    pub const leaf__firstCollAabbIndex: usize = 40;
    pub const leaf__collAabbCount: usize = 44;
    pub const leaf__brushContents: usize = 48;
    pub const leaf__terrainContents: usize = 52;
    pub const leaf__bounds: usize = 56;
    pub const leaf__bounds__midPoint: usize = 56;
    pub const leaf__bounds__halfSize: usize = 68;
    pub const leaf__leafBrushNode: usize = 80;
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
    pub const pad: usize = 17;
}
pub mod dmFloat3 {
    pub const SIZE: usize = 12;
    pub const ALIGN: usize = 4;
    pub const x: usize = 0;
    pub const y: usize = 4;
    pub const z: usize = 8;
}
pub mod dmFloat4 {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const x: usize = 0;
    pub const y: usize = 4;
    pub const z: usize = 8;
    pub const w: usize = 12;
}
pub mod dmMeshData {
    pub const SIZE: usize = 80;
    pub const ALIGN: usize = 8;
    pub const m_pRoot: usize = 0;
    pub const m_aVertices: usize = 8;
    pub const m_aTriangles: usize = 16;
    pub const m_center: usize = 24;
    pub const m_center__x: usize = 24;
    pub const m_center__y: usize = 28;
    pub const m_center__z: usize = 32;
    pub const m_extents: usize = 36;
    pub const m_extents__x: usize = 36;
    pub const m_extents__y: usize = 40;
    pub const m_extents__z: usize = 44;
    pub const m_unquantize: usize = 48;
    pub const m_unquantize__x: usize = 48;
    pub const m_unquantize__y: usize = 52;
    pub const m_unquantize__z: usize = 56;
    pub const m_nodeCount: usize = 60;
    pub const m_vertexCount: usize = 64;
    pub const m_triangleCount: usize = 68;
    pub const m_height: usize = 72;
    pub const contents: usize = 76;
}
pub mod dmMeshNode {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const lowerX: usize = 0;
    pub const lowerY: usize = 2;
    pub const lowerZ: usize = 4;
    pub const upperX: usize = 6;
    pub const upperY: usize = 8;
    pub const upperZ: usize = 10;
    pub const anon: usize = 12;
    pub const anon__fields: usize = 12;
    pub const anon__fields__axis: usize = 12;
    pub const anon__fields__triangleCount: usize = 12;
    pub const anon__fields__index: usize = 12;
    pub const anon__packed: usize = 12;
}
pub mod dmMeshNode_anon {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 4;
    pub const fields: usize = 0;
    pub const fields__axis: usize = 0;
    pub const fields__triangleCount: usize = 0;
    pub const fields__index: usize = 0;
    pub const packed: usize = 0;
}
pub mod dmMeshNode_anon_fields {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 4;
    pub const axis: usize = 0;
    pub const triangleCount: usize = 0;
    pub const index: usize = 0;
}
pub mod dmMeshTriangle {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 4;
    pub const i1: usize = 0;
    pub const i2: usize = 4;
    pub const i3: usize = 8;
    pub const w1: usize = 12;
    pub const w2: usize = 16;
    pub const w3: usize = 20;
    pub const materialIndex: usize = 24;
    pub const collisionFlags: usize = 28;
}
pub mod dmPlane {
    pub const SIZE: usize = 16;
    pub const ALIGN: usize = 4;
    pub const normal: usize = 0;
    pub const normal__x: usize = 0;
    pub const normal__y: usize = 4;
    pub const normal__z: usize = 8;
    pub const offset: usize = 12;
}
pub mod dmPolytopeData {
    pub const SIZE: usize = 112;
    pub const ALIGN: usize = 8;
    pub const m_aVertices: usize = 0;
    pub const m_aPlanes: usize = 8;
    pub const m_surfaceTypes: usize = 16;
    pub const m_vertexMaterials: usize = 24;
    pub const m_aSubEdges: usize = 32;
    pub const m_aFaceSubEdges: usize = 40;
    pub const m_centroid: usize = 48;
    pub const m_centroid__x: usize = 48;
    pub const m_centroid__y: usize = 52;
    pub const m_centroid__z: usize = 56;
    pub const m_vertexCount: usize = 60;
    pub const m_faceCount: usize = 64;
    pub const m_subEdgeCount: usize = 68;
    pub const m_volume: usize = 72;
    pub const m_area: usize = 76;
    pub const unk1: usize = 80;
    pub const unk1__x: usize = 80;
    pub const unk1__y: usize = 84;
    pub const unk1__z: usize = 88;
    pub const unk2: usize = 92;
    pub const unk2__x: usize = 92;
    pub const unk2__y: usize = 96;
    pub const unk2__z: usize = 100;
    pub const contents: usize = 104;
    pub const pad2: usize = 108;
}
pub mod dmSubEdge {
    pub const SIZE: usize = 4;
    pub const ALIGN: usize = 1;
    pub const twinOffset: usize = 0;
    pub const tail: usize = 1;
    pub const left: usize = 2;
    pub const next: usize = 3;
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
pub mod grapple_data_t {
    pub const SIZE: usize = 48;
    pub const ALIGN: usize = 8;
    pub const magnetTree: usize = 0;
    pub const magnetTree__treeCount: usize = 0;
    pub const magnetTree__tree: usize = 8;
    pub const magnetTree__numObjects: usize = 16;
    pub const magnetTree__objects: usize = 24;
    pub const magnets: usize = 32;
    pub const magnetCount: usize = 40;
}
pub mod grapple_magnet_t {
    pub const SIZE: usize = 40;
    pub const ALIGN: usize = 4;
    pub const flags: usize = 0;
    pub const origin: usize = 4;
    pub const normal: usize = 16;
    pub const length: usize = 28;
    pub const next: usize = 32;
    pub const prev: usize = 36;
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
    pub const SIZE: usize = 88;
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
    pub const pathAbility: usize = 68;
    pub const __pad0: usize = 69;
    pub const totalLinkCount: usize = 70;
    pub const Links: usize = 72;
    pub const remoteMissileEnt: usize = 80;
    pub const __pad1: usize = 81;
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
    pub const SIZE: usize = 184;
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
    pub const constant__pathAbility: usize = 68;
    pub const constant____pad0: usize = 69;
    pub const constant__totalLinkCount: usize = 70;
    pub const constant__Links: usize = 72;
    pub const constant__remoteMissileEnt: usize = 80;
    pub const constant____pad1: usize = 81;
    pub const dynamic: usize = 88;
    pub const dynamic__pOwner: usize = 88;
    pub const dynamic__pOwner__number: usize = 88;
    pub const dynamic__pOwner__infoIndex: usize = 90;
    pub const dynamic__iFreeTime: usize = 92;
    pub const dynamic__iValidTime: usize = 96;
    pub const dynamic__wLinkCount: usize = 108;
    pub const dynamic__wOverlapCount: usize = 110;
    pub const dynamic__turretEntNumber: usize = 112;
    pub const dynamic__userCount: usize = 114;
    pub const dynamic__hasBadPlaceLink: usize = 115;
    pub const dynamic__spreadUsedTime: usize = 116;
    pub const dynamic__flags: usize = 124;
    pub const dynamic__dangerousCount: usize = 126;
    pub const dynamic__recentUseProxTime: usize = 128;
    pub const transient: usize = 136;
    pub const transient__iSearchFrame: usize = 136;
    pub const transient__pNextOpen: usize = 144;
    pub const transient__pPrevOpen: usize = 152;
    pub const transient__pParent: usize = 160;
    pub const transient__fCost: usize = 168;
    pub const transient__fHeuristic: usize = 172;
    pub const transient_____u6: usize = 176;
    pub const transient_____u6__nodeCost: usize = 176;
    pub const transient_____u6__linkIndex: usize = 176;
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
pub mod snd_alias_context_list {
    pub const SIZE: usize = 2;
    pub const ALIGN: usize = 1;
    pub const aliasOffset: usize = 0;
    pub const count: usize = 1;
}
pub mod snd_alias_list_t {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const aliasName: usize = 0;
    pub const name: usize = 0;
    pub const head: usize = 8;
    pub const contextList: usize = 16;
    pub const count: usize = 24;
    pub const contextListCount: usize = 25;
}
pub mod snd_alias_t {
    pub const SIZE: usize = 240;
    pub const ALIGN: usize = 8;
    pub const aliasName: usize = 0;
    pub const subtitle: usize = 8;
    pub const secondaryAliasName: usize = 16;
    pub const chainAliasName: usize = 24;
    pub const soundFile: usize = 32;
    pub const squelchName: usize = 40;
    pub const polyCount: usize = 48;
    pub const dspBusIndex: usize = 50;
    pub const priority: usize = 51;
    pub const playCount: usize = 52;
    pub const polyClass: usize = 56;
    pub const volMin: usize = 60;
    pub const volMax: usize = 64;
    pub const volModIndex: usize = 68;
    pub const pitchMin: usize = 72;
    pub const pitchMax: usize = 76;
    pub const distMin: usize = 80;
    pub const distMax: usize = 84;
    pub const velocityMin: usize = 88;
    pub const flags: usize = 92;
    pub const masterPriority: usize = 96;
    pub const masterPercentage: usize = 100;
    pub const slavePercentage: usize = 104;
    pub const playbackPercentage: usize = 108;
    pub const probability: usize = 112;
    pub const variationType: usize = 116;
    pub const sndContext: usize = 120;
    pub const sequence: usize = 128;
    pub const lfePercentage: usize = 132;
    pub const centerPercentage: usize = 136;
    pub const startDelay: usize = 140;
    pub const volumeFalloffCurve: usize = 144;
    pub const envelopMin: usize = 152;
    pub const envelopMax: usize = 156;
    pub const lpfCurve: usize = 160;
    pub const reverbSendCurve: usize = 168;
    pub const speakerMap: usize = 176;
    pub const wetMixOverride: usize = 184;
    pub const focusPercentage: usize = 188;
    pub const smartpanDistance2d: usize = 192;
    pub const smartpanDistance3d: usize = 196;
    pub const smartpanAttenuation3d: usize = 200;
    pub const minSmartpan2dContribution: usize = 204;
    pub const stereo3DAngle: usize = 208;
    pub const stereo3DStart: usize = 212;
    pub const stereo3DEnd: usize = 216;
    pub const allowDoppler: usize = 220;
    pub const dopplerPreset: usize = 224;
    pub const threshold: usize = 232;
    pub const lockedLoopTime: usize = 236;
}
pub mod sphere_tree_data_t {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const treeCount: usize = 0;
    pub const tree: usize = 8;
    pub const numObjects: usize = 16;
    pub const objects: usize = 24;
}
pub mod sphere_tree_obj_t {
    pub const SIZE: usize = 20;
    pub const ALIGN: usize = 4;
    pub const origin: usize = 0;
    pub const radius: usize = 12;
    pub const object: usize = 16;
}
pub mod sphere_tree_t {
    pub const SIZE: usize = 32;
    pub const ALIGN: usize = 8;
    pub const axis: usize = 0;
    pub const dist: usize = 4;
    pub const numObjects: usize = 8;
    pub const objIdx: usize = 16;
    pub const child: usize = 24;
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
