#![allow(dead_code)]

use crate::{aabb, prelude::*};
use std::collections::btree_set::Intersection;
use std::rc::Rc;
use std::sync::Arc;

pub mod prelude {
    pub use super::BVHNode;
    pub use super::BVHNodeIntersectionResult;
    pub use super::BVHSubdivideConfig;
}

#[derive(Copy, Clone, Debug)]
pub struct BVHSubdivideConfig {
    pub num_tris_per_leaf: usize,
    pub max_tris_per_leaf: usize,
}

impl BVHSubdivideConfig {
    fn can_subsubdivide(&self, bvh: &BVHNode) -> bool {
        bvh.idx_buf.len() > self.num_tris_per_leaf
    }

    fn is_valid(&self, bvh: &BVHNode) -> bool {
        let len = bvh.idx_buf.len();
        !(len < self.num_tris_per_leaf || len > self.max_tris_per_leaf)
    }
}

impl Default for BVHSubdivideConfig {
    fn default() -> Self {
        Self {
            num_tris_per_leaf: 4,
            max_tris_per_leaf: 15,
        }
    }
}

#[derive(Clone)]
pub struct BVHNode {
    pub vtx_buf: Rc<Vec<Vec3>>,
    pub idx_buf: Vec<TriIndex>,
    pub aabb: AABB,
    pub children: Vec<Rc<BVHNode>>,
}

pub enum BVHNodeIntersectionResult {
    One(Rc<BVHNode>),
    Multiple(Vec<Rc<BVHNode>>),
    Zero,
}

impl BVHNodeIntersectionResult {
    pub fn to_leaves(res: Self) -> Vec<Rc<BVHNode>> {
        match res {
            BVHNodeIntersectionResult::Zero => vec![],
            BVHNodeIntersectionResult::One(ele) => vec![ele],
            BVHNodeIntersectionResult::Multiple(eles) => eles,
        }
    }
}

impl BVHNode {
    pub fn get_interseced_leaves(bvh: Rc<Self>, aabb: &AABB) -> BVHNodeIntersectionResult {
        if bvh.aabb.intersect_with_aabb(aabb) {
            if bvh.is_leaf() {
                return BVHNodeIntersectionResult::One(bvh);
            } else {
                let mut ret = vec![];
                for child in bvh.children.iter() {
                    let intersection = Self::get_interseced_leaves(child.clone(), aabb);
                    match intersection {
                        BVHNodeIntersectionResult::One(ele) => {
                            ret.push(ele);
                        }
                        BVHNodeIntersectionResult::Multiple(eles) => {
                            ret.extend(eles);
                        }
                        _ => {}
                    }
                }
                return match ret.len() {
                    0 => BVHNodeIntersectionResult::Zero,
                    1 => BVHNodeIntersectionResult::One(ret[0].clone()),
                    _ => BVHNodeIntersectionResult::Multiple(ret),
                };
            }
        }
        BVHNodeIntersectionResult::Zero
    }

    pub fn get_all_nodes(bvh: Rc<Self>) -> Vec<Rc<Self>> {
        let mut ret = Vec::<Rc<Self>>::new();
        let mut ptr_stack = vec![bvh];
        while !ptr_stack.is_empty() {
            while let Some(ptr) = ptr_stack.pop() {
                for child in ptr.children.iter() {
                    ptr_stack.push(child.clone());
                }
                ret.push(ptr.clone());
            }
        }
        ret
    }

    pub fn get_all_leaves(bvh: Rc<Self>) -> Vec<Rc<Self>> {
        let mut ret = Vec::<Rc<Self>>::new();
        let mut ptr_stack = vec![bvh];
        while !ptr_stack.is_empty() {
            while let Some(ptr) = ptr_stack.pop() {
                for child in ptr.children.iter() {
                    ptr_stack.push(child.clone());
                }
                if ptr.is_leaf() {
                    ret.push(ptr.clone());
                }
            }
        }
        ret
    }

    pub fn directional_hit(
        bvh: Rc<BVHNode>,
        block_size: &Vec3,
        start: &Vec3,
        end: &Vec3,
        step_into: f64,
        break_on_hit: bool,
    ) -> usize {
        let mut local_peak = 0_usize;
        let mut local_pos = *start;
        let half_aabb_size = *block_size / Vec3::new(2.0, 2.0, 2.0);
        let dist = end.distance_to(start);
        let dir = start.direction_to(end);
        let nchunks = (dist / step_into).ceil() as usize;
        for _a in 0..nchunks {
            let min = local_pos - half_aabb_size;
            let max = local_pos + half_aabb_size;
            let aabb = AABB::new(&min, &max);
            let intersection = Self::get_interseced_leaves(bvh.clone(), &aabb);
            let leaves = BVHNodeIntersectionResult::to_leaves(intersection);
            local_peak = local_peak.max(leaves.len());
            if break_on_hit {
                break;
            }
            local_pos.move_towards(&dir, step_into);
        }
        local_peak
    }

