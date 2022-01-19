use homotopy_graphics::{
    gl::{shader::Program, GlCtx, Result},
    program,
};

pub struct Shaders {
    pub geometry_3d: Program,
    pub geometry_4d: Program,
    pub wireframe: Program,
    pub cyllinder_pass: Program,
    pub lighting_pass: Program,
}

impl Shaders {
    pub fn new(ctx: &GlCtx) -> Result<Self> {
        Ok(Self {
            geometry_3d: program!(
                ctx,
                "glsl/vert_3d.glsl",
                "glsl/frag.glsl",
                { position, normal },
                { mvp, albedo, t },
            )?,
            geometry_4d: program!(
                ctx,
                "glsl/vert_4d.glsl",
                "glsl/frag.glsl",
                { position_start, position_end, normal_start, normal_end },
                { mvp, albedo, t },
            )?,
            wireframe: program!(
                ctx,
                "glsl/wireframe_vert.glsl",
                "glsl/wireframe_frag.glsl",
                { position, albedo },
                { mvp },
            )?,
            cyllinder_pass: program!(
                ctx,
                "glsl/deferred_vert.glsl",
                "glsl/cyllinder_pass_frag.glsl",
                { position, uv },
                { in_position, in_albedo },
            )?,
            lighting_pass: program!(
                ctx,
                "glsl/deferred_vert.glsl",
                "glsl/lighting_pass_frag.glsl",
                { position, uv },
                {
                    g_position,
                    g_normal,
                    g_albedo,
                    camera_pos,
                    disable_lighting,
                    debug_normals
                },
            )?,
        })
    }
}
