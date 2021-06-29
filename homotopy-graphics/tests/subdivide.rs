#[cfg(test)]
mod tests {
    use homotopy_core::mesh::{CubeMesh, SquareMesh, Vertex, VertexId};
    use homotopy_graphics::subdivide::{subdivide3, subdivide4};

    fn create_simple_3d_input() -> SquareMesh {
        let mut mesh = SquareMesh::new();
        let verts = [
            [0, -1, 0, 1, 2],
            [0, 0, -1, 1, 2],
            [0, 0, 0, 0, 2],
            [0, 1, 0, 1, 2],
            [0, 0, 1, 1, 2],
            [0, -1, 0, 2, 1],
            [0, 0, -1, 2, 1],
            [0, 1, 0, 2, 1],
            [0, 0, 1, 2, 1],
        ];
        let mut ids = Vec::new();
        for i in &verts {
            let b = match i[4] {
                0 => 0,
                1 => 1,
                2 => 2,
                _ => panic!("Boundaries can be 0-2"),
            };
            let id = mesh.mk_vertex(Vertex::new(
                i[0] as f64,
                i[1] as f64,
                i[2] as f64,
                i[3] as f64,
                b,
            ));
            ids.push(id);
        }
        let squares = [
            [0, 1, 2, 2],
            [3, 1, 2, 2],
            [0, 4, 2, 2],
            [3, 4, 2, 2],
            [0, 5, 1, 6],
            [3, 7, 1, 6],
            [0, 5, 4, 8],
            [3, 7, 4, 8],
        ];
        for i in &squares {
            let mut v_ids = [ids[0]; 4];
            for (j, id) in i.iter().enumerate() {
                v_ids[j] = ids[*id];
            }
            mesh.mk_square(v_ids);
        }
        mesh
    }

    fn create_simple_4d_input() -> CubeMesh {
        let mut mesh = CubeMesh::new();
        let verts = [
            [0, 0, -1, 1, 3],
            [0, -1, 0, 1, 3],
            [-1, 0, 0, 1, 3],
            [0, 0, 0, 0, 3],
            [0, 0, 1, 1, 3],
            [0, 1, 0, 1, 3],
            [1, 0, 0, 1, 3],
            [0, 0, -1, 2, 2],
            [0, -1, 0, 2, 2],
            [-1, 0, 0, 2, 2],
            [0, 0, 1, 2, 2],
            [0, 1, 0, 2, 2],
            [1, 0, 0, 2, 2],
        ];
        let mut ids = Vec::new();
        for i in &verts {
            let b = match i[4] {
                0 => 0,
                1 => 1,
                2 => 2,
                3 => 3,
                _ => panic!("Boundaries can be 0-3"),
            };
            let id = mesh.mk_vertex(Vertex::new(
                i[0] as f64,
                i[1] as f64,
                i[2] as f64,
                i[3] as f64,
                b,
            ));
            ids.push(id);
        }
        let cubes = [
            [0, 1, 2, 2, 3, 3, 3, 3],
            [4, 1, 2, 2, 3, 3, 3, 3],
            [0, 5, 2, 2, 3, 3, 3, 3],
            [4, 5, 2, 2, 3, 3, 3, 3],
            [0, 1, 6, 6, 3, 3, 3, 3],
            [4, 1, 6, 6, 3, 3, 3, 3],
            [0, 5, 6, 6, 3, 3, 3, 3],
            [4, 5, 6, 6, 3, 3, 3, 3],
            [0, 7, 1, 8, 2, 9, 2, 9],
            [4, 10, 1, 8, 2, 9, 2, 9],
            [0, 7, 5, 11, 2, 9, 2, 9],
            [4, 10, 5, 11, 2, 9, 2, 9],
            [0, 7, 1, 8, 6, 12, 6, 12],
            [4, 10, 1, 8, 6, 12, 6, 12],
            [0, 7, 5, 11, 6, 12, 6, 12],
            [4, 10, 5, 11, 6, 12, 6, 12],
        ];
        for i in &cubes {
            let mut v_ids = [ids[0]; 8];
            for (j, id) in i.iter().enumerate() {
                v_ids[j] = ids[*id];
            }
            mesh.mk_cube(v_ids);
        }
        mesh
    }

    fn are_vertices_equal(v: &Vertex, w: &Vertex, e: f64) -> bool {
        (v.x <= w.x + e && v.x + e >= w.x)
            && (v.y <= w.y + e && v.y + e >= w.y)
            && (v.z <= w.z + e && v.z + e >= w.z)
            && (v.t <= w.t + e && v.t + e >= w.t)
    }

    fn within_simple4d_bounding_box(v: &Vertex) -> bool {
        (v.x >= -1.0 && v.x <= 1.0)
            && (v.y >= -1.0 && v.y <= 1.0)
            && (v.z >= -1.0 && v.z <= 1.0)
            && (v.t >= 0.0 && v.t <= 2.0)
    }

    fn within_simple3d_bounding_box(v: &Vertex) -> bool {
        (v.x >= 0.0 && v.x <= 0.0)
            && (v.y >= -1.0 && v.y <= 1.0)
            && (v.z >= -1.0 && v.z <= 1.0)
            && (v.t >= 0.0 && v.t <= 2.0)
    }

    #[test]
    fn subdivide3_test() {
        let mesh = create_simple_3d_input();
        let mut meshes = Vec::new();
        meshes.push(subdivide3(&mesh));
        meshes.push(subdivide3(&meshes[0]));
        meshes.push(subdivide3(&meshes[1]));
        let cubes = vec![8 * 4, 8 * 4 * 4, 8 * 4 * 4 * 4];
        let vertices = vec![33, 129, 513];
        for (i, sub_mesh) in meshes.iter().enumerate() {
            assert_eq!(cubes[i], sub_mesh.squares.len());
            assert_eq!(vertices[i], sub_mesh.vertices.len());
            let v_ids: Vec<VertexId> = sub_mesh.vertices.keys().collect();
            for v in &v_ids {
                assert!(within_simple3d_bounding_box(&sub_mesh.vertices[*v]));
                for w in &v_ids {
                    if v != w {
                        assert!(!are_vertices_equal(
                            &sub_mesh.vertices[*v],
                            &sub_mesh.vertices[*w],
                            0.000001
                        ));
                    }
                }
            }
        }
    }

    #[test]
    fn subdivide4_test() {
        let mesh = create_simple_4d_input();
        let mut meshes = Vec::new();
        meshes.push(subdivide4(&mesh));
        meshes.push(subdivide4(&meshes[0]));
        meshes.push(subdivide4(&meshes[1]));
        let cubes = vec![16 * 8, 16 * 8 * 8, 16 * 8 * 8 * 8];
        let vertices = vec![105, 913, 7713];
        for (i, sub_mesh) in meshes.iter().enumerate() {
            assert_eq!(cubes[i], sub_mesh.cubes.len());
            assert_eq!(vertices[i], sub_mesh.vertices.len());
            let v_ids: Vec<VertexId> = sub_mesh.vertices.keys().collect();
            for v in &v_ids {
                assert!(within_simple4d_bounding_box(&sub_mesh.vertices[*v]));
                for w in &v_ids {
                    if v != w {
                        assert!(!are_vertices_equal(
                            &sub_mesh.vertices[*v],
                            &sub_mesh.vertices[*w],
                            0.000001
                        ));
                    }
                }
            }
        }
    }
}
