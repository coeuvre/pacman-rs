import Cocoa
import OpenGL.GL

class MyOpenGLView: NSOpenGLView {

    override func draw(_ dirtyRect: NSRect) {
        super.draw(dirtyRect)

        glClearColor(0, 0, 0, 0);
        glClear(UInt32(GL_COLOR_BUFFER_BIT));
        glFlush();
    }
    
}
