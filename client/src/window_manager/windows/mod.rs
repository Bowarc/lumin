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

#[derive(Default)]
pub enum WorkerWHandle {
    #[default]
    Void,
    FetcherRunning {
        fetcher_process: std::process::Child,
        explorer_pid: sysinfo::Pid,
    },
    Received {
        hwnd: usize,
        explorer_pid: sysinfo::Pid,
    },
}

#[derive(Default)]
pub struct Explorer {
    // pub pid: usize,
    // pub workerw: usize,
    pub workerw_handle: WorkerWHandle,
    pub default_workerw_position: (i32, i32),
    pub default_workerw_size: (i32, i32),
    pub system: sysinfo::System, // implement a way to handle the workerw fetcher
}

// -> Option<>
impl Explorer {
    // creates a new self, this is different than default because it runs an update
    pub fn new() -> Option<Self> {
        // todo!("Add a way to handle workerw fetcher");
        let mut o = Self::default();
        crate::window_manager::WindowManager::update(&mut o).ok()?;
        Some(o)
    }

    fn get_windows_explorer_process(&mut self) -> Option<&sysinfo::Process> {
        use sysinfo::SystemExt;
        // let mut system = ;
        self.system.refresh_all();

        let explorer_processes = self
            .system
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
            error!("Could not get explorer.exe {explorer_processes:#?}");

            // crate::EXIT_REQUESTED.store(true, std::sync::atomic::Ordering::Relaxed);
            error!("[CRITICAL] Requesting an exit");
            None
        } else {
            let process = explorer_processes.get(0).copied();

            process
        }
    }
}

impl crate::window_manager::WindowManager for Explorer {
    // don't care about what is the error, if this faills, exit the programm
    fn update(&mut self) -> Result<(), ()> {
        use sysinfo::{ProcessExt, SystemExt};
        let mut system = sysinfo::System::new();
        system.refresh_all();
        if let WorkerWHandle::Received {
            hwnd: _,
            explorer_pid,
        } = self.workerw_handle
        {
            if system
                .process(explorer_pid)
                .filter(|p| {
                    // Verify that it's the right `explorer.exe`
                    // (for the very small case that the user has a non-windows related app named 'explorer.exe')
                    validate_explorer_process(p)
                })
                .is_some()
            {
                // debug!("All good");
                return Ok(());
            } else {
                debug!("[WM] the saved pid is not the right one");
                self.workerw_handle = WorkerWHandle::Void;
            }
        }

        if let WorkerWHandle::FetcherRunning {
            fetcher_process,
            explorer_pid,
        } = &mut self.workerw_handle
        {
            // check if the fetcher is still running

            match fetcher_process.try_wait() {
                Ok(Some(status)) => {
                    println!("fetcher exited with: {status}");

                    if status.success() {
                        use std::io::BufRead;
                        let mut child_out =
                            std::io::BufReader::new(fetcher_process.stdout.as_mut().unwrap());
                        let mut line = String::new();

                        child_out.read_line(&mut line).unwrap();
                        println!("fetcher wrote '{}' before dying", line);

                        self.workerw_handle = WorkerWHandle::Received {
                            hwnd: line.parse().unwrap(),
                            explorer_pid: *explorer_pid,
                        }
                    } else {
                        // when the process fail, looking at the stdout (and reading it like the above is doing)
                        // does not crash the current program as it's like channels and the client is only reading what it receied,
                        // client is not atempting to call remote process
                        println!("fetcher failled");

                        // Reset Fetcher handle
                        self.workerw_handle = WorkerWHandle::Void
                    }
                }
                Ok(None) => {
                    println!("Fetcher is still running");
                    // let res = fetcher_process.wait();
                    // println!("result: {res:?}");
                }
                Err(e) => {
                    println!("Could not get exit code");
                    println!("error attempting to wait: {e}");
                }
            };
        }

        if let WorkerWHandle::Void = self.workerw_handle {
            // start it
            debug!("Starting the fetcher. . .");

            const FETCHER_EXE_PATH: &str = "workerw_fetcher.exe";
            let mut path = std::env::current_exe().unwrap();

            path.set_file_name(FETCHER_EXE_PATH);

            let child = std::process::Command::new(path)
                .stderr(std::process::Stdio::null())
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::piped())
                .spawn()
                .unwrap();

            let explorer_process = self.get_windows_explorer_process().unwrap();

            self.workerw_handle = WorkerWHandle::FetcherRunning {
                fetcher_process: child,
                explorer_pid: explorer_process.pid(),
            }
        }

