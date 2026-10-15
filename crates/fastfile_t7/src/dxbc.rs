//! bo2mp: what a compiled shader program (`DXBC` container, shader model 5)
//! says about its inputs: its resource definitions (`RDEF` chunk), read
//! from the program the zone carries. A Black Ops III material keeps its
//! numbers (`colorTint`, `glossRange`, `detailScale1`, ...) in the initial
//! data of its techniques' `$Globals` constant buffers, laid out as each
//! technique's shader declares them; this gives the names and offsets, and
//! the textures the shader actually binds.
//!
//! ```text
//! DXBC: "DXBC", 16-byte checksum, u32 1, u32 total size, u32 chunk count,
//!       u32 chunk offsets[]; each chunk: fourcc, u32 size, data
//! RDEF: u32 cbuffer count, cbuffer offset, binding count, binding offset;
//!       u16 version (minor, major), u32 flags, creator offset (+ "RD11" ...)
//!   binding (32): name offset, type (0 cbuffer, 2 texture, 3 sampler),
//!       return type, dimension, samples, bind point, bind count, flags
//!   cbuffer (24): name offset, variable count, variable offset, size,
//!       flags, type
//!   variable (40 in shader model 5, 24 before): name offset, start offset,
//!       size, flags, type offset, default offset, ...
//! ```
//! (Every offset inside RDEF counts from the chunk's data.)

/// One variable of a constant buffer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variable {
    pub name: String,
    pub offset: u32,
    pub size: u32,
    /// `D3D_SVF_USED`: the shader reads it.
    pub used: bool,
}

/// A constant buffer's layout.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConstantBuffer {
    pub name: String,
    pub size: u32,
    pub variables: Vec<Variable>,
}

/// What a program's `RDEF` declares.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Reflection {
    pub buffers: Vec<ConstantBuffer>,
    /// Texture bindings: (name, bind point).
    pub textures: Vec<(String, u32)>,
}

impl Reflection {
    /// The `$Globals` buffer.
    pub fn globals(&self) -> Option<&ConstantBuffer> {
        self.buffers.iter().find(|b| b.name == "$Globals")
    }
    /// Whether the program binds a texture called `name`.
    pub fn binds(&self, name: &str) -> bool {
        self.textures.iter().any(|(n, _)| n == name)
    }
}

fn u32_at(b: &[u8], o: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(o..o + 4)?.try_into().ok()?))
}

fn cstr(b: &[u8], o: usize) -> Option<String> {
    let s = b.get(o..)?;
    let end = s.iter().position(|&c| c == 0)?;
    Some(String::from_utf8_lossy(&s[..end]).into_owned())
}

/// A chunk's data by its fourcc.
pub fn chunk<'a>(program: &'a [u8], fourcc: &[u8; 4]) -> Option<&'a [u8]> {
    if program.get(0..4)? != b"DXBC" {
        return None;
    }
    let count = u32_at(program, 28)? as usize;
    for k in 0..count.min(64) {
        let at = u32_at(program, 32 + 4 * k)? as usize;
        if program.get(at..at + 4)? == fourcc {
            let size = u32_at(program, at + 4)? as usize;
            return program.get(at + 8..at + 8 + size);
        }
    }
    None
}

