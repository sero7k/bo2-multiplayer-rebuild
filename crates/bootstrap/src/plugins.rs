use assets::AssetPlugin;
use audio::AudioPlugin;
use bevy::{
    log::LogPlugin,
    prelude::*,
    render::{
        RenderPlugin as BevyRenderPlugin,
        pipelined_rendering::PipelinedRenderingPlugin,
        settings::{RenderCreation, WgpuFeatures, WgpuSettings},
    },
};
use bots::BotsPlugin;
use console::ConsolePlugin;
use frame::RuntimeRole;
use hud::HudPlugin;
use net::NetPlugin;
use render::RenderPlugin;
use replay::ReplayPlugin;
use session::SessionPlugin;
use ui::UiPlugin;

/// Bevy's log lines go to stderr on the log writer's thread, never from the
/// frame that logged them (a write Windows held up stalled it for seconds).
fn log_lines(_: &mut App) -> Option<bevy::log::BoxedFmtLayer> {
    Some(Box::new(
        bevy::log::tracing_subscriber::fmt::Layer::default().with_writer(diag::stderr_writer),
    ))
}

pub fn add_runtime_plugins(app: &mut App) {
    add_runtime_plugins_with_role(app, RuntimeRole::Listen);
}

pub fn add_runtime_plugins_with_role(app: &mut App, role: RuntimeRole) {
    let net = match role {
        RuntimeRole::Listen => NetPlugin::listen(),
        RuntimeRole::Dedicated => NetPlugin::dedicated(),
        RuntimeRole::Client => NetPlugin::client(),
        RuntimeRole::Replay => NetPlugin::replay(),
    };
    if crate::launch::test_window_hidden() {
        // A hidden test run: about 30 updates a second, whatever the focus
        // (the server keeps its own 20 Hz), so it costs the PC little.
        let pace = bevy::winit::UpdateMode::reactive_low_power(std::time::Duration::from_millis(33));
        app.insert_resource(bevy::winit::WinitSettings {
            focused_mode: pace,
            unfocused_mode: pace,
        });
    } else if crate::bench::enabled() || std::env::var_os("IW4L_WINDOW_POS").is_some() {
        // Unfocused benchmarks (and test runs on another monitor) must not
        // inherit the window runner's 60 Hz sleep.
        app.insert_resource(bevy::winit::WinitSettings::continuous());
    } else if !frame::Headless::requested() && !app.world().contains_resource::<frame::Headless>() {
        // bo2mp: the window he plays in runs flat out, focused or not. Test
        // games are hidden or headless now and keep their own pace above, so
        // a background cap only slowed his game: with another window focused
        // it dropped to 30 fps, which read as "laggier towards the end of the
        // match". IW4L_BACKGROUND_FPS still caps it when set (0 for no limit).
        let fps = std::env::var("IW4L_BACKGROUND_FPS").ok().and_then(|v| v.trim().parse::<u32>().ok()).unwrap_or(0);
        if fps > 0 {
            diag::info!(Launch, "window: {fps} fps while another window has focus (IW4L_BACKGROUND_FPS)");
        }
        app.insert_resource(bevy::winit::WinitSettings {
            focused_mode: bevy::winit::UpdateMode::Continuous,
            unfocused_mode: match fps {
                0 => bevy::winit::UpdateMode::Continuous,
                fps => bevy::winit::UpdateMode::reactive_low_power(std::time::Duration::from_secs_f64(1.0 / fps as f64)),
            },
        });
    }
    app.add_plugins(AssetPlugin)
        .add_plugins(UiPlugin)
        .add_plugins(ConsolePlugin)
        .add_plugins(net)
        .add_plugins((BotsPlugin, HudPlugin))
        .add_plugins(AudioPlugin)
        .add_plugins(ReplayPlugin)
        .add_plugins(RenderPlugin)
        .add_plugins(SessionPlugin);

    app.edit_schedule(Update, |schedule| {
        schedule.set_executor(bevy::ecs::schedule::SingleThreadedExecutor::new());
    });

    app.edit_schedule(First, |schedule| {
        schedule.set_executor(bevy::ecs::schedule::SingleThreadedExecutor::new());
    });
    app.edit_schedule(PreUpdate, |schedule| {
        schedule.set_executor(bevy::ecs::schedule::SingleThreadedExecutor::new());
    });
    app.edit_schedule(PostUpdate, |schedule| {
        schedule.set_executor(bevy::ecs::schedule::SingleThreadedExecutor::new());
    });
    app.edit_schedule(Last, |schedule| {
        schedule.set_executor(bevy::ecs::schedule::SingleThreadedExecutor::new());
    });

    if let Some(render_app) = app.get_sub_app_mut(bevy::render::RenderApp) {
        render_app.edit_schedule(bevy::render::renderer::RenderGraph, |schedule| {
            schedule.set_executor(bevy::ecs::schedule::SingleThreadedExecutor::new());
        });
        render_app.edit_schedule(bevy::core_pipeline::Core3d, |schedule| {
            schedule.set_executor(bevy::ecs::schedule::SingleThreadedExecutor::new());
        });
        render_app.edit_schedule(bevy::core_pipeline::Core2d, |schedule| {
            schedule.set_executor(bevy::ecs::schedule::SingleThreadedExecutor::new());
        });
        render_app.edit_schedule(bevy::render::Render, |schedule| {
            schedule.set_executor(bevy::ecs::schedule::SingleThreadedExecutor::new());
        });
        render_app.edit_schedule(bevy::render::ExtractSchedule, |schedule| {
            schedule.set_executor(bevy::ecs::schedule::SingleThreadedExecutor::new());
        });
    }
}

