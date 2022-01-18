use homotopy_core::{DiagramN, Generator};
use homotopy_graphics::{
    clay::clay,
    draw,
    gl::{
        array::VertexArray,
        buffer::ElementKind,
        frame::{Draw, Frame},
        shader::Program,
        texture::Texture,
        GlCtx, Result,
    },
    program, vertex_array,
};
use ultraviolet::{Mat4, Vec2, Vec3};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum ViewDimension {
    Three = 3,
    Four = 4,
}

struct SceneComponent {
    generator: Generator,
    array: VertexArray,
    wireframe_array: VertexArray,
}

pub struct Scene {
    diagram: DiagramN,
    view_dimension: ViewDimension,
    program_3d: Program,
    program_4d: Program,
    axes: VertexArray,
    lighting_screen_quad: VertexArray,
    kernel_screen_quad: VertexArray,
    components: Vec<SceneComponent>,
    kernel_components: Vec<SceneComponent>,
}

impl Scene {
    pub fn build(
        ctx: &GlCtx,
        diagram: &DiagramN,
        view_dimension: ViewDimension,
        subdivision_depth: u8,
        geometry_samples: u8,
    ) -> Result<Self> {
        let program_3d = load_program(ctx, ViewDimension::Three)?;
        let program_4d = load_program(ctx, ViewDimension::Four)?;
        let lighting_pass = load_lighting_pass_program(ctx)?;
        let kernel_pass = load_kernel_pass_program(ctx)?;
        let axes = buffer_axes(ctx, &program_3d)?;
        let lighting_screen_quad = buffer_screen_quad(ctx, &lighting_pass)?;
        let kernel_screen_quad = buffer_screen_quad(ctx, &kernel_pass)?;
        let diagram = diagram.clone();

        let mut scene = Self {
            diagram,
            view_dimension,
            program_3d,
            program_4d,
            axes,
            lighting_screen_quad,
            kernel_screen_quad,
            components: vec![],
            kernel_components: vec![],
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
        self.kernel_components.clear();

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
                        &self.program_3d,
                        &tri_buffers.element_buffer,
                        {
                            position: &tri_buffers.vertex_buffer,
                            normal: &tri_buffers.normal_buffer,
                        }
                    )?,
                    wireframe_array: vertex_array!(
                        ctx,
                        &self.program_3d,
                        &tri_buffers.wireframe_element_buffer,
                        { position: &tri_buffers.vertex_buffer }
                    )?,
                });
            }
        } else {
            let tetra_buffers = mesh.buffer_tetras(ctx)?;
            self.components.push(SceneComponent {
                generator: Generator::new(0, 0),
                array: vertex_array!(
                    ctx,
                    &self.program_4d,
                    &tetra_buffers.element_buffer,
                    {
                        position_start: &tetra_buffers.vertex_start_buffer,
                        position_end: &tetra_buffers.vertex_end_buffer,
                        normal_start: &tetra_buffers.normal_start_buffer,
                        normal_end: &tetra_buffers.normal_end_buffer,
                    }
                )?,
                wireframe_array: vertex_array!(
                    ctx,
                    &self.program_3d,
                    &tetra_buffers.projected_wireframe_element_buffer,
                    { position: &tetra_buffers.wireframe_vertex_buffer }
                )?,
            });
            // TODO(@doctorn) remove
            self.kernel_components.push(SceneComponent {
                generator: Generator::new(1, 0),
                array: vertex_array!(
                    ctx,
                    &self.program_4d,
                    &tetra_buffers.animated_wireframe_element_buffer,
                    {
                        position_start: &tetra_buffers.vertex_start_buffer,
                        position_end: &tetra_buffers.vertex_end_buffer,
                        normal_start: &tetra_buffers.normal_start_buffer,
                        normal_end: &tetra_buffers.normal_end_buffer,
                    }
                )?,
                wireframe_array: vertex_array!(
                    ctx,
                    &self.program_3d,
                    &tetra_buffers.projected_wireframe_element_buffer,
                    { position: &tetra_buffers.wireframe_vertex_buffer }
                )?,
            });
        }

        Ok(())
    }

    pub fn draw<'a, F>(&'a self, frame: &mut Frame<'a>, f: F)
    where
        F: Fn(Generator, &'a VertexArray) -> Draw<'a>,
    {
        for component in &self.components {
            frame.draw(f(component.generator, &component.array));
        }
    }

    pub fn draw_kernels<'a, F>(&'a self, frame: &mut Frame<'a>, f: F)
    where
        F: Fn(Generator, &'a VertexArray) -> Draw<'a>,
    {
        for component in &self.kernel_components {
            frame.draw(f(component.generator, &component.array));
        }
    }

    pub fn draw_wireframe<'a>(&'a self, frame: &mut Frame<'a>, transform: &Mat4) {
        for component in &self.components {
            frame.draw(draw!(
                &component.wireframe_array,
                &[],
                { mvp: *transform, albedo: Vec3::zero(), t: 0. }
            ));
        }
    }

    pub fn kernel_pass<'a>(
        &'a self,
        frame: &mut Frame<'a>,
        textures: &[&'a Texture],
    ) {
        frame.draw(draw! {
            &self.kernel_screen_quad,
            textures,
            {
                in_position: 0,
                in_albedo: 1,
            }
        });
    }

    pub fn lighting_pass<'a>(
        &'a self,
        frame: &mut Frame<'a>,
        textures: &[&'a Texture],
        camera_pos: Vec3,
    ) {
        frame.draw(draw! {
            &self.lighting_screen_quad,
            textures,
            {
                g_position: 0,
                g_normal: 1,
                g_albedo: 2,
                camera_pos: camera_pos,
            }
        });
    }

    pub fn draw_axes<'a>(&'a self, frame: &mut Frame<'a>, transform: &Mat4) {
        frame.draw(draw! {
            &self.axes,
            &[],
            { mvp: *transform, albedo: Vec3::zero(), t: 0. }
        });
    }
}

