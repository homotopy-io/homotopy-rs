use crate::*;

use crate::mesh::{Mesh, Element, Vertex};

/// Turn a diagram into its surface cubical representation.
pub fn cubicalise(diagram: &DiagramN) -> Mesh {
    let mut new_mesh = Mesh::new();
    // The code for cubicalisation goes here, in the meantime generate dummy data
    if diagram.dimension() == 3 {
        // an octahedron that makes the last frame of the birth sphere homotopy
        const V_COORDS: [i32; 24] = [0,0,0,-1, 0,0,-1,0, 0,-1,0,0, 0,0,0,1, 0,0,1,0, 0,1,0,0];
        const V_BOUNDS: [u8; 6] = [2,2,2,2,2,2];
        const SQUARES: [usize; 32] = [0,1,2,2, 3,1,2,2, 0,4,2,2, 3,4,2,2, 0,1,5,5, 3,1,5,5, 0,4,5,5, 3,4,5,5];
        let mut v_ids = Vec::new();

        let mut i = 0;
        while i < 6 {
            let v = Vertex::new(
                V_COORDS[i*4 + 3] as f64,
                V_COORDS[i*4 + 2] as f64,
                V_COORDS[i*4 + 1] as f64,
                V_COORDS[i*4] as f64,
                V_BOUNDS[i]
            );
            v_ids.push(new_mesh.push_vertex(v));
            i = i + 1;
        }
        i = 0;
        while i < 8 {
            let mut vtx_ids = Vec::new();
            for id in SQUARES[i*4..(i*4 + 4)].iter() {
                vtx_ids.push(v_ids[*id]);
            }
            let e = Element::from_list(2, vtx_ids);
            new_mesh.push_element(e);
            i = i + 1;
        }
        return new_mesh;
    } if diagram.dimension() == 4 { 
        // Sphere birth homotopy
        const V_COORDS: [i32; 13*4] = [1,0,0,-1,1,0,-1,0,1,-1,0,0,0,0,0,0,1,0,0,1,1,0,1,0,1,1,0,0,2,0,0,-1,2,0,-1,0,2,-1,0,0,2,0,0,1,2,0,1,0,2,1,0,0];
        const V_BOUNDS: [u8; 13] = [3,3,3,3,3,3,3,2,2,2,2,2,2];
        const SQUARES: [usize; 128] = [0,1,2,2,3,3,3,3,4,1,2,2,3,3,3,3,0,5,2,2,3,3,3,3,4,5,2,2,3,3,3,3,0,1,6,6,3,3,3,3,4,1,6,6,3,3,3,3,0,5,6,6,3,3,3,3,4,5,6,6,3,3,3,3,0,7,1,8,2,9,2,9,4,10,1,8,2,9,2,9,0,7,5,11,2,9,2,9,4,10,5,11,2,9,2,9,0,7,1,8,6,12,6,12,4,10,1,8,6,12,6,12,0,7,5,11,6,12,6,12,4,10,5,11,6,12,6,12];
        let mut v_ids = Vec::new();

        let mut i = 0;
        while i < 13 {
            let v = Vertex::new(
                V_COORDS[i*4 + 3] as f64,
                V_COORDS[i*4 + 2] as f64,
                V_COORDS[i*4 + 1] as f64,
                V_COORDS[i*4] as f64,
                V_BOUNDS[i]
            );
            v_ids.push(new_mesh.push_vertex(v));
            i = i + 1;
        }
        i = 0;
        while i < 16 {
            let mut vtx_ids = Vec::new();
            for id in SQUARES[i*8..(i*8 + 8)].iter() {
                vtx_ids.push(v_ids[*id]);
            }
            let e = Element::from_list(3, vtx_ids);
            new_mesh.push_element(e);
            i = i + 1;
        }
        return new_mesh;
    }
    return new_mesh;

}
