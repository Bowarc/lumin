use winapi::shared::windef;

pub mod utils;

/*----------------NOTE------------------
 In Windows, any instance of the file
 explorer is not a process, just a
 window in the process `explorer.exe`.
 This means that there can be only one
 process named `explorer.exe`.
 (if you don't run any app named 'explorer.exe')
--------------------------------------*/

pub struct Explorer {
    pub pid: usize,
    pub workerw: windef::HWND,
}

// -> Option<>
impl Explorer {
    // creates a new self, this is different than default because it runs an update
    pub fn new() -> Self {
        let mut o = Self::default();
        o.update();
        o
    }

    pub fn update(&mut self) {
        use sysinfo::{ProcessExt, SystemExt};
        let mut system = sysinfo::System::new();
        system.refresh_all();

        // if !system.process(self.pid).is {}

        if system
            .process(self.pid.into())
            .filter(|p| {
                // Verify that it's the right `explorer.exe`
                // (for the very small case that the user has a non-windows related app named 'explorer.exe')

                validate_explorer_process(p)
            })
            .is_some()
        {
            // All good
            return;
        } else {
            debug!("[WM] the saved pid is not the right one")
        }

        while system
            .processes()
            .iter()
            .filter_map(|(_pid, process)| {
                if validate_explorer_process(process) {
                    Some(process)
                } else {
                    None
                }
            })
            .collect::<Vec<&sysinfo::Process>>()
            .is_empty()
        {
            // Explorer is not running
            // Probably with std::process::Command(complete_path_of_windows_explorer)
            // todo!("Run explorer");

            let child =
                std::process::Command::new(std::path::Path::new("C:\\Windows\\explorer.exe"))
                    .stderr(std::process::Stdio::null())
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .spawn();

            debug!("New explorer child: {child:?}");

            // hehe
            drop(child);

            let arbitrary_startup_time_ms = 1000;
            std::thread::sleep(std::time::Duration::from_millis(arbitrary_startup_time_ms));
            system.refresh_processes()
        }

        // A new explorer.exe has been started, let's save it's pid

        // If this crashes, i don't understand
        let explorer_processes = system
            .processes()
            .iter()
            .filter_map(|(_pid, process)| {
                // Verify that it's the right `explorer.exe`
                if validate_explorer_process(process) {
                    Some(process)
                } else {
                    None
                }
            })
            .collect::<Vec<&sysinfo::Process>>();

        if explorer_processes.is_empty() || explorer_processes.len() > 1 {
            // If this EVER executes, nuke my house

            // This should only executes if the user kills `explorer.exe` after the daemon started it
            error!("If this EVER executes, nuke my house");
            return;
        }
        let explorer_process = explorer_processes.get(0).unwrap();

        self.pid = usize::from(explorer_process.pid());

        // let's get it's workerw

        let workerw_opt = utils::get_workerw_id_loop(1);

        if let Some(workerw) = workerw_opt {
            self.workerw = workerw
        } else {
            error!("Could not get workerW for some reason");
            panic!("Can't get workerw")
        }
    }
}

impl crate::window_manager::WindowManager for Explorer {
    fn get_bg_window(&self) -> Option<usize> {
        // {
        //     use winapi::shared::windef;
        //     let w = window_manager::windows::utils::get_workerw_id().unwrap();
        //     debug!("{w:?}");
        //     debug!("{}", w as i32);
        //     debug!("{}", w as usize);
        //     debug!("{}", w as i32 as usize);
        //     let a = w as i32;
        //     drop(w);
        //     let w = a as windef::HWND;
        //     debug!("");
        //     debug!("{:?}", a);
        //     debug!("{}", a as i32);
        //     debug!("{a:x}")
        // }
        // the test above tells us that we can confidently convert windef::HWND to usize/i32 back and forth

        Some(self.workerw as usize)
    }
    fn get_bg_window_checked(&mut self) -> Option<usize> {
        use sysinfo::{ProcessExt, SystemExt};
        let mut system = sysinfo::System::new();
        system.refresh_all();

        // This works if we're 100% sure that the user won't have a non-windows related app
        // named 'explorer.exe' running on their machine
        // if system
        //     .processes_by_name("explorer.exe")
        //     .collect::<Vec<&sysinfo::Process>>()
        //     .iter()
        //     .map(|p| p.pid())
        //     .collect::<Vec<sysinfo::Pid>>()
        //     // ^This creates a vec of 0 or 1 element (Check note) that is the pid of `explorer.exe`
        //     .get(0)
        //     == Some(&self.pid.into())
        // {
        //     return Some(self.workerw as usize);
        // }

        if system
            .processes_by_name("explorer.exe")
            .collect::<Vec<&sysinfo::Process>>()
            .iter()
            .map(|p| p.pid())
            .collect::<Vec<sysinfo::Pid>>()
            // ^This creates a vec of 0 or 1 element (Check note) that is the pid of `explorer.exe`
            // But there is a world where the user have a non-windows related app named 'explorer.exe'
            // and therefore can be retrieved by this method, which would fuck up the check.
            // So we can't match eq on `.get(0)` and have to use the `.contains` method.
            .contains(&self.pid.into())
        {
            debug!("no need to update");
            return Some(self.workerw as usize);
        }

        self.update();

        Some(self.workerw as usize)
    }
    fn get_screen_list(&self) -> std::vec::Vec<shared::monitor::Monitor> {
        // This isn't like workerW, in a way that it should not fail (at least it never happend to me)
        // If this ever gets as random as workerW(which i fcking hate), just do it only one time and save the output
        utils::get_screens()
    }
    fn prepare_for_monitor(&self, monitor: shared::monitor::Monitor) {
        utils::move_window(self.workerw, monitor.position, monitor.size);
    }
}

impl Default for Explorer {
    fn default() -> Self {
        Self {
            pid: 0,
            workerw: std::ptr::null_mut(),
        }
    }
}

pub fn validate_explorer_process(p: &sysinfo::Process) -> bool {
    // TODO check command line, check original file path etc..

    use sysinfo::ProcessExt;

    p.name() == "explorer.exe" && p.exe() == std::path::Path::new("C:\\Windows\\explorer.exe")
}
