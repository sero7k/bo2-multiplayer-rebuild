//! `<artifacts>/audio_manifest.csv`: every music candidate the director knows
//! of — the game tracks, the music in the front end's bank, the user's files
//! and the game folders this build cannot read — with whether it plays and
//! which playlists it is in.

use std::fmt::Write as _;
use std::path::Path;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ManifestRow {
    pub game: String,
    /// Zone, archive path or file path.
    pub source: String,
    pub alias: String,
    pub format: String,
    pub duration_s: Option<f64>,
    /// menu / lobby / ambience / other.
    pub intended_use: String,
    /// yes / no / unknown.
    pub playable: String,
    pub reason: String,
    /// The playlists it is in, `menu|lobby|match`, or empty.
    pub playlists: String,
}

pub const HEADER: &str =
    "game,source,alias,format,duration_s,intended_use,playable,reason,playlists";

fn field(out: &mut String, value: &str) {
    if value.contains([',', '"', '\n', '\r']) {
        out.push('"');
        out.push_str(&value.replace('"', "\"\""));
        out.push('"');
    } else {
        out.push_str(value);
    }
}

pub fn to_csv(rows: &[ManifestRow]) -> String {
    let mut out = String::from(HEADER);
    out.push('\n');
    for r in rows {
        let duration = r.duration_s.map(|d| format!("{d:.2}")).unwrap_or_default();
        for (i, v) in [
            &r.game,
            &r.source,
            &r.alias,
            &r.format,
            &duration,
            &r.intended_use,
            &r.playable,
            &r.reason,
            &r.playlists,
        ]
        .into_iter()
        .enumerate()
        {
            if i > 0 {
                out.push(',');
            }
            field(&mut out, v);
        }
        out.push('\n');
    }
    out
}

pub fn write(path: &Path, rows: &[ManifestRow]) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("csv.tmp");
    std::fs::write(&tmp, to_csv(rows)).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}

/// A one-line tally for the log.
pub fn summary(rows: &[ManifestRow]) -> String {
    let mut out = String::new();
    let yes = rows.iter().filter(|r| r.playable == "yes").count();
    let no = rows.iter().filter(|r| r.playable == "no").count();
    let listed = rows.iter().filter(|r| !r.playlists.is_empty()).count();
    let _ = write!(
        out,
        "{} candidates ({yes} playable, {no} not, {} unknown), {listed} in playlists",
        rows.len(),
        rows.len() - yes - no
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_quotes_what_needs_it() {
        let rows = vec![ManifestRow {
            game: "user file".into(),
            source: "C:\\music\\menu\\Rising Tide, Remastered.mp3".into(),
            alias: String::new(),
            format: "mp3".into(),
            duration_s: Some(181.234),
            intended_use: "menu".into(),
            playable: "yes".into(),
            reason: "say \"hi\"".into(),
            playlists: "menu|lobby".into(),
        }];
        let csv = to_csv(&rows);
        let line = csv.lines().nth(1).unwrap();
        assert_eq!(
            line,
            "user file,\"C:\\music\\menu\\Rising Tide, Remastered.mp3\",,mp3,181.23,menu,yes,\"say \"\"hi\"\"\",menu|lobby"
        );
        assert!(summary(&rows).starts_with("1 candidates (1 playable"));
    }
}
