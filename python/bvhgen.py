#encoding=utf8
import ctypes
import os


PATH = os.path.join(os.path.dirname(__file__), "..")


class PyVec3(ctypes.Structure):
    _fields_ = [
        ("x", ctypes.c_double),
        ("y", ctypes.c_double),
        ("z", ctypes.c_double),
    ]

    def __repr__(self):
        return "<Vec3 {} {} {}>".format(self.x, self.y, self.z)


class PyBVHInfo(ctypes.Structure):
    _fields_ = [
        ("center", PyVec3),
        ("extent", PyVec3),
        ("ntris", ctypes.c_longlong),
    ]

    def __repr__(self):
        return "<BVHInfo center: {} extent: {} ntris: {}>".format(
            self.center,
            self.extent,
            self.ntris,
            )


dllpath = os.path.abspath(os.path.join(PATH, "target/release/bvhgen.dll"))
dll = ctypes.cdll.LoadLibrary(dllpath)

_BVHBuildInfo_create = dll.BVHBuildInfo_create
_BVHBuildInfo_create.restype = ctypes.c_longlong
_BVHBuildInfo_create.argtypes = (ctypes.POINTER(ctypes.c_double), ctypes.c_longlong)

_BVHBuildInfo_delete = dll.BVHBuildInfo_delete
_BVHBuildInfo_delete.restype = ctypes.c_longlong
_BVHBuildInfo_delete.argtypes = (ctypes.c_longlong,)

_BVHBuildInfo_add_poly_index = dll.BVHBuildInfo_add_poly_index
_BVHBuildInfo_add_poly_index.restype = ctypes.c_longlong
_BVHBuildInfo_add_poly_index.argtypes = (ctypes.c_longlong, ctypes.POINTER(ctypes.c_longlong), ctypes.c_longlong)

_BVHBuildInfo_generate_tri_buf = dll.BVHBuildInfo_generate_tri_buf
_BVHBuildInfo_generate_tri_buf.restype = ctypes.c_longlong
_BVHBuildInfo_generate_tri_buf.argtypes = (ctypes.c_longlong,)

_BVHBuildInfo_generate_bvh = dll.BVHBuildInfo_generate_bvh
_BVHBuildInfo_generate_bvh.restype = ctypes.c_longlong
_BVHBuildInfo_generate_bvh.argtypes = (ctypes.c_longlong,)

_BVHBuildInfo_get_leaf_count = dll.BVHBuildInfo_get_leaf_count
_BVHBuildInfo_get_leaf_count.restype = ctypes.c_longlong
_BVHBuildInfo_get_leaf_count.argtypes = (ctypes.c_longlong,)

_BVHBuildInfo_get_leaves = dll.BVHBuildInfo_get_leaves
_BVHBuildInfo_get_leaves.restype = ctypes.c_longlong
_BVHBuildInfo_get_leaves.argtypes = (ctypes.c_longlong, ctypes.POINTER(PyBVHInfo), ctypes.c_longlong)

_BVHBuildInfo_get_block_overlap_peak = dll.BVHBuildInfo_get_block_overlap_peak
_BVHBuildInfo_get_block_overlap_peak.restype = ctypes.c_longlong
_BVHBuildInfo_get_block_overlap_peak.argtypes = (ctypes.c_longlong, ctypes.c_double)

_BVHBuildInfo_get_surface_hit_peak = dll.BVHBuildInfo_get_surface_hit_peak
_BVHBuildInfo_get_surface_hit_peak.restype = ctypes.c_longlong
_BVHBuildInfo_get_surface_hit_peak.argtypes = (ctypes.c_longlong, ctypes.c_double, ctypes.c_double, ctypes.c_double, ctypes.c_double)


