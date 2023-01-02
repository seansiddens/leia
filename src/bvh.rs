use crate::hittable::*;
use crate::ray::*;
use crate::triangle::*;
use glam::*;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Bvh {
    root_index: usize,
    nodes: Vec<BvhNode>,
    nodes_used: usize,
    triangles: Vec<Triangle>, // TODO: Figure out how to have Bvh not own the triangle data itself.
    triangle_indices: Vec<usize>,
    num_triangles: usize,
}

#[derive(Debug, Clone, Copy)]
struct BvhNode {
    aabb_min: Vec3A,
    aabb_max: Vec3A,
    left_child: usize, // Right child is always left_child + 1
    first_prim: usize,
    prim_count: usize, // If non-zero, is a leaf. Must zero for interior nodes.
}

impl BvhNode {
    fn is_leaf(&self) -> bool {
        self.prim_count > 0
    }
}

fn intersect_aabb(r: &Ray, t_min: f32, t_max: f32, b_min: Vec3A, b_max: Vec3A) -> bool {
    let mut t_min = t_min;
    let mut t_max = t_max;
    // Comptue t-intervals along the x-axis
    let mut inv_d = 1.0 / r.direction().x;
    let mut t0 = (b_min.x - r.origin().x) * inv_d;
    let mut t1 = (b_max.x - r.origin().x) * inv_d;

    if inv_d < 0.0 {
        let tmp = t0;
        t0 = t1;
        t1 = tmp;
    }

    t_min = if t0 > t_min { t0 } else { t_min };
    t_max = if t1 < t_max { t1 } else { t_max };

    if t_max <= t_min {
        return false;
    }

    // Comptue t-intervals along the y-axis
    inv_d = 1.0 / r.direction().y;
    t0 = (b_min.y - r.origin().y) * inv_d;
    t1 = (b_max.y - r.origin().y) * inv_d;

    if inv_d < 0.0 {
        let tmp = t0;
        t0 = t1;
        t1 = tmp;
    }

    t_min = if t0 > t_min { t0 } else { t_min };
    t_max = if t1 < t_max { t1 } else { t_max };

    if t_max <= t_min {
        return false;
    }

    // Comptue t-intervals along the z-axis
    inv_d = 1.0 / r.direction().z;
    t0 = (b_min.z - r.origin().z) * inv_d;
    t1 = (b_max.z - r.origin().z) * inv_d;

    if inv_d < 0.0 {
        let tmp = t0;
        t0 = t1;
        t1 = tmp;
    }

    t_min = if t0 > t_min { t0 } else { t_min };
    t_max = if t1 < t_max { t1 } else { t_max };

    if t_max <= t_min {
        return false;
    }

    return true;
}

