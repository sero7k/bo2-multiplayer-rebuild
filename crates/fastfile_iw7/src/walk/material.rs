//! Shaders, vertex declarations, technique sets, images and materials
//! (x64-zt `iw7/assets/{vertexshader,pixelshader,techset,gfximage,material}.cpp`).

use super::{AssetId, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::layout as L;
use crate::ptr::Ptr;
use crate::stream::{CALLBACK, TEMP, VIRTUAL};
use crate::walk::{ImageRec, MaterialConstant, MaterialRec, MaterialTexture, Span, TechniqueSetRec};

/// Material techniques (`MaterialTechniqueType`): the size of a material's
/// `stateBitsEntry` and `constantBufferIndex`.
pub const TECHNIQUE_COUNT: usize = 152;

/// A shader: its name and debug name, then its program (aligned 4): a
/// vertex shader's in VIRTUAL, the others' in TEMP.
pub(super) fn shader(w: &mut Walker, ty: AssetType, h: usize, temp_program: bool) -> Result<AssetId> {
    use L::MaterialVertexShader as S;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + S::name)?.unwrap_or_default();
    w.xstr(h + S::debugName)?;
    let size = w.s.u32_at(h + S::prog__loadDef__programSize)? as usize;
    if temp_program {
        w.s.push(TEMP);
    }
    w.blob(h + S::prog__loadDef__program, size, 3)?;
    if temp_program {
        w.s.pop();
    }
    w.s.pop();
    Ok(w.name_only(ty, name))
}

pub(super) fn vertex_decl(w: &mut Walker, h: usize) -> Result<AssetId> {
    w.s.push(VIRTUAL);
    let name = w.xstr(h + L::MaterialVertexDeclaration::name)?.unwrap_or_default();
    w.s.pop();
    Ok(w.name_only(AssetType::VertexDecl, name))
}

pub(super) fn technique_set(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::MaterialTechniqueSet as T;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + T::name)?.unwrap_or_default();
    let count = w.s.u16_at(h + T::techniqueCount)? as usize;
    let mut rec = TechniqueSetRec { name, world_vert_format: w.s.u8_at(h + T::worldVertFormat)?, techniques: Vec::new() };
    if let Some(table) = w.inline_array(h + T::techniques, 8 * count, 7)? {
        for t in 0..count {
            let field = table + 8 * t;
            match w.s.ptr_at(field)? {
                p @ (Ptr::Follow | Ptr::Insert) => {
                    if p == Ptr::Insert {
                        w.s.insert_slot();
                    }
                    let (tname, passes) = technique(w)?;
                    rec.techniques.push((t, tname, passes));
                }
                Ptr::Ref { block: VIRTUAL, offset } => {
                    // A technique shared with an earlier set: its name.
                    let at = w.file_of(offset as u64);
                    let name = at
                        .and_then(|at| w.s.ptr_at(at + L::MaterialTechniqueHeader::name).ok())
                        .and_then(|p| p.virtual_offset())
                        .and_then(|v| w.string_at(v))
                        .unwrap_or_default();
                    let passes = at.and_then(|at| w.s.u16_at(at + L::MaterialTechniqueHeader::passCount).ok()).unwrap_or(0);
                    rec.techniques.push((t, name, passes));
                }
                _ => {}
            }
        }
    }
    w.s.pop();
    w.cap.techsets.push(rec);
    Ok(AssetId { ty: AssetType::TechniqueSet, index: (w.cap.techsets.len() - 1) as u32 })
}

