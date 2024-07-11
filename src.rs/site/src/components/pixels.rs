//! broken
use gloo_timers::future::TimeoutFuture;
use pixels::{Pixels, SurfaceTexture};
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};
use yew::prelude::*;

pub struct PixelsComponent {
    pixels: Option<Pixels>,
}

pub enum Msg {
    Render,
}

impl Component for PixelsComponent {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self { pixels: None }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <canvas id="pixels-canvas"></canvas>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let canvas = document
                .get_element_by_id("pixels-canvas")
                .unwrap()
                .dyn_into::<HtmlCanvasElement>()
                .unwrap();

            let width = 320;
            let height = 240;
            canvas.set_width(width);
            canvas.set_height(height);

            let surface_texture = SurfaceTexture::new(width, height, &canvas);

            // Get WebGL2 context
            let gl_context = canvas
                .get_context("webgl2")
                .unwrap()
                .unwrap()
                .dyn_into::<WebGl2RenderingContext>()
                .unwrap();
            let pixels = Pixels::new(width, height, gl_context).unwrap();
            self.pixels = Some(pixels);

            // Set up rendering loop
            let link = ctx.link().clone();
            wasm_bindgen_futures::spawn_local(async move {
                loop {
                    link.send_message(Msg::Render);
                    TimeoutFuture::new(16).await; // ~60 FPS
                }
            });
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Render => {
                if let Some(pixels) = &mut self.pixels {
                    // Update your pixel buffer here
                    let frame = pixels.frame_mut();
                    for pixel in frame.chunks_exact_mut(4) {
                        pixel[0] = 0x5e; // R
                        pixel[1] = 0x48; // G
                        pixel[2] = 0xe8; // B
                        pixel[3] = 0xff; // A
                    }
                    pixels.render().unwrap();
                }
                false // No need to re-render the component
            }
        }
    }
}
