//
//  OpenGLView.m
//  PacMan
//
//  Created by Coeuvre Wong on 2018/3/25.
//  Copyright © 2018 Coeuvre Wong. All rights reserved.
//

#import "OpenGLView.h"

#import "bridge.h"

static CFBundleRef OPENGL_BUNDLE_REF = nil;

int QUIT = 0;
Platform PLATFORM;
void *OPENGL_VIEW;

void quit() {
    QUIT = 1;
    @autoreleasepool {
        [NSApp terminate:(__bridge OpenGLView *)OPENGL_VIEW];
    }
}

void *getGLProcAddress(const char *name) {
    CFStringRef symbolName = CFStringCreateWithCString(kCFAllocatorDefault, name, kCFStringEncodingASCII);
    void *symbol = CFBundleGetFunctionPointerForName(OPENGL_BUNDLE_REF, symbolName);
    CFRelease(symbolName);
    return symbol;
}

void swapGlBuffers() {
    [[(__bridge OpenGLView *)OPENGL_VIEW openGLContext] flushBuffer];
}

uint64_t getPerformanceCounter() {
    return mach_absolute_time();
}

uint64_t getPerformanceFrequency() {
    mach_timebase_info_data_t timebaseInfo;
    mach_timebase_info(&timebaseInfo);
    return (uint64_t)(timebaseInfo.denom * 1000000000.0 / timebaseInfo.numer);
}

@implementation OpenGLView {
    CVDisplayLinkRef displayLink;
}

- (instancetype)initWithCoder:(NSCoder *)decoder {
    self = [super initWithCoder:decoder];
    
    if (OPENGL_BUNDLE_REF == nil) {
        OPENGL_BUNDLE_REF = CFBundleGetBundleWithIdentifier(CFSTR("com.apple.opengl"));
    }
    
    NSOpenGLPixelFormatAttribute attributes[] = {
        NSOpenGLPFAAccelerated,
        NSOpenGLPFAColorSize, 32,
        NSOpenGLPFADoubleBuffer,
        NSOpenGLPFAOpenGLProfile, NSOpenGLProfileVersion3_2Core,
        0
    };
    
    NSOpenGLPixelFormat *pixelFormat = [[NSOpenGLPixelFormat alloc] initWithAttributes:attributes];
    if (pixelFormat == nil) {
        panic("Failed to init pixelFormat");
    }
    
    [self setPixelFormat:pixelFormat];
    
    NSOpenGLContext *openGLContext = [[NSOpenGLContext alloc] initWithFormat:pixelFormat shareContext:nil];
    [self setOpenGLContext:openGLContext];
    
    return self;
}

- (void)prepareOpenGL {
    [super prepareOpenGL];
    
    // Synchronize buffer swaps with vertical refresh rate
    GLint swapInt = 1;
    [[self openGLContext] setValues:&swapInt forParameter:NSOpenGLCPSwapInterval];
    
    OPENGL_VIEW = (__bridge void *)self;
    PLATFORM.quit = &quit;
    PLATFORM.get_gl_proc_address = &getGLProcAddress;
    PLATFORM.swap_gl_buffers = &swapGlBuffers;
    PLATFORM.get_performance_counter = &getPerformanceCounter;
    PLATFORM.get_performance_frequency = &getPerformanceFrequency;
    game_load(&PLATFORM);
    
    // Create a display link capable of being used with all active displays
    CVDisplayLinkCreateWithActiveCGDisplays(&displayLink);
    
    // Set the renderer output callback function
    CVDisplayLinkSetOutputCallback(displayLink, &displayLinkCallback, (__bridge void *)self);
    
    // Set the display link for the current renderer
    CGLContextObj cglContext = [[self openGLContext] CGLContextObj];
    CGLPixelFormatObj cglPixelFormat = [[self pixelFormat] CGLPixelFormatObj];
    CVDisplayLinkSetCurrentCGDisplayFromOpenGLContext(displayLink, cglContext, cglPixelFormat);
    
    // Activate the display link
    CVDisplayLinkStart(displayLink);
}

- (void)drawRect:(NSRect)dirtyRect {    
    PlatformEvent event;
    
    event.kind = PLATFORM_EVENT_UPDATE;
    game_on_platform_event(&event);
    
    event.kind = PLATFORM_EVENT_RENDER;
    game_on_platform_event(&event);
}

- (void)reshape {
    PlatformEvent event;
    event.kind = PLATFORM_EVENT_RESIZE;
    CGSize size = [self frame].size;
    event.data.resize.width = size.width;
    event.data.resize.height = size.height;
    game_on_platform_event(&event);
}

- (void)dealloc {
    game_quit();
    OPENGL_VIEW = NULL;
    
    // Release the display link
    CVDisplayLinkStop(displayLink);
    CVDisplayLinkRelease(displayLink);
}

// This is the renderer output callback function
static CVReturn displayLinkCallback(CVDisplayLinkRef displayLink, const CVTimeStamp* now, const CVTimeStamp* outputTime, CVOptionFlags flagsIn, CVOptionFlags* flagsOut, void* displayLinkContext) {
    @autoreleasepool {
        dispatch_async(dispatch_get_main_queue(), ^{
            [(__bridge OpenGLView *)displayLinkContext getFrameForTime:outputTime];
        });
    }
    
    return kCVReturnSuccess;
}

- (void)getFrameForTime:(const CVTimeStamp*)outputTime {
    [self setNeedsDisplay:YES];
}

@end

