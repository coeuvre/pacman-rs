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
DWORD LIB_THREAD_ID;
BOOL QUIT;
PlatformApi PLATFORM;
LibApi *LIB;

#define WM_USER_UPDATE (WM_USER + 1)
#define WM_USER_RENDER (WM_USER + 2)
#define WM_USER_QUIT (WM_USER + 3)

// Forward declarations of functions included in this code module:
ATOM MyRegisterClass(HINSTANCE hInstance);
BOOL InitInstance(HINSTANCE, int);
LRESULT CALLBACK WndProc(HWND, UINT, WPARAM, LPARAM);

static void Log(const char *message) {
	OutputDebugStringA(message);
}

static void Printf(const char *fmt, ...) {
	char buf[1024];
	va_list va;
	va_start(va, fmt);
	vsnprintf(buf, sizeof(buf), fmt, va);
	Log(buf);
	va_end(va);
}

void Quit() {
	QUIT = 1;
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

float GetDeltaTime() {
	return 0.016f;
}

ULONGLONG RENDER_END;

DWORD WINAPI GameLoop(LPVOID param) {
	ULONGLONG targetFrameTime = 16;

    while (!QUIT) {
		ULONGLONG frameStart = GetTickCount64();

        SendMessage(WINDOW_HANDLE, WM_USER_RENDER, 0, 0);

		ULONGLONG frameEnd = GetTickCount64();

		ULONGLONG frameCost = frameEnd - frameStart;

		Printf("Frame cost %d, Render %d\n", (int)frameCost, (int)(RENDER_END - frameStart));

		if (frameCost < targetFrameTime) {
			Sleep((DWORD)(targetFrameTime - frameCost));
		}
    }

    SendMessage(WINDOW_HANDLE, WM_USER_QUIT, 0, 0);

    return 0;
}

int APIENTRY wWinMain(_In_ HINSTANCE hInstance, _In_opt_ HINSTANCE hPrevInstance, _In_ LPWSTR lpCmdLine, _In_ int nCmdShow) {
    UNREFERENCED_PARAMETER(hPrevInstance);
    UNREFERENCED_PARAMETER(lpCmdLine);

	timeBeginPeriod(1);

    SetProcessDpiAwareness(PROCESS_SYSTEM_DPI_AWARE);

    // Initialize global strings
    LoadStringW(hInstance, IDS_APP_TITLE, szTitle, MAX_LOADSTRING);
    LoadStringW(hInstance, IDC_PACMAN, szWindowClass, MAX_LOADSTRING);
    MyRegisterClass(hInstance);

    // Perform application initialization:
    if (!InitInstance (hInstance, nCmdShow)) {
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
    wcex.style          = CS_VREDRAW | CS_HREDRAW | CS_OWNDC;
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
BOOL InitInstance(HINSTANCE hInstance, int nCmdShow) {
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
   wglMakeCurrent(DEVICE_CONTEXT, OPENGL_CONTEXT);

   PLATFORM.quit = &Quit;
   PLATFORM.log = &Log;
   PLATFORM.get_gl_proc_address = &GetGlProcAddress;
   PLATFORM.get_delta_time = &GetDeltaTime;

   LIB = pacman_load(&PLATFORM);

   CreateThread(NULL, 0, GameLoop, 0, 0, &LIB_THREAD_ID);

   ShowWindow(hWnd, nCmdShow);
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
        case WM_USER_RENDER:
        case WM_PAINT: {
			InvalidateRect(WINDOW_HANDLE, NULL, TRUE);

            if (LIB) {
                LIB->render();
            }
            
            RENDER_END = GetTickCount64();

            SwapBuffers(DEVICE_CONTEXT);

			ValidateRect(hwnd, NULL);
        } break;

        case WM_CLOSE: {
			LIB->on_platform_event(PLATFORM_EVENT_CLOSE, NULL);
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
