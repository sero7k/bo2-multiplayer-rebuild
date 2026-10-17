//! Path patterns: `/`-separated, case-insensitive; `*` is any run of
//! characters inside one folder or file name, `**` any number of folders
//! (none too).

/// Whether `path` (relative, `/`-separated) matches `pattern`.
#[must_use]
pub fn matches(pattern: &str, path: &str) -> bool {
    let pat: Vec<String> = pattern.to_ascii_lowercase().split('/').map(str::to_owned).collect();
    let parts: Vec<String> = path.to_ascii_lowercase().split('/').map(str::to_owned).collect();
    segs(&pat, &parts)
}

fn segs(pat: &[String], parts: &[String]) -> bool {
    match pat.split_first() {
        None => parts.is_empty(),
        Some((p, rest)) if p == "**" => (0..=parts.len()).any(|k| segs(rest, &parts[k..])),
        Some((p, rest)) => parts.split_first().is_some_and(|(first, more)| name(p.as_bytes(), first.as_bytes()) && segs(rest, more)),
    }
}

fn name(p: &[u8], s: &[u8]) -> bool {
    match p.split_first() {
        None => s.is_empty(),
        Some((b'*', rest)) => (0..=s.len()).any(|k| name(rest, &s[k..])),
        Some((c, rest)) => s.split_first().is_some_and(|(d, more)| c == d && name(rest, more)),
    }
}

#[cfg(test)]
mod tests {
    use super::matches;

    #[test]
    fn patterns() {
        assert!(matches("zone/*.ff", "zone/common_mp.ff"));
        assert!(matches("zone/*.ff", "Zone/Common_MP.FF"));
        assert!(!matches("zone/*.ff", "zone/english/common_mp.ff"));
        assert!(matches("zone/**/*.ff", "zone/english/common_mp.ff"));
        assert!(matches("zone/**/*.ff", "zone/common_mp.ff"));
        assert!(matches("**/*.bik", "raw/video/intro.bik"));
        assert!(matches("**", "anything/at/all.txt"));
        assert!(matches("zone/cp_*", "zone/cp_lotus.xpak"));
        assert!(!matches("zone/cp_*", "zone/mp_lotus.xpak"));
    }
}
