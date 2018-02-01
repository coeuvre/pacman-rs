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
use std::sync::mpsc::channel;
use game;
use game::*;

macro_rules! wcstr {
    ($e:expr) => {{
        use std::ffi::OsStr;
        use std::iter::once;
        use std::os::windows::ffi::OsStrExt;
        OsStr::new($e).encode_wide().chain(once(0)).collect::<Vec<u16>>()
    }};
}

lazy_static! {
    static ref INSTANCE: Win32Instance = Win32Instance::new();
}

pub struct Win32GlContext {
    hdc: HDC,
    hglrc: HGLRC,
    opengl32_module: HMODULE,
}

pub struct CurrentWin32GlContext<'a> {
    gl_ctx: &'a mut Win32GlContext,
}

impl Win32GlContext {
    pub unsafe fn new(hdc: HDC) -> Win32GlContext {
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

        Win32GlContext {
            hdc,
            hglrc,
            opengl32_module,
        }
    }
}

impl Drop for Win32GlContext {
    fn drop(&mut self) {
        unsafe {
            wglDeleteContext(self.hglrc);

            FreeLibrary(self.opengl32_module);
        }
    }
}

unsafe impl Send for Win32GlContext {}

impl GlContext for Win32GlContext {
    fn make_current<'a>(&'a mut self) -> Result<Box<CurrentGlContext + 'a>, String> {
        unsafe {
            let success = wglMakeCurrent(self.hdc, self.hglrc);
            if success == 0 {
                Err(format!("Error {}", GetLastError()))
            } else {
                Ok(Box::new(CurrentWin32GlContext { gl_ctx: self }))
            }
        }
    }
}

impl<'a> CurrentGlContext<'a> for CurrentWin32GlContext<'a> {
    // See https://www.khronos.org/opengl/wiki/Load_OpenGL_Functions#Windows
    unsafe fn proc_address(&self, name: &str) -> Result<*const c_void, String> {
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
}

impl<'a> Drop for CurrentWin32GlContext<'a> {
    fn drop(&mut self) {
        unsafe {
            wglMakeCurrent(NULL as HDC, NULL as HGLRC);
        }
    }
}

struct WindowClass {
    name: Vec<u16>,
}

impl WindowClass {
    pub fn new(hinstance: HINSTANCE) -> WindowClass {
        let name = wcstr!("PACMANRSWINDOWCLASS");

        unsafe {
            let mut wc = ::std::mem::zeroed::<WNDCLASSEXW>();
            wc.cbSize = ::std::mem::size_of_val(&wc) as UINT;
            wc.style = 0;
            wc.lpfnWndProc = Some(window_proc);
            wc.hCursor = LoadCursorW(0 as HINSTANCE, IDC_ARROW);
            wc.hInstance = hinstance;
            wc.lpszClassName = name.as_ptr();
            wc.cbWndExtra = 0;

            RegisterClassExW(&wc);
        }

        WindowClass { name }
    }
}

impl Drop for WindowClass {
    fn drop(&mut self) {
        unsafe {
            UnregisterClassW(self.name.as_ptr(), INSTANCE.hinstance);
        }
    }
}

struct Win32Instance {
    hinstance: HINSTANCE,
    window_class: WindowClass,
}

impl Win32Instance {
    pub fn new() -> Win32Instance {
        let hinstance = unsafe { GetModuleHandleW(NULL as LPCWSTR) };
        Win32Instance {
            hinstance,
            window_class: WindowClass::new(hinstance),
        }
    }
}

unsafe impl Sync for Win32Instance {}

struct Win32Platform {}

impl Win32Platform {
    pub fn new() -> Result<Win32Platform, String> {
        Ok(Win32Platform {})
    }
}

impl Platform for Win32Platform {
    fn performance_counter() -> u64 {
        0
    }
    fn performance_fraquency() -> u64 {
        0
    }
}

impl GlDesktop for Win32Platform {
    type GlWindow = Win32GlWindow;

    fn create_window(&mut self) -> Result<Self::GlWindow, String> {
        let (sender, receiver) = channel();

        ::std::thread::spawn(move || {
            unsafe {
                let title = wcstr!("Pac-Mac");

                let hwnd = create_window(
                    INSTANCE.hinstance,
                    INSTANCE.window_class.name.as_ptr(),
                    title.as_ptr(),
                );

                let window = Win32GlWindow {
                    hwnd,
                    hdc: GetDC(hwnd),
                };
                sender.send(window).unwrap();

                let mut msg = ::std::mem::uninitialized();
                while GetMessageW(&mut msg, hwnd, 0, 0) != 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            };
        });

        Ok(receiver.recv().unwrap())
    }
}

struct Win32GlWindow {
    hwnd: HWND,
    hdc: HDC,
}

impl GlWindow for Win32GlWindow {
    type GlContext = Win32GlContext;

    fn create_gl_context(&mut self) -> Result<Self::GlContext, String> {
        unsafe { Ok(Win32GlContext::new(self.hdc)) }
    }

    fn swap_buffers(&mut self) -> Result<(), String> {
        unsafe {
            SwapBuffers(self.hdc);
        }
        Ok(())
    }
}

unsafe impl Send for Win32GlWindow {}

impl Drop for Win32GlWindow {
    fn drop(&mut self) {
        unsafe {
            DestroyWindow(self.hwnd);
        }
    }
}

pub fn start() {
    let mut win32 = Win32Platform::new().unwrap();
    unsafe {
        SetProcessDpiAwareness(PROCESS_SYSTEM_DPI_AWARE);
    }
    game::start_desktop(&mut win32);
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
        WM_CLOSE => {}
        WM_DESTROY => {
            PostQuitMessage(0);
        }
        _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
    }

    return 0;
}
