type Scalar = f32;

pub struct TextMetrics {
    pub width: Scalar,
}

trait CanvasRenderingContext2D {
    // TODO: state
//    fn save(&mut self);
//    fn restore(&mut self);

    // TODO: transformations (default: transform is the identity matrix)
//    fn scale(&mut self, x: Scalar, y: Scalar);
//    fn rotate(&mut self, angle: Scalar);
//    fn translate(&mut self, x: Scalar, y: Scalar);
//    fn transform(&mut self, a: Scalar, b: Scalar, c: Scalar, d: Scalar, e: Scalar, f: Scalar);

    // TODO: compositing
    // TODO: colors and styles
    // TODO: shadows

    // TODO: rects
//    fn clear_rect(&mut self, x: Scalar, y: Scalar, w: Scalar, h: Scalar);
//    fn fill_rect(&mut self, x: Scalar, y: Scalar, w: Scalar, h: Scalar);
//    fn stroke_rect(&mut self, x: Scalar, y: Scalar, w: Scalar, h: Scalar);

    // path API
    fn begin_path(&mut self);
    fn close_path(&mut self);
    fn move_to(&mut self, x: Scalar, y: Scalar);
    fn line_to(&mut self, x: Scalar, y: Scalar);
    fn quadratic_curve_to(&mut self, cpx: Scalar, cpy:Scalar, x: Scalar, y: Scalar);
    fn bezier_curve_to(&mut self, cp1x: Scalar, cp1y:Scalar, cp2x: Scalar, cp2y:Scalar, x: Scalar, y: Scalar);
    fn arc_to(&mut self, x1: Scalar, y1: Scalar, x2: Scalar, y2: Scalar, radius: Scalar);
    fn rect(&mut self, x: Scalar, y: Scalar, w: Scalar, h: Scalar);
    fn arc(&mut self, x: Scalar, y: Scalar, radius: Scalar, start_angle: Scalar, end_angle: Scalar, counterclockwise: bool);
    fn fill(&mut self);
    fn stroke(&mut self);
//    fn draw_focus_if_needed(&mut self, ...);
    fn clip(&mut self);
    fn is_point_in_path(&self, x: Scalar, y: Scalar) -> bool;

    // TODO: text
//    fn fill_text<T: AsRef<str>>(&mut self, text: T, x: Scalar, y: Scalar, max_width: Option<Scalar>);
//    fn stroke_text<T: AsRef<str>>(&mut self, text: T, x: Scalar, y: Scalar, max_width: Option<Scalar>);
//    fn measure_text<T: AsRef<str>>(&self, text: T) -> TextMetrics;

    // TODO: drawing images
}

trait Canvas {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn context(&self) -> &CanvasRenderingContext2D;
    fn context_mut(&mut self) -> &mut CanvasRenderingContext2D;
}
