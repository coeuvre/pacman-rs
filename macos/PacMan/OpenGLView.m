#import "OpenGLView.h"

#import "pacman.h"

static CFBundleRef OPENGL_BUNDLE_REF = nil;

void *getGLProcAddress(const char *name) {
    CFStringRef symbolName = CFStringCreateWithCString(kCFAllocatorDefault, name, kCFStringEncodingASCII);
    void *symbol = CFBundleGetFunctionPointerForName(OPENGL_BUNDLE_REF, symbolName);
    CFRelease(symbolName);
    return symbol;
}

@implementation OpenGLView {
    CVDisplayLinkRef displayLink; //display link for managing rendering thread
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

- (void)renderFrame {
    [[self openGLContext] makeCurrentContext];
    
    CGLLockContext([[self openGLContext] CGLContextObj]);
    
    pacman_render();
    
    CGLFlushDrawable([[self openGLContext] CGLContextObj]);
    CGLUnlockContext([[self openGLContext] CGLContextObj]);
}

- (CVReturn)getFrameForTime:(const CVTimeStamp*)outputTime {
    pacman_update();

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
