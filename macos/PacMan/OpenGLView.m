#import "OpenGLView.h"

#import "pacman.h"

static CFBundleRef openglBundleRef;

void *getGLProcAddress(const char *name) {
    CFStringRef symbolName = CFStringCreateWithCString(kCFAllocatorDefault, name, kCFStringEncodingASCII);
    void *symbol = CFBundleGetFunctionPointerForName(openglBundleRef, symbolName);
    CFRelease(symbolName);
    return symbol;
}

@implementation OpenGLView {
    CVDisplayLinkRef displayLink; //display link for managing rendering thread
}

- (void)prepareOpenGL {
    [super prepareOpenGL];
    
    openglBundleRef = CFBundleGetBundleWithIdentifier(CFSTR("com.apple.opengl"));
    
    // Synchronize buffer swaps with vertical refresh rate
    GLint swapInt = 1;
    [[self openGLContext] setValues:&swapInt forParameter:NSOpenGLCPSwapInterval];

    pacman_init(&getGLProcAddress);

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

- (void)dealloc {
    // Release the display link
    CVDisplayLinkStop(displayLink);
    CVDisplayLinkRelease(displayLink);
}

- (void)drawRect:(NSRect)dirtyRect {
    [super drawRect:dirtyRect];

    [self renderFrame];
}

- (void)renderFrame {
    [[self openGLContext] makeCurrentContext];
    
    CGLLockContext([[self openGLContext] CGLContextObj]);
    
    pacman_render();
    
//    void (*glClearColor)(float, float, float, float) = getGLProcAddress("glClearColor");
//    glClearColor(1.0, 0.0, 0.0, 0.0);
//    glClear(GL_COLOR_BUFFER_BIT);
//    glFlush();
    
    CGLFlushDrawable([[self openGLContext] CGLContextObj]);
    CGLUnlockContext([[self openGLContext] CGLContextObj]);
}

- (CVReturn)getFrameForTime:(const CVTimeStamp*)outputTime {
        //    pacman_update();
    [self renderFrame];
    
    return kCVReturnSuccess;
}

// This is the renderer output callback function
static CVReturn displayLinkCallback(CVDisplayLinkRef displayLink, const CVTimeStamp* now, const CVTimeStamp* outputTime, CVOptionFlags flagsIn, CVOptionFlags* flagsOut, void* displayLinkContext) {
    @autoreleasepool {
        return [(__bridge OpenGLView *)displayLinkContext getFrameForTime:outputTime];
    }
}

@end
