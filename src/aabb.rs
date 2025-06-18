#![allow(dead_code)]
#![allow(unused_imports)]

use crate::prelude::*;

pub mod prelude {
    pub use super::AABBSplitAxis;
    pub use super::AABB;
}

#[derive(Copy, Clone, Debug)]
pub enum AABBSplitAxis {
    X,
    Y,
    Z,
}

#[derive(Default, Clone, Debug)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: &Vec3, max: &Vec3) -> Self {
        Self {
            min: *min,
            max: *max,
        }
    }

    pub fn from_points(points: &[Vec3]) -> Self {
        let npts = points.len();
        match npts {
            0 => Self::default(),
            _ => {
                let mut ret = Self::default();
                let first = points.first().unwrap();
                ret.min = *first;
                ret.max = *first;
                let mut iter_points = points.iter();
                iter_points.next();
                for pt in iter_points {
                    ret.min = Vec3::min(&ret.min, pt);
                    ret.max = Vec3::max(&ret.max, pt);
                }
                ret
            }
        }
    }

    pub fn from_point3(pt0: &Vec3, pt1: &Vec3, pt2: &Vec3) -> Self {
        let mut ret = Self::new(pt0, pt0);
        ret.min = Vec3::min(&ret.min, pt1);
        ret.max = Vec3::max(&ret.max, pt1);
        ret.min = Vec3::min(&ret.min, pt2);
        ret.max = Vec3::max(&ret.max, pt2);
        ret
    }

    pub fn point_in_aabb(&self, pt: &Vec3) -> bool {
        if pt.x < self.min.x {
            return false;
        }
        if pt.x > self.max.x {
            return false;
        }
        if pt.y < self.min.y {
            return false;
        }
        if pt.y > self.max.y {
            return false;
        }
        true
    }

    pub fn intersect_with_aabb(&self, other: &Self) -> bool {
        if self.max.x < other.min.x || self.min.x > other.max.x {
            return false;
        }
        if self.max.y < other.min.y || self.min.y > other.max.y {
            return false;
        }
        if self.max.z < other.min.z || self.min.z > other.max.z {
            return false;
        }
        true
    }

    pub fn extent(&self) -> Vec3 {
        Vec3 {
            x: self.max.x - self.min.x,
            y: self.max.y - self.min.y,
            z: self.max.z - self.min.z,
        }
    }

    pub fn center(&self) -> Vec3 {
        let mut ext = self.extent();
        ext *= Vec3::new(0.5, 0.5, 0.5);
        ext += self.min;
        ext
    }

    pub fn split(&self, axis: AABBSplitAxis) -> (Self, Self) {
        let center = self.center();
        let mut pos_min = self.min;
        let pos_max = self.max;
        let neg_min = self.min;
        let mut neg_max = self.max;
        match axis {
            AABBSplitAxis::X => {
                pos_min.x = center.x;
                neg_max.x = center.x;
            }
            AABBSplitAxis::Y => {
                pos_min.y = center.y;
                neg_max.y = center.y;
            }
            AABBSplitAxis::Z => {
                pos_min.z = center.z;
                neg_max.z = center.z;
            }
        }
        (Self::new(&pos_min, &pos_max), Self::new(&neg_min, &neg_max))
    }

    #[allow(unused_assignments)]
    pub fn largest_axis(&self) -> AABBSplitAxis {
        let ext = self.extent();
        let mut span = ext.x;
        let mut axis = AABBSplitAxis::X;
        if span < ext.y {
            span = ext.y;
            axis = AABBSplitAxis::Y;
        }
        if span < ext.z {
            span = ext.z;
            axis = AABBSplitAxis::Z;
        }
        axis
    }

    pub fn normalize(&mut self) {
        let extent = self.extent();
        let dim = extent.x.max(extent.y).max(extent.z);
        let next = Vec3::new(dim, dim, dim);
        let hext = next / Vec3::new(2.0, 2.0, 2.0);
        let center = self.center();
        self.min = center - hext;
        self.max = center + hext;
    }

    pub fn expand(&mut self, dist: &Vec3) {
        let half = *dist / Vec3::new(2.0, 2.0, 2.0);
        self.min -= half;
        self.max += half;
    }
}
