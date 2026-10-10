mod alloc_count;
mod cpu;
pub mod gap;
pub mod wgsl_dump;

pub use cpu::{busy_note, process_cpu_ms, thread_cpu_ms};
pub use alloc_count::{
    ProcessAllocationStats, ProcessCountingAllocator, counting_enabled,
    process_allocation_saturated, process_allocation_slots_used, process_allocations,
    process_live_heap_bytes, release_freed_heap,
};

use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock, mpsc},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

static SINK: OnceLock<Mutex<DiagState>> = OnceLock::new();
static START: OnceLock<Instant> = OnceLock::new();
/// The log writer thread's mailbox. The game's threads never write the log
/// files themselves: one write Windows held up froze a frame for seconds.
static WRITER: OnceLock<mpsc::Sender<Out>> = OnceLock::new();

const WRITER_THREAD: &str = "diag-writer";
/// How long the writer holds lines before they go to the disk (a warning or
/// an error goes at once).
const WRITE_EVERY: Duration = Duration::from_millis(250);

enum Out {
    Line {
        text: String,
        stderr: bool,
        file: bool,
        urgent: bool,
    },
    Trace(String),
    /// Everything sent so far onto the disk, then a word back.
    Flush(mpsc::SyncSender<()>),
}

fn send(out: Out) {
    if let Some(tx) = WRITER.get() {
        let _ = tx.send(out);
    } else if let Out::Line {
        text, stderr: true, ..
    } = out
    {
        eprintln!("{text}");
    }
}

/// Waits (at most `wait`) until every line sent so far is on the disk.
fn flush_writer(wait: Duration) {
    let Some(tx) = WRITER.get() else {
        return;
    };
    if std::thread::current().name() == Some(WRITER_THREAD) {
        return;
    }
    let (done, back) = mpsc::sync_channel(1);
    if tx.send(Out::Flush(done)).is_ok() {
        let _ = back.recv_timeout(wait);
    }
}

/// The writer thread: the lines in order, onto the disk every `WRITE_EVERY`
/// (a warning or an error at once).
fn writer(
    rx: mpsc::Receiver<Out>,
    mut file: Option<BufWriter<File>>,
    mut traces: Option<BufWriter<File>>,
) {
    let mut written = Instant::now();
    loop {
        let urgent = match rx.recv_timeout(WRITE_EVERY) {
            Ok(Out::Line {
                text,
                stderr,
                file: to_file,
                urgent,
            }) => {
                // (No `eprintln!`: a failed write must not end the writer.)
                if stderr {
                    let _ = writeln!(std::io::stderr(), "{text}");
                }
                if to_file && let Some(f) = file.as_mut() {
                    let _ = writeln!(f, "{text}");
                }
                urgent
            }
            Ok(Out::Trace(line)) => {
                if let Some(f) = traces.as_mut() {
                    let _ = writeln!(f, "{line}");
                }
                false
            }
            Ok(Out::Flush(done)) => {
                write_out(&mut file, &mut traces);
                written = Instant::now();
                let _ = done.send(());
                continue;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => true,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                write_out(&mut file, &mut traces);
                return;
            }
        };
        if urgent || written.elapsed() >= WRITE_EVERY {
            write_out(&mut file, &mut traces);
            written = Instant::now();
        }
    }
}

/// One line for stderr that goes out on the writer thread (Bevy's own log
/// lines: a stderr write Windows held up stalled the frame that logged it).
pub struct StderrLine(Vec<u8>);

/// Bevy's log output (`tracing_subscriber`'s `MakeWriter`): each event
/// becomes one `StderrLine`.
pub fn stderr_writer() -> StderrLine {
    StderrLine(Vec::new())
}

