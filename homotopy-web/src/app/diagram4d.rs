use homotopy_core::cubicalise::cubicalise;
use homotopy_core::mesh::CubeMesh;
use homotopy_core::DiagramN;
use im::HashMap;
use yew::prelude::*;

use homotopy_graphics::subdivide::subdivide4;

pub struct Diagram4D {
    props: Props4D,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props4D {
    pub diagram: DiagramN,
}

#[allow(clippy::pub_enum_variant_names)]
#[derive(Debug)]
pub enum Message4D {}

impl Component for Diagram4D {
    type Message = Message4D;
    type Properties = Props4D;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        //1. cubicalise the diagram and get the control mesh
        let full_mesh = cubicalise(&props.diagram);
        //2. Filter the control mesh based on the required projections.
        let mut cube_mesh = CubeMesh::new();
        let mut vert_map = HashMap::new();
        for e in full_mesh.elements.iter() {
            // Simple filter - leave the squares
            if full_mesh.order_of(e.0) == 3 {
                let flat = full_mesh.flatten(e.0);
                let mut cube = [flat[0]; 8];
                for i in 0..8 {
                    if let Some(v_id) = vert_map.get(&flat[i]) {
                        cube[i] = *v_id;
                    } else {
                        let vert = full_mesh.vertices.get(flat[i]).unwrap();
                        let v_id = cube_mesh.mk_vertex(vert.clone());
                        vert_map.insert(flat[i], v_id);
                        cube[i] = v_id;
                    }
                }
                cube_mesh.mk_cube(cube);
            }
        }
        //2. subdivide the control mesh appropriate number of times (from settings?)
        let _subdivided_mesh = subdivide4(&cube_mesh);
        //3. Turn the subdivided mesh into appropriate representation to render it
        // for 3D case probably the mesh itself is a decent representation.
        Self { props }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>{"todo: 4-dimensional diagram."}</div>
        }
    }
}
