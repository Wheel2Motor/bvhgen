#pragma once
#ifndef _BVHGEN_H_
#define _BVHGEN_H_

#include <stdlib.h>
#include <stdint.h>

#define RESULT_Good               (0)
#define RESULT_OutOfResource      (-1)
#define RESULT_ResourceNotFound   (-2)
#define RESULT_IdxBufIsEmpty      (-3)
#define RESULT_IdxOutOfVtxCnt     (-4)
#define RESULT_TriBufNotGenerated (-5)
#define RESULT_BVHNotGenerated    (-6)
#define IS_RESULT_GOOD(res)       ((res) >= RESULT_Good)

typedef double PyFloat;
typedef int64_t PyInt;
typedef PyInt Result;
typedef PyInt ID;


/*
 * Allocate BVH resource with given vertex data.
 * ** The max resource count in static buffer is 8 **
 * this means that you can't allocate more than 8 times without * BVHBuildInfo_delete * .
 * @vtxbuf: Vertex buffer.
 * @n: Vertex buffer length, n equals 3 times of vertex count.
 * RETURN: Returns resource ID(means handle), you do other operation through this ID.
 *         Use `if (IS_RESULT_GOOD(result))`.
 */
extern ID
BVHBuildInfo_create(PyFloat * vtxbuf, PyInt n);

/*
 * Release BVH resource with givent ID.
 * @id: Resource ID.
 * RETURN: Resource release result.
 *         Use `if (IS_RESULT_GOOD(result))`.
 */
extern Result
BVHBuildInfo_delete(ID id);

/*
 * Add one polygon into BVH resource. If the data you are using is Triangle,
 * just assume it as a Polygon made of only 3 verts.
 * @idxbuf: Index buffer.
 * @n: Index buffer length, n equals 3 times of triangle count.
 * RESULT: Returns add result, if result < 0, it means an error occours.
 *         Use `if (IS_RESULT_GOOD(result))`.
 */
extern Result
BVHBuildInfo_add_poly_index(ID id, PyInt * idxbuf, PyInt n);

/*
 * BVH resource will do triangle reduction internally.
 * RESULT: Returns add result, if result < 0, it means an error occours.
 *         Use `if (IS_RESULT_GOOD(result))`.
 */
extern Result
BVHBuildInfo_generate_tri_buf(ID id);

/*
 * BVH resource will generate BVH using generated tri buf.
 * RESULT: Returns add result, if result < 0, it means an error occours.
 *         Use `if (IS_RESULT_GOOD(result))`.
 */
extern Result
BVHBuildInfo_generate_bvh(ID id);

/*
 * BVH resource collision simulation profile peak.
 * The higher the value is, the worse performance the assets cause.
 * RESULT: Returns numeric value of assets profile.
 *         Use `if (IS_RESULT_GOOD(result))`.
 */
extern Result
BVHBuildInfo_get_surface_hit_peak(
	ID id,
	PyFloat step,
	PyFloat block_size_x,
	PyFloat block_size_y,
	PyFloat block_size_z);

#endif // _BVHGEN_H_