class BVHBuildInfo:


    class BVHBuildExc_OutOfResource(RuntimeError):pass
    class BVHBuildExc_ResourceNotFound(RuntimeError):pass
    class BVHBuildExc_IdxBufIsEmpty(RuntimeError):pass
    class BVHBuildExc_IdxOutOfVtxCnt(RuntimeError):pass
    class BVHBuildExc_TriBufNotGenerated(RuntimeError):pass
    class BVHBuildExc_BVHNotGenerated(RuntimeError):pass


    @classmethod
    def checkexc(cls, code):
        if code < 0:
            lut = (
                cls.BVHBuildExc_OutOfResource,
                cls.BVHBuildExc_ResourceNotFound,
                cls.BVHBuildExc_IdxBufIsEmpty,
                cls.BVHBuildExc_IdxOutOfVtxCnt,
                cls.BVHBuildExc_TriBufNotGenerated,
                cls.BVHBuildExc_BVHNotGenerated,
            )
            absv = abs(code) - 1
            raise lut[absv]


    def __init__(self, vertices):
        cnt = len(vertices)
        t = ctypes.c_double * (cnt * 3)
        data = []
        for vtx in vertices:
            x, y, z = vtx
            data.append(x)
            data.append(y)
            data.append(z)
        arr = t(*data)
        self.bvhid = _BVHBuildInfo_create(arr, cnt)
        self.__class__.checkexc(self.bvhid)


    def __del__(self):
        if self.bvhid >= 0:
            ret = _BVHBuildInfo_delete(self.bvhid)
            self.__class__.checkexc(ret)


    def add_poly_index(self, indices):
        cnt = len(indices)
        t = ctypes.c_longlong * cnt
        arr = t(*indices)
        ret = _BVHBuildInfo_add_poly_index(self.bvhid, arr, cnt)
        self.__class__.checkexc(ret)


    def build(self):
        ret = _BVHBuildInfo_generate_tri_buf(self.bvhid)
        self.__class__.checkexc(ret)
        ret = _BVHBuildInfo_generate_bvh(self.bvhid)
        self.__class__.checkexc(ret)


    def get_bvh_leaves(self):
        cnt = _BVHBuildInfo_get_leaf_count(self.bvhid)
        self.__class__.checkexc(cnt)
        t = PyBVHInfo * cnt
        arr = t()
        ret = _BVHBuildInfo_get_leaves(self.bvhid, arr, cnt)
        self.__class__.checkexc(ret)
        return [ele for ele in arr]


    def get_bvh_block_overlap_peak(self, block_size):
        return _BVHBuildInfo_get_block_overlap_peak(self.bvhid, block_size)


    def get_bvh_surface_hit_peak(self, step, block_size):
        x, y, z = block_size
        return _BVHBuildInfo_get_surface_hit_peak(self.bvhid, step, x, y, z)


if __name__ == "__main__":

    try:
        # 在Blender中运行
        import bpy
        obj = bpy.context.object
        if obj.type == 'MESH':
            mesh = obj.data
            vertices = [(vertex.co[0] * 100.0, vertex.co[1] * 100.0, vertex.co[2] * 100.0) for vertex in mesh.vertices]
            indices = [poly.vertices for poly in mesh.polygons]
            bbi = BVHBuildInfo(vertices)
            for idxs in indices:
                bbi.add_poly_index(idxs)    
            bbi.build()
            allbvh = bbi.get_bvh_leaves()
            overlap_peak = bbi.get_bvh_block_overlap_peak(30.0)
            surface_hit_peak = bbi.get_bvh_surface_hit_peak(30.0, (30.0, 30.0, 30.0))
            print("Num BVH Leaves: {}\nBlock Overlap Peak: {}\nSurface Hit Peak: {}\n".format(
                len(allbvh),
                overlap_peak,
                surface_hit_peak,
                ))
            del bbi
            if False:
                for bvh in allbvh:
                    center = [bvh.center.x / 100.0, bvh.center.y / 100.0, bvh.center.z / 100.0]
                    scale = [bvh.extent.x / 200.0, bvh.extent.y / 200.0, bvh.extent.z / 200.0]
                    bpy.ops.mesh.primitive_cube_add(
                        enter_editmode=False,
                        align='WORLD',
                        location=center,
                        scale=scale,
                        )


    except ImportError as e:
        import json
        from pprint import pprint
        sample_path = os.path.join(PATH, "python", "sample.json")
        with open(sample_path, "r") as f:
            data = json.load(f)
            vertices = data["vertices"]
            indices = data["polygon"]
            bbi = BVHBuildInfo(vertices)
            for idxs in indices:
                bbi.add_poly_index(idxs)
            bbi.build()
            pprint(bbi.get_bvh_leaves())
