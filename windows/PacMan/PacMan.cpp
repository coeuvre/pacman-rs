// PacMan.cpp : Defines the entry point for the application.
//

#include "stdafx.h"

#define MAX_LOADSTRING 100

// Global Variables:
HINSTANCE hInst;                                // current instance
WCHAR szTitle[MAX_LOADSTRING];                  // The title bar text
WCHAR szWindowClass[MAX_LOADSTRING];            // the main window class name

HWND WINDOW_HANDLE;
HDC DEVICE_CONTEXT;
HGLRC OPENGL_CONTEXT;
DWORD GAME_THREAD_ID;
BOOL QUIT = FALSE;

#define WM_USER_QUIT (WM_USER + 1)

// Forward declarations of functions included in this code module:
ATOM MyRegisterClass(HINSTANCE hInstance);
BOOL InitInstance(HINSTANCE);
LRESULT CALLBACK WndProc(HWND, UINT, WPARAM, LPARAM);

static void Quit() {
    QUIT = TRUE;
}

// See https://www.khronos.org/opengl/wiki/Load_OpenGL_Functions#Windows
void *GetGlProcAddress(const char *name) {
    void *p = (void *)wglGetProcAddress(name);
    if (p == 0 ||
        (p == (void*)0x1) || (p == (void*)0x2) || (p == (void*)0x3) ||
        (p == (void*)-1)) {
        HMODULE module = LoadLibraryA("opengl32.dll");
        p = (void *)GetProcAddress(module, name);
    }

    return p;
}

uint64_t GetPerformanceCounter() {
    LARGE_INTEGER counter;
    QueryPerformanceCounter(&counter);
    return counter.QuadPart;
}

uint64_t GetPerformanceFrequency() {
    LARGE_INTEGER frequency;
    QueryPerformanceFrequency(&frequency);
    return frequency.QuadPart;
}

int PollEvent(PlatformEvent *event) {
    MSG msg;
    if (!PeekMessage(&msg, NULL, 0, 0, PM_REMOVE)) {
        return 0;
    }

    switch (msg.message) {
        case WM_CLOSE: {
            event->kind = PLATFORM_EVENT_CLOSE;
        } break;

        default: return 0;
    }

    return 1;
}

DWORD WINAPI GameThreadMain(LPVOID param) {
    // force the system to create the message queue.
    MSG msg;
    PeekMessage(&msg, NULL, WM_USER, WM_USER, PM_NOREMOVE);

    wglMakeCurrent(DEVICE_CONTEXT, OPENGL_CONTEXT);

    // Disable VSYNC
    //void (*wglSwapIntervalEXT)(int) = (void (*)(int))wglGetProcAddress("wglSwapIntervalEXT");
    //wglSwapIntervalEXT(0);

    Platform platform;
    platform.quit = Quit;
    platform.get_gl_proc_address = &GetGlProcAddress;
    platform.get_performance_counter = &GetPerformanceCounter;
    platform.get_performance_frequency = &GetPerformanceFrequency;

    game_load(&platform);

    while (!QUIT) {
        MSG msg;
        if (PeekMessage(&msg, NULL, 0, 0, PM_REMOVE)) {
            PlatformEvent event;
            switch (msg.message) {
                case WM_CLOSE: {
                    event.kind = PLATFORM_EVENT_CLOSE;
                    game_on_platform_event(&event);
                } break;

                default:;
            }
        } else {
            game_render();

            SwapBuffers(DEVICE_CONTEXT);
        }
    }

    wglMakeCurrent(NULL, NULL);

    SendMessage(WINDOW_HANDLE, WM_USER_QUIT, 0, 0);

    return 0;
}

