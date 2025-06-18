use crate::prelude::*;

use std::os::raw::{c_double, c_longlong};
use std::rc::Rc;

type PyFloat = c_double;
type PyInt = c_longlong;

#[repr(i64)]
enum PyResult {
    Good = 0,
    OutOfResource = -1,
    ResourceNotFound = -2,
    IdxBufIsEmpty = -3,
    IdxOutOfVtxCnt = -4,
    TriBufNotGenerated = -5,
    BVHNotGenerated = -6,
}

#[repr(C)]
pub struct PyVec3 {
    pub x: PyFloat,
    pub y: PyFloat,
    pub z: PyFloat,
}

#[repr(C)]
pub struct PyBVHInfo {
    pub center: PyVec3,
    pub extent: PyVec3,
    pub ntris: PyInt,
}

struct BVHBuildInfo {
    vtx_buf: Rc<Vec<Vec3>>,
    idx_buf: Vec<IndexedPoly>,
    tri_buf: Vec<IndexedTri>,
    bvh: Option<Rc<BVHNode>>,
}

const NUM_BVH_BUILD_RESOUCE: usize = 8;
static mut BVH_BUILD_RESOURCE: [Option<BVHBuildInfo>; NUM_BVH_BUILD_RESOUCE] =
    [const { None }; NUM_BVH_BUILD_RESOUCE];

impl BVHBuildInfo {
    fn new(vtx_buf: Rc<Vec<Vec3>>) -> Self {
        Self {
            vtx_buf,
            idx_buf: Vec::<IndexedPoly>::new(),
            tri_buf: Vec::<IndexedTri>::new(),
            bvh: None,
        }
    }

    fn alloc(vtx_buf: Rc<Vec<Vec3>>) -> i64 {
        unsafe {
            for id in 0..NUM_BVH_BUILD_RESOUCE {
                let r = &mut BVH_BUILD_RESOURCE[id];
                if r.is_none() {
                    let bbi = Self::new(vtx_buf);
                    BVH_BUILD_RESOURCE[id] = Some(bbi);
                    return id as i64;
                }
            }
        }
        PyResult::OutOfResource as i64
    }

    fn dealloc(id: i64) -> i64 {
        if (id as usize) >= NUM_BVH_BUILD_RESOUCE {
            return PyResult::ResourceNotFound as i64;
        }
        unsafe {
            let rc = &BVH_BUILD_RESOURCE[id as usize];
            if rc.is_none() {
                return PyResult::ResourceNotFound as i64;
            } else {
                BVH_BUILD_RESOURCE[id as usize] = None;
            }
        }
        PyResult::Good as i64
    }

    fn add_poly_index(id: i64, pidx: Vec<usize>) -> i64 {
        if (id as usize) >= NUM_BVH_BUILD_RESOUCE {
            return PyResult::ResourceNotFound as i64;
        }
        unsafe {
            if let Some(ref mut rc) = BVH_BUILD_RESOURCE[id as usize] {
                for idx in pidx.iter() {
                    if *idx >= rc.vtx_buf.len() {
                        return PyResult::IdxOutOfVtxCnt as i64;
                    }
                }
                let ipoly = IndexedPoly::new(rc.vtx_buf.clone(), pidx);
                rc.idx_buf.push(ipoly);
            } else {
                return PyResult::ResourceNotFound as i64;
            }
        }
        PyResult::Good as i64
    }

    fn generate_tri_buf(id: i64) -> i64 {
        if (id as usize) >= NUM_BVH_BUILD_RESOUCE {
            return PyResult::ResourceNotFound as i64;
        }
        unsafe {
            if let Some(ref mut rc) = BVH_BUILD_RESOURCE[id as usize] {
                if rc.idx_buf.is_empty() {
                    return PyResult::IdxBufIsEmpty as i64;
                }
                rc.tri_buf.clear();
                for ipoly in rc.idx_buf.iter() {
                    let itri = ipoly.to_indexed_tri();
                    rc.tri_buf.extend(itri);
                }
            } else {
                return PyResult::ResourceNotFound as i64;
            }
        }
        PyResult::Good as i64
    }

    fn generate_bvh(id: i64) -> i64 {
        if (id as usize) >= NUM_BVH_BUILD_RESOUCE {
            return PyResult::ResourceNotFound as i64;
        }
        unsafe {
            if let Some(ref mut rc) = BVH_BUILD_RESOURCE[id as usize] {
                if rc.idx_buf.is_empty() {
                    return PyResult::IdxBufIsEmpty as i64;
                }
                if rc.tri_buf.is_empty() {
                    return PyResult::TriBufNotGenerated as i64;
                }
                let tri_index = rc
                    .tri_buf
                    .iter()
                    .map(|itri| TriIndex::new(itri.indices[0], itri.indices[1], itri.indices[2]))
                    .collect();
                let mut bvh = BVHNode::new(rc.vtx_buf.clone(), tri_index);
                bvh.subdivide(BVHSubdivideConfig::default());
                rc.bvh = Some(Rc::new(bvh));
            } else {
                return PyResult::ResourceNotFound as i64;
            }
        }
        PyResult::Good as i64
    }

