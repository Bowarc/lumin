pub mod windows;

pub trait WindowManager: Sync + Send {
    // fn get_bg_window(&self) -> Option<usize>;
    fn update(&mut self) -> Result<(), ()>;
    fn get_bg_window_checked(&mut self) -> Option<usize>;
    fn get_screen_list(&self) -> Vec<crate::monitor::Monitor>;
    fn prepare_for_monitor(&self, _: crate::monitor::Monitor) -> Result<(), String>;
    fn cleanup(&mut self) -> Result<(), String>;
}