/// A program's resource definitions; `None` when it has no `RDEF` chunk or
/// the chunk does not read.
pub fn reflect(program: &[u8]) -> Option<Reflection> {
    let r = chunk(program, b"RDEF")?;
    let (cb_count, cb_at, bind_count, bind_at) = (u32_at(r, 0)? as usize, u32_at(r, 4)? as usize, u32_at(r, 8)? as usize, u32_at(r, 12)? as usize);
    let major = *r.get(17)?;
    let var_size = if major >= 5 { 40 } else { 24 };
    let mut out = Reflection::default();
    for i in 0..bind_count.min(256) {
        let b = bind_at + 32 * i;
        if u32_at(r, b + 4)? == 2 {
            out.textures.push((cstr(r, u32_at(r, b)? as usize)?, u32_at(r, b + 20)?));
        }
    }
    for i in 0..cb_count.min(64) {
        let c = cb_at + 24 * i;
        let mut cb = ConstantBuffer { name: cstr(r, u32_at(r, c)? as usize)?, size: u32_at(r, c + 12)?, variables: Vec::new() };
        let (n, vat) = (u32_at(r, c + 4)? as usize, u32_at(r, c + 8)? as usize);
        for v in 0..n.min(1024) {
            let o = vat + var_size * v;
            cb.variables.push(Variable {
                name: cstr(r, u32_at(r, o)? as usize)?,
                offset: u32_at(r, o + 4)?,
                size: u32_at(r, o + 8)?,
                used: u32_at(r, o + 12)? & 2 != 0,
            });
        }
        out.buffers.push(cb);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A DXBC container with one RDEF chunk: one `$Globals` buffer of two
    /// variables, one texture and one sampler binding.
    fn sample() -> Vec<u8> {
        let mut r = vec![0u8; 0x200];
        let put = |r: &mut Vec<u8>, o: usize, v: u32| r[o..o + 4].copy_from_slice(&v.to_le_bytes());
        let name = |r: &mut Vec<u8>, o: usize, s: &str| r[o..o + s.len()].copy_from_slice(s.as_bytes());
        put(&mut r, 0, 1);
        put(&mut r, 4, 0x80);
        put(&mut r, 8, 2);
        put(&mut r, 12, 0x20);
        r[16] = 0;
        r[17] = 5;
        // Bindings at 0x20: colorMap (texture, t6), colorSampler (sampler).
        put(&mut r, 0x20, 0x140);
        put(&mut r, 0x24, 2);
        put(&mut r, 0x34, 6);
        put(&mut r, 0x40, 0x150);
        put(&mut r, 0x44, 3);
        // Buffer at 0x80: $Globals, 2 variables at 0xa0, 192 bytes.
        put(&mut r, 0x80, 0x160);
        put(&mut r, 0x84, 2);
        put(&mut r, 0x88, 0xa0);
        put(&mut r, 0x8c, 192);
        put(&mut r, 0xa0, 0x170);
        put(&mut r, 0xa4, 160);
        put(&mut r, 0xa8, 12);
        put(&mut r, 0xac, 2);
        put(&mut r, 0xc8, 0x180);
        put(&mut r, 0xcc, 176);
        put(&mut r, 0xd0, 8);
        name(&mut r, 0x140, "colorMap");
        name(&mut r, 0x150, "colorSampler");
        name(&mut r, 0x160, "$Globals");
        name(&mut r, 0x170, "colorTint");
        name(&mut r, 0x180, "glossRange");
        let mut p = vec![0u8; 36];
        p[0..4].copy_from_slice(b"DXBC");
        p[28..32].copy_from_slice(&1u32.to_le_bytes());
        p[32..36].copy_from_slice(&36u32.to_le_bytes());
        p.extend_from_slice(b"RDEF");
        p.extend_from_slice(&(r.len() as u32).to_le_bytes());
        p.extend_from_slice(&r);
        let total = p.len() as u32;
        p[24..28].copy_from_slice(&total.to_le_bytes());
        p
    }

    #[test]
    fn reads_buffers_and_textures() {
        let r = reflect(&sample()).expect("rdef");
        assert_eq!(r.textures, vec![("colorMap".to_owned(), 6)]);
        let g = r.globals().expect("globals");
        assert_eq!(g.size, 192);
        assert_eq!(g.variables[0], Variable { name: "colorTint".into(), offset: 160, size: 12, used: true });
        assert_eq!(g.variables[1].name, "glossRange");
        assert!(!g.variables[1].used);
        assert!(r.binds("colorMap") && !r.binds("normalMap"));
    }
}
