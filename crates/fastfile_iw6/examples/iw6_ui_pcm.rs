//! Ghosts' countdown tick written as a WAV: `iw6_ui_pcm <install> <out.wav>`.
fn main() {
    let mut a = std::env::args().skip(1);
    let install = std::path::PathBuf::from(a.next().expect("install"));
    let out = a.next().expect("out");
    let p = fastfile_iw6::ui_sound::loaded_pcm(&install, fastfile_iw6::ui_sound::COUNTDOWN_ZONE, fastfile_iw6::ui_sound::COUNTDOWN_ALIAS).unwrap();
    let peak = p.samples.iter().fold(0f32, |m, s| m.max(s.abs()));
    let rms = (p.samples.iter().map(|s| s * s).sum::<f32>() / p.samples.len() as f32).sqrt();
    println!("{} samples {}ch {}Hz {:.3}s peak {peak:.3} rms {rms:.3}", p.samples.len(), p.channels, p.sample_rate, p.samples.len() as f32 / p.channels as f32 / p.sample_rate as f32);
    let data: Vec<u8> = p.samples.iter().flat_map(|s| ((s * 32767.0) as i16).to_le_bytes()).collect();
    let mut w = Vec::new();
    w.extend(b"RIFF"); w.extend((36 + data.len() as u32).to_le_bytes()); w.extend(b"WAVEfmt ");
    w.extend(16u32.to_le_bytes()); w.extend(1u16.to_le_bytes()); w.extend(p.channels.to_le_bytes());
    w.extend(p.sample_rate.to_le_bytes()); w.extend((p.sample_rate * 2 * p.channels as u32).to_le_bytes());
    w.extend((2 * p.channels).to_le_bytes()); w.extend(16u16.to_le_bytes()); w.extend(b"data");
    w.extend((data.len() as u32).to_le_bytes()); w.extend(data);
    std::fs::write(out, w).unwrap();
}