    pub fn block_overlap_peak(bvh: Rc<Self>, step: f64) -> usize {
        // 开始坐标向外括了半格
        // 再加上directional_hit以该点为中心，内外各半格，刚好向外括了一格
        let halfstep = step / 2.0;
        let mut local_aabb = bvh.aabb.clone();
        local_aabb.expand(&Vec3::new(step, step, step));
        let mut peak = 0_usize;
        let mut curx = local_aabb.min;
        while curx.x < local_aabb.max.x {
            let mut cury = curx;
            while cury.y < local_aabb.max.y {
                let mut point_start = cury;
                let mut point_end = cury;
                point_start.z = bvh.aabb.min.z;
                point_end.z = bvh.aabb.max.z;
                let local_peak = Self::directional_hit(
                    bvh.clone(),
                    &Vec3::new(step, step, step),
                    &point_start,
                    &point_end,
                    halfstep,
                    false,
                );
                peak = peak.max(local_peak);
                cury.y += halfstep;
            }
            curx.x += halfstep;
        }
        peak
    }

    pub fn surface_hit_peak(bvh: Rc<Self>, step: f64, block_size: &Vec3) -> usize {
        enum Axis {
            X,
            Y,
            Z,
        }

        let axis_planar_hit =
            |axis: Axis, bvh: Rc<BVHNode>, block_size: &Vec3, step_into: f64| -> usize {
                // 开始坐标向外括了半格
                // 再加上directional_hit以该点为中心，内外各半格，刚好向外括了一格
                let half_block_size = *block_size / Vec3::new(2.0, 2.0, 2.0);
                let mut local_aabb = bvh.aabb.clone();
                local_aabb.expand(block_size);
                let ext = bvh.aabb.extent();
                let start = local_aabb.min;
                let end = local_aabb.max;
                let mut peak = 0_usize;
                let mut point1 = start;
                while match axis {
                    Axis::X => point1.y < end.y,
                    Axis::Y => point1.z < end.z,
                    Axis::Z => point1.x < end.x,
                } {
                    let mut point2 = point1;
                    while match axis {
                        Axis::X => point2.z < end.z,
                        Axis::Y => point2.x < end.x,
                        Axis::Z => point2.y < end.y,
                    } {
                        let mut point2_back = point2;
                        match axis {
                            Axis::X => {
                                point2_back.x += ext.x;
                            }
                            Axis::Y => {
                                point2_back.y += ext.y;
                            }
                            Axis::Z => {
                                point2_back.z += ext.z;
                            }
                        }
                        peak = peak.max(Self::directional_hit(
                            bvh.clone(),
                            block_size,
                            &point2,
                            &point2_back,
                            step_into,
                            true,
                        ));
                        peak = peak.max(Self::directional_hit(
                            bvh.clone(),
                            block_size,
                            &point2_back,
                            &point2,
                            step_into,
                            true,
                        ));
                        match axis {
                            Axis::X => {
                                point2.z += half_block_size.z;
                            }
                            Axis::Y => {
                                point2.x += half_block_size.x;
                            }
                            Axis::Z => {
                                point2.y += half_block_size.y;
                            }
                        }
                    }
                    match axis {
                        Axis::X => {
                            point1.y += half_block_size.y;
                        }
                        Axis::Y => {
                            point1.z += half_block_size.z;
                        }
                        Axis::Z => {
                            point1.x += half_block_size.x;
                        }
                    }
                }
                peak
            };

        let mut peak = 0_usize;

        peak = peak.max(axis_planar_hit(Axis::X, bvh.clone(), block_size, step));

        peak = peak.max(axis_planar_hit(Axis::Y, bvh.clone(), block_size, step));

        peak = peak.max(axis_planar_hit(Axis::Z, bvh.clone(), block_size, step));

        peak
    }