//int APIENTRY wWinMain(_In_ HINSTANCE hInstance, _In_opt_ HINSTANCE hPrevInstance, _In_ LPWSTR lpCmdLine, _In_ int nCmdShow) {
int main() {
//    UNREFERENCED_PARAMETER(hPrevInstance);
//    UNREFERENCED_PARAMETER(lpCmdLine);

    HINSTANCE hInstance = GetModuleHandle(NULL);

    timeBeginPeriod(1);

    SetProcessDpiAwareness(PROCESS_SYSTEM_DPI_AWARE);

    // Initialize global strings
    LoadStringW(hInstance, IDS_APP_TITLE, szTitle, MAX_LOADSTRING);
    LoadStringW(hInstance, IDC_PACMAN, szWindowClass, MAX_LOADSTRING);
    MyRegisterClass(hInstance);

    // Perform application initialization:
    if (!InitInstance(hInstance)) {
        return FALSE;
    }

    MSG msg;

    // Main message loop:
    while (GetMessage(&msg, nullptr, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }

    timeEndPeriod(1);

    return (int) msg.wParam;
}


//
//  FUNCTION: MyRegisterClass()
//
//  PURPOSE: Registers the window class.
//
ATOM MyRegisterClass(HINSTANCE hInstance) {
    WNDCLASSEXW wcex;

    wcex.cbSize = sizeof(WNDCLASSEX);
    wcex.style          = CS_OWNDC;
    wcex.lpfnWndProc    = WndProc;
    wcex.cbClsExtra     = 0;
    wcex.cbWndExtra     = 0;
    wcex.hInstance      = hInstance;
    wcex.hIcon          = LoadIcon(hInstance, MAKEINTRESOURCE(IDI_PACMAN));
    wcex.hCursor        = LoadCursor(nullptr, IDC_ARROW);
    wcex.hbrBackground  = (HBRUSH)NULL;
    wcex.lpszMenuName   = MAKEINTRESOURCEW(IDC_PACMAN);
    wcex.lpszClassName  = szWindowClass;
    wcex.hIconSm        = LoadIcon(wcex.hInstance, MAKEINTRESOURCE(IDI_SMALL));

    return RegisterClassExW(&wcex);
}

//
//   FUNCTION: InitInstance(HINSTANCE, int)
//
//   PURPOSE: Saves instance handle and creates main window
//
//   COMMENTS:
//
//        In this function, we save the instance handle in a global variable and
//        create and display the main program window.
//
BOOL InitInstance(HINSTANCE hInstance) {
   hInst = hInstance; // Store instance handle in our global variable

   HWND hWnd = CreateWindowW(szWindowClass, szTitle, WS_OVERLAPPEDWINDOW,
      CW_USEDEFAULT, 0, CW_USEDEFAULT, 0, nullptr, nullptr, hInstance, nullptr);

   if (!hWnd) {
      return FALSE;
   }

   WINDOW_HANDLE = hWnd;
   DEVICE_CONTEXT = GetDC(hWnd);

   PIXELFORMATDESCRIPTOR pfd = {
       sizeof(PIXELFORMATDESCRIPTOR),
       1,
       PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,    //Flags
       PFD_TYPE_RGBA,        // The kind of framebuffer. RGBA or palette.
       32,                   // Colordepth of the framebuffer.
       0, 0, 0, 0, 0, 0,
       0,
       0,
       0,
       0, 0, 0, 0,
       24,                   // Number of bits for the depthbuffer
       8,                    // Number of bits for the stencilbuffer
       0,                    // Number of Aux buffers in the framebuffer.
       PFD_MAIN_PLANE,
       0,
       0, 0, 0
   };
   
   int pixelFormat = ChoosePixelFormat(DEVICE_CONTEXT, &pfd);
   SetPixelFormat(DEVICE_CONTEXT, pixelFormat, &pfd);

   OPENGL_CONTEXT = wglCreateContext(DEVICE_CONTEXT);

   CreateThread(NULL, 0, GameThreadMain, 0, 0, &GAME_THREAD_ID);

   ShowWindow(hWnd, SW_SHOW);
   UpdateWindow(hWnd);

   return TRUE;
}

//
//  FUNCTION: WndProc(HWND, UINT, WPARAM, LPARAM)
//
//  PURPOSE:  Processes messages for the main window.
//
//  WM_COMMAND  - process the application menu
//  WM_PAINT    - Paint the main window
//  WM_DESTROY  - post a quit message and return
//
//
LRESULT CALLBACK WndProc(HWND hwnd, UINT message, WPARAM wparam, LPARAM lparam) {
    switch (message) {
        case WM_CLOSE: {
            PostThreadMessage(GAME_THREAD_ID, message, wparam, lparam);
        } break;

        case WM_USER_QUIT: {
            PostQuitMessage(0);
        } break;

        default: {    
            return DefWindowProc(hwnd, message, wparam, lparam);
        }
    }

    return 0;
}