        Ok(())
    }
    fn get_bg_window_checked(&mut self) -> Option<usize> {
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

        // This won't do anything if there is no need to
        self.update().ok()?;

        use sysinfo::ProcessExt;

        if let WorkerWHandle::Received { hwnd, explorer_pid } = self.workerw_handle {
            if let Some(explorer_process) = self.get_windows_explorer_process() {
                if explorer_pid == explorer_process.pid() {
                    Some(hwnd)
                } else {
                    self.workerw_handle = WorkerWHandle::Void;
                    error!("The saved pid is not the right one");
                    None
                }
            } else {
                error!("Could not get the explorer process");
                None
            }
        } else {
            warn!("The workerw fetcher is still running");
            None
        }
    }
    fn get_screen_list(&self) -> std::vec::Vec<crate::monitor::Monitor> {
        // This isn't like workerW, in a way that it should not fail (at least it never happend to me)
        // If this ever gets as random as workerW(which i fcking hate), just do it only one time and save the output
        utils::get_screens()
    }
    fn prepare_for_monitor(&self, monitor: crate::monitor::Monitor) -> Result<(), String> {
        if let WorkerWHandle::Received {
            hwnd,
            explorer_pid: _,
        } = self.workerw_handle
        {
            utils::move_window(hwnd as windef::HWND, monitor.position, monitor.size);
            Ok(())
        } else {
            // warn!("The workerw fetcher is still running");
            Err(String::from("Could not get WorkerW"))
        }
    }
    fn cleanup(&mut self) -> Result<(), String> {
        // debug!("Todo: Restore the default size and position of WorkerW
        // Well, it seems it doesn't need it.
        // Explainations:
        // While testing on my windows machine, after moving the workerW, using it, stopping,
        // They were graphical bugs that made the original background cutted and mixed with the background of other screens

        // So i was planing on restoring workerW's original size to counter this problem, butmy dbg tool
        // tells me that it auto re-shaped itself right after i delete the mpv process lmao
        //         ");

        // Un-comment this if the message above turn false

        if let WorkerWHandle::Received {
            hwnd,
            explorer_pid: _,
        } = self.workerw_handle
        {
            utils::move_window(
                hwnd as windef::HWND,
                self.default_workerw_position,
                self.default_workerw_size,
            );
            Ok(())
        } else {
            Err(String::from("Could not get WorkerW"))
        }
    }
}

pub fn validate_explorer_process(p: &sysinfo::Process) -> bool {
    // TODO check command line, check original file path etc..

    use sysinfo::ProcessExt;

    let mut ok = true;

    if p.name() != "explorer.exe" {
        // debug!("Explorer check for {p:?} failled on `p.name() != \"explorer.exe\"`");
        ok = false
    } else if p.exe().as_os_str().to_str().map(|p| p.to_lowercase())
        != Some("c:\\windows\\explorer.exe".to_string())
        && p.exe().as_os_str().to_str().map(|p| p.to_lowercase())
            != Some("c:/windows/explorer.exe".to_string())
    {
        debug!("Explorer check for {p:?} failled on `p.exe().as_os_str().to_str() != Some(\"C:\\Windows\\explorer.exe\")`: {:?}", p.exe().as_os_str().to_str());
        ok = false
    } else if p.cmd().len() != 1 {
        debug!(
            "Explorer check for {p:?} failled on `if p.cmd().len() !=1` ({})",
            p.cmd().len()
        );
        ok = false
    } else if p.cmd().get(0).unwrap().to_lowercase() != r"c:\windows\explorer.exe"
        && p.cmd().get(0).unwrap().to_lowercase() != r"c:/windows/explorer.exe"
    {
        debug!("Explorer check for {p:?} failled on `p.cmd().get(0).unwrap().to_lowercase() != \"c:\\windows\\explorer.exe\"` ({})", p.cmd().get(0).unwrap().to_lowercase());
        ok = false
    }

    ok
}