impl Write for StderrLine {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Drop for StderrLine {
    fn drop(&mut self) {
        if self.0.is_empty() {
            return;
        }
        let text = String::from_utf8_lossy(&self.0);
        send(Out::Line {
            text: text.trim_end_matches(['\r', '\n']).to_owned(),
            stderr: true,
            file: false,
            urgent: false,
        });
    }
}

fn write_out(file: &mut Option<BufWriter<File>>, traces: &mut Option<BufWriter<File>>) {
    for f in [file, traces].into_iter().flatten() {
        let _ = f.flush();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Channel {
    Launch,
    Zone,
    World,
    Fpv,
    Input,
    Sim,
    Net,
    Ui,
    Audio,
    Console,
}

impl Channel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Launch => "launch",
            Self::Zone => "zone",
            Self::World => "world",
            Self::Fpv => "fpv",
            Self::Input => "input",
            Self::Sim => "sim",
            Self::Net => "net",
            Self::Ui => "ui",
            Self::Audio => "audio",
            Self::Console => "console",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
}

impl Level {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "error" => Some(Self::Error),
            "warn" | "warning" => Some(Self::Warn),
            "info" => Some(Self::Info),
            "debug" => Some(Self::Debug),
            _ => None,
        }
    }
}

struct DiagState {
    /// The log file is open (the writer thread has it).
    file: bool,
    file_path: PathBuf,

    latest: Option<PathBuf>,
    /// The traces file is open (the writer thread has it).
    traces: bool,
    traces_path: Option<PathBuf>,
    stderr_threshold: Level,
    file_threshold: Level,

    last_key: Option<(Channel, Level, String)>,
    last_count: u32,

    started: Instant,
}

pub const LATEST_LOG_NAME: &str = "latest.log";

fn link_latest(path: &Path) -> Option<PathBuf> {
    let dir = path.parent()?;
    let name = path.file_name()?;
    if name == LATEST_LOG_NAME {
        return None;
    }
    let link = dir.join(LATEST_LOG_NAME);
    #[cfg(unix)]
    {
        match std::fs::symlink_metadata(&link) {
            Ok(md) if md.is_symlink() => std::fs::remove_file(&link).ok()?,
            Ok(_) => return None,
            Err(_) => {}
        }
        std::os::unix::fs::symlink(name, &link).ok()?;
        Some(link)
    }
    #[cfg(not(unix))]
    {
        // A symlink needs a privilege Windows withholds by default; a hard link does not.
        match std::fs::remove_file(&link) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return None,
        }
        std::fs::hard_link(path, &link).ok()?;
        Some(link)
    }
}

pub fn init_log(artifacts_root: &Path) -> PathBuf {
    let _ = START.set(Instant::now());
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let path = std::env::var_os("IW4L_LOG")
        .map(PathBuf::from)
        .unwrap_or_else(|| artifacts_root.join("logs").join(format!("{stamp}.log")));
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .ok();

    let (traces, traces_path) = match std::env::var_os("IW4L_TRACES_DIR") {
        Some(dir) => {
            let dir = PathBuf::from(dir);
            let _ = std::fs::create_dir_all(&dir);
            let tp = dir.join(format!("{stamp}.jsonl"));
            let f = OpenOptions::new().create(true).append(true).open(&tp).ok();
            (f, Some(tp))
        }
        None => (None, None),
    };

    let bump = std::env::var("IW4L_LOG_LEVEL")
        .ok()
        .and_then(|s| Level::parse(&s));
    let stderr_threshold = bump.unwrap_or(Level::Error);
    let file_threshold = bump.unwrap_or(Level::Info);

    let started = START.get().copied().unwrap_or_else(Instant::now);
    let mut traces = traces.map(|f| BufWriter::with_capacity(1 << 16, f));
    if let Some(f) = traces.as_mut() {
        let zone = std::env::var("IW4L_ZONE").unwrap_or_default();
        let role = std::env::var("IW4L_ROLE").unwrap_or_else(|_| "Listen".into());
        let games = std::env::var("IW4L_GAMES").unwrap_or_default();
        let git = option_env!("IW4L_GIT_HASH").unwrap_or("unknown");
        let header = serde_json::json!({
            "t": 0,
            "ch": "launch",
            "lvl": "info",
            "msg": "session",
            "ev": "session",
            "f": {
                "format": "iw4l-traces-v1",
                "zone": zone,
                "role": role,
                "IW4L_GAMES": games,
                "git": git,
            }
        });
        let _ = writeln!(f, "{header}");
        let _ = f.flush();
    }

    // The files go to the writer thread; nothing else writes them.
    let opened = (file.is_some(), traces.is_some());
    let file = file.map(|f| BufWriter::with_capacity(1 << 16, f));
    let (tx, rx) = mpsc::channel();
    let writing = std::thread::Builder::new()
        .name(WRITER_THREAD.into())
        .spawn(move || writer(rx, file, traces))
        .is_ok();
    if writing {
        let _ = WRITER.set(tx);
    }
    let state = DiagState {
        file: writing && opened.0,
        file_path: path.clone(),
        latest: link_latest(&path),
        traces: writing && opened.1,
        traces_path,
        stderr_threshold,
        file_threshold,
        last_key: None,
        last_count: 0,
        started,
    };

    let _ = SINK.set(Mutex::new(state));
    install_crash_log(&path);
    path
}

