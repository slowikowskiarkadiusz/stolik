use core::engine::threading_provider::Thread;
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

impl Thread for DesktopThread {
    fn start(handle: Box<dyn Fn() + Send>) -> DesktopThread {
        let t = thread::spawn(move || {
            handle();
        });
        DesktopThread::new(t)
    }

    fn sleep_for(ms: u64) {
        thread::sleep(Duration::from_millis(ms));
    }

    fn stop(&self) {
        // self.thread
    }
}
