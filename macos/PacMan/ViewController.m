#import "ViewController.h"
#import "OpenGLLayer.h"

#include <mach/mach.h>
#include <mach/mach_time.h>

#import "bridge.h"

static CFBundleRef OPENGL_BUNDLE_REF = nil;

Platform PLATFORM;
void *OPENGL_VIEW;

void quit() {
}

void *getGLProcAddress(const char *name) {
    CFStringRef symbolName = CFStringCreateWithCString(kCFAllocatorDefault, name, kCFStringEncodingASCII);
    void *symbol = CFBundleGetFunctionPointerForName(OPENGL_BUNDLE_REF, symbolName);
    CFRelease(symbolName);
    return symbol;
}

void swapGlBuffers() {
    //    @autoreleasepool {
    //        [[(__bridge OpenGLView *)OPENGL_VIEW openGLContext] flushBuffer];
    //    }
}

uint64_t getPerformanceCounter() {
    return mach_absolute_time();
}

uint64_t getPerformanceFrequency() {
    mach_timebase_info_data_t timebaseInfo;
    mach_timebase_info(&timebaseInfo);
    return (uint64_t)(timebaseInfo.denom * 1000000000.0 / timebaseInfo.numer);
}

@interface ViewController ()

@property (strong, nonatomic) OpenGLLayer *openGLLayer;

@end

@implementation ViewController

- (void)viewDidLoad {
    [super viewDidLoad];

    self.openGLLayer = [OpenGLLayer layer];
    self.gameView.wantsLayer = YES;
    self.gameView.layer = self.openGLLayer;
    
    OPENGL_BUNDLE_REF = CFBundleGetBundleWithIdentifier(CFSTR("com.apple.opengl"));
    PLATFORM.quit = &quit;
    PLATFORM.get_gl_proc_address = &getGLProcAddress;
    PLATFORM.swap_gl_buffers = &swapGlBuffers;
    PLATFORM.get_performance_counter = &getPerformanceCounter;
    PLATFORM.get_performance_frequency = &getPerformanceFrequency;
    game_load(&PLATFORM);
}

- (void)viewWillDisappear {
    self.gameView.layer = nil;
    self.openGLLayer = nil;
}

- (void)setRepresentedObject:(id)representedObject {
    [super setRepresentedObject:representedObject];

    // Update the view, if already loaded.
}


@end