/// One technique (header and passes, aligned 8), its shaders and
/// arguments, then its name.
fn technique(w: &mut Walker) -> Result<(String, u16)> {
    use L::MaterialPass as P;
    let hdr_len = L::MaterialTechniqueHeader::SIZE;
    w.s.align(7);
    let at = w.s.pos();
    let passes = w.s.u16_at(at + L::MaterialTechniqueHeader::passCount)? as usize;
    let hdr = w.st(hdr_len + P::SIZE * passes, 7)?;
    for i in 0..passes {
        let p = hdr + hdr_len + P::SIZE * i;
        w.asset(AssetType::VertexShader, p + P::vertexShader)?;
        w.asset(AssetType::VertexDecl, p + P::vertexDecl)?;
        w.asset(AssetType::HullShader, p + P::hullShader)?;
        w.asset(AssetType::DomainShader, p + P::domainShader)?;
        w.asset(AssetType::PixelShader, p + P::pixelShader)?;
        let n = w.s.u8_at(p + P::perPrimArgCount)? as usize + w.s.u8_at(p + P::perObjArgCount)? as usize + w.s.u8_at(p + P::stableArgCount)? as usize;
        let a_len = L::MaterialShaderArgument::SIZE;
        if let Some(args) = w.inline_array(p + P::args, a_len * n, 7)? {
            for k in 0..n {
                let arg = args + a_len * k;
                if w.s.u8_at(arg + L::MaterialShaderArgument::r#type)? == 4 {
                    w.blob(arg + L::MaterialShaderArgument::u, 16, 3)?;
                }
            }
        }
    }
    let name = w.xstr(hdr + L::MaterialTechniqueHeader::name)?.unwrap_or_default();
    Ok((name, passes as u16))
}

pub(super) fn image(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::GfxImage as G;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + G::name)?.unwrap_or_default();
    let len1 = w.s.u32_at(h + G::dataLen1)? as usize;
    w.s.push(TEMP);
    let pixels = w.blob(h + G::pixelData, len1, 0)?.map(|pos| Span { pos, len: len1 });
    w.s.pop();
    w.s.pop();
    let streamed = w.s.u8_at(h + G::streamed)? != 0;
    let mut streams = [(0u16, 0u16, 0u32, 0u8); 4];
    for (i, s) in streams.iter_mut().enumerate() {
        let o = h + G::streams + L::GfxImageStreamData::SIZE * i;
        let packed = w.s.u32_at(o + L::GfxImageStreamData::levelCountAndSize)?;
        *s = (w.s.u16_at(o)?, w.s.u16_at(o + 2)?, packed & 0x03FF_FFFF, (packed >> 26) as u8);
    }
    let stream_files = if streamed { w.take_stream_files() } else { [None; 4] };
    let rec = ImageRec {
        name,
        format: w.s.u32_at(h + G::imageFormat)?,
        map_type: w.s.u8_at(h + G::mapType)?,
        semantic: w.s.u8_at(h + G::semantic)?,
        category: w.s.u8_at(h + G::category)?,
        flags: w.s.u32_at(h + G::flags)? as u8,
        width: w.s.u16_at(h + G::width)?,
        height: w.s.u16_at(h + G::height)?,
        depth: w.s.u16_at(h + G::depth)?,
        num_elements: w.s.u16_at(h + G::numElements)?,
        level_count: w.s.u8_at(h + G::levelCount)?,
        streamed,
        data_len1: len1 as u32,
        data_len2: w.s.u32_at(h + G::dataLen2)?,
        pixels,
        streams,
        stream_files,
    };
    w.cap.images.push(rec);
    Ok(AssetId { ty: AssetType::Image, index: (w.cap.images.len() - 1) as u32 })
}

