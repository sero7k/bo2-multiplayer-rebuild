//! IW4L_GPU_REPORT=<secs>: every that many seconds, one log line says what the
//! graphics allocator holds: its blocks by size and the biggest owners of the
//! memory in them. Off by default. It shows which buffers keep upload memory
//! alive (on a card without resizable BAR that memory is pinned RAM, which
//! the process's working set does not count).

use bevy::prelude::*;
use bevy::render::renderer::RenderDevice;

fn every() -> Option<f64> {
    static EVERY: std::sync::OnceLock<Option<f64>> = std::sync::OnceLock::new();
    *EVERY.get_or_init(|| {
        let secs = std::env::var("IW4L_GPU_REPORT").ok()?.trim().parse::<f64>().ok()?;
        (secs > 0.0).then_some(secs)
    })
}

pub fn gpu_report_enabled() -> bool {
    every().is_some()
}

pub fn report_gpu_allocator(device: Res<RenderDevice>, mut next: Local<Option<std::time::Instant>>) {
    let Some(secs) = every() else {
        return;
    };
    let now = std::time::Instant::now();
    if next.is_some_and(|at| now < at) {
        return;
    }
    *next = Some(now + std::time::Duration::from_secs_f64(secs));
    let Some(report) = device.wgpu_device().generate_allocator_report() else {
        diag::info!(World, "gpu memory: this backend has no allocator report");
        return;
    };
    const MB: f64 = 1024.0 * 1024.0;
    let mut sizes = std::collections::BTreeMap::<u64, (usize, usize)>::new();
    for block in &report.blocks {
        let entry = sizes.entry(block.size).or_default();
        entry.0 += 1;
        entry.1 += block.allocations.len();
    }
    let blocks = sizes
        .iter()
        .rev()
        .map(|(size, (count, allocs))| format!("{:.0}MB x{count} ({allocs} allocations)", *size as f64 / MB))
        .collect::<Vec<_>>()
        .join(", ");
    let mut owners = std::collections::HashMap::<&str, (u64, usize)>::new();
    for allocation in &report.allocations {
        let entry = owners.entry(allocation.name.as_str()).or_default();
        entry.0 += allocation.size;
        entry.1 += 1;
    }
    let mut owners = owners.into_iter().collect::<Vec<_>>();
    owners.sort_by(|a, b| b.1.0.cmp(&a.1.0));
    let top = owners
        .iter()
        .take(12)
        .map(|(name, (bytes, count))| format!("{name}: {:.1}MB x{count}", *bytes as f64 / MB))
        .collect::<Vec<_>>()
        .join("; ");
    diag::info!(
        World,
        "gpu memory: {:.0} MB used of {:.0} MB held in {} blocks [{blocks}]; biggest: {top}",
        report.total_allocated_bytes as f64 / MB,
        report.total_reserved_bytes as f64 / MB,
        report.blocks.len(),
    );
}
