//
//  OpenGLLayer.m
//  PacMan
//
//  Created by Coeuvre Wong on 2018/3/24.
//  Copyright © 2018 Coeuvre Wong. All rights reserved.
//

#import "OpenGLLayer.h"

#import "bridge.h"

@implementation OpenGLLayer

- (id)init {
    self = [super init];
    
    if (self) {
        self.asynchronous = YES;
    }
    
    return self;
}

- (CGLContextObj)copyCGLContextForPixelFormat:(CGLPixelFormatObj)pixelFormat {
    CGLContextObj context = NULL;
    
    CGLCreateContext(pixelFormat, NULL, &context);
    
    if (context || (context = [super copyCGLContextForPixelFormat:pixelFormat])) {
        // Setup any OpenGL state, make sure to set the context before invoking OpenGL
        
        CGLContextObj currContext = CGLGetCurrentContext();
        CGLSetCurrentContext(context);
        
        // Issue any calls that require the context here.

        CGLSetCurrentContext(currContext);
    }
    
    return context;
}

- (CGLPixelFormatObj)copyCGLPixelFormatForDisplayMask:(uint32_t)mask {
    CGLPixelFormatAttribute attribs[] =  {
        kCGLPFADisplayMask, mask,
        kCGLPFAColorSize, 24,
        kCGLPFAAlphaSize, 8,
        kCGLPFAAccelerated,
        kCGLPFADoubleBuffer,
        NSOpenGLPFAOpenGLProfile, NSOpenGLProfileVersion3_2Core,
        0
    };
    
    CGLPixelFormatObj pixFormatObj = NULL;
    GLint numPixFormats = 0;
    
    CGLChoosePixelFormat(attribs, &pixFormatObj, &numPixFormats);
    
    return pixFormatObj;
}

- (BOOL)canDrawInCGLContext:(CGLContextObj)ctx pixelFormat:(CGLPixelFormatObj)pf forLayerTime:(CFTimeInterval)t displayTime:(const CVTimeStamp *)ts {
    return YES;
}

- (void)drawInCGLContext:(CGLContextObj)ctx pixelFormat:(CGLPixelFormatObj)pf forLayerTime:(CFTimeInterval)t displayTime:(const CVTimeStamp *)ts {
    PlatformEvent event;
    event.kind = PLATFORM_EVENT_RENDER;
    game_on_platform_event(&event);
}

@end
