use crate::*;
use gloo::events::EventListener;
use js_sys::Date;
use std::sync::{
    atomic::{AtomicI32, AtomicU32, Ordering},
    Arc,
};
use wasm_bindgen::JsCast;

mod shaders {
    use super::ShaderSource;
    include!(concat!(env!("OUT_DIR"), "/shaders.rs"));
}
mod webgl;
use webgl::{Pipeline, ShaderSource};

#[derive(Debug)]
pub struct BackGround {
    gl: Option<WebGl2RenderingContext>,
    canvas: NodeRef,
    pipeline: Option<Pipeline>,
    render_loop: Option<gloo::render::AnimationFrame>,
    frame_count: u32,
    init_time: f64,
    mouse_listener: MouseListner,
}

pub enum Msg {
    Render(f64),
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub shader_name: String,
}

#[derive(Debug)]
pub struct MouseListner {
    mouse_position: Arc<[AtomicI32; 2]>,
    _handler: EventListener,
}

impl MouseListner {
    fn set() -> MouseListner {
        let mouse_position: Arc<[AtomicI32; 2]> = Default::default();
        let cloned_mp = Arc::clone(&mouse_position);
        let closure = move |e: &Event| {
            let e = MouseEvent::from(wasm_bindgen::JsValue::from(e.clone()));
            cloned_mp[0].store(e.client_x(), Ordering::SeqCst);
            cloned_mp[1].store(e.client_y(), Ordering::SeqCst);
        };
        let _handler = if let Ok(Some(parent)) = gloo::utils::window().parent() {
            EventListener::new(&parent, "mousemove", closure)
        } else {
            EventListener::new(&gloo::utils::window(), "mousemove", closure)
        };
        MouseListner {
            mouse_position,
            _handler,
        }
    }
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
        let date = Date::new(&Date::now().into());
        let init_time = (date.get_minutes() * 60 + date.get_seconds()) as f64;
        Self {
            gl: None,
            canvas: Default::default(),
            pipeline: None,
            render_loop: None,
            frame_count: 0,
            init_time,
            mouse_listener: MouseListner::set(),
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
            if correct_canvas_size(&canvas, pipeline.pixel_ratio) {
                if let Some(gl) = &self.gl {
                    gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
                }
            }
            if self.frame_count % 2 == 0 {
                let resolution = [canvas.width() as f32, canvas.height() as f32];
                let mouse_position = [
                    self.mouse_listener.mouse_position[0].load(Ordering::SeqCst) as f32,
                    self.mouse_listener.mouse_position[1].load(Ordering::SeqCst) as f32,
                ];
                webgl::gl_rendering(
                    gl,
                    pipeline,
                    resolution,
                    (self.init_time + timestamp * 0.001) as f32,
                    mouse_position,
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
