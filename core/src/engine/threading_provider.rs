// pub trait ThreadingProvider<T: TThread> {
//     fn start(&self, handle: fn()) -> T;
//     fn sleep_for(ms: u32);
// }

pub trait TThread {
    fn stop(&self);
    fn start(handle: fn()) -> Self;
    fn sleep_for(ms: u32);
}
