//! Which of each game's files the rebuild reads: a game's file is copied
//! when it matches one of its KEEP patterns and none of its DROP ones (the
//! patterns are `glob`'s, relative to the game's folder).
//!
//! Never read for any game: videos, installers and redistributables, other
//! languages' files, crash reporters. Per game, what its loaders open (see
//! each list).

use crate::glob::matches;

pub struct Rules {
    pub keep: &'static [&'static str],
    pub drop: &'static [&'static str],
}

impl Rules {
    pub fn keeps(&self, rel: &str) -> bool {
        !COMMON_DROP.iter().any(|p| matches(p, rel)) && !self.drop.iter().any(|p| matches(p, rel)) && self.keep.iter().any(|p| matches(p, rel))
    }
}

/// Never read, whatever the game.
const COMMON_DROP: &[&str] = &[
    "**/*.bik",
    "**/*.mkv",
    "**/*.webm",
    "**/*.mp4",
    "video/**",
    "raw/video/**",
    "redist/**",
    "_commonredist/**",
    "directx/**",
    "**/*.cab",
    "**/vcredist*",
    "**/dxsetup*",
    "**/crashreporter*",
    "**/*.log",
    "**/*.dmp",
    "**/*.tmp",
    // (Other languages; the rebuild reads English.)
    "**/*.french.*",
    "**/*.german.*",
    "**/*.italian.*",
    "**/*.spanish.*",
    "**/*.polish.*",
    "**/*.russian.*",
    "**/*.japanese.*",
    "**/*.portuguese.*",
    "**/localized_french*",
    "**/localized_german*",
    "**/localized_italian*",
    "**/localized_spanish*",
    "**/localized_polish*",
    "**/localized_russian*",
    "**/localized_japanese*",
    "**/localized_portuguese*",
    "**/french/**",
    "**/german/**",
    "**/italian/**",
    "**/spanish/**",
    "**/polish/**",
    "**/russian/**",
    "**/japanese/**",
    "**/portuguese/**",
    "**/fulljap/**",
    "**/chinese/**",
    "**/korean/**",
    "**/arabic/**",
];

/// Everything but the above.
const ALL: Rules = Rules { keep: &["**"], drop: &[] };

/// Black Ops II: its multiplayer and Zombies zones (every `zone/all` zone is
/// kept: campaign and Strike Force are a small share), its image packs and
/// its English and shared sound banks.
const BO2: Rules = Rules {
    keep: &["zone/all/*.ff", "zone/all/*.ipak", "zone/english/*.ff", "zone/english/*.ipak", "sound/*.all.sab*", "sound/*.english.sab*"],
    drop: &[],
};

/// Black Ops: its multiplayer zones and English ones, its IWDs (pictures,
/// sounds; Black Ops II's DLC5 pack borrows from them). Only English: two
/// language folders make its language lookup ambiguous.
const BO1: Rules = Rules {
    keep: &["zone/common/*_mp.ff", "zone/common/mp_*.ff", "zone/english/en_*.ff", "main/*.iwd"],
    drop: &["**/*_sp.ff", "**/*_zm.ff"],
};

/// Modern Warfare 2 and 3: the multiplayer zones (MW2's common_mp is every
/// match's), the maps with their load zones, every IWD (and IW4x's and the
/// usermaps for MW2).
const IW4_IW5: Rules = Rules {
    keep: &["zone/**/*_mp.ff", "zone/**/mp_*.ff", "main/*.iwd", "main/iw4x/**", "iw4x/**", "usermaps/**"],
    drop: &[],
};

/// World at War and Call of Duty 4: the multiplayer zones and maps, every
/// IWD (WaW's guns' sounds stream from them).
const T4_IW3: Rules = Rules {
    keep: &["zone/**/*_mp.ff", "zone/**/mp_*.ff", "main/*.iwd"],
    drop: &["**/nazi_zombie_*"],
};

/// Ghosts: its multiplayer zones and maps, every image and sound pack (which
/// pack numbers a map wants is only known when it loads).
const GHOSTS: Rules = Rules {
    keep: &["zone/*_mp.ff", "zone/mp_*.ff", "zone/imagefile*.pak", "zone/soundfile*.pak", "zone/english/eng_soundfile*.pak", "zone/english/*.ff"],
    drop: &["zone/mp_alien_*"],
};

/// Advanced Warfare: its multiplayer zones and maps (its maps are coming;
/// its guns today), the guns' view and world zones, and every pack.
const AW: Rules = Rules {
    keep: &["zone/*_mp.ff", "zone/mp_*.ff", "zone/imagefile*.pak", "zone/soundfile*.pak", "zone/english/*.ff", "zone/english/eng_soundfile*.pak"],
    drop: &["zone/*_zm_mp.ff", "zone/mp_zombie_*"],
};

