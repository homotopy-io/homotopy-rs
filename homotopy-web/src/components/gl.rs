use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use gloo::render::{request_animation_frame, AnimationFrame};
use homotopy_graphics::gl::{frame::Frame, GlCtx, Result};
use ultraviolet::Vec2;
use yew::prelude::*;

use crate::components::{
    read_touch_list,
    settings::{KeyStore, Settings, Store},
    Finger,
};

pub trait Renderer: Sized + 'static {
    type Settings: Settings;

    fn init(ctx: &mut GlCtx) -> Result<Self>;

    fn update(this: &mut RendererState<Self>, dt: f32);

    fn render<'a>(&'a self, frame: Frame<'a>, settings: &Store<Self::Settings>);

    fn on_mouse_up(_this: &mut RendererState<Self>) {}

    fn on_mouse_down(_this: &mut RendererState<Self>, _point: Vec2) {}

    fn on_mouse_move(_this: &mut RendererState<Self>, _point: Vec2) {}

    fn on_mouse_wheel(_this: &mut RendererState<Self>, _point: Vec2, _delta: f32) {}

    fn on_touch_move(_this: &mut RendererState<Self>, _touches: &[(Finger, Vec2)]) {}

    fn on_touch_update(_this: &mut RendererState<Self>, _touches: &[(Finger, Vec2)]) {}
}

pub struct RendererState<R>
where
    R: Renderer,
{
    ctx: Option<GlCtx>,
    renderer: Option<R>,
    settings: Store<R::Settings>,
}

impl<R> RendererState<R>
where
    R: Renderer,
{
    #[allow(unused)]
    pub fn settings(&self) -> &Store<R::Settings> {
        &self.settings
    }

    #[allow(unused)]
    pub fn ctx(&self) -> &GlCtx {
        self.ctx.as_ref().unwrap()
    }

    fn update(&mut self, dt: f32) {
        R::update(self, dt);
    }

    fn render(&mut self) {
        let frame = Frame::new(self.ctx.as_mut().unwrap());
        R::render(self.renderer.as_ref().unwrap(), frame, &self.settings);
    }
}

impl<R> Deref for RendererState<R>
where
    R: Renderer,
{
    type Target = R;

    fn deref(&self) -> &Self::Target {
        self.renderer.as_ref().unwrap()
    }
}

impl<R> DerefMut for RendererState<R>
where
    R: Renderer,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.renderer.as_mut().unwrap()
    }
}

impl<R> Default for RendererState<R>
where
    R: Renderer,
{
    fn default() -> Self {
        Self {
            ctx: None,
            renderer: None,
            settings: Default::default(),
        }
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct GlViewportProps {}

pub enum GlViewportMessage<R>
where
    R: Renderer,
{
    Render(f64),
    Setting(<<R::Settings as Settings>::Store as KeyStore>::Message),
}

pub struct GlViewport<R>
where
    R: Renderer,
{
    props: GlViewportProps,
    link: ComponentLink<Self>,
    renderer: Rc<RefCell<RendererState<R>>>,
    canvas: NodeRef,
    t: f64,

    // If the render task is dropped, we won't get notified about `requestAnimationFrame()`
    // calls, so store a reference to the task here
    render_loop: Option<AnimationFrame>,
    _settings: R::Settings,
}

impl<R> Component for GlViewport<R>
where
    R: Renderer,
{
    type Message = GlViewportMessage<R>;
    type Properties = GlViewportProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut settings = R::Settings::connect(link.callback(GlViewportMessage::Setting));

        settings.subscribe(R::Settings::ALL);

        Self {
            props,
            link,
            renderer: Default::default(),
            canvas: Default::default(),
            t: 0.0,
            render_loop: None,
            _settings: settings,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            GlViewportMessage::Render(t) => {
                // Calculate time difference
                let dt = (t - self.t) as f32;
                // Update current time
                self.t = t;

                {
                    let mut renderer = self.renderer.borrow_mut();
                    renderer.update(dt);
                    renderer.render();
                }

                // Schedule the next frame
                self.schedule_frame();
            }
            GlViewportMessage::Setting(msg) => self.renderer.borrow_mut().settings.set(&msg),
        }

        false
    }

    fn view(&self) -> Html {
        let on_mouse_move = {
            let renderer = Rc::clone(&self.renderer);
            Callback::from(move |e: MouseEvent| {
                e.prevent_default();
                R::on_mouse_move(
                    &mut *renderer.borrow_mut(),
                    (e.client_x() as f32, e.client_y() as f32).into(),
                );
            })
        };
        let on_mouse_up = {
            let renderer = Rc::clone(&self.renderer);
            Callback::from(move |e: MouseEvent| {
                e.prevent_default();
                R::on_mouse_up(&mut *renderer.borrow_mut());
            })
        };
        let on_mouse_down = {
            let renderer = Rc::clone(&self.renderer);
            Callback::from(move |e: MouseEvent| {
                e.prevent_default();
                R::on_mouse_down(
                    &mut *renderer.borrow_mut(),
                    (e.client_x() as f32, e.client_y() as f32).into(),
                );
            })
        };
        let on_wheel = {
            let renderer = Rc::clone(&self.renderer);
            Callback::from(move |e: WheelEvent| {
                e.prevent_default();
                R::on_mouse_wheel(
                    &mut *renderer.borrow_mut(),
                    (e.client_x() as f32, e.client_y() as f32).into(),
                    e.delta_y() as f32,
                );
            })
        };
        let on_touch_move = {
            let renderer = Rc::clone(&self.renderer);
            let node_ref = self.canvas.clone();
            Callback::from(move |e: TouchEvent| {
                e.prevent_default();
                let touches = read_touch_list(&e.touches(), &node_ref)
                    .map(|(f, p)| (f, Vec2::new(p.x as f32, p.y as f32)))
                    .collect::<Vec<_>>();
                R::on_touch_move(&mut *renderer.borrow_mut(), &touches);
            })
        };
        let on_touch_update = {
            let renderer = Rc::clone(&self.renderer);
            let node_ref = self.canvas.clone();
            Callback::from(move |e: TouchEvent| {
                e.prevent_default();
                let touches = read_touch_list(&e.touches(), &node_ref)
                    .map(|(f, p)| (f, Vec2::new(p.x as f32, p.y as f32)))
                    .collect::<Vec<_>>();
                R::on_touch_update(&mut *renderer.borrow_mut(), &touches);
            })
        };

        html! {
            <canvas
                style="width: 100%; height: 100%; display: block"
                onmousemove={on_mouse_move}
                onmouseup={on_mouse_up}
                onmousedown={on_mouse_down}
                onwheel={on_wheel}
                ontouchmove={on_touch_move}
                ontouchcancel={on_touch_update.clone()}
                ontouchend={on_touch_update.clone()}
                ontouchstart={on_touch_update}
                ref={self.canvas.clone()}
            />
        }
    }

    fn rendered(&mut self, first_render: bool) {
        // TODO(@doctorn) error handling?
        {
            let mut ctx = GlCtx::attach(&self.canvas).unwrap();
            let mut renderer = self.renderer.borrow_mut();
            renderer.renderer = Some(Renderer::init(&mut ctx).unwrap());
            renderer.ctx = Some(ctx);
        }

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