pub(super) fn material(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::Material as M;
    w.s.push(VIRTUAL);
    let name = w.xstr(h + M::info__name)?.unwrap_or_default();
    let techset = w.asset(AssetType::TechniqueSet, h + M::techniqueSet)?;
    let tex_n = w.s.u8_at(h + M::textureCount)? as usize;
    let mut textures = Vec::new();
    if !matches!(w.s.ptr_at(h + M::textureTable)?, Ptr::Null) {
        w.s.inc(CALLBACK, (4 * tex_n) as u64);
    }
    let td = L::MaterialTextureDef::SIZE;
    let table = match w.inline_array(h + M::textureTable, td * tex_n, 7)? {
        Some(table) => Some((table, true)),
        None => match w.s.ptr_at(h + M::textureTable)? {
            Ptr::Ref { block: VIRTUAL, offset } => w.file_of(offset as u64).map(|t| (t, false)),
            _ => None,
        },
    };
    if let Some((table, inline)) = table {
        for i in 0..tex_n {
            let t = table + td * i;
            let image = if inline {
                w.asset(AssetType::Image, t + L::MaterialTextureDef::image)?
            } else {
                w.s.ptr_at(t + L::MaterialTextureDef::image).ok().and_then(|p| p.virtual_offset()).and_then(|v| w.resolve_slot(AssetType::Image, v))
            };
            textures.push(MaterialTexture {
                name_hash: w.s.u32_at(t + L::MaterialTextureDef::nameHash)?,
                name_start: w.s.u8_at(t + L::MaterialTextureDef::nameStart)?,
                name_end: w.s.u8_at(t + L::MaterialTextureDef::nameEnd)?,
                sampler_state: w.s.u8_at(t + L::MaterialTextureDef::samplerState)?,
                semantic: w.s.u8_at(t + L::MaterialTextureDef::semantic)?,
                image,
            });
        }
    }
    let const_n = w.s.u8_at(h + M::constantCount)? as usize;
    let cd = L::MaterialConstantDef::SIZE;
    let mut constants = Vec::new();
    if let Some(table) = w.blob(h + M::constantTable, cd * const_n, 15)? {
        for i in 0..const_n {
            let c = table + cd * i;
            let raw = w.s.slice(c + L::MaterialConstantDef::name, 12)?;
            let name = super::text(&raw[..raw.iter().position(|&b| b == 0).unwrap_or(12)]);
            let mut literal = [0f32; 4];
            for (k, v) in literal.iter_mut().enumerate() {
                *v = w.s.f32_at(c + L::MaterialConstantDef::literal + 4 * k)?;
            }
            constants.push(MaterialConstant { name_hash: w.s.u32_at(c)?, name, literal });
        }
    }
    let sb_n = w.s.u8_at(h + M::stateBitsCount)? as usize;
    let sb = L::GfxStateBits::SIZE;
    let mut state_bits = Vec::new();
    if let Some(table) = w.blob(h + M::stateBitsTable, sb * sb_n, 3)? {
        for i in 0..sb_n {
            let mut b = [0u8; L::GfxStateBits::SIZE];
            b.copy_from_slice(w.s.slice(table + sb * i, sb)?);
            state_bits.push(b);
        }
    }
    w.blob(h + M::constantBufferIndex, TECHNIQUE_COUNT, 0)?;
    let cb_n = w.s.u8_at(h + M::constantBufferCount)? as usize;
    let cbd = L::MaterialConstantBufferDef::SIZE;
    if let Some(table) = w.inline_array(h + M::constantBufferTable, cbd * cb_n, 15)? {
        use L::MaterialConstantBufferDef as C;
        for i in 0..cb_n {
            let c = table + cbd * i;
            for (data, size) in [(C::vsData, C::vsDataSize), (C::hsData, C::hsDataSize), (C::dsData, C::dsDataSize), (C::psData, C::psDataSize)] {
                let n = w.s.u32_at(c + size)? as usize;
                w.blob(c + data, n, 15)?;
            }
            for (data, size) in [
                (C::vsOffsetData, C::vsOffsetDataSize),
                (C::hsOffsetData, C::hsOffsetDataSize),
                (C::dsOffsetData, C::dsOffsetDataSize),
                (C::psOffsetData, C::psOffsetDataSize),
            ] {
                let n = w.s.u32_at(c + size)? as usize;
                w.blob(c + data, 2 * n, 1)?;
            }
        }
    }
    let layers = w.s.u8_at(h + M::layerCount)? as usize;
    if let Some(table) = w.inline_array(h + M::subMaterials, 8 * layers, 7)? {
        for i in 0..layers {
            w.xstr(table + 8 * i)?;
        }
    }
    w.s.pop();
    let rec = MaterialRec {
        name,
        game_flags: w.s.u8_at(h + M::info__gameFlags)?,
        sort_key: w.s.u8_at(h + M::info__sortKey)?,
        texture_atlas: (w.s.u8_at(h + M::info__textureAtlasRowCount)?, w.s.u8_at(h + M::info__textureAtlasColumnCount)?),
        state_flags: w.s.u8_at(h + M::stateFlags)?,
        camera_region: w.s.u8_at(h + M::cameraRegion)?,
        material_type: w.s.u8_at(h + M::materialType)?,
        techset,
        textures,
        constants,
        state_bits,
        state_bits_entry: w.s.slice(h + M::stateBitsEntry, TECHNIQUE_COUNT)?.to_vec(),
    };
    w.cap.materials.push(rec);
    Ok(AssetId { ty: AssetType::Material, index: (w.cap.materials.len() - 1) as u32 })
}
