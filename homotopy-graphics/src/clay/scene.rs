use homotopy_core::{DiagramN, Generator};
use ultraviolet::{Mat4, Vec3};

use crate::{
    clay::layout::MeshExtractor,
    draw,
    gl::{
        array::VertexArray,
        buffer::ElementKind,
        frame::{Draw, Frame},
        shader::Program,
        GlCtx, Result,
    },
    program, vertex_array,
};

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
    solid_program: Program,
    wireframe_program: Program,
    axes: VertexArray,
    components: Vec<SceneComponent>,
}

impl Scene {
    pub fn build(
        ctx: &GlCtx,
        diagram: &DiagramN,
        view_dimension: ViewDimension,
        subdivision_depth: u8,
    ) -> Result<Self> {
        let solid_program = load_solid_program(ctx, view_dimension)?;
        let wireframe_program = load_wireframe_program(ctx)?;
        let axes = buffer_axes(ctx, &wireframe_program)?;
        let diagram = diagram.clone();

        let mut scene = Self {
            diagram,
            view_dimension,
            solid_program,
            wireframe_program,
            axes,
            components: vec![],
        };

        scene.reload_meshes(ctx, subdivision_depth)?;
        Ok(scene)
    }

    pub fn reload_meshes(&mut self, ctx: &GlCtx, subdivision_depth: u8) -> Result<()> {
        self.components.clear();

        let mut extractor = MeshExtractor::new(&self.diagram, self.view_dimension as u8).unwrap();

        if self.view_dimension == ViewDimension::Four {
            extractor = extractor.extract_cubes();
        }

        let mut mesh = extractor
            .extract_squares()
            .extract_lines()
            .extract_points()
            .build();
        mesh.subdivide(subdivision_depth);

        if self.view_dimension == ViewDimension::Three {
            let square_buffers = mesh.buffer_squares(ctx)?;
            self.components.push(SceneComponent {
                generator: Generator::new(0, 0), // TODO(@doctorn) split mesh
                array: vertex_array!(
                    &self.solid_program,
                    &square_buffers.element_buffer,
                    {
                        position: &square_buffers.vertex_buffer,
                        normal: &square_buffers.normal_buffer,
                    }
                )?,
                wireframe_array: vertex_array!(
                    &self.wireframe_program,
                    &square_buffers.wireframe_element_buffer,
                    {
                        position: &square_buffers.vertex_buffer,
                        color: &square_buffers.wireframe_color_buffer,
                    }
                )?,
            });

            let line_buffers = mesh.buffer_lines(ctx)?;
            self.components.push(SceneComponent {
                generator: Generator::new(0, 0), // TODO(@doctorn) split mesh
                array: vertex_array!(
                    &self.solid_program,
                    &line_buffers.element_buffer,
                    { position: &line_buffers.vertex_buffer }
                )?,
                wireframe_array: vertex_array!(
                    &self.wireframe_program,
                    &line_buffers.element_buffer,
                    { position: &line_buffers.vertex_buffer }
                )?,
            });
        } else {
            let cube_buffers = mesh.buffer_cubes(ctx)?;
            self.components.push(SceneComponent {
                generator: Generator::new(0, 0),
                array: vertex_array!(
                    &self.solid_program,
                    &cube_buffers.element_buffer,
                    {
                        position_start: &cube_buffers.vertex_start_buffer,
                        position_end: &cube_buffers.vertex_end_buffer,
                        normal_start: &cube_buffers.normal_start_buffer,
                        normal_end: &cube_buffers.normal_end_buffer,
                    }
                )?,
                wireframe_array: vertex_array!(
                    &self.wireframe_program,
                    &cube_buffers.wireframe_element_buffer,
                    { position: &cube_buffers.wireframe_vertex_buffer }
                )?,
            });
        }

        Ok(())
    }

    pub fn draw<'a, F>(&'a self, frame: &mut Frame<'a>, f: F)
    where
        F: Fn(Generator, &VertexArray) -> Draw,
    {
        for component in &self.components {
            frame.draw(f(component.generator, &component.array));
        }
    }

    pub fn draw_wireframe<'a>(&'a self, frame: &mut Frame<'a>, transform: &Mat4) {
        for component in &self.components {
            frame.draw(draw!(
                &component.wireframe_array,
                { mvp: *transform }
            ));
        }
    }

    pub fn draw_axes<'a>(&'a self, frame: &mut Frame<'a>, transform: &Mat4) {
        frame.draw(draw! {
            &self.axes,
            { mvp: *transform }
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
    let axes_colors = ctx.mk_buffer(&[
        Vec3::unit_x(),
        Vec3::unit_x(),
        Vec3::unit_y(),
        Vec3::unit_y(),
        Vec3::unit_z(),
        Vec3::unit_z(),
    ])?;

    vertex_array!(
        program,
        &axes_elements,
        {
            position: &axes_verts,
            color: &axes_colors,
        }
    )
}

fn load_solid_program(ctx: &GlCtx, dimension: ViewDimension) -> Result<Program> {
    match dimension {
        ViewDimension::Three => program!(
            ctx,
            "../../glsl/vert_3d.glsl",
            "../../glsl/frag.glsl",
            { position, normal },
            { mvp, debug_normals, camera_pos },
        ),
        ViewDimension::Four => program!(
            ctx,
            "../../glsl/vert_4d.glsl",
            "../../glsl/frag.glsl",
            { position_start, position_end, normal_start, normal_end },
            { mvp, debug_normals, camera_pos, t },
        ),
    }
}

fn load_wireframe_program(ctx: &GlCtx) -> Result<Program> {
    program!(
        ctx,
        "../../glsl/wireframe_vert.glsl",
        "../../glsl/wireframe_frag.glsl",
        { position, color },
        { mvp },
    )
}
