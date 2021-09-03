use yew::prelude::*;

use gloo::render::{request_animation_frame, AnimationFrame};

use homotopy_graphics::gl;

pub trait Renderer: Sized + 'static {
    fn init(ctx: &mut gl::GlCtx) -> gl::Result<Self>;

    fn update(&mut self, dt: f32);

    fn render<'a>(&'a self, frame: gl::frame::Frame<'a>);
}

#[derive(Properties, Clone, PartialEq)]
pub struct GlViewportProps {}

pub enum GlViewportMessage {
    Render(f64),
}

pub struct GlViewport<R>
where
    R: Renderer,
{
    props: GlViewportProps,
    link: ComponentLink<Self>,
    ctx: Option<gl::GlCtx>,
    renderer: Option<R>,
    canvas: NodeRef,
    t: f64,

    // If the render task is dropped, we won't get notified about `requestAnimationFrame()`
    // calls, so store a reference to the task here
    render_loop: Option<AnimationFrame>,
}

impl<R> Component for GlViewport<R>
where
    R: Renderer,
{
    type Properties = GlViewportProps;
    type Message = GlViewportMessage;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            props,
            link,
            ctx: None,
            renderer: None,
            canvas: Default::default(),
            t: 0.0,
            render_loop: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            GlViewportMessage::Render(t) => {
                // Calculate time difference
                let dt = (t - self.t) as f32;
                // Update current time
                self.t = t;

                // If we have a renderer:
                if let Some(ref mut renderer) = self.renderer {
                    // 1. Update it with the time difference
                    renderer.update(dt);
                    // 2. Grab the GL context and build a frame
                    let ctx = self.ctx.as_mut().unwrap();
                    let frame = gl::frame::Frame::new(ctx);
                    // 3. Build draw calls (and render as frame is droped)
                    renderer.render(frame);
                    // 4. Schedule the next frame
                    self.schedule_frame();
                }
            }
        }

        false
    }

    fn view(&self) -> Html {
        html! {
            <canvas
                style="width: 100%; height: 100%; display: block"
                ref={self.canvas.clone()}
            />
        }
    }

    fn rendered(&mut self, first_render: bool) {
        let mut ctx = gl::GlCtx::attach(&self.canvas).unwrap();
        // TODO(@doctorn) error handling?
        self.renderer = Some(Renderer::init(&mut ctx).unwrap());
        self.ctx = Some(ctx);

        if first_render {
            self.schedule_frame();
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props != props && {
            self.props = props;
            true
        }
    }
}

impl<R> GlViewport<R>
where
    R: Renderer,
{
    fn schedule_frame(&mut self) {
        let link = self.link.clone();
        self.render_loop = Some(request_animation_frame(move |t| {
            link.send_message(GlViewportMessage::Render(t));
        }));
    }
}
