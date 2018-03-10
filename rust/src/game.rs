use gl;

pub struct Game {}

impl Game {
    pub fn new() -> Game {
        Game {}
    }

    pub fn update(&mut self, _dt: f32) {

    }

    pub fn render(&self) {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }
}