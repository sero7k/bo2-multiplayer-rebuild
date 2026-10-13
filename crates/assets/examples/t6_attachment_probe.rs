//! Prints T6 attachments' stat scales: `t6_attachment_probe <zone.ff>`.
use fastfile_t6::layout::WeaponAttachment as a;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let c = asset_t6::capture_zone(std::path::Path::new(&args[1])).unwrap();
    for at in &c.attachments {
        let f = |o| at.f32_at(o);
        println!(
            "{:10} type {:2} fire {} dmgRange {} adsIn {} adsOut {} recRate {} recLim {} kickCenter {} adsMove {} hipMin {} hipMax {} reload {} reloadEmpty {} reloadAdd {} quick {} quickEmpty {} clipScale {} clip {} perks {:#x} {:#x} stack {} silenced {} laser {} dual {} cf {} {} {}",
            at.name,
            at.i32_at(a::attachmentType),
            f(a::fFireTimeScale),
            f(a::fDamageRangeScale),
            f(a::fAdsTransInTimeScale),
            f(a::fAdsTransOutTimeScale),
            f(a::fAdsRecoilReductionRate),
            f(a::fAdsRecoilReductionLimit),
            f(a::fAdsViewKickCenterSpeedScale),
            f(a::adsMoveSpeedScale),
            f(a::fHipSpreadMinScale),
            f(a::fHipSpreadMaxScale),
            f(a::fReloadTimeScale),
            f(a::fReloadEmptyTimeScale),
            f(a::fReloadAddTimeScale),
            f(a::fReloadQuickTimeScale),
            f(a::fReloadQuickEmptyTimeScale),
            f(a::clipSizeScale),
            at.i32_at(a::iClipSize),
            at.i32_at(a::perks),
            at.i32_at(a::perks + 4),
            at.i32_at(a::stackFire),
            at.u8_at(a::bSilenced),
            at.u8_at(a::laserSight),
            at.u8_at(a::bDualMag),
            f(a::customFloat0),
            f(a::customFloat1),
            f(a::customFloat2),
        );
    }
    println!("uniques {}", c.attachment_uniques.len());
}