impl Bvh {
    pub fn new(triangles: &Vec<Triangle>) -> Self {
        let num_triangles = triangles.len();
        // Populate the triangle index vector.
        let mut triangle_indices = Vec::with_capacity(num_triangles);
        let mut tris = Vec::with_capacity(num_triangles);
        for i in 0..num_triangles {
            triangle_indices.push(i);
            // Copying triangle data.
            let verts = &triangles[i].vertices();
            tris.push(Triangle::new(
                verts[0],
                verts[1],
                verts[2],
                triangles[i].albedo(),
            ));
        }

        // Initialize the BvhNode pool.
        // Upper limit for a BVH with N triangles is 2N - 1
        let mut nodes = vec![
            BvhNode {
                aabb_min: vec3a(f32::INFINITY, f32::INFINITY, f32::INFINITY),
                aabb_max: vec3a(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
                left_child: 0,
                first_prim: 0,
                prim_count: 0,
            };
            2 * num_triangles - 1
        ];

        let root_index = 0;
        let nodes_used = 1;
        nodes[root_index].prim_count = num_triangles; // Root contains all primites.

        let mut bvh = Self {
            root_index,
            nodes,
            nodes_used,
            triangle_indices,
            triangles: tris,
            num_triangles,
        };

        bvh.update_node_bounds(bvh.root_index);

        // Recursively construct the bvh.
        bvh.subdivide(bvh.root_index);

        bvh
    }

    /// Recursively construct a BVH.
    fn subdivide(&mut self, node_index: usize) {
        let left_child;
        let left_count;
        let right_child;
        let mut i;
        let node_first_prim;
        let node_prim_count;
        // TODO: I don't need this block if I just access via node_index instead of getting a &mut to the node.
        {
            let node = &mut self.nodes[node_index];
            if node.prim_count <= 2 {
                // Reached parent of leaf nodes. Terminate recursion.
                return;
            }

            // Determine split axis and position.
            let extent = node.aabb_max - node.aabb_min;
            let mut axis = 0;
            if extent.y > extent.x {
                axis = 1;
            }
            if extent.z > extent[axis] {
                axis = 2;
            }

            // Split position is the middle of the extent along the split axis.
            let split_pos = node.aabb_min[axis] + extent[axis] * 0.5;

            // In-place partition.
            // Partition triangles based on the split.
            i = node.first_prim;
            let mut j = i + node.prim_count - 1;
            while i <= j {
                if self.triangles[self.triangle_indices[i]].centroid()[axis] < split_pos {
                    i += 1;
                } else {
                    // Swap with triangle at end.
                    self.triangle_indices.swap(i, j);
                    j -= 1;
                }
            }

            // Abort split if one of the sides is empty.
            left_count = i - node.first_prim;
            if left_count == 0 || left_count == node.prim_count {
                return;
            }

            // Create child nodes.
            left_child = self.nodes_used;
            self.nodes_used += 1;
            right_child = self.nodes_used;
            self.nodes_used += 1;

            node.left_child = left_child;
            node_prim_count = node.prim_count;
            node.prim_count = 0; // Set to 0 since it's not a leaf.

            node_first_prim = node.first_prim;
        }

        self.nodes[left_child].first_prim = node_first_prim;
        self.nodes[left_child].prim_count = left_count;

        self.nodes[right_child].first_prim = i;
        self.nodes[right_child].prim_count = node_prim_count - left_count;

        self.update_node_bounds(left_child);
        self.update_node_bounds(right_child);

        // Recurse.
        self.subdivide(left_child);
        self.subdivide(right_child);
    }

    /// Update the bounds for a given node.
    fn update_node_bounds(&mut self, node_index: usize) {
        let node = &mut self.nodes[node_index];
        // Visit every vertex of each triangle to find the lowest and highest x, y, and z components,
        // thus yielding an AABB for this node.
        for i in 0..node.prim_count {
            let tri_idx = self.triangle_indices[node.first_prim + i];
            let tri = self.triangles[tri_idx].vertices();
            // Find min components for this tri.
            node.aabb_min = node.aabb_min.min(tri[0]);
            node.aabb_min = node.aabb_min.min(tri[1]);
            node.aabb_min = node.aabb_min.min(tri[2] - 0.001); // BB must have non-zero width in each dimension.

            // Find max components for this tri.
            node.aabb_max = node.aabb_max.max(tri[0]);
            node.aabb_max = node.aabb_max.max(tri[1]);
            node.aabb_max = node.aabb_max.max(tri[2] + 0.001);
        }
    }

    fn intersect_bvh(
        &self,
        node_index: usize,
        r: &Ray,
        t_min: f32,
        t_max: f32,
        rec: &mut HitPayload,
    ) -> bool {
        let node = &self.nodes[node_index];
        if !intersect_aabb(r, t_min, t_max, node.aabb_min, node.aabb_max) {
            return false;
        }
        if node.is_leaf() {
            let mut ret = false;
            let mut temp_rec = HitPayload::new();
            // let mut closest_so_far = t_max;
            // If node is a leaf, intersect each of it's primitives and return the closest hit.
            for i in 0..node.prim_count {
                ret |= if self.triangles[self.triangle_indices[node.first_prim + i]].hit(
                    r,
                    t_min,
                    t_max,
                    &mut temp_rec,
                ) {
                    if temp_rec.hit_distance < rec.hit_distance {
                        // Hit was closest recorded so far.
                        rec.world_position = temp_rec.world_position;
                        rec.world_normal = temp_rec.world_normal;
                        rec.hit_distance = temp_rec.hit_distance;
                        rec.front_face = temp_rec.front_face;
                        rec.albedo = temp_rec.albedo;
                    }

                    true
                } else {
                    false
                };
            }
            return ret;
        } else {
            // Node is an interior node. Recurse on each of it's children.
            let left_hit = self.intersect_bvh(node.left_child, r, t_min, t_max, rec);
            let right_hit = self.intersect_bvh(node.left_child + 1, r, t_min, t_max, rec);

            left_hit || right_hit
        }
    }
}

impl Hittable for Bvh {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, rec: &mut HitPayload) -> bool {
        self.intersect_bvh(self.root_index, r, t_min, t_max, rec)
    }
}