    pub fn new(vtx_buf: Rc<Vec<Vec3>>, idx_buf: Vec<TriIndex>) -> Self {
        let mut ret = Self {
            vtx_buf,
            idx_buf,
            aabb: AABB::default(),
            children: Vec::<Rc<BVHNode>>::new(),
        };
        ret.recalc_aabb();
        ret
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    pub fn recalc_aabb(&mut self) {
        let mut points = Vec::<Vec3>::new();
        for tidx in self.idx_buf.iter() {
            let tri = tidx.to_tri(self.vtx_buf.clone());
            points.push(tri.pt0);
            points.push(tri.pt1);
            points.push(tri.pt2);
        }
        self.aabb = AABB::from_points(&points);
        let extent = self.aabb.extent();
        let dist = Vec3::new(
            if extent.x < 0.1 { 0.1 } else { 0.0 },
            if extent.y < 0.1 { 0.1 } else { 0.0 },
            if extent.z < 0.1 { 0.1 } else { 0.0 },
        );
        self.aabb.expand(&dist);
    }

    pub fn subdivide(&mut self, cfg: BVHSubdivideConfig) {
        if cfg.can_subsubdivide(self) {
            let mut valid = true;

            // 先从最大的个轴上一分为二
            {
                let axis = self.aabb.largest_axis();
                let (pos_tri_idx, neg_tri_idx) = local_split(self, axis);
                if pos_tri_idx.is_empty() || pos_tri_idx.len() == self.idx_buf.len() {
                    valid = false;
                }
                if valid {
                    let mut child_pos = BVHNode::new(self.vtx_buf.clone(), pos_tri_idx);
                    child_pos.subdivide(cfg);
                    let mut child_neg = BVHNode::new(self.vtx_buf.clone(), neg_tri_idx);
                    child_neg.subdivide(cfg);
                    self.children.push(Rc::new(child_pos));
                    self.children.push(Rc::new(child_neg));
                }
            }

            // 取最平均的轴
            if !valid {
                let mut postris = [0.0f64, 0.0f64, 0.0f64];
                let axises = [AABBSplitAxis::X, AABBSplitAxis::Y, AABBSplitAxis::Z];
                for (idx, axis) in axises.into_iter().enumerate() {
                    let (pos_tri_idx, _) = local_split(self, axis);
                    postris[idx] = (pos_tri_idx.len() as f64) / (self.idx_buf.len() as f64);
                }
                postris[0] -= 0.5;
                postris[0] *= postris[0];
                postris[1] -= 0.5;
                postris[1] *= postris[1];
                postris[2] -= 0.5;
                postris[2] *= postris[2];
                let mut minidx = 0;
                if postris[1] < postris[minidx] {
                    minidx = 1;
                }
                if postris[2] < postris[minidx] {
                    minidx = 2;
                }

                let axis = [AABBSplitAxis::X, AABBSplitAxis::Y, AABBSplitAxis::Z][minidx];
                let (pos_tri_idx, neg_tri_idx) = local_split(self, axis);
                if pos_tri_idx.is_empty() || pos_tri_idx.len() == self.idx_buf.len() {
                    valid = false;
                }
                if valid {
                    let mut child_pos = BVHNode::new(self.vtx_buf.clone(), pos_tri_idx);
                    let mut child_neg = BVHNode::new(self.vtx_buf.clone(), neg_tri_idx);
                    child_pos.subdivide(cfg);
                    child_neg.subdivide(cfg);
                    self.children.push(Rc::new(child_pos));
                    self.children.push(Rc::new(child_neg));
                }
            }

            // 还不行就对半分
            if !valid {
                let tot = self.idx_buf.len();
                let sep = tot / 2;
                let mut pos_tri_idx = Vec::<TriIndex>::new();
                let mut neg_tri_idx = Vec::<TriIndex>::new();
                for (counter, idx) in self.idx_buf.iter().enumerate() {
                    if counter <= sep {
                        pos_tri_idx.push(idx.clone());
                    } else {
                        neg_tri_idx.push(idx.clone());
                    }
                }
                let mut child_pos = BVHNode::new(self.vtx_buf.clone(), pos_tri_idx);
                let mut child_neg = BVHNode::new(self.vtx_buf.clone(), neg_tri_idx);
                child_pos.subdivide(cfg);
                child_neg.subdivide(cfg);
                self.children.push(Rc::new(child_pos));
                self.children.push(Rc::new(child_neg));
            }
        }
    }

    #[cfg(debug_assertions)]
    pub fn print(&self, depth: usize) {
        let mut text = String::new();
        for _ in 0..depth {
            text.push(' ');
        }
        let buf = format!(
            "BVHNode: {} children and {} tris. Extent: {:?}",
            self.children.len(),
            self.idx_buf.len(),
            self.aabb.extent(),
        );
        text.push_str(buf.as_str());
        println!("{}", text);
        for child in self.children.iter() {
            child.print(depth + 1);
        }
    }

    #[cfg(debug_assertions)]
    pub fn print_leaves(&self) {
        if self.is_leaf() {
            let mut text = String::new();
            let buf = format!(
                "BVHNode: {} children and {} tris. Extent: {:?}",
                self.children.len(),
                self.idx_buf.len(),
                self.aabb.extent(),
            );
            text.push_str(buf.as_str());
            println!("{}", text);
        }
        for child in self.children.iter() {
            child.print_leaves();
        }
    }
}

fn local_split(bvh: &BVHNode, axis: AABBSplitAxis) -> (Vec<TriIndex>, Vec<TriIndex>) {
    let mut pos_tri_idx = Vec::<TriIndex>::new();
    let mut neg_tri_idx = Vec::<TriIndex>::new();
    let (pos_aabb, neg_aabb) = bvh.aabb.split(axis);
    for tri_index in bvh.idx_buf.iter() {
        let tri = tri_index.to_tri(bvh.vtx_buf.clone());
        let tri_aabb = AABB::from_point3(&tri.pt0, &tri.pt1, &tri.pt2);

        // 一定不要让一个三角形同时属于两个aabb
        // 这样的话会导致 pos_tri_idx 和 neg_tri_idx的tri数量不是互补的
        // 那么subdivide每一轮的valid都是true，于是会导致无限递归
        if pos_aabb.intersect_with_aabb(&tri_aabb) {
            pos_tri_idx.push(tri_index.clone());
        } else {
            neg_tri_idx.push(tri_index.clone());
        }
    }
    (pos_tri_idx, neg_tri_idx)
}
