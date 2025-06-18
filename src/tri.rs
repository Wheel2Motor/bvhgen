#![allow(dead_code)]
#![allow(unused_imports)]

use crate::prelude::*;
use std::rc::Rc;

pub mod prelude {
    pub use super::IndexedTri;
    pub use super::Tri;
    pub use super::TriIndex;
}

#[derive(Clone, Debug)]
pub struct Tri {
    pub pt0: Vec3,
    pub pt1: Vec3,
    pub pt2: Vec3,
}

impl Tri {
    fn new(pt0: &Vec3, pt1: &Vec3, pt2: &Vec3) -> Self {
        Self {
            pt0: *pt0,
            pt1: *pt1,
            pt2: *pt2,
        }
    }
}

#[derive(Clone, Debug)]
pub struct IndexedTri {
    pub vtx_buf: Rc<Vec<Vec3>>,
    pub indices: [usize; 3],
}

impl IndexedTri {
    pub fn new(vtx_buf: Rc<Vec<Vec3>>, idxpt0: usize, idxpt1: usize, idxpt2: usize) -> Self {
        Self {
            vtx_buf,
            indices: [idxpt0, idxpt1, idxpt2],
        }
    }

    pub fn to_tri(&self) -> Tri {
        let idx0 = self.indices[0];
        let idx1 = self.indices[1];
        let idx2 = self.indices[2];
        Tri::new(
            &self.vtx_buf[idx0],
            &self.vtx_buf[idx1],
            &self.vtx_buf[idx2],
        )
    }
}

#[derive(Clone, Debug)]
pub struct TriIndex {
    pub pt0: usize,
    pub pt1: usize,
    pub pt2: usize,
}

impl TriIndex {
    pub fn new(pt0: usize, pt1: usize, pt2: usize) -> Self {
        Self { pt0, pt1, pt2 }
    }

    pub fn to_tri(&self, pts: Rc<Vec<Vec3>>) -> Tri {
        let pt0 = pts[self.pt0];
        let pt1 = pts[self.pt1];
        let pt2 = pts[self.pt2];
        Tri::new(&pt0, &pt1, &pt2)
    }

    pub fn to_indexed_tri(&self, pts: Rc<Vec<Vec3>>) -> IndexedTri {
        IndexedTri::new(pts, self.pt0, self.pt1, self.pt2)
    }
}
