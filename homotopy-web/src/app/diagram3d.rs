use yew::prelude::*;

use homotopy_core::DiagramN;

// use homotopy_graphics::cubicalise::cubicalise;
// use homotopy_graphics::mesh::SquareMesh;
// use homotopy_graphics::subdivide::subdivide3;

pub struct Diagram3D {
    props: Props3D,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props3D {
    pub diagram: DiagramN,
}

#[derive(Debug)]
pub enum Message3D {}

impl Component for Diagram3D {
    type Message = Message3D;
    type Properties = Props3D;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        // 1. cubicalise the diagram and get the control mesh
        // let full_mesh = cubicalise(&props.diagram);
        // 2. Filter the control mesh based on the required projections.
        // let mut square_mesh = SquareMesh::new();
        // let mut vert_map = HashMap::new();
        // for e in full_mesh.elements.iter() {
        //     // Simple filter - leave the squares
        //     if full_mesh.order_of(e.0) == 2 {
        //         let flat = full_mesh.flatten(e.0);
        //         let mut sq = [flat[0]; 4];
        //         for i in 0..4 {
        //             if let Some(v_id) = vert_map.get(&flat[i]) {
        //                 sq[i] = *v_id;
        //             } else {
        //                 let vert = full_mesh.vertices.get(flat[i]).unwrap();
        //                 let v_id = square_mesh.mk_vertex(vert.clone());
        //                 vert_map.insert(flat[i], v_id);
        //                 sq[i] = v_id;
        //             }
        //         }
        //         square_mesh.mk_square(sq);
        //     }
        // }
        // 3. subdivide the control mesh appropriate number of times (from settings?)
        // let _subdivided_mesh = subdivide3(&square_mesh);
        // 4. Turn the subdivided mesh into appropriate representation to render it
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
            <div>{"todo: 3-dimensional diagram"}</div>
        }
    }
}
