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

pub fn for_game(key: &str) -> Rules {
    match key {
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
    }
}
