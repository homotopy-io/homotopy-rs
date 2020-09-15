use wasm_bindgen::prelude::*;
use homotopy_core::*;
use homotopy_core::graphic2d;
use homotopy_core::layout;
use std::collections::HashMap;
use rand::Rng;

#[wasm_bindgen]
pub struct State {
    signature: HashMap<Generator, Diagram>,
    next_id: usize
}

#[wasm_bindgen]
impl State {
    #[wasm_bindgen]
    pub fn new() -> State {
        State {
            signature: HashMap::new(),
            next_id: 0
        }
    }

    #[wasm_bindgen]
    pub fn create(&mut self) -> JsValue {
        let id = self.next_id;
        self.next_id += 1;
        let generator = Generator { dimension: 0, id };
        self.signature.insert(generator, generator.into());
        JsValue::from_serde(&generator).unwrap()
    }
}

#[wasm_bindgen]
pub fn test(size: usize) -> JsValue {
    let x = Generator {
        id: 0,
        dimension: 0,
    };
    let f = Generator {
        id: 1,
        dimension: 1,
    };
    let m = Generator {
        id: 2,
        dimension: 2,
    };

    let fd = DiagramN::new(f, x, x);
    let ffd = fd.attach(fd.clone(), Boundary::Target, &[]).unwrap();
    let md = DiagramN::new(m, ffd, fd);
    let mut diagram = md.clone();

    let mut rng = rand::thread_rng();

    for i in 0..size {
        let x = rng.gen_range(0, i + 1);
        diagram = diagram.attach(md.clone(), Boundary::Source, &[x]).unwrap();
    }

    let mut solver = layout::Solver::new(diagram.clone()).unwrap();
    solver.solve(10000);
    let layout = solver.finish();

    let generators = graphic2d::Generators::new(&diagram);

    let svg = graphic2d::make_svg(&diagram, &layout, &generators);

    //JsValue::from_serde(&(layout, generators)).unwrap()
    JsValue::from_serde(&svg).unwrap()
}
