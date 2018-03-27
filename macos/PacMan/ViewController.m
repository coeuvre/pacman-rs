#import "ViewController.h"

#import "bridge.h"

extern int QUIT;
extern Platform PLATFORM;

@implementation ViewController

- (void)viewDidLoad {
    [super viewDidLoad];
}

- (void)viewDidAppear {
    self.view.window.delegate = self;
}

- (void)setRepresentedObject:(id)representedObject {
    [super setRepresentedObject:representedObject];
}

- (BOOL)windowShouldClose:(NSWindow *)sender {
    if (QUIT) {
        return YES;
    } else {
        PlatformEvent event;
        event.kind = PLATFORM_EVENT_CLOSE;
        game_on_platform_event(&event);
        return NO;
    }
}

@end
