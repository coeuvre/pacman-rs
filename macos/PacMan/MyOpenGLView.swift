import Cocoa
import OpenGL.GL

class MyOpenGLView: NSOpenGLView {

    var displayLink: CVDisplayLink?
    
    required init?(coder: NSCoder) {
        super.init(coder: coder)
        
        let attributes: [NSOpenGLPixelFormatAttribute] = [
            UInt32(NSOpenGLPFAAccelerated),
            UInt32(NSOpenGLPFAColorSize), UInt32(32),
            UInt32(NSOpenGLPFADoubleBuffer),
            UInt32(NSOpenGLPFAOpenGLProfile),
            UInt32(NSOpenGLProfileVersion3_2Core),
            UInt32(0)
        ]
        self.pixelFormat = NSOpenGLPixelFormat(attributes: attributes)
        self.openGLContext = NSOpenGLContext(format: pixelFormat!, share: nil)
        self.openGLContext?.setValues([1], for: NSOpenGLContext.Parameter.swapInterval)
    }
    
    deinit {
        CVDisplayLinkStop(displayLink!)
    }
    
    override func prepareOpenGL() {
        super.prepareOpenGL()

//        pacman_init(getProcAddress)

        func displayLinkOutputCallback(displayLink: CVDisplayLink, _ now: UnsafePointer<CVTimeStamp>, _ outputTime: UnsafePointer<CVTimeStamp>, _ flagsIn: CVOptionFlags, _ flagsOut: UnsafeMutablePointer<CVOptionFlags>, _ displayLinkContext: UnsafeMutableRawPointer?) -> CVReturn {
            unsafeBitCast(displayLinkContext, to: MyOpenGLView.self).renderFrame()
            return kCVReturnSuccess
        }
        CVDisplayLinkCreateWithActiveCGDisplays(&displayLink)
        CVDisplayLinkSetOutputCallback(displayLink!, displayLinkOutputCallback, UnsafeMutableRawPointer(Unmanaged.passUnretained(self).toOpaque()))
        CVDisplayLinkSetCurrentCGDisplayFromOpenGLContext(displayLink!, (self.openGLContext?.cglContextObj)!, (self.pixelFormat?.cglPixelFormatObj)!)
        CVDisplayLinkStart(displayLink!)
    }
    
    override func draw(_ dirtyRect: NSRect) {
        super.draw(dirtyRect)
        renderFrame()
    }
    
    func renderFrame() {
        pacman_update()
        pacman_render()
    }
}
//
//func getProcAddress(name: UnsafePointer<CChar>!) -> UnsafeMutableRawPointer! {
//    let name = String(describing: name)
//    switch name {
//    case "glGetString": return UnsafeMutableRawPointer(bitPattern: glGetString)
//    default:
//        return UnsafeMutableRawPointer(bitPattern: 0)
//    }
//}

