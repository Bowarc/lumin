#![cfg_attr(all(target_os = "windows"), windows_subsystem = "windows")]

use winapi::{
    shared::{minwindef, windef},
    um::winuser,
};

fn main() {
    let max_loops: usize = 1;

    for _i in 0..max_loops {
        if let Some(workerw) = get_workerw_id() {
            print!("{:?}", workerw as usize);
            std::process::exit(0) // success
        }
    }
    std::process::exit(1) // faill
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

    // debug!("Scanning for workerW");

    let progman_handle =
        unsafe { winuser::FindWindowA(str_ptr("progman"), std::ptr::null::<i8>()) };

    if progman_handle.is_null() {
        // debug!("Couldn't find progman");
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

        // warn!("Couldn't spawn WorkerW window, trying old method");

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
        // warn!("WorkerW could not be found");
        return None;
    }

    Some(worker)
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
        // debug!("Wallpaper is {pworker:?}\nIts parent is {wnd:?}");
        return minwindef::FALSE;
    }

    // useless
    // println!("[DEBUG] returning true in `extern \"system\" fn find_worker` ");
    minwindef::TRUE
}
