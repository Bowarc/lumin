use std::{ffi::OsString, mem, os::windows::ffi::OsStringExt};

use winapi::{
    shared::{minwindef, windef},
    um::winuser,
};

pub fn get_screens() -> Vec<shared::monitor::Monitor> {
    let mut output = Vec::new();

    for monitor in enumerate_monitors() {
        let name = match &monitor.szDevice[..].iter().position(|c| *c == 0) {
            Some(len) => OsString::from_wide(&monitor.szDevice[0..*len]),
            None => OsString::from_wide(&monitor.szDevice[0..monitor.szDevice.len()]),
        };

        output.push(shared::monitor::Monitor::from_info(
            name.to_str().unwrap_or("????").to_string(),
            monitor,
        ));
    }
    output
}

pub fn get_workerw_id_loop(max_loops: usize) -> Option<windef::HWND> {
    let mut x = 0;
    loop {
        debug!("Loop {x}");
        if let Some(w) = get_workerw_id() {
            return Some(w);
        }
        // std::thread::sleep(std::time::Duration::from_secs_f32(0.5));
        x += 1;
        if x == max_loops {
            break;
        }
    }
    error!("WorkerW could not be found, tried {max_loops} times");

    None
}

pub fn get_workerw_id() -> Option<windef::HWND> {
    // heavily inspired by https://github.com/Francesco149/weebp

    // usefull links

    // https://docs.rs/winapi/latest/winapi/um/winuser/fn.FindWindowA.html
    // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-findwindowa

    // https://docs.rs/winapi/latest/winapi/um/winuser/fn.SendMessageA.html
    // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendmessagea

    // https://docs.rs/winapi/latest/winapi/um/winuser/fn.EnumWindows.html
    // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumwindows

    // https://docs.rs/winapi/latest/winapi/um/winuser/fn.FindWindowExA.html
    // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-findwindowexa

    // https://mpv.io/manual/stable/

    debug!("Scanning for workerW");

    let progman_handle =
        unsafe { winuser::FindWindowA(str_ptr("progman"), std::ptr::null::<i8>()) };

    if progman_handle.is_null() {
        debug!("Couldn't find progman");
        return None;
    }

    // this is basically all the magic. it's an undocumented window message that
    // forces windows to spawn a window with class "WorkerW" behind deskicons

    // after some testing, this might be useless
    unsafe {
        winuser::SendMessageA(progman_handle, 0x052C, 0xD, 0);
        winuser::SendMessageA(progman_handle, 0x052C, 0xD, 1);
    }

    // Eliminate the possiblility of a race condition with explorer.exe potentially caused by SendMessageA
    std::thread::sleep(std::time::Duration::from_secs_f32(0.25));

    let mut worker = std::ptr::null_mut::<windef::HWND__>();

    unsafe {
        winuser::EnumWindows(
            Some(find_worker),
            &mut worker as *mut _ as minwindef::LPARAM,
        )
    };

    if worker.is_null() {
        // warn!("WorkerW could not be found, tried {MAX_RESETS} times");
        // warn!("WorkerW could not be found");
        // return None;

        warn!("Couldn't spawn WorkerW window, trying old method");

        if !worker.is_null() {
            unsafe {
                //
                winuser::SendMessageA(progman_handle, 0x052C, 0, 0);

                // log1("checking for wallpaper");

                winuser::EnumWindows(
                    Some(find_worker),
                    &mut worker as *mut _ as minwindef::LPARAM,
                );
            }
        }
    }

    if worker.is_null() {
        warn!("WorkerW could not be found");
        return None;
    }

    Some(worker)
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

fn str_ptr(base: &'static str) -> *const i8 {
    format!("{base}\0").as_ptr() as *const i8
}

extern "system" fn find_worker(wnd: *mut windef::HWND__, lp: minwindef::LPARAM) -> minwindef::BOOL {
    // I don't understand everything so i'll leave some links that i found to get to that code
    // https://github.com/retep998/winapi-rs/issues/746
    // this one is less usefull v
    // https://stackoverflow.com/questions/38995701/how-do-i-pass-a-closure-through-raw-pointers-as-an-argument-to-a-c-function

    // Keep in mind that `windef::HWND == *mut windef::HWND__` is true

    let pworker = unsafe { &mut *(lp as *mut windef::HWND) };

    if unsafe {
        winuser::FindWindowExA(
            wnd,
            std::ptr::null_mut(),
            str_ptr("SHELLDLL_DefView"),
            std::ptr::null(),
        )
    }
    .is_null()
    {
        return minwindef::TRUE;
    }

    // #[rustfmt::skip] // OMFG STOP MAKING IT ONE LINE
    unsafe {
        *pworker = winuser::FindWindowExA(
            std::ptr::null_mut(),
            wnd,
            str_ptr("WorkerW"),
            std::ptr::null(),
        )
    };

    if !(*pworker).is_null() {
        debug!("Wallpaper is {pworker:?}\nIts parent is {wnd:?}");
        return minwindef::FALSE;
    }

    // useless
    println!("[DEBUG] returning true in `extern \"system\" fn find_worker` ");
    minwindef::TRUE
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
