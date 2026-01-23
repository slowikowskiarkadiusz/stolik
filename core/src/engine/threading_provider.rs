pub trait TThread {
    fn start(handle: Box<dyn Fn() + Send>) -> Self;
    fn stop(&self);
    fn sleep_for(ms: u32);
}
