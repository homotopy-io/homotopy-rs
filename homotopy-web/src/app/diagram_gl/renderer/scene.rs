use std::mem;

use homotopy_common::idx::IdxVec;
use homotopy_core::{Diagram, Generator};
use homotopy_graphics::{
    geom::{CubicalGeometry, SimplicialGeometry, VertData},
    gl::{array::VertexArray, GlCtx, Result},
    style::SignatureStyleData,
    vertex_array,
};
use ultraviolet::Vec4;

use crate::model::proof::{generators::VertexShape, View};

pub struct Scene {
    pub diagram: Diagram,
    pub view: View,
    pub components: Vec<(Generator, VertexArray)>,
    pub wireframe_components: Vec<VertexArray>,
    pub cylinder_components: Vec<(Generator, VertexArray)>,
    pub animation_curves: Vec<AnimationCurve>,
    pub animation_singularities: Vec<(Generator, Vec4)>,
    pub sphere: Option<VertexArray>,
    pub cube: Option<VertexArray>,
    pub duration: f32,
}

pub struct AnimationCurve {
    pub generator: Generator,
    pub begin: f32,
    pub end: f32,
    pub key_frames: Vec<Vec4>,
}

impl AnimationCurve {
    pub fn at(&self, t: f32) -> Option<Vec4> {
        if t < self.begin || t > self.end {
            return None;
        }

        let (start, end) = self.search(t)?;
        let lerp = (t - start.w) / (end.w - start.w);

        Some(start + (end - start) * lerp)
    }

    fn search(&self, t: f32) -> Option<(Vec4, Vec4)> {
        // TODO(@doctorn) a proper search algorithm
        self.key_frames
            .iter()
            .copied()
            .zip(self.key_frames.iter().copied().skip(1))
            .find(|(u, v)| u.w <= t && v.w >= t)
    }
}

impl Scene {
    pub fn new(
        ctx: &GlCtx,
        diagram: &Diagram,
        view: View,
        smooth_time: bool,
        subdivision_depth: u8,
        geometry_samples: u8,
        signature_styles: &impl SignatureStyleData,
    ) -> Result<Self> {
        let diagram = diagram.clone();

        let mut scene = Self {
            diagram,
            view,
            components: vec![],
            wireframe_components: vec![],
            cylinder_components: vec![],
            animation_curves: vec![],
            animation_singularities: vec![],
            sphere: None,
            cube: None,
            duration: 0.,
        };

        scene.reload_meshes(
            ctx,
            smooth_time,
            subdivision_depth,
            geometry_samples,
            signature_styles,
        )?;
        Ok(scene)
    }

    pub fn reload_meshes(
        &mut self,
        ctx: &GlCtx,
        smooth_time: bool,
        subdivision_depth: u8,
        geometry_samples: u8,
        signature_styles: &impl SignatureStyleData,
    ) -> Result<()> {
        self.components.clear();
        self.wireframe_components.clear();
        self.cylinder_components.clear();
        self.animation_curves.clear();
        self.animation_singularities.clear();
        self.sphere = None;
        self.cube = None;

        let mut sphere_mesh: SimplicialGeometry = Default::default();
        let p = sphere_mesh.mk_vert(VertData {
            position: Vec4::zero(),
            boundary: [false; 4],
            generator: Generator::new(0, 0),
        });
        sphere_mesh.mk_point(p);
        sphere_mesh.inflate_point_3d(p, geometry_samples, &VertexShape::Circle.into());
        if let Some(sphere_buffers) = sphere_mesh.buffer_tris(ctx)?.into_iter().next() {
            self.sphere = Some(vertex_array!(
                ctx,
                &sphere_buffers.element_buffer,
                [&sphere_buffers.vertex_buffer, &sphere_buffers.normal_buffer]
            )?);
        }

        let mut cube_mesh: SimplicialGeometry = Default::default();
        let p = cube_mesh.mk_vert(VertData {
            position: Vec4::zero(),
            boundary: [false; 4],
            generator: Generator::new(0, 0),
        });
        cube_mesh.mk_point(p);
        cube_mesh.inflate_point_3d(p, geometry_samples, &VertexShape::Square.into());
        if let Some(cube_buffers) = cube_mesh.buffer_tris(ctx)?.into_iter().next() {
            self.cube = Some(vertex_array!(
                ctx,
                &cube_buffers.element_buffer,
                [&cube_buffers.vertex_buffer, &cube_buffers.normal_buffer]
            )?);
        }

        let mut cubical = match self.view.dimension() {
            0 => CubicalGeometry::new::<0>(&self.diagram).unwrap(),
            1 => CubicalGeometry::new::<1>(&self.diagram).unwrap(),
            2 => CubicalGeometry::new::<2>(&self.diagram).unwrap(),
            3 => CubicalGeometry::new::<3>(&self.diagram).unwrap(),
            4 => CubicalGeometry::new::<4>(&self.diagram).unwrap(),
            _ => unreachable!(),
        };

        cubical.subdivide(smooth_time, subdivision_depth);

        let mut simplicial = SimplicialGeometry::from(cubical);

        if self.view.dimension() <= 3 {
            simplicial.inflate_3d(geometry_samples, signature_styles);
            for tri_buffers in simplicial.buffer_tris(ctx)? {
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
            for tetra_buffers in simplicial.buffer_tetras(ctx)? {
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

            for projected_buffers in simplicial.buffer_projected_wireframe(ctx)? {
                self.wireframe_components.push(vertex_array!(
                    ctx,
                    &projected_buffers.element_buffer,
                    [&projected_buffers.vert_buffer]
                )?);
            }

            for cylinder_buffers in simplicial.buffer_cylinder_wireframe(ctx)? {
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

            let mut curves = IdxVec::new();
            mem::swap(&mut simplicial.curves, &mut curves);

            for mut curve in curves.into_values() {
                if curve.verts.len() < 2 {
                    continue;
                }

                let generator = curve.generator;

                curve.verts.sort_by(|i, j| simplicial.time_order(*i, *j));

                self.animation_curves.push(AnimationCurve {
                    generator,
                    begin: simplicial.verts[curve.verts[0]].position.w,
                    end: simplicial.verts[curve.verts[curve.verts.len() - 1]]
                        .position
                        .w,
                    key_frames: curve
                        .verts
                        .into_iter()
                        .map(|v| simplicial.verts[v].position)
                        .collect(),
                });
            }

            for point in simplicial.points.into_values() {
                let VertData {
                    generator,
                    position,
                    ..
                } = simplicial.verts[point];
                self.animation_singularities.push((generator, position));
            }
        }

        Ok(())
    }
}