pub fn assemble_listen_app() -> App {
    let mut app = App::new();
    app.add_plugins(default_plugins_with_quiet_log(WindowPlugin {
        primary_window: None,
        ..default()
    }));
    add_runtime_plugins(&mut app);
    app
}

pub fn default_plugins_with_quiet_log(mut window: WindowPlugin) -> bevy::app::PluginGroupBuilder {
    let headless = frame::Headless::requested();
    if headless {
        window.primary_window = None;
        window.exit_condition = bevy::window::ExitCondition::DontExit;
    }
    if let Some(primary) = window.primary_window.as_mut() {
        primary.desired_maximum_frame_latency = core::num::NonZeroU32::new(frame_latency());
    }
    let mut wgpu = WgpuSettings::default();
    wgpu.features |= WgpuFeatures::TEXTURE_FORMAT_16BIT_NORM
        | WgpuFeatures::TEXTURE_COMPRESSION_BC
        | WgpuFeatures::POLYGON_MODE_LINE
        | WgpuFeatures::TEXTURE_BINDING_ARRAY
        | WgpuFeatures::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
        | WgpuFeatures::PARTIALLY_BOUND_BINDING_ARRAY;
    // bo2mp: the card's memory comes in 8-64 MB blocks instead of 128-256 MB
    // ones, so a game holds less it does not use (8 games share one card).
    // IW4L_GPU_MEMORY=performance brings the big blocks back.
    if !std::env::var("IW4L_GPU_MEMORY").is_ok_and(|v| v.trim().eq_ignore_ascii_case("performance")) {
        wgpu.memory_hints = bevy::render::settings::MemoryHints::MemoryUsage;
    }
    if headless {
        // No backends: bevy makes no graphics device and no render world.
        wgpu.backends = None;
    }
    let plugins = DefaultPlugins
        .set(window)
        .set(log_plugin())
        .set(BevyRenderPlugin {
            render_creation: RenderCreation::Automatic(Box::new(wgpu)),
            ..default()
        });
    if headless {
        // No window loop, sound device or gamepads: a plain loop paced like a
        // hidden test run (about 30 updates a second; the server keeps 20 Hz).
        // With no render world bevy also leaves out its entity sync, yet the
        // components it syncs still report their removal to it: a despawned
        // camera or mesh would panic. Its queue is never drained here; the
        // records are a few bytes each.
        return plugins
            .disable::<bevy::winit::WinitPlugin>()
            .disable::<bevy::audio::AudioPlugin>()
            .disable::<bevy::gilrs::GilrsPlugin>()
            .disable::<PipelinedRenderingPlugin>()
            .add(bevy::render::sync_world::SyncWorldPlugin)
            .add(bevy::app::ScheduleRunnerPlugin::run_loop(
                std::time::Duration::from_millis(33),
            ));
    }
    if pipelined_rendering() {
        plugins
    } else {
        plugins.disable::<PipelinedRenderingPlugin>()
    }
}

fn log_plugin() -> LogPlugin {
    #[cfg(feature = "span-stats")]
    return LogPlugin {
        filter: crate::span_stats::FILTER.into(),
        level: bevy::log::Level::WARN,
        custom_layer: crate::span_stats::layer,
        fmt_layer: log_lines,
        ..default()
    };
    #[cfg(not(feature = "span-stats"))]
    LogPlugin {
        filter: "warn,iw4l=info".into(),
        level: bevy::log::Level::WARN,
        fmt_layer: log_lines,
        ..default()
    }
}

const PIPELINED_RENDERING_ENV: &str = "IW4L_PIPELINED_RENDERING";

/// Overlap rendering with the next main frame. Extraction remains the ownership
/// boundary; the bounded render channel permits one outstanding frame.
/// Set IW4L_PIPELINED_RENDERING=0 for synchronous presentation.
///
/// Off by default on macOS: AppKit only lets the main thread touch the NSView
/// behind the Metal surface. Bevy hands `create_surfaces` back to the main
/// thread through the multi-threaded executor, which the single-threaded
/// `Render` schedule above bypasses, so the render thread would create it and
/// panic in `raw-window-metal`.
fn pipelined_rendering() -> bool {
    match std::env::var_os(PIPELINED_RENDERING_ENV) {
        None => !cfg!(target_os = "macos"),
        Some(_) => perf::switch(PIPELINED_RENDERING_ENV),
    }
}

const FRAME_LATENCY_ENV: &str = "IW4L_FRAME_LATENCY";

/// How many frames the surface is asked to let the CPU run ahead of the GPU.
///
/// wgpu calls this a hint and the backend is free to clamp it: on Vulkan it is
/// bound to the number of swapchain images, so a run that asked for two did
/// not necessarily get two, and only a measurement says which. Three is the
/// default because it is Black Ops II's: its renderer presents with the sync
/// interval and never sets a frame latency, so D3D's own default of three
/// holds. The variable exists so the
/// other arm needs no rebuild, and the manifest records the number that was
/// asked for — never the number the driver granted, which this process cannot
/// read back.
///
/// The value is a count, not a switch: anything unparseable or zero is the
/// default, and says so rather than silently picking an arm. Read once, so the
/// window and the manifest cannot disagree and the complaint is made once.
pub(crate) fn frame_latency() -> u32 {
    const DEFAULT: u32 = 3;
    static FRAMES: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    *FRAMES.get_or_init(|| {
        let Some(asked) = std::env::var_os(FRAME_LATENCY_ENV) else {
            return DEFAULT;
        };
        match asked.to_str().map(str::trim).and_then(|v| v.parse().ok()) {
            Some(frames) if frames > 0 => frames,
            _ => {
                diag::warn!(
                    Launch,
                    "{FRAME_LATENCY_ENV}={} is not a frame count; using {DEFAULT}",
                    asked.to_string_lossy(),
                );
                DEFAULT
            }
        }
    })
}
