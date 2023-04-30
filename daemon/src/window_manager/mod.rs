pub mod windows;

pub trait WindowManager {
    fn get_bg_window(&self) -> Option<usize>;
    fn get_bg_window_checked(&mut self) -> Option<usize>;
    fn get_screen_list(&self) -> Vec<shared::monitor::Monitor>;
    fn prepare_for_monitor(&self, _: shared::monitor::Monitor);
    fn on_exit(&mut self);
}
