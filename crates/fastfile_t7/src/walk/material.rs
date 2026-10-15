//! Materials and technique sets (x64-zt `t7/structs.hpp` layouts; the
//! stream order read on retail `core_post_gfx`, `en_mp_common`, `mp_common`).
//!
//! ```text
//! MaterialTechniqueSet 0x70: 0x00 char* name; 0x08 u32 renderFlags; 0x10 MaterialTechnique* techniques[12]
//! MaterialTechnique  0x2d0: 0x00 char* name; 0x08 MaterialDrawMethod drawMethods[8]; 0x2c8 GfxStateMap* stateMap
//! MaterialDrawMethod  0x58: 0x00 pipeline; 0x08 MaterialVertexDeclaration* (0x78 bytes);
//!                     0x10 primitiveType, indexCount; 0x18 VS; 0x20 PS; 0x28 GS; 0x30 LS; 0x38 HS; 0x40 DS;
//!                     0x48 u8 argCount (x64-zt: stableArgCount), materialType, drawMethodType; 0x50 args
//! Material*Shader     0x28: char* name; char* extraDef; void* shader; u8* program; u32 programSize; u32 crc
//! MaterialShaderArgument 16: u16 type, location, size, buffer; union { float* literalConst (vec4); hashes }
//! ```
//!
//! Stream order of a technique (read, not x64-zt's): the technique struct
//! (aligned 8, no slot of its own), then per draw
//! method its vertex shader (struct, name, extraDef, program aligned 4),
//! **then** its vertex declaration (aligned 4), the pixel shader, the other
//! stages, the arguments (a literal's vec4 follows only when its union
//! pointer is `-1`); after all methods the technique's name and its state
//! map (112 bytes, aligned 4). A draw method's vertex declaration is often
//! a reference to the one an earlier method loaded.
//!
//! (The slot + 16-alignment an earlier version used only made up for
//! two miscounts before `core_post_gfx`'s technique sets: its sound
//! bank's snapshots are 32-aligned and an empty font icon's name is in
//! VIRTUAL like any other; with those, every zone resolves with plain
//! 8-alignment.)
//!
//! ```text
//! Material 0x2a0: 0x00 MaterialInfo (char* name; flags; sort key; surface bits; contents) 0x30
//!                 0x30 MaterialStaticConstantBuffer* constantBuffers[12][6]
//!                 0x270 u8 textureCount, samplerCount, constantCount; cameraRegion
//!                 0x278 techniqueSet; 0x280 MaterialTextureDef* (32 each, aligned 8: GfxImage* image; hashes, uv);
//!                 0x288 MaterialSamplerDef* (8 each, aligned 8); 0x290 MaterialConstantDef* (32 each, aligned 16);
//!                 0x298 Material* enemyMaterial
//! MaterialStaticConstantBuffer 0x48 (aligned 16): 5 runtime buffers; 0x28 u8* initData;
//!                 0x30 u32 initDataSize; 0x38 char* name; 0x40 u32 deviceSlot
//! ```
//! Order: name, the constant buffers (each: struct, initData in PHYSICAL
//! aligned 256, name), the technique set (an asset, struct in TEMP; its
//! pointer is `-2` when inline, so it takes an 8-byte VIRTUAL slot first),
//! the texture table then each texture's image (an asset), the sampler
//! table (aligned 16, also when empty), the constant table, the enemy
//! material. Later references to an image name its texture def's `image`
//! field. (Read on retail `core_post_gfx`, `en_mp_common`, `mp_patch`:
//! every reference resolves and VIRTUAL and PHYSICAL end at the header's
//! sizes.)

use super::capture::MaterialRec;
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::stream::{PHYSICAL, Stream};

const TECHNIQUES: usize = 12;
const TECHNIQUE: usize = 0x2d0;
const DRAW_METHODS: usize = 8;
const DRAW_METHOD: usize = 0x58;
const VERTEX_DECL: usize = 0x78;
const SHADER: usize = 0x28;
const ARG: usize = 16;
const STATE_MAP: usize = 112;

const CONSTANT_BUFFERS: usize = 0x30;
const CONSTANT_BUFFER_SLOTS: usize = TECHNIQUES * 6;
const CONSTANT_BUFFER: usize = 0x48;
const TEXTURE_DEF: usize = 32;
const SAMPLER_DEF: usize = 8;
const CONSTANT_DEF: usize = 32;

/// A shader stage: struct, name, extraDef, program (aligned 4).
fn shader(s: &mut Stream, field: usize) -> Result<()> {
    if let Some(sh) = s.array(field, SHADER, 7)? {
        s.no_loads(sh, SHADER, &[0, 8, 0x18])?;
        s.string(sh)?;
        s.string(sh + 8)?;
        let size = s.u32_at(sh + 0x20)? as usize;
        s.array(sh + 0x18, size, 3)?;
    }
    Ok(())
}