static CRASH_LOG: OnceLock<PathBuf> = OnceLock::new();

/// A crash's last words, straight to the log file (a fresh handle: the
/// sink's lock may be held by the code that crashed).
fn crash_line(text: &str) {
    if let Some(path) = CRASH_LOG.get()
        && let Ok(mut f) = OpenOptions::new().append(true).open(path)
    {
        let _ = writeln!(f, "{text}");
        let _ = f.flush();
    }
}

/// Every crash leaves its reason in the log: a panic (its message, place,
/// thread and backtrace) and, on Windows, an exception the process does not
/// catch (its code and address).
fn install_crash_log(path: &Path) {
    if CRASH_LOG.set(path.to_path_buf()).is_err() {
        return;
    }
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let thread = std::thread::current();
        let name = thread.name().unwrap_or("unnamed");
        let ms = START.get().map_or(0, |s| s.elapsed().as_millis());
        flush_writer(Duration::from_secs(1));
        crash_line(&format!(
            "[{ms}ms] CRASH panic on thread '{name}': {info}
{}",
            std::backtrace::Backtrace::force_capture()
        ));
        previous(info);
    }));
    #[cfg(windows)]
    windows_crash::install();
}

#[cfg(windows)]
#[allow(unsafe_code)]
mod windows_crash {
    #[repr(C)]
    struct ExceptionRecord {
        code: u32,
        flags: u32,
        next: *mut ExceptionRecord,
        address: *mut core::ffi::c_void,
        count: u32,
        info: [usize; 15],
    }

    #[repr(C)]
    struct ExceptionPointers {
        record: *mut ExceptionRecord,
        context: *mut core::ffi::c_void,
    }

    type Filter = unsafe extern "system" fn(*const ExceptionPointers) -> i32;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SetUnhandledExceptionFilter(filter: Option<Filter>) -> Option<Filter>;
    }

    unsafe extern "system" fn on_crash(pointers: *const ExceptionPointers) -> i32 {
        // SAFETY: Windows passes a valid EXCEPTION_POINTERS to the filter.
        let (code, address, info0, info1) = unsafe {
            match pointers.as_ref().and_then(|p| p.record.as_ref()) {
                Some(r) => (r.code, r.address as usize, r.info[0], r.info[1]),
                None => (0, 0, 0, 0),
            }
        };
        let base = module_base();
        super::crash_line(&format!(
            "CRASH exception 0x{code:08X} at 0x{address:X} (exe+0x{:X}) on thread '{}' (info {info0:X} {info1:X})",
            address.wrapping_sub(base),
            std::thread::current().name().unwrap_or("unnamed")
        ));
        super::flush_writer(std::time::Duration::from_secs(1));
        0 // EXCEPTION_CONTINUE_SEARCH: Windows ends the process as before.
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetModuleHandleW(name: *const u16) -> *mut core::ffi::c_void;
    }

    fn module_base() -> usize {
        // SAFETY: a null name asks for the exe's own module; no lifetime involved.
        unsafe { GetModuleHandleW(core::ptr::null()) as usize }
    }

    pub(super) fn install() {
        // SAFETY: the filter is a plain function that lives for the process.
        unsafe {
            SetUnhandledExceptionFilter(Some(on_crash));
        }
    }
}

