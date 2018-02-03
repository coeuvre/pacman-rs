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
use std::sync::mpsc::{channel, Receiver, Sender};
use winapi::um::profileapi::QueryPerformanceCounter;
use game;
use game::*;
use lazy_static;

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
    hwnd: HWND,
    hdc: HDC,
    hglrc: HGLRC,
    opengl32_module: HMODULE,
}

pub struct CurrentWin32GlContext<'a> {
    gl_ctx: &'a mut Win32GlContext,
}

impl Win32GlContext {
    pub unsafe fn new(hwnd: HWND, hdc: HDC) -> Win32GlContext {
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
            hwnd,
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

    fn swap_buffers(&mut self) -> Result<(), String> {
        run_on_window_thread(self.gl_ctx.hwnd, || {
            unsafe {
                SwapBuffers(self.gl_ctx.hdc);
            }
        });
        Ok(())
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
            wc.cbWndExtra = ::std::mem::size_of::<usize>() as i32;

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
        unsafe {
            let mut counter = ::std::mem::uninitialized();
            QueryPerformanceCounter(&mut counter);
            *counter.QuadPart() as u64
        }
    }

    fn performance_fraquency() -> u64 {
        0
    }
}

impl Desktop for Win32Platform {
    type Window = Win32Window;

    fn create_window(&mut self) -> Result<Self::Window, String> {
        let title = wcstr!("Pac-Mac");
        let (create_window_result_sender, create_wnidow_result_receiver) = channel();

        unsafe {
            ::std::thread::spawn(move || {
                let style = WS_OVERLAPPEDWINDOW ^ WS_THICKFRAME ^ WS_MAXIMIZEBOX;
                let ex_style = 0;

                let (window_event_sender, window_event_receiver) = channel();
                let state = Box::into_raw(Box::new(WindowState { sender: window_event_sender }));

                let hwnd = CreateWindowExW(
                    ex_style,
                    INSTANCE.window_class.name.as_ptr(),
                    title.as_ptr(),
                    style,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    1200,
                    740,
                    0 as HWND,
                    0 as HMENU,
                    INSTANCE.hinstance,
                    0 as LPVOID,
                );

                SetWindowLongPtr(hwnd, 0, state as isize);

                create_window_result_sender.send(Win32Window {
                    hwnd,
                    hdc: GetDC(hwnd),
                    receiver: window_event_receiver,
                }).unwrap();

                ShowWindow(hwnd, SW_SHOW);

                let mut msg = ::std::mem::uninitialized();
                while GetMessageW(&mut msg, NULL as HWND, 0, 0) != 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            });

            Ok(create_wnidow_result_receiver.recv().unwrap())
        }
    }
}

struct Win32Window {
    hwnd: HWND,
    hdc: HDC,
    receiver: Receiver<WindowEvent>,
}

struct WindowState {
    sender: Sender<WindowEvent>,
}

impl Win32Window {
    pub fn handle_event(&mut self, _event: &WindowEvent) {

    }
}

pub fn run_on_window_thread<F, T>(hwnd: HWND, f: F) -> T where F: Fn() -> T {
    let window_thread_fn = || {
        let t = f();
        Box::into_raw(Box::new(t)) as *mut c_void
    };
    let fn_trait_obj: &Fn() -> *mut c_void = &window_thread_fn;
    let ptr: & &Fn() -> *mut c_void = &fn_trait_obj;

    unsafe {
        let t = SendMessageW(
            hwnd,
            WM_USER_RUN,
            0,
            ptr as *const &Fn() -> *mut c_void as *const c_void as LPARAM,
        );
        *Box::from_raw(t as *mut T)
    }
}

impl Window for Win32Window {
    type GlContext = Win32GlContext;

    fn create_gl_context(&mut self) -> Result<Self::GlContext, String> {
        unsafe { Ok(Win32GlContext::new(self.hwnd, self.hdc)) }
    }

    fn poll_events<'a>(&'a mut self) -> Box<'a + Iterator<Item=WindowEvent>> {
        Box::new(PollEventIter { window: self })
    }
}

unsafe impl Send for Win32Window {}

impl Drop for Win32Window {
    fn drop(&mut self) {
        run_on_window_thread(self.hwnd, || {
            unsafe {
                DestroyWindow(self.hwnd);
            }
        });
    }
}

pub struct PollEventIter<'a> {
    window: &'a mut Win32Window,
}

impl<'a> Iterator for PollEventIter<'a> {
    type Item = WindowEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let event = self.window.receiver.try_recv().ok();
        if let Some(ref event) = event {
            self.window.handle_event(event);
        }
        event
    }
}

const WM_USER_RUN: UINT = WM_USER;

pub fn start() {
    lazy_static::initialize(&INSTANCE);

    let mut win32 = Win32Platform::new().unwrap();

    unsafe {
        SetProcessDpiAwareness(PROCESS_SYSTEM_DPI_AWARE);
    }

    game::start_desktop(&mut win32);
}

#[allow(non_snake_case)]
#[cfg(target_arch = "x86_64")]
unsafe fn SetWindowLongPtr(hwnd: HWND, index: i32, data: isize) {
    SetWindowLongPtrW(hwnd, index, data);
}

#[allow(non_snake_case)]
#[cfg(target_arch = "x86")]
unsafe fn SetWindowLongPtr(hwnd: HWND, index: i32, data: isize) {
    SetWindowLongW(hwnd, index, data);
}

#[allow(non_snake_case)]
#[cfg(target_arch = "x86_64")]
unsafe fn GetWindowLongPtr(hwnd: HWND, index: i32) -> isize {
    GetWindowLongPtrW(hwnd, index)
}

#[allow(non_snake_case)]
#[cfg(target_arch = "x86")]
unsafe fn GetWindowLongPtr(hwnd: HWND, index: i32) -> isize {
    GetWindowLongW(hwnd, index)
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let state = &mut *(GetWindowLongPtr(hwnd, 0) as *mut WindowState);

    match msg {
        WM_CLOSE => {
            state.sender.send(WindowEvent::CloseRequested).unwrap();
            0
        }
        WM_DESTROY => {
            Box::from_raw(state);
            PostQuitMessage(0);
            0
        }
        WM_USER_RUN => {
            let f = &*(lparam as *const &Fn() -> *mut c_void);
            f() as LRESULT
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
