use crate::engine::threading_provider::TThread;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

pub struct DesktopThread {
    pub thread: JoinHandle<()>,
}

impl DesktopThread {
    pub fn new(thread: JoinHandle<()>) -> DesktopThread {
        DesktopThread { thread }
    }
}

impl TThread for DesktopThread {
    fn start(handle: Box<dyn Fn() + Send>) -> DesktopThread {
        let t = thread::spawn(move || {
            handle();
        });
        DesktopThread::new(t)
    }

    fn sleep_for(ms: u32) {
        thread::sleep(Duration::from_millis(1000));
    }

    fn stop(&self) {
        // self.thread
    }
}
