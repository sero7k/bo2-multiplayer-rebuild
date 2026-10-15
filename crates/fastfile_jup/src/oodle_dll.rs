//! The game's own Oodle DLL (`oo2core_8_win64.dll`), loaded at runtime from
//! the owner's install. The only module of this crate with `unsafe`: one
//! `libloading` symbol lookup and the foreign call.
#![allow(unsafe_code)]

use std::path::Path;

use crate::oodle::{Decompressor, OodleError};

/// `OodleLZ_Decompress` (Oodle 2.8/2.9, x64):
/// `(comp, compLen, raw, rawLen, fuzzSafe, checkCRC, verbosity, decBufBase,
/// decBufSize, callback, callbackUserData, decoderMemory, decoderMemorySize,
/// threadPhase) -> decoded length (0 on failure)`.
type DecompressFn = unsafe extern "C" fn(
    *const u8,
    isize,
    *mut u8,
    isize,
    i32,
    i32,
    i32,
    *mut u8,
    isize,
    *const std::ffi::c_void,
    *mut std::ffi::c_void,
    *mut std::ffi::c_void,
    isize,
    i32,
) -> isize;

const FUZZ_SAFE_YES: i32 = 1;
const CHECK_CRC_NO: i32 = 0;
const VERBOSITY_NONE: i32 = 0;
/// `OodleLZ_Decode_Unthreaded`.
const THREAD_PHASE_ALL: i32 = 3;

pub struct OodleDll {
    decompress: DecompressFn,
    // Keeps the DLL mapped while `decompress` may be called.
    _lib: libloading::Library,
}

impl OodleDll {
    pub fn load(path: &Path) -> Result<Self, OodleError> {
        if !path.is_file() {
            return Err(OodleError::NoBackend(format!(
                "{} not found",
                path.display()
            )));
        }
        // SAFETY: loading the game's Oodle DLL runs its DllMain, which only
        // initialises the library; the exported symbol has the signature above.
        unsafe {
            let lib = libloading::Library::new(path)
                .map_err(|e| OodleError::NoBackend(format!("loading {}: {e}", path.display())))?;
            let sym: libloading::Symbol<DecompressFn> =
                lib.get(b"OodleLZ_Decompress\0").map_err(|e| {
                    OodleError::NoBackend(format!("OodleLZ_Decompress in {}: {e}", path.display()))
                })?;
            let decompress = *sym;
            Ok(Self {
                decompress,
                _lib: lib,
            })
        }
    }
}

impl Decompressor for OodleDll {
    fn name(&self) -> &str {
        "oo2core_8_win64.dll"
    }

    fn decompress(&self, src: &[u8], dst: &mut [u8]) -> Result<(), OodleError> {
        if src.is_empty() || dst.is_empty() {
            return Err(OodleError::WrongLength {
                got: 0,
                want: dst.len(),
            });
        }
        // SAFETY: both buffers are live slices of the lengths passed; with
        // fuzz safety on, the decoder neither reads past `src` nor writes
        // past `dst`. No callback, no external decoder memory.
        let got = unsafe {
            (self.decompress)(
                src.as_ptr(),
                src.len() as isize,
                dst.as_mut_ptr(),
                dst.len() as isize,
                FUZZ_SAFE_YES,
                CHECK_CRC_NO,
                VERBOSITY_NONE,
                std::ptr::null_mut(),
                0,
                std::ptr::null(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                THREAD_PHASE_ALL,
            )
        };
        if got as usize != dst.len() || got <= 0 {
            return Err(OodleError::WrongLength {
                got: got as i64,
                want: dst.len(),
            });
        }
        Ok(())
    }
}
