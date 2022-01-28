use homotopy_core::{DiagramN, Generator};
use homotopy_graphics::{
    clay::clay,
    gl::{array::VertexArray, GlCtx, Result},
    vertex_array,
};

use super::ViewDimension;

pub struct SceneComponent {
    pub generator: Generator,
    pub array: VertexArray,
    pub wireframe_array: VertexArray,
}

pub struct Scene {
    pub diagram: DiagramN,
    pub view_dimension: ViewDimension,
    pub components: Vec<SceneComponent>,
    pub cylinder_components: Vec<SceneComponent>,
}

impl Scene {
    pub fn new(
        ctx: &GlCtx,
        diagram: &DiagramN,
        view_dimension: ViewDimension,
        subdivision_depth: u8,
        geometry_samples: u8,
    ) -> Result<Self> {
        let diagram = diagram.clone();

        let mut scene = Self {
            diagram,
            view_dimension,
            components: vec![],
            cylinder_components: vec![],
        };

        scene.reload_meshes(ctx, subdivision_depth, geometry_samples)?;
        Ok(scene)
    }

    pub fn reload_meshes(
        &mut self,
        ctx: &GlCtx,
        subdivision_depth: u8,
        geometry_samples: u8,
    ) -> Result<()> {
        self.components.clear();
        self.cylinder_components.clear();

        let mesh = clay(
            &self.diagram,
            self.view_dimension as usize,
            subdivision_depth,
        )
        .unwrap();

        if self.view_dimension == ViewDimension::Three {
            for tri_buffers in mesh.buffer_tris(ctx, geometry_samples)? {
                self.components.push(SceneComponent {
                    generator: tri_buffers.generator,
                    array: vertex_array!(
                        ctx,
                        &tri_buffers.element_buffer,
                        [&tri_buffers.vertex_buffer, &tri_buffers.normal_buffer]
                    )?,
                    wireframe_array: vertex_array!(
                        ctx,
                        &tri_buffers.wireframe_element_buffer,
                        [&tri_buffers.vertex_buffer]
                    )?,
                });
            }
        } else {
            let tetra_buffers = mesh.buffer_tetras(ctx)?;
            self.components.push(SceneComponent {
                generator: Generator::new(0, 0),
                array: vertex_array!(
                    ctx,
                    &tetra_buffers.element_buffer,
                    [
                        &tetra_buffers.vertex_start_buffer,
                        &tetra_buffers.vertex_end_buffer,
                        &tetra_buffers.normal_start_buffer,
                        &tetra_buffers.normal_end_buffer,
                    ]
                )?,
                wireframe_array: vertex_array!(
                    ctx,
                    &tetra_buffers.projected_wireframe_element_buffer,
                    [&tetra_buffers.wireframe_vertex_buffer]
                )?,
            });
            self.cylinder_components.push(SceneComponent {
                generator: Generator::new(1, 0),
                array: vertex_array!(
                    ctx,
                    &tetra_buffers.animated_wireframe_element_buffer,
                    [
                        &tetra_buffers.vertex_start_buffer,
                        &tetra_buffers.vertex_end_buffer,
                        &tetra_buffers.normal_start_buffer,
                        &tetra_buffers.normal_end_buffer,
                    ]
                )?,
                wireframe_array: vertex_array!(
                    ctx,
                    &tetra_buffers.projected_wireframe_element_buffer,
                    [&tetra_buffers.wireframe_vertex_buffer]
                )?,
            });
        }

        Ok(())
    }
}
