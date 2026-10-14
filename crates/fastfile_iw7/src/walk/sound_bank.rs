//! Sound banks: x64-zt `iw7/assets/sound_bank.cpp`'s writer: the strings,
//! the alias lists (each list's name, its aliases, each alias's strings),
//! the alias index, the send effects, the sound table's arrays (each with
//! its strings), the ducks (with their 128-float curves) and the music
//! sets. The sound data itself is not in the zone: aliases name their
//! files in the `.sabl` (loaded) / `.sabs` (streamed) sound asset banks.

use super::{AssetId, SndAliasRec, SoundBankRec, Walker};
use crate::asset_type::AssetType;
use crate::cursor::Result;
use crate::layout as L;
use crate::stream::VIRTUAL;

/// An array of `size`-byte records whose string fields follow.
fn records(w: &mut Walker, h: usize, ptr: usize, count: usize, size: usize, mask: u64, strs: &[usize]) -> Result<()> {
    let n = w.s.u32_at(h + count)? as usize;
    if let Some(t) = w.inline_array(h + ptr, size * n, mask)? {
        for i in 0..n {
            for &f in strs {
                w.xstr(t + size * i + f)?;
            }
        }
    }
    Ok(())
}

pub(super) fn sound_bank(w: &mut Walker, h: usize) -> Result<AssetId> {
    use L::SndBank as B;
    w.s.push(VIRTUAL);
    let mut rec = SoundBankRec {
        name: w.xstr(h + B::name)?.unwrap_or_default(),
        zone: w.xstr(h + B::zone)?.unwrap_or_default(),
        ..Default::default()
    };
    rec.game_language = w.xstr(h + B::gameLanguage)?.unwrap_or_default();
    rec.sound_language = w.xstr(h + B::soundLanguage)?.unwrap_or_default();

    use L::SndAlias as A;
    use L::SndAliasList as AL;
    let lists = w.s.u32_at(h + B::aliasCount)? as usize;
    if let Some(t) = w.inline_array(h + B::alias, AL::SIZE * lists, 7)? {
        for i in 0..lists {
            let l = t + AL::SIZE * i;
            let name = w.xstr(l + AL::aliasName)?.unwrap_or_default();
            let n = w.s.i32_at(l + AL::count)?.max(0) as usize;
            let mut entries = Vec::new();
            if let Some(a) = w.inline_array(l + AL::head, A::SIZE * n, 7)? {
                for j in 0..n {
                    let e = a + A::SIZE * j;
                    for f in [A::aliasName, A::subtitle, A::secondaryAliasName, A::stopAliasName] {
                        w.xstr(e + f)?;
                    }
                    let file = w.xstr(e + A::assetFileName)?;
                    let flags = w.s.u32_at(e + A::flags)?;
                    entries.push((file, ((flags >> 7) & 3) as u8, w.s.u32_at(e + A::assetId)?));
                }
            }
            rec.aliases.push(SndAliasRec { name, entries });
        }
    }
    w.blob(h + B::aliasIndex, L::SndIndexEntry::SIZE * lists, 7)?;
    let n = w.s.u32_at(h + B::sendEffectCount)? as usize;
    w.blob(h + B::sendEffects, L::SndSendEffectParams::SIZE * n, 3)?;

    records(w, h, B::soundTable__adsrSettings, B::soundTable__adsrCount, L::ADSRSetting::SIZE, 7, &[L::ADSRSetting::name])?;
    records(w, h, B::soundTable__ambientDefs, B::soundTable__ambientDefCount, L::AmbientDef::SIZE, 7, &[L::AmbientDef::name])?;
    records(w, h, B::soundTable__ambientEvents, B::soundTable__ambientEventCount, L::AmbientEvent::SIZE, 3, &[])?;
    records(w, h, B::soundTable__ambientElements, B::soundTable__ambientElementsCount, L::AmbientElement::SIZE, 7, &[L::AmbientElement::aliasName])?;
    records(w, h, B::soundTable__filters, B::soundTable__filterCount, L::FilterDef::SIZE, 3, &[])?;
    records(w, h, B::soundTable__mixes, B::soundTable__mixCount, L::MixDef::SIZE, 3, &[])?;
    let adsr = [L::AdsrZoneEntry::name, L::AdsrZoneEntry::weaponName];
    records(w, h, B::soundTable__npcADSRZones, B::soundTable__npcADSRZoneCount, L::AdsrZoneEntry::SIZE, 7, &adsr)?;
    records(w, h, B::soundTable__occlusionFilters, B::soundTable__occlusionFilterCount, L::OccludeDef::SIZE, 3, &[])?;
    records(w, h, B::soundTable__playerADSRZones, B::soundTable__playerADSRZoneCount, L::AdsrZoneEntry::SIZE, 7, &adsr)?;
    records(w, h, B::soundTable__reverbs, B::soundTable__reverbCount, L::ReverbDef::SIZE, 3, &[])?;
    records(w, h, B::soundTable__timeScaleSettings, B::soundTable__timeScaleSettingCount, L::TimescaleEntry::SIZE, 3, &[])?;
    {
        use L::ZoneDef as Z;
        let strs = [
            Z::ambientStream,
            Z::zoneName,
            Z::stateName,
            Z::ambientName,
            Z::mixName,
            Z::reverbName,
            Z::filterName,
            Z::occlusionName,
            Z::plrAdsrName,
            Z::npcAdsrName,
            Z::weapReflName,
            Z::fullOccName,
            Z::playerBreathStateName,
        ];
        records(w, h, B::soundTable__zones, B::soundTable__zoneCount, Z::SIZE, 7, &strs)?;
    }
    records(w, h, B::soundTable__unk, B::soundTable__unkCount, L::unk_1453E2FD8::SIZE, 7, &[L::unk_1453E2FD8::name])?;
    records(w, h, B::soundTable__fullOcclusionDefs, B::soundTable__fullOcclusionDefCount, L::FullOcclusionDef::SIZE, 3, &[])?;
    let breath = [L::PlayerBreathStateDef::inhaleAlias, L::PlayerBreathStateDef::exhaleAlias];
    records(w, h, B::soundTable__plrBreathStateDefs, B::soundTable__plrBreathStateDefCount, L::PlayerBreathStateDef::SIZE, 7, &breath)?;

    use L::SndDuck as D;
    let n = w.s.u32_at(h + B::duckCount)? as usize;
    if let Some(t) = w.inline_array(h + B::ducks, D::SIZE * n, 7)? {
        for i in 0..n {
            w.blob(t + D::SIZE * i + D::attenuation, 4 * 128, 31)?;
            w.blob(t + D::SIZE * i + D::lpf, 4 * 128, 31)?;
        }
    }
    use L::SndMusicSet as M;
    use L::SndMusicState as MS;
    let n = w.s.u32_at(h + B::musicSetCount)? as usize;
    if let Some(t) = w.inline_array(h + B::musicSets, M::SIZE * n, 7)? {
        for i in 0..n {
            let m = t + M::SIZE * i;
            let states = w.s.u32_at(m + M::stateCount)? as usize;
            if let Some(st) = w.inline_array(m + M::states, MS::SIZE * states, 7)? {
                for j in 0..states {
                    let s = st + MS::SIZE * j;
                    let loops = w.s.u32_at(s + MS::loopCount)? as usize;
                    w.blob(s + MS::loops, L::SndMusicAsset::SIZE * loops, 3)?;
                }
            }
        }
    }
    w.s.pop();
    w.cap.sound_banks.push(rec);
    Ok(AssetId { ty: AssetType::SoundBank, index: (w.cap.sound_banks.len() - 1) as u32 })
}
