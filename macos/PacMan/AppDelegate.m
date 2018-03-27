#import "AppDelegate.h"

#import "bridge.h"

extern int QUIT;
extern Platform PLATFORM;

@interface AppDelegate ()

@end

@implementation AppDelegate

- (void)applicationDidFinishLaunching:(NSNotification *)aNotification {
    // Insert code here to initialize your application
}


- (void)applicationWillTerminate:(NSNotification *)aNotification {
    // Insert code here to tear down your application
}

- (BOOL)applicationShouldTerminateAfterLastWindowClosed:(NSApplication *)sender {
    return TRUE;
}

- (NSApplicationTerminateReply)applicationShouldTerminate:(NSApplication *)sender {
    if (QUIT) {
        return NSTerminateNow;
    } else {
        PlatformEvent event;
        event.kind = PLATFORM_EVENT_CLOSE;
        game_on_platform_event(&event);
        return NSTerminateCancel;
    }
}

@end
