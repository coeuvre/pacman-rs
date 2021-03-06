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
Platform PLATFORM;

#define WM_USER_QUIT (WM_USER + 1)
#define WM_USER_DRAW (WM_USER + 2)
#define WM_USER_PLATFORM_EVENT (WM_USER + 3)

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

void SwapGlBuffers() {
    SwapBuffers(DEVICE_CONTEXT);
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

DWORD WINAPI GameThreadMain(LPVOID param) {
    while (!QUIT) {
        SendMessage(WINDOW_HANDLE, WM_USER_DRAW, 0, 0);
    }

    SendMessage(WINDOW_HANDLE, WM_USER_QUIT, 0, 0);

    return 0;
}

int main() {
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

int APIENTRY wWinMain(_In_ HINSTANCE hInstance, _In_opt_ HINSTANCE hPrevInstance, _In_ LPWSTR lpCmdLine, _In_ int nCmdShow) {
    UNREFERENCED_PARAMETER(hInstance);
    UNREFERENCED_PARAMETER(hPrevInstance);
    UNREFERENCED_PARAMETER(lpCmdLine);
    UNREFERENCED_PARAMETER(nCmdShow);

    return main();
}

ATOM MyRegisterClass(HINSTANCE hInstance) {
    WNDCLASSEXW wcex;

    wcex.cbSize = sizeof(WNDCLASSEX);
    wcex.style          = CS_OWNDC | CS_VREDRAW | CS_HREDRAW;
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

BOOL InitInstance(HINSTANCE hInstance) {
   hInst = hInstance; // Store instance handle in our global variable

   HWND hWnd = CreateWindowEx(
       WS_EX_APPWINDOW | WS_EX_WINDOWEDGE,
       szWindowClass, szTitle,
       WS_OVERLAPPEDWINDOW | WS_CLIPSIBLINGS | WS_CLIPCHILDREN,
       CW_USEDEFAULT, 0,
       CW_USEDEFAULT, 0,
       NULL, NULL, hInstance, NULL);

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
   wglMakeCurrent(DEVICE_CONTEXT, OPENGL_CONTEXT);

   // Disable VSYNC
   //void (*wglSwapIntervalEXT)(int) = (void (*)(int))wglGetProcAddress("wglSwapIntervalEXT");
   //wglSwapIntervalEXT(0);

   PLATFORM.quit = Quit;
   PLATFORM.get_gl_proc_address = &GetGlProcAddress;
   PLATFORM.swap_gl_buffers = &SwapGlBuffers;
   PLATFORM.get_performance_counter = &GetPerformanceCounter;
   PLATFORM.get_performance_frequency = &GetPerformanceFrequency;
   game_load(&PLATFORM);

   CreateThread(NULL, 0, GameThreadMain, 0, 0, &GAME_THREAD_ID);

   ShowWindow(hWnd, SW_SHOW);
   UpdateWindow(hWnd);

   return TRUE;
}

LRESULT CALLBACK WndProc(HWND hwnd, UINT message, WPARAM wparam, LPARAM lparam) {
    PlatformEvent event;

    switch (message) {
        case WM_CLOSE: {
            event.kind = PLATFORM_EVENT_CLOSE;
            game_on_platform_event(&event);
        } break;

        case WM_PAINT: {
            ValidateRect(hwnd, NULL);
        }; // Fall through
        case WM_USER_DRAW: {
            event.kind = PLATFORM_EVENT_RENDER;
            game_on_platform_event(&event);
        } break;

        case WM_SIZING: {
            RECT *rect = (RECT *)lparam;

            event.kind = PLATFORM_EVENT_RESIZE;
            event.data.resize.width = rect->right - rect->left;
            event.data.resize.height = rect->bottom - rect->top;
            game_on_platform_event(&event);

            return DefWindowProc(hwnd, message, wparam, lparam);
        } break;

        case WM_USER_QUIT: {
            game_quit();
            wglMakeCurrent(NULL, NULL);
            PostQuitMessage(0);
        } break;

        default: {
            return DefWindowProc(hwnd, message, wparam, lparam);
        }
    }

    return 0;
}
