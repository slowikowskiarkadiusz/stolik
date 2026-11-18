//! Lightweight "where is each task right now" tracing.
//!
//! Every task marks the stage it is about to enter (usually right before an `.await`
//! or a blocking call). `report()` runs on CPU0 and prints, once a second, the current
//! stage of every task and for how long it has been sitting there. A task that hangs
//! shows up as a stage whose time keeps growing.

use core::ptr;
use core::sync::atomic::{AtomicPtr, AtomicU32, Ordering};

pub struct TaskTrace {
    pub name: &'static str,
    stage: AtomicPtr<&'static str>,
    since_ms: AtomicU32,
    loops: AtomicU32,
}

impl TaskTrace {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            stage: AtomicPtr::new(ptr::null_mut()),
            since_ms: AtomicU32::new(0),
            loops: AtomicU32::new(0),
        }
    }

    /// Use the `mark!` macro instead of calling this directly.
    pub fn set(&self, stage: &'static &'static str) {
        self.since_ms.store(now_ms(), Ordering::Relaxed);
        self.stage.store(stage as *const &'static str as *mut &'static str, Ordering::Relaxed);
    }

    /// Call once per loop iteration, so the report shows whether the task is still cycling.
    pub fn tick(&self) {
        self.loops.fetch_add(1, Ordering::Relaxed);
    }

    fn stage(&self) -> &'static str {
        let p = self.stage.load(Ordering::Relaxed);
        if p.is_null() {
            return "not started";
        }
        // Safe: only ever points at `static` strings set through `mark!`.
        unsafe { *p }
    }
}

/// `mark!(trace::ENGINE, "Timer::after");`
#[macro_export]
macro_rules! mark {
    ($trace:expr, $stage:literal) => {{
        static STAGE: &str = $stage;
        $trace.set(&STAGE);
    }};
}

pub fn now_ms() -> u32 {
    embassy_time::Instant::now().as_millis() as u32
}

pub static MAIN: TaskTrace = TaskTrace::new("main");
pub static ENGINE: TaskTrace = TaskTrace::new("engine");
pub static DISPLAY: TaskTrace = TaskTrace::new("display");
pub static HUB75: TaskTrace = TaskTrace::new("hub75");
pub static EXPANDER: TaskTrace = TaskTrace::new("expander");

static ALL: [&TaskTrace; 5] = [&MAIN, &ENGINE, &DISPLAY, &HUB75, &EXPANDER];

/// A stage held longer than this is flagged as suspicious.
const STUCK_MS: u32 = 500;

static LAST_LOOPS: [AtomicU32; 5] = [const { AtomicU32::new(0) }; 5];

pub fn report(heap_free: usize) {
    let now = now_ms();
    esp_println::println!("---- t={}ms heap_free={} ----", now, heap_free);
    for (i, t) in ALL.iter().enumerate() {
        let loops = t.loops.load(Ordering::Relaxed);
        let delta = loops.wrapping_sub(LAST_LOOPS[i].swap(loops, Ordering::Relaxed));
        let in_stage = now.wrapping_sub(t.since_ms.load(Ordering::Relaxed));
        let flag = if in_stage > STUCK_MS { "  <<< STUCK?" } else { "" };
        esp_println::println!(
            "  {:<9} loops/s={:<4} total={:<8} stage=\"{}\" for {}ms{}",
            t.name,
            delta,
            loops,
            t.stage(),
            in_stage,
            flag
        );
    }
}
