use std::ops::*;

pub type Scalar = f32;

#[derive(Clone, Copy)]
pub struct Vec2 {
    pub x: Scalar,
    pub y: Scalar,
}

impl Vec2 {
    #[inline]
    pub fn new(x: Scalar, y: Scalar) -> Vec2 {
        Vec2 { x, y }
    }

    #[inline]
    pub fn zero() -> Vec2 {
        Vec2::new(0.0, 0.0)
    }

    #[inline]
    pub fn hadamard(&self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl Mul<Scalar> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: Scalar) -> Vec2 {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}

impl Div<Vec2> for Scalar {
    type Output = Vec2;

    fn div(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self / rhs.x, self / rhs.y)
    }
}

impl Sub<Vec2> for Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

#[derive(Clone, Copy)]
pub struct Rect2 {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect2 {
    #[inline]
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }
}
