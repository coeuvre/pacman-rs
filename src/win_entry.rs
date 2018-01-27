use std::ffi::{CStr, CString};
use std::os::raw::*;
use gl;
use winapi::um::shellscalingapi::*;
use winapi::shared::ntdef::*;
use winapi::shared::windef::*;
use winapi::shared::minwindef::*;
use winapi::um::winuser::*;
use winapi::um::wingdi::*;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::libloaderapi::*;

macro_rules! wcstr {
    ($e:expr) => {{
        use std::ffi::OsStr;
        use std::iter::once;
        use std::os::windows::ffi::OsStrExt;
        OsStr::new($e).encode_wide().chain(once(0)).collect::<Vec<u16>>()
    }};
}

pub fn start() {
    let class_name = wcstr!("PACMANRSWINDOWCLASS");
    let title = wcstr!("Pac-Mac");

    unsafe {
        let hinstance = GetModuleHandleW(NULL as LPCWSTR);

        SetProcessDpiAwareness(PROCESS_SYSTEM_DPI_AWARE);

        register_class(hinstance, class_name.as_ptr());
        let hwnd = create_window(hinstance, class_name.as_ptr(), title.as_ptr());

        let hdc = GetDC(hwnd);
        let mut pfd = ::std::mem::zeroed::<PIXELFORMATDESCRIPTOR>();
        pfd.nSize = ::std::mem::size_of_val(&pfd) as WORD;
        pfd.nVersion = 1;
        pfd.dwFlags = PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER;
        pfd.iPixelType = PFD_TYPE_RGBA;
        pfd.cColorBits = 32;
        pfd.cDepthBits = 24;
        pfd.cStencilBits = 8;
        pfd.iLayerType = PFD_MAIN_PLANE;
        let pfi = ChoosePixelFormat(hdc, &mut pfd);
        SetPixelFormat(hdc, pfi, &mut pfd);

        let hglrc = wglCreateContext(hdc);

        let success = wglMakeCurrent(hdc, hglrc);
        if success == 0 {
            println!("Error {}", GetLastError());
            return;
        }

        gl::load_with(|s| get_gl_proc_address(s));

        let glversion = CStr::from_ptr(gl::GetString(gl::VERSION) as *const ::std::os::raw::c_char);
        println!("{}", glversion.to_str().unwrap());

        let mut msg = ::std::mem::uninitialized();
        while GetMessageW(&mut msg, NULL as HWND, 0, 0) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        wglMakeCurrent(NULL as HDC, NULL as HGLRC);
    }
}

// See https://www.khronos.org/opengl/wiki/Load_OpenGL_Functions#Windows
unsafe fn get_gl_proc_address(name: &str) -> *const c_void {
    let cstring = CString::new(name).unwrap();
    let mut p = wglGetProcAddress(cstring.as_ptr()) as isize;
    match p {
        0 | 0x1 | 0x2 | 0x3 | -1 => {
            let module = LoadLibraryA("opengl32.dll\0".as_ptr() as LPCSTR);
            assert!(module != 0 as HMODULE, "Failed to load opengl32.dll");
            p = GetProcAddress(module, cstring.as_ptr()) as isize;
        }
        _ => {}
    }
    p as *const c_void
}

unsafe fn register_class(hinstance: HINSTANCE, class_name: LPCWSTR) {
    let mut wc = ::std::mem::zeroed::<WNDCLASSEXW>();
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
