use web_sys::WebGlUniformLocation;

use super::Program;
use crate::graphics::GraphicsCtx;

pub(super) trait Uniformable {
    fn uniform(&self, ctx: &GraphicsCtx, program: Program, loc: WebGlUniformLocation);
}

impl GraphicsCtx {
    fn uniform_location(&self, program: Program, loc: &'static str) -> WebGlUniformLocation {
        if let Some(webgl_loc) = self
            .webgl_ctx
            .get_uniform_location(self.carrier_for(program), loc)
        {
            webgl_loc
        } else {
            panic!(
                "attempted to get location of uniform `{}`, which does not exist",
                loc
            )
        }
    }

    #[inline]
    pub(super) fn uniform<T>(&self, program: Program, loc: &'static str, data: T)
    where
        T: Uniformable,
    {
        let webgl_loc = self.uniform_location(program, loc);
        data.uniform(self, program, webgl_loc);
        // TODO check errors
    }
}
