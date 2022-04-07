use crate::*;
use js_sys::Date;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use wasm_bindgen::JsCast;

mod shaders {
    use super::ShaderSource;
    include!(concat!(env!("OUT_DIR"), "/shaders.rs"));
}
mod webgl;
use webgl::{Pipeline, ShaderSource};

pub enum Msg {
    Render(f64),
}

#[derive(Debug)]
pub struct BackGround {
    gl: Option<WebGl2RenderingContext>,
    canvas: NodeRef,
    pipeline: Option<Pipeline>,
    render_loop: Option<gloo::render::AnimationFrame>,
    frame_count: u32,
    init_time: f64,
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub shader_name: String,
    pub reflex_date_time: bool,
}

fn get_shader(shader_name: &str) -> Option<ShaderSource> {
    let shaders = shaders::get_texts();
    shaders
        .get(&shader_name)
        .copied()
        .or_else(|| shaders.get(&"default").copied())
}

impl Component for BackGround {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        /*
        let init_time = match ctx.props().reflex_date_time {
            true => {
                let date = Date::new(&Date::now().into());
                (date.get_minutes() * 60 + date.get_seconds()) as f64
            }
            false => 0.0,
        };
        */
        let date = Date::new(&Date::now().into());
        let init_time = (date.get_minutes() * 60 + date.get_seconds()) as f64;
        Self {
            gl: None,
            canvas: Default::default(),
            pipeline: None,
            render_loop: None,
            frame_count: 0,
            init_time,
        }
    }

    fn view(&self, _: &Context<Self>) -> Html {
        html! { <canvas ref={ self.canvas.clone() } class="background"></canvas> }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        let canvas = self.canvas.cast::<HtmlCanvasElement>().unwrap();
        self.gl = { || canvas.get_context("webgl2").ok()??.dyn_into().ok() }();
        let shader_name = &ctx.props().shader_name;
        if let Some(gl) = &self.gl {
            webgl::init_gl(gl);
            if let Some(shader) = get_shader(shader_name) {
                self.pipeline = Some(webgl::create_pipeline(gl, shader));
            } else {
                gloo::utils::window()
                    .alert_with_message("failed to load shader")
                    .expect_throw("failed to show alert");
            }
        } else {
            let ctx: CanvasRenderingContext2d =
                { || canvas.get_context("2d").ok()??.dyn_into().ok() }()
                    .expect_throw("failed to init rendering context 2d");
            ctx.set_font("30px serif");
            let y = canvas.height() - 30;
            ctx.fill_text("Failed to init WebGL2...", 30.0, y as f64)
                .unwrap_or_else(|e| gloo::console::log!(format!("{e:?}")));
        }
        if first_render {
            self.set_render_loop(ctx);
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let Msg::Render(timestamp) = msg;
        if let (Some(gl), Some(pipeline)) = (&self.gl, &mut self.pipeline) {
            let canvas = self.canvas.cast::<HtmlCanvasElement>().unwrap();
            let shader_name = &ctx.props().shader_name;
            if correct_canvas_size(&canvas, pipeline.pixel_ratio) {
                if let Some(gl) = &self.gl {
                    gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
                }
            }
            // 30 FPS, profile is only allowed 60FPS
            if self.frame_count % 2 == 0 || shader_name == "profile" {
                let resolution = [canvas.width() as f32, canvas.height() as f32];
                webgl::gl_rendering(
                    gl,
                    pipeline,
                    resolution,
                    (self.init_time + timestamp * 0.001) as f32,
                );
            }
            self.frame_count += 1;
        }
        self.set_render_loop(ctx);
        false
    }
}

impl BackGround {
    fn set_render_loop(&mut self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        let handle =
            gloo::render::request_animation_frame(move |time| link.send_message(Msg::Render(time)));
        self.render_loop = Some(handle);
    }
}

fn correct_canvas_size(canvas: &HtmlCanvasElement, pixel_ratio: u32) -> bool {
    let doc = gloo::utils::document_element();
    let resized = canvas.width() != doc.client_width() as u32 / pixel_ratio
        || canvas.height() != doc.client_height() as u32 / pixel_ratio;
    if resized {
        canvas.set_width(doc.client_width() as u32 / pixel_ratio);
        canvas.set_height(doc.client_height() as u32 / pixel_ratio);
    }
    resized
}
