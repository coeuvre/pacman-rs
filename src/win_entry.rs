use std::ffi::CString;
use std::os::raw::*;
use winapi::um::shellscalingapi::*;
use winapi::shared::ntdef::*;
use winapi::shared::windef::*;
use winapi::shared::minwindef::*;
use winapi::um::winuser::*;
use winapi::um::wingdi::*;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::libloaderapi::*;
use game::{self, CurrentGlContext, GlContext};

macro_rules! wcstr {
    ($e:expr) => {{
        use std::ffi::OsStr;
        use std::iter::once;
        use std::os::windows::ffi::OsStrExt;
        OsStr::new($e).encode_wide().chain(once(0)).collect::<Vec<u16>>()
    }};
}

pub struct WinGlContext {
    hdc: HDC,
    hglrc: HGLRC,
    opengl32_module: HMODULE,
}

pub struct CurrentWinGlContext<'a> {
    gl_ctx: &'a mut WinGlContext,
}

impl WinGlContext {
    pub unsafe fn new(hdc: HDC) -> WinGlContext {
        let opengl32_module = LoadLibraryA("opengl32.dll\0".as_ptr() as LPCSTR);
        assert_ne!(opengl32_module, 0 as HMODULE, "Failed to load opengl32.dll");

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

        WinGlContext {
            hdc,
            hglrc,
            opengl32_module,
        }
    }
}

impl Drop for WinGlContext {
    fn drop(&mut self) {
        unsafe {
            wglDeleteContext(self.hglrc);

            FreeLibrary(self.opengl32_module);
        }
    }
}

unsafe impl Send for WinGlContext {}

impl<'a> GlContext<'a, CurrentWinGlContext<'a>> for WinGlContext {
    fn make_current(&'a mut self) -> Result<CurrentWinGlContext<'a>, String> {
        unsafe {
            let success = wglMakeCurrent(self.hdc, self.hglrc);
            if success == 0 {
                Err(format!("Error {}", GetLastError()))
            } else {
                Ok(CurrentWinGlContext { gl_ctx: self })
            }
        }
    }
}

impl<'a> CurrentGlContext<'a> for CurrentWinGlContext<'a> {
    // See https://www.khronos.org/opengl/wiki/Load_OpenGL_Functions#Windows
    unsafe fn get_proc_address(&self, name: &str) -> Result<*const c_void, String> {
        let cstring = CString::new(name).unwrap();
        let mut p = wglGetProcAddress(cstring.as_ptr()) as isize;
        match p {
            0 | 0x1 | 0x2 | 0x3 | -1 => {
                p = GetProcAddress(self.gl_ctx.opengl32_module, cstring.as_ptr()) as isize;
            }
            _ => {}
        }
        Ok(p as *const c_void)
    }

    fn swap_buffers(&mut self) -> Result<(), String> {
        unsafe {
            SwapBuffers(self.gl_ctx.hdc);
        }
        Ok(())
    }
}

impl<'a> Drop for CurrentWinGlContext<'a> {
    fn drop(&mut self) {
        unsafe {
            wglMakeCurrent(NULL as HDC, NULL as HGLRC);
        }
    }
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
        let mut gl_ctx = WinGlContext::new(hdc);

        ::std::thread::spawn(move || game::start(&mut gl_ctx));

        let mut msg = ::std::mem::uninitialized();
        while GetMessageW(&mut msg, NULL as HWND, 0, 0) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
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
