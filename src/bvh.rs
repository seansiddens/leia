use crate::hittable::*;
use crate::triangle::*;
use glam::*;

pub struct Bvh {
    root_index: usize,
    nodes: Vec<BvhNode>,
    nodes_used: usize,
    triangles: Vec<Triangle>,
    triangle_indices: Vec<usize>,
    num_triangles: usize,
}

#[derive(Clone, Copy)]
struct BvhNode {
    aabb_min: Vec3,
    aabb_max: Vec3,
    left_child: usize, // Right child is always left_child + 1
    first_prim: usize,
    prim_count: usize, // If non-zero, is a leaf. Must zero for interior nodes.
}

impl BvhNode {
    fn is_leaf(&self) -> bool {
        self.prim_count > 0
    }
}

impl Bvh {
    pub fn new(triangles: Vec<Triangle>) -> Self {
        let num_triangles = triangles.len();
        // Populate the triangle index vector.
        let mut triangle_indices = Vec::with_capacity(num_triangles);
        for i in 0..num_triangles {
            triangle_indices.push(i);
        }

        // Initialize the BvhNode pool.
        // Upper limit for a BVH with N triangles is 2N - 1
        let mut nodes = vec![
            BvhNode {
                aabb_min: vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY),
                aabb_max: vec3(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
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
            triangles,
            triangle_indices,
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
        for i in node.first_prim..node.prim_count {
            let tri_idx = self.triangle_indices[node.first_prim + i];
            let tri = self.triangles[tri_idx].vertices();
            // Find min components for this tri.
            node.aabb_min = node.aabb_min.min(tri[0]);
            node.aabb_min = node.aabb_min.min(tri[1]);
            node.aabb_min = node.aabb_min.min(tri[2]);

            // Find max components for this tri.
            node.aabb_max = node.aabb_max.max(tri[0]);
            node.aabb_max = node.aabb_max.max(tri[1]);
            node.aabb_max = node.aabb_max.max(tri[2]);
        }
    }

    fn intersect_aabb(r: &Ray, b_min)
}

impl Hittable for Bvh {
    pub fn hit(&self, r: &crate::ray::Ray, t_min: f32, t_max: f32, rec: &mut HitRecord) -> bool {
        
    }
}