pub fn latest_log_path() -> Option<PathBuf> {
    SINK.get()
        .and_then(|s| s.lock().ok())
        .and_then(|g| g.latest.clone())
}

pub fn exit_launch_error(message: &str) -> ! {
    write_event(Channel::Launch, Level::Error, message, None, None);
    flush_writer(Duration::from_secs(2));
    if let Some(sink) = SINK.get()
        && let Ok(state) = sink.lock()
    {
        if state.file {
            eprintln!("log: {}", state.file_path.display());
        } else {
            eprintln!("log unavailable: {}", state.file_path.display());
        }
    }
    #[cfg(windows)]
    {
        use std::io::IsTerminal;
        if std::io::stdin().is_terminal() {
            eprintln!("\nPress Enter to close.");
            let _ = std::io::stderr().flush();
            let _ = std::io::stdin().read_line(&mut String::new());
        }
    }
    std::process::exit(2);
}

pub fn traces_path() -> Option<PathBuf> {
    SINK.get()
        .and_then(|s| s.lock().ok())
        .and_then(|g| g.traces_path.clone())
}

fn now_ms(started: Instant) -> u64 {
    started.elapsed().as_millis() as u64
}

fn flush_collapsed(state: &mut DiagState) {
    if state.last_count <= 1 {
        return;
    }
    let Some((ch, lvl, msg)) = state.last_key.clone() else {
        return;
    };
    let n = state.last_count;
    let line = format!("{}  {}  ↑ repeated {n}× — {msg}", ch.as_str(), lvl.as_str());
    emit_raw(state, ch, lvl, &line, Some(n), None, None, true);
    state.last_count = 1;
}

fn emit_raw(
    state: &mut DiagState,
    ch: Channel,
    lvl: Level,
    msg: &str,
    n: Option<u32>,
    ev: Option<&str>,
    fields: Option<&serde_json::Value>,
    collapsed_banner: bool,
) {
    let text = if collapsed_banner {
        msg.to_owned()
    } else {
        format!("{}  {}  {msg}", ch.as_str(), lvl.as_str())
    };

    let stderr = lvl <= state.stderr_threshold;
    let file = lvl <= state.file_threshold && state.file;
    if stderr || file {
        send(Out::Line {
            text,
            stderr,
            file,
            urgent: lvl <= Level::Warn,
        });
    }
    if state.traces {
        let t = now_ms(state.started);
        let mut obj = serde_json::json!({
            "t": t,
            "ch": ch.as_str(),
            "lvl": lvl.as_str(),
            "msg": if collapsed_banner { msg } else { msg },
        });
        if let Some(ev) = ev {
            obj["ev"] = serde_json::Value::String(ev.to_owned());
        }
        if let Some(n) = n.filter(|&n| n > 1) {
            obj["n"] = serde_json::Value::from(n);
        }
        if let Some(f) = fields {
            obj["f"] = f.clone();
        }

        if !collapsed_banner {
            obj["msg"] = serde_json::Value::String(msg.to_owned());
        }
        send(Out::Trace(obj.to_string()));
    }
}

pub fn write_event(
    ch: Channel,
    lvl: Level,
    msg: &str,
    ev: Option<&str>,
    fields: Option<&serde_json::Value>,
) {
    let Some(sink) = SINK.get() else {
        eprintln!("{}  {}  {msg}", ch.as_str(), lvl.as_str());
        return;
    };
    let Ok(mut state) = sink.lock() else {
        return;
    };

    let key = (ch, lvl, msg.to_owned());
    if state.last_key.as_ref() == Some(&key) {
        state.last_count = state.last_count.saturating_add(1);

        if state.last_count.is_multiple_of(64) && state.traces {
            let t = now_ms(state.started);
            let n = state.last_count;
            let obj = serde_json::json!({
                "t": t,
                "ch": ch.as_str(),
                "lvl": lvl.as_str(),
                "msg": msg,
                "n": n,
            });
            send(Out::Trace(obj.to_string()));
        }
        return;
    }

    flush_collapsed(&mut state);
    state.last_key = Some(key);
    state.last_count = 1;
    emit_raw(&mut state, ch, lvl, msg, None, ev, fields, false);
}

