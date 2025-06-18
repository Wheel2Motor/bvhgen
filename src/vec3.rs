#![allow(dead_code)]
#![allow(unused_imports)]

use std::ops::{AddAssign, DivAssign};

use crate::prelude::*;

pub mod prelude {
    pub use super::Vec3;
}

#[derive(Copy, Clone, Debug)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn min(v1: &Self, v2: &Self) -> Self {
        Self {
            x: v1.x.min(v2.x),
            y: v1.y.min(v2.y),
            z: v1.z.min(v2.z),
        }
    }

    pub fn max(v1: &Self, v2: &Self) -> Self {
        Self {
            x: v1.x.max(v2.x),
            y: v1.y.max(v2.y),
            z: v1.z.max(v2.z),
        }
    }

    pub fn min3(v1: &Self, v2: &Self, v3: &Self) -> Self {
        Self {
            x: v1.x.min(v2.x).min(v3.x),
            y: v1.y.min(v2.y).min(v3.y),
            z: v1.z.min(v2.z).min(v3.z),
        }
    }

    pub fn max3(v1: &Self, v2: &Self, v3: &Self) -> Self {
        Self {
            x: v1.x.max(v2.x).max(v3.x),
            y: v1.y.max(v2.y).max(v3.y),
            z: v1.z.max(v2.z).max(v3.z),
        }
    }

    pub fn length(&self) -> f64 {
        self.dot(self).sqrt()
    }

    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn distance_to(&self, other: &Self) -> f64 {
        let tmp = Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z);
        tmp.length()
    }

    pub fn direction_to(&self, other: &Self) -> Self {
        let mut tmp = *other - *self;
        tmp.normalize();
        tmp
    }

    pub fn move_towards(&mut self, direction: &Vec3, dist: f64) {
        let v = *direction * Vec3::new(dist, dist, dist);
        self.add_assign(v);
    }

    pub fn normalize(&mut self) {
        let len = self.length();
        if len == 0.0 {
            self.x = 0.0;
            self.y = 0.0;
            self.z = 0.0;
        } else {
            self.div_assign(Vec3::new(len, len, len));
        }
    }
}

impl Default for Vec3 {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

impl std::ops::Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl std::ops::Mul for Vec3 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl std::ops::Div for Vec3 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl std::ops::AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl std::ops::SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl std::ops::MulAssign for Vec3 {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl std::ops::DivAssign for Vec3 {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}
