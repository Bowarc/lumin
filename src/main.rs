use winapi::{
    shared::{minwindef, windef},
    um::winuser,
};

const TRUE: minwindef::BOOL = 1;
const FALSE: minwindef::BOOL = 0;

fn main() {
    tests()
}

fn str_ptr(base: &'static str) -> *const i8 {
    format!("{base}\0").as_ptr() as *const i8
}

fn tests() {
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

    println!("Scanning for progman");
    let progman_name = "progman";

    let progman_handle = unsafe { winuser::FindWindowA(str_ptr(progman_name), 0 as *const i8) };

    if progman_handle == std::ptr::null_mut::<windef::HWND__>() {
        println!("Couldn't find progman");
        return;
    }

    println!("Found progman at address: {progman_handle:?}");

    // this is basically all the magic. it's an undocumented window message that
    // forces windows to spawn a window with class "WorkerW" behind deskicons

    println!("Spawning wallpaper");

    // after some testing, this might be useless
    unsafe {
        winuser::SendMessageA(progman_handle, 0x052C, 0xD, 0);
        winuser::SendMessageA(progman_handle, 0x052C, 0xD, 1);
    }

    println!("Checking for wallpaper..");

    let mut worker = std::ptr::null_mut::<windef::HWND__>();

    unsafe {
        winuser::EnumWindows(
            Some(find_worker),
            &mut worker as *mut _ as minwindef::LPARAM,
        )
    };

    // this is a bit fcked as we can't pass a mutable ref of this variable to the `EnumWindows` function, so it's basicly useless in Rust
    println!("Worker: {worker:?}");

    // let worker_parent = 0x10262 as windef::HWND;
    // fix_workerw_position(worker_parent);
    enumerate_monitors()
}

fn fix_workerw_position(worker: windef::HWND) {
    ///////////////////////////////////////////////////////////////////////////////////////////////
    // fix a bug (at least on my machine) where the window(worker) is not centered on the screen //
    ///////////////////////////////////////////////////////////////////////////////////////////////
    // But then it created a new one where if the worker is trying to do it's normal job
    // (wallpaper tansition animation) it's graphicly fucked
    // (you don't have a single background per monitor anyomore,
    // you have some cropped superpositions of the backgrounds of the others screens on the main one)

    // This holds the default size of the WorkerW (xywh should be something like -1280, -2, 4566, 1085)
    let mut rect = windef::RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    if unsafe { winuser::GetWindowRect(worker, &mut rect) } != FALSE {
        let x = rect.left;
        let y = rect.top;
        let w = rect.right - rect.left;
        let h = rect.bottom - rect.top;

        println!("x{x}, y{y}, w{w}, h{h}");
    } else {
        eprintln!("Failled to get the rect of {worker:?}");
    }

    // ?
    unsafe { winuser::MoveWindow(worker, 0, 0, 0, 0, TRUE) };

    // This fixes a bug where the WorkerW is not centered on the window, therefore the video wallpaper is bad
    // unsafe { winuser::MoveWindow(worker, 0, 0, 1920, 1080, TRUE) };

    // pure testing
    unsafe { winuser::MoveWindow(worker, -1280, 0, 1024, 819, TRUE) };

    // Reset to default size, this allows the worker to do it's normal job.
    // unsafe {
    //     winuser::MoveWindow(
    //         worker,
    //         rect.left,
    //         rect.top,
    //         rect.right - rect.left,
    //         rect.bottom - rect.top,
    //         TRUE,
    //     )
    // };

    ///////////////////////////////////////////////////////////////////////////////////////////////
    //                                            End                                            //
    ///////////////////////////////////////////////////////////////////////////////////////////////
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
    } == std::ptr::null_mut()
    {
        return TRUE;
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

    if *pworker != std::ptr::null_mut() {
        println!("Wallpaper is {pworker:?}\nIts parent is {wnd:?}");
        // dbg!(pworker, wnd);
        return FALSE;
    }

    // useless
    println!("[DEBUG] returning true in `extern \"system\" fn find_worker` ");
    return TRUE;
}

fn enumerate_monitors() {
    unsafe {
        winuser::EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            Some(process_monitor),
            0 as minwindef::LPARAM,
        );
    }
}

unsafe extern "system" fn process_monitor(
    hmonitor: windef::HMONITOR,
    _hdc: windef::HDC,
    _rect: *mut windef::RECT,
    _lparam: minwindef::LPARAM,
) -> minwindef::BOOL {
    let mut info: winuser::MONITORINFO = std::mem::zeroed();
    info.cbSize = std::mem::size_of::<winuser::MONITORINFO>() as u32;
    winuser::GetMonitorInfoW(hmonitor, &mut info);

    println!(
        "Monitor position: ({}, {}), size: {} x {}",
        info.rcMonitor.left,
        info.rcMonitor.top,
        info.rcMonitor.right - info.rcMonitor.left,
        info.rcMonitor.bottom - info.rcMonitor.top
    );

    TRUE
}
