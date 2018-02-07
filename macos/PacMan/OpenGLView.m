#import "OpenGLView.h"

#import "pacman.h"

static int add(int x, int y) {
    return x + y;
}

@implementation OpenGLView {
    CVDisplayLinkRef displayLink; //display link for managing rendering thread
}

- (instancetype)initWithCoder:(NSCoder *)decoder {
    self = [super initWithCoder:decoder];
    
    const NSOpenGLPixelFormatAttribute attributes[] = {
        NSOpenGLPFAAccelerated,
        NSOpenGLPFAColorSize, 32,
        NSOpenGLPFADoubleBuffer,
        NSOpenGLPFAOpenGLProfile, NSOpenGLProfileVersion3_2Core,
        0
    };
    
    [self setPixelFormat:[[NSOpenGLPixelFormat alloc] initWithAttributes:attributes]];
    [self setOpenGLContext:[[NSOpenGLContext alloc] initWithFormat:self.pixelFormat shareContext:NULL]];
    
    // Synchronize buffer swaps with vertical refresh rate
    GLint swapInt = 1;
    [[self openGLContext] setValues:&swapInt forParameter:NSOpenGLCPSwapInterval];
    
    return self;
}

- (void)prepareOpenGL {
    pacman_init(&add);
    
    // Create a display link capable of being used with all active displays
    CVDisplayLinkCreateWithActiveCGDisplays(&displayLink);
    
    // Set the renderer output callback function
    CVDisplayLinkSetOutputCallback(displayLink, &displayLinkCallback, (__bridge void *) self);
    
    // Set the display link for the current renderer
    CGLContextObj cglContext = [[self openGLContext] CGLContextObj];
    CGLPixelFormatObj cglPixelFormat = [[self pixelFormat] CGLPixelFormatObj];
    CVDisplayLinkSetCurrentCGDisplayFromOpenGLContext(displayLink, cglContext, cglPixelFormat);
    
    // Activate the display link
    CVDisplayLinkStart(displayLink);
}

- (CVReturn)getFrameForTime:(const CVTimeStamp*)outputTime {
    // Add your drawing codes here
    
    return kCVReturnSuccess;
}

// This is the renderer output callback function
static CVReturn displayLinkCallback(CVDisplayLinkRef displayLink, const CVTimeStamp* now, const CVTimeStamp* outputTime, CVOptionFlags flagsIn, CVOptionFlags* flagsOut, void* displayLinkContext)
{
    CVReturn result = [(__bridge OpenGLView *) displayLinkContext getFrameForTime:outputTime];
    return result;
}

@end