fn buffer_axes(ctx: &GlCtx, program: &Program) -> Result<VertexArray> {
    let axes_elements = ctx.mk_element_buffer(&[0, 1, 2, 3, 4, 5], ElementKind::Lines)?;
    let axes_verts = ctx.mk_buffer(&[
        Vec3::zero(),
        Vec3::unit_x(),
        Vec3::zero(),
        Vec3::unit_y(),
        Vec3::zero(),
        Vec3::unit_z(),
    ])?;

    vertex_array!(ctx, program, &axes_elements, { position: &axes_verts })
}

fn buffer_screen_quad(ctx: &GlCtx, program: &Program) -> Result<VertexArray> {
    let screen_quad_elements =
        ctx.mk_element_buffer(&[0, 1, 2, 0, 2, 3], ElementKind::Triangles)?;
    let screen_quad_verts = ctx.mk_buffer(&[
        Vec3::new(-1., 1., 0.),
        Vec3::new(-1., -1., 0.),
        Vec3::new(1., -1., 0.),
        Vec3::new(1., 1., 0.),
    ])?;
    let screen_quad_uvs = ctx.mk_buffer(&[
        Vec2::new(0., 1.),
        Vec2::new(0., 0.),
        Vec2::new(1., 0.),
        Vec2::new(1., 1.),
    ])?;

    vertex_array!(
        ctx,
        program,
        &screen_quad_elements,
        {
            position: &screen_quad_verts,
            uv: &screen_quad_uvs,
        }
    )
}

fn load_program(ctx: &GlCtx, dimension: ViewDimension) -> Result<Program> {
    match dimension {
        ViewDimension::Three => program!(
            ctx,
            "glsl/vert_3d.glsl",
            "glsl/frag.glsl",
            { position, normal },
            {
                mvp,
                albedo,
                t,
            },
        ),
        ViewDimension::Four => program!(
            ctx,
            "glsl/vert_4d.glsl",
            "glsl/frag.glsl",
            { position_start, position_end, normal_start, normal_end },
            {
                mvp,
                albedo,
                t,
            },
        ),
    }
}

fn load_lighting_pass_program(ctx: &GlCtx) -> Result<Program> {
    program!(
        ctx,
        "glsl/screen_quad_vert.glsl",
        "glsl/lighting_pass_frag.glsl",
        { position, uv },
        { g_position, g_normal, g_albedo, camera_pos },
    )
}

fn load_kernel_pass_program(ctx: &GlCtx) -> Result<Program> {
    program!(
        ctx,
        "glsl/screen_quad_vert.glsl",
        "glsl/kernel_pass_frag.glsl",
        { position, uv },
        { in_position, in_albedo },
    )
}
