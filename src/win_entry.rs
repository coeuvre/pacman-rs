extern crate winapi;

use self::winapi::um::shellscalingapi::*;
use self::winapi::shared::ntdef::*;
use self::winapi::shared::windef::*;
use self::winapi::shared::minwindef::*;
use self::winapi::um::winuser::*;
use self::winapi::um::libloaderapi::GetModuleHandleW;

macro_rules! wstr {
    ($e:expr) => {{
        use std::ffi::OsStr;
        use std::iter::once;
        use std::os::windows::ffi::OsStrExt;
        OsStr::new($e).encode_wide().chain(once(0)).collect::<Vec<u16>>()
    }};
}

pub fn start() {
    unsafe { SetProcessDpiAwareness(PROCESS_SYSTEM_DPI_AWARE) };
    unsafe {
        MessageBoxW(
            ::std::ptr::null_mut(),
            wstr!("Hello Pac-Mac").as_ptr(),
            wstr!("Pac-Man").as_ptr(),
            MB_OK,
        )
    };
    let hinstance = unsafe { GetModuleHandleW(0 as LPCWSTR) };
    let class_name = wstr!("PACMANRSWINDOWCLASS");
    let title = wstr!("Pac-Mac");

    unsafe { register_class(hinstance, class_name.as_ptr()) };
    unsafe { create_window(hinstance, class_name.as_ptr(), title.as_ptr()) };

    let mut msg = unsafe { ::std::mem::uninitialized() };
    while unsafe { GetMessageW(&mut msg, 0 as HWND, 0, 0) } != 0 {
        match msg.message {
            _ => unsafe {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            },
        }
    }
}

unsafe fn register_class(hinstance: HINSTANCE, class_name: LPCWSTR) {
    let mut wc: WNDCLASSEXW = ::std::mem::zeroed();
    wc.cbSize = ::std::mem::size_of_val(&wc) as UINT;
    wc.style = 0;
    wc.lpfnWndProc = Some(window_proc);
    wc.hCursor = LoadCursorW(0 as HINSTANCE, IDC_ARROW);
    wc.hInstance = hinstance;
    wc.lpszClassName = class_name;
    wc.cbWndExtra = 0;

    RegisterClassExW(&wc);
}

unsafe fn create_window(hinstance: HINSTANCE, class_name: LPCWSTR, title: LPCWSTR) -> HWND {
    let style = WS_OVERLAPPEDWINDOW ^ WS_THICKFRAME ^ WS_MAXIMIZEBOX;
    let ex_style = 0;

    let hwnd = CreateWindowExW(
        ex_style,
        class_name,
        title,
        style,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        1200,
        740,
        0 as HWND,
        0 as HMENU,
        hinstance,
        0 as LPVOID,
    );

    ShowWindow(hwnd, SW_SHOW);

    hwnd
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CLOSE => {
            DestroyWindow(hwnd);
            PostQuitMessage(0);
        }
        _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
    }

    return 0;
}
