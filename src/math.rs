use std::ops::*;

pub type Scalar = f32;

#[derive(Clone, Copy)]
pub struct Vec2i {
    pub x: i32,
    pub y: i32,
}

impl Vec2i {
    #[inline]
    pub fn new(x: i32, y: i32) -> Vec2i {
        Vec2i { x, y }
    }

    pub fn as_vec2(self) -> Vec2 {
        Vec2::new(self.x as Scalar, self.y as Scalar)
    }
}

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
    pub fn hadamard(self, rhs: Vec2) -> Vec2 {
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

impl Add<Vec2> for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
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
    pub fn with_min_max(min: Vec2, max: Vec2) -> Rect2 {
        Rect2 {
            min,
            max,
        }
    }

    #[inline]
    pub fn with_min_size(min: Vec2, size: Vec2) -> Rect2 {
        Rect2 {
            min,
            max: min + size,
        }
    }

    #[inline]
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }
}

#[derive(Clone, Copy)]
pub struct Vec3 {
    pub x: Scalar,
    pub y: Scalar,
    pub z: Scalar,
}

impl Vec3 {
    #[inline]
    pub fn new(x: Scalar, y: Scalar, z: Scalar) -> Vec3 {
        Vec3 { x, y, z }
    }
}

impl Mul<Scalar> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f32) -> Vec3 {
        Vec3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

#[derive(Clone, Copy)]
pub struct Vec4 {
    pub x: Scalar,
    pub y: Scalar,
    pub z: Scalar,
    pub w: Scalar,
}

impl Vec4 {
    #[inline]
    pub fn new(x: Scalar, y: Scalar, z: Scalar, w: Scalar) -> Vec4 {
        Vec4 { x, y, z, w }
    }

    #[inline]
    pub fn from_xyz(xyz: Vec3, w: Scalar) -> Vec4 {
        Vec4::new(xyz.x, xyz.y, xyz.z, w)
    }

    #[inline]
    pub fn xyz(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl Div<Scalar> for Vec4 {
    type Output = Vec4;

    fn div(self, rhs: f32) -> Vec4 {
        Vec4::new(self.x / rhs, self.y / rhs, self.z / rhs, self.w / rhs)
    }
}

impl Mul<Scalar> for Vec4 {
    type Output = Vec4;

    fn mul(self, rhs: f32) -> Vec4 {
        Vec4::new(self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs)
    }
}
