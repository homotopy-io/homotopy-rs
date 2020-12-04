
// use wasm_bindgen::JsCast;
// use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, Path2d};

// pub struct PathUtil {
//     context: CanvasRenderingContext2d,
// }

// impl PathUtil {
//     pub fn new() -> Option<Self> {
//         let window = web_sys::window()?;
//         let document = window.document()?;
//         let canvas = document.create_element("canvas").ok()?;
//         let canvas = canvas.dyn_into::<HtmlCanvasElement>().ok()?;
//         let context = canvas.get_context("2d").ok()??;
//         let context = context.dyn_into::<CanvasRenderingContext2d>().ok()?;
//         Some(PathUtil { context })
//     }

//     pub fn is_point_in_fill(&self, point: (f64, f64), path: &Path2d) -> bool {
//         self.context.set_line_width(0.0);
//         self.context.is_point_in_path_with_path_2d_and_f64(path, point.0, point.1)
//     }

//     pub fn is_point_in_stroke(&self, point: (f64, f64), path: &Path2d, width: f64) -> bool {
//         self.context.set_line_width(width);
//         self.context.is_point_in_stroke_with_path_and_x_and_y(path, point.0, point.1)
//     }
// }