    fn get_leaves(id: i64, leaves: &mut Vec<Rc<BVHNode>>) -> i64 {
        if (id as usize) >= NUM_BVH_BUILD_RESOUCE {
            return PyResult::ResourceNotFound as i64;
        }
        unsafe {
            if let Some(ref rc) = BVH_BUILD_RESOURCE[id as usize] {
                if let Some(ref bvh) = rc.bvh {
                    let mut stack = vec![bvh.clone()];
                    while let Some(node) = stack.pop() {
                        stack.extend(node.children.iter().map(|ptr| ptr.clone()));
                        if node.is_leaf() {
                            leaves.push(node);
                        }
                    }
                } else {
                    return PyResult::BVHNotGenerated as i64;
                }
            } else {
                return PyResult::ResourceNotFound as i64;
            }
        }
        leaves.len() as i64
    }

    fn get_leaf_count(id: i64) -> i64 {
        let mut leaves = Vec::<Rc<BVHNode>>::new();
        Self::get_leaves(id, &mut leaves)
    }

    fn get_block_overlap_peak(id: i64, block_size: f64) -> i64 {
        unsafe {
            if let Some(ref rc) = BVH_BUILD_RESOURCE[id as usize] {
                if let Some(ref bvh) = rc.bvh {
                    BVHNode::block_overlap_peak(bvh.clone(), block_size) as i64
                } else {
                    PyResult::BVHNotGenerated as i64
                }
            } else {
                PyResult::ResourceNotFound as i64
            }
        }
    }

    fn get_surface_hit_peak(id: i64, step: f64, block_size: &Vec3) -> i64 {
        unsafe {
            if let Some(ref rc) = BVH_BUILD_RESOURCE[id as usize] {
                if let Some(ref bvh) = rc.bvh {
                    BVHNode::surface_hit_peak(bvh.clone(), step, block_size) as i64
                } else {
                    PyResult::BVHNotGenerated as i64
                }
            } else {
                PyResult::ResourceNotFound as i64
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn BVHBuildInfo_create(verts: *const PyFloat, nverts: PyInt) -> PyInt {
    unsafe {
        let data = std::slice::from_raw_parts(verts, (nverts * 3) as usize);
        let mut vtx_buf = Vec::<Vec3>::new();
        for i in 0..nverts {
            let idx = (i * 3) as usize;
            let pt0 = data[idx];
            let pt1 = data[idx + 1];
            let pt2 = data[idx + 2];
            vtx_buf.push(Vec3::new(pt0, pt1, pt2));
        }
        let id = BVHBuildInfo::alloc(Rc::new(vtx_buf));
        id as PyInt
    }
}

#[no_mangle]
pub extern "C" fn BVHBuildInfo_delete(id: PyInt) -> PyInt {
    BVHBuildInfo::dealloc(id)
}

#[no_mangle]
pub extern "C" fn BVHBuildInfo_add_poly_index(
    id: PyInt,
    indices: *const PyInt,
    nverts: PyInt,
) -> PyInt {
    unsafe {
        let data = std::slice::from_raw_parts(indices, nverts as usize)
            .iter()
            .map(|idx| *idx as usize)
            .collect::<Vec<usize>>();
        BVHBuildInfo::add_poly_index(id, data)
    }
}

#[no_mangle]
pub extern "C" fn BVHBuildInfo_generate_tri_buf(id: PyInt) -> PyInt {
    BVHBuildInfo::generate_tri_buf(id)
}

#[no_mangle]
pub extern "C" fn BVHBuildInfo_generate_bvh(id: PyInt) -> PyInt {
    BVHBuildInfo::generate_bvh(id)
}

#[no_mangle]
pub extern "C" fn BVHBuildInfo_get_leaf_count(id: PyInt) -> PyInt {
    BVHBuildInfo::get_leaf_count(id)
}

#[no_mangle]
pub extern "C" fn BVHBuildInfo_get_leaves(id: PyInt, buf: *mut PyBVHInfo, buflen: PyInt) -> PyInt {
    let mut leaves = Vec::<Rc<BVHNode>>::new();
    let nleaves = BVHBuildInfo::get_leaves(id, &mut leaves);
    if nleaves < 0 {
        return nleaves as PyInt;
    }
    for (idx, leaf) in leaves.iter().enumerate() {
        if idx >= (buflen as usize) {
            break;
        }
        let center = leaf.aabb.center();
        let extent = leaf.aabb.extent();
        let pybi = PyBVHInfo {
            center: PyVec3 {
                x: center.x,
                y: center.y,
                z: center.z,
            },
            extent: PyVec3 {
                x: extent.x,
                y: extent.y,
                z: extent.z,
            },
            ntris: leaf.idx_buf.len() as PyInt,
        };
        unsafe {
            std::ptr::write(buf.wrapping_add(idx), pybi);
        }
    }
    PyResult::Good as PyInt
}

#[no_mangle]
pub extern "C" fn BVHBuildInfo_get_block_overlap_peak(id: PyInt, block_size: PyFloat) -> PyInt {
    BVHBuildInfo::get_block_overlap_peak(id, block_size)
}

#[no_mangle]
pub extern "C" fn BVHBuildInfo_get_surface_hit_peak(
    id: PyInt,
    step: PyFloat,
    block_size_x: PyFloat,
    block_size_y: PyFloat,
    block_size_z: PyFloat,
) -> PyInt {
    BVHBuildInfo::get_surface_hit_peak(
        id,
        step,
        &Vec3::new(block_size_x, block_size_y, block_size_z),
    )
}