fn technique(s: &mut Stream, field: usize) -> Result<()> {
    if !s.ptr_at(field)?.is_inline() {
        return s.array(field, 0, 0).map(drop);
    }
    let Some(t) = s.array(field, TECHNIQUE, 7)? else { return Ok(()) };
    for m in 0..DRAW_METHODS {
        let d = t + 8 + DRAW_METHOD * m;
        s.no_loads(d, DRAW_METHOD, &[0x08, 0x18, 0x20, 0x28, 0x30, 0x38, 0x40, 0x50])?;
        shader(s, d + 0x18)?;
        s.array(d + 0x08, VERTEX_DECL, 3)?;
        for stage in [0x20, 0x28, 0x30, 0x38, 0x40] {
            shader(s, d + stage)?;
        }
        let n = s.u8_at(d + 0x48)? as usize;
        if let Some(a) = s.records(d + 0x50, n, ARG, 7)? {
            for i in 0..n {
                // A literal constant's vec4 follows when its pointer does
                // (otherwise the union holds hashes and indices).
                if s.ptr_at(a + ARG * i + 8)?.is_inline() {
                    s.array(a + ARG * i + 8, 16, 3)?;
                }
            }
        }
    }
    s.string(t)?;
    s.array(t + 0x2c8, STATE_MAP, 3)?;
    Ok(())
}

pub fn technique_set(s: &mut Stream, h: usize) -> Result<Option<String>> {
    let name = s.string(h)?;
    for i in 0..TECHNIQUES {
        technique(s, h + 0x10 + 8 * i)?;
    }
    Ok(name)
}

pub fn material(s: &mut Stream, h: usize) -> Result<Option<String>> {
    let mut skip: Vec<usize> = vec![0, 0x278, 0x280, 0x288, 0x290, 0x298];
    skip.extend((0..CONSTANT_BUFFER_SLOTS).map(|i| CONSTANT_BUFFERS + 8 * i));
    s.no_loads(h, AssetType::Material.header_size(), &skip)?;
    let name = s.string(h)?;
    let mut colour_consts = None;
    let mut globals: Vec<(u8, Vec<u8>)> = Vec::new();
    for i in 0..CONSTANT_BUFFER_SLOTS {
        if let Some(c) = s.array(h + CONSTANT_BUFFERS + 8 * i, CONSTANT_BUFFER, 15)? {
            s.no_loads(c, CONSTANT_BUFFER, &[0x28, 0x38])?;
            let size = s.u32_at(c + 0x30)? as usize;
            s.push(PHYSICAL);
            let r = s.array(c + 0x28, size, 255);
            s.pop();
            let data = r?;
            let buffer = s.string(c + 0x38)?;
            if buffer.as_deref() == Some("$Globals")
                && let Some(at) = data
                && !globals.iter().any(|(_, d): &(u8, Vec<u8>)| d.len() == size && (0..size).all(|k| s.u8_at(at + k).is_ok_and(|b| b == d[k])))
            {
                let mut d = vec![0u8; size];
                for (k, b) in d.iter_mut().enumerate() {
                    *b = s.u8_at(at + k)?;
                }
                globals.push((i as u8, d));
            }
            if colour_consts.is_none()
                && size >= 192
                && buffer.as_deref() == Some("$Globals")
                && let Some(at) = data
            {
                let mut v = [0f32; 8];
                for (k, x) in v.iter_mut().enumerate() {
                    *x = s.f32_at(at + 4 * (40 + k))?;
                }
                colour_consts = Some(v);
            }
        }
    }
    let mut rec = MaterialRec {
        name: name.clone().unwrap_or_else(String::new),
        colour_consts,
        globals,
        ..Default::default()
    };
    rec.techset = s.asset_named(AssetType::TechniqueSet, h + 0x278)?;
    let textures = s.u8_at(h + 0x270)? as usize;
    if let Some(t) = s.records(h + 0x280, textures, TEXTURE_DEF, 7)? {
        for i in 0..textures {
            let def = t + TEXTURE_DEF * i;
            if let Some(img) = s.asset_named(AssetType::Image, def)? {
                rec.textures.push((s.u32_at(def + 8)?, img));
            }
        }
    }
    let samplers = s.u8_at(h + 0x271)? as usize;
    // The sampler table is 16-aligned.
    s.records(h + 0x288, samplers, SAMPLER_DEF, 15)?;
    let constants = s.u8_at(h + 0x272)? as usize;
    s.records(h + 0x290, constants, CONSTANT_DEF, 15)?;
    s.asset(AssetType::Material, h + 0x298)?;
    s.capture.add_material(rec);
    Ok(name)
}