/// Everything logged so far onto the disk (the process is about to end).
pub fn flush() {
    if let Some(sink) = SINK.get()
        && let Ok(mut state) = sink.lock()
    {
        flush_collapsed(&mut state);
    }
    flush_writer(Duration::from_secs(2));
}

pub fn announce_log_stdout(path: &Path, latest: Option<&Path>) {
    let line = match latest {
        Some(latest) => format!("log: {} ({})", path.display(), latest.display()),
        None => format!("log: {}", path.display()),
    };
    announce_stdout(&line);
}

pub fn process_elapsed_ns() -> u128 {
    START
        .get()
        .map(|start| start.elapsed().as_nanos())
        .unwrap_or(0)
}

pub fn lifecycle_boundary(name: &str, detail: &str) {
    controller_line("lifecycle", name, detail);
}

pub fn script_boundary(name: &str, detail: &str) {
    controller_line("gsc", name, detail);
}

fn controller_line(prefix: &str, name: &str, detail: &str) {
    let line = format!(
        "{prefix}: {name} pid={} ns={}{detail}",
        std::process::id(),
        process_elapsed_ns()
    );
    write_event(Channel::Launch, Level::Info, &line, None, None);
    announce_stdout(&line);
}

pub fn announce_stdout(line: &str) {
    let _ = writeln!(std::io::stdout(), "{line}");
    let _ = std::io::stdout().flush();
}

pub fn write_line(category: &str, message: &str) {
    let ch = match category {
        "launch" => Channel::Launch,
        "zone" => Channel::Zone,
        "world" => Channel::World,
        "fpv" => Channel::Fpv,
        "input" => Channel::Input,
        "sim" | "spawn" | "score" => Channel::Sim,
        "net" => Channel::Net,
        "ui" | "menu" | "hud" => Channel::Ui,
        "audio" => Channel::Audio,
        "console" => Channel::Console,
        _ => Channel::Launch,
    };
    let msg = if category.is_empty() {
        message.to_owned()
    } else if message.starts_with(category) {
        message.to_owned()
    } else {
        format!("{category}: {message}")
    };
    write_event(ch, Level::Info, &msg, None, None);
}

#[macro_export]
macro_rules! log_line {
    ($cat:ident, $($arg:tt)*) => {{
        $crate::write_line(stringify!($cat), &format!($($arg)*));
    }};
    ($($arg:tt)*) => {{
        $crate::write_line("", &format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! error {
    ($ch:ident, $($arg:tt)*) => {{
        $crate::write_event(
            $crate::Channel::$ch,
            $crate::Level::Error,
            &format!($($arg)*),
            None,
            None,
        );
    }};
}

#[macro_export]
macro_rules! warn {
    ($ch:ident, $($arg:tt)*) => {{
        $crate::write_event(
            $crate::Channel::$ch,
            $crate::Level::Warn,
            &format!($($arg)*),
            None,
            None,
        );
    }};
}

#[macro_export]
macro_rules! info {
    ($ch:ident, $($arg:tt)*) => {{
        $crate::write_event(
            $crate::Channel::$ch,
            $crate::Level::Info,
            &format!($($arg)*),
            None,
            None,
        );
    }};
}

#[macro_export]
macro_rules! debug {
    ($ch:ident, $($arg:tt)*) => {{
        $crate::write_event(
            $crate::Channel::$ch,
            $crate::Level::Debug,
            &format!($($arg)*),
            None,
            None,
        );
    }};
}

#[macro_export]
macro_rules! event {
    ($ch:ident, $lvl:ident, $ev:literal, $($arg:tt)*) => {{
        $crate::write_event(
            $crate::Channel::$ch,
            $crate::Level::$lvl,
            &format!($($arg)*),
            Some($ev),
            None,
        );
    }};
}
