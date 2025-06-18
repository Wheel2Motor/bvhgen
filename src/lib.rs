#![allow(dead_code)]
#![allow(unused_imports)]

mod aabb;
mod bvh;
mod cexport;
mod poly;
mod tri;
mod vec3;

pub mod prelude {
    pub use super::aabb::prelude::*;
    pub use super::bvh::prelude::*;
    pub use super::poly::prelude::*;
    pub use super::tri::prelude::*;
    pub use super::vec3::prelude::*;
}

#[cfg(test)]
mod tests {
    use std::ops::Index;

    #[test]
    fn test_bvh_subdivide() {
        use super::prelude::*;
        use std::rc::Rc;

        let mut vtx_buf = Vec::<Vec3>::new();
        let mut idx_buf = Vec::<TriIndex>::new();
        for (idx, _) in (0..(10000 * 9)).enumerate() {
            let tri_idx = TriIndex::new(idx * 3, idx * 3 + 1, idx * 3 + 2);
            let x0 = rand::random_range(-100.0f64..100.0f64);
            let y0 = rand::random_range(-100.0f64..100.0f64);
            let z0 = rand::random_range(-100.0f64..100.0f64);
            let x1 = x0 + rand::random_range(-20.0..20.0);
            let y1 = y0 + rand::random_range(-20.0..20.0);
            let z1 = z0 + rand::random_range(-20.0..20.0);
            let x2 = x0 + rand::random_range(-20.0..20.0);
            let y2 = y0 + rand::random_range(-20.0..20.0);
            let z2 = z0 + rand::random_range(-20.0..20.0);
            vtx_buf.push(Vec3::new(x0, y0, z0));
            vtx_buf.push(Vec3::new(x1, y1, z1));
            vtx_buf.push(Vec3::new(x2, y2, z2));
            idx_buf.push(tri_idx);
        }
        let mut bvh = BVHNode::new(Rc::new(vtx_buf), idx_buf);
        {
            bvh.subdivide(BVHSubdivideConfig::default());
            //bvh.print_leaves();
        }
        let rbvh = Rc::new(bvh);
        let aabb = AABB::new(
            &Vec3::new(-20.0, -20.0, -20.0),
            &Vec3::new(20.0, 20.0, 20.0),
        );
        let intersection = BVHNode::get_interseced_leaves(rbvh.clone(), &aabb);
        println!(
            "Intersected leaves: {}",
            BVHNodeIntersectionResult::to_leaves(intersection).len()
        );
    }

    #[test]
    fn test_trireduce() {
        use super::prelude::*;
        use std::rc::Rc;

        let vtx_buf = Rc::new(vec![
            Vec3::new(-1.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(2.0, 0.0, 1.0),
            Vec3::new(1.0, -1.0, 1.0),
            Vec3::new(-1.0, -1.0, 1.0),
        ]);
        let idx_buf = vec![0_usize, 1_usize, 2_usize, 3_usize, 4_usize];
        let ipoly = IndexedPoly::new(vtx_buf, idx_buf);
        let tris = ipoly.to_indexed_tri();
        println!("{:#?}", tris);
    }
}
