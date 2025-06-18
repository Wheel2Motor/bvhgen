#![allow(dead_code)]
#![allow(unused_imports)]

use crate::prelude::*;
use std::{fmt::Error, rc::Rc};

pub mod prelude {
    pub use super::IndexedPoly;
    pub use super::Poly;
    pub use super::PolyIndex;
}

#[derive(Clone, Debug)]
pub struct Poly {
    pub vtx_buf: Vec<Vec3>,
}

impl Poly {
    pub fn new(vtx_buf: Vec<Vec3>) -> Self {
        Self { vtx_buf }
    }
}

#[derive(Clone, Debug)]
pub struct IndexedPoly {
    pub vtx_buf: Rc<Vec<Vec3>>,
    pub idx_buf: Vec<usize>,
}

impl IndexedPoly {
    pub fn new(vtx_buf: Rc<Vec<Vec3>>, idx_buf: Vec<usize>) -> Self {
        Self { vtx_buf, idx_buf }
    }

    pub fn to_poly(&self) -> Poly {
        Poly::new(self.idx_buf.iter().map(|idx| self.vtx_buf[*idx]).collect())
    }

    pub fn to_indexed_tri(&self) -> Vec<IndexedTri> {
        // 还只支持凸包。
        // 互联网上能找到的3D册多边形剖分实在太拉跨。
        let mut ret = Vec::<IndexedTri>::new();
        let mut counter = 1;
        while counter < self.idx_buf.len() - 1 {
            let idx0 = self.idx_buf[0];
            let idx1 = self.idx_buf[counter];
            let idx2 = self.idx_buf[counter + 1];
            let itri = IndexedTri::new(self.vtx_buf.clone(), idx0, idx1, idx2);
            ret.push(itri);
            counter += 1;
        }
        ret
    }
}

#[derive(Clone, Debug)]
pub struct PolyIndex {
    pub idx_buf: Vec<usize>,
}

impl PolyIndex {
    pub fn new(pts: Vec<usize>) -> Self {
        Self { idx_buf: pts }
    }

    pub fn to_poly(&self, vtx_buf: Rc<Vec<Vec3>>) -> Poly {
        Poly::new(self.idx_buf.iter().map(|idx| vtx_buf[*idx]).collect())
    }

    pub fn to_indexed_poly(&self, vtx_buf: Rc<Vec<Vec3>>) -> IndexedPoly {
        let idx_buf = self.idx_buf.clone();
        IndexedPoly::new(vtx_buf, idx_buf)
    }
}
