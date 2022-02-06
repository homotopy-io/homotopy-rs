use homotopy_core::{DiagramN, Generator};
use homotopy_graphics::{
    clay::clay,
    gl::{array::VertexArray, GlCtx, Result},
    vertex_array,
};

use super::ViewDimension;

pub struct Scene {
    pub diagram: DiagramN,
    pub view_dimension: ViewDimension,
    pub components: Vec<(Generator, VertexArray)>,
    pub wireframe_components: Vec<VertexArray>,
    pub cylinder_components: Vec<(Generator, VertexArray)>,
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
            wireframe_components: vec![],
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
        self.wireframe_components.clear();
        self.cylinder_components.clear();

        let mut mesh = clay(
            &self.diagram,
            self.view_dimension as usize,
            subdivision_depth,
        )
        .unwrap();

        if self.view_dimension == ViewDimension::Three {
            mesh.inflate_3d(geometry_samples);
            for tri_buffers in mesh.buffer_tris(ctx)? {
                self.components.push((
                    tri_buffers.generator,
                    vertex_array!(
                        ctx,
                        &tri_buffers.element_buffer,
                        [&tri_buffers.vertex_buffer, &tri_buffers.normal_buffer]
                    )?,
                ));

                self.wireframe_components.push(vertex_array!(
                    ctx,
                    &tri_buffers.wireframe_element_buffer,
                    [&tri_buffers.vertex_buffer]
                )?);
            }
        } else {
            for tetra_buffers in mesh.buffer_tetras(ctx)? {
                self.components.push((
                    tetra_buffers.generator,
                    vertex_array!(
                        ctx,
                        &tetra_buffers.element_buffer,
                        [
                            &tetra_buffers.vert_start_buffer,
                            &tetra_buffers.vert_end_buffer,
                            &tetra_buffers.normal_start_buffer,
                            &tetra_buffers.normal_end_buffer,
                        ]
                    )?,
                ));
            }

            for projected_buffers in mesh.buffer_projected_wireframe(ctx)? {
                self.wireframe_components.push(vertex_array!(
                    ctx,
                    &projected_buffers.element_buffer,
                    [&projected_buffers.vert_buffer]
                )?);
            }

            for cylinder_buffers in mesh.buffer_cylinder_wireframe(ctx)? {
                self.cylinder_components.push((
                    cylinder_buffers.generator,
                    vertex_array!(
                        ctx,
                        &cylinder_buffers.element_buffer,
                        [
                            &cylinder_buffers.vert_start_buffer,
                            &cylinder_buffers.vert_end_buffer
                        ]
                    )?,
                ));
            }
        }

        Ok(())
    }
}