/// Black Ops III: its common zones, its multiplayer maps, every package
/// (each map draws on the shared ones), and the multiplayer sound bank.
const BO3: Rules = Rules {
    keep: &[
        "zone/core_*.ff",
        "zone/mp_*.ff",
        "zone/zm_common.ff",
        "zone/zm_levelcommon.ff",
        "zone/*.xpak",
        "zone/**/mp_common.all.sab*",
        "zone/**/mp_common.en.sab*",
    ],
    drop: &["zone/cp_*"],
};

/// Modern Warfare III (2023): the zones its two maps and Rebirth Island read,
/// its guns' part zones, every package (opened together) and its
/// decompressor.
const JUP: Rules = Rules {
    keep: &[
        "oo2core_*.dll",
        "zone/ingame*.ff",
        "zone/eng_ingame*.ff",
        "zone/global*.ff",
        "zone/code_post_gfx*.ff",
        "zone/mp_jup_escape5*",
        "zone/eng_mp_jup_escape5*",
        "zone/mp_jup_elevated*",
        "zone/eng_mp_jup_elevated*",
        "zone/mp_av_*mike93*",
        "zone/mp_aw_*mike93*",
        "zone/*.xpak",
        "zone/*.xsub",
        "jupiter/assets/mapents/**",
    ],
    drop: &[],
};

/// Infinite Warfare: its menus' zone, its multiplayer and Zombies maps with
/// their technique zones and patches, the shared zones and the asset and
/// image packs (wherever the zones sit). None of its sound is read.
const IW: Rules = Rules {
    keep: &["**/ui.ff", "**/iw7_ui.ff", "**/mp_*.ff", "**/cp_*.ff", "**/techsets_*.ff", "**/patch_*.ff", "**/common*.ff", "**/global*.ff", "**/assetfile*.pak", "**/imagefile*.pak", "**/iw7_imagefile*.bin"],
    drop: &["**/*.sab*", "**/soundfile*"],
};

/// The @leetinservers mod: its zone, its settings and the archives at its
/// top.
const BO2MOD: Rules = Rules { keep: &["mod.ff", "mod.json", "*.iwd", "*.sab*"], drop: &[] };

pub fn for_game(key: &str) -> Rules {
    match key {
        "bo2" => BO2,
        "bo1" => BO1,
        "mw2" | "mw3" => IW4_IW5,
        "waw" | "cod4" => T4_IW3,
        "ghosts" => GHOSTS,
        "aw" => AW,
        "bo3" => BO3,
        "jup" => JUP,
        "iw" => IW,
        "bo2mod" => BO2MOD,
        // (The DLC5 pack, and any game not listed: whole.)
        _ => ALL,
    }
}

#[cfg(test)]
mod tests {
    use super::for_game;

    #[test]
    fn videos_and_installers_never_go() {
        let r = for_game("bo2");
        assert!(r.keeps("zone/all/common_mp.ff"));
        assert!(!r.keeps("video/intro.bik"));
        assert!(!r.keeps("redist/DirectX/DXSETUP.exe"));
        assert!(!r.keeps("zone/french/common_mp.ff"));
        assert!(r.keeps("sound/mpl_common.all.sabs"));
        assert!(!r.keeps("sound/mpl_common.french.sabs"));
    }

    #[test]
    fn each_game_keeps_what_it_reads() {
        assert!(for_game("bo1").keeps("main/iw_00.iwd"));
        assert!(!for_game("bo1").keeps("main/localized_french_iw00.iwd"));
        assert!(for_game("mw2").keeps("main/localized_english_iw00.iwd"));
        assert!(for_game("mw2").keeps("zone/english/mp_rust_load.ff"));
        assert!(!for_game("mw2").keeps("zone/english/cliffhanger.ff"));
        assert!(for_game("bo3").keeps("zone/base.xpak"));
        assert!(for_game("bo3").keeps("zone/snd/all/mp_common.all.sabs"));
        assert!(!for_game("bo3").keeps("zone/snd/all/zm_genesis.all.sabs"));
        assert!(!for_game("bo3").keeps("zone/cp_mi_eth_prologue.ff"));
        assert!(for_game("iw").keeps("ui.ff") && for_game("iw").keeps("zone/mp_breakneck.ff"));
        assert!(!for_game("iw").keeps("english/soundfile1.pak"));
        assert!(for_game("aw").keeps("zone/mp_venus.ff"));
        assert!(for_game("aw").keeps("zone/imagefile36.pak"));
        assert!(for_game("aw").keeps("zone/mp_vm_bal27_tr.ff"));
        assert!(for_game("jup").keeps("oo2core_8_win64.dll"));
        assert!(!for_game("bo2mod").keeps("sub/notes.txt"));
        assert!(for_game("bo2extra").keeps("mods/dlc5/mod.ff"));
    }
}
