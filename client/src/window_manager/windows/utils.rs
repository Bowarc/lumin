use std::{ffi::OsString, mem, os::windows::ffi::OsStringExt};

use winapi::{
    shared::{minwindef, windef},
    um::winuser,
};

pub fn get_screens() -> Vec<crate::monitor::Monitor> {
    let mut output = Vec::new();

    for monitor in enumerate_monitors() {
        let name = match &monitor.szDevice[..].iter().position(|c| *c == 0) {
            Some(len) => OsString::from_wide(&monitor.szDevice[0..*len]),
            None => OsString::from_wide(&monitor.szDevice[0..monitor.szDevice.len()]),
        };

        output.push(crate::monitor::Monitor::from_info(
            name.to_str().unwrap_or("????").to_string(),
            monitor,
        ));
    }
    output
}
pub fn move_window(window_id: *mut windef::HWND__, position: (i32, i32), size: (i32, i32)) {
    unsafe {
        winuser::MoveWindow(
            window_id,
            position.0,
            position.1,
            size.0,
            size.1,
            minwindef::TRUE,
        )
    };
}

pub fn get_window_pos_size(window_id: *mut windef::HWND__) -> ((i32, i32), (i32, i32)) {
    let mut rect = windef::RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    unsafe { winuser::GetWindowRect(window_id, &mut rect) };

    let position = (rect.left, rect.top);
    let size = (rect.right - rect.left, rect.bottom - rect.top);
    (position, size)
}

fn enumerate_monitors() -> Vec<winuser::MONITORINFOEXW> {
    // I feel like putting the function here is cleaner than outside

    // Define the vector where we will store the result
    let mut monitors = Vec::<winuser::MONITORINFOEXW>::new();
    let userdata = &mut monitors as *mut _;

    let result = unsafe {
        winuser::EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null(),
            Some(enumerate_monitors_callback),
            userdata as minwindef::LPARAM,
        )
    };

    if result != minwindef::TRUE {
        // Get the last error for the current thread.
        // This is analogous to calling the Win32 API GetLastio::Error.
        panic!(
            "Could not enumerate monitors: {}",
            std::io::Error::last_os_error()
        );
    }

    monitors
}

unsafe extern "system" fn enumerate_monitors_callback(
    monitor: windef::HMONITOR,
    _: windef::HDC,
    _: windef::LPRECT,
    userdata: minwindef::LPARAM,
) -> minwindef::BOOL {
    // Get the userdata where we will store the result
    let monitors: &mut Vec<winuser::MONITORINFOEXW> = mem::transmute(userdata);

    // Initialize the MONITORINFOEXW structure and get a pointer to it
    let mut monitor_info: winuser::MONITORINFOEXW = mem::zeroed();
    monitor_info.cbSize = mem::size_of::<winuser::MONITORINFOEXW>() as u32;
    let monitor_info_ptr = <*mut _>::cast(&mut monitor_info);

    // Call the GetMonitorInfoW win32 API
    let result = winuser::GetMonitorInfoW(monitor, monitor_info_ptr);
    if result == minwindef::TRUE {
        // Push the information we received to userdata
        monitors.push(monitor_info);
    }

    minwindef::TRUE
}
