use crate::engine::threading_provider::TThread;
use crate::engine::threading_provider::ThreadingProvider;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

pub struct DesktopThreadProvider;

impl DesktopThreadProvider {
    pub fn new() -> Self {
        Self {}
    }
}

impl ThreadingProvider<DesktopThread> for DesktopThreadProvider {
    fn start(&self, handle: fn()) -> DesktopThread {
        let t = thread::spawn(handle);
        DesktopThread::new(t)
    }

    fn sleep_for(ms: u32) {
        thread::sleep(Duration::from_millis(1000));
    }
}

pub struct DesktopThread {
    pub thread: JoinHandle<()>,
}

impl DesktopThread {
    pub fn new(thread: JoinHandle<()>) -> DesktopThread {
        DesktopThread { thread }
    }
}

impl TThread for DesktopThread {
    fn stop(&self) {
        // self.thread
    }
}
